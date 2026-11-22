//! # Feature-Bits Tests
//!
//! SPEC §9.2:
//! - Unbekanntes Pflicht-Bit führt zu `UnsupportedFeature`.
//! - Unbekanntes optionales Bit wird ignoriert.
//! - Pflicht-Bits werden nur hinzugefügt (`|=`).

use contextra_storage::catalog::format::{
    add_required_feature, FormatError, FormatHeader, IntegrityMode, FEATURE_AUTHENTICATED,
    FEATURE_DOMAIN_SEALED, FEATURE_ENCRYPTED,
};

#[test]
fn test_unknown_required_bit_fails() {
    let instance_id = [0x11u8; 16];
    // Bit 1 << 4 ist ein unbekanntes Pflicht-Bit (< 1 << 32)
    let unknown_required_bit = 1u64 << 4;
    let header = FormatHeader::new(
        IntegrityMode::Plain,
        instance_id,
        1_000,
        unknown_required_bit,
        0,
    );
    let bytes = header.encode_plain();

    let err = FormatHeader::decode(&bytes, Some(IntegrityMode::Plain), None)
        .expect_err("Unbekanntes Pflicht-Bit muss scheitern");

    assert_eq!(err, FormatError::UnsupportedFeature(unknown_required_bit));
}

#[test]
fn test_unknown_optional_bit_is_ignored() {
    let instance_id = [0x22u8; 16];
    // Bits >= 1 << 32 sind für optionale Features reserviert
    let unknown_optional_bit = 1u64 << 40;
    let header = FormatHeader::new(
        IntegrityMode::Plain,
        instance_id,
        2_000,
        0,
        unknown_optional_bit,
    );
    let bytes = header.encode_plain();

    let decoded = FormatHeader::decode(&bytes, Some(IntegrityMode::Plain), None)
        .expect("Unbekanntes optionales Bit muss ignoriert werden");

    assert_eq!(decoded.optional_features, unknown_optional_bit);
}

#[test]
fn test_required_bits_are_only_added() {
    let instance_id = [0x33u8; 16];
    let mut header = FormatHeader::new(
        IntegrityMode::Plain,
        instance_id,
        3_000,
        FEATURE_ENCRYPTED,
        0,
    );

    assert_eq!(header.required_features, FEATURE_ENCRYPTED);

    // Hinzufügen weiterer Bits
    add_required_feature(&mut header, FEATURE_AUTHENTICATED);
    assert_eq!(
        header.required_features,
        FEATURE_ENCRYPTED | FEATURE_AUTHENTICATED
    );

    add_required_feature(&mut header, FEATURE_DOMAIN_SEALED);
    assert_eq!(
        header.required_features,
        FEATURE_ENCRYPTED | FEATURE_AUTHENTICATED | FEATURE_DOMAIN_SEALED
    );

    // Bereits gesetztes Bit nochmal hinzufügen ändert nichts
    add_required_feature(&mut header, FEATURE_ENCRYPTED);
    assert_eq!(
        header.required_features,
        FEATURE_ENCRYPTED | FEATURE_AUTHENTICATED | FEATURE_DOMAIN_SEALED
    );
}
