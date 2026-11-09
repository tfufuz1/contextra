# Übergabebericht: MISSION `MVCC-1` · MVCC: Sequenzlog, Snapshots, Serializable Snapshot Isolation

## 1. Zusammenfassung & Zielerreichung
Der Transaktionskern `contextra-mvcc` wurde gemäß Soll-Zustand (SPEC v4.0, Ring 0, Klasse B Storage-Bundle) gehärtet, verifiziert und dokumentiert.
Alle Invarianten bzgl. Sichtbarkeit, Snapshot-Registry, SSI-Validierung, Coarsening, Bounded Resources und Fail-Closed-Durability wurden am Code belegt und getestet.

## 2. Soll/Ist-Vergleich
| Soll-Anforderung (§3) | Status | Beleg / Umsetzung |
|---|---|---|
| **Sichtbarkeit:** Version `(key, seq)` sichtbar $\iff seq \le s.read\_seq$ und committed; Tombstones und Updates folgen demselben Gesetz | Erfüllt | `SeqLogEntry::is_visible`, `test_sequence_log_tombstones_and_updates_follow_same_law`, `visibility_proptest.rs` |
| **Snapshot-Registry:** Thread-sichere Registrierung, `oldest_active_snapshot()` als GC-Untergrenze, konfigurierbare Kapazität (`max_active_snapshots`, `max_snapshot_age`), fail-closed Abweisung bei Limitüberschreitung | Erfüllt | `SnapshotRegistryConfig`, `oldest_active_snapshot()`, `try_register()`, `try_register_for_tenant()`, `test_snapshot_registry_try_register_capacity_limit` |
| **SSI Write-Skew & Dangerous Structures:** Konservative Konflikterkennung von rw-Antidependenzen und Pivots | Erfüllt | `SequenceLogSsiValidator::validate_internal`, `validate_and_record`, `ssi_write_skew.rs`, `history_anomaly_checker.rs` |
| **SSI Speicherbegrenzung & Coarsening:** Lesemengen vergröbern bei Schwellenwert (80 %), 0 % False Negatives, Fail-Closed Protection | Erfüllt | `check_and_coarsen_if_needed`, `coarsen_keys_to_prefixes`, `ssi_coarsening_proptest.rs` |
| **SSI Sentinel-Schutz:** `prune_through_checked` weist `u64::MAX`, `TOMBSTONE_BIT` und `>= i64::MAX as u64` fail-closed ab | Erfüllt | `SequenceLogSsiValidator::prune_through_checked`, `test_sentinel_boundary_values_and_tombstone_bit` |
| **Tx-Puffer:** Byte-Budget je Transaktion und global (`max_tx_staged_bytes`, `max_total_staged_bytes`), deterministisches Orphan-Reaping via injizierte Clock | Erfüllt | `TxBuffer::stage_bounded`, `reap_orphans_at`, `tx_buffer_byte_budget.rs`, `campaign_cov_tx_buffer_reap.rs` |
| **Umwelt & Reinheit:** `Clock`/`IdGen` injiziert; keine `SystemTime`-Nutzung im Kern; Bundle-Kanten nur zu `contextra-types`/`contextra-ports` | Erfüllt | `determinism_guard` (0 Verstöße), `erasure_leak_prober` (0 Leaks), M2 Isolation Proof SUCCESS |
| **Lock-Disziplin:** Dokumentierte Sperrenordnung (INV-LOCK-ORDER-1), keine Locks über `.await` | Erfüllt | `concurrency_lock_auditor` (0 await hazards), `loom_snapshot_registry.rs` |
| **Verifikation:** Elle/Jepsen History-Checker, Brute-Force Serialisierbarkeit, Benchmarks (Commits/s, Lesemengen-Größe) | Erfüllt | `history_anomaly_checker.rs`, `mvcc_bench.rs` |

## 3. Messwerte & Benchmarks (Criterion)
- `snapshot_registry_min_active/min_active_seqno/10`: ~1.16 ns
- `snapshot_registry_min_active/min_active_seqno/1000`: ~1.16 ns
- `snapshot_registry_register_drop/10`: ~159 ns
- `seq_log_entry_is_visible_o1`: ~1.37 ns
- `ssi_validation_by_read_set_size/validate_disjoint/10`: ~495 ns
- `ssi_validation_by_read_set_size/validate_disjoint/100`: ~6.66 µs
- `ssi_validation_by_read_set_size/validate_disjoint/1000`: ~106 µs
- `ssi_commit_throughput`: ~273 ns/op (~3.66 Millionen Commits/s)

## 4. Funktionsinventar
- Vorher: 7616 Funktionen (MVCC: 186 gesamt / 127 pub)
- Nachher: 7626 Funktionen (MVCC: 196 gesamt / 136 pub)
- Diff: +10 Funktionen, rein additiv (neue Prüfungen, try_register, oldest_active_snapshot, prune_through_checked, History Checker Tests). Kein unbegründeter Funktionsverlust.

## 5. Exklusive Dateibesitz-Einhaltung (OWNS)
- Modifizierte/Hinzugefügte Dateien:
  - `crates/contextra-mvcc/src/snapshot.rs`
  - `crates/contextra-mvcc/src/seq_log.rs`
  - `crates/contextra-mvcc/src/ssi.rs`
  - `crates/contextra-mvcc/tests/history_anomaly_checker.rs`
  - `crates/contextra-mvcc/tests/ssi_write_skew.rs`
  - `crates/contextra-mvcc/benches/mvcc_bench.rs`
  - `docs/handoff/MVCC-1.md`
- Keine Dateien außerhalb von `crates/contextra-mvcc/**` und `docs/handoff/MVCC-1.md` wurden verändert. `Cargo.lock` bleibt unberührt.
