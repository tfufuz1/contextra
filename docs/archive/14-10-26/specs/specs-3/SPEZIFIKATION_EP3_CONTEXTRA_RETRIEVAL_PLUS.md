superseded by CONTEXTRA_ENDPRODUKT_SPEZIFIKATION.md

# Contextra — Implementierungsspezifikation EP3: Contextra Retrieval+ (Graph, OpenIE & Conformal PathRAG)

**Dokumenten-ID:** `DOC-SPEC-EP3-RETRIEVAL-2026-10-14`  
**Version:** 2.0 (Punktgenaue Feinspezifikation) · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Wissensgraph & Conformal PathRAG  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (CSR-Graph & Hyperkanten):** [`contextra-graph`](../../../../../crates/contextra-graph)
- **Ring 1 (Ports):** [`contextra-ports`](../../../../../crates/contextra-ports) (Traits: `GraphEngine`, `PathFinder`, `Calibrator`)
- **Ring 2 (Fusion & Conformal Ranking):** [`contextra-rank`](../../../../../crates/contextra-rank)
- **Ring 3 (Hybride Multi-Signal-Suche):** [`contextra-engine`](../../../../../crates/contextra-engine), [`contextra-db`](../../../../../crates/contextra-db)
- **Ring 4 (Fassade & MCP):** [`contextra`](../../../../../crates/contextra), [`contextra-mcp`](../../../../../crates/contextra-mcp) (Tools `contextra_relate`, `contextra_explain`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Retrieval+ (EP3) erweitert die Vektor- und Volltextsuche um strukturierte Wissensgraphen, Multi-Hop-Pfadsuche (`PathRAG`), N-äre Hyperkanten und mathematisch garantierte Konformitätskalibrierung.

**Disjunktionsregeln für EP3:**
1. **Keine direkte Storage-Abhängigkeit:** `contextra-graph` ist ein Ring-0-Crate und darf **weder** `contextra-store`, `contextra-engine`, noch `tokio` als Abhängigkeit führen. Sämtliche Graphmutationen erfolgen im Speicher (CSR-Repräsentation) und werden deterministisch aus WAL/SSTables rehydriert.
2. **Snapshot-Isolation auf Graphen:** Graphabfragen (`find_path`, `k_path`, `ppr`) müssen deterministisch den exakten Snapshot-Stand (`TxId` / `as_of`) respektieren (AGT-GRAPH-001).
3. **Deterministische Sortierung (P28 & check-nan-hot-loop):** Alle Sortierungen von Distanzen, PageRank-Werten und Konformitäts-Scores müssen zwingend über `f64::total_cmp` / `f32::total_cmp` erfolgen. `partial_cmp().unwrap()` ist strikt verboten.

---

## 2. Befund- und Korrekturmatrix EP3

| ID | Datei | Zeile(n) | Problem & Befund | Exakte Korrekturmaßnahme |
|---|---|---|---|---|
| **W5** | `crates/contextra-graph/src/path_rag/k_path.rs` | 1–321 | `find_k_paths` existiert vollständig, hat aber keinen Produktionsaufrufer (`unwired-ports`). | An `contextra-engine::search::hybrid_search` anbinden; Pfad-Scores in Ranking-Pipeline einspeisen. |
| **CONFORMAL-01** | `crates/contextra-rank/src/calibration/` | diverse | Platt- und PAVA-Skalierer setzen parametrische Verteilungsannahmen voraus und bieten keine Coverage-Garantie. | `ConformalCalibrator` als Standard-Kalibrierer für Multi-Signal-Fusion etablieren. |
| **NAN-DET-01** | `crates/contextra-rank/src/fusion/` | diverse | `partial_cmp` in Sortierschleifen birgt Non-Determinismus bei NaNs. | Durchgängig `f64::total_cmp` erzwingen (P28 / `check-nan-hot-loop`). |
| **GRAPH-ORPHAN** | `crates/contextra-graph/src/cascade.rs` | 45 | `GraphGarbageCollection` unverdrahtet (Orphan). | Im Hintergrund-Worker `start_orphan_reaper` zur Kantenbereinigung nach Dokumentenlöschung aufrufen. |
| **HYPER-01** | `crates/contextra-graph/src/hyperedge.rs` | neu | N-äre Relationen (> 2 Entitäten) konnten im CSR-Format nicht nativ modelliert werden. | Star-Expansion mit virtuellen Centroid-Knoten deterministisch abbilden. |

---

## 3. Punktgenaue Schnittstellen & Algorithmen

### 3.1 Budgeted k-Path Diffusion (`crates/contextra-graph/src/path_rag/k_path.rs`)
```rust
// In crates/contextra-graph/src/path_rag/k_path.rs
use contextra_types::{ContextraError, NodeId, Result, TxId};
use std::collections::{BinaryHeap, HashSet};

#[derive(Debug, Clone)]
pub struct PathResult {
    pub nodes: Vec<NodeId>,
    pub relevance_score: f64,
    pub diversity_score: f64,
    pub combined_score: f64,
}

pub trait KPathDiffusion: Send + Sync {
    fn find_k_paths(
        &self,
        seeds: &[NodeId],
        k: usize,
        node_budget: usize,
        diversity_weight: f64,
        as_of: TxId,
    ) -> Result<Vec<PathResult>>;
}

impl<G: crate::csr::CsrGraphAccess> KPathDiffusion for G {
    fn find_k_paths(
        &self,
        seeds: &[NodeId],
        k: usize,
        node_budget: usize,
        diversity_weight: f64,
        as_of: TxId,
    ) -> Result<Vec<PathResult>> {
        if seeds.is_empty() || k == 0 || node_budget == 0 {
            return Ok(Vec::new());
        }

        let mut discovered_paths: Vec<PathResult> = Vec::with_capacity(k);
        let mut visited_global: HashSet<NodeId> = HashSet::with_capacity(node_budget);

        for &seed in seeds {
            if visited_global.len() >= node_budget {
                break;
            }

            // Dijkstra-Expansion unter Berücksichtigung von as_of Snapshot-Isolation
            let neighbors = self.get_neighbors_at(seed, as_of)?;
            for neighbor in neighbors {
                if visited_global.contains(&neighbor.target) {
                    continue;
                }

                let relevance = neighbor.weight as f64;
                let overlap = if visited_global.is_empty() {
                    0.0
                } else {
                    visited_global.intersection(&neighbor.attributes).count() as f64
                        / visited_global.len() as f64
                };
                let combined = relevance - (diversity_weight * overlap);

                discovered_paths.push(PathResult {
                    nodes: vec![seed, neighbor.target],
                    relevance_score: relevance,
                    diversity_score: 1.0 - overlap,
                    combined_score: combined,
                });

                visited_global.insert(neighbor.target);
                if discovered_paths.len() >= k {
                    break;
                }
            }
        }

        // Deterministische Sortierung nach kombiniertem Score via total_cmp (NAN-DET-01)
        discovered_paths.sort_by(|a, b| b.combined_score.total_cmp(&a.combined_score));
        discovered_paths.truncate(k);
        Ok(discovered_paths)
    }
}
```

### 3.2 Snapshot-Isolated Dijkstra Shortest Path (`crates/contextra-graph/src/dijkstra.rs`)
```rust
// In crates/contextra-graph/src/dijkstra.rs
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use contextra_types::{ContextraError, NodeId, Result, TxId};

#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f64,
    node: NodeId,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Umgekehrte Ordnung für Min-Heap, deterministisch via total_cmp
        other.cost.total_cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn shortest_path<G: crate::csr::CsrGraphAccess>(
    graph: &G,
    start: NodeId,
    goal: NodeId,
    as_of: TxId,
) -> Result<Option<(Vec<NodeId>, f64)>> {
    let mut dist = std::collections::HashMap::new();
    let mut prev = std::collections::HashMap::new();
    let mut heap = BinaryHeap::new();

    dist.insert(start, 0.0);
    heap.push(State { cost: 0.0, node: start });

    while let Some(State { cost, node }) = heap.pop() {
        if node == goal {
            let mut path = Vec::new();
            let mut curr = goal;
            path.push(curr);
            while let Some(&p) = prev.get(&curr) {
                path.push(p);
                curr = p;
            }
            path.reverse();
            return Ok(Some((path, cost)));
        }

        if let Some(&best) = dist.get(&node) {
            if cost > best {
                continue;
            }
        }

        for edge in graph.get_edges_at(node, as_of)? {
            let next_cost = cost + edge.weight as f64;
            let is_better = match dist.get(&edge.target) {
                Some(&current_best) => next_cost < current_best,
                None => true,
            };

            if is_better {
                dist.insert(edge.target, next_cost);
                prev.insert(edge.target, node);
                heap.push(State { cost: next_cost, node: edge.target });
            }
        }
    }

    Ok(None)
}
```

### 3.3 Forward-Push Personalized PageRank (`crates/contextra-graph/src/ppr.rs`)
```rust
// In crates/contextra-graph/src/ppr.rs
use std::collections::HashMap;
use contextra_types::{ContextraError, NodeId, Result, TxId};

pub struct PprEngine {
    pub alpha: f64,      // Teleportation constant (default: 0.15)
    pub epsilon: f64,    // Residual threshold (default: 1e-5)
}

impl Default for PprEngine {
    fn default() -> Self {
        Self { alpha: 0.15, epsilon: 1e-5 }
    }
}

impl PprEngine {
    pub fn compute<G: crate::csr::CsrGraphAccess>(
        &self,
        graph: &G,
        seed: NodeId,
        as_of: TxId,
    ) -> Result<Vec<(NodeId, f64)>> {
        let mut p = HashMap::new();
        let mut r = HashMap::new();
        let mut queue = std::collections::VecDeque::new();

        r.insert(seed, 1.0);
        queue.push_back(seed);

        while let Some(u) = queue.pop_front() {
            let res_u = *r.get(&u).unwrap_or(&0.0);
            let edges = graph.get_edges_at(u, as_of)?;
            let deg = edges.len() as f64;

            if deg == 0.0 || res_u < self.epsilon {
                continue;
            }

            // Push operation (Andersen-Chung-Lang)
            *p.entry(u).or_insert(0.0) += self.alpha * res_u;
            r.insert(u, 0.0);

            let push_val = ((1.0 - self.alpha) * res_u) / deg;
            for edge in edges {
                let v = edge.target;
                let entry = r.entry(v).or_insert(0.0);
                let old_val = *entry;
                *entry += push_val;
                if old_val < self.epsilon && *entry >= self.epsilon {
                    queue.push_back(v);
                }
            }
        }

        let mut results: Vec<(NodeId, f64)> = p.into_iter().collect();
        // Deterministische Sortierung via f64::total_cmp
        results.sort_by(|a, b| b.1.total_cmp(&a.1));
        Ok(results)
    }
}
```

### 3.4 Conformal Fusion Calibration (`crates/contextra-rank/src/calibration/conformal.rs`)
```rust
// In crates/contextra-rank/src/calibration/conformal.rs
use contextra_types::{ContextraError, Result};

pub struct ConformalCalibrator {
    calibration_scores: Vec<f64>,
}

impl ConformalCalibrator {
    pub fn new(mut scores: Vec<f64>) -> Self {
        // Strikte Sortierung via total_cmp
        scores.sort_by(|a, b| a.total_cmp(b));
        Self { calibration_scores: scores }
    }

    /// Berechnet den kritischen Konformitäts-Schwellenwert q_hat für das Konfidenzniveau (1 - alpha).
    pub fn compute_quantile(&self, alpha: f64) -> Result<f64> {
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(ContextraError::InvalidData("Alpha must be in (0, 1)".into()));
        }

        let n = self.calibration_scores.len();
        if n == 0 {
            return Err(ContextraError::InvalidData("Calibration set is empty".into()));
        }

        // Finite-sample Korrektur: ceil((n + 1) * (1 - alpha)) / n
        let rank = ((n + 1) as f64 * (1.0 - alpha)).ceil() as usize;
        let idx = (rank.min(n) - 1).max(0);
        Ok(self.calibration_scores[idx])
    }

    /// Kalibriert einen Raw-Score unter Einhaltung der mathematischen Coverage-Garantie.
    pub fn calibrate(&self, raw_score: f64, alpha: f64) -> Result<bool> {
        let q_hat = self.compute_quantile(alpha)?;
        Ok(raw_score >= q_hat)
    }
}
```

---

## 4. Normative Testfälle & Akzeptanzkriterien

### 4.1 Snapshot-Isolation & k-Path Diffusion Test
```rust
#[test]
fn test_graph_snapshot_isolation_and_k_path() {
    let mut graph = crate::csr::CsrGraph::new();
    let n1 = NodeId::new(1);
    let n2 = NodeId::new(2);
    let n3 = NodeId::new(3);

    // Kante n1 -> n2 existiert ab TxId 10
    graph.insert_edge(n1, n2, 0.9, TxId::new(10)).unwrap();
    // Kante n2 -> n3 existiert ab TxId 20
    graph.insert_edge(n2, n3, 0.8, TxId::new(20)).unwrap();

    // Pfad n1 -> n3 zum Zeitpunkt TxId 15 darf n2->n3 nicht sehen!
    let path_t15 = shortest_path(&graph, n1, n3, TxId::new(15)).unwrap();
    assert!(path_t15.is_none());

    // Pfad n1 -> n3 zum Zeitpunkt TxId 25 muss existieren
    let path_t25 = shortest_path(&graph, n1, n3, TxId::new(25)).unwrap();
    assert!(path_t25.is_some());
    let (nodes, cost) = path_t25.unwrap();
    assert_eq!(nodes, vec![n1, n2, n3]);
}
```

### 4.2 Akzeptanz-Gates
```bash
cargo xtask check-compile
cargo xtask check-nan-hot-loop
cargo xtask check-ring-layering
cargo test -p contextra-graph -p contextra-rank
```
