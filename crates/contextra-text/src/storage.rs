//! Lokales Trait `PostingStore` für Speicherung und Abruf von Posting-Listen und Metadaten (SPEC §6.2 text, B04).
// ANCHOR[contextra-text:STORE-001]

use bytes::Bytes;
use contextra_ids::TxId;
use std::future::Future;
use std::pin::Pin;

/// Typ-Alias für asynchrone Ergebnisse.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Lokale Speicherabstraktion für den invertierten Index.
///
/// Enthält exakt die im Kern genutzten Methoden aus `StorageEngine` (Inventar aus TX6.0).
pub trait PostingStore: Send + Sync {
    /// Liest den Wert zu einem Schlüssel.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>>;

    /// Liest den Wert zu einem Schlüssel unter Snapshot-Isolation zur Sequenznummer `seq`.
    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>>;

    /// Schreibt einen Schlüssel/Wert-Eintrag im Kontext der Transaktion `tx`.
    fn put<'a>(
        &'a self,
        tx: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<()>>;

    /// Löscht einen Schlüssel im Kontext der Transaktion `tx`.
    fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<()>>;

    /// Bestätigt die Transaktion `tx`.
    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>>;

    /// Rollt die Transaktion `tx` zurück.
    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>>;

    /// Rollt alle Transaktionen bis einschließlich `tx_id` zurück.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, crate::error::Result<()>>;

    /// Liefert die höchste im Speicher vergebene Transaktions-ID.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<TxId>>;

    /// Liefert die aktuelle MVCC-Sequenznummer.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<u64>>;

    /// Scannt Schlüssel/Werte mit dem gegebenen Präfix.
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>>;

    /// Scannt Schlüssel/Werte mit dem gegebenen Präfix unter Snapshot-Isolation.
    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>>;
}
