//! Zugriffsberechtigungen (ACL) für WAL-Dateien.
use crate::error::{StoreError, StoreResult as Result};
use std::path::Path;
use crate::wal::{WAL_V3_HEADER, WalVersion};
use super::scan::do_scan_entries_with_callback;

#[cfg(windows)]
pub(crate) fn set_restrictive_file_acl(path: &Path) -> Result<()> {
    contextra_sys::set_restrictive_file_acl(path)
        .map_err(|e| StoreError::Storage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_do_scan_entries_with_callback_v3_without_key_manager() {
        let dir = tempdir().expect("tempdir creation failed");
        let file_path = dir.path().join("test_v3.wal");

        {
            let mut file = crate::wal::fs::File::create(&file_path)
                .await
                .expect("file create failed");
            file.write_all(&WAL_V3_HEADER)
                .await
                .expect("write header failed");
            file.flush().await.expect("flush failed");
        }

        let mut file = crate::wal::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .await
            .expect("open read failed");

        let fallback_key = [1u8; 32];
        let mut scanned_entries = Vec::new();

        let dummy_legacy_flag = std::sync::atomic::AtomicBool::new(false);
        let res = do_scan_entries_with_callback(
            &mut file,
            4,
            &file_path,
            None,
            Some(fallback_key),
            false,
            &dummy_legacy_flag,
            |_seq, entry, _pos| {
                scanned_entries.push(entry);
                true
            },
        )
        .await;

        assert!(res.is_ok());
        assert_eq!(res.expect("scan result"), WalVersion::V3);
        assert!(scanned_entries.is_empty());
    }
}
