use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Context {
    pub root_dir: PathBuf,
    pub commit_hash: String,
}

impl Context {
    pub fn discover() -> Self {
        let root_dir = crate::find_root_dir();
        let commit_hash = Self::get_commit_hash(&root_dir);
        Context {
            root_dir,
            commit_hash,
        }
    }

    pub fn for_path(root_dir: &Path) -> Self {
        let commit_hash = Self::get_commit_hash(root_dir);
        Context {
            root_dir: root_dir.to_path_buf(),
            commit_hash,
        }
    }

    fn get_commit_hash(root: &Path) -> String {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    String::from_utf8(out.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }
}
