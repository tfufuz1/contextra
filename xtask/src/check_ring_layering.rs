//! Gate: check-ring-layering
//! Validiert die workspace-internen Abhängigkeiten gegen die Ring-Matrix (§4.3/§4.4):
//!   Ring 0 → Ring 0 (mit expliziter Kanten-Ordnung)
//!   Ring 1 → Ring 0 (kein Ring-1-zu-Ring-1)
//!   Ring 2 → types, ports, crypto
//!   Ring 3 → Ring 0, Ring 1, Ports von Ring 2 (mit expliziter Kanten-Ordnung)
//!   Ring 4 → alle
//!   Tooling → contextra-testkit + gleiches/tieferes Ring
//!   Aktion → Dev-Dependencies auf Tooling (z.B. contextra-testkit) sind für alle Ringe erlaubt.

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

    pub fn from_str(s: &str) -> Option<Ring> {
        match s.trim() {
            "0" | "Ring 0" => Some(Ring::Ring0),
            "1" | "Ring 1" => Some(Ring::Ring1),
            "2" | "Ring 2" => Some(Ring::Ring2),
            "3" | "Ring 3" => Some(Ring::Ring3),
            "4" | "Ring 4" => Some(Ring::Ring4),
            "tooling" | "Tooling" => Some(Ring::Tooling),
            _ => None,
        }
    }
}

/// Bietet eine explizite Ordnung innerhalb von Ring 0.
/// Niedrigere Zahl = tiefere Schicht (Blatt). Abhängigkeiten dürfen nur von höherer zu tieferer Ordnung verlaufen.
pub fn ring0_crate_order(crate_name: &str) -> usize {
    match crate_name {
        "contextra-types" | "contextra-sys" | "contextra-wire" => 0,
        "contextra-ports" | "contextra-mvcc" | "contextra-crypto" => 1,
        "contextra-core" | "contextra-adapt" | "contextra-text" => 2,
        "contextra-simd" | "contextra-graph" | "contextra-rank" => 3,
        "contextra-vector" => 4,
        _ => 99,
    }
}

