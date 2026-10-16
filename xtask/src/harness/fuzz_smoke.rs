//! Gate harness for fuzz smoke execution over touched crates containing fuzz targets.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FuzzSmokeFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FuzzSmokeOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<FuzzSmokeFinding>,
}

pub fn fuzz_smoke_get_touched_fuzz_crates(
    root: &Path,
    base: &str,
    head: &str,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let diff_output = Command::new("git")
        .args(["diff", "--name-only", base, head])
        .current_dir(root)
        .output();

    let mut touched_crates = BTreeMap::new();
    if let Ok(o) = diff_output {
        let diff_text = String::from_utf8_lossy(&o.stdout);
        for line in diff_text.lines() {
            if line.starts_with("crates/") {
                let parts: Vec<&str> = line.split('/').collect();
                if parts.len() >= 2 {
                    let cname = parts[1];
                    let fuzz_dir = root.join("crates").join(cname).join("fuzz/fuzz_targets");
                    if fuzz_dir.exists() {
                        let mut targets = Vec::new();
                        if let Ok(entries) = fs::read_dir(&fuzz_dir) {
                            for entry in entries.flatten() {
                                if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                                    if let Some(stem) =
                                        entry.path().file_stem().and_then(|s| s.to_str())
                                    {
                                        targets.push(stem.to_string());
                                    }
                                }
                            }
                        }
                        if !targets.is_empty() {
                            touched_crates.insert(cname.to_string(), targets);
                        }
                    }
                }
            }
        }
    }

    Ok(touched_crates)
}

pub fn run_fuzz_smoke(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;
    let mut secs = 60u64;
    let mut base_rev = None;
    let mut head_rev = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--json" => {
                json = true;
            }
            "--secs" => {
                if i + 1 < args.len() {
                    if let Ok(s) = args[i + 1].parse() {
                        secs = s;
                    }
                    i += 1;
                }
            }
            "--base" => {
                if i + 1 < args.len() {
                    let val = args[i + 1].trim();
                    if !val.is_empty() {
                        base_rev = Some(val.to_string());
                    }
                    i += 1;
                }
            }
            "--head" => {
                if i + 1 < args.len() {
                    let val = args[i + 1].trim();
                    if !val.is_empty() {
                        head_rev = Some(val.to_string());
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let base = base_rev.unwrap_or_else(|| "HEAD~1".to_string());
    let head = head_rev.unwrap_or_else(|| "HEAD".to_string());

    let touched_crates = match fuzz_smoke_get_touched_fuzz_crates(&root, &base, &head) {
        Ok(c) => c,
        Err(e) => {
            let out = FuzzSmokeOutput {
                gate: "fuzz-smoke".to_string(),
                status: "error".to_string(),
                summary: e,
                findings: vec![],
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            }
            return 2;
        }
    };

    if touched_crates.is_empty() {
        let output = FuzzSmokeOutput {
            gate: "fuzz-smoke".to_string(),
            status: "not_applicable".to_string(),
            summary: "No touched crates with fuzz targets in diff".to_string(),
            findings: vec![],
        };
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
        } else {
            println!("fuzz-smoke: not_applicable (no fuzz targets touched)");
        }
        return 0;
    }

    let mut findings = Vec::new();
    let mut has_error = false;

    for (cname, targets) in &touched_crates {
        let crate_dir = root.join("crates").join(cname);
        for target in targets {
            let fuzz_cmd = Command::new("cargo")
                .args([
                    "+nightly",
                    "fuzz",
                    "run",
                    target,
                    "--",
                    &format!("-max_total_time={}", secs),
                ])
                .current_dir(&crate_dir)
                .output();

            match fuzz_cmd {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let combined = format!("{}\n{}", stdout, stderr);

                    if !o.status.success() {
                        if combined.contains("no such subcommand")
                            || combined.contains("toolchain 'nightly")
                            || combined.contains("error:")
                        {
                            has_error = true;
                            findings.push(FuzzSmokeFinding {
                                id: "FUZZ_TOOLCHAIN_MISSING".to_string(),
                                severity: "error".to_string(),
                                file: format!("crates/{}/fuzz/fuzz_targets/{}.rs", cname, target),
                                line: 0,
                                message: format!(
                                    "cargo-fuzz or +nightly toolchain missing for {}",
                                    cname
                                ),
                                fix: "Install cargo-fuzz and nightly toolchain".to_string(),
                            });
                            continue;
                        }

                        let artifact_path = if combined.contains("fuzz/artifacts") {
                            "artifacts found in fuzz/artifacts"
                        } else {
                            "crash artifact generated"
                        };

                        findings.push(FuzzSmokeFinding {
                            id: "FUZZ_CRASH_DETECTED".to_string(),
                            severity: "error".to_string(),
                            file: format!("crates/{}/fuzz/fuzz_targets/{}.rs", cname, target),
                            line: 0,
                            message: format!(
                                "Fuzz target {} in {} crashed: {}",
                                target, cname, artifact_path
                            ),
                            fix: "Fix panic or memory safety issue triggered by fuzz input"
                                .to_string(),
                        });
                    }
                }
                Err(_) => {
                    has_error = true;
                    findings.push(FuzzSmokeFinding {
                        id: "FUZZ_EXECUTION_ERROR".to_string(),
                        severity: "error".to_string(),
                        file: format!("crates/{}/fuzz/fuzz_targets/{}.rs", cname, target),
                        line: 0,
                        message: format!("Failed to execute cargo +nightly fuzz for {}", cname),
                        fix: "Install cargo-fuzz and nightly toolchain".to_string(),
                    });
                }
            }
        }
    }

    let status = if has_error {
        "error"
    } else if findings.iter().any(|f| f.severity == "error") {
        "fail"
    } else {
        "pass"
    };

    let summary = format!(
        "Fuzz smoke testing completed across {} crates: {} findings",
        touched_crates.len(),
        findings.len()
    );

    let output = FuzzSmokeOutput {
        gate: "fuzz-smoke".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: findings.clone(),
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!("{}", summary);
        for f in &findings {
            println!(
                "  [{}] {}: {}",
                f.severity.to_uppercase(),
                f.file,
                f.message
            );
        }
    }

    match status {
        "pass" => 0,
        "fail" => 1,
        _ => 2,
    }
}
