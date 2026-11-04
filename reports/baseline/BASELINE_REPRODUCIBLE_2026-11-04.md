# Reproduzierbarer Baseline-Bericht (WP-W.0, Gate G-A)

Datum: 2026-11-04
Verfasser: Jules (Principal Rust Engineer)

## 1. Umgebungs- und Versionsangaben

- **Aktueller Commit (origin/main):** `cb0d8a123d03c0ea2cb61dedcbf88a2e8e379d5d` (Merge pull request #5639 from tfufuz1/wp/P02-2453246744350013902)
- **Referenzierter Checkout-Stand:** `f6ab520e543559a4f8286801397edc899e4e8639`
  *(Hinweis: Zwischen `f6ab520` und `cb0d8a123` wurden ausschließlich Dokumentations- und ADR-Dateien geändert; der Stand von Quelltext und `Cargo.lock` / `Cargo.toml` ist identisch, was durch vergleichende Messung belegt wurde).*
- **Gepinnter Rust-Compiler (rust-toolchain.toml):**
  `rustc 1.97.0 (2d8144b78 2026-07-07)`

## 2. Arbeitsverzeichnis-Prüfung

Befehle:
```bash
git status --ignored
git clean -nd
```

Ergebnis:
- `git status --ignored`:
  Keine veralteten Dateien wie `mod.rs` neben `*.rs` vorhanden (insbesondere in `crates/contextra-erasure/src/` keine `mod.rs` vorhanden, da diese im PR #5637 entfernt wurden).
  Ignorierte Verzeichnisse:
  - `.agents/jules/context/`
  - `target/`
- `git clean -nd`:
  Keine unversionierten / zu entfernenden Dateien im Arbeitsverzeichnis (`git clean -nd` liefert leere Ausgabe).

## 3. Gate-Messung

Befehl:
```bash
cargo check --workspace --all-targets --locked --keep-going 2> baseline.err
```

- **Gesamtergebnis:** FEHLGESCHLAGEN (Exit-Code 101)
- **Ungeschnittene Rohdaten:** Gesichert in `baseline.err` (6.237 Zeilen, 271 KB).
- **Anzahl fehlschlagender Compile-Einheiten (`error: could not compile`):** 218 Compile-Einheiten.

### Bestätigung der Erwartung
- **lib-Fehler:** Exakt **1** lib schlägt fehl: ausschließlich `contextra-engine` (`error: could not compile contextra-engine (lib)`).
- **Andere Produktions-Libraries:** Alle anderen 14 Produktions-Crates (`contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-simd`, `contextra-durable-fs`, `contextra-crypto`, `contextra-mvcc`, `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank`, `contextra-erasure`, `contextra`, `contextra-testkit`, `ctx-verify`) kompilieren als Library fehlerfrei (`cargo check --workspace --lib` zeigt nur Fehler in `contextra-engine`).
- **Tests und Benchmarks:** Hunderte Compilerfehler in Integrationstests und Benches resultierend aus fehlenden Dev-Abhängigkeiten in `Cargo.toml` (z. B. `tokio`, `proptest`, `trybuild`) und unvollständigen Test-Surface-Wirings.

## 4. Fehleranzahl je Compile-Einheit

Zusammenfassung nach Crate:
- `contextra-store`: 102 Einheiten (6 Benches, 1 Lib-Test, 95 Tests)
- `contextra-vector`: 46 Einheiten (9 Benches, 1 Lib-Test, 36 Tests)
- `contextra-text`: 16 Einheiten (1 Lib-Test, 15 Tests)
- `contextra-mvcc`: 16 Einheiten (3 Benches, 1 Lib-Test, 12 Tests)
- `contextra-durable-fs`: 8 Einheiten (2 Benches, 6 Tests)
- `contextra-ports`: 7 Einheiten (1 Bench, 1 Lib-Test, 5 Tests)
- `contextra-types`: 6 Einheiten (1 Lib-Test, 5 Tests)
- `contextra-testkit`: 4 Einheiten (1 Bench, 1 Lib-Test, 2 Tests)
- `ctx-verify`: 4 Einheiten (4 Tests)
- `contextra-engine`: 2 Einheiten (1 Lib, 1 Lib-Test)
- `contextra-simd`: 2 Einheiten (1 Bench, 1 Test)
- `contextra-crypto`: 2 Einheiten (1 Bench, 1 Test)
- `contextra-testmacros`: 1 Einheit (1 Test)
- `contextra-rank`: 1 Einheit (1 Test)
- `contextra-sys`: 1 Einheit (1 Test)

Gesamt: **218 fehlschlagende Compile-Einheiten**.

### Detaillierte Liste der fehlschlagenden Einheiten

#### `contextra-engine` (2 Einheiten)
- `contextra-engine` (lib)
- `contextra-engine` (lib test)

#### `contextra-store` (102 Einheiten)
- `contextra-store` (lib test)
- `contextra-store` (bench "block_cache_bench")
- `contextra-store` (bench "lsm_concurrency_bench")
- `contextra-store` (bench "memtable_bench")
- `contextra-store` (bench "redb_comparative_bench")
- `contextra-store` (bench "sstable_bench")
- `contextra-store` (bench "wal_bench")
- `contextra-store` (test "adaptive_compaction_regression")
- `contextra-store` (test "adaptive_compaction_tombstone")
- `contextra-store` (test "amplification_benchmark")
- `contextra-store` (test "budget_drift")
- `contextra-store` (test "campaign_j11_h3")
- `contextra-store` (test "campaign_j11_memory_deletion")
- `contextra-store` (test "campaign_j21_hypotheses")
- `contextra-store` (test "campaign_prop_lsm_ops")
- `contextra-store` (test "campaign_stress_store")
- `contextra-store` (test "chaos_bitflip_sstable")
- `contextra-store` (test "chaos_dropped_write")
- `contextra-store` (test "chaos_matrix")
- `contextra-store` (test "chaos_memory_pressure")
- `contextra-store` (test "chaos_power_cut")
- `contextra-store` (test "chaos_task_massacre")
- `contextra-store` (test "commit_cancel_chain_consistency")
- `contextra-store` (test "commit_cancellation_atomicity")
- `contextra-store` (test "commit_core_batch_rollback")
- `contextra-store` (test "commit_core_cancel_safety")
- `contextra-store` (test "commit_paths_equivalence")
- `contextra-store` (test "commit_read_lock_concurrency")
- `contextra-store` (test "compaction_contiguous_selection")
- `contextra-store` (test "compaction_correctness_and_pinning")
- `contextra-store` (test "compaction_key_range_selection")
- `contextra-store` (test "compaction_merge_operator")
- `contextra-store` (test "compaction_noncontiguous_shadowing")
- `contextra-store` (test "compaction_overlap_scan_and_tombstone")
- `contextra-store` (test "crash_kill_durability")
- `contextra-store` (test "crash_prefix_enumeration")
- `contextra-store` (test "crash_recovery")
- `contextra-store` (test "differential_redb")
- `contextra-store` (test "docid_128_kv_engine")
- `contextra-store` (test "encryption_test")
- `contextra-store` (test "executor_starvation_test")
- `contextra-store` (test "fault_injection_recovery")
- `contextra-store` (test "filename_uniqueness_test")
- `contextra-store` (test "flush_crash_simulation")
- `contextra-store` (test "flush_durability")
- `contextra-store` (test "flush_mvcc_versions")
- `contextra-store` (test "flush_recovery_lifecycle")
- `contextra-store` (test "flush_threshold_boundary")
- `contextra-store` (test "fsync_syscall_verification")
- `contextra-store` (test "fsync_timeout_rollback")
- `contextra-store` (test "group_commit_fault_injection")
- `contextra-store` (test "group_commit_follower_timeout_chain")
- `contextra-store` (test "group_commit_leader_cancel_chain")
- `contextra-store` (test "group_commit_stress")
- `contextra-store` (test "lsm_robustness_fixes")
- `contextra-store` (test "lsm_storage_merge_operator")
- `contextra-store` (test "manifest_compaction_audit")
- `contextra-store` (test "manifest_corruption")
- `contextra-store` (test "manifest_crash_no_resurrection")
- `contextra-store` (test "manifest_replace_rank_compat")
- `contextra-store` (test "memory_only_crash_semantics")
- `contextra-store` (test "memtable_shard_distribution")
- `contextra-store` (test "metrics_sink_test")
- `contextra-store` (test "model_based_mvcc_flush")
- `contextra-store` (test "observer_blocking_failopen")
- `contextra-store` (test "proptest_model_based")
- `contextra-store` (test "proptest_sstable_zero_copy_block_decode")
- `contextra-store` (test "read_path_audit")
- `contextra-store` (test "recovery_fd_exhaustion_regression")
- `contextra-store` (test "recovery_salt_dirlock")
- `contextra-store` (test "resource_leaks")
- `contextra-store` (test "rollback_sstables")
- `contextra-store` (test "spec_store_tests")
- `contextra-store` (test "ssi_tracked_read_seq_clamp")
- `contextra-store` (test "ssi_tracked_reads_via_trait")
- `contextra-store` (test "ssi_unbounded_snapshot_seq")
- `contextra-store` (test "ssi_write_skew_integration")
- `contextra-store` (test "sstable_audit")
- `contextra-store` (test "sstable_block_bloom_fpr")
- `contextra-store` (test "sstable_format_v2_compat")
- `contextra-store` (test "sstable_format_v3_to_v4_compat")
- `contextra-store` (test "tenant_scoped_storage")
- `contextra-store` (test "test_wp02_flush_mvcc_bug")
- `contextra-store` (test "tmp_file_naming_determinism")
- `contextra-store` (test "toctou_put_if_absent")
- `contextra-store` (test "tombstone_scan")
- `contextra-store` (test "wal_backpressure")
- `contextra-store` (test "wal_frame_limits")
- `contextra-store` (test "wal_hmac_chain_ordering")
- `contextra-store` (test "wal_key_lifecycle")
- `contextra-store` (test "wal_no_hmac_replay")
- `contextra-store` (test "wal_observer")
- `contextra-store` (test "wal_observer_contract")
- `contextra-store` (test "wal_observer_integration")
- `contextra-store` (test "wal_observer_test")
- `contextra-store` (test "wal_open_heal")
- `contextra-store` (test "wal_recovery_runtime_stall")
- `contextra-store` (test "wal_replay_parity")
- `contextra-store` (test "wal_spec_pending")
- `contextra-store` (test "wal_systematic_crash")
- `contextra-store` (test "wal_truncate_ordering")
- `contextra-store` (test "wal_truncation_and_legacy_isolation")

#### `contextra-vector` (46 Einheiten)
- `contextra-vector` (lib test)
- `contextra-vector` (bench "acorn_vs_post_filter")
- `contextra-vector` (bench "audit_benchmarks")
- `contextra-vector` (bench "distance_bench")
- `contextra-vector` (bench "distance_nan_impact_bench")
- `contextra-vector` (bench "flush_threshold_amplification")
- `contextra-vector` (bench "hnsw_bench")
- `contextra-vector` (bench "hnsw_delete_search_bench")
- `contextra-vector` (bench "overfetch_selectivity")
- `contextra-vector` (bench "sq8_bench")
- `contextra-vector` (test "acorn_hnsw_differential")
- `contextra-vector` (test "adaptive_ef_recall")
- `contextra-vector` (test "campaign_hnsw_backoff")
- `contextra-vector` (test "campaign_j23_vector_index")
- `contextra-vector` (test "campaign_j24_test")
- `contextra-vector` (test "deleted_nodes_lock_contention")
- `contextra-vector` (test "determinism_test")
- `contextra-vector` (test "differential_testing")
- `contextra-vector` (test "distance_determinism")
- `contextra-vector` (test "docid_128_hnsw")
- `contextra-vector` (test "filtered_selectivity_recall")
- `contextra-vector` (test "hnsw_edge_cases")
- `contextra-vector` (test "hnsw_layer_determinism")
- `contextra-vector` (test "hnsw_rebuild_search_consistency")
- `contextra-vector` (test "hnsw_seed_determinism")
- `contextra-vector` (test "hnsw_snapshot_delete_test")
- `contextra-vector` (test "k02_snapshot_rebuild_determinism")
- `contextra-vector` (test "loom_quantizer_race_test")
- `contextra-vector` (test "malformed_data_test")
- `contextra-vector` (test "nan_validation_policy")
- `contextra-vector` (test "partial_rebuild_recall")
- `contextra-vector` (test "partial_rebuild_veto_f02_guard")
- `contextra-vector` (test "poisoning")
- `contextra-vector` (test "proptest_distance_quantize")
- `contextra-vector` (test "quantizer_encapsulation_regression")
- `contextra-vector` (test "r1_vector_repair_test")
- `contextra-vector` (test "ram_reduction")
- `contextra-vector` (test "recalibration")
- `contextra-vector` (test "recall")
- `contextra-vector` (test "recall_after_deletion_gates")
- `contextra-vector` (test "recall_vs_brute_force")
- `contextra-vector` (test "rollback")
- `contextra-vector` (test "simd_numerical_audit")
- `contextra-vector` (test "sq8_bias_calibration")
- `contextra-vector` (test "sq8_rerank_recall")
- `contextra-vector` (test "vector_candidate_stream")

#### `contextra-text` (16 Einheiten)
- `contextra-text` (lib test)
- `contextra-text` (test "alloc_profiler")
- `contextra-text` (test "audit_inspection")
- `contextra-text` (test "bm25_candidate_stream")
- `contextra-text` (test "concurrent_metadata")
- `contextra-text` (test "index_purge_proptests")
- `contextra-text` (test "pipeline_identity_regression")
- `contextra-text` (test "proptest_bm25_invariants")
- `contextra-text` (test "purge_error_injection_tests")
- `contextra-text` (test "t_text_spec_tests")
- `contextra-text` (test "text_search_at_wrappers")
- `contextra-text` (test "tombstone_read_your_writes")
- `contextra-text` (test "tombstone_update")
- `contextra-text` (test "wand_differential")
- `contextra-text` (test "wand_tie_break_threshold")
- `contextra-text` (test "write_amplification")

#### `contextra-mvcc` (16 Einheiten)
- `contextra-mvcc` (lib test)
- `contextra-mvcc` (bench "bm_mvcc_seq")
- `contextra-mvcc` (bench "bm_mvcc_ssi")
- `contextra-mvcc` (bench "mvcc_bench")
- `contextra-mvcc` (test "b16_reproduction")
- `contextra-mvcc` (test "campaign_cov_tx_buffer_reap")
- `contextra-mvcc` (test "isolation_tests")
- `contextra-mvcc` (test "mv_seq_restore_tests")
- `contextra-mvcc` (test "mv_ssi_validation_test")
- `contextra-mvcc` (test "reference_model_proptest")
- `contextra-mvcc` (test "ssi_coarsening_proptest")
- `contextra-mvcc` (test "ssi_determinism_proptest")
- `contextra-mvcc` (test "ssi_hardening")
- `contextra-mvcc` (test "ssi_metrics_observability")
- `contextra-mvcc` (test "t_snap_phase_a")
- `contextra-mvcc` (test "visibility_proptest")

#### `contextra-durable-fs` (8 Einheiten)
- `contextra-durable-fs` (bench "bm_dfs_atomic")
- `contextra-durable-fs` (bench "bm_dfs_probe")
- `contextra-durable-fs` (test "basic")
- `contextra-durable-fs` (test "df_atomic_crash_matrix")
- `contextra-durable-fs` (test "df_lock_tests")
- `contextra-durable-fs` (test "df_probe_tests")
- `contextra-durable-fs` (test "df_scrub_tests")
- `contextra-durable-fs` (test "recovery_aries_matrix")

#### `contextra-ports` (7 Einheiten)
- `contextra-ports` (lib test)
- `contextra-ports` (bench "bm_dispatch_overhead")
- `contextra-ports` (test "po_barrier_memory_contract")
- `contextra-ports` (test "po_erasure_contract")
- `contextra-ports` (test "po_keys_contract")
- `contextra-ports` (test "po_ledger_contract")
- `contextra-ports` (test "po_types_ordering_proptests")

#### `contextra-types` (6 Einheiten)
- `contextra-types` (lib test)
- `contextra-types` (test "distance_metrics_and_embeddings_suite")
- `contextra-types` (test "error_and_witness_suite")
- `contextra-types` (test "model_fingerprint_and_schema_suite")
- `contextra-types` (test "newtype_invariants_suite")
- `contextra-types` (test "spec_compliance_suite")

#### `contextra-testkit` (4 Einheiten)
- `contextra-testkit` (lib test)
- `contextra-testkit` (bench "bm_tk_fault_vfs")
- `contextra-testkit` (test "conformance_selftest")
- `contextra-testkit` (test "in_memory_doubles")

#### `ctx-verify` (4 Einheiten)
- `ctx-verify` (test "field_scan")
- `ctx-verify` (test "measurements")
- `ctx-verify` (test "parser_robustness")
- `ctx-verify` (test "report_verification")

#### `contextra-simd` (2 Einheiten)
- `contextra-simd` (bench "simd_bench")
- `contextra-simd` (test "simd_cross_validation")

#### `contextra-crypto` (2 Einheiten)
- `contextra-crypto` (bench "crypto_benchmarks")
- `contextra-crypto` (test "wp32_spec_tests")

#### `contextra-testmacros` (1 Einheit)
- `contextra-testmacros` (test "trybuild_tests")

#### `contextra-rank` (1 Einheit)
- `contextra-rank` (test "rrf_property_tests")

#### `contextra-sys` (1 Einheit)
- `contextra-sys` (test "sy_platform_tests")

## 5. Abschlussprüfungen

### 1. `cargo check --workspace --all-targets --locked --keep-going`
- **Exit-Code:** 101
- **Status:** 218 Compile-Einheiten fehlerhaft. Vollständige Logausgabe in `baseline.err`.

### 2. `cargo test -p contextra-arch-tests --locked`
- **Exit-Code:** 101 (3 Fehlgeschlagen, 46 Bestanden)
- **Unit-Tests (`src/lib.rs`):** 30 bestanden, 0 fehlgeschlagen.
- **Integrationstests (`tests/arch.rs`):** 16 bestanden, 3 fehlgeschlagen:
  1. `test_t_gov_1_ratchet_hygiene`: `.agents/governance/ratchet.toml` existiert nicht am erwarteten Pfad.
  2. `test_t_arch_3b_matrix_edges`: Kanten `contextra-vector -> contextra-mvcc` und `contextra-vector -> contextra-sys` laut Matrix (SPEC §5.3) nicht erlaubt.
  3. `test_t_lint_3_obsolete_evidence_terms`: Veralteter Begriff 'unwiederbringlich' in `crates/contextra-ports/src/vfs.rs:125`.

### 3. `cargo fmt --all -- --check`
- **Exit-Code:** 1
- **Status:** Formatierungsabweichungen in mehreren Crates (`contextra-store`, `contextra-testkit`, `contextra-text`).
