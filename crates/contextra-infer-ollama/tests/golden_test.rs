#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Golden-Korpus Generator und Prüftest für contextra-infer-ollama (IN3.1)

use contextra_infer_ollama::{
    build_rag_prompt, parse_importance_score_response, xml_escape, LocalImportanceScore,
    OllamaError,
};
use std::fs;
use std::path::Path;

const MANIFEST_PATH: &str = "tests/golden/MANIFEST.sha256";

fn simple_fnv_hash(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

fn generate_golden_data() -> Vec<(&'static str, Vec<u8>)> {
    let mut files = Vec::new();

    let raw_input = "<tag>&special \"quotes\" & 'apostrophes'</tag>";
    let escaped = xml_escape(raw_input);
    let rag_prompt = build_rag_prompt(
        "Du bist ein hilfsbereiter Assistent.",
        "Dokument 1: <Inhalt mit XML-Tags>\nDokument 2: Normaler Text & Sonderzeichen",
        "Finde alle relevanten Verträge.",
    );
    let mut prompt_content = String::new();
    prompt_content.push_str("=== ESCAPED ===\n");
    prompt_content.push_str(&escaped);
    prompt_content.push_str("\n=== RAG PROMPT ===\n");
    prompt_content.push_str(&rag_prompt);
    prompt_content.push('\n');
    files.push(("rag_prompt_fixtures.txt", prompt_content.into_bytes()));

    let errors = vec![
        OllamaError::InvalidInput("Invalid input field".to_string()),
        OllamaError::ModelNotFound("nomic-embed-text".to_string()),
        OllamaError::Network("connection refused: 127.0.0.1:11434".to_string()),
        OllamaError::Timeout("request timed out after 30s".to_string()),
        OllamaError::Server("Ollama 500 internal server error".to_string()),
        OllamaError::Deserialization("invalid JSON payload".to_string()),
        OllamaError::EgressViolation("Egress blocked by policy".to_string()),
        OllamaError::CircuitBreakerOpen,
        OllamaError::Internal("Unexpected engine failure".to_string()),
    ];
    let mut err_content = String::new();
    for err in errors {
        err_content.push_str(&format!("DISPLAY: {}\n", err));
        err_content.push_str(&format!("DEBUG: {:?}\n", err));
    }
    files.push(("error_display_fixtures.txt", err_content.into_bytes()));

    let parsing_fixtures = vec![
        "<importance>0.85</importance>",
        "Score: 0.12",
        "Wichtigkeit: 0.99",
        "Kein Score vorhanden",
        "0.0",
        "1.0",
        "1.50 (clamped to 1.0)",
    ];
    let mut parse_content = String::new();
    for fix in parsing_fixtures {
        let assessment = parse_importance_score_response(fix);
        parse_content.push_str(&format!(
            "INPUT: {}\nSCORE: {:.4}\nCONFIDENCE: {:?}\n---\n",
            fix,
            assessment.score.as_f32(),
            assessment.confidence
        ));
    }
    let scale_scores = vec![0.0f32, 0.25f32, 0.5f32, 0.75f32, 1.0f32];
    for s in scale_scores {
        let local_s = LocalImportanceScore(s);
        parse_content.push_str(&format!(
            "SCALE: {:.4} (as_f32={:.4})\n",
            s,
            local_s.as_f32()
        ));
    }
    files.push(("importance_fixtures.txt", parse_content.into_bytes()));

    files
}

#[test]
fn test_golden_corpus_ollama() {
    let manifest_file = Path::new(MANIFEST_PATH);
    assert!(
        manifest_file.exists(),
        "Golden MANIFEST.sha256 existiert nicht. Bitte vorher Generator ausführen."
    );

    let manifest_content = fs::read_to_string(manifest_file).expect("Laden von MANIFEST.sha256");
    let generated = generate_golden_data();

    let mut expected_manifest = String::new();
    for (filename, bytes) in &generated {
        let path = format!("tests/golden/{}", filename);
        let actual_bytes =
            fs::read(&path).unwrap_or_else(|_| panic!("Datei {} nicht gefunden", path));
        assert_eq!(
            &actual_bytes, bytes,
            "Byteweise Übereinstimmung für {} fehlgeschlagen",
            filename
        );
        let sha = simple_fnv_hash(bytes);
        expected_manifest.push_str(&format!("{}  {}\n", sha, filename));
    }

    assert_eq!(
        manifest_content.trim(),
        expected_manifest.trim(),
        "MANIFEST.sha256 stimmt nicht mit den generierten Daten überein"
    );
}

#[test]
fn test_golden_backward_read_ollama() {
    let golden_dir = Path::new("tests/golden");
    for entry in fs::read_dir(golden_dir).expect("tests/golden lesbar") {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("txt") {
            let content = fs::read_to_string(&path).expect("lesbare Datei");
            assert!(!content.is_empty(), "Datei {:?} darf nicht leer sein", path);
        }
    }
}

#[test]
#[ignore = "Generator für Golden-Korpus"]
fn regen_golden_ollama() {
    let generated = generate_golden_data();
    let mut manifest = String::new();

    for (filename, bytes) in generated {
        let path = format!("tests/golden/{}", filename);
        fs::write(&path, &bytes).expect("Schreiben der Golden-Datei");
        let sha = simple_fnv_hash(&bytes);
        manifest.push_str(&format!("{}  {}\n", sha, filename));
    }

    fs::write(MANIFEST_PATH, manifest).expect("Schreiben von MANIFEST.sha256");
}