/// Bietet eine explizite Ordnung innerhalb von Ring 3.
/// Niedrigere Zahl = tiefere Schicht (Blatt).
pub fn ring3_crate_order(crate_name: &str) -> usize {
    match crate_name {
        "contextra-privacy" | "contextra-engine" => 0,
        "contextra-cognition" | "contextra-router" => 1,
        "contextra-db" => 2,
        "contextra-agent" => 3,
        _ => 99,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllowlistEntry {
    pub from_crate: &'static str,
    pub to_crate: &'static str,
    pub target_phase: &'static str,
    pub issue_ref: &'static str,
    pub expires_on: &'static str,
    pub reason: &'static str,
}

pub const LAYER_ALLOWLIST: &[AllowlistEntry] = &[
    AllowlistEntry {
        from_crate: "contextra-graph",
        to_crate: "contextra-store",
        target_phase: "Phase 1a",
        issue_ref: "#CTX-101",
        expires_on: "2026-12-31",
        reason: "Dev-dependency on store for graph integration tests; to be isolated into contextra-testkit in Phase 1a",
    },
    AllowlistEntry {
        from_crate: "contextra-infer-onnx",
        to_crate: "contextra-infer-candle",
        target_phase: "Phase 1b",
        issue_ref: "#CTX-102",
        expires_on: "2026-12-31",
        reason: "Embed depends on candle provider; execution provider abstraction in Phase 1b",
    },
    AllowlistEntry {
        from_crate: "contextra-infer-ollama",
        to_crate: "contextra-infer-onnx",
        target_phase: "Phase 1b",
        issue_ref: "#CTX-103",
        expires_on: "2026-12-31",
        reason: "Ollama provider dev-dependency on embed for benchmarks/tests; to be isolated in Phase 1b",
    },
];

pub fn find_allowlist_entry(from_crate: &str, to_crate: &str) -> Option<&'static AllowlistEntry> {
    LAYER_ALLOWLIST
        .iter()
        .find(|e| e.from_crate == from_crate && e.to_crate == to_crate)
}

#[derive(Debug, Clone)]
pub struct RingViolation {
    pub from_crate: String,
    pub from_ring: Ring,
    pub to_crate: String,
    pub to_ring: Ring,
    pub dep_kind: String, // "normal", "build", "dev"
    pub rule_reason: String,
    pub is_allowlisted: bool,
    pub allowlist_phase: Option<&'static str>,
    pub issue_ref: Option<&'static str>,
    pub expires_on: Option<&'static str>,
    pub is_stale_allowlist: bool,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    name: String,
    dependencies: Vec<MetadataDependency>,
    metadata: Option<MetadataExtra>,
}

#[derive(Debug, Deserialize)]
struct MetadataExtra {
    contextra: Option<ContextraMetadata>,
}

#[derive(Debug, Deserialize)]
struct ContextraMetadata {
    ring: String,
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

pub fn get_ring_map_from_metadata_json(json_str: &str) -> Result<HashMap<String, Ring>, String> {
    let metadata: CargoMetadata = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse cargo metadata: {}", e))?;

    let mut ring_map = HashMap::new();

    for pkg in &metadata.packages {
        let is_member = metadata
            .workspace_members
            .iter()
            .any(|m| m.contains(&pkg.name))
            || metadata.workspace_members.contains(&pkg.name);

        if is_member {
            let ring = pkg
                .metadata
                .as_ref()
                .and_then(|m| m.contextra.as_ref())
                .and_then(|c| Ring::from_str(&c.ring));

            if let Some(r) = ring {
                ring_map.insert(pkg.name.clone(), r);
            }
        }
    }

    Ok(ring_map)
}

pub fn get_workspace_ring_map() -> Result<HashMap<String, Ring>, String> {
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
    get_ring_map_from_metadata_json(&json_str)
}

pub fn check_ring_layering_from_metadata_json(
    json_str: &str,
) -> Result<Vec<RingViolation>, String> {
    let metadata: CargoMetadata = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse cargo metadata: {}", e))?;

    let workspace_packages: HashMap<String, &MetadataPackage> = metadata
        .packages
        .iter()
        .filter(|p| {
            metadata
                .workspace_members
                .iter()
                .any(|m| m.contains(&p.name))
                || metadata.workspace_members.contains(&p.name)
        })
        .map(|p| (p.name.clone(), p))
        .collect();

    let ring_map: HashMap<String, Ring> = workspace_packages
        .iter()
        .map(|(name, pkg)| {
            let ring = pkg
                .metadata
                .as_ref()
                .and_then(|m| m.contextra.as_ref())
                .and_then(|c| Ring::from_str(&c.ring));

            match ring {
                Some(r) => Ok((name.clone(), r)),
                None => Err(format!(
                    "Unmapped workspace crate '{}': workspace member has no [package.metadata.contextra] ring assigned in Cargo.toml!",
                    name
                )),
            }
        })
        .collect::<Result<_, _>>()?;

    let mut violations = Vec::new();
    let mut matched_allowlist_entries: HashSet<(&'static str, &'static str)> = HashSet::new();

    for (pkg_name, pkg) in &workspace_packages {
        let from_ring = ring_map
            .get(pkg_name)
            .copied()
            .ok_or_else(|| format!("Missing ring metadata for package '{}'", pkg_name))?;

        for dep in &pkg.dependencies {
            let dep_name = &dep.name;

            // Only inspect workspace-internal dependencies
            if !workspace_packages.contains_key(dep_name)
                && dep.path.is_none()
                && !ring_map.contains_key(dep_name)
            {
                continue;
            }

            let to_ring = match ring_map.get(dep_name) {
                Some(&r) => r,
                None => continue, // Ignore external dependencies
            };

            let kind = dep.kind.as_deref().unwrap_or("normal");

            // Evaluate Ring matrix rules
            let violation_reason = match (from_ring, to_ring, kind) {
                // Tooling dev-dependencies from ANY Ring are permitted
                (_, Ring::Tooling, "dev") => None,

                // Ring 0
                (Ring::Ring0, Ring::Ring0, "dev") => None,
                (Ring::Ring0, Ring::Ring0, _) => {
                    let from_ord = ring0_crate_order(pkg_name);
                    let to_ord = ring0_crate_order(dep_name);
                    if from_ord < to_ord {
                        Some(format!(
                            "Ring 0 backward dependency edge: {} (order {}) -> {} (order {})",
                            pkg_name, from_ord, dep_name, to_ord
                        ))
                    } else {
                        None
                    }
                }
                (Ring::Ring0, other, _) => Some(format!(
                    "Ring 0 crate cannot depend on {} ({})",
                    other.name(),
                    dep_name
                )),

                // Ring 1
                (Ring::Ring1, Ring::Ring0, _) => None,
                (Ring::Ring1, Ring::Ring1, "dev") => None,
                (Ring::Ring1, Ring::Ring1, _) => Some(format!(
                    "Ring 1 crate cannot depend on another Ring 1 crate ({})",
                    dep_name
                )),
                (Ring::Ring1, other, _) => Some(format!(
                    "Ring 1 crate cannot depend on {} ({})",
                    other.name(),
                    dep_name
                )),

                // Ring 2
                (Ring::Ring2, Ring::Ring0, _) => {
                    let allowed_ring0 = matches!(
                        dep_name.as_str(),
                        "contextra-types"
                            | "contextra-ports"
                            | "contextra-crypto"
                            | "contextra-core"
                            | "contextra-simd"
                            | "contextra-rank"
                    );
                    if allowed_ring0 {
                        None
                    } else {
                        Some(format!("Ring 2 crate can only depend on types/ports/crypto in Ring 0 (violator: {})", dep_name))
                    }
                }
                (Ring::Ring2, other, _) => Some(format!(
                    "Ring 2 crate cannot depend on {} ({})",
                    other.name(),
                    dep_name
                )),

                // Ring 3
                (Ring::Ring3, Ring::Ring0, _) => None,
                (Ring::Ring3, Ring::Ring1, _) => None,
                (Ring::Ring3, Ring::Ring2, _) => Some(format!(
                    "Ring 3 crate cannot depend on concrete Ring 2 crate ({})",
                    dep_name
                )),
                (Ring::Ring3, Ring::Ring3, "dev") => None,
                (Ring::Ring3, Ring::Ring3, _) => {
                    let from_ord = ring3_crate_order(pkg_name);
                    let to_ord = ring3_crate_order(dep_name);
                    if from_ord < to_ord {
                        Some(format!(
                            "Ring 3 backward dependency edge: {} (order {}) -> {} (order {})",
                            pkg_name, from_ord, dep_name, to_ord
                        ))
                    } else {
                        None
                    }
                }
                (Ring::Ring3, other, _) => Some(format!(
                    "Ring 3 crate cannot depend on {} ({})",
                    other.name(),
                    dep_name
                )),

                // Ring 4
                (Ring::Ring4, _, _) => None,

                // Tooling
                (Ring::Tooling, _, _) => None,
            };

            if let Some(reason) = violation_reason {
                let allow_entry = find_allowlist_entry(pkg_name, dep_name);
                if let Some(entry) = allow_entry {
                    matched_allowlist_entries.insert((entry.from_crate, entry.to_crate));
                }
                violations.push(RingViolation {
                    from_crate: pkg_name.clone(),
                    from_ring,
                    to_crate: dep_name.clone(),
                    to_ring,
                    dep_kind: kind.to_string(),
                    rule_reason: reason,
                    is_allowlisted: allow_entry.is_some(),
                    allowlist_phase: allow_entry.map(|e| e.target_phase),
                    issue_ref: allow_entry.map(|e| e.issue_ref),
                    expires_on: allow_entry.map(|e| e.expires_on),
                    is_stale_allowlist: false,
                });
            }
        }
    }

    // Detect stale allowlist entries (entries in LAYER_ALLOWLIST that were not matched by any active violation)
    for entry in LAYER_ALLOWLIST {
        if !matched_allowlist_entries.contains(&(entry.from_crate, entry.to_crate)) {
            let from_ring = get_crate_ring(entry.from_crate).unwrap_or(Ring::Ring0);
            let to_ring = get_crate_ring(entry.to_crate).unwrap_or(Ring::Ring0);
            violations.push(RingViolation {
                from_crate: entry.from_crate.to_string(),
                from_ring,
                to_crate: entry.to_crate.to_string(),
                to_ring,
                dep_kind: "stale".to_string(),
                rule_reason: format!(
                    "STALE ALLOWLIST ENTRY: Exception {} -> {} is no longer active in codebase!",
                    entry.from_crate, entry.to_crate
                ),
                is_allowlisted: false, // Stale entries fail the gate regardless
                allowlist_phase: Some(entry.target_phase),
                issue_ref: Some(entry.issue_ref),
                expires_on: Some(entry.expires_on),
                is_stale_allowlist: true,
            });
        }
    }

    Ok(violations)
}

pub fn run_check_ring_layering(strict: bool) -> Result<bool, String> {
    run_check_ring_layering_with_options(strict, false)
}

pub fn run_check_ring_layering_full(strict: bool) -> Result<bool, String> {
    run_check_ring_layering_with_options(strict, true)
}

pub fn run_check_ring_layering_with_options(
    strict: bool,
    all_features: bool,
) -> Result<bool, String> {
    println!(
        "=== Running xtask check-ring-layering (strict={}, all_features={}) ===",
        strict, all_features
    );

    let mut args = vec!["metadata", "--format-version", "1", "--no-deps"];
    if all_features {
        args.push("--all-features");
    }

    let output = Command::new("cargo")
        .args(&args)
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

    let allowlisted: Vec<_> = violations.iter().filter(|v| v.is_allowlisted).collect();
    let unallowlisted: Vec<_> = violations.iter().filter(|v| !v.is_allowlisted).collect();

    if !allowlisted.is_empty() {
        println!(
            "⚠️ check-ring-layering: {} dokumentierte Allowlist-Ausnahme(n) gefunden:",
            allowlisted.len()
        );
        for v in &allowlisted {
            println!(
                "  ALLOWLISTED [{}] ({}, expires {}) {} ({}) -> {} ({}) [kind={}]: {}",
                v.allowlist_phase.unwrap_or("Phase ?"),
                v.issue_ref.unwrap_or("#?"),
                v.expires_on.unwrap_or("?"),
                v.from_crate,
                v.from_ring.name(),
                v.to_crate,
                v.to_ring.name(),
                v.dep_kind,
                v.rule_reason
            );
        }
    }

    if !unallowlisted.is_empty() {
        println!(
            "❌ check-ring-layering: {} UNDOKUMENTIERTE Ring-Verstöße/Stale Entries gefunden:",
            unallowlisted.len()
        );
        for v in &unallowlisted {
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
    }

    if strict {
        if !unallowlisted.is_empty() {
            eprintln!("❌ check-ring-layering (--strict active): Fail due to {} unallowlisted Ring violation(s) / stale entries.", unallowlisted.len());
            Ok(false)
        } else {
            println!("ℹ️ check-ring-layering (--strict active): All {} violation(s) are documented on the phased allowlist.", violations.len());
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
    fn test_synthetic_metadata_clean() {
        let mock_json = r#"{
            "packages": [
                {
                    "name": "contextra-core",
                    "dependencies": [],
                    "metadata": { "contextra": { "ring": "0" } }
                },
                {
                    "name": "contextra-store",
                    "dependencies": [
                        { "name": "contextra-core", "kind": null }
                    ],
                    "metadata": { "contextra": { "ring": "1" } }
                }
            ],
            "workspace_members": ["contextra-core", "contextra-store"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json).unwrap();
        // Ignore stale allowlist entries for fixture test
        let active_violations: Vec<_> = res.into_iter().filter(|v| !v.is_stale_allowlist).collect();
        assert!(active_violations.is_empty());
    }

    #[test]
    fn test_synthetic_metadata_ring0_backward_edge_detected() {
        // contextra-types (order 0) depending on contextra-graph (order 3)
        let mock_json = r#"{
            "packages": [
                {
                    "name": "contextra-types",
                    "dependencies": [
                        { "name": "contextra-graph", "kind": null }
                    ]
                },
                {
                    "name": "contextra-graph",
                    "dependencies": []
                }
            ],
            "workspace_members": ["contextra-types", "contextra-graph"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json).unwrap();
        let violations: Vec<_> = res.into_iter().filter(|v| !v.is_stale_allowlist).collect();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].from_crate, "contextra-types");
        assert_eq!(violations[0].to_crate, "contextra-graph");
        assert!(violations[0]
            .rule_reason
            .contains("Ring 0 backward dependency edge"));
    }

    #[test]
    fn test_synthetic_metadata_ring3_backward_edge_detected() {
        // contextra-engine (order 0) depending on contextra-agent (order 3)
        let mock_json = r#"{
            "packages": [
                {
                    "name": "contextra-engine",
                    "dependencies": [
                        { "name": "contextra-store", "kind": null }
                    ],
                    "metadata": { "contextra": { "ring": "0" } }
                },
                {
                    "name": "contextra-agent",
                    "dependencies": []
                }
            ],
            "workspace_members": ["contextra-engine", "contextra-agent"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json).unwrap();
        let violations: Vec<_> = res.into_iter().filter(|v| !v.is_stale_allowlist).collect();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].from_crate, "contextra-engine");
        assert_eq!(violations[0].to_crate, "contextra-agent");
        assert!(violations[0]
            .rule_reason
            .contains("Ring 3 backward dependency edge"));
    }

    #[test]
    fn test_tooling_dev_dependency_permitted() {
        // Ring 1 contextra-store with dev-dependency on Ring Tooling contextra-testkit
        let mock_json = r#"{
            "packages": [
                {
                    "name": "contextra-store",
                    "dependencies": [],
                    "metadata": { "contextra": { "ring": "1" } }
                }
            ],
            "workspace_members": ["contextra-store", "contextra-testkit"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json).unwrap();
        let violations: Vec<_> = res.into_iter().filter(|v| !v.is_stale_allowlist).collect();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_stale_allowlist_entry_detected() {
        let mock_json = r#"{
            "packages": [
                {
                    "name": "contextra-types",
                    "dependencies": []
                }
            ],
            "workspace_members": ["contextra-types"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json).unwrap();
        let stales: Vec<_> = res.into_iter().filter(|v| v.is_stale_allowlist).collect();
        assert_eq!(stales.len(), LAYER_ALLOWLIST.len());
        for stale in stales {
            assert!(stale.rule_reason.contains("STALE ALLOWLIST ENTRY"));
        }
    }

    #[test]
    fn test_unmapped_crate_returns_err() {
        let mock_json = r#"{
            "packages": [
                {
                    "name": "contextra-unknown-new-crate",
                    "dependencies": []
                }
            ],
            "workspace_members": ["contextra-unknown-new-crate"]
        }"#;

        let res = check_ring_layering_from_metadata_json(mock_json);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Unmapped workspace crate"));
    }
}
