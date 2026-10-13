# AUDIT-Bericht: Collection-Orchestrierung, Kognitions-Pipeline, Routing & Agenten

**Spezialist:** Spezialist für Collection-Orchestrierung, Kognitions-Pipeline, Routing und Agenten
**Commit:** `59f853cb`
**Datum:** 2026-10-13
**Vorgänger:**
- `docs/audits/AUDIT_contextra-engine_2026-10-12.md`
- `docs/audits/AUDIT_contextra-cognition_2026-10-11.md`
- `docs/audits/AUDIT_contextra-router_2026-10-12.md`
- `docs/audits/AUDIT_contextra-agent_2026-10-12.md`
- `docs/audits/AUDIT_contextra-db_2026-10-12.md`
**VERIFIED-BY-SESSION:** PENDING

---

## 1. Scope und Abdeckung

| Datei | Ring | Gelesen (Ja/Nein) | Begründung / Bemerkung |
|---|---|---|---|
| `crates/contextra-engine/src/lib.rs` | Ring 3 | Ja | Root, `#![forbid(unsafe_code)]`, Re-Exports & Configuration |
| `crates/contextra-engine/src/chunker.rs` | Ring 3 | Ja | MarkdownChunker Implementierung |
| `crates/contextra-engine/src/decay_controller.rs` | Ring 3 | Ja | AdaptiveDecayController |
| `crates/contextra-engine/src/export.rs` | Ring 3 | Ja | ExportDocumentV1 & Format-Export |
| `crates/contextra-engine/src/import.rs` | Ring 3 | Ja | ImportSummary & Format-Import |
| `crates/contextra-engine/src/filter.rs` | Ring 3 | Ja | MetadataFilter & Predicate Matching |
| `crates/contextra-engine/src/fusion.rs` | Ring 3 | Ja | Re-export & Integration mit contextra-rank |
| `crates/contextra-engine/src/limits.rs` | Ring 3 | Ja | Capacity limits & bounds |
| `crates/contextra-engine/src/merge_operator_adapter.rs` | Ring 3 | Ja | WASM Sandbox Merge Operator Adapter |
| `crates/contextra-engine/src/metrics_names.rs` | Ring 3 | Ja | Metric constants |
| `crates/contextra-engine/src/no_graph_stubs.rs` | Ring 3 | Ja | Fallback CsrGraph / HyperEdge Stubs (ohne `graph` Feature) |
| `crates/contextra-engine/src/temporal_filter.rs` | Ring 3 | Ja | ValidityWindow & Bitemporal Filters |
| `crates/contextra-engine/src/background_workers/*` | Ring 3 | Ja | Expiry, Orphan & Hyperedge background tasks |
| `crates/contextra-engine/src/collection/*` | Ring 3 | Ja | Collection CRUD, KV Locking, Maintenance, Query Builder, Search & Relate |
| `crates/contextra-engine/src/contextra_impl/*` | Ring 3 | Ja | Multitenant Collection lifecycle, wiring, diagnostics & CRUD |
| `crates/contextra-engine/src/deletion/*` | Ring 3 | Ja | Document erasure footprint, witnesses, purge driver & reverse sweep |
| `crates/contextra-engine/src/extraction/*` | Ring 3 | Ja | OpenIE & rule-based triple extraction |
| `crates/contextra-engine/src/kv_cache_integration/*` | Ring 3 | Ja | AccessCounterAttentionExporter & Eviction Worker wiring |
| `crates/contextra-engine/src/transaction/*` | Ring 3 | Nein (Exkludiert) | Laut Auftrag explizit vom Scope ausgeschlossen |
| `crates/contextra-cognition/src/lib.rs` | Ring 3 | Ja | Root, `#![forbid(unsafe_code)]` & Re-Exports |
| `crates/contextra-cognition/src/aggregation_phase.rs` | Ring 3 | Ja | Semantic Aggregation Pipeline |
| `crates/contextra-cognition/src/consolidation_executor.rs` | Ring 3 | Ja | Memory consolidation execution engine |
| `crates/contextra-cognition/src/consolidation_locks.rs` | Ring 3 | Ja | Key-locking strategy for memory consolidation |
| `crates/contextra-cognition/src/context.rs` | Ring 3 | Ja | ContextManager token budget allocation |
| `crates/contextra-cognition/src/graph_sink.rs` | Ring 3 | Ja | Knowledge Graph sync sink |
| `crates/contextra-cognition/src/leanrag_input.rs` | Ring 3 | Ja | LeanRAG input formatting |
| `crates/contextra-cognition/src/maintenance_config.rs` | Ring 3 | Ja | Maintenance scheduler configuration |
| `crates/contextra-cognition/src/maintenance_scheduler.rs` | Ring 3 | Ja | Background maintenance scheduler |
| `crates/contextra-cognition/src/memory_consolidation.rs` | Ring 3 | Ja | Memory consolidation passes |
| `crates/contextra-cognition/src/semantic_aggregation_facade.rs` | Ring 3 | Ja | Aggregation facade API |
| `crates/contextra-cognition/src/synthesis_phase.rs` | Ring 3 | Ja | Generative synthesis pass |
| `crates/contextra-cognition/src/transitivity_veto.rs` | Ring 3 | Ja | Transitivity Veto verification |
| `crates/contextra-cognition/src/context_compaction/*` | Ring 3 | Ja | Context compaction session, compactor, cleanup & types |
| `crates/contextra-router/src/lib.rs` | Ring 3 | Ja | Root, `#![forbid(unsafe_code)]` & Re-Exports |
| `crates/contextra-router/src/arm_registry.rs` | Ring 3 | Ja | Bandit Arm registry |
| `crates/contextra-router/src/dispatch.rs` | Ring 3 | Ja | SLM Dispatch & Route Execution |
| `crates/contextra-router/src/fc_ts_dispatch.rs` | Ring 3 | Ja | Flow-Corrected Thompson Sampling Dispatch |
| `crates/contextra-router/src/outcome.rs` | Ring 3 | Ja | Outcome feedback & Conformal Risk tracking |
| `crates/contextra-router/src/ports_local.rs` | Ring 3 | Ja | Local port traits & adapters |
| `crates/contextra-router/src/profile.rs` | Ring 3 | Ja | SLM Profile definitions & validation |
| `crates/contextra-router/src/routing_strategy.rs` | Ring 3 | Ja | Cascade & Ensemble Routing Strategies |
| `crates/contextra-router/src/serde_helpers.rs` | Ring 3 | Ja | Serialization helpers |
| `crates/contextra-router/src/transport.rs` | Ring 3 | Ja | Transport protocol abstractions |
| `crates/contextra-router/src/router/*` | Ring 3 | Ja | RouterEngine core, dispatch, outcomes & lifecycle |
| `crates/contextra-agent/src/lib.rs` | Ring 3 | Ja | Root, `#![forbid(unsafe_code)]` & Re-Exports |
| `crates/contextra-agent/src/audit.rs` | Ring 3 | Ja | AuditLog & AuditEntry persistence |
| `crates/contextra-agent/src/budget.rs` | Ring 3 | Ja | TokenBudget & Reservation RAII guards |
| `crates/contextra-agent/src/clm_scratchpad.rs` | Ring 3 | Ja | Scratchpad checkpointing |
| `crates/contextra-agent/src/context.rs` | Ring 3 | Ja | AgentContext, AgentStatus & telemetry deque |
| `crates/contextra-agent/src/dlq.rs` | Ring 3 | Ja | DeadLetterQueue persistence & metrics |
| `crates/contextra-agent/src/engine.rs` | Ring 3 | Ja | OrchestratorEngine event loop & tool execution |
| `crates/contextra-agent/src/event_source.rs` | Ring 3 | Ja | EventSource & PollingDocumentEventSource |
| `crates/contextra-agent/src/goal_condition.rs` | Ring 3 | Ja | GoalCondition evaluation |
| `crates/contextra-agent/src/graph.rs` | Ring 3 | Ja | StateGraph, AgentNode & WorkflowEdge |
| `crates/contextra-agent/src/step.rs` | Ring 3 | Ja | AgentTool, StepResult & StepDeadLetter |
| `crates/contextra-agent/src/engine/condition.rs` | Ring 3 | Ja | Execution condition evaluation |
| `crates/contextra-db/src/lib.rs` | Ring 3 | Ja | Strangler Shell / Re-export Facade |
| `crates/contextra-db/src/homeostat.rs` | Ring 3 | Ja | Rerank PID Controller wrapper |
| `crates/contextra-db/src/multistep.rs` | Ring 3 | Ja | MultiStepEngine & QueryRewriter |
| `crates/contextra-db/src/volatile_vault.rs` | Ring 3 | Ja | VolatileContextVault ephemeral RAM storage |

