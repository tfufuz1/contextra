# contextra-agent

Autonome Agent-Workflow-Engine und Audit-Pipeline für Contextra.

Normativitätsklasse: **Rand `[R]`** (Ring 3, `beta`; §6.0, `capabilities.toml`).

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-agent` ist die autarke Agenten-Workflow-Engine des Contextra-Ökosystems. Sie steuert mehrstufige KI-Agenten-Prozesse entlang der Kernschleife `checkpoint → execute → commit → audit`.

Im Baukastensystem funktioniert `contextra-agent` völlig unabhängig von externen Workflow-Frameworks (wie LangGraph oder AutoGen) und ist von historischen In-Core-SQLite-Altlasten entkoppelt.

## 2. Baukasten-Einordnung & Status

- **Ring:** Ring 3 (Anwendungskern / Workflow Engine)
- **Status:** 🟢 Autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## 3. Deprecated Debt & Altlastenbereinigung
- **SQLite- & Tabellenaltlasten:** Frühe Prototypen stützten sich auf relationale Tabellen im Zustandsspeicher. Diese sind offiziell als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)) deklariert. Zustandsspeicherung und Checkpoints laufen rein über abstrakte Speicher- und Snapshot-Ports.
- **Entkopplung der Löschkette:** Sämtliche Löschbegrifflichkeiten und DSGVO-Löschketten verbleiben exklusiv in `contextra-erasure`. `contextra-agent` enthält keine juristischen Rechtsbegriffe oder Löschketten-Relikte.

## 4. Öffentliche API-Übersicht

- **Context & Graph:** `AgentContext`, `StateGraph`, `AgentNode`, `NodeType`, `WorkflowEdge`
- **Orchestrator Engine:** `OrchestratorEngine`, `StepResult`, `AgentTool`
- **Audit & Dead Letter Queue:** `AuditEntry`, `AuditLog`, `DeadLetterQueue`
- **Goal Condition & Budget:** `GoalCondition`, `TokenBudget`

## 5. Invarianten & Core Loop

1. **Auto-Checkpointing:** Vor jedem Handlungsschritt wird ein Zustands-Snapshot erzeugt. Tritt ein Fehler auf, sorgt der RAII-Drop des Guards für einen sauberen Rollback.
2. **Deterministisches Replay:** Zustandsspeicher ermöglichen den deterministischen Replay beliebiger Zwischenschritte.
3. **Unveränderliches Audit-Log:** Ausführungsschritte schreiben unveränderliche `AuditEntry`-Einträge.
4. **Token-Budget:** Jeder Schritt unterliegt strikten Limits via `TokenBudget`.
5. **Fail-Closed Validierung:** Validierung aller Node- und Task-IDs gegen Überlauf, Leerheit und Nullbytes.

## 6. Baukasten-Nutzung & Minimalbeispiel

Ein vollständiges, autarkes Minimalbeispiel findet sich unter `examples/minimal_pipeline.rs`:

```rust
use contextra_agent::{
    AgentContext, AgentNode, AgentTool, NodeType, StateGraph, StepResult, WorkflowEdge,
};
use contextra_ports::BoxFuture;
use contextra_types::Result;
use serde_json::json;

struct EchoTool;

impl AgentTool for EchoTool {
    fn name(&self) -> &'static str {
        "echo_tool"
    }

    fn execute<'a>(
        &'a self,
        _ctx: &'a AgentContext,
        input: serde_json::Value,
    ) -> BoxFuture<'a, Result<StepResult>> {
        Box::pin(async move {
            Ok(StepResult {
                output: json!({ "echo": input }),
                next_node: None,
                tokens_consumed: 15,
            })
        })
    }
}

fn main() -> Result<()> {
    let mut graph = StateGraph::new();
    graph.try_add_node(AgentNode {
        id: "start".to_string(),
        description: "Start step".to_string(),
        node_type: NodeType::Start,
        handler: None,
    })?;
    Ok(())
}
```
