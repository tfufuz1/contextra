#![allow(unused_imports, dead_code, unused_variables)]
use crate::*;
use std::path::{Path, PathBuf};
use std::process;

pub fn run_bench_trend(args: &[String]) -> i32 {
    if !proof::bench_trend::run_bench_trend() {
        return 1;
    }
    0
}

pub fn run_bench_gate(args: &[String]) -> i32 {
    let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
    let status = std::process::Command::new("cargo")
        .args(["run", "--quiet", "-p", "xtask-heavy", "--", "bench-gate"])
        .args(extra_args)
        .status();
    match status {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("❌ Failed to execute xtask-heavy bench-gate: {}", e);
            1
        }
    }
}

pub fn run_bench_compile(args: &[String]) -> i32 {
    let run_for_real = args.iter().any(|arg| arg == "--run");
    let root = crate::find_root_dir();
    match bench_compile::run_bench_compile(run_for_real, &root) {
        Ok(results) => {
            println!(
                "✅ bench-compile completed for {} benchmark target(s)",
                results.len()
            );
        }
        Err(e) => {
            eprintln!("❌ bench-compile failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_bench_download(args: &[String]) -> i32 {
    let dataset_str = args.iter().find_map(|arg| arg.strip_prefix("--dataset="));
    let dataset = match dataset_str {
        Some("ann-sift1m") => bench_download::Dataset::AnnSift1m,
        Some("beir") => bench_download::Dataset::Beir,
        Some("onnx-test-model") => bench_download::Dataset::OnnxTestModel,
        Some("all") => bench_download::Dataset::All,
        Some(other) => {
            eprintln!(
                "❌ Unknown dataset '{}'. Valid options: ann-sift1m, beir, onnx-test-model, all",
                other
            );
            return 1;
        }
        None => {
            eprintln!(
                "❌ Missing required parameter --dataset=<ann-sift1m|beir|onnx-test-model|all>"
            );
            return 1;
        }
    };
    let root = crate::find_root_dir();
    if let Err(e) = bench_download::run_bench_download(dataset, &root) {
        eprintln!("❌ bench-download failed: {}", e);
        return 1;
    }
    0
}
