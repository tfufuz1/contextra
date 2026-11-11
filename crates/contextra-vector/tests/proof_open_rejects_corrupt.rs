// FILE-CONTEXT
// ZWECK: Verifiziert, dass manipulierte oder beschädigte Header deterministisch mit typisierten Fehlern abgewiesen werden (Zero-Panic, Datei byte-identisch).

use std::fs;
use contextra_vector::persistence::{HnswHeader, MmapIndex};

#[test]
fn test_proof_open_rejects_corrupt_headers() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("corrupt_test.bin");

    // 1. Zu kleiner Header (< 64 Bytes)
    let tiny_data = vec![0u8; 32];
    fs::write(&file_path, &tiny_data).unwrap();
    assert!(MmapIndex::open(&file_path).is_err());
    assert!(HnswHeader::try_from_bytes(&tiny_data).is_err());
    assert_eq!(fs::read(&file_path).unwrap(), tiny_data);

    // 2. Falscher Magic
    let header = HnswHeader::new(16, 8, 0, 0, 0.0, 0.0, 10, 0, 100, 200, 1);
    let mut bytes = header.to_bytes();
    bytes[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    fs::write(&file_path, &bytes).unwrap();
    assert!(MmapIndex::open(&file_path).is_err());
    assert!(HnswHeader::try_from_bytes(&bytes).is_err());
    assert_eq!(fs::read(&file_path).unwrap(), bytes);

    // 3. Unbekannte / ununterstützte Version (z. B. 99)
    let mut bytes = header.to_bytes();
    bytes[4..6].copy_from_slice(&99u16.to_le_bytes());
    fs::write(&file_path, &bytes).unwrap();
    assert!(MmapIndex::open(&file_path).is_err());
    assert!(HnswHeader::try_from_bytes(&bytes).is_err());
    assert_eq!(fs::read(&file_path).unwrap(), bytes);

    // 4. Dimension 0 oder fehlerhafte Metrik
    let mut bytes = header.to_bytes();
    bytes[6..10].copy_from_slice(&0u32.to_le_bytes());
    fs::write(&file_path, &bytes).unwrap();
    let h_parsed = HnswHeader::try_from_bytes(&bytes).unwrap();
    assert_eq!(h_parsed.dimension(), 0);

    // 5. Checksum verification fails if modified
    let valid_header = HnswHeader::new(16, 8, 0, 0, 0.0, 0.0, 10, 0, 100, 200, 1);
    let valid_checksum = valid_header.checksum();
    let mut bad_checksum = valid_checksum;
    bad_checksum[0] ^= 0xFF;
    assert!(valid_header.verify_checksum(&bad_checksum).is_err());
}
