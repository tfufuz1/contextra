# Extraktion fuer `acta-seg`

**Aufgabe:** Versiegelter schluesselfreier Segmentspeicher: Journal, Segmente, Manifest  
**Level:** L2 · **Budget:** 6500 LOC · **Spezifikation:** docs/SPEC.md Kap. 5.1  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs, acta-gate  
**Umfang des Archivs:** 133 Dateien, 29356 Zeilen (inkl. Tests)  

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
| `crypto/src/wal_completeness.rs` | 78 |  |
| `crypto/src/wal_crypto.rs` | 1164 | async=4, panic=36, hr6=1 |
| `crypto/tests/proptest_wal_hmac_v2_collision.rs` | 273 | panic=8 |
| `crypto/tests/wal_completeness_tests.rs` | 49 |  |
| `crypto/tests/wal_integrity_audit.rs` | 201 | panic=6 |
| `store/src/compat/scanner/purge.rs` | 291 | async=13, std_fs=1, hub=3 |
| `store/src/compat/scanner/sstable.rs` | 198 | async=2, hub=4 |
| `store/src/compat/scanner/wal.rs` | 179 | async=2, hub=4 |
| `store/src/key_source.rs` | 68 | zeit_zufall=2 |
| `store/src/lsm/group_commit.rs` | 84 | async=4, alt_spec=1 |
| `store/src/lsm/lock_order.rs` | 108 | async=7, alt_spec=2 |
| `store/src/manifest/apply.rs` | 62 | async=3, alt_spec=3 |
| `store/src/manifest/cleanup.rs` | 110 |  |
| `store/src/manifest/core.rs` | 157 | async=6, std_fs=1 |
| `store/src/manifest/entry.rs` | 371 |  |
| `store/src/manifest/handle.rs` | 112 | async=8, std_fs=2, alt_spec=3 |
| `store/src/manifest/mod.rs` | 28 |  |
| `store/src/manifest/recovery.rs` | 131 |  |
| `store/src/manifest/rollover.rs` | 198 | async=7, std_fs=2 |
| `store/src/manifest/tests.rs` | 445 | async=26, panic=54, alt_spec=3 |
| `store/src/sstable/block_cache/lru.rs` | 76 |  |
| `store/src/sstable/block_cache/mod.rs` | 179 | float=3 |
| `store/src/sstable/block_cache/quick.rs` | 69 |  |
| `store/src/sstable/block_cache/sieve.rs` | 127 |  |
| `store/src/sstable/block_cache/traits.rs` | 39 |  |
| `store/src/sstable/block_search.rs` | 225 |  |
| `store/src/sstable/bloom.rs` | 178 | float=4 |
| `store/src/sstable/builder/block.rs` | 181 |  |
| `store/src/sstable/builder/meta.rs` | 18 |  |
| `store/src/sstable/builder/mod.rs` | 13 |  |
| `store/src/sstable/builder/writer.rs` | 437 | float=2, async=6, std_fs=2, alt_spec=4 |
| `store/src/sstable/format.rs` | 205 | alt_spec=1 |
| `store/src/sstable/io.rs` | 26 |  |
| `store/src/sstable/reader/block.rs` | 100 | async=3 |
| `store/src/sstable/reader/iter.rs` | 130 | async=2 |
| `store/src/sstable/reader/lookup.rs` | 220 | async=3 |
| `store/src/sstable/reader/mod.rs` | 272 | std_fs=1 |
| `store/src/sstable/reader/open.rs` | 452 | async=7, std_fs=2 |
| `store/src/sstable/reader_ext.rs` | 304 | async=2 |
| `store/src/sstable/stream.rs` | 139 | async=2 |
| `store/src/sstable/tests.rs` | 919 | float=6, async=45, zeit_zufall=2, panic=117, std_fs=1 |
| `store/src/tenant_codec.rs` | 201 | alt_spec=2 |
| `store/src/wal/config.rs` | 60 |  |
| `store/src/wal/core.rs` | 668 | async=20, std_fs=3, alt_spec=4 |
| `store/src/wal/encode.rs` | 448 |  |
| `store/src/wal/flusher.rs` | 772 | async=6, zeit_zufall=1 |
| `store/src/wal/flusher/command.rs` | 66 | async=6 |
| `store/src/wal/fs.rs` | 268 | async=23, std_fs=21 |
| `store/src/wal/hmac.rs` | 571 | async=15, std_fs=4 |
| `store/src/wal/io/acl.rs` | 64 | async=3, panic=6, std_fs=2 |
| `store/src/wal/io/append.rs` | 318 | async=23 |
| `store/src/wal/io/mod.rs` | 13 |  |
| `store/src/wal/io/scan.rs` | 468 | async=4 |
| `store/src/wal/io/truncate.rs` | 203 | async=13 |
| `store/src/wal/mod.rs` | 35 | alt_spec=2 |
| `store/src/wal/open_heal_tests.rs` | 364 | async=16, std_fs=2 |
| `store/src/wal/poison.rs` | 46 | alt_spec=1 |
| `store/src/wal/replay.rs` | 730 | async=10, std_fs=5 |
| `store/src/wal/tests.rs` | 19 |  |
| `store/src/wal/tests/encode_tests.rs` | 260 | async=5, panic=24 |
| `store/src/wal/tests/flusher_tests.rs` | 407 | async=26, panic=15 |
| `store/src/wal/tests/hmac_tests.rs` | 417 | async=30, panic=48, std_fs=1 |
| `store/src/wal/tests/io_tests.rs` | 573 | async=41, panic=58, std_fs=4 |
| `store/src/wal/tests/replay_tests.rs` | 1623 | async=108, zeit_zufall=5, panic=222, std_fs=7 |
| `store/tests/chaos_bitflip_sstable.rs` | 453 | float=3, async=5, panic=21, std_fs=5, hub=2 |
| `store/tests/chaos_dropped_write.rs` | 167 | async=2, panic=18, std_fs=7, hub=2 |
| `store/tests/chaos_matrix.rs` | 322 | async=15, zeit_zufall=3, panic=18, hub=2 |
| `store/tests/chaos_memory_pressure.rs` | 253 | async=6, panic=13, hub=2 |
| `store/tests/chaos_power_cut.rs` | 183 | async=3, zeit_zufall=3, panic=8, std_fs=1, hub=2 |
| `store/tests/chaos_task_massacre.rs` | 193 | float=1, async=6, zeit_zufall=3, panic=7, hub=2 |
| `store/tests/corrupted_files_fail_closed.rs` | 38 | async=5, panic=7 |
| `store/tests/crash_kill_durability.rs` | 493 | async=9, zeit_zufall=3, panic=26, std_fs=7, hub=2 |
| `store/tests/crash_prefix_enumeration.rs` | 156 | async=5, unsafe=1, panic=11, hub=2 |
| `store/tests/crash_recovery.rs` | 129 | async=8, panic=24, hub=2 |
| `store/tests/fault_injection_durability_points.rs` | 170 | async=13, panic=35, std_fs=1, hub=2 |
| `store/tests/fault_injection_recovery.rs` | 139 | async=7, panic=23, hub=2 |
| `store/tests/fixtures/.wal_integrity_key` | 0 |  |
| `store/tests/format_version_compat.rs` | 146 | async=14, panic=29, hub=2 |
| `store/tests/golden/.wal_integrity_key` | 1 |  |
| `store/tests/golden/tenant_codec_vectors.json` | 971 |  |
| `store/tests/golden_corpus.rs` | 321 | async=6, panic=57, std_fs=1, hub=1 |
| `store/tests/group_commit_fault_injection.rs` | 112 | async=3, panic=14, hub=2 |
| `store/tests/group_commit_flush_race.rs` | 217 | async=21, panic=29, std_fs=3, hub=2 |
| `store/tests/group_commit_follower_timeout_chain.rs` | 88 | async=7, panic=11, hub=2 |
| `store/tests/group_commit_leader_cancel_chain.rs` | 84 | async=8, panic=7, hub=2 |
| `store/tests/group_commit_overlapping_batch_rollback.rs` | 90 | async=2, panic=14, hub=2 |
| `store/tests/group_commit_stress.rs` | 152 | async=7, zeit_zufall=2, panic=15, hub=2 |
| `store/tests/group_commit_test.rs` | 265 | float=2, async=12, zeit_zufall=2, panic=29, hub=3 |
| `store/tests/group_commit_timeout.rs` | 204 | async=16, panic=17, hub=2 |
| `store/tests/loom_group_commit.rs` | 118 | panic=11 |
| `store/tests/loom_group_commit_handoff.rs` | 138 | panic=21 |
| `store/tests/manifest_compaction_audit.rs` | 139 | async=2, panic=11, hub=2 |
| `store/tests/manifest_corruption.rs` | 221 | async=20, panic=32 |
| `store/tests/manifest_crash_no_resurrection.rs` | 253 | async=6, panic=12, std_fs=9, hub=2 |
| `store/tests/manifest_replace_rank_compat.rs` | 83 | async=2, panic=1, hub=1 |
| `store/tests/mz_a_frame_bounds.rs` | 12 |  |
| `store/tests/mz_a_poison_table.rs` | 31 | alt_spec=1 |
| `store/tests/mz_b_builder_byte_identity.rs` | 148 | async=6, panic=27, std_fs=6 |
| `store/tests/mz_b_support/mod.rs` | 2 |  |
| `store/tests/mz_c_manifest_byte_identity.rs` | 65 | async=6, panic=6, std_fs=1 |
| `store/tests/mz_c_support/mod.rs` | 32 |  |
| `store/tests/proof_format_constants.rs` | 53 |  |
| `store/tests/proof_open_rejects_corrupt.rs` | 105 | async=6, panic=20, std_fs=1, hub=1 |
| `store/tests/sstable_audit.rs` | 322 | async=10, zeit_zufall=2, panic=40, std_fs=4 |
| `store/tests/sstable_block_bloom_fpr.rs` | 83 | float=2, async=2, panic=5 |
| `store/tests/sstable_format_v2_compat.rs` | 379 | float=1, async=16, panic=59, std_fs=1 |
| `store/tests/sstable_format_v3_to_v4_compat.rs` | 185 | async=8, panic=38, hub=1 |
| `store/tests/support/mod.rs` | 87 | async=1, std_fs=1, hub=1 |
| `store/tests/wal_backpressure.rs` | 346 | async=25, panic=34, hub=1 |
| `store/tests/wal_boundary_and_mutation_hardening.rs` | 114 | async=10, hub=4, alt_spec=1 |
| `store/tests/wal_every_byte_truncation.rs` | 99 | async=9, panic=13, hub=2 |
| `store/tests/wal_flusher_rollback.rs` | 125 | async=6, hub=3 |
| `store/tests/wal_frame_limits.rs` | 215 | async=10, panic=23, hub=1 |
| `store/tests/wal_fuzzing.rs` | 2 |  |
| `store/tests/wal_hmac_binding_attack_tests.rs` | 2 |  |
| `store/tests/wal_hmac_chain_ordering.rs` | 211 | async=10, panic=13, hub=2 |
| `store/tests/wal_hmac_rollback_race.rs` | 143 | async=5, panic=14, hub=2 |
| `store/tests/wal_key_lifecycle.rs` | 77 | async=11, panic=15 |
| `store/tests/wal_lifecycle_crash_point_matrix.rs` | 259 | float=2, async=3, panic=1, hub=1 |
| `store/tests/wal_no_hmac_replay.rs` | 65 | async=3, panic=10, hub=1 |
| `store/tests/wal_observer.rs` | 213 | async=15, zeit_zufall=1, panic=27, hub=2 |
| `store/tests/wal_observer_contract.rs` | 183 | async=10, zeit_zufall=1, panic=17, hub=2 |
| `store/tests/wal_observer_integration.rs` | 323 | async=14, zeit_zufall=1, panic=32, hub=2 |
| `store/tests/wal_observer_test.rs` | 221 | async=11, zeit_zufall=1, panic=40, hub=2 |
| `store/tests/wal_open_heal.rs` | 217 | async=12, std_fs=2, hub=1 |
| `store/tests/wal_poison_causes.rs` | 210 | async=12, hub=6, alt_spec=1 |
| `store/tests/wal_recovery_runtime_stall.rs` | 140 | float=2, async=2, zeit_zufall=2, panic=11, std_fs=2, hub=1 |
| `store/tests/wal_replay_parity.rs` | 184 | async=18, panic=18, hub=1 |
| `store/tests/wal_robustness.rs` | 2 |  |
| `store/tests/wal_spec_pending.rs` | 315 | async=13, panic=51, hub=1 |
| `store/tests/wal_systematic_crash.rs` | 230 | async=10, panic=5, std_fs=1, hub=4 |
| `store/tests/wal_torn_write_fault_injection.rs` | 140 | async=6, panic=17, std_fs=2, hub=1 |
| `store/tests/wal_truncate_ordering.rs` | 307 | async=22, panic=5, hub=2 |