---

## 2. Inventar der pub-Items

### `contextra-engine` (Ring 3)
- `pub struct Contextra` (`src/lib.rs:88`): Haupt-Laufzeitumgebung und Multitenant Collection Registry.
- `pub struct Collection<S>` (`src/collection/mod.rs:56`): Thread-sichere Handle für Multi-Index Collection Operations.
- `pub struct HybridQueryBuilder<'a, S>` (`src/collection/query_builder/builder.rs:24`): Fluent Query Builder.
- `pub struct MarkdownChunker` (`src/chunker.rs:18`): Strukturierter Markdown Chunker.
- `pub struct AdaptiveDecayController` (`src/decay_controller.rs:14`): Time-weighted eviction controller.
- `pub struct DpiPurgeDriver<'a>` (`src/deletion/purge_driver.rs:38`): Erasure runtime driver (ORPHANED).
- `pub fn reverse_sweep<S>` (`src/deletion/reverse_sweep.rs:83`): Residual key reverse sweep (ORPHANED).
- `pub fn start_expiry_reaper` (`src/background_workers/expiry_workers.rs:67`): Background expiry reaper (ORPHANED).
- `pub fn start_orphan_reaper` (`src/background_workers/orphan_workers.rs:169`): Background orphan reaper (ORPHANED).
- `pub fn start_thermostat_reaper` (`src/background_workers/expiry_workers.rs:106`): Background thermostat reaper (ORPHANED).
- `pub fn start_kv_cache_eviction` (`src/kv_cache_integration/mod.rs:79`): KV cache eviction worker starter (ORPHANED).
- `pub type TenantCollectionMap` (`src/lib.rs:358`): Multitenant map alias (ORPHANED).

