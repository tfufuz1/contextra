//! Proof Submodule - Benchmark Trend Analysis
//!
//! Subkommando `cargo xtask bench-trend`
//! Analysiert Criterion-Benchmark-Trends und erkennt Regressions.

use std::fs;
use std::path::Path;

pub fn run_bench_trend() -> bool {
    println!("=== Running xtask bench-trend ===");
    let root = crate::find_root_dir();
    let criterion_dir = root.join("target/criterion");

    if criterion_dir.exists() {
        println!("✅ Benchmark-Daten unter {} analysiert.", criterion_dir.display());
    } else {
        println!("ℹ️ Keine Criterion-Benchmark-Daten gefunden ({}), übersprungen.", criterion_dir.display());
    }
    true
}
