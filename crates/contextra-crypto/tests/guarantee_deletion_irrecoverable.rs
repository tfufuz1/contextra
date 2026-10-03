// FILE-CONTEXT
// ZWECK: Unabhängiger Funktionsnachweis für Garantie 2b (Löschung ist nicht rückholbar, Proof extern prüfbar).
// SPEZIFIKATION: AUFTRAG 3 / Spec v17 §16.1 & §10.1 / DSGVO Art. 17 Compliance.
// INVARIANTEN:
// 1. Nach Vernichtung des Mandanten-/Segment-Schlüssels in KeyRegistry ist der Klartext nicht mehr ableitbar.
// 2. Ein DeletionProof v3 ist ohne Vertrauen in Contextra-Interna verifizierbar und bei jeder Mutation ungültig.
// 3. Replay- und Downgrade-Angriffe werden strikt fail-closed abgelehnt.

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_crypto::{
    kv_cipher::ModelFingerprint, kv_shredding::KeyRegistry, CryptoError, DeletionLayer,
    DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, KeyManager,
    LayerCleanupProof,
};
use contextra_types::{DocId, TenantId, TxId};
use ed25519_dalek::{Signer, Verifier, VerifyingKey};
use hkdf::Hkdf;
use sha2::Sha256;
use std::env;
use std::io::Write;

/// Multi-source seed generator for deterministic test execution.
fn init_test_seed() -> u64 {
    let seed = env::var("CONTEXTRA_TEST_SEED")
        .or_else(|_| env::var("SEED"))
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0xCAFE_BABE_1234_5678)
        });

    println!("CONTEXTRA_TEST_SEED={seed}");
    seed
}

// ============================================================================
// RULE 2: INDEPENDENCE & RFC REFERENCE VECTORS
// ============================================================================

/// Verifies RFC 8032 Section 7.1 Test Vector 1 (Ed25519 signature verification).
/// Source: https://datatracker.ietf.org/doc/html/rfc8032#section-7.1
#[test]
fn test_rfc_8032_ed25519_reference_vector_1() {
    init_test_seed();

    let secret_bytes: [u8; 32] = [
        0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c,
        0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae,
        0x7f, 0x60,
    ];

    let expected_pub_bytes: [u8; 32] = [
        0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07,
        0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07,
        0x51, 0x1a,
    ];

    let expected_sig_bytes: [u8; 64] = [
        0xe5, 0x56, 0x43, 0x00, 0xc3, 0x60, 0xac, 0x72, 0x90, 0x86, 0xe2, 0xcc, 0x80, 0x6e, 0x82,
        0x8a, 0x84, 0x87, 0x7f, 0x1e, 0xb8, 0xe5, 0xd9, 0x74, 0xd8, 0x73, 0xe0, 0x65, 0x22, 0x49,
        0x01, 0x55, 0x5f, 0xb8, 0x82, 0x15, 0x90, 0xa3, 0x3b, 0xac, 0xc6, 0x1e, 0x39, 0x70, 0x1c,
        0xf9, 0xb4, 0x6b, 0xd2, 0x5b, 0xf5, 0xf0, 0x59, 0x5b, 0xbe, 0x24, 0x65, 0x51, 0x41, 0x43,
        0x8e, 0x7a, 0x10, 0x0b,
    ];

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    assert_eq!(
        verifying_key.to_bytes(),
        expected_pub_bytes,
        "RFC 8032 public key derivation mismatch"
    );

    let message: &[u8] = b"";
    let sig = signing_key.sign(message);
    assert_eq!(
        sig.to_bytes(),
        expected_sig_bytes,
        "RFC 8032 signature generation mismatch"
    );

    assert!(
        verifying_key.verify(message, &sig).is_ok(),
        "RFC 8032 signature verification failed"
    );
}

/// Verifies RFC 5869 Section 3 Test Case 1 (HKDF-SHA256 reference vector).
/// Source: https://datatracker.ietf.org/doc/html/rfc5869#section-3
#[test]
fn test_rfc_5869_hkdf_sha256_reference_vector_1() {
    init_test_seed();

    let ikm = [0x0b; 22];
    let salt = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
    ];
    let info = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
    let expected_okm = [
        0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, 0xd0, 0x36, 0x2f,
        0x2a, 0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, 0x5d, 0xb0, 0x2d, 0x56, 0xec, 0xc4,
        0xc5, 0xbf, 0x34, 0x00, 0x72, 0x08, 0xd5, 0xb8, 0x87, 0x18, 0x58, 0x65,
    ];

    let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
    let mut okm = [0u8; 42];
    hk.expand(&info, &mut okm)
        .expect("RFC 5869 HKDF expansion failed");

    assert_eq!(
        okm, expected_okm,
        "RFC 5869 HKDF-SHA256 test case 1 output mismatch"
    );
}

