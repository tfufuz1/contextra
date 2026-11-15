//! Ed25519-Signaturen mit Domänentrennung (Spec 4.1).

use acta_base::{CryptoRng, EntropyError};
use acta_crypto::domain::{CHECKPOINT, REPORT};
use acta_crypto::error::CryptoError;
use acta_crypto::sign::{verify, NotarySigner};

struct FailingRng;
impl CryptoRng for FailingRng {
    fn try_fill(&self, _dst: &mut [u8]) -> Result<(), EntropyError> {
        Err(EntropyError)
    }
}

#[test]
fn signature_verifies_and_binds_domain_and_message() -> Result<(), CryptoError> {
    let signer = NotarySigner::from_seed([7; 32]);
    let pk = signer.public_key();
    let sig = signer.sign(REPORT, b"bericht");
    verify(&pk, REPORT, b"bericht", &sig)?;

    // Eine Signatur gilt nicht in einer anderen Domäne, nicht für eine andere Nachricht.
    assert!(matches!(
        verify(&pk, CHECKPOINT, b"bericht", &sig),
        Err(CryptoError::SignatureInvalid)
    ));
    assert!(matches!(
        verify(&pk, REPORT, b"anders", &sig),
        Err(CryptoError::SignatureInvalid)
    ));

    // Fremder Schlüssel.
    let other = NotarySigner::from_seed([8; 32]).public_key();
    assert!(matches!(
        verify(&other, REPORT, b"bericht", &sig),
        Err(CryptoError::SignatureInvalid)
    ));
    Ok(())
}

#[test]
fn generation_fails_closed_without_entropy() {
    assert!(matches!(
        NotarySigner::generate(&FailingRng),
        Err(CryptoError::Entropy)
    ));
}