### `contextra-cognition` (Ring 3)
- `pub struct ContextManager` (`src/context.rs:25`): Cognition context manager for token budgeting.
- `pub struct MemoryConsolidationExecutor` (`src/consolidation_executor.rs:42`): Consolidation pipeline executor.
- `pub struct SemanticAggregationFacade` (`src/semantic_aggregation_facade.rs:18`): Aggregation facade.
- `pub struct ConsolidationPipelineResult` (`src/aggregation_phase.rs:120`): Consolidation result summary (ORPHANED).

### `contextra-router` (Ring 3)
- `pub struct RouterEngine` (`src/router/mod.rs:38`): Cascade & LinUCB SLM routing engine.
- `pub struct SlmProfile` (`src/profile.rs:15`): SLM Candidate profile with latency/cost/error bounds.
- `pub struct LyapunovDriftWatcher` (`src/router/mod.rs`): Distribution drift watcher for LinUCB adaptation.
- `pub fn route_and_dispatch_to_slm` (`src/dispatch.rs:171`): Routing and dispatch combined entrypoint (ORPHANED).

### `contextra-agent` (Ring 3)
- `pub struct OrchestratorEngine` (`src/engine.rs:38`): Deterministic event-loop orchestrator for agent tasks.
- `pub struct AgentContext` (`src/context.rs:18`): Agent state & telemetry container.
- `pub struct StateGraph` (`src/graph.rs:22`): Directed graph for agent workflow nodes & edges.
- `pub struct DeadLetterQueue` (`src/dlq.rs:32`): Disk-backed dead-letter queue for failed steps.
- `pub struct AuditLog` (`src/audit.rs:24`): Unalterable append-only LSM audit log.
- `pub struct AgentEngine` (`src/context.rs:223`): Deprecated wrapper struct (ORPHANED).

