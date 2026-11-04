#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_borrows_for_generic_args
)]
//! Tests für den Scanner und Messklassen (SPEC §12.1, §12.2).
//!
//! SPEC: T-RAW-1, T-RAW-2, T-RAW-3, T-RAW-4, T-RAW-6, T-CLASS-1, T-KEYCOV-1

use contextra_lab::format::file_header::{FileHeader, FileKind};
use contextra_lab::format::record_envelope::RecordEnvelopeV2;
use contextra_lab::format::report::{ClassStatus, ErasureReport, KeyCoverage, LayerKeyCoverage};
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

    // Auch Indexschlüssel auf Treffer prüfen
    let index_keys = [[0x10; 16], [0x20; 16], [0x30; 16]];
    assert_eq!(
        Mk1StructureScanner::scan_index_keys(&index_keys, &target_tag),
        0
    );
    assert_eq!(
        Mk1StructureScanner::scan_index_keys(&index_keys, &[0x20; 16]),
        1
    );
}

/// SPEC: T-RAW-4 (MK2: Rohscan auf Klartext-Needles & Index-Wörterbuch-Tags)
#[test]
fn t_raw_4_mk2_rohscan() {
    let needle = b"SECRET_USER_NAME";
    let clean_buffer = b"CTXFILE\0random_encrypted_bytes_with_no_plain_data";
    assert_eq!(Mk2RawScanner::scan_bytes(clean_buffer, needle), 0);

    let dirty_buffer = b"CTXFILE\0prefix_SECRET_USER_NAME_suffix";
    assert_eq!(Mk2RawScanner::scan_bytes(dirty_buffer, needle), 1);

    // Index-Wörterbuch-Tags prüfen
    let clean_terms: [&[u8]; 2] = [b"blind_tag_01\0\0\0\0", b"blind_tag_02\0\0\0\0"];
    assert_eq!(
        Mk2RawScanner::scan_index_terms_for_plain_leak(&clean_terms, needle),
        0
    );

    let dirty_terms: [&[u8]; 2] = [b"blind_tag_01\0\0\0\0", b"SECRET_USER_NAME_tag"];
    assert_eq!(
        Mk2RawScanner::scan_index_terms_for_plain_leak(&dirty_terms, needle),
        1
    );
}

/// SPEC: T-RAW-6 (MK2: Verzeichnis- und Dateinamen nur blinde Tags / file_id)
#[test]
fn t_raw_6_mk2_dateinamen_blind_tags() {
    let forbidden = ["alice_smith", "sensitive_user"];

    // Valider Pfad nach D-75: text/<tenant_tag_hex>/<coll_tag_hex>/<file_id_hex>.seg
    let valid_path = "text/0123456789abcdef/fedcba9876543210/aabbccddeeff0011.seg";
    let leaks = Mk2RawScanner::validate_path_elements_blind_tags_only(valid_path, &forbidden)
        .expect("Validate path");
    assert_eq!(
        leaks, 0,
        "Konformer Pfad darf keine verbotenen Klartextnamen enthalten"
    );

    // Invalider Pfad mit Personenbezug im Verzeichnisnamen
    let invalid_path = "text/alice_smith/documents/aabbccddeeff0011.seg";
    let leaks = Mk2RawScanner::validate_path_elements_blind_tags_only(invalid_path, &forbidden)
        .expect("Validate path");
    assert_eq!(
        leaks, 1,
        "Klartext-Nutzername im Pfad muss als Leck erkannt werden"
    );
}

/// SPEC: T-RAW-3 (MK3: Schlüsselprobe mit altem DEK auf Envelopes & Vektor-Slots)
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

    // 3. Vektor-Slot Envelope Probe (D-19, T-RAW-3)
    let slot_aad = b"slot_0001_aad";
    let vector_env =
        RecordEnvelopeV2::create_bytes(doc_tag, nonce, b"embedding_bytes", &old_dek, slot_aad)
            .expect("Create vector env");
    let vec_openable =
        Mk3KeyProbeScanner::probe_vector_slot_envelope_decryptable(&vector_env, &old_dek, slot_aad)
            .expect("Probe vector slot");
    assert!(
        vec_openable,
        "Vektor-Slot mit altem DEK ist vor Löschung öffenbar"
    );

    let vec_closed =
        Mk3KeyProbeScanner::probe_vector_slot_envelope_decryptable(&vector_env, &new_dek, slot_aad)
            .expect("Probe vector slot");
    assert!(
        !vec_closed,
        "Vektor-Slot ist mit vernichtetem/falschem DEK unöffenbar"
    );
}

/// SPEC: T-CLASS-1 (Klassenverband G/K/P je einzeln, keine Gesamtnote)
#[test]
fn t_class_1_klassenverband_einzeln_ausgewiesen() {
    let report = ErasureReport {
        erasure_id: "erasure-99".to_string(),
        requested_at_micros: 100,
        gated_at_micros: 200,
        completed_at_micros: 300,
        class_g: ClassStatus::Reached,
        class_k: ClassStatus::Reached,
        class_p: ClassStatus::Reached,
        key_coverage: LayerKeyCoverage::default(),
        residual_risks: vec!["FlashRemanence".to_string()],
        pub_fingerprint: "0123456789abcdef0123456789abcdef".to_string(),
        signature: "aabb".to_string(),
    };

    assert_eq!(report.class_g, ClassStatus::Reached);
    assert_eq!(report.class_k, ClassStatus::Reached);
    assert_eq!(report.class_p, ClassStatus::Reached);
    assert!(report.verify_class_lattice().is_ok());
}

/// SPEC: T-KEYCOV-1 (key_coverage je Schicht stimmt mit Sollzustand überein)
#[test]
fn t_keycov_1_schichtdeckung_vertrag() {
    let mut cov = LayerKeyCoverage::default();
    assert_eq!(cov.doc_store, KeyCoverage::Full);
    assert_eq!(cov.vector_payload, KeyCoverage::Full);
    assert_eq!(cov.text_postings, KeyCoverage::Unlinked);
    assert_eq!(cov.vector_edges, KeyCoverage::None);
    assert_eq!(cov.file_structure, KeyCoverage::None);

    let report = ErasureReport {
        erasure_id: "erasure-100".to_string(),
        requested_at_micros: 100,
        gated_at_micros: 200,
        completed_at_micros: 300,
        class_g: ClassStatus::Reached,
        class_k: ClassStatus::Reached,
        class_p: ClassStatus::Reached,
        key_coverage: cov.clone(),
        residual_risks: vec![],
        pub_fingerprint: "0123456789abcdef0123456789abcdef".to_string(),
        signature: "aabb".to_string(),
    };
    assert!(report.verify_key_coverage_contract().is_ok());

    // Abweichung bei text_postings -> Verletzung von D-69
    cov.text_postings = KeyCoverage::Full;
    let bad_report = ErasureReport {
        key_coverage: cov,
        ..report
    };
    assert!(bad_report.verify_key_coverage_contract().is_err());
}
