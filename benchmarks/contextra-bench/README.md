# contextra-bench

**Zweck:** Reproduzierbares Benchmark-Harness für Retrieval-Genauigkeit und Leistung. SPEC §10.4 (Messmethodik) und Anhang D (`benchmarks/budgets.toml`).

**Ebene:** E3, Benchmark-Crate (benchmarks/contextra-bench)

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod adapters
pub mod ann_benchmarks
pub mod beir_eval
pub mod compare
pub mod matrix
pub mod metrics
pub mod path_rag_sweep
pub mod regression_gate
```

## Features (`Cargo.toml`)

`datasets`, `default`, `external-benchmarks`

## Dokumentation

Das Dokument `PLAN.md` in diesem Verzeichnis ist historisch und nicht normativ (siehe `CLAUDE.md`). Maßgeblich sind `docs/SPEC.md` v4.1, §10.4 und Anhang D.

