use contextra_types::TxId;
use contextra_store::manifest::{Manifest, ManifestEntry};
use contextra_store::sstable::SstableBuilder;
use contextra_store::wal::{WalEntry, WalOp, WAL_V3_HEADER};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

pub async fn generate_all() {
    let dir = golden_dir();
    fs::create_dir_all(&dir).expect("create golden dir");

    // 1. Integrity Key & UUID
    let key_path = dir.join(".wal_integrity_key");
    let integrity_key = [0xAAu8; 32];
    fs::write(&key_path, &integrity_key).expect("write key");

    let uuid_path = dir.join("wal_v3.wal.uuid");
    let uuid_bytes = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10,
    ];
    fs::write(&uuid_path, &uuid_bytes).expect("write uuid");

    // 2. WAL V3 Golden File
    let wal_path = dir.join("wal_v3.wal");
    let mut wal_data = Vec::new();
    wal_data.extend_from_slice(&WAL_V3_HEADER);

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"golden_wal_k1".to_vec(),
        value: b"golden_wal_v1".to_vec(),
    };
    let entry1 = WalEntry::try_new(op1, 1, &integrity_key, [0u8; 32]).expect("entry1");
    wal_data.extend_from_slice(&entry1.to_bytes().expect("bytes1"));

    let op2 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"golden_wal_k2".to_vec(),
        value: b"golden_wal_v2".to_vec(),
    };
    let entry2 = WalEntry::try_new(op2, 2, &integrity_key, entry1.checksum).expect("entry2");
    wal_data.extend_from_slice(&entry2.to_bytes().expect("bytes2"));

    let op3 = WalOp::TxEnd {
        tx_id: TxId::new(1),
        committed: true,
    };
    let entry3 = WalEntry::try_new(op3, 3, &integrity_key, entry2.checksum).expect("entry3");
    wal_data.extend_from_slice(&entry3.to_bytes().expect("bytes3"));

    fs::write(&wal_path, &wal_data).expect("write wal");

    // 3. SSTable MFSX Golden File
    let sst_path = dir.join("sstable_mfsx.sst");
    if sst_path.exists() {
        let _ = fs::remove_file(&sst_path);
    }
    let mut builder = SstableBuilder::create_with_version(&sst_path, 4)
        .await
        .expect("sst builder");
    for i in 1..=5 {
        let k = format!("golden_sst_k{:02}", i);
        let v = format!("golden_sst_v{:02}", i);
        builder
            .add(k.as_bytes(), v.as_bytes(), i as u64, i as u64)
            .await
            .expect("sst add");
    }
    builder.finish().await.expect("sst finish");

    // 4. Manifest MFMN Golden File
    let manifest_path = dir.join("manifest_mfmn.manifest");
    if manifest_path.exists() {
        let _ = fs::remove_file(&manifest_path);
    }
    let manifest = Manifest::open(&manifest_path).await.expect("manifest open");
    manifest
        .append(&ManifestEntry::FormatVersionRecord {
            file_kind: 0,
            max_version: 4,
            migrated_from: 3,
        })
        .await
        .expect("manifest format record");
    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("sstable_mfsx.sst"),
            max_tx: 100,
        })
        .await
        .expect("manifest add");
    manifest
        .append(&ManifestEntry::WalCheckpoint { hmac: [0x55u8; 32] })
        .await
        .expect("manifest wal checkpoint");
    manifest
        .append(&ManifestEntry::RollbackComplete { target_tx: 99 })
        .await
        .expect("manifest rollback complete");

    // 5. MANIFEST.txt with SHA-256 hashes
    let files_to_hash = vec![
        "wal_v3.wal",
        "wal_v3.wal.uuid",
        "sstable_mfsx.sst",
        "manifest_mfmn.manifest",
    ];

    let mut manifest_txt = String::new();
    for fname in files_to_hash {
        let fpath = dir.join(fname);
        let bytes = fs::read(&fpath).expect("read file for hash");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());
        manifest_txt.push_str(&format!("{}  {}\n", hash, fname));
    }

    fs::write(dir.join("MANIFEST.txt"), manifest_txt).expect("write MANIFEST.txt");
}

pub fn verify_file_hash(dir: &Path, filename: &str) -> bool {
    let manifest_path = dir.join("MANIFEST.txt");
    if !manifest_path.exists() {
        return false;
    }
    let content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 && parts[1] == filename {
            let expected_hash = parts[0];
            let target_path = dir.join(filename);
            let bytes = match fs::read(&target_path) {
                Ok(b) => b,
                Err(_) => return false,
            };
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual_hash = format!("{:x}", hasher.finalize());
            return expected_hash == actual_hash;
        }
    }
    false
}
