// FILE-CONTEXT
// ZWECK: Smoke-Treiber zur Ausführung der Fuzzing-Logik gegen alle Korpusdateien ohne libfuzzer/cargo-fuzz Laufzeitabhängigkeit.

use std::fs;
use std::path::PathBuf;
use contextra_vector::persistence::HnswHeader;
use contextra_vector::quantize::ScalarQuantizer;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz")
        .join("corpus")
}

#[test]
fn test_smoke_fuzz_hnsw_header_versions() {
    let dir = corpus_dir().join("fuzz_hnsw_header_versions");
    if dir.exists() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let data = fs::read(&path).unwrap();
                if let Ok(header) = HnswHeader::try_from_bytes(&data) {
                    let serialized = header.to_bytes();
                    let _ = header.checksum();
                    let _ = HnswHeader::try_from_bytes(&serialized);
                }
            }
        }
    }
}

#[test]
fn test_smoke_fuzz_sq8_quantizer() {
    let dir = corpus_dir().join("fuzz_sq8_quantizer");
    if dir.exists() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let data = fs::read(&path).unwrap();
                if data.len() >= 16 {
                    let dim = (data[0] as usize % 64) + 1;
                    let floats: Vec<f32> = data[1..]
                        .chunks_exact(4)
                        .map(|chunk| {
                            let mut arr = [0u8; 4];
                            arr.copy_from_slice(chunk);
                            f32::from_le_bytes(arr)
                        })
                        .collect();
                    if floats.len() >= dim {
                        let refs: Vec<&[f32]> = floats.chunks_exact(dim).collect();
                        if !refs.is_empty() {
                            let quantizer = ScalarQuantizer::train(&refs, dim);
                            if let Ok(quantized) = quantizer.quantize(refs[0]) {
                                let _ = quantizer.dequantize(&quantized);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "experimental-diskann")]
#[test]
fn test_smoke_fuzz_diskann_formats() {
    let dir = corpus_dir().join("fuzz_diskann_formats");
    if dir.exists() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let data = fs::read(&path).unwrap();
                if data.len() >= 5 {
                    if let Ok(temp_dir) = tempfile::tempdir() {
                        let wal_pwal = temp_dir.path().join("fuzz.pending.wal");
                        let wal_twal = temp_dir.path().join("fuzz.tombstone.wal");

                        let _ = fs::write(&wal_pwal, &data);
                        let _ = fs::write(&wal_twal, &data);

                        let config = contextra_vector::diskann::DiskAnnConfig {
                            dimension: 16,
                            max_degree: 8,
                            beam_width: 16,
                            index_path: temp_dir.path().join("fuzz.dann"),
                            ..Default::default()
                        };
                        if let Ok(index) = contextra_vector::diskann::DiskAnnIndex::try_new(config) {
                            let _ = index.recover_pending_delta_sync();
                        }
                    }
                }
            }
        }
    }
}
