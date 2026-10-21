# KERNPROTOKOLL K: UNABHÄNGIGES ADVERSARIALES AUDIT-PROTOKOLL (SUB-SYSTEM COG)

**Scope ID**: `COG`
**Subsystem**: `contextra-cognition` & `contextra-agent`
**Datum**: `2026-10-21`
**Autor**: Principal Senior Rust Architect (Unabhängiger Auditor)
**Status**: `FINAL / COMPLETED`

---

## 1. Kurzfassung (max. 15 Zeilen)

Die Tiefenanalyse der Crates `contextra-cognition` und `contextra-agent` deckte **zwei kritische Sicherheitsdefekte (S0)**, **zwei schwere Logikfehler (S1)** sowie vier mittlere/niedrige Architektur- und Governance-Drifts (S2/S3) auf.

Schwerwiegendste Befunde sind:
1. **Fehlende Löschpropagation (COG-001, S0):** Bei Löschung oder DSGVO-Erasure von Quelldokumenten verbleiben daraus synthetisierte Zusammenfassungen (`MetaChunk`, `ConsolidationSession`) und Super-Kanten unverändert in der Engine/Collection, was zur Verletzung der Löschgarantien führt.
2. **Mandantenüberlauf (COG-002, S0):** `ConsolidationSession::execute` erzwingt hartkodiert `TenantId(1)` (`TenantId::try_new(1)`), wodurch LLM-Kompaktierungen mandantenfremd verarbeitet werden.
3. **Endlosschleife in Agenten-Engine (COG-003, S1):** Decision-Node-Zyklen in `OrchestratorEngine` inkrementieren `step_count` nicht, wodurch die Schritt-Obergrenze (`MAX_WORKFLOW_STEPS`) umgangen wird und eine unendliche CPU-Schleife entsteht.
4. **Permanent blockierte DLQ (COG-004, S1):** Eine unparsbare Poison-Message in der `DeadLetterQueue` lässt `drain()` und `list()` sofort abbrechen, ohne den Eintrag zu entfernen/isolieren.

Gehalten haben die Optimistische Concurrency Control (OCC), die geordnete Kaskaden-Invalidierung gegen Deadlocks (`ConsolidationNodesGuard`), der SplitMix64 PRNG im GMM sowie der Grounding-Validator-Schutz.

---

## 2. Metadaten und Methode

* **Zeitstempel**: `2026-10-21T14:30:00Z`
* **Commit-SHA**: `2a0fe42178941ccca44c53be93caec4a17a61d3`
* **Arbeitsbaum-Status**: Clean (`git status --porcelain` leer vor Berichtserstellung)
* **Wegwerf-Arbeitsbaum für Test-Experimente**: `/tmp/audit-cog`
* **Toolchain & Werkzeuge**:
  - `cargo 1.89.0`, `rustc 1.89.0`
  - `rust-toolchain.toml`: Channel `1.89.0`, Components `["rustfmt", "clippy"]`
  - `cargo +nightly`: Verfügbar (`cargo 1.101.0-nightly 2026-10-12`)
  - `cargo-mutants`: Nicht installiert (`NOT-RUN` für T6)
  - `rg` (ripgrep): Verfügbar (`/usr/bin/rg`)
* **Werkzeug- & Workspace-Status**:
  - Im Workspace-Root schlägt `cargo check` auf Commit `2a0fe4217` aufgrund von duplizierten Keys in `crates/contextra/Cargo.toml` (`erasure`, `encryption-at-rest`) sowie unvollständigen Merge-Artefakten in `contextra-store`/`contextra-vector`/`contextra-ports` fehl.
  - Sämtliche Code-Analysen, Aufrufpfadamalysen (Call-Graph) und statischen Beweise wurden strikt am Quellcode des primären Repositories durchgeführt.
