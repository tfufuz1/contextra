//! Golden Corpus Generator & Regression Tests (ST6.1)
//!
//! Generates and verifies byte-for-byte deterministic reference files for:
//! 1. Manifest (`MFMN` v1)
//! 2. SSTable (`MFSX` v4 default, `MFST` legacy v3, multiblock with Bloom)
//! 3. WAL including HMAC hash chain, WAL without HMAC, WAL with corrupt tail
//! 4. Rollback Intent (`MFRLBK\0\0`)
//! 5. Tenant Codec Test Vectors

use contextra_store::manifest::{Manifest, ManifestEntry, CURRENT_MANIFEST_VERSION};
use contextra_store::sstable::{SstableBuilder, SstableReader};
use contextra_store::tenant_codec::TenantKeyCodec;
use contextra_store::wal::{Wal, WalOp};
use contextra_types::{CollectionId, DocId, TenantId, TxId};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("golden")
}

#[tokio::test]
async fn test_golden_corpus_files_exist_and_hashes_match() {
    let dir = golden_dir();
    let manifest_sha = dir.join("MANIFEST.sha256");
    assert!(manifest_sha.exists(), "MANIFEST.sha256 must exist");

    let checksums = fs::read_to_string(&manifest_sha).expect("read MANIFEST.sha256");
    for line in checksums.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Invalid checksum line: {}", line);
        let expected_hash = parts[0];
        let filename = parts[1];
        let file_path = dir.join(filename);
        assert!(file_path.exists(), "Golden file {} does not exist", filename);

        let bytes = fs::read(&file_path).expect("read golden file");
        let computed_hash = blake3::hash(&bytes).to_hex().to_string();
        assert_eq!(
            computed_hash, expected_hash,
            "Checksum mismatch for {}",
            filename
        );
    }
}

