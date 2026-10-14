// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für CopyStatus, CpyCopyRegistry Erweiterungen und B11 Kopien-Geltungsbereich Verifikation.
// INVARIANTEN: Fail-Closed, Standard CopyStatus ist Unknown, Hardlink-Klon scheitert ohne Registrierung.

use contextra_checkpoint::copy_registry::{file_id_for_name, CopyKind, CopyRegEntry, CopyStatus, CpyCopyRegistry};
use contextra_checkpoint::{purge_registered_checkpoint, DefaultHardlinkCloner};
use contextra_core::SnapshotRegistry;
use contextra_ports::BoxFuture;
use contextra_store::copy_registry::CopyRegistry;
use contextra_types::{ContextraError, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

pub struct FailingCopyRegistry {
    pub fail_register: bool,
    pub fail_sync: bool,
    pub entries: Vec<CopyRegEntry>,
    pub statuses: HashMap<[u8; 16], CopyStatus>,
}

impl FailingCopyRegistry {
    pub fn new() -> Self {
        Self {
            fail_register: false,
            fail_sync: false,
            entries: Vec::new(),
            statuses: HashMap::new(),
        }
    }
}

impl CpyCopyRegistry for FailingCopyRegistry {
    fn register<'a>(&'a mut self, entry: CopyRegEntry) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if self.fail_register {
                return Err(ContextraError::Storage("Simulated register error".into()));
            }
            self.statuses.insert(entry.copy_id, CopyStatus::Unknown);
            self.entries.push(entry);
            Ok(())
        })
    }

    fn remove<'a>(&'a mut self, copy_id: &'a [u8; 16]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.entries.retain(|e| &e.copy_id != copy_id);
            self.statuses.insert(*copy_id, CopyStatus::Deleted);
            Ok(())
        })
    }

    fn list(&self) -> Vec<CopyRegEntry> {
        self.entries.clone()
    }

    fn holds_files(&self, file_ids: &[[u8; 16]]) -> Vec<[u8; 16]> {
        let mut matched = Vec::new();
        for entry in &self.entries {
            for fid in &entry.file_ids {
                if file_ids.contains(fid) {
                    matched.push(entry.copy_id);
                    break;
                }
            }
        }
        matched
    }

    fn status<'a>(&'a self, copy_id: &'a [u8; 16]) -> CopyStatus {
        self.statuses.get(copy_id).cloned().unwrap_or(CopyStatus::Unknown)
    }

    fn update_status<'a>(&'a mut self, copy_id: &'a [u8; 16], status: CopyStatus) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.statuses.insert(*copy_id, status);
            Ok(())
        })
    }

    fn sync_registry<'a>(&'a mut self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            if self.fail_sync {
                return Err(ContextraError::Storage("Simulated fsync error".into()));
            }
            Ok(())
        })
    }
}

#[tokio::test]
async fn test_hardlink_registration_failure_aborts_link_creation() {
    let src = tempdir().unwrap();
    let tgt = tempdir().unwrap();

    let sst_path = src.path().join("000001.sst");
    File::create(&sst_path)
        .unwrap()
        .write_all(b"sstable_1_data")
        .unwrap();

    let cloner = DefaultHardlinkCloner::new();
    let snap_registry = Arc::new(SnapshotRegistry::new());
    let mut reg = FailingCopyRegistry::new();

    // 1. Register failure
    reg.fail_register = true;
    let res = cloner
        .clone_sstable_hardlinks_registered(
            1,
            src.path(),
            tgt.path(),
            &snap_registry,
            &mut reg,
            100,
        )
        .await;

    assert!(res.is_err());
    let target_file = tgt.path().join("000001.sst");
    assert!(
        !target_file.exists(),
        "Hardlink file MUST NOT exist on disk if registration failed"
    );

    // 2. Fsync failure
    reg.fail_register = false;
    reg.fail_sync = true;
    let res2 = cloner
        .clone_sstable_hardlinks_registered(
            1,
            src.path(),
            tgt.path(),
            &snap_registry,
            &mut reg,
            101,
        )
        .await;

    assert!(res2.is_err());
    assert!(
        !target_file.exists(),
        "Hardlink file MUST NOT exist on disk if registry sync failed"
    );
}

#[tokio::test]
async fn test_copy_status_transitions_and_deletion() {
    let src = tempdir().unwrap();
    let tgt = tempdir().unwrap();

    let sst_path = src.path().join("000002.sst");
    File::create(&sst_path)
        .unwrap()
        .write_all(b"sstable_2_data")
        .unwrap();

    let cloner = DefaultHardlinkCloner::new();
    let snap_registry = Arc::new(SnapshotRegistry::new());
    let mut reg = FailingCopyRegistry::new();

    let res = cloner
        .clone_sstable_hardlinks_registered(
            2,
            src.path(),
            tgt.path(),
            &snap_registry,
            &mut reg,
            200,
        )
        .await
        .unwrap();

    let target_file = tgt.path().join("000002.sst");
    assert!(target_file.exists());

    // Newly registered copy has status Unknown by default
    assert_eq!(reg.status(&res.copy_id), CopyStatus::Unknown);
    assert!(reg.status(&res.copy_id).is_unknown());
    assert!(!reg.status(&res.copy_id).is_deleted());

    // Purge checkpoint and verify status becomes Deleted
    purge_registered_checkpoint(res.copy_id, &mut reg, tgt.path())
        .await
        .unwrap();

    assert!(!target_file.exists());
    assert_eq!(reg.status(&res.copy_id), CopyStatus::Deleted);
    assert!(reg.status(&res.copy_id).is_deleted());
}

#[test]
fn test_gdpr_art19_and_art28_out_of_scope_notices() {
    let art19 = CopyStatus::art19_recipient("Empfänger X");
    let art28 = CopyStatus::art28_processor("Auftragsverarbeiter Y");

    match art19 {
        CopyStatus::OutOfScope { reason } => {
            assert!(reason.contains("Art. 19 DSGVO"));
            assert!(reason.contains("Empfänger X"));
        }
        _ => panic!("Expected OutOfScope status"),
    }

    match art28 {
        CopyStatus::OutOfScope { reason } => {
            assert!(reason.contains("Art. 28 DSGVO"));
            assert!(reason.contains("Auftragsverarbeiter Y"));
        }
        _ => panic!("Expected OutOfScope status"),
    }
}
