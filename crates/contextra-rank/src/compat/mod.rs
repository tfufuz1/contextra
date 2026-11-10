//! # Kompatibilitätsbrücke (`compat`)
//!
//! **Zweck:** Enthält alle Konvertierungen und Schnittstellenanbindungen an die
//! Workspace-Crates `contextra_types` und `contextra_ports`.
//!
//! Dieses Modul wird ausschließlich über das Feature `workspace` aktiviert.
//! Der Kern von `contextra-rank` ist vollständig unabhängig von diesen Crates.

pub mod clock;
pub mod error;
pub mod types;

pub use clock::{IsotonicPortsClockExt, PortsClockAdapter};
pub use types::{
    apply_temporal_validity_filter_with_ports_clock,
    signal_contribution_synthesized_from_types,
    weights_to_signal_factors_types_compat,
};
