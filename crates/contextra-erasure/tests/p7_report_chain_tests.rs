#![cfg(feature = "signing")]

//! Spezifikationstests für P7 (WP-9, INV-ERA-7, INV-CLASS-MEASURED-1, INV-CLAIM-BIND-1).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::report::build::{
    build_report, build_report_with_options, try_build_report_with_options, LedgerConfirmation,
    ReportBuildOptions,
};
use contextra_erasure::report::canonical::to_canonical_json_string;
use contextra_erasure::report::signing::ReportSigner;
use contextra_erasure::types_local::{
    ControlEvidence, ErasureId, LayerId, LayerWitness, MeasureClass, Scanner,
};
use ed25519_dalek::SigningKey;
use std::time::Duration;

fn make_valid_witness(layer_byte: u8, method_id: u16, hash_byte: u8) -> LayerWitness {
    Scanner::new(LayerId::new([layer_byte; 16]), MeasureClass::Mk1, method_id)
        .arm(ControlEvidence::observed())
        .finish(0, Duration::from_micros(10_000), [hash_byte; 32])
        .expect("witness must be created successfully")
}

/// Invariante INV-ERA-7: Ein Löschbericht bindet die Zeugen, auf die er sich stützt (Hash-Kette).
#[test]
fn inv_era_7_report_binds_witness_chain() {
    let erasure_id = ErasureId::new([0x77; 16]);
    let w1 = make_valid_witness(1, 10, 0x11);
    let w2 = make_valid_witness(2, 20, 0x22);
    let w3 = make_valid_witness(3, 30, 0x33);

    let witnesses = vec![w1.clone(), w2.clone(), w3.clone()];
    let report = build_report(erasure_id, 1000, 2000, witnesses, 5, 2048, true);

    // Kette muss genau 3 Digests und einen nicht-leeren Wurzel-Digest aufweisen
    assert_eq!(report.witnesses.len(), 3);
    assert_eq!(report.witness_chain.witness_digests.len(), 3);
    assert_ne!(
        report.witness_chain.chain_root,
        "0000000000000000000000000000000000000000000000000000000000000000"
    );

    let signing_key = SigningKey::from_bytes(&[0x88; 32]);
    let signed = ReportSigner::sign_report(&report, &signing_key).expect("signing succeeds");

    let verified_report = signed
        .verify_report_with_chain(&signing_key.verifying_key())
        .expect("verification with valid witness chain must succeed");
    assert_eq!(verified_report.erasure_id, report.erasure_id);
}

/// Sabotagetest: Entfernte, umgeordnete oder manipulierte Zeugen werden abgewiesen.
#[test]
fn tamper_removed_reordered_modified_witness_rejected() {
    let erasure_id = ErasureId::new([0x88; 16]);
    let w1 = make_valid_witness(1, 10, 0x11);
    let w2 = make_valid_witness(2, 20, 0x22);
    let w3 = make_valid_witness(3, 30, 0x33);

    let witnesses = vec![w1, w2, w3];
    let report = build_report(erasure_id, 1000, 2000, witnesses, 5, 2048, true);
    let signing_key = SigningKey::from_bytes(&[0x99; 32]);
    let signed = ReportSigner::sign_report(&report, &signing_key).expect("signing succeeds");

    // 1. Unveränderter Bericht verifiziert
    assert!(signed
        .verify_report_with_chain(&signing_key.verifying_key())
        .is_ok());

    // 2. Zeuge modifiziert im kanonischen Body
    let mut manipulated_body = signed.clone();
    manipulated_body.canonical_body = manipulated_body.canonical_body.replace(
        "\"method_id\":\"mk_scan_v10\"",
        "\"method_id\":\"mk_scan_v99\"",
    );
    // Signatur bricht
    assert!(manipulated_body
        .verify_report_with_chain(&signing_key.verifying_key())
        .is_err());

    // 3. Zeuge neu signiert, aber Kette manipuliert (z. B. ein Digest verändert)
    let mut bad_report = report.clone();
    if let Some(first_digest) = bad_report.witness_chain.witness_digests.get_mut(0) {
        *first_digest =
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
    }
    let bad_signed =
        ReportSigner::sign_report(&bad_report, &signing_key).expect("signing succeeds");
    assert!(bad_signed
        .verify_report_with_chain(&signing_key.verifying_key())
        .is_err());

    // 4. Zeugen umgeordnet (Reordered)
    let mut reordered_report = report.clone();
    reordered_report.witnesses.swap(0, 1);
    reordered_report.witness_chain.witness_digests.swap(0, 1);
    // Chain-Root bleibt die alte -> Recomputed chain_root passt nicht mehr
    let reordered_signed =
        ReportSigner::sign_report(&reordered_report, &signing_key).expect("signing succeeds");
    assert!(reordered_signed
        .verify_report_with_chain(&signing_key.verifying_key())
        .is_err());

    // 5. Zeuge entfernt (Removed)
    let mut removed_report = report.clone();
    removed_report.witnesses.pop();
    removed_report.witness_chain.witness_digests.pop();
    let removed_signed =
        ReportSigner::sign_report(&removed_report, &signing_key).expect("signing succeeds");
    assert!(removed_signed
        .verify_report_with_chain(&signing_key.verifying_key())
        .is_err());
}

