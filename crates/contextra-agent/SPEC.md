# Spezifikation: `contextra-agent`

**Ring:** Ring 3 (Orchestration & Cognition)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.3
**Klasse:** C (Autonome Workflows und Agentik)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-agent` implementiert autonome Agenten-Workflows, Werkzeugnutzung (Tool Calling), iteratives Reasoning und Scratchpad-Zustände auf Basis des Contextra-Speicherkerns:
- **Scope:** Orchestrierung von Prompt-Zyklen, Retrieval-Augmented Generation (RAG)-Agenten, Planungs- und Reflexionsschleifen.
- **Speicher-Entkopplung:** Nutzt das Trait `ContextStore` und `ScratchpadStore` zur Entkopplung von konkreten Datenbank-Backends.

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-agent`:*
- **LOC:** 4.297 LOC über 18 Rust-Dateien.
- **Typen-Inventar:** 5 Traits, 32 Structs, 6 Enums, 100 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 6 interne Crates (`contextra-types`, `contextra-ports`, `contextra-engine`, `contextra-db`, `contextra-checkpoint`, `contextra-cognition`).
- **Audit-Befund:** Saubere Entkopplung von der Löschdomäne (0 Deletion-Leaks).

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `agent.rs` | Hauptagent-Schleife, Zustandsmaschine und Workflow-Ausführung | `Agent`, `AgentConfig`, `AgentState` |
| `context.rs` | Kontext-Aggregation, Sliding Window und Token-Budgetierung | `ContextManager`, `ContextWindow` |
| `clm_scratchpad.rs` | Ephemeres Notizbuch für Reasoning-Ketten und Tool-Ausgaben | `ScratchpadStore`, `ScratchpadEntry` |
| `tools/` | Integrierte Werkzeuge: Retrieve, Search, Graph-Lookup, Calculator | `Tool`, `ToolRegistry`, `ToolOutput` |
| `planning.rs` | Zielzerlegung, Planungs- und Re-Planning-Algorithmen | `Plan`, `PlanStep`, `Planner` |
| `error.rs` | Spezifische Agenten- und Ausführungsfehler | `AgentError`, `Result<T>` |

---

## 4. Invarianten (INV-AGT-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-AGT-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` ist im gesamten Crate verbindlich gesetzt. | `cargo check -p contextra-agent` |
| `INV-AGT-BUDGET-BOUND-1` | Kontextfenster überschreiten niemals das konfigurierte Token-Budget (`INV-BOUNDED-MEMORY`). | `tests/budget_tests.rs` |
| `INV-AGT-TOOL-FAIL-CLOSED` | Fehlerhafte Tool-Ausführungen führen zu typisierten `ToolError`-Ereignissen ohne Prozessabsturz. | `tests/tool_failure_resilience.rs` |
| `INV-AGT-ISOLATED-STORE` | Speicherzugriffe erfolgen strikt über das `ContextStore`-Trait. | `src/compat/mod.rs` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. Lokale Schnittstellen `ContextStore` und `ScratchpadStore` etablieren.
2. Speicheranbindung (`context.rs`, `clm_scratchpad.rs`, `tools/retrieve.rs`) in `src/compat/` kapseln.
3. Kopplung an `contextra-router` hinter optionales Feature `routing` stellen.
