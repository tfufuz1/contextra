// FILE-CONTEXT
// ZWECK: Unit-Tests für HnswHeader-Serialisierung, Checksumme und Validierung.
// INVARIANTEN: Zero-Panic; bitgleiche Header-Struktur.
// HOTSPOTS: test_hnsw_header_roundtrip_and_errors, test_hnsw_header_version_and_checksum_verification
// STAND: TS:2026-11-12T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use crate::error::{Result, VectorError};
use crate::persistence::header::{HnswHeader, HNSW_MAGIC};

#[test]
fn test_hnsw_header_roundtrip_and_errors() -> Result<()> {
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 100, 5, 84, 256, 42);

    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), HnswHeader::SIZE);
    let parsed = HnswHeader::try_from_bytes(&bytes)?;
    assert_eq!(header, parsed);

    let small_bytes = vec![0u8; 10];
    let err_small = HnswHeader::try_from_bytes(&small_bytes);
    assert!(matches!(err_small, Err(VectorError::Storage(_))));

    let mut bad_magic_bytes = bytes;
    bad_magic_bytes[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    let err_magic = HnswHeader::try_from_bytes(&bad_magic_bytes);
    assert!(matches!(err_magic, Err(VectorError::Storage(_))));

    Ok(())
}

#[test]
fn test_hnsw_header_version_and_checksum_verification() {
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 100, 5, 84, 256, 42);
    let checksum = header.checksum();
    assert!(header.verify_checksum(&checksum).is_ok());

    let mut bad_checksum = checksum;
    bad_checksum[0] ^= 0xFF;
    assert!(header.verify_checksum(&bad_checksum).is_err());
}
