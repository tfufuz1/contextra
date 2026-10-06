//! Harness Modul: Zyklenerkennung und Ring-Distanz-Report im Crate-Abhängigkeitsgraphen (dependency-graph-audit).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xtask::check_ring_layering::{get_ring_map_from_metadata_json, get_workspace_ring_map, Ring};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum Finding {
    Cycle {
        path: Vec<String>,
        message: String,
    },
    RingJump {
        from_crate: String,
        from_ring: u32,
        to_crate: String,
        to_ring: u32,
        distance: u32,
        message: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<Finding>,
}

pub fn build_graph_from_crates(root: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Err(format!(
            "Crates directory not found at {}",
            crates_dir.display()
        ));
    }

    let entries = fs::read_dir(&crates_dir).map_err(|e| {
        format!(
            "Failed to read crates directory {}: {}",
            crates_dir.display(),
            e
        )
    })?;

    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let cargo_toml_path = path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&cargo_toml_path)
            .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path.display(), e))?;

        let table: toml::Table = content
            .parse::<toml::Table>()
            .map_err(|e| format!("Failed to parse {}: {}", cargo_toml_path.display(), e))?;

        let pkg_name = table
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let crate_name = match pkg_name {
            Some(name) => name,
            None => continue,
        };

        let mut internal_deps = Vec::new();
        if let Some(deps_table) = table.get("dependencies").and_then(|d| d.as_table()) {
            for key in deps_table.keys() {
                if key.starts_with("contextra") {
                    internal_deps.push(key.clone());
                }
            }
        }
        internal_deps.sort();
        internal_deps.dedup();

        graph.insert(crate_name, internal_deps);
    }

    // Ensure all referenced target crates are present in graph as keys
    let mut missing_targets = Vec::new();
    for deps in graph.values() {
        for dep in deps {
            if !graph.contains_key(dep) {
                missing_targets.push(dep.clone());
            }
        }
    }
    for target in missing_targets {
        graph.entry(target).or_default();
    }

    Ok(graph)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeState {
    Unvisited,
    Visiting,
    Visited,
}

pub fn detect_cycles(graph: &BTreeMap<String, Vec<String>>) -> Vec<Finding> {
    let mut state: HashMap<String, NodeState> = HashMap::new();
    for node in graph.keys() {
        state.insert(node.clone(), NodeState::Unvisited);
    }

    let mut found_cycles: Vec<Vec<String>> = Vec::new();
    let mut seen_cycle_keys: HashSet<String> = HashSet::new();

    for start_node in graph.keys() {
        if state.get(start_node) == Some(&NodeState::Visited) {
            continue;
        }

        let mut stack: Vec<(String, usize)> = Vec::new();
        state.insert(start_node.clone(), NodeState::Visiting);
        stack.push((start_node.clone(), 0));

        while !stack.is_empty() {
            let (u, neighbor_idx) = match stack.last_mut() {
                Some(pair) => (pair.0.clone(), pair.1),
                None => break,
            };

            let empty_vec = Vec::new();
            let neighbors = graph.get(&u).unwrap_or(&empty_vec);

            if neighbor_idx < neighbors.len() {
                let v = neighbors[neighbor_idx].clone();
                if let Some(pair) = stack.last_mut() {
                    pair.1 += 1;
                }

                let v_state = state.get(&v).copied().unwrap_or(NodeState::Unvisited);

                if v_state == NodeState::Visiting {
                    if let Some(pos) = stack.iter().position(|(n, _)| n == &v) {
                        let mut cycle_path: Vec<String> =
                            stack[pos..].iter().map(|(n, _)| n.clone()).collect();
                        cycle_path.push(v.clone());

                        let cycle_key = cycle_path.join("->");
                        if seen_cycle_keys.insert(cycle_key) {
                            found_cycles.push(cycle_path);
                        }
                    }
                } else if v_state == NodeState::Unvisited {
                    state.insert(v.clone(), NodeState::Visiting);
                    stack.push((v, 0));
                }
            } else {
                state.insert(u, NodeState::Visited);
                stack.pop();
            }
        }
    }

    found_cycles
        .into_iter()
        .map(|path| {
            let message = format!("Cycle detected: {}", path.join(" -> "));
            Finding::Cycle { path, message }
        })
        .collect()
}

