use serde_json::{Map, Value, json};
use std::sync::Arc;
use tx3_sdk::trp::Client;

use crate::config::trp_options_for_network;
use crate::registry::TiiRegistry;
use crate::rpc::error::RpcError;

pub struct AppState {
    pub registry: TiiRegistry,
    pub trp_override: Option<String>,
    pub trp_headers: Option<std::collections::HashMap<String, String>>,
    pub network: String,
}

pub async fn invoke_tx(
    state: &Arc<AppState>,
    protocol_name: &str,
    tx_name: &str,
    args: Map<String, Value>,
) -> Result<Value, RpcError> {
    let protocol = state
        .registry
        .get(protocol_name)
        .ok_or_else(|| RpcError::protocol_not_found(protocol_name))?;

    let network = &state.network;

    let invocation = protocol
        .invoke(tx_name, Some(network))
        .map_err(|e| match e {
            tx3_sdk::tii::Error::UnknownTx(_) => RpcError::tx_not_found(tx_name),
            tx3_sdk::tii::Error::UnknownProfile(_) => RpcError::network_not_found(network),
            other => RpcError::internal(other.to_string()),
        })?;

    let mut invocation = invocation;
    invocation.set_args(args);

    let missing: Vec<_> = invocation
        .unspecified_params()
        .map(|(name, _)| name.clone())
        .collect();

    if !missing.is_empty() {
        return Err(RpcError::args_mismatch(format!(
            "missing required params: {}",
            missing.join(", ")
        )));
    }

    let resolve_params = invocation
        .into_resolve_request()
        .map_err(|e| RpcError::args_mismatch(e.to_string()))?;

    let trp_options = trp_options_for_network(network, &state.trp_override, &state.trp_headers)
        .ok_or_else(|| {
            RpcError::build_error(format!(
                "no TRP endpoint configured for network '{network}'"
            ))
        })?;

    let trp_client = Client::new(trp_options);

    let envelope = trp_client
        .resolve(resolve_params)
        .await
        .map_err(|e| RpcError::build_error(e.to_string()))?;

    let mut result = json!({ "tx": envelope.tx });
    if !envelope.hash.is_empty() {
        result["hash"] = json!(envelope.hash);
    }

    Ok(result)
}
