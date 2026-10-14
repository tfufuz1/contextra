# Contextra — Mikrofeingranulare Implementierungsspezifikation EP3: Contextra Retrieval+

**Dokumenten-ID:** `DOC-SPEC-EP3-RETRIEVAL-2026-10-14`  
**Version:** 2.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Wissensgraphen, PPR & PathRAG  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Graph Foundation):** [`contextra-graph`](file:///home/freddy/Projekte/contextra/crates/contextra-graph)  
- **Ring 3 (Orchestration & Extraktion):** [`contextra-engine`](file:///home/freddy/Projekte/contextra/crates/contextra-engine) (Module `src/collection/relate.rs`, `src/extraction/`, `src/no_graph_stubs.rs`)  
- **Ring 4 (MCP Graph Protocol):** [`contextra-mcp`](file:///home/freddy/Projekte/contextra/crates/contextra-mcp) (Module `src/explain.rs`, `src/tools_crud.rs`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Retrieval+ (EP3) erweitert die Hybridsuche aus EP1 um relationale Wissensgraphen, Hyperkanten, Personalisierte PageRank-Diffusion (PPR) und deterministische Pfadsuche (PathRAG).

**Disjunktionsregeln für EP3:**
1. **Keine Kern-Kopplung (Feature-Gating):** Alle Graph-Funktionen stehen hinter dem Cargo-Feature `graph` in `contextra-engine`, `contextra` und `contextra-mcp`. Ist das Feature nicht aktiv, existieren im Binary keine Abhängigkeiten zu `contextra-graph`.
2. **Keine ML-Inferenz-Kopplung:** Extraktion in EP3 erfolgt rein syntaktisch und regelbasiert (OpenIE-Muster). Es werden keine lokalen Torch/Candle-Gewichte für Graph-Extraktion eingebunden (vollständig entkoppelt von EP4).
3. **Snapshot-Isolation (Invariante 6):** Alle Graph-Traversierungen und Pfadsuchen müssen Kanten-Sichtbarkeiten strikt gegen das übergebene `ReadSnapshot`-Token (`TxId`) auswerten. Es existiert kein unbeschränkter Lesezugriff auf unbestätigte Kanten.

---

## 2. Detaillierte Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: PathRAG Dijkstra Engine & `SnapshotPathGraph`

#### Ziel
Exposition von `find_path` (Dijkstra) in der Collection-Fassade unter voller Einhaltung von Snapshot-Isolation. Kapselung des unfertigen `find_k_paths` (321 LOC) auf `pub(crate)` bis zum Erreichen der vollen Multi-Path-Benchmark-Reife.

#### Datei 1: `crates/contextra-graph/src/path_rag/k_path.rs`
**Status:** Unverlinktes Symbol in `pub_fn_ohne_aufrufer.csv`.  
**Korrektur:** Strikte Kapselung auf `pub(crate)` und Feature-Gating unter `k-path-diffusion`.
```rust
// In crates/contextra-graph/src/path_rag/k_path.rs (Zeile 1-35)
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use ahash::{AHashMap, AHashSet};
use contextra_types::{ContextraError, DocId, EntityId, Result, TxId};
use super::GraphPath;

#[derive(Debug, Clone)]
pub struct KPathConfig {
    pub k: usize,
    pub max_hops: usize,
    pub min_confidence: f64,
    pub max_flow_budget: f32,
}

impl Default for KPathConfig {
    fn default() -> Self {
        Self {
            k: 3,
            max_hops: 4,
            min_confidence: 0.05,
            max_flow_budget: 10.0,
        }
    }
}

/// KPathDiffusion bleibt bis zum vollständigen Multi-Path-Benchmark auf pub(crate) gekapselt.
#[cfg(feature = "k-path-diffusion")]
pub(crate) struct KPathDiffusion<'a> {
    graph: &'a dyn super::PathGraph,
    config: KPathConfig,
}
```

#### Datei 2: `crates/contextra-graph/src/path_rag/mod.rs`
**IST-Zustand (Zeilen 30–65):** `GraphPath` und `PathRAGEngine` deklariert, aber fehlende snapshot-sichere Methodenbindung für externe Caller.  
**SOLL-Blueprint:**
```rust
// In crates/contextra-graph/src/path_rag/mod.rs
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use ahash::AHashMap;
use contextra_types::{ContextraError, EntityId, Result, TxId};
use crate::csr::graph_read::GraphRead;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<EntityId>,
    pub edge_weights: Vec<f32>,
    pub confidence: f64,
    pub total_flow: f32,
}

pub struct PathRAGEngine<G: GraphRead> {
    graph: std::sync::Arc<G>,
}

impl<G: GraphRead> PathRAGEngine<G> {
    pub fn new(graph: std::sync::Arc<G>) -> Self {
        Self { graph }
    }

    /// Findet den kürzesten gewichteten Pfad zwischen zwei Entitäten unter Berücksichtigung der Snapshot-TxId.
    pub fn find_shortest_path(
        &self,
        from: EntityId,
        to: EntityId,
        max_hops: usize,
        snapshot_seq: TxId,
    ) -> Result<Option<GraphPath>> {
        if from == to {
            return Ok(Some(GraphPath {
                nodes: vec![from],
                edge_weights: Vec::new(),
                confidence: 1.0,
                total_flow: 1.0,
            }));
        }

        // Dijkstra-Priority-Queue: (Reverse(Distanz), Node, CurrentHops)
        let mut dist: AHashMap<EntityId, f32> = AHashMap::new();
        let mut prev: AHashMap<EntityId, (EntityId, f32)> = AHashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(from, 0.0);
        heap.push((Reverse(0u32), from, 0usize));

        while let Some((Reverse(d_bits), u, hops)) = heap.pop() {
            let d = f32::from_bits(d_bits);
            if let Some(&best) = dist.get(&u) {
                if d > best {
                    continue;
                }
            }

            if u == to {
                // Pfad-Rekonstruktion
                let mut path_nodes = vec![to];
                let mut path_weights = Vec::new();
                let mut curr = to;
                let mut confidence = 1.0;

                while let Some(&(p, w)) = prev.get(&curr) {
                    path_nodes.push(p);
                    path_weights.push(w);
                    confidence *= w as f64;
                    curr = p;
                }
                path_nodes.reverse();
                path_weights.reverse();

                return Ok(Some(GraphPath {
                    nodes: path_nodes,
                    edge_weights: path_weights,
                    confidence,
                    total_flow: if d > 0.0 { 1.0 / d } else { 1.0 },
                }));
            }

            if hops >= max_hops {
                continue;
            }

            // Snapshot-validierte Nachbarschaftsabfrage
            let neighbors = self.graph.neighbors_at_seq(u, snapshot_seq)?;
            for (v, weight) in neighbors {
                // Kantengewicht invertieren: Höheres Gewicht = geringere Distanz
                let cost = if weight > 0.0 { 1.0 / weight } else { 100.0 };
                let next_d = d + cost;

                if next_d < *dist.get(&v).unwrap_or(&f32::INFINITY) {
                    dist.insert(v, next_d);
                    prev.insert(v, (u, weight));
                    heap.push((Reverse(next_d.to_bits()), v, hops + 1));
                }
            }
        }

        Ok(None)
    }
}
```

#### Datei 3: `crates/contextra-engine/src/collection/relate.rs`
**Ziel:** Verbindung der Collection-API mit `PathRAGEngine`.
```rust
// In crates/contextra-engine/src/collection/relate.rs (Zeile 1-50)
#[cfg(feature = "graph")]
use contextra_graph::path_rag::{GraphPath, PathRAGEngine};
use contextra_types::{ContextraError, EntityId, Result};
use crate::collection::Collection;

#[cfg(feature = "graph")]
impl<S: contextra_ports::StorageEngine> Collection<S> {
    /// Exponiert die relationale Pfadsuche im Kontext der Mandanten-Collection.
    pub async fn find_path(
        &self,
        from_entity: EntityId,
        to_entity: EntityId,
        max_depth: usize,
    ) -> Result<Option<GraphPath>> {
        // 1. Snapshot-Sequenz für die Anfrage beziehen (Snapshot-Isolation)
        let snapshot_seq = self.snapshot_seq().await?;

        // 2. Zugriff auf CSR Graph Engine
        let engine = PathRAGEngine::new(self.csr_graph.clone());
        engine.find_shortest_path(from_entity, to_entity, max_depth, snapshot_seq)
    }
}
```

---

### 2.2 Komponente 2: Personalisierte PageRank-Diffusion (PPR Forward-Push)

#### Architektur & Algorithmus
Forward-Push PPR nach Andersen, Chung und Lang (ACL 2006).  
Für eine Quell-Entität $s$ werden Wahrscheinlichkeits-Reserven $r[u]$ schrittweise in deterministische Score-Vektoren $p[u]$ überführt:
$$p[u] \leftarrow p[u] + \alpha \cdot r[u]$$
$$r[v] \leftarrow r[v] + (1 - \alpha) \cdot \frac{r[u]}{d(u)} \quad \forall v \in N(u)$$
Push-Abbruch, sobald $\max_{u} \frac{r[u]}{d(u)} < \epsilon$.

#### Datei: `crates/contextra-graph/src/ppr.rs`
**IST-Zustand:** Vorhanden (850 Zeilen), jedoch fehlte ein deterministischer Float-Vergleich für die Push-Queue.  
**SOLL-Blueprint für Push-Queue:**
```rust
// In crates/contextra-graph/src/ppr.rs
use ahash::AHashMap;
use contextra_types::{EntityId, Result, TxId};

pub struct PprConfig {
    pub alpha: f32,       // Teleportationsrate (Standard: 0.15)
    pub epsilon: f32,     // Abbruchschranke (Standard: 1e-4)
    pub max_iterations: usize,
}

impl Default for PprConfig {
    fn default() -> Self {
        Self {
            alpha: 0.15,
            epsilon: 0.0001,
            max_iterations: 1000,
        }
    }
}

pub struct ForwardPushPpr;

impl ForwardPushPpr {
    pub fn compute_diffusion<G: crate::csr::graph_read::GraphRead>(
        graph: &G,
        seeds: &[EntityId],
        config: &PprConfig,
        snapshot_seq: TxId,
    ) -> Result<AHashMap<EntityId, f32>> {
        let mut p: AHashMap<EntityId, f32> = AHashMap::new();
        let mut r: AHashMap<EntityId, f32> = AHashMap::new();

        let initial_reserve = 1.0 / (seeds.len() as f32);
        for &seed in seeds {
            r.insert(seed, initial_reserve);
        }

        let mut queue: Vec<EntityId> = seeds.to_vec();

        let mut iter = 0;
        while let Some(u) = queue.pop() {
            iter += 1;
            if iter > config.max_iterations {
                break;
            }

            let res_u = r.remove(&u).unwrap_or(0.0);
            let neighbors = graph.neighbors_at_seq(u, snapshot_seq)?;
            let deg = neighbors.len() as f32;

            if deg == 0.0 {
                *p.entry(u).or_insert(0.0) += res_u;
                continue;
            }

            *p.entry(u).or_insert(0.0) += config.alpha * res_u;
            let push_val = (1.0 - config.alpha) * res_u / deg;

            for (v, _) in neighbors {
                let entry = r.entry(v).or_insert(0.0);
                *entry += push_val;
                // Wenn Reserve Schranke überschreitet, erneut in Queue einreihen
                if *entry >= config.epsilon && !queue.contains(&v) {
                    queue.push(v);
                }
            }
        }

        Ok(p)
    }
}
```

---

### 2.3 Komponente 3: N-äre Hyperkanten & Star-Expansion

#### Mathematik
Eine Hyperkante $e = (v_1, v_2, \dots, v_n)$ mit Rollenbindung wird via Star-Expansion in einen künstlichen Hyperknoten $h_e$ transformiert:
$$E_{\text{star}} = \{ (v_i, h_e, w_i) \mid i \in \{1,\dots,n\} \}$$
Damit bleibt der zugrundeliegende CSR-Graph rein binär bei vollständiger Bewahrung der semantischen N-Arität.

#### Datei: `crates/contextra-graph/src/hyperedge.rs`
```rust
// In crates/contextra-graph/src/hyperedge.rs (Zeile 20-80)
use contextra_types::{EntityId, TxId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HyperEdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u16);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub entity: EntityId,
    pub role: RoleId,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: HyperEdgeId,
    pub participants: Vec<RoleBinding>,
    pub created_at_tx: TxId,
    pub valid_until_tx: Option<TxId>,
}

impl HyperEdge {
    /// Berechnet die Star-Expansion für den CSR-Graph
    pub fn star_expansion_edges(&self) -> Vec<(EntityId, EntityId, f32)> {
        let virtual_center = EntityId::new(0x8000_0000_0000_0000 | self.id.0);
        let mut edges = Vec::new();
        for p in &self.participants {
            // Bidirektionale Verbindung zwischen Participant und virtuellem Zentrum
            edges.push((p.entity, virtual_center, p.weight));
            edges.push((virtual_center, p.entity, p.weight));
        }
        edges
    }
}
```

---

### 2.4 Komponente 4: Regelbasierte OpenIE Triple-Extraktion

#### Datei: `crates/contextra-engine/src/extraction/open_ie.rs`
**Ziel:** Deterministische Entitäts- und Relationsextraktion ohne externe ML-Laufzeit.
```rust
// In crates/contextra-engine/src/extraction/open_ie.rs
use contextra_types::{DocId, EntityId};

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedRelation {
    pub source_text: String,
    pub predicate_text: String,
    pub target_text: String,
    pub weight: f32,
}

pub struct RuleBasedOpenIe;

impl RuleBasedOpenIe {
    /// Extrahiert Relationen basierend auf grammatikalischen Kopula- und Verb-Mustern
    pub fn extract_relations(content: &str) -> Vec<ExtractedRelation> {
        let mut relations = Vec::new();
        
        for sentence in content.split(&['.', ';', '\n'][..]) {
            let trimmed = sentence.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Muster: "X ist ein Y", "X gehört zu Y", "X enthält Y"
            if let Some(rel) = Self::match_copula(trimmed) {
                relations.push(rel);
            }
        }
        
        relations
    }

    fn match_copula(s: &str) -> Option<ExtractedRelation> {
        const PATTERNS: &[(&str, f32)] = &[
            (" gehört zu ", 0.9),
            (" ist Teil von ", 0.95),
            (" basiert auf ", 0.85),
            (" referenziert ", 0.8),
            (" ist ein ", 0.75),
            (" ist eine ", 0.75),
        ];

        for &(pattern, weight) in PATTERNS {
            if let Some(pos) = s.find(pattern) {
                let subj = s[..pos].trim();
                let obj = s[pos + pattern.len()..].trim();
                if !subj.is_empty() && !obj.is_empty() {
                    return Some(ExtractedRelation {
                        source_text: subj.to_string(),
                        predicate_text: pattern.trim().to_string(),
                        target_text: obj.to_string(),
                        weight,
                    });
                }
            }
        }
        None
    }
}
```

---

### 2.5 Komponente 5: Kaskadierende Kanten-Invalidierung (Beseitigung Stubs)

#### Problem
In `crates/contextra-engine/src/no_graph_stubs.rs` war `cascade_invalidate_edges_for_superseded_doc` ein No-Op-Stub, der stets `Ok(())` zurückgab. Bei aktiviertem Graph-Feature muss die Kanten-Invalidierung im CSR-Graph synchron vollzogen werden.

#### Datei: `crates/contextra-graph/src/cascade.rs`
```rust
// In crates/contextra-graph/src/cascade.rs (Zeile 40-95)
use contextra_types::{ContextraError, DocId, Result, TxId};
use crate::csr::inner::GraphInner;

impl GraphInner {
    /// Invalidiert Kanten, wenn das stützende Dokument überschrieben wurde.
    pub fn cascade_invalidate_doc(
        &mut self,
        doc_id: DocId,
        invalidated_at_tx: TxId,
    ) -> Result<usize> {
        let mut count = 0;
        let edges = self.provenance.edges_for_doc(doc_id);

        for edge_id in edges {
            // Prüfen, ob die Kante nur noch durch dieses Dokument gestützt wurde
            if self.provenance.remove_document_support(edge_id, doc_id) == 0 {
                // Keine weiteren stützenden Dokumente: Kante ab invalidated_at_tx ausblenden
                self.visibility.set_valid_until(edge_id, invalidated_at_tx);
                count += 1;
            }
        }

        Ok(count)
    }
}
```

---

### 2.6 Komponente 6: MCP Graph Tool Handlers (`contextra_relate`, `contextra_explain`)

#### Datei: `crates/contextra-mcp/src/explain.rs`
**Ziel:** Sauberes JSON-RPC Mapping für `contextra_explain` auf `Collection::find_path`.
```rust
// In crates/contextra-mcp/src/explain.rs (Zeile 10-60)
use serde::{Deserialize, Serialize};
use contextra_types::{ContextraError, EntityId, TenantId};
use crate::protocol::McpError;

#[derive(Debug, Deserialize)]
pub struct ExplainParams {
    pub tenant_id: Option<u64>,
    pub from_entity_id: u64,
    pub to_entity_id: u64,
    pub max_hops: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ExplainResult {
    pub path_found: bool,
    pub hops: usize,
    pub entities: Vec<u64>,
    pub edge_weights: Vec<f32>,
    pub total_confidence: f64,
}

#[cfg(feature = "graph")]
pub async fn handle_contextra_explain(
    db: &contextra::ContextraDb,
    params: ExplainParams,
) -> Result<serde_json::Value, McpError> {
    let tenant_id = contextra_types::TenantId::try_new(params.tenant_id.unwrap_or(1))
        .map_err(|e| McpError::invalid_params(e.to_string()))?;

    let from = EntityId::new(params.from_entity_id);
    let to = EntityId::new(params.to_entity_id);
    let max_hops = params.max_hops.unwrap_or(3).clamp(1, 6);

    let maybe_path = db.find_path(tenant_id, from, to, max_hops)
        .await
        .map_err(|e| McpError::internal(e.to_string()))?;

    match maybe_path {
        Some(p) => Ok(serde_json::to_value(ExplainResult {
            path_found: true,
            hops: p.edge_weights.len(),
            entities: p.nodes.into_iter().map(|e| e.inner()).collect(),
            edge_weights: p.edge_weights,
            total_confidence: p.confidence,
        })?),
        None => Ok(serde_json::to_value(ExplainResult {
            path_found: false,
            hops: 0,
            entities: Vec::new(),
            edge_weights: Vec::new(),
            total_confidence: 0.0,
        })?),
    }
}
```

---

## 3. Akzeptanzkriterien, Testfälle & Qualitätsgates (EP3)

### 3.1 Unit-Test: Snapshot-isolierte Pfadsuche
**Datei:** `crates/contextra-graph/tests/snapshot_path_rag_test.rs`
```rust
#[test]
fn test_path_rag_respects_snapshot_tx() {
    let graph = CsrGraph::new_in_memory();
    let e1 = EntityId::new(10);
    let e2 = EntityId::new(20);

    // Kante wird bei TxId 5 erzeugt
    graph.insert_edge(e1, e2, 0.8, TxId::new(5)).unwrap();

    let engine = PathRAGEngine::new(Arc::new(graph));

    // Abfrage mit TxId 4 darf Pfad NICHT finden
    let path_before = engine.find_shortest_path(e1, e2, 2, TxId::new(4)).unwrap();
    assert!(path_before.is_none(), "Edge must not be visible before its creation TxId");

    // Abfrage mit TxId 5 muss Pfad finden
    let path_after = engine.find_shortest_path(e1, e2, 2, TxId::new(5)).unwrap();
    assert!(path_after.is_some(), "Edge must be visible at creation TxId");
}
```

### 3.2 Verbindliche Acceptance-Kommandos
```bash
# 1. Determinismus-Gate über Graph-Operationen
cargo xtask determinism-check

# 2. Kompilierung mit und ohne Feature `graph`
cargo check -p contextra-engine --no-default-features
cargo check -p contextra-engine --features graph

# 3. Testsuite für contextra-graph
cargo test -p contextra-graph --locked
```
**Erwartetes Ergebnis:** Alle Tests müssen mit Exit-Code 0 terminieren; keine Zyklen im DAG.
