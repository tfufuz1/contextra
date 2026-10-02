# Audit-Bericht: Lokale-Umgehung-Scan für den Clock-Port

**Port**: `Clock` (`crates/contextra-ports/src/clock.rs`)
**Spezifikation**: P28 ("Injizierter Determinismus")

## Executive Summary

In Übereinstimmung mit P28 müssen alle Zeitabfragen im Produktionscode über den injizierten Clock-Port laufen, um Vorhersehbarkeit und Steuerung in Tests via FakeClock/ManualClock zu gewährleisten.
Dieser Scan wurde ohne funktionale Code-Änderungen als statisches Audit durchgeführt.

## 1. Clock-Implementierungsblöcke im Workspace

### Produktions-Implementierungen
- **`contextra-ports`** | `crates/contextra-ports/src/clock.rs:39` | `impl Clock for SystemClock {`
- **`contextra-ports`** | `crates/contextra-ports/src/clock.rs:54` | `impl<T: Clock + ?Sized> Clock for Arc<T> {`

### Test- / Fake- / Mock-Implementierungen
- **`contextra-rank`** | `crates/contextra-rank/src/calibration/isotonic.rs:286` | `impl Clock for FixedTestClock {`
- **`contextra-license`** | `crates/contextra-license/tests/check_ring_order.rs:15` | `impl Clock for TestClock {`
- **`contextra-license`** | `crates/contextra-license/tests/legacy_payload_compat.rs:19` | `impl Clock for TestClock {`
- **`contextra-license`** | `crates/contextra-license/tests/installation_binding.rs:19` | `impl Clock for TestClock {`
- **`contextra-license`** | `crates/contextra-license/tests/signed_activation_7_steps.rs:17` | `impl Clock for TestClock {`
- **`contextra-license`** | `crates/contextra-license/tests/signed_gate.rs:18` | `impl Clock for TestClock {`
- **`contextra-privacy`** | `crates/contextra-privacy/src/egress_guard.rs:324` | `impl contextra_ports::Clock for MockClock {`
- **`contextra-privacy`** | `crates/contextra-privacy/src/context_edit_audit.rs:225` | `impl Clock for FixedTestClock {`
- **`contextra-privacy`** | `crates/contextra-privacy/tests/context_edit_audit_chain.rs:29` | `impl Clock for MockSequenceClock {`
- **`contextra-privacy`** | `crates/contextra-privacy/tests/audit_trace_tests.rs:20` | `impl Clock for FixedClock {`
- **`contextra-mcp`** | `crates/contextra-mcp/src/prompt_injection/tests.rs:357` | `impl contextra_ports::Clock for FixedTestClock {`
- **`contextra-testkit`** | `crates/contextra-testkit/src/manual_clock.rs:58` | `impl Clock for ManualClock {`
- **`contextra-agent`** | `crates/contextra-agent/tests/dlq_clock_determinism_test.rs:12` | `impl Clock for TestClock {`
- **`contextra-agent`** | `crates/contextra-agent/tests/clock_tests.rs:22` | `impl Clock for TestClock {`
- **`contextra-engine`** | `crates/contextra-engine/tests/decay_and_expiry.rs:113` | `impl contextra_ports::Clock for TestClock {`
- **`contextra-cognition`** | `crates/contextra-cognition/src/maintenance_scheduler.rs:486` | `impl Clock for TestClock {`

## 2. Injizierungsstellen von Clock (dyn Clock, Arc<dyn Clock>, C: Clock)

