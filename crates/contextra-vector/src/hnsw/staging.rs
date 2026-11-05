//! # Lokaler Transaktions-Staging-Puffer für Vektoroperationen
//!
//! Implementiert einen indexlokalen Staging-Puffer für Operationen innerhalb einer
//! Transaktion (`TxId`), getrennt für uncommitted Writes und Deletes.
//! Ersetzt die externe `contextra-mvcc`-Abhängigkeit im Vektorindex gemäß SPEC D-88.

use ahash::AHashMap;
use contextra_types::{DocId, Result, TxId};
use parking_lot::RwLock;

/// Lokale Vektoroperation im Staging-Puffer.
#[derive(Debug, Clone)]
pub enum LocalVectorOp {
    /// Einfügen eines Vektors.
    Insert { doc_id: DocId, data: Vec<f32> },
    /// Löschen eines Dokuments.
    Delete { doc_id: DocId },
}

/// Indexlokaler Transaktions-Puffer.
#[derive(Debug, Default)]
pub struct LocalTxBuffer {
    shards: RwLock<AHashMap<TxId, Vec<LocalVectorOp>>>,
}

impl LocalTxBuffer {
    /// Erzeugt einen neuen leeren Staging-Puffer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shards: RwLock::new(AHashMap::new()),
        }
    }

    /// Nimmt eine Operation für die angegebene Transaktion auf.
    pub fn stage(&self, tx: TxId, op: LocalVectorOp) -> Result<()> {
        let mut map = self.shards.write();
        map.entry(tx).or_default().push(op);
        Ok(())
    }

    /// Entnimmt alle gestagten Operationen einer Transaktion.
    pub fn drain(&self, tx: TxId) -> Vec<LocalVectorOp> {
        let mut map = self.shards.write();
        map.remove(&tx).unwrap_or_default()
    }

    /// Verwirft alle gestagten Operationen einer Transaktion (z. B. bei Rollback).
    pub fn discard(&self, tx: TxId) {
        let mut map = self.shards.write();
        map.remove(&tx);
    }
}
