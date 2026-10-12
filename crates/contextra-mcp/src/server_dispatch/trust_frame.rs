use super::error_codes::{McpErrorCode, StructuredMcpError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    UntrustedExternal = 0,
    ExternalSystem = 1,
    Agent = 2,
    User = 3,
    SystemAdmin = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    pub source: String,
    pub timestamp_nanos: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustFrame<T> {
    pub content: T,
    pub trust: TrustLevel,
    pub source: String,
    pub lineage: LineageInfo,
    pub _data_marker: String,
}

impl<T> TrustFrame<T> {
    pub fn new(content: T, trust: TrustLevel, source: impl Into<String>) -> Self {
        Self {
            content,
            trust,
            source: source.into(),
            lineage: LineageInfo {
                source: "contextra_mcp".into(),
                timestamp_nanos: 0,
            },
            _data_marker: "untrusted_data_boundary".into(),
        }
    }

    pub fn validate_write_trust(
        caller_level: TrustLevel,
        requested_level: TrustLevel,
    ) -> Result<(), StructuredMcpError> {
        if requested_level > caller_level {
            Err(StructuredMcpError::new(
                McpErrorCode::Denied,
                format!(
                    "Trust level escalation denied: caller level {:?} cannot grant requested level {:?}",
                    caller_level, requested_level
                ),
            ))
        } else {
            Ok(())
        }
    }
}
