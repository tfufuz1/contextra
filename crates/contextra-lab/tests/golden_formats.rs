#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_borrows_for_generic_args
)]
//! Golden-Tests für alle Byte-Formate (Anhang F.1 bis F.10).
//!
//! SPEC: T-FMT-1, T-PROOF-1

use contextra_lab::format::append_logs::AppendLogFile;
use contextra_lab::format::file_header::{
    FileHeader, FileKind, FORMAT_VERSION_V1, STANDARD_ALG_IDS,
};
use contextra_lab::format::header_ab::HeaderAb;
use contextra_lab::format::record_envelope::RecordEnvelopeV2;
use contextra_lab::format::report::{ClassStatus, ErasureReport};
use contextra_lab::format::sstable::{SsTableEntry, SsTableFile, SSTABLE_FOOTER_MAGIC};
use contextra_lab::format::text_segment::{TermEntry, TextSegmentFile};
use contextra_lab::format::vault::{VaultFile, VaultSlotStatus, VaultSlotType};
use contextra_lab::format::vector_snapshot::{VectorMetric, VectorSnapshotFile};
use contextra_lab::format::wal_manifest::{ManifestLog, WalSegment};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

/// SPEC: T-FMT-1 (F.1 Dateikopf)
#[test]
fn t_fmt_1_f1_dateikopf_golden() {
    let file_key = [0x42u8; 32];
    let file_id = [0x07u8; 16];
    let created_seq = 1001u64;

    let header_bytes =
        FileHeader::create_bytes(FileKind::WalSegment, file_id, created_seq, &file_key);
    let parsed = FileHeader::parse(&header_bytes).expect("Parse Dateikopf");

    assert_eq!(parsed.format_version, FORMAT_VERSION_V1);
    assert_eq!(parsed.kind, FileKind::WalSegment);
    assert_eq!(parsed.alg_ids, STANDARD_ALG_IDS);
    assert_eq!(parsed.file_id, file_id);
    assert_eq!(parsed.created_seq, created_seq);
    assert_eq!(parsed.key_mode, 0);

    // Krypto-Verifikation
    parsed
        .verify_tag(&header_bytes, &file_key)
        .expect("AEAD Tag Verifikation");

    // Negativtest: Manipulation an AAD muss fehlschlagen
    let mut corrupted = header_bytes;
    corrupted[10] = FileKind::SsTable as u8;
    assert!(
        FileHeader::parse(&corrupted).is_err() || parsed.verify_tag(&corrupted, &file_key).is_err()
    );
}

/// SPEC: T-FMT-1 (F.2 HEADER.a/b)
#[test]
fn t_fmt_1_f2_header_ab_golden() {
    let mut buf = [0u8; 1024];
    buf[0..8].copy_from_slice(b"CTXHDR\x00\x01");
    buf[8..10].copy_from_slice(&1u16.to_be_bytes());
    buf[10] = 0x20;
    buf[11..15].copy_from_slice(&[1, 0, 1, 1]);
    buf[15..31].copy_from_slice(&[0xaa; 16]);
    buf[31..39].copy_from_slice(&42u64.to_be_bytes());
    buf[39] = 1; // 1 Slot
    buf[40..72].copy_from_slice(&[0x11; 32]); // report_pub

    // Slot 0 ab Byte 72
    let s_off = 72;
    buf[s_off] = 0; // slot_id
    buf[s_off + 1] = 0; // anchor_id
    buf[s_off + 2] = 0; // kdf_id
    buf[s_off + 3] = 0; // status: leer

    // SHA-256 Prüfsumme über [0..584)
    let digest = Sha256::digest(&buf[0..584]);
    buf[584..600].copy_from_slice(&digest[0..16]);

    let parsed = HeaderAb::parse(&buf).expect("Parse HeaderAb");
    assert_eq!(parsed.generation, 42);
    assert_eq!(parsed.slot_count, 1);
    assert_eq!(parsed.slots.len(), 4);
}

