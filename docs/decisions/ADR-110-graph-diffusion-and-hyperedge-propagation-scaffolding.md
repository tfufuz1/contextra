# ADR-110: Status und Architekturanalyse des CSR Graph Diffusion, Hyperedges & Session Propagation Scaffolding-Codes

* **Datum**: 2026-10-04
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-graph` (`apprh/mod.rs`, `apprh/selector.rs`, `tl_hfd/mod.rs`, `ppr/shadow_hook.rs`, `ppr/cost.rs`, `ppr_stream.rs`, `edge_reinforcement_buffer.rs`, `hyperedge.rs`, `provenance.rs`, `session_dag.rs`, `consistency_enforcement.rs`, `csr/graph_write.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_VOLLSTAENDIGE_FEATURE_SPEZIFIKATION.md` (§6.3, Y.1.2), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen des Auditings der öffentlichen API-Fläche wurden 36 Funktionen in `contextra-graph` identifiziert, die derzeit keine direkten Konsumenten aus höheren Ringen besitzen.

### Kernbefunde:
1. **Hypergraph Diffusion & Shadow Mode Comparison (`apprh/`, `tl_hfd/`, `ppr/shadow_hook.rs`, `ppr/cost.rs`, `ppr_stream.rs`)**:
   Funktionen wie `apprh_diffusion`, `apprh_shadow_compare`, `with_default_monitor`, `thresholded_local_hfd`, `log_tl_hfd_vs_baseline_discrepancy`, `recommended_multiplier` und `with_exclude_seeds` bilden das Gerüst für fortgeschrittene Hypergraph-Diffusions-Algorithmen (APPRH, TL-HFD).
   *Architektur-Bezug*: Gemäß §Y.1.2 und §6.3 müssen neue Diffusionsvarianten mindestens 10.000 Samples im ShadowMode absolvieren und ein Jaccard-Threshold ≥ 0.85 erreichen, bevor sie in Produktion befördert werden.
2. **Edge Reinforcement & Co-Occurrence Buffering (`edge_reinforcement_buffer.rs`)**:
   `push_cooccurrence`, `push_traversal`, `cooccurrence_count` und `traversal_count` dienen der Erfassung von Lerneffekten auf Graphkanten (Edge Reinforcement Learning).
3. **Hyperedges, Provenance & Kaskaden (`hyperedge.rs`, `provenance.rs`, `csr/graph_write.rs`)**:
   `slice_participants`, `get_or_intern`, `resolve_string`, `contains_role`, `contains_id`, `record_provenance`, `remove_doc`, `persist_hyperedge`, `insert_edge_direct_with_bitemporal_validity`, `with_storage`, `with_consistency_enforcer`, `pending_cascade_queue_len`, `process_cascade_queue`, `get_source_doc_id`, `insert_edge_direct` und `insert_edge_direct_with_validity`.
4. **Session DAG & Contradiction Detection (`session_dag.rs`, `consistency_enforcement.rs`)**:
   `append_step`, `set_active_head`, `children_of`, `detect_contradiction`, `is_suppressed`, `active_patterns`, `get_pattern` und `suggest_tombstone_candidates`.

---

## 2. Architekturanalyse & Kontext

`contextra-graph` ist ein Kern-Crate in Ring 0. Er stellt den Compressed Sparse Row (CSR) Wissensgraphen bereit. Die nicht aufgerufenen Methoden gehören zu Shadow-Mode-Vergleichsinfrastrukturen (APPRH vs. Forward-Push PPR) und asynchronen Kaskaden-Queues. Sie stellen sicher, dass Graph-Operationen die Invarianten P24 (Löschkosten proportional zur Nachbarschaft) und AGT-GRAPH-001 (Validierung von TxIds) einhalten.

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige Aktivierung von APPRH & Edge-Reinforcement im Produktionspfad
* **Beschreibung**:
  Beförderung von APPRH aus dem ShadowMode in den aktiven Retrieval-Hot-Path der Hybrid-Suche und direkte Anbindung von `EdgeReinforcementBuffer` an die Result-Processing-Schleife der Engine.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. Absolvierung der 10.000 Shadow-Mode Samples, Jaccard-Kalibrierung, Lock-Free Buffer-Flush und Latenz-Benchmarks)*.
* **Pro**:
  - Höhere Retrieval-Präzision bei n-ären Wissensrelationen und Hyperkanten.
  - Dynamisches Lernen von Aufrufpfaden durch Edge Reinforcement.
* **Contra**:
  - Höherer Rechenaufwand im Hot-Path.
  - Erfordert strenge Überwachung der Shadow-Gate-Invariante.

### Option B: Rückbau der unintegrierten Diffusions- & Reinforcement-Infrastruktur
* **Beschreibung**:
  Entfernen der Shadow-Compare-Module für APPRH/TL-HFD sowie des Edge-Reinforcement-Puffers.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Bereinigung von contextra-graph)*.
* **Pro**:
  - Reduktion des Crate-Umfangs in Ring 0 um ca. 1.500 LOC.
  - Vereinfachung der CSR-Graph-Struktur.
* **Contra**:
  - Verlust von SOTA Hypergraph-Retrieval-Technologien.

### Option C: Beibehaltung des Ist-Zustands als Shadow-Mode Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Belassen aller Funktionen in `contextra-graph`. APPRH verbleibt im ShadowMode zur fortlaufenden Evaluierung.
* **Aufwandsschätzung**: **0 Stunden**
* **Pro**:
  - Keine Beeinträchtigung des bestehenden PPR/Forward-Push Retrievals.
  - Erfüllung der Spezifikationsanforderungen bezüglich Shadow-Evaluation (Y.1.2).
* **Contra**:
  - Öffentliche Hilfsmethoden ohne direkte Aufrufer.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Beibehaltung der Methoden im Crate. Die Shadow-Compare-Hooks erzeugen keine Latenzkosten im Standard-PPR-Pfad.

2. **Langfristig**:
   Durchführung der verlangten 10.000 Shadow-Run Samples zur Validierung von APPRH (Option A).
