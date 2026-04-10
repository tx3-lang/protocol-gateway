use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::sync::Arc;
use tx3_sdk::tii::ParamType;

use crate::rpc::handler::AppState;

fn param_type_to_schema(pt: &ParamType) -> Value {
    match pt {
        ParamType::Bytes => json!({
            "type": "string",
            "description": "Hex-encoded bytes"
        }),
        ParamType::Integer => json!({
            "type": "integer"
        }),
        ParamType::Boolean => json!({
            "type": "boolean"
        }),
        ParamType::Address => json!({
            "type": "string",
            "description": "Cardano address (bech32)"
        }),
        ParamType::UtxoRef => json!({
            "type": "string",
            "description": "UTxO reference (txhash#index)"
        }),
        ParamType::List(inner) => json!({
            "type": "array",
            "items": param_type_to_schema(inner)
        }),
        ParamType::Custom(schema) => {
            serde_json::to_value(schema).unwrap_or(json!({ "type": "object" }))
        }
    }
}

fn param_type_label(pt: &ParamType) -> &'static str {
    match pt {
        ParamType::Bytes => "Bytes",
        ParamType::Integer => "Integer",
        ParamType::Boolean => "Boolean",
        ParamType::Address => "Address",
        ParamType::UtxoRef => "UtxoRef",
        ParamType::List(_) => "List",
        ParamType::Custom(_) => "Custom",
    }
}

fn tx_result_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "tx": {
                "type": "string",
                "description": "Hex-encoded unsigned CBOR transaction"
            },
            "hash": {
                "type": "string",
                "description": "Transaction hash (if provided by TRP)"
            }
        },
        "required": ["tx"]
    })
}

/// Convert a raw JSON schema `$ref` or `type` into an OpenRPC-friendly schema object.
fn raw_schema_to_openrpc(prop: &Value) -> (Value, &'static str) {
    if let Some(reference) = prop.get("$ref").and_then(|r| r.as_str()) {
        return match reference {
            "https://tx3.land/specs/v1beta0/core#Bytes" => (
                json!({ "type": "string", "description": "Hex-encoded bytes" }),
                "Bytes",
            ),
            "https://tx3.land/specs/v1beta0/core#Address" => (
                json!({ "type": "string", "description": "Cardano address (bech32)" }),
                "Address",
            ),
            "https://tx3.land/specs/v1beta0/core#UtxoRef" => (
                json!({ "type": "string", "description": "UTxO reference (txhash#index)" }),
                "UtxoRef",
            ),
            _ => (json!({ "type": "string" }), "Unknown"),
        };
    }

    match prop.get("type").and_then(|t| t.as_str()) {
        Some("integer") => (json!({ "type": "integer" }), "Integer"),
        Some("boolean") => (json!({ "type": "boolean" }), "Boolean"),
        Some("object") => (json!({ "type": "object" }), "Object"),
        Some("array") => {
            let items = prop
                .get("items")
                .map(|i| raw_schema_to_openrpc(i).0)
                .unwrap_or(json!({ "type": "object" }));
            (json!({ "type": "array", "items": items }), "List")
        }
        _ => (json!({ "type": "object" }), "Unknown"),
    }
}

