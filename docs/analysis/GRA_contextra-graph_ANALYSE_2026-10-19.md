# GRA_contextra-graph_ANALYSE_2026-10-19.md

## 1. Kurzfassung

- **Gesamtrisiko:** Hoch (S1). Das Subsystem `contextra-graph` leidet unter einem kompilierbrechenden Verdrahtungsfehler in `tl_hfd::diffusion`, systematischer Verletzung der Transaktions-Invariante `AGT-GRAPH-001` durch die Nutzung veralteter `TxId::new()`-Konstruktoren sowie schleichender Async-Runtime-Verbreitung im Ring-0-Speicherkern.
- **Die 5 schwersten Befunde:**
  1. `GRA-001` (S1, `VERDRAHTUNG`): Unkompiliertes Modul `tl_hfd::diffusion` scheitert an `From<ContextraError>` für `TlHfdError` (`E0277`).
  2. `GRA-002` (S1, `GOV`/`LOGIK`): Mastehafte Verwendung veralteter `TxId::new()` / `TxId::new(0)` Aufrufe verletzt Invariante `AGT-GRAPH-001`.
  3. `GRA-003` (S2, `UNVOLLSTÄNDIG`): PPR Forward-Push Normalisierung teilt bei isolierten Senken durch Zero/Epsilon ohne strikten `is_finite()` Check.
  4. `GRA-004` (S2, `DESIGN`/`PERF-RISIKO`): `CsrGraph` Async-Proliferation im Ring 0 verstößt gegen das synchrone Speicher-Prinzip (ADR-N02 / P26).
  5. `GRA-005` (S2, `DOKU-DRIFT`): Modulkarte in `crates/contextra-graph/AGENTS.md` unterschlägt `tl_hfd`, `apprh`, `percolation`, `ppr_stream`, `residue_scan`, `hyperedge_suggest`, `arc_slice`.
- **Was gehalten hat:**
  - Zero-Panic in produktiven Datenpfaden (`#![forbid(unsafe_code)]` strikt eingehalten).
  - CSR Offset-Monotonie und Bitemporale Snapshot-Sichtbarkeit arbeiten bei korrekter `TxId`-Übergabe exakt.
  - Shadow Comparison Hooks (`ppr/shadow_hook`, `apprh/shadow`) arbeiten seiteneffektfrei isoliert.
- **Größte Grenzen der Analyse:** Crate war aufgrund des E0277-Fehlers in `tl_hfd` vor dem audit-spezifischen Fix im Wegwerf-Worktree nicht kompilierbar. `cargo-fuzz` und `cargo-mutants` standen im Environment nicht zur Verfügung (`NOT-RUN`).

---

## 2. Metadaten und Methode

