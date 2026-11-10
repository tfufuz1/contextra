# Implementierungsspezifikation: `contextra-graph` → entkoppelte Graph-Bausteine

| | |
|---|---|
| **Crate** | `crates/contextra-graph` (CSR-Graph, PPR, PathRAG, Communities, Hyperkanten, Kaskade, experimentelle Diffusion) |
| **Zielzustand** | Sammlung isolierter, einzeln entnehmbarer Bausteine; Experimente gekapselt und isoliert testbar |
| **Methode** | Rein statische Analyse (Quelltext, `Cargo.toml`, `AGENTS.md`, `grep`-Kopplungsmatrix). **Nichts wurde kompiliert oder ausgeführt.** Build-Aussagen sind Schlussfolgerungen und vor der Umsetzung per `cargo` zu bestätigen. |
| **Umfang Ist** | 15.630 Zeilen Nicht-Test-Quelltext in `src/`, 52 Integrationstests, 1 Fuzz-Target, 2 Benchmarks |

---

## 1. Ziel und Nicht-Ziele

**Ziel.** Algorithmen (Forward-Push-PPR, dichte Power-Iteration, Community-Erkennung, PathRAG, Hyperkanten-Sterne, bitemporale CSR-Struktur) sind einzeln entnehmbar. Kein Baustein importiert `contextra_*`; experimentelle Verfahren (APPRH, TL-HFD, Perkolation, Edge-Reinforcement, Session-DAG) hängen nur am stabilen Kern, nie umgekehrt.

**Nicht-Ziele.** Keine Algorithmusänderung, keine Änderung des Persistenzformats (`PersistedEdgePayload`), keine Änderung der Sichtbarkeitssemantik.

---

## 2. Befund (Ist-Analyse)

### 2.1 Was bereits sauber entkoppelt ist (wiederverwenden)

| Naht | Beleg |
|---|---|
| `PathGraph`-Trait (`neighbors_with_weights`, `predecessors_with_weights`, Hyperkanten-Zugriff) und `forward_push_ppr<G: PathGraph>` | `path_rag/mod.rs:47,96` |
| `GraphProvider`-Trait + `InMemoryGraphProvider`; generische `compute_pagerank<G>` und `detect_communities_on_provider<G>` | `provider.rs:10,67,141,206` |
| Diffusions-Module ohne Bezug zu `CsrGraph`/`GraphIndex`/`StorageEngine` | `ppr/{mod,forward_push,dense}.rs`, `tl_hfd/diffusion.rs`, `apprh/diffusion.rs`, `path_rag/mod.rs` (Treffer jeweils 0) |
| Reine Sichtbarkeitsfunktionen (bitemporal) | `csr/visibility.rs` (nur `TxId`) |
| Tokio nur als Dev-Dependency, Bibliothekscode ist runtime-frei | `Cargo.toml` |

### 2.2 Konkrete Befunde