pub fn ring_to_u32(ring: Ring) -> Option<u32> {
    match ring {
        Ring::Ring0 => Some(0),
        Ring::Ring1 => Some(1),
        Ring::Ring2 => Some(2),
        Ring::Ring3 => Some(3),
        Ring::Ring4 => Some(4),
        Ring::Tooling => None,
    }
}

pub fn detect_ring_jumps(
    graph: &BTreeMap<String, Vec<String>>,
    ring_map: &HashMap<String, Ring>,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (from_crate, deps) in graph {
        let from_ring = match ring_map.get(from_crate) {
            Some(&r) => r,
            None => continue,
        };

        let from_ring_num = match ring_to_u32(from_ring) {
            Some(n) => n,
            None => continue,
        };

        for to_crate in deps {
            let to_ring = match ring_map.get(to_crate) {
                Some(&r) => r,
                None => continue,
            };

            let to_ring_num = match ring_to_u32(to_ring) {
                Some(n) => n,
                None => continue,
            };

            if from_ring_num >= to_ring_num {
                let distance = from_ring_num - to_ring_num;
                if distance > 1 {
                    let message = format!(
                        "Ring jump detected: {} ({}) -> {} ({}) with distance {}",
                        from_crate,
                        from_ring.name(),
                        to_crate,
                        to_ring.name(),
                        distance
                    );
                    findings.push(Finding::RingJump {
                        from_crate: from_crate.clone(),
                        from_ring: from_ring_num,
                        to_crate: to_crate.clone(),
                        to_ring: to_ring_num,
                        distance,
                        message,
                    });
                }
            }
        }
    }

    findings
}

pub fn run_dependency_graph_audit(args: &[String]) -> i32 {
    let mut root = PathBuf::from(".");
    let mut use_json = false;

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
                use_json = true;
            }
            _ => {}
        }
        i += 1;
    }

    let ring_map_res = if root == Path::new(".") || root == Path::new("") {
        get_workspace_ring_map()
    } else {
        let output = Command::new("cargo")
            .args(["metadata", "--format-version", "1", "--no-deps"])
            .current_dir(&root)
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let json_str = String::from_utf8_lossy(&out.stdout);
                get_ring_map_from_metadata_json(&json_str)
            }
            Ok(out) => Err(format!(
                "cargo metadata command failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )),
            Err(e) => Err(format!("Failed to execute cargo metadata: {}", e)),
        }
    };

    let ring_map = match ring_map_res {
        Ok(m) => m,
        Err(e) => {
            let output = AuditOutput {
                gate: "dependency-graph-audit".to_string(),
                status: "error".to_string(),
                summary: format!("Failed to obtain ring map: {}", e),
                findings: vec![],
            };
            if use_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).unwrap_or_default()
                );
            } else {
                eprintln!("Error: Failed to obtain ring map: {}", e);
            }
            return 2;
        }
    };

    let graph = match build_graph_from_crates(&root) {
        Ok(g) => g,
        Err(e) => {
            let output = AuditOutput {
                gate: "dependency-graph-audit".to_string(),
                status: "error".to_string(),
                summary: format!("Failed to build dependency graph: {}", e),
                findings: vec![],
            };
            if use_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).unwrap_or_default()
                );
            } else {
                eprintln!("Error: Failed to build dependency graph: {}", e);
            }
            return 2;
        }
    };

    let cycle_findings = detect_cycles(&graph);
    let ring_jump_findings = detect_ring_jumps(&graph, &ring_map);

    let has_cycles = !cycle_findings.is_empty();
    let status = if has_cycles { "fail" } else { "pass" };

    let summary = format!(
        "Dependency graph audit completed: {} cycle(s), {} ring jump(s) found across {} crates",
        cycle_findings.len(),
        ring_jump_findings.len(),
        graph.len()
    );

    let mut findings = cycle_findings;
    findings.extend(ring_jump_findings);

    let output = AuditOutput {
        gate: "dependency-graph-audit".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: findings.clone(),
    };

    if use_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!("{}", summary);
        for finding in &findings {
            match finding {
                Finding::Cycle { message, .. } => println!("  [CYCLE] {}", message),
                Finding::RingJump { message, .. } => println!("  [RING JUMP] {}", message),
            }
        }
    }

    if has_cycles {
        1
    } else {
        0
    }
}
