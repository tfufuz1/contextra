// FILE-CONTEXT
// STAND: 2026-09-09T15:45:22Z (SESSION: 6cae458a)
// ZWECK: GGUF model container header and metadata parser for Candle models.
// INVARIANTEN: No full tensor payload loading during metadata parsing; error propagation via ContextraError.

use candle_core::quantized::gguf_file;
use contextra_types::ContextraError;
use std::fs::File;
use std::path::Path;

/// Metadata extracted from a GGUF model container.
#[derive(Debug, Clone)]
pub struct GgufMetadata {
    /// Identified architecture (e.g., "llama", "mistral").
    pub architecture: String,
    /// Number of tensors present in the container.
    pub tensor_count: usize,
    /// Metadata keys present in the GGUF header.
    pub metadata_keys: Vec<String>,
}

/// Inspects a GGUF model file and extracts its architecture and metadata header keys.
///
/// Uses `candle_core::quantized::gguf_file` to parse container headers without reading tensor payloads into memory.
/// Verified GGUF container loader contract: Extracts metadata and header keys via zero-payload parsing without loading tensor binary data.
pub fn parse_gguf_metadata(model_path: &Path) -> Result<GgufMetadata, ContextraError> {
    // STARTUP-ONLY: kein Hot-Path, spawn_blocking nicht erforderlich
    let mut file = File::open(model_path).map_err(|e| {
        ContextraError::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to open GGUF model file {}: {e}",
                model_path.display()
            ),
        ))
    })?;

    let content = gguf_file::Content::read(&mut file).map_err(|e| {
        ContextraError::Internal(format!(
            "Failed to parse GGUF container header for {}: {e}",
            model_path.display()
        ))
    })?;

    let architecture = content
        .metadata
        .get("general.architecture")
        .and_then(|v| v.to_string().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let metadata_keys = content.metadata.keys().cloned().collect();
    let tensor_count = content.tensor_infos.len();

    Ok(GgufMetadata {
        architecture,
        tensor_count,
        metadata_keys,
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;
    use crate::model_registry::{compute_fingerprint, CandleQuantization};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_synthetic_gguf_header(
        architecture: &str,
        extra_key: Option<(&str, &str)>,
        has_tensor: bool,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF"); // Magic
        bytes.extend_from_slice(&3u32.to_le_bytes()); // Version 3

        let tensor_count = if has_tensor { 1u64 } else { 0u64 };
        bytes.extend_from_slice(&tensor_count.to_le_bytes());

        let kv_count = if extra_key.is_some() { 2u64 } else { 1u64 };
        bytes.extend_from_slice(&kv_count.to_le_bytes());

        // KV 1: "general.architecture" -> architecture
        let key = b"general.architecture";
        bytes.extend_from_slice(&(key.len() as u64).to_le_bytes());
        bytes.extend_from_slice(key);
        bytes.extend_from_slice(&8u32.to_le_bytes()); // Type 8 = GGUF_METADATA_VALUE_TYPE_STRING
        let val = architecture.as_bytes();
        bytes.extend_from_slice(&(val.len() as u64).to_le_bytes());
        bytes.extend_from_slice(val);

        if let Some((k, v)) = extra_key {
            let k_bytes = k.as_bytes();
            bytes.extend_from_slice(&(k_bytes.len() as u64).to_le_bytes());
            bytes.extend_from_slice(k_bytes);
            bytes.extend_from_slice(&8u32.to_le_bytes());
            let v_bytes = v.as_bytes();
            bytes.extend_from_slice(&(v_bytes.len() as u64).to_le_bytes());
            bytes.extend_from_slice(v_bytes);
        }

        if has_tensor {
            let t_name = b"blk.0.attn.weight";
            bytes.extend_from_slice(&(t_name.len() as u64).to_le_bytes());
            bytes.extend_from_slice(t_name);
            bytes.extend_from_slice(&1u32.to_le_bytes()); // 1 dimension
            bytes.extend_from_slice(&64u64.to_le_bytes()); // dim 0 = 64
            bytes.extend_from_slice(&0u32.to_le_bytes()); // GGML F32 = 0
            bytes.extend_from_slice(&0u64.to_le_bytes()); // offset = 0
        }

        bytes
    }

    #[test]
    fn test_zero_payload_metadata_extraction_and_non_constant_proof(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut tmp_a = NamedTempFile::new()?;
        let header_a = create_synthetic_gguf_header("llama", None, false);
        tmp_a.write_all(&header_a)?;
        tmp_a.flush()?;

        let mut tmp_b = NamedTempFile::new()?;
        let header_b = create_synthetic_gguf_header(
            "mistral",
            Some(("general.name", "mistral-7b-v0.1")),
            true,
        );
        tmp_b.write_all(&header_b)?;
        tmp_b.flush()?;

        let meta_a = parse_gguf_metadata(tmp_a.path())?;
        let meta_b = parse_gguf_metadata(tmp_b.path())?;

        assert_eq!(meta_a.architecture, "llama");
        assert_eq!(meta_a.tensor_count, 0);
        assert_eq!(meta_b.architecture, "mistral");
        assert_eq!(meta_b.tensor_count, 1);
        assert!(meta_b.metadata_keys.contains(&"general.name".to_string()));

        // Non-constant output proof: distinct headers yield distinct metadata
        assert_ne!(
            meta_a.architecture, meta_b.architecture,
            "Architectures must differ"
        );
        assert_ne!(
            meta_a.tensor_count, meta_b.tensor_count,
            "Tensor counts must differ"
        );

        let fp_a = compute_fingerprint(tmp_a.path(), &CandleQuantization::Q4KM)?;
        let fp_b = compute_fingerprint(tmp_b.path(), &CandleQuantization::Q4KM)?;

        // Non-constant fingerprint proof
        assert_ne!(
            fp_a.hash, fp_b.hash,
            "Fingerprints for distinct GGUF files must be non-constant"
        );

        Ok(())
    }

    #[test]
    fn test_parse_gguf_metadata_nonexistent_path_returns_io_error() {
        let path = Path::new("/nonexistent/file/path.gguf");
        let res = parse_gguf_metadata(path);
        assert!(res.is_err());
        match res.unwrap_err() {
            ContextraError::Io(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
            other => panic!("Expected ContextraError::Io, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_gguf_metadata_invalid_header_bytes_returns_internal_error(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut tmp_file = NamedTempFile::new()?;
        tmp_file.write_all(b"CORRUPT_NOT_GGUF_HEADER_BYTES_12345678")?;

        let res = parse_gguf_metadata(tmp_file.path());
        assert!(res.is_err());
        match res.unwrap_err() {
            ContextraError::Internal(msg) => {
                assert!(msg.contains("Failed to parse GGUF container header"));
            }
            other => panic!("Expected ContextraError::Internal, got {:?}", other),
        }
        Ok(())
    }
}
