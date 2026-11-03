//! # Konformitätsprüfsuite für den `FileKeyring`-Port (`conformance/file_keyring`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für den Dateischlüssel-Verwalter [`FileKeyring`] gemäß SPEC §4.1, D-85 und PO-06.
//!
//! ## Getestete Verträge (Pflichtfälle):
//! 1. `po_06_create_open_destroy_lifecycle`: Vollständiger Lebenszyklus von Schlüsseln (`create` -> `open` -> `destroy`).
//! 2. `po_06_open_after_destroy_fails`: Nach `destroy` ist `open` garantiert ein Fehler (`NotFound` oder `Erased`).
//! 3. `po_06_destroy_idempotent`: Wiederholtes `destroy` auf dieselbe `FileId` ist idempotent und liefert `Ok(())`.
//! 4. `po_06_reopen_verification_on_failure`: Neuöffnen-Prüfung nach dem Nullen des Slots (PO-06 / D-55).

use contextra_ports::{BoxFuture, ContextraError, Result};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Numerical file kind descriptor (1..=10 per SPEC F.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileKind(pub u8);

/// Unique 128-bit file identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub u128);

/// File key handle wrapping a 32-byte symmetric cryptographic key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileKey {
    /// File identifier bound to key.
    pub file_id: FileId,
    /// Symmetric encryption key bytes.
    pub key_bytes: [u8; 32],
}

/// FileKeyring Port Trait according to SPEC §4.1 & D-85.
pub trait FileKeyring: Send + Sync {
    /// Generates and persists a fresh file key for given kind and id.
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey>>;
    /// Opens and decrypts an existing file key.
    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey>>;
    /// Nulls the key slot, syncs storage, and verifies absence (PO-06). Idempotent.
    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<()>>;
}

/// Conformance failure descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Suite identifier.
    pub suite_name: &'static str,
    /// Test case.
    pub test_case: &'static str,
    /// Diagnostic failure message.
    pub message: String,
}

impl ConformanceFailure {
    /// Creates a new failure record.
    #[must_use]
    pub fn new(
        suite_name: &'static str,
        test_case: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            suite_name,
            test_case,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ConformanceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Conformance failure in test '{}': {}",
            self.suite_name, self.test_case, self.message
        )
    }
}

impl std::error::Error for ConformanceFailure {}

/// Conformance result type.
pub type ConformanceResult = std::result::Result<(), ConformanceFailure>;

/// Führt die vollständige Konformitätsprüfung für eine [`FileKeyring`]-Implementierung durch.
pub async fn run<F, Fut, K>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = K>,
    K: FileKeyring,
{
    let suite = "FileKeyring";

    // Fall 1: PO-06 / D-85 create/open/destroy lifecycle
    {
        let keyring = factory().await;
        let fid = FileId(1001);
        let kind = FileKind(1); // SSTable

        let key = match keyring.create(kind, fid).await {
            Ok(k) => k,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_06_create_open_destroy_lifecycle",
                    format!("create failed: {e:?}"),
                ));
            }
        };

        if key.file_id != fid {
            return Err(ConformanceFailure::new(
                suite,
                "po_06_create_open_destroy_lifecycle",
                format!("returned key has mismatched file_id: {:?}", key.file_id),
            ));
        }

        let opened_key = match keyring.open(fid).await {
            Ok(k) => k,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_06_create_open_destroy_lifecycle",
                    format!("open failed: {e:?}"),
                ));
            }
        };

        if opened_key != key {
            return Err(ConformanceFailure::new(
                suite,
                "po_06_create_open_destroy_lifecycle",
                "key opened after create did not match created key bytes",
            ));
        }
    }

    // Fall 2: PO-06 Nach destroy ist open ein Fehler
    {
        let keyring = factory().await;
        let fid = FileId(2002);
        let _ = keyring.create(FileKind(2), fid).await;

        if let Err(e) = keyring.destroy(fid).await {
            return Err(ConformanceFailure::new(
                suite,
                "po_06_open_after_destroy_fails",
                format!("destroy failed: {e:?}"),
            ));
        }

        match keyring.open(fid).await {
            Err(_) => {} // Expected failure
            Ok(k) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_06_open_after_destroy_fails",
                    format!("open after destroy unexpectedly succeeded: {k:?}"),
                ));
            }
        }
    }

    // Fall 3: PO-06 Destroy ist idempotent
    {
        let keyring = factory().await;
        let fid = FileId(3003);
        let _ = keyring.create(FileKind(3), fid).await;
        let _ = keyring.destroy(fid).await;

        // Second destroy must succeed idempotently
        if let Err(e) = keyring.destroy(fid).await {
            return Err(ConformanceFailure::new(
                suite,
                "po_06_destroy_idempotent",
                format!("second destroy failed: {e:?}"),
            ));
        }

        // Destroy of non-existent key is also idempotent and safe
        if let Err(e) = keyring.destroy(FileId(999_999)).await {
            return Err(ConformanceFailure::new(
                suite,
                "po_06_destroy_idempotent",
                format!("destroy of non-existent file_id failed: {e:?}"),
            ));
        }
    }

    // Fall 4: PO-06 Neuöffnen-Prüfung (Lesende Verifikation des Fehlens)
    {
        let keyring = factory().await;
        let fid = FileId(4004);
        let _ = keyring.create(FileKind(4), fid).await;
        let _ = keyring.destroy(fid).await;

        let check_open = keyring.open(fid).await;
        match check_open {
            Err(ContextraError::NotFound(_)) => {}
            Err(_) => {} // valid error variant
            Ok(_) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_06_reopen_verification_on_failure",
                    "verification open did not fail after destroy",
                ));
            }
        }
    }

    Ok(())
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

