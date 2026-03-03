use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::rpc::error::RpcError;
use crate::rpc::handler::{self, AppState};

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

pub async fn dispatch(
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
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

    if request.method != "apply_tx" {
        let err = RpcError::method_not_found(&request.method);
        return (StatusCode::OK, Json(err.to_json_rpc(id)));
    }

    let params = request.params.unwrap_or(Value::Object(Default::default()));

    match handler::apply_tx(&state, params).await {
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
