//! # Minimalverträge und Umgebungsabstraktionen für `contextra-version`
//!
//! LINK crates/contextra-version/src/lib.rs
//! LINK crates/contextra-env/src/lib.rs
//!
//! Dieses Modul definiert die minimalen Umgebungs-Traits (`Clock`, `MetricsSink`),
//! von denen der autarke Kern abhängt. Ports gehören dem Konsumenten;
//! der Kern importiert keine höheren Schichten.

pub use contextra_env::{Clock, MetricsSink, SystemClock};
