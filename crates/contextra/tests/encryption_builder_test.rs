#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::{
    builder, ContextraBuilder, ContextraConfig, ContextraConfigExt, ContextraError, EncryptionMode,
};
use std::sync::{Arc, Mutex};
use tracing_subscriber::layer::SubscriberExt;
use zeroize::Zeroizing;

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

#[tokio::test]
async fn test_explicit_unencrypted_mode_building() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_integ_unencrypted_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let builder = builder(4)
        .with_storage_path(&tmp_path)
        .with_encryption(EncryptionMode::Unencrypted);

    assert!(!builder.is_encrypted());

    let db = builder.build().await?;
    assert_eq!(db.len().await?, 0);

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}

#[tokio::test]
async fn test_implicit_unencrypted_mode_warning() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Clone, Default)]
    struct StringWriter(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let logs = StringWriter::default();
    let logs_clone = logs.clone();
    let subscriber = tracing_subscriber::fmt::layer()
        .with_writer(move || logs_clone.clone())
        .with_ansi(false);
    let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(subscriber));

    let tmp_path =
        std::env::temp_dir().join(format!("contextra_implicit_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_path);

    {
        let _guard = tracing::dispatcher::set_default(&dispatch);
        let db = ContextraBuilder::new(4)
            .with_storage_path(&tmp_path)
            .build()
            .await?;
        assert_eq!(db.len().await?, 0);
    }

    let output = String::from_utf8(logs.0.lock().unwrap().clone())?;
    assert!(
        output.contains("implicit unencrypted mode"),
        "Expected warning log for implicit unencrypted mode, got: {}",
        output
    );

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}

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
