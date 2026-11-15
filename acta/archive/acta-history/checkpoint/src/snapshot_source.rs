//! Lokales SnapshotSource Trait zur Entkopplung der Speicher-Engine.
//!
//! Exakt die 12 Methoden, die in `src/` von der StorageEngine aufgerufen werden.

use crate::error::CheckpointResult as Result;
use bytes::Bytes;
use acta_ids::TxId;
use std::future::Future;
use std::pin::Pin;

/// Lokaler Alias für pinning-boxed Futures ohne externe Abhängigkeit.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Lokale Schnittstelle zur Speicher-Engine für Snapshot-, Checkpoint- und Rollback-Mechanik.
pub trait SnapshotSource: Send + Sync + 'static {
    /// Liest einen Wert zu einem Schlüssel.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Schreibt ein Schlüssel-Wert-Paar im Transaktionskontext.
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Löscht einen Schlüssel im Transaktionskontext.
    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Bestätigt eine Transaktion.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Rollt eine Transaktion zurück.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Setzt die Engine auf den Zustand einer früheren Transaktion zurück.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;

    /// Flusht anstehende Daten auf die Festplatte.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>;

    /// Liefert die höchste vergebene Sequenznummer.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;

    /// Liefert die ID der letzten Transaktion.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;

    /// Pinnt einen Snapshot bei der angegebenen Sequenznummer gegen GC.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Gibt den gepinnten Snapshot wieder frei.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;

    /// Scannt Schlüssel mit dem angegebenen Präfix.
    #[allow(clippy::type_complexity)]
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
}
