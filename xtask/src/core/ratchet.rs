use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetFile {
    #[serde(flatten)]
    pub entries: BTreeMap<String, toml::Value>,
}

pub struct Ratchet {
    pub file: RatchetFile,
}

impl Ratchet {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| {
            format!(
                "Ratchet file missing or unreadable at {}: {}",
                path.display(),
                e
            )
        })?;

        let file: RatchetFile = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse ratchet TOML at {}: {}", path.display(), e))?;

        Ok(Ratchet { file })
    }
}
