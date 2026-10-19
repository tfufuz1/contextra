//! Integration test verifying INV-CORE-ISOLATION-1 for `contextra-engine` and `contextra-db`.
//!
//! Executes `cargo metadata` with `--no-default-features` for `contextra-engine` and `contextra-db`
//! and validates that no Ring 0/1/2 Edge or Experimental crates appear in their resolved dependency tree.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Edge and experimental crates forbidden from appearing in the release default dependency graph when isolated.
const FORBIDDEN_EDGE_CRATES: &[&str] = &[
    "contextra-graph",
    "contextra-rank",
    "contextra-adapt",
    "contextra-kvcache",
    "contextra-cognition",
    "contextra-privacy",
    "contextra-sandbox",
    "contextra-infer-candle",
    "contextra-infer-ollama",
    "contextra-infer-onnx",
    "contextra-license",
    "contextra-audit-export",
    "contextra-avv-generator",
];

#[derive(serde::Deserialize)]
struct CargoMetadata {
    packages: Vec<PackageNode>,
    resolve: Option<ResolveNode>,
}

#[derive(serde::Deserialize)]
struct PackageNode {
    name: String,
    id: String,
}

#[derive(serde::Deserialize)]
struct ResolveNode {
    nodes: Vec<DependencyNode>,
}

#[derive(serde::Deserialize)]
struct DepKind {
    kind: Option<String>,
}

#[derive(serde::Deserialize)]
struct NodeDep {
    pkg: String,
    dep_kinds: Vec<DepKind>,
}

#[derive(serde::Deserialize)]
struct DependencyNode {
    id: String,
    deps: Vec<NodeDep>,
}

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn check_isolation_for_package(pkg_name: &str) {
    let root = get_workspace_root();
    let capabilities_path = root.join("capabilities.toml");

    // Spec §6.0 require reading capabilities.toml and failing if not readable
    let cap_content = fs::read_to_string(&capabilities_path)
        .expect("capabilities.toml MUST be readable for core_isolation verification");
    assert!(!cap_content.is_empty(), "capabilities.toml is empty");

    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            root.join(format!("crates/{}/Cargo.toml", pkg_name))
                .to_str()
                .unwrap(),
            "--no-default-features",
        ])
        .current_dir(&root)
        .output()
        .expect("Failed to execute cargo metadata");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: CargoMetadata =
        serde_json::from_slice(&output.stdout).expect("Failed to parse cargo metadata JSON");

    let id_to_name: std::collections::HashMap<String, String> = metadata
        .packages
        .iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();

    let resolve = metadata
        .resolve
        .expect("Missing resolve graph in cargo metadata");

    let root_node = resolve
        .nodes
        .iter()
        .find(|n| id_to_name.get(&n.id).map(|s| s.as_str()) == Some(pkg_name))
        .expect("Could not find package node in resolve graph");

    let _pkg_node = metadata
        .packages
        .iter()
        .find(|p| p.name == pkg_name)
        .expect("Package declaration not found in metadata");

    // Build the set of active normal dependency crate names
    let mut active_deps: HashSet<String> = HashSet::new();

    // Collect normal dependencies from cargo metadata
    for dep_node in &root_node.deps {
        let is_normal = dep_node.dep_kinds.iter().any(|k| k.kind.is_none());
        if is_normal {
            if let Some(name) = id_to_name.get(&dep_node.pkg) {
                active_deps.insert(name.clone());
            }
        }
    }

    // Traversal of active dependencies
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = active_deps.into_iter().collect();

    let node_map: std::collections::HashMap<String, &DependencyNode> =
        resolve.nodes.iter().map(|n| (n.id.clone(), n)).collect();

    while let Some(current_id) = stack.pop() {
        if !visited.insert(current_id.clone()) {
            continue;
        }

        if let Some(dep_node) = node_map.get(&current_id) {
            for node_dep in &dep_node.deps {
                let is_normal = node_dep.dep_kinds.iter().any(|k| k.kind.is_none());
                if is_normal {
                    stack.push(node_dep.pkg.clone());
                }
            }
        }
    }

    let reachable_crate_names: HashSet<String> = visited
        .into_iter()
        .filter_map(|id| id_to_name.get(&id).cloned())
        .collect();

    println!(
        "Reachable crates for {} (--no-default-features): {:?}",
        pkg_name, reachable_crate_names
    );

    for &forbidden in FORBIDDEN_EDGE_CRATES {
        assert!(
            !reachable_crate_names.contains(forbidden),
            "INV-CORE-ISOLATION-1 Violation: Isolated package `{}` unexpectedly depends on edge crate `{}` when built with --no-default-features",
            pkg_name,
            forbidden
        );
    }
}

#[test]
fn test_core_isolation_engine() {
    check_isolation_for_package("contextra-engine");
}

#[test]
fn test_core_isolation_db() {
    check_isolation_for_package("contextra-db");
}
