//! Module: check_ring_capabilities_consistency
//! Compares architecture ring declarations between `capabilities.toml` and `Cargo.toml` package.metadata.contextra.ring metadata.
//! Returns findings for any crates where both sources diverge or where ring metadata is missing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingConsistencyFinding {
    pub crate_name: String,
    pub declared_in_capabilities_toml: String,
    pub declared_in_ring_layering: Option<String>,
}

#[derive(Deserialize)]
struct CapabilitiesManifestPartial {
    crates: BTreeMap<String, CrateEntryPartial>,
}

#[derive(Deserialize)]
struct CrateEntryPartial {
    ring: String,
}

fn find_root_dir() -> PathBuf {
    if let Ok(cargo_manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(cargo_manifest);
        if path.file_name().and_then(|s| s.to_str()) == Some("xtask") {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
        return path;
    }
    let mut curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if curr.join("Cargo.toml").exists() && curr.join("capabilities.toml").exists() {
            return curr;
        }
        if !curr.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

pub fn run_check_ring_capabilities_consistency() -> Result<Vec<RingConsistencyFinding>, String> {
    let root = find_root_dir();
    run_check_ring_capabilities_consistency_in_root(&root)
}

pub fn run_check_ring_capabilities_consistency_in_root(
    root: &Path,
) -> Result<Vec<RingConsistencyFinding>, String> {
    let caps_path = root.join("capabilities.toml");
    if !caps_path.exists() {
        return Err(format!(
            "capabilities.toml not found at {}",
            caps_path.display()
        ));
    }

    let content = fs::read_to_string(&caps_path)
        .map_err(|e| format!("Failed to read {}: {}", caps_path.display(), e))?;

    let parsed: CapabilitiesManifestPartial = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse capabilities.toml: {}", e))?;

    let mut findings = Vec::new();

    let ring_map = crate::check_ring_layering::get_workspace_ring_map()
        .map_err(|e| format!("Failed to get workspace ring map: {}", e))?;

    for (crate_name, entry) in parsed.crates {
        let caps_ring = entry.ring.trim();
        let layering_ring_opt = ring_map.get(&crate_name).map(|r| r.name().to_string());

        let is_consistent = match &layering_ring_opt {
            Some(layering_ring) => caps_ring == layering_ring,
            None => false,
        };

        if !is_consistent {
            findings.push(RingConsistencyFinding {
                crate_name,
                declared_in_capabilities_toml: caps_ring.to_string(),
                declared_in_ring_layering: layering_ring_opt,
            });
        }
    }

    findings.sort_by(|a, b| a.crate_name.cmp(&b.crate_name));

    Ok(findings)
}
