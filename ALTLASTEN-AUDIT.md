# 🧹 Altlasten-Audit & Implementierungs-Spezifikation: Entfernte Crate-Referenzen

> **Stand:** 2026-11-18T16:38:00+01:00
> **Auditor:** Principal Rust Engineer
> **Scope:** Alle `.rs`, `.toml`-Dateien im aktiven Quellcode (exkl. `target/`, `archive/`)
> **Ziel:** Vollständige Entfernung aller Referenzen auf 15 nicht mehr vorhandene Crates

---

## Zusammenfassung

| Metrik | Wert |
|--------|------|
| Entfernte Crates mit Altlasten | **15** |
| Betroffene Dateien | **~30** |
| Gesamtfunde (Code + Kommentare + Config) | **181** |
| Davon **harte Code-Referenzen** (kompilierbar) | **~25** |
| Davon **Kommentar/Doc-Referenzen** | **~45** |
| Davon **Config/TOML-Referenzen** | **~111** |

### Entfernte Crates (15 Stück)

| Crate | Ehemaliger Zweck |
|-------|-----------------|
| `contextra-cognition` | Sleep-Cycle-Konsolidierung, Meta-Chunk-Synthese |
| `contextra-agent` | Agenten-Session-Verwaltung, CLM-Scratchpad |
| `contextra-adapt` | PID-Controller, Adaptive Decay |
| `contextra-wire` | JSON-RPC Serialisierungstypen, Unsafe Island |
| `contextra-py` | Python FFI Bindings (PyO3) |
| `contextra-prompt-guard` | Prompt-Injection-Erkennung |
| `contextra-ingest` | Dokumenten-Ingestion-Pipeline |
| `contextra-infer-onnx` | ONNX-Embedding-Backend |
| `contextra-infer-ollama` | Ollama-Embedding/LLM-Backend |
| `contextra-infer-candle` | Candle-Embedding/LLM-Backend |
| `contextra-sandbox` | WASM-Sandbox, Merge-Operatoren |
| `contextra-router` | SLM-Routing, Drift-Detection |
| `contextra-kv-cache` / `contextra-kvcache` | KV-Cache-Brücke für Inferenz |

---

## Arbeitspakete (AP)

---

### AP-01: `contextra/src/lib.rs` — Candle Re-Export entfernen

**Datei:** `crates/contextra/src/lib.rs`
**Zeile:** 59-60
**Schweregrad:** 🔴 Kompilier-Blocker (wenn Feature `candle` aktiviert)
**Feature-Gate:** `#[cfg(feature = "candle")]`

**Aktueller Code:**
```rust
#[cfg(feature = "candle")]
pub use contextra_infer_candle as candle;
```

**Aktion:** Zeilen 59-60 vollständig entfernen. Das Feature `candle` existiert nicht mehr in `crates/contextra/Cargo.toml` (dort sind keine `candle`-Features definiert), daher ist der Code bereits tot.

**Verifikation:** `cargo check -p contextra` muss grün bleiben.

---

### AP-02: `contextra-mcp/src/protocol.rs` — Wire-Re-Export durch lokale Typen ersetzen

**Datei:** `crates/contextra-mcp/src/protocol.rs`
**Zeile:** 87
**Schweregrad:** 🔴 Kompilier-Blocker

**Aktueller Code:**
```rust
pub use contextra_wire::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
```

**Aktion:** Die drei Typen `JsonRpcError`, `JsonRpcRequest`, `JsonRpcResponse` müssen lokal in `protocol.rs` definiert werden. Die Typen aus `contextra-wire` sind simple JSON-RPC 2.0 Strukturen:

**Ersetzung — Zeile 87 ersetzen durch:**
```rust
/// JSON-RPC 2.0 Error Object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Request Object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response Object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}
```

