//! Prompt-Injection-Erkennung (Stub — contextra-prompt-guard wurde entfernt).
//!
//! Die externen Typen werden als leere Stubs beibehalten, um die öffentliche API
//! von `contextra-mcp` nicht zu brechen. Funktionalität ist deaktiviert.

/// Platzhalter für den entfernten Redaction-Marker.
pub const DEFAULT_REDACTION_PLACEHOLDER: &str = "[REDACTED]";

/// Stub-Fehlertyp für Prompt-Injection.
#[derive(Debug, thiserror::Error)]
pub enum GuardError {
    #[error("Prompt injection guard disabled")]
    Disabled,
}

/// Stub-Clock.
pub trait Clock: Send + Sync {
    fn now_iso8601(&self) -> String;
    fn now_unix_nanos(&self) -> u64 {
        0
    }
}

#[derive(Debug, Clone, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_iso8601(&self) -> String {
        "1970-01-01T00:00:00Z".to_string()
    }
}

/// Stub-Konfiguration für Prompt-Injection-Erkennung.
#[derive(Debug, Clone, Default)]
pub struct PromptInjectionConfig {
    pub enabled: bool,
}

/// Stub-Quarantäne-Policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuarantinePolicy {
    #[default]
    Strict,
    FlagOnly,
    Escalate,
}

/// Stub-Guard (No-Op).
#[derive(Debug, Clone, Default)]
pub struct PromptInjectionGuard;

impl PromptInjectionGuard {
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> Self {
        Self
    }

    pub fn detect(&self, _text: &str) -> Option<String> {
        None
    }

    pub fn process_result(
        &self,
        _doc_id: &str,
        _collection: &str,
        _obj: &mut serde_json::Map<String, serde_json::Value>,
    ) -> bool {
        false
    }
}

/// Stub-Audit-Logger (No-Op).
#[derive(Debug, Clone, Default)]
pub struct SecurityAuditLogger;

/// Stub-Audit-Record.
#[derive(Debug, Clone, Default)]
pub struct SecurityAuditRecord;

pub mod audit {
    pub use super::{SecurityAuditLogger, SecurityAuditRecord};
}

pub mod clock {
    pub use super::{Clock, SystemClock};
}

pub mod error {
    pub use super::GuardError;
}

pub mod guard {
    /// Stub für normalisierte Muster.
    #[derive(Debug, Clone)]
    pub struct NormalizedPattern;
    pub use super::PromptInjectionGuard;
}

pub mod policy {
    pub use super::{PromptInjectionConfig, QuarantinePolicy, DEFAULT_REDACTION_PLACEHOLDER};
}

pub use audit::*;
pub use clock::*;
pub use error::*;
pub use guard::*;
pub use policy::*;
