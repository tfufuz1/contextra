// FILE-CONTEXT
// STAND: 2026-10-17
// ZWECK: Deterministischer Canary-Generator fuer alle Feldtypen & Kodierungen (LN-6.1).
// INVARIANTEN: #![forbid(unsafe_code)], Ring-2-Reinheit (keine Db/Engine-Imports), deterministisch via Seed.

#![forbid(unsafe_code)]

use std::fmt;
use crate::erasure_harness::raw_sweep::SweepHit;

/// Supported database field types for canary injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanaryFieldType {
    DocId,
    Text,
    MetaKey,
    MetaVal,
    Relation,
    Vector,
}

impl fmt::Display for CanaryFieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocId => write!(f, "doc_id"),
            Self::Text => write!(f, "text"),
            Self::MetaKey => write!(f, "meta_key"),
            Self::MetaVal => write!(f, "meta_val"),
            Self::Relation => write!(f, "relation"),
            Self::Vector => write!(f, "vector"),
        }
    }
}

/// Encoding/Representation variants of a canary byte pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanaryEncoding {
    RawUtf8,
    JsonEscaped,
    HexLower,
    HexUpper,
    Base64Shift0,
    Base64Shift1,
    Base64Shift2,
    Utf16Le,
    Utf16Be,
    LowercaseUtf8,
    Fragment12b,
    F32LeVector,
    F32LeVecFrag12b,
}

impl fmt::Display for CanaryEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawUtf8 => write!(f, "raw_utf8"),
            Self::JsonEscaped => write!(f, "json_escaped"),
            Self::HexLower => write!(f, "hex_lower"),
            Self::HexUpper => write!(f, "hex_upper"),
            Self::Base64Shift0 => write!(f, "base64_shift0"),
            Self::Base64Shift1 => write!(f, "base64_shift1"),
            Self::Base64Shift2 => write!(f, "base64_shift2"),
            Self::Utf16Le => write!(f, "utf16_le"),
            Self::Utf16Be => write!(f, "utf16_be"),
            Self::LowercaseUtf8 => write!(f, "lowercase_utf8"),
            Self::Fragment12b => write!(f, "fragment_12b"),
            Self::F32LeVector => write!(f, "f32_le_vector"),
            Self::F32LeVecFrag12b => write!(f, "f32_le_vec_frag12b"),
        }
    }
}

/// Metadata attached to each search needle within a `CanarySet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanaryNeedleInfo {
    pub field_type: CanaryFieldType,
    pub encoding: CanaryEncoding,
    pub canary_id: String,
    pub bytes: Vec<u8>,
}

/// Holds all generated canary values and search needles for a test run.
#[derive(Debug, Clone)]
pub struct CanarySet {
    pub seed: u64,
    pub doc_id: String,
    pub text: String,
    pub meta_key: String,
    pub meta_val: String,
    pub relation: String,
    pub vector: Vec<f32>,
    needles_bytes: Vec<Vec<u8>>,
    needle_infos: Vec<CanaryNeedleInfo>,
}

impl CanarySet {
    /// Generates a deterministic set of canary fields from a given seed.
    pub fn generate(seed: u64) -> Self {
        let mut lcg = SimpleLcg::new(seed);

        let doc_id = format!("doc_cnry_{}", lcg.next_hex_string(32));
        let text = format!("CNRY_TXT_{}", lcg.next_hex_string(32));
        let meta_key = format!("CNRY_MKEY_{}", lcg.next_hex_string(32));
        let meta_val = format!("CNRY_MVAL_{}", lcg.next_hex_string(32));
        let relation = format!("CNRY_REL_{}", lcg.next_hex_string(32));
        let vector = lcg.next_f32_vec(4);

        let mut needles_bytes = Vec::new();
        let mut needle_infos = Vec::new();

        let str_fields = [
            (CanaryFieldType::DocId, &doc_id),
            (CanaryFieldType::Text, &text),
            (CanaryFieldType::MetaKey, &meta_key),
            (CanaryFieldType::MetaVal, &meta_val),
            (CanaryFieldType::Relation, &relation),
        ];

        for (field_type, val_str) in str_fields {
            build_string_needles(
                field_type,
                val_str,
                &mut needles_bytes,
                &mut needle_infos,
            );
        }

        build_vector_needles(
            CanaryFieldType::Vector,
            &vector,
            &mut needles_bytes,
            &mut needle_infos,
        );

        Self {
            seed,
            doc_id,
            text,
            meta_key,
            meta_val,
            relation,
            vector,
            needles_bytes,
            needle_infos,
        }
    }

