#![forbid(unsafe_code)]
#![deny(dead_code)]
//! # contextra-prompt-guard
//!
//! Signatur- und phrasenbasierte Prompt-Injection-Erkennung und Quarantäne-Policy.
//! Vollständig autarker E0-Baustein ohne `contextra-*`-Abhängigkeiten.

pub mod audit;
pub mod clock;
pub mod error;
pub mod guard;
pub mod policy;

#[cfg(test)]
mod tests;

pub use audit::{SecurityAuditLogger, SecurityAuditRecord};
pub use clock::{Clock, SystemClock};
pub use error::GuardError;
pub use guard::{NormalizedPattern, PromptInjectionGuard};
pub use policy::{PromptInjectionConfig, QuarantinePolicy, DEFAULT_REDACTION_PLACEHOLDER};