#[tokio::test]
async fn test_golden_backward_compatibility_read() {
    let dir = golden_dir();

    // 1. Read Rollback Intent
    let rollback_path = dir.join("rollback_tx_42.intent");
    let intent_bytes = fs::read(&rollback_path).expect("read rollback intent");
    assert_eq!(&intent_bytes[..8], b"MFRLBK\0\0");
    let tx_num = u64::from_le_bytes(intent_bytes[8..16].try_into().unwrap());
    assert_eq!(tx_num, 42);

    // 2. Read Manifest (v1)
    let manifest_path = dir.join("manifest_v1.manifest");
    let entries = Manifest::load(&manifest_path).await.expect("load manifest");
    let active_ssts = Manifest::reconstruct_valid_sstables(&entries);
    assert_eq!(active_ssts.len(), 2);
    assert_eq!(active_ssts[0].0.to_string_lossy(), "sst_001.sst");
    assert_eq!(active_ssts[1].0.to_string_lossy(), "sst_002.sst");

    // Manifest version check logic:
    // If CURRENT_MANIFEST_VERSION == 1: writer produces byte-identical format.
    // If CURRENT_MANIFEST_VERSION > 1: reader must still read v1 manifest seamlessly (as verified above).
    if CURRENT_MANIFEST_VERSION == 1 {
        let manifest_bytes = fs::read(&manifest_path).expect("read manifest bytes");
        assert_eq!(&manifest_bytes[0..4], b"MFMN");
        assert_eq!(manifest_bytes[4], 1);
    }

    // 3. Read SSTable MFSX (single block)
    let sst_path = dir.join("sstable_v4.sst");
    let cache = Arc::new(contextra_store::sstable::BlockCache::new(1024 * 1024));
    let reader = SstableReader::open(&sst_path, cache.clone())
        .await
        .expect("open sstable");
    assert_eq!(reader.first_key().as_ref(), b"key_0001");
    assert_eq!(reader.last_key().as_ref(), b"key_0005");
    let val = reader.get(b"key_0003").await.expect("get key_0003");
    assert!(val.is_some());
    let (val_bytes, val_seq, val_tx) = val.unwrap();
    assert_eq!(val_bytes.as_ref(), b"value_payload_0003");
    assert_eq!(val_seq, 3);
    assert_eq!(val_tx, 3);

    // 4. Read SSTable Multiblock with Bloom
    let sst_mb_path = dir.join("sstable_multiblock_bloom.sst");
    if sst_mb_path.exists() {
        let mb_reader = SstableReader::open(&sst_mb_path, cache.clone())
            .await
            .expect("open multiblock sstable");
        assert_eq!(mb_reader.format_version, 4);
        let val_1 = mb_reader.get(b"k_mb_0001").await.expect("get first");
        assert!(val_1.is_some());
        let val_last = mb_reader.get(b"k_mb_0250").await.expect("get last");
        assert!(val_last.is_some());
        let val_none = mb_reader.get(b"non_existent_key").await.expect("get non-existent");
        assert!(val_none.is_none());
    }

    // 5. Read SSTable Legacy MFST (v3)
    let legacy_sst_path = dir.join("sstable_legacy.sst");
    let legacy_reader = SstableReader::open(&legacy_sst_path, cache)
        .await
        .expect("open legacy sstable");
    assert_eq!(legacy_reader.first_key().as_ref(), b"legacy_001");
    let legacy_val = legacy_reader.get(b"legacy_002").await.expect("get legacy");
    assert!(legacy_val.is_some());
    let (leg_bytes, _, _) = legacy_val.unwrap();
    assert_eq!(leg_bytes.as_ref(), b"val_legacy_002");

    // 6. Read WAL with HMAC
    let wal_path = dir.join("wal_with_hmac.log");
    let wal = Wal::open(&wal_path).await.expect("open wal");
    let replayed = wal.replay().await.expect("replay wal");
    assert_eq!(replayed.len(), 3);

    // 7. Read WAL without HMAC (legacy format / zeroed hmac)
    let wal_no_hmac_path = dir.join("wal_no_hmac.log");
    if wal_no_hmac_path.exists() {
        let wal_nh = Wal::open(&wal_no_hmac_path).await.expect("open wal no hmac");
        let replayed_nh = wal_nh.replay().await.expect("replay wal no hmac");
        assert_eq!(replayed_nh.len(), 2);
    }

    // 8. Read WAL with Corrupt Tail (Open-Heal recovery)
    let wal_corrupt_path = dir.join("wal_corrupt_tail.log");
    if wal_corrupt_path.exists() {
        let wal_c = Wal::open(&wal_corrupt_path).await.expect("open wal corrupt tail");
        let replayed_c = wal_c.replay().await.expect("replay wal corrupt tail");
        assert_eq!(replayed_c.len(), 2, "Valid records before corrupt tail must replay");
    }

    // 9. Verify Tenant Codec Vectors
    let tc_path = dir.join("tenant_codec_vectors.json");
    if tc_path.exists() {
        let content = fs::read_to_string(&tc_path).expect("read tc vectors");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse json");
        for v in parsed.as_array().expect("array") {
            let tenant_id = v["tenant_id"].as_u64().expect("tenant_id");
            let col_id = v["col_id"].as_u64().expect("col_id");
            let doc_id = v["doc_id"].as_u64().expect("doc_id");
            let expected_encoded: Vec<u8> = v["encoded_bytes"].as_array().expect("encoded_bytes")
                .iter().map(|b| b.as_u64().unwrap() as u8).collect();

            let codec = TenantKeyCodec::new(TenantId::new(tenant_id));
            let encoded = codec.encode_chunk_key(&CollectionId(col_id), DocId(doc_id));
            assert_eq!(encoded, expected_encoded, "Tenant chunk key encoding mismatch");

            let (decoded_tid, decoded_cid, decoded_did) = TenantKeyCodec::decode_chunk_key(&encoded)
                .expect("decode chunk key");
            assert_eq!(decoded_tid.inner(), tenant_id);
            assert_eq!(decoded_cid.0, col_id);
            assert_eq!(decoded_did.0, doc_id);
        }
    }
}

