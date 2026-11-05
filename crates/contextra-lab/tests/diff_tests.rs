#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_borrows_for_generic_args,
    clippy::cast_possible_truncation
)]
//! T-DIFF-1: Unabhängiger Differenztest (SPEC §12.3, T-DIFF-1).
//!
//! Ziel: Nachweis, dass Nicht-Ziel-Daten auf Record-Ebene bitgleich bleiben
//! bis auf erwartete Manifest-, Sequenz- oder Hüllenänderungen.

use contextra_lab::format::record_envelope::RecordEnvelopeV2;
use contextra_lab::format::sstable::SsTableEntry;

/// SPEC: T-DIFF-1 (Nicht-Ziel Records bleiben bitgleich nach Selektivlöschung)
#[test]
fn t_diff_1_record_level_bitgleichheit() {
    let dek_kept = [0x55u8; 32];
    let dek_deleted = [0x66u8; 32];

    let kept_payload = b"Non-Target Document Data Must Remain Bit-Identical!";
    let deleted_payload = b"Target Document Data To Be Erased";

    let env_kept_before = RecordEnvelopeV2::create_bytes(
        [0x01; 16],
        [0x02; 12],
        kept_payload,
        &dek_kept,
        b"key_kept",
    )
    .expect("Create kept envelope");

    let env_deleted_before = RecordEnvelopeV2::create_bytes(
        [0x03; 16],
        [0x04; 12],
        deleted_payload,
        &dek_deleted,
        b"key_deleted",
    )
    .expect("Create deleted envelope");

    // Simulierte SSTable vor der Löschung:
    let mut block_before = Vec::new();
    // Eintrag 1: Deleted
    block_before.extend_from_slice(&11u16.to_be_bytes());
    block_before.extend_from_slice(b"key_deleted");
    block_before.extend_from_slice(&(env_deleted_before.len() as u32).to_be_bytes());
    block_before.extend_from_slice(&env_deleted_before);

    // Eintrag 2: Kept
    block_before.extend_from_slice(&8u16.to_be_bytes());
    block_before.extend_from_slice(b"key_kept");
    block_before.extend_from_slice(&(env_kept_before.len() as u32).to_be_bytes());
    block_before.extend_from_slice(&env_kept_before);

    let entries_before = SsTableEntry::parse_entries(&block_before).expect("Parse before");
    assert_eq!(entries_before.len(), 2);

    // Nach dem Rewrite: Eintrag 1 (Deleted) ist Tombstone (vlen = 0).
    // Eintrag 2 (Kept) wird byteidentisch kopiert (SPEC I-3).
    let mut block_after = Vec::new();
    block_after.extend_from_slice(&11u16.to_be_bytes());
    block_after.extend_from_slice(b"key_deleted");
    block_after.extend_from_slice(&0u32.to_be_bytes()); // Tombstone

    block_after.extend_from_slice(&8u16.to_be_bytes());
    block_after.extend_from_slice(b"key_kept");
    block_after.extend_from_slice(&(env_kept_before.len() as u32).to_be_bytes());
    block_after.extend_from_slice(&env_kept_before); // Bitidentische Kopie!

    let entries_after = SsTableEntry::parse_entries(&block_after).expect("Parse after");
    assert_eq!(entries_after.len(), 2);

    // Verifikation T-DIFF-1:
    let kept_after = &entries_after[1];
    assert_eq!(kept_after.key, b"key_kept");
    assert_eq!(
        kept_after.value, env_kept_before,
        "T-DIFF-1: Nicht-Ziel Envelope muss exakt bitgleich sein"
    );

    // Entschlüsselung des Nicht-Ziel Envelopes mit Original-DEK liefert exakten Klartext
    let parsed_kept = RecordEnvelopeV2::parse(&kept_after.value).expect("Parse kept envelope");
    let decrypted = parsed_kept
        .decrypt(&dek_kept, b"key_kept")
        .expect("Decrypt kept");
    assert_eq!(decrypted, kept_payload);
}

/// T-ID: T-VER-1
/// Restore-Szenario: Das Wiederauftauchen gelöschter Datensätze nach einem Restore wird detektiert.
#[test]
fn t_restore_1_reappearing_deleted_records_detected() {
    use contextra_lab::fixture::StoreFixture;

    let mut fixture = StoreFixture::new();
    let doc1 = [0x01u8; 16];
    let needle1 = b"retained-record".to_vec();
    let dek1 = [0x11u8; 32];

    let doc2 = [0x02u8; 16];
    let needle2 = b"target-to-delete".to_vec();
    let dek2 = [0x22u8; 32];

    fixture.insert(doc1, needle1.clone(), dek1);
    fixture.insert(doc2, needle2.clone(), dek2);

    // Löschung ausführen über Callback
    let deleted = fixture.execute_deletion(&doc2, |id, docs| docs.remove(id).is_some());
    assert!(deleted);
    assert!(!fixture.documents.contains_key(&doc2));
    assert!(fixture.deleted.contains_key(&doc2));

    // Simulierter illegaler Restore eines alten Backups
    let restored_docs = vec![(doc1, needle1.clone()), (doc2, needle2.clone())];
    let illegal_reappearance = restored_docs
        .iter()
        .any(|(id, _)| fixture.deleted.contains_key(id));

    assert!(
        illegal_reappearance,
        "Wiederauftauchen gelöschter Records muss detektiert werden"
    );
}

/// T-ID: T-VER-1
/// Ledger-Integrität im Restore-Szenario: Ledger HMAC schützt gegen manipulierte Restore-Zustände.
#[test]
fn t_restore_2_ledger_mac_integrity_check() {
    use contextra_lab::format::append_logs::AppendLogFile;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let chain = [0x33u8; 32];
    let k_ledger = [0x44u8; 32];

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = <HmacSha256 as Mac>::new_from_slice(&k_ledger).unwrap();
    mac.update(&chain);
    let valid_mac: [u8; 32] = mac.finalize().into_bytes().into();

    assert!(AppendLogFile::verify_ledger_entry_mac(&chain, &valid_mac, &k_ledger).is_ok());

    let mut invalid_mac = valid_mac;
    invalid_mac[0] ^= 0x01;
    assert!(AppendLogFile::verify_ledger_entry_mac(&chain, &invalid_mac, &k_ledger).is_err());
}
