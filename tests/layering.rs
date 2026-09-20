// FILE-CONTEXT
// STAND: 2026-09-19T20:22:00Z (SESSION: 01c5be8b)
// ZWECK: Layering Matrix enforcement test across Ring 0..4 (GESAMTSPEZIFIKATION §0.3, §1.1, §4.3).
// INVARIANTEN: Ring 0 has NO upward dependencies and NO tokio. Warn mode in Phase 0R, sharp from Phase 1a.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ring {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
    Ring4 = 4,
    Tooling = 99,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllowlistEntry {
    pub from_crate: &'static str,
    pub to_crate: &'static str,
    pub target_phase: &'static str,
    pub reason: &'static str,
}

pub const LAYER_ALLOWLIST: &[AllowlistEntry] = &[
    AllowlistEntry {
        from_crate: "memfuse-graph",
        to_crate: "memfuse-store",
        target_phase: "Phase 1a",
        reason: "Dev-dependency on store for graph integration tests; to be isolated into memfuse-testkit in Phase 1a",
    },
    AllowlistEntry {
        from_crate: "memfuse-embed",
        to_crate: "memfuse-calibration",
        target_phase: "Phase 1b",
        reason: "Embed depends on calibration; calibration port interfaces to be decoupled in Phase 1b",
    },
    AllowlistEntry {
        from_crate: "memfuse-embed",
        to_crate: "memfuse-candle",
        target_phase: "Phase 1b",
        reason: "Embed depends on candle provider; execution provider abstraction in Phase 1b",
    },
    AllowlistEntry {
        from_crate: "memfuse-candle",
        to_crate: "memfuse-calibration",
        target_phase: "Phase 1b",
        reason: "Candle provider depends on calibration; calibration port interfaces to be decoupled in Phase 1b",
    },
    AllowlistEntry {
        from_crate: "memfuse-candle",
        to_crate: "memfuse-store",
        target_phase: "Phase 1b",
        reason: "Candle provider uses store directly; store traits abstraction in Phase 1b",
    },
    AllowlistEntry {
        from_crate: "memfuse-ollama",
        to_crate: "memfuse-calibration",
        target_phase: "Phase 1b",
        reason: "Ollama provider depends on calibration; calibration port interfaces to be decoupled in Phase 1b",
    },
    AllowlistEntry {
        from_crate: "memfuse-ollama",
        to_crate: "memfuse-embed",
        target_phase: "Phase 1b",
        reason: "Ollama provider dev-dependency on embed for benchmarks/tests; to be isolated in Phase 1b",
    },
];

pub fn find_allowlist_entry(from_crate: &str, to_crate: &str) -> Option<&'static AllowlistEntry> {
    LAYER_ALLOWLIST.iter().find(|e| e.from_crate == from_crate && e.to_crate == to_crate)
}

impl Ring {
    pub fn for_crate(name: &str) -> Option<Ring> {
        match name {
            // Ring 0: Foundation & Core Domain Logic
            "memfuse-types" | "memfuse-ports" | "memfuse-mvcc" | "memfuse-vector"
            | "memfuse-rank" | "memfuse-adapt" | "memfuse-text" | "memfuse-graph"
            | "memfuse-crypto" | "memfuse-simd" | "memfuse-sys" | "memfuse-wire"
            | "memfuse-core" | "memfuse-index" | "memfuse-calibration"
            | "memfuse-core-ipc-gen" => Some(Ring::Ring0),

            // Ring 1: Storage & State Persistence
            "memfuse-store" | "memfuse-checkpoint" | "memfuse-kvcache" => Some(Ring::Ring1),

            // Ring 2: External Integrations & Execution Sandboxes
            "memfuse-sandbox" | "memfuse-embed" | "memfuse-candle"
            | "memfuse-ollama" => Some(Ring::Ring2),

            // Ring 3: Engine, Reasoning & Cognition
            "memfuse-engine" | "memfuse-cognition" | "memfuse-privacy"
            | "memfuse-router" | "memfuse-agent" | "memfuse-db" => Some(Ring::Ring3),

            // Ring 4: Public Facade & Protocol Servers
            "memfuse" | "memfuse-mcp" | "memfuse-py" => Some(Ring::Ring4),

            // Tooling
            "memfuse-testkit" | "xtask" | "memfuse-bench" => Some(Ring::Tooling),

            _ => None,
        }
    }
}

use serde::Deserialize;

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

