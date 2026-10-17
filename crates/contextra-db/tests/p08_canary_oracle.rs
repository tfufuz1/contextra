mod support_canary {
    pub mod canary {
        use std::fmt;

        /// Represents a needle to be searched in binary file streams.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct SearchNeedle {
            pub bytes: Vec<u8>,
            pub encoding: String,
            pub canary_id: String,
            pub owner_person: String,
        }

        /// A hit found by the independent canary scanner.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Hit {
            pub file_path: std::path::PathBuf,
            pub offset: u64,
            pub encoding: String,
            pub canary_id: String,
            pub owner_person: String,
        }

        impl fmt::Display for Hit {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "[Hit person={} canary={} enc={} at {}:{}]",
                    self.owner_person,
                    self.canary_id,
                    self.encoding,
                    self.file_path.display(),
                    self.offset
                )
            }
        }

        /// Helper for generating deterministic 24+ byte entropy canaries and all their encoded representations.
        pub struct PersonCanaryGenerator {
            owner_person: String,
            base_seed: u64,
        }

        impl PersonCanaryGenerator {
            pub fn new(owner_person: &str, base_seed: u64) -> Self {
                Self {
                    owner_person: owner_person.to_string(),
                    base_seed,
                }
            }

            /// Generates a unique 24+ byte entropy string canary for text/metadata.
            pub fn generate_text_canary(&self, label: &str) -> String {
                let mut state = self.base_seed.wrapping_add(hash_label(label));
                let mut bytes = Vec::with_capacity(32);
                for _ in 0..4 {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    bytes.extend_from_slice(&state.to_le_bytes());
                }
                let hex_str: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                format!("CANARY_{}_{}_{}", self.owner_person, label, hex_str)
            }

            /// Generates a vector with unique f32 components for vector canary.
            pub fn generate_vector_canary(&self, label: &str, dim: usize) -> Vec<f32> {
                let mut state = self.base_seed.wrapping_add(hash_label(label));
                let mut vec = Vec::with_capacity(dim);
                for i in 0..dim {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407 + (i as u64));
                    let val = ((state >> 32) as f32) / (u32::MAX as f32) + (i as f32 + 1.0) * 100.0;
                    vec.push(val);
                }
                vec
            }

            /// Generates all multi-encoding needles for a text canary string.
            pub fn build_text_needles(&self, canary_str: &str, label: &str) -> Vec<SearchNeedle> {
                let canary_id = format!("{}_{}", self.owner_person, label);
                let mut needles = Vec::new();

                let raw_bytes = canary_str.as_bytes().to_vec();
                needles.push(SearchNeedle {
                    bytes: raw_bytes.clone(),
                    encoding: "raw_utf8".to_string(),
                    canary_id: canary_id.clone(),
                    owner_person: self.owner_person.clone(),
                });

                let json_escaped = serde_json::to_string(canary_str)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .as_bytes()
                    .to_vec();
                if json_escaped != raw_bytes {
                    needles.push(SearchNeedle {
                        bytes: json_escaped,
                        encoding: "json_escaped".to_string(),
                        canary_id: canary_id.clone(),
                        owner_person: self.owner_person.clone(),
                    });
                }

                let hex_lower: String = raw_bytes.iter().map(|b| format!("{:02x}", b)).collect();
                needles.push(SearchNeedle {
                    bytes: hex_lower.as_bytes().to_vec(),
                    encoding: "hex_lower".to_string(),
                    canary_id: canary_id.clone(),
                    owner_person: self.owner_person.clone(),
                });

                let hex_upper: String = raw_bytes.iter().map(|b| format!("{:02X}", b)).collect();
                needles.push(SearchNeedle {
                    bytes: hex_upper.as_bytes().to_vec(),
                    encoding: "hex_upper".to_string(),
                    canary_id: canary_id.clone(),
                    owner_person: self.owner_person.clone(),
                });

                let b64_shift0 = base64_encode(&raw_bytes);
                needles.push(SearchNeedle {
                    bytes: b64_shift0.as_bytes().to_vec(),
                    encoding: "base64_shift0".to_string(),
                    canary_id: canary_id.clone(),
                    owner_person: self.owner_person.clone(),
                });

                let mut bytes_shift1 = vec![0x00];
                bytes_shift1.extend_from_slice(&raw_bytes);
                let b64_full1 = base64_encode(&bytes_shift1);
                if b64_full1.len() > 4 {
                    let b64_shift1 = b64_full1[2..b64_full1.len() - 2].to_string();
                    needles.push(SearchNeedle {
                        bytes: b64_shift1.as_bytes().to_vec(),
                        encoding: "base64_shift1".to_string(),
                        canary_id: canary_id.clone(),
                        owner_person: self.owner_person.clone(),
                    });
                }

                let mut bytes_shift2 = vec![0x00, 0x00];
                bytes_shift2.extend_from_slice(&raw_bytes);
                let b64_full2 = base64_encode(&bytes_shift2);
                if b64_full2.len() > 4 {
                    let b64_shift2 = b64_full2[3..b64_full2.len() - 2].to_string();
                    needles.push(SearchNeedle {
                        bytes: b64_shift2.as_bytes().to_vec(),
                        encoding: "base64_shift2".to_string(),
                        canary_id: canary_id.clone(),
                        owner_person: self.owner_person.clone(),
                    });
                }

                let utf16_le: Vec<u8> = canary_str
                    .encode_utf16()
                    .flat_map(|u| u.to_le_bytes())
                    .collect();
                needles.push(SearchNeedle {
                    bytes: utf16_le,
                    encoding: "utf16_le".to_string(),
                    canary_id: canary_id.clone(),
                    owner_person: self.owner_person.clone(),
                });

                let utf16_be: Vec<u8> = canary_str
                    .encode_utf16()
                    .flat_map(|u| u.to_be_bytes())
                    .collect();
                needles.push(SearchNeedle {
                    bytes: utf16_be,
                    encoding: "utf16_be".to_string(),
                    canary_id: canary_id.clone(),
                    owner_person: self.owner_person.clone(),
                });

                let lower_str = canary_str.to_lowercase();
                if lower_str.as_bytes() != raw_bytes {
                    needles.push(SearchNeedle {
                        bytes: lower_str.as_bytes().to_vec(),
                        encoding: "lowercase_utf8".to_string(),
                        canary_id: canary_id.clone(),
                        owner_person: self.owner_person.clone(),
                    });
                }

                if raw_bytes.len() >= 12 {
                    let fragment = raw_bytes[4..16].to_vec();
                    needles.push(SearchNeedle {
                        bytes: fragment,
                        encoding: "fragment_12b".to_string(),
                        canary_id: canary_id.clone(),
                        owner_person: self.owner_person.clone(),
                    });
                }

                needles
            }

            /// Generates search needles for a vector canary.
            pub fn build_vector_needles(&self, vec_canary: &[f32], label: &str) -> Vec<SearchNeedle> {
                let canary_id = format!("{}_{}_vec", self.owner_person, label);
                let mut bytes = Vec::with_capacity(vec_canary.len() * 4);
                for &val in vec_canary {
                    bytes.extend_from_slice(&val.to_le_bytes());
                }

                let mut needles = Vec::new();
                needles.push(SearchNeedle {
                    bytes: bytes.clone(),
                    encoding: "f32_le_vector".to_string(),
                    canary_id: canary_id.clone(),
                    owner_person: self.owner_person.clone(),
                });

                if bytes.len() >= 12 {
                    needles.push(SearchNeedle {
                        bytes: bytes[..12].to_vec(),
                        encoding: "f32_le_vec_frag12b".to_string(),
                        canary_id,
                        owner_person: self.owner_person.clone(),
                    });
                }

                needles
            }
        }

        fn hash_label(label: &str) -> u64 {
            let mut h = 0xcbf29ce484222325u64;
            for b in label.as_bytes() {
                h ^= u64::from(*b);
                h = h.wrapping_mul(0x100000001b3);
            }
            h
        }

        fn base64_encode(data: &[u8]) -> String {
            const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

            for chunk in data.chunks(3) {
                let b0 = u32::from(chunk[0]);
                let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
                let b2 = u32::from(*chunk.get(2).unwrap_or(&0));

                let triple = (b0 << 16) | (b1 << 8) | b2;

                result.push(B64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
                result.push(B64_CHARS[((triple >> 12) & 0x3F) as usize] as char);

                if chunk.len() > 1 {
                    result.push(B64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
                } else {
                    result.push('=');
                }

                if chunk.len() > 2 {
                    result.push(B64_CHARS[(triple & 0x3F) as usize] as char);
                } else {
                    result.push('=');
                }
            }

            result
        }
    }

    pub mod injector {
        use std::fs::{self, OpenOptions};
        use std::io::{Result as IoResult, Write};
        use std::path::Path;

        /// Injects a byte needle into a target file.
        #[allow(dead_code)]
        pub fn inject_needle_into_file(file_path: &Path, needle_bytes: &[u8]) -> IoResult<()> {
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(file_path)?;

            file.write_all(needle_bytes)?;
            file.flush()?;
            Ok(())
        }
    }

    pub mod scanner {
        use super::canary::{Hit, SearchNeedle};
        use std::fs::{self, File};
        use std::io::{Read, Result as IoResult};
        use std::path::Path;

        const CHUNK_SIZE: usize = 64 * 1024;

        /// Recursively scans a directory for occurrences of any `SearchNeedle`.
        pub fn scan_dir(root: &Path, needles: &[SearchNeedle]) -> Vec<Hit> {
            let mut hits = Vec::new();
            if needles.is_empty() || !root.exists() {
                return hits;
            }

            let max_needle_len = needles.iter().map(|n| n.bytes.len()).max().unwrap_or(0);
            if max_needle_len == 0 {
                return hits;
            }

            let mut stack = vec![root.to_path_buf()];

            while let Some(current_path) = stack.pop() {
                if current_path.is_dir() {
                    if let Ok(entries) = fs::read_dir(&current_path) {
                        for entry in entries.flatten() {
                            stack.push(entry.path());
                        }
                    }
                } else if current_path.is_file() {
                    if let Ok(file_hits) = scan_file(&current_path, needles, max_needle_len) {
                        hits.extend(file_hits);
                    }
                }
            }

            hits
        }

        /// Scans a single file for occurrences of any `SearchNeedle`.
        pub fn scan_file(
            file_path: &Path,
            needles: &[SearchNeedle],
            max_needle_len: usize,
        ) -> IoResult<Vec<Hit>> {
            let mut hits = Vec::new();
            let mut file = File::open(file_path)?;

            let mut buffer = vec![0u8; CHUNK_SIZE];
            let mut carryover = Vec::new();
            let mut file_offset: u64 = 0;

            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }

                let carryover_len = carryover.len();
                let mut active_chunk = Vec::with_capacity(carryover_len + bytes_read);
                active_chunk.extend_from_slice(&carryover);
                active_chunk.extend_from_slice(&buffer[..bytes_read]);

                let chunk_base_offset = file_offset.saturating_sub(carryover_len as u64);

                for needle in needles {
                    if needle.bytes.is_empty() {
                        continue;
                    }
                    let n_len = needle.bytes.len();
                    if active_chunk.len() < n_len {
                        continue;
                    }

                    for idx in 0..=(active_chunk.len() - n_len) {
                        if carryover_len > 0 && idx + n_len <= carryover_len {
                            continue;
                        }

                        if &active_chunk[idx..idx + n_len] == needle.bytes.as_slice() {
                            let hit_offset = chunk_base_offset + idx as u64;
                            hits.push(Hit {
                                file_path: file_path.to_path_buf(),
                                offset: hit_offset,
                                encoding: needle.encoding.clone(),
                                canary_id: needle.canary_id.clone(),
                                owner_person: needle.owner_person.clone(),
                            });
                        }
                    }
                }

                file_offset += bytes_read as u64;

                let overlap_len = max_needle_len.saturating_sub(1);
                if active_chunk.len() >= overlap_len {
                    carryover = active_chunk[active_chunk.len() - overlap_len..].to_vec();
                } else {
                    carryover = active_chunk;
                }
            }

            Ok(hits)
        }
    }

    pub use canary::PersonCanaryGenerator;
    #[allow(unused_imports)]
    pub use injector::inject_needle_into_file;
    pub use scanner::scan_dir;
}

use std::fs::{self, File};
use std::io::Write;
use support_canary::{inject_needle_into_file, scan_dir, PersonCanaryGenerator};
use tempfile::TempDir;

/// Phase 2.1: Injection Matrix Test over all file families from KEYFAMILIES.toml
#[test]
fn test_injection_matrix_all_file_families() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let gen = PersonCanaryGenerator::new("TestPersonFamilies", 0x1234_5678);
    let canary_str = gen.generate_text_canary("family_test");
    let needles = gen.build_text_needles(&canary_str, "family_test");

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

        let mut file = File::create(&file_path).expect("create file");
        file.write_all(format!("HEADER_{}_PADDING_", desc).as_bytes())
            .expect("write header");
        file.flush().expect("flush file");

        inject_needle_into_file(&file_path, canary_str.as_bytes()).expect("inject canary");
        inject_needle_into_file(&file_path, b"_TRAILING_DUMMY_DATA").expect("append trailing");

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

    for (i, needle) in all_needles.iter().enumerate() {
        let file_name = format!("file_enc_{}_{}.dat", i, needle.encoding);
        let file_path = temp_dir.path().join(&file_name);

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
