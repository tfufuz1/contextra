// FILE-CONTEXT
// ZWECK: Unabhängiger Funktionsnachweis für Garantie 2b: Löschung ist nicht rückholbar und der Nachweis ist extern prüfbar.
// INVARIANTEN: INV-DELETION-1 (Proof-Erstellung erst nach physikalischer Bereinigung), AES-256 KEK-Vernichtung, Ed25519 V3 Audit-Verifikation.

#![forbid(unsafe_code)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_crypto::{
    crypto::KeyManager,
    deletion_proof::{
        DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
        GraphRepairAttestation, LayerCleanupProof,
    },
    ed25519_proof::SignatureVersion,
    error::CryptoError,
    kv_shredding::KeyRegistry,
};
use contextra_types::{DocId, TenantId, TxId};
use ed25519_dalek::Verifier;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::env;
use std::io::Write;
use tempfile::NamedTempFile;

/// Initialisiert eine deterministische RNG mit ausgebbarer/über Umgebungsvariable steuerbarer Seed.
fn get_test_rng() -> (StdRng, u64) {
    let seed = if let Ok(seed_str) = env::var("TEST_SEED") {
        seed_str
            .parse::<u64>()
            .expect("TEST_SEED env var must be a valid u64 integer")
    } else {
        0x5EED_2B00_2026_0930_u64
    };
    println!("[TEST_SEED] Using RNG seed: {seed}");
    (StdRng::seed_from_u64(seed), seed)
}

/// Helper for constructing independent Ed25519 V3 payload according to specification:
/// payload = scope_bytes || deleted_keys_hash (32) || tx_bytes (8) || timestamp_bytes (8)
///           || covered_layers_bytes || excluded_scopes_bytes || graph_repair_bytes || receipt_part || audit_pos_part
fn construct_independent_v3_payload(proof: &DeletionProof) -> Vec<u8> {
    let scope_bytes =
        bincode::serialize(&proof.scope).expect("Serialization of scope must succeed");
    let tx_bytes = proof.deleted_after_tx.0.to_le_bytes();
    let timestamp_bytes = proof.timestamp.to_le_bytes();
    let covered_layers_bytes = bincode::serialize(&proof.covered_layers)
        .expect("Serialization of covered_layers must succeed");
    let excluded_scopes_bytes = bincode::serialize(&proof.excluded_scopes)
        .expect("Serialization of excluded_scopes must succeed");
    let graph_repair_bytes = bincode::serialize(&proof.graph_repair)
        .expect("Serialization of graph_repair must succeed");

    let receipt_bytes = proof.wal_chain_receipt.unwrap_or([0u8; 32]);
    let receipt_part = if proof.wal_chain_receipt.is_some() {
        receipt_bytes.as_slice()
    } else {
        &[]
    };

    let audit_pos_bytes = proof.audit_chain_position.map(|p| p.to_le_bytes());
    let audit_pos_part = if let Some(ref pos_b) = audit_pos_bytes {
        pos_b.as_slice()
    } else {
        &[]
    };

    let mut payload = Vec::with_capacity(
        scope_bytes.len()
            + 32
            + tx_bytes.len()
            + timestamp_bytes.len()
            + covered_layers_bytes.len()
            + excluded_scopes_bytes.len()
            + graph_repair_bytes.len()
            + receipt_part.len()
            + audit_pos_part.len(),
    );
    payload.extend_from_slice(&scope_bytes);
    payload.extend_from_slice(&proof.deleted_keys_hash);
    payload.extend_from_slice(&tx_bytes);
    payload.extend_from_slice(&proof.timestamp.to_le_bytes());
    payload.extend_from_slice(&covered_layers_bytes);
    payload.extend_from_slice(&excluded_scopes_bytes);
    payload.extend_from_slice(&graph_repair_bytes);
    payload.extend_from_slice(receipt_part);
    payload.extend_from_slice(audit_pos_part);

    payload
}

