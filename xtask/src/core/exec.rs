use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub exit_code: i32,
    pub duration_ms: u64,
    pub log_sha256: String,
    pub output: String,
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

    match output {
        Ok(out) => {
            let exit_code = out.status.code().unwrap_or(1);
            let combined = format!(
                "STDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let log_sha256 = blake3::hash(combined.as_bytes()).to_hex().to_string();
            ExecResult {
                exit_code,
                duration_ms,
                log_sha256,
                output: combined,
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
            }
        }
    }
}
