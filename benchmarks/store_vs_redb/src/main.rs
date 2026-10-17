mod harness;
mod metrics;
mod system_info;
mod workloads;

use anyhow::Result;
use clap::Parser;
use metrics::AggregatedMetrics;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use system_info::SystemMetadata;
use workloads::{
    run_mixed_50_50, run_open_close_cycles, run_random_read, run_range_scan, run_recovery_open,
    run_seq_write, EngineType, WorkloadConfig,
};

#[derive(Parser, Debug)]
#[command(name = "store_vs_redb", about = "Contextra vs redb benchmark suite")]
struct Args {
    #[arg(long, help = "Run fast smoke test profile under 60 seconds")]
    smoke: bool,

    #[arg(long, help = "Evaluate decision F5 threshold (Contextra / redb <= 1.5x)")]
    verdict: bool,

    #[arg(long, default_value = "1000000", help = "Number of keys to prefill")]
    prefill_keys: usize,

    #[arg(long, default_value = "128", help = "Value size in bytes")]
    value_bytes: usize,

    #[arg(long, default_value = "5", help = "Number of benchmark runs")]
    runs: usize,

    #[arg(long, default_value = "10000", help = "Warmup operations for warm read benchmark")]
    warmup: usize,

    #[arg(long, default_value = "bench_data", help = "Target directory for benchmark database files")]
    data_dir: PathBuf,

    #[arg(long, help = "Path to write machine-readable JSON results")]
    output_json: Option<PathBuf>,

