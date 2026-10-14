# Contextra — Mikrofeingranulare Implementierungsspezifikation EP3: Contextra Retrieval+

**Dokumenten-ID:** `DOC-SPEC-EP3-RETRIEVAL-2026-10-14`  
**Version:** 1.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Wissensgraphen & PathRAG  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (CSR Graph):** [`contextra-graph`](file:///home/freddy/Projekte/contextra/crates/contextra-graph)  
- **Ring 3 (Graph-Orchestrierung):** [`contextra-engine`](file:///home/freddy/Projekte/contextra/crates/contextra-engine) (Modul `src/collection/relate.rs`, `src/extraction/`)  
- **Ring 4 (MCP Graph Tools):** [`contextra-mcp`](file:///home/freddy/Projekte/contextra/crates/contextra-mcp) (Tools `contextra_relate`, `contextra_explain`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Retrieval+ (EP3) erweitert die Hybridsuche aus EP1 um relationale Wissensgraphen, Hyperkanten, Personalisierte PageRank-Diffusion (PPR) und deterministische Pfadsuche (PathRAG).

**Disjunktionsregeln für EP3:**
1. **Keine Kern-Kopplung (Feature-Gating):** Alle Graph-Funktionen stehen hinter dem Cargo-Feature `graph`. Ohne dieses Feature kompiliert EP1 vollständig ohne Graph-Abhängigkeiten.
2. **Keine ML-Inferenz:** Triple-Extraktion in EP3 erfolgt regelbasiert (OpenIE-Syntaxmuster) oder über externe Port-Adapter; EP3 bindet keine lokalen Torch/Candle-Gewichte ein.
3. **Snapshot-Isolation:** Alle Graph-Traversierungen müssen die Kanten-Sichtbarkeit strikt gegen das übergebene `ReadSnapshot`-Token (TxId) auswerten.

---

## 2. Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Verdrahtung von `find_path` & Kapselung von `find_k_paths`

#### Problem
In `crates/contextra-graph/src/path_rag/` existiert `find_k_paths` (321 LOC), besaß jedoch keinen Produktionsaufrufer (`orphan-symbols`). Umgekehrt fehlte der Durchstich für `find_path` (Dijkstra) in die Collection-Fassade.

#### Datei: `crates/contextra-graph/src/path_rag/mod.rs` & `k_path.rs`
```rust
// In crates/contextra-graph/src/path_rag/k_path.rs:
// KPathDiffusion bleibt bis zum vollständigen Multi-Path-Benchmark auf pub(crate) gekapselt
pub(crate) struct KPathDiffusion {
    // ...
}

// In crates/contextra-graph/src/path_rag/engine.rs:
impl PathRAGEngine {
    /// Deterministischer Shortest-Path zwischen zwei Entitäten unter Berücksichtigung von TxId
    pub fn find_path(
        &self,
        from: EntityId,
        to: EntityId,
        max_hops: usize,
        snapshot_seq: TxId,
    ) -> Result<Option<GraphPath>, ContextraError> {
        let graph = self.graph.read();
        graph.shortest_path_dijkstra(from, to, max_hops, snapshot_seq)
    }
}
```

#### Datei: `crates/contextra-engine/src/collection/relate.rs`
```rust
#[cfg(feature = "graph")]
impl<S: contextra_ports::StorageEngine> Collection<S> {
    /// Exponiert Pfadsuche in der öffentlichen Collection-API
    pub async fn find_path(
        &self,
        from: EntityId,
        to: EntityId,
        max_hops: usize,
    ) -> Result<Option<GraphPath>, ContextraError> {
        let snapshot = self.snapshot_seq().await?;
        self.graph_engine.find_path(from, to, max_hops, snapshot)
    }
}
```

---

### 2.2 Komponente 2: Regelbasierte OpenIE Triple-Extraktion (`contextra-engine`)

#### Architektur
Beim Speichern eines Dokuments (`remember`) werden Entitäten und Relationen deterministisch extrahiert und als Knoten und Kanten im CSR-Graph synchronisiert.

#### Blueprint: `crates/contextra-engine/src/extraction/open_ie.rs`
```rust
use contextra_types::{EntityId, Result};

pub struct ExtractedTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
}

pub struct RuleBasedTripleExtractor;

impl RuleBasedTripleExtractor {
    pub fn extract_triples(text: &str) -> Vec<ExtractedTriple> {
        let mut triples = Vec::new();
        // Deterministische Parsing-Regeln: "X ist ein Y", "X gehört zu Y", etc.
        for line in text.lines() {
            if let Some(triple) = Self::parse_copula(line) {
                triples.push(triple);
            }
        }
        triples
    }

    fn parse_copula(line: &str) -> Option<ExtractedTriple> {
        // Syntaktischer Pattern Matcher ohne Allokationen
        // ...
        None
    }
}
```

---

### 2.3 Komponente 3: Kaskadierende Kanten-Invalidierung (`contextra-graph`)

#### Problem
In `no_graph_stubs.rs` lieferte `cascade_invalidate_edges_for_superseded_doc` immer `Ok(())`. Bei aktiviertem Graph-Feature müssen veraltete Kanten topologisch unterdrückt werden.

#### Datei: `crates/contextra-graph/src/cascade.rs`
```rust
impl CsrGraph {
    /// Markiert alle Kanten, die ausschließlich durch ein überschriebenes Dokument gestützt wurden,
    /// als ungültig ab der aktuellen Sequenznummer.
    pub fn cascade_invalidate_edges_for_superseded_doc(
        &self,
        doc_id: DocId,
        superseded_at: TxId,
    ) -> Result<usize, ContextraError> {
        let mut inner = self.inner.write();
        let affected_edges = inner.provenance.get_edges_for_doc(doc_id);
        let mut invalidated_count = 0;

        for edge_id in affected_edges {
            if inner.provenance.is_solely_supported_by(edge_id, doc_id) {
                inner.visibility.set_invalid_from(edge_id, superseded_at);
                invalidated_count += 1;
            }
        }

        Ok(invalidated_count)
    }
}
```

---

### 2.4 Komponente 4: MCP Graph-Tools (`contextra_relate`, `contextra_explain`)

#### Datei: `crates/contextra-mcp/src/explain.rs`
```rust
#[cfg(feature = "graph")]
pub async fn handle_explain(
    db: &ContextraDb,
    tenant_id: TenantId,
    req: ExplainRequest,
) -> Result<serde_json::Value, McpError> {
    let path = db.find_path(tenant_id, req.from_entity, req.to_entity, req.max_depth)
        .await
        .map_err(|e| McpError::internal(e.to_string()))?;

    match path {
        Some(p) => Ok(serde_json::json!({
            "found": true,
            "hops": p.edges.len(),
            "nodes": p.nodes,
            "relations": p.predicates,
        })),
        None => Ok(serde_json::json!({ "found": false })),
    }
}
```

---

## 3. Akzeptanzkriterien & Qualitätsgates (EP3)

1. **CSR Graph Determinismus-Gate:**
   ```bash
   cargo xtask determinism-check
   ```
   Muss mit Exit-Code 0 bestätigen, dass CSR-Traversierung und PPR-Scores exakt deterministisch reproduzierbar sind.
2. **Kaskadierungs-Test:**
   Ein überschriebenes Dokument muss seine Kanten in Abfragen mit $\text{as\_of} > \text{superseded\_at}$ unsichtbar machen, während historische Abfragen mit $\text{as\_of} \le \text{superseded\_at}$ die Kanten weiterhin sehen.
