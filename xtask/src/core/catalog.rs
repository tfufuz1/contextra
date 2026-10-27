use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::gate::GateSpec;

pub fn parse_phase_level(phase: &str) -> Result<u32, String> {
    let p = phase.trim().to_uppercase();
    if let Some(num_str) = p.strip_prefix("PH") {
        num_str
            .parse::<u32>()
            .map_err(|_| format!("Unknown phase format: {}", phase))
    } else {
        Err(format!("Unknown phase: {}", phase))
    }
}

pub fn is_phase_due(from_phase: &str, active_phase: &str) -> Result<bool, String> {
    let from_lvl = parse_phase_level(from_phase)?;
    let active_lvl = parse_phase_level(active_phase)?;
    Ok(from_lvl <= active_lvl)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogMeta {
    pub active_phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEntry {
    pub name: String,
    pub tier: String,
    pub from_phase: String,
    pub kind: String,
    pub enforces: Vec<String>,
    pub why: String,
    pub owner: String,
    pub cmd: Option<Vec<String>>,
    pub toolchain: Option<String>,
}

impl From<&GateEntry> for GateSpec {
    fn from(e: &GateEntry) -> Self {
        GateSpec {
            name: e.name.clone(),
            tier: e.tier.clone(),
            from_phase: e.from_phase.clone(),
            kind: e.kind.clone(),
            enforces: e.enforces.clone(),
            why: e.why.clone(),
            owner: e.owner.clone(),
            cmd: e.cmd.clone(),
            toolchain: e.toolchain.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatesCatalogFile {
    pub meta: CatalogMeta,
    pub gate: Vec<GateEntry>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub active_phase: String,
    pub gates: HashMap<String, GateEntry>,
    pub gate_order: Vec<String>,
}

impl Catalog {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read gates catalog at {}: {}", path.display(), e))?;

        let catalog_file: GatesCatalogFile = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse gates catalog TOML at {}: {}", path.display(), e))?;

        // Validate active phase format
        parse_phase_level(&catalog_file.meta.active_phase)?;

        let mut gates = HashMap::new();
        let mut gate_order = Vec::new();
        for g in catalog_file.gate {
            if gates.contains_key(&g.name) {
                return Err(format!("Duplicate gate name in catalog: {}", g.name));
            }
            // Validate gate's from_phase
            parse_phase_level(&g.from_phase)?;

            gate_order.push(g.name.clone());
            gates.insert(g.name.clone(), g);
        }

        Ok(Catalog {
            active_phase: catalog_file.meta.active_phase,
            gates,
            gate_order,
        })
    }
}