/// Fallback: extract params directly from the serialized Protocol JSON when
/// `invoke()` fails (e.g. because the SDK cannot parse a param type like `object`).
fn params_from_raw_protocol(
    protocol: &tx3_sdk::tii::Protocol,
    tx_name: &str,
    network: &str,
) -> Option<(Vec<Value>, String)> {
    let proto_json = serde_json::to_value(protocol).ok()?;
    let spec = proto_json.get("spec")?;

    // Collect environment property names
    let env_keys: HashSet<String> = spec
        .get("environment")
        .and_then(|e| e.get("properties"))
        .and_then(|p| p.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    // Collect party names (lowercased, as the SDK does)
    let party_keys: HashSet<String> = spec
        .get("parties")
        .and_then(|p| p.as_object())
        .map(|obj| obj.keys().map(|k| k.to_lowercase()).collect())
        .unwrap_or_default();

    // Collect pre-filled keys from profile
    let profile_env_keys: HashSet<String> = spec
        .get("profiles")
        .and_then(|p| p.get(network))
        .and_then(|p| p.get("environment"))
        .and_then(|e| e.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    let profile_party_keys: HashSet<String> = spec
        .get("profiles")
        .and_then(|p| p.get(network))
        .and_then(|p| p.get("parties"))
        .and_then(|e| e.as_object())
        .map(|obj| obj.keys().map(|k| k.to_lowercase()).collect())
        .unwrap_or_default();

    // Get tx-level params
    let tx = spec.get("transactions")?.get(tx_name)?;
    let tx_props = tx
        .get("params")
        .and_then(|p| p.get("properties"))
        .and_then(|p| p.as_object());

    // Get environment-level params schema
    let env_props = spec
        .get("environment")
        .and_then(|e| e.get("properties"))
        .and_then(|p| p.as_object());

    let mut descriptors: Vec<Value> = Vec::new();
    let mut prefilled: Vec<String> = Vec::new();
    let mut required: Vec<String> = Vec::new();

    // Add party params
    for party in &party_keys {
        let is_prefilled = profile_party_keys.contains(party);
        let desc = if is_prefilled {
            prefilled.push(party.clone());
            format!("Address (pre-filled by {network} profile)")
        } else {
            required.push(party.clone());
            "Address".to_string()
        };
        descriptors.push(json!({
            "name": party,
            "description": desc,
            "required": !is_prefilled,
            "schema": { "type": "string", "description": "Cardano address (bech32)" }
        }));
    }

    // Add environment params
    if let Some(props) = env_props {
        for (name, schema) in props {
            let is_prefilled = profile_env_keys.contains(name);
            let (openrpc_schema, label) = raw_schema_to_openrpc(schema);
            let desc = if is_prefilled {
                prefilled.push(name.clone());
                format!("{label} (pre-filled by {network} profile)")
            } else {
                required.push(name.clone());
                label.to_string()
            };
            descriptors.push(json!({
                "name": name,
                "description": desc,
                "required": !is_prefilled,
                "schema": openrpc_schema
            }));
        }
    }

    // Add tx-level params (never pre-filled by profile)
    if let Some(props) = tx_props {
        for (name, schema) in props {
            // Skip if already added as env or party param
            if env_keys.contains(name) || party_keys.contains(name) {
                continue;
            }
            let (openrpc_schema, label) = raw_schema_to_openrpc(schema);
            required.push(name.clone());
            descriptors.push(json!({
                "name": name,
                "description": label,
                "required": true,
                "schema": openrpc_schema
            }));
        }
    }

    // Sort: required first
    descriptors.sort_by(|a, b| {
        let a_req = a["required"].as_bool().unwrap_or(false);
        let b_req = b["required"].as_bool().unwrap_or(false);
        b_req.cmp(&a_req)
    });

    prefilled.sort();
    required.sort();

    let mut desc = format!(
        "Build an unsigned Cardano transaction for {tx_name} \
         in the {protocol_name} protocol.\n\nActive network: {network}",
        protocol_name = spec
            .get("protocol")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown"),
    );

    if !prefilled.is_empty() {
        desc.push_str(&format!(
            "\nProfile pre-fills: {}",
            prefilled.join(", ")
        ));
    }
    if !required.is_empty() {
        desc.push_str(&format!(
            "\nCaller must supply: {}",
            required.join(", ")
        ));
    }

    Some((descriptors, desc))
}

fn common_errors() -> Vec<Value> {
    vec![
        json!({ "code": -32001, "message": "Transaction not found" }),
        json!({ "code": -32002, "message": "Arguments mismatch" }),
        json!({ "code": -32003, "message": "Build error" }),
        json!({ "code": -32004, "message": "Network not found" }),
    ]
}

pub fn generate_openrpc(state: &AppState, protocol_name: &str) -> Value {
    let network = &state.network;
    let mut methods: Vec<Value> = Vec::new();

    if let Some(protocol) = state.registry.get(protocol_name) {
        for tx_name in protocol.txs().keys() {
            let (params_descriptors, description) = match protocol.invoke(tx_name, Some(network)) {
                Ok(mut invocation) => {
                    let all_params = invocation.params().clone();
                    let unspecified: Vec<String> = invocation
                        .unspecified_params()
                        .map(|(k, _)| k.clone())
                        .collect();

                    let mut prefilled: Vec<String> = Vec::new();
                    let mut required: Vec<String> = Vec::new();

                    let mut descriptors: Vec<Value> = Vec::new();

                    for (name, pt) in &all_params {
                        let is_required = unspecified.contains(name);
                        let label = param_type_label(pt);

                        let desc = if is_required {
                            required.push(name.clone());
                            label.to_string()
                        } else {
                            prefilled.push(name.clone());
                            format!("{label} (pre-filled by {network} profile)")
                        };

                        descriptors.push(json!({
                            "name": name,
                            "description": desc,
                            "required": is_required,
                            "schema": param_type_to_schema(pt)
                        }));
                    }

                    // Sort so required params come first
                    descriptors.sort_by(|a, b| {
                        let a_req = a["required"].as_bool().unwrap_or(false);
                        let b_req = b["required"].as_bool().unwrap_or(false);
                        b_req.cmp(&a_req)
                    });

                    prefilled.sort();
                    required.sort();

                    let mut desc = format!(
                        "Build an unsigned Cardano transaction for {tx_name} \
                             in the {protocol_name} protocol.\n\nActive network: {network}"
                    );

                    if !prefilled.is_empty() {
                        desc.push_str(&format!(
                            "\nProfile pre-fills: {}",
                            prefilled.join(", ")
                        ));
                    }

                    if !required.is_empty() {
                        desc.push_str(&format!(
                            "\nCaller must supply: {}",
                            required.join(", ")
                        ));
                    }

                    (descriptors, desc)
                }
                Err(_) => {
                    // Fallback: parse params from raw protocol JSON when the SDK
                    // cannot handle a param type (e.g. "type": "object" enums).
                    if let Some(fallback) =
                        params_from_raw_protocol(protocol, tx_name, network)
                    {
                        fallback
                    } else {
                        let desc = format!(
                            "Build an unsigned Cardano transaction for {tx_name} \
                                 in the {protocol_name} protocol."
                        );
                        (vec![], desc)
                    }
                }
            };

            methods.push(json!({
                "name": tx_name,
                "summary": format!("Build {tx_name} transaction"),
                "description": description,
                "paramStructure": "by-name",
                "params": params_descriptors,
                "result": {
                    "name": "TxEnvelope",
                    "schema": tx_result_schema()
                },
                "errors": common_errors()
            }));
        }
    }

    methods.push(json!({
        "name": "rpc.discover",
        "summary": "Returns the OpenRPC specification for this protocol",
        "params": [],
        "result": {
            "name": "OpenRPC Document",
            "schema": { "type": "object" }
        }
    }));

    json!({
        "openrpc": "1.4.1",
        "info": {
            "title": format!("{protocol_name} — Tx3 Protocol API"),
            "version": "0.1.0",
            "description": format!(
                "JSON-RPC 2.0 API for the {protocol_name} protocol. \
                 Active network: {network}."
            )
        },
        "methods": methods
    })
}

pub async fn list_protocols(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let protocols: Vec<&String> = state.registry.names().collect();
    (StatusCode::OK, Json(json!({ "protocols": protocols })))
}

pub async fn openrpc_handler(
    State(state): State<Arc<AppState>>,
    Path(protocol_name): Path<String>,
) -> impl IntoResponse {
    let doc = generate_openrpc(&state, &protocol_name);
    (StatusCode::OK, Json(doc))
}

pub async fn docs_redirect(Path(protocol_name): Path<String>) -> impl IntoResponse {
    let base_url = std::env::var("PUBLIC_URL").unwrap_or_else(|_| {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
        format!("http://localhost:{port}")
    });

    let url = format!(
        "https://playground.open-rpc.org/?schemaUrl={base_url}/{protocol_name}/openrpc.json"
    );

    axum::response::Redirect::temporary(&url)
}
