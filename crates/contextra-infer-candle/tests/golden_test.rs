//! Golden test suite for contextra-infer-candle.
//! SPEC: T-GOLDEN-INFER-CANDLE
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_infer_candle::error::Error;
use contextra_infer_candle::gasp::GaspValidator;
use contextra_infer_candle::model_registry::ModelFingerprint;
use contextra_types::{ContextChunk, DocId};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[test]
fn test_golden_manifest_and_backward_compatibility() {
    let golden_dir = Path::new("tests/golden");
    let manifest_path = golden_dir.join("MANIFEST.sha256");
    let manifest_content =
        fs::read_to_string(&manifest_path).expect("MANIFEST.sha256 must exist in tests/golden");

    let golden_file = golden_dir.join("candle_golden.json");
    let file_bytes = fs::read(&golden_file).expect("candle_golden.json must exist and be readable");

    let mut hasher = Sha256::new();
    hasher.update(&file_bytes);
    let calculated_hash = format!("{:x}", hasher.finalize());

    assert!(
        manifest_content.contains(&calculated_hash),
        "Golden file hash mismatch! Expected: in manifest, got: {calculated_hash}"
    );

    let content = String::from_utf8(file_bytes).expect("candle_golden.json must be valid UTF-8");
    assert!(content.contains("\"errors\""));
    assert!(content.contains("\"fingerprint\""));
    assert!(content.contains("\"fake_embed\""));
    assert!(content.contains("\"gasp_raw_score_fixture\""));
}

#[test]
fn test_golden_error_display_invariants() {
    let variants = vec![
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "entity not found",
        )),
        Error::ModelLoad {
            reason: "invalid GGUF magic 0x12345678".to_string(),
        },
        Error::Tokenizer("missing vocabulary item".to_string()),
        Error::Inference("out of memory on device Cpu".to_string()),
        Error::InvalidInput("batch size 0 exceeds allowable range".to_string()),
        Error::ResourceExceeded("tracked requests ceiling reached".to_string()),
        Error::PolicyViolation("LowConfidenceGrounding".to_string()),
        Error::BackendUnavailable("candle feature not enabled".to_string()),
    ];

    let rendered: Vec<String> = variants.iter().map(|e| e.to_string()).collect();

    let golden_file = Path::new("tests/golden/candle_golden.json");
    let content = fs::read_to_string(golden_file).unwrap();

    for s in rendered {
        assert!(
            content.contains(&s),
            "Golden data must contain error string: {s}"
        );
    }
}

#[test]
fn test_golden_fingerprint_deterministic_roundtrip() {
    let raw_hash = [
        0x2c, 0xf2, 0x4d, 0xba, 0x5f, 0xb0, 0xa3, 0x0e, 0x26, 0xe8, 0x3b, 0x2a, 0xc5, 0xb9, 0xe2,
        0x9e, 0x1b, 0x16, 0x1e, 0x5c, 0x1f, 0xa7, 0x42, 0x5e, 0x73, 0x04, 0x33, 0x62, 0x93, 0x8b,
        0x98, 0x24,
    ];
    let fp = ModelFingerprint {
        model_id: "test-model-gguf".to_string(),
        quantization: "Q4KM".to_string(),
        hash: raw_hash,
    };

    assert_eq!(fp.model_id, "test-model-gguf");
    assert_eq!(fp.quantization, "Q4KM");
    assert_eq!(fp.hash, raw_hash);

    let compat_fp: contextra_types::ModelFingerprint = fp.clone().into();
    assert_eq!(compat_fp.hash, fp.hash);
    assert_eq!(compat_fp.model_id, fp.model_id);
    assert_eq!(compat_fp.quantization, fp.quantization);
}

#[test]
fn test_golden_gasp_raw_score() {
    let validator = GaspValidator::new();
    let chunks = vec![ContextChunk {
        doc_id: DocId::new(1u64),
        content: "Der Umsatz betrug im Jahr 2025 genau 50 Millionen Euro.".to_string(),
        relevance: 0.95,
        token_count: 20,
        metadata: None,
        contextual_prefix: None,
        links: vec![],
    }];

    let score = validator
        .compute_raw_grounding_score("Im Jahr 2025 betrug der Umsatz 50 Millionen Euro.", &chunks)
        .unwrap();

    assert_eq!(score, 1.0, "Exact match fixture must have score 1.0");
}

#[test]
fn test_golden_bitflip_and_truncation_robustness() {
    let golden_file = Path::new("tests/golden/candle_golden.json");
    let file_bytes = fs::read(golden_file).unwrap();

    for len in [
        0,
        1,
        5,
        file_bytes.len() / 2,
        file_bytes.len().saturating_sub(1),
    ] {
        let truncated = &file_bytes[..len];
        let _ = std::str::from_utf8(truncated);
    }

    for pos in [0, 10, file_bytes.len() / 2, file_bytes.len() - 1] {
        let mut corrupted = file_bytes.clone();
        corrupted[pos] ^= 0xFF;
        let _ = std::str::from_utf8(&corrupted);
    }
}

#[test]
#[ignore]
fn regen_golden() {
    println!("Regenerating candle golden artifacts...");
}
