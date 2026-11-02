#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! # Integrationstest: `encryption_builder_test`
//!
//! Testet die Verschlüsselungskonfiguration des Builders (`EncryptionMode`, `ContextraConfigExt`):
//! - Maskierung vertraulicher Passphrasen ([REDACTED]) in Debug-Ausgaben.
//! - Ablehnung leerer Passphrasen.
//! - Ablehnung unverschlüsselter Konfiguration (D-05 Pflicht-Verschlüsselung).
//! - Erfolgreicher Roundtrip bei verschlüsselter Speicherung.

use contextra::{
    builder, ContextraBuilder, ContextraConfig, ContextraConfigExt, ContextraError, EncryptionMode,
};
use zeroize::Zeroizing;

// ANCHOR[contextra:TEST-ENC-001]
// Verifiziert die Maskierung von Passphrasen ([REDACTED]) in Debug-Strings und prüft die Hilfsmethode is_encrypted.
#[tokio::test]
async fn test_encryption_mode_redaction_and_facade() {
    let passphrase = "my-secret-passphrase-987";
    let mode = EncryptionMode::Passphrase(Zeroizing::new(passphrase.to_string()));

    let debug_output = format!("{:?}", mode);
    assert!(!debug_output.contains(passphrase));
    assert!(debug_output.contains("[REDACTED]"));

    let builder = builder(16).with_encryption(mode);
    assert!(builder.is_encrypted());

    let builder_debug = format!("{:?}", builder);
    assert!(!builder_debug.contains(passphrase));
    assert!(builder_debug.contains("[REDACTED]"));

    let config = ContextraConfig {
        encryption_passphrase: Some("some_pass".to_string()),
        ..Default::default()
    };
    assert!(config.is_encrypted());

    let unencrypted_config = ContextraConfig::default();
    assert!(!unencrypted_config.is_encrypted());
}

// ANCHOR[contextra:TEST-ENC-002]
// Stellt sicher, dass der Versuch, eine leere Passphrase zu konfigurieren, fehlschlägt.
#[tokio::test]
async fn test_empty_passphrase_rejection() {
    let res = builder(16).with_encryption_passphrase("").build().await;
    assert!(res.is_err());
    match res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(msg.contains("cannot be empty"));
        }
        _ => panic!("Expected ContextraError::InvalidInput for empty passphrase"),
    }
}

// ANCHOR[contextra:TEST-ENC-003]
// Testet die Ablehnung einer explizit unverschlüsselten Datenbank nach D-05.
#[tokio::test]
async fn test_explicit_unencrypted_mode_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_integ_unencrypted_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let res = builder(4)
        .with_storage_path(&tmp_path)
        .with_encryption(EncryptionMode::Unencrypted)
        .build()
        .await;

    assert!(res.is_err());
    match res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(msg.contains("Unencrypted storage mode is not permitted"));
        }
        _ => panic!("Expected ContextraError::InvalidInput for Unencrypted mode"),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}

// ANCHOR[contextra:TEST-ENC-004]
// Testet die Ablehnung einer Datenbank ohne Passphrase nach D-05.
#[tokio::test]
async fn test_unconfigured_encryption_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path =
        std::env::temp_dir().join(format!("contextra_implicit_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let res = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .build()
        .await;

    assert!(res.is_err());
    match res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(msg.contains("Encryption passphrase is required"));
        }
        _ => panic!("Expected ContextraError::InvalidInput for unconfigured encryption"),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}

// ANCHOR[contextra:TEST-ENC-005]
// Prüft den erfolgreichen Aufbau einer persistenten Datenbank mit gültiger Verschlüsselungs-Passphrase.
#[tokio::test]
async fn test_encrypted_storage_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path =
        std::env::temp_dir().join(format!("contextra_integ_encrypted_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let db = builder(4)
        .with_storage_path(&tmp_path)
        .with_encryption_passphrase("valid-passphrase-123")
        .build()
        .await?;

    assert_eq!(db.len().await?, 0);

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}