// ============================================================================
// SCENARIO 1: MANDANT A & B VERSCHLÜSSELN + MANDANT A SCHLÜSSELVERNICHTUNG
// ============================================================================

/// Scenario 1:
/// - Mandant A (group 100) und Mandant B (group 200) verschlüsseln Daten via Envelope-Encryption.
/// - Mandant A wird gelöscht (Schlüsselvernichtung `revoke_group(100)`).
/// - Entschlüsselung von A schlägt mit `CryptoError::KeyRevoked` fehl.
/// - Mandant B bleibt unbeeinträchtigt lesbar.
/// - Über die öffentliche API ist für Mandant A kein Schlüsselmaterial mehr erreichbar.
#[test]
fn test_scenario_1_shredding_and_isolation() {
    init_test_seed();

    let master_km = KeyManager::try_new("master-passphrase-scenario-1", b"master-salt-1").unwrap();
    let registry = KeyRegistry::new();

    let group_a = 100u64;
    let group_b = 200u64;

    let payload_a_1 = registry
        .encrypt_record(
            &master_km,
            group_a,
            1,
            b"Mandant A vertraulicher Datensatz 1",
        )
        .unwrap();
    let payload_a_2 = registry
        .encrypt_record(
            &master_km,
            group_a,
            2,
            b"Mandant A vertraulicher Datensatz 2",
        )
        .unwrap();
    let payload_b_1 = registry
        .encrypt_record(
            &master_km,
            group_b,
            1,
            b"Mandant B vertraulicher Datensatz 1",
        )
        .unwrap();

    // Vor der Löschung: Alle Datensätze lesbar
    assert_eq!(
        registry.decrypt_record(&master_km, &payload_a_1).unwrap(),
        b"Mandant A vertraulicher Datensatz 1"
    );
    assert_eq!(
        registry.decrypt_record(&master_km, &payload_a_2).unwrap(),
        b"Mandant A vertraulicher Datensatz 2"
    );
    assert_eq!(
        registry.decrypt_record(&master_km, &payload_b_1).unwrap(),
        b"Mandant B vertraulicher Datensatz 1"
    );

    assert!(registry.is_group_active(group_a));
    assert!(registry.is_group_active(group_b));

    // ACT: Mandant A "löschen" (Schlüsselvernichtung über die öffentliche API)
    let revoked = registry.revoke_group(group_a);
    assert!(revoked, "revoke_group(100) muss true zurückgeben");

    // ASSERT: Entschlüsselung von Mandant A schlägt mit CryptoError::KeyRevoked fehl
    let res_a_1 = registry.decrypt_record(&master_km, &payload_a_1);
    assert!(
        matches!(res_a_1, Err(CryptoError::KeyRevoked(ref msg)) if msg.contains("100")),
        "Entschlüsselung von Mandant A muss mit KeyRevoked fehlschlagen, got: {res_a_1:?}"
    );

    let res_a_2 = registry.decrypt_record(&master_km, &payload_a_2);
    assert!(
        matches!(res_a_2, Err(CryptoError::KeyRevoked(_))),
        "Entschlüsselung von Mandant A Datensatz 2 muss mit KeyRevoked fehlschlagen"
    );

    // ASSERT: Mandant B bleibt vollständig lesbar
    assert_eq!(
        registry.decrypt_record(&master_km, &payload_b_1).unwrap(),
        b"Mandant B vertraulicher Datensatz 1",
        "Mandant B muss nach Löschung von Mandant A unberührt lesbar bleiben"
    );

    // ASSERT: Zustand der öffentlichen API nach der Löschung
    assert!(
        !registry.is_group_active(group_a),
        "Mandant A Gruppe darf nach Revocation nicht mehr aktiv sein"
    );
    assert!(
        registry.is_group_revoked(group_a),
        "is_group_revoked(100) muss true sein"
    );
    assert!(
        registry.get_wrapped_kek(group_a).is_none(),
        "get_wrapped_kek für Gruppe A muss None zurückgeben"
    );
    assert!(
        registry.get_wrapped_dek(group_a, 1).is_none(),
        "get_wrapped_dek für Mandant A Datensatz 1 muss None zurückgeben"
    );

    // Versuch der erneuten Schlüsselableitung über get_or_derive schlägt fehl
    let res_rederive = registry.get_or_derive(&master_km, group_a);
    assert!(
        matches!(res_rederive, Err(CryptoError::KeyRevoked(_))),
        "get_or_derive für widerrufene Gruppe muss KeyRevoked zurückgeben"
    );
}

