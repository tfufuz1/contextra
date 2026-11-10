// ANCHOR[contextra-mvcc:TEST-GOLDEN-CORPUS]
//! # Golden- und Differenz-Korpus für `contextra-mvcc` (MV1.2, SPEC-v2 §5.1)
#![allow(unexpected_cfgs)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_mvcc::seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::types::{DocId, TxId, TOMBSTONE_BIT};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct GoldenSeqEntry {
    doc_id: u64,
    insert_seq: u64,
    delete_seq: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum GoldenIndexOp {
    Insert { doc_id: u64, data: Vec<u8> },
    Delete { doc_id: u64, data: Option<Vec<u8>> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct GoldenData {
    seq_log_entries: Vec<GoldenSeqEntry>,
    visibility_matrix: Vec<(u64, u64, Option<u64>, bool)>,
    tombstone_cases: Vec<(u64, bool, u64)>,
    ssi_outcomes: Vec<(u64, bool)>,
    serde_index_ops: Vec<GoldenIndexOp>,
}

fn build_golden_data() -> GoldenData {
    let mut seq_log = SequenceLog::new();
    let d1 = DocId::from(101u64);
    let d2 = DocId::from(102u64);
    let d3 = DocId::from(103u64);

    seq_log.record_insert(d1, 10);
    seq_log.record_insert(d2, 20);
    seq_log.record_delete(d1, 30);
    seq_log.record_insert(d3, 40);

    let changes = seq_log.changes_since(0);
    let mut entries = Vec::new();
    for c in changes {
        match c {
            SeqLogChange::Insert { doc_id, seq } => {
                entries.push(GoldenSeqEntry {
                    doc_id: doc_id.inner(),
                    insert_seq: seq,
                    delete_seq: None,
                });
            }
            SeqLogChange::Delete { doc_id, seq, .. } => {
                entries.push(GoldenSeqEntry {
                    doc_id: doc_id.inner(),
                    insert_seq: 0,
                    delete_seq: Some(seq),
                });
            }
        }
    }

    let test_triples = [
        (15, 10, None, true),
        (5, 10, None, false),
        (10, 10, None, true),
        (25, 10, Some(30), true),
        (30, 10, Some(30), false),
        (35, 10, Some(30), false),
    ];
    let mut vis_matrix = Vec::new();
    for (snap, born, died, expected) in test_triples {
        let entry = SeqLogEntry {
            doc_id: DocId::from(1u64),
            insert_seq: born,
            delete_seq: died,
        };
        let actual = entry.is_visible(snap);
        assert_eq!(actual, expected);
        vis_matrix.push((snap, born, died, actual));
    }

    let raw_cases = [
        0u64,
        1,
        42,
        100,
        1_000_000,
        1 | TOMBSTONE_BIT,
        42 | TOMBSTONE_BIT,
    ];
    let mut tomb_cases = Vec::new();
    for &seq in &raw_cases {
        let is_tomb = (seq & TOMBSTONE_BIT) != 0;
        let masked = seq & !TOMBSTONE_BIT;
        tomb_cases.push((seq, is_tomb, masked));
    }

    let ssi = SequenceLogSsiValidator::new();
    let tx2 = TxId::new(2);

    let mut rs1 = ReadSet::new();
    rs1.record_read(b"key:A", 100);
    rs1.record_read(b"key:B", 100);

    ssi.record_commit_key(b"key:A", 110);

    let mut outcomes = Vec::new();
    let res_conflict = ssi.validate(tx2, &rs1);
    outcomes.push((tx2.inner(), res_conflict.is_err()));

    let tx3 = TxId::new(3);
    let mut rs3 = ReadSet::new();
    rs3.record_read(b"key:A", 120);
    let res_no_conflict = ssi.validate(tx3, &rs3);
    outcomes.push((tx3.inner(), res_no_conflict.is_err()));

    let ops = vec![
        GoldenIndexOp::Insert {
            doc_id: 10,
            data: b"payload_alpha".to_vec(),
        },
        GoldenIndexOp::Delete {
            doc_id: 20,
            data: None,
        },
        GoldenIndexOp::Delete {
            doc_id: 30,
            data: Some(b"tombstone_meta".to_vec()),
        },
    ];

    GoldenData {
        seq_log_entries: entries,
        visibility_matrix: vis_matrix,
        tombstone_cases: tomb_cases,
        ssi_outcomes: outcomes,
        serde_index_ops: ops,
    }
}

#[test]
fn test_golden_corpus_matches_reference() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let json_path = golden_dir.join("mvcc_golden.json");
    let bin_path = golden_dir.join("mvcc_golden.bin");

    let current_data = build_golden_data();

    if !json_path.exists() || !bin_path.exists() {
        panic!("Golden files missing! Run tests/golden/regen.sh first.");
    }

    let json_bytes = fs::read(&json_path).expect("read json");
    let bin_bytes = fs::read(&bin_path).expect("read bin");

    let from_json: GoldenData = serde_json::from_slice(&json_bytes).expect("deserialize json");
    let from_bin: GoldenData = bincode::deserialize(&bin_bytes).expect("deserialize bincode");

    assert_eq!(
        current_data, from_json,
        "Current data must match golden JSON"
    );
    assert_eq!(
        current_data, from_bin,
        "Current data must match golden bincode"
    );
}

#[test]
fn test_golden_bitflip_and_truncation_resilience() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let bin_path = golden_dir.join("mvcc_golden.bin");
    let bin_bytes = fs::read(bin_path).expect("read bin");

    for cut in [1, 5, 10, bin_bytes.len() / 2, bin_bytes.len() - 1] {
        let truncated = &bin_bytes[..cut];
        let res: Result<GoldenData, _> = bincode::deserialize(truncated);
        assert!(
            res.is_err(),
            "Truncated payload at {cut} bytes must fail with Err, not panic"
        );
    }

    for &pos in &[0, 4, 8, 16, 32, bin_bytes.len() - 1] {
        let mut corrupted = bin_bytes.clone();
        corrupted[pos] ^= 0xFF;
        let res: Result<GoldenData, _> = bincode::deserialize(&corrupted);
        let _ = res;
    }
}

#[test]
#[ignore]
fn generate_golden_mvcc() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    fs::create_dir_all(&golden_dir).expect("create dir");

    let data = build_golden_data();
    let json_str = serde_json::to_string_pretty(&data).expect("serialize json");
    let bin_bytes = bincode::serialize(&data).expect("serialize bincode");

    fs::write(golden_dir.join("mvcc_golden.json"), json_str).expect("write json");
    fs::write(golden_dir.join("mvcc_golden.bin"), bin_bytes).expect("write bin");
    println!("Golden files generated successfully in {:?}", golden_dir);
}
