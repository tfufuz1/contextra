//! # F-15 Downgrade-Schutz Test
//!
//! SPEC §9.2, §18, F-15:
//! Config::integrity ist beim Öffnen verbindlich.
//! Weicht FORMAT.integrity_mode ab, ist das Ergebnis Err(Unsupported{integrity_mode}),
//! bevor irgendeine andere Datei gelesen wird.

use contextra_crypto_ref::Blake3Mac;
use contextra_storage::catalog::format::{FormatError, FormatHeader, IntegrityMode};
use contextra_storage::keys::secret::Secret32;

#[test]
fn test_f15_downgrade_protection_plain_to_authenticated_mismatch() {
    // // F-15: Manipulierter Modus muss sofort abgelehnt werden
    let instance_id = [0x42u8; 16];
    let header = FormatHeader::new(IntegrityMode::Plain, instance_id, 1_000, 0, 0);
    let bytes = header.encode_plain();

    // Versuch, eine Plain-FORMAT-Datei mit Config = Authenticated zu öffnen
    let err = FormatHeader::decode(&bytes, Some(IntegrityMode::Authenticated), None)
        .expect_err("Modus-Abweichung muss scheitern");

    assert_eq!(err, FormatError::UnsupportedIntegrityMode(0));
}

#[test]
fn test_f15_downgrade_protection_authenticated_to_plain_mismatch() {
    // // F-15: Ein Angreifer versucht eine Authenticated-Instanz auf Plain umzuschreiben
    let instance_id = [0x55u8; 16];
    let header = FormatHeader::new(IntegrityMode::Authenticated, instance_id, 2_000, 0, 0);

    let mac = Blake3Mac;
    let format_key = Secret32::new([0x33u8; 32]);
    let bytes = header.encode_authenticated(&mac, &format_key);

    // Config verlangt Plain, Datei hat aber Authenticated
    let err = FormatHeader::decode(&bytes, Some(IntegrityMode::Plain), None)
        .expect_err("Modus-Abweichung muss scheitern");

    assert_eq!(err, FormatError::UnsupportedIntegrityMode(1));
}

#[test]
fn test_f15_downgrade_protection_encrypted_to_plain_mismatch() {
    // // F-15: Downgrade von Encrypted auf Plain
    let instance_id = [0x77u8; 16];
    let header = FormatHeader::new(IntegrityMode::Encrypted, instance_id, 3_000, 0, 0);

    let mac = Blake3Mac;
    let format_key = Secret32::new([0x88u8; 32]);
    let bytes = header.encode_authenticated(&mac, &format_key);

    let err = FormatHeader::decode(&bytes, Some(IntegrityMode::Plain), None)
        .expect_err("Modus-Abweichung muss scheitern");

    assert_eq!(err, FormatError::UnsupportedIntegrityMode(2));
}

#[test]
fn test_f15_matching_mode_succeeds() {
    let instance_id = [0x99u8; 16];
    let header = FormatHeader::new(IntegrityMode::Authenticated, instance_id, 4_000, 0, 0);

    let mac = Blake3Mac;
    let format_key = Secret32::new([0x12u8; 32]);
    let bytes = header.encode_authenticated(&mac, &format_key);

    let decoded = FormatHeader::decode(
        &bytes,
        Some(IntegrityMode::Authenticated),
        Some((&mac, &format_key)),
    )
    .expect("Passender Modus muss erfolgreich sein");

    assert_eq!(decoded.integrity_mode, IntegrityMode::Authenticated);
}