| ID | Befund | Fundstelle |
|---|---|---|
| **G-1** | **Verwaiste Dateien, die den Standalone-Kern bereits vorzeichnen:** `types.rs` (388 Z.: `EntityId`, `DocId`, `TxId`, `Entity`, `Edge` mit bitemporalen Feldern, `HyperEdgeId`, `RoleId`, `RoleBinding`, `TlHfdParams`, `PprAlgorithm`, `PprConfig`, `ResourceTracker`, `RequestContext`), `storage.rs` (81 Z.: `GraphStorage`-Trait + `InMemoryGraphStorage`), `contract.rs` (21 Z.: `Error`, `Clock`, `Rng`, `NodeId`, `Weight`). Keine ist in `lib.rs` per `mod` deklariert → toter Code, aber das fertige Gerüst für M-1. | `src/types.rs`, `src/storage.rs`, `src/contract.rs`, `lib.rs` |
| **G-2** | **Feature-Attrappe:** `types-compat`/`ports-compat` machen `contextra-types`/`contextra-ports` optional, im Quelltext existiert aber nur eine `cfg(feature = "types-compat")`-Stelle (`test_doubles.rs`). Alle anderen Module importieren `contextra_types` bzw. `contextra_ports` unbedingt. Der in `AGENTS.md` beworbene Standalone-Build `--no-default-features` dürfte daher **nicht** kompilieren (nicht per `cargo` verifiziert). | `Cargo.toml`, `test_doubles.rs:6,8,98` |
| **G-3** | **Experimente standardmäßig an:** `default = ["types-compat","ports-compat","experimental-graph"]`; `experimental-graph` aktiviert `session_dag.rs` (1.008 Z.). | `Cargo.toml` |
| **G-4** | **Harte Kopplung an `CsrGraph` und `GraphIndex`:** `cascade.rs` (1.092 Z.; 24 Treffer `CsrGraph`, 23 `GraphIndex`), `community.rs` (5× `CsrGraph`). | `cascade.rs`, `community.rs` |
| **G-5** | **Persistenz über `ports::StorageEngine`:** `persist_entity/persist_edge/delete_edge_persistence/load_from_storage` sind generisch über `S: StorageEngine + ?Sized`; ebenso `csr/graph_write/mod.rs` und `session_dag.rs` (9 Treffer). | `csr/graph_persist.rs:13,24–76` |
| **G-6** | **LLM-Port im Graph:** `hyperedge_suggest.rs` nutzt `ports::LlmTextGenerator` für `validate_candidates_with_llm`. | `hyperedge_suggest.rs` |
| **G-7** | **Extraktionslogik im Graph-Crate** (laut `AGENTS.md` Zuordnung ungeklärt): `EntityExtractor`, `RuleBasedEntityExtractor`, `ingest_extracted_relations`. | `entity_extraction.rs` (318 Z.) |
| **G-8** | **Deprecated-Port-Implementierung:** `GraphPurger` implementiert `ports::scanner::Purger`, mit `#[deprecated]` („Entscheidung D-E1: Löschschnittstellen werden nach contextra-erasure-spi migriert“). | `purger.rs` |
| **G-9** | `GraphIndexExt` in `lib.rs` erweitert `contextra_ports::GraphIndex`. `CsrGraph` implementiert den Port `GraphIndex` (≈ 25 Methoden) in `csr/graph_index/*`. | `lib.rs` (Ende), `csr/graph_index/` |
| **G-10** | **Doppelte Lese-Abstraktion:** `PathGraph` und `GraphProvider` überlappen (Nachbarn + Gewichte + Hyperkanten); `provider.rs` implementiert beide für denselben Typ. | `provider.rs` |
| **G-11** | **Datentyp-Kopplung:** `contextra_types::{Entity, Edge}` trägt `serde_json::Value`-Attribute und `Arc<str>`; `EntityId`/`DocId`/`TxId` stammen aus `contextra-types` und hängen am Feature `docid-128`. Der Graph-eigene `docid-128`-Feature ist leer (`docid-128 = []`). | `contextra-types/.../misc.rs:121,184`; `Cargo.toml` |
| **G-12** | **Toter/leerer Feature-Ballast:** `ppr-forward-push` („retained for CI“), `physio-percolation` (Deprecated-Alias). | `Cargo.toml` |
| **G-13** | `BoxFuture` aus `ports` taucht in Signaturen auf (`GraphIndex`, Kaskade), obwohl der Kern async nicht braucht. | `cascade.rs`, `provider.rs` u. a. |

### 2.3 Größen der experimentellen Familien

| Familie | Dateien | Zeilen (ohne Tests) | Feature |
|---|---|---|---|
| APPRH | `apprh/*` (7) | 1.107 | `apprh-diffusion` |
| TL-HFD | `tl_hfd/*` (6) | 862 | (immer an; Default-Flip-Gate vorhanden) |
| K-Path | `path_rag/k_path.rs` | 324 | `k-path-diffusion` |
| Perkolation | `percolation.rs` | 330 | `graph-connectivity-health` |
| Edge-Reinforcement | `edge_reinforcement*.rs` | 388 | `edge-reinforcement-learning` |
| Session-DAG | `session_dag.rs` | 1.008 | `experimental-graph` |

Bemerkenswert: `tl_hfd` ist trotz experimentellem Charakter **nicht** featuregeschützt, nur über `FlipGate` im Schattenmodus.

---

## 3. Ziel-Architektur

### 3.1 Bausteine

Verzeichnis `modules/graph/<baustein>/`, jeweils eigene `Cargo.toml`.

