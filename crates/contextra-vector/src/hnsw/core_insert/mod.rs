
//! HNSW Einfüge-Pipeline und Graph-Konnektivitätsprüfungen.
//!
//! Dieses Modul gliedert das Einfügen und Validieren von Graph-Kanten:
//! - [`compute`]: Vorberechnung von Schichten, Nachbarn und Backlinks ohne direkte Graphmutation.
//! - [`apply`]: Atomare Übernahme vorbereiteter Knoten (`PreparedInsert`) in Arena und Lookup-Tabellen.
//! - [`connectivity`]: Integritäts- und Konnektivitätswächter (`check_connectivity`, `connectivity_score`).

pub(crate) mod apply;
pub(crate) mod compute;
pub(crate) mod connectivity;
