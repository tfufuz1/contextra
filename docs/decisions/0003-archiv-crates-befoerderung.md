# Decision Proposal: Status der als „Archiv“ markierten Crates mit Fan-in > 0

> **Status-Update 30.10.2026:** Überholt durch die Reduktion (D-R1, SPEC §15).

- **Status:** Proposed — Entscheidungsvorlage für Architekten und Maintainer
- **Datum:** 2026-10-29
- **Betroffene Crates:** `contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent`, `contextra-kvcache`, `contextra-infer-onnx`, `contextra-infer-ollama`

## Problemstellung
In der Dokumentation und früheren Arbeitspaketen wurden diverse Crates als „Archiv — NICHT anfassen" oder auslaufend deklariert. Eine Statikanalyse des Workspace-Graphen zeigt jedoch, dass diese Crates eine wesentliche Rolle spielen und von tragenden Kern-Crates (`contextra-db`, `contextra-engine`, `contextra-mcp`, `contextra-py`, `contextra-checkpoint`) aktiv als Abhängigkeit eingebunden werden (Fan-in > 0).

Acht Crates weisen aktiven Fan-in auf:
1. `contextra-graph` (Fan-in: 4: db, engine, cognition, agent)
2. `contextra-adapt` (Fan-in: 4: db, graph, router, py)
3. `contextra-router` (Fan-in: 2: agent, mcp, py)
4. `contextra-cognition` (Fan-in: 1: db)
5. `contextra-kvcache` (Fan-in: 0 im Normal-Build, optional in Features)
6. `contextra-agent` (Fan-in: 0 im Normal-Build, optional in Features)
7. `contextra-infer-onnx` (Fan-in: 1: mcp)
8. `contextra-infer-ollama` (Fan-in: 1: mcp)

Dies erzeugt einen direkten Widerspruch zu AGENTS.md §5.4.

## Optionen

### Option A: Beförderung aus dem Archiv in das Baukasten-Modell [M] (Empfehlung)
Die Crates werden nicht mehr als verbotenes „Archiv" eingestuft, sondern gemäß SPEC 3.0 als aktive Baukasten-Module `[M]` mit isoliertem Vertrag und definierter Schnittstelle geführt.

- **Vorteile:** Keine funktionale Verstümmelung des bestehenden Produktes; spiegelt den realen Code-Build wider; entspricht SPEC 3.0 §5.2.
- **Nachteile:** Maintenance-Aufwand für Baukasten-Schnittstellen.

### Option B: Vollständiger Abbau aller Abhängigkeiten & physikalische Entfernung
Sämtliche Vorkommen und Imports dieser Crates in `db`, `engine`, `mcp` etc. werden entfernt und die Crates per Git-Tag archiviert.

- **Vorteile:** Minimierung der Gesamt-LOC des Kern-Footprints.
- **Nachteile:** Hohes Invasivitätspotenzial; Verlust von RAG-Features (GraphRAG, Lyapunov Drift Control, Agent Pipeline).

## Empfehlung
**Option A** wird ausdrücklich empfohlen für `graph`, `adapt`, `router`, `cognition`, `infer-ollama` und `infer-onnx` gemäß SPEC 3.0 Baukasten-Modell.
