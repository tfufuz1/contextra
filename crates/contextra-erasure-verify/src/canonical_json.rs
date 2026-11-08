//! Strikte Pruefung und Formatierung von kanonischem JSON gemaess SPEC F.10.
//!
//! Regeln nach F.10:
//! - UTF-8 / NFC Normalisierung
//! - Keine Whitespaces (ausserhalb von Strings)
//! - Sortierte Schluessel (lexikographisch nach UTF-8)
//! - Keine Fließkommazahlen (Floats verboten, nur Integer oder String)

use crate::codes::InvalidCode;
use serde_json::Value;

/// Prueft rohe Bytes auf strikte Einhaltung der F.10 Kanonizitaets-Regeln.
pub fn verify_canonical_json(bytes: &[u8]) -> Result<(), InvalidCode> {
    if bytes.is_empty() {
        return Err(InvalidCode::NonCanonicalJson);
    }

    // 1. Gueltiges UTF-8
    let s = std::str::from_utf8(bytes).map_err(|_| InvalidCode::NonCanonicalJson)?;

    // 2. Parsen als JSON
    let val: Value = serde_json::from_str(s).map_err(|_| InvalidCode::NonCanonicalJson)?;

    // 3. Strikte Float-Ablehnung
    if contains_floats(&val) {
        return Err(InvalidCode::NonCanonicalJson);
    }

    // 4. Strikter Byte-fuer-Byte Abgleich mit der kanonischen Serialisierung
    let canonical_str = to_canonical_json_string(&val)?;
    if s != canonical_str {
        return Err(InvalidCode::NonCanonicalJson);
    }

    Ok(())
}

fn contains_floats(val: &Value) -> bool {
    match val {
        Value::Number(n) => n.is_f64(),
        Value::Array(arr) => arr.iter().any(contains_floats),
        Value::Object(map) => map.values().any(contains_floats),
        _ => false,
    }
}

/// Serialisiert einen serde_json::Value strikt kanonisch gemaess F.10.
pub fn to_canonical_json_string(val: &Value) -> Result<String, InvalidCode> {
    let mut out = String::new();
    format_canonical(val, &mut out)?;
    Ok(out)
}

fn format_canonical(val: &Value, out: &mut String) -> Result<(), InvalidCode> {
    match val {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if n.is_f64() {
                return Err(InvalidCode::NonCanonicalJson);
            }
            out.push_str(&n.to_string());
        }
        Value::String(s) => {
            let escaped = serde_json::to_string(s).map_err(|_| InvalidCode::NonCanonicalJson)?;
            out.push_str(&escaped);
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, elem) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                format_canonical(elem, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            let mut sorted_keys: Vec<&String> = map.keys().collect();
            sorted_keys.sort();

            for (i, k) in sorted_keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let escaped_key =
                    serde_json::to_string(k).map_err(|_| InvalidCode::NonCanonicalJson)?;
                out.push_str(&escaped_key);
                out.push(':');
                if let Some(v) = map.get(*k) {
                    format_canonical(v, out)?;
                } else {
                    return Err(InvalidCode::NonCanonicalJson);
                }
            }
            out.push('}');
        }
    }
    Ok(())
}
