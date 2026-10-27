use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub spec_version: Option<String>,
    pub spec_path: Option<String>,
    pub plan_path: Option<String>,
    pub meta_plan_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principle {
    pub id: String,
    pub class: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    pub id: String,
    pub class: Option<String>,
    pub spec: Option<String>,
    pub mech: Option<String>,
    pub phase: Option<String>,
    #[serde(default)]
    pub reqs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Veto {
    pub id: String,
    pub status: Option<String>,
    pub adr: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub project: Option<ProjectConfig>,
    #[serde(default)]
    pub principle: Vec<Principle>,
    #[serde(default)]
    pub invariant: Vec<Invariant>,
    #[serde(default)]
    pub veto: Vec<Veto>,
}

impl Constitution {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read constitution at {}: {}", path.display(), e))?;

        let constitution: Constitution = toml::from_str(&content).map_err(|e| {
            format!(
                "Failed to parse constitution TOML at {}: {}",
                path.display(),
                e
            )
        })?;

        let mut seen_ids = HashSet::new();

        for p in &constitution.principle {
            if !seen_ids.insert(&p.id) {
                return Err(format!("Duplicate ID in constitution: {}", p.id));
            }
        }
        for inv in &constitution.invariant {
            if !seen_ids.insert(&inv.id) {
                return Err(format!("Duplicate ID in constitution: {}", inv.id));
            }
        }
        for v in &constitution.veto {
            if !seen_ids.insert(&v.id) {
                return Err(format!("Duplicate ID in constitution: {}", v.id));
            }
        }

        Ok(constitution)
    }
}