#[test]
fn test_scenario_1_tenant_shredding_and_isolation() {
    let (_rng, seed) = get_test_rng();
    println!("--- SCHRITT 1: Szenario Mandant A vs Mandant B Shredding --- (Seed: {seed})");

    let master_km = KeyManager::try_new("master-passphrase-shredding", b"master-salt-123")
        .expect("Master KeyManager initialization must succeed");
    let registry = KeyRegistry::new();

    let tenant_a_group = 1001u64;
    let tenant_b_group = 2002u64;

    let record_a = 1u64;
    let record_b = 1u64;

    let plaintext_a = b"CONFIDENTIAL DATA FOR TENANT A - TOP SECRET";
    let plaintext_b = b"CONFIDENTIAL DATA FOR TENANT B - KEEP SAFE";

    // Encrypt records for Tenant A and Tenant B
    let payload_a = registry
        .encrypt_record(&master_km, tenant_a_group, record_a, plaintext_a)
        .expect("Encryption for Tenant A must succeed");

    let payload_b = registry
        .encrypt_record(&master_km, tenant_b_group, record_b, plaintext_b)
        .expect("Encryption for Tenant B must succeed");

    // Pre-check: Both records decrypt correctly
    let decrypted_a_before = registry
        .decrypt_record(&master_km, &payload_a)
        .expect("Decryption of Tenant A before revocation must succeed");
    assert_eq!(decrypted_a_before, plaintext_a);

    let decrypted_b_before = registry
        .decrypt_record(&master_km, &payload_b)
        .expect("Decryption of Tenant B before revocation must succeed");
    assert_eq!(decrypted_b_before, plaintext_b);

    assert!(registry.is_group_active(tenant_a_group));
    assert!(registry.is_group_active(tenant_b_group));

    // "Delete" Tenant A (Schlüsselvernichtung über die öffentliche API)
    let revoked = registry.revoke_group(tenant_a_group);
    assert!(
        revoked,
        "Public API revoke_group(tenant_a) must return true for active group"
    );

    // Verifikation: Mandant A ist gelöscht und Entschlüsselung schlägt fehl
    assert!(
        !registry.is_group_active(tenant_a_group),
        "Tenant A group must be marked inactive after revocation"
    );
    assert!(
        registry.get_wrapped_kek(tenant_a_group).is_none(),
        "Tenant A wrapped KEK must not be reachable via public API after revocation"
    );

    let decrypt_a_res = registry.decrypt_record(&master_km, &payload_a);
    assert!(
        decrypt_a_res.is_err(),
        "Decryption of Tenant A payload MUST fail after revocation"
    );
    match decrypt_a_res {
        Err(CryptoError::KeyRevoked(ref msg)) => {
            assert!(
                msg.contains("revoked"),
                "Expected CryptoError::KeyRevoked error message containing 'revoked', got: {msg}"
            );
        }
        res => panic!("Expected CryptoError::KeyRevoked, got: {:?}", res),
    }

    // Verifikation: Mandant B bleibt unberührt und lesbar
    assert!(
        registry.is_group_active(tenant_b_group),
        "Tenant B group must remain active"
    );
    let decrypted_b_after = registry
        .decrypt_record(&master_km, &payload_b)
        .expect("Decryption of Tenant B payload MUST remain fully operational");
    assert_eq!(
        decrypted_b_after, plaintext_b,
        "Tenant B plaintext must match original payload"
    );
}

