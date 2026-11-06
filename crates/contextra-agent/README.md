# contextra-agent

Autonome Agent-Workflow-Engine und Audit-Pipeline.

Normativitätsklasse: **Kern `[K]`** (Ring 3, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-agent` ist die autarke Agenten-Workflow-Engine des Contextra-Ökosystems. Sie steuert mehrstufige KI-Agenten-Prozesse entlang der Kernschleife `checkpoint → execute → commit → audit`.

Im Baukastensystem funktioniert `contextra-agent` völlig unabhängig von externen Workflow-Frameworks (wie LangGraph oder AutoGen) und ist von historischen In-Core-SQLite-Altlasten entkoppelt.

## Baukasten-Einordnung & Status

- **Ring:** Ring 3 (Anwendungskern / Workflow Engine)
- **Status:** 🟢 Autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Deprecated Debt & Altlastenbereinigung
- **SQLite- & Tabellenaltlasten:** Frühe Prototypen stützten sich auf relationale Tabellen im Zustandsspeicher. Diese sind offiziell als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)) deklariert. Zustandsspeicherung und Checkpoints laufen rein über abstrakte Speicher- und Snapshot-Ports.

## Öffentliche API-Übersicht

- **Context & Graph:** `AgentContext`, `StateGraph`, `AgentNode`, `NodeType`, `WorkflowEdge`
- **Orchestrator Engine:** `OrchestratorEngine`, `StepResult`, `AgentTool`
- **Audit & Checkpoints:** `AuditEntry`, `PersistentCheckpointStore`, `CheckpointGuard`

## Invarianten & Core Loop

1. **Auto-Checkpointing:** Vor jedem Handlungsschritt wird ein Zustands-Snapshot erzeugt. Tritt ein Fehler auf, sorgt der RAII-Drop des Guards für einen sauberen Rollback.
2. **Deterministisches Replay:** Zustandsspeicher ermöglichen den deterministischen Replay beliebiger Zwischenschritte.
3. **Unveränderliches Audit-Log:** Ausführungsschritte schreiben unveränderliche `AuditEntry`-Einträge.
4. **Token-Budget:** Jeder Schritt unterliegt strikten Limits via `TokenBudget`.

## Baukasten-Nutzung

```rust
use contextra_agent::{AgentContext, OrchestratorEngine, StateGraph, NodeType};
use contextra_types::TokenBudget;

// Kann mit beliebigen Speichern kombiniert werden, die die Port-Schnittstellen erfüllen
```
