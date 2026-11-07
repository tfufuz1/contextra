# AGENTS.md — `contextra-agent`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-agent` bildet eine leichtgewichtige, zustandslose Workflow-Ausführungsumgebung für lokale KI-Agenten (Ring 3).
Er kapselt `OrchestratorEngine` (Event-Loop), `StateGraph` (Workflow-Definitionen), `AgentContext` und das Append-Only `AuditLog`.
Historische Relikte (eingebettete SQLite-Tabellen, Robotik-spezifische Feedback-Schleifen gem. DEBT-02) wurden vollständig eliminiert: Der Agent operiert als rein deterministischer Workflow-Orchestrator (`checkpoint -> execute -> commit -> audit`) über generische Ports.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 3 (Agent Execution Pipeline & Event Loop)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `beta`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-db`, `contextra-graph`, `contextra-checkpoint`, `contextra-store`, `contextra-router` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Deterministische Workflows:** Workflow-Zustände werden über RAII-Guards und Checkpoints gesichert; bei Task-Abbruch erfolgt automatischer Rollback.
- **Dependency Injection:** Tools (wie `RetrieveTool`) und Speicher-Zugriffe werden über Trait-Ports (`ContextRetriever`) injiziert statt hart verdrahtet.
- **Audit-Compliance:** Jeder Agenten-Schritt wird unveränderlich im `AuditLog` protokolliert.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; robuste Fehlerfortpflanzung im gesamten Event-Loop.

## 4. Verbotene Muster
- Eingebettete relationale SQL-Engines oder SQLite-Tabellen im Crate (vollständig entfernt gem. DEBT-02).
- Feste Verdrahtung auf monolithische DB-Interna ohne Trait-Abstraktion.
- Halten von veränderlichen Zuständen außerhalb des `AgentContext`.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** Agent Execution Engine, Tool-Orchestrierung.
- **Konsumiert:** `contextra-ports`, `contextra-types`, `StorageEngine`, `CheckpointCoordinator`.
- **Bereitgestellt für:** MCP-Server (`contextra-mcp`) und CLI.

## 6. Tests, Sabotage & Qualitätssicherung
- **State-Graph Transition Tests:** Exhaustive Validierung aller Workflow-Übergänge und Zyklen-Erkennung.
- **Sabotage-Testing:** Simulation von unvollständigen Tool-Ausführungen und Netzwerk-Abbrüchen zur Prüfung der Rollback-Integrität.
- **Audit-Log Verifikation:** Prüfung auf lückenlose Aufzeichnung aller Schritte im Fehlerfall.

## 7. Offene Punkte & Technische Schulden
- **DEBT-02:** Vollständiger Abschluss der Bereinigung verbliebener Übergangs-Mocks.
