// ZWECK: Test sovereign profile encryption requirement and silent downgrade protection.

use contextra::performance_profile::PerformanceProfile;
use contextra::ContextraBuilder;
use contextra_store::lsm::config::{EncryptionConfig, KekSource};
use tempfile::tempdir;

#[tokio::test]
async fn test_sovereign_profile_without_encryption_fails_open() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // Sovereign profile (PerformanceProfile::Compliance) without encryption config MUST fail open.
    let result = ContextraBuilder::new(4)
        .with_storage_path(tmp.path())
        .with_performance_profile(PerformanceProfile::Compliance)
        .build()
        .await;

    assert!(
        result.is_err(),
        "Opening Contextra with Sovereign profile without encryption MUST fail closed"
    );

    Ok(())
}

#[tokio::test]
async fn test_silent_downgrade_attempt_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // 1. Open database with encryption enabled and write a record.
    {
        let db = ContextraBuilder::new(4)
            .with_storage_path(tmp.path())
            .with_encryption_passphrase("sovereign_secret_key_123")
            .build()
            .await?;

        db.insert("doc_1", &[0.1, 0.2, 0.3, 0.4], Some(serde_json::json!({"canary": "secret_data"})))
            .await?;
        db.close().await?;
    }

    // 2. Attempting to reopen the encrypted database WITHOUT encryption config MUST fail.
    let downgrade_result = ContextraBuilder::new(4)
        .with_storage_path(tmp.path())
        .build()
        .await;

    assert!(
        downgrade_result.is_err(),
        "Opening an encrypted database directory with EncryptionConfig::None MUST be rejected as silent downgrade"
    );

    Ok(())
}

#[test]
fn test_encryption_config_debug_never_leaks_passphrase() {
    let passphrase_cfg = EncryptionConfig::Passphrase("super_secret_passphrase_999".to_string());
    let debug_str = format!("{:?}", passphrase_cfg);

    assert!(
        !debug_str.contains("super_secret_passphrase_999"),
        "EncryptionConfig Debug representation MUST NOT leak passphrase: {}",
        debug_str
    );
    assert!(
        debug_str.contains("***"),
        "EncryptionConfig Debug representation should mask passphrase with ***: {}",
        debug_str
    );

    let keyring_cfg = EncryptionConfig::Keyring {
        dir: std::path::PathBuf::from("/tmp/keyring"),
        kek_source: KekSource::Passphrase("kek_secret_pass_123".to_string()),
    };
    let keyring_debug = format!("{:?}", keyring_cfg);

    assert!(
        !keyring_debug.contains("kek_secret_pass_123"),
        "KekSource Debug representation MUST NOT leak secret: {}",
        keyring_debug
    );
}
