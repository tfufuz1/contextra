# LOG (Welle 2 - P20 - WP-14)

## Vorab-Check (Start-Check)
- [V] Analyse von `docs/handoff/fragments/P00/BUILD_LADDER.md` und Klon-Zustand am HEAD `6f7454d1`:
  - `contextra-crypto` baute bereits vor Arbeitsbeginn nicht (Fehler: unerwartetes `cfg`-Feature `serde`).
  - `contextra-graph` baute bereits vor Arbeitsbeginn nicht (Fehler: Mehrfachdeklaration von `contract`).
  - `contextra-vector` baute bereits vor Arbeitsbeginn nicht (Fehler: Doppeldefinition `remove`).
  - `contextra-engine`, `contextra-db`, `contextra`, `contextra-mcp` waren dadurch bereits vorab blockiert.
  - Fremdfehler werden in `OPEN_ISSUES.md` festgehalten.

## Entscheidung zu `lifecycle.rs` (Aufgabe 1)
- [V] Analyse von `crates/contextra-engine/src/contextra_impl/lifecycle.rs`:
  - Das Modul verwaltet ausschließlich den Lebenszyklus der Engine-Instanz: `open()`, `open_with_config()`, `open_authorized()`, SSI-Tracking-Prüfung, Reparatur nach Crashs (`repair_on_open()`), Intent-Scanning (`scan_pending_intents()`), Shutdown (`shutdown()`, `wait_shutdown()`) sowie Embedder-Konfiguration (`with_embedder()`, `set_embedder()`).
  - Es enthält keine Löschlogik, keine DeletionProofs und keine Erasure-Sagas (nur die Initialisierung des Felds `deletion_proof_active = false` für Config-Kompatibilität).
  - Mechanische Integrität und Crash-Recovery beim Öffnen müssen intakt bleiben.
  - **Entscheidung:** `lifecycle.rs` ist NICHT lösch- oder nachweisspezifisch und verbleibt daher in `crates/contextra-engine/src/contextra_impl/lifecycle.rs`. Es wird NICHT archiviert.

## Commits & Arbeitspakete

### WP-14: Erasure-/Beweis-Pfad zurückbauen, crypto-Abhängigkeit entfernen
- **Aktionen:**
  - `crates/contextra-engine/src/contextra_impl/erasure.rs` per `git mv` nach `archive/contextra-engine-erasure/erasure.rs` verschoben.
  - In `crates/contextra-engine/src/contextra_impl/mod.rs` die Moduldeklaration `pub mod erasure;` entfernt.
  - In `crates/contextra-engine/src/lib.rs` Re-Exports (`summary_from_witness`, `ErasureReport`, etc.) bereinigt.
  - In `crates/contextra-engine/src/contextra_impl/collections.rs` `drop_collection` auf rein mechanische Löschung (`col.drop_collection()`) umgestellt.
  - In `crates/contextra-engine/Cargo.toml` Abhängigkeit `contextra-crypto` entfernt.
  - Proof-spezifische Tests (`erasure_orchestrator_tests.rs`, `erasure_report_honesty.rs`, `proof_verify_not_constant.rs`) per `git mv` nach `archive/tests-legacy/contextra-engine/` verschoben.
  - `tests/tx_recovery_and_erasure.rs` beibehalten, da es rein mechanische ACID/Tombstone-Invariante prüft.
  - Fragmente `LOG.md`, `OPEN_ISSUES.md` und `PENDING_API_CHANGES.md` erstellt.
- **Dateien:**
  - `crates/contextra-engine/Cargo.toml`
  - `crates/contextra-engine/src/contextra_impl/collections.rs`
  - `crates/contextra-engine/src/contextra_impl/mod.rs`
  - `crates/contextra-engine/src/lib.rs`
  - `archive/contextra-engine-erasure/erasure.rs`
  - `archive/tests-legacy/contextra-engine/erasure_orchestrator_tests.rs`
  - `archive/tests-legacy/contextra-engine/erasure_report_honesty.rs`
  - `archive/tests-legacy/contextra-engine/proof_verify_not_constant.rs`
  - `docs/handoff/fragments/P20/LOG.md`
  - `docs/handoff/fragments/P20/OPEN_ISSUES.md`
  - `docs/handoff/fragments/P20/PENDING_API_CHANGES.md`
- **Prüfbefehle und Ergebnisse:**
  1. `cargo tree -p contextra-engine -e normal --no-default-features --depth 1`:
     - Ergebnis: 9 interne Crates (`checkpoint`, `graph`, `mvcc`, `ports`, `rank`, `store`, `text`, `types`, `vector`). Weder `crypto`, `sandbox` noch `infer-onnx` vorhanden. -> [V] grün
  2. `grep -rn erasure crates/contextra-engine/src`:
     - Ergebnis: 0 Treffer. -> [V] grün
  3. `python3 tools/erasure_leak_prober.py -c contextra-engine`:
     - Ergebnis: 0 kernel domain violations found. -> [V] grün
  4. `cargo check -p contextra-engine --no-default-features`:
     - Ergebnis: rot (Fremdblocker: `contextra-graph` E0428 `contract` Mehrfachdefinition, `contextra-crypto` cfg `serde`). -> [V] dokumentiert in OPEN_ISSUES.md
  5. `cargo check -p contextra -p contextra-db -p contextra-mcp`:
     - Ergebnis: rot (Fremdblocker: `contextra-graph`). Keine direkten Compiler-Fehler bezüglich entfernter Engine-Erasure-Symbole. -> [V] dokumentiert in OPEN_ISSUES.md