### `contextra-db` (Ring 3)
- `pub struct MultiStepEngine` (`src/multistep.rs:65`): Iterative LLM query rewriter & multi-round search engine.
- `pub struct VolatileContextVault` (`src/volatile_vault.rs:35`): Ephemeral in-memory RAM vault with mlock/zeroization.
- `pub struct CommitReceipt` (`src/volatile_vault.rs:104`): Ephemeral commit receipt (ORPHANED).

---

## 3. Antworten zu den fachlichen Prüffragen

1. **Pipeline-Verdrahtung & Reihenfolge:**
   Die Pipeline-Stufen existieren als eigenständige Bausteine (`Collection` für Retrieval/Fusion in `contextra-engine`, `MultiStepEngine` für Planner in `contextra-db`, `RouterEngine` für Routing/SLM-Dispatch in `contextra-router`, `EgressFilter` in `contextra-privacy`), sind jedoch **nicht** zu einer automatischen Gesamtkaskade im Produktionspfad von `contextra-engine` oder `contextra-db` verdrahtet. Innerhalb von `Collection::hybrid_search_with_query_and_report_at` ist die Ausführungsreihenfolge festgeschrieben: Vector-Suche -> Text-Suche (BM25) -> Graph-Traversierung (PPR) -> Hydratisierung aus Storage -> RRF/Score-Fusion.

2. **Stubs & Konstante Rückgaben im Collection-Pfad:**
   In `crates/contextra-engine/src/no_graph_stubs.rs` liefert `CsrGraph` (wenn das Feature `graph` inaktiv ist) triviale Stub-Antworten (`Ok(vec![])`, `Ok(0)`, `Ok(AHashMap::default())`). In `crates/contextra-engine/src/kv_cache_integration/mod.rs` nutzt `AccessCounterAttentionExporter` bei fehlenden ML-Model-Aufmerksamkeitsgewichten einfache Zugriffszähler als Fallback.

3. **Verdrahtung des Löschpfads:**
   `Collection::delete` (Tombstone in LSM + Löschung in HNSW/BM25/Graph), `Collection::delete_many`, `Collection::delete_prefix`, `Collection::forget_document` (vollständiger Erasure-Pfad mit Footprint & Layer Witnesses) und `Collection::drop_collection` (Prefix-Delete + Ed25519 `DeletionProof`) sind im Collection-Pfad verdrahtet.
   **Unverzweigte Löschfunktionen (Orphans):** Der Runtime Purge Driver `DpiPurgeDriver` (`crates/contextra-engine/src/deletion/purge_driver.rs`) sowie die Residual-Key-Sweep-Funktion `reverse_sweep` (`crates/contextra-engine/src/deletion/reverse_sweep.rs`) besitzen im Produktionscode von `contextra-engine` und `contextra-db` keinen Aufrufer.

4. **Fehlerweiterreichung & Retry-Idempotenz:**
   Fehler werden in `contextra-engine`, `contextra-router`, `contextra-agent` und `contextra-db` durchgehend als `Result<T, ContextraError>` weitergereicht. Einzige Ausnahme: In `contextra-cognition/src/synthesis_phase.rs:run_synthesis_pass` werden Fehler einzelner Segmente verworfen und lediglich als `skipped_segments` gezählt. Retries in `OrchestratorEngine` (`contextra-agent`) sind strikt auf Tool-Ausführungen beschränkt und durch `tool.max_retries()` und `MAX_WORKFLOW_STEPS = 10_000` geschützt.

