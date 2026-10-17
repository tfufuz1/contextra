use crate::harness::{ContextraHarness, EngineHarness, RedbHarness};
use crate::metrics::{aggregate_runs, AggregatedMetrics, BenchmarkRecorder};
use crate::system_info::{get_dir_size_bytes, get_open_fd_count, get_peak_rss_bytes};
use anyhow::Result;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy)]
pub struct WorkloadConfig {
    pub prefill_keys: usize,
    pub value_bytes: usize,
    pub runs: usize,
    pub warmup_ops: usize,
    pub is_smoke: bool,
}

impl Default for WorkloadConfig {
    fn default() -> Self {
        Self {
            prefill_keys: 1_000_000,
            value_bytes: 128,
            runs: 5,
            warmup_ops: 10_000,
            is_smoke: false,
        }
    }
}

impl WorkloadConfig {
    pub fn smoke() -> Self {
        Self {
            prefill_keys: 5_000,
            value_bytes: 128,
            runs: 2,
            warmup_ops: 100,
            is_smoke: true,
        }
    }
}

pub enum EngineType {
    Contextra,
    Redb,
}

impl EngineType {
    pub fn create_harness(&self, dir: &Path, rt: Arc<Runtime>) -> Result<Box<dyn EngineHarness>> {
        match self {
            EngineType::Contextra => {
                let harness = ContextraHarness::open_sync(dir, rt)?;
                Ok(Box::new(harness))
            }
            EngineType::Redb => {
                let harness = RedbHarness::open(dir)?;
                Ok(Box::new(harness))
            }
        }
    }
}

fn make_key(id: usize) -> Vec<u8> {
    format!("k_{:012}", id).into_bytes()
}

fn make_value(id: usize, size: usize) -> Vec<u8> {
    let mut val = Vec::with_capacity(size);
    let id_bytes = id.to_le_bytes();
    while val.len() < size {
        let remaining = size - val.len();
        let chunk = if remaining >= id_bytes.len() {
            &id_bytes[..]
        } else {
            &id_bytes[..remaining]
        };
        val.extend_from_slice(chunk);
    }
    val
}

fn prefill_engine(
    harness: &dyn EngineHarness,
    count: usize,
    val_bytes: usize,
    batch_size: usize,
) -> Result<u64> {
    let mut total_logical_bytes = 0u64;

    if batch_size == 1 {
        for i in 0..count {
            let k = make_key(i);
            let v = make_value(i, val_bytes);
            total_logical_bytes += (k.len() + v.len()) as u64;
            harness.put(&k, &v)?;
        }
    } else {
        let mut idx = 0;
        while idx < count {
            let end = (idx + batch_size).min(count);
            let keys: Vec<Vec<u8>> = (idx..end).map(make_key).collect();
            let vals: Vec<Vec<u8>> = (idx..end).map(|i| make_value(i, val_bytes)).collect();
            let items: Vec<(&[u8], &[u8])> = keys
                .iter()
                .zip(vals.iter())
                .map(|(k, v)| (k.as_slice(), v.as_slice()))
                .collect();

            for (k, v) in &items {
                total_logical_bytes += (k.len() + v.len()) as u64;
            }

            harness.put_batch(&items)?;
            idx = end;
        }
    }

    Ok(total_logical_bytes)
}

// Workload 1: Sequential Writes
pub fn run_seq_write(
    engine_type: &EngineType,
    base_dir: &Path,
    config: &WorkloadConfig,
    batch_size: usize,
    rt: Arc<Runtime>,
) -> Result<AggregatedMetrics> {
    let mut run_metrics = Vec::new();
    let write_count = if config.is_smoke { 1_000 } else { 50_000 };

    for run_idx in 0..config.runs {
        let run_dir = base_dir.join(format!("seq_write_b{}_{}", batch_size, run_idx));
        std::fs::create_dir_all(&run_dir)?;

        let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
        let mut recorder = BenchmarkRecorder::new();

        let mut total_logical = 0u64;
        let start_time = Instant::now();

        if batch_size == 1 {
            for i in 0..write_count {
                let k = make_key(i);
                let v = make_value(i, config.value_bytes);
                total_logical += (k.len() + v.len()) as u64;

                let t0 = Instant::now();
                harness.put(&k, &v)?;
                recorder.record_nanos(t0.elapsed().as_nanos() as u64);
            }
        } else {
            let mut idx = 0;
            while idx < write_count {
                let end = (idx + batch_size).min(write_count);
                let keys: Vec<Vec<u8>> = (idx..end).map(make_key).collect();
                let vals: Vec<Vec<u8>> = (idx..end).map(|i| make_value(i, config.value_bytes)).collect();
                let items: Vec<(&[u8], &[u8])> = keys
                    .iter()
                    .zip(vals.iter())
                    .map(|(k, v)| (k.as_slice(), v.as_slice()))
                    .collect();

                for (k, v) in &items {
                    total_logical += (k.len() + v.len()) as u64;
                }

                let t0 = Instant::now();
                harness.put_batch(&items)?;
                recorder.record_nanos(t0.elapsed().as_nanos() as u64);
                idx = end;
            }
        }

        harness.flush()?;
        let elapsed = start_time.elapsed().as_secs_f64();
        let disk_bytes = get_dir_size_bytes(&run_dir);
        let rss = get_peak_rss_bytes();
        let fds = get_open_fd_count();

        harness.close()?;

        let ops = if batch_size == 1 {
            write_count as u64
        } else {
            ((write_count + batch_size - 1) / batch_size) as u64
        };

        run_metrics.push(recorder.finish_run(ops, elapsed, total_logical, disk_bytes, rss, fds));
        let _ = std::fs::remove_dir_all(&run_dir);
    }

    Ok(aggregate_runs(run_metrics))
}