    #[arg(long, help = "Path to write Markdown summary table")]
    output_md: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkloadResult {
    workload_name: String,
    contextra: AggregatedMetrics,
    redb: AggregatedMetrics,
    ratio_p50: f64,
    ratio_p99: f64,
    ratio_throughput: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkReport {
    system_metadata: SystemMetadata,
    durability_config: String,
    workloads: Vec<WorkloadResult>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config = if args.smoke {
        WorkloadConfig::smoke()
    } else {
        WorkloadConfig {
            prefill_keys: args.prefill_keys,
            value_bytes: args.value_bytes,
            runs: args.runs,
            warmup_ops: args.warmup,
            is_smoke: false,
        }
    };

    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?,
    );

    let base_dir = args.data_dir;
    if base_dir.exists() {
        let _ = fs::remove_dir_all(&base_dir);
    }
    fs::create_dir_all(&base_dir)?;

    let sys_metadata = SystemMetadata::collect_for_path(&base_dir);

    println!("============================================================");
    println!(" Contextra vs redb Storage Benchmark Suite");
    println!(" Mode: {}", if args.smoke { "SMOKE TEST" } else { "FULL BENCHMARK" });
    println!(" Commit Hash: {}", sys_metadata.commit_hash);
    println!(" CPU: {}", sys_metadata.cpu_model);
    println!(" Kernel: {}", sys_metadata.kernel_version);
    println!(" Filesystem: {} ({})", sys_metadata.filesystem, sys_metadata.mount_options);
    println!(" Durability Mode: Equal Durability (fsync per commit)");
    println!("   - Contextra: durability_mode=Full, group_commit_window_micros=0");
    println!("   - redb: Durability::Immediate");
    println!("============================================================");

    let mut workloads_to_run: Vec<(&str, Box<dyn Fn(&EngineType) -> Result<AggregatedMetrics>>)> = Vec::new();

    // 1. Sequential Writes (batches 1, 100, 5000)
    for b in [1, 100, 5000] {
        let name = Box::leak(format!("seq_write_batch_{}", b).into_boxed_str());
        let dir = base_dir.clone();
        let rt_clone = Arc::clone(&rt);
        workloads_to_run.push((
            name,
            Box::new(move |eng| run_seq_write(eng, &dir, &config, b, Arc::clone(&rt_clone))),
        ));
    }

    // 2. Random Read Cold
    {
        let dir = base_dir.clone();
        let rt_clone = Arc::clone(&rt);
        workloads_to_run.push((
            "random_read_cold",
            Box::new(move |eng| run_random_read(eng, &dir, &config, true, Arc::clone(&rt_clone))),
        ));
    }

    // 3. Random Read Warm
    {
        let dir = base_dir.clone();
        let rt_clone = Arc::clone(&rt);
        workloads_to_run.push((
            "random_read_warm",
            Box::new(move |eng| run_random_read(eng, &dir, &config, false, Arc::clone(&rt_clone))),
        ));
    }

    // 4. Range Scan
    {
        let dir = base_dir.clone();
        let rt_clone = Arc::clone(&rt);
        workloads_to_run.push((
            "range_scan_100_keys",
            Box::new(move |eng| run_range_scan(eng, &dir, &config, Arc::clone(&rt_clone))),
        ));
    }

    // 5. Mixed 50/50
    {
        let dir = base_dir.clone();
        let rt_clone = Arc::clone(&rt);
        workloads_to_run.push((
            "mixed_50_50_read_write",
            Box::new(move |eng| run_mixed_50_50(eng, &dir, &config, Arc::clone(&rt_clone))),
        ));
    }

    // 6. Recovery Open
    {
        let dir = base_dir.clone();
        let rt_clone = Arc::clone(&rt);
        workloads_to_run.push((
            "recovery_open_time",
            Box::new(move |eng| run_recovery_open(eng, &dir, &config, Arc::clone(&rt_clone))),
        ));
    }

    // 7. Open/Close Cycles
    {
        let dir = base_dir.clone();
        let rt_clone = Arc::clone(&rt);
        workloads_to_run.push((
            "open_close_cycles_1000",
            Box::new(move |eng| run_open_close_cycles(eng, &dir, &config, Arc::clone(&rt_clone))),
        ));
    }

    let mut results = Vec::new();

    for (name, runner) in workloads_to_run {
        println!("\nRunning Workload: {}", name);

        print!("  -> Contextra ... ");
        let ctx_metrics = runner(&EngineType::Contextra)?;
        println!(
            "p50: {:.1}µs, p99: {:.1}µs, throughput: {:.1} ops/s",
            ctx_metrics.latency_p50_micros_mean,
            ctx_metrics.latency_p99_micros_mean,
            ctx_metrics.throughput_ops_sec_mean
        );

        print!("  -> redb ... ");
        let redb_metrics = runner(&EngineType::Redb)?;
        println!(
            "p50: {:.1}µs, p99: {:.1}µs, throughput: {:.1} ops/s",
            redb_metrics.latency_p50_micros_mean,
            redb_metrics.latency_p99_micros_mean,
            redb_metrics.throughput_ops_sec_mean
        );

        let ratio_p50 = if redb_metrics.latency_p50_micros_mean > 0.0 {
            ctx_metrics.latency_p50_micros_mean / redb_metrics.latency_p50_micros_mean
        } else {
            0.0
        };

        let ratio_p99 = if redb_metrics.latency_p99_micros_mean > 0.0 {
            ctx_metrics.latency_p99_micros_mean / redb_metrics.latency_p99_micros_mean
        } else {
            0.0
        };

        let ratio_throughput = if ctx_metrics.throughput_ops_sec_mean > 0.0 {
            redb_metrics.throughput_ops_sec_mean / ctx_metrics.throughput_ops_sec_mean
        } else {
            0.0
        };

        println!(
            "  -> Ratio (Contextra / redb): Latency p50={:.2}x, p99={:.2}x",
            ratio_p50, ratio_p99
        );

        results.push(WorkloadResult {
            workload_name: name.to_string(),
            contextra: ctx_metrics,
            redb: redb_metrics,
            ratio_p50,
            ratio_p99,
            ratio_throughput,
        });
    }

    let report = BenchmarkReport {
        system_metadata: sys_metadata.clone(),
        durability_config: "Contextra: durability_mode=Full, group_commit_window_micros=0; redb: Durability::Immediate".to_string(),
        workloads: results,
    };

    let md_table = generate_markdown_table(&report);

    println!("\n============================================================");
    println!(" Benchmark Summary Table");
    println!("============================================================");
    println!("{}", md_table);

    if let Some(json_path) = args.output_json {
        let json_str = serde_json::to_string_pretty(&report)?;
        fs::write(&json_path, json_str)?;
        println!("JSON report written to {:?}", json_path);
    }

    if let Some(md_path) = args.output_md {
        fs::write(&md_path, &md_table)?;
        println!("Markdown report written to {:?}", md_path);
    }

    let _ = fs::remove_dir_all(&base_dir);

    if args.verdict {
        println!("\n============================================================");
        println!(" Decision F5 Verdict Evaluation (Threshold <= 1.5x redb)");
        println!("============================================================");

        let mut pass = true;

        for w in &report.workloads {
            let is_critical = w.workload_name.contains("random_read")
                || w.workload_name.contains("range_scan")
                || w.workload_name.contains("seq_write");

            if is_critical {
                let status_p99 = if w.ratio_p99 <= 1.5 { "PASS" } else { "FAIL" };
                println!(
                    "  [{}] Workload '{}' p99 Latency Ratio: {:.2}x (Threshold 1.50x)",
                    status_p99, w.workload_name, w.ratio_p99
                );
                if w.ratio_p99 > 1.5 {
                    pass = false;
                }
            }
        }

        if pass {
            println!("\nVERDICT: PASS");
        } else {
            println!("\nVERDICT: FAIL");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn generate_markdown_table(report: &BenchmarkReport) -> String {
    let mut out = String::new();
    out.push_str("# Performance Baseline: Contextra vs redb\n\n");
    out.push_str(&format!("**Commit Hash:** `{}`  \n", report.system_metadata.commit_hash));
    out.push_str(&format!("**CPU Model:** {}  \n", report.system_metadata.cpu_model));
    out.push_str(&format!("**Kernel:** {}  \n", report.system_metadata.kernel_version));
    out.push_str(&format!("**OS:** {}  \n", report.system_metadata.os_name));
    out.push_str(&format!(
        "**Filesystem & Mount:** {} ({})  \n",
        report.system_metadata.filesystem, report.system_metadata.mount_options
    ));
    out.push_str(&format!("**Durability:** {}\n\n", report.durability_config));

    out.push_str("| Workload | Metric | Contextra | redb | Ratio (Ctx / redb) |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

    for w in &report.workloads {
        out.push_str(&format!(
            "| {} | Latency p50 | {:.2} µs | {:.2} µs | {:.2}x |\n",
            w.workload_name,
            w.contextra.latency_p50_micros_mean,
            w.redb.latency_p50_micros_mean,
            w.ratio_p50
        ));
        out.push_str(&format!(
            "| | Latency p99 | {:.2} µs | {:.2} µs | {:.2}x |\n",
            w.contextra.latency_p99_micros_mean,
            w.redb.latency_p99_micros_mean,
            w.ratio_p99
        ));
        out.push_str(&format!(
            "| | Throughput | {:.1} ops/s | {:.1} ops/s | {:.2}x |\n",
            w.contextra.throughput_ops_sec_mean,
            w.redb.throughput_ops_sec_mean,
            w.ratio_throughput
        ));
        out.push_str(&format!(
            "| | Write Amp | {:.2}x | {:.2}x | - |\n",
            w.contextra.write_amplification_mean,
            w.redb.write_amplification_mean
        ));
        out.push_str(&format!(
            "| | Max FDs | {} | {} | - |\n",
            w.contextra.open_fd_count_max,
            w.redb.open_fd_count_max
        ));
    }

    out
}
