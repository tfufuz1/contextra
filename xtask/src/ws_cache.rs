use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

static FILE_LIST_CACHE: Mutex<Option<HashMap<PathBuf, Arc<[PathBuf]>>>> = Mutex::new(None);
static FILE_CONTENT_CACHE: Mutex<Option<HashMap<PathBuf, Arc<str>>>> = Mutex::new(None);

/// Returns the cached list of all `.rs` files under `crates/` (excluding `target/`).
pub fn workspace_rs_files() -> Arc<[PathBuf]> {
    let root = crate::find_root_dir();
    workspace_rs_files_from_root(&root)
}

/// Returns the cached list of all `.rs` files under `<root>/crates/` (excluding `target/`).
pub fn workspace_rs_files_from_root(root: &Path) -> Arc<[PathBuf]> {
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    if let Ok(guard) = FILE_LIST_CACHE.lock() {
        if let Some(ref cache) = *guard {
            if let Some(list) = cache.get(&canonical_root) {
                return Arc::clone(list);
            }
        }
    }

    let mut files = Vec::new();
    let crates_dir = canonical_root.join("crates");
    if crates_dir.exists() {
        for entry in WalkDir::new(&crates_dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "target" && name != ".git"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path.to_path_buf());
            }
        }
    }
    files.sort();

    let arc_slice: Arc<[PathBuf]> = Arc::from(files);

    if let Ok(mut guard) = FILE_LIST_CACHE.lock() {
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(canonical_root, Arc::clone(&arc_slice));
    }

    arc_slice
}

/// Reads file content into a cached `Arc<str>`.
pub fn read_cached(path: &Path) -> io::Result<Arc<str>> {
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if let Ok(guard) = FILE_CONTENT_CACHE.lock() {
        if let Some(ref cache) = *guard {
            if let Some(content) = cache.get(&canonical_path) {
                return Ok(Arc::clone(content));
            }
        }
    }

    let raw = fs::read_to_string(path)?;
    let arc_str: Arc<str> = Arc::from(raw);

    if let Ok(mut guard) = FILE_CONTENT_CACHE.lock() {
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(canonical_path, Arc::clone(&arc_str));
    }

    Ok(arc_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ws_cache_file_list_and_read() {
        let dir = tempdir().unwrap();
        let crates_dir = dir.path().join("crates").join("my_crate");
        fs::create_dir_all(&crates_dir).unwrap();

        let rs_file = crates_dir.join("lib.rs");
        fs::write(&rs_file, "pub fn foo() {}").unwrap();

        let files = workspace_rs_files_from_root(dir.path());
        assert_eq!(files.len(), 1);

        let content = read_cached(&rs_file).unwrap();
        assert_eq!(&*content, "pub fn foo() {}");

        // Verify caching returns same Arc ptr
        let content2 = read_cached(&rs_file).unwrap();
        assert!(Arc::ptr_eq(&content, &content2));
    }
}
