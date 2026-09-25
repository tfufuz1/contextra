//! Subcommand `generate-diagnostics` (§A.4)
//! Runs registered gate subcommands, captures structured results and metadata header,
//! and writes report to `target/diagnostics/gate-report-<commit-kurz>.md`.

use crate::artifact_header::ArtifactHeader;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct GateResult {
    pub name: String,
    pub passed: bool,
    pub output: String,
    pub duration_ms: u128,
}

pub fn run_generate_diagnostics() -> Result<(), String> {
    println!("=== Running xtask generate-diagnostics ===");

    let header = ArtifactHeader::capture("cargo xtask generate-diagnostics")
        .map_err(|e| format!("Failed to capture artifact header: {}", e))?;

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to determine current executable path: {}", e))?;

    let gates: &[(&str, &[&str])] = &[
        ("check-flatbuffers-drift", &["check-flatbuffers-drift"]),
        ("check-bandit-latency-budget", &["check-bandit-latency-budget"]),
        ("check-module-reachability", &["check-module-reachability"]),
        (
            "check-duplicate-symbols-cross-file",
            &["check-duplicate-symbols", "--cross-module"],
        ),
    ];

    let mut results = Vec::new();

    for (name, args) in gates {
        println!("Running gate: {} ...", name);
        let start = Instant::now();
        let output_res = Command::new(&current_exe).args(*args).output();

        let (passed, output_text) = match output_res {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let combined = format!("{}\n{}", stdout.trim(), stderr.trim())
                    .trim()
                    .to_string();
                (out.status.success(), combined)
            }
            Err(e) => (false, format!("Failed to execute process: {}", e)),
        };

        let duration_ms = start.elapsed().as_millis();
        let status_str = if passed { "PASSED" } else { "FAILED" };
        println!("  -> {} in {} ms", status_str, duration_ms);

        results.push(GateResult {
            name: name.to_string(),
            passed,
            output: output_text,
            duration_ms,
        });
    }

    let commit_short = if header.commit.len() >= 7 {
        &header.commit[..7]
    } else {
        &header.commit
    };

    let target_dir = Path::new("target/diagnostics");
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create target/diagnostics directory: {}", e))?;

    let report_filename = format!("gate-report-{}.md", commit_short);
    let report_path = target_dir.join(&report_filename);

    let mut md = String::new();
    md.push_str(&header.render_markdown_frontmatter());
    md.push_str("\n# Gate Diagnostics Report\n\n");
    md.push_str(&format!("- **Generated At**: {}\n", header.generated_at));
    md.push_str(&format!("- **Commit**: {}\n", header.commit));
    md.push_str(&format!("- **Toolchain**: {}\n", header.toolchain));
    md.push_str(&format!("- **Command**: {}\n\n", header.generated_by));

    md.push_str("## Summary\n\n");
    md.push_str("| Gate | Status | Duration (ms) |\n");
    md.push_str("|---|---|---|\n");
    for r in &results {
        let status_icon = if r.passed { "✅ PASS" } else { "❌ FAIL" };
        md.push_str(&format!(
            "| `{}` | {} | {} |\n",
            r.name, status_icon, r.duration_ms
        ));
    }
    md.push_str("\n## Gate Details\n\n");

    for r in &results {
        let status_str = if r.passed { "PASSED" } else { "FAILED" };
        md.push_str(&format!("### {}\n\n", r.name));
        md.push_str(&format!("- **Status**: {}\n", status_str));
        md.push_str(&format!("- **Duration**: {} ms\n\n", r.duration_ms));
        md.push_str("```\n");
        md.push_str(&r.output);
        md.push_str("\n```\n\n");
    }

    fs::write(&report_path, &md)
        .map_err(|e| format!("Failed to write report to {}: {}", report_path.display(), e))?;

    println!("\n✅ Diagnostic report successfully written to {}", report_path.display());

    let any_failed = results.iter().any(|r| !r.passed);
    if any_failed {
        Err("One or more gates failed during generate-diagnostics run".to_string())
    } else {
        Ok(())
    }
}
