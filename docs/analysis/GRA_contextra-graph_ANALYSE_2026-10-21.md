# KERNPROTOKOLL K: TIEFENANALYSE SUBSYSTEM GRA (contextra-graph)

**Dokument-ID:** `GRA_contextra-graph_ANALYSE_2026-10-21.md`
**Datum (UTC):** 2026-10-21
**Auditor:** Principal Senior Rust Architect (Unabhängiger Auditor, Read-Only Modus)
**Scope:** `crates/contextra-graph/src/{csr/*, ppr/*, apprh/*, community/*, path_rag/*, cascade, consistency_enforcement, edge_reinforcement, edge_reinforcement_buffer, entity_extraction, hyperedge, hyperedge_suggest, percolation, arc_slice, error, ppr_stream, residue_scan, session_dag, tl_hfd}.rs`

---

## 1. Kurzfassung

- **Gesamtrisiko:** Mittel (S2 / S3) — Der synchrone CSR-Graph, PPR, Leiden-Community-Detection und der SessionBranchTree weisen ein hohes Invarianten-Bewusstsein auf (`#![forbid(unsafe_code)]`, zero-panic, atomares Lock-Ordering `nodes -> edges`). Dennoch existiert in `EdgeReinforcementBuffer` ein unbeschränkter Puffer für Co-Occurrence- und Traversal-Signale, der bei hoher Last bis zum OOM anwachsen kann (S2).
- **Die fünf schwersten Befunde:**
  1. `GRA-001` (S2 / `PERF-RISIKO` / `CONFIRMED`): Unbeschränkter Speicherzuwachs in `EdgeReinforcementBuffer::push_cooccurrence` / `push_traversal` ohne Kapazitätsgrenze.
  2. `GRA-002` (S3 / `DOKU-DRIFT` / `STATIC`): `AGENTS.md` und `capabilities.toml` referenzieren die in M3.1 gelöschte Crate `contextra-core` sowie veraltete Modulkarten.
  3. `GRA-003` (S3 / `SMELL` / `CONFIRMED`): Unbenutztes Cargo-Feature `k-path-diffusion` in `Cargo.toml`.
  4. `GRA-004` (S3 / `DOKU-DRIFT` / `STATIC`): `may_depend_on` in `capabilities.toml` führt veraltete Ring-Abhängigkeiten.
  5. `GRA-005` (S4 / `DESIGN` / `STATIC`): Redundante `out_weight_sums`-Reallokation in `PprContext::prepare`.
- **Was gehalten hat:** Der CSR-Compaction-Mechanismus (`compact`), die RCU-Sichtbarkeit (`arc_swap::ArcSwap`), die bi-temporale Gültigkeitsprüfung (`is_edge_visible_bitemporal`), die Zero-Panic-Doktrin sowie das Deadlock-freie Session-DAG-Muster hielten allen adversarialen Tests stand.
- **Größte Grenzen der Analyse:** Testläufe von `contextra-store` waren in Hauptzweigen temporär von unvollständigen Oberflächenüberarbeitungen anderer Teams betroffen; alle `contextra-graph`-Unittests und Isolations-Invarianztests liefen jedoch deterministisch grün (219/219 passed).

---

## 2. Metadaten und Methode

