//! Betroffenen-Tag (Spec 2.4): Eindeutigkeit, Mandantentrennung und NFC-Normalisierung.

use acta_crypto::error::CryptoError;
use acta_crypto::kdf::derive_subject_tag;
use acta_crypto::key::IndexSecret;

#[test]
fn tag_matches_reference_and_normalizes_nfc() -> Result<(), CryptoError> {
    let index = IndexSecret::from_bytes([0x55; 32]);
    let tenant = [0x44; 16];
    // Festwert aus Python (HMAC über NFC("émail:Ana")), Eingabe mit zerlegtem Akzent.
    let tag = derive_subject_tag(&index, &tenant, 1, "e\u{301}mail:Ana")?;
    assert_eq!(
        tag.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "df715e4b33dcd81bdef682b08d27315d"
    );
    // Dieselbe Kennung in NFC-Form ergibt denselben Tag.
    assert_eq!(
        tag,
        derive_subject_tag(&index, &tenant, 1, "\u{e9}mail:Ana")?
    );
    Ok(())
}

#[test]
fn tags_separate_tenants_kinds_and_identifiers() -> Result<(), CryptoError> {
    let index = IndexSecret::from_bytes([0x55; 32]);
    let base = derive_subject_tag(&index, &[1; 16], 1, "az:123")?;
    assert_ne!(base, derive_subject_tag(&index, &[2; 16], 1, "az:123")?);
    assert_ne!(base, derive_subject_tag(&index, &[1; 16], 2, "az:123")?);
    assert_ne!(base, derive_subject_tag(&index, &[1; 16], 1, "az:124")?);
    Ok(())
}
