// FILE-CONTEXT
// STAND: 2026-09-30
// ZWECK: Skalierungs-Benchmark fuer 100.000 Dokumente zur Messung von Einfügerat, Suchlatenz und RSS-Speicher

use contextra_db::{Contextra, ContextraConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalResult {
    pub docs_inserted: u64,
    pub insert_rate_docs_per_sec: f64,
    pub search_p50_ns: u64,
    pub search_p95_ns: u64,
    pub search_p99_ns: u64,
    pub rss_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleBenchmarkOutput {
    pub disclaimer: String,
    pub total_docs_requested: u64,
    pub step_size: u64,
    pub dimension: usize,
    pub seed: u64,
    pub aborted_at_docs: Option<u64>,
    pub error: Option<String>,
    pub intervals: Vec<IntervalResult>,
}

struct SimplePrng {
    state: u64,
}

impl SimplePrng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        let u = (self.next_u64() >> 11) as f64;
        (u / (1u64 << 53) as f64) as f32
    }

    fn next_vector(&mut self, dim: usize) -> Vec<f32> {
        let mut vec: Vec<f32> = (0..dim).map(|_| self.next_f32() - 0.5).collect();
        let norm_sq: f32 = vec.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-6 {
            for x in vec.iter_mut() {
                *x /= norm;
            }
        }
        vec
    }
}

fn read_rss_bytes() -> u64 {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb * 1024;
                    }
                }
            }
        }
    }
    0
}

fn parse_cli_args() -> (u64, u64, usize, u64, PathBuf) {
    let args: Vec<String> = std::env::args().collect();
    let mut docs = 100_000u64;
    let mut step = 10_000u64;
    let mut dim = 128usize;
    let mut seed = 42u64;
    let mut out_path = PathBuf::from("benchmarks/results/scale_100k.json");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--docs" if i + 1 < args.len() => {
                if let Ok(val) = args[i + 1].parse() {
                    docs = val;
                }
                i += 1;
            }
            "--step" if i + 1 < args.len() => {
                if let Ok(val) = args[i + 1].parse() {
                    step = val;
                }
                i += 1;
            }
            "--dim" if i + 1 < args.len() => {
                if let Ok(val) = args[i + 1].parse() {
                    dim = val;
                }
                i += 1;
            }
            "--seed" if i + 1 < args.len() => {
                if let Ok(val) = args[i + 1].parse() {
                    seed = val;
                }
                i += 1;
            }
            "--out" if i + 1 < args.len() => {
                out_path = PathBuf::from(&args[i + 1]);
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    (docs, step, dim, seed, out_path)
}

fn write_output(output: &ScaleBenchmarkOutput, out_path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = out_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(output)?;
    fs::write(out_path, json)
}

async fn run_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let (total_docs, step_size, dim, seed, out_path) = parse_cli_args();

    let disclaimer = format!(
        "Hinweis: Diese Zahlen messen nur Speicher- und Indexlatenz bzw. Retrieval-Genauigkeit auf dem genannten Testkorpus. Embedding-Inferenz addiert 10 bis 500 ms. Der Testkorpus umfasst {} Chunks; die Zahlen sind nicht auf größere Bestände extrapolierbar.",
        total_docs
    );

    let mut output = ScaleBenchmarkOutput {
        disclaimer,
        total_docs_requested: total_docs,
        step_size,
        dimension: dim,
        seed,
        aborted_at_docs: None,
        error: None,
        intervals: Vec::new(),
    };

    let mut prng = SimplePrng::new(seed);
    let temp_dir = TempDir::new()?;

    let db_cfg = ContextraConfig {
        dimension: dim,
        max_ram_mb: 2048,
        ..Default::default()
    };

    let db = match Contextra::open_with_config(temp_dir.path(), db_cfg).await {
        Ok(db) => db,
        Err(e) => {
            output.error = Some(format!("Failed to open Contextra DB: {}", e));
            write_output(&output, &out_path)?;
            return Err(e.into());
        }
    };

    let col = match db.collection("scale_100k_col").await {
        Ok(col) => col,
        Err(e) => {
            output.error = Some(format!("Failed to open collection: {}", e));
            write_output(&output, &out_path)?;
            return Err(e.into());
        }
    };

    let mut current_docs = 0u64;

    while current_docs < total_docs {
        let target_docs = (current_docs + step_size).min(total_docs);
        let docs_to_insert = target_docs - current_docs;

        let insert_start = Instant::now();
        let mut insert_err = None;

        for i in 0..docs_to_insert {
            let doc_num = current_docs + i + 1;
            let doc_id = format!("doc_{}", doc_num);
            let vec = prng.next_vector(dim);
            let metadata = serde_json::json!({
                "title": format!("Document {}", doc_num),
                "text": format!("Synthetic text body for document number {}", doc_num),
            });

            if let Err(e) = col.insert(&doc_id, &vec, Some(metadata)).await {
                insert_err = Some(e);
                break;
            }
        }

        let insert_elapsed = insert_start.elapsed();

        if let Some(err) = insert_err {
            output.aborted_at_docs = Some(current_docs);
            output.error = Some(format!("Error during insertion: {}", err));
            write_output(&output, &out_path)?;
            return Err(err.into());
        }

        current_docs = target_docs;
        let insert_rate = if insert_elapsed.as_secs_f64() > 0.0 {
            docs_to_insert as f64 / insert_elapsed.as_secs_f64()
        } else {
            docs_to_insert as f64
        };

        // Measure search latencies over 200 query iterations
        let num_queries = 200;
        let mut query_latencies_ns = Vec::with_capacity(num_queries);
        let mut query_prng = SimplePrng::new(seed.wrapping_add(current_docs));

        let mut search_err = None;
        for _ in 0..num_queries {
            let q_vec = query_prng.next_vector(dim);
            let q_start = Instant::now();
            let res = col.query().embedding(&q_vec).k(10).execute().await;
            let q_duration = q_start.elapsed();

            match res {
                Ok(_) => {
                    let ns = q_duration.as_nanos().max(1) as u64;
                    query_latencies_ns.push(ns);
                }
                Err(e) => {
                    search_err = Some(e);
                    break;
                }
            }
        }

        if let Some(err) = search_err {
            output.aborted_at_docs = Some(current_docs);
            output.error = Some(format!("Error during search latency measurement: {}", err));
            write_output(&output, &out_path)?;
            return Err(err.into());
        }

        query_latencies_ns.sort_unstable();
        let p50_ns = query_latencies_ns[num_queries * 50 / 100];
        let p95_ns = query_latencies_ns[num_queries * 95 / 100];
        let p99_ns = query_latencies_ns[num_queries * 99 / 100];
        let rss = read_rss_bytes();

        output.intervals.push(IntervalResult {
            docs_inserted: current_docs,
            insert_rate_docs_per_sec: insert_rate,
            search_p50_ns: p50_ns,
            search_p95_ns: p95_ns,
            search_p99_ns: p99_ns,
            rss_bytes: rss,
        });

        println!(
            "[{}/{} docs] Insert: {:.1} docs/s | Search p50: {} ns, p95: {} ns, p99: {} ns | RSS: {} MB",
            current_docs,
            total_docs,
            insert_rate,
            p50_ns,
            p95_ns,
            p99_ns,
            rss / (1024 * 1024)
        );
    }

    write_output(&output, &out_path)?;
    println!(
        "Benchmark completed successfully. Output written to `{}`.",
        out_path.display()
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_benchmark().await
}