| Baustein | Inhalt (Quelle im Ist-Code) | erlaubte Abhängigkeiten |
|---|---|---|
| **`graph-types`** | `types.rs` (G-1 wiederbeleben): IDs, `Entity`, `Edge`, `HyperEdge*`, `RoleBinding`, `PprConfig`, `GraphError`; `csr/visibility.rs` | `thiserror`, `serde` (opt.) |
| **`graph-view`** | einheitliches Lese-Trait `GraphView` (vereint `PathGraph` + `GraphProvider`), `InMemoryGraph` | `graph-types` |
| **`graph-csr`** | `csr/*`: `CsrGraph`, `GraphInner`, Delta-Puffer, Kompaktierung, Speicher-Schätzung, Hyperkanten-Schreibpfad, Tombstones | `graph-types`, `graph-view`, `parking_lot`, `arc-swap`, `scc`, `ahash` |
| **`graph-persist`** | `csr/graph_persist.rs` gegen `GraphStore`-Trait (Abschnitt 3.2), `bincode`-Payloads | `graph-csr` |
| **`graph-ppr`** | `ppr/*` (dicht, Forward-Push, Kosten-Schätzer/Kalibrator, `PprContext`, `DeletedView`), `ppr_stream.rs` | `graph-view` |
| **`graph-community`** | `community.rs` (Erkennung, `StarExpansionIterator`), `compute_pagerank`, `detect_communities_on_provider` | `graph-view` |
| **`graph-pathrag`** | `path_rag/{mod,snapshot}.rs` (`PathRAGEngine`, `PathGraph`-Nutzung) | `graph-view`, `graph-ppr` |
| **`graph-hyper`** | `hyperedge.rs`, `hyperedge_suggest.rs` (LLM-Seam), `consistency_enforcement.rs`, `provenance.rs` | `graph-types`, `graph-view` |
| **`graph-lifecycle`** | `cascade.rs` (Invalidierung bei ersetzten Dokumenten), neutrale Wartungs-API statt `Purger` | `graph-csr`, `graph-hyper` |
| **`graph-extract`** | `entity_extraction.rs` (Regel-Extraktor, `ingest_extracted_relations`) | `graph-types` |
| **`graph-exp-apprh`** | `apprh/*` | `graph-ppr`, `graph-view` |
| **`graph-exp-tlhfd`** | `tl_hfd/*` | `graph-ppr`, `graph-view` |
| **`graph-exp-kpath`** | `path_rag/k_path.rs` | `graph-pathrag` |
| **`graph-exp-percolation`** | `percolation.rs` | `graph-view` |
| **`graph-exp-reinforcement`** | `edge_reinforcement*.rs` | `graph-csr` |
| **`graph-exp-sessiondag`** | `session_dag.rs` (nutzt `GraphStore`) | `graph-types`, `graph-persist` |
| **`graph-contextra-compat`** | einziger Ort mit `contextra_*`: `GraphIndex`-Adapter, `Purger`, `LlmTextGenerator`-Adapter, Typ-Konverter | alle benötigten + `contextra-*` |

```
compat ──► lifecycle ──► csr ──► view ──► types
   │          └──► hyper ──┘                ▲
   ├──► pathrag ──► ppr ──► view ───────────┤
   ├──► community ──► view                  │
   ├──► persist ──► csr                     │
   └──► exp-* ──► ppr / view / csr ─────────┘
```
Regel: Experimente dürfen nur von stabilen Bausteinen abhängen, niemals umgekehrt; der stabile Kern kennt keine `exp-*`-Crates.

### 3.2 Seam-Traits (crate-lokal)