5. **Agenten-Zustandsübergänge & Typestate:**
   In `contextra-agent/src/context.rs` ist das Feld `status` als `pub status: AgentStatus` auf `AgentContext` deklariert. Dadurch existiert keine typestate- oder matrixbasierte Kapselung: Externe Aufrufer können `ctx.status` direkt und ohne Prüfung durch `OrchestratorEngine` in einen beliebigen Status (z. B. `Completed` oder `Failed`) versetzen. Dies verletzt die AGENTS.md Invariante von `contextra-agent`.

6. **Trennung von Validitäts- und Systemzeit:**
   In `contextra-engine/src/temporal_filter.rs` sind Systemzeit (`TxId` Sequenzzähler) und Geschäftszeit (`query_timestamp` Unix-Timestamp) in `is_valid_bitemporal` getrennt. **Jedoch:** In `apply_temporal_validity_filter_at(results, as_of_timestamp: u64)` wird der übergebene Unix-Zeitstempel `as_of_timestamp` fälschlicherweise als `TxId::new(as_of_timestamp)` für die Systemzeit eingesetzt, wodurch System-Transaktions-IDs und Geschäfts-Zeitstempel miteinander vermengt werden.

7. **Locks über `.await`:**
   In allen Modulen im Scope wurden Asynchronitätsgrenzen geprüft. Es werden keine synchronen Mutex- oder RwLock-Guards über `.await`-Punkte gehalten. `Collection::consolidation_guard` und `stability_tracker` nutzen explizit `tokio::sync::Mutex`.

8. **Determinismus von Zeitstempeln und Rangfolge:**
   In `contextra-agent` und `contextra-db` werden Zeitstempel ausschließlich über den Port `Clock` bezogen (`now_unix_nanos()` / `monotonic_nanos()`). Es existiert kein Aufruf von `SystemTime::now()` im Produktionscode aller 5 Crates. `Instant::now()` wird in `volatile_vault.rs:243` ausschließlich für in-memory Quittungsmessungen verwendet.

---

## 4. Ergebnisse je Kategorie A bis M

### A. Unvollständige oder angefangene Implementierungen
- `TODO(contextra-plan)` Kommentare in `contextra-cognition/src/synthesis_phase.rs:44`, `memory_consolidation.rs:268, 410`, `lib.rs:2`, `contextra-db/src/homeostat.rs:12`, `contextra-db/src/lib.rs:8`.
- CsrGraph Stubs in `contextra-engine/src/no_graph_stubs.rs` liefern Dummy-Werte, wenn `feature = "graph"` inaktiv ist.

### B. Fehlerhafte Logik
- `apply_temporal_validity_filter_at` in `contextra-engine/src/temporal_filter.rs:118`: `as_of_timestamp` (i64 Business-Zeit) wird als `TxId::new(as_of_timestamp)` (u64 System-TxId) interpretiert.
- `drop_collection` in `contextra-engine/src/contextra_impl/collections.rs`: Enthält ein ungesperrtes Scan-Commit-Fenster während `delete_prefix`, wodurch parallele Schreibzugriffe entkommen können (INV-DELETION-1).

### C. Fehlende Verdrahtung
- `RouterEngine` in `contextra-router` ist nicht in die Abfragepfade von `contextra-engine` oder `contextra-db` eingebunden (`// TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task`).
- Deletion Purge Driver `DpiPurgeDriver` und `reverse_sweep` in `crates/contextra-engine/src/deletion/` besitzen keine Produktionsaufrufer.

### D. Dead Code / Ungenutzte pub-Items
- `AgentEngine` in `crates/contextra-agent/src/context.rs:223`.
- `CommitReceipt` in `crates/contextra-db/src/volatile_vault.rs:104`.
- `ConsolidationPipelineResult` in `crates/contextra-cognition/src/aggregation_phase.rs:120`.
- `ForgetOutcome` in `crates/contextra-engine/src/deletion/forget_options.rs:226`.
- `TenantCollectionMap` in `crates/contextra-engine/src/lib.rs:358`.

