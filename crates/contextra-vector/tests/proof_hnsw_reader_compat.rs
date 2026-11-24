// FILE-CONTEXT
// ZWECK: Verifiziert, dass bestehende v2-Dateien (hnsw_raw.bin, hnsw_sq8.bin) auch nach HNSW_VERSION-Bump auf 3 korrekt geladen und gesucht werden können (Reader-Kompatibilität).

use std::path::PathBuf;
use contextra_vector::persistence::MmapIndex;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

#[test]
fn test_proof_hnsw_reader_compat_v2_headers() {
    let raw_file = golden_dir().join("hnsw_raw.bin");
    let sq8_file = golden_dir().join("hnsw_sq8.bin");

    assert!(raw_file.exists());
    assert!(sq8_file.exists());

    let mmap_raw = MmapIndex::open(&raw_file).expect("open hnsw_raw.bin v2");
    assert_eq!(mmap_raw.header.node_count(), 20);
    assert_eq!(mmap_raw.header.dimension(), 16);
    assert!(!mmap_raw.header.is_quantized());

    for i in 0..20 {
        let node = mmap_raw.get_node_record(i).expect("get node record");
        assert!(node.doc_id >= 1 && node.doc_id <= 20);
        let vec_bytes = mmap_raw.get_vector(&node).expect("get vector bytes");
        assert_eq!(vec_bytes.len(), 16 * 4);
    }

    let mmap_sq8 = MmapIndex::open(&sq8_file).expect("open hnsw_sq8.bin v2");
    assert_eq!(mmap_sq8.header.node_count(), 20);
    assert_eq!(mmap_sq8.header.dimension(), 16);
    assert!(mmap_sq8.header.is_quantized());

    for i in 0..20 {
        let node = mmap_sq8.get_node_record(i).expect("get sq8 node record");
        assert!(node.doc_id >= 1 && node.doc_id <= 20);
        let vec_bytes = mmap_sq8.get_vector(&node).expect("get sq8 vector bytes");
        assert_eq!(vec_bytes.len(), 16);
    }
}
