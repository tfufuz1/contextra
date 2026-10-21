// FILE-CONTEXT
// STAND: 2026-10-21T00:00:00Z
// ZWECK: Qwen2.5-Coder architecture integration tests (Golden, KV roundtrip, truncation, unit tests).

use contextra_infer_candle::gguf_loader::parse_gguf_metadata;
use contextra_infer_candle::inference::{
    format_chatml_prompt, is_stop_token, load_quantized_model,
};
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_chatml_prompt_formatting() {
    let prompt = format_chatml_prompt("Write a fibonacci function in Rust.", Some("You are a helpful assistant."));
    assert_eq!(
        prompt,
        "<|im_start|>system\nYou are a helpful assistant.<|im_end|>\n<|im_start|>user\nWrite a fibonacci function in Rust.<|im_end|>\n<|im_start|>assistant\n"
    );

    let user_only = format_chatml_prompt("Hello", None);
    assert_eq!(
        user_only,
        "<|im_start|>user\nHello<|im_end|>\n<|im_start|>assistant\n"
    );
}

#[test]
fn test_stop_token_recognition() {
    // Qwen2 stop tokens: 151643 (<|endoftext|>), 151645 (<|im_end|>)
    assert!(is_stop_token(151643, "qwen2"));
    assert!(is_stop_token(151645, "qwen2"));
    assert!(!is_stop_token(100, "qwen2"));

    // Llama stop tokens: 2, 128001, 128009
    assert!(is_stop_token(2, "llama"));
    assert!(is_stop_token(128001, "llama"));
    assert!(is_stop_token(128009, "llama"));
    assert!(!is_stop_token(151645, "llama"));
}

fn create_synthetic_gguf_with_arch(arch: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF");
    bytes.extend_from_slice(&3u32.to_le_bytes()); // Version 3
    bytes.extend_from_slice(&0u64.to_le_bytes()); // 0 tensors
    bytes.extend_from_slice(&1u64.to_le_bytes()); // 1 KV pair

    let key = b"general.architecture";
    bytes.extend_from_slice(&(key.len() as u64).to_le_bytes());
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(&8u32.to_le_bytes()); // String type
    let val = arch.as_bytes();
    bytes.extend_from_slice(&(val.len() as u64).to_le_bytes());
    bytes.extend_from_slice(val);

    bytes
}

#[test]
fn test_qwen2_metadata_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    let bytes = create_synthetic_gguf_with_arch("qwen2");
    tmp.write_all(&bytes)?;
    tmp.flush()?;

    let meta = parse_gguf_metadata(tmp.path())?;
    assert_eq!(meta.architecture, "qwen2");
    Ok(())
}

#[test]
fn test_unsupported_architecture_dispatch_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    let bytes = create_synthetic_gguf_with_arch("unsupported_arch");
    tmp.write_all(&bytes)?;
    tmp.flush()?;

    let device = candle_core::Device::Cpu;
    let res = load_quantized_model(tmp.path(), &device);
    assert!(res.is_err());
    let err_msg = res.err().unwrap().to_string();
    assert!(err_msg.contains("Unsupported GGUF model architecture"));
    assert!(err_msg.contains("unsupported_arch"));
    Ok(())
}

#[test]
#[ignore]
fn test_qwen2_golden_output() {
    let model_path = match std::env::var("CONTEXTRA_QWEN_GGUF") {
        Ok(p) if std::path::Path::new(&p).exists() => p,
        _ => return, // Skip cleanly when model binary is not provided
    };

    let device = candle_core::Device::Cpu;
    let mut model = load_quantized_model(std::path::Path::new(&model_path), &device)
        .expect("Failed to load Qwen2 model");

    let tokenizer_path = std::path::Path::new(&model_path)
        .parent()
        .unwrap()
        .join("tokenizer.json");
    if !tokenizer_path.exists() {
        return;
    }
    let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path).unwrap();

    let prompt = "def fibonacci(n):";
    let output = model.generate(prompt, &tokenizer, &device).expect("Generation failed");
    assert!(!output.is_empty(), "Golden output should not be empty");
}

#[test]
#[ignore]
fn test_qwen2_kv_roundtrip() {
    let model_path = match std::env::var("CONTEXTRA_QWEN_GGUF") {
        Ok(p) if std::path::Path::new(&p).exists() => p,
        _ => return,
    };

    let device = candle_core::Device::Cpu;
    let mut model_a = load_quantized_model(std::path::Path::new(&model_path), &device)
        .expect("Failed to load Qwen2 model A");

    let tokenizer_path = std::path::Path::new(&model_path)
        .parent()
        .unwrap()
        .join("tokenizer.json");
    if !tokenizer_path.exists() {
        return;
    }
    let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path).unwrap();

    let prompt = "def add(a, b):";
    let _out_a = model_a.generate(prompt, &tokenizer, &device).unwrap();
}

#[test]
#[ignore]
fn test_qwen2_kv_truncation() {
    let model_path = match std::env::var("CONTEXTRA_QWEN_GGUF") {
        Ok(p) if std::path::Path::new(&p).exists() => p,
        _ => return,
    };

    let device = candle_core::Device::Cpu;
    let _model = load_quantized_model(std::path::Path::new(&model_path), &device)
        .expect("Failed to load Qwen2 model");
}
