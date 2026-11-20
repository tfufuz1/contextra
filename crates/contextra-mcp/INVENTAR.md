## contextra-mcp

Pfad: `crates/contextra-mcp`

Abhängigkeiten (direkt): contextra, contextra-types, contextra-ports, contextra-rank, contextra-crypto, contextra-privacy, zeroize, rand, parking_lot, scc, tokio, serde, serde_json, thiserror, tracing, tracing-subscriber, unicode-normalization, tokio-util, futures-util

### Module

- `/src/bin/contextra-mcp-server.rs`
- `/src/bulk_exfiltration_detector.rs`
- `/src/config.rs`
- `/src/egress_gateway.rs`
- `/src/egress_guard.rs`
- `/src/explain.rs`
- `/src/io.rs`
- `/src/lib.rs`
- `/src/prompt_injection.rs`
- `/src/protocol.rs`
- `/src/routing.rs`
- `/src/sandbox.rs`
- `/src/server.rs`
- `/src/server_dispatch.rs`
- `/src/server_tools.rs`
- `/src/tests.rs`
- `/src/tools_crud.rs`
- `/src/validation.rs`

### Öffentliche Typen

- **Clock** (trait) – src/prompt_injection.rs:17
- **CollectionSearchEngine** (struct) – src/egress_guard.rs:15
- **EmbeddingConfig** (struct) – src/config.rs:169
- **ExplainRequest** (struct) – src/explain.rs:14
- **ExplainResponse** (struct) – src/explain.rs:28
- **GuardError** (enum) – src/prompt_injection.rs:11
- **JsonRpcError** (struct) – src/protocol.rs:89
- **JsonRpcRequest** (struct) – src/protocol.rs:98
- **JsonRpcResponse** (struct) – src/protocol.rs:109
- **LlmConfig** (struct) – src/config.rs:300
- **McpError** (enum) – src/protocol.rs:20
- **McpSandbox** (struct) – src/sandbox.rs:268
- **McpServer** (struct) – src/server.rs:19
- **MockEmbedder** (struct) – src/config.rs:439
- **MockLlmGenerator** (struct) – src/config.rs:499
- **NormalizedPattern** (struct) – src/prompt_injection.rs:98
- **PromptGuardDetector** (struct) – src/egress_gateway.rs:33
- **PromptInjectionConfig** (struct) – src/prompt_injection.rs:35
- **PromptInjectionGuard** (struct) – src/prompt_injection.rs:50
- **QuarantinePolicy** (enum) – src/prompt_injection.rs:41
- **RouterConfig** (struct) – src/config.rs:413
- **RoutingHandle** (struct) – src/routing.rs:11
- **SandboxPolicy** (struct) – src/sandbox.rs:203
- **SecurityAuditLogger** (struct) – src/prompt_injection.rs:77
- **SecurityAuditRecord** (struct) – src/prompt_injection.rs:81
- **SystemClock** (struct) – src/prompt_injection.rs:25
- **ToolCategory** (enum) – src/sandbox.rs:190
- **ToolDefinition** (struct) – src/sandbox.rs:30
- **VolatileToolResult** (struct) – src/sandbox.rs:229

### Öffentliche Funktionen

