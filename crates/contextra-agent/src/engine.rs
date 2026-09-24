// FILE-CONTEXT Header (Format v3)
// ZWECK: Deterministic graph-walker orchestrator engine for autonomous agent workflows.
// INVARIANTEN: Enforces Checkpoint -> Execute -> Commit -> Audit loop per step; atomic commit & RAII guard protection.
// NICHT-OFFENSICHTLICH: Persists final state to LSM before final checkpoint; replay_from reconstructs state from checkpoint registry.
// HOTSPOTS: run_internal (ll. 90-180), replay_from (ll. 185-230).
// STAND: TS:2026-09-01T23:11:04Z (SESSION: 5a38054a)

//! Deterministic, persistent graph-walker engine for agent workflows.
//!
//! Implements the core execution loop: checkpoint → execute → commit → audit → resolve-next.

use crate::context::{validate_node_id, AgentContext, MAX_ID_LEN};
use crate::dlq::DeadLetterQueue;
use crate::graph::{AgentNode, NodeType, StateGraph};
use crate::step::{AgentTool, DeadLetterReason, StepDeadLetter, StepResult};
use contextra_checkpoint::{
    CheckpointGuard, CheckpointMeta, CheckpointRegistry, PersistentCheckpointStore,
};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, Result};
use contextra_store::LsmStorage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

/// Reason for exiting `OrchestratorEngine::run_event_loop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLoopExitReason {
    Shutdown,
    SourceExhausted,
}

/// Maximum allowed steps in a single workflow execution to prevent unbounded loops.
pub const MAX_WORKFLOW_STEPS: u64 = 10_000;

/// Async executor engine applying nodes in Sequence.
pub struct OrchestratorEngine {
    pub tools: HashMap<String, Box<dyn AgentTool>>,
    pub checkpoint_store: Arc<dyn CheckpointRegistry>,
    pub dead_letter_queue: Option<DeadLetterQueue>,
}

impl OrchestratorEngine {
    /// Attempts to construct a new [`OrchestratorEngine`], returning an error if checkpoint store initialization fails.
    pub fn try_new(storage: Arc<LsmStorage>) -> Result<Self> {
        let checkpoint_store = PersistentCheckpointStore::new(storage.clone(), "agent")?;
        Ok(Self {
            tools: HashMap::new(),
            checkpoint_store: Arc::new(checkpoint_store),
            dead_letter_queue: Some(DeadLetterQueue::new(storage)),
        })
    }

    /// Attempts to construct an [`OrchestratorEngine`] directly from a Contextra DB handle.
    pub fn try_from_db(db: &contextra_db::Contextra) -> Result<Self> {
        Self::try_new(db.inner_storage())
    }

