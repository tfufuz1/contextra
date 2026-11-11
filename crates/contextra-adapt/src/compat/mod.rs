//! Kompatibilitätsbrücke für `contextra-types` und `contextra-ports`.
//!
//! Nur aktiv hinter dem optionalen Feature `workspace`.

#[cfg(feature = "ports-compat")]
pub mod clock;
#[cfg(feature = "types-compat")]
pub mod error;

#[cfg(feature = "ports-compat")]
pub use clock::PortsClockAdapter;
