// FILE-CONTEXT
// ZWECK: Homeostat Wrapper in contextra-db für Rerank PID Candidate Pool Regulation und Deadline Management.
// INVARIANTEN: Re-exportiert `RerankPidController` und `RerankDeadline` aus `contextra-adapt`.

pub use contextra_adapt::homeostat::RerankDeadline;
#[allow(deprecated)]
pub use contextra_adapt::homeostat::RerankPidController;

// TODO(contextra-plan): Enforce candidate pool PID regulation invariants (k_min >= 50 hard floor recall ground, integral windup bound [-100.0, 100.0]) for RerankPidController [ref:eigenbau-index-store-sync]
