//! MCP JSON-RPC 2.0 Protokoll-Typen (Model Context Protocol Spec v2024-11-05)
//!
//! Begrenzte und gehärtete Protokollstrukturen für JSON-RPC 2.0.

use serde_json::Value;
use thiserror::Error;

/// Maximale Rekursionstiefe für eingehende JSON-RPC Payloads (Schutz gegen Stack-Overflow).
pub const MAX_JSON_DEPTH: usize = 32;

/// Maximale Anzahl von Elementen in einem JSON-RPC Batch-Request (Schutz gegen Batch-DoS).
pub const MAX_BATCH_SIZE: usize = 100;

/// MCP Error representation matching standard JSON-RPC 2.0 error codes.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum McpError {
    #[error("{0}")]
    ParseError(String),
    #[error("{0}")]
    InvalidRequest(String),
    #[error("{0}")]
    MethodNotFound(String),
    #[error("{0}")]
    InvalidParams(String),
    #[error("{0}")]
    InternalError(String),
}

impl McpError {
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    pub fn method_not_found(msg: impl Into<String>) -> Self {
        Self::MethodNotFound(msg.into())
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }

    pub fn code(&self) -> i32 {
        match self {
            Self::ParseError(_) => -32700,
            Self::InvalidRequest(_) => -32600,
            Self::MethodNotFound(_) => -32601,
            Self::InvalidParams(_) => -32602,
            Self::InternalError(_) => -32603,
        }
    }
}

impl From<contextra_types::ContextraError> for McpError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg)
            | contextra_types::ContextraError::NotFound(msg) => Self::invalid_params(msg),
            other => Self::internal_error(other.to_string()),
        }
    }
}

impl From<String> for McpError {
    fn from(msg: String) -> Self {
        Self::invalid_params(msg)
    }
}

impl From<&str> for McpError {
    fn from(msg: &str) -> Self {
        Self::invalid_params(msg.to_string())
    }
}

pub use contextra_wire::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

/// Ermittelt die Schachtelungstiefe eines JSON-Value.
pub fn measure_json_depth(val: &Value) -> usize {
    fn depth_recursive(val: &Value, current_depth: usize) -> usize {
        match val {
            Value::Array(arr) => {
                let mut max_d = current_depth;
                for item in arr {
                    max_d = max_d.max(depth_recursive(item, current_depth.saturating_add(1)));
                }
                max_d
            }
            Value::Object(map) => {
                let mut max_d = current_depth;
                for item in map.values() {
                    max_d = max_d.max(depth_recursive(item, current_depth.saturating_add(1)));
                }
                max_d
            }
            _ => current_depth,
        }
    }
    depth_recursive(val, 1)
}

/// Validiert, dass ein JSON-Value die maximale Tiefe nicht überschreitet.
pub fn validate_json_depth(val: &Value, max_depth: usize) -> Result<(), McpError> {
    if measure_json_depth(val) > max_depth {
        Err(McpError::invalid_request(format!(
            "JSON depth exceeds maximum allowed limit of {max_depth}"
        )))
    } else {
        Ok(())
    }
}

/// Helper function to convert an `McpError` directly into a `JsonRpcResponse`.
pub fn response_from_error(id: Option<Value>, err: McpError) -> JsonRpcResponse {
    JsonRpcResponse::err(id, err.code(), err.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_measure_json_depth() {
        let v1 = json!(1);
        assert_eq!(measure_json_depth(&v1), 1);

        let v2 = json!({"a": 1});
        assert_eq!(measure_json_depth(&v2), 2);

        let v3 = json!({"a": [1, 2, {"b": 3}]});
        assert_eq!(measure_json_depth(&v3), 4);
    }

    #[test]
    fn test_validate_json_depth() {
        let val = json!({"a": {"b": {"c": 1}}});
        assert!(validate_json_depth(&val, 4).is_ok());
        assert!(validate_json_depth(&val, 3).is_err());
    }
}