// ============================================================================
// SCENARIO 2: SCHLÜSSELABLEITUNGS-PFAD ANALYSE & BRUTE-FORCE GEGENPROBE
// ============================================================================

/// Scenario 2:
/// - Dokumentation: AES-256 hat einen Schlüsselraum von 2^256 (~1.15 x 10^77 Schlüssel).
///   Ein Brute-Force-Angriff ist physikalisch unmöglich (benötigt mehr Energie als das beobachtbare Universum).
///   Die Sicherheit beruht somit mathematisch auf der Vernichtung des 256-Bit-Schlüssels.
/// - Test zweiter Ableitungspfade:
///   1) `KeyRegistry` Path: Generiert pro Gruppe einen echten Zufallsschlüssel (`OsRng`). Nach `revoke_group`
///      ist dieser Zufallsschlüssel gelöscht. Selbst mit demselben Master-Key erzeugt eine frische `KeyRegistry`
///      für dieselbe Group-ID einen NEUEN Zufallsschlüssel — der alte Ciphertext bleibt mathematisch unlesbar.
///   2) `KeyManager::derive_kv_key` HKDF Path: Bietet eine rein deterministische Schlüsselableitung aus
///      (Master-Key, Salt, TenantID, ModelFingerprint).
///
/// BEFUND DOKUMENTATION (Kernbefund):
/// Da HKDF eine reine mathematische Funktion f(MasterKey, Salt, TenantID) ist, liefert `derive_kv_key`
/// vor und nach einer logischen "Löschung" exakt denselben 256-Bit Schlüssel, solange MasterKey und Salt
/// unverändert bleiben! Eine echte, unumkehrbare kryptographische Löschung auf HKDF-Ebene erfordert
/// zwingend die Vernichtung oder Rotation des Master-Keys / Salts (Master Key Shredding / Salt Shredding)
/// oder die Nutzung der zufallsbasierten `KeyRegistry` Envelope-Encryption Path.
#[test]
fn test_scenario_2_key_derivation_analysis_and_brute_force_counter_proof() {
    init_test_seed();

    let master_km = KeyManager::try_new("master-passphrase-scenario-2", b"master-salt-2").unwrap();

    // ------------------------------------------------------------------------
    // PATH 1: KeyRegistry (Random KEK Envelope Encryption Shredding)
    // ------------------------------------------------------------------------
    let registry1 = KeyRegistry::new();
    let group_id = 42u64;

    // Subkey vor Löschung
    let subkey_orig = registry1.get_or_derive(&master_km, group_id).unwrap();

    // Encrypt payload under registry1
    let (ct, nonce) = registry1
        .encrypt_with_group(&master_km, group_id, b"Garantie 2b Testdaten")
        .unwrap();

    // Revoke in registry1
    registry1.revoke_group(group_id);

    // Eine zweite Instanz der KeyRegistry mit demselben Master-Key
    let registry2 = KeyRegistry::new();
    let subkey_new = registry2.get_or_derive(&master_km, group_id).unwrap();

    // ASSERT: Der neu abgeleitete KEK in registry2 unterscheidet sich vom gelöschten subkey_orig,
    // da get_or_derive bei frischen Gruppen kryptographischen Zufall (OsRng) verwendet!
    assert_ne!(
        subkey_orig.0, subkey_new.0,
        "KeyRegistry KEKs müssen dank OsRng-Zufall bei jeder Instanziierung unterschiedlich sein"
    );

    // Versuch, den alten Ciphertext mit der neuen Registry2 zu entschlüsseln schlägt fehl (AES-GCM Auth Tag Failure)
    let decrypt_res = registry2.decrypt_with_group(group_id, &ct, &nonce);
    assert!(
        decrypt_res.is_err(),
        "Entschlüsselung mit neu generiertem KEK muss fehlschlagen"
    );

    // ------------------------------------------------------------------------
    // PATH 2: HKDF Path (KeyManager::derive_kv_key) — ANALYSIS & FINDING
    // ------------------------------------------------------------------------
    let tenant_id = TenantId::try_new(101).unwrap();
    let fp = ModelFingerprint::new([0xAAu8; 32], "test-model", "Q4_0");

    let sub_km_before = master_km.derive_kv_key(tenant_id, &fp).unwrap();
    let sub_km_after = master_km.derive_kv_key(tenant_id, &fp).unwrap();

    // BEFUND: HKDF ist eine reine, zustandslose mathematische Funktion.
    // derive_kv_key(master, tenant) liefert deterministisch stets denselben Schlüssel!
    assert_eq!(
        sub_km_before.inspect_key_bytes_for_test(),
        sub_km_after.inspect_key_bytes_for_test(),
        "BEFUND: HKDF derive_kv_key liefert nach der 'Löschung' denselben Schlüssel, wenn Master-Key/Salt erhalten bleiben."
    );

    // HINWEIS FÜR PR-REPORT / BEFUND:
    // "BEFUND: Für den HKDF-basierten Ableitungspfad (derive_kv_key) ist eine unumkehrbare
    // Löschung nur dann gegeben, wenn der Master-Key oder der Salt vernichtet/rotiert werden.
    // Für Einzellöschungen auf Mandanten-Ebene bietet die KeyRegistry (Envelope-Encryption)
    // durch zufallsgenerierte KEKs eine O(1)-Vernichtungsgarantie ohne Master-Key-Rotation."
}

