# ADR-109: Status und Architekturanalyse des Vector Index Rebuild, Repair & DiskANN Scaffolding-Codes

* **Datum**: 2026-10-04
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-vector` (`partial_rebuild.rs`, `hnsw/arena.rs`, `hnsw/core_insert.rs`, `hnsw/core_rebuild.rs`, `diskann/build.rs`, `persistence/mmap.rs`, `persistence/header.rs`, `hnsw/types.rs`, `acorn/naive_reference.rs`, `candidate_stream.rs`, `quantize/mod.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_VOLLSTAENDIGE_FEATURE_SPEZIFIKATION.md` (§6.1, VETO-F02), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen des Codebase-Auditing wurden 15 öffentliche Funktionen in `contextra-vector` identifiziert, die aktuell keine Aufrufer aus höheren Ringen oder Fassaden aufweisen.

### Kernbefunde:
1. **Partial Rebuild & Rewiring (`partial_rebuild.rs`, `core_rebuild.rs`)**:
   Funktionen wie `should_trigger_partial_rebuild` und `wait_for_rebuild` / `rebuild_status` dienen der Steuerung des partiellen Index-Rebuilds.
   *Architektur-Bezug*: VETO-F02 verbietet partielles HNSW-Rewiring im Standardbetrieb; zugelassen ist ausschließlich Tombstone-Pruning ohne Topologieänderungen.
2. **Arena & Low-Level Node Management (`hnsw/arena.rs`, `hnsw/core_insert.rs`)**:
   `allocate_node`, `free_node` und `compute_insert` bieten direkte Arena-Speicherverwaltung für HNSW-Knoten.
3. **DiskANN & Persistent Storage Scaffolding (`diskann/build.rs`, `persistence/mmap.rs`, `persistence/header.rs`)**:
   `recover_pending_delta`, `open_async`, `set_connections_offset` und `q_range` bilden die Infrastruktur für festplattenbasiertes Vector-Retrieval via DiskANN (§6.1, Feature `experimental-diskann`).
4. **Quantisierungs- & Reference-Utillities (`quantize/mod.rs`, `acorn/naive_reference.rs`, `candidate_stream.rs`, `hnsw/types.rs`)**:
   `train_with_percentiles`, `check_drift`, `from_vectors` (Naive Reference Index), `fetched_depth` und `set_layer_seed`.

---

## 2. Architekturanalyse & Kontext

`contextra-vector` ist in Ring 0 angesiedelt und stellt HNSW- sowie DiskANN-Vektorindizes bereit. Ein Großteil dieser Methoden bildet das Fundament für fortgeschrittene Indextransformationen (DiskANN Delta Recovery, Partial Rebuild, SQ8 Drift Monitoring). Partial Rebuild ist durch VETO-F02 reguliert und hinter dem Cargo-Feature `partial-index-rebuild` gesperrt.

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige DiskANN- & Partial-Rebuild-Integration
* **Beschreibung**:
  Freischaltung und Fertigstellung von DiskANN-Delta-Recovery sowie Einbindung von `should_trigger_partial_rebuild` unter Aufhebung/Anpassung von VETO-F02 nach formalem Security- & Recall-Audit.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. DiskANN Async IO Alignment, VETO-F02 Re-Evaluation, Recall-Benchmarks auf 1M+ Korpora)*.
* **Pro**:
  - Unterstützung von Vektor-Korpora >1 Mio. Dokumenten jenseits des RAM-Limits.
  - Inkrementelle HNSW-Reparatur ohne vollständigen Index-Rebuild.
* **Contra**:
  - Verletzung/Anpassung der VETO-F02 Invariante erforderlich.
  - Hohe Komplexität bei concurrent DiskANN Updates.

### Option B: Rückbau von DiskANN- und Partial-Rebuild-Scaffolding
* **Beschreibung**:
  Entfernen der DiskANN-Recovery-Logik und des `partial_rebuild.rs`-Moduls aus Ring 0.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Bereinigung von Feature-Flags und Cargo.toml)*.
* **Pro**:
  - Kompaktere Codebasis in Ring 0.
  - Exakte Ausrichtung am VETO-F02 Status Quo.
* **Contra**:
  - Verwerfen von fortgeschrittener DiskANN-Infrastruktur.

### Option C: Beibehaltung des Ist-Zustands als Opt-In Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Belassen der Funktionen in Ring 0. `partial-index-rebuild` bleibt standardmäßig deaktiviert (VETO-F02 konform).
* **Aufwandsschätzung**: **0 Stunden**
* **Pro**:
  - Keine Beeinträchtigung der Standard-HNSW-Suche.
  - Infrastruktur für künftige DiskANN-Erweiterungen bleibt erhalten.
* **Contra**:
  - Unaufgerufene Funktionen verbleiben im Crate.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Belassen der Funktionen im Crate. Beibehaltung der Sperre von `partial-index-rebuild` gemäß VETO-F02.

2. **Langfristig**:
   Evaluierung von Option A im Rahmen des DiskANN-Meilensteins für Korpora > 100k Vektoren.
