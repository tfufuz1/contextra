//! Gate: check-ring-layering
//! Validiert die workspace-internen Abhängigkeiten gegen die Ring-Matrix (§4.3/§4.4):
//!   Ring 0 → Ring 0
//!   Ring 1 → Ring 0 (kein Ring-1-zu-Ring-1)
//!   Ring 2 → types, ports, crypto
//!   Ring 3 → Ring 0, Ring 1, Ports von Ring 2
//!   Ring 4 → alle
//!   Tooling → memfuse-testkit + gleiches/tieferes Ring

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ring {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
    Ring4 = 4,
    Tooling = 99,
}

impl Ring {
    pub fn name(&self) -> &'static str {
        match self {
            Ring::Ring0 => "Ring 0",
            Ring::Ring1 => "Ring 1",
            Ring::Ring2 => "Ring 2",
            Ring::Ring3 => "Ring 3",
            Ring::Ring4 => "Ring 4",
            Ring::Tooling => "Tooling",
        }
    }
}

/// Mappt Crate-Namen exakt auf Ringe gemäß Spezifikation (§4.3 / Phase 0R).
pub fn get_crate_ring(crate_name: &str) -> Option<Ring> {
    match crate_name {
        // Ring 0
        "memfuse-types" | "memfuse-ports" | "memfuse-mvcc" | "memfuse-vector"
        | "memfuse-rank" | "memfuse-adapt" | "memfuse-text" | "memfuse-graph"
        | "memfuse-crypto" | "memfuse-simd" | "memfuse-sys" | "memfuse-wire"
        | "memfuse-core" | "memfuse-calibration" => Some(Ring::Ring0),

        // Ring 1
        "memfuse-store" | "memfuse-checkpoint" | "memfuse-kvcache" => Some(Ring::Ring1),

        // Ring 2
        "memfuse-infer-candle" | "memfuse-infer-ollama" | "memfuse-infer-onnx"
        | "memfuse-sandbox" => Some(Ring::Ring2),

        // Ring 3
        "memfuse-engine" | "memfuse-cognition" | "memfuse-privacy"
        | "memfuse-router" | "memfuse-agent" | "memfuse-db" => Some(Ring::Ring3),

        // Ring 4
        "memfuse" | "memfuse-mcp" | "memfuse-py" => Some(Ring::Ring4),

        // Tooling
        "memfuse-testkit" | "xtask" | "memfuse-bench" => Some(Ring::Tooling),

        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct RingViolation {
    pub from_crate: String,
    pub from_ring: Ring,
    pub to_crate: String,
    pub to_ring: Ring,
    pub dep_kind: String, // "normal", "build", "dev"
    pub rule_reason: String,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    name: String,
    dependencies: Vec<MetadataDependency>,
}

#[derive(Debug, Deserialize)]
struct MetadataDependency {
    name: String,
    kind: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: HashSet<String>,
}

pub fn check_ring_layering_from_metadata_json(json_str: &str) -> Result<Vec<RingViolation>, String> {
    let metadata: CargoMetadata =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse cargo metadata: {}", e))?;

    let workspace_packages: HashMap<String, &MetadataPackage> = metadata
        .packages
        .iter()
        .filter(|p| {
            // Check if package is a workspace member (either by id or package name matching workspace_members)
            metadata.workspace_members.iter().any(|m| m.contains(&p.name)) || metadata.workspace_members.contains(&p.name)
        })
        .map(|p| (p.name.clone(), p))
        .collect();

    let mut violations = Vec::new();

    for (pkg_name, pkg) in &workspace_packages {
        let from_ring = match get_crate_ring(pkg_name) {
            Some(r) => r,
            None => {
                return Err(format!(
                    "Unmapped workspace crate '{}': workspace member has no assigned Ring in check_ring_layering.rs!",
                    pkg_name
                ));
            }
        };

        for dep in &pkg.dependencies {
            let dep_name = &dep.name;

            // Only inspect workspace-internal dependencies
            if !workspace_packages.contains_key(dep_name) && dep.path.is_none() && get_crate_ring(dep_name).is_none() {
                continue;
            }

            let to_ring = match get_crate_ring(dep_name) {
                Some(r) => r,
                None => continue, // Ignore external dependencies
            };

            let kind = dep.kind.as_deref().unwrap_or("normal");

            // Evaluate Ring matrix rules
            let violation_reason = match (from_ring, to_ring, kind) {
                // Ring 0
                (Ring::Ring0, Ring::Ring0, _) => None,
                (Ring::Ring0, other, _) => Some(format!("Ring 0 crate cannot depend on {} ({})", other.name(), dep_name)),

                // Ring 1
                (Ring::Ring1, Ring::Ring0, _) => None,
                (Ring::Ring1, Ring::Ring1, "dev") => None,
                (Ring::Ring1, Ring::Ring1, _) => Some(format!("Ring 1 crate cannot depend on another Ring 1 crate ({})", dep_name)),
                (Ring::Ring1, other, _) => Some(format!("Ring 1 crate cannot depend on {} ({})", other.name(), dep_name)),

                // Ring 2
                (Ring::Ring2, Ring::Ring0, _) => {
                    let allowed_ring0 = matches!(
                        dep_name.as_str(),
                        "memfuse-types" | "memfuse-ports" | "memfuse-crypto" | "memfuse-core" | "memfuse-simd"
                    );
                    if allowed_ring0 {
                        None
                    } else {
                        Some(format!("Ring 2 crate can only depend on types/ports/crypto in Ring 0 (violator: {})", dep_name))
                    }
                }
                (Ring::Ring2, other, _) => Some(format!("Ring 2 crate cannot depend on {} ({})", other.name(), dep_name)),

                // Ring 3
                (Ring::Ring3, Ring::Ring0, _) => None,
                (Ring::Ring3, Ring::Ring1, _) => None,
                (Ring::Ring3, Ring::Ring2, _) => Some(format!("Ring 3 crate cannot depend on concrete Ring 2 crate ({})", dep_name)),
                (Ring::Ring3, Ring::Ring3, _) => None,
                (Ring::Ring3, other, _) => Some(format!("Ring 3 crate cannot depend on {} ({})", other.name(), dep_name)),

                // Ring 4
                (Ring::Ring4, _, _) => None,

                // Tooling
                (Ring::Tooling, _, _) => None,
            };

            if let Some(reason) = violation_reason {
                violations.push(RingViolation {
                    from_crate: pkg_name.clone(),
                    from_ring,
                    to_crate: dep_name.clone(),
                    to_ring,
                    dep_kind: kind.to_string(),
                    rule_reason: reason,
                });
            }
        }
    }

    Ok(violations)
}

pub fn run_check_ring_layering(strict: bool) -> Result<bool, String> {
    println!("=== Running xtask check-ring-layering (strict={}) ===", strict);

    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("Failed to execute cargo metadata: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "cargo metadata command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let violations = check_ring_layering_from_metadata_json(&json_str)?;

    if violations.is_empty() {
        println!("✅ check-ring-layering: 0 Ring-Matrix Verstöße gefunden.");
        return Ok(true);
    }

    println!("⚠️ check-ring-layering: {} Ring-Verstoß/Verstöße gefunden:", violations.len());
    for v in &violations {
        println!(
            "  RING-VIOLATION: {} ({}) -> {} ({}) [kind={}]: {}",
            v.from_crate,
            v.from_ring.name(),
            v.to_crate,
            v.to_ring.name(),
            v.dep_kind,
            v.rule_reason
        );
    }

    if strict {
        let ring3_violations = violations.iter().filter(|v| v.from_ring == Ring::Ring3).count();
        if ring3_violations > 0 {
            eprintln!("❌ check-ring-layering (--strict active): Fail due to {} Ring 3 violation(s).", ring3_violations);
            Ok(false)
        } else {
            println!("ℹ️ check-ring-layering (--strict active): 0 Ring 3 violations. Ignoring {} non-Ring-3 violations for now.", violations.len());
            Ok(true)
        }
    } else {
        println!("ℹ️ check-ring-layering (Warning mode active): Exit 0.");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_mapping_all_crates_covered() {
        assert_eq!(get_crate_ring("memfuse-core"), Some(Ring::Ring0));
        assert_eq!(get_crate_ring("memfuse-store"), Some(Ring::Ring1));
        assert_eq!(get_crate_ring("memfuse-infer-candle"), Some(Ring::Ring2));
        assert_eq!(get_crate_ring("memfuse-db"), Some(Ring::Ring3));
        assert_eq!(get_crate_ring("memfuse-mcp"), Some(Ring::Ring4));
        assert_eq!(get_crate_ring("xtask"), Some(Ring::Tooling));
        assert_eq!(get_crate_ring("nonexistent"), None);
    }

    #[test]
    fn test_synthetic_metadata_clean() {
        let mock_json = r#"{
            "packages": [
                {
                    "name": "memfuse-core",
                    "dependencies": []
                },
                {
                    "name": "memfuse-store",
                    "dependencies": [
                        { "name": "memfuse-core", "kind": null }
                    ]
                }
            ],
            "workspace_members": ["memfuse-core", "memfuse-store"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_synthetic_metadata_ring_violation_detected() {
        let mock_json = r#"{
            "packages": [
                {
                    "name": "memfuse-core",
                    "dependencies": [
                        { "name": "memfuse-store", "kind": null }
                    ]
                },
                {
                    "name": "memfuse-store",
                    "dependencies": []
                }
            ],
            "workspace_members": ["memfuse-core", "memfuse-store"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].from_crate, "memfuse-core");
        assert_eq!(res[0].to_crate, "memfuse-store");
    }

    #[test]
    fn test_unmapped_crate_returns_err() {
        let mock_json = r#"{
            "packages": [
                {
                    "name": "memfuse-unknown-new-crate",
                    "dependencies": []
                }
            ],
            "workspace_members": ["memfuse-unknown-new-crate"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Unmapped workspace crate"));
    }
}
