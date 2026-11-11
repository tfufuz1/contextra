# contextra-graph — Spezifikation (Klasse A Baustein)

> **Geltung:** Crate-lokale Spezifikation für `contextra-graph`. Normativ ist `docs/SPEC.md` (v2, 10.11.2026).
> Dieses Dokument definiert die Architekturprinzipien, Systemdesignentscheidungen, Modulgrenzen und Schnittstellen zur vollständigen Entkopplung in ein wiederverwendbares, autarkes Bausteinsystem.
> **Klassifikation:** Ring 1 · Klasse `[K]` (Klasse A autonomer Baustein) · Reifegrad `stable` · Unsafe: `#![forbid(unsafe_code)]`

**Pfad:** `crates/contextra-graph/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

---

## 1. Zweck und Architektur-Rolle

`contextra-graph` ist der autarke Graph-Engine-Baustein für:
1. **CSR Graph (Compressed Sparse Row):** Speichereffiziente, performante Repräsentation statischer und dynamischer Graph-Topologien mit bitemporaler Sichtbarkeit (`born_seq` / `died_seq`) und lock-freien Lesepfaden.
2. **Personalized PageRank (PPR):** Lokale und globale Diffusionsverfahren via Andersen-Chung-Lang Forward-Push (`ppr/forward_push.rs`) und dichter Matrix-PPR (`ppr/dense.rs`) mit exaktem Kostenmodell (`ppr/cost.rs`).
3. **GraphRAG Community Detection:** Deterministische Erkennung semantischer Cluster via Leiden/Louvain (`community.rs`) mit modularer Gewichtung.
4. **Pfad-RAG & K-Path Traversal:** Budgetierte K-Path-Traversierung (`path_rag/k_path.rs`) mit Snapshot-Isolation zur kontextuellen Pfadextraktion.
5. **Session DAG:** Verzweigung persistenter Agenten-Zustände (Contextra Session-DAG Pattern in `session_dag.rs`) mit Typsicherheit bei Lock-Akquise (`NodesGuard`).
6. **Autonomes Bausteindesign:** Vollständige Entkopplung externer Abhängigkeiten; Neutraler Vertrag (`Clock`, `Rng`) in `contract.rs`; Brücken zu Workspace-Ports strikt isoliert in `src/compat/`.

---

## 2. Zielzustand & Entkopplungs-Architektur (docs/SPEC.md §3.1)

| Merkmal | Zielzustand |
|---|---|
| **Ring** | Ring 1 (Kern-Closure autonom) |
| **Klasse** | Klasse A (Autonomer Baustein, 0 Pflicht-Abhängigkeiten auf andere Triebwerks-Crates) |
| **Default-Features** | `default = ["workspace", "experimental-graph"]` (Transitional) -> Ziel: `default = []` |
| **Abhängigkeiten (Kern)** | Standard-Bibliothek, `bytes`, `parking_lot`, `tracing`, `ahash`, `serde`, `bincode`, `blake3`, `arc-swap`, `scc`, `thiserror` |
| **Transitional Bridge** | `src/compat/` hinter Features `ports-compat`, `types-compat`, `workspace` |
| **Fehler-Taxonomie** | Kanonisches `GraphError` (`thiserror`, `#[non_exhaustive]`) & `GraphMutationError` |
| **Sicherheit** | `#![forbid(unsafe_code)]` in allen Produktionsmodulen, Zero-Panic-Disziplin |
| **Determinismus** | Deterministische Tie-Breaks, keine ungeordneten Hashes bei API-Ausgaben |

---

## 3. Modul-Architektur & Systemdesign

