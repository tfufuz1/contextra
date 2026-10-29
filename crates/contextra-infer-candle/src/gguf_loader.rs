// FILE-CONTEXT
// STAND: 2026-09-12T00:00:00Z
// ZWECK: Robust GGUF model container header parser, metadata extractor, and binary validator for Candle models.
// INVARIANTEN: Zero panic or unbuffered allocation on corrupt files; memory limits enforced before allocation; SHA-256 verification.

use contextra_types::ContextraError;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Maximum number of tensors permitted in a valid GGUF header.
pub const MAX_GGUF_TENSORS: u64 = 65_536;

/// Maximum number of metadata key-value pairs permitted in a valid GGUF header.
pub const MAX_GGUF_METADATA_KVS: u64 = 65_536;

/// Maximum length of a string in bytes permitted in a GGUF metadata string field.
pub const MAX_GGUF_STRING_LEN: u64 = 16_777_216; // 16 MiB

/// Maximum permitted depth for nested GGUF metadata array parsing.
pub const MAX_GGUF_METADATA_DEPTH: usize = 2;

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

/// Inspects a GGUF model file and extracts its architecture and metadata header keys
/// after performing strict header validation and optional SHA-256 verification.
pub fn parse_gguf_metadata(model_path: &Path) -> Result<GgufMetadata, ContextraError> {
    parse_gguf_metadata_with_checksum(model_path, None)
}

