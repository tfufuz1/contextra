// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z
// ZWECK: 15+ synthetic fuzz and corruption tests for GGUF model loader hardening.

use contextra_infer_candle::gguf_loader::{
    parse_gguf_metadata, parse_gguf_metadata_with_checksum, MAX_GGUF_METADATA_KVS, MAX_GGUF_TENSORS,
};
use contextra_types::ContextraError;
use sha2::{Digest, Sha256};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_valid_gguf_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF"); // magic
    bytes.extend_from_slice(&3u32.to_le_bytes()); // version
    bytes.extend_from_slice(&1u64.to_le_bytes()); // tensor count = 1
    bytes.extend_from_slice(&1u64.to_le_bytes()); // metadata kv count = 1

    // Metadata KV 1: key "general.architecture", string value "llama"
    let key = b"general.architecture";
    bytes.extend_from_slice(&(key.len() as u64).to_le_bytes());
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(&8u32.to_le_bytes()); // value type 8 = String
    let val = b"llama";
    bytes.extend_from_slice(&(val.len() as u64).to_le_bytes());
    bytes.extend_from_slice(val);

    // Tensor 1: name "blk.0.weight", 1D, dim [10], GGML_TYPE_F32 (0), offset 0
    let tname = b"blk.0.weight";
    bytes.extend_from_slice(&(tname.len() as u64).to_le_bytes());
    bytes.extend_from_slice(tname);
    bytes.extend_from_slice(&1u32.to_le_bytes()); // n_dims = 1
    bytes.extend_from_slice(&10u64.to_le_bytes()); // dim[0] = 10
    bytes.extend_from_slice(&0u32.to_le_bytes()); // GGML_TYPE_F32
    bytes.extend_from_slice(&0u64.to_le_bytes()); // offset = 0

    // Pad with dummy tensor payload bytes
    bytes.extend_from_slice(&[0u8; 100]);
    bytes
}

#[test]
fn test_fuzz_01_valid_gguf_header_succeeds() {
    let bytes = create_valid_gguf_bytes();
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let meta = parse_gguf_metadata(tmp.path()).unwrap();
    assert_eq!(meta.architecture, "llama");
    assert_eq!(meta.tensor_count, 1);
}

#[test]
fn test_fuzz_02_truncated_file_below_24_bytes() {
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(b"GGUF_SHORT").unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(matches!(err, ContextraError::ModelLoad { .. }));
}

#[test]
fn test_fuzz_03_corrupt_magic_bytes() {
    let mut bytes = create_valid_gguf_bytes();
    bytes[0..4].copy_from_slice(b"BADM");
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("Invalid magic bytes"));
}

#[test]
fn test_fuzz_04_unsupported_version_1() {
    let mut bytes = create_valid_gguf_bytes();
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("Unsupported GGUF version"));
}

#[test]
fn test_fuzz_05_unsupported_version_999() {
    let mut bytes = create_valid_gguf_bytes();
    bytes[4..8].copy_from_slice(&999u32.to_le_bytes());
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("Unsupported GGUF version"));
}

#[test]
fn test_fuzz_06_tensor_count_overflow_2_pow_32() {
    let mut bytes = create_valid_gguf_bytes();
    let count = (MAX_GGUF_TENSORS + 1) as u64;
    bytes[8..16].copy_from_slice(&count.to_le_bytes());
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err
        .to_string()
        .contains("GGUF tensor count exceeds upper limit"));
}

#[test]
fn test_fuzz_07_metadata_kv_count_overflow() {
    let mut bytes = create_valid_gguf_bytes();
    let count = (MAX_GGUF_METADATA_KVS + 1) as u64;
    bytes[16..24].copy_from_slice(&count.to_le_bytes());
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err
        .to_string()
        .contains("GGUF metadata KV count exceeds upper limit"));
}

#[test]
fn test_fuzz_08_string_length_overflow_claims_gigabytes() {
    let mut bytes = create_valid_gguf_bytes();
    // Key length field starts after offset 24
    let huge_len = 10_000_000_000u64;
    bytes[24..32].copy_from_slice(&huge_len.to_le_bytes());
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err
        .to_string()
        .contains("GGUF string length bounds exceeded"));
}

#[test]
fn test_fuzz_09_truncated_string_payload() {
    let mut bytes = create_valid_gguf_bytes();
    // Claim string length exceeding file length
    bytes[24..32].copy_from_slice(&100u64.to_le_bytes());
    bytes.truncate(40);
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err
        .to_string()
        .contains("GGUF string length bounds exceeded"));
}

#[test]
fn test_fuzz_10_invalid_metadata_value_type() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF");
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes()); // tensor count = 0
    bytes.extend_from_slice(&1u64.to_le_bytes()); // metadata count = 1

    let key = b"test.key";
    bytes.extend_from_slice(&(key.len() as u64).to_le_bytes());
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(&99u32.to_le_bytes()); // Invalid type 99

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("Unknown metadata GGUF value type"));
}

#[test]
fn test_fuzz_11_tensor_dimension_count_zero() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF");
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&1u64.to_le_bytes()); // 1 tensor
    bytes.extend_from_slice(&0u64.to_le_bytes()); // 0 KVs

    let tname = b"tensor0";
    bytes.extend_from_slice(&(tname.len() as u64).to_le_bytes());
    bytes.extend_from_slice(tname);
    bytes.extend_from_slice(&0u32.to_le_bytes()); // n_dims = 0

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("Invalid tensor dimension count"));
}

#[test]
fn test_fuzz_12_tensor_dimension_count_too_high() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF");
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&1u64.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());

    let tname = b"tensor0";
    bytes.extend_from_slice(&(tname.len() as u64).to_le_bytes());
    bytes.extend_from_slice(tname);
    bytes.extend_from_slice(&10u32.to_le_bytes()); // n_dims = 10 (> 8)

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("Invalid tensor dimension count"));
}

#[test]
fn test_fuzz_13_tensor_dimension_zero() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF");
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&1u64.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());

    let tname = b"tensor0";
    bytes.extend_from_slice(&(tname.len() as u64).to_le_bytes());
    bytes.extend_from_slice(tname);
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes()); // dim = 0

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("Tensor dimension cannot be 0"));
}

#[test]
fn test_fuzz_14_tensor_offset_exceeds_file_len() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF");
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&1u64.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());

    let tname = b"tensor0";
    bytes.extend_from_slice(&(tname.len() as u64).to_le_bytes());
    bytes.extend_from_slice(tname);
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&10u64.to_le_bytes()); // dim = 10
    bytes.extend_from_slice(&0u32.to_le_bytes()); // type F32
    bytes.extend_from_slice(&999_999u64.to_le_bytes()); // Out-of-bounds offset!

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    let err = parse_gguf_metadata(tmp.path()).unwrap_err();
    assert!(err
        .to_string()
        .contains("Tensor offset exceeds total file length"));
}

#[test]
fn test_fuzz_15_sha256_checksum_verification_success_and_failure() {
    let bytes = create_valid_gguf_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let correct_hash: [u8; 32] = hasher.finalize().into();

    let mut wrong_hash = correct_hash;
    wrong_hash[0] ^= 0xFF;

    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&bytes).unwrap();

    // 1. Correct hash succeeds
    let ok_res = parse_gguf_metadata_with_checksum(tmp.path(), Some(&correct_hash));
    assert!(ok_res.is_ok());

    // 2. Wrong hash fails closed
    let err_res = parse_gguf_metadata_with_checksum(tmp.path(), Some(&wrong_hash)).unwrap_err();
    assert!(err_res.to_string().contains("SHA-256 checksum mismatch"));
}
