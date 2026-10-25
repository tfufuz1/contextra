use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::gate::GateSpec;

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
}

impl Catalog {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read gates catalog at {}: {}", path.display(), e))?;

        let catalog_file: GatesCatalogFile = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse gates catalog TOML at {}: {}", path.display(), e))?;

        let mut gates = HashMap::new();
        for g in catalog_file.gate {
            if gates.contains_key(&g.name) {
                return Err(format!("Duplicate gate name in catalog: {}", g.name));
            }
            gates.insert(g.name.clone(), g);
        }

        Ok(Catalog {
            active_phase: catalog_file.meta.active_phase,
            gates,
        })
    }
}
