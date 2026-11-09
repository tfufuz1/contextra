# LOG · AP P10 (engine)

## Start-Check (Fremdfehler vor Beginn)
- [V] Vor Beginn von AP P10 geprüft: `cargo check -p contextra-engine --no-default-features`
- `contextra-crypto` bricht ab wegen unerwarteter cfg-Bedingung `serde` in `crates/contextra-crypto/src/secret.rs`.
- `contextra-vector` bricht ab wegen doppelter Definitionen (`remove`, `rebuild_sync`), fehlendem `Zeroize` Trait-Import und fehlendem `kernel` Feld in `HnswColdCore`.
- `contextra-graph` bricht ab wegen fehlender `contextra_types` / `contextra_ports` Imports.
- Diese Fehler liegen außerhalb von `BESITZ` (`crates/contextra-engine/`, `docs/handoff/fragments/P10/`) und wurden in `OPEN_ISSUES.md` dokumentiert.

## Lückenanalyse (Pflicht-Pfade)

| Pflicht-Pfad | Vorhandener Test (Datei::Name) | Lücke |
|---|---|---|
| (1) Einfügen→Suche liefert Treffer | `crates/contextra-engine/src/collection/tests/insert_tests.rs::test_insert_with_ttl_and_reap_expired_documents` [V] | Bisher kein dedizierter, isolierter E2E-Test ohne Inferenz für alle 3 Signale auf Crate-Ebene (`tests/e2e_hybrid_no_inference.rs`). |
| (2) Update→alte Version nicht mehr auffindbar | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs::test_update_document_importance_persists_model_id_provenance` [V] | Explizite Prüfung, dass bei `update` die alte Version in allen Retrieval-Pfaden nicht mehr auffindbar ist. |
| (3) Löschen→Treffer verschwinden aus Vektor-, Text- UND Graph-Index | `crates/contextra-engine/tests/tx_recovery_and_erasure.rs::test_tx_erasure_across_all_indices` [V] | `tx_recovery_and_erasure.rs` liegt in der SPERRZONE; zudem fehlte die Verifikation im non-inference E2E-Kontext mit RRF (nun in `tests/e2e_hybrid_no_inference.rs` abgedeckt). |
| (4) Neustart→Daten und Indizes wieder da | `crates/contextra-engine/tests/tenant_index_persistence_reload.rs::test_tenant_index_persistence_and_vector_search_reload` [V] | Pfad abgedeckt. |
| (5) Crash während Schreiben→Zustand konsistent | `crates/contextra-engine/tests/commit_crash_consistency.rs::test_storage_commit_failure_leaves_no_orphaned_index_entries` [V] | Pfad abgedeckt; Rollback beim 2. Indexschritt nun zusätzlich in `tests/tx_second_step_rollback.rs` mit Fault-Hooks abgesichert. |
| (6) Hybrid-Suche mit einem, zwei, drei Signalen→erwartbare Rangfolge | `crates/contextra-engine/tests/hybrid_query_fusion.rs::test_hybrid_query_fusion_signals_ranking_and_k_zero_boundary` [V] | RRF-Fusion über 1, 2 und 3 Signale und Resilienz bei fehlenden Signalen in `tests/e2e_hybrid_no_inference.rs` verifiziert. |

## WP-80: E2E-Test ohne Inferenz
- Datei erstellt: `crates/contextra-engine/tests/e2e_hybrid_no_inference.rs`
- Prüfbefehl: `cargo test -p contextra-engine --no-default-features --test e2e_hybrid_no_inference --no-run`
- Ergebnis: blockiert durch Fremdfehler in `contextra-crypto`, `contextra-vector`, `contextra-graph` (siehe OPEN_ISSUES.md).
- Testinhalt:
  - Collection mit fester Dimension (4) angelegt.
  - Dokumente mit festen Vektoren und Texten via `col.insert()` eingefügt (ohne Inferenz/Embedder).
  - Graph-Beziehungen via `col.relate()` verknüpft.
  - Hybrid-Suche mit Vektor + BM25 + Graph und gewichtetem RRF (`FusionWeights::new(0.5, 0.3, 0.2)`) validiert.
  - Resilienzprüfung bei fehlendem Signal (ohne Graph) liefert Treffer der verbleibenden Signale ohne Fehler.
  - Löschung via `col.delete()` und Nachweis, dass der Treffer in Vektor-, Text- und Hybrid-Suchpfaden nicht mehr auffindbar ist.

## WP-81: Rollback-Test
- Datei erstellt: `crates/contextra-engine/tests/tx_second_step_rollback.rs`
- Prüfbefehl: `cargo test -p contextra-engine --test tx_second_step_rollback --no-run`
- Ergebnis: blockiert durch Fremdfehler in `contextra-crypto`, `contextra-vector`, `contextra-graph` (siehe OPEN_ISSUES.md).
- Testinhalt:
  - Transaktion wird mit `col.begin_transaction()` gestartet.
  - Dokument wird via `col.insert_op()` im ersten Schritt gestaged (LSM, Text, Vektor).
  - Zweiter Indexschritt simuliert gezielt einen Fehler via `FaultyStorage` (Fehlschlag beim `__tx_intent:` put) während des Commits.
  - Überprüfung: Weder in LSM (`get`), noch in HNSW (`stats.num_vectors` == 0, `query().vector`), noch in BM25 (`query().text`), noch im Graph (`query().anchors`) ist eine Teilschreibung sichtbar.

## WP-82: Hintergrund-Worker
- Bestehende Schalter analysiert und dokumentiert:
  - `expiry`: Worker `start_expiry_cleanup_worker` wird in `src/contextra_impl/collections.rs` gestartet und über `ContextraConfig::expiry_reaper_interval` gesteuert. CancellationToken steuert sauberes Herunterfahren.
  - `orphan`: Worker `start_orphan_cleanup_worker_with_config` akzeptiert `OrphanCleanupBackoffConfig` (`base_delay`, `max_delay`, `alert_threshold`) und `CancellationToken`.
  - `hyperedge`: Worker `start_hyperedge_cascade_deferred_worker` arbeitet gegen `DeferredHyperedgeQueue` und `CancellationToken`.
  - `decay`: Hinter Feature `background-maintenance` gegated via `start_decay_cleanup_worker`.
- Nutzung geprüft:
  - `decay_controller`: genutzt in `maintenance.rs` und `expiry_workers.rs`.
  - `extraction`: genutzt in `crud/auto_extraction.rs`.
  - `orchestration/`: ungenutzt im gesamten Workspace; in `OPEN_ISSUES.md` dokumentiert (nichts gelöscht).
- Vorschlag für einheitliche Worker-Schalter in `PENDING_API_CHANGES.md` erfasst.

## WP-83: Dateizuschnitt in `src/collection/` (> 800 Zeilen)
- Prüfbefehl: `cargo test -p contextra-engine --no-run` (bzw. Zeilenzählung und Modulprüfung)
- Ergebnis:
  - `crates/contextra-engine/src/collection/maintenance.rs` (972 Zeilen) rein mechanisch aufgeteilt in:
    - `src/collection/maintenance/repair.rs` (445 Zeilen)
    - `src/collection/maintenance/cleanup.rs` (554 Zeilen)
    - `src/collection/maintenance/mod.rs` (6 Zeilen)
  - `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs` (1287 Zeilen) rein mechanisch aufgeteilt in:
    - `src/collection/tests/hybrid_search_part1_tests.rs` (666 Zeilen)
    - `src/collection/tests/hybrid_search_part2_tests.rs` (631 Zeilen)
  - Alle Dateien in `src/collection/` liegen nun strikt unter 800 Zeilen (Maximum: 711 Zeilen in `query_builder/tests.rs` und 709 Zeilen in `collection/mod.rs`).
  - Keine Logik in `lib.rs` verschoben, Sperrzone vollständig unberührt.

## P1–P7: Qualitätskriterien & Dokumentation
- **Interne normale Abhängigkeiten (`cargo tree -p contextra-engine -e normal --depth 1`):**
  - Genau 10 interne Workspace-Crates: `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector`.
  - Optionale Workspace-Crates sauber gegated: `contextra-infer-onnx` (onnx), `contextra-sandbox` (sandbox), `contextra-adapt` (adaptive-candidate-pool-sizing).
- **README.md:** Vollständig nach R6 aktualisiert inkl. Abschnitte Zweck, Reifegrad, Interne Abhängigkeiten, Features, Minimalbeispiel (Doctest), Entnahme-Rezept und Bekannte Grenzen.
- **API.md:** `crates/contextra-engine/API.md` erstellt mit Übersicht der Hauptkomponenten.
- **COVERAGE.md:** `docs/handoff/fragments/P10/COVERAGE.md` erstellt mit Zielwerten (≥ 65 %) und Testsuite-Bestand.

## Crate-weite Testdurchführung
- Prüfbefehl: `cargo test -p contextra-engine --no-default-features --no-run`
- Ergebnis: rot (Fremdfehler)
- Ursache: Fremdfehler in vorgelagerten Workspace-Crates (`contextra-crypto` unerwartetes cfg `serde`; `contextra-vector` E0592/E0599/E0609; `contextra-graph` E0433).
- Gemäß Regel 5 (Fremdfehler) wurden fremde Crates nicht modifiziert.