// Workload 2: Random Reads (Warm & Cold Cache)
pub fn run_random_read(
    engine_type: &EngineType,
    base_dir: &Path,
    config: &WorkloadConfig,
    is_cold: bool,
    rt: Arc<Runtime>,
) -> Result<AggregatedMetrics> {
    let mut run_metrics = Vec::new();
    let read_ops = if config.is_smoke { 1_000 } else { 10_000 };

    for run_idx in 0..config.runs {
        let run_dir = base_dir.join(format!(
            "rand_read_{}_{}",
            if is_cold { "cold" } else { "warm" },
            run_idx
        ));
        std::fs::create_dir_all(&run_dir)?;

        // 1. Prefill
        {
            let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
            prefill_engine(harness.as_ref(), config.prefill_keys, config.value_bytes, 1000)?;
            harness.flush()?;
            harness.close()?;
        }

        // Reopen engine
        let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;

        // Seeded RNG for reproducible random read keys
        let mut rng = StdRng::seed_from_u64(42 + run_idx as u64);

        if !is_cold {
            // Warmup
            for _ in 0..config.warmup_ops.min(config.prefill_keys) {
                let id = rng.gen_range(0..config.prefill_keys);
                let k = make_key(id);
                let _ = harness.get(&k)?;
            }
        }

        let mut recorder = BenchmarkRecorder::new();
        let start_time = Instant::now();

        for _ in 0..read_ops {
            let id = rng.gen_range(0..config.prefill_keys);
            let k = make_key(id);

            let t0 = Instant::now();
            let res = harness.get(&k)?;
            recorder.record_nanos(t0.elapsed().as_nanos() as u64);
            assert!(res.is_some(), "Key must exist during random read test");
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let disk_bytes = get_dir_size_bytes(&run_dir);
        let rss = get_peak_rss_bytes();
        let fds = get_open_fd_count();

        harness.close()?;

        run_metrics.push(recorder.finish_run(read_ops as u64, elapsed, 0, disk_bytes, rss, fds));
        let _ = std::fs::remove_dir_all(&run_dir);
    }

    Ok(aggregate_runs(run_metrics))
}

// Workload 3: Range Scan
pub fn run_range_scan(
    engine_type: &EngineType,
    base_dir: &Path,
    config: &WorkloadConfig,
    rt: Arc<Runtime>,
) -> Result<AggregatedMetrics> {
    let mut run_metrics = Vec::new();
    let scan_ops = if config.is_smoke { 100 } else { 1_000 };
    let scan_limit = 100;

    for run_idx in 0..config.runs {
        let run_dir = base_dir.join(format!("range_scan_{}", run_idx));
        std::fs::create_dir_all(&run_dir)?;

        {
            let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
            prefill_engine(harness.as_ref(), config.prefill_keys, config.value_bytes, 1000)?;
            harness.flush()?;
            harness.close()?;
        }

        let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
        let mut rng = StdRng::seed_from_u64(100 + run_idx as u64);

        let mut recorder = BenchmarkRecorder::new();
        let start_time = Instant::now();

        for _ in 0..scan_ops {
            let max_start = config.prefill_keys.saturating_sub(scan_limit);
            let id = if max_start > 0 { rng.gen_range(0..max_start) } else { 0 };
            let start_key = make_key(id);

            let t0 = Instant::now();
            let res = harness.range_scan(&start_key, scan_limit)?;
            recorder.record_nanos(t0.elapsed().as_nanos() as u64);
            assert!(!res.is_empty(), "Range scan should return entries");
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let disk_bytes = get_dir_size_bytes(&run_dir);
        let rss = get_peak_rss_bytes();
        let fds = get_open_fd_count();

        harness.close()?;

        run_metrics.push(recorder.finish_run(scan_ops as u64, elapsed, 0, disk_bytes, rss, fds));
        let _ = std::fs::remove_dir_all(&run_dir);
    }

    Ok(aggregate_runs(run_metrics))
}

// Workload 4: Mixed 50/50 Read/Write
pub fn run_mixed_50_50(
    engine_type: &EngineType,
    base_dir: &Path,
    config: &WorkloadConfig,
    rt: Arc<Runtime>,
) -> Result<AggregatedMetrics> {
    let mut run_metrics = Vec::new();
    let total_ops = if config.is_smoke { 1_000 } else { 10_000 };

    for run_idx in 0..config.runs {
        let run_dir = base_dir.join(format!("mixed_50_50_{}", run_idx));
        std::fs::create_dir_all(&run_dir)?;

        {
            let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
            prefill_engine(harness.as_ref(), config.prefill_keys, config.value_bytes, 1000)?;
            harness.flush()?;
            harness.close()?;
        }

        let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
        let mut rng = StdRng::seed_from_u64(200 + run_idx as u64);

        let mut recorder = BenchmarkRecorder::new();
        let mut next_new_id = config.prefill_keys;
        let mut total_logical = 0u64;

        let start_time = Instant::now();

        for _ in 0..total_ops {
            let is_write: bool = rng.gen_bool(0.5);
            if is_write {
                let k = make_key(next_new_id);
                let v = make_value(next_new_id, config.value_bytes);
                next_new_id += 1;
                total_logical += (k.len() + v.len()) as u64;

                let t0 = Instant::now();
                harness.put(&k, &v)?;
                recorder.record_nanos(t0.elapsed().as_nanos() as u64);
            } else {
                let id = rng.gen_range(0..next_new_id);
                let k = make_key(id);

                let t0 = Instant::now();
                let _ = harness.get(&k)?;
                recorder.record_nanos(t0.elapsed().as_nanos() as u64);
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let disk_bytes = get_dir_size_bytes(&run_dir);
        let rss = get_peak_rss_bytes();
        let fds = get_open_fd_count();

        harness.close()?;

        run_metrics.push(recorder.finish_run(total_ops as u64, elapsed, total_logical, disk_bytes, rss, fds));
        let _ = std::fs::remove_dir_all(&run_dir);
    }

    Ok(aggregate_runs(run_metrics))
}

// Workload 5a: Recovery Open Time (after N writes)
pub fn run_recovery_open(
    engine_type: &EngineType,
    base_dir: &Path,
    config: &WorkloadConfig,
    rt: Arc<Runtime>,
) -> Result<AggregatedMetrics> {
    let mut run_metrics = Vec::new();

    for run_idx in 0..config.runs {
        let run_dir = base_dir.join(format!("recovery_open_{}", run_idx));
        std::fs::create_dir_all(&run_dir)?;

        {
            let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
            prefill_engine(harness.as_ref(), config.prefill_keys, config.value_bytes, 1000)?;
            harness.flush()?;
            harness.close()?;
        }

        let mut recorder = BenchmarkRecorder::new();
        let start_time = Instant::now();

        let t0 = Instant::now();
        let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
        recorder.record_nanos(t0.elapsed().as_nanos() as u64);

        let elapsed = start_time.elapsed().as_secs_f64();
        let disk_bytes = get_dir_size_bytes(&run_dir);
        let rss = get_peak_rss_bytes();
        let fds = get_open_fd_count();

        harness.close()?;

        run_metrics.push(recorder.finish_run(1, elapsed, 0, disk_bytes, rss, fds));
        let _ = std::fs::remove_dir_all(&run_dir);
    }

    Ok(aggregate_runs(run_metrics))
}

// Workload 5b: Recovery 1,000 Open/Close Cycles
pub fn run_open_close_cycles(
    engine_type: &EngineType,
    base_dir: &Path,
    config: &WorkloadConfig,
    rt: Arc<Runtime>,
) -> Result<AggregatedMetrics> {
    let mut run_metrics = Vec::new();
    let cycles = if config.is_smoke { 50 } else { 1_000 };

    for run_idx in 0..config.runs {
        let run_dir = base_dir.join(format!("open_close_cycles_{}", run_idx));
        std::fs::create_dir_all(&run_dir)?;

        {
            let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
            prefill_engine(harness.as_ref(), 100, config.value_bytes, 10)?;
            harness.flush()?;
            harness.close()?;
        }

        let mut recorder = BenchmarkRecorder::new();
        let start_time = Instant::now();

        for _ in 0..cycles {
            let t0 = Instant::now();
            let harness = engine_type.create_harness(&run_dir, Arc::clone(&rt))?;
            harness.close()?;
            recorder.record_nanos(t0.elapsed().as_nanos() as u64);
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let disk_bytes = get_dir_size_bytes(&run_dir);
        let rss = get_peak_rss_bytes();
        let fds = get_open_fd_count();

        run_metrics.push(recorder.finish_run(cycles as u64, elapsed, 0, disk_bytes, rss, fds));
        let _ = std::fs::remove_dir_all(&run_dir);
    }

    Ok(aggregate_runs(run_metrics))
}