**Nacharbeiten:** Prüfen, ob `JsonRpcResponse::err()` die einzige verwendete Methode ist (wird in `response_from_error()` Z.126 verwendet). Falls `JsonRpcResponse::ok()` auch genutzt wird, muss das ebenfalls abgedeckt sein. Grep nach `JsonRpcResponse::` im gesamten `contextra-mcp`.

**Verifikation:** `cargo check -p contextra-mcp`

---

### AP-03: `contextra-mcp/src/prompt_injection.rs` — Komplett leeren (Stub)

**Datei:** `crates/contextra-mcp/src/prompt_injection.rs`
**Zeilen:** 1-76 (komplette Datei)
**Schweregrad:** 🔴 Kompilier-Blocker

**Aktueller Code:** Re-exportiert alle Typen aus `contextra_prompt_guard` (audit, guard, policy Module).

**Aktion:** Kompletten Inhalt ersetzen durch Stub-Typen, da `lib.rs` Z.65-68 diese Typen re-exportiert:
```rust
pub use prompt_injection::{
    PromptInjectionConfig, PromptInjectionGuard, QuarantinePolicy, SecurityAuditLogger,
    SecurityAuditRecord, DEFAULT_REDACTION_PLACEHOLDER,
};
```

**Ersetzung — gesamte Datei ersetzen durch:**
```rust
//! Prompt-Injection-Erkennung (Stub — contextra-prompt-guard wurde entfernt).
//!
//! Die externen Typen werden als leere Stubs beibehalten, um die öffentliche API
//! von `contextra-mcp` nicht zu brechen. Funktionalität ist deaktiviert.

/// Platzhalter für den entfernten Redaction-Marker.
pub const DEFAULT_REDACTION_PLACEHOLDER: &str = "[REDACTED]";

/// Stub-Konfiguration für Prompt-Injection-Erkennung.
#[derive(Debug, Clone, Default)]
pub struct PromptInjectionConfig {
    pub enabled: bool,
}

/// Stub-Quarantäne-Policy.
#[derive(Debug, Clone, Default)]
pub struct QuarantinePolicy;

/// Stub-Guard (No-Op).
#[derive(Debug, Clone)]
pub struct PromptInjectionGuard;

/// Stub-Audit-Logger (No-Op).
#[derive(Debug, Clone, Default)]
pub struct SecurityAuditLogger;

/// Stub-Audit-Record.
#[derive(Debug, Clone)]
pub struct SecurityAuditRecord;

pub mod audit {
    pub use super::{SecurityAuditLogger, SecurityAuditRecord};
}

pub mod guard {
    /// Stub für normalisierte Muster.
    #[derive(Debug, Clone)]
    pub struct NormalizedPattern;
    pub use super::PromptInjectionGuard;
}

pub mod policy {
    pub use super::{PromptInjectionConfig, QuarantinePolicy, DEFAULT_REDACTION_PLACEHOLDER};
}
```

**Verifikation:** `cargo check -p contextra-mcp` — alle Konsumenten von `prompt_injection::*` müssen kompilieren.

---

### AP-04: `contextra-mcp/src/routing.rs` — Router-Referenzen durch Ports-Traits ersetzen

**Datei:** `crates/contextra-mcp/src/routing.rs`
**Zeilen:** 71, 81, 100
**Schweregrad:** 🔴 Kompilier-Blocker

**Aktueller Code:** Verwendet `contextra_router::DefaultRouterEngine` und `contextra_router::RouterEngine::new()`.

**Aktion:** Das komplette Routing-Setup entfernen. `RoutingHandle` wird zum leeren Struct, `setup_routing` gibt immer `Ok(None)` zurück.

**Ersetzung — Zeilen 71-122 ersetzen durch:**
```rust
/// Holds strong Arc references to routing and calibration components.
/// NOTE: Router-Funktionalität wurde entfernt (contextra-router Crate nicht mehr vorhanden).
pub struct RoutingHandle {
    pub calibrator: Arc<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>,
}

pub async fn setup_routing(
    _db: &Arc<Contextra>,
    _config: &RouterConfig,
) -> Result<Option<Arc<RoutingHandle>>, ContextraError> {
    // Router-Funktionalität entfernt — Routing ist deaktiviert.
    Ok(None)
}
```

