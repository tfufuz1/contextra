//! WAL Truncation, Rollback, Rewrite und Rotation.
//! Low-level WAL IO Submodule.
use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot};
use crate::error::{StoreError, StoreResult as Result};
use contextra_ids::TxId;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::wal::legacy_integrity_key;
use crate::wal::{
    PreparedBatch, Wal, WalCommand, WalEntry, WalOp, WalVersion, MAX_WAL_ENTRY_SIZE, WAL_V2_HEADER,
    WAL_V3_HEADER,
};

#[cfg(feature = "fault-injection")]
use crate::wal::{DELAY_APPEND_FOR_TX, DELAY_APPEND_MS, FAIL_APPEND_FOR_TX};

impl Wal {

    /// Scans the WAL entry by entry, executing full HMAC chain validation, CRC checks, and key manager decryption.
    ///
    /// Invokes `callback(seq_no, entry, end_offset)` for each valid entry.
    /// If `callback` returns `false`, scanning halts early.
    pub async fn scan_entries_with_callback<F>(
        &self,
        file_size: u64,
        mut callback: F,
    ) -> Result<WalVersion>
    where
        F: FnMut(u64, WalEntry, u64) -> bool,
    {
        let flusher_tx = {
            let guard = self.flusher_tx.read().unwrap_or_else(|e| e.into_inner());
            guard.clone()
        };

        let tx = flusher_tx
            .ok_or_else(|| StoreError::Storage("WAL flusher actor is not enabled".into()))?;

        let (item_tx, mut item_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();

        tx.send(WalCommand::Scan {
            file_size,
            item_tx,
            ack: ack_tx,
        })
        .await
        .map_err(|_| StoreError::Storage("WAL flusher channel closed".into()))?;

        let mut stopped = false;
        while let Some((seq, entry, pos)) = item_rx.recv().await {
            if !stopped && !callback(seq, entry, pos) {
                stopped = true;
            }
        }

        let version = ack_rx
            .await
            .map_err(|_| StoreError::Storage("WAL flusher dropped".into()))??;

        Ok(version)
    }

    /// Rewrites legacy V1 or V2 WAL files as V3.
    pub(crate) async fn rewrite_as_v3(
        &self,
        replayed_entries: &[(u64, WalEntry, u64)],
    ) -> Result<()> {
        let integrity_key = self.get_integrity_key()?;

        let flusher_tx = {
            let guard = self.flusher_tx.read().unwrap_or_else(|e| e.into_inner());
            guard.clone()
        };

        let tx = flusher_tx
            .ok_or_else(|| StoreError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Rewrite {
            replayed_entries: replayed_entries.to_vec(),
            integrity_key,
            ack: ack_tx,
        })
        .await
        .map_err(|_| StoreError::Storage("WAL flusher channel closed".into()))?;

        ack_rx
            .await
            .map_err(|_| StoreError::Storage("WAL flusher dropped".into()))??;

        Ok(())
    }

    pub async fn truncate(&self, offset: u64, new_last_hmac: [u8; 32]) -> Result<()> {
        let _truncate_guard = self.truncate_lock.lock().await;

        if self.is_sealed() {
            return Err(StoreError::Storage(format!(
                "Cannot truncate sealed WAL segment {}",
                self.path.display()
            )));
        }

        let flusher_tx = {
            let guard = self.flusher_tx.read().unwrap_or_else(|e| e.into_inner());
            guard.clone()
        };
        let tx = flusher_tx
            .ok_or_else(|| StoreError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Truncate {
            offset,
            new_last_hmac,
            ack: ack_tx,
        })
        .await
        .map_err(|_| StoreError::Storage("WAL flusher channel closed".into()))?;

        ack_rx.await.map_err(|_| {
            StoreError::Storage("WAL flusher dropped ack for truncate command".into())
        })??;

        Ok(())
    }

    pub async fn rotate_and_seal(&self) -> Result<PathBuf> {
        self.sealed.store(true, std::sync::atomic::Ordering::SeqCst);

        let flusher_tx = {
            let mut guard = self
                .flusher_tx
                .write()
                .map_err(|_| StoreError::Storage("flusher_tx RwLock poisoned".into()))?;
            guard.take()
        };

        let tx = flusher_tx
            .ok_or_else(|| StoreError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Seal { ack: ack_tx })
            .await
            .map_err(|_| StoreError::Storage("WAL flusher channel closed".into()))?;

        let sealed_path = ack_rx
            .await
            .map_err(|_| StoreError::Storage("WAL flusher dropped".into()))??;

        Ok(sealed_path)
    }

    pub async fn find_tx_offset(&self, target_tx_id: TxId) -> Result<(u64, [u8; 32])> {
        let metadata = crate::wal::fs::metadata(&self.path)
            .await
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        let file_size = metadata.len();

        let mut last_offset = 0u64;
        let mut last_hmac = [0u8; 32];
        let mut found_rollback_point = false;

        self.scan_entries_with_callback(file_size, |_seq, entry, offset| {
            let entry_tx = entry.tx_id().inner();
            if target_tx_id.inner() < TxId::INTERNAL_BASE && entry_tx >= TxId::INTERNAL_BASE {
                last_offset = offset;
                last_hmac = entry.checksum;
                return true;
            }

            if entry_tx > target_tx_id.inner() {
                found_rollback_point = true;
                return false;
            }
            last_offset = offset;
            last_hmac = entry.checksum;
            true
        })
        .await?;

        let _ = found_rollback_point;
        Ok((last_offset, last_hmac))
    }

    /// Returns a snapshot of the last HMAC written to the log.
    pub async fn last_hmac_snapshot(&self) -> [u8; 32] {
        *self.last_hmac.lock().await
    }

    /// Aktiviert eine einmalige simulierte Truncate-I/O-Failure für Tests.
    /// Der Flusher gibt beim nächsten `WalCommand::Truncate` einen Fehler zurück,
    /// ohne `set_len()` auszuführen. `size` wird daher nicht aktualisiert.
    ///
    /// Setzt `FAIL_TRUNCATE_ONCE = true`. Wird vom Flusher nach Auslösung zurückgesetzt.
    #[cfg(feature = "fault-injection")]
    pub fn arm_truncate_failure_for_test(&self) {
        crate::wal::FAIL_TRUNCATE_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
