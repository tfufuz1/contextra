use std::path::{Path, PathBuf};

pub fn not_implemented(cmd: &str) -> Result<(), String> {
    Err(format!("NOT IMPLEMENTED: {cmd}"))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TagItem {
    pub file_path: String,
    pub line_num: usize,
    pub tag_type: String,
    pub raw: String,
    pub timestamp: String,
    pub category: Option<String>,
    pub severity: Option<String>,
    pub id: Option<String>,
    pub session: Option<String>,
    pub status: Option<String>,
    pub description: String,
    pub is_resolved: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub path: String,
    pub layer: u8,
    pub loc: usize,
    pub status: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub ring: String,
    pub maturity: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CapabilitiesManifest {
    pub crates: std::collections::HashMap<String, CrateCapability>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrateCapability {
    pub ring: String,
    pub maturity: String,
    pub status: String,
    pub description: String,
}

pub fn scan_tags<P: AsRef<Path>>(_root: P) -> Vec<TagItem> {
    Vec::new()
}

pub fn calculate_crate_loc<P: AsRef<Path>>(_dir: P) -> usize {
    0
}

pub fn get_workspace_crates() -> Vec<CrateInfo> {
    Vec::new()
}

pub fn get_workspace_crates_from_root(
    _root_path: &Path,
    _caps: &CapabilitiesManifest,
) -> Vec<CrateInfo> {
    Vec::new()
}

pub fn load_capabilities_manifest(_root_dir: &Path) -> CapabilitiesManifest {
    CapabilitiesManifest::default()
}
