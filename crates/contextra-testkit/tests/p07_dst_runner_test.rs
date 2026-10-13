// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: DST runner test harness, regression file runner, and NaiveWalSut verification.
// INVARIANTEN: Zero divergences for NaiveWalSut; regression seeds file MUST be parsed and tested.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_testkit::dst::{NaiveWalSut, P07DstRunner};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_p07_naive_wal_sut_passes_campaign() {
    let report = P07DstRunner::run_campaign(|sim_fs| {
        NaiveWalSut::new(sim_fs.clone(), PathBuf::from("wal_test.log"))
    })
    .expect("DST campaign failed execution");

    assert!(
        report.divergences.is_empty(),
        "NaiveWalSut MUST pass all DST campaign seeds without divergence: {:?}",
        report.divergences
    );

    println!(
        "NaiveWalSut PASSED campaign: {} seeds tested, duration: {}ms, rate: {:.1} seeds/s",
        report.seeds_tested, report.duration_ms, report.seeds_per_sec
    );

    // Verify report file exists and is valid JSON
    let report_content = fs::read_to_string("target/dst/report.json").expect("read report.json");
    assert!(report_content.contains("\"seeds_tested\":"));
    assert!(report_content.contains("\"divergences\":"));
}

#[test]
fn test_p07_regression_seeds_file_parsed() {
    let seeds_file = Path::new("tests/regressions/seeds.txt");
    assert!(
        seeds_file.exists(),
        "Regression seeds file MUST exist at tests/regressions/seeds.txt"
    );

    let content = fs::read_to_string(seeds_file).expect("read seeds.txt");
    let mut parsed_seeds = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Ok(seed) = trimmed.parse::<u64>() {
            parsed_seeds.push(seed);
        }
    }

    println!(
        "Parsed {} regression seeds from seeds.txt",
        parsed_seeds.len()
    );

    for seed in parsed_seeds {
        let res = P07DstRunner::run_seed(seed, 40, |sim_fs| {
            NaiveWalSut::new(sim_fs.clone(), PathBuf::from("wal_reg.log"))
        });
        assert!(
            res.is_ok(),
            "Regression seed {} failed for NaiveWalSut: {:?}",
            seed,
            res.err()
        );
    }
}