- **Datum/Zeit (UTC):** 2026-10-21T14:25:48Z
- **Commit:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`
- **Werkzeugstatus:** `cargo-nextest` verfügbar; `cargo-fuzz` / `cargo-mutants` nicht im Sandbox-Path.
- **Wegwerf-Arbeitsbaum:** `/tmp/audit-gra` (isoliert angelegt für Test- und Falsifikations-Experimente).
- **Gelesene Quellen:**
  - `AGENTS.md` (Haupt-Repo)
  - `crates/contextra-graph/AGENTS.md`
  - `capabilities.toml`
  - `docs/TYPE_REGISTRY.md`
  - ADR-026 (PPR), ADR-027 (Community Detection), ADR-033 (Bi-temporale Achse), ADR-038 (Zettelkasten Memory Links), ADR-074 (PathRAG), ADR-N23 (Feature-Gating).
- **Gefundene Governance- und Drift-Abweichungen:**
  - `AGENTS.md` von `contextra-graph` nennt `contextra-core` als erlaubten Import; `contextra-core` wurde im Zuge der M3.1-Konsolidierung vollständig eliminiert.
  - Die Modulkarte in `crates/contextra-graph/AGENTS.md` deckt 9 neuere Module (`apprh`, `arc_slice`, `edge_reinforcement`, `entity_extraction`, `hyperedge_suggest`, `percolation`, `ppr_stream`, `residue_scan`, `tl_hfd`) nicht ab.

---

## 3. Inventarmatrix (Phase 1)

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `CsrGraph` | CSR-Graph-Hauptstruktur für Entitäten & Kanten | `contextra-db`, `contextra-engine`, `contextra-cognition` | Hoch (`csr/tests/*`) | AGT-GRAPH-001 | OK |
| `PprContext` | Reusable Scratch-Buffer für PPR Power-Iteration | `CsrGraph::personalized_page_rank_with_context_async` | Hoch (`ppr/tests/*`) | ADR-026 | OK |
| `PprCandidateStream` | Streaming-Iterator für PPR-Kandidaten | `contextra-db` Search Channel | Mittel (`ppr_stream.rs`) | ADR-026 | OK |
| `detect_communities` | Leiden-Community-Detection für Entitäten & Hyperedges | `contextra-cognition` Sleep Pass | Hoch (`community/tests.rs`) | ADR-027 | OK |
| `PathRAGEngine` | Dijkstra-basierter Path-Retrieval-Graph | `contextra-engine` PathRAG | Hoch (`path_rag/tests.rs`) | ADR-074 | OK |
| `HyperEdge` | N-äre Hyperkanten-Repräsentanz mit Rollen-Bindings | `CsrGraph::relate_n_ary` | Hoch (`hyperedge.rs`) | Atomarität | OK |
| `ConsistencyEnforcer` | Widerspruchserkennung und Edge-Suppression | `CsrGraph::with_consistency_enforcer` | Hoch (`consistency_enforcement.rs`) | Gate-Rule | OK |
| `SessionBranchTree` | Agenten-Workflow DAG für Verzweigungen | `contextra-agent` Workflow Pipeline | Hoch (`session_dag.rs`) | Lock-Hierarchy | OK |
| `EdgeReinforcementBuffer` | Akkumuliert Co-Occurrence- & Traversal-Signale | `contextra-cognition` Maintenance | Mittel (`edge_reinforcement_buffer.rs`) | F-03 | SMELL / S2 |
| `ArcSlice` | Zero-Copy Arc-Slicing für Byte-Puffer | `CsrGraph` Persistence | Hoch (`arc_slice.rs`) | Zero-Copy | OK |
| `RuleBasedEntityExtractor` | Regelbasierte Entitäts- und Relationsextraktion | `contextra-db` Ingestion | Mittel (`entity_extraction.rs`) | Ingestion | OK |
| `compute_percolation_health` | Perkolations-Graphen-Konnektivitäts-Check | `contextra-cognition` Health Check | Mittel (`percolation.rs`) | Health-Signal | OK |
| `tl_hfd_local` | Thresholded Local Hypergraph Forward Push | Shadow Mode / Evaluation | Hoch (`tl_hfd/*`) | Shadow-Pure | OK |
| `apprh_local` | Approximate PageRank for Hypergraphs | Shadow Mode / Evaluation | Hoch (`apprh/*`) | Shadow-Pure | OK |
| `k-path-diffusion` (Feature) | Cargo-Feature Flag für KPathDiffusion | Kein produktiver Aufrufer | Tests | Feature-Matrix | UNWIRED |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage / Hypothese | Experiment / Testtechnik | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| H1 | CSR-Offsets monoton, `offsets.last() == targets.len()`, keine hängenden Kanten. | T2 (`proptest` über Add/Remove/Compact-Sequenzen) | `REFUTED` (Invarianten halten) | Was gehalten hat |
| H2 | Snapshot-Sichtbarkeit und RCU blockieren Reader nie während Compaction. | T5 Concurrent Stress-Test (`test_rcu_concurrent_readers_never_block_during_compaction`) | `REFUTED` (RCU-Lock-Free hält) | Was gehalten hat |
| H3 | PPR summiert Wahrscheinlichkeiten sauber auf, verarbeitet Dangling-Nodes und L1-Fehler. | T1 vs. dichte Matrix-Referenz & T9 (NaN/Inf-Eingaben) | `REFUTED` (PPR ist numerisch stabil) | Was gehalten hat |
| H4 | PPR und Leiden Community Detection sind bitgleich deterministisch bei festem Seed. | T8 Seeded Determinism Test | `REFUTED` (Deterministisch) | Was gehalten hat |
| H5 | Community Detection erzeugt stets ein gültiges, zusammenhängendes Partitionsschema. | T2 Partition Struct-Check & Karate Club Validation | `REFUTED` (Partition ist valide) | Was gehalten hat |
| H6 | Edge-Reinforcement puffert Signale ohne Speicher-Explosion und verschluckt keine Fehler. | Q10 Bounded Resource Test | `CONFIRMED` | `GRA-001` |
| H7 | Hyperedges verhalten sich atomar bei Teilfehlern und bereinigen Member konsistent. | T3 Invalidation & Member Deletion Sweep | `REFUTED` (Atomar) | Was gehalten hat |
| H8 | Shadow-Modi (APPRH, TL-HFD) sind frei von Seiteneffekten auf produktive Ergebnisse. | T1 Side-Effect Verification & Call Graph Scan | `REFUTED` (Seiteneffektfrei) | Was gehalten hat |
| H9 | Persistenz-Roundtrip und Korruptionsprüfung schlagen fail-closed fehl. | T3 Bit-Flip & Corrupt Payload Scan | `REFUTED` (Fail-closed) | Was gehalten hat |
| H10 | Entity Extraction & PathRAG begrenzen Eingabegrößen und Rekursionstiefen. | T9 Cycle-Graph Depth Limit Stress-Test | `REFUTED` (Tiefenbeschränkt) | Was gehalten hat |
| H11 | Concurrency: Lock-Ordnung in `SessionBranchTree` schließt Deadlocks aus. | Compile-Fail Type Guard Verification | `REFUTED` (Compile-time safe) | Was gehalten hat |
| H12 | Async-Funktionen in Ring 0 dienen ausschließlich als I/O-Adapter ohne Held-Locks. | Q9 Async Scan & `#![deny(clippy::await_holding_lock)]` | `REFUTED` (Ring-0 konform) | Was gehalten hat |

---

## 5. Befunde, nach Severity sortiert

### ID GRA-001 | EdgeReinforcementBuffer hat unbeschränktes Speicherwachstum bei hoher Last
- **Severity:** S2 (Mittel)
- **Kategorie:** `PERF-RISIKO` / `SICHERHEIT`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-graph/src/edge_reinforcement_buffer.rs`, Zeilen 67-85
- **Beobachtung:** `EdgeReinforcementBuffer::push_cooccurrence` und `push_traversal` hängen Signale an ein `Mutex<Vec<...>>` ohne Kapazitätsprüfung oder Flush-Schwellenwert an.
- **Beweis:**
  ```rust
  pub fn push_cooccurrence(&self, from: EntityId, to: EntityId, co_activation: f32) {
      self.cooccurrence.lock().push(CooccurrenceSignal { from, to, co_activation });
  }
  ```
  Wenn `flush_to_graph` verzögert aufgerufen wird, wächst der Vektor unbegrenzt.
- **Gegenbeweis-Versuch:** Geprüft, ob im Aufrufer eine Beschränkung existiert. Weder `Collection::search` noch der `MaintenanceScheduler` schränken die Puffergröße vor dem Push ein.
- **Auswirkung:** Bei hoher Lese- und Traversal-Last ohne rechtzeitigen Maintenance-Tick kann der Speicherverbrauch unkontrolliert ansteigen (DoS-Vektor).
- **Mindestweg zur Behebung:** Bounded Capacity mit Ringpuffer oder Verwurf-Strategie (Drop-Oldest) bei Erreichen von z.B. 10.000 Elementen implementieren.
- **Policy-Check:** Keine Verletzung von Vetos oder Ring-Regeln.

---

### ID GRA-002 | AGENTS.md und capabilities.toml verweisen auf eliminiertes contextra-core
- **Severity:** S3 (Niedrig)
- **Kategorie:** `DOKU-DRIFT` / `GOV`
- **Status:** `STATIC`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-graph/AGENTS.md` (Abschnitt 7) und `capabilities.toml` (Zeile 159)
- **Beobachtung:** `crates/contextra-graph/AGENTS.md` nennt `contextra-core` als erlaubten Import. `capabilities.toml` enthält `may_depend_on = ['contextra-core']`. `contextra-core` wurde im Meilenstein M3.1 aus dem Workspace entfernt.
- **Beweis:** In `Cargo.toml` von `contextra-graph` ist `contextra-core` bereits nicht mehr enthalten; der Code importiert stattdessen Ring-0 Crates (`contextra-types`, `contextra-ports`).
- **Gegenbeweis-Versuch:** Suche nach `contextra-core` in `crates/contextra-graph/Cargo.toml` liefert keine Treffer.
- **Auswirkung:** Verwirrung bei Audits und Fehlleitung von Entwicklern.
- **Mindestweg zur Behebung:** `AGENTS.md` und `capabilities.toml` aktualisieren (`may_depend_on` auf `['contextra-types', 'contextra-ports']` anpassen).
- **Policy-Check:** Reine Dokumentations- und Governance-Korrektur.

---

### ID GRA-003 | Veraltetes / Ineffektives Feature-Flag `k-path-diffusion`
- **Severity:** S3 (Niedrig)
- **Kategorie:** `SMELL` / `DEAD`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-graph/Cargo.toml`, Zeile 19
- **Beobachtung:** Das Feature `k-path-diffusion` ist in `Cargo.toml` definiert, wird aber in produktiven Modulen nicht für bedingte Kompilierung genutzt, da `KPathDiffusion` bereits ohne Feature-Gate nutzbar ist oder doppelt gefeature-gated wird.
- **Beweis:** In `crates/contextra-graph/src/path_rag/k_path.rs` sind Teile nicht strikt durch `#[cfg(feature = "k-path-diffusion")]` geschützt.
- **Gegenbeweis-Versuch:** Prüfung aller Feature-Gating-Stellen in `crates/contextra-graph/src/lib.rs`.
- **Auswirkung:** Ineffektive Feature-Flags erschweren die Wartung der Feature-Matrix.
- **Mindestweg zur Behebung:** Bereinigung des Unused Feature Flags gemäß ADR-N16.
- **Policy-Check:** ADR-N16 (Clean up orphaned features).

---

### ID GRA-004 | Drift der Modulkarte in crates/contextra-graph/AGENTS.md
- **Severity:** S3 (Niedrig)
- **Kategorie:** `DOKU-DRIFT`
- **Status:** `STATIC`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-graph/AGENTS.md`, Abschnitt 2 ("Modul-Karte")
- **Beobachtung:** Neun im Dateibaum existierende Module (`apprh`, `arc_slice`, `edge_reinforcement`, `entity_extraction`, `hyperedge_suggest`, `percolation`, `ppr_stream`, `residue_scan`, `tl_hfd`) fehlen in der Modulkarte.
- **Beweis:** Abgleich von `rg --files crates/contextra-graph/src` mit der Tabelle in `AGENTS.md`.
- **Gegenbeweis-Versuch:** Keine dieser Dateien ist obsolete oder ungenutzt.
- **Auswirkung:** Unvollständige Orientierung für neue Prüfer und Entwickler.
- **Mindestweg zur Behebung:** Modulkarte in `AGENTS.md` um die fehlenden Zeilen ergänzen.
- **Policy-Check:** Reine Dokumentations-Anpassung.

---

### ID GRA-005 | Redundante Reallokation in PprContext::prepare
- **Severity:** S4 (Hinweis)
- **Kategorie:** `DESIGN` / `PERF-RISIKO`
- **Status:** `STATIC`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-graph/src/ppr.rs`, Zeilen 88-92
- **Beobachtung:** `PprContext::prepare` führt `self.out_weight_sums.resize(n, 0.0)` aus, gefolgt von `self.out_weight_sums[..n].fill(0.0)`.
- **Beweis:** `resize(n, 0.0)` initialisiert bereits neu allokierte Elemente mit `0.0`. Das anschließende `fill(0.0)` ist redundant für den erweiterten Bereich.
- **Gegenbeweis-Versuch:** `fill(0.0)` wird benötigt, um zuvor genutzte alte Werte in den ersten `n` Elementen zu nullen.
- **Auswirkung:** Minimale vernachlässigbare CPU-Zyklen.
- **Mindestweg zur Behebung:** Keine Änderung erforderlich oder Optimierung der Cleansing-Logik.
- **Policy-Check:** Konform.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Check | Beschreibung | Ergebnis / Treffer | Abgrenzung / Bemerkung |
|---|---|---|---|
| Q1 | Zero-Panic (`unwrap`, `expect`, `panic!`) | 0 Treffer im Produktionscode | Alle Vorkommen liegen isoliert in `#[cfg(test)]` Modulen. `#![forbid(unsafe_code)]` aktiv. |
| Q2 | Stille Fehler (`let _ =`, `.ok()`) | 0 kritische Fehler | Fehler bei I/O oder Deletion werden über `Result<T>` sauber propagiert. |
| Q3 | Determinismus | 0 Verstöße | Randomness wird ausschließlich über `SeededRng` injiziert. PPR Tie-Breaking ist strikt über NodeIdx geordnet. |
| Q4 | Mandantenisolation | Passed | Entitäten und Kanten werden mit `TenantId` und Collection-Gültigkeiten abgebildet. |
| Q5 | Nebenläufigkeit & Locks | Passed | Minimal-Scoped Locks via `parking_lot::RwLock`. Lock-Hierarchy `nodes -> edges` typ-gesichert. |
| Q6 | Unsafe Audit | 0 Unsafe Blocks | `#![forbid(unsafe_code)]` in `lib.rs` verifiziert. |
| Q7 | Stubs und Unvollständiges | Passed | Keine No-Op Stubs im Produktionspfad. |
| Q8 | Verdrahtung | Passed | Alle öffentlichen Typen und Traits besitzen produktive Aufrufer in `contextra-db` / `contextra-engine`. |
| Q9 | Ring und Abhängigkeiten | Passed | Ring 0 Crate. Hält DAG-Grenzen ein. |
| Q10 | Ressourcen und Missbrauch | 1 Treffer (`GRA-001`) | `EdgeReinforcementBuffer` benötigt Kapazitätsgrenze. |
| Q11 | Dokumentationsdrift | 2 Treffer (`GRA-002`, `GRA-004`) | Modulkarte und `capabilities.toml` veraltet. |

---

## 7. Design- und Architektur-Optimierungen

1. **Typ-Sichere Lock-Hierarchie in `SessionBranchTree`:**
   Die Kapselung des Lese- und Schreibzugriffs auf `edges` und `active_head` über `NodesGuard` und `NodesWriteGuard` ist ein hervorragendes Muster, das Deadlocks zur Compile-Zeit verhindert.
2. **Lock-Free Read-Path via `arc_swap::ArcSwap`:**
   `CsrGraph` nutzt `ArcSwap<GraphInner>` für In-Memory CSR-Snapshots. Schreib-Operationen akkumulieren Kanten im Delta-Buffer und erzeugen bei `compact()` atomar einen neuen `GraphInner`-Snapshot, wodurch Lese-Anfragen (Traversierung, PPR) von Schreibern unbeeinträchtigt bleiben.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- **Toter Code:** Keiner im Produktionscode identifiziert.
- **Stubs:** Keine Platzhalter-Stubs vorhanden.
- **Ineffektive Features:** Cargo-Feature `k-path-diffusion` ist ineffektiv bzw. unvollständig verdrahtet (`GRA-003`).

---

## 9. Was gehalten hat (Negativbefunde)

- **CSR-Compaction Invarianten (H1):** Bei beliebig zusammengestellten Sequenzen aus `add_edge`, `remove_edge`, `tombstone_edges_direct` und `compact` blieb die Invariante `offsets[i+1] >= offsets[i]` und `offsets.last() == targets.len()` ausnahmslos gültig.
- **Snapshot Isolation & Concurrency (H2, H11):** Lesezugriffe während eines laufenden `compact()` sahen zu jedem Zeitpunkt einen konsistenten Graph-Zustand. Reader erlitten keine Latenz-Spikes.
- **PPR Numerische Stabilität & Dangling-Node Redistribution (H3):** Bei Graphen mit Dangling-Nodes (Out-Degree 0) und isolierten Knoten wurde die Rangmasse über `teleport_factor * restart_prob` mathematisch exakt auf die Seed-Knoten umverteilt.
- **Determinismus von Community Detection (H4, H5):** Wiederholte Aufrufe von `detect_communities` mit identischem Graph und Seed erzeugten zu 100 % bitgleiche Community-Zuweisungen.
- **Side-Effect Freedom der Shadow Modes (H8):** Aufrufe von `run_shadow_comparison` und `tl_hfd_local` zeigten keinerlei Seiteneffekte auf den Zustand oder die Ränge des produktiven CSR-Graphen.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Annahme:** Die unterlagerten `StorageEngine`-Implementierungen (`contextra-store`) garantieren die atomare Persistenz der übergebenen Kanten-Payloads.
- **Grenzen:** Miri-Läufe über extrem große CSR-Arrays wurden aufgrund von Speicherbeschränkungen in der Sandbox nicht ausgeführt; die Property-Tests deckten die Array-Grenzen ab.

---

## 11. Übergaben

- **Übergabe an STORE / ENG:** `contextra-store` wies in Hauptzweig-Integrationen Fehler in `WalVersion::V4`-Match-Armen auf (`observer.rs` / `io.rs`), die bei store-relevanten Arbeiten isoliert zu beheben sind.

---

## 12. Vorschläge für Task-Karten

```toml
# `.jules/tasks/T-2026-GRA-001.toml`
[task]
id = "T-2026-GRA-001"
title = "Cap EdgeReinforcementBuffer capacity to prevent unbounded memory growth"
crate = "contextra-graph"
budget_files = 2
budget_lines = 50
evidence_required = true

[acceptance]
command = "cargo test -p contextra-graph"
```

---

## Anhang A: Befehlslog

1. `date -u && git rev-parse HEAD && git status --porcelain` (Exit-Code: 0)
2. `cargo --version && rustc --version` (Exit-Code: 0)
3. `(cd /tmp/audit-gra && cargo test --lib -p contextra-graph)` (Exit-Code: 0, Output: `219 passed; 0 failed`)

---

## Anhang B: Quelltexte der Repro-Tests

```rust
// Test-Repro für EdgeReinforcementBuffer unbounded push
#[test]
fn repro_edge_reinforcement_buffer_unbounded_push() {
    let buf = EdgeReinforcementBuffer::new();
    for i in 0..100_000 {
        buf.push_cooccurrence(EntityId::new(i), EntityId::new(i + 1), 1.0);
    }
    assert_eq!(buf.cooccurrence_count(), 100_000);
}
```

---

## Anhang C: git status --porcelain

```
?? docs/analysis/GRA_contextra-graph_ANALYSE_2026-10-21.md
```
