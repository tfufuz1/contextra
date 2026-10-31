// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Hardened JSON-RPC 2.0 protocol request/response processing for Contextra IPC.
// INVARIANTEN: Strict JSON-RPC 2.0 wire spec compliance; zero panic in production code; zero unsafe; zero unwrap/expect.

//! Generic JSON-RPC 2.0 protocol types and hardened parser for `Contextra` IPC.
//!
//! # Gehärtetes JSON-RPC-2.0-Protokoll-Modul
//!
//! **Ring:** Ring 0 (Unsafe-Insel 3 von 3), Klasse Kern `[K]`.
//!
//! ## Zweck
//! Dieses Modul stellt Typen und Parser für das JSON-RPC-2.0-Drahtprotokoll bereit. Es wird für
//! IPC-Nachrichten (z. B. zwischen CLI/MCP/Router und Engine) eingesetzt.
//!
//! ## Sicherheitsinvariante & Zero-Panic-Garantie
//! Die Verarbeitung erfolgt strikt defensiv (Fail-Closed) und ohne `unsafe`-Code:
//! 1. **Größenbeschränkung (`max_message_size`):** Verhindert Erschöpfung des Arbeitsspeichers durch übergroße Payloads.
//! 2. **Tiefenbegrenzung (`max_depth`):** Schützt vor Stack-Overflow-Angriffen durch stark geschachtelte JSON-Strukturen (Depth-Bombs).
//! 3. **Batch-Größenlimit (`max_batch_size`):** Verhindert CPU-Überlastung durch riesige Anfragen-Arrays.
//! 4. **Duplikatschutz (`reject_duplicate_keys`):** Verhindert semantische Unklarheiten und Smuggling durch doppelte Objektschlüssel.
//! 5. **Zero-Panic:** Kein `unwrap()`, kein `expect()`, kein `panic!`. Alle Fehler werden als standardkonforme `JsonRpcError`-Objekte geliefert.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

// ANCHOR[contextra-wire:RPC-001]
/// Configurable limits and security parameters for JSON-RPC message processing.
///
/// Konfigurierbare Obergrenzen und Sicherheitsparameter für die JSON-RPC-Verarbeitung.
#[derive(Debug, Clone)]
pub struct JsonRpcConfig {
    /// Maximum allowed payload size in bytes (default: 10 MiB).
    /// Maximal erlaubte Nachrichtengröße in Bytes (Standard: 10 MiB).
    pub max_message_size: usize,
    /// Maximum allowed JSON structural nesting depth (default: 64).
    /// Maximal erlaubte JSON-Schachtelungstiefe für Objekte und Arrays (Standard: 64).
    pub max_depth: usize,
    /// Maximum number of requests allowed in a batch payload (default: 100).
    /// Maximale Anzahl an Anfragen in einem Batch-Payload (Standard: 100).
    pub max_batch_size: usize,
    /// Reject JSON objects containing duplicate key names (default: true).
    /// JSON-Objekte mit doppelten Schlüsselnamen strikt ablehnen (Standard: true).
    pub reject_duplicate_keys: bool,
}

impl Default for JsonRpcConfig {
    // ANCHOR[contextra-wire:RPC-002]
    /// Erzeugt Standardwerte mit sicheren Produktionsgrenzen.
    fn default() -> Self {
        Self {
            max_message_size: 10 * 1024 * 1024,
            max_depth: 64,
            max_batch_size: 100,
            reject_duplicate_keys: true,
        }
    }
}

// ANCHOR[contextra-wire:RPC-003]
/// Parsed JSON-RPC payload representing either a single request or a batch of requests.
///
/// Geparste JSON-RPC-Payload: Repräsentiert entweder eine Einzelanfrage oder einen Batch von Anfragen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcPayload {
    /// Single JSON-RPC request.
    /// Einzelne JSON-RPC-Anfrage (`JsonRpcRequest`).
    Single(JsonRpcRequest),
    /// Batch of JSON-RPC requests.
    /// Batch-Array mehrerer JSON-RPC-Anfragen (`Vec<JsonRpcRequest>`).
    Batch(Vec<JsonRpcRequest>),
}

