#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Specification tests for ADR-N12 (High-Water-Mark Anchoring) and ADR-N13 (Frame Header V4 Integrity).
//!
//! These pending tests specify the target behavior defined in ADR-N12 and ADR-N13.
//! They are ignored until the corresponding production implementations (H1, H2) are integrated.

use contextra_core::{ContextraError, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::manifest::{Manifest, ManifestEntry};
use contextra_store::wal::{Wal, WalOp};
use tempfile::tempdir;
use tokio::fs;

// ============================================================================
// ADR-N12 Specification Tests (High-Water-Mark-Verankerung - H2)
// ============================================================================

/// ADR-N12 Spec Test 1: Verifies that WAL tail truncation is reliably detected
/// after SSTable flush and startup when Manifest High-Water-Mark anchoring is active.
///
/// Target Behavior (ADR-N12 / INV-WAL-TRUNCATION-1):
/// After a MemTable flush creates an SSTable and writes a `WalCheckpoint` to Manifest,
/// any uncommitted or committed truncation of the active WAL tail before the HWM
/// MUST fail LsmStorage startup with `ContextraError::WalTruncationDetected`.
#[tokio::test]
#[ignore = "ADR-N12 pending"]
async fn spec_n12_wal_tail_truncation_detection_after_sstable_flush() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .expect("write salt");

    let manifest_path = dir.path().join("MANIFEST");
    let wal_path = dir.path().join("wal.log");

    // 1. Write initial WAL entries and record checkpoint HWM in MANIFEST
    let checkpoint_hwm = {
        let wal = Wal::open(&wal_path).await.expect("open wal");
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"spec_k1".to_vec(),
            value: b"spec_v1".to_vec(),
        };
        let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.expect("batch 1");
        wal.append_batch(batch1).await.expect("append 1");

        let op2 = WalOp::Put {
            tx_id: TxId::new(2),
            key: b"spec_k2".to_vec(),
            value: b"spec_v2".to_vec(),
        };
        let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await.expect("batch 2");
        wal.append_batch(batch2).await.expect("append 2");

        let hwm = wal.last_hmac_snapshot().await;
        drop(wal);

        let manifest = Manifest::open(&manifest_path).await.expect("manifest open");
        manifest
            .append(&ManifestEntry::WalCheckpoint { hmac: hwm })
            .await
            .expect("checkpoint append");

        hwm
    };

    // 2. Truncate active wal.log to remove the second entry
    let wal_read = Wal::open(&wal_path).await.expect("open wal read");
    let entries = wal_read.replay().await.expect("replay entries");
    assert_eq!(entries.len(), 2);
    let (_, _, offset_first) = entries[0];
    drop(wal_read);

    let mut wal_bytes = fs::read(&wal_path).await.expect("read wal");
    wal_bytes.truncate(offset_first as usize);
    fs::write(&wal_path, wal_bytes)
        .await
        .expect("write truncated wal");

    // 3. Opening LSM storage MUST fail with WalTruncationDetected even if valid SSTables exist
    let reopen_res = LsmStorage::new(config).await;
    assert!(
        reopen_res.is_err(),
        "LsmStorage startup MUST detect external WAL truncation against MANIFEST HWM"
    );
    match reopen_res.err().unwrap() {
        ContextraError::WalTruncationDetected {
            expected_hmac,
            actual_hmac,
        } => {
            assert_eq!(expected_hmac, checkpoint_hwm);
            assert_ne!(actual_hmac, expected_hmac);
        }
        err => panic!("Expected WalTruncationDetected, got: {:?}", err),
    }
}

/// ADR-N12 Spec Test 2: Verifies cross-segment HMAC chain continuity.
///
/// Target Behavior (ADR-N12 / Option B):
/// Segment N+1 embeds `prev_segment_last_hmac` from Segment N in its header.
/// Deleting or swapping intermediate segment N MUST cause replay of Segment N+1 to fail
/// with `ContextraError::WalTruncationDetected` or `ContextraError::WalCorruption`.
#[tokio::test]
#[ignore = "ADR-N12 pending"]
async fn spec_n12_cross_segment_hmac_chain_continuity() {
    let dir = tempdir().expect("tempdir");

    let wal1_path = dir.path().join("wal-00000000000000000001.log");
    let wal2_path = dir.path().join("wal-00000000000000000002.log");

    // Create Segment 1
    let wal1 = Wal::open(&wal1_path).await.expect("open wal1");
    let op1 = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"seg1_k".to_vec(),
        value: b"seg1_v".to_vec(),
    };
    let (batch1, _) = wal1.prepare_batch(vec![(op1, 10)]).await.expect("batch1");
    wal1.append_batch(batch1).await.expect("append batch1");
    let seg1_last_hmac = wal1.last_hmac_snapshot().await;
    drop(wal1);

    // Create Segment 2 chained to Segment 1
    let wal2 = Wal::open(&wal2_path).await.expect("open wal2");
    let op2 = WalOp::Put {
        tx_id: TxId::new(11),
        key: b"seg2_k".to_vec(),
        value: b"seg2_v".to_vec(),
    };
    let (batch2, _) = wal2.prepare_batch(vec![(op2, 11)]).await.expect("batch2");
    wal2.append_batch(batch2).await.expect("append batch2");
    drop(wal2);

    // Deleting Segment 1 interrupts the cross-segment HMAC chain for Segment 2
    fs::remove_file(&wal1_path).await.expect("remove seg 1");

    let wal2_reopen = Wal::open(&wal2_path).await.expect("reopen seg 2");
    let replay_res = wal2_reopen.replay().await;
    assert!(
        replay_res.is_err(),
        "Replaying segment 2 without valid preceding segment 1 HMAC ({:?}) MUST fail",
        seg1_last_hmac
    );
}

