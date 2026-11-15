// FILE-CONTEXT
// ZWECK: HNSW Einfüge-Pipeline (Compute & Apply) und Konnektivitätswächter.
// INVARIANTEN: Lock-Hierarchie: write_mutex -> entry_point -> nodes; Zero-Panic.
// HOTSPOTS: compute_insert_with_context, apply_insert, check_connectivity
// STAND: 2026-11-12

//! HNSW Einfüge-Pipeline und Graph-Konnektivitätsprüfungen.
//!
//! Dieses Modul gliedert das Einfügen und Validieren von Graph-Kanten:
//! - [`compute`]: Vorberechnung von Schichten, Nachbarn und Backlinks ohne direkte Graphmutation.
//! - [`apply`]: Atomare Übernahme vorbereiteter Knoten (`PreparedInsert`) in Arena und Lookup-Tabellen.
//! - [`connectivity`]: Integritäts- und Konnektivitätswächter (`check_connectivity`, `connectivity_score`).

pub(crate) mod apply;
pub(crate) mod compute;
pub(crate) mod connectivity;