**Nacharbeit:** `RouterDriftAdapter` (Z.71-77) und alle `contextra_router::*`-Referenzen entfernen.

**Verifikation:** `cargo check -p contextra-mcp`

---

### AP-05: `contextra-mcp/src/config.rs` — Provider- und Router-Referenzen entfernen

**Datei:** `crates/contextra-mcp/src/config.rs`
**Zeilen:** 291-304 (candle), 309-313 (ollama), 317-329 (onnx), 444-470 (candle/ollama LLM), 487-530 (RouterConfig)
**Schweregrad:** 🔴 Kompilier-Blocker

**Aktionen (5 Teilschritte):**

**AP-05a:** `create_embedding_provider()` — Feature-gated Branches für `candle`, `ollama`, `onnx` entfernen.
Ersetze die `#[cfg(feature = "candle")]`-Blöcke (Z.291-307), `#[cfg(feature = "ollama")]`-Blöcke (Z.309-315), und `#[cfg(feature = "onnx")]`-Blöcke (Z.317-332) durch einheitliche `Err(uncompiled_provider_error(...))`:

```rust
"candle" => Err(uncompiled_provider_error("candle")),
"ollama" => Err(uncompiled_provider_error("ollama")),
"onnx" => Err(uncompiled_provider_error("onnx")),
```

**AP-05b:** `create_llm_text_generator()` — Analog candle/ollama Feature-Branches entfernen (Z.444-473).
Gleiche Ersetzung wie AP-05a.

**AP-05c:** `RouterConfig` — `contextra_router::SlmProfile` durch eigene Stub-Struktur ersetzen:

```rust
/// Router configuration settings (Stub — contextra-router wurde entfernt).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RouterConfig {
    pub profiles_path: Option<PathBuf>,
    pub calibration_store_path: Option<PathBuf>,
}

impl RouterConfig {
    pub fn from_env() -> Self {
        let profiles_path = std::env::var("CONTEXTRA_ROUTER_PROFILES_PATH")
            .ok()
            .map(PathBuf::from);
        let calibration_store_path = std::env::var("CONTEXTRA_ROUTER_CALIBRATION_PATH")
            .ok()
            .map(PathBuf::from);
        Self {
            profiles_path,
            calibration_store_path,
        }
    }
}
```

**AP-05d:** Features `candle`, `ollama`, `onnx` aus `crates/contextra-mcp/Cargo.toml` entfernen (falls vorhanden — aktuell sind dort keine solchen Features definiert, was bereits korrekt ist).

**AP-05e:** Feature-gated Tests (Z.832-910) entfernen, die `candle`/`ollama`/`onnx` Features referenzieren.

**Verifikation:** `cargo check -p contextra-mcp` und `cargo test -p contextra-mcp`

---

### AP-06: `contextra-mcp/src/bin/contextra-mcp-server.rs` — Router-Import entfernen

**Datei:** `crates/contextra-mcp/src/bin/contextra-mcp-server.rs`
**Zeile:** 68
**Schweregrad:** 🔴 Kompilier-Blocker

**Aktueller Code:**
```rust
serde_json::from_slice::<Vec<contextra_router::SlmProfile>>(&bytes)
```

**Aktion:** Der gesamte `--router-profiles-path`-Block (Z.63-75) muss entfernt werden, da `SlmProfile` nicht mehr existiert. Ersetzung:

```rust
    // Router-Profiles-Laden entfernt (contextra-router nicht mehr vorhanden).
    let router_config = RouterConfig::from_env();
```

**Verifikation:** `cargo check -p contextra-mcp`

---

### AP-07: `contextra-db/src/merge_operator_adapter.rs` — Sandbox-Abhängigkeit eliminieren

