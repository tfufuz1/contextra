// FILE-CONTEXT
// ZWECK: Homeostat Wrapper in contextra-db für Rerank PID Candidate Pool Regulation und Deadline Management.
// INVARIANTEN: Re-exportiert `RerankPidController`, `pid_regulated_candidate_pool` und `RerankDeadline` aus `contextra-adapt`.

#![allow(deprecated)]

pub use contextra_adapt::homeostat::{
    pid_regulated_candidate_pool, RerankDeadline, RerankPidController,
};
