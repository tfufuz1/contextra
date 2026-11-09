# Hand-Off Fragment P02: Protokoll & Änderungsprotokoll

## Start-Check (vor der ersten Änderung)
- `docs/handoff/fragments/P00/BUILD_LADDER.md` geprüft: [V] Datei existiert nicht.
- Crate-Status vor Bearbeitungsbeginn:
  - `cargo build -p contextra-mvcc`: [V] grün (0 Fehler)
  - `cargo test -p contextra-mvcc`: [V] grün (alle 51 Unit-Tests, 48 Integrationstests und Property-Tests grün)

## Aufgabe 1: Analyse der Testdateien und Lückenermittlung
Untersuchte Dateien:
- `tests/visibility_proptest.rs`
- `tests/ssi_determinism_proptest.rs`
- `tests/loom_snapshot_registry.rs`
- `tests/isolation_tests.rs`
- `tests/history_anomaly_checker.rs`
- `tests/ssi_write_skew.rs`
- `src/seq_log.rs`, `src/snapshot.rs`, `src/ssi.rs`, `src/tx_buffer.rs`

### Tabelle: Eigenschaft | vorhandener Test | Lücke

| Eigenschaft | Vorhandener Test | Lücke |
|---|---|---|
| (1) Snapshot sieht keine späteren Commits | `visibility_proptest.rs` (`prop_mvcc_visibility_matches_reference_10k`), `isolation_tests.rs` (`test_isolation_read_only_snapshot_never_aborted_by_writes`, `test_isolation_seq_log_boundary_seq_numbers`), `seq_log.rs` (`test_seq_log_entry_is_visible_o1`, `test_sequence_log_insert_delete_visibility`) | Handgerechneter, linearer Ablauf-Test mit fest vorgegebenen Snapshot-Zeiten, Commits und Löschungen ohne Referenzfunktionsschleife, der Schritt für Schritt nachweist, dass Snapshot `T_snap` niemals Commits `T_commit > T_snap` sieht. |
| (2) Write-Konflikte werden erkannt | `isolation_tests.rs` (`test_isolation_write_skew_classic_bank_account`, `test_isolation_lost_update_atomic_validate_and_record`), `ssi_write_skew.rs`, `history_anomaly_checker.rs` | Handgerechneter Testfall für direkte Concurrent-Write / Anti-Dependency Konflikte mit handfesten Zahlenwerten (Erwartung aus dem Modell abgeleitet, nicht aus der Implementierung). |
| (3) Lesen eigener Schreibvorgänge (Read-your-own-writes) | `tx_buffer.rs` (`test_is_key_staged_for_tx_semantics`, `test_tx_buffer_stage_many_and_max_ops_constant`) | **Vollständige Lücke:** Es existiert kein expliziter Integrationstest in `tests/isolation_tests.rs`, der das Zusammenspiel von `TxBuffer` (staged writes vor Commit) und `SequenceLog` / Snapshot liest: Eine Transaktion sieht ihre eigenen uncommitted Änderungen im lokalen Puffer, während ein fremder Snapshot zur gleichen Zeit nur den alten Stand sieht. Nach Commit sieht auch der neue Snapshot die Änderung. |
| (4) Nebenläufigkeit der zentralen Struktur unter loom | `tests/loom_snapshot_registry.rs` (`test_loom_snapshot_registry_concurrent_register_release`, `test_loom_snapshot_registry_interleaved_pin_unpin`) | Loom-Modell für `SnapshotRegistry` existiert bereits. Allerdings schlägt die Ausführung unter `RUSTFLAGS="--cfg loom"` im Workspace fehl wegen externer dev-dependency `tokio` 1.53.1 (in `criterion 0.5.1`). Lücke: Nachweis und Dokumentation der Ausführbarkeit bzw. Isolation der Loom-Ausführung und Stresstest. |

## Aufgabe 2: Lückenfüllung mit handgerechneten Isolationstests
In `crates/contextra-mvcc/tests/isolation_tests.rs` wurden 3 handgerechnete Modelltests ergänzt:
- `test_isolation_handcrafted_snapshot_never_sees_future_commits`: Lineare Prüfung über Zeitschritte T=5..40 mit Einfügung, Löschung und Nachweis der Invariante `delete_seq > as_of`.
- `test_isolation_handcrafted_write_conflict_detection_and_disjoint_reads`: Prüfung von Concurrent-Write-Konflikt (Commit 105 > Read 100 -> Conflict), disjunktem Lesezugriff (Read K2 100 -> Ok) und Grenzfall (Read 105 == Commit 105 -> Ok).
- `test_isolation_handcrafted_read_your_own_writes_lifecycle`: End-to-End-Lebenszyklus mit `TxBuffer` (Staging, `is_key_staged_for_tx`, `get_ops`) und `SequenceLog` (Isolation fremder Leser, Freigabe nach Drain und Commit).
- Prüfbefehl: `cargo test -p contextra-mvcc --test isolation_tests` → 29 passed; 0 failed (grün).

## Aufgabe 3: Prüfung der Dev-Abhängigkeiten
- Analyse von `crates/contextra-mvcc/Cargo.toml`:
  - `[dependencies]`: `contextra-types`, `contextra-ports`, `parking_lot`, `ahash`, `serde`, `bytes`, `thiserror`, `tracing`.
  - `[dev-dependencies]`: `proptest = "1.4"`, `criterion`, `loom = "0.7"`, `contextra-testkit`.
- Befund: [V] Keine verbotenen Abhängigkeiten auf höhere Crates (`store`, `vector`, `graph`, `db`, `engine`). Alle Abhängigkeiten entsprechen der Ring-0 / Foundation-Spezifikation.