/// Kanonische Form ist deterministisch und stimmt exakt mit Testvektoren überein.
#[test]
fn canonical_form_is_deterministic() {
    let erasure_id = ErasureId::new([0x42; 16]);
    let w1 = Scanner::new(LayerId::new([1; 16]), MeasureClass::Mk1, 1)
        .arm(ControlEvidence::observed())
        .finish(0, Duration::from_micros(10_000), [0xaa; 32])
        .expect("witness 1");
    let w2 = Scanner::new(LayerId::new([2; 16]), MeasureClass::Mk2, 2)
        .arm(ControlEvidence::observed())
        .finish(0, Duration::from_micros(20_000), [0xbb; 32])
        .expect("witness 2");

    let opts = ReportBuildOptions {
        store_id: "store_p7_fixed".to_string(),
        target_type: "Document".to_string(),
        commitment: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        requested_at: 1000,
        gated_at: 1050,
        key_destroyed_at: 1100,
        completed_at: 1200,
        sla_us: 30000000,
        sla_violation: false,
        retention_code: "NoObligation".to_string(),
        lab_evidence: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        vault_medium: "flash".to_string(),
        vault_rehash_count: 0,
        vault_old_overwritten: true,
        vault_backup_scope: "Unknown".to_string(),
        verified_via: "Direct".to_string(),
        epoch_id: "0000000000000000".to_string(),
        replaced: false,
        replacement_tag: None,
        signer_id: 0,
        process_hardening: "Full".to_string(),
        risk_context: Default::default(),
        ledger_confirmation: Some(LedgerConfirmation::new(42, [0xcc; 32])),
    };

    let report = build_report_with_options(erasure_id, vec![w1, w2], 2, 4096, opts);
    let canonical = to_canonical_json_string(&report).expect("canonical serialization");
    let hash = blake3::hash(canonical.as_bytes()).to_hex().to_string();

    let expected_canonical = std::fs::read_to_string("tests/p7_vectors/sample_report_v1.canonical")
        .expect("sample_report_v1.canonical must exist");
    let expected_hash = std::fs::read_to_string("tests/p7_vectors/sample_report_v1.hash")
        .expect("sample_report_v1.hash must exist");

    assert_eq!(canonical, expected_canonical.trim());
    assert_eq!(hash, expected_hash.trim());
}

/// Invariante INV-CLASS-MEASURED-1: Kein Bericht ohne verifizierte Scanner-Zeugen.
#[test]
fn report_without_verified_witness_refused() {
    let erasure_id = ErasureId::new([0x33; 16]);

    // 1. Leere Zeugenliste -> try_build_report_with_options verweigert
    let res_empty =
        try_build_report_with_options(erasure_id, vec![], 0, 0, ReportBuildOptions::default());
    assert!(
        res_empty.is_err(),
        "Report creation without witnesses MUST fail fail-closed"
    );

    // 2. Zeuge mit unvollständigem Scan-Hash ([0u8; 32])
    let unverified_witness = LayerWitness::empty();
    let res_unverified = try_build_report_with_options(
        erasure_id,
        vec![unverified_witness],
        0,
        0,
        ReportBuildOptions::default(),
    );
    assert!(
        res_unverified.is_err(),
        "Report creation with unverified empty witness MUST fail"
    );

    // 3. Verifizierter Zeuge -> Erfolg
    let valid_witness = make_valid_witness(1, 1, 0x55);
    let res_valid = try_build_report_with_options(
        erasure_id,
        vec![valid_witness],
        0,
        0,
        ReportBuildOptions::default(),
    );
    assert!(
        res_valid.is_ok(),
        "Report creation with verified witness must succeed"
    );
}

/// Invariante INV-CLAIM-BIND-1: Berichtstext enthält keine rechtsverbindlichen Garantieformulierungen.
#[test]
fn report_text_contains_no_binding_guarantee_phrases() {
    let erasure_id = ErasureId::new([0x55; 16]);
    let w = make_valid_witness(1, 1, 0x11);
    let report = build_report(erasure_id, 1000, 2000, vec![w], 1, 512, true);
    let canonical = to_canonical_json_string(&report).expect("canonical json");

    // Liste verbotener Phrasen gemäß SPEC §6.4 und INV-CLAIM-BIND-1
    let forbidden_phrases = [
        "garantiert",
        "guaranteed",
        "100 %",
        "100%",
        "DSGVO-konform",
        "GDPR-compliant",
        "rechtskonform",
        "Rechtsberatung obliegt nicht",
    ];

    for phrase in &forbidden_phrases {
        assert!(
            !canonical.to_lowercase().contains(&phrase.to_lowercase()),
            "Report JSON contains forbidden guarantee phrase: '{}'",
            phrase
        );
    }

    // Explizite Prüfung der Pflichttexte auf neutrale Formulierungen
    assert!(report
        .legal_notice
        .contains("Keine Rechtsberatung; die rechtliche Wertung obliegt dem Betreiber."));
    assert!(report
        .signature_scope_notice
        .contains("nicht die Wahrheit des Inhalts"));
}

/// Regressionstest: Externer Verify-Test für DeletionProof bleibt grün.
#[test]
fn existing_external_verify_test_still_green() {
    let keypair = contextra_erasure::deletion_proof::DeletionProofKeyPair::generate();
    let tenant_id = contextra_erasure::types_local::TenantId(1);
    let scope = contextra_erasure::deletion_proof::DeletionScope::Document {
        doc_id: contextra_erasure::types_local::DocId(42),
        tenant_id,
    };
    #[allow(deprecated)]
    let layer_proof =
        contextra_erasure::deletion_proof::LayerCleanupProof::new_after_verified_empty(
            contextra_erasure::deletion_proof::DeletionLayer::LsmMemtable,
            0,
        )
        .unwrap();

    let proof = contextra_erasure::deletion_proof::DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec(), b"key2".to_vec()],
        contextra_erasure::types_local::TxId::new(100),
        vec![layer_proof],
        vec![contextra_erasure::deletion_proof::ExcludedScope::LlmParameterMemory],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .expect("Failed to create valid v3 proof");

    assert!(proof.verify_external(&keypair.verifying_key).is_ok());
}