```
crates/contextra-graph/src/
├── lib.rs                       # Crate-Wurzel, Re-Exports, Lints (#![forbid(unsafe_code)])
├── contract.rs                  # Neutraler Vertrag (Clock, Rng, NodeId, Weight)
├── types.rs                     # Autonome Domänentypen (EntityId, TxId, DocId, Edge, Entity)
├── error.rs                     # GraphError & GraphMutationError Taxonomie
├── csr.rs                       # CSR-Wurzelmodul & Re-Exports
├── csr/                         # Compressed Sparse Row Kern-Implementierung
│   ├── graph_index.rs           # Zentraler GraphIndex-Typ
│   ├── graph_read/              # Lock-freie Lesepfade (BFS, Traversierung, PageRank, Path-RAG)
│   ├── graph_write/             # Transaktionale Schreibpfade (Entity, Edge, Hyperedge, Tombstone)
│   ├── inner/                   # Interne Graph-Topologie & Speicher-Layout
│   ├── graph_persist.rs         # Robuste bincode-Persistenz mit Crash-Sicherheit
│   ├── visibility.rs            # Bitemporale Sichtbarkeit nach Snapshot-Sequenz
│   └── path_graph.rs            # Pfad-Graph-Strukturen
├── ppr/                         # Personalized PageRank (Forward-Push, Dense, Cost-Model)
├── ppr_stream.rs                # Streaming PPR-Kandidaten-Generator
├── community.rs & community/    # Leiden/Louvain Community Detection für GraphRAG
├── path_rag/                    # K-Path Pfad-Diffusionssuche & Snapshot-Traversierung
├── session_dag.rs               # Agent State Branching Tree mit Lock-Hierarchie-Typen
├── hyperedge.rs                 # Hypergraphen & n-äre Rollenbindungen
├── hyperedge_suggest.rs         # Hyperkanten-Vorschlagssystem
├── cascade.rs                   # Topologische Kaskaden-Propagierung
├── edge_reinforcement.rs        # Dynamisches Kanten-Reinforcement-Lernen
├── edge_reinforcement_buffer.rs # Pufferung für Kanten-Verstärkung
├── percolation.rs               # Perkolation & Resilienz-Metriken
├── entity_extraction.rs         # Regelbasierte Entitätsextraktion
├── provenance.rs                # Herkunftsnachweise & Audit-Trails
├── apprh/                       # Adaptiver PageRank mit Hyperkanten (Feature `apprh-diffusion`)
├── tl_hfd/                      # Topologisches Lovász-High-Order-Diffusion
├── storage.rs                   # Neutrale Speicher-Abstraktionen
├── arc_slice.rs                 # Null-Kopie ArcSlice Hilfstypen
├── purger.rs                    # Neutrale Wartungs- & Bereinigungs-Methoden
└── compat/                      # Kompatibilitätsbrücke (ports-compat / types-compat)
    ├── error.rs                 # From<GraphError> for ContextraError Konvertierung
    ├── provider.rs              # Brücke zu contextra_ports::EmbeddingProvider / Clock
    ├── purger.rs                # Brücke zu contextra_ports::Purger
    ├── graph_index/             # contextra_ports::GraphIndex Implementierung
    └── collection_mutation.rs   # CollectionMutationPort Implementierung
```

---

## 4. Invarianten und Verträge

1. **INV-GRAPH-LOCK-FREE-READ (Lock-freier Lesepfad):**
   - CSR-Topologiesegmente sind unveränderlich oder arbeiten mit RCU/Arc-Swap.
   - Lesepfade akquirieren niemals exklusive Locks über längere Traversierungszeiten.
   - Keine Locks dürfen über `.await`-Aufrufe gehalten werden (`#![deny(clippy::await_holding_lock)]`).

2. **INV-GRAPH-TENANT-ISOLATION (VETO-F10):**
   - Keine Kante oder Hyperkante darf Mandantengrenzen (Tenants) überschreiten.

3. **INV-GRAPH-BITEMPORAL-VISIBILITY:**
   - Kanten und Entitäten besitzen `born_seq` und `died_seq`.
   - Sichtbarkeitsprüfungen an Snapshots filtern gelöschte oder zukünftige Mutationen strikt aus.

4. **INV-GRAPH-DETERMINISM:**
   - Bei Algorithmen mit gleichen Scores (Tie-Breaks) entscheidet deterministisch die aufsteigende ID (`EntityId`, gefolgt von Ziel-ID).
   - Zufall und Zeit werden ausschließlich über `contract::Rng` und `contract::Clock` injiziert.

5. **INV-GRAPH-ZERO-PANIC:**
   - Keine unkontrollierten Panics in Produktionsmodulen.
   - Alle Fehler propagieren typisiert über `GraphError` / `Result<T>`.

---

## 5. Comment-Anchors & Dokumentations-System

Das gesamte Crate folgt dem strikten Comment-Anchors Schema:
- `// FILE-CONTEXT` am Dateianfang mit `STAND:`, `ZWECK:`, `INVARIANTEN:`, `HOTSPOTS:`, `SIEHE AUCH:`.
- `// SECTION <Name>` und `// !SECTION <Name>` für logische Code-Blöcke.
- `// ANCHOR[contextra-graph:<BEREICH>-<NNN>]` vor allen öffentlichen Funktionen, Datenstrukturen und Kernalgorithmen.
- `// NOTE:`, `// TODO:`, `// FIXME:`, `// STUB:` für Inline-Erklärungen und Nachvollziehbarkeit.
