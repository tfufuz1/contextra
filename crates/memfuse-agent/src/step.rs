// FILE-CONTEXT Header (Format v3)
// ZWECK: Step result structures and AgentTool trait definitions for orchestration.
// INVARIANTEN: StepResult contains node_id, output payload, and consumed tokens; AgentTool enforces async execution and cost estimation.
// NICHT-OFFENSICHTLICH: estimated_cost defaults to 0 for zero-cost tools and enables pre-execution budget validation.
// HOTSPOTS: AgentTool::execute (ll. 25-35).
// STAND: TS:2026-09-02T23:19:10Z (SESSION: 088b4a44)

//! Step result and tool trait definitions for agent workflows.
//!
//! Each agent step produces a [`StepResult`] and tools implement the
//! [`AgentTool`] trait to participate in the orchestration loop.

use crate::context::AgentContext;
use memfuse_core::{BoxFuture, MemFuseError, Result, TxId};
use memfuse_sandbox::SandboxedTool;
use serde::{Deserialize, Serialize};

/// Ein fehlgeschlagener Agent-Schritt der für spätere Analyse und Idempotenz-Prüfung persistiert wird.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepDeadLetter {
    /// Session-ID des Agenten
    pub session_id: String,
    /// Node-ID des fehlgeschlagenen Schritts
    pub node_id: String,
    /// Schrittnummer im Workflow (0-indexed)
    #[serde(default)]
    pub step_index: u64,
    /// WAL Transaktions-ID zum Zeitpunkt des Fehlschlags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TxId>,
    /// Ursache des Fehlers
    pub failure_reason: DeadLetterReason,
    /// Input der zum Fehler geführt hat
    pub input: serde_json::Value,
    /// Anzahl bisheriger Versuche (für Retry-Steuerung)
    pub attempt: u32,
    /// Zeitstempel des Fehlers (Unix-Sekunden)
    pub failed_at_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeadLetterReason {
    /// Tool hat nicht innerhalb des Timeouts geantwortet
    Timeout { timeout_ms: u64 },
    /// Budget erschöpft bevor der Schritt starten konnte
    BudgetExhausted { available: usize, required: usize },
    /// Tool hat einen nicht-retriable Fehler zurückgegeben
    ToolError { message: String },
    /// Maximale Retry-Anzahl für diesen Schritt erreicht
    MaxRetriesExceeded { attempts: u32 },
}

/// The explicit result of an agent step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub node_id: String,
    pub output: serde_json::Value,
    pub tokens_consumed: usize,
    /// Identifier condition of the next edge transition if dictated dynamically.
    pub next_edge: Option<String>,
}

pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;

    /// Returns the estimated token cost of executing this tool with the given input.
    ///
    /// Used for strict pre-execution budget validation to prevent side-effects when budget is exhausted.
    fn estimated_cost(&self, _input: &serde_json::Value) -> usize {
        0
    }

    fn execute<'a>(
        &'a self,
        ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>>;

    /// Maximale Ausführungszeit dieses Tools in Millisekunden.
    /// Standard: 30 Sekunden. Überschreibe für langläufige Tools.
    fn timeout_ms(&self) -> u64 {
        30_000
    }

    /// Ob dieser Schritt bei Timeout/transientem Fehler wiederholbar ist.
    fn is_retriable(&self) -> bool {
        true
    }

    /// Maximale Anzahl Retries. Standard: 2 (d.h. insgesamt 3 Versuche).
    fn max_retries(&self) -> u32 {
        2
    }
}

impl AgentTool for SandboxedTool {
    fn name(&self) -> &str {
        self.name()
    }

    fn estimated_cost(&self, _input: &serde_json::Value) -> usize {
        (self.capabilities().max_fuel / 1_000) as usize
    }