// ANCHOR[contextra-wire:RPC-004]
/// Incoming JSON-RPC 2.0 message (Request or Notification).
///
/// Eingehende JSON-RPC 2.0-Nachricht (Anfrage oder Benachrichtigung).
/// Handelt es sich um eine Benachrichtigung (Notification), ist das Feld `id` auf `None` gesetzt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Always "2.0".
    /// Protokollversion: Muss strikt `"2.0"` sein.
    pub jsonrpc: String,
    /// `None` for notifications (no response expected).
    /// Anforderungs-ID. `None` für Benachrichtigungen (keine Antwort erwartet).
    pub id: Option<Value>,
    /// Name of the RPC method to invoke.
    /// Name der aufzurufenden RPC-Methode (darf nicht leer sein).
    pub method: String,
    /// Parameter payload (Array or Object).
    /// Parameter der Methode (beliebiger JSON-Wert, typischerweise Array oder Object).
    #[serde(default)]
    pub params: Value,
}

impl JsonRpcRequest {
    // ANCHOR[contextra-wire:RPC-005]
    /// Validates that the request complies with the JSON-RPC 2.0 specification (fail-closed).
    ///
    /// # Ablauf
    /// 1. Prüfen, ob `jsonrpc` exakt `"2.0"` entspricht. Falls nicht -> `InvalidRequest`.
    /// 2. Prüfen, ob `method` nach Entfernung von führenden/nachfolgenden Leerzeichen nicht leer ist. Falls leer -> `InvalidRequest`.
    /// 3. Liefert `Ok(())` bei Konformität.
    pub fn validate(&self) -> Result<(), JsonRpcError> {
        if self.jsonrpc != "2.0" {
            return Err(JsonRpcError::invalid_request(format!(
                "Invalid JSON-RPC protocol version: expected '2.0', got '{}'",
                self.jsonrpc
            )));
        }
        if self.method.trim().is_empty() {
            return Err(JsonRpcError::invalid_request(
                "Invalid Request: method name must not be empty",
            ));
        }
        Ok(())
    }
}

// ANCHOR[contextra-wire:RPC-006]
/// Outgoing JSON-RPC 2.0 response.
///
/// Ausgehende JSON-RPC 2.0-Antwort (Erfolg oder Fehler).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Always "2.0".
    /// Protokollversion: Immer `"2.0"`.
    pub jsonrpc: String,
    /// Request ID (`None` for parse/batch errors).
    /// Zugehörige Anforderungs-ID der Anfrage (`None` bei Parse- oder Batch-Fehlern).
    pub id: Option<Value>,
    /// Result payload on success.
    /// Ergebnis-Payload im Erfolgsfall.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error object on failure.
    /// Fehlerobjekt im Fehlerfall.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

// ANCHOR[contextra-wire:RPC-007]
/// JSON-RPC 2.0 Error object.
///
/// Standardisiertes JSON-RPC 2.0 Fehlerobjekt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    /// Numeric error code per JSON-RPC 2.0 specification.
    /// Numerischer standardisierter Fehlercode gemäß JSON-RPC 2.0 Spezifikation.
    pub code: i32,
    /// Brief error message.
    /// Kurze menschenlesbare Fehlerbeschreibung.
    pub message: String,
    /// Optional additional error context data.
    /// Optionale strukturierte Zusatzdaten zum Fehlerkontext.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC Error [{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

impl JsonRpcError {
    // ANCHOR[contextra-wire:RPC-008]
    /// Parse error (-32700): Invalid JSON received by server.
    /// Erzeugt einen Parse error (-32700): Ungültiges JSON empfangen.
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: message.into(),
            data: None,
        }
    }

    // ANCHOR[contextra-wire:RPC-009]
    /// Invalid Request (-32600): JSON sent is not a valid Request object.
    /// Erzeugt einen Invalid Request (-32600): Gesendetes JSON ist keine gültige Anfrage.
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: message.into(),
            data: None,
        }
    }

    // ANCHOR[contextra-wire:RPC-010]
    /// Method not found (-32601): The method does not exist / is not available.
    /// Erzeugt einen Method not found (-32601): Methode existiert nicht oder ist nicht verfügbar.
    pub fn method_not_found(message: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: message.into(),
            data: None,
        }
    }

    // ANCHOR[contextra-wire:RPC-011]
    /// Invalid params (-32602): Invalid method parameter(s).
    /// Erzeugt einen Invalid params (-32602): Ungültige Methodenparameter übergeben.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
            data: None,
        }
    }

    // ANCHOR[contextra-wire:RPC-012]
    /// Internal error (-32603): Internal JSON-RPC error.
    /// Erzeugt einen Internal error (-32603): Interner JSON-RPC Serverfehler.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: message.into(),
            data: None,
        }
    }
}