- **`contextra-ports`** | `crates/contextra-ports/src/clock.rs:68` | `fn _assert_dyn_clock(_: Option<&dyn Clock>) {}`
- **`contextra-ports`** | `crates/contextra-ports/src/clock.rs:88` | `let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());`
- **`contextra-ports`** | `crates/contextra-ports/src/lib.rs:104` | `fn _assert_dyn_clock(_: Option<&dyn Clock>) {}`
- **`contextra-db`** | `crates/contextra-db/src/multistep.rs:69` | `clock: Arc<dyn Clock>,`
- **`contextra-db`** | `crates/contextra-db/src/multistep.rs:84` | `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-store`** | `crates/contextra-store/src/lsm/engine.rs:184` | `pub fn set_clock(&self, clock: Arc<dyn contextra_ports::Clock>) {`
- **`contextra-store`** | `crates/contextra-store/src/lsm/engine.rs:354` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-store`** | `crates/contextra-store/src/lsm/observer.rs:121` | `fn new(observer: Arc<dyn WalObserver>, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-store`** | `crates/contextra-store/src/lsm/observer.rs:181` | `clock: parking_lot::RwLock<Arc<dyn Clock>>,`
- **`contextra-store`** | `crates/contextra-store/src/lsm/observer.rs:280` | `pub fn set_clock(&self, clock: Arc<dyn Clock>) {`
- **`contextra-store`** | `crates/contextra-store/src/compaction/engine.rs:27` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-store`** | `crates/contextra-store/src/compaction/engine.rs:74` | `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self {`
- **`contextra-store`** | `crates/contextra-store/tests/wal_legacy_key_migration_store.rs:81` | `let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());`
- **`contextra-rank`** | `crates/contextra-rank/src/calibration/isotonic.rs:26` | `clock: Arc<dyn Clock>,`
- **`contextra-rank`** | `crates/contextra-rank/src/calibration/isotonic.rs:44` | `.field("clock", &"<dyn Clock>")`
- **`contextra-rank`** | `crates/contextra-rank/src/calibration/isotonic.rs:67` | `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-crypto`** | `crates/contextra-crypto/src/revocation_log.rs:120` | `clock: Arc<dyn Clock>,`
- **`contextra-crypto`** | `crates/contextra-crypto/src/revocation_log.rs:128` | `clock: Arc<dyn Clock>,`
- **`contextra-crypto`** | `crates/contextra-crypto/src/revocation_log.rs:149` | `clock: Arc<dyn Clock>,`
- **`contextra-license`** | `crates/contextra-license/src/signed_gate.rs:172` | `clock: Arc<dyn Clock>,`
- **`contextra-license`** | `crates/contextra-license/src/signed_gate.rs:207` | `clock: Arc<dyn Clock>,`
- **`contextra-license`** | `crates/contextra-license/src/signed_gate.rs:244` | `clock: Arc<dyn Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/store.rs:155` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/store.rs:373` | `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/store.rs:381` | `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:16` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:33` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:44` | `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:96` | `pub(crate) clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:103` | `let clock: Arc<dyn contextra_ports::Clock> = Arc::new(SystemClock::new());`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:111` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:169` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:196` | `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:228` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/guard.rs:637` | `let clock: Arc<dyn contextra_ports::Clock> = Arc::new(SystemClock::new());`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:46` | `pub fn monotonic_timestamp_ms_with(clock: &dyn contextra_ports::Clock) -> u64 {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:56` | `pub(crate) fn clock_timestamp_ms(clock: &dyn contextra_ports::Clock) -> u64 {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:199` | `clock: Arc<dyn contextra_ports::Clock>,`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:209` | `.field("clock", &"<dyn Clock>")`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:226` | `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:231` | `pub fn set_clock(&mut self, clock: Arc<dyn contextra_ports::Clock>) {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:235` | `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> {`
- **`contextra-checkpoint`** | `crates/contextra-checkpoint/src/orphan.rs:240` | `let clock: Arc<dyn contextra_ports::Clock> = Arc::new(SystemClock::new());`
- **`contextra-privacy`** | `crates/contextra-privacy/src/context_edit_audit.rs:134` | `clock: &dyn Clock,`
- **`contextra-privacy`** | `crates/contextra-privacy/src/audit_trace.rs:57` | `clock: &'a dyn Clock,`
- **`contextra-privacy`** | `crates/contextra-privacy/src/audit_trace.rs:65` | `clock: &'a dyn Clock,`
- **`contextra-mcp`** | `crates/contextra-mcp/src/prompt_injection/guard.rs:39` | `clock: Arc<dyn Clock>,`
- **`contextra-mcp`** | `crates/contextra-mcp/src/prompt_injection/guard.rs:96` | `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-mcp`** | `crates/contextra-mcp/src/prompt_injection/guard.rs:101` | `pub fn clock(&self) -> &Arc<dyn Clock> {`
- **`contextra-testkit`** | `crates/contextra-testkit/src/manual_clock.rs:99` | `let dyn_clock: &dyn Clock = &clock;`
- **`contextra-agent`** | `crates/contextra-agent/src/engine.rs:40` | `pub clock: Arc<dyn Clock>,`
- **`contextra-agent`** | `crates/contextra-agent/src/engine.rs:56` | `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-agent`** | `crates/contextra-agent/src/clm_scratchpad.rs:195` | `clock: Arc<dyn Clock>,`
- **`contextra-agent`** | `crates/contextra-agent/src/clm_scratchpad.rs:241` | `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-engine`** | `crates/contextra-engine/src/collection/mod.rs:291` | `pub(super) clock: parking_lot::RwLock<Arc<dyn Clock>>,`
- **`contextra-engine`** | `crates/contextra-engine/src/collection/mod.rs:422` | `pub fn with_clock(self, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-engine`** | `crates/contextra-engine/src/collection/mod.rs:428` | `pub fn set_clock(&self, clock: Arc<dyn Clock>) {`
- **`contextra-engine`** | `crates/contextra-engine/src/collection/mod.rs:433` | `pub fn clock(&self) -> Arc<dyn Clock> {`
- **`contextra-cognition`** | `crates/contextra-cognition/src/maintenance_scheduler.rs:29` | `clock: Arc<dyn Clock>,`
- **`contextra-cognition`** | `crates/contextra-cognition/src/maintenance_scheduler.rs:51` | `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {`
- **`contextra-cognition`** | `crates/contextra-cognition/src/maintenance_scheduler.rs:299` | `clock: &dyn Clock,`
- **`contextra-cognition`** | `crates/contextra-cognition/src/maintenance_scheduler.rs:323` | `clock: &dyn Clock,`

## 3., 4. & 5. Direkte Zeitzugriffe im Workspace

Crate | Datei:Zeile | Kontext (Funktion) | Clock lokal verfügbar? | Einstufung
--- | --- | --- | --- | ---
contextra-ports | crates/contextra-ports/src/clock.rs:28 | `pub fn new() -> Self` | nein | legitime Quelle
contextra-ports | crates/contextra-ports/src/clock.rs:41 | `fn now_unix_nanos(&self) -> u64` | ja | legitime Quelle
contextra-ports | crates/contextra-ports/src/clock.rs:48 | `fn monotonic_nanos(&self) -> u64` | ja | legitime Quelle
contextra-db | crates/contextra-db/src/volatile_vault.rs:236 | `pub fn purge(mut self) -> PurgeReceipt` | nein | lokale Umgehung
contextra-db | crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs:56 | `fn resolve_and_log_seed() -> u64` | nein | legitime Quelle (Test/Bench)
contextra-db | crates/contextra-db/tests/backpressure_test.rs:31 | `async fn test_insert_backpressure_when_pressure_critical()` | nein | legitime Quelle (Test/Bench)
contextra-db | crates/contextra-db/tests/backpressure_test.rs:52 | `async fn test_insert_backpressure_when_pressure_critical()` | nein | legitime Quelle (Test/Bench)
contextra-db | crates/contextra-db/tests/backpressure_test.rs:73 | `async fn test_insert_backpressure_when_pressure_critical()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/src/system_pressure.rs:163 | `pub async fn run_with_metrics_sink(` | nein | lokale Umgehung
contextra-store | crates/contextra-store/src/system_pressure.rs:343 | `async fn test_blocking_pool_load_triggers_pressure()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/src/lsm/ops/write.rs:209 | `unknown` | nein | lokale Umgehung
contextra-store | crates/contextra-store/src/lsm/tests/mvcc_tests.rs:221 | `async fn test_concurrent_get_and_flush_latency()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/src/lsm/tests/mvcc_tests.rs:480 | `async fn test_put_if_absent_no_commit_mutex_hold()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/src/lsm/tests/commit_tests.rs:319 | `async fn test_system_pressure_wal_queue_backpressure_transition()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/src/compaction/engine.rs:514 | `fn cmp(&self, other: &Self) -> std::cmp::Ordering` | nein | lokale Umgehung
contextra-store | crates/contextra-store/src/compaction/engine.rs:543 | `fn cmp(&self, other: &Self) -> std::cmp::Ordering` | nein | lokale Umgehung
contextra-store | crates/contextra-store/src/compaction/engine.rs:668 | `fn cmp(&self, other: &Self) -> std::cmp::Ordering` | nein | lokale Umgehung
contextra-store | crates/contextra-store/src/compaction/engine.rs:746 | `fn cmp(&self, other: &Self) -> std::cmp::Ordering` | nein | lokale Umgehung
contextra-store | crates/contextra-store/src/compaction/tests/advanced.rs:457 | `async fn test_compaction_pressure_awareness()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/src/compaction/tests/advanced.rs:482 | `async fn test_compaction_pressure_awareness()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/src/wal/flusher.rs:238 | `pub(crate) fn enable_flusher_with_config(` | nein | lokale Umgehung
contextra-store | crates/contextra-store/src/sstable/tests.rs:878 | `async fn test_block_cache_32_concurrent_readers_hotset_latency()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/fuzz/fuzz_targets/fuzz_memtable_concurrent.rs:99 | `unknown` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/benches/redb_comparative_bench.rs:100 | `fn bench_sequential_write(c: &mut Criterion)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/benches/redb_comparative_bench.rs:135 | `fn bench_sequential_write(c: &mut Criterion)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/benches/redb_comparative_bench.rs:472 | `fn bench_mixed_workload(c: &mut Criterion)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/benches/redb_comparative_bench.rs:531 | `fn bench_mixed_workload(c: &mut Criterion)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/wal_observer_test.rs:147 | `async fn test_slow_observer_deregistered_fail_open()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/group_commit_test.rs:178 | `async fn test_group_commit_micro_benchmark_throughput_comparison()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/group_commit_test.rs:211 | `async fn test_group_commit_micro_benchmark_throughput_comparison()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/observer_blocking_failopen.rs:74 | `async fn test_sleeping_observer_does_not_delay_commits()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/observer_blocking_failopen.rs:194 | `async fn test_shutdown_drop_does_not_block()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/group_commit_stress.rs:25 | `async fn test_group_commit_concurrency_stress_200_tasks()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/group_commit_stress.rs:99 | `async fn run_latency_benchmark(` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/executor_starvation_test.rs:39 | `async fn run_starvation_benchmark(test_name: &str, num_entries: usize, payload_size: usize)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/executor_starvation_test.rs:70 | `async fn run_starvation_benchmark(test_name: &str, num_entries: usize, payload_size: usize)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/executor_starvation_test.rs:96 | `async fn run_starvation_benchmark(test_name: &str, num_entries: usize, payload_size: usize)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/blocking_util_pool_measurement.rs:73 | `async fn test_blocking_util_increases_under_pool_load()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/wal_recovery_runtime_stall.rs:26 | `fn new() -> Self` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/wal_recovery_runtime_stall.rs:38 | `fn on_entry_replayed(&self, _seq: WalSeq, _entries_total_estimate: Option<u64>)` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/wal_observer_integration.rs:228 | `async fn test_observer_fail_open_timeout_and_latency_unaffected()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/fsync_syscall_verification.rs:48 | `async fn test_measure_fsync_overhead_benchmark()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/compaction_io_rate_limit.rs:38 | `async fn test_token_bucket_rate_limits_io_throughput()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/compaction_io_rate_limit.rs:39 | `async fn test_token_bucket_rate_limits_io_throughput()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/compaction_io_rate_limit.rs:53 | `async fn test_token_bucket_rate_limits_io_throughput()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/observer_deadline_no_runtime_block.rs:45 | `async fn test_observer_notify_does_not_block_tokio_runtime_thread()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/wal_observer.rs:146 | `async fn test_observer_fail_open_timeout_deregistration()` | nein | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/wal_observer_contract.rs:97 | `async fn test_blocking_observer_no_preemptive_timeout_and_eviction_on_return()` | ja | legitime Quelle (Test/Bench)
contextra-store | crates/contextra-store/tests/metrics_sink_test.rs:145 | `async fn test_metrics_sink_records_commit_events()` | nein | legitime Quelle (Test/Bench)
contextra-sandbox | crates/contextra-sandbox/src/executor.rs:161 | `pub async fn execute(` | nein | lokale Umgehung
contextra-sandbox | crates/contextra-sandbox/src/executor.rs:218 | `pub async fn execute(` | ja | lokale Umgehung
contextra-sandbox | crates/contextra-sandbox/src/executor.rs:598 | `fn test_memory_growing_div_ceil_boundary() -> TestResult` | ja | legitime Quelle (Test/Bench)
contextra-sandbox | crates/contextra-sandbox/src/executor.rs:625 | `fn test_table_growing_limit() -> TestResult` | ja | legitime Quelle (Test/Bench)
contextra-infer-candle | crates/contextra-infer-candle/src/inference/tests.rs:525 | `async fn test_inference_backpressure_single_permit_awaits()` | nein | legitime Quelle (Test/Bench)
contextra-crypto | crates/contextra-crypto/src/kv_segment/eviction_worker.rs:114 | `fn test_eviction_worker_nonblocking_trigger()` | nein | legitime Quelle (Test/Bench)
contextra-crypto | crates/contextra-crypto/tests/kv_segment_concurrency.rs:178 | `fn test_adr082_rollback_om_complexity_not_retain()` | nein | legitime Quelle (Test/Bench)
contextra-crypto | crates/contextra-crypto/tests/eviction_worker_tokio_runtime_nonblocking_regression.rs:38 | `fn eviction_worker_drop_does_not_starve_other_tasks_indefinitely()` | nein | legitime Quelle (Test/Bench)
contextra-crypto | crates/contextra-crypto/tests/eviction_worker_tokio_runtime_nonblocking_regression.rs:44 | `fn eviction_worker_drop_does_not_starve_other_tasks_indefinitely()` | nein | legitime Quelle (Test/Bench)
contextra-checkpoint | crates/contextra-checkpoint/src/store.rs:430 | `pub async fn create_checkpoint(` | ja | lokale Umgehung
contextra-checkpoint | crates/contextra-checkpoint/src/guard.rs:179 | `pub fn with_registry_counter_and_clock(` | ja | lokale Umgehung
contextra-checkpoint | crates/contextra-checkpoint/src/guard.rs:647 | `async fn checkpoint_guard_CASE_rollback_consumed_returns_err()` | ja | legitime Quelle (Test/Bench)
contextra-checkpoint | crates/contextra-checkpoint/src/guard.rs:662 | `async fn checkpoint_guard_CASE_rollback_consumed_returns_err()` | ja | legitime Quelle (Test/Bench)
contextra-checkpoint | crates/contextra-checkpoint/src/guard.rs:677 | `async fn checkpoint_guard_CASE_rollback_consumed_returns_err()` | ja | legitime Quelle (Test/Bench)
contextra-checkpoint | crates/contextra-checkpoint/tests/guard_exit_paths.rs:486 | `async fn test_checkpoint_guard_drop_latency_sub_millisecond()` | nein | legitime Quelle (Test/Bench)
contextra-checkpoint | crates/contextra-checkpoint/tests/guard_exit_paths.rs:518 | `fn test_concurrent_50_parallel_guard_drops_no_worker_blocking()` | nein | legitime Quelle (Test/Bench)
contextra-checkpoint | crates/contextra-checkpoint/tests/guard_exit_paths.rs:531 | `fn test_concurrent_50_parallel_guard_drops_no_worker_blocking()` | nein | legitime Quelle (Test/Bench)
contextra-checkpoint | crates/contextra-checkpoint/tests/guard_exit_paths.rs:577 | `async fn test_pin_guard_drop_latency_sub_millisecond()` | nein | legitime Quelle (Test/Bench)
contextra-privacy | crates/contextra-privacy/src/bulk_exfiltration_detector.rs:147 | `pub fn record_and_check(` | nein | lokale Umgehung
contextra-privacy | crates/contextra-privacy/src/bulk_exfiltration_detector.rs:208 | `fn test_sliding_window_expiry_resets_volume()` | nein | legitime Quelle (Test/Bench)
contextra-privacy | crates/contextra-privacy/src/bulk_exfiltration_detector.rs:291 | `fn test_time_regression_fails_closed()` | nein | legitime Quelle (Test/Bench)
contextra-privacy | crates/contextra-privacy/src/egress_vault.rs:637 | `async fn test_redos_and_timeout_fail_closed()` | nein | legitime Quelle (Test/Bench)
contextra-mcp | crates/contextra-mcp/src/prompt_injection/tests.rs:226 | `fn test_detect_recursive_precomputed_patterns_performance()` | nein | legitime Quelle (Test/Bench)
contextra-mcp | crates/contextra-mcp/tests/mcp_test.rs:1231 | `async fn test_slowloris_stdio_attack_simulation()` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/src/bandit_regret_tests.rs:206 | `fn test_bandit_diagonal_vs_linucb_latency_budget()` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/src/router/outcomes.rs:82 | `pub(super) fn evict_stale_decisions(&self)` | nein | lokale Umgehung
contextra-router | crates/contextra-router/src/router/dispatch_core.rs:162 | `pub async fn route(` | nein | lokale Umgehung
contextra-router | crates/contextra-router/src/router/dispatch_core.rs:173 | `pub async fn route(` | nein | lokale Umgehung
contextra-router | crates/contextra-router/src/router/tests.rs:157 | `fn test_pending_bandit_eviction_unit()` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/src/tests/misc_tests/part2.rs:312 | `async fn test_pending_decisions_evicted_after_ttl() -> Result<(), Box<dyn std::error::Error>>` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/src/tests/misc_tests/part2.rs:368 | `async fn test_pending_decisions_max_capacity_enforced() -> Result<(), Box<dyn std::error::Error>>` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/src/tests/routing_tests/engine_part2.rs:620 | `async fn test_router_engine_additional_coverage_paths() -> contextra_types::Result<()>` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/src/tests/routing_tests/engine_part2.rs:634 | `async fn test_router_engine_additional_coverage_paths() -> contextra_types::Result<()>` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/tests/bandit_latency.rs:34 | `fn test_bandit_sherman_morrison_latency_budget()` | nein | legitime Quelle (Test/Bench)
contextra-router | crates/contextra-router/tests/bandit_latency.rs:65 | `fn test_bandit_sherman_morrison_latency_budget()` | nein | legitime Quelle (Test/Bench)
contextra-mvcc | crates/contextra-mvcc/src/tx_buffer.rs:302 | `pub fn begin(&self, tx: TxId)` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/tx_buffer.rs:426 | `pub fn stage_bounded(&self, tx: TxId, op: IndexOp<T>) -> Result<()>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/tx_buffer.rs:473 | `pub fn stage_many(&self, tx: TxId, ops: impl IntoIterator<Item = IndexOp<T>>) -> Result<()>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/tx_buffer.rs:574 | `pub fn reap_orphans(&self) -> Vec<TxId>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/tx_buffer.rs:622 | `pub fn reap_orphans_bounded(&self, max: usize) -> Vec<TxId>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/tx_buffer.rs:992 | `fn test_tx_buffer_reap_orphans_deterministic_time_injection()` | nein | legitime Quelle (Test/Bench)
contextra-mvcc | crates/contextra-mvcc/src/snapshot.rs:63 | `pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/snapshot.rs:97 | `pub fn pin(&self, seq_no: u64)` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/snapshot.rs:164 | `pub fn longest_active_pin(&self) -> Option<(u64, Duration)>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/ssi.rs:599 | `fn check_and_coarsen_if_needed(&self, writes: &mut CommittedWrites)` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/ssi.rs:618 | `pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/seq_log.rs:130 | `pub fn pin_snapshot(&mut self, seq_no: u64)` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/seq_log.rs:171 | `pub fn expired_pins(&self) -> Vec<u64>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/seq_log.rs:204 | `pub fn min_retention_seq(&self) -> Option<u64>` | nein | lokale Umgehung
contextra-mvcc | crates/contextra-mvcc/src/seq_log.rs:553 | `fn test_sequence_log_pin_ttl_and_expiration()` | nein | legitime Quelle (Test/Bench)
contextra-mvcc | crates/contextra-mvcc/src/seq_log.rs:595 | `fn test_sequence_log_pin_ttl_custom_duration()` | nein | legitime Quelle (Test/Bench)
contextra-mvcc | crates/contextra-mvcc/src/seq_log.rs:609 | `fn test_sequence_log_zero_panic_time_arithmetic()` | nein | legitime Quelle (Test/Bench)
contextra-mvcc | crates/contextra-mvcc/tests/b16_reproduction.rs:72 | `fn test_long_lived_snapshot_pin_blocker_alarm()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/vertical_path_ssi_under_write_load.rs:88 | `async fn vertical_path_test_3_ssi_under_write_load_b16() -> Result<(), Box<dyn std::error::Error>>` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_enforcement.rs:17 | `async fn test_compliance_profile_without_gate_fails()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_enforcement.rs:52 | `async fn test_compliance_profile_with_explicit_open_fast_gate_fails()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_enforcement.rs:88 | `async fn test_builder_without_performance_profile_succeeds()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_enforcement.rs:125 | `async fn test_compliance_profile_with_allow_all_gate_succeeds()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_always_enforced.rs:15 | `async fn open_without_gate_never_grants_sovereign()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_always_enforced.rs:33 | `async fn compliance_profile_without_signed_license_returns_error()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_always_enforced.rs:66 | `async fn compliance_profile_with_valid_signed_license_activates_deletion_proof()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_always_enforced.rs:102 | `async fn expired_signature_rejected()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_always_enforced.rs:143 | `async fn tampered_signature_rejected()` | nein | legitime Quelle (Test/Bench)
contextra | crates/contextra/tests/license_gate_always_enforced.rs:184 | `async fn explicit_conflicting_user_value_to_profile_returns_policy_violation()` | nein | legitime Quelle (Test/Bench)
contextra-kvcache | crates/contextra-kvcache/src/attention_score.rs:187 | `fn test_null_attention_score_source_preserves_pure_lru()` | nein | legitime Quelle (Test/Bench)
contextra-kvcache | crates/contextra-kvcache/src/attention_score.rs:203 | `fn test_attention_score_overrides_lru_order_when_weighted()` | nein | legitime Quelle (Test/Bench)
contextra-kvcache | crates/contextra-kvcache/src/attention_score.rs:226 | `fn test_rank_empty_and_single_candidate()` | nein | legitime Quelle (Test/Bench)
contextra-kvcache | crates/contextra-kvcache/src/attention_score.rs:232 | `fn test_v5_numerical_stability_edge_cases()` | nein | legitime Quelle (Test/Bench)
contextra-kvcache | crates/contextra-kvcache/src/store.rs:109 | `fn insert_returning_evicted_with_directive(` | nein | lokale Umgehung
contextra-kvcache | crates/contextra-kvcache/src/store.rs:177 | `fn pop_eviction_candidate(` | nein | lokale Umgehung
contextra-kvcache | crates/contextra-kvcache/src/store.rs:210 | `fn pop_eviction_candidate(` | nein | lokale Umgehung
contextra-kvcache | crates/contextra-kvcache/src/eviction_worker.rs:224 | `fn test_eviction_worker_nonblocking_trigger()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/src/cascade.rs:721 | `async fn test_cascade_invalidate_hyperedges_high_fanout()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/src/ppr/tests/context_tests.rs:168 | `async fn test_ppr_pathological_max_iterations_ceiling()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/src/ppr/tests/context_tests.rs:760 | `async fn test_forward_push_performance_benchmark_vs_dense()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/src/ppr/tests/context_tests.rs:764 | `async fn test_forward_push_performance_benchmark_vs_dense()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/src/csr/tests/graph_index_tests.rs:498 | `async fn test_hub_node_1m_neighbors_bfs_capped() -> contextra_types::Result<()>` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/src/csr/tests/bitemporal_tests.rs:114 | `async fn test_rcu_concurrent_readers_never_block_during_compaction()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/hub_node_benchmark.rs:68 | `async fn test_hub_node_bfs_scaling_benchmark()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/hyperedge_cascade_fanout.rs:33 | `async fn test_hyperedge_cascade_fanout_limit_and_termination()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/csr_complexity_bench.rs:57 | `async fn bench_single_compaction_scaling()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/csr_complexity_bench.rs:98 | `async fn bench_amortized_1m_edge_inserts()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/csr_complexity_bench.rs:104 | `async fn bench_amortized_1m_edge_inserts()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/csr_benchmark.rs:40 | `async fn test_csr_delta_buffer_incremental_benchmark()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/csr_benchmark.rs:89 | `async fn test_csr_delta_buffer_incremental_benchmark()` | nein | legitime Quelle (Test/Bench)
contextra-graph | crates/contextra-graph/tests/csr_benchmark.rs:154 | `async fn test_add_edge_median_latency_10k_nodes_100k_edges()` | nein | legitime Quelle (Test/Bench)
contextra-infer-ollama | crates/contextra-infer-ollama/src/client/tests.rs:925 | `async fn test_retry_on_500_and_503_with_backoff_timing()` | nein | legitime Quelle (Test/Bench)
contextra-adapt | crates/contextra-adapt/src/homeostat.rs:213 | `fn test_rerank_deadline_exceeded()` | nein | legitime Quelle (Test/Bench)
contextra-adapt | crates/contextra-adapt/src/pid_latency_controller.rs:165 | `pub fn new(budget_ms: f64) -> Self` | nein | lokale Umgehung
contextra-adapt | crates/contextra-adapt/src/bandit.rs:895 | `fn bench_sherman_morrison_p95_latency()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/src/tokenizer.rs:378 | `fn test_trie_caching_performance_bug_txt_002()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/src/inverted/tests/search_tests.rs:699 | `async fn test_high_frequency_term_resident_index_performance() -> Result<()>` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/src/inverted/tests/search_tests.rs:708 | `async fn test_high_frequency_term_resident_index_performance() -> Result<()>` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/src/morphology/tests.rs:717 | `fn fuzz_german_compound_splitter_utf8_panic_free_10k()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/rca_investigation.rs:23 | `fn test_rca_investigation_full_suite()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/rca_investigation.rs:61 | `fn test_rca_investigation_full_suite()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/rca_investigation.rs:108 | `fn test_rca_investigation_full_suite()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/rca_investigation.rs:159 | `fn test_rca_investigation_full_suite()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/domain_vocab_quality.rs:250 | `fn test_domain_vocabulary_cold_start_loading_performance()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/alloc_profiler.rs:260 | `async fn run_profile_10k_documents()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/alloc_profiler.rs:310 | `async fn run_profile_10k_documents()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/alloc_profiler.rs:359 | `async fn run_profile_10k_documents()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/alloc_profiler.rs:404 | `async fn run_profile_10k_documents()` | nein | legitime Quelle (Test/Bench)
contextra-text | crates/contextra-text/tests/alloc_profiler.rs:455 | `async fn run_profile_10k_documents()` | nein | legitime Quelle (Test/Bench)
contextra-agent | crates/contextra-agent/src/context.rs:469 | `async fn test_telemetry_100k_insertions_and_amortized_performance(` | nein | legitime Quelle (Test/Bench)
contextra-agent | crates/contextra-agent/src/context.rs:480 | `async fn test_telemetry_100k_insertions_and_amortized_performance(` | nein | legitime Quelle (Test/Bench)
contextra-agent | crates/contextra-agent/tests/system_prompt_pinning_latency.rs:30 | `async fn test_system_prompt_pinning_latency_under_100_micros() -> contextra_types::Result<()>` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/src/hnsw/core_rebuild.rs:28 | `pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | nein | lokale Umgehung
contextra-vector | crates/contextra-vector/src/hnsw/core_rebuild.rs:30 | `pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | nein | lokale Umgehung
contextra-vector | crates/contextra-vector/src/hnsw/core_rebuild.rs:460 | `pub fn rebuild_sync(&self) -> Result<()>` | nein | lokale Umgehung
contextra-vector | crates/contextra-vector/src/hnsw/core_rebuild.rs:530 | `pub fn rebuild_sync(&self) -> Result<()>` | nein | lokale Umgehung
contextra-vector | crates/contextra-vector/benches/flush_threshold_amplification.rs:99 | `async fn run_benchmark_for_config(` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:60 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:66 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:76 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:82 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:92 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:98 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:147 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:200 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:320 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:360 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:421 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/audit_benchmarks.rs:452 | `async fn main()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/benches/filtered_search_selectivity_bench.rs:142 | `fn bench_filtered_search_selectivity(c: &mut Criterion)` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/tests/deleted_nodes_lock_contention.rs:180 | `async fn proof_search_throughput_not_degraded_by_deletes()` | nein | legitime Quelle (Test/Bench)
contextra-vector | crates/contextra-vector/tests/deleted_nodes_lock_contention.rs:187 | `async fn proof_search_throughput_not_degraded_by_deletes()` | nein | legitime Quelle (Test/Bench)
contextra-engine | crates/contextra-engine/src/collection/maintenance.rs:181 | `async fn repair_internal(&self) -> Result<()>` | ja | lokale Umgehung
contextra-engine | crates/contextra-engine/src/collection/search/hybrid/query.rs:132 | `pub async fn hybrid_search_with_query_and_report_at(` | nein | lokale Umgehung
contextra-engine | crates/contextra-engine/src/collection/search/hybrid/query.rs:230 | `pub async fn hybrid_search_with_query_and_report_at(` | nein | lokale Umgehung
contextra-engine | crates/contextra-engine/src/collection/search/hybrid/query.rs:344 | `pub async fn hybrid_search_with_query_and_report_at(` | nein | lokale Umgehung
contextra-engine | crates/contextra-engine/src/collection/query_builder/builder_exec.rs:172 | `pub async fn execute_with_report(` | nein | lokale Umgehung
contextra-engine | crates/contextra-engine/src/contextra_impl/lifecycle.rs:242 | `async fn repair_on_open_internal(&self) -> Result<()>` | nein | lokale Umgehung
contextra-engine | crates/contextra-engine/src/background_workers/orphan_workers.rs:88 | `pub fn start_orphan_cleanup_worker_with_config<` | nein | lokale Umgehung
contextra-engine | crates/contextra-engine/src/background_workers/tests.rs:531 | `async fn trigger_expiry_cleanup_deletes_expired_documents()` | nein | legitime Quelle (Test/Bench)
contextra-cognition | crates/contextra-cognition/tests/transitivity_veto_tests.rs:179 | `fn test_p24_locality_bounded_runtime_on_large_graph()` | nein | legitime Quelle (Test/Bench)