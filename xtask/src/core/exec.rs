use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub exit_code: i32,
    pub duration_ms: u64,
    pub log_sha256: String,
    pub output: String,
    pub tests_run: u64,
    pub tests_skipped: u64,
    pub test_parse_error: bool,
}

pub fn parse_test_counts(output: &str) -> Option<(u64, u64)> {
    // 1. Try parsing JSON lines if libtest-JSON is used
    for line in output.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if val.get("type").and_then(|v| v.as_str()) == Some("suite") {
                    if let (Some(passed), Some(failed), Some(ignored)) = (
                        val.get("passed").and_then(|v| v.as_u64()),
                        val.get("failed").and_then(|v| v.as_u64()),
                        val.get("ignored").and_then(|v| v.as_u64()),
                    ) {
                        return Some((passed + failed, ignored));
                    }
                }
            }
        }
    }

    // 2. Try parsing human readable / terse output line: "test result: ok." or "test result: FAILED."
    for line in output.lines().rev() {
        if line.contains("test result:") {
            let mut passed = None;
            let mut failed = None;
            let mut ignored = None;

            for part in line.split(';') {
                let tokens: Vec<&str> = part.split_whitespace().collect();
                if let Some(pos) = tokens.iter().position(|&t| t == "passed") {
                    if pos > 0 {
                        passed = tokens[pos - 1].parse::<u64>().ok();
                    }
                }
                if let Some(pos) = tokens.iter().position(|&t| t == "failed") {
                    if pos > 0 {
                        failed = tokens[pos - 1].parse::<u64>().ok();
                    }
                }
                if let Some(pos) = tokens.iter().position(|&t| t == "ignored") {
                    if pos > 0 {
                        ignored = tokens[pos - 1].parse::<u64>().ok();
                    }
                }
            }

            if let (Some(p), Some(f), Some(i)) = (passed, failed, ignored) {
                return Some((p + f, i));
            }
        }
    }

    None
}

pub fn run_cmd(
    cmd_name: &str,
    args: &[&str],
    work_dir: &Path,
) -> ExecResult {
    let start = Instant::now();
    let output = Command::new(cmd_name)
        .args(args)
        .current_dir(work_dir)
        .output();

    let duration_ms = start.elapsed().as_millis() as u64;

    let zero_count: u64 = 0;

    match output {
        Ok(out) => {
            let exit_code = out.status.code().unwrap_or(1);
            let combined = format!(
                "STDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let log_sha256 = blake3::hash(combined.as_bytes()).to_hex().to_string();

            let (tests_run, tests_skipped, test_parse_error) = if let Some((r, s)) = parse_test_counts(&combined) {
                (r, s, false)
            } else if combined.contains("test result:") || combined.contains("running ") {
                (zero_count, zero_count, true)
            } else {
                (zero_count, zero_count, false)
            };

            ExecResult {
                exit_code,
                duration_ms,
                log_sha256,
                output: combined,
                tests_run,
                tests_skipped,
                test_parse_error,
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to execute process '{}': {}", cmd_name, e);
            let log_sha256 = blake3::hash(err_msg.as_bytes()).to_hex().to_string();
            ExecResult {
                exit_code: 2,
                duration_ms,
                log_sha256,
                output: err_msg,
                tests_run: zero_count,
                tests_skipped: zero_count,
                test_parse_error: false,
            }
        }
    }
}