#[test]
fn test_scenario_2_key_derivation_boundary_and_reconstruction_check() {
    let (_rng, seed) = get_test_rng();
    println!("--- SCHRITT 2: Re-Derivations und Rekonstruktions-Analyse --- (Seed: {seed})");

    /*
     * BEFUND & ARCHITEKTUR-ANALYSE ZUR SCHLÜSSELVERNICHTUNG (AES-256):
     * 1. Die mathematische Unumkehrbarkeit der Löschung beruht auf der Vernichtung von AES-256 Schlüsselmaterial.
     *    Ein 256-Bit Schlüsselraum (2^256 Möglichkeiten) ist gegen Brute-Force-Angriffe selbst mit allen
     *    weltweiten Rechenressourcen unüberwindbar.
     * 2. KeyRegistry nutzt für Gruppen (Shred Groups) 256-Bit KEKs (Group Key Encryption Keys), die per OsRng
     *    rein zufällig erzeugt und im RAM verwaltet werden. Nach `revoke_group(group_id)` wird das Schlüssel-Array
     *    ge-zeroized (ZeroizeOnDrop) und aus der Hashmap entfernt sowie in `revoked_groups` eingetragen.
     * 3. WIEDERHERSTELLUNGSPFAD-PRÜFUNG:
     *    `KeyRegistry::get_or_derive(&master_km, group_id)` prüft explizit `is_group_revoked(group_id)`.
     *    Wenn die Gruppe widerrufen wurde, wird `CryptoError::KeyRevoked` zurückgegeben, d. h. selbst bei erneutem
     *    Aufruf derselben Methode über die KeyRegistry wird kein neuer Schlüssel für die gelöschte Gruppe erzeugt.
     * 4. UNTERSCHIED ZU KvSegmentCipher/HKDF:
     *    `KeyManager::derive_kv_key(tenant_id, model_fingerprint)` ist eine zustandslose HKDF-Sha256-Ableitung.
     *    Für unzerstörbares Crypto-Shredding auf Segment-/Record-Ebene nutzt Contextra die `KeyRegistry`,
     *    welche zufällige KEKs speichert und explizit vernichtet.
     */

    let master_km = KeyManager::try_new("master-passphrase-shredding", b"master-salt-123")
        .expect("Master KeyManager initialization must succeed");
    let registry = KeyRegistry::new();
    let tenant_id = 999u64;

    // Erzeuge initialen Schlüssel für Tenant 999
    let _initial_key = registry
        .get_or_derive(&master_km, tenant_id)
        .expect("Initial get_or_derive must succeed");

    // Löschung / Revocation
    let revoked = registry.revoke_group(tenant_id);
    assert!(revoked, "Revocation of tenant_id 999 must succeed");

    // Versuche Schlüssel nach Löschung neu abzuleiten
    let re_derive_res = registry.get_or_derive(&master_km, tenant_id);
    assert!(
        re_derive_res.is_err(),
        "get_or_derive MUST fail for revoked tenant_id 999"
    );
    match re_derive_res {
        Err(CryptoError::KeyRevoked(ref msg)) => {
            assert!(msg.contains("revoked"));
        }
        res => panic!("Expected CryptoError::KeyRevoked, got: {:?}", res),
    }

    assert!(
        !registry.is_group_active(tenant_id),
        "Tenant group must not be active"
    );
}

#[test]
fn test_scenario_3_deletion_proof_v3_creation_and_exhaustive_mutation_test() {
    let (_rng, seed) = get_test_rng();
    println!("--- SCHRITT 3: DeletionProof v3 Mutationstest (100% Byte-Flips) --- (Seed: {seed})");

    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(777).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id,
    };

    let deleted_keys = vec![b"doc_key_1".to_vec(), b"doc_key_2".to_vec()];
    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap(),
    ];

    let proof = DeletionProof::create_full_v3(
        scope,
        deleted_keys,
        TxId(500),
        cleanup_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        Some([0x33u8; 32]),
        Some(1024),
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .expect("v3 DeletionProof creation must succeed");

    // Positive Verifikation
    assert!(
        proof.verify_external(&keypair.verifying_key).is_ok(),
        "Original intact proof MUST pass verify_external"
    );

    // 1. Mutationstest der Signatur-Bytes (jedes einzelne Byte der 64-Byte Ed25519 Signatur flippen)
    assert_eq!(proof.signature.len(), 64);
    for i in 0..proof.signature.len() {
        let mut tampered = proof.clone();
        tampered.signature[i] ^= 0x01; // Byte flippen
        let res = tampered.verify_external(&keypair.verifying_key);
        assert!(
            res.is_err(),
            "Flipping signature byte at index {i} MUST cause verify_external to fail"
        );
    }

    // 2. Mutationstest des deleted_keys_hash (jedes der 32 Bytes flippen)
    for i in 0..proof.deleted_keys_hash.len() {
        let mut tampered = proof.clone();
        tampered.deleted_keys_hash[i] ^= 0x01;
        let res = tampered.verify_external(&keypair.verifying_key);
        assert!(
            res.is_err(),
            "Flipping deleted_keys_hash byte at index {i} MUST cause verify_external to fail"
        );
    }

    // 3. Mutationstest des JSON-Exports (jedes einzelne Zeichen/Byte des exportierten JSON-Strings verändern)
    let json_export = proof
        .export_for_audit()
        .expect("export_for_audit must succeed for v3 proof");

    let json_bytes = json_export.as_bytes();
    println!(
        "Performing exhaustive byte mutation test on JSON export (total bytes: {})",
        json_bytes.len()
    );

    let mut rejected_mutations = 0;
    for i in 0..json_bytes.len() {
        let mut mutated_bytes = json_bytes.to_vec();
        mutated_bytes[i] ^= 0x01;

        // Entweder schlägt das Serde-Parsing fehl ODER die anschließende verify_external schlägt fehl.
        // Es darf NIEMALS Ok(()) aus der Verifikation eines mutierten JSON hervorgehen!
        if let Ok(deserialized_proof) = serde_json::from_slice::<DeletionProof>(&mutated_bytes) {
            let verify_res = deserialized_proof.verify_external(&keypair.verifying_key);
            assert!(
                verify_res.is_err(),
                "Mutated JSON at byte index {i} deserialized but MUST fail verify_external"
            );
        }
        rejected_mutations += 1;
    }
    assert_eq!(
        rejected_mutations,
        json_bytes.len(),
        "All JSON byte mutations were tested and rejected"
    );
}

