use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStatsMicros {
    pub p50: f64,
    pub p99: f64,
    pub p99_9: f64,
    pub mean: f64,
    pub stddev: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub latency: LatencyStatsMicros,
    pub throughput_ops_sec: f64,
    pub total_ops: u64,
    pub wall_time_secs: f64,
    pub bytes_written_logical: u64,
    pub bytes_written_disk: u64,
    pub write_amplification: f64,
    pub peak_rss_bytes: u64,
    pub open_fd_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub latency_p50_micros_mean: f64,
    pub latency_p50_micros_stddev: f64,
    pub latency_p99_micros_mean: f64,
    pub latency_p99_micros_stddev: f64,
    pub latency_p99_9_micros_mean: f64,
    pub latency_p99_9_micros_stddev: f64,
    pub throughput_ops_sec_mean: f64,
    pub throughput_ops_sec_stddev: f64,
    pub wall_time_secs_mean: f64,
    pub wall_time_secs_stddev: f64,
    pub write_amplification_mean: f64,
    pub write_amplification_stddev: f64,
    pub peak_rss_bytes_mean: f64,
    pub open_fd_count_max: usize,
    pub runs: Vec<RunMetrics>,
}

pub struct BenchmarkRecorder {
    histogram: Histogram<u64>,
}

impl BenchmarkRecorder {
    pub fn new() -> Self {
        // Record latency in nanoseconds up to 10 minutes (600_000_000_000 ns) with 3 sig figs
        let histogram = Histogram::<u64>::new_with_bounds(1, 600_000_000_000, 3)
            .unwrap_or_else(|_| Histogram::<u64>::new(3).unwrap());
        Self { histogram }
    }

    pub fn record_nanos(&mut self, duration_nanos: u64) {
        let value = duration_nanos.max(1);
        let _ = self.histogram.record(value);
    }

    pub fn finish_run(
        &self,
        total_ops: u64,
        wall_time_secs: f64,
        logical_bytes: u64,
        disk_bytes: u64,
        peak_rss_bytes: u64,
        open_fd_count: usize,
    ) -> RunMetrics {
        let p50 = self.histogram.value_at_quantile(0.50) as f64 / 1_000.0;
        let p99 = self.histogram.value_at_quantile(0.99) as f64 / 1_000.0;
        let p99_9 = self.histogram.value_at_quantile(0.999) as f64 / 1_000.0;
        let mean = self.histogram.mean() / 1_000.0;
        let stddev = self.histogram.stdev() / 1_000.0;

        let throughput_ops_sec = if wall_time_secs > 0.0 {
            (total_ops as f64) / wall_time_secs
        } else {
            0.0
        };

        let write_amplification = if logical_bytes > 0 {
            (disk_bytes as f64) / (logical_bytes as f64)
        } else {
            0.0
        };

        RunMetrics {
            latency: LatencyStatsMicros {
                p50,
                p99,
                p99_9,
                mean,
                stddev,
            },
            throughput_ops_sec,
            total_ops,
            wall_time_secs,
            bytes_written_logical: logical_bytes,
            bytes_written_disk: disk_bytes,
            write_amplification,
            peak_rss_bytes,
            open_fd_count,
        }
    }
}

pub fn aggregate_runs(runs: Vec<RunMetrics>) -> AggregatedMetrics {
    if runs.is_empty() {
        return AggregatedMetrics {
            latency_p50_micros_mean: 0.0,
            latency_p50_micros_stddev: 0.0,
            latency_p99_micros_mean: 0.0,
            latency_p99_micros_stddev: 0.0,
            latency_p99_9_micros_mean: 0.0,
            latency_p99_9_micros_stddev: 0.0,
            throughput_ops_sec_mean: 0.0,
            throughput_ops_sec_stddev: 0.0,
            wall_time_secs_mean: 0.0,
            wall_time_secs_stddev: 0.0,
            write_amplification_mean: 0.0,
            write_amplification_stddev: 0.0,
            peak_rss_bytes_mean: 0.0,
            open_fd_count_max: 0,
            runs: vec![],
        };
    }

    let p50s: Vec<f64> = runs.iter().map(|r| r.latency.p50).collect();
    let p99s: Vec<f64> = runs.iter().map(|r| r.latency.p99).collect();
    let p99_9s: Vec<f64> = runs.iter().map(|r| r.latency.p99_9).collect();
    let tps: Vec<f64> = runs.iter().map(|r| r.throughput_ops_sec).collect();
    let walls: Vec<f64> = runs.iter().map(|r| r.wall_time_secs).collect();
    let wamp: Vec<f64> = runs.iter().map(|r| r.write_amplification).collect();
    let rsses: Vec<f64> = runs.iter().map(|r| r.peak_rss_bytes as f64).collect();

    let (p50_m, p50_s) = mean_stddev(&p50s);
    let (p99_m, p99_s) = mean_stddev(&p99s);
    let (p99_9_m, p99_9_s) = mean_stddev(&p99_9s);
    let (tp_m, tp_s) = mean_stddev(&tps);
    let (wall_m, wall_s) = mean_stddev(&walls);
    let (wamp_m, wamp_s) = mean_stddev(&wamp);
    let (rss_m, _) = mean_stddev(&rsses);

    let max_fds = runs.iter().map(|r| r.open_fd_count).max().unwrap_or(0);

    AggregatedMetrics {
        latency_p50_micros_mean: p50_m,
        latency_p50_micros_stddev: p50_s,
        latency_p99_micros_mean: p99_m,
        latency_p99_micros_stddev: p99_s,
        latency_p99_9_micros_mean: p99_9_m,
        latency_p99_9_micros_stddev: p99_9_s,
        throughput_ops_sec_mean: tp_m,
        throughput_ops_sec_stddev: tp_s,
        wall_time_secs_mean: wall_m,
        wall_time_secs_stddev: wall_s,
        write_amplification_mean: wamp_m,
        write_amplification_stddev: wamp_s,
        peak_rss_bytes_mean: rss_m,
        open_fd_count_max: max_fds,
        runs,
    }
}

fn mean_stddev(vals: &[f64]) -> (f64, f64) {
    if vals.is_empty() {
        return (0.0, 0.0);
    }
    let n = vals.len() as f64;
    let mean = vals.iter().sum::<f64>() / n;
    if vals.len() == 1 {
        return (mean, 0.0);
    }
    let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, variance.sqrt())
}
