//! # ErasureLedger: Persistentes Fail-Closed Ledger (INV-ERA-4 / WP-8).
//!
//! Stellt sicher, dass jede Schlüsselvernichtung vor dem Ende der Lösch-Saga
//! dauerhaft und unveränderlich im Ledger verankert ist (INV-ERA-4).
//! Bei beschädigter oder nicht lesbarer Ledger-Datei verhält sich das System fail-closed.

use crate::error::ErasureError;
use crate::ledger::entry::{LedgerEntry, GENESIS_HASH};
use crate::types_local::Vfs;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Persistentes, fehlertolerantes und fail-closed Ledger für Erasure-Nachweise.
pub struct ErasureLedger {
    vfs: Arc<dyn Vfs>,
    path: PathBuf,
    ledger_key: [u8; 32],
    entries: RwLock<Vec<LedgerEntry>>,
    last_hash: RwLock<[u8; 32]>,
    max_gated_seq: RwLock<u64>,
}

impl ErasureLedger {
    /// Öffnet oder erstellt ein persistentes `ErasureLedger`.
    ///
    /// Fail-Closed: Wenn die Datei existiert, aber beschädigt ist (z. B. MAC-Mismatch
    /// oder Ketten-Unterbrechung), bricht das Öffnen sofort mit `Err(ErasureError::Corrupt)` ab.
    pub fn open_or_create(
        vfs: Arc<dyn Vfs>,
        path: &Path,
        ledger_key: &[u8; 32],
    ) -> Result<Self, ErasureError> {
        let mut entries = Vec::new();
        let mut last_hash = GENESIS_HASH;
        let mut max_gated_seq = 0u64;

        if vfs.exists(path).map_err(|e| ErasureError::Io(std::io::Error::other(e.to_string())))? {
            let data = vfs
                .read(path)
                .map_err(|e| ErasureError::Io(std::io::Error::other(e.to_string())))?;

            if !data.is_empty() {
                entries = bincode::deserialize::<Vec<LedgerEntry>>(&data)
                    .map_err(|_| ErasureError::Corrupt)?;

                // Fail-Closed Integritätsprüfung
                let mut prev = GENESIS_HASH;
                for entry in &entries {
                    if !entry.verify(&prev, ledger_key) {
                        return Err(ErasureError::Corrupt);
                    }
                    prev = entry.chain_hash;
                    if entry.gated_seq > max_gated_seq {
                        max_gated_seq = entry.gated_seq;
                    }
                }
                last_hash = prev;
            }
        }

        Ok(Self {
            vfs,
            path: path.to_path_buf(),
            ledger_key: *ledger_key,
            entries: RwLock::new(entries),
            last_hash: RwLock::new(last_hash),
            max_gated_seq: RwLock::new(max_gated_seq),
        })
    }

    /// Verankert eine Schlüsselvernichtung dauerhaft vor dem Abschluss der Saga (INV-ERA-4).
    pub fn record_key_destruction(
        &self,
        commitment: [u8; 32],
        range_tags: Vec<u8>,
        gated_seq: u64,
        completed_at: u64,
    ) -> Result<LedgerEntry, ErasureError> {
        let mut entries_guard = self.entries.write();
        let mut hash_guard = self.last_hash.write();
        let mut max_seq_guard = self.max_gated_seq.write();

        let entry = LedgerEntry::new(
            commitment,
            range_tags,
            gated_seq,
            completed_at,
            &hash_guard,
            &self.ledger_key,
        );

        *hash_guard = entry.chain_hash;
        if gated_seq > *max_seq_guard {
            *max_seq_guard = gated_seq;
        }

        entries_guard.push(entry.clone());

        // Atomare Persistenz auf Vfs
        let data = bincode::serialize(&*entries_guard)
            .map_err(|e| ErasureError::Internal(e.to_string()))?;
        self.vfs
            .write_atomic(&self.path, &data)
            .map_err(|e| ErasureError::Io(std::io::Error::other(e.to_string())))?;

        Ok(entry)
    }

    /// Liefert alle Einträge.
    pub fn entries(&self) -> Vec<LedgerEntry> {
        self.entries.read().clone()
    }

    /// Prüft die Kette auf Integrität.
    pub fn verify_integrity(&self) -> bool {
        let entries = self.entries.read();
        let mut prev = GENESIS_HASH;
        for entry in entries.iter() {
            if !entry.verify(&prev, &self.ledger_key) {
                return false;
            }
            prev = entry.chain_hash;
        }
        true
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_erasure_ledger_persistence_and_fail_closed() {
        // Einfache In-Memory Vfs Simulation
        #[derive(Default)]
        struct MemVfs {
            files: parking_lot::RwLock<std::collections::BTreeMap<PathBuf, Vec<u8>>>,
        }
        impl crate::types_local::Vfs for MemVfs {
            fn read(&self, path: &Path) -> Result<Vec<u8>, ErasureError> {
                self.files.read().get(path).cloned().ok_or_else(|| {
                    ErasureError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
                })
            }
            fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<(), ErasureError> {
                self.files.write().insert(path.to_path_buf(), data.to_vec());
                Ok(())
            }
            fn exists(&self, path: &Path) -> Result<bool, ErasureError> {
                Ok(self.files.read().contains_key(path))
            }
            fn create_dir_all(&self, _path: &Path) -> Result<(), ErasureError> { Ok(()) }
            fn remove_file(&self, path: &Path) -> Result<(), ErasureError> {
                self.files.write().remove(path);
                Ok(())
            }
            fn len(&self, path: &Path) -> Result<u64, ErasureError> {
                Ok(self.files.read().get(path).map(|v| v.len() as u64).unwrap_or(0))
            }
            fn list_dir(&self, _path: &Path) -> Result<Vec<String>, ErasureError> { Ok(vec![]) }
            fn sync_dir(&self, _path: &Path) -> Result<(), ErasureError> { Ok(()) }
        }

        let vfs = Arc::new(MemVfs::default());
        let path = PathBuf::from("erasure.ledger");
        let key = [0x42; 32];

        let ledger = ErasureLedger::open_or_create(vfs.clone(), &path, &key).unwrap();
        ledger.record_key_destruction([0x11; 32], vec![1, 2, 3], 1, 100).unwrap();
        assert!(ledger.verify_integrity());

        // Erneut öffnen
        let reopened = ErasureLedger::open_or_create(vfs.clone(), &path, &key).unwrap();
        assert_eq!(reopened.entries().len(), 1);
        assert!(reopened.verify_integrity());

        // Korruption provozieren (Fail-Closed)
        vfs.write_atomic(&path, b"corrupted bytes").unwrap();
        let fail = ErasureLedger::open_or_create(vfs.clone(), &path, &key);
        assert!(matches!(fail, Err(ErasureError::Corrupt)));
    }
}