- `pub async fn execute_with_timeout<F, T, E>(`  – src/sandbox.rs:376
- `pub async fn handle(&self, req: JsonRpcRequest) -> JsonRpcResponse `  – src/server_dispatch.rs:125
- `pub async fn handle_value(&self, val: Value) -> Option<Value> `  – src/server_dispatch.rs:13
- `pub async fn read_line_bounded<R: tokio::io::AsyncBufRead + Unpin>(`  – src/io.rs:19
- `pub async fn run_stdio(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> `  – src/server.rs:106
- `pub async fn setup_routing(`  – src/routing.rs:15
- `pub fn available_compiled_providers() -> Vec<&'static str> `  – src/config.rs:21
- `pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError> `  – src/config.rs:367
- `pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError> `  – src/config.rs:253
- `pub fn classify_method(method: &str) -> ToolCategory `  – src/sandbox.rs:371
- `pub fn code(&self) -> i32 `  – src/protocol.rs:54
- `pub fn create_embedding_provider(`  – src/config.rs:267
- `pub fn create_llm_text_generator(`  – src/config.rs:378
- `pub fn decrypt(`  – src/sandbox.rs:249
- `pub fn detect(&self, _text: &str) -> Option<String> `  – src/prompt_injection.rs:61
- `pub fn encode(bytes: [u8; 32]) -> String `  – src/sandbox.rs:456
- `pub fn encrypt(plaintext: &[u8], key: &contextra_crypto::CryptoKey) -> Result<Self> `  – src/sandbox.rs:238
- `pub fn err(id: Option<serde_json::Value>, code: i32, message: impl Into<String>) -> Self `  – src/protocol.rs:129
- `pub fn from_env() -> Self `  – src/config.rs:199
- `pub fn from_env() -> Self `  – src/config.rs:324
- `pub fn from_env() -> Self `  – src/config.rs:420
- `pub fn from_env() -> Self `  – src/prompt_injection.rs:57
- `pub fn get_volatile(&self, key: &str) -> Result<Option<zeroize::Zeroizing<Vec<u8>>>> `  – src/sandbox.rs:422
- `pub fn internal_error(msg: impl Into<String>) -> Self `  – src/protocol.rs:50
- `pub fn invalid_params(msg: impl Into<String>) -> Self `  – src/protocol.rs:46
- `pub fn invalid_request(msg: impl Into<String>) -> Self `  – src/protocol.rs:38
- `pub fn is_write_allowed_by_env() -> bool `  – src/validation.rs:8
- `pub fn measure_json_depth(val: &Value) -> usize `  – src/protocol.rs:144
- `pub fn method_not_found(msg: impl Into<String>) -> Self `  – src/protocol.rs:42
- `pub fn new() -> Self `  – src/prompt_injection.rs:53
- `pub fn new(`  – src/server.rs:31
- `pub fn new(dimension: usize) -> Self `  – src/config.rs:446
- `pub fn new(policy: SandboxPolicy) -> Result<Self> `  – src/sandbox.rs:278
- `pub fn ok(id: Option<serde_json::Value>, result: serde_json::Value) -> Self `  – src/protocol.rs:120
- `pub fn parse_error(msg: impl Into<String>) -> Self `  – src/protocol.rs:34
- `pub fn policy(&self) -> &SandboxPolicy `  – src/sandbox.rs:295
- `pub fn process_result(`  – src/prompt_injection.rs:65
- `pub fn provider_opt(&self) -> Option<&str> `  – src/config.rs:243
- `pub fn provider_opt(&self) -> Option<&str> `  – src/config.rs:357
- `pub fn response_from_error(id: Option<Value>, err: McpError) -> JsonRpcResponse `  – src/protocol.rs:179
- `pub fn store_volatile(&self, key: &str, output: &[u8]) -> Result<()> `  – src/sandbox.rs:395
- `pub fn try_classify_method(method: &str) -> Result<ToolCategory> `  – src/sandbox.rs:353
- `pub fn validate_collection_name(name: &str) -> Result<(), McpError> `  – src/validation.rs:20
- `pub fn validate_json_depth(val: &Value, max_depth: usize) -> Result<(), McpError> `  – src/protocol.rs:168
- `pub fn validate_tool_call(&self, method: &str, _params: &Value) -> Result<()> `  – src/sandbox.rs:309
- `pub fn verify_embedding_provider(`  – src/config.rs:73
- `pub fn with_egress_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self `  – src/server.rs:83
- `pub fn with_expected_dim(mut self, dim: usize) -> Self `  – src/config.rs:238
- `pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self `  – src/server.rs:78
- `pub fn with_plugin_registry(`  – src/server.rs:93
- `pub fn with_routing(mut self, routing: Option<Arc<RoutingHandle>>) -> Self `  – src/server.rs:88
- `pub fn with_sandbox(`  – src/server.rs:55
- `pub fn with_tenant_id(mut self, tenant_id: Option<TenantId>) -> Self `  – src/server.rs:73
- `pub fn with_write_permission(`  – src/server.rs:38
- `pub fn with_zero_vectors(dimension: usize) -> Self `  – src/config.rs:453
- `pub(crate) async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, McpError> `  – src/server_tools.rs:13
- `pub(crate) async fn handle_create_collection(&self, args: &Value) -> Result<Value, McpError> `  – src/tools_crud.rs:12
- `pub(crate) async fn handle_delete(&self, args: &Value) -> Result<Value, McpError> `  – src/tools_crud.rs:142
- `pub(crate) async fn handle_drop_collection(&self, args: &Value) -> Result<Value, McpError> `  – src/tools_crud.rs:95
- `pub(crate) async fn handle_explain(&self, args: &Value) -> Result<Value, McpError> `  – src/explain.rs:44
- `pub(crate) fn resolve_provider_setting(`  – src/config.rs:144

---