/// In-Memory Reference Implementation of [`FileKeyring`].
#[derive(Clone, Default)]
#[allow(clippy::type_complexity)]
pub struct ReferenceFileKeyring {
    keys: Arc<RwLock<BTreeMap<FileId, (FileKind, [u8; 32])>>>,
    destroyed: Arc<RwLock<BTreeMap<FileId, bool>>>,
    reopen_verified: Arc<AtomicBool>,
}

impl ReferenceFileKeyring {
    /// Creates a new reference keyring.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if reopen verification was executed during destroy.
    pub fn was_reopen_verified(&self) -> bool {
        self.reopen_verified.load(Ordering::SeqCst)
    }
}

impl FileKeyring for ReferenceFileKeyring {
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey>> {
        Box::pin(async move {
            let mut key_bytes = [0u8; 32];
            // Synthetic reproducible key bytes
            let id_bytes = id.0.to_le_bytes();
            key_bytes[..16].copy_from_slice(&id_bytes);
            key_bytes[16..].fill(0x5A);

            self.keys.write().insert(id, (kind, key_bytes));
            Ok(FileKey {
                file_id: id,
                key_bytes,
            })
        })
    }

    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey>> {
        Box::pin(async move {
            let guard = self.keys.read();
            if let Some((_, key_bytes)) = guard.get(&id) {
                Ok(FileKey {
                    file_id: id,
                    key_bytes: *key_bytes,
                })
            } else {
                Err(ContextraError::NotFound(format!(
                    "file key {id:?} not found"
                )))
            }
        })
    }

    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Null slot, simulate sync_data
            self.keys.write().remove(&id);
            self.destroyed.write().insert(id, true);

            // PO-06: Lesende Prüfung des Fehlens (Reopen verification)
            let check = self.open(id).await;
            if check.is_ok() {
                return Err(ContextraError::Internal(
                    "destroy verification failed: key still readable".to_string(),
                ));
            }
            self.reopen_verified.store(true, Ordering::SeqCst);
            Ok(())
        })
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: destroy does not delete key (open still succeeds).
#[derive(Clone, Default)]
pub struct MutantOpenAfterDestroySucceedsKeyring(ReferenceFileKeyring);

impl FileKeyring for MutantOpenAfterDestroySucceedsKeyring {
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey>> {
        self.0.create(kind, id)
    }
    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey>> {
        self.0.open(id)
    }
    fn destroy(&self, _id: FileId) -> BoxFuture<'_, Result<()>> {
        // DEFECT: Destroy is a no-op, key remains present!
        Box::pin(async move { Ok(()) })
    }
}

/// Mutant 2: second destroy returns an error instead of being idempotent.
#[derive(Clone, Default)]
pub struct MutantNonIdempotentDestroyKeyring {
    base: ReferenceFileKeyring,
    destroyed_once: Arc<parking_lot::Mutex<BTreeMap<FileId, bool>>>,
}

impl FileKeyring for MutantNonIdempotentDestroyKeyring {
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey>> {
        self.base.create(kind, id)
    }
    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey>> {
        self.base.open(id)
    }
    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<()>> {
        let mut map = self.destroyed_once.lock();
        if map.contains_key(&id) {
            // DEFECT: Returns error on second destroy!
            return Box::pin(async move {
                Err(ContextraError::Conflict(format!(
                    "file {id:?} already destroyed"
                )))
            });
        }
        map.insert(id, true);
        self.base.destroy(id)
    }
}

/// Helper executing reference test.
pub async fn run_reference_test() -> ConformanceResult {
    run(|| async { ReferenceFileKeyring::new() }).await
}

/// Helper executing mutant tests, ensuring 100% detection rate.
pub async fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| async { MutantOpenAfterDestroySucceedsKeyring::default() }).await;
    if res1.is_ok() {
        return Err("MutantOpenAfterDestroySucceedsKeyring escaped detection!".to_string());
    }

    let res2 = run(|| async { MutantNonIdempotentDestroyKeyring::default() }).await;
    if res2.is_ok() {
        return Err("MutantNonIdempotentDestroyKeyring escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn po_06_file_keyring_reference_passes() {
        let res = run_reference_test().await;
        assert!(res.is_ok(), "Reference must pass: {res:?}");
    }

    #[tokio::test]
    async fn po_06_file_keyring_mutants_fail() {
        let res = run_mutant_tests().await;
        assert!(res.is_ok(), "Mutants must be caught: {res:?}");
    }
}