**Datei:** `crates/contextra-db/src/merge_operator_adapter.rs`
**Zeilen:** 1-201 (komplette Datei)
**Schweregrad:** 🟡 Hinter Feature-Gate `#[cfg(feature = "sandbox")]`

**Aktueller Code:** 201 Zeilen, importiert `contextra_sandbox::{MergeOperatorCapabilities, SandboxError, WasmMergeFunction}`.

**Aktion:** Die gesamte Datei leeren und als Stub hinterlegen. Die Datei wird in `lib.rs` Z.37 nur hinter `#[cfg(feature = "sandbox")]` eingebunden, und das Feature ist in `crates/contextra-db/Cargo.toml` nicht als Default aktiviert.

**Ersetzung — gesamte Datei ersetzen durch:**
```rust
//! Sandbox-Merge-Operator-Adapter (Stub — contextra-sandbox wurde entfernt).
//!
//! Das Feature `sandbox` ist deaktiviert. Dieser Stub verhindert Kompilier-Fehler
//! bei versehentlicher Aktivierung.

/// Platzhalter für den entfernten WasmMergeOperator.
pub struct WasmMergeOperator;
```

**Nacharbeit in `lib.rs`:** Die Feature-gated Referenzen bleiben bestehen, da sie sowieso nur mit Feature `sandbox` aktiviert werden:
- Z.36-37: `#[cfg(feature = "sandbox")] pub mod merge_operator_adapter;`
- Z.55-56: `#[cfg(feature = "sandbox")] use contextra_ports::BoxFuture;`
- Z.71-72: `#[cfg(feature = "sandbox")] pub use merge_operator_adapter::WasmMergeOperator;`
- Z.377-456: `#[cfg(feature = "sandbox")] pub trait SandboxBridge` + impl

Das `SandboxBridge`-Trait und dessen Impl in `lib.rs` verwenden kein `contextra_sandbox` direkt — sie bleiben bestehen.

**Verifikation:** `cargo check -p contextra-db`

---

### AP-08: `contextra-db/src/volatile_vault.rs` + Cargo.toml — Vault-Abhängigkeit entfernen

**Datei:** `crates/contextra-db/src/volatile_vault.rs` (2 Zeilen)
**Datei:** `crates/contextra-db/Cargo.toml` (Z.16)
**Schweregrad:** 🟡 Hinter Feature-Gate `#[cfg(feature = "volatile-vault")]`

**Aktueller Code (volatile_vault.rs):**
```rust
//! Re-export von contextra-vault für Rückwärtskompatibilität (S-22).
pub use contextra_vault::*;
```

**Aktueller Code (Cargo.toml Z.16):**
```toml
contextra-vault = { path = "../contextra-vault" }
```

**Aktion:**
1. `volatile_vault.rs` — Datei leeren, Stub-Kommentar:
   ```rust
   //! Volatile Vault (Stub — contextra-vault wurde entfernt).
   ```
2. `Cargo.toml` Z.16 — Dependency `contextra-vault` entfernen.

**ACHTUNG:** `contextra-vault` existiert noch als Verzeichnis `crates/contextra-vault/` und ist in `Cargo.toml` Workspace unter `exclude` gelistet (Z.29). Die Workspace-Exclude-Zeile kann bleiben oder entfernt werden.

**Verifikation:** `cargo check -p contextra-db`

---

### AP-09: `contextra-db` — `contextra-adapt::PidController`-Referenzen entfernen

**Dateien:**
- `crates/contextra-db/src/collection/query_builder/builder.rs` (Z.52, Z.233)
- `crates/contextra-db/src/collection/query_builder/tests.rs` (Z.455)
**Schweregrad:** 🟡 Hinter Feature-Gate `#[cfg(feature = "adaptive-candidate-pool-sizing")]`

**Aktueller Code (builder.rs Z.52):**
```rust
#[cfg(feature = "adaptive-candidate-pool-sizing")]
pub(super) pid_controller: Option<Arc<parking_lot::Mutex<contextra_adapt::PidController>>>,
```

