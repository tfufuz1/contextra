//! Harness Modul zur Berechnung des Auswirkungsbereichs (blast-radius).

use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_blast_radius(args: &[String]) -> i32 {
    let mut root_dir = blast_radius_find_repo_root();
    let mut base_rev = "origin/main".to_string();
    let mut head_rev = "HEAD".to_string();
    let mut use_json = false;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--base" => {
                if idx + 1 < args.len() {
                    base_rev = args[idx + 1].clone();
                    idx += 1;
                }
            }
            "--head" => {
                if idx + 1 < args.len() {
                    head_rev = args[idx + 1].clone();
                    idx += 1;
                }
            }
            "--json" => {
                use_json = true;
            }
            _ => {}
        }
        idx += 1;
    }

    let caps_path = root_dir.join("capabilities.toml");
    if !caps_path.exists() {
        let msg = "capabilities.toml nicht gefunden.";
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "blast-radius",
                    "status": "error",
                    "summary": msg,
                    "findings": []
                })
            );
        } else {
            eprintln!("FEHLER: {}", msg);
        }
        return 2;
    }

    let caps_content = match fs::read_to_string(&caps_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Konnte capabilities.toml nicht lesen: {}", e);
            if use_json {
                println!(
                    "{}",
                    json!({
                        "gate": "blast-radius",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER: {}", msg);
            }
            return 2;
        }
    };

    let parsed: Result<toml::Value, _> = toml::from_str(&caps_content);
    let table = match parsed {
        Ok(toml::Value::Table(t)) => t,
        _ => {
            let msg = "Ungueltiges TOML-Format in capabilities.toml.";
            if use_json {
                println!(
                    "{}",
                    json!({
                        "gate": "blast-radius",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER: {}", msg);
            }
            return 2;
        }
    };

    let crates_table = match table.get("crates").and_then(|v| v.as_table()) {
        Some(t) => t,
        None => {
            let msg = "Keine [crates] Tabelle in capabilities.toml gefunden.";
            if use_json {
                println!(
                    "{}",
                    json!({
                        "gate": "blast-radius",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER: {}", msg);
            }
            return 2;
        }
    };

    let mut dep_graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut reverse_graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut crate_rings: HashMap<String, String> = HashMap::new();
    let mut crate_tests: HashMap<String, String> = HashMap::new();

    for (crate_name, crate_val) in crates_table {
        let mut deps = Vec::new();
        let ring = crate_val.get("ring").and_then(|v| v.as_str()).unwrap_or("Ring ?").to_string();
        let test_cmd = crate_val.get("test").and_then(|v| v.as_str()).unwrap_or("").to_string();

        crate_rings.insert(crate_name.clone(), ring);
        crate_tests.insert(crate_name.clone(), test_cmd);

        if let Some(arr) = crate_val.get("may_depend_on").and_then(|v| v.as_array()) {
            for dep_val in arr {
                if let Some(dep) = dep_val.as_str() {
                    deps.push(dep.to_string());
                    reverse_graph.entry(dep.to_string()).or_default().push(crate_name.clone());
                }
            }
        }
        dep_graph.insert(crate_name.clone(), deps);
    }

    if blast_radius_has_cycle(&dep_graph) {
        let msg = "Zyklus im Abhängigkeitsgraph von capabilities.toml erkannt.";
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "blast-radius",
                    "status": "error",
                    "summary": msg,
                    "findings": [{
                        "id": "blast-radius-cycle",
                        "severity": "error",
                        "file": "capabilities.toml",
                        "line": 0,
                        "message": msg,
                        "fix": "Entferne den zyklischen Crate-Verweis in capabilities.toml."
                    }]
                })
            );
        } else {
            eprintln!("FEHLER: {}", msg);
        }
        return 2;
    }

    let changed_files = blast_radius_get_changed_files(&root_dir, &base_rev, &head_rev);
    let mut directly_changed_crates = HashSet::new();

    for file in &changed_files {
        if file.starts_with("crates/") {
            let parts: Vec<_> = file.split('/').collect();
            if parts.len() >= 2 {
                directly_changed_crates.insert(parts[1].to_string());
            }
        }
    }

    let mut affected_crates = HashSet::new();
    let mut queue: Vec<_> = directly_changed_crates.into_iter().collect();

    while let Some(c) = queue.pop() {
        if affected_crates.insert(c.clone()) {
            if let Some(dependents) = reverse_graph.get(&c) {
                for dep in dependents {
                    if !affected_crates.contains(dep) {
                        queue.push(dep.clone());
                    }
                }
            }
        }
    }

    let mut sorted_affected: Vec<_> = affected_crates.into_iter().collect();
    sorted_affected.sort();

    let mut affected_tests = Vec::new();
    for c in &sorted_affected {
        if let Some(t) = crate_tests.get(c) {
            if !t.is_empty() {
                affected_tests.push(t.clone());
            }
        }
    }
    affected_tests.sort();
    affected_tests.dedup();

    if use_json {
        println!(
            "{}",
            json!({
                "gate": "blast-radius",
                "status": "pass",
                "summary": format!("Betroffene Crates: {}", sorted_affected.len()),
                "findings": sorted_affected.iter().map(|c| json!({
                    "id": c,
                    "severity": "info",
                    "file": format!("crates/{}", c),
                    "line": 0,
                    "message": format!("Ring: {}", crate_rings.get(c).unwrap_or(&"Ring ?".to_string())),
                    "fix": crate_tests.get(c).cloned().unwrap_or_default()
                })).collect::<Vec<_>>()
            })
        );
    } else {
        println!("=== Auswirkungsbereich (Blast Radius) ===");
        println!("Betroffene Crates (transitiv):");
        for c in &sorted_affected {
            println!(" - {} ({})", c, crate_rings.get(c).unwrap_or(&"Ring ?".to_string()));
        }
        println!("\nAffected Tests:");
        for t in &affected_tests {
            println!(" - {}", t);
        }
    }

    0
}

fn blast_radius_find_repo_root() -> PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return PathBuf::from(path_str);
            }
        }
    }
    PathBuf::from(".")
}

fn blast_radius_get_changed_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", base, head])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

fn blast_radius_has_cycle(graph: &HashMap<String, Vec<String>>) -> bool {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for node in graph.keys() {
        if blast_radius_dfs_cycle(node, graph, &mut visited, &mut rec_stack) {
            return true;
        }
    }
    false
}

fn blast_radius_dfs_cycle(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    if rec_stack.contains(node) {
        return true;
    }
    if visited.contains(node) {
        return false;
    }

    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if blast_radius_dfs_cycle(neighbor, graph, visited, rec_stack) {
                return true;
            }
        }
    }

    rec_stack.remove(node);
    false
}
