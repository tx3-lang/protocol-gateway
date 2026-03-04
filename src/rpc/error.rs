use serde_json::{json, Value};

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;
pub const PROTOCOL_NOT_FOUND: i32 = -32000;
pub const TX_NOT_FOUND: i32 = -32001;
pub const ARGS_MISMATCH: i32 = -32002;
pub const BUILD_ERROR: i32 = -32003;
pub const NETWORK_NOT_FOUND: i32 = -32004;

#[derive(Debug)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl RpcError {
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self { code: PARSE_ERROR, message: msg.into(), data: None }
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self { code: INVALID_REQUEST, message: msg.into(), data: None }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: METHOD_NOT_FOUND,
            message: format!("method '{method}' not found; expected format 'protocol.tx'"),
            data: None,
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self { code: INVALID_PARAMS, message: msg.into(), data: None }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self { code: INTERNAL_ERROR, message: msg.into(), data: None }
    }

    pub fn protocol_not_found(name: &str) -> Self {
        Self {
            code: PROTOCOL_NOT_FOUND,
            message: format!("protocol '{name}' not found"),
            data: None,
        }
    }

    pub fn tx_not_found(name: &str) -> Self {
        Self {
            code: TX_NOT_FOUND,
            message: format!("transaction '{name}' not found"),
            data: None,
        }
    }

    pub fn args_mismatch(msg: impl Into<String>) -> Self {
        Self { code: ARGS_MISMATCH, message: msg.into(), data: None }
    }

    pub fn build_error(msg: impl Into<String>) -> Self {
        Self { code: BUILD_ERROR, message: msg.into(), data: None }
    }

    pub fn network_not_found(name: &str) -> Self {
        Self {
            code: NETWORK_NOT_FOUND,
            message: format!("network '{name}' not found"),
            data: None,
        }
    }

    pub fn to_json_rpc(&self, id: Option<Value>) -> Value {
        let mut error = json!({
            "code": self.code,
            "message": self.message,
        });

        if let Some(data) = &self.data {
            error["data"] = data.clone();
        }

        json!({
            "jsonrpc": "2.0",
            "id": id.unwrap_or(Value::Null),
            "error": error,
        })
    }
}
