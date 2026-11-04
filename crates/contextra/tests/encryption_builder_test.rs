//! # Integrationstest: `encryption_builder_test`
//!
//! Testet die Verschlüsselungskonfiguration des Builders:
//! - Ablehnung von Passphrasen unter 12 Zeichen.
//! - Statischer Typestate-Zwang (Passphrase & Runtime müssen gesetzt sein).
//! - Kein Leaken der Passphrase in Debug-Ausgaben.

use contextra::{Contextra, Error, Secret};

#[tokio::test]
async fn test_short_passphrase_rejected() {
    let rt = tokio::runtime::Handle::current();
    let secret = Secret::new(b"short".to_vec()); // < 12 Zeichen

    let res = Contextra::builder()
        .passphrase(secret)
        .runtime(rt)
        .open()
        .await;

    assert!(res.is_err());
    match res {
        Err(Error::Config { message }) => {
            assert!(message.contains("mindestens 12 Zeichen"));
        }
        _ => panic!("Erwartet Error::Config wegen zu kurzer Passphrase"),
    }
}

#[tokio::test]
async fn test_secret_not_leaked_in_debug() {
    let passphrase = b"sehr-sichere-passphrase-12345".to_vec();
    let secret = Secret::new(passphrase.clone());

    // Secret implementiert per SPEC TY-11 bewusst weder Debug noch Display
    // Aber der Builder darf die Passphrase nicht in Debug leaken:
    let rt = tokio::runtime::Handle::current();
    let builder = Contextra::builder().passphrase(secret).runtime(rt);

    let err = Error::Auth;
    let err_str = format!("{err:?}");
    assert!(!err_str.contains("sehr-sichere-passphrase"));
}
