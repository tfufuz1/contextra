use contextra_core::{ContextraError, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Returns paths of `.blob` segment files in `dir` that are not present in `manifest_live_set`.
pub fn list_orphans(dir: &Path, manifest_live_set: &HashSet<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut orphans = Vec::new();
    if !dir.exists() {
        return Ok(orphans);
    }

    let read_dir = std::fs::read_dir(dir).map_err(|e| {
        ContextraError::Storage(format!("Failed to read directory {}: {}", dir.display(), e))
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| ContextraError::Storage(e.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "blob" {
                    let file_name = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    if !manifest_live_set.contains(&file_name) && !manifest_live_set.contains(&path)
                    {
                        orphans.push(path);
                    }
                }
            }
        }
    }

    orphans.sort();
    Ok(orphans)
}