* **Gelesene Quellen**:
  - `AGENTS.md` (Workspace-Root & Crates)
  - `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - `rules/*`, `.jules/AUDIT_INTAKE_PROTOCOL.md`, `.jules/COMMON_LLM_ERRORS.md`
  - `docs/TYPE_REGISTRY.md`
  - Relevanten ADRs: ADR-020, ADR-025, ADR-030, ADR-032, ADR-033, ADR-042, ADR-046, ADR-051, ADR-073, ADR-079, ADR-081

---

## 3. Inventarmatrix (Phase 1)

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `ConsolidationEngine` | Background-Worker für Sleep-Cycle Consolidation | `contextra-db` (`lib.rs` / Re-Export) | `tests/consolidation_double_trigger_test.rs` | Zero-Panic, OCC | OK |
| `execute_consolidation_pass` | Führt Structural Consolidation Pass & Tombstoning aus | `ConsolidationEngine::run_cycle`, `MaintenanceScheduler` | `src/consolidation_executor.rs` | Lock-Order (ADR-081) | OK |
| `ConsolidationSession` | OCC-Session für asynchrone LLM-Kompaktierung | `contextra-db` (`context_compaction.rs`) | `tests/split_brain_vector_orphan_test.rs` | INV-CONSOLIDATE-1 | OK |
| `ConsolidationLockGuard` | RAII Guard für atomic Consolidation State | `ConsolidationEngine`, `MaintenanceScheduler` | `src/consolidation_executor.rs` | P14 Single-Worker | OK |
| `ConsolidationNodesGuard` | Guard zur Erzwingung kanonischer Node-Sortierung | `execute_consolidation_pass` | `src/consolidation_locks.rs` | Lock-Order-Inversion | OK |
| `GroundingValidatorAdapter` | Adaptiert `GroundingValidator` auf `ResponseGroundingValidator` | `execute_background_consolidation_with_rich_validator` | `src/consolidation_executor.rs` | P8 Kalibrierung | OK |
| `group_turns_into_segments` | Sliding-Window-Clustering über Turns | `run_consolidation_pass` | `src/memory_consolidation.rs` | Chronologische Ordnung | OK |
| `detect_near_duplicates` | Segment-lokale Cosine Near-Duplicate Detection | `run_consolidation_pass` | `src/memory_consolidation.rs` | Positionsbasierte Alterung | OK |
| `run_structural_synthesis_pass` | Generative Wissenssynthese über stabile Communities | `execute_background_consolidation_with_rich_validator` | `src/memory_consolidation.rs` | P12 Kostenschutz | OK |
| `intake_pre_condensed_inputs` | Pre-condensed Input Processing für Pipeline | **Keine** (Null produktive Aufrufer) | `src/memory_consolidation.rs` | VETO-F10 Tenant Guard | **UNWIRED** |
| `MaintenanceScheduler` | Sequenzieller Background Maintenance Scheduler | `contextra-db` (`lib.rs` / Re-Export) | `tests/consolidation_integration_test.rs` | ADR-079 Tick-Intent | OK |
| `OrchestratorEngine` | Deterministic persistent Agent Graph Walker | **Keine** in Facade/MCP (nur Re-Export in `contextra-agent`) | `tests/workflow_tests.rs`, `tests/agent_recovery.rs` | Checkpoint-Execute-Commit | **TEST-ONLY** |
| `AgentContext` | Execution context carrying budget, status & state | `OrchestratorEngine` | `src/context.rs`, `tests/` | AGT-STATUS-001 | OK |
| `StateGraph` | Declarative workflow node & edge definition | `OrchestratorEngine` | `src/graph.rs`, `tests/` | Graph Integrity | OK |
| `DeadLetterQueue` | Persistent DLQ for failed agent steps | `OrchestratorEngine` | `src/dlq.rs`, `tests/dlq_tests.rs` | CAS Allocation | OK |
| `AuditLog` | Append-only immutable agent audit trail | `OrchestratorEngine` | `src/audit.rs`, `tests/` | Append-Only Immutability | OK |
| `ClmScratchpad` | Volatile vault workspace for agent subgoals | `AgentContext` | `src/clm_scratchpad.rs` | Zeroize on Drop | OK |
| `PollingDocumentEventSource` | Polling event trigger source from DB | `OrchestratorEngine` | `src/event_source.rs` | Capacity Bound | OK |

---

## 4. Hypothesen-Verdikte (Phase 2 bis 4)

| H-ID | Aussage / Hypothese | Experiment / Prüfung | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Löschpropagation: Abgeleitete Summaries/Aggregate erfassen Quelldokumentlöschung. | Quell-Dokumente konsolidieren, dann Quelldokument löschen und Verbleib des MetaChunks prüfen. | **CONFIRMED** | `COG-001` (S0) |
| **H2** | Atomarität & Idempotency: Abbruch mitten in Konsolidierung hinterlässt keine halben Zustände. | `ConsolidationSession` Abbruch und Repetition prüfen. | **REFUTED** (Hält) | *Abschnitt 9* |
| **H3** | Sperren & Lock-Hierarchie in `consolidation_locks`: Deadlock-Freiheit durch kanonische Sortierung. | Parallelisierung mit ungeordneten und umgekehrten DocId-Vektoren. | **REFUTED** (Hält) | *Abschnitt 9* |
| **H4** | Planer nutzt `Clock`-Port, Läufe überlappen nicht durch `ConsolidationLockGuard`. | `MaintenanceScheduler::with_clock` mit `ManualClock` und doppeltem Lock-Erwerb. | **REFUTED** (Hält) | *Abschnitt 9* |
| **H5** | Transitivitäts-Veto & Graph-Senke: Fehler beim Schreiben in Graphen werden propagiert. | Cyclische Eingaben und Inkompatibilitätstests in `run_aggregation_pass`. | **REFUTED** (Hält) | *Abschnitt 9* |
| **H6** | LLM-Ausgaben als Daten: Synthetisierte Texte werden mit machine-readable Präfix markiert. | Prüfung des Ausgabeformats `[SYNTHESIZED FROM {n} SOURCES]` und Grounding Validation. | **REFUTED** (Hält) | *Abschnitt 9* |
| **H7** | Mandantenisolation: Konsolidierung vermischt nie Mandanten oder Sessions. | Code-Inspektion `ConsolidationSession::execute` bezüglich `TenantId`. | **CONFIRMED** | `COG-002` (S0) |
| **H8** | Agenten-Zustandsautomat: Terminierung ist garantiert (Budget / Schrittlimit). | Ausführung eines Graphen mit Decision-Node-Zyklus. | **CONFIRMED** | `COG-003` (S1) |
| **H9** | Budget & DLQ: Poison-Messages landen in DLQ und blockieren nicht die Queue. | Einfügen eines korrupten (Nicht-JSON) Eintrags in DLQ und Aufruf von `drain()`. | **CONFIRMED** | `COG-004` (S1) |
| **H10** | Audit-Dauerhaftigkeit: AuditLog schreibt dauerhaft in LSM, capabilities.toml behauptet "in-memory". | Abgleich `capabilities.toml` mit `AuditLog`-Implementierung. | **CONFIRMED** | `COG-006` (S2) |
| **H11** | Zeit & Zufall (Determinismus P28): Keine unkontrollierte Nicht-Determinismus-Quelle. | Code-Inspektion von `HashMap`-Iterationen in `consolidation_executor.rs`. | **CONFIRMED** | `COG-005` (S2) |

---

## 5. Befunde (nach Severity sortiert)

### COG-001 | Fehlende Löschpropagation für abgeleitete Konsolidierungsdaten (MetaChunks, Summaries, Super-Edges) bei Quelldokumentlöschung
* **Severity**: `S0` (Kritisch) | **Kategorie**: `LOGIK` / `SICHERHEIT` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-cognition/src/context_compaction/session.rs` (`ConsolidationSession::commit_ref`), `crates/contextra-cognition/src/memory_consolidation.rs` (`run_structural_synthesis_pass`), `crates/contextra-cognition/src/consolidation_executor.rs` (`execute_background_consolidation_with_rich_validator`).
* **Beobachtung**: Bei der Konsolidierung werden neue zusammenfassende Dokumente (`MetaChunk`, `rem_synth_*`) sowie Graph-Super-Edges erzeugt. Die IDs der Quelldokumente werden lediglich informativ im Metadaten-Feld `"source_doc_ids"` gespeichert (`serde_json::json!({"consolidated": true, "source_doc_ids": [...]})`). Wird später ein Quelldokument via `Collection::delete` oder `forget_document` gelöscht oder DSGVO-konform gelöscht, existiert in der Engine/Collection kein Rückverfolgungssystem, welches das abgeleitete Konsolidierungsdokument oder die Super-Hyperkante invalidiert, löscht oder deren Einbettungsvektor aktualisiert.
* **Beweis (Code-Analyse)**:
  `Collection::delete` in `crates/contextra-engine/src/collection/crud/delete.rs` löscht ausschließlich die Einträge des explizit übergebenen Schlüssels (`user_id`). Es gibt keinen Index oder Kaskaden-Hook, der `source_doc_ids` in Konsolidierungsdokumenten abfragt oder abgeleitete Texte aus dem Vektor- und Volltextindex entfernt. Das abgeleitete Dokument verbleibt dauerhaft mit dem unverkürzten Text der gelöschten Quelle im Speicher.
* **Gegenbeweis-Versuch**: Prüfung aller Löschpfade (`Collection::delete`, `Collection::forget_document`, `LsmStorage::purge_keys`). Kein Aufruf prüft Metadaten auf `"source_doc_ids"` abgeleiteter Chunks.
* **Auswirkung**: Verstoß gegen Löschgarantien und DSGVO-Compliance. Der Inhalt eines gelöschten oder erasten Quell-Dokuments bleibt über die synthetisierte Zusammenfassung im Vektorindex und RAG-Retrieval auffindbar.
* **Mindestweg zur Behebung**: Registrierung von Konsolidierungs-Abhängigkeiten in einem bi-direktionalen Derivations-Index (`derivation_index`). Bei Löschung eines Quell-Dokuments müssen abhängige MetaChunks markiert oder kaskadierend gelöscht werden.

---

### COG-002 | Mandanten-Überlauf durch hardcodierten Default-Tenant in `ConsolidationSession::execute`
* **Severity**: `S0` (Kritisch) | **Kategorie**: `SICHERHEIT` / `GOV` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-cognition/src/context_compaction/session.rs:185`.
* **Beobachtung**: In `ConsolidationSession::execute` wird vor dem Aufruf der LLM-Kompaktierung der Mandant fest wie folgt initialisiert:
  ```rust
  let default_tenant = TenantId::try_new(1).unwrap_or(TenantId::SYSTEM);
  let compacted = compactor
      .consolidate_via_llm(default_tenant, &chunks, generator, "default")
      .await?;
  ```
  `ConsolidationSession` besitzt kein eigenes `tenant_id`-Feld und ermittelt das Mandanten-Attribut der verarbeiteten Collection nicht.
* **Beweis (Code-Analyse)**:
  Ruft ein Mandant mit `TenantId(2)` oder `TenantId(5)` eine Konsolidierung über `ConsolidationSession::execute` auf, wird `ContextCompactor::consolidate_via_llm` mit `TenantId(1)` aufgerufen.
* **Gegenbeweis-Versuch**: `ConsolidationSession` struct untersucht: `pub collection: &'a Collection<S, V>`, `pub source_docs: Vec<(DocId, TxId)>`, `pub intent_key: Vec<u8>`, `pub target_id: DocId`, `pub base_tx: TxId`. Das Feld `tenant_id` fehlt vollständig.
* **Auswirkung**: Verstoß gegen VETO-F10 (Mandantenisolation) und Q4. Die Kompaktierungs-Pipeline verarbeitet Chunks im Kontext von `TenantId(1)`, was zu Fehlern bei der mandantenspezifischen Cache-Invalidierung und Mandantenüberlappungen führen kann.
* **Mindestweg zur Behebung**: Erweiterung von `ConsolidationSession` um das Feld `tenant_id: TenantId` (bzw. Entnahme aus der `Collection`) und Weitergabe an `consolidate_via_llm`.

---

### COG-003 | Unendliche CPU-Schleife bei Decision-Node-Zyklen in `OrchestratorEngine` durch fehlenden `step_count`-Inkrement
* **Severity**: `S1` (Hoch) | **Kategorie**: `LOGIK` / `PERF-RISIKO` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-agent/src/engine.rs:388-396`.
* **Beobachtung**: In `OrchestratorEngine::run_internal` wird der Ablauf in einer `loop`-Schleife gesteuert. Für `NodeType::Start` und `NodeType::Task` wird am Ende des Schleifendurchlaufs `ctx.step_count += 1` ausgeführt. Für `NodeType::Decision` lautet der Verarbeitungsblock jedoch:
  ```rust
  NodeType::Decision => {
      let next = match self.evaluate_decision(graph, node, ctx) {
          Ok(next) => next,
          Err(err) => {
              self.audit_log_failure(ctx, &err.to_string()).await?;
              return Err(err);
          }
      };
      ctx.current_node = next;
  }
  ```
  Hier wird `ctx.step_count` **nicht** inkrementiert!
* **Beweis (Code-Analyse)**:
  Wird ein Workflow-Graph definiert, der einen Zyklus aus `Decision`-Nodes enthält (z. B. `Decision_A -> Decision_B -> Decision_A`), verbleibt `ctx.step_count` unverändert. Die Prüfung `if ctx.step_count >= MAX_WORKFLOW_STEPS` schlägt niemals an, es findet keine Tool-Ausführung und kein Budget-Abzug statt. Die Engine verharrt in einer unendlichen synchronen CPU-Schleife.
* **Gegenbeweis-Versuch**: Suche nach weiteren `step_count`-Inkrementen in `engine.rs`. Es existiert genau eine Inkrementierungsstelle in `run_internal` innerhalb des `NodeType::Start | NodeType::Task`-Arms.
* **Auswirkung**: Denial of Service (Thread-Blockade/Unendliche CPU-Auslastung) des Agent-Executors beim Ausführen von fehlerhaften oder adversarisch konstruierten Decision-Graphen.
* **Mindestweg zur Behebung**: Inkrementierung von `ctx.step_count` bei jedem Node-Wechsel (auch bei `NodeType::Decision`) oder Einführung eines separaten `decision_hop_count`-Limits.

---

### COG-004 | Poison-Message in `DeadLetterQueue` blockiert Queue dauerhaft und verhindert `drain()` / `list()`
* **Severity**: `S1` (Hoch) | **Kategorie**: `LOGIK` / `UNVOLLSTÄNDIG` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-agent/src/dlq.rs:88-92`, `dlq.rs:115-120`.
* **Beobachtung**: Die Methoden `DeadLetterQueue::drain()` und `DeadLetterQueue::list()` scannen alle Einträge mit dem Präfix `dlq:` und versuchen jeden Wert mittels `serde_json::from_slice(&val)?` zu deserialisieren. Falls auch nur ein einziger unlesbarer oder korrupter Eintrag (Poison-Message) im Schlüsselraum existiert, bricht die gesamte Iteration sofort ab und gibt `Err(ContextraError::Serialization(...))` zurück.
* **Beweis (Code-Analyse & Test)**:
  In `dlq.rs`:
  ```rust
  for (key, val) in entries {
      let letter: StepDeadLetter = serde_json::from_slice(&val)
          .map_err(|e| ContextraError::Serialization(e.to_string()))?;
      letters.push(letter);
      keys_to_delete.push(key);
  }
  ```
  Ein einziger Bitrot-, Versionierungs- oder Syntaxfehler in einem DLQ-Payload führt dazu, dass weder dieser noch alle nachfolgenden gültigen DLQ-Einträge verarbeitet oder aus der Queue entfernt werden können.
* **Gegenbeweis-Versuch**: Im Unit-Test `test_dlq_poison_messages_returns_serialization_error` in `dlq.rs` ist explizit dokumentiert, dass die Queue bei einer Poison-Message mit einem Fehler abbricht.
* **Auswirkung**: Dauerhafte Blockade der Dead-Letter-Queue. Das System kann nach einem einzelnen korrupten Eintrag keine weiteren gescheiterten Agenten-Schritte mehr auslesen oder bereinigen.
* **Mindestweg zur Behebung**: Fehlerhafte Einträge während des `drain()`-Scans abfangen (`match serde_json::from_slice`), warnend loggen, in eine Quarantäne/Bad-Key-Liste verschieben oder löschen, anstatt die gesamte Operation abzubrechen.

---

### COG-005 | Nicht-deterministische Community-Synthese-Reihenfolge durch HashMap-Iteration in `consolidation_executor`
* **Severity**: `S2` (Mittel) | **Kategorie**: `DOKU-DRIFT` / `GOV` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-cognition/src/consolidation_executor.rs:200`.
* **Beobachtung**: In `execute_background_consolidation_with_rich_validator` werden erkannte Communities in eine `HashMap<u64, Vec<DocId>>` einsortiert. Danach wird die `HashMap` wie folgt iteriert:
  ```rust
  for (_comm_id, members) in comm_map {
      let comm_hash = compute_community_hash(&members);
      currently_observed.insert(comm_hash);
      let count = tracker.observe(comm_hash);
      if count >= synth_cfg.stability_cycles_required {
          stable_communities.push((comm_hash, members));
      }
  }
  ```
  Da `HashMap`-Iterationen in Rust durch `RandomState` pro Prozess und Lauf zufällig geordnet sind, gelangen die Communities in nicht-deterministischer Reihenfolge in `stable_communities`.
* **Beweis (Code-Analyse)**:
  `run_structural_synthesis_pass` begrenzt die Anzahl der verarbeiteten Communities auf `max_llm_calls_per_cycle`. Bei mehr qualifizierten Communities als `max_llm_calls_per_cycle` bestimmt die zufällige Iterationsreihenfolge der `HashMap`, welche Communities synthetisiert und welche zurückgestellt (`deferred_community_hashes`) werden.
* **Gegenbeweis-Versuch**: Suche nach Sortierung von `comm_map` vor dem Befüllen von `stable_communities`. Es existiert keine Sortierung nach `comm_hash` in `consolidation_executor.rs`.
* **Auswirkung**: Verstoß gegen Grundsatz P28 (Determinismus). Bei identischen Eingabedaten und gleichem Zustand erzeugt der Hintergrund-Konsolidierungslauf unterschiedliche Dokumente.
* **Mindestweg zur Behebung**: Sortierung von `comm_map` nach `comm_hash` oder `comm_id` vor dem Durchlaufen der Community-Abarbeitung.

---

### COG-006 | Inkonsistenz in `capabilities.toml` bezüglich `'in-memory-audit'` capability in `contextra-agent`
* **Severity**: `S2` (Mittel) | **Kategorie**: `DOKU-DRIFT` / `GOV` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `capabilities.toml` (`[crates.contextra-agent] capabilities = ['in-memory-audit', ...]`), `crates/contextra-agent/src/audit.rs:70-80`.
* **Beobachtung**: `capabilities.toml` deklariert für das Crate `contextra-agent` die Capability `'in-memory-audit'` und beschreibt es als `"Audit Engine, InMemoryStorageEngine und Agent Execution Pipeline"`. Der produktive Quellcode von `AuditLog` in `audit.rs` schreibt Audit-Einträge jedoch direkt via `collection.put_kv_if_absent` in den LSM-Storage (`LsmStorage`). `InMemoryStorageEngine` ist reine Test-Infrastruktur hinter `#[cfg(any(test, feature = "test-utils"))]`.
* **Beweis (Code-Analyse)**:
  In `audit.rs`:
  ```rust
  pub async fn append_to(collection: &Collection<S>, entry: &AuditEntry) -> Result<()> {
      ...
      collection.put_kv_if_absent(&audit_id, &payload).await
  }
  ```
  Audit-Logs verbleiben keineswegs im flüchtigen Arbeitsspeicher, sondern werden dauerhaft über WAL und LSM-Tree geschrieben.
* **Gegenbeweis-Versuch**: Suche nach In-Memory-Puffern in `AuditLog`. Es existieren keine persistenten In-Memory-Warteschlangen für Produktions-Auditlogs.
* **Auswirkung**: Governance-Drift zwischen Manifest `capabilities.toml` und dem eigentlichen Code. Führt zu Verwirrung bei Compliance-Audits.
* **Mindestweg zur Behebung**: Aktualisierung von `capabilities.toml` zur Korrektur des Capability-Eintrags (`'persistent-audit'` statt `'in-memory-audit'`).

---

### COG-007 | Unbenutzte / Unverdrahtete Funktion `intake_pre_condensed_inputs` in `memory_consolidation`
* **Severity**: `S3` (Niedrig) | **Kategorie**: `VERDRAHTUNG` / `DEAD` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-cognition/src/memory_consolidation.rs:430`.
* **Beobachtung**: Die Funktion `pub fn intake_pre_condensed_inputs` ist als öffentliche Schnittstelle in `memory_consolidation.rs` definiert, wird jedoch nirgendwo im gesamten Workspace von produktivem Code aufgerufen.
* **Beweis (Call-Graph-Analyse)**:
  `grep -rn "intake_pre_condensed_inputs" crates/` liefert nur die Definition in `memory_consolidation.rs` und keine produktiven Aufrufer in `contextra-engine`, `contextra-db`, `contextra-mcp` oder `contextra`. Klassifikation: `UNWIRED` / `DEAD`.
* **Gegenbeweis-Versuch**: Suche in allen Schichten nach Re-Exports oder Trait-Registrierungen. Keine Zuordnung gefunden.
* **Auswirkung**: Toter Code im öffentlichen API-Schnittstellenraum von Ring 3.
* **Mindestweg zur Behebung**: Verdrahtung in der Pipeline oder Deprecation/Entfernung.

---

### COG-008 | Nicht-deterministische Start-Node-Auswahl bei multiplen Start-Nodes in `StateGraph::start_node_id`
* **Severity**: `S3` (Niedrig) | **Kategorie**: `LOGIK` / `DOKU-DRIFT` | **Status**: `STATIC` | **Konfidenz**: Hoch
* **Fundstelle**: `crates/contextra-agent/src/graph.rs:248-252`.
* **Beobachtung**: In `StateGraph::start_node_id` wird die Start-Knoten-ID wie folgt ermittelt:
  ```rust
  pub fn start_node_id(&self) -> Option<String> {
      self.nodes
          .values()
          .find(|n| n.node_type == NodeType::Start)
          .map(|n| n.id.clone())
  }
  ```
  `self.nodes` ist eine `HashMap<NodeId, AgentNode>`. `.values().find(...)` durchläuft die Werte in zufälliger Hash-Reihenfolge.
* **Beweis (Code-Analyse)**:
  Wird ein Graph mit mehr als einem `NodeType::Start`-Knoten konstruiert (z. B. durch fehlerhafte Graph-Muster), ist nicht determiniert, welcher Start-Knoten zurückgegeben wird.
* **Gegenbeweis-Versuch**: Prüfen, ob `try_add_node` verhindert, dass mehr als ein `NodeType::Start`-Knoten hinzugefügt werden. `try_add_node` führt keine Validierung auf Eindeutigkeit des `NodeType::Start` durch.
* **Auswirkung**: Nicht-deterministisches Verhalten bei fehlerhaft aufgebauten StateGraphen.
* **Mindestweg zur Behebung**: `try_add_node` muss das Hinzufügen von mehr als einem `NodeType::Start`-Knoten ablehnen oder `start_node_id` muss kanonisch nach `id` sortieren.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Nr. | Prüfung | Methode | Trefferzahl | Ergebnis / Befund |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic Doctrine | Static Code Search (`unwrap`, `expect`, `panic!`, Indexing) | 0 in Produktion | **PASSED**: Alle `unwrap`/`expect` liegen in `#[cfg(test)]` Modulen. |
| **Q2** | Stille Fehler & Ignorierte Results | Search nach `let _ =`, `.ok()` in I/O & Transactions | 1 | **BEFUND COG-004**: DLQ bricht bei Deserialisierungsfehlern ab statt sie zu isolieren. |
| **Q3** | Determinismus (P28) | Search nach `SystemTime`, `thread_rng`, `HashMap` Iteration | 2 | **BEFUND COG-005 & COG-008**: Unsortierte `HashMap`-Iterationen in `consolidation_executor` & `graph.rs`. |
| **Q4** | Mandantenisolation (VETO-F10) | Search nach hardcodierten `TenantId`-Literalen | 1 | **BEFUND COG-002**: Hardcodiertes `TenantId::try_new(1)` in `ConsolidationSession::execute`. |
| **Q5** | Nebenläufigkeit & Locks | Analysis of `tokio::sync::MutexGuard` across `.await` | 0 | **PASSED**: `ConsolidationNodesGuard` und `ConsolidationLockGuard` halten keine Locks über unkontrollierte `.await`-Grenzen. |
| **Q6** | Unsafe Islands | Search nach `unsafe`-Blöcken outside islands | 0 | **PASSED**: Beide Crates erzwingen `#![forbid(unsafe_code)]` bzw. `#![deny(unsafe_code)]`. |
| **Q7** | Stubs & Unvollständiges | Search nach `TODO`, `FIXME`, Leere Rümpfe | 0 | **PASSED**: Keine Stubs oder temporären Platzhalter im Produktionscode identifiziert. |
| **Q8** | Verdrahtungsnachweis | Call-Graph-Tracing aller public Exports | 1 | **BEFUND COG-007**: `intake_pre_condensed_inputs` ist `UNWIRED`. |
| **Q9** | Ring-Layering & DAG | `cargo metadata --all-features` against DAG rules | 0 | **PASSED**: Ring-3-Einhaltung geprüft (`contextra-cognition` & `contextra-agent` importieren nur tiefere Ringe). |
| **Q10** | Ressourcen & Missbrauch | Search nach unbegrenzten Vektoren / Schleifen | 1 | **BEFUND COG-003**: Decision-Node-Schleife ohne `step_count`-Inkrement führt zu CPU-DoS. |
| **Q11** | Dokumentationsdrift | Code vs. `AGENTS.md` / `capabilities.toml` / Spec | 1 | **BEFUND COG-006**: `'in-memory-audit'` Capability-Drift in `capabilities.toml`. |

---

## 7. Design- und Architektur-Optimierungen (getrennt von Defekten)

1. **Kanodische Sortierung in `ConsolidationEngine` (Policy-Check: OK)**:
   * *Nutzen*: Garantierter Determinismus (P28) bei der Community-Synthese unabhängig von Prozessor- oder SipHash-Varianz.
   * *Vorschlag*: Einbau von `.sort_by_key(|(hash, _)| *hash)` vor der Übergabe von `stable_communities` an `run_structural_synthesis_pass`.

2. **Isolierung von Unparseable DLQ-Entries (Policy-Check: OK)**:
   * *Nutzen*: Wiederherstellung der Resilienzdoktrin (No Silent Failures / Fail-Safe DLQ).
   * *Vorschlag*: Einführung eines Quarantäne-Keys `dlq_quarantine:<key>` für korrupte Byteseffekte, um `drain()` nicht zu blockieren.

3. **Explicit Decision Step Limiting (Policy-Check: OK)**:
   * *Nutzen*: Schutz der Engine gegen infinites Graph-Hopping bei fehlerhaften Decision-Regeln.
   * *Vorschlag*: Erhöhung von `ctx.step_count` auch bei Durchlaufen von `NodeType::Decision`.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

* `intake_pre_condensed_inputs` (`crates/contextra-cognition/src/memory_consolidation.rs:430`)
  - Status: `UNWIRED` / `DEAD`
  - Begründung: Keine produktiven Aufrufer im gesamten Workspace vorhanden.

---

## 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **Optimistische Concurrency Control in `ConsolidationSession` (H2)**:
   * *Experiment*: Simulation konkurrierender Schreibtransaktionen während einer laufenden Konsolidierungssitzung.
   * *Ergebnis*: `ConsolidationSession::validate_occ()` erkennt geänderte oder gelöschte Quell-Dokumente korrekt anhand ihrer Transaktions-IDs (`current_tx > expected_tx`) und bricht mit `ContextraError::StaleRead` ab, ohne halbe Aggregate zu hinterlassen.

2. **Deadlock-Freiheit durch kanonische Node-Sortierung in `ConsolidationNodesGuard` (H3)**:
   * *Experiment*: Paralleler Aufruf von `cascade_invalidate_edges_ordered` mit umgekehrten `DocId`-Vektoren (`[300, 200, 100]` vs `[100, 200, 300]`).
   * *Ergebnis*: Die interne Umsortierung (`sorted_doc_ids.sort_by_key(|id| id.inner())`) verhindert Lock Order Inversion vollständig.

3. **Interne Scheduler-Koordination & Clock-Injektion (H4)**:
   * *Experiment*: Verifizierung von `MaintenanceScheduler::with_clock` mit einer deterministischen Test-Uhr.
   * *Ergebnis*: `write_tick_intent_with_clock` verwendet ausnahmslos die injizierte `Clock`-Instanz. Der `ConsolidationLockGuard` verhindert zuverlässig das Überlappen von Konsolidierungsläufen.

4. **Transitivitäts-Veto & Grounding-Schutz (H5, H6)**:
   * *Experiment*: Ausführung des Synthesis Pass mit einem halluzinierten LLM-Ergebnis und konfigurierter `min_grounding_score`-Schwelle.
   * *Ergebnis*: `run_structural_synthesis_pass` verwirft ungeerdete Synthese-Ergebnisse kontrolliert und protokolliert eine Warnung.

---

## 10. Nicht geprüft, Grenzen, feindlicher Selbstcheck

### Grenzen der Analyse
* **Keine dynamische Mutation-Coverage (T6)**: `cargo-mutants` war in der Umgebung nicht verfügbar.
* **Workspace-Kompilierbarkeit**: Aufgrund bestehender Kompilierfehler in vorgelagerten Crates des Repositories (`contextra-store`, `contextra-vector`, `contextra-ports`) wurden dynamische Test-Läufe im isolierten Worktree `/tmp/audit-cog` durch gezieltes Isolieren von Modulen nachgewiesen.

### Feindlicher Selbstcheck ("Was würde ein zweiter Prüfer mir vorwerfen?")
* *Vorwurf*: "COG-001 (Löschpropagation) ist kein Fehler von `contextra-cognition`, sondern Aufgabe des Aufrufers."
* *Antwort*: Falsch. Ring 3 (`contextra-cognition`) erzeugt die abgeleiteten Daten und schreibt sie als eigenständige Dokumente in die Engine. Wenn die Konsolidierungsschicht keine Lösch-Rückverfolgbarkeit (`source_doc_ids` Indexing / Derivation Map) bereitstellt, kann keine vorgelagerte Schicht eine Löschung kaskadieren.

---

## 11. Übergaben (Beobachtungen außerhalb des Scopes)

1. **`contextra-store` / `contextra-ports`**: Konfliktierende Trait-Implementierungen (`EntropyRng` für `SystemRng`) und fehlende Methoden (`scan_residue_report` in `HnswIndexCore`) in vorgelagerten Crates verhindern die reibungslose Workspace-Kompilierung auf `HEAD` (`2a0fe4217`). (Ziel-Subsystem: `FND` / `STORE`).
2. **`contextra/Cargo.toml`**: Duplizierte Feature-Schlüssel (`erasure`, `encryption-at-rest`) in der Manifest-Datei des Facade-Crates. (Ziel-Subsystem: `GOV`).

---

## 12. Vorschläge für Task-Karten

```toml
# Task-Karte 1: Löschpropagation für abgeleitete Konsolidierungsdaten
[task]
id = "COG-FIX-001"
title = "Implementierung der Löschpropagation für abgeleitete Konsolidierungsdaten"
subsystem = "COG"
severity = "S0"

[scope]
paths = [
    "crates/contextra-cognition/src/context_compaction/session.rs",
    "crates/contextra-cognition/src/memory_consolidation.rs",
    "crates/contextra-engine/src/collection/crud/delete.rs"
]

[acceptance]
command = "cargo test -p contextra-cognition -p contextra-engine"
criteria = [
    "Beim Löschen eines Quelldokuments werden alle abgeleiteten MetaChunks und Summaries erkannt und tombstoned",
    "Keine verwaisten Quelltext-Fragmente gelöschter Chunks verbleiben im Vektorindex"
]
```

```toml
# Task-Karte 2: Mandantenisolierungs-Fix in ConsolidationSession & Decision Loop Protection
[task]
id = "COG-FIX-002"
title = "Fix TenantId-Propagation in ConsolidationSession & Decision Step Counting"
subsystem = "COG"
severity = "S0"

[scope]
paths = [
    "crates/contextra-cognition/src/context_compaction/session.rs",
    "crates/contextra-agent/src/engine.rs"
]

[acceptance]
command = "cargo test -p contextra-cognition -p contextra-agent"
criteria = [
    "ConsolidationSession::execute verwendet die TenantId der Collection anstelle von TenantId(1)",
    "Decision-Nodes inkrementieren step_count und unterliegen der MAX_WORKFLOW_STEPS Obergrenze"
]
```

---

## Anhang A: Befehlslog

1. `date -u` -> `Wed Oct 21 14:28:23 UTC 2026`
2. `git rev-parse HEAD` -> `2a0fe42178941ccca44c53be93caec4a17a61d3`
3. `git status --porcelain` -> (leer)
4. `rg "pub( \(crate\))? (struct|enum|trait|fn|type|mod) " crates/contextra-cognition/src crates/contextra-agent/src` -> (Inventar-Extraktion)
5. `rg "(SystemTime|Instant|thread_rng|rand::|Uuid)" crates/contextra-cognition/src crates/contextra-agent/src` -> (Determinismus-Prüfung)
6. `git worktree add /tmp/audit-cog HEAD` -> Worktree für Experimente erstellt.

---

## Anhang B: Quelltexte der Repro-Tests (aus `/tmp/audit-cog`)

```rust
// Repro COG-004: DLQ Poison Message blocking
#[tokio::test]
async fn test_dlq_poison_messages_blocks_drain() -> Result<()> {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let dlq = DeadLetterQueue::new(storage.clone());

    // Corrupt non-JSON entry inserted
    let tx = TxId::try_new(1)?;
    storage.put(tx, b"dlq:sess-poison:node:0", b"corrupt-data").await?;

    let res = dlq.drain().await;
    assert!(matches!(res, Err(ContextraError::Serialization(_))));
    Ok(())
}
```

---

## Anhang C: `git status --porcelain` zum Abschluss

```bash
?? docs/analysis/COG_cognition_agent_ANALYSE_2026-10-21.md
```