### E. Fehlerbehandlung
- `run_synthesis_pass` in `contextra-cognition/src/synthesis_phase.rs:60` verwirft Segmentfehler stumm.
- Upstream `contextra-store` Kompilierfehler (67 Fehler in `lsm/purge/orchestrator.rs` und `lsm/engine.rs`) blockiert die Testausführung von `contextra-engine`, `contextra-cognition`, `contextra-db` und `contextra-agent`.

### F. Panics und Unwraps im Produktionscode
- Geprüft gegen `.github/unwrap_baseline.txt`: Keine unerlaubten Unwraps oder Panics im Produktionscode der 5 Scope-Crates vorhanden.

### G. Nebenläufigkeit
- `AgentContext.status` ist ein öffentliches Feld (`pub status: AgentStatus`), wodurch unkontrollierte Statusänderungen ohne Validierung möglich sind.
- Sequentielle Signalabfrage (Vector -> BM25 -> Graph) in `Collection::hybrid_search_with_query_and_report_at` bremst die Retrieval-Latenz aus.

### H. Determinismus
- In allen 5 Crates wird für Transaktions- und Log-Zeitstempel ausschließlich der `Clock`-Port genutzt. `SystemTime::now()` wird im Produktionscode nirgends verwendet.

### I. Mandanten- und Berechtigungsgrenzen
- Mandanten-Isolierung wird über `TenantPolicy::Required` und `TenantIsolatedKvStore` durchgesetzt.

### J. Architektur und Design
- `#![forbid(unsafe_code)]` in allen 5 Crates im Scope eingehalten. `contextra-db` dient korrekt als Strangler Shell Re-Export-Fassade.

### K. Code-Smells
- Kaskadierendes Invalidation-Interface `cascade_invalidate_edges_for_superseded_doc` in `no_graph_stubs.rs` liefert stets `Ok(())`.

### L. Tests
- `cargo test -p contextra-router` kompiliert und läuft grün.
- Tests für `contextra-engine`, `contextra-cognition`, `contextra-agent` und `contextra-db` sind wegen des upstream `contextra-store` Kompilierfehlers derzeit nicht ausführbar.

### M. Dokumentationsdrift
- AGENTS.md von `contextra-agent` fordert strikte Kapselung von `AgentStatus`, der Code exponiert jedoch `pub status: AgentStatus` öffentlich.

---

## 5. Detaillierte Befunde

### FINDING-01: Systemzeit- und Geschäftszeit-Vermengung in `apply_temporal_validity_filter_at`
- **ID:** `ENG-FILTER-001`
- **Titel:** `apply_temporal_validity_filter_at` vermengt Unix-Zeitstempel mit TxId
- **Schwere:** Hoch
- **Kategorie:** B (Fehlerhafte Logik)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-engine/src/temporal_filter.rs:118`
- **Befund:** In `apply_temporal_validity_filter_at` wird `let as_of_tx = TxId::new(as_of_timestamp);` ausgeführt. Ein Unix-Zeitstempel (z. B. `1700000000`) wird dadurch als System-Transaktions-ID (`TxId`) interpretiert.
- **Evidenz:**
  ```rust
  pub fn apply_temporal_validity_filter_at(
      results: Vec<crate::SearchResult>,
      as_of_timestamp: u64,
  ) -> Vec<crate::SearchResult> {
      let as_of_tx = TxId::new(as_of_timestamp);
      results
          .into_iter()
          .filter(|res| {
              let window = extract_validity_window(res.metadata.as_ref());
              window.is_valid_bitemporal(as_of_tx, Some(as_of_timestamp))
          })
          .collect()
  }
  ```
- **Betroffene Invariante:** L.8 Bi-Temporaler Filter
- **Empfehlung:** `apply_temporal_validity_filter_at` muss `as_of_tx: TxId` und `as_of_business: Option<u64>` als getrennte Parameter entgegennehmen.
- **Red-Test-Vorschlag:** Erstelle ein Dokument mit `tx_valid_from: 5` und `valid_from: 1700000000`. Frage mit `as_of_timestamp = 1700000000` ab und verifiziere, dass es nicht fälschlich wegen `tx_valid_from` abgelehnt wird.
- **Owner:** Engine / Spec Team

---

### FINDING-02: Ungekapselte Status-Mutation in `AgentContext`
- **ID:** `AGT-STATUS-001`
- **Titel:** Öffentliches Feld `pub status: AgentStatus` ermöglicht illegale Zustandsübergänge
- **Schwere:** Hoch
- **Kategorie:** G (Nebenläufigkeit) / M (Dokumentationsdrift)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-agent/src/context.rs:20`
- **Befund:** `AgentContext` deklariert `pub status: AgentStatus`. Externe Aufrufer können ohne Prüfung durch die `OrchestratorEngine` beliebige Zustände setzen.
- **Evidenz:**
  ```rust
  pub struct AgentContext {
      pub task_id: String,
      pub status: AgentStatus,
  ```