/// SPEC: T-FMT-1 (F.3 Tresor)
#[test]
fn t_fmt_1_f3_tresor_golden() {
    let file_key = [0x55u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::Vault, [0x01; 16], 1, &file_key);

    let mut vault_bytes = Vec::new();
    vault_bytes.extend_from_slice(&header_bytes);

    // Metablock (64 B): vault_id, shard_no=0, cap_log2=1 (capacity=2), live=1, tombstones=1, gen=1
    let mut meta = [0u8; 64];
    meta[0..16].copy_from_slice(&[0xaa; 16]); // vault_id
    meta[16] = 0; // shard_no
    meta[17] = 1; // cap_log2 = 1 (2^1 = 2)
    meta[18..26].copy_from_slice(&1u64.to_be_bytes());
    meta[26..34].copy_from_slice(&1u64.to_be_bytes());
    meta[34..42].copy_from_slice(&1u64.to_be_bytes());
    vault_bytes.extend_from_slice(&meta);

    // Slot 0 (Offset 128..192): Aktiv Document (64 B)
    let mut slot0 = [0u8; 64];
    slot0[0] = VaultSlotType::Document as u8;
    slot0[1] = VaultSlotStatus::Active as u8;
    slot0[8..24].copy_from_slice(&[0xdd; 16]);
    slot0[24..56].copy_from_slice(&[0xee; 32]);
    vault_bytes.extend_from_slice(&slot0);

    // Slot 1 (Offset 192..256): Grabstein (alle Bytes 0 außer Byte 1 = 0x02)
    let mut slot1 = [0u8; 64];
    slot1[1] = VaultSlotStatus::Tombstone as u8;
    vault_bytes.extend_from_slice(&slot1);

    let parsed = VaultFile::parse(&vault_bytes).expect("Parse Vault");
    assert_eq!(parsed.meta.capacity(), 2);
    assert_eq!(parsed.slots[0].status, VaultSlotStatus::Active);
    assert_eq!(parsed.slots[1].status, VaultSlotStatus::Tombstone);
}

/// SPEC: T-FMT-1 (F.4 WAL & Manifest-Log)
#[test]
fn t_fmt_1_f4_wal_manifest_golden() {
    let file_key = [0x66u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::ManifestLog, [0x04; 16], 1, &file_key);

    let parsed_manifest =
        ManifestLog::parse(&header_bytes, &file_key).expect("Parse Empty Manifest");
    assert_eq!(parsed_manifest.entries.len(), 0);

    let wal_header = FileHeader::create_bytes(FileKind::WalSegment, [0x05; 16], 1, &file_key);
    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&wal_header);
    // Segmentkopf: prev_mac(32) + first_seq(8)
    wal_bytes.extend_from_slice(&[0u8; 32]);
    wal_bytes.extend_from_slice(&1u64.to_be_bytes());

    let k_wal = [0x99u8; 32];
    let parsed_wal = WalSegment::parse(&wal_bytes, &k_wal, &file_key).expect("Parse Empty Wal");
    assert_eq!(parsed_wal.frames.len(), 0);
}

/// SPEC: T-FMT-1 (F.5 SSTable)
#[test]
fn t_fmt_1_f5_sstable_golden() {
    let file_key = [0x77u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::SsTable, [0x02; 16], 10, &file_key);

    let mut sst_bytes = Vec::new();
    sst_bytes.extend_from_slice(&header_bytes);

    // Datenblock: Eintrag 1 (normal) + Eintrag 2 (Tombstone)
    let mut block = Vec::new();
    // klen=4, key="key1", vlen=5, val="value"
    block.extend_from_slice(&4u16.to_be_bytes());
    block.extend_from_slice(b"key1");
    block.extend_from_slice(&5u32.to_be_bytes());
    block.extend_from_slice(b"value");

    // klen=4, key="key2", vlen=0 (Tombstone)
    block.extend_from_slice(&4u16.to_be_bytes());
    block.extend_from_slice(b"key2");
    block.extend_from_slice(&0u32.to_be_bytes());

    let entries = SsTableEntry::parse_entries(&block).expect("Parse Block Entries");
    assert_eq!(entries.len(), 2);
    assert!(!entries[0].is_tombstone);
    assert!(entries[1].is_tombstone);

    sst_bytes.extend_from_slice(&block);

    // Block geht von 64 bis 64 + block.len()
    let index_off = 64 + block.len() as u64;
    let bloom_off = index_off + 10;
    let stats_off = bloom_off + 10;
    let block_count = 1u64;

    // Fülle Dummy-Bytes für Index, Bloom und Stats ein
    sst_bytes.extend_from_slice(&[0u8; 30]);

    let mut footer = [0u8; 64];
    footer[0..8].copy_from_slice(&index_off.to_be_bytes());
    footer[8..16].copy_from_slice(&bloom_off.to_be_bytes());
    footer[16..24].copy_from_slice(&stats_off.to_be_bytes());
    footer[24..32].copy_from_slice(&block_count.to_be_bytes());
    footer[56..64].copy_from_slice(SSTABLE_FOOTER_MAGIC);

    sst_bytes.extend_from_slice(&footer);

    let parsed_sst = SsTableFile::parse(&sst_bytes).expect("Parse SSTable");
    assert_eq!(parsed_sst.footer.block_count, 1);
}