/// ADR-N12 Spec Test 3: Verifies periodic Manifest HWM checkpointing.
///
/// Target Behavior (ADR-N12 / Option B):
/// Periodically written `WalCheckpoint` entries in MANIFEST provide a monotonically
/// advancing high-water-mark anchor that is verified upon startup.
#[tokio::test]
#[ignore = "ADR-N12 pending"]
async fn spec_n12_manifest_periodic_hwm_checkpoint_verification() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("manifest open");
    let hmac_check = [0x55u8; 32];

    manifest
        .append(&ManifestEntry::WalCheckpoint { hmac: hmac_check })
        .await
        .expect("append checkpoint");
    drop(manifest);

    let entries = Manifest::load(&manifest_path).await.expect("manifest load");
    let extracted_hwm = Manifest::extract_high_water_mark(&entries);

    assert_eq!(
        extracted_hwm,
        Some(hmac_check),
        "Manifest extract_high_water_mark MUST return the last written periodic HWM"
    );
}

// ============================================================================
// ADR-N13 Specification Tests (WAL-Frame-V4 Header-Integrität - H1)
// ============================================================================

/// ADR-N13 Spec Test 1: Bitflip in 4-byte length header of a middle entry fails hard.
///
/// Target Behavior (ADR-N13 / H1):
/// In WAL V4 (`MFW4`), each frame has a protected header `[len: u32][header_crc: u32]`.
/// A bitflip in `len` causes `header_crc` validation to fail. When this occurs for a middle entry,
/// the reader MUST return `ContextraError::WalCorruption` rather than silently breaking
/// and losing subsequent valid commits.
#[tokio::test]
#[ignore = "ADR-N13 pending"]
async fn spec_n13_v4_frame_header_crc_bitflip_fails_hard_middle_entry() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("v4_corrupt_middle.wal");

    // Write 3 WAL entries
    {
        let wal = Wal::open(&wal_path).await.expect("open wal");
        for i in 1..=3 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k_{}", i).into_bytes(),
                value: format!("v_{}", i).into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await.expect("prepare");
            wal.append_batch(batch).await.expect("append");
        }
    }

    // Mutate the length header of entry #2 (middle entry)
    let mut bytes = fs::read(&wal_path).await.expect("read wal");
    assert!(
        bytes.len() > 100,
        "WAL file must be large enough for 3 entries"
    );

    // Corrupt a middle length byte (e.g., offset 60)
    bytes[60] ^= 0xFF;
    fs::write(&wal_path, bytes).await.expect("write corrupted");

    // Replay MUST return ContextraError::WalCorruption, NOT a truncated Ok(vec) with entry 1
    let wal = Wal::open(&wal_path).await.expect("open corrupted wal");
    let replay_res = wal.replay().await;

    assert!(
        replay_res.is_err(),
        "Corrupted length header in middle entry MUST fail with WalCorruption, not silent truncation"
    );
    match replay_res.err().unwrap() {
        ContextraError::WalCorruption { .. } => {}
        err => panic!("Expected WalCorruption error, got: {:?}", err),
    }
}

/// ADR-N13 Spec Test 2: Clean torn tail distinguished from middle header corruption.
///
/// Target Behavior (ADR-N13 / H1 Policy Matrix):
/// An incomplete write or zero-fill padding at the exact EOF (file tail) is accepted as a
/// clean torn write during crash recovery. A corrupted header before EOF MUST trigger `WalCorruption`.
#[tokio::test]
#[ignore = "ADR-N13 pending"]
async fn spec_n13_v4_clean_torn_tail_distinguished_from_middle_corrupted_header() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("v4_torn_tail.wal");

    // Write 2 valid entries
    {
        let wal = Wal::open(&wal_path).await.expect("open wal");
        for i in 1..=2 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k_{}", i).into_bytes(),
                value: format!("v_{}", i).into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await.expect("prepare");
            wal.append_batch(batch).await.expect("append");
        }
    }

    // Append 3 incomplete bytes at EOF (torn write at tail)
    let mut bytes = fs::read(&wal_path).await.expect("read wal");
    bytes.extend_from_slice(&[0x01, 0x02, 0x03]);
    fs::write(&wal_path, bytes).await.expect("write torn tail");

    // Replay MUST succeed by recovering the 2 valid entries before the torn tail
    let wal = Wal::open(&wal_path).await.expect("open wal with torn tail");
    let replay_res = wal.replay().await;

    assert!(
        replay_res.is_ok(),
        "Clean torn tail write at EOF MUST be recovered without error: {:?}",
        replay_res.err()
    );
    let entries = replay_res.unwrap();
    assert_eq!(
        entries.len(),
        2,
        "Should recover exactly the 2 valid entries preceding torn tail"
    );
}

/// ADR-N13 Spec Test 3: Backward compatibility with V2/V3 and transparent V4 rewrite.
///
/// Target Behavior (ADR-N13 / Migration Path):
/// V2 and V3 format WALs remain readable by the WAL parser. When rewritten or consolidated
/// via `.bak` / `recover_from_bak_if_present`, old formats are transparently converted to V4 (`MFW4`).
#[tokio::test]
#[ignore = "ADR-N13 pending"]
async fn spec_n13_v2_v3_to_v4_transparent_migration_and_compatibility() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_v3.wal");

    // Write a V3 WAL file
    {
        let wal = Wal::open(&wal_path).await.expect("open wal");
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"compat_k".to_vec(),
            value: b"compat_v".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.expect("prepare");
        wal.append_batch(batch).await.expect("append");
    }

    // V3 WAL must be replayed successfully
    let wal = Wal::open(&wal_path).await.expect("open legacy v3 wal");
    let entries = wal.replay().await.expect("replay legacy v3");
    assert_eq!(entries.len(), 1);
}
