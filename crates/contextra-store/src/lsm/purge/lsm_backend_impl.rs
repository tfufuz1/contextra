// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Implementation of PurgeBackend trait for LsmStorage (§C5.1).

use super::backend::PurgeBackend;
use super::dispose::dispose_obsolete_files;
use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use contextra_ports::erasure::{FenceGuard, PurgeError, PurgeId};
use contextra_ports::StorageEngine;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

impl<S: PurgeBackend> PurgeBackend for Arc<S> {
    fn acquire_fence(&self, scope: &PurgeScope, id: PurgeId) -> Result<FenceGuard, PurgeError> {
        (**self).acquire_fence(scope, id)
    }

    fn flush_memtable<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>> {
        (**self).flush_memtable()
    }

    fn purge_keys<'a>(
        &'a self,
        scope: PurgeScope,
        as_of_tx: u64,
        probe: Option<&'a (dyn FreeSpaceProbe + 'a)>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PurgeReport, PurgeError>> + Send + 'a>> {
        (**self).purge_keys(scope, as_of_tx, probe)
    }

    fn dispose<'a>(
        &'a self,
        obsolete_files: Vec<PathBuf>,
        secure_delete: bool,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<usize, PurgeError>> + Send + 'a>> {
        (**self).dispose(obsolete_files, secure_delete)
    }

    fn retire_wal<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>> {
        (**self).retire_wal()
    }

    fn scan_residue<'a>(&'a self, scope: &'a PurgeScope) -> Pin<Box<dyn std::future::Future<Output = Result<bool, PurgeError>> + Send + 'a>> {
        (**self).scan_residue(scope)
    }
}

impl PurgeBackend for LsmStorage {
    fn acquire_fence(&self, scope: &PurgeScope, id: PurgeId) -> Result<FenceGuard, PurgeError> {
        let fence_targets = match scope {
            PurgeScope::Keys(keys) => keys.clone(),
            PurgeScope::Prefixes(prefixes) => prefixes.clone(),
        };

        if fence_targets.is_empty() {
            return Err(PurgeError::UnclassifiedFiles { count: 0 });
        }

        let fen_guard = self
            .erasure_fence
            .acquire_scope_fence_for(fence_targets)
            .map_err(|_| PurgeError::CompactionBusy)?;

        let fence_id = fen_guard.id();
        std::mem::forget(fen_guard);

        let fence = Arc::clone(&self.erasure_fence);
        Ok(FenceGuard::with_release(id, move || {
            fence.release_prefix_fence(fence_id);
        }))
    }

    fn flush_memtable<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>> {
        Box::pin(async move {
            StorageEngine::flush(self).await.map_err(|e| PurgeError::WriteError {
                file: "memtable_flush".into(),
                details: e.to_string(),
            })
        })
    }

    fn purge_keys<'a>(
        &'a self,
        scope: PurgeScope,
        as_of_tx: u64,
        probe: Option<&'a (dyn FreeSpaceProbe + 'a)>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PurgeReport, PurgeError>> + Send + 'a>> {
        Box::pin(async move {
            self.purge_keys(scope, as_of_tx, probe).await
        })
    }

    fn dispose<'a>(
        &'a self,
        obsolete_files: Vec<PathBuf>,
        secure_delete: bool,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<usize, PurgeError>> + Send + 'a>> {
        Box::pin(async move {
            let disp_rep = dispose_obsolete_files(&self.config.path, obsolete_files, secure_delete)
                .await
                .map_err(PurgeError::Io)?;
            Ok(disp_rep.files_removed)
        })
    }

    fn retire_wal<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan_residue<'a>(&'a self, scope: &'a PurgeScope) -> Pin<Box<dyn std::future::Future<Output = Result<bool, PurgeError>> + Send + 'a>> {
        Box::pin(async move {
            let sstables = self.sstables.read().await;
            for sst in sstables.iter() {
                let mut stream = sst.stream().await.map_err(|e| PurgeError::ReadError {
                    file: sst.file_path().display().to_string(),
                    offset: 0,
                    source_details: e.to_string(),
                })?;
                while let Some((key, _val, _seq, _tx)) = stream.next_entry().await.map_err(|e| PurgeError::ReadError {
                    file: sst.file_path().display().to_string(),
                    offset: 0,
                    source_details: e.to_string(),
                })? {
                    if scope.matches_key(&key) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        })
    }
}
