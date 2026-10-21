// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Trait description for PurgeBackend allowing orchestrator decoupling (§C5.1).

use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeReport, PurgeScope};
use contextra_ports::erasure::{FenceGuard, PurgeError, PurgeId};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Abstract backend contract required by [`super::orchestrator::PurgeOrchestrator`].
pub trait PurgeBackend: Send + Sync {
    /// Acquires write fence for `scope` under `id`.
    fn acquire_fence(&self, scope: &PurgeScope, id: PurgeId) -> Result<FenceGuard, PurgeError>;

    /// Flushes active memtables to disk.
    fn flush_memtable<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>>;

    /// Executes purge compaction rewrite over `scope`.
    fn purge_keys<'a>(
        &'a self,
        scope: PurgeScope,
        as_of_tx: u64,
        probe: Option<&'a (dyn FreeSpaceProbe + 'a)>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PurgeReport, PurgeError>> + Send + 'a>>;

    /// Disposes obsolete files identified during rewrite.
    fn dispose<'a>(
        &'a self,
        obsolete_files: Vec<PathBuf>,
        secure_delete: bool,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<usize, PurgeError>> + Send + 'a>>;

    /// Retires WAL records for purged scope.
    fn retire_wal<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>>;

    /// Verification scan over live SSTables for scoped residue before release.
    fn scan_residue<'a>(&'a self, scope: &'a PurgeScope) -> Pin<Box<dyn std::future::Future<Output = Result<bool, PurgeError>> + Send + 'a>> {
        let _ = scope;
        Box::pin(async move { Ok(false) })
    }
}

/// Mock implementation of `PurgeBackend` for testing.
#[derive(Debug, Default)]
pub struct MockBackend {
    /// Invocations counter for `purge_keys`.
    pub purge_keys_calls: AtomicUsize,
    /// Controlled residue flag returned by `scan_residue`.
    pub residue_detected: bool,
}

impl MockBackend {
    /// Creates a new `MockBackend`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl PurgeBackend for MockBackend {
    fn acquire_fence(&self, _scope: &PurgeScope, id: PurgeId) -> Result<FenceGuard, PurgeError> {
        Ok(FenceGuard::new(id))
    }

    fn flush_memtable<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge_keys<'a>(
        &'a self,
        _scope: PurgeScope,
        _as_of_tx: u64,
        _probe: Option<&'a (dyn FreeSpaceProbe + 'a)>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PurgeReport, PurgeError>> + Send + 'a>> {
        self.purge_keys_calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            Ok(PurgeReport {
                records_purged: 0,
                bytes_reclaimed: 0,
                sstables_replaced: 0,
                obsolete_files: Vec::new(),
                residual_risks: Vec::new(),
            })
        })
    }

    fn dispose<'a>(
        &'a self,
        _obsolete_files: Vec<PathBuf>,
        _secure_delete: bool,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<usize, PurgeError>> + Send + 'a>> {
        Box::pin(async move { Ok(0) })
    }

    fn retire_wal<'a>(&'a self) -> Pin<Box<dyn std::future::Future<Output = Result<(), PurgeError>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan_residue<'a>(&'a self, _scope: &'a PurgeScope) -> Pin<Box<dyn std::future::Future<Output = Result<bool, PurgeError>> + Send + 'a>> {
        let residue = self.residue_detected;
        Box::pin(async move { Ok(residue) })
    }
}
