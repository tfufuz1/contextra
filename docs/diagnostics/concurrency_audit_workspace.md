# Concurrency & Lock Safety Audit Report

> **Werkzeug:** `concurrency_lock_auditor.py`  
> **Erstellungszeit:** `2026-11-07T23:15:26.964661+00:00`  
> **Status/Befunde:** `646 Befunde erfasst`  

## Beschreibung
Audits synchronization primitives, Mutex/RwLock guards across .await (INV-LOCK-ORDER-1), channel architectures, and Loom model-checking support.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `locks_held_across_await` | `191` |
| `std_sync_locks` | `43` |
| `parking_lot_locks` | `225` |
| `tokio_sync_locks` | `49` |
| `channels_used` | `37` |
| `loom_verification_points` | `101` |
| `total_concurrency_items` | `646` |

## Synchronisations-Übersicht
| Mechanismus / Risiko | Anzahl | Relevanz |
|---|---|---|
| **Lock über `.await`** | 191 | Kritische Deadlock-Gefahr / Async-Invariante |
| `std::sync::{Mutex, RwLock}` | 43 | Standardbibliothek Sperren |
| `parking_lot::{Mutex, RwLock}` | 225 | Schnelle synchrone Locks |
| `tokio::sync::{Mutex, RwLock}` | 49 | Kooperative asynchrone Sperren |
| Kanäle (`mpsc`/`broadcast`) | 37 | Message-Passing Architekturen |
| `loom` Test-Konfigurationen | 101 | Deterministische Nebenläufigkeitsprüfung |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `24` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `294` | Potential lock inversion / deadlock: Guard 'guard' (line 294) held across .await |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `330` | Potential lock inversion / deadlock: Guard 'final_guard' (line 330) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard.rs` | `515` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/hardlink_cloner.rs` | `61` | Loom permutation concurrency verification present |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/hardlink_cloner.rs` | `64` | Loom permutation concurrency verification present |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/orphan.rs` | `21` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `33` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `239` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `245` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `319` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `322` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `421` | Potential lock inversion / deadlock: Guard '_guard' (line 421) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `536` | Potential lock inversion / deadlock: Guard '_guard' (line 536) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `613` | Potential lock inversion / deadlock: Guard '_guard' (line 613) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `758` | Potential lock inversion / deadlock: Guard '_guard' (line 758) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `865` | Potential lock inversion / deadlock: Guard '_guard' (line 865) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store.rs` | `912` | Potential lock inversion / deadlock: Guard '_guard' (line 912) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/guard/tests.rs` | `7` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/store/tests.rs` | `17` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/benches/checkpoint_bench.rs` | `17` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/cache_concurrency_pinning.rs` | `25` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/cache_concurrency_pinning.rs` | `167` | Potential lock inversion / deadlock: Guard 'initial_get_count' (line 164) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/cache_concurrency_pinning.rs` | `186` | Potential lock inversion / deadlock: Guard 'get_count_after_miss' (line 185) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/checkpoint_systematic_crash.rs` | `20` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/concurrency.rs` | `19` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/crash_recovery_and_tenant_isolation.rs` | `24` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/error_paths_and_boundaries.rs` | `19` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_exit_paths.rs` | `19` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_exit_paths.rs` | `207` | Potential lock inversion / deadlock: Guard '_lock' (line 194) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_exit_paths.rs` | `238` | Potential lock inversion / deadlock: Guard '_lock' (line 231) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_exit_paths.rs` | `286` | Potential lock inversion / deadlock: Guard '_lock' (line 281) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_exit_paths.rs` | `322` | Potential lock inversion / deadlock: Guard '_lock' (line 314) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_exit_paths.rs` | `439` | Potential lock inversion / deadlock: Guard '_lock' (line 427) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_exit_paths.rs` | `587` | Potential lock inversion / deadlock: Guard '_lock' (line 577) held across .await |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/guard_proptest.rs` | `20` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/time_travel_correctness.rs` | `20` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/tests/txid_monotonicity_and_recovery.rs` | `19` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-cognition` | `crates/contextra-cognition/src/channel.rs` | `11` | Message-passing channel initialized |
| `contextra-cognition` | `crates/contextra-cognition/src/channel.rs` | `37` | Message-passing channel initialized |
| `contextra-cognition` | `crates/contextra-cognition/src/channel.rs` | `90` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-cognition` | `crates/contextra-cognition/src/channel.rs` | `97` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-cognition` | `crates/contextra-cognition/src/consolidation_executor.rs` | `526` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-cognition` | `crates/contextra-cognition/src/consolidation_executor.rs` | `548` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-cognition` | `crates/contextra-cognition/src/context_compaction/session.rs` | `271` | Potential lock inversion / deadlock: Guard '_guard' (line 271) held across .await |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `30` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-crypto` | `crates/contextra-crypto/src/revocation_log.rs` | `36` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` | `41` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` | `42` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` | `48` | Message-passing channel initialized |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` | `72` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` | `73` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_segment/store.rs` | `21` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-crypto` | `crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs` | `11` | Loom permutation concurrency verification present |
| `contextra-crypto` | `crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs` | `158` | Loom permutation concurrency verification present |
| `contextra-crypto` | `crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs` | `164` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-crypto` | `crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs` | `15` | Loom permutation concurrency verification present |
| `contextra-crypto` | `crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs` | `140` | Loom permutation concurrency verification present |
| `contextra-db` | `crates/contextra-db/src/lib.rs` | `381` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/src/lib.rs` | `723` | Potential lock inversion / deadlock: Guard 'embedder' (line 722) held across .await |
| `contextra-db` | `crates/contextra-db/src/multistep.rs` | `100` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/src/multistep.rs` | `119` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `170` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `171` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `179` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `180` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | `287` | Potential lock inversion / deadlock: Guard 'committed' (line 279) held across .await |
| `contextra-db` | `crates/contextra-db/tests/fault_injection_2pc.rs` | `29` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/fault_injection_2pc.rs` | `30` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/fault_injection_2pc.rs` | `42` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/fault_injection_2pc.rs` | `43` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-db` | `crates/contextra-db/tests/multistep_retrieval_tests.rs` | `14` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-db` | `crates/contextra-db/tests/multistep_retrieval_tests.rs` | `92` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-db` | `crates/contextra-db/tests/multistep_retrieval_tests.rs` | `127` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-db` | `crates/contextra-db/tests/multistep_retrieval_tests.rs` | `178` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-db` | `crates/contextra-db/tests/multistep_retrieval_tests.rs` | `230` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-engine` | `crates/contextra-engine/src/backends.rs` | `19` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/backends.rs` | `21` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/backends.rs` | `25` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/backends.rs` | `29` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `29` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `31` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `41` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `43` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `46` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `48` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `51` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `53` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `55` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `57` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `59` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `61` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `63` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `71` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `74` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `78` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `81` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `84` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `87` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `89` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `91` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `95` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `98` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `103` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `107` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `124` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `145` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `148` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `223` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `270` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `294` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `334` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `362` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `370` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `381` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `382` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `385` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `387` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `388` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `389` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `391` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `393` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `401` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `407` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/merge_operator_adapter.rs` | `95` | Message-passing channel initialized |
| `contextra-engine` | `crates/contextra-engine/src/merge_operator_adapter.rs` | `139` | Message-passing channel initialized |
| `contextra-engine` | `crates/contextra-engine/src/collection/kv_lock.rs` | `51` | Potential lock inversion / deadlock: Guard 'guard' (line 51) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/kv_lock.rs` | `157` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/kv_lock.rs` | `246` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/maintenance.rs` | `170` | Potential lock inversion / deadlock: Guard '_guard' (line 170) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/maintenance.rs` | `568` | Potential lock inversion / deadlock: Guard 'now_ms' (line 556) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/maintenance.rs` | `853` | Potential lock inversion / deadlock: Guard '_guard' (line 853) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `20` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `22` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `24` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `26` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `28` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `30` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `33` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `38` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `40` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `45` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `47` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `49` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `52` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `55` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `57` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `272` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `282` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `283` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `292` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `295` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `297` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `299` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `302` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `314` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `322` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `323` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `324` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `325` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `330` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `354` | Loom permutation concurrency verification present |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `394` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `395` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `400` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `401` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `402` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs` | `403` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/search/basic.rs` | `122` | Potential lock inversion / deadlock: Guard 'guard' (line 112) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/builder.rs` | `52` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/builder.rs` | `233` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/tests.rs` | `454` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/tests.rs` | `483` | Potential lock inversion / deadlock: Guard 'updated_size' (line 471) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/crud/auto_extraction.rs` | `156` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/crud/auto_extraction.rs` | `166` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/collection/crud/insert.rs` | `38` | Potential lock inversion / deadlock: Guard 'guard' (line 28) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/crud/insert.rs` | `86` | Potential lock inversion / deadlock: Guard 'guard' (line 76) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/collection/crud/internal.rs` | `79` | Potential lock inversion / deadlock: Guard 'rx_opt' (line 59) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/transaction/db_transaction.rs` | `15` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-engine` | `crates/contextra-engine/src/transaction/db_transaction_commit.rs` | `244` | Potential lock inversion / deadlock: Guard 'hyperedgs' (line 191) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/collections.rs` | `54` | Potential lock inversion / deadlock: Guard 'read_guard' (line 54) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/collections.rs` | `165` | Potential lock inversion / deadlock: Guard 'read_guard' (line 165) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/collections.rs` | `259` | Potential lock inversion / deadlock: Guard 'guard' (line 259) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `136` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `137` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `140` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `142` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `143` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `144` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `145` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `497` | Potential lock inversion / deadlock: Guard 'cols' (line 497) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | `514` | Potential lock inversion / deadlock: Guard 'collections_read' (line 514) held across .await |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/wiring.rs` | `32` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-engine` | `crates/contextra-engine/src/background_workers/hyperedge_worker.rs` | `18` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger.rs` | `20` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1408` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1421` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/keyring.rs` | `14` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/sealer.rs` | `15` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/gate/batch.rs` | `20` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/gate/batch.rs` | `197` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/gate/batch.rs` | `198` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/gate/read_gate.rs` | `8` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/core.rs` | `268` | Potential lock inversion / deadlock: Guard 'keys' (line 268) held across .await |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/core.rs` | `270` | Potential lock inversion / deadlock: Guard 'map' (line 270) held across .await |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/core.rs` | `302` | Message-passing channel initialized |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/core.rs` | `592` | Potential lock inversion / deadlock: Guard 'map' (line 592) held across .await |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/core.rs` | `769` | Potential lock inversion / deadlock: Guard 'map' (line 769) held across .await |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/core.rs` | `864` | Potential lock inversion / deadlock: Guard 'map' (line 864) held across .await |
| `contextra-erasure` | `crates/contextra-erasure/tests/gate_linearization_test.rs` | `21` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/tests/journal_durable_spec_tests.rs` | `17` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/tests/ledger_spec_tests.rs` | `16` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-erasure` | `crates/contextra-erasure/tests/loom_gate.rs` | `8` | Loom permutation concurrency verification present |
| `contextra-erasure` | `crates/contextra-erasure/tests/loom_gate.rs` | `93` | Loom permutation concurrency verification present |
| `contextra-erasure` | `crates/contextra-erasure/tests/loom_gate.rs` | `100` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/tests/report_spec_tests.rs` | `17` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-erasure` | `crates/contextra-erasure/tests/scope_codec_spec_tests.rs` | `18` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-erasure` | `crates/contextra-erasure/tests/vault_spec_conformance.rs` | `240` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-graph` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs` | `23` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-graph` | `crates/contextra-graph/src/provenance.rs` | `15` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-graph` | `crates/contextra-graph/src/community/tests.rs` | `251` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-graph` | `crates/contextra-graph/src/community/tests.rs` | `353` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-graph` | `crates/contextra-graph/src/ppr/cost.rs` | `4` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-graph` | `crates/contextra-graph/src/ppr/tests/algo_tests.rs` | `829` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-graph` | `crates/contextra-graph/src/ppr/tests/context_tests.rs` | `13` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-graph` | `crates/contextra-graph/src/ppr/tests/context_tests.rs` | `81` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-graph` | `crates/contextra-graph/src/csr/tests/graph_index_tests.rs` | `138` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-graph` | `crates/contextra-graph/src/csr/tests/graph_index_tests.rs` | `165` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-graph` | `crates/contextra-graph/src/apprh/gate_monitor.rs` | `14` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-graph` | `crates/contextra-graph/src/apprh/selector.rs` | `18` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-graph` | `crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs` | `18` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-graph` | `crates/contextra-graph/tests/graph_persist_robustness.rs` | `13` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/attention_exporter.rs` | `6` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/embedding.rs` | `40` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/embedding.rs` | `65` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/gasp.rs` | `26` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/inference/core.rs` | `101` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/inference/core.rs` | `136` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/inference/core.rs` | `320` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-infer-candle` | `crates/contextra-infer-candle/src/inference/core.rs` | `948` | Message-passing channel initialized |
| `contextra-infer-candle` | `crates/contextra-infer-candle/tests/gguf_corruption_and_batch_boundary_test.rs` | `299` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-infer-candle` | `crates/contextra-infer-candle/tests/gguf_corruption_and_batch_boundary_test.rs` | `309` | Potential lock inversion / deadlock: Guard 'guard' (line 309) held across .await |
| `contextra-infer-candle` | `crates/contextra-infer-candle/tests/gguf_corruption_and_batch_boundary_test.rs` | `342` | Potential lock inversion / deadlock: Guard 'final_fp' (line 342) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/context_prefixer.rs` | `331` | Potential lock inversion / deadlock: Guard 'n' (line 331) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/context_prefixer.rs` | `377` | Potential lock inversion / deadlock: Guard 'n' (line 377) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/context_prefixer.rs` | `417` | Potential lock inversion / deadlock: Guard '_' (line 417) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/embedding.rs` | `12` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/embedding.rs` | `222` | Potential lock inversion / deadlock: Guard '_' (line 222) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/embedding.rs` | `265` | Potential lock inversion / deadlock: Guard '_' (line 265) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/embedding.rs` | `317` | Potential lock inversion / deadlock: Guard 'n' (line 317) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/embedding.rs` | `373` | Potential lock inversion / deadlock: Guard '_' (line 373) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/importance.rs` | `13` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/importance.rs` | `296` | Potential lock inversion / deadlock: Guard 'n' (line 296) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/importance.rs` | `352` | Potential lock inversion / deadlock: Guard '_' (line 352) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/importance.rs` | `404` | Potential lock inversion / deadlock: Guard '_' (line 404) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/importance.rs` | `440` | Potential lock inversion / deadlock: Guard '_' (line 440) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/importance.rs` | `542` | Potential lock inversion / deadlock: Guard '_' (line 542) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/model_info.rs` | `137` | Potential lock inversion / deadlock: Guard '_' (line 137) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/model_info.rs` | `214` | Potential lock inversion / deadlock: Guard '_' (line 214) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/model_info.rs` | `250` | Potential lock inversion / deadlock: Guard '_' (line 250) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/model_info.rs` | `274` | Potential lock inversion / deadlock: Guard '_' (line 274) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/model_info.rs` | `300` | Potential lock inversion / deadlock: Guard '_' (line 300) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/circuit_breaker.rs` | `2` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/policy.rs` | `184` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `29` | Potential lock inversion / deadlock: Guard '_' (line 29) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `64` | Potential lock inversion / deadlock: Guard '_' (line 64) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `106` | Potential lock inversion / deadlock: Guard '_' (line 106) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `139` | Potential lock inversion / deadlock: Guard '_' (line 139) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `298` | Potential lock inversion / deadlock: Guard '_' (line 298) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `343` | Potential lock inversion / deadlock: Guard '_' (line 343) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `520` | Potential lock inversion / deadlock: Guard '_' (line 520) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `549` | Potential lock inversion / deadlock: Guard '_' (line 549) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `585` | Potential lock inversion / deadlock: Guard 'n' (line 585) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `654` | Potential lock inversion / deadlock: Guard '_' (line 654) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `683` | Potential lock inversion / deadlock: Guard '_' (line 683) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `715` | Potential lock inversion / deadlock: Guard '_' (line 715) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `757` | Potential lock inversion / deadlock: Guard 'n' (line 757) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `808` | Potential lock inversion / deadlock: Guard '_' (line 808) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `852` | Potential lock inversion / deadlock: Guard '_' (line 852) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `901` | Potential lock inversion / deadlock: Guard '_' (line 901) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `962` | Potential lock inversion / deadlock: Guard '_' (line 962) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `992` | Potential lock inversion / deadlock: Guard '_' (line 992) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1091` | Potential lock inversion / deadlock: Guard '_' (line 1091) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1116` | Potential lock inversion / deadlock: Guard 'n' (line 1116) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1177` | Potential lock inversion / deadlock: Guard '_' (line 1177) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1237` | Potential lock inversion / deadlock: Guard '_' (line 1237) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1280` | Potential lock inversion / deadlock: Guard '_' (line 1280) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1300` | Potential lock inversion / deadlock: Guard '_' (line 1300) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1326` | Potential lock inversion / deadlock: Guard 'n' (line 1326) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1441` | Message-passing channel initialized |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1447` | Potential lock inversion / deadlock: Guard 'n' (line 1447) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1486` | Message-passing channel initialized |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1492` | Potential lock inversion / deadlock: Guard 'n' (line 1492) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1537` | Potential lock inversion / deadlock: Guard 'n' (line 1537) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/tests.rs` | `1673` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/src/client/trait_impls.rs` | `18` | Message-passing channel initialized |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/tests/campaign_stub_ollama.rs` | `19` | Potential lock inversion / deadlock: Guard '_' (line 19) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/tests/campaign_stub_ollama.rs` | `45` | Potential lock inversion / deadlock: Guard '_' (line 45) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/tests/campaign_stub_ollama.rs` | `70` | Potential lock inversion / deadlock: Guard '_' (line 70) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/tests/campaign_stub_ollama.rs` | `102` | Potential lock inversion / deadlock: Guard '_' (line 102) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/tests/campaign_stub_ollama.rs` | `128` | Potential lock inversion / deadlock: Guard '_' (line 128) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/tests/campaign_stub_ollama.rs` | `154` | Potential lock inversion / deadlock: Guard '_' (line 154) held across .await |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama/tests/campaign_stub_ollama.rs` | `182` | Potential lock inversion / deadlock: Guard '_' (line 182) held across .await |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs` | `506` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs` | `699` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs` | `1227` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `50` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `54` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `56` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `144` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `146` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `147` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `196` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `198` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `199` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `207` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `209` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `210` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` | `260` | Potential lock inversion / deadlock: Guard 'calib' (line 259) held across .await |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/onnx.rs` | `52` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/onnx.rs` | `94` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/onnx.rs` | `163` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/reranker/onnx.rs` | `197` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/tests/onnx_embedder_test.rs` | `31` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/tests/onnx_embedder_test.rs` | `54` | Potential lock inversion / deadlock: Guard '_guard' (line 38) held across .await |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/tests/onnx_embedder_test.rs` | `83` | Potential lock inversion / deadlock: Guard '_guard' (line 69) held across .await |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/tests/onnx_embedder_test.rs` | `117` | Potential lock inversion / deadlock: Guard '_guard' (line 103) held across .await |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/tests/onnx_embedder_test.rs` | `154` | Potential lock inversion / deadlock: Guard '_guard' (line 138) held across .await |
| `contextra-kvcache` | `crates/contextra-kvcache/src/eviction_worker.rs` | `20` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/eviction_worker.rs` | `87` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/eviction_worker.rs` | `88` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/eviction_worker.rs` | `107` | Message-passing channel initialized |
| `contextra-kvcache` | `crates/contextra-kvcache/src/eviction_worker.rs` | `139` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/eviction_worker.rs` | `140` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/prefix_store.rs` | `23` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/segment.rs` | `27` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/service.rs` | `44` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/src/store.rs` | `30` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/tests/eviction_segment_mapping.rs` | `14` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/tests/eviction_with_attention_exporter.rs` | `13` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-kvcache` | `crates/contextra-kvcache/tests/service_tests.rs` | `29` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mcp` | `crates/contextra-mcp/src/routing.rs` | `78` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mcp` | `crates/contextra-mcp/src/routing.rs` | `104` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mcp` | `crates/contextra-mcp/src/sandbox.rs` | `13` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mcp` | `crates/contextra-mcp/tests/tools_crud_direct_tests.rs` | `13` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-mcp` | `crates/contextra-mcp/tests/tools_crud_direct_tests.rs` | `211` | Potential lock inversion / deadlock: Guard '_guard' (line 210) held across .await |
| `contextra-mcp` | `crates/contextra-mcp/tests/tools_crud_direct_tests.rs` | `327` | Potential lock inversion / deadlock: Guard '_guard' (line 326) held across .await |
| `contextra-mvcc` | `crates/contextra-mvcc/src/snapshot.rs` | `35` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mvcc` | `crates/contextra-mvcc/src/ssi.rs` | `36` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mvcc` | `crates/contextra-mvcc/src/ssi.rs` | `512` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mvcc` | `crates/contextra-mvcc/src/ssi.rs` | `606` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mvcc` | `crates/contextra-mvcc/src/ssi.rs` | `615` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mvcc` | `crates/contextra-mvcc/src/tx_buffer.rs` | `42` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/isolation_tests.rs` | `24` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/loom_snapshot_registry.rs` | `10` | Loom permutation concurrency verification present |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/loom_snapshot_registry.rs` | `125` | Loom permutation concurrency verification present |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/ssi_metrics_observability.rs` | `15` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-ports` | `crates/contextra-ports/src/contract_tests.rs` | `17` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-ports` | `crates/contextra-ports/src/kv.rs` | `162` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-privacy` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs` | `18` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-py` | `crates/contextra-py/src/bindings/tests.rs` | `953` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-router` | `crates/contextra-router/src/router/mod.rs` | `34` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-router` | `crates/contextra-router/src/router/mod.rs` | `63` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-router` | `crates/contextra-router/src/router/mod.rs` | `71` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `10` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-router` | `crates/contextra-router/src/tests/fixtures.rs` | `337` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-router` | `crates/contextra-router/src/tests/misc_tests/part1.rs` | `102` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-router` | `crates/contextra-router/tests/router_audit_and_isolation_tests.rs` | `64` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-router` | `crates/contextra-router/tests/router_audit_and_isolation_tests.rs` | `65` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `47` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `52` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `54` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `56` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `58` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `60` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `62` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `64` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `74` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `82` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `87` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `89` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lib.rs` | `91` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/memtable.rs` | `43` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/system_pressure.rs` | `137` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/util.rs` | `12` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/util.rs` | `14` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/util.rs` | `20` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/util.rs` | `28` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `21` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `52` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `54` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `56` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `58` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `64` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `67` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `69` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `72` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/engine.rs` | `486` | Potential lock inversion / deadlock: Guard 'state' (line 486) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `182` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `184` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `185` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `191` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `193` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `194` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `202` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/lsm/observer.rs` | `210` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `17` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `497` | Potential lock inversion / deadlock: Guard 'ssts_read' (line 497) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `505` | Potential lock inversion / deadlock: Guard 'initial_wal_hmac' (line 505) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `525` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `572` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `601` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `602` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `603` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `609` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `612` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `614` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `633` | Potential lock inversion / deadlock: Guard 'wal' (line 633) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `684` | Potential lock inversion / deadlock: Guard '_commit_lock' (line 684) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `727` | Potential lock inversion / deadlock: Guard 'state' (line 727) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/recovery.rs` | `728` | Potential lock inversion / deadlock: Guard 'wal' (line 728) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/scan.rs` | `48` | Potential lock inversion / deadlock: Guard 'sstables' (line 48) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/scan.rs` | `98` | Potential lock inversion / deadlock: Guard 'state' (line 98) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/scan.rs` | `99` | Potential lock inversion / deadlock: Guard 'sstables' (line 99) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/compaction.rs` | `20` | Potential lock inversion / deadlock: Guard '_flush_lock' (line 20) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/compaction.rs` | `24` | Potential lock inversion / deadlock: Guard 'state' (line 24) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/compaction.rs` | `46` | Potential lock inversion / deadlock: Guard '_commit_lock' (line 46) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/compaction.rs` | `72` | Potential lock inversion / deadlock: Guard 'hmac' (line 72) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/compaction.rs` | `134` | Potential lock inversion / deadlock: Guard 'pending' (line 134) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/compaction.rs` | `184` | Potential lock inversion / deadlock: Guard 'wal_read' (line 184) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/maintenance.rs` | `31` | Potential lock inversion / deadlock: Guard 'state' (line 31) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/maintenance.rs` | `32` | Potential lock inversion / deadlock: Guard 'sstables' (line 32) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/read.rs` | `90` | Potential lock inversion / deadlock: Guard 'state' (line 90) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/read.rs` | `122` | Potential lock inversion / deadlock: Guard 'sstables' (line 122) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `253` | Potential lock inversion / deadlock: Guard 'commit_lock' (line 253) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `277` | Potential lock inversion / deadlock: Guard 'wal' (line 277) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `283` | Potential lock inversion / deadlock: Guard 'queue_guard' (line 283) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `417` | Potential lock inversion / deadlock: Guard 'truncate_guard' (line 417) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `464` | Potential lock inversion / deadlock: Guard 'state' (line 464) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `502` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `690` | Potential lock inversion / deadlock: Guard 'commit_lock' (line 690) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `734` | Potential lock inversion / deadlock: Guard 'truncate_guard' (line 734) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/ops/write.rs` | `799` | Potential lock inversion / deadlock: Guard 'state' (line 799) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/commit_tests.rs` | `210` | Potential lock inversion / deadlock: Guard 'wal' (line 210) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/commit_tests.rs` | `221` | Potential lock inversion / deadlock: Guard 'wal' (line 221) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/commit_tests.rs` | `487` | Potential lock inversion / deadlock: Guard 'wal' (line 487) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/commit_tests.rs` | `495` | Potential lock inversion / deadlock: Guard 'wal' (line 495) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/flush_tests.rs` | `237` | Potential lock inversion / deadlock: Guard 'state' (line 237) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/mvcc_tests.rs` | `383` | Potential lock inversion / deadlock: Guard 'commit_guard' (line 383) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `112` | Potential lock inversion / deadlock: Guard 'sstables' (line 112) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `119` | Potential lock inversion / deadlock: Guard 'sstables' (line 119) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `129` | Potential lock inversion / deadlock: Guard 'ssts' (line 129) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `178` | Potential lock inversion / deadlock: Guard 'sstables' (line 178) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `190` | Potential lock inversion / deadlock: Guard 'sstables' (line 190) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `259` | Potential lock inversion / deadlock: Guard 'sstables' (line 259) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `409` | Potential lock inversion / deadlock: Guard 'state' (line 409) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `555` | Potential lock inversion / deadlock: Guard 'sst_count_before' (line 555) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `566` | Potential lock inversion / deadlock: Guard 'sst_count_after' (line 566) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `840` | Potential lock inversion / deadlock: Guard 'ssts' (line 840) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/recovery_tests.rs` | `851` | Potential lock inversion / deadlock: Guard 'ssts' (line 851) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/scan_tests.rs` | `504` | Potential lock inversion / deadlock: Guard 'state' (line 504) held across .await |
| `contextra-store` | `crates/contextra-store/src/lsm/tests/scan_tests.rs` | `517` | Potential lock inversion / deadlock: Guard 'sstables' (line 517) held across .await |
| `contextra-store` | `crates/contextra-store/src/compaction/engine.rs` | `18` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/compaction/engine.rs` | `139` | Potential lock inversion / deadlock: Guard 'ssts' (line 139) held across .await |
| `contextra-store` | `crates/contextra-store/src/compaction/engine.rs` | `335` | Potential lock inversion / deadlock: Guard 'ssts' (line 335) held across .await |
| `contextra-store` | `crates/contextra-store/src/compaction/engine.rs` | `824` | Potential lock inversion / deadlock: Guard 'ssts' (line 824) held across .await |
| `contextra-store` | `crates/contextra-store/src/compaction/tests.rs` | `14` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/compaction/tests/advanced.rs` | `255` | Potential lock inversion / deadlock: Guard 'ssts' (line 255) held across .await |
| `contextra-store` | `crates/contextra-store/src/compaction/tests/advanced.rs` | `324` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/compaction/tests/advanced.rs` | `546` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/compaction/tests/basic.rs` | `604` | Potential lock inversion / deadlock: Guard 'ssts' (line 604) held across .await |
| `contextra-store` | `crates/contextra-store/src/compaction/tests/basic.rs` | `861` | Potential lock inversion / deadlock: Guard 'ssts_read' (line 861) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/flusher.rs` | `39` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/flusher.rs` | `174` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `636` | Potential lock inversion / deadlock: Guard 'truncate_guard' (line 636) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `658` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `673` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `679` | Potential lock inversion / deadlock: Guard 'guard' (line 666) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `701` | Potential lock inversion / deadlock: Guard 'truncate_guard' (line 701) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `723` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `738` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `745` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `748` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `788` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `789` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `796` | Potential lock inversion / deadlock: Guard 'guard' (line 781) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `828` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `834` | Potential lock inversion / deadlock: Guard 'guard' (line 821) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `845` | Potential lock inversion / deadlock: Guard '_truncate_guard' (line 845) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `861` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `867` | Potential lock inversion / deadlock: Guard 'guard' (line 855) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/io.rs` | `891` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `20` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `25` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `259` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `376` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `377` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `377` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `378` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `380` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |

*... und 146 weitere Befunde (siehe JSON-Export).*

---
*Automatisch generiert durch `concurrency_lock_auditor.py` für Agentic Engineering & Context Auditing.*