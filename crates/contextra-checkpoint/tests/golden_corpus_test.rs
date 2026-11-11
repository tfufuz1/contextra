//! # Golden Corpus Test für contextra-checkpoint
//!
//! Verifiziert INV-1: Persistierte Bytes (Meta, Manifest, Orphan-Zustand, HardlinkClone-Struktur)
//! bleiben exakt stabil und abwärtskompatibel.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use contextra_checkpoint::{
    CheckpointManifest, CheckpointMeta, HardlinkCloneResult, OrphanState, PinnedSeqNoOrphan,
    StateCheckpoint,
};
use contextra_types::TxId;
use std::fs;
use std::path::PathBuf;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

#[test]
fn test_golden_meta_byte_exact() {
    let path = golden_dir().join("meta_golden.json");
    let content = fs::read_to_string(&path).expect("read meta_golden.json");

    let deserialized: CheckpointMeta =
        serde_json::from_str(&content).expect("deserialize CheckpointMeta");

    assert_eq!(deserialized.name, "golden_snapshot_v1");
    assert_eq!(deserialized.collection_id, "coll_golden_001");
    assert_eq!(deserialized.seq_no, 42000);
    assert_eq!(deserialized.tx_id, TxId::new(1337));
    assert_eq!(deserialized.created_at, 1700000000000);

    let reserialized =
        serde_json::to_string_pretty(&deserialized).expect("serialize CheckpointMeta");
    assert_eq!(
        reserialized.trim(),
        content.trim(),
        "Reserialization must match golden byte-for-byte"
    );
}

#[test]
fn test_golden_manifest_byte_exact() {
    let path = golden_dir().join("manifest_golden.json");
    let content = fs::read_to_string(&path).expect("read manifest_golden.json");

    let deserialized: CheckpointManifest =
        serde_json::from_str(&content).expect("deserialize CheckpointManifest");

    assert_eq!(deserialized.meta.name, "golden_snapshot_v1");
    assert_eq!(deserialized.meta.seq_no, 42000);
    assert_eq!(deserialized.meta.tx_id, TxId::new(1337));
    assert_eq!(deserialized.components, vec!["lsm", "vector", "meta"]);
    assert!(deserialized.verify().is_ok());

    let reserialized =
        serde_json::to_string_pretty(&deserialized).expect("serialize CheckpointManifest");
    assert_eq!(
        reserialized.trim(),
        content.trim(),
        "Reserialization of manifest must match golden byte-for-byte"
    );
}

#[test]
fn test_golden_orphan_state_byte_exact() {
    let path = golden_dir().join("orphan_state_golden.json");
    let content = fs::read_to_string(&path).expect("read orphan_state_golden.json");

    let deserialized: OrphanState =
        serde_json::from_str(&content).expect("deserialize OrphanState");

    assert_eq!(deserialized.pinned_seq_nos.len(), 1);
    assert_eq!(deserialized.pinned_seq_nos[0].seq_no, 555);
    assert_eq!(deserialized.checkpoints.len(), 1);
    assert_eq!(deserialized.checkpoints[0].tx_id, TxId::new(888));

    let reserialized = serde_json::to_string_pretty(&deserialized).expect("serialize OrphanState");
    assert_eq!(
        reserialized.trim(),
        content.trim(),
        "Reserialization of orphan state must match golden byte-for-byte"
    );
}

#[test]
fn test_golden_hardlink_clone_result_byte_exact() {
    let path = golden_dir().join("hardlink_clone_golden.json");
    let content = fs::read_to_string(&path).expect("read hardlink_clone_golden.json");

    let deserialized: HardlinkCloneResult =
        serde_json::from_str(&content).expect("deserialize HardlinkCloneResult");

    assert_eq!(deserialized.source_seq_no, 42000);
    assert_eq!(
        deserialized.linked_files,
        vec![
            PathBuf::from("/tmp/ckpt_dst/000001.sst"),
            PathBuf::from("/tmp/ckpt_dst/000002.sst"),
        ]
    );
    assert_eq!(deserialized.wal_tail_offset, Some(1024));

    let reserialized =
        serde_json::to_string_pretty(&deserialized).expect("serialize HardlinkCloneResult");
    assert_eq!(
        reserialized.trim(),
        content.trim(),
        "Reserialization of hardlink clone result must match golden byte-for-byte"
    );
}

#[test]
fn test_golden_manifest_sha256_integrity() {
    let manifest_path = golden_dir().join("MANIFEST.sha256");
    let lines = fs::read_to_string(&manifest_path).expect("read MANIFEST.sha256");

    for line in lines.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let expected_hash = parts.next().expect("hash part");
        let filename = parts.next().expect("filename part");

        let file_path = golden_dir().join(filename);
        let output = std::process::Command::new("sha256sum")
            .arg(&file_path)
            .output()
            .expect("run sha256sum");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let actual_hash = stdout.split_whitespace().next().expect("sha256 output");

        assert_eq!(
            actual_hash, expected_hash,
            "SHA256 mismatch for golden file {}",
            filename
        );
    }
}

#[test]
#[ignore]
fn regen_golden_corpus() {
    let dir = golden_dir();
    fs::create_dir_all(&dir).expect("create golden dir");

    // 1. Meta
    let meta = CheckpointMeta {
        name: "golden_snapshot_v1".to_string(),
        collection_id: "coll_golden_001".to_string(),
        seq_no: 42000,
        tx_id: TxId::new(1337),
        metadata: serde_json::json!({
            "env": "golden_production",
            "retention_days": 30
        }),
        created_at: 1700000000000,
        id_width_bits: contextra_checkpoint::CURRENT_ID_WIDTH_BITS,
    };
    let meta_json = serde_json::to_string_pretty(&meta).expect("serialize meta");
    fs::write(dir.join("meta_golden.json"), format!("{}\n", meta_json)).expect("write meta");

    // 2. Manifest
    let manifest = CheckpointManifest::new(
        meta,
        vec!["lsm".to_string(), "vector".to_string(), "meta".to_string()],
    )
    .expect("create manifest");
    let manifest_json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    fs::write(
        dir.join("manifest_golden.json"),
        format!("{}\n", manifest_json),
    )
    .expect("write manifest");

    // 3. Orphan State
    let orphan_state = OrphanState {
        checkpoints: vec![StateCheckpoint {
            tx_id: TxId::new(888),
            timestamp_ms: 1700000000200,
            namespace: Some("test_ns".to_string()),
        }],
        pinned_seq_nos: vec![PinnedSeqNoOrphan {
            seq_no: 555,
            timestamp_ms: 1700000000100,
        }],
        persist_path: PathBuf::from("/tmp/orphans.json"),
    };
    let orphan_json = serde_json::to_string_pretty(&orphan_state).expect("serialize orphan");
    fs::write(
        dir.join("orphan_state_golden.json"),
        format!("{}\n", orphan_json),
    )
    .expect("write orphan");

    // 4. HardlinkCloneResult
    let clone_result = HardlinkCloneResult {
        source_seq_no: 42000,
        linked_files: vec![
            PathBuf::from("/tmp/ckpt_dst/000001.sst"),
            PathBuf::from("/tmp/ckpt_dst/000002.sst"),
        ],
        wal_tail_offset: Some(1024),
    };
    let clone_json = serde_json::to_string_pretty(&clone_result).expect("serialize clone");
    fs::write(
        dir.join("hardlink_clone_golden.json"),
        format!("{}\n", clone_json),
    )
    .expect("write clone");
}
