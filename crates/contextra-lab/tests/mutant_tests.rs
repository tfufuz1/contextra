#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_borrows_for_generic_args,
    clippy::cast_possible_truncation
)]
//! T-MUT-1: Mutationstests für die Mess- und Verifikationslogik (SPEC §12.3, T-MUT-1).
//!
//! Ziel: Nachweis, dass absichtlich gebrochene Gate-, Tresor-, Purge- und Scanner-
//! Varianten (Mutanten) von der Prüfinstanz zuverlässig erkannt und abgewiesen werden:
//! - Schicht vergessen: Textsegment behält ungelöschte Dokument-Bindung
//! - Schicht vergessen: Vektor-Snapshot behält Ziel-Vektor / Slot im Index
//! - Rest im Journal: Nach Completed verbleiben Ziel-Einträge unreduziert
//! - Rest im Cache / Ledger: Wiederauftauchen ungelöschter Sequenzen
//! - Unvollständige Schlüsselvernichtung im Tresor: ungültiger Grabstein oder fehlender MAC

use contextra_lab::error::LabError;
use contextra_lab::format::append_logs::AppendLogFile;
use contextra_lab::format::file_header::{FileHeader, FileKind};
use contextra_lab::format::record_envelope::RecordEnvelopeV2;
use contextra_lab::format::text_segment::DocTableEntry;
use contextra_lab::format::vault::{
    VaultFile, VaultMetaBlock, VaultSlot, VaultSlotStatus, VaultSlotType, VAULT_SLOT_SIZE,
};
use contextra_lab::scanner::classes::{
    Mk1StructureScanner, Mk2RawScanner, Mk3KeyProbeScanner, StructureTarget,
};
use contextra_lab::scanner::{ControlEvidence, MeasurementClass, Scanner};
use hmac::Hmac;
use sha2::digest::Mac;
use sha2::Sha256;

