//! WAL Scanner (C4.3) Witness Integration Tests.
//!
//! Tests injection lines 14 (sealed segment), 15 (marker behind logical EOF in cleartext),
//! 16 (archived segment), and a clean control scan.

use contextra_types::{ContextraError, TxId};
use contextra_store::wal::{scan_wal_directory, Wal, WalOp, WalScanInputs, WalScopeSpec};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_scanner_control_clean_wal() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("clean.wal");

    let wal = Wal::open(&wal_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"normal_key".to_vec(),
        value: b"normal_val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"target_scope_".to_vec()],
            raw_needles: vec![b"SECRET_MARKER".to_vec()],
        },
        key_identifiers_claimed: false,
    };

    let evidence = scan_wal_directory(&inputs).await?;

    if evidence.segments_total != 1
        || evidence.segments_scanned != 1
        || evidence.segments_removed != 0
        || evidence.frames_matching_scope != 0
        || !evidence.is_valid()
    {
        return Err(ContextraError::Storage(
            "scan_clean_wal evidence mismatch".into(),
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_scanner_injection_line_14_sealed_segment_with_scope() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("sealed_scope.wal");

    let wal = Wal::open(&wal_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"target_scope_key1".to_vec(),
        value: b"val1".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    // Seal the segment
    let sealed_path = wal.rotate_and_seal().await?;
    if !sealed_path.exists() {
        return Err(ContextraError::Storage("sealed path missing".into()));
    }

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"target_scope_".to_vec()],
            raw_needles: vec![],
        },
        key_identifiers_claimed: false,
    };

    let evidence = scan_wal_directory(&inputs).await?;

    if evidence.segments_total != 1
        || evidence.segments_scanned != 1
        || evidence.frames_matching_scope == 0
        || evidence.invariant_violations().is_empty()
    {
        return Err(ContextraError::Storage("injection 14 mismatch".into()));
    }

    Ok(())
}

#[tokio::test]
async fn test_scanner_injection_line_15_marker_behind_logical_eof_cleartext(
) -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("tail_marker.wal");

    let wal = Wal::open(&wal_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"unrelated_key".to_vec(),
        value: b"unrelated_val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;
    wal.close().await?;

    // Append marker needle behind logical EOF
    let mut file_bytes = tokio::fs::read(&wal_path)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;
    file_bytes.extend_from_slice(b"---RESIDUAL_SECRET_MARKER---");
    tokio::fs::write(&wal_path, &file_bytes)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![],
            raw_needles: vec![b"RESIDUAL_SECRET_MARKER".to_vec()],
        },
        key_identifiers_claimed: false,
    };

    let evidence = scan_wal_directory(&inputs).await?;

    if evidence.tail_bytes_raw_scanned == 0
        || evidence.frames_matching_scope == 0
        || evidence.is_valid()
    {
        return Err(ContextraError::Storage("injection 15 mismatch".into()));
    }

    Ok(())
}

#[tokio::test]
async fn test_scanner_injection_line_16_archived_segment_with_scope() -> Result<(), ContextraError>
{
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("segment1.wal.archived");

    let wal = Wal::open(&wal_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"target_scope_archived".to_vec(),
        value: b"val_archived".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"target_scope_".to_vec()],
            raw_needles: vec![],
        },
        key_identifiers_claimed: false,
    };

    let evidence = scan_wal_directory(&inputs).await?;

    if evidence.segments_total != 1 || evidence.frames_matching_scope == 0 || evidence.is_valid() {
        return Err(ContextraError::Storage("injection 16 mismatch".into()));
    }

    Ok(())
}
