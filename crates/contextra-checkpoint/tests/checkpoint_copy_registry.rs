// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für CopyRegistry Verankerung und purge_checkpoint Hardlink-Entfernung mit nlink==1 Garantie.

use contextra_checkpoint::{purge_registered_checkpoint, CpyCopyRegistry, DefaultHardlinkCloner};
use contextra_mvcc::SnapshotRegistry;
use contextra_ports::BoxFuture;
use contextra_store::copy_registry::CopyRegistry;
use contextra_types::Result;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tempfile::tempdir;

pub struct CopyRegistryAdapter<'a>(pub &'a mut CopyRegistry);

impl<'a> CpyCopyRegistry for CopyRegistryAdapter<'a> {
    fn register<'b>(
        &'b mut self,
        entry: contextra_checkpoint::CopyRegEntry,
    ) -> BoxFuture<'b, Result<()>> {
        Box::pin(async move {
            let store_entry = contextra_store::copy_registry::CopyRegEntry {
                copy_id: entry.copy_id,
                kind: match entry.kind {
                    contextra_checkpoint::CopyKind::Hardlink => {
                        contextra_store::copy_registry::CopyKind::Hardlink
                    }
                    contextra_checkpoint::CopyKind::Reflink => {
                        contextra_store::copy_registry::CopyKind::Reflink
                    }
                    contextra_checkpoint::CopyKind::FullCopy => {
                        contextra_store::copy_registry::CopyKind::FullCopy
                    }
                },
                file_ids: entry.file_ids,
                created_tx: entry.created_tx,
            };
            self.0.register(store_entry).await
        })
    }

    fn remove<'b>(&'b mut self, copy_id: &'b [u8; 16]) -> BoxFuture<'b, Result<()>> {
        Box::pin(async move { self.0.remove(copy_id).await })
    }

    fn list(&self) -> Vec<contextra_checkpoint::CopyRegEntry> {
        self.0
            .list()
            .iter()
            .map(|e| contextra_checkpoint::CopyRegEntry {
                copy_id: e.copy_id,
                kind: match e.kind {
                    contextra_store::copy_registry::CopyKind::Hardlink => {
                        contextra_checkpoint::CopyKind::Hardlink
                    }
                    contextra_store::copy_registry::CopyKind::Reflink => {
                        contextra_checkpoint::CopyKind::Reflink
                    }
                    contextra_store::copy_registry::CopyKind::FullCopy => {
                        contextra_checkpoint::CopyKind::FullCopy
                    }
                },
                file_ids: e.file_ids.clone(),
                created_tx: e.created_tx,
            })
            .collect()
    }

    fn holds_files(&self, file_ids: &[[u8; 16]]) -> Vec<[u8; 16]> {
        self.0.holds_files(file_ids)
    }
}

#[tokio::test]
async fn test_purge_checkpoint_reduces_nlink_and_unlinks_target() {
    let src = tempdir().unwrap();
    let tgt = tempdir().unwrap();
    let reg_dir = tempdir().unwrap();

    let sst_path = src.path().join("000005.sst");
    let original_content = b"sstable_5_data_content";
    File::create(&sst_path)
        .unwrap()
        .write_all(original_content)
        .unwrap();

    let cloner = DefaultHardlinkCloner::new();
    let snap_registry = Arc::new(SnapshotRegistry::new());
    let mut copy_registry = CopyRegistry::open(reg_dir.path()).await.unwrap();

    let res = {
        let mut adapter = CopyRegistryAdapter(&mut copy_registry);
        cloner
            .clone_sstable_hardlinks_registered(
                5,
                src.path(),
                tgt.path(),
                &snap_registry,
                &mut adapter,
                500,
            )
            .await
            .unwrap()
    };

    let target_file = tgt.path().join("000005.sst");
    assert!(target_file.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(&sst_path).unwrap();
        assert_eq!(meta.nlink(), 2, "After hardlink clone, nlink must be 2");
    }

    // Execute purge_registered_checkpoint
    {
        let mut adapter = CopyRegistryAdapter(&mut copy_registry);
        purge_registered_checkpoint(res.copy_id, &mut adapter, tgt.path())
            .await
            .unwrap();
    }

    // Target hardlink should be removed
    assert!(!target_file.exists());

    // Registry should be empty
    assert_eq!(copy_registry.list().len(), 0);

    // Source file must remain intact with nlink == 1
    assert!(sst_path.exists());
    let read_back = std::fs::read(&sst_path).unwrap();
    assert_eq!(
        read_back, original_content,
        "Source file content must NOT be overwritten"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(&sst_path).unwrap();
        assert_eq!(
            meta.nlink(),
            1,
            "After purge_checkpoint, source file nlink must be 1"
        );
    }

    // Idempotency test: second purge on already purged copy_id returns Ok(())
    {
        let mut adapter = CopyRegistryAdapter(&mut copy_registry);
        assert!(
            purge_registered_checkpoint(res.copy_id, &mut adapter, tgt.path())
                .await
                .is_ok()
        );
    }
}
