use contextra_vector::persistence::MmapIndex;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_hnsw_corrupted_file_recovery_fail_closed() {
    let golden_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("hnsw_raw.bin");

    assert!(golden_path.exists(), "golden file hnsw_raw.bin must exist");
    let original_bytes = fs::read(&golden_path).expect("read golden");

    let dir = tempdir().expect("tempdir");

    // 1. Truncation Tests
    let truncate_sizes = [0, 4, 16, 32, 63, 70];
    for &sz in &truncate_sizes {
        if sz < original_bytes.len() {
            let trunc_path = dir.path().join(format!("trunc_{sz}.bin"));
            fs::write(&trunc_path, &original_bytes[..sz]).expect("write trunc");

            let res = MmapIndex::open(&trunc_path);
            assert!(
                res.is_err(),
                "Opening truncated file with size {sz} should return Err",
            );
        }
    }

    // 2. Deterministic Bitflip Tests across header
    let flip_offsets = [0, 1, 2, 3, 4, 5, 6, 14, 15, 24, 32, 40, 48, 56];
    for &offset in &flip_offsets {
        if offset < original_bytes.len() {
            let mut corrupted = original_bytes.clone();
            corrupted[offset] ^= 0xFF;

            let flip_path = dir.path().join(format!("flip_{offset}.bin"));
            fs::write(&flip_path, &corrupted).expect("write flip");

            let res = MmapIndex::open(&flip_path);
            if let Ok(idx) = res {
                let check = idx.header.verify_checksum(&[0u8; 32]);
                assert!(
                    check.is_err(),
                    "Checksum verification must detect corruption"
                );
            }
        }
    }
}
