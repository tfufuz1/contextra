// FILE-CONTEXT
// STAND: 2026-10-11T15:02:08Z (SESSION: 1b322f26)
// ZWECK: Homeostat Wrapper in contextra-db für Rerank PID Candidate Pool Regulation und Deadline Management.
// INVARIANTEN: Re-exportiert `RerankPidController` und `RerankDeadline` aus `contextra-adapt`.
// NICHT-OFFENSICHTLICH: k_min >= 50 hard floor und Integral-Windup [-100.0, 100.0] werden in contextra-adapt durchgesetzt.
// SIEHE AUCH: crates/contextra-adapt/src/homeostat.rs

pub use contextra_adapt::homeostat::RerankDeadline;
#[allow(deprecated)]
pub use contextra_adapt::homeostat::RerankPidController;