    fn timeout_ms(&self) -> u64 {
        let caps = self.capabilities();
        let timeout_ms = self.timeout().as_millis() as u64;
        if caps.max_wall_clock_ms > 0 {
            std::cmp::min(timeout_ms, caps.max_wall_clock_ms)
        } else {
            timeout_ms
        }
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            let input_bytes = serde_json::to_vec(&input).map_err(|e| {
                MemFuseError::InvalidInput(format!(
                    "Failed to serialize input for WASM tool: {}",
                    e
                ))
            })?;

            let output = self
                .execute_bytes(&input_bytes)
                .await
                .map_err(|err| match err {
                    memfuse_sandbox::SandboxError::Timeout { timeout_ms } => {
                        MemFuseError::SandboxTimeout(format!("Timeout after {}ms", timeout_ms))
                    }
                    memfuse_sandbox::SandboxError::MemoryExceeded { pages, max_pages } => {
                        MemFuseError::MemoryLimitExceeded(format!(
                            "Exceeded memory: {} pages > max {}",
                            pages, max_pages
                        ))
                    }
                    memfuse_sandbox::SandboxError::FuelExhausted { consumed } => {
                        MemFuseError::Sandbox(format!("Fuel exhausted: {} consumed", consumed))
                    }
                    memfuse_sandbox::SandboxError::CapabilityViolation { capability } => {
                        MemFuseError::PolicyViolation(format!(
                            "Capability violation: {}",
                            capability
                        ))
                    }
                    memfuse_sandbox::SandboxError::WasmTrap(msg) => {
                        MemFuseError::Sandbox(format!("WASM trap: {}", msg))
                    }
                    memfuse_sandbox::SandboxError::InvalidModule(msg) => {
                        MemFuseError::InvalidInput(format!("Invalid WASM module: {}", msg))
                    }
                    memfuse_sandbox::SandboxError::Runtime(msg) => {
                        MemFuseError::Sandbox(format!("Runtime error: {}", msg))
                    }
                })?;

            let parsed_output = serde_json::from_slice::<serde_json::Value>(&output.stdout)
                .unwrap_or_else(|_| {
                    serde_json::json!({
                        "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
                        "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
                    })
                });

            Ok(StepResult {
                node_id: self.name().to_string(),
                output: parsed_output,
                tokens_consumed: output.fuel_consumed as usize,
                next_edge: None,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memfuse_sandbox::{WasmCapabilities, WasmExecutor};
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_sandboxed_tool_agent_tool_execution_success() -> memfuse_core::Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let config = memfuse_db::MemFuseConfig::default();
        let db = Arc::new(memfuse_db::MemFuse::open_with_config(temp_dir.path(), config).await?);
        let state_coll = db.collection("test_tool_coll").await?;
        let ctx = AgentContext::try_new(
            "session_123",
            "start",
            db,
            state_coll,
            memfuse_core::TokenBudget::new(1000, 0),
        )?;

        let wat = r#"
            (module
                (import "wasi_snapshot_preview1" "fd_write"
                    (func $fd_write (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "{\"status\":\"ok\"}")
                (func (export "_start")
                    (i32.store (i32.const 0) (i32.const 16))
                    (i32.store (i32.const 4) (i32.const 15))
                    (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 32)))
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("valid WAT");
        let executor = Arc::new(WasmExecutor::new().expect("executor init"));
        let caps = WasmCapabilities::default();
        let tool = SandboxedTool::new(
            "test_wasm_tool",
            executor,
            wasm_bytes,
            caps,
            Duration::from_secs(2),
        )
        .expect("tool init");

        let result = tool
            .execute(&ctx, serde_json::json!({"param": "value"}))
            .await
            .expect("execution succeeded");

        assert_eq!(result.node_id, "test_wasm_tool");
        assert_eq!(result.output["status"], "ok");
        assert_eq!(tool.estimated_cost(&serde_json::Value::Null), 10_000);
        assert_eq!(tool.timeout_ms(), 2_000);
        Ok(())
    }

    #[tokio::test]
    async fn test_sandboxed_tool_agent_tool_fuel_exhaustion() -> memfuse_core::Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let config = memfuse_db::MemFuseConfig::default();
        let db = Arc::new(memfuse_db::MemFuse::open_with_config(temp_dir.path(), config).await?);
        let state_coll = db.collection("test_fuel_coll").await?;
        let ctx = AgentContext::try_new(
            "session_456",
            "start",
            db,
            state_coll,
            memfuse_core::TokenBudget::new(1000, 0),
        )?;

        let wat = r#"
            (module
                (func (export "_start")
                    (loop (br 0))
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("valid WAT");
        let executor = Arc::new(WasmExecutor::new().expect("executor init"));
        let caps = WasmCapabilities {
            max_fuel: 1_000,
            ..Default::default()
        };
        let tool = SandboxedTool::new(
            "loop_tool",
            executor,
            wasm_bytes,
            caps,
            Duration::from_secs(5),
        )
        .expect("tool init");

        let result = tool.execute(&ctx, serde_json::json!({})).await;

        assert!(
            matches!(result, Err(MemFuseError::Sandbox(ref msg)) if msg.contains("Fuel exhausted")),
            "Expected MemFuseError::Sandbox with fuel exhaustion message, got: {:?}",
            result
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sandboxed_tool_agent_tool_capability_violation() -> memfuse_core::Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let config = memfuse_db::MemFuseConfig::default();
        let db = Arc::new(memfuse_db::MemFuse::open_with_config(temp_dir.path(), config).await?);
        let state_coll = db.collection("test_cap_coll").await?;
        let ctx = AgentContext::try_new(
            "session_789",
            "start",
            db,
            state_coll,
            memfuse_core::TokenBudget::new(1000, 0),
        )?;

        let wat = r#"
            (module
                (import "memfuse" "host_cloud_query" (func $host_cloud_query (result i32)))
                (func (export "_start")
                    (drop (call $host_cloud_query))
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat).expect("valid WAT");
        let executor = Arc::new(WasmExecutor::new().expect("executor init"));
        let caps = WasmCapabilities {
            allow_cloud_egress: false,
            ..Default::default()
        };
        let tool = SandboxedTool::new(
            "egress_tool",
            executor,
            wasm_bytes,
            caps,
            Duration::from_secs(2),
        )
        .expect("tool init");

        let result = tool.execute(&ctx, serde_json::json!({})).await;

        assert!(
            matches!(result, Err(MemFuseError::PolicyViolation(ref msg)) if msg.contains("allow_cloud_egress")),
            "Expected PolicyViolation error for cloud egress, got: {:?}",
            result
        );
        Ok(())
    }
}
