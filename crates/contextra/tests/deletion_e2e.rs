// FILE-CONTEXT
// ZWECK: E2E-Löschtests (T-DEL-1 und T-DEL-3) für Contextra Facade (Ring 4).
// INVARIANTEN: Nutzt ausschließlich bestehende öffentliche Schnittstellen.

//! # Integrationstest: `deletion_e2e`
//!
//! Prüft End-to-End-Löschanforderungen der Spezifikation (T-DEL-1 und T-DEL-3):
//! - T-DEL-1: Physische Abwesenheit gelöschter Canary-Bytes auf Datenträgern nach Kompaktierung.
//! - T-DEL-3: Verifikation eines kryptografischen Löschnachweises (DeletionProof) schlägt bei falschem Schlüssel fehl.


use contextra::{open_with_config, ContextraConfig};
use contextra_core::{ContextraError, TenantId};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ANCHOR[contextra:TEST-DE2-001]
/// Rekursive Hilfsfunktion zur Zählung von Canary-Byte-Treffern in allen Dateien eines Verzeichnisses.
fn count_canary_occurrences(dir: &Path, canary: &[u8]) -> usize {
    let mut matches = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                matches += count_canary_occurrences(&path, canary);
            } else if path.is_file() {
                if let Ok(content) = fs::read(&path) {
                    if content.windows(canary.len()).any(|window| window == canary) {
                        matches += 1;
                    }
                }
            }
        }
    }
    matches
}

// ANCHOR[contextra:TEST-DE2-002]
/// T-DEL-1: Schreiben, Löschen, Vollkompaktion/Flush -> Canary-Bytesuche im Datenverzeichnis erwartet 0 Treffer.
#[tokio::test]
async fn test_tdel_1_canary_bytes_erased_after_compaction() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let canary = b"CANARY_SENSITIVE_DATA_987654321_E2E_DELETE_TEST";

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    // 1. Schreiben: Dokument mit Canary-Bytes einfügen und flush auf Disk
    {
        let db = open_with_config(tmp.path(), config.clone()).await?;
        db.insert(
            "doc_canary_1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({
                "secret": std::str::from_utf8(canary).expect("valid utf8")
            })),
        )
        .await?;
        db.flush().await?;
        db.close().await?;
    }

    // 2. Löschen & Flush: Dokument über Re-opened DB löschen und flushen
    {
        let db = open_with_config(tmp.path(), config).await?;
        db.delete("doc_canary_1").await?;
        db.flush().await?;
        db.close().await?;
    }

    // 3. Canary-Bytesuche im Datenverzeichnis
    let matches = count_canary_occurrences(tmp.path(), canary);

    // Erwartet: 0 Treffer (schlägt auf aktuellem Stand fehl durch Assertion mit matches > 0)
    assert_eq!(
        matches, 0,
        "T-DEL-1: Canary-Bytes wurden im Datenverzeichnis ({matches} Datei(en)) nach Schreiben, Löschen und Flush gefunden"
    );

    Ok(())
}

// ANCHOR[contextra:TEST-DE2-003]
/// T-DEL-3: Verifikation eines DeletionProof mit falschem Schlüssel muss Err liefern.
#[tokio::test]
async fn test_tdel_3_proof_verification_fails_with_invalid_key() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };
    let db = open_with_config(tmp.path(), config).await?;

    let tenant_id = TenantId::try_new(100)?;
    let correct_key = b"correct_secret_proof_key_32bytes";
    let wrong_key = b"wrong_invalid_proof_key_32bytes_";

    // Erstelle eine Collection für Tenant
    let _col = db.collection_for_tenant("test_col", tenant_id).await?;

    // Generiere einen DeletionProof über drop_collection
    let proof = db
        .drop_collection("test_col", tenant_id, correct_key)
        .await?;

    // Verifikation mit falschem Schlüssel muss Err liefern
    let verify_result = proof.verify(wrong_key);

    assert!(
        verify_result.is_err(),
        "T-DEL-3: Verifikation eines DeletionProof mit falschem Schlüssel muss Err liefern, lieferte aber {:?}",
        verify_result
    );

    db.close().await?;
    Ok(())
}