- **Betroffene Invariante:** AGENTS.md Invariante "Strikte State-Machine Transitions" & "Manuelle Statusmanipulation verboten".
- **Empfehlung:** Ändere das Feld in `status: AgentStatus` (private) und biete Getter `pub fn status(&self) -> AgentStatus` sowie kontrollierte Setter-Methoden für die `OrchestratorEngine` an.
- **Red-Test-Vorschlag:** Versuche in einem Test `ctx.status = AgentStatus::Completed` außerhalb der OrchestratorEngine zu setzen (muss nach Sichtbarkeitseinschränkung fehlschlagen).
- **Owner:** Agent Team

---

### FINDING-03: Unverdrahtete Deletion-Komponenten `DpiPurgeDriver` und `reverse_sweep`
- **ID:** `ENG-DEL-001`
- **Titel:** `DpiPurgeDriver` und `reverse_sweep` haben keine Produktionsaufrufer
- **Schwere:** Mittel
- **Kategorie:** C (Fehlende Verdrahtung) / D (Dead Code)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-engine/src/deletion/purge_driver.rs:38` und `reverse_sweep.rs:83`
- **Befund:** `DpiPurgeDriver` und `reverse_sweep` sind im Modul `deletion` als `pub` definiert, werden aber im normalen Collection-Löschpfad oder Hintergrunddienst von `contextra-engine` nicht aufgerufen.
- **Evidenz:** Logdatei `logs/audits/collection-cognition-router-agent-db_2026-10-13/orphan_symbols.log` und Quelltext-Grep zeigen 0 Aufrufe im Produktionscode.
- **Betroffene Invariante:** INV-DELETION-WIRE-1
- **Empfehlung:** Verdrahte den `DpiPurgeDriver` im `forget_document` / `drop_collection` Pfad oder dokumentiere die geplante Facade-Anbindung.
- **Red-Test-Vorschlag:** Schreibe einen Integrationstest, der `DpiPurgeDriver::run_purge` nach einem Document Erasure Event ausführt und die Bereinigung der Residual-Keys verifiziert.
- **Owner:** Engine / Deletion Team

---

### FINDING-04: Stummes Verwerfen von Segment-Fehlern in `synthesis_phase.rs`
- **ID:** `COG-SYNTH-001`
- **Titel:** Einzelne Segment-Synthese-Fehler werden stumm verworfen
- **Schwere:** Niedrig
- **Kategorie:** E (Fehlerbehandlung)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-cognition/src/synthesis_phase.rs:60`
- **Befund:** Wenn `synthesizer.synthesize_segment` während des Generative Synthesis Pass fehlschlägt, wird der Fehler ignoriert und lediglich `skipped_segments` inkrementiert. Es erfolgt keine Protokollierung oder Fehlerübermittlung.
- **Evidenz:**
  ```rust
  let chunk = match synthesizer.synthesize_segment(segment, text_slice).await {
      Ok(c) => c,
      Err(_) => {
          skipped_segments += 1;
          continue;
      }
  };
  ```
