# Open Issues – Fragment P08

## Fremdfehler (außerhalb BESITZ)

1. **`contextra-crypto` Kompilierfehler (unexpected-cfgs & clippy)** [V]
   - **Fehler:** `error: unexpected 'cfg' condition value: 'serde'` in `crates/contextra-crypto/src/secret.rs:94:7` und `105:7`.
   - **Clippy-Fehler:** 25 Indexing-Slicing- und Type-Truncation-Fehler in `crates/contextra-crypto/src/kv_segment/store.rs` und `wal_crypto.rs`.
   - **Auswirkung:** `cargo check -p contextra-kvcache --features crypto-compat` und `cargo test -p contextra-kvcache` scheitern upstream an `contextra-crypto`.
   - **Regel:** Fremdfehler nicht im eigenen PR beheben (Regel 5).

2. **`contextra-kvcache` Bauzustand ohne Default-Features / `ports-compat`** [V]
   - **Status:** `contextra-kvcache` besitzt Module (`service.rs`, `segment.rs`, `store.rs`), die direkt Typen aus `contextra-types` (`ContextraError`, `TenantId`, `TxId`) und `contextra-ports` (`AttentionExporter`, `KvBridgeStorage`, `Clock`) importieren, ohne dass ein vollständiger lokaler Typ-Fallback für `no-default-features` vorliegt.
   - **Meldung als `frozen`-Kandidat:** Gemäß BUILD_LADDER.md / P1–P4 wird `contextra-kvcache` hiermit als `frozen`-Kandidat für den Workspace-Default-Build gemeldet, bis die Entkopplung von `contextra-crypto` (WP-35) und saubere Feature-Gatings vollständig konsolidiert sind.

3. **`Cargo.lock` Lockfile-Diskrepanz** [V]
   - Das Repository enthält auf `restructure/main` / `origin/main` nicht synchronisierte Abhängigkeiten in `Cargo.lock`, sodass Builds mit `--locked` fehlschlagen. Gemäß Regel 2 wird `Cargo.lock` vor Commits zurückgesetzt (`git checkout -- Cargo.lock`).

4. **GitHub Actions CI-Blocker (Workspace-Level / Außerhalb BESITZ)** [V]
   - **xtask Workspace-Fehlstelle:** Workflows scheitern mit `manifest path xtask/Cargo.toml does not exist` / `package(s) xtask not found in workspace` (u. a. `check-duplicate-symbols-cross-file`, `check-fbs-drift`, `check-marker-drift`, `check-commit-diff-integrity`, `check-ring-layering`).
   - **Baukasten Verification Gates (G1-G10):** 40 pre-existing G1-Verletzungen in fremden Crates (`contextra`, `contextra-agent`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-checkpoint`, etc.).
   - **Dependency Policy Gate (cargo deny):** Bekannte Sicherheitsadvisories in `wasmtime` (RUSTSEC-2026-0424) und `zip` (RUSTSEC-2026-0367, RUSTSEC-2026-0368, RUSTSEC-2026-0369, RUSTSEC-2026-0429) in `Cargo.lock`.
   - **Cargo clippy / build in contextra-vector & contextra-text:** Mehrfache Duplikate (`duplicate definitions for remove`, `duplicate definitions for rebuild_sync`) und fehlende Traits/Felder (`zeroize`, `kernel`) in `contextra-vector` sowie clippy disallowed types (`std::collections::HashSet`, `HashMap`) und Slicing-Warnungen in `contextra-text`.
   - **TSan / Mutants / Loom Gates:** Scheitern infolge der Upstream-Kompilierfehler in `contextra-crypto` und `contextra-vector`.
   - **Regel-Konformität:** Gemäß Regel 2 (Besitzprinzip) und Regel 5 (Fremdfehler) darf P08 keine fremden Crates oder Root-Konfigurationsdateien reparieren. Diese Fehler blockieren die CI-Checks auf Workspace-Ebene vollständig.