fn format_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Inspects a GGUF model file with optional SHA-256 verification.
pub fn parse_gguf_metadata_with_checksum(
    model_path: &Path,
    expected_sha256: Option<&[u8; 32]>,
) -> Result<GgufMetadata, ContextraError> {
    let mut file = File::open(model_path).map_err(|e| {
        ContextraError::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to open GGUF model file {}: {e}",
                model_path.display()
            ),
        ))
    })?;

    let file_len = file.metadata().map_err(ContextraError::Io)?.len();
    if file_len < 24 {
        return Err(ContextraError::ModelLoad {
            path: model_path.display().to_string(),
            reason: format!("File size too small for GGUF header: {file_len} bytes"),
        });
    }

    if let Some(expected) = expected_sha256 {
        verify_file_sha256(&mut file, model_path, expected)?;
        file.seek(SeekFrom::Start(0)).map_err(ContextraError::Io)?;
    }

    let mut reader = BufReader::new(&file);
    validate_gguf_header_and_offsets(&mut reader, model_path, file_len)?;

    file.seek(SeekFrom::Start(0)).map_err(ContextraError::Io)?;
    let content = candle_core::quantized::gguf_file::Content::read(&mut file).map_err(|e| {
        ContextraError::ModelLoad {
            path: model_path.display().to_string(),
            reason: format!("Failed to parse GGUF container content: {e}"),
        }
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

fn verify_file_sha256(
    file: &mut File,
    path: &Path,
    expected: &[u8; 32],
) -> Result<(), ContextraError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];

    loop {
        let count = file.read(&mut buffer).map_err(ContextraError::Io)?;
        if count == 0 {
            break;
        }
        let chunk = buffer.get(..count).ok_or_else(|| {
            ContextraError::Internal("Buffer slice bounds error during SHA-256 read".to_string())
        })?;
        hasher.update(chunk);
    }

    let result = hasher.finalize();
    if result.as_slice() != expected {
        return Err(ContextraError::ModelLoad {
            path: path.display().to_string(),
            reason: format!(
                "SHA-256 checksum mismatch: expected {}, calculated {}",
                format_hex(expected),
                format_hex(result.as_slice())
            ),
        });
    }
    Ok(())
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32, std::io::Error> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64<R: Read>(r: &mut R) -> Result<u64, std::io::Error> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn validate_string<R: Read>(r: &mut R, file_len: u64) -> Result<u64, ContextraError> {
    let len = read_u64(r).map_err(ContextraError::Io)?;
    if len > MAX_GGUF_STRING_LEN || len > file_len {
        return Err(ContextraError::ModelLoad {
            path: "gguf_header".to_string(),
            reason: format!("GGUF string length bounds exceeded: {len} bytes"),
        });
    }
    let mut take = r.take(len);
    let mut dummy = [0u8; 1024];
    let mut read_bytes = 0u64;
    while read_bytes < len {
        let remaining = len - read_bytes;
        let to_read_u64 = remaining.min(dummy.len() as u64);
        let to_read = usize::try_from(to_read_u64)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        let target_slice = dummy
            .get_mut(..to_read)
            .ok_or_else(|| ContextraError::Internal("Slice bounds error".into()))?;
        let n = take.read(target_slice).map_err(ContextraError::Io)?;
        if n == 0 {
            return Err(ContextraError::ModelLoad {
                path: "gguf_header".to_string(),
                reason: "Truncated GGUF string field".to_string(),
            });
        }
        read_bytes += n as u64;
    }
    Ok(len)
}

fn skip_value_data<R: Read>(
    r: &mut R,
    val_type: u32,
    file_len: u64,
    depth: usize,
) -> Result<(), ContextraError> {
    if depth > MAX_GGUF_METADATA_DEPTH {
        return Err(ContextraError::ModelLoad {
            path: "gguf_header".to_string(),
            reason: format!(
                "Exceeded maximum metadata nesting depth limit {MAX_GGUF_METADATA_DEPTH}"
            ),
        });
    }

    match val_type {
        0 | 1 | 7 => {
            let mut buf = [0u8; 1];
            r.read_exact(&mut buf).map_err(ContextraError::Io)?;
        }
        2 | 3 => {
            let mut buf = [0u8; 2];
            r.read_exact(&mut buf).map_err(ContextraError::Io)?;
        }
        4..=6 => {
            let mut buf = [0u8; 4];
            r.read_exact(&mut buf).map_err(ContextraError::Io)?;
        }
        10..=12 => {
            let mut buf = [0u8; 8];
            r.read_exact(&mut buf).map_err(ContextraError::Io)?;
        }
        8 => {
            validate_string(r, file_len)?;
        }
        9 => {
            let elem_type = read_u32(r).map_err(ContextraError::Io)?;
            if elem_type == 9 {
                return Err(ContextraError::ModelLoad {
                    path: "gguf_header".to_string(),
                    reason: "Nested GGUF array types are prohibited".to_string(),
                });
            }
            let array_len = read_u64(r).map_err(ContextraError::Io)?;
            if array_len > file_len {
                return Err(ContextraError::ModelLoad {
                    path: "gguf_header".to_string(),
                    reason: format!("Metadata array length bounds exceeded: {array_len}"),
                });
            }
            for _ in 0..array_len {
                skip_value_data(r, elem_type, file_len, depth + 1)?;
            }
        }
        other => {
            return Err(ContextraError::ModelLoad {
                path: "gguf_header".to_string(),
                reason: format!("Unknown metadata GGUF value type: {other}"),
            });
        }
    }
    Ok(())
}

fn estimate_ggml_element_size(ggml_type: u32) -> u64 {
    match ggml_type {
        0 => 4, // F32
        1 => 2, // F16
        2 => 1, // Q4_0
        3 => 1, // Q4_1
        6 => 1, // Q8_0
        _ => 1, // Conservative lower estimate for other types
    }
}

fn validate_gguf_header_and_offsets<R: Read>(
    r: &mut R,
    path: &Path,
    file_len: u64,
) -> Result<(), ContextraError> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)
        .map_err(|e| ContextraError::ModelLoad {
            path: path.display().to_string(),
            reason: format!("Failed to read magic bytes: {e}"),
        })?;

    if &magic != b"GGUF" {
        return Err(ContextraError::ModelLoad {
            path: path.display().to_string(),
            reason: format!("Invalid magic bytes: expected 'GGUF', got {:?}", magic),
        });
    }

    let version = read_u32(r).map_err(|e| ContextraError::ModelLoad {
        path: path.display().to_string(),
        reason: format!("Failed to read version: {e}"),
    })?;

    if version != 2 && version != 3 {
        return Err(ContextraError::ModelLoad {
            path: path.display().to_string(),
            reason: format!("Unsupported GGUF version: {version}. Expected 2 or 3"),
        });
    }

    let tensor_count = read_u64(r).map_err(ContextraError::Io)?;
    if tensor_count > MAX_GGUF_TENSORS {
        return Err(ContextraError::ModelLoad {
            path: path.display().to_string(),
            reason: format!(
                "GGUF tensor count exceeds upper limit: {tensor_count} > {MAX_GGUF_TENSORS}"
            ),
        });
    }

    let metadata_kv_count = read_u64(r).map_err(ContextraError::Io)?;
    if metadata_kv_count > MAX_GGUF_METADATA_KVS {
        return Err(ContextraError::ModelLoad {
            path: path.display().to_string(),
            reason: format!(
                "GGUF metadata KV count exceeds upper limit: {metadata_kv_count} > {MAX_GGUF_METADATA_KVS}"
            ),
        });
    }

    for _ in 0..metadata_kv_count {
        validate_string(r, file_len)?;
        let val_type = read_u32(r).map_err(ContextraError::Io)?;
        skip_value_data(r, val_type, file_len, 0)?;
    }

    for _ in 0..tensor_count {
        validate_string(r, file_len)?;
        let n_dims = read_u32(r).map_err(ContextraError::Io)?;
        if n_dims == 0 || n_dims > 8 {
            return Err(ContextraError::ModelLoad {
                path: path.display().to_string(),
                reason: format!("Invalid tensor dimension count: {n_dims}"),
            });
        }
        let mut total_elements = 1u64;
        for _ in 0..n_dims {
            let dim = read_u64(r).map_err(ContextraError::Io)?;
            if dim == 0 {
                return Err(ContextraError::ModelLoad {
                    path: path.display().to_string(),
                    reason: "Tensor dimension cannot be 0".to_string(),
                });
            }
            total_elements = total_elements.saturating_mul(dim);
        }
        let ggml_type = read_u32(r).map_err(ContextraError::Io)?;
        if ggml_type > 39 {
            return Err(ContextraError::ModelLoad {
                path: path.display().to_string(),
                reason: format!("Unknown GGML type: {ggml_type}"),
            });
        }
        let offset = read_u64(r).map_err(ContextraError::Io)?;
        if offset >= file_len {
            return Err(ContextraError::ModelLoad {
                path: path.display().to_string(),
                reason: format!("Tensor offset exceeds total file length: {offset} >= {file_len}"),
            });
        }
        let approx_bytes = total_elements.saturating_mul(estimate_ggml_element_size(ggml_type));
        let required_end = offset.saturating_add(approx_bytes);
        if required_end > file_len {
            return Err(ContextraError::ModelLoad {
                path: path.display().to_string(),
                reason: format!(
                    "Tensor end position exceeds total file length: {required_end} > {file_len}"
                ),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
    fn test_parse_gguf_metadata_invalid_header_bytes_returns_model_load_error() {
        let mut tmp_file = NamedTempFile::new().unwrap();
        tmp_file
            .write_all(b"CORRUPT_NOT_GGUF_HEADER_BYTES_12345678")
            .unwrap();

        let res = parse_gguf_metadata(tmp_file.path());
        assert!(res.is_err());
        match res.unwrap_err() {
            ContextraError::ModelLoad { path: _, reason } => {
                assert!(reason.contains("Invalid magic bytes"));
            }
            other => panic!("Expected ContextraError::ModelLoad, got {:?}", other),
        }
    }
}
