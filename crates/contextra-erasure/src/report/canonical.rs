//! Kanonisierung des Berichts als deterministisches JSON (SPEC §11.7, Anhang F.10, ER-27).
//!
//! Kanonisches JSON (F.10):
//!   - UTF-8 NFC
//!   - Sortierte Objektschlüssel
//!   - Keine Leerzeichen außerhalb von Strings
//!   - Keine Gleitkommazahlen (Ganzzahlen, Dezimalstrings für Dauern)
//!   - Zeiten als Ganzzahl-Mikrosekunden UTC
//!   - Dauern als Dezimalstring

use crate::error::ErasureError;
use serde::Serialize;
use serde_json::Value;

/// Prüft, ob ein UTF-8-String die Anforderungen an Normalform C (NFC) erfüllt (SPEC F.10).
///
/// Zur Vermeidung neuer Abhängigkeiten (AR-3, Regel 4) wird eine deterministische Prüfung
/// auf nicht zusammengesetzte Kombinationszeichen durchgeführt:
/// Dekomponierte Folgen (Combining Diacritical Marks U+0300..=U+036F u. a.)
/// werden erkannt und abgelehnt.
pub fn verify_canonical_nfc(s: &str) -> Result<(), ErasureError> {
    for c in s.chars() {
        let u = c as u32;
        if (0x0300..=0x036F).contains(&u)
            || (0x1AB0..=0x1AFF).contains(&u)
            || (0x1DC0..=0x1DFF).contains(&u)
            || (0x20D0..=0x20FF).contains(&u)
            || (0xFE20..=0xFE2F).contains(&u)
        {
            return Err(ErasureError::Json(
                "Non-NFC Unicode detected: decomposed combining diacritical marks are rejected (F.10)"
                    .to_string(),
            ));
        }
    }
    Ok(())
}

/// Kanonisiert einen `serde_json::Value` nach F.10.
pub fn canonicalize_value(v: &Value) -> Result<Value, ErasureError> {
    match v {
        Value::Object(map) => {
            let mut sorted: Vec<(String, Value)> = Vec::with_capacity(map.len());
            for (k, val) in map {
                verify_canonical_nfc(k)?;
                let c_val = canonicalize_value(val)?;
                sorted.push((k.clone(), c_val));
            }
            sorted.sort_by(|(a, _), (b, _)| a.cmp(b));
            Ok(Value::Object(sorted.into_iter().collect()))
        }
        Value::Array(arr) => {
            let mut c_arr = Vec::with_capacity(arr.len());
            for item in arr {
                c_arr.push(canonicalize_value(item)?);
            }
            Ok(Value::Array(c_arr))
        }
        Value::String(s) => {
            verify_canonical_nfc(s)?;
            Ok(Value::String(s.clone()))
        }
        Value::Number(num) => {
            if num.is_f64() && !num.is_i64() && !num.is_u64() {
                return Err(ErasureError::Json(
                    "Floating point numbers are forbidden in canonical report JSON (SPEC F.10)"
                        .to_string(),
                ));
            }
            Ok(Value::Number(num.clone()))
        }
        other => Ok(other.clone()),
    }
}

/// Serialisiert einen Wert als kanonisches JSON (keine Leerzeichen, sortierte Schlüssel).
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ErasureError> {
    let v = serde_json::to_value(value).map_err(|e| ErasureError::Json(e.to_string()))?;
    let canonical = canonicalize_value(&v)?;
    serde_json::to_vec(&canonical).map_err(|e| ErasureError::Json(e.to_string()))
}

/// Serialisiert und kanonisiert einen Wert als String.
pub fn to_canonical_json_string<T: Serialize>(value: &T) -> Result<String, ErasureError> {
    let bytes = to_canonical_json(value)?;
    String::from_utf8(bytes).map_err(|e| ErasureError::Json(e.to_string()))
}

/// Alias für `to_canonical_json_string` für Abwärtskompatibilität.
pub fn canonicalize_json<T: Serialize>(val: &T) -> Result<String, ErasureError> {
    to_canonical_json_string(val)
}