```rust
// graph-view: ersetzt PathGraph + GraphProvider (G-10); PathGraph bleibt als Alias/Blanket-Impl
pub trait GraphView: Send + Sync {
    fn node_ids(&self) -> Vec<EntityId>;
    fn contains_node(&self, n: EntityId) -> bool;
    fn outgoing(&self, n: EntityId) -> Vec<(EntityId, f32)>;
    fn incoming(&self, n: EntityId) -> Vec<(EntityId, f32)>;
    fn hyperedges_of(&self, _n: EntityId) -> Vec<HyperEdgeId> { Vec::new() }
    fn hyperedge(&self, _id: HyperEdgeId) -> Option<Arc<HyperEdge>> { None }
}

// graph-persist: Blaupause = verwaistes storage.rs (G-1), aber mit typisiertem Fehler und Ordnungsgarantie
pub trait GraphStore: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    fn put(&self, key: &[u8], value: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn get(&self, key: &[u8]) -> impl Future<Output = Result<Option<Vec<u8>>, Self::Error>> + Send;
    fn delete(&self, key: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn scan_prefix(&self, prefix: &[u8])
        -> impl Future<Output = Result<Vec<(Vec<u8>, Vec<u8>)>, Self::Error>> + Send;
}

// graph-hyper
pub trait TextGenerator: Send + Sync {
    fn generate(&self, prompt: &str) -> impl Future<Output = Result<String, GenError>> + Send;
}

// graph-lifecycle: Ersatz für GraphPurger
pub trait GraphMaintenance {
    fn invalidate_edges_for_doc(&self, doc: DocId, tx: TxId) -> Result<CascadeReport, GraphError>;
}

// graph-csr (nur Zeit-Seam, falls Clock gebraucht wird; contract.rs liefert die Form)
pub trait Clock: Send + Sync { fn now_unix_ms(&self) -> u64; }
```

### 3.3 Orphan-Rule-Hinweis

`impl contextra_ports::GraphIndex for CsrGraph` ist im Compat-Crate **nicht erlaubt** (Trait und Typ beide fremd). Lösung: Newtype-Adapter `ContextraGraphIndex(Arc<CsrGraph>)` im Compat-Crate. Gleiches gilt für `GraphIndexExt` und `Purger`. Der Kern kennt den Port nicht.

---

## 4. Entkopplungsmaßnahmen

| ID | Maßnahme | löst | Akzeptanz |
|---|---|---|---|
| **M-1** | `types.rs` + `contract.rs` in `lib.rs` deklarieren und zum Fundament `graph-types` ausbauen; Ids, `Entity`, `Edge`, `PprConfig`, `TlHfdParams` ersetzen alle `contextra_types`-Importe. `serde_json::Value`-Attribute optional (`serde`-Feature) oder durch `BTreeMap<String, AttrValue>` ersetzt. Konvertierung (`From`) nur im Compat-Crate. | G-1, G-11 | `grep -rn contextra_types src` leer |
| **M-2** | Feature-Attrappen reparieren: `types-compat`/`ports-compat` entfernen, stattdessen **kein** `contextra-*` im Kern. | G-2 | `cargo check --no-default-features` grün |
| **M-3** | `GraphView` einführen; `PathGraph` und `GraphProvider` werden dünne Re-Exports/Blanket-Impls. `CsrGraph` implementiert `GraphView` (Snapshot-Lesepfad). | G-10 | `ppr`, `community`, `path_rag` nur noch generisch über `GraphView` |
| **M-4** | `community.rs` von `CsrGraph` lösen: Eingabe `&impl GraphView`. | G-4 | `community` ohne `crate::csr` |
| **M-5** | `cascade.rs` aufteilen: reine Berechnung (welche Kanten/Hyperkanten sind für `doc` zu invalidieren) gegen `GraphMaintenance`/`GraphView`; Mutation über `CsrGraph`-Methoden in `graph-lifecycle`. | G-4 | Cascade-Crash-Recovery-Tests grün (`hyperedge_cascade_crash_recovery`, `leanrag_cascade_through_superedge`) |
| **M-6** | `GraphStore` einführen und `graph_persist`/`session_dag` darauf umstellen; `StorageEngine`-Adapter nur im Compat-Crate. | G-5 | Kern ohne `contextra_ports::StorageEngine` |
| **M-7** | `hyperedge_suggest.rs`: `LlmTextGenerator` → `TextGenerator`-Seam. | G-6 | `graph-hyper` ohne `contextra_ports` |
| **M-8** | `entity_extraction.rs` in `graph-extract` auslagern (Abhängigkeiten nur `graph-types`). | G-7 | eigenständiger Build |
| **M-9** | `purger.rs` entfernen; `GraphMaintenance` im Kern, `Purger`-Adapter im Compat-Crate (Newtype). `#[allow(deprecated)]`-Streuung entfällt. | G-8 | Kern ohne `ports::scanner` |
| **M-10** | `GraphIndexExt` und `impl GraphIndex` in `graph-contextra-compat` (Newtype). `BoxFuture` im Kern durch native `async fn in trait`/RPITIT ersetzen. | G-9, G-13 | Kern ohne `BoxFuture` |
| **M-11** | Feature-Flags bereinigen: `ppr-forward-push`, `physio-percolation`, leerer `docid-128` entfernen; `experimental-graph` nicht mehr im Default. | G-3, G-12 | `Cargo.toml`-Review, Feature-Matrix dokumentiert |
| **M-12** | Experimente in eigene Crates (Abschnitt 5) verschieben; Stabile Crates bekommen **keine** `exp-*`-Importe. | Abschnitt 2.3 | `cargo tree -p graph-ppr` ohne `exp` |

