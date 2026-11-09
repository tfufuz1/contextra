# Hand-Off Fragment P02: Offene Punkte und Fremdfehler

Stand: 2026-11-09
Bearbeiter: P02 (contextra-mvcc)

## Fremdfehler & CI-Blocker (außerhalb BESITZ)
1. **`tokio` 1.53.1 Kompilierfehler unter `--cfg loom` [V]:**
   - Befehl: `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --release loom`
   - Ursache: Die externe Abhängigkeit `tokio` 1.53.1 (transitiv herangezogen über `criterion 0.5.1` im dev-dependency-Tree) schlägt fehl:
     `error[E0432]: unresolved import crate::sync::AtomicWaker in tokio-1.53.1/src/task/local.rs:9:5 (gated behind #[cfg(not(loom))])`.
   - Bewertung: Tokio 1.53.1 unterstützt in der verwendeten Konfiguration kein Kompilieren unter `--cfg loom`.
   - Lokaler Status: Der Test `tests/loom_snapshot_registry.rs` existiert und ist unter Standard-Laufzeit (`cargo test -p contextra-mvcc --test loom_snapshot_registry`) grün.
2. **`contextra-ports` Deprecation-Warnungen bei `-D warnings` [V]:**
   - Bei globalem `cargo clippy -p contextra-mvcc -- -D warnings` bricht die Kompilierung in `contextra-ports` (außerhalb BESITZ) wegen depricateter Typen (`ErasureId`, `GateId`, etc.) ab.
   - Mit `--no-deps` ist `contextra-mvcc` zu 100 % sauber ohne Warnungen.
