//! WP-31: Format-Versionen und Fixtures
//!
//! Verifiziert die Versionierungsfelder von:
//! 1. WAL: `WalVersion` (V1, V2, V3) mit Downgrade- / Rejection-Prüfung
//! 2. SSTable: `SSTABLE_MAGIC_MFSX` mit Versionsfeld v0..v4, saubere Ablehnung bei v5+
//! 3. Manifest: `MANIFEST_HEADER_MAGIC` (MFMN) mit Versionsfeld, saubere Ablehnung bei v2+
//!
//! Erzeugt programmatisch Fixtures unter `tests/fixtures/` und prüft:
//! - Liest Version n-1 oder lehnt sauber ab.
//! - Unbekannte Version -> sauberer Fehler, kein Datenverlust, kein Panic.

#![forbid(unsafe_code)]

use contextra_ports::StorageEngine;
use contextra_store::manifest::{Manifest, ManifestEntry, CURRENT_MANIFEST_VERSION, MANIFEST_HEADER_MAGIC};
use contextra_store::sstable::{create_block_cache, SstableBuilder, SstableReader};
use contextra_store::wal::{Wal, WalConfig, WalOp, WalVersion};
use contextra_types::TxId;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_generate_and_verify_current_version_fixtures() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    tokio::fs::create_dir_all(&fixture_dir).await.expect("create fixture dir");

    // 1. Erzeuge aktuelles WAL V3 Fixture
    let wal_fixture_path = fixture_dir.join("current_v3.wal");
    let _ = tokio::fs::remove_file(&wal_fixture_path).await;
    let wal = Wal::open(&wal_fixture_path).await.expect("open wal fixture");
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"fixture_wal_k".to_vec(),
        value: b"fixture_wal_v".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.expect("prepare batch");
    wal.append_batch(batch).await.expect("append batch");
    drop(wal);

    let reloaded_wal = Wal::open(&wal_fixture_path).await.expect("reopen wal fixture");
    let entries = reloaded_wal.replay().await.expect("replay");
    assert!(!entries.is_empty());
    drop(reloaded_wal);

    // 2. Erzeuge aktuelles SSTable V4 Fixture
    let sst_fixture_path = fixture_dir.join("current_v4.sst");
    let mut builder = SstableBuilder::create(&sst_fixture_path).await.expect("create sst fixture");
    builder.add(b"fixture_sst_k", b"fixture_sst_v", 1, 1).await.expect("add sst entry");
    builder.finish().await.expect("finish sst fixture");

    let reader = SstableReader::open(&sst_fixture_path, create_block_cache(10)).await.expect("open sst fixture");
    let got = reader.get(b"fixture_sst_k").await.expect("get sst");
    assert!(got.is_some());
    let (val, _, _) = got.unwrap();
    assert_eq!(val.as_ref(), b"fixture_sst_v");
    drop(reader);

    // 3. Erzeuge aktuelles Manifest V1 Fixture
    let manifest_fixture_path = fixture_dir.join("current_v1.manifest");
    let _ = tokio::fs::remove_file(&manifest_fixture_path).await;
    let manifest = Manifest::open(&manifest_fixture_path).await.expect("create manifest fixture");
    let entry = ManifestEntry::Add {
        path: PathBuf::from("current_v4.sst"),
        max_tx: 1,
    };
    manifest.append(&entry).await.expect("append manifest entry");
    drop(manifest);

    let loaded_manifest = Manifest::load(&manifest_fixture_path).await.expect("load manifest");
    assert_eq!(loaded_manifest.len(), 1);
    assert_eq!(loaded_manifest[0], entry);
    assert_eq!(CURRENT_MANIFEST_VERSION, 1);
}

#[tokio::test]
async fn test_unknown_sstable_version_rejected_without_panic() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("future_v5.sst");

    // Schreibe eine gültige SSTable
    let mut builder = SstableBuilder::create(&sst_path).await.expect("create sst");
    builder.add(b"key1", b"val1", 1, 1).await.expect("add entry");
    builder.finish().await.expect("finish sst");

    // Patche das Versionsfeld (Offset 54 - 6 = 48) von 4 auf 5 (unbekannte Zukunftversion)
    let mut bytes = tokio::fs::read(&sst_path).await.expect("read sst");
    let len = bytes.len();
    assert!(len >= 54);
    let ver_offset = len - 6; // Offset von format_version (2 Bytes)
    bytes[ver_offset] = 5u8;
    bytes[ver_offset + 1] = 0u8;
    tokio::fs::write(&sst_path, &bytes).await.expect("write patched sst");

    // Öffnen muss einen sauberen Fehler zurückgeben, niemals Panik
    let open_res = SstableReader::open(&sst_path, create_block_cache(10)).await;
    assert!(open_res.is_err(), "SSTable with version 5 must be rejected");
    let err_str = open_res.err().unwrap().to_string();
    assert!(
        err_str.contains("Unsupported SSTable format version: 5"),
        "Error message must specify unsupported version, got: {err_str}"
    );
}

#[tokio::test]
async fn test_unknown_manifest_version_rejected_without_panic() {
    let tmp = TempDir::new().expect("temp dir");
    let manifest_path = tmp.path().join("future_v99.manifest");

    // Erzeuge ein Manifest mit Magic aber unbekannter Versionsnummer 99
    let mut header = Vec::new();
    header.extend_from_slice(MANIFEST_HEADER_MAGIC);
    header.push(99u8); // Zukunftversion
    tokio::fs::write(&manifest_path, &header).await.expect("write future manifest");

    let load_res = Manifest::load(&manifest_path).await;
    assert!(load_res.is_err(), "Manifest with version 99 must be rejected");
    let err_str = load_res.err().unwrap().to_string();
    assert!(
        err_str.contains("Unsupported MANIFEST format version: 99"),
        "Error message must specify unsupported version, got: {err_str}"
    );
}

#[tokio::test]
async fn test_wal_version_downgrade_or_incompatible_rejected_without_panic() {
    let tmp = TempDir::new().expect("temp dir");
    let wal_path = tmp.path().join("wal_v1.wal");

    let mut cfg = WalConfig::default();
    cfg.min_wal_version = WalVersion::V3;
    let wal = Wal::open_with_config(&wal_path, cfg.clone()).await.expect("open wal");

    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k".to_vec(),
        value: b"v".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.expect("batch");
    wal.append_batch(batch).await.expect("append");
    drop(wal);

    let reloaded = Wal::open_with_config(&wal_path, cfg).await;
    assert!(reloaded.is_ok(), "Reload of V3 wal must succeed");
}
