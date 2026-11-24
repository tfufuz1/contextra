//! # Facade-Methoden fuer LsmStorage (Ring 1)
//!
//! ZWECK: Bereitstellung von inherenten Hilfsmethoden auf `LsmStorage`, die auf
//! die Subsysteme `read`, `write`, `maintenance` und `compaction` weiterleiten.

use crate::error::StoreResult as Result;
use crate::lsm::engine::LsmStorage;
use contextra_ids::TxId;

impl LsmStorage {
    /// Inherent read operations for standalone usage.
    pub async fn get<'a>(&'a self, key: &'a [u8]) -> Result<Option<bytes::Bytes>> {
        crate::lsm::ops::read::get(self, key).await
    }

    /// Punktlesen zu einer bestimmten Sequenznummer.
    pub async fn get_at_seq<'a>(&'a self, key: &'a [u8], seq_no: u64) -> Result<Option<bytes::Bytes>> {
        crate::lsm::ops::read::get_at_seq(self, key, seq_no).await
    }

    /// Praefix-Scan ueber alle Schichten.
    pub async fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        crate::lsm::ops::read::scan_prefix(self, prefix).await
    }

    /// Begrenzter Praefix-Scan mit Paginierungs-Cursor.
    pub async fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)> {
        crate::lsm::ops::read::scan_prefix_bounded(self, prefix, limit, cursor).await
    }

    /// Praefix-Scan zu einer bestimmten Sequenznummer.
    pub async fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        crate::lsm::ops::read::scan_prefix_at(self, prefix, seq_no).await
    }

    /// Bereichs-Scan mit Grenzen und Paginierungs-Cursor.
    pub async fn scan_bounded<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)> {
        crate::lsm::ops::read::scan_bounded(self, start, end, limit, cursor).await
    }

    /// Bereichs-Scan ueber Schichten mit optionalem Limit.
    pub async fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        crate::lsm::ops::read::scan(self, start, end, limit).await
    }

    /// Inherent write and maintenance operations.
    pub async fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> Result<()> {
        crate::lsm::ops::write::put(self, tx_id, key, value).await
    }

    /// Einfuegen nur wenn Schluessel nicht vorhanden.
    pub async fn put_if_absent<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> Result<bool> {
        crate::lsm::ops::write::put_if_absent(self, tx_id, key, value).await
    }

    /// Loeschen mehrerer Schluessel.
    pub async fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> Result<u64> {
        crate::lsm::ops::write::delete_many(self, tx_id, keys).await
    }

    /// Loeschen aller Schluessel mit Praefix.
    pub async fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> Result<u64> {
        crate::lsm::ops::write::delete_prefix(self, tx_id, prefix).await
    }

    /// Loeschen eines einzelnen Schluessels.
    pub async fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> Result<()> {
        crate::lsm::ops::write::delete(self, tx_id, key).await
    }

    /// Transaktion committen.
    pub async fn commit<'a>(&'a self, tx_id: TxId) -> Result<()> {
        crate::lsm::ops::write::commit(self, tx_id).await
    }

    /// Transaktion zurueckrollen.
    pub async fn rollback<'a>(&'a self, tx_id: TxId) -> Result<()> {
        crate::lsm::ops::maintenance::rollback(self, tx_id).await
    }

    /// Checkpoint pinnen.
    pub async fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> Result<()> {
        crate::lsm::ops::maintenance::pin_checkpoint(self, seq_no).await
    }

    /// Checkpoint freigeben.
    pub async fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> Result<()> {
        crate::lsm::ops::maintenance::unpin_checkpoint(self, seq_no).await
    }

    /// MemTable synchron flushen.
    pub async fn flush<'a>(&'a self) -> Result<()> {
        crate::lsm::ops::compaction::flush(self).await
    }

    /// Speicherstatistiken ermitteln.
    pub async fn stats<'a>(&'a self) -> Result<crate::stats::StorageStats> {
        crate::lsm::ops::maintenance::stats(self).await
    }

    /// Letzte Sequenznummer abfragen.
    pub async fn last_seq_no<'a>(&'a self) -> Result<u64> {
        crate::lsm::ops::maintenance::last_seq_no(self).await
    }

    /// Letzte Transaktions-ID abfragen.
    pub async fn last_tx_id<'a>(&'a self) -> Result<TxId> {
        crate::lsm::ops::maintenance::last_tx_id(self).await
    }
}
