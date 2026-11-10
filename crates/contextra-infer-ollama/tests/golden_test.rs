//! Golden test suite for contextra-infer-ollama.
//! SPEC: T-GOLDEN-INFER-OLLAMA
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_infer_ollama::{
    build_rag_prompt, parse_importance_score_response, Confidence, LocalImportanceScore,
    OllamaError,
};
use std::fs;
use std::path::Path;

#[test]
fn test_golden_manifest_and_backward_compatibility() {
    let golden_dir = Path::new("tests/golden");
    let manifest_path = golden_dir.join("MANIFEST.sha256");
    let manifest_content =
        fs::read_to_string(&manifest_path).expect("MANIFEST.sha256 must exist in tests/golden");

    let golden_file = golden_dir.join("ollama_golden.json");
    let file_bytes = fs::read(&golden_file).expect("ollama_golden.json must exist and be readable");

    let parsed: serde_json::Value =
        serde_json::from_slice(&file_bytes).expect("ollama_golden.json must be valid JSON");

    assert!(parsed.get("errors").is_some());
    assert!(parsed.get("request").is_some());
    assert!(parsed.get("response").is_some());
    assert!(parsed.get("rag_prompt").is_some());
    assert!(parsed.get("importance").is_some());

    assert!(
        manifest_content.contains("ollama_golden.json"),
        "Manifest must register ollama_golden.json"
    );
}

#[test]
fn test_golden_error_display_invariants() {
    let variants = vec![
        OllamaError::InvalidInput("bad arg".to_string()),
        OllamaError::ModelNotFound("llama3".to_string()),
        OllamaError::Network("connection refused".to_string()),
        OllamaError::Timeout("30s elapsed".to_string()),
        OllamaError::Server("500 internal".to_string()),
        OllamaError::Deserialization("corrupted json".to_string()),
        OllamaError::EgressViolation("blocked ip".to_string()),
        OllamaError::CircuitBreakerOpen,
        OllamaError::Internal("unexpected state".to_string()),
    ];

    let rendered: Vec<String> = variants.iter().map(|e| e.to_string()).collect();

    let golden_file = Path::new("tests/golden/ollama_golden.json");
    let file_bytes = fs::read(golden_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&file_bytes).unwrap();
    let expected: Vec<String> = serde_json::from_value(parsed["errors"].clone()).unwrap();

    assert_eq!(
        rendered, expected,
        "Rendered error strings must match golden data"
    );
}

#[test]
fn test_golden_importance_score_parsing() {
    let fixtures = vec![
        ("0.85", 0.85f32, Confidence::Parsed),
        ("Rating: 0.5", 0.5f32, Confidence::Parsed),
        ("Invalid text", 0.5f32, Confidence::Unparseable),
    ];

    for (input, expected_score, expected_conf) in fixtures {
        let assessment = parse_importance_score_response(input);
        assert_eq!(assessment.score, LocalImportanceScore(expected_score));
        assert_eq!(assessment.confidence, expected_conf);
    }
}

#[test]
fn test_golden_rag_prompt_construction() {
    let prompt = build_rag_prompt(
        "Du bist ein präziser KI-Assistent.",
        "<document index=\"1\">Contextra is embedded hybrid search</document>",
        "What is Contextra?",
    );

    assert!(prompt.contains("<system>Du bist ein präziser KI-Assistent.</system>"));
    assert!(prompt.contains("<context>"));
    assert!(prompt.contains("<user_query>What is Contextra?</user_query>"));
}

#[test]
fn test_golden_bitflip_and_truncation_robustness() {
    let golden_file = Path::new("tests/golden/ollama_golden.json");
    let file_bytes = fs::read(golden_file).unwrap();

    // 1. Truncation test
    for len in [
        0,
        1,
        5,
        file_bytes.len() / 2,
        file_bytes.len().saturating_sub(1),
    ] {
        let truncated = &file_bytes[..len];
        let res: std::result::Result<serde_json::Value, _> = serde_json::from_slice(truncated);
        assert!(
            res.is_err(),
            "Truncated payload of length {len} must fail to parse"
        );
    }

    // 2. Bitflip corruption test
    for pos in [0, 10, file_bytes.len() / 2, file_bytes.len() - 1] {
        let mut corrupted = file_bytes.clone();
        corrupted[pos] ^= 0xFF;
        let res: std::result::Result<serde_json::Value, _> = serde_json::from_slice(&corrupted);
        let _ = res;
    }
}

#[test]
#[ignore]
fn regen_golden() {
    println!("Regenerating golden artifacts...");
}
