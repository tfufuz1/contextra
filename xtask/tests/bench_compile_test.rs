#[path = "../src/bench_compile.rs"]
mod bench_compile;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_discover_benches_synthetic() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let crate_benches = root.join("crates").join("crate-a").join("benches");
    let bench_benches = root.join("benchmarks").join("bench-b").join("benches");
    fs::create_dir_all(&crate_benches).unwrap();
    fs::create_dir_all(&bench_benches).unwrap();

    fs::write(crate_benches.join("my_bench.rs"), "// bench").unwrap();
    fs::write(bench_benches.join("another_bench.rs"), "// bench").unwrap();

    let discovered = bench_compile::discover_benches(root);

    assert_eq!(discovered.len(), 2);
    assert_eq!(
        discovered,
        vec![
            ("bench-b".to_string(), "another_bench".to_string()),
            ("crate-a".to_string(), "my_bench".to_string()),
        ]
    );
}

#[test]
fn test_build_bench_command_args_check() {
    let args = bench_compile::build_bench_command_args("contextra-store", "wal_bench", false);
    assert_eq!(
        args,
        vec![
            "check",
            "-p",
            "contextra-store",
            "--locked",
            "--bench",
            "wal_bench",
        ]
    );
}

#[test]
fn test_build_bench_command_args_real() {
    let args = bench_compile::build_bench_command_args("contextra-store", "wal_bench", true);
    assert_eq!(
        args,
        vec![
            "bench",
            "-p",
            "contextra-store",
            "--locked",
            "--bench",
            "wal_bench",
            "--",
            "--test",
        ]
    );
}
