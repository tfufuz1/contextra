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

    // Detect duplicate keys and ensure canonical key ordering at raw JSON syntax level
    check_no_duplicate_keys(text)?;

    // Check that text exactly matches our strict canonical serialization
    let canonical = to_canonical_json_string(&val)?;

    if canonical.as_bytes() != bytes {
        return Err("bytes do not match canonical JSON serialization");
    }

    Ok(())
}

/// Strict check for duplicate keys in any JSON object.
fn check_no_duplicate_keys(json_str: &str) -> Result<(), &'static str> {
    let mut deserializer = serde_json::Deserializer::from_str(json_str);
    check_value_duplicates(&mut deserializer)?;
    Ok(())
}

fn check_value_duplicates<'de, R: serde_json::de::Read<'de>>(
    de: &mut serde_json::Deserializer<R>,
) -> Result<(), &'static str> {
    use serde::de::DeserializeSeed;

    struct DuplicateChecker;

    impl<'de> serde::de::Visitor<'de> for DuplicateChecker {
        type Value = ();

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("valid JSON without duplicate keys")
        }

        fn visit_bool<E>(self, _v: bool) -> Result<Self::Value, E> {
            Ok(())
        }

        fn visit_i64<E>(self, _v: i64) -> Result<Self::Value, E> {
            Ok(())
        }

        fn visit_u64<E>(self, _v: u64) -> Result<Self::Value, E> {
            Ok(())
        }

        fn visit_f64<E>(self, _v: f64) -> Result<Self::Value, E> {
            Ok(())
        }

        fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E> {
            Ok(())
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(())
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(DuplicateChecker)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            while let Some(()) = seq.next_element_seed(DuplicateCheckerSeed)? {}
            Ok(())
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut seen_keys = std::collections::BTreeSet::new();
            while let Some(key) = map.next_key::<String>()? {
                if !seen_keys.insert(key) {
                    return Err(serde::de::Error::custom("duplicate key in JSON object"));
                }
                map.next_value_seed(DuplicateCheckerSeed)?;
            }
            Ok(())
        }
    }

    struct DuplicateCheckerSeed;

    impl<'de> DeserializeSeed<'de> for DuplicateCheckerSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(DuplicateChecker)
        }
    }

    DuplicateCheckerSeed
        .deserialize(de)
        .map_err(|_| "duplicate key in JSON object or syntax error")
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
