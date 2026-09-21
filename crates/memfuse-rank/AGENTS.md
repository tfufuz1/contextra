# AGENTS.md — memfuse-rank

## Crate Scope
`memfuse-rank` is a Ring 0 crate implementing multi-signal rank fusion, score calibration (Platt/Isotonic), and drift monitoring.

## Guidelines
- `#![forbid(unsafe_code)]` must be respected.
- Ensure all tests pass with `cargo test -p memfuse-rank`.
