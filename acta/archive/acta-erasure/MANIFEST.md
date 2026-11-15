# Extraktion fuer `acta-erasure`

**Aufgabe:** Loeschkern: Saga, Gate-Journal, Messung, Restore-Schutz, Mehrbetroffenen-Plan  
**Level:** L4 · **Budget:** 9000 LOC · **Spezifikation:** docs/SPEC.md Kap. 8  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs, acta-gate, acta-ledger, acta-vault, acta-seg, acta-pstore  
**Umfang des Archivs:** 58 Dateien, 12015 Zeilen (inkl. Tests)  

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
| `erasure/benches/bm_erase.rs` | 47 | zeit_zufall=3, panic=1 |
| `erasure/fuzz/fuzz_targets/fuzz_ledger_record.rs` | 12 |  |
| `erasure/src/backend.rs` | 33 |  |
| `erasure/src/erasure_domain.rs` | 634 | serde=1, hr6=23, alt_spec=13 |
| `erasure/src/error.rs` | 126 | alt_spec=1 |
| `erasure/src/io.rs` | 65 | async=1, std_fs=1, alt_spec=1 |
| `erasure/src/journal.rs` | 76 |  |
| `erasure/src/journal/appendlog.rs` | 386 |  |
| `erasure/src/journal/epoch.rs` | 54 | alt_spec=1 |
| `erasure/src/journal/record.rs` | 406 | serde=1 |
| `erasure/src/journal/reduce.rs` | 24 |  |
| `erasure/src/journal/scope_purge.rs` | 79 | alt_spec=2, liste_a=1 |
| `erasure/src/journal/wal.rs` | 86 | serde=3, panic=2, alt_spec=1 |
| `erasure/src/journal/wal_sink.rs` | 407 |  |
| `erasure/src/lib.rs` | 45 |  |
| `erasure/src/measure.rs` | 142 | hr6=5, alt_spec=6 |
| `erasure/src/measure/field.rs` | 327 | alt_spec=5, liste_a=1 |
| `erasure/src/measure/positive_control.rs` | 177 |  |
| `erasure/src/measure/residue.rs` | 567 | serde=1 |
| `erasure/src/measure/scanner_flow.rs` | 197 | hr6=1, alt_spec=1 |
| `erasure/src/measure/witness.rs` | 62 | alt_spec=2 |
| `erasure/src/orchestrator.rs` | 148 | serde=1, alt_spec=3 |
| `erasure/src/purge.rs` | 27 |  |
| `erasure/src/purge/cx.rs` | 48 | alt_spec=3 |
| `erasure/src/purge/order.rs` | 54 | alt_spec=1 |
| `erasure/src/purge/phase.rs` | 309 | serde=1 |
| `erasure/src/purge/reserve.rs` | 41 | alt_spec=1 |
| `erasure/src/purge/resume.rs` | 87 | serde=1 |
| `erasure/src/purge/window.rs` | 76 | async=1, alt_spec=2 |
| `erasure/src/replace.rs` | 139 | serde=1, alt_spec=5 |
| `erasure/src/restore.rs` | 30 |  |
| `erasure/src/restore/guard.rs` | 68 | alt_spec=5 |
| `erasure/src/restore/replay.rs` | 49 | alt_spec=4 |
| `erasure/src/restricted.rs` | 212 | serde=1, alt_spec=4 |
| `erasure/src/saga.rs` | 26 |  |
| `erasure/src/saga/core.rs` | 933 | async=11, alt_spec=17 |
| `erasure/src/saga/driver.rs` | 146 | async=1, alt_spec=4 |
| `erasure/src/saga/idempotency.rs` | 20 | alt_spec=1 |
| `erasure/src/saga/machine.rs` | 192 | serde=1, alt_spec=7 |
| `erasure/src/saga/retention.rs` | 23 |  |
| `erasure/src/saga/seams.rs` | 129 | async=1 |
| `erasure/src/scope.rs` | 18 | alt_spec=1 |
| `erasure/src/scope/codec.rs` | 163 |  |
| `erasure/src/scope/file.rs` | 186 | async=2, alt_spec=3 |
| `erasure/src/scope/flight.rs` | 119 | serde=1, panic=1, alt_spec=3 |
| `erasure/src/scope/set.rs` | 78 | serde=1, alt_spec=3 |
| `erasure/src/scope/single_flight.rs` | 362 | serde=1, alt_spec=5 |
| `erasure/src/supersede.rs` | 175 | serde=3, alt_spec=3 |
| `erasure/src/test_mocks.rs` | 81 | async=1 |
| `erasure/src/types_local.rs` | 2440 | serde=1, zeit_zufall=3, std_fs=7, alt_spec=5, liste_a=1 |
| `erasure/tests/crash_matrix_recovery.rs` | 276 | async=2, panic=8 |
| `ports/src/barrier.rs` | 166 | alt_spec=2 |
| `ports/src/lifecycle.rs` | 68 | float=3, serde=1, alt_spec=1 |
| `ports/src/scanner/layer.rs` | 124 | serde=1 |
| `ports/src/scanner/mod.rs` | 216 | panic=5, hub=1, alt_spec=2 |
| `ports/src/scanner/session.rs` | 107 |  |
| `ports/src/scanner/witness.rs` | 109 | serde=1 |
| `types/src/erasure.rs` | 618 | serde=1, hr6=24, alt_spec=13 |
