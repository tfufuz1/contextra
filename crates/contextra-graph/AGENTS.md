# AGENTS.md — contextra-graph
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.11

## 1. Zweck
Wissensgraph-Engine des Contextra-Systems (Signal 3 der 4-Signal-Fusion). Implementiert Compressed Sparse Row (CSR) Graph-Speicherung (`CsrGraph`), bi-temporale Kanten-Gültigkeit, Personalized PageRank (PPR), PathRAG, N-äre Hyperkanten sowie Community Detection und Arbeitsablauf-Session-DAGs. Implementiert `GraphIndex` aus `contextra-ports`.

## 2. Modul-Karte

| Verzeichnis / Datei | Verantwortung |
|---|---|
| `lib.rs` | Modul-Deklaration, `#![forbid(unsafe_code)]` Crate-Boundary |
| `csr/` | CSR-Kernstrukturen (`graph_index.rs`, `graph_read.rs`, `graph_write.rs`, `graph_persist.rs`, `inner.rs`, `path_graph.rs`, `types.rs`, `visibility.rs`) |
| `apprh/` | APPRH Approximate Personalized PageRank & Heuristics (`diffusion.rs`, `gate_monitor.rs`, `params.rs`, `selector.rs`, `shadow.rs`, `error.rs`) |
| `ppr/` | Personalized PageRank Engine (`ppr.rs`, `ppr_stream.rs`, `cost.rs`, `shadow_hook.rs`, `snapshot.rs`) |
| `path_rag/` | PathRAG Engine für k-Path-Retrieval & Snapshot-Isolation (`snapshot.rs`, `k_path.rs`) |
| `community/` | Community Detection Algorithmen & Leiden-Gruppierung (`community.rs`) |
| `tl_hfd/` | TL-HFD Topological Heat Flow Diffusion (`diffusion.rs`, `lovasz.rs`, `params.rs`, `shadow.rs`, `error.rs`) |
| `provenance.rs` | Herkunftsnachweis (`EdgeProvenance`, `DocEdgeIndex`, `INV-GRAPH-PROV-1`) |
| `consistency_enforcement.rs` | L.9 Widerspruchserkennung und Edge-Suppression (`ConsistencyEnforcer`, `ExactPredicateConflictDetector`, `ConflictPattern`) |
| `hyperedge.rs` | N-äre Hyperkanten-Definition (`HyperEdge`, `HyperEdgeId`, `RoleId`, `RoleBinding`) & `hyperedge_suggest.rs` |
| `session_dag.rs` | Agenten-Workflow-DAG (`SessionBranchTree`, `AgentStateNode`, `DagEdge`) |
| `edge_reinforcement.rs` | Kanten-Verstärkungslernen & Puffermanagement (`edge_reinforcement_buffer.rs`) |
| `cascade.rs` | Kaskadierende Kanten-Invalidierung bei Dokument-Superseding |
| `percolation.rs` | Graph-Perkolationsanalyse |
| `entity_extraction.rs` | Extraktions-Schnittstelle für Entitäten |
| `arc_slice.rs` | Zero-Copy Arc-Slice Hilfsstrukturen |
| `error.rs` | Spezifische Fehler-Taxonomie für Graph-Operationen |

## 3. Invarianten

- **AGT-GRAPH-001:** `TxId` muss für alle Graph-Mutationen `TxId::is_valid_origin()` erfüllen (`tx != TxId::INVALID && tx.is_valid_origin()`). Direct Wall-Clock `SystemTime::now()` als TxId ist strikt verboten. (`cargo test -p contextra-graph`)
- **INV-GRAPH-PROV-1:** Jede eingefügte Kante trägt nachvollziehbare Herkunftsmetadaten (`EdgeProvenance`). (`cargo test -p contextra-graph provenance`)
- **Bi-temporale Achsen:** Bi-temporale Kanten filtern Sichtbarkeit strikt über `valid_from` und `valid_to` Kausal-TxIds. (`cargo test -p contextra-graph csr::tests::bitemporal_tests`)
- **INV-GRAPH-SAFE:** 100% Safe Rust mit `#![forbid(unsafe_code)]` im Crate-Root. (`cargo xtask check-agents-integrity`)

## 4. Verboten / Anti-Patterns

```rust
// ❌ FALSCH — SystemTime als TxId für Graph-Operationen verwenden:
let tx = TxId::new(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64);
// ✅ KORREKT — Gültige TxId aus der Collection / Transaction Engine verwenden.

// ❌ FALSCH — tokio::spawn oder async runtime im Graph-Kern aufrufen:
tokio::spawn(async move { ... });
// ✅ KORREKT — Ring 0 ist synchron (P26); Graph-Traversierung läuft rein synchron.

// ❌ FALSCH — Unbegrenzte Graph-Traversierung ohne Hop-Limit oder Cutoff:
// ✅ KORREKT — Immer max_hops, PPR-Minkowski-Norm-Abbruch oder Edge-Budgeting angeben.
```

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- **Ring-0 Sync-Reinheit (P26):** `contextra-graph` ist ein Ring-0-Crate ohne `tokio`-Abhängigkeit. Algorithmen arbeiten synchron über In-Memory-Snapshots.
- In-Memory CSR-Arrays und Indizes werden durch feingranulare `parking_lot::RwLock` geschützt.
- Locks werden nur für minimal erforderliche Abschnitte gehalten.

## 6. Verifikation

```bash
cargo test -p contextra-graph --locked
cargo xtask check-agents-integrity
```

## 7. Bekannte Lücken / SOLL

- Hyperkanten-Vorschläge (`hyperedge_suggest.rs`) und Perkolation (`percolation.rs`) bieten fortgeschrittene Graph-Analyse, werden jedoch selektiv je nach Feature-Konfiguration aktiviert.
