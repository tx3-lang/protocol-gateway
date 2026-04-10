use axum::Json;
use axum::extract::{Path, State};
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

pub async fn dispatch(
    State(state): State<Arc<AppState>>,
    Path(protocol_name): Path<String>,
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

    if request.method == "rpc.discover" {
        let doc = discover::generate_openrpc(&state, &protocol_name);
        let response = json!({
            "jsonrpc": "2.0",
            "id": id.unwrap_or(Value::Null),
            "result": doc,
        });
        return (StatusCode::OK, Json(response));
    }

    let tx_name = &request.method;

    if tx_name.is_empty() {
        let err = RpcError::method_not_found(tx_name);
        return (StatusCode::OK, Json(err.to_json_rpc(id)));
    }

    let args = match request.params.unwrap_or(Value::Object(Default::default())) {
        Value::Object(map) => map,
        _ => {
            let err = RpcError::invalid_params("params must be an object");
            return (StatusCode::OK, Json(err.to_json_rpc(id)));
        }
    };

    match handler::invoke_tx(&state, &protocol_name, tx_name, args).await {
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

