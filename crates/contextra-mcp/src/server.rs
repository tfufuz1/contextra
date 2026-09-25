use crate::egress_gateway::{DefaultEgressClassifier, EgressClassifier};
use crate::io::{read_line_bounded, MAX_RPC_BYTES};
use crate::prompt_injection::PromptInjectionGuard;
use crate::protocol::JsonRpcResponse;
#[cfg(feature = "kv-bridge")]
use crate::routing::setup_kv_bridge;
use crate::routing::RoutingHandle;
use crate::sandbox::{McpSandbox, SandboxPolicy};
use crate::validation::is_write_allowed_by_env;
use contextra::Contextra;
use contextra_ports::EmbeddingProvider;
use contextra_types::ContextraError;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};

pub struct McpServer {
    pub db: Arc<Contextra>,
    pub embedder: Arc<dyn EmbeddingProvider>,
    pub sandbox: Arc<McpSandbox>,
    pub injection_guard: Arc<PromptInjectionGuard>,
    pub egress_classifier: Arc<dyn EgressClassifier>,
    pub routing: Option<Arc<RoutingHandle>>,
    #[cfg(feature = "kv-bridge")]
    pub kv_bridge: Option<Arc<contextra_infer_candle::KvBridgeAdapter>>,
}

impl McpServer {
    pub fn new(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
    ) -> Result<Self, ContextraError> {
        Self::with_write_permission(db, embedder, is_write_allowed_by_env())
    }

    pub fn with_write_permission(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
        allow_db_writes: bool,
    ) -> Result<Self, ContextraError> {
        let policy = SandboxPolicy {
            allow_db_reads: true,
            allow_db_writes,
            allow_code_execution: false,
            allow_cloud_egress: false,
            max_execution_ms: 5_000,
        };
        let sandbox = McpSandbox::new(policy)
            .map_err(|e| ContextraError::Internal(format!("Sandbox init: {e}")))?;
        Ok(Self::with_sandbox(db, embedder, Arc::new(sandbox)))
    }

    pub fn with_sandbox(
        db: Arc<Contextra>,
        embedder: Arc<dyn EmbeddingProvider>,
        sandbox: Arc<McpSandbox>,
    ) -> Self {
        #[cfg(feature = "kv-bridge")]
        let kv_bridge = setup_kv_bridge(&db);

        Self {
            db,
            embedder,
            sandbox,
            injection_guard: Arc::new(PromptInjectionGuard::from_env()),
            egress_classifier: Arc::new(DefaultEgressClassifier::default()),
            routing: None,
            #[cfg(feature = "kv-bridge")]
            kv_bridge,
        }
    }

    #[cfg(feature = "kv-bridge")]
    pub fn with_kv_bridge(
        mut self,
        kv_bridge: Option<Arc<contextra_infer_candle::KvBridgeAdapter>>,
    ) -> Self {
        self.kv_bridge = kv_bridge;
        self
    }

    pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self {
        self.injection_guard = injection_guard;
        self
    }

    pub fn with_egress_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self {
        self.egress_classifier = classifier;
        self
    }

    pub fn with_routing(mut self, routing: Option<Arc<RoutingHandle>>) -> Self {
        self.routing = routing;
        self
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