    #[deprecated(note = "Use try_new instead to handle initialization errors without panicking")]
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        struct FallbackRegistry;
        impl CheckpointRegistry for FallbackRegistry {
            fn save_checkpoint<'a>(
                &'a self,
                _meta: CheckpointMeta,
            ) -> contextra_ports::BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
            fn load_checkpoint<'a>(
                &'a self,
                _seq_no: u64,
            ) -> contextra_ports::BoxFuture<'a, Result<Option<CheckpointMeta>>> {
                Box::pin(async move { Ok(None) })
            }
            fn list_checkpoints<'a>(
                &'a self,
            ) -> contextra_ports::BoxFuture<'a, Result<Vec<CheckpointMeta>>> {
                Box::pin(async move { Ok(Vec::new()) })
            }
        }
        impl contextra_ports::Checkpoint for FallbackRegistry {
            fn take_snapshot<'a>(
                &'a self,
                tx: contextra_types::TxId,
            ) -> contextra_ports::BoxFuture<'a, Result<contextra_types::WorkflowState>> {
                Box::pin(async move {
                    Ok(contextra_types::WorkflowState {
                        tx,
                        graph_hash: [0u8; 32],
                    })
                })
            }
            fn restore<'a>(
                &'a self,
                _state: &'a contextra_types::WorkflowState,
            ) -> contextra_ports::BoxFuture<'a, Result<()>> {
                Box::pin(async move { Ok(()) })
            }
        }

        Self::try_new(storage.clone()).unwrap_or_else(|e| {
            tracing::error!("Failed to initialize OrchestratorEngine: {}", e);
            Self {
                tools: HashMap::new(),
                checkpoint_store: Arc::new(FallbackRegistry),
                dead_letter_queue: Some(DeadLetterQueue::new(storage)),
            }
        })
    }

    /// Helper constructor creating OrchestratorEngine directly from Contextra DB handle.
    #[allow(deprecated)]
    #[deprecated(
        note = "Use try_from_db instead to handle initialization errors without panicking"
    )]
    pub fn from_db(db: &contextra_db::Contextra) -> Self {
        Self::new(db.inner_storage())
    }

    /// Attempts to register an agent tool with boundary validation on the tool name.
    pub fn try_register_tool(&mut self, tool: Box<dyn AgentTool>) -> Result<()> {
        let name = tool.name();
        if name.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Tool name cannot be empty".to_string(),
            ));
        }
        if name.len() > MAX_ID_LEN {
            return Err(ContextraError::InvalidInput(format!(
                "Tool name length {} exceeds maximum allowed length of {}",
                name.len(),
                MAX_ID_LEN
            )));
        }
        if name.contains('\0') {
            return Err(ContextraError::InvalidInput(
                "Tool name cannot contain null bytes".to_string(),
            ));
        }
        self.tools.insert(name.to_string(), tool);
        Ok(())
    }

    /// Recovers all registered/persisted orphaned sequence pins and checkpoints.
    pub async fn recover_orphans(&self) -> Result<()> {
        self.checkpoint_store.recover_orphaned_pins().await?;
        self.checkpoint_store.recover_orphaned_checkpoints().await?;
        Ok(())
    }

    pub async fn run(&self, ctx: &mut AgentContext, graph: &StateGraph) -> Result<()> {
        ctx.status = crate::context::AgentStatus::Running;
        let res = self.run_internal(ctx, graph).await;
        if res.is_err() {
            ctx.status = crate::context::AgentStatus::Failed;
        }
        res
    }

    async fn run_internal(&self, ctx: &mut AgentContext, graph: &StateGraph) -> Result<()> {
        // Startup orphan recovery MUST execute before first agent step (Befund A.3)
        self.recover_orphans().await?;

        loop {
            tokio::task::yield_now().await;

            if ctx.step_count >= MAX_WORKFLOW_STEPS {
                let err = ContextraError::Internal(format!(
                    "Maximum workflow step limit of {} exceeded for task {}",
                    MAX_WORKFLOW_STEPS, ctx.task_id
                ));
                self.audit_log_failure(ctx, &err.to_string()).await?;
                return Err(err);
            }

            let node = graph.get_node(&ctx.current_node).ok_or_else(|| {
                ContextraError::Internal(format!("Node {} not found", ctx.current_node))
            })?;

            match node.node_type {
                NodeType::End => {
                    ctx.status = crate::context::AgentStatus::Completed;
                    // Persist final state before last checkpoint (FIND-SAOS-001)
                    self.persist_final_state(ctx).await?;
                    ctx.db.inner_storage().flush().await?; // Ensure durability
                    self.checkpoint(ctx).await?;
                    return Ok(());
                }
                NodeType::Start | NodeType::Task => {
                    // 1. Checkpoint BEFORE execution (AC-1) with RAII CheckpointGuard
                    let tx_id = ctx.db.inner_storage().last_tx_id().await?;
                    let guard = if let Some(orphan_reg) = self.checkpoint_store.orphan_registry() {
                        CheckpointGuard::for_agent_step_with_registry(
                            ctx.db.inner_storage(),
                            tx_id,
                            orphan_reg.clone(),
                        )
                        .await?
                    } else {
                        CheckpointGuard::for_agent_step(ctx.db.inner_storage(), tx_id).await?
                    };
                    self.checkpoint(ctx).await?;

                    // Prepare step input
                    let input = ctx
                        .memory
                        .get("last_output")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);

                    // PRE-CHECK & ATOMIC RESERVATION vor Ausführung: Reserve budget strictly before tool execution
                    let estimated_cost = if node.node_type != NodeType::Start {
                        if let Some(handler_name) = &node.handler {
                            if let Some(tool) = self.tools.get(handler_name) {
                                tool.estimated_cost(&input)
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    };

                    let reservation = if node.node_type != NodeType::Start && estimated_cost > 0 {
                        match ctx.budget.reserve(estimated_cost) {
                            Ok(res) => Some(res),
                            Err(err) => {
                                if let Some(ref dlq) = self.dead_letter_queue {
                                    let letter = StepDeadLetter {
                                        session_id: ctx.task_id.clone(),
                                        node_id: node.id.clone(),
                                        step_index: ctx.step_count,
                                        tx_id: Some(tx_id),
                                        failure_reason: DeadLetterReason::BudgetExhausted {
                                            available: ctx.budget.available(),
                                            required: estimated_cost,
                                        },
                                        input: input.clone(),
                                        attempt: 0,
                                        failed_at_secs: SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs(),
                                    };
                                    if let Err(e) = dlq.push(&letter).await {
                                        tracing::error!("DLQ push failed: {}", e);
                                    }
                                }
                                self.audit_log_failure(ctx, &err.to_string()).await?;
                                return Err(err);
                            }
                        }
                    } else {
                        None
                    };

                    // 2. Resolve handler (Optional for Start nodes)
                    let result_res = if let Some(handler_name) = &node.handler {
                        if let Some(tool) = self.tools.get(handler_name) {
                            let timeout_duration =
                                std::time::Duration::from_millis(tool.timeout_ms());
                            let max_attempts = if tool.is_retriable() {
                                tool.max_retries() + 1
                            } else {
                                1
                            };
                            let mut execution_res = Err(ContextraError::Internal(format!(
                                "Tool {} failed without execution",
                                handler_name
                            )));

                            'retry: for attempt in 0..max_attempts {
                                if attempt > 0 {
                                    // Verify budget availability before retry attempt
                                    if ctx.budget.available() == 0 {
                                        execution_res = Err(ContextraError::MemoryBudgetExceeded {
                                            used_mb: ctx.budget.consumed() as u64,
                                            limit_mb: ctx.budget.limit as u64,
                                        });
                                        if let Some(ref dlq) = self.dead_letter_queue {
                                            let letter = StepDeadLetter {
                                                session_id: ctx.task_id.clone(),
                                                node_id: node.id.clone(),
                                                step_index: ctx.step_count,
                                                tx_id: Some(tx_id),
                                                failure_reason: DeadLetterReason::BudgetExhausted {
                                                    available: 0,
                                                    required: estimated_cost,
                                                },
                                                input: input.clone(),
                                                attempt,
                                                failed_at_secs: SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs(),
                                            };
                                            if let Err(dlq_err) = dlq.push(&letter).await {
                                                tracing::error!(
                                                    "DLQ push failed on retry budget exhaustion: {}",
                                                    dlq_err
                                                );
                                            }
                                        }
                                        break 'retry;
                                    }

                                    let wait_ms = 100u64 * (1u64 << attempt.min(4));
                                    tokio::time::sleep(std::time::Duration::from_millis(wait_ms))
                                        .await;
                                }

                                let execute_future = tool.execute(ctx, input.clone());

                                match tokio::time::timeout(timeout_duration, execute_future).await {
                                    Ok(Ok(result)) => {
                                        execution_res = Ok(result);
                                        break 'retry;
                                    }
                                    Ok(Err(e)) => {
                                        let err_msg: String = e.to_string();
                                        execution_res = Err(e);
                                        if !tool.is_retriable() {
                                            if let Some(ref dlq) = self.dead_letter_queue {
                                                let letter = StepDeadLetter {
                                                    session_id: ctx.task_id.clone(),
                                                    node_id: node.id.clone(),
                                                    step_index: ctx.step_count,
                                                    tx_id: Some(tx_id),
                                                    failure_reason: DeadLetterReason::ToolError {
                                                        message: err_msg,
                                                    },
                                                    input: input.clone(),
                                                    attempt,
                                                    failed_at_secs: SystemTime::now()
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                        .unwrap_or_default()
                                                        .as_secs(),
                                                };
                                                if let Err(dlq_err) = dlq.push(&letter).await {
                                                    tracing::error!("DLQ push failed: {}", dlq_err);
                                                }
                                            }
                                            break 'retry;
                                        } else if attempt + 1 == max_attempts {
                                            if let Some(ref dlq) = self.dead_letter_queue {
                                                let letter = StepDeadLetter {
                                                    session_id: ctx.task_id.clone(),
                                                    node_id: node.id.clone(),
                                                    step_index: ctx.step_count,
                                                    tx_id: Some(tx_id),
                                                    failure_reason:
                                                        DeadLetterReason::MaxRetriesExceeded {
                                                            attempts: max_attempts,
                                                        },
                                                    input: input.clone(),
                                                    attempt,
                                                    failed_at_secs: SystemTime::now()
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                        .unwrap_or_default()
                                                        .as_secs(),
                                                };
                                                if let Err(dlq_err) = dlq.push(&letter).await {
                                                    tracing::error!("DLQ push failed: {}", dlq_err);
                                                }
                                            }
                                        }
                                    }
                                    Err(_elapsed) => {
                                        let timeout_err = ContextraError::Timeout {
                                            operation: format!("tool:{}", handler_name),
                                            timeout_ms: tool.timeout_ms(),
                                        };
                                        execution_res = Err(timeout_err);

                                        if let Some(ref dlq) = self.dead_letter_queue {
                                            let letter = StepDeadLetter {
                                                session_id: ctx.task_id.clone(),
                                                node_id: node.id.clone(),
                                                step_index: ctx.step_count,
                                                tx_id: Some(tx_id),
                                                failure_reason: DeadLetterReason::Timeout {
                                                    timeout_ms: tool.timeout_ms(),
                                                },
                                                input: input.clone(),
                                                attempt,
                                                failed_at_secs: SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs(),
                                            };
                                            if let Err(dlq_err) = dlq.push(&letter).await {
                                                tracing::error!("DLQ push failed: {}", dlq_err);
                                            }
                                        }

                                        if !tool.is_retriable() {
                                            break 'retry;
                                        }
                                    }
                                }
                            }
                            execution_res
                        } else {
                            Err(ContextraError::Internal(format!(
                                "Tool {} not registered",
                                handler_name
                            )))
                        }
                    } else if node.node_type == NodeType::Start {
                        // Pass-through result for start nodes without handlers
                        Ok(StepResult {
                            node_id: node.id.clone(),
                            output: serde_json::Value::Null,
                            tokens_consumed: 0,
                            next_edge: None,
                        })
                    } else {
                        Err(ContextraError::Internal(format!(
                            "Task Node {} lacks handler",
                            node.id
                        )))
                    };

                    if let Some((router, decision_id)) = ctx.pending_routing_decision.take() {
                        let outcome = match &result_res {
                            Ok(_) => contextra_router::RoutingOutcome::Success,
                            Err(err) => contextra_router::RoutingOutcome::Rejected {
                                reason: Some(err.to_string()),
                            },
                        };
                        router.record_outcome(decision_id, outcome);
                    }

                    let result = match result_res {
                        Ok(res) => {
                            // Settle RAII reservation and reconcile actual tokens consumed
                            if let Some(res_guard) = reservation {
                                res_guard.settle();
                                if res.tokens_consumed > estimated_cost {
                                    ctx.budget.consume(res.tokens_consumed - estimated_cost);
                                } else if estimated_cost > res.tokens_consumed {
                                    ctx.budget.refund(estimated_cost - res.tokens_consumed);
                                }
                            } else if res.tokens_consumed > 0 {
                                ctx.budget.consume(res.tokens_consumed);
                            }
                            ctx.memory
                                .insert("last_output".to_string(), res.output.clone());
                            res
                        }
                        Err(err) => {
                            // On execution failure, reservation is dropped without settle(),
                            // which automatically refunds estimated_cost via RAII Drop guard.
                            self.audit_log_failure(ctx, &err.to_string()).await?;
                            return Err(err);
                        }
                    };

                    // 3. Audit log (AC-3) - Audit trail is source of truth for "what was attempted".
                    // Executed before commit_step so a failure here leaves state uncommitted for clean retry.
                    self.audit_log(ctx, &result).await?;

                    // 4. Atomic commit to LSM - Source of truth for "what was accepted".
                    self.commit_step(ctx, &result).await?;

                    // 6. Resolve next edge
                    let next_node = match self.resolve_next_node(graph, &ctx.current_node, &result)
                    {
                        Ok(next) => next,
                        Err(err) => {
                            self.audit_log_failure(ctx, &err.to_string()).await?;
                            return Err(err);
                        }
                    };
                    ctx.current_node = next_node;
                    ctx.step_count += 1;

                    // 7. Step completed successfully: commit CheckpointGuard RAII guard
                    guard.commit()?;
                }
                NodeType::Decision => {
                    let next = match self.evaluate_decision(graph, node, ctx) {
                        Ok(next) => next,
                        Err(err) => {
                            self.audit_log_failure(ctx, &err.to_string()).await?;
                            return Err(err);
                        }
                    };
                    ctx.current_node = next;
                }
            }
        }
    }

    /// Setzt den AgentContext auf einen früheren Checkpoint zurück (AC-2).
    ///
    /// # Adressierung von Checkpoints
    /// Das `identifier`-Argument unterstützt folgende Adressierungsformate:
    /// - `"step:<N>"`: Explizite Adressierung nach Schrittnummer (z. B. `"step:1"`).
    /// - `"node:<name>"`: Explizite Adressierung nach Node-Name (z. B. `"node:1"` oder `"node:step_a"`).
    ///
    /// **Fallback (Abwärtskompatibilität):**
    /// Falls kein Präfix (`step:` oder `node:`) angegeben ist:
    /// - Wenn `identifier` als `u64` geparst werden kann, wird es als Schrittnummer interpretiert.
    /// - Andernfalls wird es als Node-Name interpretiert.
    pub async fn replay_from(&self, ctx: &mut AgentContext, identifier: &str) -> Result<()> {
        validate_node_id(identifier)?;

        let checkpoints = self.checkpoint_store.list_checkpoints().await?;

        let checkpoint = checkpoints
            .iter()
            .rfind(|c| {
                if !c.name.starts_with(&format!("task:{}:", ctx.task_id)) {
                    return false;
                }
                if let Some(step_str) = identifier.strip_prefix("step:") {
                    if let Ok(step) = step_str.parse::<u64>() {
                        return c.name.contains(&format!(":step:{}:", step));
                    }
                }
                if let Some(node_name) = identifier.strip_prefix("node:") {
                    return c.name.ends_with(&format!(":node:{}", node_name));
                }
                if let Ok(step) = identifier.parse::<u64>() {
                    c.name.contains(&format!(":step:{}:", step))
                } else {
                    c.name.ends_with(&format!(":node:{}", identifier))
                }
            })
            .ok_or_else(|| {
                let parsed_num = if let Some(s) = identifier.strip_prefix("step:") {
                    s.parse::<u64>().ok()
                } else {
                    identifier.parse::<u64>().ok()
                };

                let extra_hint = if let Some(num) = parsed_num {
                    format!(
                        " Konnte keinen Checkpoint für Schritt {} finden. Falls ein Node mit dem Namen '{}' gemeint war, nutze das Format 'node:{}' zur expliziten Adressierung.",
                        num, num, num
                    )
                } else {
                    String::new()
                };

                ContextraError::Internal(format!(
                    "Checkpoint '{}' für Task '{}' nicht gefunden.{}",
                    identifier, ctx.task_id, extra_hint
                ))
            })?;

        if let Some(node) = checkpoint
            .metadata
            .get("current_node")
            .and_then(|v| v.as_str())
        {
            ctx.current_node = node.to_string();
        }
        if let Some(step) = checkpoint
            .metadata
            .get("step_count")
            .and_then(|v| v.as_u64())
        {
            ctx.step_count = step;
        }
        if let Some(memory) = checkpoint.metadata.get("memory").and_then(|v| {
            serde_json::from_value::<HashMap<String, serde_json::Value>>(v.clone()).ok()
        }) {
            ctx.memory = memory;
        }

        if let Some(consumed) = checkpoint
            .metadata
            .get("budget_consumed")
            .and_then(|v| v.as_u64())
        {
            let restored_budget =
                contextra_types::TokenBudget::new(ctx.budget.limit, ctx.budget.reserved)
                    .with_strategy(ctx.budget.strategy.clone());
            restored_budget.consume(consumed as usize);
            ctx.budget = restored_budget;
        } else if let Some(available) = checkpoint
            .metadata
            .get("budget_available")
            .and_then(|v| v.as_u64())
        {
            let total_usable = ctx
                .budget
                .effective_limit()
                .saturating_sub(ctx.budget.reserved);
            let consumed = total_usable.saturating_sub(available as usize);
            let restored_budget =
                contextra_types::TokenBudget::new(ctx.budget.limit, ctx.budget.reserved)
                    .with_strategy(ctx.budget.strategy.clone());
            restored_budget.consume(consumed);
            ctx.budget = restored_budget;
        } else {
            tracing::warn!(
                task_id = %ctx.task_id,
                checkpoint_name = %checkpoint.name,
                "Checkpoint metadata does not contain budget state; proceeding with default/current budget."
            );
        }

        self.checkpoint_store
            .restore(&checkpoint.into_workflow_state())
            .await
    }

    pub async fn checkpoint(&self, ctx: &AgentContext) -> Result<()> {
        let checkpoint_name = format!(
            "task:{}:step:{}:node:{}",
            ctx.task_id, ctx.step_count, ctx.current_node
        );
        let metadata = serde_json::json!({
            "current_node": ctx.current_node,
            "step_count":   ctx.step_count,
            "memory":       ctx.memory,
            "budget_consumed": ctx.budget.consumed(),
            "budget_available": ctx.budget.available()
        });

        let seq_no = ctx.db.last_committed_seq().await?;
        let tx_id = ctx.db.inner_storage().last_tx_id().await?;

        let meta = CheckpointMeta {
            name: checkpoint_name,
            collection_id: ctx.state_collection.name().to_string(),
            seq_no,
            tx_id,
            metadata,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        };

        self.checkpoint_store.save_checkpoint(meta).await
    }

    /// Continuous event loop reading telemetry events from an `EventSource`,
    /// attaching each event to `AgentContext`, executing `run()`, and checkpointing state after each event.
    pub async fn run_event_loop(
        &self,
        ctx: &mut AgentContext,
        graph: &StateGraph,
        source: &mut dyn crate::event_source::EventSource,
        shutdown: tokio_util::sync::CancellationToken,
    ) -> Result<EventLoopExitReason> {
        loop {
            if shutdown.is_cancelled() {
                return Ok(EventLoopExitReason::Shutdown);
            }

            tokio::select! {
                _ = shutdown.cancelled() => {
                    return Ok(EventLoopExitReason::Shutdown);
                }
                event_res = source.next_event() => {
                    match event_res? {
                        Some(event) => {
                            ctx.attach_event(event);
                            self.run(ctx, graph).await?;
                            self.checkpoint(ctx).await?;
                        }
                        None => {
                            if source.is_exhausted() {
                                return Ok(EventLoopExitReason::SourceExhausted);
                            }
                            tokio::select! {
                                _ = shutdown.cancelled() => {
                                    return Ok(EventLoopExitReason::Shutdown);
                                }
                                _ = source.wait_for_event() => {}
                            }
                        }
                    }
                }
            }
        }
    }

    async fn commit_step(&self, ctx: &AgentContext, result: &StepResult) -> Result<()> {
        let state_doc_id = format!("task:{}:step:{}", ctx.task_id, ctx.step_count);
        let tx_id = ctx.db.inner_storage().last_tx_id().await?;
        let metadata = serde_json::json!({
            "stage": "commit",
            "node": ctx.current_node,
            "memory": ctx.memory,
            "output": result.output,
            "tokens_consumed": result.tokens_consumed,
            "status": ctx.status,
            "tx_id": tx_id.0
        });

        // Use direct KV storage pattern for workflow history without vector index participation
        ctx.state_collection.put_kv(&state_doc_id, &metadata).await
    }

    /// Writes an immutable audit entry for a step execution.
    ///
    /// Executed before `commit_step()`. Note: A `ContextraError::Conflict` here signifies that
    /// an audit entry for this `(task_id, step_count)` already exists from a previous partial attempt.
    /// In such recovery scenarios, the caller must advance to a new `step_count` (e.g., via
    /// `ctx.next_retry_step_count()`), and NOT attempt to overwrite the existing entry.
    async fn audit_log(&self, ctx: &AgentContext, result: &StepResult) -> Result<()> {
        let tx_id = ctx.db.inner_storage().last_tx_id().await?;
        // Generate immutable audit trace and store it
        let entry = crate::audit::AuditEntry {
            task_id: ctx.task_id.clone(),
            step_count: ctx.step_count,
            node_id: ctx.current_node.clone(),
            tokens_consumed: result.tokens_consumed,
            payload: result.output.clone(),
            error: None,
            tx_id: Some(tx_id),
        };

        crate::audit::AuditLog::append_to(&ctx.state_collection, &entry).await
    }

    async fn audit_log_failure(&self, ctx: &AgentContext, error_message: &str) -> Result<()> {
        let tx_id = ctx.db.inner_storage().last_tx_id().await.ok();
        let entry = crate::audit::AuditEntry {
            task_id: ctx.task_id.clone(),
            step_count: ctx.step_count,
            node_id: ctx.current_node.clone(),
            tokens_consumed: 0,
            payload: serde_json::Value::Null,
            error: Some(error_message.to_string()),
            tx_id,
        };

        crate::audit::AuditLog::append_to(&ctx.state_collection, &entry).await
    }

    async fn persist_final_state(&self, ctx: &AgentContext) -> Result<()> {
        let final_id = format!("task:{}:final", ctx.task_id);
        let metadata = serde_json::json!({
            "stage": "final",
            "status": ctx.status,
            "task_id": ctx.task_id,
            "step_count": ctx.step_count,
            "memory": ctx.memory,
            "tokens_total": ctx.budget.consumed()
        });

        ctx.state_collection.put_kv(&final_id, &metadata).await
    }

    fn resolve_next_node(
        &self,
        graph: &StateGraph,
        current_node: &str,
        result: &StepResult,
    ) -> Result<String> {
        let edges = graph
            .edges
            .iter()
            .filter(|e| e.from == current_node)
            .collect::<Vec<_>>();

        if edges.is_empty() {
            return Err(ContextraError::Internal(format!(
                "Dead end at node {}",
                current_node
            )));
        }

        if let Some(ref forced_next) = result.next_edge {
            if edges.iter().any(|e| &e.to == forced_next) {
                return Ok(forced_next.to_string());
            }
        }

        // Default to highest priority
        let edge = edges
            .iter()
            .max_by_key(|e| e.priority)
            .ok_or_else(|| ContextraError::Internal("No edges found".to_string()))?;
        Ok(edge.to.to_string())
    }

    fn evaluate_decision(
        &self,
        graph: &StateGraph,
        node: &AgentNode,
        ctx: &AgentContext,
    ) -> Result<String> {
        let edges = graph
            .edges
            .iter()
            .filter(|e| e.from == node.id)
            .collect::<Vec<_>>();

        if edges.is_empty() {
            return Err(ContextraError::Internal(format!(
                "Decision Node {} has no outgoing edges",
                node.id
            )));
        }

        let mut matching: Vec<_> = edges
            .iter()
            .filter(|e| match &e.condition {
                None => true,
                Some(expr) => evaluate_condition_expr(expr, ctx),
            })
            .collect();

        matching.sort_by_key(|e| std::cmp::Reverse(e.priority));

        matching.first().map(|e| e.to.to_string()).ok_or_else(|| {
            ContextraError::Internal(format!(
                "Decision Node {} has no matching edge for current context",
                node.id
            ))
        })
    }
}

mod condition;

pub use condition::evaluate_condition_expr;

#[cfg(test)]
mod tests;
