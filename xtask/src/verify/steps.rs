// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Verification step definitions and execution routines for `cargo xtask verify`.

use crate::verify::evidence::{StepEvidence, StepStatus};
use blake3;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

pub struct StepRunner<'a> {
    pub root: &'a Path,
    pub run_dir: &'a Path,
    pub rel_run_dir: &'a str,
    pub toolchain_info: &'a str,
    pub commit_sha: &'a str,
    pub allow_skips: &'a [String],
}

impl<'a> StepRunner<'a> {
    pub fn run_step(
        &self,
        step_id: &str,
        argv: &[&str],
        timeout_secs: Option<u64>,
        missing_tool_check: Option<&str>,
    ) -> StepEvidence {
        let start = Instant::now();
        let argv_vec: Vec<String> = argv.iter().map(|s| s.to_string()).collect();

        if self.allow_skips.contains(&step_id.to_string()) {
            return self.write_skipped_by_flag_evidence(step_id, argv_vec);
        }

        if let Some(tool) = missing_tool_check {
            if !is_tool_available(tool) {
                return self.write_skipped_tool_missing_evidence(step_id, argv_vec, tool);
            }
        }

        let log_file_name = format!("{}.log", step_id);
        let log_path = self.run_dir.join(&log_file_name);
        let rel_log_path = format!("{}/{}", self.rel_run_dir, log_file_name);

        let mut file = match File::create(&log_path) {
            Ok(f) => f,
            Err(e) => {
                return StepEvidence {
                    step_id: step_id.to_string(),
                    status: StepStatus::Error,
                    argv: argv_vec,
                    toolchain: self.toolchain_info.to_string(),
                    commit: self.commit_sha.to_string(),
                    exit_code: -1,
                    duration_ms: start.elapsed().as_millis() as u64,
                    log_path: rel_log_path,
                    log_sha256: String::new(),
                    bytes: 0,
                };
            }
        };

        let mut cmd = Command::new(argv[0]);
        cmd.args(&argv[1..])
            .current_dir(self.root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Failed to spawn process {:?}: {}\n", argv, e);
                let _ = file.write_all(err_msg.as_bytes());
                return StepEvidence {
                    step_id: step_id.to_string(),
                    status: StepStatus::SkippedToolMissing,
                    argv: argv_vec,
                    toolchain: self.toolchain_info.to_string(),
                    commit: self.commit_sha.to_string(),
                    exit_code: 127,
                    duration_ms: start.elapsed().as_millis() as u64,
                    log_path: rel_log_path,
                    log_sha256: String::new(),
                    bytes: 0,
                };
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let file_clone_1 = file.try_clone();
        let t1 = std::thread::spawn(move || {
            if let (Some(mut out), Ok(mut f)) = (stdout, file_clone_1) {
                let mut buf = [0u8; 8192];
                while let Ok(n) = out.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let _ = f.write_all(&buf[..n]);
                }
            }
        });

        let file_clone_2 = file.try_clone();
        let t2 = std::thread::spawn(move || {
            if let (Some(mut err), Ok(mut f)) = (stderr, file_clone_2) {
                let mut buf = [0u8; 8192];
                while let Ok(n) = err.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let _ = f.write_all(&buf[..n]);
                }
            }
        });

        let exit_code = match child.wait() {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        };

        let _ = t1.join();
        let _ = t2.join();

        let duration_ms = start.elapsed().as_millis() as u64;

        let (bytes, sha256) = hash_file(&log_path);

        let status = if exit_code == 0 {
            StepStatus::Passed
        } else {
            StepStatus::Failed
        };

        let ev = StepEvidence {
            step_id: step_id.to_string(),
            status,
            argv: argv_vec,
            toolchain: self.toolchain_info.to_string(),
            commit: self.commit_sha.to_string(),
            exit_code,
            duration_ms,
            log_path: rel_log_path,
            log_sha256: sha256,
            bytes,
        };

        self.save_json(step_id, &ev);
        ev
    }

    fn write_skipped_by_flag_evidence(&self, step_id: &str, argv: Vec<String>) -> StepEvidence {
        let log_file_name = format!("{}.log", step_id);
        let log_path = self.run_dir.join(&log_file_name);
        let rel_log_path = format!("{}/{}", self.rel_run_dir, log_file_name);

        let _ = fs::write(&log_path, "Step skipped by explicit --allow-skip flag.\n");
        let (bytes, sha256) = hash_file(&log_path);

        let ev = StepEvidence {
            step_id: step_id.to_string(),
            status: StepStatus::SkippedByFlag,
            argv,
            toolchain: self.toolchain_info.to_string(),
            commit: self.commit_sha.to_string(),
            exit_code: 0,
            duration_ms: 0,
            log_path: rel_log_path,
            log_sha256: sha256,
            bytes,
        };
        self.save_json(step_id, &ev);
        ev
    }

    fn write_skipped_tool_missing_evidence(
        &self,
        step_id: &str,
        argv: Vec<String>,
        tool: &str,
    ) -> StepEvidence {
        let log_file_name = format!("{}.log", step_id);
        let log_path = self.run_dir.join(&log_file_name);
        let rel_log_path = format!("{}/{}", self.rel_run_dir, log_file_name);

        let _ = fs::write(
            &log_path,
            format!(
                "Step skipped because required tool '{}' was missing.\n",
                tool
            ),
        );
        let (bytes, sha256) = hash_file(&log_path);

        let ev = StepEvidence {
            step_id: step_id.to_string(),
            status: StepStatus::SkippedToolMissing,
            argv,
            toolchain: self.toolchain_info.to_string(),
            commit: self.commit_sha.to_string(),
            exit_code: 127,
            duration_ms: 0,
            log_path: rel_log_path,
            log_sha256: sha256,
            bytes,
        };
        self.save_json(step_id, &ev);
        ev
    }

    fn save_json(&self, step_id: &str, ev: &StepEvidence) {
        let json_path = self.run_dir.join(format!("{}.json", step_id));
        if let Ok(content) = serde_json::to_string_pretty(ev) {
            let _ = fs::write(json_path, content);
        }
    }
}

fn is_tool_available(tool: &str) -> bool {
    match tool {
        "cargo-nextest" => Command::new("cargo")
            .args(["nextest", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        "cargo-mutants" => Command::new("cargo")
            .args(["mutants", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        "cargo-fuzz" => Command::new("cargo")
            .args(["+nightly", "fuzz", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        "miri" => Command::new("cargo")
            .args(["+nightly", "miri", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        _ => Command::new(tool)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
    }
}

fn hash_file(path: &Path) -> (u64, String) {
    if let Ok(mut f) = File::open(path) {
        let mut hasher = blake3::Hasher::new();
        let mut buf = [0u8; 8192];
        let mut total_bytes = 0u64;
        while let Ok(n) = f.read(&mut buf) {
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            total_bytes += n as u64;
        }
        let res = hasher.finalize();
        (total_bytes, res.to_hex().to_string())
    } else {
        (0, String::new())
    }
}
