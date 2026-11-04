# Contextra Voll-Audit

- Commit: `762807450` (2026-11-04 02:35:33 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: True
- Start: 2026-11-04T01:50:55+00:00  Dauer: 0.1 min
- Toolchain-Pin: `1.97.0`; Preflight: **PASS** (cargo 1.97.0 (c980f4866 2026-06-30) | Pin 1.97.0)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121012 kB
- Nutzer: freddy

## Gesamt

- **INFO**: 3
- **PASS**: 4
- **PENDING**: 185

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung oder ein Hinweis ohne Urteil.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|

## Reife-Behauptung vs. Beleg (capabilities.toml gegen diesen Lauf)

| Crate | behauptet | lib-Tests | Clippy lib | Urteil |
|---|---|---|---|---|
| `contextra` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-arch-tests` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-crypto` | beta | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-durable-fs` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-engine` | beta | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-mvcc` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-ports` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-rank` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-simd` | beta | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-store` | beta | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-sys` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-testkit` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-text` | beta | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-types` | stable | PENDING | PENDING | **Behauptung ohne Beleg** |
| `contextra-vector` | beta | PENDING | PENDING | **Behauptung ohne Beleg** |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | PASS | 0.1 | keine undefinierten Features |
| B-befunde | `befund_default_members` | INFO | 0.0 | members=19 default-members=16; nicht in default-members (von `cargo test` ohne --workspace nicht erfasst): ['contextra-lab', 'contextra-testmacros', 'ctx-verify'] |
| B-befunde | `befund_maturity_claims` | INFO | 0.0 | stable=9 beta=6 Eintraege ohne Crate=0 |
| B-befunde | `befund_bench_fuzz_dateien` | PASS | 0.0 | 0 fehlende Dateien/Verzeichnisse |
| 0-basis | `toolchain_preflight` | PASS | 0.1 | cargo 1.97.0 (c980f4866 2026-06-30) | Pin 1.97.0 |
| 0-basis | `toolchain_versionen` | INFO | 0.5 |  |
| 0-basis | `cargo_fetch` | PASS | 1.1 |  |
| 0-basis | `cargo_tree_duplicates` | PENDING | 0.0 |  |
| 1-statisch | `fmt_check` | PENDING | 0.0 |  |
| 1-statisch | `check_alle_targets` | PENDING | 0.0 |  |
| 1-statisch | `check_no_default_features` | PENDING | 0.0 |  |
| 1-statisch | `doc` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-arch-tests` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-arch-tests` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-crypto` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-durable-fs` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-durable-fs` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-engine` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-engine` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-erasure` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-erasure` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-lab` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-lab` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-mvcc` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-ports` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-ports` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-rank` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-rank` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-simd` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-store` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-store` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-sys` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-testkit` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-testmacros` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-testmacros` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-text` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-text` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-types` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-types` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-vector` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-vector` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__ctx-verify` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__ctx-verify` | PENDING | 0.0 |  |
| 1-statisch | `feature_matrix_hack` | PENDING | 0.0 |  |
| 1-statisch | `deny` | PENDING | 0.0 |  |
| 1-statisch | `audit` | PENDING | 0.0 |  |
| 1-statisch | `panic_flaeche_lib` | PENDING | 0.0 |  |
| 1-statisch | `unsafe_inventar` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-arch-tests` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-arch-tests` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-arch-tests` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-arch-tests` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_lib__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `test_integration__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `test_doc__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `nextest__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_debug` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_release` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_crates` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_funktionen` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_verwaiste_dateien` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-arch-tests` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-crypto` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-durable-fs` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-engine` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-erasure` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-lab` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-mvcc` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-ports` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-rank` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-simd` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-store` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-sys` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-testkit` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-testmacros` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-text` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-types` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-vector` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__ctx-verify` | PENDING | 0.0 |  |
| 3-testbarkeit | `coverage` | PENDING | 0.0 |  |
| 3-testbarkeit | `coverage_funktionen` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-mvcc__mvcc_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-crypto__crypto_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__distance_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__sq8_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__hnsw_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__hnsw_delete_search_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__audit_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__flush_threshold_amplification` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__partial_rebuild_recall_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__distance_nan_impact_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__filtered_search_selectivity_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__sstable_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__memtable_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__wal_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__lsm_concurrency_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__block_cache_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__redb_comparative_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-simd__simd_bench` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__wal_hmac_chain_verify_fuzz` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_compaction_interleave` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_manifest_load` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_memtable_concurrent` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_recovery_arbitrary_state` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_sstable_binary_search` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_wal_replay` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_mutation_chaos` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_roundtrip` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_bm25_tokenize` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_tokenizer_smoke` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__fuzz_hnsw_persistence` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__hnsw_insert_search` | PENDING | 0.0 |  |
| 5-tief | `mutation_tier0_1` | PENDING | 0.0 |  |
| 5-tief | `loom_mvcc` | PENDING | 0.0 |  |
| 5-tief | `miri_vector_scalar` | PENDING | 0.0 |  |
| 5-tief | `asan_vector` | PENDING | 0.0 |  |
