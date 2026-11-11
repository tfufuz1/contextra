# BASELINE — contextra-mvcc

- **Commit-Stand:** `882aada727461d7567f25693f668bbe75c9877d5`
- **Datum:** 11. November 2026
- **Toolchain:** `rustc 1.89.0`

## 1. Messungen & Befehle

### 1.1 Crate-Builds & Tests
- `cargo check -p contextra-mvcc --all-targets --locked`: **GRÜN**
- `cargo check -p contextra-mvcc --no-default-features --locked`: **GRÜN**
- `cargo test -p contextra-mvcc --locked`: **GRÜN** (52 Unit-Tests, 21 Integrationstest-Suiten bestanden)
- `cargo clippy -p contextra-mvcc --all-targets --locked -- -D warnings`: **GRÜN** (0 Warnungen)

### 1.2 Loom Concurrency Tests
- `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-mvcc --test loom_sequence_log`: **GRÜN**
- `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-mvcc --test loom_snapshot_registry`: **GRÜN**
- `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-mvcc --test loom_tx_buffer`: **GRÜN**

### 1.3 Golden-Korpus
- `tests/golden_corpus.rs`: Alle Tests bestanden, Golden-Korpus unverändert.
