//! Harness module for `cargo xtask check-port-names` verifying port trait names against §7.4.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortNameFinding {
    pub trait_name: String,
    pub status: String,
    pub location: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortNameGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<PortNameFinding>,
}

const REQUIRED_PORT_TRAITS: &[(&str, &str)] = &[
    ("Clock", "crates/contextra-ports/src/clock.rs"),
    ("EntropyRng", "crates/contextra-ports/src/rng.rs"),
    ("SeededRng", "crates/contextra-ports/src/rng.rs"),
    ("IdGenerator", "crates/contextra-ports/src/id_gen.rs"),
];

fn find_workspace_root() -> PathBuf {
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest);
        if p.file_name().and_then(|s| s.to_str()) == Some("xtask") {
            if let Some(parent) = p.parent() {
                return parent.to_path_buf();
            }
        }
        return p;
    }
    PathBuf::from(".")
}

pub fn run_check_port_names(args: &[String]) -> i32 {
    let json_output = args.iter().any(|a| a == "--json");
    let root = find_workspace_root();
    let ports_dir = root.join("crates").join("contextra-ports").join("src");

    let mut findings = Vec::new();
    let mut found_traits = Vec::new();

    if let Ok(entries) = fs::read_dir(&ports_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let rel_path = path
                    .strip_prefix(&root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");

                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(syn_file) = syn::parse_file(&content) {
                        for item in syn_file.items {
                            if let syn::Item::Trait(item_trait) = item {
                                if matches!(item_trait.vis, syn::Visibility::Public(_)) {
                                    let trait_name = item_trait.ident.to_string();
                                    found_traits.push((trait_name.clone(), rel_path.clone()));

                                    if trait_name == "Rng" {
                                        findings.push(PortNameFinding {
                                            trait_name: trait_name.clone(),
                                            status: "prohibited".to_string(),
                                            location: rel_path.clone(),
                                            message: "Prohibited standalone 'pub trait Rng' found. Use EntropyRng or SeededRng per §7.4 and E55.".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut missing_count = 0;
    for &(req_trait, expected_file) in REQUIRED_PORT_TRAITS {
        let exists = found_traits.iter().any(|(t, _)| t == req_trait);
        if exists {
            findings.push(PortNameFinding {
                trait_name: req_trait.to_string(),
                status: "pass".to_string(),
                location: expected_file.to_string(),
                message: format!("Required port trait '{}' present in contextra-ports.", req_trait),
            });
        } else {
            missing_count += 1;
            findings.push(PortNameFinding {
                trait_name: req_trait.to_string(),
                status: "missing".to_string(),
                location: expected_file.to_string(),
                message: format!("Required port trait '{}' is missing from contextra-ports.", req_trait),
            });
        }
    }

    let has_errors = missing_count > 0 || findings.iter().any(|f| f.status == "prohibited");
    let status = if has_errors { "fail" } else { "pass" };
    let summary = if has_errors {
        format!("Port name check failed: {} missing required trait(s) or prohibited trait name(s).", missing_count)
    } else {
        "All required port traits from §7.4 (Clock, EntropyRng, SeededRng, IdGenerator) are present.".to_string()
    };

    let result = PortNameGateResult {
        gate: "check-port-names".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings,
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
        println!("### Port Names Check (§7.4 / G-23): {}\n", status.to_uppercase());
        println!("{}", summary);
        println!("\n| Trait | Status | Expected Location | Details |");
        println!("|---|---|---|---|");
        for f in &result.findings {
            let icon = match f.status.as_str() {
                "pass" => "✅ PASS",
                "prohibited" => "🔴 PROHIBITED",
                _ => "🔴 MISSING",
            };
            println!("| {} | {} | {} | {} |", f.trait_name, icon, f.location, f.message);
        }
    }

    if has_errors { 2 } else { 0 }
}
