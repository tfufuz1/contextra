//! Canonical JSON verification according to SPEC F.10.
//!
//! Rules:
//! 1. UTF-8 NFC normalization.
//! 2. Object keys sorted lexicographically ascending.
//! 3. No extraneous whitespace outside string literals (no space after colon or comma).
//! 4. No floating-point numbers (integers or decimal strings only).
//! 5. Microsecond integer timestamps.

use unicode_normalization::UnicodeNormalization;

/// Maximum size allowed for input reports (16 MiB).
pub const MAX_REPORT_BYTES: usize = 16 * 1024 * 1024;

/// Verifies whether the raw bytes represent valid canonical JSON per SPEC F.10.
pub fn verify_canonical_json(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() > MAX_REPORT_BYTES {
        return Err("report exceeds maximum allowed size");
    }

    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return Err("invalid utf-8 encoding"),
    };

    // Check NFC normalization
    if !text.nfc().eq(text.chars()) {
        return Err("json is not unicode NFC normalized");
    }

    // Parse with serde_json to check structure and formatting
    let val: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return Err("invalid json syntax"),
    };

    // Check no floating point numbers
    if has_floating_point(&val) {
        return Err("floating point numbers are forbidden");
    }

    // Check that text exactly matches our strict canonical serialization
    let canonical = to_canonical_json_string(&val)?;

    if canonical.as_bytes() != bytes {
        return Err("bytes do not match canonical JSON serialization");
    }

    Ok(())
}

fn has_floating_point(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::Number(n) => n.is_f64(),
        serde_json::Value::Array(arr) => arr.iter().any(has_floating_point),
        serde_json::Value::Object(map) => map.values().any(has_floating_point),
        _ => false,
    }
}

/// Serializes a serde_json::Value into canonical JSON format (sorted keys, no whitespace).
pub fn to_canonical_json_string(val: &serde_json::Value) -> Result<String, &'static str> {
    let mut out = String::new();
    serialize_val(val, &mut out)?;
    Ok(out)
}

fn serialize_val(val: &serde_json::Value, out: &mut String) -> Result<(), &'static str> {
    match val {
        serde_json::Value::Null => out.push_str("null"),
        serde_json::Value::Bool(b) => {
            if *b {
                out.push_str("true");
            } else {
                out.push_str("false");
            }
        }
        serde_json::Value::Number(n) => {
            if n.is_f64() {
                return Err("floating point numbers are forbidden");
            }
            out.push_str(&n.to_string());
        }
        serde_json::Value::String(s) => {
            // Write string with serde_json escape
            let s_escaped = serde_json::to_string(s).map_err(|_| "string serialization error")?;
            out.push_str(&s_escaped);
        }
        serde_json::Value::Array(arr) => {
            out.push('[');
            for (idx, elem) in arr.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                serialize_val(elem, out)?;
            }
            out.push(']');
        }
        serde_json::Value::Object(map) => {
            out.push('{');
            // Sort keys in lexicographical ascending order
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for (idx, k) in keys.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                let k_escaped = serde_json::to_string(k).map_err(|_| "key serialization error")?;
                out.push_str(&k_escaped);
                out.push(':');
                if let Some(v) = map.get(*k) {
                    serialize_val(v, out)?;
                }
            }
            out.push('}');
        }
    }
    Ok(())
}
