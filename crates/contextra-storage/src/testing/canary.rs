//! Canary token scanning across persisted files (SPEC §13.2, INV-KEY-1, INV-KEY-5).

use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanaryLeak {
    pub path: PathBuf,
    pub offset: usize,
}

pub fn make_canary_bytes(tag: &str) -> Vec<u8> {
    format!("SECRET_CANARY_{tag}").into_bytes()
}

pub fn find_canary_in_slice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

pub fn scan_fs_for_canaries(
    fs: &dyn Fs,
    dir: &Path,
    canaries: &[Vec<u8>],
) -> Result<Vec<CanaryLeak>, Error> {
    let mut leaks = Vec::new();
    let paths = fs.list(dir)?;
    let io = IoCtx {
        role: FileRole::Other,
        purpose: "testing_canary_scan",
    };
    for path in paths {
        let Ok(file) = fs.open(&path, io) else {
            continue;
        };
        let size = usize::try_from(file.size()?).unwrap_or(0);
        if size == 0 {
            continue;
        }
        let mut buf = vec![0u8; size];
        let n = file.read_at(0, &mut buf, io)?;
        buf.truncate(n);
        for needle in canaries {
            if let Some(offset) = find_canary_in_slice(&buf, needle) {
                leaks.push(CanaryLeak {
                    path: path.clone(),
                    offset,
                });
            }
        }
    }
    Ok(leaks)
}