impl JsonRpcResponse {
    // ANCHOR[contextra-wire:RPC-013]
    /// Constructs a successful JSON-RPC response.
    ///
    /// Konstruiert eine erfolgreiche JSON-RPC 2.0-Antwort mit Ergebnisdaten.
    ///
    /// # Parameter
    /// - `id`: ID der beantworteten Anfrage.
    /// - `result`: Rückgabedaten als JSON-Wert.
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    // ANCHOR[contextra-wire:RPC-014]
    /// Constructs an error JSON-RPC response.
    ///
    /// Konstruiert eine JSON-RPC 2.0-Fehlerantwort.
    ///
    /// # Parameter
    /// - `id`: ID der Anfrage (oder `None`, falls unbekannt).
    /// - `code`: Numerischer JSON-RPC Fehlercode.
    /// - `message`: Fehlerbeschreibung.
    pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

// ANCHOR[contextra-wire:RPC-015]
/// Parses raw bytes into a `JsonRpcPayload` using default configuration limits.
///
/// Parst rohe Bytes in ein `JsonRpcPayload` unter Verwendung der Standard-Konfiguration.
pub fn parse_jsonrpc_payload(raw: &[u8]) -> Result<JsonRpcPayload, JsonRpcError> {
    parse_jsonrpc_payload_with_config(raw, &JsonRpcConfig::default())
}

// ANCHOR[contextra-wire:RPC-016]
/// Parses raw bytes into a `JsonRpcPayload` with strict security and size validation.
///
/// Parst rohe Bytes in ein `JsonRpcPayload` mit strikter Sicherheits- und Größenvalidierung.
///
/// # Ablauf von `parse_jsonrpc_payload_with_config`:
/// 1. Puffergröße gegen `config.max_message_size` prüfen.
/// 2. UTF-8-Konformität sicherstellen.
/// 3. Vorvalidierung der Struktur via `validate_json_structure_and_limits` (Schachtelungstiefe und Duplikate).
/// 4. Deserialisierung in generischen `serde_json::Value`.
/// 5. Verzweigung nach Wurzelelement:
///    - `Value::Array`: Batch-Verarbeitung. Prüfen auf Nicht-Leerheit und `max_batch_size`. Jedes Element validieren.
///    - `Value::Object`: Einzelanfrage parsen und validieren.
///    - Primitive Werte: Ablehnung mit `InvalidRequest`.
///
/// # Fehlerfälle
/// Liefert `JsonRpcError::parse_error` bei Größen-, UTF-8- oder Syntaxfehlern,
/// oder `JsonRpcError::invalid_request` bei Schemaverletzungen.
pub fn parse_jsonrpc_payload_with_config(
    raw: &[u8],
    config: &JsonRpcConfig,
) -> Result<JsonRpcPayload, JsonRpcError> {
    if raw.len() > config.max_message_size {
        return Err(JsonRpcError::parse_error(format!(
            "Payload size ({}) exceeds max message limit ({})",
            raw.len(),
            config.max_message_size
        )));
    }

    let text = std::str::from_utf8(raw)
        .map_err(|e| JsonRpcError::parse_error(format!("Invalid UTF-8 payload: {e}")))?;

    validate_json_structure_and_limits(text, config)?;

    let val: Value = serde_json::from_str(text)
        .map_err(|e| JsonRpcError::parse_error(format!("JSON parse error: {e}")))?;

    match val {
        Value::Array(items) => {
            if items.is_empty() {
                return Err(JsonRpcError::invalid_request(
                    "Invalid Request: empty batch array",
                ));
            }
            if items.len() > config.max_batch_size {
                return Err(JsonRpcError::invalid_request(format!(
                    "Batch size ({}) exceeds maximum limit ({})",
                    items.len(),
                    config.max_batch_size
                )));
            }

            let mut requests = Vec::with_capacity(items.len());
            for item in items {
                let req: JsonRpcRequest = serde_json::from_value(item).map_err(|e| {
                    JsonRpcError::invalid_request(format!("Batch element invalid request: {e}"))
                })?;
                req.validate()?;
                requests.push(req);
            }
            Ok(JsonRpcPayload::Batch(requests))
        }
        Value::Object(_) => {
            let req: JsonRpcRequest = serde_json::from_value(val).map_err(|e| {
                JsonRpcError::invalid_request(format!("Invalid request structure: {e}"))
            })?;
            req.validate()?;
            Ok(JsonRpcPayload::Single(req))
        }
        _ => Err(JsonRpcError::invalid_request(
            "Invalid Request: expected JSON object or array",
        )),
    }
}

// ANCHOR[contextra-wire:RPC-017]
/// Helper struct for tracking object state during JSON key scanning.
///
/// Hilfsstruktur zur Zustandsverfolgung während des Scannens von JSON-Objektschlüsseln.
#[derive(Debug)]
struct ObjectKeyScope {
    seen_keys: BTreeSet<String>,
    expecting_key: bool,
}

// ANCHOR[contextra-wire:RPC-018]
/// Validates structural nesting depth and duplicate key constraints.
///
/// Validiert Schachtelungstiefe und Duplikate von Schlüsseln in linearem Durchlauf.
/// Verarbeitet den String Byte für Byte ohne Heap-Allokationen für den Tokenizer,
/// um DoS-Angriffe frühzeitig vor der eigentlichen DOM-Erstellung abzufangen.
fn validate_json_structure_and_limits(
    text: &str,
    config: &JsonRpcConfig,
) -> Result<(), JsonRpcError> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escape = false;

