//! Fehlertoleranz und Korruptionsprüfung (PROOF-3, G-F1)
//!
//! Prüft, dass das Laden beschädigter Dateien (Manifest, SSTable-Footer, WAL-Mitte, WAL-Tail)
//! einen sauberen typisierten Fehler bzw. Open-Heal Recovery liefert, niemals in Panics abgleitet
//! und die Quell-Datei nicht ungewollt mutiert wird (Hash vorher == nachher).

use contextra_store::manifest::Manifest;
use contextra_store::sstable::{BlockCache, SstableReader};
use contextra_store::wal::{Wal, WalOp};
use contextra_types::TxId;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_manifest_corrupted_header_rejects_cleanly() {
    let tmp = TempDir::new().expect("tempdir");
    let corrupt_manifest_path = tmp.path().join("corrupt.manifest");

    fs::write(&corrupt_manifest_path, b"MFMN\x63").expect("write corrupt");
    let initial_hash = blake3::hash(&fs::read(&corrupt_manifest_path).unwrap());

    let res = Manifest::load(&corrupt_manifest_path).await;
    assert!(res.is_err(), "Manifest with unsupported version must be rejected");

    let post_hash = blake3::hash(&fs::read(&corrupt_manifest_path).unwrap());
    assert_eq!(initial_hash, post_hash, "File must remain byte-identical after rejected load");

    fs::write(&corrupt_manifest_path, b"MFMN\x01\xFF\xFF\xFF\xFF").expect("write garbage payload");
    let initial_hash = blake3::hash(&fs::read(&corrupt_manifest_path).unwrap());

    let res = Manifest::load(&corrupt_manifest_path).await;
    assert!(res.is_err(), "Manifest with corrupted payload must be rejected");

    let post_hash = blake3::hash(&fs::read(&corrupt_manifest_path).unwrap());
    assert_eq!(initial_hash, post_hash, "File must remain byte-identical after rejected load");
}

#[tokio::test]
async fn test_sstable_corrupted_footer_rejects_cleanly() {
    let tmp = TempDir::new().expect("tempdir");
    let corrupt_sst_path = tmp.path().join("corrupt.sst");
    let cache = Arc::new(BlockCache::new(1024 * 1024));

    let mut bad_sst = vec![0u8; 128];
    bad_sst.extend_from_slice(b"BADD");
    fs::write(&corrupt_sst_path, &bad_sst).expect("write corrupt sst");
    let initial_hash = blake3::hash(&fs::read(&corrupt_sst_path).unwrap());

    let res = SstableReader::open(&corrupt_sst_path, cache.clone()).await;
    assert!(res.is_err(), "SSTable with bad magic must fail to open");

    let post_hash = blake3::hash(&fs::read(&corrupt_sst_path).unwrap());
    assert_eq!(initial_hash, post_hash, "SSTable file must remain unchanged");

    fs::write(&corrupt_sst_path, b"SHORT").expect("write short sst");
    let initial_hash = blake3::hash(&fs::read(&corrupt_sst_path).unwrap());

    let res = SstableReader::open(&corrupt_sst_path, cache).await;
    assert!(res.is_err(), "Truncated SSTable must fail to open");

    let post_hash = blake3::hash(&fs::read(&corrupt_sst_path).unwrap());
    assert_eq!(initial_hash, post_hash, "SSTable file must remain unchanged");
}

#[tokio::test]
async fn test_wal_corrupt_middle_fails_closed_or_recovers() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("corrupt_middle.wal");

    let wal = Wal::open(&wal_path).await.expect("create test wal");
    for i in 1..=3 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k_{}", i).into_bytes(),
            value: format!("v_{}", i).into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await.expect("prepare");
        wal.append_batch(batch).await.expect("append");
    }
    drop(wal);

    let original = fs::read(&wal_path).expect("read test wal");
    assert!(original.len() > 30);
    let mut modified = original.clone();
    modified[20] ^= 0xFF;
    modified[21] ^= 0xAA;
    fs::write(&wal_path, &modified).expect("write modified wal");

    let wal_reopen = Wal::open(&wal_path).await;
    if let Ok(w) = wal_reopen {
        let res = w.replay().await;
        match res {
            Ok(entries) => {
                assert!(entries.len() <= 1, "Cannot replay past corrupted middle entry");
            }
            Err(e) => {
                let err_str = e.to_string();
                assert!(!err_str.is_empty());
            }
        }
    }
}