/// SPEC: T-FMT-1 (F.6 Record-Envelope v2)
#[test]
fn t_fmt_1_f6_record_envelope_golden() {
    let dek = [0x33u8; 32];
    let doc_tag = [0x99u8; 16];
    let nonce = [0x12u8; 12];
    let key_bytes = b"my_sstable_key!000100";
    let plaintext = b"Sensible Document Payload";

    let env_bytes = RecordEnvelopeV2::create_bytes(doc_tag, nonce, plaintext, &dek, key_bytes)
        .expect("Create Envelope");

    let parsed = RecordEnvelopeV2::parse(&env_bytes).expect("Parse Envelope");
    assert_eq!(parsed.version, 2);
    assert_eq!(parsed.doc_tag, doc_tag);
    assert_eq!(parsed.nonce, nonce);

    let decrypted = parsed.decrypt(&dek, key_bytes).expect("Decrypt Envelope");
    assert_eq!(decrypted, plaintext);

    // Falscher Schlüssel oder falsche AAD
    assert!(parsed.decrypt(&dek, b"wrong_key").is_err());
    assert!(parsed.decrypt(&[0x00; 32], key_bytes).is_err());
}

/// SPEC: T-FMT-1 (F.7 Textsegment)
#[test]
fn t_fmt_1_f7_textsegment_golden() {
    let file_key = [0x88u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::TextSegment, [0x06; 16], 1, &file_key);

    let mut text_bytes = Vec::new();
    text_bytes.extend_from_slice(&header_bytes);

    let mut seg_head = [0u8; 33];
    seg_head[0] = 0; // bucket
    seg_head[1] = 1; // bucket_count
    seg_head[2] = 1; // analyzer_id
    seg_head[3..5].copy_from_slice(&1u16.to_be_bytes()); // version
    seg_head[5..8].copy_from_slice(b"de\0"); // lang
    seg_head[8] = 1; // flags: positions
    seg_head[9..13].copy_from_slice(&10u32.to_be_bytes()); // doc_count
    seg_head[13..17].copy_from_slice(&50u32.to_be_bytes()); // term_count
    seg_head[17..21].copy_from_slice(&100u32.to_be_bytes()); // member_count
    seg_head[21..29].copy_from_slice(&500u64.to_be_bytes()); // total_len

    text_bytes.extend_from_slice(&seg_head);

    let parsed = TextSegmentFile::parse(&text_bytes).expect("Parse TextSegment");
    assert_eq!(parsed.segment_header.doc_count, 10);
    assert_eq!(parsed.segment_header.term_count, 50);
    assert_eq!(parsed.segment_header.member_count, 100);

    // Test Wörterbucheintrag (32 Bytes)
    let mut term_raw = [0u8; 32];
    term_raw[0..16].copy_from_slice(&[0x11; 16]); // term_tag
    term_raw[16..20].copy_from_slice(&5u32.to_be_bytes()); // df
    term_raw[20..28].copy_from_slice(&100u64.to_be_bytes()); // posting_off
    term_raw[28..32].copy_from_slice(&24u32.to_be_bytes()); // posting_len

    let term = TermEntry::parse(&term_raw).expect("Parse TermEntry");
    assert_eq!(term.df, 5);
}

