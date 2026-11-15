# Extraktion fuer `acta-history`

**Aufgabe:** Versionskontrolle K: Objekte, Refs, Sichtbarkeitsmengen, Diff, Merge, Takedown  
**Level:** L3 · **Budget:** 6500 LOC · **Spezifikation:** docs/SPEC.md Kap. 6  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-pii, acta-ledger, acta-vault, acta-seg, acta-gate  
**Umfang des Archivs:** 37 Dateien, 11401 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `mvcc` | Nur Referenz: tx_seq (6.3) und Schnappschuss-Registry (3.6). Git-Objektmodell (6.1) ist Neubau |
| `checkpoint` | Nur Referenz: Schnappschuss-/Zeitreise-Muster |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 5 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 7 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 90 | tokio/async (G-NOTOKIO: Kern synchron) |
| panic | 137 | unwrap/expect/panic (G-LINT) |
| hub | 10 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 2 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |
| liste_a | 2 | verbotene Formulierung (1.6, Liste A) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `blake3`, `bytes`, `parking_lot`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-history/`) | LOC | Befunde |
|---|---:|---|
| `checkpoint/src/contract.rs` | 162 | async=12 |
| `checkpoint/src/error.rs` | 82 |  |
| `checkpoint/src/fs_seam.rs` | 69 | std_fs=2 |
| `checkpoint/src/guard.rs` | 855 | panic=13, async=47, float=4 |
| `checkpoint/src/guard/tests.rs` | 300 | panic=10, async=32 |
| `checkpoint/src/hardlink_cloner.rs` | 294 | async=11, serde=2, std_fs=3 |
| `checkpoint/src/lib.rs` | 72 | async=1, serde=1 |
| `checkpoint/src/manifest.rs` | 259 | panic=2, serde=1 |
| `checkpoint/src/meta.rs` | 270 | panic=5, serde=3 |
| `checkpoint/src/meta/tests.rs` | 97 | panic=5 |
| `checkpoint/src/orphan/tests.rs` | 134 | panic=2 |
| `checkpoint/src/snapshot_source.rs` | 55 |  |
| `checkpoint/src/store.rs` | 960 | async=39, float=1, serde=1 |
| `checkpoint/src/store/tests.rs` | 886 | panic=88, async=86 |
| `checkpoint/src/vfs.rs` | 15 |  |
| `checkpoint/tests/cache_concurrency_pinning.rs` | 303 | panic=16, async=25 |
| `checkpoint/tests/checkpoint_clock_determinism.rs` | 207 | panic=14, async=10 |
| `checkpoint/tests/concurrency.rs` | 214 | panic=9, async=24 |
| `checkpoint/tests/corruption_hardening_test.rs` | 81 | panic=5, std_fs=1 |
| `checkpoint/tests/crash_recovery_and_tenant_isolation.rs` | 675 | panic=66, async=47, std_fs=2 |
| `checkpoint/tests/error_paths_and_boundaries.rs` | 272 | panic=4, async=24 |
| `checkpoint/tests/guard_exit_paths.rs` | 604 | panic=46, async=46, std_fs=2 |
| `checkpoint/tests/guard_proptest.rs` | 233 | panic=5, async=18 |
| `checkpoint/tests/hardlink_cloner_tests.rs` | 180 | async=6, std_fs=1 |
| `checkpoint/tests/id_gen_determinism_test.rs` | 33 |  |
| `checkpoint/tests/manifest_fault_injection.rs` | 94 | panic=7 |
| `checkpoint/tests/time_travel_correctness.rs` | 853 | panic=61, async=54 |
| `checkpoint/tests/txid_monotonicity_and_recovery.rs` | 304 | panic=17, async=21 |
| `db/src/temporal_filter.rs` | 383 |  |
| `mvcc/src/lib.rs` | 112 | panic=1 |
| `mvcc/src/seq_log.rs` | 855 | panic=11 |
| `mvcc/src/snapshot.rs` | 962 | panic=4 |
| `mvcc/src/tombstone.rs` | 52 |  |
| `mvcc/tests/loom_sequence_log.rs` | 160 | panic=10 |
| `mvcc/tests/loom_snapshot_registry.rs` | 166 | panic=8 |
| `mvcc/tests/tombstone_diff.rs` | 40 |  |
| `mvcc/tests/visibility_proptest.rs` | 108 |  |
