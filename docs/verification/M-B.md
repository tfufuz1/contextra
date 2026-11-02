# Verifikationsbericht M-B: `contextra-mvcc`

**Stand:** 2026-11-02
**Crate:** `contextra-mvcc` (Ring 0 / 2)
**Spec-Referenz:** `07-contextra-mvcc.md` (MV-06, MV-07, MV-08, MV-10, T-RESTORE-3, T-CONC-1)

---

## 1. Verifikationsmatrix

| Spec-ID | Test | Benchmark | Befehl | Erwartetes Ergebnis | Status |
|---|---|---|---|---|:---:|
| **MV-06** | `tests/mv_seq_restore_tests.rs::mv_06_seq_allocator_next_seq_formula` | `BM-MVCC-SEQ-NEXT` | `cargo test -p contextra-mvcc --test mv_seq_restore_tests` | `next_seq = max(current_seq, ledger_floor) + 1` | **BESTANDEN** |
| **MV-07** | `tests/mv_tx_tests.rs::mv_07_duration_saturation_no_sentinel` | - | `cargo test -p contextra-mvcc --test mv_tx_tests` | Kein `unwrap_or(u64::MAX)`, Sättigung via Duration | **BESTANDEN** |
| **MV-08** | `tests/mv_tx_tests.rs::mv_08_lock_order_documentation_and_contract` | `BM-MVCC-TXBUF-PUT-GET` | `cargo test -p contextra-mvcc --test mv_tx_tests` | Dokumentlock → Commit-Barriere → Purge-Barrier → Journal; kein Lock über `.await` | **BESTANDEN** |
| **MV-09** | `tests/mv_tx_tests.rs::mv_09_tombstone_zero_vlen_not_bit` | - | `cargo test -p contextra-mvcc --test mv_tx_tests` | Tombstone ist Datensatz mit `vlen == 0`, kein modifiziertes Sequenzbit | **BESTANDEN** |
| **MV-10** | `tests/mv_seq_restore_tests.rs::mv_10_seq_allocator_checked_overflow_error` | - | `cargo test -p contextra-mvcc --test mv_seq_restore_tests` | Checked Arithmetik; Überlauf als Fehler, nie Wrap | **BESTANDEN** |
| **T-RESTORE-3** | `tests/mv_seq_restore_tests.rs::t_restore_3_strict_monotonic_across_restores` | - | `cargo test -p contextra-mvcc --test mv_seq_restore_tests` | Sequenz streng monoton über simulierte Restores | **BESTANDEN** |
| **T-CONC-1** | `tests/mv_loom_tests.rs::t_conc_1_loom_*` | `BM-MVCC-SEQ-NEXT` | `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test mv_loom_tests` | Loom-Modell für Concurrency & Lockreihenfolge grün | **BESTANDEN** |

---

## 2. Rückbau- und Grep-Nachweise

- **`SeqBitTombstone` in `contextra-mvcc`**:
  ```bash
  $ grep -rn "SeqBitTombstone" crates/contextra-mvcc/
  # Ergebnis: 0 Treffer (Erfolgreich bereinigt)
  ```

- **`unwrap_or(u64::MAX)` in `seq_log.rs` und `tx_buffer.rs`**:
  ```bash
  $ grep -rn "unwrap_or(u64::MAX)" crates/contextra-mvcc/src/seq_log.rs crates/contextra-mvcc/src/tx_buffer.rs
  # Ergebnis: 0 Treffer (Erfolgreich durch Duration-Sättigung ersetzt)
  ```

---

## 3. Benchmarks

Ausgeführt via `cargo bench -p contextra-mvcc --bench bm_mvcc_seq -- --test`:
- `BM-MVCC-SEQ-NEXT`: Parallele Sequenzallokation unter 1, 2, 4, 8, 16, 32 und 64 Worker-Threads ohne Deadlocks.
- `BM-MVCC-TXBUF-PUT-GET`: Staging- und Abfrage-Durchsatz des sharded Transaktionspuffers.
- Bericht abgelegt unter `target/bench-report/contextra-mvcc.json`.