/// SPEC: T-FMT-1 (F.8 Vektor-Snapshot)
#[test]
fn t_fmt_1_f8_vector_snapshot_golden() {
    let file_key = [0x99u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::VectorSnapshot, [0x07; 16], 1, &file_key);

    let mut vec_bytes = Vec::new();
    vec_bytes.extend_from_slice(&header_bytes);

    let mut snap_head = [0u8; 24];
    snap_head[0..2].copy_from_slice(&128u16.to_be_bytes()); // dim
    snap_head[2] = VectorMetric::Cosine as u8; // metric
    snap_head[3..5].copy_from_slice(&16u16.to_be_bytes()); // M
    snap_head[5..7].copy_from_slice(&64u16.to_be_bytes()); // ef_c
    snap_head[7..11].copy_from_slice(&1000u32.to_be_bytes()); // slot_count
    snap_head[11..15].copy_from_slice(&0u32.to_be_bytes()); // entry_slot
    snap_head[15] = 4; // max_level
    snap_head[16..24].copy_from_slice(&500u64.to_be_bytes()); // built_through_seq

    vec_bytes.extend_from_slice(&snap_head);

    let parsed = VectorSnapshotFile::parse(&vec_bytes).expect("Parse VectorSnapshot");
    assert_eq!(parsed.snapshot_header.dim, 128);
    assert_eq!(parsed.snapshot_header.metric, VectorMetric::Cosine);
    assert_eq!(parsed.snapshot_header.built_through_seq, 500);
}

/// SPEC: T-FMT-1 (F.9 Append-Logs: Journal, Gate-Log, Ledger)
#[test]
fn t_fmt_1_f9_append_logs_golden() {
    let file_key = [0x11u8; 32];
    let journal_head = FileHeader::create_bytes(FileKind::Journal, [0x08; 16], 1, &file_key);
    let parsed_journal =
        AppendLogFile::parse(&journal_head, FileKind::Journal).expect("Parse Empty Journal");
    assert_eq!(parsed_journal.entries.len(), 0);

    let gate_head = FileHeader::create_bytes(FileKind::GateLog, [0x09; 16], 1, &file_key);
    let parsed_gate =
        AppendLogFile::parse(&gate_head, FileKind::GateLog).expect("Parse Empty GateLog");
    assert_eq!(parsed_gate.entries.len(), 0);

    let ledger_head = FileHeader::create_bytes(FileKind::Ledger, [0x0a; 16], 1, &file_key);
    let parsed_ledger =
        AppendLogFile::parse(&ledger_head, FileKind::Ledger).expect("Parse Empty Ledger");
    assert_eq!(parsed_ledger.entries.len(), 0);
}

/// SPEC: T-FMT-1 (F.10 Bericht & Ed25519 Signatur)
#[test]
fn t_fmt_1_f10_erasure_report_golden() {
    let signing_key = SigningKey::from_bytes(&[0x44; 32]);
    let pub_key = signing_key.verifying_key();
    let pub_bytes = pub_key.to_bytes();

    let digest = Sha256::digest(&pub_bytes);
    let fp: String = digest[0..16].iter().map(|b| format!("{b:02x}")).collect();

    let canonical_json_without_sig =
        br#"{"class_g":"reached","class_k":"reached","class_p":"reached","completed_at_micros":1000,"erasure_id":"test-01","gated_at_micros":500,"pub_fingerprint":""#;

    let mut signed_data = Vec::new();
    signed_data.extend_from_slice(b"ctx/report/v1");
    signed_data.extend_from_slice(canonical_json_without_sig);

    let sig = signing_key.sign(&signed_data);
    let sig_hex: String = sig.to_bytes().iter().map(|b| format!("{b:02x}")).collect();

    let full_json = format!(
        r#"{{"erasure_id":"test-01","requested_at_micros":100,"gated_at_micros":500,"completed_at_micros":1000,"class_g":"reached","class_k":"reached","class_p":"reached","residual_risks":["FlashRemanence"],"pub_fingerprint":"{fp}","signature":"{sig_hex}"}}"#
    );

    let report = ErasureReport::from_json_str(&full_json).expect("Parse Report JSON");
    assert_eq!(report.class_g, ClassStatus::Reached);
    assert_eq!(report.class_k, ClassStatus::Reached);
    assert_eq!(report.class_p, ClassStatus::Reached);

    report
        .verify_signature(canonical_json_without_sig, &pub_bytes)
        .expect("Verify Ed25519");
}

/// T-ID: T-FMT-1
#[test]
fn t_fmt_1_f9_scope_file_golden() {
    use contextra_lab::format::scope::ScopeFile;

    let file_id = [0x5au8; 16];
    let file_key = [0x7bu8; 32];
    let payload = b"scope:tenant=1,coll=2,doc=3";

    let serialized = ScopeFile::serialize(file_id, payload, &file_key).expect("serialize scope");
    let parsed = ScopeFile::parse(&serialized, &file_key).expect("parse scope");

    assert_eq!(parsed.header.file_id, file_id);
    assert_eq!(parsed.payload, payload);
}
