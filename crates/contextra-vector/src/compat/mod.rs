//! Kompatibilitätsbrücke für bestehende Contextra-Workspace-Kerne (SPEC-v2 §4.3).

#[cfg(any(feature = "workspace", feature = "ports-compat"))]
pub mod error;
#[cfg(any(feature = "workspace", feature = "ports-compat"))]
pub mod types;