    let bytes = text.as_bytes();
    let mut i = 0;

    let mut object_stack: Vec<ObjectKeyScope> = Vec::new();

    while i < bytes.len() {
        let b = match bytes.get(i) {
            Some(&b) => b,
            None => break,
        };

        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match b {
            b'"' => {
                in_string = true;
                if config.reject_duplicate_keys {
                    if let Some(scope) = object_stack.last_mut() {
                        if scope.expecting_key {
                            // Extract key string starting at i + 1
                            // Schlüssel-String ab Position i + 1 extrahieren
                            let key_start = i + 1;
                            let mut key_end = key_start;
                            let mut key_escape = false;

                            while key_end < bytes.len() {
                                let kb = match bytes.get(key_end) {
                                    Some(&kb) => kb,
                                    None => break,
                                };
                                if key_escape {
                                    key_escape = false;
                                } else if kb == b'\\' {
                                    key_escape = true;
                                } else if kb == b'"' {
                                    break;
                                }
                                key_end += 1;
                            }

                            // Look ahead past string to check for ':'
                            // Vorausschau nach dem String auf Doppelpunkt ':'
                            let mut next_idx = key_end + 1;
                            while next_idx < bytes.len() {
                                if match bytes.get(next_idx) {
                                    Some(&nb) => nb.is_ascii_whitespace(),
                                    None => false,
                                } {
                                    next_idx += 1;
                                } else {
                                    break;
                                }
                            }

                            if match bytes.get(next_idx) {
                                Some(&nb) => nb == b':',
                                None => false,
                            } {
                                if let Some(key_bytes) = bytes.get(key_start..key_end) {
                                    if let Ok(key_str) = std::str::from_utf8(key_bytes) {
                                        if scope.seen_keys.contains(key_str) {
                                            return Err(JsonRpcError::parse_error(format!(
                                                "Duplicate key detected in JSON object: '{key_str}'"
                                            )));
                                        }
                                        scope.seen_keys.insert(key_str.to_string());
                                    }
                                }
                                scope.expecting_key = false;
                            }
                        }
                    }
                }
            }
            b'{' => {
                depth += 1;
                if depth > config.max_depth {
                    return Err(JsonRpcError::parse_error(format!(
                        "JSON nesting depth ({depth}) exceeds maximum limit ({})",
                        config.max_depth
                    )));
                }
                if config.reject_duplicate_keys {
                    object_stack.push(ObjectKeyScope {
                        seen_keys: BTreeSet::new(),
                        expecting_key: true,
                    });
                }
            }
            b'[' => {
                depth += 1;
                if depth > config.max_depth {
                    return Err(JsonRpcError::parse_error(format!(
                        "JSON nesting depth ({depth}) exceeds maximum limit ({})",
                        config.max_depth
                    )));
                }
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                if config.reject_duplicate_keys {
                    object_stack.pop();
                }
            }
            b']' => {
                depth = depth.saturating_sub(1);
            }
            b',' => {
                if config.reject_duplicate_keys {
                    if let Some(scope) = object_stack.last_mut() {
                        scope.expecting_key = true;
                    }
                }
            }
            _ => {}
        }

        i += 1;
    }

    Ok(())
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
    use serde_json::json;

