// FILE-CONTEXT Header (Format v3)
// ZWECK: Declarative tool manifest specification for agent tool registration and capability verification.
// INVARIANTEN: Non-empty name, no null bytes, timeout > 0.
// STAND: TS:2026-10-29T00:00:00Z

//! Declarative tool manifest specification.

use crate::context::MAX_ID_LEN;
use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

/// Maximum text length for manifest description.
pub const MAX_MANIFEST_DESC_LEN: usize = 4096;

fn default_timeout_ms() -> u64 {
    30_000
}

fn default_retriable() -> bool {
    true
}

fn default_max_retries() -> u32 {
    2
}

/// Declarative manifest specifying operational properties, capability requirements, and retry policies for a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolManifest {
    /// Unique identifier of the tool.
    pub name: String,
    /// Human-readable description of the tool's purpose.
    pub description: String,
    /// Required capability identifier checked before execution (fail-closed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_capability: Option<String>,
    /// Execution timeout in milliseconds (default: 30,000 ms).
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Whether execution errors are retriable.
    #[serde(default = "default_retriable")]
    pub is_retriable: bool,
    /// Maximum retry attempts if retriable.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

impl ToolManifest {
    /// Creates a new `ToolManifest` with the given name and description.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required_capability: None,
            timeout_ms: default_timeout_ms(),
            is_retriable: default_retriable(),
            max_retries: default_max_retries(),
        }
    }

    /// Sets the required capability for execution.
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capability = Some(capability.into());
        self
    }

    /// Sets the timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Sets retry behavior.
    pub fn with_retry(mut self, is_retriable: bool, max_retries: u32) -> Self {
        self.is_retriable = is_retriable;
        self.max_retries = max_retries;
        self
    }

    /// Validates the tool manifest fields against boundary rules.
    pub fn validate(&self) -> Result<()> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Tool manifest name cannot be empty".to_string(),
            ));
        }
        if name.len() > MAX_ID_LEN {
            return Err(ContextraError::InvalidInput(format!(
                "Tool manifest name length {} exceeds maximum allowed length of {}",
                name.len(),
                MAX_ID_LEN
            )));
        }
        if name.contains('\0') {
            return Err(ContextraError::InvalidInput(
                "Tool manifest name cannot contain null bytes".to_string(),
            ));
        }

        if self.description.len() > MAX_MANIFEST_DESC_LEN {
            return Err(ContextraError::InvalidInput(format!(
                "Tool description length {} exceeds limit of {}",
                self.description.len(),
                MAX_MANIFEST_DESC_LEN
            )));
        }
        if self.description.contains('\0') {
            return Err(ContextraError::InvalidInput(
                "Tool description cannot contain null bytes".to_string(),
            ));
        }

        if let Some(ref cap) = self.required_capability {
            if cap.trim().is_empty() {
                return Err(ContextraError::InvalidInput(
                    "Required capability cannot be empty when specified".to_string(),
                ));
            }
            if cap.contains('\0') {
                return Err(ContextraError::InvalidInput(
                    "Required capability cannot contain null bytes".to_string(),
                ));
            }
        }

        if self.timeout_ms == 0 {
            return Err(ContextraError::InvalidInput(
                "Tool timeout_ms must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }
}