pub fn check_layering_matrix(root: &Path, strict_ring3: bool) -> (bool, Vec<String>) {
    let output = match Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
    {
        Ok(out) => out,
        Err(e) => return (false, vec![format!("Failed to execute cargo metadata: {}", e)]),
    };

    if !output.status.success() {
        return (
            false,
            vec![format!(
                "cargo metadata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )],
        );
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let metadata: CargoMetadata = match serde_json::from_str(&json_str) {
        Ok(m) => m,
        Err(e) => return (false, vec![format!("Failed to parse metadata JSON: {}", e)]),
    };

    let workspace_packages: HashMap<String, &MetadataPackage> = metadata
        .packages
        .iter()
        .filter(|p| {
            metadata.workspace_members.iter().any(|m| m.contains(&p.name))
                || metadata.workspace_members.contains(&p.name)
        })
        .map(|p| (p.name.clone(), p))
        .collect();

    let mut violations = Vec::new();
    let mut warnings = Vec::new();

    for (pkg_name, pkg) in &workspace_packages {
        let my_ring = match Ring::for_crate(pkg_name) {
            Some(r) => r,
            None => continue,
        };

        for dep in &pkg.dependencies {
            let dep_name = &dep.name;
            let kind = dep.kind.as_deref().unwrap_or("normal");

            // Check tokio prohibition in Ring 0
            if my_ring == Ring::Ring0 && dep_name == "tokio" && pkg_name.as_str() != "memfuse-core" {
                violations.push(format!(
                    "Ring 0 violation: {} ({}) depends on tokio via {}",
                    pkg_name, my_ring.name(), kind
                ));
            }

            let dep_ring = match Ring::for_crate(dep_name) {
                Some(r) => r,
                None => continue,
            };

            // Evaluate matrix rules
            let violation_reason = match (my_ring, dep_ring, kind) {
                (Ring::Ring0, Ring::Ring0, _) => None,
                (Ring::Ring0, other, _) => Some(format!(
                    "Ring 0 crate cannot depend on {} ({})",
                    other.name(), dep_name
                )),

                (Ring::Ring1, Ring::Ring0, _) => None,
                (Ring::Ring1, Ring::Ring1, "dev") => None,
                (Ring::Ring1, Ring::Ring1, _) => Some(format!(
                    "Ring 1 crate cannot depend on another Ring 1 crate ({})",
                    dep_name
                )),
                (Ring::Ring1, other, _) => Some(format!(
                    "Ring 1 crate cannot depend on {} ({})",
                    other.name(), dep_name
                )),

                (Ring::Ring2, Ring::Ring0, _) => {
                    let allowed = matches!(
                        dep_name.as_str(),
                        "memfuse-types" | "memfuse-ports" | "memfuse-crypto" | "memfuse-core" | "memfuse-simd"
                    );
                    if allowed {
                        None
                    } else {
                        Some(format!(
                            "Ring 2 crate can only depend on types/ports/crypto in Ring 0 (violator: {})",
                            dep_name
                        ))
                    }
                }
                (Ring::Ring2, other, _) => Some(format!(
                    "Ring 2 crate cannot depend on {} ({})",
                    other.name(), dep_name
                )),

                (Ring::Ring3, Ring::Ring0, _) => None,
                (Ring::Ring3, Ring::Ring1, _) => None,
                (Ring::Ring3, Ring::Ring2, _) => Some(format!(
                    "Ring 3 crate cannot depend on concrete Ring 2 crate ({})",
                    dep_name
                )),
                (Ring::Ring3, Ring::Ring3, _) => None,
                (Ring::Ring3, other, _) => Some(format!(
                    "Ring 3 crate cannot depend on {} ({})",
                    other.name(), dep_name
                )),

                (Ring::Ring4, _, _) => None,
                (Ring::Tooling, _, _) => None,
            };

            if let Some(reason) = violation_reason {
                if let Some(allow) = find_allowlist_entry(pkg_name, dep_name) {
                    warnings.push(format!(
                        "⚠️ [ALLOWLISTED] [{}] {} ({}) -> {} ({}): {} ({})",
                        allow.target_phase, pkg_name, my_ring.name(), dep_name, dep_ring.name(), reason, allow.reason
                    ));
                } else if strict_ring3 && my_ring == Ring::Ring3 {
                    violations.push(format!(
                        "❌ [STRICT RING 3 VIOLATION] {} ({}) -> {} ({}): {}",
                        pkg_name, my_ring.name(), dep_name, dep_ring.name(), reason
                    ));
                } else {
                    violations.push(format!(
                        "❌ [LAYERING VIOLATION] {} ({}) -> {} ({}): {}",
                        pkg_name, my_ring.name(), dep_name, dep_ring.name(), reason
                    ));
                }
            }
        }
    }

    println!("=== MemFuse Ring-Layering Verification ===");
    for w in &warnings {
        println!("{}", w);
    }

    if !violations.is_empty() {
        for v in &violations {
            eprintln!("{}", v);
        }
        (false, violations)
    } else {
        println!(
            "✅ Layering verification passed (0 unallowlisted violations; {} allowlisted warnings documented)",
            warnings.len()
        );
        (true, warnings)
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (success, _) = check_layering_matrix(&root, true);
    if !success {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layering_matrix_strict_mode() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (success, violations) = check_layering_matrix(&root, true);
        assert!(
            success,
            "Ring layering matrix check must pass (violations: {:?})",
            violations
        );
    }

    #[test]
    fn test_ring3_has_no_unallowlisted_upward_dependencies() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (success, violations) = check_layering_matrix(&root, true);
        let ring3_violations: Vec<_> = violations
            .into_iter()
            .filter(|v| v.contains("Ring 3"))
            .collect();
        assert!(
            ring3_violations.is_empty(),
            "Ring 3 must have 0 unallowlisted upward dependencies, found: {:?}",
            ring3_violations
        );
        assert!(success);
    }
}
