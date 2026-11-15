# Extraktion fuer `acta-seg`

**Aufgabe:** Versiegelter schluesselfreier Segmentspeicher: Journal, Segmente, Manifest  
**Level:** L2 · **Budget:** 6500 LOC · **Spezifikation:** docs/SPEC.md Kap. 5.1  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs, acta-gate  
**Umfang des Archivs:** 143 Dateien, 28922 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `store` | Segmente/Journal/Manifest (5.1): Blockformat ersetzen durch Umschlag v1; Frames wie 5.1; kein LSM, keine Compaction |
| `crypto` | Journal-HMAC-Kette (chain_tag) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 28 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| async | 1003 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 1 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 34 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 1656 | unwrap/expect/panic (G-LINT) |
| std_fs | 116 | std::fs direkt (G-FS: nur acta-fs) |
| hr6 | 1 | include!/#[deprecated] (HR-6) |
| hub | 103 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 29 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `blake3`, `bytes`, `crc32fast`, `lru`, `parking_lot`, `quick_cache`, `rand`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-seg/`) | LOC | Befunde |
|---|---:|---|
| `crypto/src/wal_completeness.rs` | 77 | async=1 |
| `crypto/src/wal_crypto.rs` | 1163 | panic=36, async=4 |
| `crypto/tests/proptest_wal_hmac_v2_collision.rs` | 272 | panic=8 |
| `crypto/tests/wal_completeness_tests.rs` | 48 |  |
| `crypto/tests/wal_integrity_audit.rs` | 200 | panic=6 |
| `store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs` | 123 | panic=2, async=2 |
| `store/fuzz/fuzz_targets/fuzz_manifest_load.rs` | 29 | async=3, std_fs=1 |
| `store/fuzz/fuzz_targets/fuzz_memtable_concurrent.rs` | 118 | panic=1 |
| `store/fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs` | 62 | async=5, std_fs=3 |
| `store/fuzz/fuzz_targets/fuzz_rollback_intent.rs` | 20 |  |
| `store/fuzz/fuzz_targets/fuzz_sstable_binary_search.rs` | 36 |  |
| `store/fuzz/fuzz_targets/fuzz_sstable_block_footer.rs` | 15 |  |
| `store/fuzz/fuzz_targets/fuzz_sstable_legacy_footer.rs` | 32 | async=3, std_fs=1 |
| `store/fuzz/fuzz_targets/fuzz_tenant_codec.rs` | 9 |  |
| `store/fuzz/fuzz_targets/fuzz_wal_hmac_chain.rs` | 31 | async=3, std_fs=1 |
| `store/fuzz/fuzz_targets/fuzz_wal_replay.rs` | 40 | async=4, std_fs=2 |
| `store/fuzz/fuzz_targets/wal_mutation_chaos.rs` | 297 | async=4, std_fs=2 |
| `store/fuzz/fuzz_targets/wal_roundtrip.rs` | 38 | async=3, std_fs=1 |
| `store/src/compat/scanner/purge.rs` | 290 | async=14, std_fs=12 |
| `store/src/compat/scanner/sstable.rs` | 197 | async=5, std_fs=2 |
| `store/src/compat/scanner/wal.rs` | 178 | async=5, std_fs=2 |
| `store/src/key_source.rs` | 67 |  |
| `store/src/lsm/group_commit.rs` | 83 | async=4 |
| `store/src/lsm/lock_order.rs` | 107 | async=7 |
| `store/src/manifest/apply.rs` | 61 | async=3, alt_spec=1 |
| `store/src/manifest/cleanup.rs` | 109 |  |
| `store/src/manifest/core.rs` | 156 | async=6, std_fs=2 |
| `store/src/manifest/entry.rs` | 370 |  |
| `store/src/manifest/handle.rs` | 111 | async=8, std_fs=3 |
| `store/src/manifest/mod.rs` | 27 |  |
| `store/src/manifest/recovery.rs` | 130 |  |
| `store/src/manifest/rollover.rs` | 197 | async=8, std_fs=4 |
| `store/src/manifest/tests.rs` | 444 | panic=54, async=26, alt_spec=2, std_fs=10 |
| `store/src/sstable/block_cache/lru.rs` | 75 |  |
| `store/src/sstable/block_cache/mod.rs` | 178 | float=3 |
| `store/src/sstable/block_cache/quick.rs` | 68 |  |
| `store/src/sstable/block_cache/sieve.rs` | 126 |  |
| `store/src/sstable/block_cache/traits.rs` | 38 |  |
| `store/src/sstable/block_search.rs` | 224 |  |
| `store/src/sstable/bloom.rs` | 177 | float=4 |
| `store/src/sstable/builder/block.rs` | 180 |  |
| `store/src/sstable/builder/meta.rs` | 17 |  |
| `store/src/sstable/builder/mod.rs` | 12 |  |
| `store/src/sstable/builder/writer.rs` | 436 | async=6, float=2 |
| `store/src/sstable/format.rs` | 204 |  |
| `store/src/sstable/io.rs` | 25 |  |
| `store/src/sstable/reader/block.rs` | 99 | async=3 |
| `store/src/sstable/reader/iter.rs` | 129 | async=2 |
| `store/src/sstable/reader/lookup.rs` | 219 | async=3 |
| `store/src/sstable/reader/mod.rs` | 271 |  |
| `store/src/sstable/reader/open.rs` | 451 | async=7 |
| `store/src/sstable/reader_ext.rs` | 303 | async=2 |
| `store/src/sstable/stream.rs` | 138 | async=2 |
| `store/src/sstable/tests.rs` | 918 | panic=117, async=45, float=6, std_fs=4 |
| `store/src/tenant_codec.rs` | 200 |  |
| `store/src/wal/config.rs` | 60 |  |
| `store/src/wal/core.rs` | 668 | async=20, alt_spec=1 |
| `store/src/wal/encode.rs` | 447 |  |
| `store/src/wal/flusher.rs` | 772 | async=13 |
| `store/src/wal/flusher/command.rs` | 65 | async=6 |
| `store/src/wal/fs.rs` | 267 | async=24, std_fs=18 |
| `store/src/wal/hmac.rs` | 432 | async=15 |
| `store/src/wal/io/acl.rs` | 63 | panic=6, async=3 |
| `store/src/wal/io/append.rs` | 317 | async=23 |
| `store/src/wal/io/mod.rs` | 12 |  |
| `store/src/wal/io/scan.rs` | 468 | async=4 |
| `store/src/wal/io/truncate.rs` | 202 | async=12 |
| `store/src/wal/mod.rs` | 34 |  |
| `store/src/wal/open_heal_tests.rs` | 363 | async=16, std_fs=1 |
| `store/src/wal/poison.rs` | 45 |  |
| `store/src/wal/replay.rs` | 730 | async=10, std_fs=2 |
| `store/src/wal/tests.rs` | 18 |  |
| `store/src/wal/tests/encode_tests.rs` | 259 | panic=24, async=5, std_fs=1 |
| `store/src/wal/tests/flusher_tests.rs` | 406 | panic=15, async=25, std_fs=2 |
| `store/src/wal/tests/hmac_tests.rs` | 417 | panic=48, async=34, std_fs=5 |
| `store/src/wal/tests/io_tests.rs` | 572 | panic=58, async=43, std_fs=11 |
| `store/src/wal/tests/replay_tests.rs` | 1623 | panic=222, async=112, std_fs=39 |
| `store/tests/chaos_bitflip_sstable.rs` | 452 | panic=21, async=9, float=3, std_fs=2 |
| `store/tests/chaos_dropped_write.rs` | 166 | panic=18, async=2, std_fs=7 |
| `store/tests/chaos_matrix.rs` | 321 | panic=18, async=15, std_fs=3 |
| `store/tests/chaos_memory_pressure.rs` | 252 | panic=13, async=7 |
| `store/tests/chaos_power_cut.rs` | 182 | panic=8, async=3, std_fs=1 |
| `store/tests/chaos_task_massacre.rs` | 192 | panic=7, async=6, float=1 |
| `store/tests/corrupted_files_fail_closed.rs` | 37 | panic=7, async=5, std_fs=3 |
| `store/tests/crash_kill_durability.rs` | 492 | panic=26, async=10, std_fs=7 |
| `store/tests/crash_prefix_enumeration.rs` | 155 | panic=11, async=5, std_fs=2, unsafe=1 |
| `store/tests/crash_recovery.rs` | 128 | panic=24, async=8 |
| `store/tests/fault_injection_durability_points.rs` | 169 | panic=35, async=13, std_fs=2 |
| `store/tests/fault_injection_recovery.rs` | 138 | panic=23, async=7, std_fs=1 |
| `store/tests/format_version_compat.rs` | 145 | panic=29, async=14, std_fs=6 |
| `store/tests/golden/tenant_codec_vectors.json` | 0 |  |
| `store/tests/golden_corpus.rs` | 320 | panic=57, async=6, std_fs=1 |
| `store/tests/group_commit_fault_injection.rs` | 111 | panic=14, async=4 |
| `store/tests/group_commit_flush_race.rs` | 216 | panic=29, async=25, std_fs=3 |
| `store/tests/group_commit_follower_timeout_chain.rs` | 87 | panic=11, async=7 |
| `store/tests/group_commit_leader_cancel_chain.rs` | 83 | panic=7, async=9 |
| `store/tests/group_commit_overlapping_batch_rollback.rs` | 89 | panic=14, async=2 |
| `store/tests/group_commit_stress.rs` | 151 | panic=15, async=9 |
| `store/tests/group_commit_test.rs` | 264 | panic=29, async=16, float=2 |
| `store/tests/group_commit_timeout.rs` | 203 | panic=17, async=20 |
| `store/tests/loom_group_commit.rs` | 117 | panic=11 |
| `store/tests/loom_group_commit_handoff.rs` | 137 | panic=21 |
| `store/tests/manifest_compaction_audit.rs` | 138 | panic=11, async=2 |
| `store/tests/manifest_corruption.rs` | 220 | panic=32, async=20, std_fs=10 |
| `store/tests/manifest_crash_no_resurrection.rs` | 252 | panic=12, async=6, std_fs=9 |
| `store/tests/manifest_replace_rank_compat.rs` | 82 | panic=1, async=2 |
| `store/tests/mz_a_frame_bounds.rs` | 11 |  |
| `store/tests/mz_a_poison_table.rs` | 30 |  |
| `store/tests/mz_b_builder_byte_identity.rs` | 147 | panic=27, async=6, std_fs=6 |
| `store/tests/mz_b_support/mod.rs` | 1 |  |
| `store/tests/mz_c_manifest_byte_identity.rs` | 64 | panic=6, async=6, std_fs=1 |
| `store/tests/mz_c_support/mod.rs` | 31 |  |
| `store/tests/proof_open_rejects_corrupt.rs` | 104 | panic=20, async=6, std_fs=1 |
| `store/tests/sstable_audit.rs` | 321 | panic=40, async=10, std_fs=4 |
| `store/tests/sstable_block_bloom_fpr.rs` | 82 | panic=5, async=2, float=2 |
| `store/tests/sstable_format_v2_compat.rs` | 378 | panic=59, async=16, float=1, std_fs=3 |
| `store/tests/sstable_format_v3_to_v4_compat.rs` | 184 | panic=38, async=8 |
| `store/tests/support/mod.rs` | 86 | async=1, std_fs=1 |
| `store/tests/wal_backpressure.rs` | 345 | panic=34, async=26, std_fs=1 |
| `store/tests/wal_boundary_and_mutation_hardening.rs` | 113 | async=10, std_fs=4 |
| `store/tests/wal_every_byte_truncation.rs` | 98 | panic=13, async=8, std_fs=6 |
| `store/tests/wal_flusher_rollback.rs` | 124 | async=6, std_fs=2 |
| `store/tests/wal_frame_limits.rs` | 214 | panic=23, async=10, std_fs=4 |
| `store/tests/wal_fuzzing.rs` | 1 |  |
| `store/tests/wal_hmac_binding_attack_tests.rs` | 1 |  |
| `store/tests/wal_hmac_chain_ordering.rs` | 210 | panic=13, async=13 |
| `store/tests/wal_hmac_rollback_race.rs` | 142 | panic=14, async=7 |
| `store/tests/wal_key_lifecycle.rs` | 76 | panic=15, async=12, std_fs=2 |
| `store/tests/wal_lifecycle_crash_point_matrix.rs` | 259 | panic=1, async=3, float=2 |
| `store/tests/wal_no_hmac_replay.rs` | 64 | panic=10, async=3, std_fs=1 |
| `store/tests/wal_observer.rs` | 212 | panic=27, async=15 |
| `store/tests/wal_observer_contract.rs` | 182 | panic=17, async=11 |
| `store/tests/wal_observer_integration.rs` | 322 | panic=32, async=14 |
| `store/tests/wal_observer_test.rs` | 220 | panic=40, async=11 |
| `store/tests/wal_open_heal.rs` | 216 | async=12, std_fs=1 |
| `store/tests/wal_poison_causes.rs` | 209 | async=12 |
| `store/tests/wal_recovery_runtime_stall.rs` | 139 | panic=11, async=2, float=2, std_fs=1 |
| `store/tests/wal_replay_parity.rs` | 183 | panic=18, async=18, std_fs=8 |
| `store/tests/wal_robustness.rs` | 1 |  |
| `store/tests/wal_spec_pending.rs` | 314 | panic=51, async=13, std_fs=1 |
| `store/tests/wal_systematic_crash.rs` | 229 | panic=5, async=10, std_fs=7 |
| `store/tests/wal_torn_write_fault_injection.rs` | 139 | panic=17, async=6, std_fs=2 |
| `store/tests/wal_truncate_ordering.rs` | 306 | panic=5, async=27, std_fs=1 |
