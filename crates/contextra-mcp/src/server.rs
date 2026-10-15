use crate::config::LlmConfig;
use crate::io::{read_line_bounded, MAX_RPC_BYTES};
use crate::prompt_injection::PromptInjectionGuard;
use crate::proof_key::deletion_proof_key_from_env;
use crate::protocol::JsonRpcResponse;
use crate::sandbox::{McpSandbox, SandboxPolicy};
use crate::server_dispatch::cancellation::ConcurrencyGovernor;
use crate::server_dispatch::forget_guard::ForgetGuard;
use crate::server_dispatch::recall_truncator::RecallTruncator;
use crate::validation::{is_write_allowed_by_env, resolve_session_tenant_from_env};
use contextra::Contextra;
use contextra_ports::EmbeddingProvider;
use contextra_ports::SystemClock;
use contextra_types::{ContextraError, TenantId};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};

pub struct McpServer {
    pub db: Arc<Contextra>,
    pub embedder: Arc<dyn EmbeddingProvider>,
    pub sandbox: Arc<McpSandbox>,
    pub tenant_id: TenantId,
    pub injection_guard: Arc<PromptInjectionGuard>,
    pub plugin_registry: Option<Arc<contextra_ports::plugin::PluginRegistry>>,
    pub forget_guard: Arc<ForgetGuard>,
    pub concurrency_governor: Arc<ConcurrencyGovernor>,
    pub recall_truncator: Arc<RecallTruncator>,
}

impl McpServer {
    pub fn new(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
    ) -> Result<Self, ContextraError> {
        let tenant = resolve_session_tenant_from_env()?;
        Self::with_write_permission_and_tenant(db, embedder, is_write_allowed_by_env(), tenant)
    }

    /// Instantiates an `McpServer` fully configured from environment variables and default runtime components.
    pub fn from_env(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
    ) -> Result<Self, ContextraError> {
        let tenant = resolve_session_tenant_from_env()?;
        let allow_write = is_write_allowed_by_env();
        let server = Self::with_write_permission_and_tenant(db, embedder, allow_write, tenant)?;

        // Verify LLM generator build configuration from environment
        let _llm_generator = LlmConfig::from_env().build_generator()?;

        // Verify deletion proof key configuration from environment if available
        let _deletion_key = deletion_proof_key_from_env().ok();

        let injection_guard = Arc::new(PromptInjectionGuard::from_env());
        let plugin_registry = Some(Arc::new(contextra_ports::plugin::PluginRegistry::new(
            server.db.license_gate(),
        )));

        Ok(server
            .with_injection_guard(injection_guard)
            .with_plugin_registry(plugin_registry))
    }

    pub fn with_write_permission(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
        allow_db_writes: bool,
    ) -> Result<Self, ContextraError> {
        let tenant = resolve_session_tenant_from_env()?;
        Self::with_write_permission_and_tenant(db, embedder, allow_db_writes, tenant)
    }

    pub fn with_write_permission_and_tenant(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
        allow_db_writes: bool,
        tenant_id: TenantId,
    ) -> Result<Self, ContextraError> {
        let policy = SandboxPolicy {
            allow_db_reads: true,
            allow_db_writes,
            allow_code_execution: false,
            allow_cloud_egress: false,
            max_execution_ms: 5_000,
            sidecar_limits: crate::sandbox::SidecarLimits::default(),
        };
        let sandbox = McpSandbox::new(policy)
            .map_err(|e| ContextraError::Internal(format!("Sandbox init: {e}")))?;
        Self::with_sandbox_and_tenant(db, embedder, Arc::new(sandbox), tenant_id)
    }

    pub fn with_sandbox(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
        sandbox: Arc<McpSandbox>,
    ) -> Self {
        let tenant = resolve_session_tenant_from_env()
            .unwrap_or_else(|_| TenantId::try_new(1).expect("tenant 1"));
        Self::with_sandbox_and_tenant(db, embedder, sandbox, tenant).expect("valid tenant")
    }

    pub fn with_sandbox_and_tenant(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
        sandbox: Arc<McpSandbox>,
        tenant_id: TenantId,
    ) -> Result<Self, ContextraError> {
        if tenant_id.inner() == 0 {
            return Err(ContextraError::InvalidInput(
                "TenantId 0 (SYSTEM) is prohibited for MCP server session (INV-TENANT-1)."
                    .to_string(),
            ));
        }

        let clock = Arc::new(SystemClock::new());
        let plugin_registry = Some(Arc::new(contextra_ports::plugin::PluginRegistry::new(
            db.license_gate(),
        )));
        Ok(Self {
            db,
            embedder,
            sandbox,
            tenant_id,
            injection_guard: Arc::new(PromptInjectionGuard::from_env()),
            plugin_registry,
            forget_guard: Arc::new(ForgetGuard::new(10, clock)),
            concurrency_governor: Arc::new(ConcurrencyGovernor::new(8)),
            recall_truncator: Arc::new(RecallTruncator::new(4000)),
        })
    }

    pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self {
        self.injection_guard = injection_guard;
        self
    }

    pub fn with_plugin_registry(
        mut self,
        registry: Option<Arc<contextra_ports::plugin::PluginRegistry>>,
    ) -> Self {
        self.plugin_registry = registry;
        self
    }

    /// Retrieves and decrypts a volatile tool result stored in the MCP sandbox.
    pub fn get_volatile_output(
        &self,
        key: &str,
    ) -> Result<Option<zeroize::Zeroizing<Vec<u8>>>, ContextraError> {
        self.sandbox.get_volatile(key)
    }

    /// Startet den MCP stdio-Loop.
    ///
    /// Liest zeilenweise von stdin, dispatcht JSON-RPC-Requests und schreibt
    /// Antworten als einzelne JSON-Zeile nach stdout.
    /// Der Loop endet, wenn stdin geschlossen wird (EOF).
    pub async fn run_stdio(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line_buf = String::new();
        let timeout_secs = std::env::var("CONTEXTRA_MCP_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(30);
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        loop {
            let read_res = tokio::time::timeout(
                timeout_duration,
                read_line_bounded(&mut reader, &mut line_buf, MAX_RPC_BYTES),
            )
            .await;

            let read_line_res = match read_res {
                Ok(res) => res,
                Err(_) => {
                    tracing::warn!("stdio read timeout after {timeout_duration:?} inactivity");
                    return Err("stdio idle timeout".into());
                }
            };

            match read_line_res {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line_buf.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    let response_opt = match serde_json::from_str::<Value>(trimmed) {
                        Ok(val) => self.handle_value(val).await,
                        Err(e) => Some(
                            serde_json::to_value(JsonRpcResponse::err(
                                None,
                                -32700,
                                format!("Parse error: {e}"),
                            ))
                            .unwrap_or_default(),
                        ),
                    };

                    if let Some(resp) = response_opt {
                        // MCP-Protokoll: eine JSON-Antwort pro Zeile, abgeschlossen mit '\n'.
                        let mut out = serde_json::to_string(&resp)?;
                        out.push('\n');
                        stdout.write_all(out.as_bytes()).await?;
                        stdout.flush().await?;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                    let response = JsonRpcResponse::err(None, -32700, format!("Parse error: {e}"));
                    let mut out = serde_json::to_string(&response)?;
                    out.push('\n');
                    stdout.write_all(out.as_bytes()).await?;
                    stdout.flush().await?;
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }
}
