#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Golden-Korpus Generator und Prüftest für contextra-infer-candle (IN3.1)

use contextra_infer_candle::gasp::GaspValidator;
use contextra_infer_candle::model_registry::{compute_fingerprint, CandleQuantization};
use contextra_infer_candle::traits::{Embedder, TextGenerator};
use contextra_infer_candle::{FakeEmbedder, FakeTextGenerator, GenerationConfig};
use contextra_types::ContextChunk;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const MANIFEST_PATH: &str = "tests/golden/MANIFEST.sha256";

fn compute_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn generate_golden_data() -> Vec<(&'static str, Vec<u8>)> {
    let mut files = Vec::new();

    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let model_path = tmp_dir.path().join("mock_weights.gguf");
    {
        let mut f = File::create(&model_path).expect("create mock model");
        f.write_all(b"GGUF_MOCK_WEIGHTS_DETERMINISTIC_BLOB_BYTE_STREAM_1234567890")
            .expect("write mock");
    }

    let fp_q4 = compute_fingerprint(&model_path, &CandleQuantization::Q4KM).expect("fp q4");
    let fp_q8 = compute_fingerprint(&model_path, &CandleQuantization::Q8_0).expect("fp q8");
    let fp_f16 = compute_fingerprint(&model_path, &CandleQuantization::F16).expect("fp f16");

    let mut fp_content = String::new();
    fp_content.push_str(&format!(
        "Q4KM: hash={:x?} quant={} model={}\n",
        fp_q4.hash, fp_q4.quantization, fp_q4.model_id
    ));
    fp_content.push_str(&format!(
        "Q8_0: hash={:x?} quant={} model={}\n",
        fp_q8.hash, fp_q8.quantization, fp_q8.model_id
    ));
    fp_content.push_str(&format!(
        "F16: hash={:x?} quant={} model={}\n",
        fp_f16.hash, fp_f16.quantization, fp_f16.model_id
    ));
    files.push(("model_fingerprints.txt", fp_content.into_bytes()));

    let fake_embedder = FakeEmbedder::new(16);
    let embedding_out = fake_embedder
        .embed("Contextra Retrieval Engine Text")
        .expect("embed");
    let fake_generator = FakeTextGenerator::new("Antwort auf Prompt: ");
    let text_out = fake_generator
        .generate("Was ist Hybrid-Retrieval?", &GenerationConfig::default())
        .expect("generate");

    let mut fake_content = String::new();
    fake_content.push_str("EMBEDDING:\n");
    for val in &embedding_out {
        fake_content.push_str(&format!("{:.6}\n", val));
    }
    fake_content.push_str("GENERATION:\n");
    fake_content.push_str(&text_out);
    fake_content.push('\n');
    files.push(("fake_provider_fixtures.txt", fake_content.into_bytes()));

    let validator = GaspValidator::new();
    let chunks = vec![
        ContextChunk {
            doc_id: contextra_types::DocId::from(1u64),
            content: "Der Umsatz stieg im Geschäftsjahr 2024 auf 15 Millionen Euro.".to_string(),
            relevance: 1.0,
            token_count: 10,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
        ContextChunk {
            doc_id: contextra_types::DocId::from(2u64),
            content: "Die Mitarbeiterzahl betrug genau 42 Spezialisten.".to_string(),
            relevance: 0.9,
            token_count: 8,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        },
    ];

    let supported_score = validator
        .compute_raw_grounding_score(
            "Im Geschäftsjahr 2024 betrug der Umsatz 15 Millionen Euro.",
            &chunks,
        )
        .expect("supported score");
    let hallucinated_score = validator
        .compute_raw_grounding_score(
            "Der Gewinn lag im Jahr 2024 bei 99 Milliarden Dollar.",
            &chunks,
        )
        .expect("hallucinated score");

    let mut gasp_content = String::new();
    gasp_content.push_str(&format!("SUPPORTED_SCORE: {:.6}\n", supported_score));
    gasp_content.push_str(&format!("HALLUCINATED_SCORE: {:.6}\n", hallucinated_score));
    files.push(("gasp_fixtures.txt", gasp_content.into_bytes()));

    files
}

#[test]
fn test_golden_corpus_candle() {
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
        let sha = compute_sha256(bytes);
        expected_manifest.push_str(&format!("{}  {}\n", sha, filename));
    }

    assert_eq!(
        manifest_content.trim(),
        expected_manifest.trim(),
        "MANIFEST.sha256 stimmt nicht mit den generierten Daten überein"
    );
}

#[test]
fn test_golden_backward_read_candle() {
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
fn regen_golden_candle() {
    let generated = generate_golden_data();
    let mut manifest = String::new();

    for (filename, bytes) in generated {
        let path = format!("tests/golden/{}", filename);
        fs::write(&path, &bytes).expect("Schreiben der Golden-Datei");
        let sha = compute_sha256(&bytes);
        manifest.push_str(&format!("{}  {}\n", sha, filename));
    }

    fs::write(MANIFEST_PATH, manifest).expect("Schreiben von MANIFEST.sha256");
}
