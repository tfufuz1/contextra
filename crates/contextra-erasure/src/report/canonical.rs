//! Kanonisierung des Berichts als deterministisches JSON (SPEC §11.7, Anhang F.10, ER-27).
//!
//! Kanonisches JSON (F.10):
//!   - UTF-8 NFC
//!   - Sortierte Objektschlüssel
//!   - Keine Leerzeichen außerhalb von Strings
//!   - Keine Gleitkommazahlen
//!   - Zeiten als Ganzzahl-Mikrosekunden UTC
//!   - Dauern als Dezimalstring

use crate::error::ErasureError;
use serde::Serialize;
use serde_json::Value;

/// Kanonisiert einen `serde_json::Value` nach F.10.
///
/// Sortiert rekursiv alle Objektschlüssel, entfernt Whitespace.
pub fn canonicalize_value(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut sorted: Vec<(String, Value)> = map
                .iter()
                .map(|(k, v)| (k.clone(), canonicalize_value(v)))
                .collect();
            sorted.sort_by(|(a, _), (b, _)| a.cmp(b));
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(arr) => Value::Array(arr.iter().map(canonicalize_value).collect()),
        other => other.clone(),
    }
}

/// Serialisiert einen Wert als kanonisches JSON (keine Leerzeichen, sortierte Schlüssel).
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ErasureError> {
    let v = serde_json::to_value(value).map_err(|e| ErasureError::Json(e.to_string()))?;
    let canonical = canonicalize_value(&v);
    serde_json::to_vec(&canonical).map_err(|e| ErasureError::Json(e.to_string()))
}

/// Serialisiert und kanonisiert einen Wert als String.
pub fn to_canonical_json_string<T: Serialize>(value: &T) -> Result<String, ErasureError> {
    let bytes = to_canonical_json(value)?;
    String::from_utf8(bytes).map_err(|e| ErasureError::Json(e.to_string()))
}

/// Alias für `to_canonical_json_string` für Abwärtskompatibilität.
pub fn canonicalize_json<T: serde::Serialize>(
    val: &T,
) -> Result<String, crate::error::ErasureError> {
    to_canonical_json_string(val)
}
