//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-13
//! Zweck: Integrationstests fuer die Haertung des WAL-Witness Scanners (A14 / T-2026-LA14).
//! Invarianten:
//! - Lesefehler im Verzeichnis muessen fehlschlagen (fail-closed, Err).
//! - Segment x.ledger.wal darf nicht als Sidecar skipped werden.
//! - Schreibzugriffe nach Stichtag (tx > as_of_tx) erzeugen post_cutoff_hits.
//! - Unklassifizierte Dateien invalidieren den Beweis.
//! - Tail-Inspektion mit Raw-Needles laeuft auch unter encrypted_mode.

use contextra_types::{ContextraError, TxId};
use contextra_store::wal::{
    classify_wal_filename, scan_wal_directory, Wal, WalFileType, WalOp, WalScanInputs, WalScopeSpec,
};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_directory_read_error_returns_err() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let fake_dir = dir.path().join("non_existent_wal_directory");

    let inputs = WalScanInputs {
        wal_dir: fake_dir,
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"scope_".to_vec()],
            raw_needles: vec![],
        },
        key_identifiers_claimed: false,
    };

    let res = scan_wal_directory(&inputs).await;
    assert!(
        res.is_err(),
        "Non-existent WAL directory read must fail closed with Err"
    );
    Ok(())
}

#[tokio::test]
async fn test_ledger_wal_segment_is_scanned_not_skipped() -> Result<(), ContextraError> {
    assert_eq!(classify_wal_filename("x.ledger.wal"), WalFileType::Active);

    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("x.ledger.wal");

    let wal = Wal::open(&wal_path).await?;
    let tx_id = TxId::try_from(10u64).map_err(|e| ContextraError::Storage(e.to_string()))?;
    let op = WalOp::Put {
        tx_id,
        key: b"unrelated_key".to_vec(),
        value: b"unrelated_val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"scope_prefix_".to_vec()],
            raw_needles: vec![],
        },
        key_identifiers_claimed: false,
    };

    let evidence = scan_wal_directory(&inputs).await?;
    assert_eq!(evidence.segments_total, 1);
    assert_eq!(evidence.segments_scanned, 1);
    assert_eq!(evidence.unclassified_files_found, 0);
    assert_eq!(evidence.frames_matching_scope, 0);
    assert!(evidence.is_valid());

    Ok(())
}

#[tokio::test]
async fn test_post_cutoff_put_hit() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("post_cutoff.wal");

    let wal = Wal::open(&wal_path).await?;
    let tx_id = TxId::try_from(20u64).map_err(|e| ContextraError::Storage(e.to_string()))?;
    let op = WalOp::Put {
        tx_id,
        key: b"target_scope_key".to_vec(),
        value: b"val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 10, // Stichtag is 10, write occurred at tx 20
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"target_scope_".to_vec()],
            raw_needles: vec![],
        },
        key_identifiers_claimed: false,
    };

    let evidence = scan_wal_directory(&inputs).await?;
    assert_eq!(evidence.post_cutoff_hits, 1);
    assert_eq!(evidence.frames_matching_scope, 1);
    assert!(
        !evidence.is_valid(),
        "Post-cutoff write hits must invalidate evidence"
    );

    let violations = evidence.invariant_violations();
    assert!(
        violations
            .iter()
            .any(|v| v.contains("Post-cutoff write hits present")),
        "Violation log must mention post-cutoff write hits"
    );

    Ok(())
}

#[tokio::test]
async fn test_unclassified_file_invalidates_evidence() -> Result<(), ContextraError> {
    assert_eq!(
        classify_wal_filename("unknown.txt"),
        WalFileType::Unclassified
    );
    assert_eq!(classify_wal_filename("data.sst"), WalFileType::Unclassified);

    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let unclass_path = dir.path().join("unclassified_data.txt");
    tokio::fs::write(&unclass_path, b"random_content")
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"scope_".to_vec()],
            raw_needles: vec![],
        },
        key_identifiers_claimed: false,
    };

    let evidence = scan_wal_directory(&inputs).await?;
    assert_eq!(evidence.unclassified_files_found, 1);
    assert!(
        !evidence.is_valid(),
        "Unclassified file must invalidate scan evidence"
    );

    Ok(())
}

#[tokio::test]
async fn test_exact_sidecar_allowlist_classification() {
    // Exact sidecars must be classified as Sidecar
    assert_eq!(classify_wal_filename("MANIFEST"), WalFileType::Sidecar);
    assert_eq!(classify_wal_filename("SALT"), WalFileType::Sidecar);
    assert_eq!(classify_wal_filename("LOCK"), WalFileType::Sidecar);
    assert_eq!(classify_wal_filename("purge.intent"), WalFileType::Sidecar);
    assert_eq!(
        classify_wal_filename("pending_delete.journal"),
        WalFileType::Sidecar
    );
    assert_eq!(
        classify_wal_filename(".wal_integrity_key"),
        WalFileType::Sidecar
    );
    assert_eq!(
        classify_wal_filename("wal-001.log.rekeyed"),
        WalFileType::Sidecar
    );
    assert_eq!(
        classify_wal_filename("wal-001.log.uuid"),
        WalFileType::Sidecar
    );

    // Non-allowlisted or substring matches must NOT be sidecars
    assert_ne!(classify_wal_filename("my.ledger.wal"), WalFileType::Sidecar);
    assert_ne!(classify_wal_filename("some_file.sst"), WalFileType::Sidecar);
    assert_ne!(classify_wal_filename("file.tmp.123"), WalFileType::Sidecar);
    assert_ne!(classify_wal_filename(".hidden_wal"), WalFileType::Sidecar);
}

#[tokio::test]
async fn test_tail_raw_needle_inspection_in_encrypted_mode() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("encrypted_tail.wal");

    let wal = Wal::open(&wal_path).await?;
    let tx_id = TxId::try_from(10u64).map_err(|e| ContextraError::Storage(e.to_string()))?;
    let op = WalOp::Put {
        tx_id,
        key: b"clean_key".to_vec(),
        value: b"clean_val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;
    wal.close().await?;

    // Append raw needle after logical EOF
    let mut bytes = tokio::fs::read(&wal_path)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;
    bytes.extend_from_slice(b"---SECRET_KEY_NEEDLE---");
    tokio::fs::write(&wal_path, &bytes)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: true, // Verification in encrypted_mode must still check raw needles
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"scope_".to_vec()],
            raw_needles: vec![b"SECRET_KEY_NEEDLE".to_vec()],
        },
        key_identifiers_claimed: true,
    };

    let evidence = scan_wal_directory(&inputs).await?;
    assert!(
        evidence.tail_bytes_raw_scanned > 0,
        "Tail bytes must be scanned in encrypted_mode"
    );
    assert_eq!(
        evidence.frames_matching_scope, 1,
        "Raw needle match in tail must be reported"
    );
    assert!(!evidence.is_valid());

    Ok(())
}
