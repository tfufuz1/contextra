//! # Externe Verifizierbarkeit von Löschnachweisen (AK-20)
//!
//! Dieses Beispiel demonstriert die **externe Verifizierbarkeit** von DSGVO-Art.-17-Löschnachweisen
//! (`DeletionProof`) durch unbeteiligte Dritte (Third-Party Verifiers) gemäß
//! `CONTEXTRA_SPEC_2_.md` §2.5.1 ("Sovereign Ring vs. Third-Party Attestation") und §15.6.
//!
//! ## Kernkonzept
//! Ein Löschbeweis, den niemand außer dem Hersteller lesen oder verifizieren kann, ist eine bloße
//! Selbstattestierung. Contextra stellt deshalb Ed25519-signierte Löschnachweise (Version 3) bereit,
//! die von jedem unabhängigen Dritten ohne Zugriff auf die Contextra-Instanz, deren Storage-Backends,
//! oder deren internen `KeyManager` eigenständig mathematisch verifiziert werden können.
//!
//! ## Ausführung
//! `cargo run -p contextra-db --example verify_deletion_proof_external`

#![allow(deprecated)]

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
    LayerCleanupProof,
};
use contextra_crypto::CryptoError;
use contextra_db::{Contextra, ContextraConfig};
use contextra_types::{CollectionId, TenantId, TxId};
use ed25519_dalek::VerifyingKey;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// =========================================================================
// FREISTEHENDE EXTERNE PRÜFFUNKTION (AK-20)
// =========================================================================

/// Externe Verifikationsfunktion für unabhängige Dritte (Rolle b: Externer Prüfer).
///
/// ABHÄNGIGKEIT & CODE-STRUKTUR:
/// Diese Funktion hat nachweislich KEINEN Zugriff auf Contextra-DB-, Collection- oder
/// KeyManager-Instanzen. Sie nimmt ausschliesslich den `DeletionProof` und den
/// öffentlichen Ed25519 `VerifyingKey` entgegen und ist somit strikt von der Betreiber-
/// und Storage-Ebene isoliert.
fn verify_external(
    proof: &DeletionProof,
    verifying_key: &VerifyingKey,
) -> Result<(), CryptoError> {
    let is_valid = proof
        .verify(verifying_key)
        .map_err(|e| CryptoError::Crypto(format!("InvalidProofSignature: {}", e)))?;

    if is_valid {
        Ok(())
    } else {
        Err(CryptoError::Crypto(
            "InvalidProofSignature: Ed25519 signature verification failed or proof payload tampered"
                .to_string(),
        ))
    }
}

// =========================================================================
// ROLLE: Contextra-Betreiber (Operator)
// =========================================================================

/// Simuliert die Betreiber-Rolle: Erzeugt Daten, führt Löschung durch und erzeugt Löschnachweis.
async fn run_operator_role(
    proof_file_path: &Path,
    pubkey_file_path: &Path,
) -> contextra_types::Result<()> {
    println!("=========================================================================");
    println!("  ROLLE: Contextra-Betreiber (Operator)");
    println!("=========================================================================");

    // 1. Ed25519-Schlüsselpaar für Löschnachweise erzeugen
    let keypair = DeletionProofKeyPair::generate();
    let verifying_key_bytes = keypair.verifying_key_bytes();

    // 2. Temporäre Contextra DB-Instanz initialisieren
    let db_dir = TempDir::new().expect("Failed to create temp db dir");
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(db_dir.path(), config).await?;

    // 3. Collection anlegen und sensitive Kundendaten (PII) einfügen
    let col_name = "gdpr_user_pii";
    let tenant_id = TenantId::new(42);
    let col = db.collection(col_name).await?;

    col.insert(
        "user_101",
        &[0.1, 0.2, 0.3, 0.4],
        Some(serde_json::json!({
            "email": "user101@example.com",
            "text": "Personenbezogene Kundendaten DSGVO Art. 17"
        })),
    )
    .await?;

    println!("  [Betreiber] Collection '{col_name}' mit sensiblen Daten angelegt.");

    // 4. Physikalische Löschung der Collection ausführen
    let proof_key_hmac = b"hmac-key-for-internal-drop";
    let _internal_proof = db.drop_collection(col_name, tenant_id, proof_key_hmac).await?;

    // Erzeuge Ed25519-Löschnachweis (Version 3) für den externen Prüfer
    let col_id = CollectionId::new(100);
    let scope = DeletionScope::Collection {
        collection_id: col_id,
        tenant_id,
    };

    let layer_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)?,
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0)?,
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0)?,
    ];

    let proof_v3 = DeletionProof::create_v3(
        scope,
        vec![
            format!("__col:{col_name}:user_101").into_bytes(),
            format!("__txt:{col_name}:user_101").into_bytes(),
        ],
        TxId::new(105),
        layer_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        keypair.signing_key(),
    )?;

    println!("  [Betreiber] Physische Löschung abgeschlossen. Ed25519 DeletionProof (v3) erzeugt.");

    // 5. "Veröffentlichen": DeletionProof & öffentlichen Schlüssel in Dateien schreiben
    let proof_json = proof_v3.export_for_audit()?;
    fs::write(proof_file_path, &proof_json).expect("Failed to write proof file");
    fs::write(pubkey_file_path, &verifying_key_bytes).expect("Failed to write pubkey file");

    println!("  [Betreiber] Proof in '{}' veröffentlicht.", proof_file_path.display());
    println!("  [Betreiber] Öffentlicher Schlüssel in '{}' veröffentlicht.", pubkey_file_path.display());

    db.close().await?;
    Ok(())
}