**Aktion:** `contextra_adapt::PidController` wird im eigenen Crate `contextra-db` über `pid_latency_controller` bereits nachgebaut. Alle Referenzen umbiegen:

1. **builder.rs Z.52:** Ersetze `contextra_adapt::PidController` durch `crate::pid_latency_controller::PidLatencyController` oder entferne das Feature-gated Feld komplett.
2. **builder.rs Z.233:** Analog.
3. **tests.rs Z.455:** Analog.

**Alternative (einfacher):** Da das Feature `adaptive-candidate-pool-sizing` in keinem `Cargo.toml` als Default definiert ist und das Feature auch nicht in `crates/contextra-db/Cargo.toml` aufgeführt ist, die cfg-gated Blöcke einfach komplett entfernen.

**Verifikation:** `cargo check -p contextra-db`

---

### AP-10: `contextra-db/src/contextra_impl/lifecycle.rs` — ONNX-Referenzen entfernen

**Datei:** `crates/contextra-db/src/contextra_impl/lifecycle.rs`
**Zeilen:** 196-206
**Schweregrad:** 🟡 Hinter Feature-Gate `#[cfg(feature = "onnx")]`

**Aktueller Code:**
```rust
#[cfg(feature = "onnx")]
{
    let model_dir = contextra_infer_onnx::ensure_onnx_model_download(
        model_name,
        cache_dir.as_deref(),
    )
    .await?;
    let embedder = contextra_infer_onnx::OnnxEmbedder::load(model_dir)?;
    ...
}
```

**Aktion:** Den `#[cfg(feature = "onnx")]`-Block entfernen. Der `#[cfg(not(feature = "onnx"))]`-Block (Z.207+) bleibt als einziger Pfad.

**Verifikation:** `cargo check -p contextra-db`

---

### AP-11: `contextra-db/src/cognition/` — Kommentar-Referenzen bereinigen

**Dateien:**
- `crates/contextra-db/src/cognition/mod.rs` (Z.1)
- `crates/contextra-db/src/cognition/bridge.rs` (Z.1)
**Schweregrad:** 🟢 Nur Kommentare

**Aktion:**
- `mod.rs` Z.1: Ersetze `//! Brückenmodul zwischen contextra-cognition und contextra-engine` durch `//! Brückenmodul für Cognition-Pipelines`.
- `bridge.rs` Z.1: Ersetze `//! Bridge-Implementierung für Cognition auf contextra-db / contextra-engine.` durch `//! Bridge-Implementierung für Cognition-Pipelines.`

---

### AP-12: `contextra-db/src/decay_controller.rs` — Kommentar-Referenzen bereinigen

**Datei:** `crates/contextra-db/src/decay_controller.rs`
**Zeilen:** 9-10, 15
**Schweregrad:** 🟢 Nur Kommentare

**Aktion:** Kommentare Z.9-10 und Z.15 ersetzen:
- Z.9: `// ZWECK: Adaptive Decay Controller (lokal implementiert).`
- Z.10: Zeile entfernen.
- Z.15: `//! Adaptive Decay Controller (lokal implementiert, ehemals contextra-adapt).`

---

### AP-13: Test-Dateien mit entfernten Crate-Imports entfernen

**Schweregrad:** 🔴 Kompilier-Blocker (bei Testausführung)

**AP-13a:** `crates/contextra-engine/tests/proof_regulators_vs_adapt.rs`
- **Komplette Datei entfernen.** Importiert `contextra_adapt::decay_controller::*` direkt.
- Differenztest gegen entferntes Crate ist obsolet.

**AP-13b:** `crates/contextra-engine/tests/proof_chunker_vs_ingest.rs`
- **Z.74 entfernen:** `let est_ingest = contextra_ingest::chunking::common::estimate_tokens(sample);`
- **Z.75 entfernen:** `assert_eq!(est_eng, est_ingest, ...);`
- Oder komplette Datei entfernen, da der Vergleich mit `contextra-ingest` obsolet ist.

