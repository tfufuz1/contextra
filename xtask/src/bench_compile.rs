use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchCompileResult {
    pub crate_name: String,
    pub bench_name: String,
    pub passed: bool,
    pub output: String,
}

/// Discovers benchmark files in `crates/*/benches/*.rs` and `benchmarks/*/benches/*.rs`.
/// Returns a list of `(crate_name, bench_name)` tuples.
pub fn discover_benches(root: &Path) -> Vec<(String, String)> {
    let mut discovered = Vec::new();

    let scan_dirs = [root.join("crates"), root.join("benchmarks")];

    for scan_dir in &scan_dirs {
        if !scan_dir.exists() {
            continue;
        }

        for entry in WalkDir::new(scan_dir).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(rel_path) = path.strip_prefix(root) {
                let components: Vec<_> = rel_path.components().map(|c| c.as_os_str()).collect();
                // Expected path structures:
                // crates/<crate_name>/benches/<bench_name>.rs
                // benchmarks/<crate_name>/benches/<bench_name>.rs
                if components.len() >= 4
                    && (components[0] == "crates" || components[0] == "benchmarks")
                    && components[2] == "benches"
                {
                    if let (Some(file_name), Some(crate_name)) = (
                        path.file_name().and_then(|n| n.to_str()),
                        components[1].to_str(),
                    ) {
                        if file_name.ends_with(".rs") {
                            let bench_name = file_name.strip_suffix(".rs").unwrap_or(file_name);
                            discovered.push((crate_name.to_string(), bench_name.to_string()));
                        }
                    }
                }
            }
        }
    }

    discovered.sort();
    discovered.dedup();
    discovered
}

/// Helper function to construct cargo argument list for benchmark check/run.
pub fn build_bench_command_args(
    crate_name: &str,
    bench_name: &str,
    run_for_real: bool,
) -> Vec<String> {
    let mut args = Vec::new();
    if run_for_real {
        args.push("bench".to_string());
        args.push("-p".to_string());
        args.push(crate_name.to_string());
        args.push("--locked".to_string());
        args.push("--bench".to_string());
        args.push(bench_name.to_string());
        args.push("--".to_string());
        args.push("--test".to_string());
    } else {
        args.push("check".to_string());
        args.push("-p".to_string());
        args.push(crate_name.to_string());
        args.push("--locked".to_string());
        args.push("--bench".to_string());
        args.push(bench_name.to_string());
    }
    args
}

/// Checks or runs all benchmark suites found in the workspace.
pub fn run_bench_compile(
    run_for_real: bool,
    root: &Path,
) -> Result<Vec<BenchCompileResult>, String> {
    let benches = discover_benches(root);
    let mut results = Vec::new();

    for (crate_name, bench_name) in benches {
        let args = build_bench_command_args(&crate_name, &bench_name, run_for_real);
        let mut cmd = Command::new("cargo");
        cmd.current_dir(root);
        cmd.args(&args);

        let output = cmd.output().map_err(|e| {
            format!(
                "Failed to execute cargo for benchmark '{bench_name}' in crate '{crate_name}': {e}"
            )
        })?;

        let passed = output.status.success();
        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
        let combined_output = format!("{stdout_str}\n{stderr_str}");

        results.push(BenchCompileResult {
            crate_name,
            bench_name,
            passed,
            output: combined_output,
        });
    }

    Ok(results)
}