#[test]
fn test_repro_bug_graph_repair_omitted_from_v3_signature() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(777).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id,
    };

    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
    ];

    let graph_repair = vec![GraphRepairAttestation {
        doc_id: DocId(42),
        verified_no_ghost_pointers: true,
        attested_at: 1700000000,
    }];

    let proof = DeletionProof::create_full_v3(
        scope,
        vec![b"k1".to_vec()],
        TxId(100),
        cleanup_proofs,
        vec![],
        None,
        None,
        1700000000,
        &graph_repair,
        keypair.signing_key(),
    )
    .unwrap();

    assert!(proof.verify_external(&keypair.verifying_key).is_ok());

    let mut tampered_proof = proof.clone();
    tampered_proof.graph_repair[0].verified_no_ghost_pointers = false;
    tampered_proof.graph_repair[0].attested_at = 9999999999;

    let verify_res = tampered_proof.verify_external(&keypair.verifying_key);
    assert!(
        verify_res.is_err(),
        "BUG REPRO: Mutated graph_repair must fail signature verification, but construct_v3_payload omitted graph_repair!"
    );
}

#[test]
fn test_scenario_4_independent_external_verification_and_tempfile() {
    let (_rng, seed) = get_test_rng();
    println!("--- SCHRITT 4: Unabhängige Verifikation & Tempfile Export --- (Seed: {seed})");

    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(888).unwrap();
    let scope = DeletionScope::Tenant { tenant_id };

    let deleted_keys = vec![b"tenant_wide_key_a".to_vec(), b"tenant_wide_key_b".to_vec()];
    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(
            DeletionLayer::WalAllSegments { seq_after: 100 },
            0,
        )
        .unwrap(),
    ];

    let proof = DeletionProof::create_full_v3(
        scope,
        deleted_keys,
        TxId(12345),
        cleanup_proofs,
        vec![
            ExcludedScope::LlmParameterMemory,
            ExcludedScope::ConsolidatedAndDistilled,
        ],
        Some([0x99u8; 32]),
        Some(42),
        1700001000,
        &[],
        keypair.signing_key(),
    )
    .expect("v3 proof creation must succeed");

    // 1. Verifikation über contextra-crypto API
    let internal_verify_res = proof.verify_external(&keypair.verifying_key);
    assert!(
        internal_verify_res.is_ok(),
        "verify_external must succeed on original proof"
    );

    // 2. UNABHÄNGIGE PRÜFUNG: Konstruiere den Signatur-Payload OHNE contextra-crypto Verifikationscode
    // nur mit ed25519_dalek::VerifyingKey::verify direkt auf den rekonstruierten Bytes!
    let independent_payload = construct_independent_v3_payload(&proof);
    let ed25519_signature = ed25519_dalek::Signature::from_slice(&proof.signature)
        .expect("Signature bytes must parse into valid ed25519_dalek::Signature");

    let independent_verify_res = keypair
        .verifying_key
        .verify(&independent_payload, &ed25519_signature);

    assert!(
        independent_verify_res.is_ok(),
        "Independent ed25519_dalek verification MUST succeed for valid proof"
    );

    // Beide Ergebnisse stimmen überein
    assert_eq!(
        internal_verify_res.is_ok(),
        independent_verify_res.is_ok(),
        "Internal verify_external and independent ed25519_dalek verification MUST yield identical results"
    );

    // 3. Schreiben des Audit-Exports in eine Tempfile und Ausgabe des Pfads für externe Prüfung (z. B. Python)
    let json_export = proof
        .export_for_audit()
        .expect("Audit JSON export must succeed");

    let mut temp_file = NamedTempFile::new().expect("Creation of NamedTempFile must succeed");
    temp_file
        .write_all(json_export.as_bytes())
        .expect("Writing JSON to tempfile must succeed");

    let temp_path = temp_file.path().to_path_buf();
    println!(
        ">>> AUDIT PROOF TEMPFILE CREATED FOR EXTERNAL INSPECTION: {} <<<",
        temp_path.display()
    );

    // Datei nach dem Schreiben wieder einlesen und Integrität prüfen
    let read_back_json =
        std::fs::read_to_string(&temp_path).expect("Read-back from tempfile must succeed");
    let parsed_back: DeletionProof =
        serde_json::from_str(&read_back_json).expect("Parsing read-back JSON must succeed");
    assert_eq!(parsed_back, proof);
}