**AP-13c:** `crates/contextra-db/tests/pid_pool_production_caller.rs`
- **Z.8 ersetzen:** `use contextra_adapt::PidController;` — Entweder komplett entfernen oder durch lokalen PID-Controller ersetzen. Der Test nutzt `PidController::new()` und `pid.update()` — Prüfen ob `contextra_db::pid_latency_controller::PidLatencyController` kompatibel ist. Falls nicht: **Testdatei entfernen.**

**AP-13d:** `crates/contextra-mcp/tests/mcp_test.rs` Z.983
- **Ersetze:** `contextra_router::SlmProfile::new(...)` — Der gesamte Test `test_mcp_boot_with_routing_config` muss entfernt oder angepasst werden, da `SlmProfile` nicht mehr existiert.

**AP-13e:** `crates/contextra-engine/tests/chunker_parity_ingest.rs`
- **Z.4 und Z.33 — Kommentare bereinigen:** `contextra-ingest` Referenzen entfernen.

**AP-13f:** `crates/contextra-engine/tests/s64_public_surface.rs`
- **Z.4 und Z.34 — Kommentare bereinigen:** `contextra-cognition` Referenzen entfernen.

---

### AP-14: `benchmarks/contextra-bench/src/main.rs` — Inferenz-Backend-Referenzen entfernen

**Datei:** `benchmarks/contextra-bench/src/main.rs`
**Zeilen:** 13 (onnx), 1029-1031 (candle)
**Schweregrad:** 🔴 Kompilier-Blocker

**AP-14a:** Z.12-13 — ONNX Import entfernen:
```rust
#[cfg(feature = "onnx-bench")]
use contextra_infer_onnx::{CrossEncoderReranker, RerankConfig};
```
Komplett entfernen. Feature `onnx-bench` existiert nicht in Bench-Cargo.toml.

**AP-14b:** Z.1029-1031 — Candle-Embedding-Instantiierung:
```rust
let embedder = std::sync::Arc::new(contextra_infer_candle::CandleEmbedClient::from_dir(
    embed_dir,
    contextra_infer_candle::model_registry::CandleQuantization::Q4KM,
)?);
```
Durch `compile_error!("Candle backend entfernt")` oder Auskommentieren ersetzen. Prüfen, ob der umgebende Code hinter einem Feature-Gate liegt — falls ja, Gate beibehalten.

**Verifikation:** `cargo check -p contextra-bench`

---

### AP-15: `tests/unsafe_islands.rs` — Wire-Referenz entfernen

**Datei:** `tests/unsafe_islands.rs`
**Zeilen:** 4, 15
**Schweregrad:** 🟡 Test-Failure

**Aktueller Code (Z.4):**
```rust
// INVARIANTEN: Exactly 3 Unsafe Islands (contextra-simd, contextra-sys, contextra-wire).
```

**Aktueller Code (Z.12-16):**
```rust
pub const ALLOWED_UNSAFE_ISLANDS: &[&str] = &[
    "contextra-simd",
    "contextra-sys",
    "contextra-wire",
];
```

**Aktion:**
- Z.4: Kommentar aktualisieren: `// INVARIANTEN: Exactly 2 Unsafe Islands (contextra-simd, contextra-sys).`
- Z.14-16: `"contextra-wire"` aus der Liste entfernen.

**Verifikation:** `cargo test --test unsafe_islands`

---

### AP-16: `deny.toml` — Referenzen entfernter Crates löschen

**Datei:** `deny.toml`
**Schweregrad:** 🟢 Config-Bereinigung

**Aktionen (in der `[bans].deny`-Liste):**
- Z.48: `"contextra-agent",` — entfernen
- Z.51: `"contextra-cognition",` — entfernen
- Z.56-58: `"contextra-infer-candle"`, `"contextra-infer-ollama"`, `"contextra-infer-onnx"` — entfernen
- Z.64: `"contextra-py",` — entfernen
- Z.66-67: `"contextra-router"`, `"contextra-sandbox"` — entfernen