- **Betroffene Invariante:** INV-FAIL-CLOSED / Error Observability
- **Empfehlung:** Binde Tracing-Logs ein (`tracing::warn!`) und nimm auf aufgetretene Fehler im `SegmentSynthesisResult` Bezug.
- **Red-Test-Vorschlag:** Erstelle einen Mock-Synthesizer, der bei Segment 2 einen Fehler zurückgibt, und prüfe, ob der Warn-Event geloggt wird.
- **Owner:** Cognition Team

---

## 6. Vorgänger-Abgleich

| Vorgänger-ID / Befund | Vorgänger-Bericht | Status im aktuellen Commit (`59f853cb`) | Bemerkung / Fundstelle |
|---|---|---|---|
| P1 Async Purity in Ring 0 | `AUDIT_contextra-engine_2026-10-12.md` | BEHOBEN | `cargo xtask check-ring0-async-purity` ergab 0 Verstöße |
| P2 Monotone TxId-Erzeugung | `AUDIT_contextra-engine_2026-10-12.md` / `AUDIT_contextra-db_2026-10-12.md` | BEHOBEN | `AtomicU64` mit `Ordering::SeqCst` in `Collection::allocate_tx()` |
| P3 Worker Shutdown | `AUDIT_contextra-engine_2026-10-12.md` | BEHOBEN | `CancellationToken` und `TaskTracker` sauber verdrahtet |
| P4 Sequentielle Hybrid-Suche | `AUDIT_contextra-db_2026-10-12.md` | OFFEN | Vector, BM25 und Graph laufen sequentiell in `hybrid/query.rs` |
| P5 DLQ Disk-Backed & Metrics | `AUDIT_contextra-agent_2026-10-12.md` | BEHOBEN | LSM-backed Key-Präfix `b"dlq:"` mit Telemetrie-Limits |

---

## 7. Testlage im Scope

- `crates/contextra-router`: `cargo test -p contextra-router` verläuft **100% grün** (34/34 passed).
- `crates/contextra-engine`, `crates/contextra-cognition`, `crates/contextra-agent`, `crates/contextra-db`: Die Ausführung von `cargo test` ist derzeit blockiert durch einen Kompilierfehler in der vorgelagerten Crate `contextra-store` (67 Fehler in `src/lsm/purge/orchestrator.rs` und `src/lsm/engine.rs`).

---

## 8. Nicht prüfbar und warum

- **E2E Integrationstests unter realer Last:** Die Ausführung der Integrationstests für `contextra-engine`, `contextra-cognition`, `contextra-agent` und `contextra-db` war im aktuellen Stand wegen des Kompilierfehlers in `contextra-store` nicht möglich.

---

## 9. Out-of-Scope-Befunde

- **`contextra-store` (Ring 1):** 67 Syntax- und Typfehler in `crates/contextra-store/src/lsm/purge/orchestrator.rs` und `crates/contextra-store/src/lsm/engine.rs` verhindern die Kompilierung der Bibliotheksziele für alle abhängigen Ring-3-Crates.

---

## 10. Priorisierte Maßnahmen

1. **Behebung des `contextra-store` Kompilierfehlers (Höchste Priorität):**
   - Befehl: `cargo check -p contextra-store`
2. **Korrektur der Bitemporal-Filter-Logik (`ENG-FILTER-001`):**
   - Befehl: `cargo test -p contextra-engine --lib temporal_filter`
3. **Kapselung von `AgentContext.status` (`AGT-STATUS-001`):**
   - Befehl: `cargo test -p contextra-agent --lib context`
4. **Verdrahtung oder Dokumentation von `DpiPurgeDriver` (`ENG-DEL-001`):**
   - Befehl: `cargo xtask orphan-symbols --json`
