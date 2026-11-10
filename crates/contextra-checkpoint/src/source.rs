//! # Lokales SnapshotSource Trait und Dateisystem-Naht
//!
//! Entkoppelt `contextra-checkpoint` von `contextra_ports::StorageEngine` und `contextra_ports::Vfs`.
//!
//! ## Invarianten
//! - `SnapshotSource`: Exakt die im Checkpoint-Kern benötigten Operationen (13 Methoden).
//! - `FsSeam`: Minimale synchrone und asynchrone Dateisystem-Operationen für Hardlink-Cloner und Orphan-Registry.

use bytes::Bytes;
use contextra_types::{Result, TxId};
use std::future::Future;
use std::pin::Pin;

/// Typalias für gepinnte Heap-Futures (lokal analog zu BoxFuture).
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Typalias für Scan-Prefix Schlüssel-Wert-Paare.
pub type KeyValuePair = (Vec<u8>, Vec<u8>);

/// Lokale Schnittstelle zur Speicher-Engine für Snapshotting, Checkpointing und Rollbacks.
///
/// Enthält exakt die Methoden, die von `PersistentCheckpointStore`, `CheckpointGuard`
/// und `InstanceOrphanRegistry` aufgerufen werden.
pub trait SnapshotSource: Send + Sync + 'static {
    /// Liest einen Wert zu einem Schlüssel.
    fn get<'a>(&'a self, key: &'a [u8]) -> LocalBoxFuture<'a, Result<Option<Bytes>>>;

    /// Schreibt einen Schlüssel-Wert-Eintrag im Kontext einer Transaktion.
    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> LocalBoxFuture<'a, Result<()>>;

    /// Bestätigt eine Transaktion atomar.
    fn commit<'a>(&'a self, tx_id: TxId) -> LocalBoxFuture<'a, Result<()>>;

    /// Bricht eine offene Transaktion ab.
    fn rollback<'a>(&'a self, tx_id: TxId) -> LocalBoxFuture<'a, Result<()>>;

    /// Rollt den Zustand der Engine bis auf `tx_id` zurück.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> LocalBoxFuture<'a, Result<()>>;

    /// Synchronisiert alle ausstehenden Schreiboperationen auf ein persistentes Medium.
    fn flush<'a>(&'a self) -> LocalBoxFuture<'a, Result<()>>;

    /// Liefert die höchste bestätigte Sequenznummer.
    fn last_seq_no<'a>(&'a self) -> LocalBoxFuture<'a, Result<u64>>;

    /// Liefert die höchste vergebene Transaktions-ID.
    fn last_tx_id<'a>(&'a self) -> LocalBoxFuture<'a, Result<TxId>>;

    /// Pinnt eine Sequenznummer gegen Compaction und Bereinigung.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> LocalBoxFuture<'a, Result<()>>;

    /// Gibt eine gepinnte Sequenznummer wieder frei.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> LocalBoxFuture<'a, Result<()>>;

    /// Löscht einen Schlüssel innerhalb einer Transaktion.
    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> LocalBoxFuture<'a, Result<()>>;

    /// Scannt alle Schlüssel-Wert-Paare mit dem angegebenen Präfix.
    fn scan_prefix<'a>(&'a self, prefix: &'a [u8])
        -> LocalBoxFuture<'a, Result<Vec<KeyValuePair>>>;
}

// Blanket implementation for any StorageEngine from contextra_ports
impl<T: contextra_ports::StorageEngine + ?Sized> SnapshotSource for T {
    fn get<'a>(&'a self, key: &'a [u8]) -> LocalBoxFuture<'a, Result<Option<Bytes>>> {
        self.get(key)
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> LocalBoxFuture<'a, Result<()>> {
        self.put(tx_id, key, value)
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> LocalBoxFuture<'a, Result<()>> {
        self.commit(tx_id)
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> LocalBoxFuture<'a, Result<()>> {
        self.rollback(tx_id)
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> LocalBoxFuture<'a, Result<()>> {
        self.rollback_to_tx(tx_id)
    }

    fn flush<'a>(&'a self) -> LocalBoxFuture<'a, Result<()>> {
        self.flush()
    }

    fn last_seq_no<'a>(&'a self) -> LocalBoxFuture<'a, Result<u64>> {
        self.last_seq_no()
    }

    fn last_tx_id<'a>(&'a self) -> LocalBoxFuture<'a, Result<TxId>> {
        self.last_tx_id()
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> LocalBoxFuture<'a, Result<()>> {
        self.pin_checkpoint(seq_no)
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> LocalBoxFuture<'a, Result<()>> {
        self.unpin_checkpoint(seq_no)
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> LocalBoxFuture<'a, Result<()>> {
        self.delete(tx_id, key)
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> LocalBoxFuture<'a, Result<Vec<KeyValuePair>>> {
        self.scan_prefix(prefix)
    }
}
