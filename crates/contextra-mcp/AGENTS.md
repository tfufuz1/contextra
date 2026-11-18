# AGENTS.md — contextra-mcp — Model Context Protocol Server

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-mcp.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-mcp.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §1 (Produktdefinition), §2 (Architektur), §3 (Datenklassifikation), §6 (ReadGate)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E3 (Endpunkte / Anwendungen)  
**Kern-Closure:** Darf alle Ebenen E0-E2 nutzen  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

MCP-Server über stdio (JSON-RPC 2.0) für LLM-Integration, Retrieval-Tools und DSGVO-Löschung.

---

## 2. Kern-Richtlinien für Agenten

- **Schnittstelle:** Standard-I/O (`stdio`) via JSON-RPC 2.0. Keine unautorisierten Netzwerk-Ports!
- **MCP-Tools:**
  - `search_hybrid`: Hybride Suche mit Durchsetzung des `ReadGate`.
  - `get_document`: Dokumentabruf (Fail-Closed).
  - `erase_subject`: Startet atomare Betroffenen-Löschung und liefert `EraseReceipt`.
  - `verify_erasure`: Fragt Löschstatus ab.
- **Sicherheitsfilter:**
  - `PromptInjectionGuard`: Erkennung von Prompt-Injection-Angriffen in Eingaben.
  - `Egress PII Filter`: Verhindert die versehentliche Ausgabe unmaskierter Personendaten in Tool-Antworten.
- **Keine PII in Logs:** stderr-Logs enthalten nur `ErasureId`, Zeitstempel und Tool-Namen.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-mcp` · **Ring:** 4 · **Version:** 0.1.0
- **Beschreibung:** –
- **src:** 18 Dateien, 3794 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 6):** `contextra`, `contextra-crypto`, `contextra-ports`, `contextra-privacy`, `contextra-rank`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 13):** `futures-util`, `parking_lot`, `rand`, `scc`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `tracing-subscriber`, `unicode-normalization`, `zeroize`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra`, `contextra-ports`, `contextra-types`; eingehend –
- **Features (2):** `default`, `test-utils`

### Öffentliche API (grob, aus src/)
- struct (24): `CollectionSearchEngine`, `EmbeddingConfig`, `ExplainRequest`, `ExplainResponse`, `JsonRpcError`, `JsonRpcRequest`, `JsonRpcResponse`, `LlmConfig`, `McpSandbox`, `McpServer`, `MockEmbedder`, `MockLlmGenerator`, `NormalizedPattern`, `PromptGuardDetector`, `PromptInjectionConfig` … (+9)
- enum (4): `GuardError`, `McpError`, `QuarantinePolicy`, `ToolCategory`
- trait (1): `Clock`
- fn (48): `available_compiled_providers`, `build_generator`, `build_provider`, `classify_method`, `code`, `create_embedding_provider`, `create_llm_text_generator`, `decrypt`, `detect`, `encode`, `encrypt`, `err`, `execute_with_timeout`, `from_env`, `get_volatile` … (+33)
- const (9): `DEFAULT_REDACTION_PLACEHOLDER`, `MAX_BATCH_SIZE`, `MAX_JSON_DEPTH`, `MAX_RPC_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `MAX_VOLATILE_KEY_BYTES`, `MAX_VOLATILE_OUTPUT_BYTES`, `MAX_VOLATILE_RESULTS`, `TOOL_REGISTRY`
- mod (20): `audit`, `bulk_exfiltration_detector`, `clock`, `config`, `egress_gateway`, `egress_guard`, `error`, `explain`, `guard`, `io`, `policy`, `prompt_injection`, `protocol`, `routing`, `sandbox` … (+5)
- pub use: `audit::*`, `bulk_exfiltration_detector::{ BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId, }`, `clock::*`, `config::*`, `contextra_privacy::bulk_exfiltration_detector::*`, `contextra_privacy::egress_gateway::*`, `contextra_privacy::egress_guard::*`, `contextra_privacy::egress_vault::{EgressClassification, EgressClassifier}`, `egress_gateway::{ CloudQueryRequest, CloudQueryResponse, DefaultEgressClassifier, EgressClassification, EgressClassifier, }`, `error::*` … (+13)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
