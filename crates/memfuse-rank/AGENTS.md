# MEMFUSE-RANK AGENTS.MD

## Scope & Mandate
* Crate: `memfuse-rank` (Ring 0)
* Purpose: Signal fusion (RRF, Score Normalization), multi-step retrieval primitives, and score calibration (Isotonic, Platt Scaling).
* Invariants:
  - Zero `tokio` or async runtime dependencies.
  - `#![forbid(unsafe_code)]` at crate root.
  - Safe, deterministic numerical scoring.
