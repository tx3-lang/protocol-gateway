use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::{Value, json};
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

fn common_errors() -> Vec<Value> {
    vec![
        json!({ "code": -32000, "message": "Protocol not found" }),
        json!({ "code": -32001, "message": "Transaction not found" }),
        json!({ "code": -32002, "message": "Arguments mismatch" }),
        json!({ "code": -32003, "message": "Build error" }),
        json!({ "code": -32004, "message": "Network not found" }),
    ]
}

pub fn generate_openrpc(state: &AppState) -> Value {
    let network = &state.network;
    let mut methods: Vec<Value> = Vec::new();

    for (protocol_name, protocol) in state.registry.iter() {
        for tx_name in protocol.txs().keys() {
            let method_name = format!("{protocol_name}.{tx_name}");

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
                        desc.push_str(&format!("\nProfile pre-fills: {}", prefilled.join(", ")));
                    }

                    if !required.is_empty() {
                        desc.push_str(&format!("\nCaller must supply: {}", required.join(", ")));
                    }

                    (descriptors, desc)
                }
                Err(_) => {
                    let desc = format!(
                        "Build an unsigned Cardano transaction for {tx_name} \
                             in the {protocol_name} protocol."
                    );
                    (vec![], desc)
                }
            };

            methods.push(json!({
                "name": method_name,
                "summary": format!("Build {tx_name} transaction ({protocol_name})"),
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
        "summary": "Returns the OpenRPC specification for this API",
        "params": [],
        "result": {
            "name": "OpenRPC Document",
            "schema": { "type": "object" }
        }
    }));

    json!({
        "openrpc": "1.4.1",
        "info": {
            "title": "Tx3 Protocol API",
            "version": "0.1.0",
            "description": format!(
                "JSON-RPC 2.0 API dynamically generated from loaded Tx3 protocols. \
                 Active network: {network}."
            )
        },
        "methods": methods
    })
}

pub async fn openrpc_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let doc = generate_openrpc(&state);
    (StatusCode::OK, Json(doc))
}

pub async fn docs_redirect() -> impl IntoResponse {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let url =
        format!("https://playground.open-rpc.org/?schemaUrl=http://localhost:{port}/openrpc.json");

    axum::response::Redirect::temporary(&url)
}