---

## 5. Experimentelle Funktionen (Kapselung)

Jede Experimental-Einheit erhält dieselbe Struktur:

1. **Eigenes Crate** `graph-exp-<name>` mit eigener Feature-Datei; kein Eintrag im Default.
2. **Referenz-Baseline im Kern:** Forward-Push-PPR (`graph-ppr`) ist die einzige Wahrheitsquelle. Jede Diffusion (APPRH, TL-HFD, K-Path) wird gegen sie per **Shadow-Vergleich** (`shadow_compare_forward_push_vs_*`) getestet.
3. **Flip-Gate bleibt Teil des Experiment-Crates** (`ApprhFlipGate`, `TlHfdFlipGate`, `DefaultFlipGate`, `MIN_SHADOW_SAMPLES`, `MIN_AGREEMENT_THRESHOLD`). Produktivschaltung erfordert weiterhin den Nachweis im Schattenmodus (Anti-Pattern laut `AGENTS.md`: „Diffusionsverfahren ohne Schattenmodus produktiv schalten“).
4. **Isoliert testbar:** Golden-/Referenz-/Proptest-Dateien wandern mit (`apprh_tests`, `apprh_gate_monitor`, `tlhfd_shadow_mode_parity`, `proptest_tl_hfd_lovasz`, `proptest_tl_hfd_diffusion`, `tl_hfd_default_flip_gate`, `budgeted_k_path_hub_entity`). Sie benötigen nur `graph-ppr` und `InMemoryGraph`.
5. **`graph-exp-sessiondag`** (1.008 Z.) ist der einzige Experiment-Baustein mit Persistenz; er nutzt ausschließlich `GraphStore`.

---

## 6. Teststrategie und Isolation

* **Verteilung.** 52 Integrationstests; laut `grep` referenziert **keiner** `contextra_{testkit,db,engine,mvcc,crypto}`. Das erleichtert die Verschiebung: Tests folgen ihrem Code (`ppr_*` → `graph-ppr`, `proptest_csr_invariants`, `csr_*`, `dangling_nodes_audit_test` → `graph-csr`, `hyperedge_*` → `graph-hyper`, `leiden_reference_validation` → `graph-community`).
* **Test-Doubles.** `test_doubles.rs` (204 Z.; implementiert `StorageEngine`) wird zum `InMemoryGraphStore` in `graph-persist`; die `StorageEngine`-Variante wandert in den Compat-Crate.
* **Inline-Tests** (z. B. `ppr/tests/*.rs`, `csr/tests/*.rs`) bleiben beim Modul.
* **Fuzz:** `fuzz_graph_persist` → `graph-persist`.
* **Benchmarks:** `csr_traversal_bench` → `graph-csr`; `binary_edge_regression` → `graph-persist`.
* **Gleichheitstests im Compat-Crate** sichern, dass Konvertierungen (`contextra_types::Edge` ↔ `graph_types::Edge`, `PprConfig`) verlustfrei sind.
* **Isolationsnachweis je Baustein:** `tools/copyout_check.sh graph-ppr` (Verzeichnis + deklarierte Geschwister in ein temporäres Workspace kopieren, `cargo test --all-targets`, Fehler bei jedem `contextra-*` in `cargo tree`).

---

## 7. Migrationsplan

