#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_borrows_for_generic_args
)]
//! Tests für den Scanner und Messklassen (SPEC §12.1, §12.2).
//!
//! SPEC: T-RAW-1, T-RAW-2, T-RAW-3, T-RAW-4, T-RAW-5, T-RAW-6

use contextra_lab::format::file_header::{FileHeader, FileKind};
use contextra_lab::format::record_envelope::RecordEnvelopeV2;
use contextra_lab::format::vault::{VaultFile, VaultSlotStatus};
use contextra_lab::scanner::classes::{
    Mk1StructureScanner, Mk2RawScanner, Mk3KeyProbeScanner, StructureTarget,
};
use contextra_lab::scanner::{ControlEvidence, MeasurementClass, Scanner};

/// SPEC: T-RAW-2 (Positivkontrolle ist typisiert erzwungen, D-72)
#[test]
fn t_raw_2_positivkontrolle_typisiert() {
    let needle = b"KNOWN_CONTROL_PATTERN";

    // 1. Fehlgeschlagene Positivkontrolle: 0 Matches
    let fail_res = ControlEvidence::try_new(needle, 0);
    assert!(
        fail_res.is_err(),
        "0 Matches dürfen kein ControlEvidence erzeugen"
    );

    // 2. Gültige Positivkontrolle: 1 Match
    let evidence = ControlEvidence::try_new(needle, 1).expect("Gültiges ControlEvidence");

    let scanner = Scanner::new("doc_store", MeasurementClass::Mk2);
    let mut armed_scanner = scanner.arm(evidence);
    armed_scanner.record_progress(5, 4096);

    // Zielmessung mit 0 Matches -> Gültiger LayerWitness
    let witness = armed_scanner.finish(0).expect("Gültiger Zeuge");
    assert_eq!(witness.matches, 0);
    assert!(witness.control_ok);
    assert_eq!(witness.scanned_files, 5);
}

/// SPEC: T-RAW-2 (Scanner schlägt fehl wenn Reste gefunden werden)
#[test]
fn t_raw_2_scanner_findet_reste_schlaegt_fehl() {
    let evidence = ControlEvidence::try_new(b"CONTROL", 1).expect("Evidence");
    let scanner = Scanner::new("doc_store", MeasurementClass::Mk2).arm(evidence);

    // Messung findet 1 Zielmatch -> Darf keinen gültigen Zeugen erzeugen
    let res = scanner.finish(1);
    assert!(res.is_err(), "matches > 0 muss fehlschlagen");
}

/// SPEC: T-RAW-1 (MK1: Strukturscan)
#[test]
fn t_raw_1_mk1_strukturscan() {
    let file_key = [0x55u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::Vault, [0x01; 16], 1, &file_key);

    let mut vault_bytes = Vec::new();
    vault_bytes.extend_from_slice(&header_bytes);

    let mut meta = [0u8; 64];
    meta[0..8].copy_from_slice(&2u64.to_be_bytes()); // capacity = 2
    vault_bytes.extend_from_slice(&meta);
    vault_bytes.extend_from_slice(&[0u8; 64]); // base offset padding

    let target_tag = [0x42u8; 16];

    // Slot 0: Grabstein für target_tag
    let mut slot0 = [0u8; 80];
    slot0[49] = VaultSlotStatus::Tombstone as u8;
    vault_bytes.extend_from_slice(&slot0);

    // Slot 1: Unbelegter Slot
    vault_bytes.extend_from_slice(&[0u8; 80]);

    let vault = VaultFile::parse(&vault_bytes).expect("Parse Vault");
    let target = StructureTarget {
        doc_tag: target_tag,
    };

    let matches = Mk1StructureScanner::scan_vault(&vault, &target);
    assert_eq!(
        matches, 0,
        "Grabstein zählt nicht als aktiver Strukturtreffer"
    );
}

/// SPEC: T-RAW-4, T-RAW-6 (MK2: Rohscan auf Klartext-Needles)
#[test]
fn t_raw_4_mk2_rohscan() {
    let needle = b"SECRET_USER_NAME";
    let clean_buffer = b"CTXFILE\0random_encrypted_bytes_with_no_plain_data";
    assert_eq!(Mk2RawScanner::scan_bytes(clean_buffer, needle), 0);

    let dirty_buffer = b"CTXFILE\0prefix_SECRET_USER_NAME_suffix";
    assert_eq!(Mk2RawScanner::scan_bytes(dirty_buffer, needle), 1);
}

/// SPEC: T-RAW-3 (MK3: Schlüsselprobe mit altem DEK)
#[test]
fn t_raw_3_mk3_schluesselprobe() {
    let old_dek = [0x77u8; 32];
    let new_dek = [0x88u8; 32];
    let doc_tag = [0x11u8; 16];
    let nonce = [0x22u8; 12];
    let key_bytes = b"key";

    // 1. Chiffrat noch unter altem DEK verschlüsselt -> öffenbar (Verstoß)
    let dirty_env = RecordEnvelopeV2::create_bytes(doc_tag, nonce, b"secret", &old_dek, key_bytes)
        .expect("Create");
    let is_openable =
        Mk3KeyProbeScanner::probe_envelope_decryptable(&dirty_env, &old_dek, key_bytes)
            .expect("Probe");
    assert!(is_openable, "Altes Chiffrat ist öffenbar");

    // 2. Chiffrat wurde neu verschlüsselt / rekeyed -> mit altem DEK nicht öffenbar (sauber)
    let clean_env = RecordEnvelopeV2::create_bytes(doc_tag, nonce, b"secret", &new_dek, key_bytes)
        .expect("Create");
    let is_openable =
        Mk3KeyProbeScanner::probe_envelope_decryptable(&clean_env, &old_dek, key_bytes)
            .expect("Probe");
    assert!(!is_openable, "Neues Chiffrat ist mit altem DEK unöffenbar");
}