#[test]
fn test_scenario_5_replay_cross_tenant_and_downgrade_attacks() {
    let (_rng, seed) = get_test_rng();
    println!("--- SCHRITT 5: Replay- & Downgrade-Angriffe --- (Seed: {seed})");

    let keypair = DeletionProofKeyPair::generate();

    // Proof erzeugen für Mandant A (TenantId 100)
    let tenant_a = TenantId::try_new(100).unwrap();
    let scope_a = DeletionScope::Document {
        doc_id: DocId(10),
        tenant_id: tenant_a,
    };

    let proof_a = DeletionProof::create_v3(
        scope_a,
        vec![b"key_a".to_vec()],
        TxId(10),
        vec![],
        vec![],
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    assert!(proof_a.verify_external(&keypair.verifying_key).is_ok());

    // REPLAY / VERWECHSLUNGSANGRIFF: Proof von Mandant A gegen Mandant B verifizieren
    let tenant_b = TenantId::try_new(200).unwrap();
    let scope_b = DeletionScope::Document {
        doc_id: DocId(10),
        tenant_id: tenant_b,
    };

    let mut replayed_proof = proof_a.clone();
    replayed_proof.scope = scope_b; // Versuche Scope im Proof auszutauschen

    let replay_verify_res = replayed_proof.verify_external(&keypair.verifying_key);
    assert!(
        replay_verify_res.is_err(),
        "Replay attack (substituting Scope A with Scope B) MUST fail signature verification"
    );
    assert!(matches!(
        replay_verify_res,
        Err(CryptoError::InvalidProofSignature)
    ));

    // DOWNGRADE-ANGRIFFE:
    // 1. Signature Version auf Version 1 (HMAC legacy) abändern
    let mut downgrade_v1 = proof_a.clone();
    downgrade_v1.signature_version = SignatureVersion::V1.as_u8();

    let downgrade_v1_res = downgrade_v1.verify_external(&keypair.verifying_key);
    assert!(
        matches!(
            downgrade_v1_res,
            Err(CryptoError::UnsupportedProofVersion(1))
        ),
        "verify_external MUST reject downgraded signature_version 1 with UnsupportedProofVersion"
    );

    // 2. Signature Version auf Version 2 (HMAC v2) abändern
    let mut downgrade_v2 = proof_a.clone();
    downgrade_v2.signature_version = SignatureVersion::V2.as_u8();

    let downgrade_v2_res = downgrade_v2.verify_external(&keypair.verifying_key);
    assert!(
        matches!(
            downgrade_v2_res,
            Err(CryptoError::UnsupportedProofVersion(2))
        ),
        "verify_external MUST reject downgraded signature_version 2 with UnsupportedProofVersion"
    );

    // 3. Signature Version auf ungültigen Wert 255 abändern
    let mut invalid_version = proof_a.clone();
    invalid_version.signature_version = 255;

    let invalid_version_res = invalid_version.verify_external(&keypair.verifying_key);
    assert!(
        matches!(
            invalid_version_res,
            Err(CryptoError::UnsupportedProofVersion(255))
        ),
        "verify_external MUST reject invalid signature_version 255 with UnsupportedProofVersion"
    );
}
