use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpErrorCode {
    // Retryable errors (§21.8)
    SerializationFailure,
    Busy,
    StorageFull,

    // Final errors (§21.8)
    Denied,
    Corrupt,
    ConfirmationRequired,
}

impl McpErrorCode {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            McpErrorCode::SerializationFailure | McpErrorCode::Busy | McpErrorCode::StorageFull
        )
    }

    pub fn to_rpc_code(&self) -> i32 {
        match self {
            McpErrorCode::SerializationFailure => -32001,
            McpErrorCode::Busy => -32002,
            McpErrorCode::StorageFull => -32003,
            McpErrorCode::Denied => -32004,
            McpErrorCode::Corrupt => -32005,
            McpErrorCode::ConfirmationRequired => -32006,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredMcpError {
    pub code: McpErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl StructuredMcpError {
    pub fn new(code: McpErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for StructuredMcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for StructuredMcpError {}
