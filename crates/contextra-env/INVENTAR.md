## contextra-env

Pfad: `crates/contextra-env`

Abhängigkeiten (direkt): contextra-ids, tracing

### Module

- `/src/clock.rs`
- `/src/env.rs`
- `/src/id_gen.rs`
- `/src/lib.rs`
- `/src/manual_clock.rs`
- `/src/metrics.rs`
- `/src/rng.rs`
- `/src/seq_log.rs`

### Öffentliche Typen

- **Clock** (trait) – src/clock.rs:18
- **GraphClock** (trait) – tests/legacy_reference_unix_ms.rs:18
- **HistogramSnapshot** (struct) – src/metrics.rs:131
- **IdGen** (trait) – src/id_gen.rs:14
- **InMemoryMetricsSink** (struct) – src/metrics.rs:242
- **ManualClock** (struct) – src/manual_clock.rs:29
- **MetricKey** (struct) – src/metrics.rs:97
- **MetricSnapshot** (struct) – src/metrics.rs:147
- **MetricsSink** (trait) – src/metrics.rs:26
- **NoopMetricsSink** (struct) – src/metrics.rs:74
- **Rng** (trait) – src/rng.rs:39
- **SeededRng** (struct) – src/rng.rs:105
- **SeqLogChange** (enum) – src/seq_log.rs:104
- **SeqLogEntry** (struct) – src/seq_log.rs:58
- **SequenceLog** (struct) – src/seq_log.rs:151
- **SequentialIdGen** (struct) – src/id_gen.rs:39
- **SystemClock** (struct) – src/clock.rs:54
- **TextClock** (trait) – tests/legacy_reference_unix_ms.rs:13
- **VectorClock** (trait) – tests/legacy_reference_unix_ms.rs:23

### Öffentliche Funktionen

- `pub fn advance_millis(&self, ms: u64) `  – src/manual_clock.rs:56
- `pub fn advance_nanos(&self, delta: u64) `  – src/manual_clock.rs:48
- `pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange> `  – src/seq_log.rs:484
- `pub fn clock(&self) -> &Arc<dyn Clock> `  – src/seq_log.rs:223
- `pub fn compact(&mut self, min_active_seqno: u64) `  – src/seq_log.rs:450
- `pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64> `  – src/seq_log.rs:365
- `pub fn doc_id(&self) -> DocId `  – src/seq_log.rs:130
- `pub fn expired_pins(&self) -> Vec<u64> `  – src/seq_log.rs:308
- `pub fn expired_pins_at(&self, now_nanos: u64) -> Vec<u64> `  – src/seq_log.rs:292
- `pub fn from_entropy() -> Self `  – src/rng.rs:131
- `pub fn is_empty(&self) -> bool `  – src/seq_log.rs:474
- `pub fn is_visible(&self, as_of: u64) -> bool `  – src/seq_log.rs:89
- `pub fn is_visible(&self, doc_id: DocId, seq_no: u64) -> bool `  – src/seq_log.rs:418
- `pub fn len(&self) -> usize `  – src/seq_log.rs:469
- `pub fn max_pin_duration(&self) -> Duration `  – src/seq_log.rs:244
- `pub fn min_retention_seq(&self) -> Option<u64> `  – src/seq_log.rs:360
- `pub fn min_retention_seq_at(&self, now_nanos: u64) -> Option<u64> `  – src/seq_log.rs:337
- `pub fn new() -> Self `  – src/clock.rs:66
- `pub fn new() -> Self `  – src/metrics.rs:250
- `pub fn new() -> Self `  – src/seq_log.rs:188
- `pub fn new(name: impl Into<String>, labels: &[(&str, &str)]) -> Self `  – src/metrics.rs:108
- `pub fn pin_snapshot(&mut self, seq_no: u64) `  – src/seq_log.rs:255
- `pub fn pin_snapshot_at(&mut self, seq_no: u64, at_nanos: u64) `  – src/seq_log.rs:261
- `pub fn reap_expired_pins(&mut self) -> Vec<u64> `  – src/seq_log.rs:327
- `pub fn reap_expired_pins_at(&mut self, now_nanos: u64) -> Vec<u64> `  – src/seq_log.rs:318
- `pub fn record_delete(&mut self, doc_id: DocId, seq: u64) `  – src/seq_log.rs:401
- `pub fn record_insert(&mut self, doc_id: DocId, seq: u64) `  – src/seq_log.rs:378
- `pub fn seq(&self) -> u64 `  – src/seq_log.rs:123
- `pub fn set_clock(&mut self, clock: Arc<dyn Clock>) `  – src/seq_log.rs:228
- `pub fn set_max_pin_duration(&mut self, duration: Duration) `  – src/seq_log.rs:239
- `pub fn set_monotonic_nanos(&self, nanos: u64) `  – src/manual_clock.rs:66
- `pub fn set_unix_nanos(&self, nanos: u64) `  – src/manual_clock.rs:61
- `pub fn snapshot(&self) -> MetricSnapshot `  – src/metrics.rs:255
- `pub fn unix_ms(c: &dyn Clock) -> u64 `  – src/lib.rs:84
- `pub fn unpin_snapshot(&mut self, seq_no: u64) `  – src/seq_log.rs:275
- `pub fn with_clock(clock: Arc<dyn Clock>) -> Self `  – src/seq_log.rs:194
- `pub fn with_max_pin_duration(mut self, duration: Duration) -> Self `  – src/seq_log.rs:233
- `pub fn with_time_fn<F>(time_fn: F) -> Self`  – src/seq_log.rs:206

---
