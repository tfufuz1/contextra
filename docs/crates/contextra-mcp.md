# contextra-mcp — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E3 (Endpunkte / Anwendungen)  
**Kern-Closure:** `contextra`, `types`, `ports`, `rank`, `crypto`, `privacy`  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-mcp`  
**Referenzdokumente:** `docs/SPEC.md` §1, §2, §3, §6; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-02 bis AP-06, AP-13d, AP-18b)  

---

## 1. Zweck & Architekturgarantien

`contextra-mcp` implementiert den offiziellen Model Context Protocol (MCP) Server für Contextra:
- **LLM-Integration:** Bereitstellung von standardisierten JSON-RPC 2.0 Tools und Ressourcen für KI-Agenten (Claude Desktop, Cursor, AGY CLI) ausschließlich über `stdio` (kein HTTP / kein Axum, ADR-010).
- **Such- und Retrieval-Tools:** Hybride Suche (`search_hybrid`), semantische Vektorsuche, Volltextsuche und Wissensgraph-Traversierung.
- **DSGVO-Löschverwaltung:** Werkzeuge zur Initiierung und Überprüfung von Löschsagas (`erase_subject`, `verify_erasure`).
- **Egress-Schutz & DLP:** Filterung sensibler PII vor der Auslieferung an den MCP-Client (`contextra-privacy`).

### Beseitigung aller Altlasten (AP-02 bis AP-06)
1. **AP-02 (`protocol.rs`):** Re-Export aus `contextra-wire` wird durch lokale Definition von `JsonRpcRequest`, `JsonRpcResponse` und `JsonRpcError` ersetzt.
2. **AP-03 (`prompt_injection.rs`):** Die Abhängigkeit auf `contextra-prompt-guard` wird durch lokale Stubs ersetzt.
3. **AP-04 & AP-06 (`routing.rs`, `contextra-mcp-server.rs`):** Alle Referenzen auf `contextra-router` (`SlmProfile`, `DefaultRouterEngine`) werden entfernt; `setup_routing` liefert `Ok(None)`.
4. **AP-05 (`config.rs`):** Entfernen der Provider-Branches für `candle`, `ollama` und `onnx`; Provider liefern kontrollierte Fehler (`uncompiled_provider_error`).
5. **AP-13d & AP-18b:** Test- und Fragmentbereinigungen.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-12)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Reine `stdio`-Kommunikation:** Kein Netzwerktransport, keine offenen Sockets (ADR-010).
3. **Fail-Closed vor Datenausgabe (SPEC §6.2):** Jeder Tool-Aufruf prüft synchron das `ReadGate`. Gelöschte Dokumente sind unsichtbar.
4. **Keine PII in Logs (INV-12):** stderr-Logging enthält niemals Payloads, Prompts oder Schlüssel.

---

## 2. Kanonisches JSON-RPC 2.0 Protokoll (`protocol.rs`, AP-02)

```rust
use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Fehlerobjekt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Anfrageobjekt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Antwortobjekt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn err(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError { code, message, data: None }),
            id,
        }
    }
}
```

---

## 3. Bereitgestellte MCP-Tools

### 3.1 `search_hybrid`
- **Eingabe:** `{ "query": string, "collection": string, "top_k": int, "as_of": optional int }`
- **Ausgabe:** Liste relevanter Dokumentensnippets.
- **Invariante:** Ergebnisse durchlaufen zwingend das `ReadGate`. Gelöschte Dokumente tauchen niemals auf.

### 3.2 `get_document`
- **Eingabe:** `{ "collection": string, "doc_id": string }`
- **Ausgabe:** Dokumentinhalt. Bei gelöschten Dokumenten `404 Not Found` (Fail-Closed).

### 3.3 `erase_subject` (DSGVO Art. 17)
- **Eingabe:** `{ "subject_ref": string, "attestation_code": string, "operator_ref": string }`
- **Ablauf:** Startet die 6-Schritte-Löschsaga in `contextra-erasure`.
- **Ausgabe:** Signiertes `EraseReceipt` (Status `KeyRevoked`).

### 3.4 `verify_erasure`
- **Eingabe:** `{ "erasure_id": string }`
- **Ausgabe:** Aktueller Status der Saga (`Completed`, `Pending`, etc.).

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-mcp"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Model Context Protocol (MCP) server for Contextra (stdio transport)"

[lints]
workspace = true

[package.metadata.contextra]
ring = "3"
level = 3

[dependencies]
contextra = { workspace = true }
contextra-types = { workspace = true }
contextra-ports = { workspace = true }
contextra-rank = { workspace = true }
contextra-crypto = { workspace = true }
contextra-privacy = { workspace = true }
tokio = { workspace = true }
tokio-util = { version = "0.7", features = ["codec"] }
futures-util = "0.3"
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }

[features]
default = []
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **AP-02 (`protocol.rs`):**
   - Wire-Importe durch lokale Structs `JsonRpcError`, `JsonRpcRequest`, `JsonRpcResponse` ersetzen.
2. **AP-03 (`prompt_injection.rs`):**
   - Kompletten Inhalt durch Stub-Definitionen ersetzen.
3. **AP-04 & AP-06 (`routing.rs`, `contextra-mcp-server.rs`):**
   - Router-Imports und Profile-Lader entfernen.
4. **AP-05 (`config.rs`):**
   - Unkompilierbare Inferenz-Provider-Branches (Candle, Ollama, ONNX) entfernen.
5. **AP-13d (`tests/mcp_test.rs`):**
   - Obsoleten Routing-Test anpassen oder entfernen.
6. **Verifikation:**
   - `cargo test -p contextra-mcp`