// ============================================================================
// SCENARIO 3: DELETION PROOF EXPORT & COMPREHENSIVE SINGLE-BYTE MUTATION
// ============================================================================

/// Scenario 3:
/// - DeletionProof v3 mit Ed25519-Signatur, WAL-Receipt und Audit-Position erzeugen.
/// - Export via `export_for_audit()` und Prüfung mit `verify_external()`.
/// - Mutationstest: JEDES EINZELNE BYTE des exportierten JSON / der Signatur flippen (alle Positionen 0..len).
/// - ASSERT: Jede Mutation führt zu Verifikationsfehler oder JSON-Parsingfehler, NIEMALS zu Ok(()) oder true.
#[test]
fn test_scenario_3_deletion_proof_export_and_exhaustive_mutation() {
    init_test_seed();

    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0).unwrap(),
    ];

    let proof = DeletionProof::create_full_v3(
        scope,
        vec![b"doc_key_1".to_vec(), b"doc_key_2".to_vec()],
        TxId(1001),
        cleanup_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        Some([0x33u8; 32]), // WAL receipt
        Some(42),           // Audit chain position
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    // Assert base proof is valid
    assert_eq!(proof.signature_version, 3);
    assert!(
        proof.verify_external(&keypair.verifying_key).is_ok(),
        "Original v3 proof must verify successfully with verify_external"
    );

    let json_export = proof.export_for_audit().unwrap();
    assert!(json_export.contains("\"signature_version\": 3"));
    assert!(json_export.contains("LlmParameterMemory"));

    // ------------------------------------------------------------------------
    // MUTATION TEST 1: Flipping every single byte of the exported JSON string
    // ------------------------------------------------------------------------
    let json_bytes = json_export.as_bytes();
    let total_json_len = json_bytes.len();

    let mut parsing_failures = 0;
    let mut verification_failures = 0;

    for i in 0..total_json_len {
        let mut mutated_bytes = json_bytes.to_vec();
        mutated_bytes[i] ^= 0x01; // Flip lowest bit at position i

        match serde_json::from_slice::<DeletionProof>(&mutated_bytes) {
            Err(_) => {
                parsing_failures += 1;
            }
            Ok(mutated_proof) => {
                let verify_res = mutated_proof.verify_external(&keypair.verifying_key);
                assert!(
                    verify_res.is_err(),
                    "CRITICAL SECURITY FAILURE: Mutated JSON at byte offset {i} passed verify_external!"
                );
                verification_failures += 1;
            }
        }
    }

    println!(
        "JSON Exhaustive Mutation Test: {total_json_len} positions tested ({parsing_failures} parse errors, {verification_failures} signature verification rejections, 0 false acceptances)"
    );

    // ------------------------------------------------------------------------
    // MUTATION TEST 2: Flipping every single bit/byte of the raw signature
    // ------------------------------------------------------------------------
    let sig_len = proof.signature.len();
    assert_eq!(sig_len, 64, "Ed25519 signature must be exactly 64 bytes");

    for i in 0..sig_len {
        let mut tampered_proof = proof.clone();
        tampered_proof.signature[i] ^= 0xFF; // Flip byte i

        let verify_res = tampered_proof.verify_external(&keypair.verifying_key);
        assert!(
            matches!(verify_res, Err(CryptoError::InvalidProofSignature)),
            "Mutated signature at byte offset {i} MUST be rejected with InvalidProofSignature"
        );
    }
}

