# Diagnose-Report: `memfuse-db` (check)

Fehler: **1**  |  Warnungen: **8**

## `crates/memfuse-db/../../benches/relate_bench.rs`  (1 Diagnose(n))

- **ERROR [E0599]** @ `crates/memfuse-db/../../benches/relate_bench.rs:70:24`: no method named `relate_bidirectional` found for struct `MemFuse` in the current scope

## `crates/memfuse-db/src/memory_consolidation.rs`  (2 Diagnose(n))

- **WARNING [deprecated]** @ `crates/memfuse-db/src/memory_consolidation.rs:388:68`: use of deprecated method `memfuse_core::DocId::as_u64`: Nutze inner()
  - _Hinweis:_ `#[warn(deprecated)]` on by default
- **WARNING [deprecated]** @ `crates/memfuse-db/src/memory_consolidation.rs:388:68`: use of deprecated method `memfuse_core::DocId::as_u64`: Nutze inner()
  - _Hinweis:_ `#[warn(deprecated)]` on by default

## `crates/memfuse-db/tests/consolidation_integration_test.rs`  (2 Diagnose(n))

- **WARNING [deprecated]** @ `crates/memfuse-db/tests/consolidation_integration_test.rs:5:67`: use of deprecated function `memfuse_db::start_consolidation_worker`: Konsolidiert in MaintenanceScheduler — siehe maintenance_scheduler.rs. Wird nach Migrationsfrist entfernt.
  - _Hinweis:_ `#[warn(deprecated)]` on by default
- **WARNING [deprecated]** @ `crates/memfuse-db/tests/consolidation_integration_test.rs:82:18`: use of deprecated function `memfuse_db::start_consolidation_worker`: Konsolidiert in MaintenanceScheduler — siehe maintenance_scheduler.rs. Wird nach Migrationsfrist entfernt.

## `crates/memfuse-db/tests/deletion_proof_integration.rs`  (4 Diagnose(n))

- **WARNING [unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:233:9`: unused variable: `lsm_proof_1`
  - _Hinweis:_ `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default
  - _Hinweis:_ if this is intentional, prefix it with an underscore
- **WARNING [unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:238:9`: unused variable: `sstable_proof_1`
  - _Hinweis:_ if this is intentional, prefix it with an underscore
- **WARNING [unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:258:9`: unused variable: `lsm_proof_2`
  - _Hinweis:_ if this is intentional, prefix it with an underscore
- **WARNING [unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:263:9`: unused variable: `sstable_proof_2`
  - _Hinweis:_ if this is intentional, prefix it with an underscore
