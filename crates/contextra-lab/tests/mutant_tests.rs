#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_borrows_for_generic_args
)]
//! T-MUT-1: Mutationstests für die Mess- und Verifikationslogik (SPEC §12.3, T-MUT-1).
//!
//! Ziel: Nachweis, dass absichtlich gebrochene Gate-, Tresor-, Purge- und Scanner-
//! Varianten (Mutanten) von der Prüfinstanz zuverlässig erkannt und abgewiesen werden.

use contextra_lab::error::LabError;
use contextra_lab::format::file_header::{FileHeader, FileKind};
use contextra_lab::format::record_envelope::RecordEnvelopeV2;
use contextra_lab::format::vault::{VaultFile, VaultSlotStatus};
use contextra_lab::scanner::classes::{Mk2RawScanner, Mk3KeyProbeScanner};
use contextra_lab::scanner::{ControlEvidence, MeasurementClass, Scanner};

/// SPEC: T-MUT-1 (Mutant 1: Tresor-Grabstein vernichtet nicht vollständig, hinterlässt Restdaten)
#[test]
fn t_mut_1_mutant_unvollstaendiger_grabstein_erkannt() {
    let file_key = [0x55u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::Vault, [0x01; 16], 1, &file_key);

    let mut vault_bytes = Vec::new();
    vault_bytes.extend_from_slice(&header_bytes);

    let mut meta = [0u8; 64];
    meta[0..8].copy_from_slice(&1u64.to_be_bytes()); // capacity = 1
    vault_bytes.extend_from_slice(&meta);
    vault_bytes.extend_from_slice(&[0u8; 64]); // padding bis Offset 192

    // Mutant: Slot hat status = Tombstone (0x02), aber Byte 10 ist NICHT 0 (Schlüsselrest nicht genullt)
    let mut bad_slot = [0u8; 80];
    bad_slot[10] = 0xff; // Verstoß!
    bad_slot[49] = VaultSlotStatus::Tombstone as u8;
    vault_bytes.extend_from_slice(&bad_slot);

    // Der Parser muss die ungültige Grabstein-Mutation erkennen
    let res = VaultFile::parse(&vault_bytes);
    assert!(
        res.is_err(),
        "Mutant mit nicht-genulltem Grabstein muss abgewiesen werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 2: Scanner meldet Erfolg trotz Zieltreffern)
#[test]
fn t_mut_1_mutant_scanner_ignoriert_reste_abgewiesen() {
    let evidence = ControlEvidence::try_new(b"CONTROL", 1).expect("Evidence");
    let scanner = Scanner::new("doc_store", MeasurementClass::Mk2).arm(evidence);

    // Mutant-Szenario: Scanner soll Zieltreffer (matches = 2) ignorieren
    let res = scanner.finish(2);
    assert!(
        matches!(res, Err(LabError::VerificationFailed(_))),
        "Mutant mit Resttreffern darf keinen Zeugen erzeugen"
    );
}

/// SPEC: T-MUT-1 (Mutant 3: Rohscan übersieht Needle durch falsches Fensterschrittmaß)
#[test]
fn t_mut_1_mutant_rohscan_fenster_mutation() {
    let raw = b"AAAA_TARGET_NEEDLE_BBBB";
    let needle = b"TARGET_NEEDLE";

    let count = Mk2RawScanner::scan_bytes(raw, needle);
    assert_eq!(count, 1, "Exakter Needle-Match muss detektiert werden");

    let count_missing = Mk2RawScanner::scan_bytes(raw, b"NON_EXISTENT");
    assert_eq!(count_missing, 0);
}

/// SPEC: T-MUT-1 (Mutant 4: Schlüsselprobe schlägt fälschlicherweise nicht an, wenn Chiffrat lesbar)
#[test]
fn t_mut_1_mutant_schluesselprobe_erkennt_offenes_chiffrat() {
    let dek = [0x42u8; 32];
    let key_bytes = b"my_key";
    let env_bytes =
        RecordEnvelopeV2::create_bytes([0x01; 16], [0x02; 12], b"secret", &dek, key_bytes)
            .expect("Create");

    let is_openable =
        Mk3KeyProbeScanner::probe_envelope_decryptable(&env_bytes, &dek, key_bytes).expect("Probe");
    assert!(
        is_openable,
        "Wenn Altschlüssel noch funktioniert, MUSS dies als Verstoß (true) gemeldet werden"
    );
}