// ============================================================================
// SCENARIO 4: UNABHÄNGIGE PRÜFUNG OHNE CONTEXTRA-CODE & TEMPFILE EXPORT
// ============================================================================

/// Scenario 4:
/// - Reorganisiere/Rekonstruiere die signierten Bytes exakt nach dem v3 Payload-Layout
///   OHNE jeglichen contextra-crypto Verifikationscode, nur mit `ed25519_dalek::VerifyingKey::verify`.
/// - Vergleiche das Ergebnis mit `proof.verify_external()`. Beide MÜSSEN `Ok` zurückgeben.
/// - Schreibe den Export in eine temporäre Datei (`tempfile`) und gib den Pfad für manuelle/Python-Prüfung aus.
#[test]
fn test_scenario_4_independent_ed25519_verification_and_tempfile_export() {
    init_test_seed();

    let keypair = DeletionProofKeyPair::generate();
    let scope = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(99).unwrap(),
    };

    let proof = DeletionProof::create_full_v3(
        scope.clone(),
        vec![b"tenant_key_alpha".to_vec()],
        TxId(500),
        vec![],
        vec![
            ExcludedScope::ConsolidatedAndDistilled,
            ExcludedScope::LlmParameterMemory,
        ],
        Some([0x77u8; 32]), // WAL chain receipt
        Some(12345),        // Audit chain position
        1700000000,
        &[],
        keypair.signing_key(),
    )
    .unwrap();

    // 1. Contextra-crypto built-in external verification
    let contextra_verify_res = proof.verify_external(&keypair.verifying_key);
    assert!(contextra_verify_res.is_ok());

    // 2. UNABHÄNGIGE REKONSTRUKTION DES V3 PAYLOADS (Gemäß Doku/Code Spec)
    // Payload Format v3 (Ed25519):
    // - bincode::serialize(&scope)
    // - deleted_keys_hash (32 bytes)
    // - deleted_after_tx (8 bytes LE)
    // - timestamp (8 bytes LE)
    // - bincode::serialize(&covered_layers)
    // - bincode::serialize(&excluded_scopes)
    // - wal_chain_receipt (32 bytes if present, else empty)
    // - audit_chain_position (8 bytes LE if present, else empty)

    let scope_bytes = bincode::serialize(&proof.scope).unwrap();
    let tx_bytes = proof.deleted_after_tx.0.to_le_bytes();
    let timestamp_bytes = proof.timestamp.to_le_bytes();
    let covered_layers_bytes = bincode::serialize(&proof.covered_layers).unwrap();
    let excluded_scopes_bytes = bincode::serialize(&proof.excluded_scopes).unwrap();
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

    let mut reconstructed_payload = Vec::new();
    reconstructed_payload.extend_from_slice(&scope_bytes);
    reconstructed_payload.extend_from_slice(&proof.deleted_keys_hash);
    reconstructed_payload.extend_from_slice(&tx_bytes);
    reconstructed_payload.extend_from_slice(&timestamp_bytes);
    reconstructed_payload.extend_from_slice(&covered_layers_bytes);
    reconstructed_payload.extend_from_slice(&excluded_scopes_bytes);
    reconstructed_payload.extend_from_slice(receipt_part);
    reconstructed_payload.extend_from_slice(audit_pos_part);

    // Direktes ed25519_dalek Verification
    let raw_sig = ed25519_dalek::Signature::from_slice(&proof.signature).unwrap();
    let raw_pubkey: VerifyingKey = keypair.verifying_key;

    let independent_verify_res = raw_pubkey.verify(&reconstructed_payload, &raw_sig);
    assert!(
        independent_verify_res.is_ok(),
        "Unabhängige ed25519_dalek Verifikation des rekonstruierten Payloads muss erfolgreich sein"
    );

    // Beide Verifikationsergebnisse stimmen überein
    assert_eq!(contextra_verify_res.is_ok(), independent_verify_res.is_ok());

    // 3. Export in Tempfile für externe Audits (z.B. Python script)
    let json_export = proof.export_for_audit().unwrap();
    let mut temp_file = tempfile::NamedTempFile::new().expect("Tempfile Erstellung fehlgeschlagen");
    temp_file
        .write_all(json_export.as_bytes())
        .expect("Schreiben in Tempfile fehlgeschlagen");

    let temp_path = temp_file.path().to_path_buf();
    println!("EXTERNAL_AUDIT_PROOF_FILE_PATH={}", temp_path.display());
    println!(
        "EXTERNAL_AUDIT_PUBLIC_KEY_HEX={}",
        hex_encode(&raw_pubkey.to_bytes())
    );

    // Datei nach dem Test erhalten
    let (_, path) = temp_file.keep().expect("Tempfile keep failed");
    assert!(
        path.exists(),
        "Exportierte Audit-Datei muss auf der Festplatte verbleiben"
    );
}

