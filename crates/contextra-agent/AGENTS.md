# AGENTS.md — contextra-agent
> Ring 3 · stable · Quelle: capabilities.toml · Spec: III.18, K.2, L.12, Teil 11

## 1. Zweck

`contextra-agent` bildet die persistent ausgeführte KI-Agenten-Umgebung in Ring 3. Die Crate kapselt die `OrchestratorEngine` (Event-Loop, Step-Dispatch), den `StateGraph` (Workflow-Graphen), das `AgentContext` (State-Management pro Task) und die Dead-Letter-Queue (`DeadLetterQueue`) zur sicheren Fehlerbehandlung unter Anbindung an `contextra-checkpoint`.

## 2. Modul-Karte

| Pfad | Verantwortung |
|---|---|
| `src/lib.rs` | Public API & Re-Exports für Orchestrator, StateGraph und Audit |
| `src/audit.rs` | Unveränderliches `AuditLog` & `AuditEntry` Logging über LSM Storage |
| `src/budget.rs` | Budget-Verwaltung (`TokenBudget`, `BudgetStrategy`, `reserve_tokens`) |
| `src/clm_scratchpad.rs` | Scratchpad-Checkpointing für Context-Augmented Working Memory |
| `src/context.rs` | `AgentContext`, `AgentStatus` & Identifier-Validierung (`AGT-AGN-001`) |
| `src/dlq.rs` | Dead-Letter-Queue (`DeadLetterQueue`) zur Kapselung fehlgeschlagener Task-Schritte |
| `src/engine/` | `OrchestratorEngine` Event-Loop, Tool-Execution & Ausführungsbedingungen |
| `src/engine.rs` | Haupt-Orchestrator-Modul (`EventLoopExitReason`, `MAX_WORKFLOW_STEPS`) |
| `src/event_source.rs` | `EventSource`-Protokoll & `PollingDocumentEventSource` für Hintergrund-Trigger |
| `src/goal_condition.rs` | `GoalCondition` Auswertung für dynamische Zielbedingungen |
| `src/graph.rs` | `StateGraph`, `AgentNode` & `WorkflowEdge` Definitionen |
| `src/step.rs` | `AgentTool` Trait, `StepResult` & `StepDeadLetter` Datenstrukturen |

## 3. Invarianten

- **Strikte State-Machine Transitions**: Zustandsübergänge im `AgentContext` (`Pending` -> `Running` -> `Suspended`/`Completed`/`Failed`) werden ausschließlich durch die `OrchestratorEngine` gesteuert.
- **Identifier-Validierung (AGT-AGN-001)**: `task_id` und `node_id` dürfen nicht leer sein, keine Null-Bytes oder Pfadtrenner enthalten und sind auf maximal 256 Bytes begrenzt (`cargo test -p contextra-agent`).
- **P28 Determinismus über Ports**: Zeit und Zufall werden strictly via injizierten `Clock` und `Rng` Ports bezogen (z. B. deterministisches Dead-Letter-Timestamping via `now_secs()`).

## 4. Verboten / Anti-Patterns

- **Manuelle Statusmanipulation**: Der `AgentStatus` darf von außen nicht direkt ohne OrchestratorEngine verändert werden.
- **Fehlendes Audit-Logging**: State-Transitions, Tool-Ausführungen und Fehler MÜSSEN im `AuditLog` aufgezeichnet werden.
- **Unvalidierte Task-IDs**: Task-IDs dürfen nicht direkt aus Rohnutzereingaben ohne `AgentContext::try_new` bzw. `validate_task_id` erzeugt werden.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- `OrchestratorEngine` führt async-Tasks aus; Lock-Pausen in Tools über `.await`-Grenzen sind verboten (P26).
- Hintergrund-Event-Quellen (`EventSource`) sind thread-sicher (`Send + Sync`) aufgebaut.
- `DeadLetterQueue` und In-Memory-Audits nutzen synchrone `parking_lot::Mutex` bzw. atomic primitives für kurze Sperrzeiten.

## 6. Verifikation

```bash
cargo test -p contextra-agent
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-agent
cargo xtask determinism-check
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- **Kapazitätsgrenzen für Event-Puffer**: `PollingDocumentEventSource` verwendet `MAX_PENDING_EVENTS_CAPACITY` (10.000), bei deren Überschreiten älteste Ereignisse verworfen werden.