    /// Returns byte patterns for all generated canary needles.
    pub fn needles(&self) -> &[Vec<u8>] {
        &self.needles_bytes
    }

    /// Returns detailed metadata for all generated canary needles.
    pub fn needle_infos(&self) -> &[CanaryNeedleInfo] {
        &self.needle_infos
    }

    /// Formats a descriptive summary for a sweep hit.
    pub fn describe(&self, hit: &SweepHit) -> String {
        if let Some(info) = self.needle_infos.get(hit.needle_index) {
            format!(
                "CanaryHit[field={}, enc={}, id='{}', path='{}', offset={}]",
                info.field_type,
                info.encoding,
                info.canary_id,
                hit.relative_path.display(),
                hit.offset
            )
        } else {
            format!(
                "CanaryHit[unknown_idx={}, path='{}', offset={}]",
                hit.needle_index,
                hit.relative_path.display(),
                hit.offset
            )
        }
    }
}

struct SimpleLcg {
    state: u64,
}

impl SimpleLcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn next_hex_string(&mut self, len: usize) -> String {
        let mut res = String::with_capacity(len);
        while res.len() < len {
            let val = self.next_u64();
            let hex = format!("{:016x}", val);
            res.push_str(&hex);
        }
        res.truncate(len);
        res
    }

    fn next_f32_vec(&mut self, dim: usize) -> Vec<f32> {
        let mut res = Vec::with_capacity(dim);
        for i in 0..dim {
            let val = self.next_u64();
            let f = ((val >> 32) as f32) / (u32::MAX as f32) + (i as f32 + 1.0) * 100.0;
            res.push(f);
        }
        res
    }
}

fn build_string_needles(
    field_type: CanaryFieldType,
    val_str: &str,
    out_bytes: &mut Vec<Vec<u8>>,
    out_infos: &mut Vec<CanaryNeedleInfo>,
) {
    let raw_bytes = val_str.as_bytes().to_vec();
    let canary_id = format!("{}:{}", field_type, val_str);

    let add_needle = |enc: CanaryEncoding,
                      bytes: Vec<u8>,
                      out_bytes: &mut Vec<Vec<u8>>,
                      out_infos: &mut Vec<CanaryNeedleInfo>| {
        if !bytes.is_empty() {
            out_infos.push(CanaryNeedleInfo {
                field_type,
                encoding: enc,
                canary_id: canary_id.clone(),
                bytes: bytes.clone(),
            });
            out_bytes.push(bytes);
        }
    };

    // 1. Raw UTF-8
    add_needle(CanaryEncoding::RawUtf8, raw_bytes.clone(), out_bytes, out_infos);

    // 2. JSON Escaped
    let json_escaped = escape_json_str(val_str);
    add_needle(CanaryEncoding::JsonEscaped, json_escaped.into_bytes(), out_bytes, out_infos);

    // 3. Hex Lower
    let hex_lower: String = raw_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    add_needle(CanaryEncoding::HexLower, hex_lower.into_bytes(), out_bytes, out_infos);

    // 4. Hex Upper
    let hex_upper: String = raw_bytes.iter().map(|b| format!("{:02X}", b)).collect();
    add_needle(CanaryEncoding::HexUpper, hex_upper.into_bytes(), out_bytes, out_infos);

    // 5. Base64 Shift 0
    let b64_shift0 = base64_encode(&raw_bytes);
    add_needle(CanaryEncoding::Base64Shift0, b64_shift0.into_bytes(), out_bytes, out_infos);

    // 6. Base64 Shift 1
    let mut bytes_shift1 = vec![0x00];
    bytes_shift1.extend_from_slice(&raw_bytes);
    let b64_full1 = base64_encode(&bytes_shift1);
    if b64_full1.len() > 4 {
        let b64_shift1 = b64_full1[2..b64_full1.len() - 2].to_string();
        add_needle(CanaryEncoding::Base64Shift1, b64_shift1.into_bytes(), out_bytes, out_infos);
    }

    // 7. Base64 Shift 2
    let mut bytes_shift2 = vec![0x00, 0x00];
    bytes_shift2.extend_from_slice(&raw_bytes);
    let b64_full2 = base64_encode(&bytes_shift2);
    if b64_full2.len() > 4 {
        let b64_shift2 = b64_full2[3..b64_full2.len() - 2].to_string();
        add_needle(CanaryEncoding::Base64Shift2, b64_shift2.into_bytes(), out_bytes, out_infos);
    }

    // 8. UTF-16LE
    let utf16_le: Vec<u8> = val_str
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect();
    add_needle(CanaryEncoding::Utf16Le, utf16_le, out_bytes, out_infos);

    // 9. UTF-16BE
    let utf16_be: Vec<u8> = val_str
        .encode_utf16()
        .flat_map(|u| u.to_be_bytes())
        .collect();
    add_needle(CanaryEncoding::Utf16Be, utf16_be, out_bytes, out_infos);

    // 10. Lowercase UTF-8
    let lower_bytes = val_str.to_lowercase().into_bytes();
    add_needle(CanaryEncoding::LowercaseUtf8, lower_bytes, out_bytes, out_infos);

    // 11. Fragment 12b
    if raw_bytes.len() >= 12 {
        let end = (4 + 12).min(raw_bytes.len());
        let fragment = raw_bytes[4..end].to_vec();
        add_needle(CanaryEncoding::Fragment12b, fragment, out_bytes, out_infos);
    }
}

