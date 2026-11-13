# AGENTS.md — `contextra-mcp`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-mcp` · **Ring:** 4 · **Version:** 0.1.0
- **Beschreibung:** –
- **src:** 18 Dateien, 3909 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 12):** `contextra`, `contextra-crypto`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-ports`, `contextra-privacy`, `contextra-prompt-guard`, `contextra-rank`, `contextra-router`, `contextra-types`, `contextra-wire`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 13):** `futures-util`, `parking_lot`, `rand`, `scc`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `tracing-subscriber`, `unicode-normalization`, `zeroize`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra`, `contextra-ports`, `contextra-types`; eingehend –
- **Features (5):** `candle`, `default`, `ollama`, `onnx`, `test-utils`

### Öffentliche API (grob, aus src/)
- struct (15): `CollectionSearchEngine`, `EmbeddingConfig`, `ExplainRequest`, `ExplainResponse`, `LlmConfig`, `McpSandbox`, `McpServer`, `MockEmbedder`, `MockLlmGenerator`, `PromptGuardDetector`, `RouterConfig`, `RoutingHandle`, `SandboxPolicy`, `ToolDefinition`, `VolatileToolResult`
- enum (2): `McpError`, `ToolCategory`
- fn (44): `available_compiled_providers`, `build_generator`, `build_provider`, `classify_method`, `code`, `create_embedding_provider`, `create_llm_text_generator`, `decrypt`, `encode`, `encrypt`, `execute_with_timeout`, `from_env`, `get_volatile`, `handle`, `handle_value` … (+29)
- const (8): `MAX_BATCH_SIZE`, `MAX_JSON_DEPTH`, `MAX_RPC_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `MAX_VOLATILE_KEY_BYTES`, `MAX_VOLATILE_OUTPUT_BYTES`, `MAX_VOLATILE_RESULTS`, `TOOL_REGISTRY`
- mod (18): `audit`, `bulk_exfiltration_detector`, `config`, `egress_gateway`, `egress_guard`, `explain`, `guard`, `io`, `policy`, `prompt_injection`, `protocol`, `routing`, `sandbox`, `server`, `server_dispatch` … (+3)
- pub use: `bulk_exfiltration_detector::{ BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId, }`, `config::*`, `contextra_privacy::bulk_exfiltration_detector::*`, `contextra_privacy::egress_gateway::*`, `contextra_privacy::egress_guard::*`, `contextra_privacy::egress_vault::{EgressClassification, EgressClassifier}`, `contextra_prompt_guard::audit::{SecurityAuditLogger, SecurityAuditRecord}`, `contextra_prompt_guard::guard::{NormalizedPattern, PromptInjectionGuard}`, `contextra_prompt_guard::policy::{ PromptInjectionConfig, QuarantinePolicy, DEFAULT_REDACTION_PLACEHOLDER, }`, `contextra_prompt_guard::{ audit::{SecurityAuditLogger, SecurityAuditRecord}, clock::{Clock, SystemClock}, error::GuardError, guard::{NormalizedPattern, PromptInjectionGuard}, policy::{PromptInjectionConfig, QuarantinePolicy, DEFAULT_REDACTION_PLACEHOLDER}, }` … (+8)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
