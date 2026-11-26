// FILE-CONTEXT
// ZWECK: Persistenz Mutationsmatrix-Test (Header- und Offset-Beschädigung).

use contextra_vector::persistence::{write_atomic, HnswHeader};
use tempfile::tempdir;

#[test]
fn test_persistence_header_mutation_matrix() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_mut.hnsw");

    let header = HnswHeader::new_v2_with_bias(
        128, 16, 0, 1, 0.0, 1.0, 10, 0, 100, 500, 1, 0, 0, 0.0, 0.0,
    );
    let bytes = header.to_bytes();

    // 1. Bit-Mutation an Magic-Bytes -> Rejection
    for offset in 0..4 {
        let mut corrupted = bytes;
        corrupted[offset] ^= 0xFF;
        assert!(HnswHeader::try_from_bytes(&corrupted).is_err());
    }

    // 2. Ungültige Version (z.B. 99) -> Rejection
    let mut bad_ver = bytes;
    bad_ver[4] = 99;
    bad_ver[5] = 0;
    assert!(HnswHeader::try_from_bytes(&bad_ver).is_err());

    // 3. Truncated Header (< 64 Bytes) -> Rejection
    assert!(HnswHeader::try_from_bytes(&bytes[0..30]).is_err());

    // 4. write_atomic Roundtrip
    write_atomic(&file_path, &bytes).unwrap();
    let loaded = std::fs::read(&file_path).unwrap();
    assert_eq!(loaded, bytes);

    let parsed = HnswHeader::try_from_bytes(&loaded).unwrap();
    assert_eq!(parsed.dimension(), 128);
    assert_eq!(parsed.m(), 16);
}