fn build_vector_needles(
    field_type: CanaryFieldType,
    vector: &[f32],
    out_bytes: &mut Vec<Vec<u8>>,
    out_infos: &mut Vec<CanaryNeedleInfo>,
) {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for &val in vector {
        bytes.extend_from_slice(&val.to_le_bytes());
    }

    let canary_id = format!("{}:len_{}", field_type, vector.len());

    out_infos.push(CanaryNeedleInfo {
        field_type,
        encoding: CanaryEncoding::F32LeVector,
        canary_id: canary_id.clone(),
        bytes: bytes.clone(),
    });
    out_bytes.push(bytes.clone());

    if bytes.len() >= 12 {
        let frag_bytes = bytes[..12].to_vec();
        out_infos.push(CanaryNeedleInfo {
            field_type,
            encoding: CanaryEncoding::F32LeVecFrag12b,
            canary_id,
            bytes: frag_bytes.clone(),
        });
        out_bytes.push(frag_bytes);
    }
}

fn escape_json_str(s: &str) -> String {
    let mut res = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => res.push_str("\\\""),
            '\\' => res.push_str("\\\\"),
            '\n' => res.push_str("\\n"),
            '\r' => res.push_str("\\r"),
            '\t' => res.push_str("\\t"),
            c => res.push(c),
        }
    }
    res
}

fn base64_encode(data: &[u8]) -> String {
    const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canary_set_deterministic_generation() {
        let set1 = CanarySet::generate(42);
        let set2 = CanarySet::generate(42);

        assert_eq!(set1.doc_id, set2.doc_id);
        assert_eq!(set1.text, set2.text);
        assert_eq!(set1.meta_key, set2.meta_key);
        assert_eq!(set1.meta_val, set2.meta_val);
        assert_eq!(set1.relation, set2.relation);
        assert_eq!(set1.vector, set2.vector);
        assert_eq!(set1.needles(), set2.needles());

        let set3 = CanarySet::generate(43);
        assert_ne!(set1.doc_id, set3.doc_id);
    }

    #[test]
    fn test_canary_set_needle_counts_and_types() {
        let set = CanarySet::generate(100);
        assert!(!set.needles().is_empty());
        assert_eq!(set.needles().len(), set.needle_infos().len());

        let has_docid = set.needle_infos().iter().any(|i| i.field_type == CanaryFieldType::DocId);
        let has_text = set.needle_infos().iter().any(|i| i.field_type == CanaryFieldType::Text);
        let has_vector = set.needle_infos().iter().any(|i| i.field_type == CanaryFieldType::Vector);

        assert!(has_docid && has_text && has_vector);
    }
}
