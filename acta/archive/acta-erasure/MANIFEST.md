# Extraktion fuer `acta-erasure`

**Aufgabe:** Loeschkern: Saga, Gate-Journal, Messung, Restore-Schutz, Mehrbetroffenen-Plan  
**Level:** L4 · **Budget:** 9000 LOC · **Spezifikation:** docs/SPEC.md Kap. 8  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs, acta-gate, acta-ledger, acta-vault, acta-seg, acta-pstore  
**Umfang des Archivs:** 76 Dateien, 16365 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `erasure` | Saga S0-S9 (8.3), Messung/Positivkontrolle (8.5), Restore (8.8); async-Seams -> synchron (3.1 Nr. 7) |
| `ports` | Port-Vorlagen Messung/Barriere |
| `types` | Domaenentypen Loeschung |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 3 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 23 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 20 | tokio/async (G-NOTOKIO: Kern synchron) |
| zeit_zufall | 6 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 17 | unwrap/expect/panic (G-LINT) |
| std_fs | 8 | std::fs direkt (G-FS: nur acta-fs) |
| hr6 | 53 | include!/#[deprecated] (HR-6) |
| hub | 1 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 131 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |
| liste_a | 3 | verbotene Formulierung (1.6, Liste A) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `bincode`, `blake3`, `futures-util`, `parking_lot`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-erasure/`) | LOC | Befunde |
|---|---:|---|
| `erasure/benches/bm_erase.rs` | 46 | panic=1 |
| `erasure/fuzz/fuzz_targets/fuzz_ledger_record.rs` | 11 |  |
| `erasure/src/backend.rs` | 32 | async=2 |
| `erasure/src/erasure_domain.rs` | 633 | serde=1 |
| `erasure/src/error.rs` | 125 |  |
| `erasure/src/io.rs` | 64 | async=1, std_fs=2 |
| `erasure/src/journal.rs` | 75 |  |
| `erasure/src/journal/appendlog.rs` | 385 |  |
| `erasure/src/journal/epoch.rs` | 53 |  |
| `erasure/src/journal/record.rs` | 405 | serde=1 |
| `erasure/src/journal/reduce.rs` | 23 |  |
| `erasure/src/journal/scope_purge.rs` | 78 |  |
| `erasure/src/journal/wal.rs` | 85 | panic=2, serde=3 |
| `erasure/src/journal/wal_sink.rs` | 406 |  |
| `erasure/src/lib.rs` | 44 |  |
| `erasure/src/measure.rs` | 141 |  |
| `erasure/src/measure/field.rs` | 326 |  |
| `erasure/src/measure/positive_control.rs` | 176 |  |
| `erasure/src/measure/residue.rs` | 566 | serde=1 |
| `erasure/src/measure/scanner_flow.rs` | 196 |  |
| `erasure/src/measure/witness.rs` | 61 | alt_spec=1 |
| `erasure/src/orchestrator.rs` | 147 | serde=1 |
| `erasure/src/purge.rs` | 26 | async=1 |
| `erasure/src/purge/cx.rs` | 47 |  |
| `erasure/src/purge/order.rs` | 53 |  |
| `erasure/src/purge/phase.rs` | 308 | serde=1 |
| `erasure/src/purge/reserve.rs` | 40 |  |
| `erasure/src/purge/resume.rs` | 86 | serde=1 |
| `erasure/src/purge/window.rs` | 75 | async=3 |
| `erasure/src/replace.rs` | 138 | serde=1 |
| `erasure/src/restore.rs` | 29 |  |
| `erasure/src/restore/guard.rs` | 67 |  |
| `erasure/src/restore/replay.rs` | 48 |  |
| `erasure/src/restricted.rs` | 211 | serde=1 |
| `erasure/src/saga.rs` | 25 | async=2 |
| `erasure/src/saga/core.rs` | 932 | async=13 |
| `erasure/src/saga/driver.rs` | 145 | async=8 |
| `erasure/src/saga/idempotency.rs` | 19 |  |
| `erasure/src/saga/machine.rs` | 191 | serde=1 |
| `erasure/src/saga/retention.rs` | 22 |  |
| `erasure/src/saga/seams.rs` | 128 | async=3 |
| `erasure/src/scope.rs` | 17 |  |
| `erasure/src/scope/codec.rs` | 162 |  |
| `erasure/src/scope/file.rs` | 185 | async=2 |
| `erasure/src/scope/flight.rs` | 118 | panic=1, serde=1, alt_spec=1 |
| `erasure/src/scope/set.rs` | 77 | serde=1 |
| `erasure/src/scope/single_flight.rs` | 361 | serde=1 |
| `erasure/src/supersede.rs` | 174 | serde=3 |
| `erasure/src/test_mocks.rs` | 80 | async=4 |
| `erasure/src/types_local.rs` | 2439 | async=2, serde=1, alt_spec=3 |
| `erasure/tests/crash_matrix_recovery.rs` | 275 | panic=8, async=19 |
| `erasure/tests/journal_durable_spec_tests.rs` | 397 | panic=17 |
| `erasure/tests/loom_erasure_models.rs` | 76 | panic=2 |
| `erasure/tests/measure_real_scan_tests.rs` | 368 | panic=27 |
| `erasure/tests/measure_spec_tests.rs` | 280 | panic=20 |
| `erasure/tests/p3_purge_and_scope_tests.rs` | 460 | panic=27, async=14 |
| `erasure/tests/p4_fence_tests.rs` | 325 | panic=18, alt_spec=1 |
| `erasure/tests/p8_residue_scan_tests.rs` | 247 | panic=7, alt_spec=2 |
| `erasure/tests/proof_crash_resume_matrix.rs` | 207 | panic=13 |
| `erasure/tests/proof_fail_closed_and_consistency.rs` | 149 | panic=12, std_fs=1 |
| `erasure/tests/saga_core_orchestration.rs` | 547 | panic=12, async=36 |
| `erasure/tests/scope_codec_spec_tests.rs` | 209 | panic=5, async=6 |
| `erasure/tests/special_targets_test.rs` | 289 | panic=22 |
| `erasure/tests/t_inv_era.rs` | 169 | panic=8, async=2, alt_spec=1 |
| `erasure/tests/t_measure.rs` | 84 | panic=4 |
| `erasure/tests/t_observer.rs` | 69 | async=4 |
| `erasure/tests/t_saga_journal.rs` | 186 | panic=17, async=9 |
| `erasure/tests/t_saga_spec_extensions.rs` | 189 | panic=4 |
| `ports/src/barrier.rs` | 165 |  |
| `ports/src/lifecycle.rs` | 67 | float=3, serde=1 |
| `ports/src/observer.rs` | 157 | panic=1, async=1 |
| `ports/src/scanner/layer.rs` | 123 | serde=1 |
| `ports/src/scanner/mod.rs` | 215 | panic=5 |
| `ports/src/scanner/session.rs` | 106 | alt_spec=3 |
| `ports/src/scanner/witness.rs` | 108 | serde=1 |
| `types/src/erasure.rs` | 617 | serde=1 |