// ============================================================================
// SCENARIO 5: REPLAY-, SCOPE-CONFUSION & DOWNGRADE-ANGRIFFE
// ============================================================================

/// Scenario 5:
/// - Replay / Scope Confusion: Proof von Mandant A gegen Mandant B verifizieren -> Fehler.
/// - Downgrade Angriff: Modifikation der `signature_version` von v3 auf v1 oder v2 oder ungültiges 0xFF -> Verifikation schlägt fehl oder wird definiert abgelehnt.
#[test]
fn test_scenario_5_replay_scope_confusion_and_downgrade_attacks() {
    init_test_seed();

    let keypair_a = DeletionProofKeyPair::generate();
    let keypair_b = DeletionProofKeyPair::generate();

    let scope_a = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(100).unwrap(),
    };
    let scope_b = DeletionScope::Tenant {
        tenant_id: TenantId::try_new(200).unwrap(),
    };

    let proof_a = DeletionProof::create_v3(
        scope_a,
        vec![b"key_a".to_vec()],
        TxId(10),
        vec![],
        vec![],
        1700000000,
        &[],
        keypair_a.signing_key(),
    )
    .unwrap();

    // 1. REPLAY / SCOPE CONFUSION: Proof A mit Public Key B verifizieren
    let res_cross_key = proof_a.verify_external(&keypair_b.verifying_key);
    assert!(
        matches!(res_cross_key, Err(CryptoError::InvalidProofSignature)),
        "Verifikation mit fremdem Public Key B MUSS mit InvalidProofSignature fehlschlagen"
    );

    // 2. REPLAY / SCOPE CONFUSION: Scope von Mandant A auf Mandant B abändern
    let mut tampered_scope_proof = proof_a.clone();
    tampered_scope_proof.scope = scope_b;
    let res_tampered_scope = tampered_scope_proof.verify_external(&keypair_a.verifying_key);
    assert!(
        matches!(res_tampered_scope, Err(CryptoError::InvalidProofSignature)),
        "Verifikation mit manipuliertem Scope MUSS mit InvalidProofSignature fehlschlagen"
    );

    // 3. DOWNGRADE ANGRIFF: Modifikation von signature_version
    // Case 3a: Downgrade v3 -> v2
    let mut downgrade_v2 = proof_a.clone();
    downgrade_v2.signature_version = 2;
    let res_v2 = downgrade_v2.verify_external(&keypair_a.verifying_key);
    assert!(
        matches!(res_v2, Err(CryptoError::UnsupportedProofVersion(2))),
        "verify_external muss v2 Proofs für Ed25519 als UnsupportedProofVersion(2) ablehnen"
    );

    // Case 3b: Downgrade v3 -> v1
    let mut downgrade_v1 = proof_a.clone();
    downgrade_v1.signature_version = 1;
    let res_v1 = downgrade_v1.verify_external(&keypair_a.verifying_key);
    assert!(
        matches!(res_v1, Err(CryptoError::UnsupportedProofVersion(1))),
        "verify_external muss v1 Proofs als UnsupportedProofVersion(1) ablehnen"
    );

    // Case 3c: Invalid signature version byte (0xFF)
    let mut invalid_version = proof_a.clone();
    invalid_version.signature_version = 0xFF;
    let res_invalid = invalid_version.verify_external(&keypair_a.verifying_key);
    assert!(
        matches!(res_invalid, Err(CryptoError::UnsupportedProofVersion(0xFF))),
        "verify_external muss unbekannte Versions-Bytes fail-closed als UnsupportedProofVersion ablehnen"
    );
}

/// Helper function to encode bytes to hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
