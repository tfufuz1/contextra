# Open Issues P06 · contextra-graph

- [Fremdfehler] `crates/contextra-crypto/src/` enthält Syntaxfehler bei Deletion-Proof-Prüfungen (`duplicate definitions with name remove` und ungelöste Imports in `contextra-vector`), wodurch Workspace-weites `cargo check --workspace` fehlschlägt. Liegt außerhalb BESITZ.
- [Fremdfehler] `cargo clippy -p contextra-graph --all-targets -- -D warnings` bricht ab, weil `contextra-ports/src/lib.rs` veraltete Deprecated-Typen (`ErasureId`, `GateId`, `ScanMethod`, etc.) re-exportiert, die unter `-D warnings` als Fehler gewertet werden. Da `contextra-ports` außerhalb BESITZ liegt, wurde dieser Fremdfehler hier erfasst.
- [H] Prüfung der Engine-Abhängigkeiten auf experimentelle Module ergab, dass `contextra-engine` Typen aus `csr`, `hyperedge`, `percolation` und `ppr` nutzt.