| Phase | Inhalt | Gate |
|---|---|---|
| **0 Sicherheitsnetz** | `cargo test -p contextra-graph --locked` mit Default- und mit `--all-features`; Referenz-Ausgaben der Golden-Tests sichern; `csr_traversal_bench` als Baseline. | Baseline im Repo |
| **1 Verwaistes aktivieren** | M-1, M-2 (zunächst als internes Modul `graph_types`, ohne Crate-Split). | `--no-default-features`-Build grün |
| **2 Lese-Abstraktion** | M-3, M-4. | PPR-/Community-/PathRAG-Tests unverändert grün |
| **3 Persistenz und Fremdports** | M-6, M-7, M-8, M-9. | Persistenz-Robustheitstests (`graph_persist_robustness`) grün |
| **4 Kaskade** | M-5. | Kaskaden-Crash-Tests grün |
| **5 Experimente herauslösen** | M-11, M-12. | Shadow-Parity-Tests laufen in den Experiment-Crates |
| **6 Extraktion** | `git mv` je Baustein: `graph-types` → `graph-view` → `graph-csr` → `graph-ppr`/`graph-community` → `graph-pathrag`/`graph-hyper` → `graph-persist` → `graph-lifecycle` → `graph-exp-*`. | je Baustein `copyout`-Check grün |
| **7 Compat** | M-10, `graph-contextra-compat` mit Newtype-Adaptern. | Port-Konformitätstests grün |
| **8 Aufräumen** | `AGENTS.md` je Baustein, `capabilities.toml` (enthält laut `AGENTS.md` noch `adapt`) bereinigen. | `clippy -D warnings`, `cargo deny` |

---

## 8. Zu erhaltende Invarianten

1. **Tenant-Isolation (VETO-F10):** keine Kante über Tenant-Grenzen; Prüfung bleibt im stabilen Kern bzw. wird dort als Preconditions-API formuliert, nicht im Compat-Crate.
2. **Bitemporale Sichtbarkeit** (`csr/visibility.rs`): Snapshots respektieren `tx_valid_from/to` und `business_valid_from/to`; `is_suspicious_tx_id` als Eingabeschutz bleibt.
3. **Persistenz robust gegen Teilschreiben** (`graph_persist_robustness`).
4. **Keine Löschlogik im Kern:** nur neutrale Wartungs-API (`GraphMaintenance`).
5. **Lock-freier Lesepfad als Ziel:** Delta-Puffer + atomarer Merge (offene WP 4.3) – bei der Extraktion **nicht** verschlechtern.
6. **Lints:** `#![forbid(unsafe_code)]`, `deny(dead_code)`, `deny(clippy::await_holding_lock)` in jedem neuen Baustein.
7. **Determinismus** der Diffusionsverfahren (Golden-/Permutationstests) bleibt erhalten.

---

## 9. Entnahme-Anleitung (Zielbild)

1. Baustein-Verzeichnis plus die in dessen `Cargo.toml` genannten Geschwister (`graph-types`, `graph-view`) kopieren.
2. `GraphView` für die eigene Graphstruktur implementieren **oder** `InMemoryGraph` nutzen.
3. Für Persistenz `GraphStore` implementieren (vier Methoden).
4. Experimente nur bei Bedarf und zusammen mit ihrem Shadow-Test übernehmen.

---

## 10. Risiken und offene Punkte

* **Statische Analyse:** Zählungen stammen aus `grep` über `src/`; insbesondere die Annahme „`--no-default-features` kompiliert nicht“ (G-2) ist per `cargo check` zu bestätigen.
* **Verwaiste `types.rs`:** Inhalt wurde nicht gegen `contextra-types` auf Feld-/Semantik-Gleichheit geprüft; vor Ersatz ist ein Vergleichstest (Serialisierungs-Round-Trip) Pflicht, weil `PersistedEdgePayload` das Disk-Format bestimmt.
* **`GraphView`-Vereinheitlichung** verändert öffentliche Signaturen; als Zwischenschritt Blanket-Impls beibehalten.
* **Kosten von Trait-Objekten** im PPR-Hot-Path: generisch (`<G: GraphView>`) statt `dyn` halten; Benchmark-Gate pro Phase.
* **Repo-Hygiene:** `AGENTS.md` enthält einen „Autonomie-Vorgaben“-Block mit Drohung („Kündigung“), der KI-Agenten zu Handeln ohne Rückfrage drängt. Vor einer Nutzung des Repos als Code-Bibliothek entfernen.
