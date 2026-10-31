// FILE-CONTEXT
// ZWECK: Homeostat Wrapper in contextra-db für Rerank PID Candidate Pool Regulation und Deadline Management.
// INVARIANTEN: Re-exportiert `RerankPidController` und `RerankDeadline` aus `contextra-adapt`.

//! Homeostat-Modul für Rerank-Kandidatenpool-Regulierung und Deadline-Steuerung.
//!
//! # Architektur & Ring-Zugehörigkeit
//! Dieses Modul gehört zur Ring-3-Fassade (`contextra-db`) und stellt Re-Exports
//! aus dem Adaptionssubsystem (`contextra-adapt`, Ring 2) bereit.
//!
//! # Zweck für Entwickler
//! Bei mehrstufigen Suchvorgängen (Multi-Step Retrieval) und nachgelagerten
//! Re-Ranking-Schritten muss die Latenz dynamisch geregelt werden:
//! - [`RerankDeadline`]: Verwaltet Zeitbudgets für Re-Ranking-Operationen, um
//!   SLA-Ziele (Service Level Agreements) einzuhalten.
//! - [`RerankPidController`]: Ein PID-Regler (Proportional-Integral-Derivative), der
//!   die Größe des Kandidatenpools anhand historischer Latenzen anpasst.

// ANCHOR[contextra-db:HOMEOSTAT-001]
// NOTE: Re-Export von RerankDeadline für Latenzbudget-Überwachung im Re-Ranking-Schritt.
pub use contextra_adapt::homeostat::RerankDeadline;
// ANCHOR[contextra-db:HOMEOSTAT-002]
// NOTE: Re-Export des PID-Reglers zur dynamischen Kandidatenpool-Dimensionierung.
#[allow(deprecated)]
pub use contextra_adapt::homeostat::RerankPidController;
