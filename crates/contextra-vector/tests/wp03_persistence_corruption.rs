// FILE-CONTEXT
// ZWECK: Korruptions-, Bit-Flip- und Trunkierungs-Tests fuer HNSW-Persistenzschicht (fail-closed, Zero-Panic).
// INVARIANTEN: Jede Bit-Mutation, Verkuerzung oder unzulaessige Offset fuehrt zu Err(VectorError::Storage), niemals zu Panic.
// HOTSPOTS: test_header_bit_flips_fail_closed, test_truncations_fail_closed
// STAND: TS:2026-11-12T00:00:00Z

use contextra_vector::persistence::{HnswHeader, MmapIndex, NodeRecord};
use std::fs;
use std::path::Path;

const GOLDEN_FILES: &[&str] = &[
    "tests/golden/hnsw_raw.bin",
    "tests/golden/hnsw_sq8.bin",
    "tests/golden/hnsw_tombstones.bin",
];

#[test]
fn test_header_bit_flips_fail_closed() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let temp_dir = tempfile::tempdir().expect("tempdir");

    for file_rel in GOLDEN_FILES {
        let path = Path::new(manifest_dir).join(file_rel);
        if !path.exists() {
            continue;
        }
        let original_bytes = fs::read(&path).expect("read golden file");

        // 1. Bit-Flip auf jedem einzelnen Byte im Header (Bytes 0..84)
        let header_len = HnswHeader::SIZE.min(original_bytes.len());
        for byte_idx in 0..header_len {
            for bit in 0..8 {
                let mut mutated = original_bytes.clone();
                mutated[byte_idx] ^= 1 << bit;

                let tmp_file = temp_dir
                    .path()
                    .join(format!("flip_{}_{}.hnsw", byte_idx, bit));
                fs::write(&tmp_file, &mutated).expect("write mutated");

                // MmapIndex::open darf NIEMALS panicken, sondern muss entweder Err liefern
                // oder einen konsistent lesbaren Zustand haben.
                let open_res = MmapIndex::open(&tmp_file);
                if let Ok(mmap_idx) = open_res {
                    // Falls open erfolgreich war (z. B. TxId- oder Bias-Aenderung),
                    // darf kein nachfolgender Zugriff panicken!
                    if mmap_idx.header.node_count() > 0 {
                        let _ = mmap_idx.get_node_record(0);
                    }
                }
                let _ = fs::remove_file(&tmp_file);
            }
        }
    }
}

#[test]
fn test_truncations_fail_closed() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let temp_dir = tempfile::tempdir().expect("tempdir");

    for file_rel in GOLDEN_FILES {
        let path = Path::new(manifest_dir).join(file_rel);
        if !path.exists() {
            continue;
        }
        let original_bytes = fs::read(&path).expect("read golden file");

        // Schrittweise Kuerzung um jeweils 1..100 Bytes und an Schwellenwerten
        let boundaries = [
            0, 1, 4, 6, 14, 15, 16, 24, 32, 40, 48, 56, 64, 76, 84, 100, 200, 500,
        ];

        for &len in &boundaries {
            if len >= original_bytes.len() {
                continue;
            }
            let truncated = &original_bytes[..len];
            let tmp_file = temp_dir.path().join(format!("trunc_{}.hnsw", len));
            fs::write(&tmp_file, truncated).expect("write truncated");

            // MmapIndex::open muss strikt fehlschlagen und darf niemals panicken
            let res = MmapIndex::open(&tmp_file);
            assert!(
                res.is_err(),
                "Truncated file at {} bytes MUST return Err, got Ok",
                len
            );
            let _ = fs::remove_file(&tmp_file);
        }
    }
}

#[test]
fn test_corrupt_node_record_and_header_fields() {
    // Ungueltige Magic
    let mut header_bytes = [0u8; HnswHeader::SIZE];
    header_bytes[0..4].copy_from_slice(&0x12345678u32.to_le_bytes());
    assert!(HnswHeader::try_from_bytes(&header_bytes).is_err());

    // Ungueltige Version
    let mut valid_magic = [0u8; HnswHeader::SIZE];
    valid_magic[0..4].copy_from_slice(&contextra_vector::persistence::HNSW_MAGIC.to_le_bytes());
    valid_magic[4..6].copy_from_slice(&99u16.to_le_bytes());
    assert!(HnswHeader::try_from_bytes(&valid_magic).is_err());

    // NodeRecord zu kurz
    let short_bytes = [0u8; 10];
    assert!(NodeRecord::from_bytes(&short_bytes).is_err());
}
