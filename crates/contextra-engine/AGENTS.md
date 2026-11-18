# AGENTS.md — contextra-engine — Deprecated Façade

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-engine.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-engine.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), ADR-0122, ADR-SCOPE-REDUCTION-001  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E2 (Veraltete Re-Export Façade)  
**Kern-Closure:** Ausschließlich contextra-db  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Deprecated Façade: Re-Export von `contextra-db` zur Abwärtskompatibilität bis zum API-Freeze (S-70).

---

## 2. Kern-Richtlinien für Agenten

- **Keine eigene Geschäftslogik:** `src/lib.rs` besteht ausschließlich aus `pub use contextra_db::*;`.
- **Keine neuen Typen oder Funktionen hier implementieren:** Alle Neuentwicklungen erfolgen direkt in `contextra-db`.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-engine` · **Ring:** 3 · **Version:** 0.1.0
- **Beschreibung:** Contextra — Core storage, index, and transaction orchestrator engine (Deprecated Façade)
- **src:** 1 Dateien, 7 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 1):** `contextra-db`
- **Interne Dev/Build-Abhängigkeiten (9):** `contextra-checkpoint`, `contextra-graph`, `contextra-ports`, `contextra-rank`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector`
- **Externe Abhängigkeiten (normal, 0):** –
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-db`; eingehend –
- **Features (9):** `auto-extraction-opt-out`, `background-maintenance`, `default`, `edge-reinforcement-learning`, `entity-extraction`, `exp-extraction`, `experimental-diskann`, `graph-connectivity-health`, `reranking`

### Öffentliche API (grob, aus src/)
- pub use: `contextra_db::*`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
