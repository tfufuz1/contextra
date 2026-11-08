# Concurrency & Lock Safety Audit Report

> **Werkzeug:** `concurrency_lock_auditor.py`  
> **Erstellungszeit:** `2026-11-07T23:01:10.599082+00:00`  
> **Status/Befunde:** `216 Befunde erfasst`  

## Beschreibung
Audits synchronization primitives, Mutex/RwLock guards across .await (INV-LOCK-ORDER-1), channel architectures, and Loom model-checking support.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `locks_held_across_await` | `77` |
| `std_sync_locks` | `15` |
| `parking_lot_locks` | `41` |
| `tokio_sync_locks` | `29` |
| `channels_used` | `26` |
| `loom_verification_points` | `28` |
| `total_concurrency_items` | `216` |

## Synchronisations-Übersicht
| Mechanismus / Risiko | Anzahl | Relevanz |
|---|---|---|
| **Lock über `.await`** | 77 | Kritische Deadlock-Gefahr / Async-Invariante |
| `std::sync::{Mutex, RwLock}` | 15 | Standardbibliothek Sperren |
| `parking_lot::{Mutex, RwLock}` | 41 | Schnelle synchrone Locks |
| `tokio::sync::{Mutex, RwLock}` | 29 | Kooperative asynchrone Sperren |
| Kanäle (`mpsc`/`broadcast`) | 26 | Message-Passing Architekturen |
| `loom` Test-Konfigurationen | 28 | Deterministische Nebenläufigkeitsprüfung |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
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
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `587` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `588` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `589` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `591` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `692` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `693` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `694` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `696` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `909` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `923` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `924` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `925` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | `927` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/wal/tests/flusher_tests.rs` | `212` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/tests/flusher_tests.rs` | `213` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/tests/flusher_tests.rs` | `222` | Potential lock inversion / deadlock: Guard 'guard' (line 208) held across .await |
| `contextra-store` | `crates/contextra-store/src/wal/tests/flusher_tests.rs` | `379` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/src/wal/tests/flusher_tests.rs` | `387` | Potential lock inversion / deadlock: Guard 'flusher_tx' (line 380) held across .await |
| `contextra-store` | `crates/contextra-store/src/manifest/core.rs` | `29` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/manifest/core.rs` | `100` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/src/sstable/block_cache.rs` | `8` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/src/engine/memory_only.rs` | `6` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/fuzz/fuzz_targets/fuzz_memtable_concurrent.rs` | `52` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/benches/memtable_bench.rs` | `8` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/amplification_benchmark.rs` | `17` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/tests/amplification_benchmark.rs` | `25` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/tests/blocking_util_pool_measurement.rs` | `7` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/campaign_fuzzregress_memtable_concurrent.rs` | `92` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/tests/campaign_j21_hypotheses.rs` | `113` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/tests/campaign_j21_hypotheses.rs` | `156` | Potential lock inversion / deadlock: Guard 'expected_val' (line 156) held across .await |
| `contextra-store` | `crates/contextra-store/tests/chaos_memory_pressure.rs` | `164` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/tests/chaos_memory_pressure.rs` | `195` | Potential lock inversion / deadlock: Guard 'gt_keys' (line 195) held across .await |
| `contextra-store` | `crates/contextra-store/tests/chaos_task_massacre.rs` | `8` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/chaos_task_massacre.rs` | `126` | Potential lock inversion / deadlock: Guard 'confirmed' (line 120) held across .await |
| `contextra-store` | `crates/contextra-store/tests/commit_rollback_batch_scoped.rs` | `12` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/tests/commit_rollback_batch_scoped.rs` | `16` | Potential lock inversion / deadlock: Guard '_guard' (line 16) held across .await |
| `contextra-store` | `crates/contextra-store/tests/commit_rollback_batch_scoped.rs` | `74` | Potential lock inversion / deadlock: Guard '_guard' (line 74) held across .await |
| `contextra-store` | `crates/contextra-store/tests/compaction_contiguous_selection.rs` | `49` | Potential lock inversion / deadlock: Guard 'sstables_guard' (line 49) held across .await |
| `contextra-store` | `crates/contextra-store/tests/executor_starvation_test.rs` | `33` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/group_commit_timeout.rs` | `19` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/tests/group_commit_timeout.rs` | `23` | Potential lock inversion / deadlock: Guard '_lock_guard' (line 23) held across .await |
| `contextra-store` | `crates/contextra-store/tests/group_commit_timeout.rs` | `112` | Potential lock inversion / deadlock: Guard '_lock_guard' (line 112) held across .await |
| `contextra-store` | `crates/contextra-store/tests/loom_commit_flush_visibility.rs` | `6` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/tests/loom_commit_flush_visibility.rs` | `56` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit.rs` | `7` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit.rs` | `72` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit_handoff.rs` | `12` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit_handoff.rs` | `95` | Loom permutation concurrency verification present |
| `contextra-store` | `crates/contextra-store/tests/metrics_sink_test.rs` | `246` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `85` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `86` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `87` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `116` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `117` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `118` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `121` | Potential lock inversion / deadlock: Guard 'guard' (line 121) held across .await |
| `contextra-store` | `crates/contextra-store/tests/wal_backpressure.rs` | `136` | Potential lock inversion / deadlock: Guard 'guard' (line 136) held across .await |
| `contextra-store` | `crates/contextra-store/tests/wal_legacy_key_tests.rs` | `222` | Potential lock inversion / deadlock: Guard 'captured_logs' (line 208) held across .await |
| `contextra-store` | `crates/contextra-store/tests/wal_observer.rs` | `12` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer.rs` | `19` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer.rs` | `30` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer.rs` | `62` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer.rs` | `91` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer.rs` | `122` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer.rs` | `167` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_contract.rs` | `85` | Message-passing channel initialized |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `14` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `23` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `36` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `50` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `63` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `109` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `125` | Potential lock inversion / deadlock: Guard 'recs' (line 117) held across .await |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `134` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `167` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `206` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `207` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `254` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `255` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_integration.rs` | `295` | Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking) |
| `contextra-store` | `crates/contextra-store/tests/wal_observer_test.rs` | `133` | Potential lock inversion / deadlock: Guard 'order' (line 130) held across .await |
| `contextra-store` | `crates/contextra-store/tests/wal_recovery_runtime_stall.rs` | `10` | Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async) |
| `contextra-store` | `crates/contextra-store/tests/wal_torn_write_fault_injection.rs` | `7` | Usage of tokio::sync::{Mutex, RwLock} for async boundaries |
| `contextra-store` | `crates/contextra-store/tests/wal_torn_write_fault_injection.rs` | `11` | Potential lock inversion / deadlock: Guard '_guard' (line 11) held across .await |
| `contextra-store` | `crates/contextra-store/tests/wal_torn_write_fault_injection.rs` | `83` | Potential lock inversion / deadlock: Guard '_guard' (line 83) held across .await |

---
*Automatisch generiert durch `concurrency_lock_auditor.py` für Agentic Engineering & Context Auditing.*