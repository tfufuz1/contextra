//! Prüft Port-Features im Workspace (G-12).

use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug)]
struct FeatureViolation {
    file: String,
    line: usize,
    message: String,
}

/// Runs the `check-port-features` gate.
pub fn run_check_port_features(args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let root = match xtask::find_root_dir_uncached(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
        r => r,
    };

    let mut violations = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != ".git"
                && name != ".cargo"
                && name != "node_modules"
                && name != "dist"
                && name != "build"
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|s| s.to_str()) == Some("Cargo.toml") {
            let rel_path = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if let Ok(content) = fs::read_to_string(path) {
                let mut in_dev_dependencies = false;
                let mut in_dependencies = false;

                for (idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    if trimmed.starts_with("[dev-dependencies]") {
                        in_dev_dependencies = true;
                        in_dependencies = false;
                        continue;
                    } else if trimmed.starts_with('[') {
                        in_dev_dependencies = false;
                        in_dependencies = trimmed.starts_with("[dependencies]");
                        continue;
                    }

                    // Check self-dependency in contextra-ports Cargo.toml
                    if rel_path == "crates/contextra-ports/Cargo.toml" && in_dev_dependencies {
                        if trimmed.starts_with("contextra-ports") {
                            violations.push(FeatureViolation {
                                file: rel_path.clone(),
                                line: idx + 1,
                                message: "Selbst-Abhängigkeit in contextra-ports/Cargo.toml [dev-dependencies] gefunden (F-41)".to_string(),
                            });
                        }
                    }

                    // Check backend-impl feature enabled outside contextra-store
                    if trimmed.contains("backend-impl") && !rel_path.contains("contextra-store") && !rel_path.contains("contextra-ports") {
                        violations.push(FeatureViolation {
                            file: rel_path.clone(),
                            line: idx + 1,
                            message: format!("Port-Feature 'backend-impl' darf nur in contextra-store aktiviert werden, gefunden in {}", rel_path),
                        });
                    }

                    // Check test-utils feature enabled in non-dev dependencies
                    if in_dependencies && trimmed.contains("test-utils") && trimmed.contains("contextra-ports") {
                        violations.push(FeatureViolation {
                            file: rel_path.clone(),
                            line: idx + 1,
                            message: format!("Port-Feature 'test-utils' darf nur in [dev-dependencies] stehen, gefunden in {}", rel_path),
                        });
                    }
                }
            }
        }
    }

    let count = violations.len();
    let status = if count == 0 { "pass" } else { "fail" };

    if json {
        let json_v: Vec<_> = violations
            .iter()
            .map(|v| {
                serde_json::json!({
                    "file": v.file,
                    "line": v.line,
                    "message": v.message
                })
            })
            .collect();

        let output = serde_json::json!({
            "gate": "check-port-features",
            "status": status,
            "count": count,
            "violations": json_v,
            "summary": format!("G-12 Port-Feature-Check: {} Verstöße im Workspace", count)
        });
        println!("{}", output);
    } else if count == 0 {
        println!("✅ [G-12 check-port-features]: 0 Port-Feature-Verstöße im Workspace.");
    } else {
        println!("❌ [G-12 check-port-features]: {} Port-Feature-Verstöße im Workspace:", count);
        for v in &violations {
            println!("  - {}:{}: {}", v.file, v.line, v.message);
        }
        println!("    Invariante: Spec F-40 / G-12 (backend-impl nur in store; test-utils nur dev; keine Ports-Selbst-Dependency).");
    }

    if count == 0 {
        0
    } else {
        1
    }
}
