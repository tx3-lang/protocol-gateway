use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::rpc::discover;
use crate::rpc::error::RpcError;
use crate::rpc::handler::{self, AppState};

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

pub async fn dispatch(State(state): State<Arc<AppState>>, body: String) -> impl IntoResponse {
    let request: JsonRpcRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(_) => {
            let err = RpcError::parse_error("failed to parse JSON");
            return (StatusCode::OK, Json(err.to_json_rpc(None)));
        }
    };

    let id = request.id.clone();

    if request.jsonrpc != "2.0" {
        let err = RpcError::invalid_request("jsonrpc must be \"2.0\"");
        return (StatusCode::OK, Json(err.to_json_rpc(id)));
    }

    if request.method == "rpc.discover" {
        let doc = discover::generate_openrpc(&state);
        let response = json!({
            "jsonrpc": "2.0",
            "id": id.unwrap_or(Value::Null),
            "result": doc,
        });
        return (StatusCode::OK, Json(response));
    }

    let (protocol_name, tx_name) = match parse_method(&request.method) {
        Some(parts) => parts,
        None => {
            let err = RpcError::method_not_found(&request.method);
            return (StatusCode::OK, Json(err.to_json_rpc(id)));
        }
    };

    let args = match request.params.unwrap_or(Value::Object(Default::default())) {
        Value::Object(map) => map,
        _ => {
            let err = RpcError::invalid_params("params must be an object");
            return (StatusCode::OK, Json(err.to_json_rpc(id)));
        }
    };

    match handler::invoke_tx(&state, &protocol_name, &tx_name, args).await {
        Ok(result) => {
            let response = json!({
                "jsonrpc": "2.0",
                "id": id.unwrap_or(Value::Null),
                "result": result,
            });
            (StatusCode::OK, Json(response))
        }
        Err(err) => (StatusCode::OK, Json(err.to_json_rpc(id))),
    }
}

/// Parses a method string in the format "protocol.tx_name".
fn parse_method(method: &str) -> Option<(String, String)> {
    let dot_pos = method.find('.')?;
    let protocol = &method[..dot_pos];
    let tx = &method[dot_pos + 1..];

    if protocol.is_empty() || tx.is_empty() || tx.contains('.') {
        return None;
    }

    Some((protocol.to_string(), tx.to_string()))
}

#[cfg(test)]
mod tests {
    use super::parse_method;

    #[test]
    fn valid_method() {
        assert_eq!(
            parse_method("ticketing-2026.buy_ticket"),
            Some(("ticketing-2026".into(), "buy_ticket".into()))
        );
    }

    #[test]
    fn no_dot() {
        assert_eq!(parse_method("buy_ticket"), None);
    }

    #[test]
    fn empty_protocol() {
        assert_eq!(parse_method(".buy_ticket"), None);
    }

    #[test]
    fn empty_tx() {
        assert_eq!(parse_method("ticketing-2026."), None);
    }

    #[test]
    fn multiple_dots() {
        assert_eq!(parse_method("a.b.c"), None);
    }
}
