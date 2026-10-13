#[path = "support_canary/mod.rs"]
mod support_canary;

use std::fs::{self, File};
use std::io::Write;
use support_canary::{inject_needle_into_file, scan_dir, PersonCanaryGenerator};
use tempfile::TempDir;

/// Phase 2.1: Injection Matrix Test over all file families from KEYFAMILIES.toml
/// WAL, SSTable, Manifest, Text index, Graph index, Temp files, Checkpoints, Audit files, Unknown files.
#[test]
fn test_injection_matrix_all_file_families() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gen = PersonCanaryGenerator::new("TestPersonFamilies", 0x1234_5678);
    let canary_str = gen.generate_text_canary("family_test");
    let needles = gen.build_text_needles(&canary_str, "family_test");

    // File family representations based on KEYFAMILIES.toml
    let file_families = vec![
        ("wal/000001.wal", "WAL log segment"),
        ("sstables/000001.sst", "LSM SSTable file"),
        ("manifest/MANIFEST-000001", "Manifest file"),
        ("index/text_bm25.idx", "Text BM25 index file"),
        ("index/hnsw_vectors.bin", "HNSW vector index file"),
        ("index/graph_csr.adj", "Graph CSR adjacency file"),
        ("tmp/compaction_scratch.tmp", "Temporary scratch file"),
        ("checkpoints/chk_001.meta", "Checkpoint metadata file"),
        ("audit/revocation.log", "Audit revocation log file"),
        ("unknown/custom_extra.dat", "Unknown extra file family"),
    ];

    for (rel_path, desc) in &file_families {
        let file_path = temp_dir.path().join(rel_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }

        // Create base dummy content
        let mut file = File::create(&file_path).expect("create file");
        file.write_all(format!("HEADER_{}_PADDING_", desc).as_bytes())
            .expect("write header");
        file.flush().expect("flush file");

        // Inject raw canary needle into the file copy
        inject_needle_into_file(&file_path, canary_str.as_bytes()).expect("inject canary");

        // Append trailing content
        inject_needle_into_file(&file_path, b"_TRAILING_DUMMY_DATA").expect("append trailing");

        // Run independent scanner
        let hits = scan_dir(temp_dir.path().join(rel_path).parent().unwrap(), &needles);

        assert!(
            !hits.is_empty(),
            "INJECTION MATRIX FAILURE: Canary was NOT found in file family '{}' ({}) at path {}",
            desc,
            rel_path,
            file_path.display()
        );
    }
}

/// Phase 2.2: Encoding Matrix Test for all supported encoding variants
#[test]
fn test_encoding_matrix_all_encodings() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gen = PersonCanaryGenerator::new("TestPersonEncoding", 0x8765_4321);
    let canary_str = gen.generate_text_canary("enc_test");
    let needles = gen.build_text_needles(&canary_str, "enc_test");

    let vec_canary = gen.generate_vector_canary("vec_enc_test", 4);
    let vec_needles = gen.build_vector_needles(&vec_canary, "vec_enc_test");

    let mut all_needles = needles;
    all_needles.extend(vec_needles);

    // Verify each needle encoding individually
    for (i, needle) in all_needles.iter().enumerate() {
        let file_name = format!("file_enc_{}_{}.dat", i, needle.encoding);
        let file_path = temp_dir.path().join(&file_name);

        // Create file with padding
        let mut file = File::create(&file_path).expect("create enc test file");
        file.write_all(b"HEADER_PADDING_BYTES_").expect("write hdr");
        file.write_all(&needle.bytes).expect("write needle bytes");
        file.write_all(b"_FOOTER_PADDING_BYTES").expect("write ftr");
        file.flush().expect("flush enc file");

        let hits = scan_dir(temp_dir.path(), &[needle.clone()]);
        assert!(
            !hits.is_empty(),
            "ENCODING MATRIX FAILURE: Canary needle with encoding '{}' (id: '{}') was NOT detected by scanner in file {}",
            needle.encoding,
            needle.canary_id,
            file_path.display()
        );

        // Verify hit properties
        let hit = &hits[0];
        assert_eq!(hit.encoding, needle.encoding);
        assert_eq!(hit.canary_id, needle.canary_id);
    }
}

/// Phase 2.3: Counter-probe (Gegenprobe) - Clean directory must yield 0 hits
#[test]
fn test_counter_probe_clean_directory_yields_zero_hits() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gen = PersonCanaryGenerator::new("TestPersonClean", 0x9999_8888);
    let canary_str = gen.generate_text_canary("clean_test");
    let needles = gen.build_text_needles(&canary_str, "clean_test");

    // Populate directory with dummy clean database files
    let clean_file1 = temp_dir.path().join("sstables/000001.sst");
    fs::create_dir_all(clean_file1.parent().unwrap()).expect("create sst dir");
    fs::write(
        &clean_file1,
        b"COMPLETELY_UNRELATED_DATABASE_HEADER_AND_PAYLOAD_DATA_9999",
    )
    .expect("write clean1");

    let clean_file2 = temp_dir.path().join("wal/000001.wal");
    fs::create_dir_all(clean_file2.parent().unwrap()).expect("create wal dir");
    fs::write(
        &clean_file2,
        b"WAL_LOG_RECORD_SEQUENCE_NUMBER_100_COMMIT_INTENT_SUCCESS",
    )
    .expect("write clean2");

    let hits = scan_dir(temp_dir.path(), &needles);
    assert_eq!(
        hits.len(),
        0,
        "COUNTER-PROBE FAILURE: Clean directory yielded false positive hits: {:?}",
        hits
    );
}

/// Boundary straddling test: needle straddling 64 KiB chunk boundary
#[test]
fn test_scanner_detects_canary_straddling_chunk_boundary() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gen = PersonCanaryGenerator::new("TestPersonStraddle", 0x5555_4444);
    let canary_str = gen.generate_text_canary("straddle_test");
    let needles = gen.build_text_needles(&canary_str, "straddle_test");

    let chunk_size = 64 * 1024;
    let straddle_file = temp_dir.path().join("straddle_boundary.dat");
    let mut file = File::create(&straddle_file).expect("create straddle file");

    // Write padding so needle starts 10 bytes before the 64 KiB chunk boundary
    let padding = vec![0x37u8; chunk_size - 10];
    file.write_all(&padding).expect("write padding");
    file.write_all(canary_str.as_bytes()).expect("write canary");
    file.write_all(&padding).expect("write trailing padding");
    file.flush().expect("flush straddle file");

    let hits = scan_dir(temp_dir.path(), &needles);
    assert!(
        !hits.is_empty(),
        "BOUNDARY STRADDLE FAILURE: Canary crossing 64 KiB chunk boundary was NOT detected by scanner!"
    );
}
