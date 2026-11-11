//! Format-Konstanten und Header-Verifikation (PROOF-3)
//!
//! Prüft Magic-Bytes und Versionskonstanten gegen feste Erwartungswerte aus PROOF-1.

mod legacy_reference;

use contextra_store::manifest::{CURRENT_MANIFEST_VERSION, MANIFEST_HEADER_MAGIC};
use contextra_store::sstable::{SSTABLE_MAGIC_LEGACY, SSTABLE_MAGIC_MFSX};
use contextra_store::wal::{WAL_V2_HEADER, WAL_V3_HEADER};
use legacy_reference::format_constants_legacy::*;
use legacy_reference::manifest_header_legacy::*;

#[test]
fn test_magic_and_constants_match_expected() {
    assert_eq!(MANIFEST_HEADER_MAGIC, b"MFMN");
    assert_eq!(LEGACY_MANIFEST_HEADER_MAGIC, b"MFMN");
    assert!(
        CURRENT_MANIFEST_VERSION == 1 || CURRENT_MANIFEST_VERSION == 2,
        "Manifest version must be 1 (current) or 2 (after STORE-A bump), but was {}",
        CURRENT_MANIFEST_VERSION
    );

    assert_eq!(SSTABLE_MAGIC_MFSX, 0x5853_464D); // "MFSX"
    assert_eq!(LEGACY_SSTABLE_MAGIC_MFSX, 0x5853_464D);
    assert_eq!(SSTABLE_MAGIC_LEGACY, 0x4D46_5354); // "MFST"
    assert_eq!(LEGACY_SSTABLE_MAGIC_LEGACY, 0x4D46_5354);

    assert_eq!(WAL_V3_HEADER, *b"MFW3");
    assert_eq!(LEGACY_WAL_V3_HEADER, *b"MFW3");
    assert_eq!(WAL_V2_HEADER, *b"MFW2");
    assert_eq!(LEGACY_WAL_V2_HEADER, *b"MFW2");

    assert_eq!(LEGACY_INTENT_MAGIC, b"MFRLBK\0\0");
}

#[test]
fn test_manifest_header_parsing_legacy() {
    let valid_v1_header = create_manifest_header(1);
    assert_eq!(parse_manifest_header_v1(&valid_v1_header), Ok(1));

    let valid_v2_header = create_manifest_header(2);
    assert_eq!(parse_manifest_header_v1(&valid_v2_header), Ok(2));

    let invalid_magic = b"ABCD\x01";
    assert!(parse_manifest_header_v1(invalid_magic).is_err());

    let too_short = b"MFMN";
    assert!(parse_manifest_header_v1(too_short).is_err());

    let invalid_version_zero = b"MFMN\x00";
    assert!(parse_manifest_header_v1(invalid_version_zero).is_err());
}