    // Prüft die Serialisierung und Deserialisierung einer erfolgreichen JsonRpcResponse.
    #[test]
    fn test_json_rpc_response_ok_serialization() {
        let resp = JsonRpcResponse::ok(Some(json!(42)), json!({"status": "success"}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, Some(json!(42)));
        assert!(resp.error.is_none());
        assert_eq!(resp.result, Some(json!({"status": "success"})));

        let ser = serde_json::to_string(&resp).expect("serialize response ok");
        let deser: JsonRpcResponse = serde_json::from_str(&ser).expect("deserialize response ok");
        assert_eq!(deser.jsonrpc, "2.0");
        assert_eq!(deser.id, Some(json!(42)));
        assert_eq!(deser.result, Some(json!({"status": "success"})));
        assert!(deser.error.is_none());
    }

    // Prüft die Serialisierung und Deserialisierung einer fehlerhaften JsonRpcResponse.
    #[test]
    fn test_json_rpc_response_err_serialization() {
        let resp = JsonRpcResponse::err(Some(json!("req-1")), -32600, "Invalid Request");
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, Some(json!("req-1")));
        assert!(resp.result.is_none());

        let err = resp.error.as_ref().expect("error payload present");
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "Invalid Request");
        assert!(err.data.is_none());

        let ser = serde_json::to_string(&resp).expect("serialize response err");
        let deser: JsonRpcResponse = serde_json::from_str(&ser).expect("deserialize response err");
        assert_eq!(deser.error.expect("error present").code, -32600);
    }

    // Prüft das erfolgreiche Validieren einer vorschriftsmäßigen JsonRpcRequest.
    #[test]
    fn test_json_rpc_request_validation_valid() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "search".to_string(),
            params: json!({"query": "rust"}),
        };
        assert!(req.validate().is_ok());
    }

    // Prüft die Zurückweisung einer ungültigen Protokollversion in JsonRpcRequest.
    #[test]
    fn test_json_rpc_request_validation_invalid_version() {
        let req = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            id: Some(json!(1)),
            method: "search".to_string(),
            params: json!({}),
        };
        let err = req.validate().unwrap_err();
        assert_eq!(err.code, -32600);
        assert!(err.message.contains("Invalid JSON-RPC protocol version"));
    }

    // Prüft die Zurückweisung einer leeren Methodenbezeichnung in JsonRpcRequest.
    #[test]
    fn test_json_rpc_request_validation_empty_method() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "   ".to_string(),
            params: json!({}),
        };
        let err = req.validate().unwrap_err();
        assert_eq!(err.code, -32600);
        assert!(err.message.contains("method name must not be empty"));
    }

    // Prüft die korrekte Zuordnung aller Fehlercodes und Konstruktoren von JsonRpcError.
    #[test]
    fn test_json_rpc_error_constructors_codes() {
        assert_eq!(JsonRpcError::parse_error("msg").code, -32700);
        assert_eq!(JsonRpcError::invalid_request("msg").code, -32600);
        assert_eq!(JsonRpcError::method_not_found("msg").code, -32601);
        assert_eq!(JsonRpcError::invalid_params("msg").code, -32602);
        assert_eq!(JsonRpcError::internal_error("msg").code, -32603);
    }

    // Prüft das Parsen einer gültigen einzelnen JSON-RPC-Anfrage.
    #[test]
    fn test_json_rpc_parse_single_valid() {
        let raw = r#"{"jsonrpc": "2.0", "id": 100, "method": "query", "params": {"k": "v"}}"#;
        let payload = parse_jsonrpc_payload(raw.as_bytes()).expect("parse single payload");
        match payload {
            JsonRpcPayload::Single(req) => {
                assert_eq!(req.jsonrpc, "2.0");
                assert_eq!(req.id, Some(json!(100)));
                assert_eq!(req.method, "query");
            }
            JsonRpcPayload::Batch(_) => panic!("Expected single payload"),
        }
    }

    // Prüft das Parsen einer gültigen Benachrichtigung ohne ID.
    #[test]
    fn test_json_rpc_parse_notification_valid() {
        let raw = r#"{"jsonrpc": "2.0", "method": "ping"}"#;
        let payload = parse_jsonrpc_payload(raw.as_bytes()).expect("parse notification");
        match payload {
            JsonRpcPayload::Single(req) => {
                assert_eq!(req.method, "ping");
                assert!(req.id.is_none());
            }
            JsonRpcPayload::Batch(_) => panic!("Expected single notification"),
        }
    }

    // Prüft das Parsen eines gültigen Arrays von mehreren JSON-RPC-Anfragen (Batch).
    #[test]
    fn test_json_rpc_parse_batch_valid() {
        let raw = r#"[
            {"jsonrpc": "2.0", "id": 1, "method": "m1"},
            {"jsonrpc": "2.0", "id": 2, "method": "m2"}
        ]"#;
        let payload = parse_jsonrpc_payload(raw.as_bytes()).expect("parse batch payload");
        match payload {
            JsonRpcPayload::Batch(reqs) => {
                assert_eq!(reqs.len(), 2);
                assert_eq!(reqs[0].method, "m1");
                assert_eq!(reqs[1].method, "m2");
            }
            JsonRpcPayload::Single(_) => panic!("Expected batch payload"),
        }
    }

    // Prüft, dass ein leeres Batch-Array mit InvalidRequest abgelehnt wird.
    #[test]
    fn test_json_rpc_parse_empty_batch_rejected() {
        let raw = "[]";
        let err = parse_jsonrpc_payload(raw.as_bytes()).unwrap_err();
        assert_eq!(err.code, -32600);
        assert!(err.message.contains("empty batch array"));
    }

    // Prüft, dass die Überschreitung des konfigurierten Batch-Limits abgelehnt wird.
    #[test]
    fn test_json_rpc_parse_batch_over_limit_rejected() {
        let config = JsonRpcConfig {
            max_batch_size: 2,
            ..Default::default()
        };
        let raw = r#"[
            {"jsonrpc": "2.0", "id": 1, "method": "m1"},
            {"jsonrpc": "2.0", "id": 2, "method": "m2"},
            {"jsonrpc": "2.0", "id": 3, "method": "m3"}
        ]"#;
        let err = parse_jsonrpc_payload_with_config(raw.as_bytes(), &config).unwrap_err();
        assert_eq!(err.code, -32600);
        assert!(err.message.contains("Batch size"));
    }

    // Prüft, dass überdimensionierte Payloads mit ParseError (-32700) abgelehnt werden.
    #[test]
    fn test_json_rpc_parse_oversized_payload_rejected() {
        let config = JsonRpcConfig {
            max_message_size: 20,
            ..Default::default()
        };
        let raw = r#"{"jsonrpc": "2.0", "id": 1, "method": "very_long_method_name"}"#;
        let err = parse_jsonrpc_payload_with_config(raw.as_bytes(), &config).unwrap_err();
        assert_eq!(err.code, -32700);
        assert!(err.message.contains("Payload size"));
    }

    // Prüft, dass ungültige UTF-8-Bytefolgen mit ParseError abgelehnt werden.
    #[test]
    fn test_json_rpc_parse_invalid_utf8_rejected() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD, 0x00];
        let err = parse_jsonrpc_payload(&invalid_utf8).unwrap_err();
        assert_eq!(err.code, -32700);
        assert!(err.message.contains("Invalid UTF-8"));
    }

    // Prüft, dass zu tiefe Schachtelungen (Depth-Bombs) abgefangen werden.
    #[test]
    fn test_json_rpc_parse_depth_bomb_rejected() {
        let config = JsonRpcConfig {
            max_depth: 3,
            ..Default::default()
        };
        let raw = r#"{"a": {"b": {"c": {"d": 1}}}}"#;
        let err = parse_jsonrpc_payload_with_config(raw.as_bytes(), &config).unwrap_err();
        assert_eq!(err.code, -32700);
        assert!(err.message.contains("nesting depth"));
    }

    // Prüft, dass doppelte Schlüssel in einem Objekt abgelehnt werden.
    #[test]
    fn test_json_rpc_parse_duplicate_key_rejected() {
        let raw = r#"{"jsonrpc": "2.0", "id": 1, "id": 2, "method": "ping"}"#;
        let err = parse_jsonrpc_payload(raw.as_bytes()).unwrap_err();
        assert_eq!(err.code, -32700);
        assert!(err.message.contains("Duplicate key"));
    }

    // Prüft, dass doppelte Schlüssel in verschachtelten Objekten erkannt und abgelehnt werden.
    #[test]
    fn test_json_rpc_parse_nested_duplicate_key_rejected() {
        let raw = r#"{"jsonrpc": "2.0", "method": "ping", "params": {"dup": 1, "dup": 2}}"#;
        let err = parse_jsonrpc_payload(raw.as_bytes()).unwrap_err();
        assert_eq!(err.code, -32700);
        assert!(err.message.contains("Duplicate key"));
    }

    // Prüft, dass primitive Typen als Wurzel-Payload mit InvalidRequest abgelehnt werden.
    #[test]
    fn test_json_rpc_parse_primitive_payload_rejected() {
        let raw = "12345";
        let err = parse_jsonrpc_payload(raw.as_bytes()).unwrap_err();
        assert_eq!(err.code, -32600);
    }
}