**Wrapper-Referenzen entfernen (Z.77-80):**
- Z.77: `{ crate = "candle-core", wrappers = [..., "contextra-infer-candle"] }` — `contextra-infer-candle` aus Wrapper-Liste entfernen oder ganze Zeile entfernen
- Z.78: `{ crate = "pyo3", wrappers = ["contextra-py", ...] }` — `contextra-py` entfernen oder ganze Zeile entfernen
- Z.79: `{ crate = "reqwest", wrappers = ["contextra-infer-ollama"] }` — ganze Zeile entfernen
- Z.80: `{ crate = "wasmtime", wrappers = ["contextra-sandbox"] }` — ganze Zeile entfernen

Auch `"contextra-kvcache"` Z.62 (falls vorhanden) entfernen.

**Verifikation:** `cargo deny check`

---

### AP-17: `capabilities.toml` — Sektionen entfernter Crates löschen

**Datei:** `capabilities.toml`
**Schweregrad:** 🟢 Config-Bereinigung

**Komplette Sektionen entfernen:**
| Sektion | Zeilen (ca.) |
|---------|-------------|
| `[crates.contextra-wire]` | 65-74 |
| `[crates.contextra-router]` | 214-224 |
| `[crates.contextra-adapt]` | 226-236 |
| `[crates.contextra-infer-ollama]` | 238-248 |
| `[crates.contextra-infer-candle]` | 250-261 |
| `[crates.contextra-infer-onnx]` | 263-273 |
| `[crates.contextra-ingest]` | 275-285 |
| `[crates.contextra-sandbox]` | 299-308 |
| `[crates.contextra-prompt-guard]` | 310-319 |
| `[crates.contextra-vault]` | 321-331 |
| `[crates.contextra-agent]` | 333-343 |
| `[crates.contextra-cognition]` | 345-355 |
| `[crates.contextra-py]` | 414-423 |

**Referenzen in bestehenden Sektionen bereinigen:**
- `[crates.contextra-engine]` Z.375 `allowed_internal`: `"contextra-adapt"` und `"contextra-kvcache"` entfernen
- `[crates.contextra-engine]` Feature-Closures: `infer-candle` Referenz entfernen
- `[crates.contextra]` Z.399 `infer-candle` Feature-Closure entfernen
- `[crates.contextra-mcp]` Z.409 `allowed_internal`: `"contextra-wire"`, `"contextra-router"`, `"contextra-prompt-guard"` entfernen

---

### AP-18: Fragment-TOMLs bereinigen

**AP-18a:** `crates/contextra-engine/capabilities.fragment.toml`

Entfernen (Z.21-26):
```toml
adaptive-candidate-pool-sizing = ["contextra-adapt"]
onnx = ["contextra-infer-onnx"]
sandbox = ["contextra-sandbox"]
...
ingest = ["contextra-ingest"]
```

**AP-18b:** `crates/contextra-mcp/capabilities.fragment.toml`

Entfernen (Z.11, 15-16, 18):
```toml
"contextra-wire",
"contextra-router",
"contextra-prompt-guard",
feature_closure_allowed = { candle = ["contextra-infer-candle"], ollama = ["contextra-infer-ollama"], onnx = ["contextra-infer-onnx"] }
```

---

### AP-19: `.cargo/audit.toml` — PyO3 Isolation-Proofs entfernen

**Datei:** `.cargo/audit.toml`
**Zeilen:** 6-18
**Schweregrad:** 🟢 Config-Bereinigung

**Aktion:** Die Kommentare referenzieren `contextra-py` als Isolation-Proof. Da `contextra-py` entfernt wurde, müssen die Advisories-Einträge geprüft werden:
- Falls `pyo3` nicht mehr als transitive Dependency existiert, die RUSTSEC-Einträge komplett entfernen.
- Falls `pyo3` noch transitiv vorhanden ist, die Kommentare aktualisieren.

---

