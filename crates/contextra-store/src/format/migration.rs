use contextra_types::{ContextraError, Result};
use std::path::{Path, PathBuf};

use super::header::FormatHeader;
use super::openable::FileKind;
use crate::manifest::{Manifest, ManifestEntry, MANIFEST_HEADER_MAGIC};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupVerificationStatus {
    Verified,
    NotVerified,
}

pub trait BackupVerifier: Send + Sync {
    fn verify_backup(&self, dir: &Path) -> Result<BackupVerificationStatus>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationAction {
    pub path: PathBuf,
    pub file_kind: FileKind,
    pub current_version: u16,
    pub target_version: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    pub dir: PathBuf,
    pub initial_dir_hash: [u8; 32],
    pub actions: Vec<MigrationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationResult {
    pub migrated_files_count: usize,
}

pub fn compute_directory_hash(dir: &Path) -> Result<[u8; 32]> {
    let mut entries = Vec::new();
    if dir.exists() {
        let read_dir = std::fs::read_dir(dir).map_err(|e| {
            ContextraError::Storage(format!("Failed to read directory {}: {}", dir.display(), e))
        })?;
        for entry in read_dir {
            let entry = entry.map_err(|e| ContextraError::Storage(e.to_string()))?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let content = std::fs::read(&path).map_err(|e| {
                        ContextraError::Storage(format!(
                            "Failed to read file {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    entries.push((file_name.to_string(), content));
                }
            }
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = blake3::Hasher::new();
    for (name, content) in &entries {
        hasher.update(name.as_bytes());
        hasher.update(content);
    }
    let hash = hasher.finalize();
    Ok(*hash.as_bytes())
}

/// Pure dry-run planning function. Inspects directory and returns MigrationPlan without modifying any files.
pub fn plan_migration(dir: &Path) -> Result<MigrationPlan> {
    let initial_dir_hash = compute_directory_hash(dir)?;
    let mut actions = Vec::new();

    if dir.exists() {
        let read_dir = std::fs::read_dir(dir).map_err(|e| {
            ContextraError::Storage(format!("Failed to read directory {}: {}", dir.display(), e))
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|e| ContextraError::Storage(e.to_string()))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            if ext == "sst" {
                let bytes = std::fs::read(&path).map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to read SSTable {}: {}",
                        path.display(),
                        e
                    ))
                })?;
                if bytes.len() >= 54 {
                    let ver_slice = &bytes[bytes.len() - 6..bytes.len() - 4];
                    if let Ok(arr) = ver_slice.try_into() {
                        let cur_ver = u16::from_le_bytes(arr);
                        let target_ver = 4;
                        if cur_ver < target_ver {
                            actions.push(MigrationAction {
                                path: path.clone(),
                                file_kind: FileKind::Sstable,
                                current_version: cur_ver,
                                target_version: target_ver,
                            });
                        }
                    }
                }
            } else if ext == "blob" {
                let bytes = std::fs::read(&path).map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to read Blob {}: {}",
                        path.display(),
                        e
                    ))
                })?;
                if bytes.len() >= FormatHeader::LAYOUT_SIZE {
                    if let Ok(hdr) = FormatHeader::parse(&bytes[0..FormatHeader::LAYOUT_SIZE]) {
                        let target_ver = 1;
                        if hdr.version < target_ver {
                            actions.push(MigrationAction {
                                path: path.clone(),
                                file_kind: FileKind::Blob,
                                current_version: hdr.version,
                                target_version: target_ver,
                            });
                        }
                    }
                }
            } else if file_name == "MANIFEST" {
                let bytes = std::fs::read(&path).map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to read Manifest {}: {}",
                        path.display(),
                        e
                    ))
                })?;
                if bytes.len() >= 5 && &bytes[0..4] == MANIFEST_HEADER_MAGIC {
                    let cur_ver = bytes[4] as u16;
                    let target_ver = 1;
                    if cur_ver < target_ver {
                        actions.push(MigrationAction {
                            path: path.clone(),
                            file_kind: FileKind::Manifest,
                            current_version: cur_ver,
                            target_version: target_ver,
                        });
                    }
                }
            }
        }
    }

    Ok(MigrationPlan {
        dir: dir.to_path_buf(),
        initial_dir_hash,
        actions,
    })
}

/// Executes migration actions specified in plan, requiring verified backup.
pub async fn execute_migration(
    plan: &MigrationPlan,
    verifier: &dyn BackupVerifier,
) -> Result<MigrationResult> {
    let status = verifier.verify_backup(&plan.dir)?;
    if status != BackupVerificationStatus::Verified {
        return Err(ContextraError::Storage(
            "Backup verification failed: fail-closed migration aborted".into(),
        ));
    }

    let manifest_path = plan.dir.join("MANIFEST");
    let manifest = Manifest::open(&manifest_path).await?;

    let mut migrated_files_count = 0;
    for action in &plan.actions {
        let file_kind_u8 = match action.file_kind {
            FileKind::Sstable => 0,
            FileKind::Blob => 1,
            FileKind::Manifest => 2,
            FileKind::Wal => 3,
            FileKind::Ledger => 4,
        };
        manifest
            .append(&ManifestEntry::FormatVersionRecord {
                file_kind: file_kind_u8,
                max_version: action.target_version,
                migrated_from: action.current_version,
            })
            .await?;
        migrated_files_count += 1;
    }

    Ok(MigrationResult {
        migrated_files_count,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    struct DummyBackupVerifier {
        verified: bool,
    }

    impl BackupVerifier for DummyBackupVerifier {
        fn verify_backup(&self, _dir: &Path) -> Result<BackupVerificationStatus> {
            if self.verified {
                Ok(BackupVerificationStatus::Verified)
            } else {
                Ok(BackupVerificationStatus::NotVerified)
            }
        }
    }

    #[tokio::test]
    async fn test_plan_migration_dry_run_is_immutable() {
        let dir = tempdir().expect("tempdir");
        let sample_file = dir.path().join("sst-0001.sst");
        std::fs::write(&sample_file, b"test payload for sst file").expect("write file");

        let hash_before = compute_directory_hash(dir.path()).expect("hash before");
        let plan = plan_migration(dir.path()).expect("plan_migration");
        let hash_after = compute_directory_hash(dir.path()).expect("hash after");

        assert_eq!(
            hash_before, hash_after,
            "plan_migration dry run must not modify any file in directory"
        );
        assert_eq!(plan.initial_dir_hash, hash_before);
    }

    #[tokio::test]
    async fn test_execute_migration_fail_closed_unverified_backup() {
        let dir = tempdir().expect("tempdir");
        let plan = plan_migration(dir.path()).expect("plan");
        let verifier = DummyBackupVerifier { verified: false };

        let res = execute_migration(&plan, &verifier).await;
        assert!(
            res.is_err(),
            "execute_migration must fail closed if backup is not verified"
        );
    }
}
