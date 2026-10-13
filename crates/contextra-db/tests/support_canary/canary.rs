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
        // Format as alphanumeric canary string with person label and 32 hex chars (16 bytes entropy = total 32+ bytes)
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

        // 1. Raw UTF-8 bytes
        let raw_bytes = canary_str.as_bytes().to_vec();
        needles.push(SearchNeedle {
            bytes: raw_bytes.clone(),
            encoding: "raw_utf8".to_string(),
            canary_id: canary_id.clone(),
            owner_person: self.owner_person.clone(),
        });

        // 2. JSON-escaped string
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

        // 3. Hex lowercase
        let hex_lower: String = raw_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        needles.push(SearchNeedle {
            bytes: hex_lower.as_bytes().to_vec(),
            encoding: "hex_lower".to_string(),
            canary_id: canary_id.clone(),
            owner_person: self.owner_person.clone(),
        });

        // 4. Hex uppercase
        let hex_upper: String = raw_bytes.iter().map(|b| format!("{:02X}", b)).collect();
        needles.push(SearchNeedle {
            bytes: hex_upper.as_bytes().to_vec(),
            encoding: "hex_upper".to_string(),
            canary_id: canary_id.clone(),
            owner_person: self.owner_person.clone(),
        });

        // 5. Base64 Alignments (0, 1, and 2 byte offset shifts)
        // Offset 0 shift:
        let b64_shift0 = base64_encode(&raw_bytes);
        needles.push(SearchNeedle {
            bytes: b64_shift0.as_bytes().to_vec(),
            encoding: "base64_shift0".to_string(),
            canary_id: canary_id.clone(),
            owner_person: self.owner_person.clone(),
        });

        // Offset 1 shift (prepend 1 dummy byte, b64 encode, slice off first 2 b64 chars and trailing padding if needed):
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

        // Offset 2 shift (prepend 2 dummy bytes, b64 encode, slice off first 3 b64 chars and trailing padding):
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

        // 6. UTF-16 LE
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

        // 7. UTF-16 BE
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

        // 8. Lowercase string
        let lower_str = canary_str.to_lowercase();
        if lower_str.as_bytes() != raw_bytes {
            needles.push(SearchNeedle {
                bytes: lower_str.as_bytes().to_vec(),
                encoding: "lowercase_utf8".to_string(),
                canary_id: canary_id.clone(),
                owner_person: self.owner_person.clone(),
            });
        }

        // 9. Substring fragment (at least 12 bytes length)
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

    /// Generates search needles for a vector canary (f32 little-endian component byte sequence).
    pub fn build_vector_needles(&self, vec_canary: &[f32], label: &str) -> Vec<SearchNeedle> {
        let canary_id = format!("{}_{}_vec", self.owner_person, label);
        let mut bytes = Vec::with_capacity(vec_canary.len() * 4);
        for &val in vec_canary {
            bytes.extend_from_slice(&val.to_le_bytes());
        }

        let mut needles = Vec::new();
        // Full vector byte sequence
        needles.push(SearchNeedle {
            bytes: bytes.clone(),
            encoding: "f32_le_vector".to_string(),
            canary_id: canary_id.clone(),
            owner_person: self.owner_person.clone(),
        });

        // First 3 components (12 bytes fragment)
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
