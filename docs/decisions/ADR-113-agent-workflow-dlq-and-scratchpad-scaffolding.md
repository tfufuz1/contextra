# ADR-113: Status und Architekturanalyse des Agent Workflow Execution, DLQ & CLM Scratchpad Scaffolding-Codes

* **Datum**: 2026-10-04
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-agent` (`clm_scratchpad.rs`, `engine.rs`, `event_source.rs`, `dlq.rs`, `context.rs`, `graph.rs`, `audit.rs`), `contextra-checkpoint` (`store.rs`, `orphan.rs`), `contextra-cognition` (`consolidation_executor.rs`, `maintenance_scheduler.rs`, `context.rs`), `contextra-db` (`volatile_vault.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_VOLLSTAENDIGE_FEATURE_SPEZIFIKATION.md` (§7, §11, §11.1), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen des Auditings wurden 34 öffentliche Funktionen identifiziert, die sich über Agenten-Workflow, Dead-Letter-Queues, Checkpointing und Kognitions-Konsolidierung erstrecken.

### Kernbefunde:
1. **CLM Scratchpad & Volatile Context Vault (`clm_scratchpad.rs`, `volatile_vault.rs`)**:
   Funktionen wie `with_created_at_tx`, `with_tenant_id`, `with_audit_sink`, `with_cache_invalidator`, `with_pinned_region`, `with_clm_scratchpad` und `preview_metadata` bilden den temporären Arbeitskontext für sehr lange Agentenschleifen (§11.1, RAM-only `mlock` Vault).
2. **Agent Engine, DLQ & State Graph (`agent/engine.rs`, `dlq.rs`, `context.rs`, `graph.rs`, `event_source.rs`, `audit.rs`)**:
   `from_db`, `try_register_tool`, `new_with_metrics`, `is_already_committed`, `set_cache_directive`, `get_cache_directive`, `directive_for_node`, `try_attach_event`, `try_add_edge_with_goal`, `with_last_seen_seq` und `migrate_legacy_audit_entries` bilden die Steuerung für den Agenten-Loop `Checkpoint → Execute → Commit → Audit` (§11).
3. **Checkpoint Hardlink Cloning & Orphan Recovery (`checkpoint/store.rs`, `checkpoint/orphan.rs`)**:
   `create_hardlink_clone`, `create_guard`, `get_checkpoint`, `checkpoint_guard_skipped_rollback_count`, `register_orphan`, `recover_and_clean`, `drain_orphan_pins` und `drain_orphaned_checkpoints`.
4. **Cognition Consolidation & Session Maintenance (`cognition/consolidation_executor.rs`, `maintenance_scheduler.rs`, `context.rs`)**:
   `with_validator`, `with_rich_validator`, `start_worker`, `with_llm`, `with_leanrag`, `with_edge_reinforcement_buffer`, `increment_active_sessions`, `decrement_active_sessions` und `set_relevance_threshold`.

---

## 2. Architekturanalyse & Kontext

Dieser Scaffolding-Code verbindet Ring 1 (Checkpoint) und Ring 3 (Agent, Cognition, DB). Er ist Teil des in Teil 11 beschriebenen Resilienzmusters. Insbesondere stellt der CLM-Scratchpad (§11.1) ein deterministisches Mittel zur Kontextbereinigung dar, das unbegrenzten Prompt-Wachstum bei >100 Steps verhindert.

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige Integration des CLM Scratchpad & End-to-End Agent Replay
* **Beschreibung**:
  Integrieren des CLM-Scratchpads in den Standard-Orchestrator-Loop von `contextra-agent` sowie Anbindung der Orphan-Cleanup-Worker an den Systemstart.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. Validation an 500+ Step Agenten-Workflows, KV-Cache Branch Invalidation, Hardlink-Cloning Test-Suite)*.
* **Pro**:
  - Unterstützung für ultra-lange Agentenläufe ohne Context Window Overflow.
  - Vollständige Crash-Recovery über Checkpoint Guards.
* **Contra**:
  - Hohe Komplexität in der Zustandsverwaltung bei Multi-Agent-Forks.

### Option B: Rückbau des CLM Scratchpads & vereinfachtes Checkpointing
* **Beschreibung**:
  Entfernen des `clm_scratchpad.rs`-Moduls und Reduktion des Agenten-State-Graphs auf synchrone In-Memory-Ausführung.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Bereinigung in contextra-agent und contextra-db)*.
* **Pro**:
  - Vereinfachung des Agent-Engine-Codes.
* **Contra**:
  - Verlust der Fähigkeit zur kontrollierten Kontext-Ausdünnung.

### Option C: Beibehaltung des Ist-Zustands als Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Belassen der Module in `contextra-agent`, `contextra-checkpoint` und `contextra-cognition`.
* **Aufwandsschätzung**: **0 Stunden**
* **Pro**:
  - Sofort verfügbare Bausteine für komplexe Agenten-Orchestrierung.
  - Keine Laufzeitkosten bei Standard-Query-Nutzung.
* **Contra**:
  - Vorhandensein von ungenutzten Methoden im Agent-Subsystem.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Belassen des Scaffolding-Codes. Der CLM-Scratchpad ist isoliert und stört Standard-Retrieval nicht.

2. **Langfristig**:
   Validierung an realen Multi-Step Agenten-Workflows (Option A) gemäß Abnahmekriterium §11.1.
