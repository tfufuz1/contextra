// FILE-CONTEXT
// STAND: 2026-11-09T00:00:00Z
// ZWECK: GGUF corruption matrix, mutation verification, and boundary tests.
// INVARIANTEN: Zero real model downloads (hermetic bytes); no panics on corrupt binary inputs; strict fail-closed.

//! Tests für GGUF-Mutationsmatrix und Korruptionsbehandlung.
//!
//! # Zweck
//! Verifiziert, dass jede Mutation im GGUF-Header deterministisch zu einem typisierten Fehler
//! führt (Fail-Closed) und niemals zu Panics, OOM oder Endlosschleifen führt.

// ANCHOR[contextra-infer-candle:TEST-GGUF-MUTATION-MATRIX]
use contextra_infer_candle::error::Error;
use contextra_infer_candle::gguf_loader::{
    parse_gguf_metadata, parse_gguf_metadata_with_checksum,
};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_minimal_valid_gguf() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF"); // Magic
    bytes.extend_from_slice(&3u32.to_le_bytes()); // Version
    bytes.extend_from_slice(&1u64.to_le_bytes()); // Tensor count = 1
    bytes.extend_from_slice(&1u64.to_le_bytes()); // Metadata KV count = 1

    // Metadata KV: key="general.architecture", type=8 (String), val="llama"
    let key = b"general.architecture";
    bytes.extend_from_slice(&(key.len() as u64).to_le_bytes());
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(&8u32.to_le_bytes()); // val_type = 8 (string)
    let val = b"llama";
    bytes.extend_from_slice(&(val.len() as u64).to_le_bytes());
    bytes.extend_from_slice(val);

    // Tensor info: name="blk.0.weight", n_dims=1, dim[0]=10, type=0 (F32), offset=0
    let tensor_name = b"blk.0.weight";
    bytes.extend_from_slice(&(tensor_name.len() as u64).to_le_bytes());
    bytes.extend_from_slice(tensor_name);
    bytes.extend_from_slice(&1u32.to_le_bytes()); // n_dims = 1
    bytes.extend_from_slice(&10u64.to_le_bytes()); // dim[0] = 10
    bytes.extend_from_slice(&0u32.to_le_bytes()); // GGML type F32 = 0
    bytes.extend_from_slice(&0u64.to_le_bytes()); // offset = 0

    // Tensor payload: 10 * 4 bytes = 40 bytes
    bytes.extend_from_slice(&[0u8; 40]);

    bytes
}

#[test]
fn test_valid_minimal_gguf_parses_successfully() {
    let mut file = NamedTempFile::new().unwrap();
    let bytes = create_minimal_valid_gguf();
    file.write_all(&bytes).unwrap();
    file.flush().unwrap();

    let meta = parse_gguf_metadata(file.path()).expect("Valid GGUF header should parse");
    assert_eq!(meta.tensor_count, 1);
    assert_eq!(meta.architecture, "llama");
    assert!(meta.metadata_keys.contains(&"general.architecture".to_string()));
}

#[test]
fn test_gguf_mutation_matrix() {
    let base_bytes = create_minimal_valid_gguf();

    // Mutationsmatrix: Mutation an spezifischen Header-Feldern
    let mutations: Vec<(&str, Box<dyn Fn(&mut [u8])>)> = vec![
        ("corrupt_magic", Box::new(|b| b[0] = b'X')),
        ("unsupported_version", Box::new(|b| b[4] = 99)),
        ("tensor_count_overflow", Box::new(|b| {
            // Tensor count at bytes 8..16
            b[8..16].copy_from_slice(&1_000_000u64.to_le_bytes());
        })),
        ("metadata_count_overflow", Box::new(|b| {
            // Metadata count at bytes 16..24
            b[16..24].copy_from_slice(&1_000_000u64.to_le_bytes());
        })),
        ("zero_dim_count", Box::new(|b| {
            // Letzte Dimension vor Payload manipulieren
            for i in 0..b.len().saturating_sub(4) {
                if &b[i..i+4] == &1u32.to_le_bytes() {
                    b[i..i+4].copy_from_slice(&0u32.to_le_bytes());
                    break;
                }
            }
        })),
    ];

    for (name, mutate) in mutations {
        let mut mutated = base_bytes.clone();
        mutate(&mut mutated);

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&mut mutated).unwrap();
        file.flush().unwrap();

        let res = parse_gguf_metadata(file.path());
        assert!(
            res.is_err(),
            "Mutation '{name}' must result in Err, but succeeded"
        );
        match res.unwrap_err() {
            Error::ModelLoad { .. } | Error::Io(_) | Error::InvalidInput(_) => {}
            other => panic!("Unexpected error type for mutation '{name}': {other:?}"),
        }
    }
}

#[test]
fn test_gguf_checksum_verification_fail_closed() {
    let mut file = NamedTempFile::new().unwrap();
    let bytes = create_minimal_valid_gguf();
    file.write_all(&bytes).unwrap();
    file.flush().unwrap();

    let wrong_checksum = [0xAAu8; 32];
    let res = parse_gguf_metadata_with_checksum(file.path(), Some(&wrong_checksum));

    assert!(res.is_err(), "Wrong checksum must fail closed");
    match res.unwrap_err() {
        Error::ModelLoad { reason } => {
            assert!(reason.contains("SHA-256 checksum mismatch"));
        }
        other => panic!("Expected Error::ModelLoad with mismatch reason, got {other:?}"),
    }
}