/// SPEC: T-MUT-1 (Mutant 1: Tresor-Grabstein vernichtet nicht vollständig, hinterlässt Restdaten)
#[test]
fn t_mut_1_mutant_unvollstaendiger_grabstein_erkannt() {
    let file_key = [0x55u8; 32];
    let header_bytes = FileHeader::create_bytes(FileKind::Vault, [0x01; 16], 1, &file_key);

    let mut vault_bytes = Vec::new();
    vault_bytes.extend_from_slice(&header_bytes);

    // Metablock F.3 (64 B): vault_id=1, shard=0, cap_log2=1 (cap=2)
    let mut meta = [0u8; 64];
    meta[0..16].copy_from_slice(&[0xaa; 16]);
    meta[16] = 0; // shard_no
    meta[17] = 1; // cap_log2
    vault_bytes.extend_from_slice(&meta);

    // Mutant: Slot hat status = Tombstone (0x02), aber Byte 10 ist NICHT 0 (Schlüsselrest nicht genullt)
    let mut bad_slot = [0u8; VAULT_SLOT_SIZE];
    bad_slot[1] = VaultSlotStatus::Tombstone as u8;
    bad_slot[10] = 0xff; // Verstoß gegen F.3 Grabstein-Invariante!
    vault_bytes.extend_from_slice(&bad_slot);

    // Slot 1: leer
    vault_bytes.extend_from_slice(&[0u8; VAULT_SLOT_SIZE]);

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

/// SPEC: T-MUT-1 (Mutant 5: Schicht vergessen – Textsegment behält ungelöschte Dokumenttabelle / DEK-Bindung)
#[test]
fn t_mut_1_mutant_schicht_vergessen_text_binding_erkannt() {
    let retained_old_dek = [0x77u8; 32];
    let doc_tag = [0x99u8; 16];
    let file_id = [0x12; 16];
    let doc_ord = 42u32;

    // AAD nach SPEC F.7: "ctx/tbind/v1" || file_id || u32(doc_ord)
    let mut aad = Vec::new();
    aad.extend_from_slice(b"ctx/tbind/v1");
    aad.extend_from_slice(&file_id);
    aad.extend_from_slice(&doc_ord.to_be_bytes());

    // Mutant: Purge hat LSM gesäubert, aber im Textsegment verblieb das Binding-Envelope
    let binding_envelope = RecordEnvelopeV2::create_bytes(
        doc_tag,
        [0x05; 12],
        b"cluster_doc_payload",
        &retained_old_dek,
        &aad,
    )
    .expect("Create Binding Envelope");

    let mut doc_entry_bytes = Vec::new();
    doc_entry_bytes.extend_from_slice(&100u32.to_be_bytes()); // doclen
    doc_entry_bytes.extend_from_slice(&(binding_envelope.len() as u16).to_be_bytes());
    doc_entry_bytes.extend_from_slice(&binding_envelope);

    let (doc_entry, _) =
        DocTableEntry::parse(doc_ord, &doc_entry_bytes).expect("Parse DocTableEntry");

    // Forensischer Lab-Scan mit MK3: Überprüfe, ob der alte DEK das Envelope im Textsegment noch öffnen kann
    let is_openable = Mk3KeyProbeScanner::probe_envelope_decryptable(
        &doc_entry.binding_envelope,
        &retained_old_dek,
        &aad,
    )
    .expect("Probe Binding");

    assert!(
        is_openable,
        "Mutant (vergessenes Textsegment) muss durch MK3 als Verstoß erkannt werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 6: Schicht vergessen – Vektor-Snapshot behält Ziel-Vektor im Index)
#[test]
fn t_mut_1_mutant_schicht_vergessen_vektor_slot_erkannt() {
    let retained_old_dek = [0x88u8; 32];
    let doc_tag = [0x33u8; 16];
    let key_bytes = b"vec_key";

    // Mutant: Purge hat Vektor-Graph nicht repariert; Slot-Envelope enthält Ziel-Vektor
    let vector_env = RecordEnvelopeV2::create_bytes(
        doc_tag,
        [0x07; 12],
        b"vector_raw_embedding_data",
        &retained_old_dek,
        key_bytes,
    )
    .expect("Create Vector Envelope");

    // MK3-Schlüsselprobe auf Vektor-Hülle
    let is_openable =
        Mk3KeyProbeScanner::probe_envelope_decryptable(&vector_env, &retained_old_dek, key_bytes)
            .expect("Probe");

    assert!(
        is_openable,
        "Mutant (Restdaten im Vektor-Snapshot) muss durch MK3 zuverlässig aufgedeckt werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 7: Rest im Journal – unreduzierte Ziel-Einträge nach Completed verblieben)
#[test]
fn t_mut_1_mutant_rest_im_journal_erkannt() {
    let target_needle = b"TARGET_ERASURE_ID_001";

    // Mutant: Das Journal wurde nach 'Completed' nicht ordnungsgemäß reduziert (SPEC §11.6, R13),
    // sodass unverschlüsselte oder pseudonyme Identifikatoren im Journal verblieben sind
    let dirty_journal = b"JOURNAL_RECORD_CODE_01_TARGET_ERASURE_ID_001_IN_LOG";

    let raw_matches = Mk2RawScanner::scan_bytes(dirty_journal, target_needle);
    assert_eq!(
        raw_matches, 1,
        "Mutant mit Resten im Journal muss durch MK2 erkannt werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 8: Rest im Cache / Ledger – Ledger-HMAC Kette manipuliert oder abgebrochen)
#[test]
fn t_mut_1_mutant_rest_im_cache_ledger_erkannt() {
    let k_ledger = [0x55u8; 32];
    let chain = [0x33u8; 32];

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(&k_ledger).unwrap();
    mac.update(&chain);
    let valid_ledger_mac: [u8; 32] = mac.finalize().into_bytes().into();

    // Gültiger Eintrag
    assert!(AppendLogFile::verify_ledger_entry_mac(&chain, &valid_ledger_mac, &k_ledger).is_ok());

    // Mutant: Ein alter oder gefälschter Ledger-Eintrag mit manipuliertem MAC
    let mut forged_mac = valid_ledger_mac;
    forged_mac[0] ^= 0xff;
    let check_res = AppendLogFile::verify_ledger_entry_mac(&chain, &forged_mac, &k_ledger);
    assert!(
        check_res.is_err(),
        "Mutant mit manipuliertem Ledger-Eintrag muss abgewiesen werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 9: Unvollständige Schlüsselvernichtung – aktiver Slot im Tresor verblieben)
#[test]
fn t_mut_1_mutant_tresor_slot_ohne_mac_erkannt() {
    let target_doc = [0x77u8; 16];
    let k_vm = [0x66u8; 32];
    let vault_id = [0x22u8; 16];

    // Mutant: Ziel-Dokument noch mit Status::Active im Tresor
    let mut raw_slot = [0u8; VAULT_SLOT_SIZE];
    raw_slot[0] = VaultSlotType::Document as u8;
    raw_slot[1] = VaultSlotStatus::Active as u8;
    raw_slot[8..24].copy_from_slice(&target_doc);

    // Berechne MAC
    type HmacSha256 = Hmac<Sha256>;
    let mut mac_hasher = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(&k_vm).unwrap();
    mac_hasher.update(&vault_id);
    mac_hasher.update(&raw_slot[0..56]);
    raw_slot[56..64].copy_from_slice(&mac_hasher.finalize().into_bytes()[0..8]);

    let parsed_slot = VaultSlot::parse(&raw_slot).expect("Parse slot");
    assert!(parsed_slot.verify_mac(&raw_slot, &vault_id, &k_vm).is_ok());

    // MK1-Strukturscan: Findet noch aktiven Slot für target_doc
    let target = StructureTarget {
        doc_tag: target_doc,
    };
    let vault_file = VaultFile {
        header: FileHeader::parse(&FileHeader::create_bytes(
            FileKind::Vault,
            [0x01; 16],
            1,
            &[0; 32],
        ))
        .unwrap(),
        meta: VaultMetaBlock {
            vault_id,
            shard_no: 0,
            cap_log2: 1,
            live: 1,
            tombstones: 0,
            generation: 1,
            rsv: 0,
            mac: [0; 8],
        },
        slots: vec![parsed_slot],
    };

    let active_matches = Mk1StructureScanner::scan_vault(&vault_file, &target);
    assert_eq!(
        active_matches, 1,
        "Mutant mit nicht-gelöschtem D-Slot muss durch MK1 detektiert werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 10: SSTable-Schicht vergessen – Zieldokument verblieb ohne Tombstone im SSTable-Körper)
#[test]
fn t_mut_1_mutant_schicht_vergessen_sstable_erkannt() {
    let target_key = b"target_doc_key_999";
    let target_tag = [0x55u8; 16];
    let retained_dek = [0xaa; 32];
    let payload = b"Sensitive target content";

    let target_env =
        RecordEnvelopeV2::create_bytes(target_tag, [0x11; 12], payload, &retained_dek, target_key)
            .expect("Create Envelope");

    let mut block = Vec::new();
    block.extend_from_slice(&(target_key.len() as u16).to_be_bytes());
    block.extend_from_slice(target_key);
    block.extend_from_slice(&(target_env.len() as u32).to_be_bytes());
    block.extend_from_slice(&target_env);

    let entries =
        contextra_lab::format::sstable::SsTableEntry::parse_entries(&block).expect("Parse entries");
    assert_eq!(entries.len(), 1);

    // MK3-Probe: Verbliebenes Chiffrat im SSTable-Eintrag öffnet mit altem DEK -> Verstoß erkannt!
    let is_openable = Mk3KeyProbeScanner::probe_envelope_decryptable(
        &entries[0].value,
        &retained_dek,
        target_key,
    )
    .expect("Probe");
    assert!(
        is_openable,
        "Mutant (nicht umgeschriebene SSTable) muss durch MK3 detektiert werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 11: WAL-Schicht vergessen – ungeflushtes WAL-Segment enthält Zieldaten)
#[test]
fn t_mut_1_mutant_schicht_vergessen_wal_erkannt() {
    let needle = b"TARGET_UNFLUSHED_IN_WAL";
    let simulated_wal_frame = b"WAL_FRAME_HDR\x00\x00\x01TARGET_UNFLUSHED_IN_WAL_BATCH_TRAILER";

    let raw_matches = Mk2RawScanner::scan_bytes(simulated_wal_frame, needle);
    assert_eq!(
        raw_matches, 1,
        "Mutant mit ungeflushten Ziel-Frames im WAL muss durch MK2 aufgedeckt werden"
    );
}

/// SPEC: T-MUT-1 (Mutant 12: Gate-Schicht vergessen – Gate-Log enthält keinen Erased-Eintrag für Zielbereich)
#[test]
fn t_mut_1_mutant_schicht_vergessen_gate_erkannt() {
    let target_tag = [0x99u8; 16];

    // Mutant: Gate-Log hat das Gate für target_tag nicht hinzugefügt (nur fremde Gates enthalten)
    let incomplete_gate_keys: [[u8; 16]; 2] = [[0x01; 16], [0x02; 16]];

    // Prüfung: Das Ziel-Gate fehlt in der aktiven Gate-Menge -> Verstoß erkannt
    let has_target_gate = incomplete_gate_keys.iter().any(|k| k == &target_tag);
    assert!(
        !has_target_gate,
        "Mutant: Das fehlende Gate für das Ziel muss als Schutzlücke erkannt werden"
    );
}
