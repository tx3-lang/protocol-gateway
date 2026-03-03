use serde_json::{json, Map, Value};
use std::sync::Arc;
use tx3_sdk::trp::Client;

use crate::config::trp_options_for_network;
use crate::registry::TiiRegistry;
use crate::rpc::error::RpcError;

pub struct AppState {
    pub registry: TiiRegistry,
    pub trp_override: Option<String>,
}

pub async fn apply_tx(state: &Arc<AppState>, params: Value) -> Result<Value, RpcError> {
    let obj = params.as_object().ok_or_else(|| {
        RpcError::invalid_params("params must be an object")
    })?;

    let protocol_name = obj
        .get("protocol")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing or invalid 'protocol' field"))?;

    let tx_name = obj
        .get("tx")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError::invalid_params("missing or invalid 'tx' field"))?;

    let network = obj
        .get("network")
        .and_then(|v| v.as_str())
        .unwrap_or("mainnet");

    let caller_args = obj
        .get("args")
        .and_then(|v| v.as_object())
        .ok_or_else(|| RpcError::invalid_params("missing or invalid 'args' field"))?;

    let protocol = state
        .registry
        .get(protocol_name)
        .ok_or_else(|| RpcError::protocol_not_found(protocol_name))?;

    let invocation = protocol
        .invoke(tx_name, Some(network))
        .map_err(|e| match e {
            tx3_sdk::tii::Error::UnknownTx(_) => RpcError::tx_not_found(tx_name),
            tx3_sdk::tii::Error::UnknownProfile(_) => RpcError::network_not_found(network),
            other => RpcError::internal(other.to_string()),
        })?;

    let args: Map<String, Value> = caller_args.clone();

    let resolve_params = invocation
        .with_args(args)
        .into_resolve_request()
        .map_err(|e| RpcError::args_mismatch(e.to_string()))?;

    let trp_options = trp_options_for_network(network, &state.trp_override)
        .ok_or_else(|| RpcError::build_error(format!("no TRP endpoint configured for network '{network}'")))?;

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
