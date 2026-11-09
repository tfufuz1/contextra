# Open Issues P06 · contextra-graph

- [Fremdfehler] `Production Unwrap Counter Gate`: Der Zähler auf `main` liegt bei >1300, während `.github/unwrap_baseline.txt` bei 1197 steht (Baseline-Drift auf `main`). Außerhalb BESITZ (.github/).
- [Fremdfehler] `Cargo Deny Check`: Meldet Sicherheitsadvisories für `lopdf` (in `contextra-ingest`), `lru`, `tokio` (AsyncFd), `zip` (in `contextra-ingest`/`candle-core`). Außerhalb BESITZ (`deny.toml`/`Cargo.lock`).
- [Fremdfehler] `check` / `proof-tests`: Rufen `cargo xtask` auf, welches aus dem Workspace entfernt wurde (`error: package(s) xtask not found in workspace`). Außerhalb BESITZ (.github/workflows/).
- [Fremdfehler] `Compile Check`: Bricht ab an Syntaxfehlern in `contextra-crypto` (`duplicate definitions with name remove` und ungelöste Imports in `contextra-vector`). Außerhalb BESITZ.
- [Fremdfehler] `Format Check`: Schlägt an unformatierten Bestandsdateien in `contextra-store` und `contextra-vector` fehl. Außerhalb BESITZ.
- [Fremdfehler] `ThreadSanitizer`: Bricht ab am Kompilierfehler in `contextra-crypto`. Außerhalb BESITZ.
- [Fremdfehler] `context-gates`: G1-Prüfung `tools/gate_dependencies.py` schlägt fehl an 41 Bestandsverletzungen anderer Crates (u. a. `contextra-testkit` Dev-Deps, `contextra-types`).
- [H] Prüfung der Engine-Abhängigkeiten auf experimentelle Module ergab, dass `contextra-engine` Typen aus `csr`, `hyperedge`, `percolation` und `ppr` nutzt.
