// MemFuse — Ring Layering Gate (GESAMTSPEZIFIKATION §0.3, §1.1, §4.3, §20 Phase 0R)

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ring {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
    Ring4 = 4,
    Tooling = 99,
}

impl Ring {
    pub fn for_crate(name: &str) -> Option<Ring> {
        match name {
            // Ring 0: Types & Foundation
            "memfuse-types" | "memfuse-wire" | "memfuse-sys" | "memfuse-simd" | "memfuse-testkit" => {
                Some(Ring::Ring0)
            }
            // Ring 1: Storage & Specialized Compute
            "memfuse-mvcc" | "memfuse-vector" | "memfuse-rank" | "memfuse-adapt" | "memfuse-store"
            | "memfuse-text" | "memfuse-checkpoint" | "memfuse-graph" | "memfuse-kvcache" => Some(Ring::Ring1),
            // Ring 2: Ports & External Providers
            "memfuse-ports" | "memfuse-calibration" | "memfuse-crypto" | "memfuse-infer-candle"
            | "memfuse-infer-ollama" | "memfuse-infer-onnx" => Some(Ring::Ring2),
            // Ring 3: Engine & Cognition
            "memfuse-engine" | "memfuse-cognition" | "memfuse-router" | "memfuse-agent"
            | "memfuse-privacy" | "memfuse-db" | "memfuse-sandbox" => Some(Ring::Ring3),
            // Ring 4: Facade & Protocols
            "memfuse" | "memfuse-mcp" | "memfuse-py" => Some(Ring::Ring4),
            // Tooling / Deprecated
            "memfuse-core" | "memfuse-core-ipc-gen" => Some(Ring::Ring0),
            "memfuse-bench" | "xtask" => Some(Ring::Tooling),
            _ => None,
        }
    }
}

pub fn run_check_layering(root: &Path, warn_only: bool) -> bool {
    let crates_dir = root.join("crates");
    let mut violations = Vec::new();
    let mut warnings = Vec::new();

    if !crates_dir.exists() {
        return true;
    }

    let entries = match fs::read_dir(&crates_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("❌ Failed to read crates directory: {}", err);
            return false;
        }
    };

    for entry in entries.flatten() {
        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        let content = match fs::read_to_string(&cargo_toml) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let parsed: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let pkg_name = parsed
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("");

        let my_ring = match Ring::for_crate(pkg_name) {
            Some(r) => r,
            None => continue,
        };

        let mut deps = Vec::new();
        if let Some(d) = parsed.get("dependencies").and_then(|d| d.as_table()) {
            for k in d.keys() {
                deps.push((k.clone(), "dependencies"));
            }
        }

        for (dep, kind) in deps {
            if my_ring == Ring::Ring0 {
                if dep == "tokio" && pkg_name != "memfuse-core" {
                    violations.push(format!(
                        "Ring 0 violation: {} ({:?}) depends on tokio via {}",
                        pkg_name, my_ring, kind
                    ));
                }
                if let Some(dep_ring) = Ring::for_crate(&dep) {
                    if dep_ring > Ring::Ring0 && dep_ring != Ring::Tooling {
                        violations.push(format!(
                            "Ring 0 violation: {} ({:?}) depends on higher ring crate {} ({:?})",
                            pkg_name, my_ring, dep, dep_ring
                        ));
                    }
                }
            } else if let Some(dep_ring) = Ring::for_crate(&dep) {
                if dep_ring > my_ring && dep_ring != Ring::Tooling {
                    let msg = format!(
                        "Layering violation: {} ({:?}) depends on higher ring {} ({:?})",
                        pkg_name, my_ring, dep, dep_ring
                    );
                    if warn_only {
                        warnings.push(format!("⚠️ [WARN] {}", msg));
                    } else {
                        violations.push(msg);
                    }
                }
            }
        }
    }

    println!("=== MemFuse Ring-Layering Verification (Phase 0R) ===");
    for w in &warnings {
        println!("{}", w);
    }

    if !violations.is_empty() {
        for v in &violations {
            eprintln!("❌ {}", v);
        }
        false
    } else {
        println!("✅ Layering verification passed (Ring 0 strict clean; {} legacy warnings documented)", warnings.len());
        true
    }
}