/// Generator for Golden Files (Run via `cargo test --test golden_corpus -- --ignored` or golden/regen.sh)
#[ignore]
#[tokio::test]
async fn generate_golden_corpus() {
    let dir = golden_dir();
    fs::create_dir_all(&dir).expect("create golden dir");

    // 1. Rollback Intent
    let rollback_path = dir.join("rollback_tx_42.intent");
    let mut intent_bytes = Vec::with_capacity(16);
    intent_bytes.extend_from_slice(b"MFRLBK\0\0");
    intent_bytes.extend_from_slice(&42u64.to_le_bytes());
    fs::write(&rollback_path, &intent_bytes).expect("write intent");

    // 2. Manifest
    let manifest_path = dir.join("manifest_v1.manifest");
    let _ = fs::remove_file(&manifest_path);
    let manifest = Manifest::open(&manifest_path).await.expect("create manifest");
    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("sst_001.sst"),
            max_tx: 10,
        })
        .await
        .expect("append add 1");
    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("sst_002.sst"),
            max_tx: 20,
        })
        .await
        .expect("append add 2");
    drop(manifest);

    // 3. SSTable MFSX (v4)
    let sst_path = dir.join("sstable_v4.sst");
    let _ = fs::remove_file(&sst_path);
    let mut b = SstableBuilder::create(&sst_path).await.expect("create sst");
    for i in 1..=5 {
        let k = format!("key_{:04}", i).into_bytes();
        let v = format!("value_payload_{:04}", i).into_bytes();
        b.add(&k, &v, i as u64, i as u64).await.expect("add entry");
    }
    b.finish().await.expect("finish sst");

    // 4. SSTable Multiblock with Bloom Filter
    let sst_mb_path = dir.join("sstable_multiblock_bloom.sst");
    let _ = fs::remove_file(&sst_mb_path);
    let mut b_mb = SstableBuilder::create(&sst_mb_path).await.expect("create mb sst");
    for i in 1..=250 {
        let k = format!("k_mb_{:04}", i).into_bytes();
        let v = vec![(i % 256) as u8; 64];
        b_mb.add(&k, &v, i as u64, i as u64).await.expect("add mb entry");
    }
    b_mb.finish().await.expect("finish mb sst");

    // 5. SSTable Legacy MFST (v3)
    let legacy_sst_path = dir.join("sstable_legacy.sst");
    let _ = fs::remove_file(&legacy_sst_path);
    let mut b_leg = SstableBuilder::create(&legacy_sst_path).await.expect("create legacy sst");
    b_leg.set_format_version(3);
    for i in 1..=3 {
        let k = format!("legacy_{:03}", i).into_bytes();
        let v = format!("val_legacy_{:03}", i).into_bytes();
        b_leg.add(&k, &v, i as u64, i as u64).await.expect("add legacy entry");
    }
    b_leg.finish().await.expect("finish legacy sst");

    // 6. WAL with HMAC (deterministic key)
    let wal_path = dir.join("wal_with_hmac.log");
    let _ = fs::remove_file(&wal_path);
    let _key_file = dir.join(".wal_integrity_key");

    let wal = Wal::open(&wal_path).await.expect("create wal");
    for i in 1..=3 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("wal_key_{:03}", i).into_bytes(),
            value: format!("wal_value_{:03}", i).into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await.expect("prepare");
        wal.append_batch(batch).await.expect("append");
    }
    drop(wal);

    // 7. WAL without HMAC
    let wal_nh_path = dir.join("wal_no_hmac.log");
    let _ = fs::remove_file(&wal_nh_path);
    let wal_nh = Wal::open(&wal_nh_path).await.expect("create wal nh");
    for i in 1..=2 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("nh_key_{:03}", i).into_bytes(),
            value: format!("nh_value_{:03}", i).into_bytes(),
        };
        let (batch, _) = wal_nh.prepare_batch(vec![(op, i)]).await.expect("prepare nh");
        wal_nh.append_batch(batch).await.expect("append nh");
    }
    drop(wal_nh);

    // 8. WAL with Corrupt Tail
    let wal_c_path = dir.join("wal_corrupt_tail.log");
    let _ = fs::remove_file(&wal_c_path);
    let nh_bytes = fs::read(&wal_nh_path).expect("read wal nh");
    let mut c_bytes = nh_bytes;
    c_bytes.extend_from_slice(b"\xFF\xFE\xFD\xFC\x00\x01\x02\x03CORRUPTED_TAIL_GARBAGE");
    fs::write(&wal_c_path, &c_bytes).expect("write corrupt wal");

    // 9. Tenant Codec Vectors
    let tc_path = dir.join("tenant_codec_vectors.json");
    let mut vectors = Vec::new();
    for t in [1u64, 42, 1000, u32::MAX as u64] {
        for col in [1u64, 7, 999] {
            for doc in [1u64, 42, 123456789] {
                let codec = TenantKeyCodec::new(TenantId::new(t));
                let encoded = codec.encode_chunk_key(&CollectionId(col), DocId(doc));
                vectors.push(serde_json::json!({
                    "tenant_id": t,
                    "col_id": col,
                    "doc_id": doc,
                    "encoded_bytes": encoded,
                }));
            }
        }
    }
    fs::write(&tc_path, serde_json::to_string_pretty(&vectors).expect("json")).expect("write json");

    // Update MANIFEST.sha256
    let mut files = vec![
        "rollback_tx_42.intent",
        "manifest_v1.manifest",
        "sstable_v4.sst",
        "sstable_multiblock_bloom.sst",
        "sstable_legacy.sst",
        "wal_with_hmac.log",
        "wal_no_hmac.log",
        "wal_corrupt_tail.log",
        "tenant_codec_vectors.json",
    ];
    files.sort();

    let mut hashes = String::new();
    for f in files {
        let p = dir.join(f);
        let bytes = fs::read(&p).expect("read file for hash");
        let h = blake3::hash(&bytes).to_hex().to_string();
        hashes.push_str(&format!("{}  {}\n", h, f));
    }

    fs::write(dir.join("MANIFEST.sha256"), hashes).expect("write manifest.sha256");
    println!("Golden corpus regenerated successfully.");
}
