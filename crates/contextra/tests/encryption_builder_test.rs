//! # Test: Passphrase-Validierung und Typestate-Absicherung (SPEC 5.7, 7.1)

use contextra::{Contextra, Error, Secret};
use std::time::Duration;

/// SPEC: T-API-1
#[tokio::test]
async fn test_short_passphrase_rejected() {
    let rt = tokio::runtime::Handle::current();
    let short_secret = Secret::new(b"kurz".to_vec());

    let builder = Contextra::builder()
        .data_dir("./target/short_pass_test")
        .passphrase(short_secret)
        .runtime(rt)
        .backup_retention(Duration::from_secs(86400));

    let result = builder.build().await;
    match result {
        Err(Error::Config(c)) => {
            assert!(c.to_string().contains("12 Zeichen"));
        }
        other => panic!("Expected Config error for short passphrase, got: {other:?}"),
    }
}
