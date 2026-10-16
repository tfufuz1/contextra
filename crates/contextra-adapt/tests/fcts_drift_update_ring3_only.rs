//! AK-18 Architektur-Lint: Stellt sicher, dass `drift_rate` Updates exklusiv aus Ring-3 stammen.

#![cfg(feature = "flow-corrected-thompson")]

use std::fs;
use std::path::{Path, PathBuf};

fn find_workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !dir.join("Cargo.toml").exists() || !dir.join("crates").exists() {
        if !dir.pop() {
            return Err("Workspace root not found!".into());
        }
    }
    Ok(dir)
}

#[test]
fn test_ak18_drift_rate_assignment_restricted_to_recompute(
) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = find_workspace_root()?;
    let flow_thompson_path = workspace_root.join("crates/contextra-adapt/src/flow_thompson.rs");
    let content = fs::read_to_string(&flow_thompson_path)?;

    // (a) Zuweisung an drift_rate kommt nur innerhalb von recompute_drift_rate_from_window vor
    let mut in_recompute = false;
    let mut brace_level = 0;
    let mut line_num = 0;

    for line in content.lines() {
        line_num += 1;
        if line.contains("fn recompute_drift_rate_from_window") {
            in_recompute = true;
        }

        if in_recompute {
            brace_level += line.chars().filter(|&c| c == '{').count();
            brace_level -= line.chars().filter(|&c| c == '}').count();
            if brace_level == 0 && line.contains('}') {
                in_recompute = false;
            }
        } else if line.contains("drift_rate =")
            || line.contains("drift_rate.fill")
            || line.contains("drift_rate.clear")
            || (line.contains("drift_rate[") && line.contains("]="))
        {
            // Checks that outside recompute_drift_rate_from_window there is no assignment or mutation to drift_rate
            let trimmed = line.trim();
            if !trimmed.starts_with("//")
                && !trimmed.starts_with("///")
                && !trimmed.starts_with("!")
            {
                return Err(format!(
                    "AK-18 Violation: Direct assignment or modification of `drift_rate` outside `recompute_drift_rate_from_window` at line {line_num}: {line}"
                ).into());
            }
        }
    }
    Ok(())
}

#[test]
fn test_ak18_recompute_signature_requires_ring3_token() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = find_workspace_root()?;
    let flow_thompson_path = workspace_root.join("crates/contextra-adapt/src/flow_thompson.rs");
    let content = fs::read_to_string(&flow_thompson_path)?;

    // (b) recompute_drift_rate_from_window verlangt &Ring3Token (Signatur-Check per Text)
    if !content.contains("pub fn recompute_drift_rate_from_window(&mut self, _token: &Ring3Token)")
    {
        return Err(
            "AK-18 Violation: `recompute_drift_rate_from_window` signature must accept `&Ring3Token`".into(),
        );
    }
    Ok(())
}

fn scan_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if !dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_rs_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

#[test]
fn test_ak18_ring3_token_not_called_in_ring0_crates() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = find_workspace_root()?;
    let ring0_crate_names = [
        "contextra-text",
        "contextra-graph",
        "contextra-crypto",
        "contextra-adapt",
    ];

    for crate_name in ring0_crate_names {
        let crate_dir = workspace_root.join("crates").join(crate_name);
        let mut rs_files = Vec::new();
        scan_rs_files(&crate_dir, &mut rs_files);

        for file in rs_files {
            // Ignore definition in flow_thompson.rs, lib.rs export, and test files
            let rel_path = match file.strip_prefix(&workspace_root) {
                Ok(p) => match p.to_str() {
                    Some(s) => s,
                    None => continue,
                },
                Err(_) => continue,
            };

            if rel_path.contains("flow_thompson.rs")
                || rel_path.contains("lib.rs")
                || rel_path.contains("/tests/")
                || rel_path.contains("fcts_")
            {
                continue;
            }

            let content = fs::read_to_string(&file)?;

            for (idx, line) in content.lines().enumerate() {
                if line.contains("ring3_background_task_token") {
                    let trimmed = line.trim();
                    if !trimmed.starts_with("//") && !trimmed.starts_with("///") {
                        return Err(format!(
                            "AK-18 Violation: `ring3_background_task_token` called in Ring-0 crate `{crate_name}` at {rel_path}:{}: {line}",
                            idx + 1
                        ).into());
                    }
                }
            }
        }
    }
    Ok(())
}
