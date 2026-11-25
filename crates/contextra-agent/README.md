# contextra-agent

Persistent agent workflow engine for Contextra — `checkpoint → execute → commit → audit` loop.

## Overview

`contextra-agent` provides a pure Rust, sovereign orchestrator engine for multi-step AI agent workflows without external dependencies like LangGraph or AutoGen.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 3 (Anwendungskern / Workflow Engine)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Context & Graph:** `AgentContext`, `StateGraph`, `AgentNode`, `NodeType`, `WorkflowEdge`
- **Orchestrator Engine:** `OrchestratorEngine`, `StepResult`, `AgentTool`
- **Audit & Checkpoints:** `AuditEntry`, `PersistentCheckpointStore`

## Invariants & Core Loop

1. **AC-1: Auto-checkpoint before step**:
   Before executing any node handler, a snapshot checkpoint is stored in `PersistentCheckpointStore`, and a RAII `CheckpointGuard` is initialized. If an error occurs during execution, dropping the guard triggers an automatic transaction rollback to preserve state consistency.
2. **AC-2: Deterministic replay & rollback**:
   State checkpoints support restoring `AgentContext` (`current_node`, `step_count`, `memory`) to any step index or node identifier.
3. **AC-3: Immutable audit log**:
   All step executions append immutable `AuditEntry` records keyed by `audit:{task_id}:step:{step_count}` into the agent state collection. No delete/update paths exist.
4. **Token Budget Enforcement**:
   Steps consume tokens from `TokenBudget`. Budget exhaustion immediately halts execution and returns a `ContextraError::Internal`.

## Usage Example

```rust
use contextra_agent::{AgentContext, AgentTool, NodeType, OrchestratorEngine, StateGraph, StepResult};
use contextra_types::TokenBudget;
use contextra_db::{Contextra, ContextraConfig};
use std::sync::Arc;

let db = Arc::new(Contextra::open_with_config(path, config).await?);
let state_col = db.collection("agent_state").await?;
let mut ctx = AgentContext::try_new("task-100", "start", db.clone(), state_col, TokenBudget::new(1000, 0))?;

let mut graph = StateGraph::new();
graph.try_add_node("start", "Start Node", NodeType::Start, None)?;
graph.try_add_node("end", "End Node", NodeType::End, None)?;
graph.try_add_edge("start", "end", None, 1)?;

let engine = OrchestratorEngine::try_new(db.inner_storage())?;
engine.run(&mut ctx, &graph).await?;
```

## Architektur & Verweise

Details zur Workflow Engine finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §4.2.