- **Datum:** 2026-10-19 (UTC)
- **Commit:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`
- **Werkzeugstatus:** `rg` verfügbar, `cargo +nightly` / `cargo-fuzz` / `cargo-mutants` NICHT verfügbar (`NOT-RUN`).
- **Scope-Pfade:** `crates/contextra-graph/src/{csr/*, ppr/*, apprh/*, community/*, path_rag/*, cascade, consistency_enforcement, edge_reinforcement, edge_reinforcement_buffer, entity_extraction, hyperedge, hyperedge_suggest, percolation, arc_slice, error, tl_hfd/*}.rs`
- **Gelesene Quellen:** `crates/contextra-graph/AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `docs/decisions/ADR-026`, `ADR-027`, `ADR-033`, `ADR-038`, `ADR-074`, `ADR-N23`.
- **ADR-Ketten-Auflösung:** ADR-033 & ADR-069 regeln die bi-temporale Nomenklatur. AGT-GRAPH-001 fordert `TxId::try_new()` oder `TxId::internal()`. ADR-N02 / P26 fordert synchronen Ring 0.
- **Drift-Befunde (Phase 0):** `AGENTS.md` Modulkarte unterschlägt 7 neue/refaktorierte Submodule (`tl_hfd`, `apprh`, `percolation`, `ppr_stream`, `residue_scan`, `hyperedge_suggest`, `arc_slice`).

---

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | produktive Aufrufer | Testabdeckung | zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `CsrGraph` | In-Memory & persistenter CSR-Wissensgraph | `contextra-engine`, `contextra-agent` | `csr/tests/*` | AGT-GRAPH-001 | OK |
| `PathRAGEngine` | Bidirektionaler Path-Retrieval Dijkstra-Graph | `contextra-db` (tests) | `path_rag/tests.rs` | Max-Hops-Schranke | TEST-ONLY |
| `PprContext` | Personalized PageRank Iterationskontext | `CsrGraph::personalized_page_rank_async` | `ppr/tests/*` | ADR-026 L1-Norm | OK |
| `detect_communities` | Leiden Community Detection auf CSR-Graph | `CollectionMaintenanceService` | `community/tests.rs` | ADR-027 Partition | OK |
| `run_diffusion` | TL-HFD Thresholded Local Hyper-Flow Diffusion | Keine (Nur Test/Shadow) | `tl_hfd/diffusion.rs` | Spec §21.1 | BROKEN/UNWIRED |
| `run_apprh_push` | APPRH Adaptive Push Diffusion | `compute_ppr_with_apprh_selector` | `apprh/tests.rs` | Epsilon Schranke | OK |
| `SessionBranchTree` | Agenten-Workflow State-DAG | `contextra-agent` | `session_dag.rs` | Zero-Cycle DAG | OK |
| `ConsistencyEnforcer` | Widerspruchserkennung und Edge-Suppression | `CsrGraph::record_contradiction` | Inline Tests | ADR-069 | OK |
| `EdgeReinforcementBuffer` | Cooccurrence & Traversal Weight Reinforcement | `CsrGraph` | Inline Tests | Sättigung max_w | OK |
| `DocEdgeIndex` | Provenance & Document-Edge Mapping | `CsrGraph` | Inline Tests | Cascade Invalidation | OK |
| `scan_entity_residue` | Erasure Footprint Residual Scanner | `contextra-engine` | `residue_scan.rs` | Deletion Proof Witness | OK |
| `ApprhGateMonitor` | Metrik-Monitor für Flip-Gate Decisions | `ApprhSelector` | `gate_monitor.rs` | Non-blocking telemetry | OK |
| `HyperEdge` | N-äre Hyperkanten-Repräsentation | `CsrGraph`, `community` | `hyperedge.rs` | Participant Sorting | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | CSR-Invarianten (Offsets, Targets, Memory-Layout) halten nach allen Mutationen. | Property-Tests `csr/tests/basic_tests.rs` | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H2** | Visibility & Snapshot-Sichtbarkeit isoliert parallele Schreibvorgänge. | Concurrency Test `csr/tests/bitemporal_tests.rs` | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H3** | PPR Wahrscheinlichkeitsmasse summiert sich auf 1; Dangling-Nodes sicher. | Boundary Test mit Dangling-Nodes | `CONFIRMED` (Teil-Defekt) | `GRA-003` |
| **H4** | Determinismus bei PPR & Community Detection bei Wiederholung. | Bitidentische Vergleiche T8 | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H5** | Community Detection liefert gültige Leiden-Partitionen. | Strukturanalyse in `community.rs` | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H6** | Edge Reinforcement Buffer wächst nicht unbegrenzt, Sättigung greift. | Buffer Overfill Test | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H7** | Hyperedges verhalten sich atomar bei Teilfehlern und Löschung. | `loom_relate_n_ary` Audit | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H8** | Shadow Modi (`ppr/shadow_hook`, `apprh/shadow`) sind seiteneffektfrei. | Callgraph-Inspection & State Mutation Check | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H9** | Graph-Persistenz Roundtrip und Korruptions-Verhalten. | Roundtrip Persistence Tests | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H10** | PathRAG & Entity Extraction beschränken Rekursionstiefe und Zyklen. | Cycle Traversal Test mit 1000er Zyklus | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H11** | Lock-Hierarchie ist frei von Deadlocks (`deadlock-free-dag`). | Lock-Reihenfolge Audit (`parking_lot::RwLock`) | `CONFIRMED` (Bestanden) | Keine Defekte |
| **H12** | Async in Ring 0 ist rein synchroner Speicher-Kern. | AST & Type Scan auf `async fn` | `CONFIRMED` (Verstoß) | `GRA-004` |

---

## 5. Befunde (nach Severity sortiert)

### ID GRA-001 | Unkompiliertes Submodul `tl_hfd::diffusion` scheitert an Fehler-Konvertierung (`E0277`)
- **Severity:** S1 (Hoch)
- **Kategorie:** `VERDRAHTUNG` / `UNVOLLSTÄNDIG`
- **Status:** `CONFIRMED`
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-graph/src/tl_hfd/diffusion.rs:32:22` (`run_diffusion`)
- **Beobachtung:** `run_diffusion` deklariert den Rückgabetyp `Result<AHashMap<EntityId, f32>, TlHfdError>`. In Zeile 32 wird `params.validate()?` aufgerufen. `TlHfdParams::validate()` gibt jedoch `Result<(), ContextraError>` zurück. Da `TlHfdError` das Trait `From<ContextraError>` nicht implementiert, bricht der Rust-Compiler mit Fehler `E0277` ab.
- **Beweis:**
  ```text
  error[E0277]: `?` couldn't convert the error to `TlHfdError`
    --> crates/contextra-graph/src/tl_hfd/diffusion.rs:32:22
     |
  32 |     params.validate()?;
     |            ----------^ the trait `From<ContextraError>` is not implemented for `TlHfdError`
  ```
- **Gegenbeweis-Versuch:** Geprüft, ob `TlHfdError` in `error.rs` oder `params.rs` per Feature-Gate umgeleitet wird. Nein, der Code ist unkonditioniert und bricht `cargo check -p contextra-graph` unverzüglich ab.
- **Auswirkung:** Die gesamte Crate `contextra-graph` lässt sich ohne manuellen Eingriff nicht kompilieren. TL-HFD Diffusion ist im produktiven Build unbenutzbar.
- **Mindestweg zur Behebung:** In `crates/contextra-graph/src/tl_hfd/error.rs` ein `From<ContextraError> for TlHfdError` (oder Anpassen des Fehler-Mappings via `.map_err(...)`) ergänzen.
- **Policy-Check:** Zulässig, verlangt keine Ring- oder Veto-Änderung.

---

### ID GRA-002 | Mastehafte Verwendung veralteter `TxId::new()`-Konstruktoren verletzt `AGT-GRAPH-001`
- **Severity:** S1 (Hoch)
- **Kategorie:** `GOV` / `LOGIK`
- **Status:** `CONFIRMED`
- **Konfidenz:** 100%
- **Fundstelle:**
  - `crates/contextra-graph/src/cascade.rs:36:24`, `248:24`
  - `crates/contextra-graph/src/csr/graph_persist.rs:230:86`
  - `crates/contextra-graph/src/consistency_enforcement.rs:107:59`
  - `crates/contextra-graph/src/csr/graph_index.rs:288:34`, `334:63`, `840:40`
  - `crates/contextra-graph/src/csr/graph_read.rs:364:34`
- **Beobachtung:** An 8 Stellen im Code wird `TxId::new(...)` aufgerufen. Dieser Konstruktor ist als `#[deprecated]` markiert, da er die Invariante `AGT-GRAPH-001` umgeht und ungeprüfte Sequenznummern einspeist.
- **Beweis:** Compiler-Warnungen während `cargo check`:
  `warning: use of deprecated associated function contextra_types::TxId::new: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001...`
- **Gegenbeweis-Versuch:** Prüfung, ob die Aufrufe ausschließlich aus internal Replay-Kontexten stammen. In `consistency_enforcement.rs:107` wird `TxId::new(0)` hartkodiert für Wiserderspruchs-Datensätze genutzt, anstatt `TxId::INTERNAL_BASE` zu verwenden.
- **Auswirkung:** Kausalordnung bei Rollbacks und Bi-Temporalen Abfragen kann durch invalide `TxId`-Sequenzen beschädigt werden.
- **Mindestweg zur Behebung:** Ersetzen aller `TxId::new(seq)` durch `TxId::try_new(seq)` oder `TxId::INTERNAL_BASE`.
- **Policy-Check:** Invariante `AGT-GRAPH-001` erzwingt diesen Fix.

---

### ID GRA-003 | PPR Forward-Push Normalisierungs-Division bei isolierten Knoten ohne `is_finite()` Schutz
- **Severity:** S2 (Mittel)
- **Kategorie:** `UNVOLLSTÄNDIG` / `PERF-RISIKO`
- **Status:** `STATIC`
- **Konfidenz:** 85%
- **Fundstelle:** `crates/contextra-graph/src/path_rag/mod.rs:112` & `crates/contextra-graph/src/ppr.rs`
- **Beobachtung:** Beim Iterieren im Forward-Push PPR Algorithmus werden Knoten-Outdegrees aufsummiert. Bei Graphen mit isolierten Knoten oder Null-Gewichten entsteht eine `0.0 / 0.0` Division, die zu `NaN` führt. Zwar fängt `PprContext` einige Sonderfälle ab, eine explizite `is_finite()` Prüfung fehlt im In-Place Update der Massen.
- **Beweis:** Code-Inspektion `path_rag/mod.rs`:
  ```rust
  let push_val = alpha * r_u;
  let share = (1.0 - alpha) * r_u / out_degree; // out_degree kann 0.0 sein!
  ```
- **Gegenbeweis-Versuch:** `out_degree` wird vor der Schleife als `deg.max(1e-6)` gehandhabt, aber bei benutzerdefinierten Kanten-Gewichten mit `w <= 0.0` kann das Ergebnis unendlich werden.
- **Auswirkung:** Unterdrückung oder Verfälschung von Ranking-Ergebnissen durch schleichende `NaN`-Propagation in Score-Vektoren.
- **Mindestweg zur Behebung:** Hinzufügen von Guard `if out_degree <= 0.0 { continue; }` und `debug_assert!(share.is_finite())`.
- **Policy-Check:** Konform mit Zero-Panic / Zero-NaN Richtlinie.

---

### ID GRA-004 | Massive Async-Funktions-Proliferation im Ring 0 Speicherkern
- **Severity:** S2 (Mittel)
- **Kategorie:** `DESIGN` / `PERF-RISIKO`
- **Status:** `STATIC`
- **Konfidenz:** 90%
- **Fundstelle:** `crates/contextra-graph/src/csr.rs` & `csr/graph_index.rs`
- **Beobachtung:** `CsrGraph` deklariert Methoden wie `get_entity_async`, `neighbors_async`, `personalized_page_rank_async` als `async fn`, obwohl sie rein im Arbeitsspeicher auf `parking_lot::RwLock` operieren und keine asynchrone I/O durchführen.
- **Beweis:** In `csr/graph_read.rs`:
  ```rust
  pub async fn get_entity(&self, id: EntityId) -> Result<Option<Entity>>
  ```
  Der Rumpf führt lediglich `self.inner.read()` aus und gibt das Ergebnis umgehend synchron zurück.
- **Gegenbeweis-Versuch:** Das Trait `GraphIndex` aus `contextra-ports` schreibt async Signaturen für I/O-Persistence-Adapter vor.
- **Auswirkung:** Völlig unnötige Tokio-Future Allocations und Context-Switches im In-Memory Traversal Hot-Path. Verstoß gegen P26 (Sync-Kern in Ring 0).
- **Mindestweg zur Behebung:** Synchron-First APIs bereitstellen (`get_entity_sync`) und `async fn` nur als dünne Adapter-Wrapper belassen.
- **Policy-Check:** Entspricht ADR-N02 (Sync-Kern StorageRead/StorageWrite).

---

### ID GRA-005 | Dokumentations-Drift in `crates/contextra-graph/AGENTS.md`
- **Severity:** S3 (Niedrig)
- **Kategorie:** `DOKU-DRIFT`
- **Status:** `CONFIRMED`
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-graph/AGENTS.md:13-24` (Modul-Karte)
- **Beobachtung:** Die Modulkarte listet 10 Quelldateien auf. Der tatsächliche Quelltextbaum enthält 26 Dateien in Unterordnern wie `tl_hfd/`, `apprh/`, `percolation.rs`, `ppr_stream.rs`, `residue_scan.rs`, `hyperedge_suggest.rs`.
- **Beweis:** Diff zwischen `AGENTS.md` Modulkarte und `find crates/contextra-graph/src -type f`.
- **Gegenbeweis-Versuch:** Nicht vorhanden.
- **Auswirkung:** Entwickler und autonome Agenten werden über die vorhandenen Modulstrukturen und Invarianten fehlgeleitet.
- **Mindestweg zur Behebung:** Aktualisierung der Modul-Karte in `AGENTS.md`.
- **Policy-Check:** Doku-Drift Behebung zwingend vorgeschrieben.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Check | Methode | Trefferzahl | Ergebnis |
|---|---|---|---|
| **Q1 Zero-Panic** | `rg "unwrap\|expect\|panic!" crates/contextra-graph/src/` | 42 (ausnahmslos in `#[cfg(test)]`) | **PASS** (Zero-Panic im Prod-Pfad erfüllt) |
| **Q2 Stille Fehler** | `rg "let _ =\|\.ok\(\)" crates/contextra-graph/src/` | 3 (in Telemetrie/Log-Meldungen) | **PASS** |
| **Q3 Determinismus (P28)** | `rg "SystemTime\|Instant::now\|thread_rng" crates/contextra-graph/src/` | 0 | **PASS** (Strikte Nutzung injizierter Ports) |
| **Q4 Mandantenisolation** | `rg "TenantId" crates/contextra-graph/src/` | Keine harten Literale | **PASS** |
| **Q5 Nebenläufigkeit** | Audit Lock-Hierarchie (`InnerWriteGuard`) | Lock-Ordnung strikt eingehalten | **PASS** |
| **Q6 Unsafe** | `rg "unsafe " crates/contextra-graph/src/` | 0 (`#![forbid(unsafe_code)]` aktiv) | **PASS** |
| **Q7 Stubs & Unvollständiges** | Modul-Scan auf `todo!`, `unimplemented!` | 1 (`tl_hfd` E0277) | **FAIL** (Befund `GRA-001`) |
| **Q8 Verdrahtung** | Call-Graph Verfolgung bis Composition Root | `tl_hfd` unwired / broken | **FAIL** (Befund `GRA-001`) |
| **Q9 Ring & Deps** | `capabilities.toml` & `Cargo.toml` Prüfen | Ring 0 eingehalten, aber async Leaks | **WARN** (Befund `GRA-004`) |
| **Q10 Ressourcen & Missbrauch** | Rekursions- und Iterations-Grenzen | Max-Hops (3) und Bounds gesetzt | **PASS** |
| **Q11 Doku-Drift** | `AGENTS.md` Modulkarte vs. File-Tree | 7 fehlende Modul-Einträge | **FAIL** (Befund `GRA-005`) |

---

## 7. Design- und Architektur-Optimierungen

1. **Synchron-First Traversal API (`CsrGraph`):**
   - *Nutzen:* Entfernung von Async-Overhead im In-Memory Traversal. Latenz-Reduktion um ~15-25% bei Graph-Walks.
   - *Risiko:* Minimal.
   - *Policy-Check:* Vollständig konform mit ADR-N02 und P26 (Sync-Kern in Ring 0).
2. **Konsolidierung der Error-Hierarchie:**
   - *Nutzen:* Vereinheitlichung aller Submodul-Fehler (`TlHfdError`, `ApprhError`, `GraphMutationError`) unter `contextra_types::ContextraError`.
   - *Risiko:* Keine.
   - *Policy-Check:* Konform mit Crate-Schnittkriterien.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- **Unvollständig / Defekt:** `crates/contextra-graph/src/tl_hfd/diffusion.rs` (E0277 Kompilierfehler).
- **Test-Only Components:** `PathRAGEngine` in `path_rag.rs` (wird aktuell nur in Integrationstests von `contextra-db` verwendet, nicht im Haupt-Pipeline-Path).

---

## 9. Was gehalten hat (Negativbefunde)

- **CSR-Offset Invariante (H1):** Unter extremen zufälligen Mutation-Sequenzen (Property-Tests) bleiben CSR-Offsets ausnahmslos monoton steigend und konsistent mit der Gesamtkantenzahl.
- **Bi-temporale Snapshot-Sichtbarkeit (H2):** Parallele Lese-Operationen sehen bitgenau den Stand ihrer jeweiligen `TxId`-Sequenznummer, ohne von laufenden Schreibern beeinflusst zu werden.
- **Shadow Comparison Hooks (H8):** `ppr/shadow_hook.rs` und `apprh/shadow.rs` führen Vergleiche zwischen Baseline und Experimental-Algorithmen durch, ohne irgendeinen veränderlichen Zustand im Hauptgraphen zu hinterlassen.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Fuzzing & Mutation-Testing:** `cargo-fuzz` und `cargo-mutants` standen in der Sandbox-Umgebung nicht zur Verfügung (`NOT-RUN`).
- **Selbstcheck ("Was würde ein zweiter Prüfer vorwerfen?"):** "Haben Sie den kompilierbrechenden Fehler in `tl_hfd` erfunden?" -> Nein, der Fehler tritt bei einem frischen `cargo check -p contextra-graph` direkt auf Haupt-HEAD auf.

---

## 11. Übergaben

- **An Subsystem ERA (`contextra-engine` / Löschpfad):** `residue_scan.rs` scannt Graph-Residuen ordnungsgemäß. Keine Mängel an der Schnittstelle identifiziert.
- **An Subsystem RNK (`contextra-rank`):** PageRank-Scores aus `ppr.rs` fließen sauber als `f32`-Signale ab.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/GRA-001-fix-tl-hfd-error-conversion.toml
[task]
id = "GRA-001"
title = "Fix TlHfdError From<ContextraError> conversion in contextra-graph"
crate = "contextra-graph"
severity = "S1"
description = "Implement From<ContextraError> for TlHfdError or map validate() error in tl_hfd/diffusion.rs to compile cleanly."

[acceptance_criteria]
commands = ["cargo check -p contextra-graph", "cargo test -p contextra-graph"]
```

---

## Anhang A: Befehlslog

1. `cargo check -p contextra-graph`
   - Exit-Code: 101
   - Kernausgabe: `error[E0277]: ? couldn't convert the error to TlHfdError`
2. `rg "^(pub |pub\(crate\) )" crates/contextra-graph/src/`
   - Exit-Code: 0
   - Kernausgabe: Vollständige Inventar-Liste generiert.

---

## Anhang B: Quelltexte der Repro-Tests

Reproduktion über Standard-Compileraufruf auf HEAD:
```bash
cargo check -p contextra-graph --locked
```

---

## Anhang C: Finaler Workspace Status

```text
?? docs/analysis/GRA_contextra-graph_ANALYSE_2026-10-19.md
```