### AP-20: `Cargo.toml` (Workspace Root) — Frozen-Zone bereinigen

**Datei:** `Cargo.toml`
**Zeilen:** 137-153
**Schweregrad:** 🟢 Config-Bereinigung

**Aktion:** Die Kommentare in der Frozen Zone (Z.137-153) referenzieren entfernte Crates. Bereinigen:
- Z.138: `# contextra-cluster, contextra-sandbox, contextra-saos-agent` — `contextra-sandbox` als entfernt markieren
- Z.146: `# Embedding & ML (activate with contextra-infer-onnx feature)` — entfernen

Auch Z.29 prüfen: `"crates/contextra-vault"` im `exclude`-Block. Falls Vault-Verzeichnis gelöscht wird, Zeile entfernen.

---

### AP-21: Dokumentations-Kommentare in ~20 Dateien bereinigen

**Schweregrad:** 🟢 Nur Kommentare/Docs

Alle betroffenen Dateien und die Aktion:

| Datei | Referenzierte Crates | Aktion |
|-------|---------------------|--------|
| `contextra-rank/src/lib.rs` Z.27-28 | infer-candle, infer-ollama, infer-onnx, py | Entfernte Crates aus der Liste streichen |
| `contextra-types/src/error.rs` Z.122 | py | `contextra-py` aus Kommentar entfernen |
| `contextra-types/src/error_dto.rs` Z.6 | py | `contextra-py` aus Kommentar entfernen |
| `contextra-types/src/retrieval_strategy.rs` Z.9 | router | `contextra_router` aus Kommentar entfernen |
| `contextra-types/src/types/domain/misc.rs` Z.92 | py | `contextra-py` aus Kommentar entfernen |
| `contextra-testkit/src/lib.rs` Z.27-28 | agent, cognition | Entfernte Crates aus der Liste streichen |
| `contextra-ports/src/embedding.rs` Z.7 | infer-candle, infer-ollama | Kommentar aktualisieren |
| `contextra-graph/src/hyperedge.rs` Z.310-313 | cognition | Referenz auf `contextra-cognition` entfernen |
| `contextra-graph/src/percolation.rs` Z.19-22 | cognition | Referenz auf `contextra-cognition` entfernen |
| `contextra-crypto/src/kv_cipher.rs` Z.24 | kvcache, infer-candle | Referenz entfernen |
| `contextra-crypto/src/lib.rs` Z.22 | infer-candle, kvcache | Referenz entfernen |
| `contextra-text/src/lib.rs` Z.25 | router, agent | Entfernte Crates streichen |
| `contextra-store/src/lib.rs` Z.26 | agent | `contextra-agent` entfernen |
| `contextra-store/src/compaction/merge_operator.rs` Z.11 | sandbox | Kommentar entfernen |
| `contextra-checkpoint/src/lib.rs` Z.34 | agent | `contextra-agent` entfernen |
| `contextra-privacy/src/egress_vault.rs` Z.555 | router | `contextra-router` entfernen |
| `contextra-sys/src/vault.rs` Z.20 | vault | Referenz aktualisieren |
| `contextra-sys/src/lib.rs` Z.29, 93 | vault | Referenz aktualisieren |

---

## Ausführungsreihenfolge

```
Phase 1 — Kompilier-Blocker (Prio 🔴):
  AP-01 → AP-02 → AP-03 → AP-04 → AP-05 → AP-06 → AP-07 → AP-08 → AP-09 → AP-10

Phase 2 — Tests (Prio 🔴/🟡):
  AP-13 → AP-14 → AP-15

Phase 3 — Config (Prio 🟢):
  AP-16 → AP-17 → AP-18 → AP-19 → AP-20

Phase 4 — Docs (Prio 🟢):
  AP-11 → AP-12 → AP-21
```

**Validierungskette nach Abschluss:**
```bash
cargo check --workspace
cargo test --workspace --lib
cargo test --workspace --tests
cargo deny check  # falls cargo-deny installiert
```