// =========================================================================
// ROLLE: Externer Prüfer (Third-Party Verifier)
// =========================================================================

/// Simuliert den externen Prüfer: Liest AUSSCHLIESSLICH die Proof-Datei und den öffentlichen Schlüssel von Disk.
fn run_external_verifier_role(
    proof_file_path: &Path,
    pubkey_file_path: &Path,
    label: &str,
) -> Result<(), CryptoError> {
    println!("=========================================================================");
    println!("  ROLLE: Externer Prüfer (Third-Party Verifier) — {}", label);
    println!("=========================================================================");

    // 1. AUSSCHLIESSLICH die veröffentlichte Proof-Datei und den öffentlichen Schlüssel von Disk lesen
    let proof_json = fs::read_to_string(proof_file_path)
        .map_err(|e| CryptoError::InvalidInput(format!("Proof-Datei konnte nicht gelesen werden: {e}")))?;
    let pubkey_bytes = fs::read(pubkey_file_path)
        .map_err(|e| CryptoError::InvalidInput(format!("Schlüssel-Datei konnte nicht gelesen werden: {e}")))?;

    let proof: DeletionProof = serde_json::from_str(&proof_json)
        .map_err(|e| CryptoError::InvalidInput(format!("Proof JSON Deserialisierung fehlgeschlagen: {e}")))?;

    let pk_array: [u8; 32] = pubkey_bytes.as_slice().try_into()
        .map_err(|_| CryptoError::InvalidLength("Öffentlicher Schlüssel hat keine 32 Bytes".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&pk_array)
        .map_err(|e| CryptoError::InvalidInput(format!("Ungültiger Ed25519 Schlüssel: {e}")))?;

    println!("  [Externer Prüfer] Proof & Öffentlichen Ed25519-Schlüssel ({}) von Disk eingelesen.", pubkey_file_path.display());
    println!("  [Externer Prüfer] Starte eigenständige mathematische Verifikation...");

    // 2. Rufe verify_external auf (hat KEINEN Zugriff auf DB / KeyManager / KeyPair)
    verify_external(&proof, &verifying_key)?;

    println!("  ✅ [Externer Prüfer] ERFOLG: DeletionProof ist GÜLTIG und mathematisch verifiziert!");
    println!("     • Tenant ID:         {:?}", proof.tenant_id());
    println!("     • Deletion Scope:    {:?}", proof.scope);
    println!("     • Abgedeckte Layer:  {:?}", proof.covered_layers);
    println!("     • Excluded Scopes:   {:?}", proof.excluded_scopes);
    println!("     • Ed25519 Signature: {} bytes", proof.signature.len());

    Ok(())
}

// =========================================================================
// MAIN ENTRYPOINT
// =========================================================================

#[tokio::main]
async fn main() -> contextra_types::Result<()> {
    println!("=========================================================================");
    println!("  CONTEXTRA — EXTERNE VERIFIZIERBARKEIT VON LÖSCHNACHWEISEN (AK-20)");
    println!("  Referenz: CONTEXTRA_SPEC_2_.md §2.5.1 & §15.6");
    println!("=========================================================================\n");

    let temp_dir = TempDir::new().expect("Failed to create temp exchange dir");
    let proof_file = temp_dir.path().join("deletion_proof_v3.json");
    let pubkey_file = temp_dir.path().join("operator_ed25519.pub");

    // --- DURCHLAUF 1: GÜLTIGER PROOF ---
    println!(">>> DURCHLAUF 1: Regulärer Ablauf mit gültigem Löschnachweis <<<\n");
    run_operator_role(&proof_file, &pubkey_file).await?;
    println!();

    let verifier_result = run_external_verifier_role(&proof_file, &pubkey_file, "Gültiger Proof");
    if let Err(e) = verifier_result {
        panic!("Verifikation des gültigen Proofs fehlgeschlagen: {e}");
    }

    // --- DURCHLAUF 2: MANIPULIERTER PROOF ---
    println!("\n>>> DURCHLAUF 2: Negativtest mit manipuliertem Löschnachweis <<<\n");
    println!("  [Simulation] Ein Angreifer oder fehlerhafter Server manipuliert das TxId/Timestamp-Feld im Proof...");

    // Proof-Datei einlesen und gezielt manipulieren
    let raw_json = fs::read_to_string(&proof_file).expect("read proof");
    let mut tampered_proof: DeletionProof = serde_json::from_str(&raw_json).expect("parse json");
    tampered_proof.deleted_after_tx = TxId::new(999999);
    let tampered_json = serde_json::to_string_pretty(&tampered_proof).expect("serialize tampered");

    let tampered_proof_file = temp_dir.path().join("deletion_proof_tampered.json");
    fs::write(&tampered_proof_file, &tampered_json).expect("write tampered");

    let tampered_result = run_external_verifier_role(&tampered_proof_file, &pubkey_file, "Manipulierter Proof");
    match tampered_result {
        Ok(_) => panic!("FEHLER: Manipulierter Proof wurde fälschlicherweise als gültig akzeptiert!"),
        Err(crypto_err) => {
            println!("\n  ❌ [Externer Prüfer] FEHLER ABGEFANGEN (Erwartetes Verhalten):");
            println!("     Fehlerdetails: {crypto_err}");
            println!("  ✅ Verifikation des manipulierten Proofs korrekt mit InvalidProofSignature-Fehler abgelehnt!");
        }
    }

    println!("\n=========================================================================");
    println!("  AK-20 BEWEIS ERFOLGREICH: Externe Verifizierbarkeit vollständig demonstriert!");
    println!("=========================================================================");

    Ok(())
}
