# Algorithmitischer Tiefenaudit: Graph-Engine (`contextra-graph`)

**Datum:** 2026-10-12
**Crate:** `contextra-graph` (`crates/contextra-graph/src/`)
**Auditor:** Principal Senior Rust Architect (Jules)
**Session Hash:** ed552506

---

## Executive Summary

Der mathematische und algorithmische Tiefenaudit der Graph-Engine (`contextra-graph`) bestätigt die Einhaltung aller Kern-Invarianten, In-Memory CSR-Datenstrukturen, Personalized PageRank (PPR) Konvergenzgarantien, Bi-Temporale Sichtbarkeitsfenster sowie des Session-DAGs für Agenten-State-Branching.

Sämtliche 213 Modultests, 5 Proptest-Invarianztests und 3 Regressionstests unter `crates/contextra-graph` wurden erfolgreich ausgeführt und verifiziert.

---

## 1. CSR-Interne Konsistenz (G1)

### Befund & Code-Analyse
Der Compressed Sparse Row (CSR) Graph ist in `crates/contextra-graph/src/csr/` modularisiert (`graph_write.rs`, `graph_read.rs`, `graph_index.rs`, `inner.rs`).

- **Doppelte Adjazenzspeicherung:** Der Graph unterstützt gerichtet gezielte Kanten ($u \to v$) als Primärstruktur.
- **Undirektierte/Bidirektionale Kanten:** Werden über `add_bidirectional` explizit durch Einfügen zweier symmetrischer Kanten ($u \to v$ und $v \to u$) realisiert (`EVIDENCE-G1-1`: `csr/graph_index.rs:809-811`).
- **Lösch-Invariante (`remove_entity`):** Beim Entfernen einer Entity `u` werden sowohl alle ausgehenden ($u \to v$) als auch alle eingehenden ($w \to u$) Kanten in der Adjazenz gesucht und via `remove_edge` tombstoned (`EVIDENCE-G1-2`: `csr/graph_index.rs:25-78`).
- **Kompaktierung:** `compact()` aggregiert gepufferte unkompaktierte Deltakanten (`pending_edges`), filtert tombstoned Kanten heraus, sortiert Adjazenzlisten nach Ziel-Indizes und baut die CSR-Arrays (`offsets`, `targets`, `weights`, `out_weight_sums`) deterministisch und idempotent neu auf.

---

## 2. PPR-Algorithmus-Identifikation & Konvergenz (G2, G3, G4)

### 2.1 Algorithmus-Klassifikation & Routing (G2, G3)
`contextra-graph` unterstützt zwei Haupt-PPR-Algorithmen sowie ein automatisches Kostenmodell (`PprAlgorithm::Auto`):

1. **Push-Based PPR (Andersen-Chung-Lang Forward Push):**
   - Implementiert in `ppr.rs` (`forward_push_ppr`) und `path_rag/mod.rs` (`forward_push_ppr`).
   - Sparse Push mit Residual-Push-Bedingung: $r[u] / \text{degree}(u) > \epsilon$ (`EVIDENCE-G3-1`: `ppr.rs:567-575`).
   - Laufzeit: $O(1 / (\alpha \cdot \epsilon))$, unabhängig von $|V| + |E|$.

2. **Dense Power Iteration:**
   - Implementiert in `ppr.rs` (`compute_ppr_dense`).
   - Matrix-Vektor-Multiplikation direkt über CSR-Arrays (`offsets`, `targets`, `weights`).
   - L1-Norm Abbruchbedingung: $\sum_{i} |r^{(t+1)}_i - r^{(t)}_i| < \epsilon$ (`EVIDENCE-G2-1`: `ppr.rs:881-888`).

3. **Dämpfungsfaktor (Damping Factor $\alpha$):**
   - Standard-Dämpfungsfaktor: $\alpha = 0.85$ (`config.damping_factor = 0.85`).
   - Teleportations-Wahrscheinlichkeit: $1 - \alpha = 0.15$ (`EVIDENCE-G2-2`: `ppr.rs:518-524`, `788-796`).

### 2.2 Seed-Normalisierung & Null-Norm-Verhalten (G4)
- **Seed-Gültigkeit:** In `forward_push_ppr` und `compute_ppr_dense` werden übergebene Seeds gegen `id_map`, `deleted_nodes` und existierende Entities gefiltert.
- **Normalisierung:** Werden $k$ gültige Seed-Knoten gefunden, erhält jeder Seed das initiale Residuum $r[s] = 1/k$, so dass die Gesamtsumme der initialen Masse exakt $1.0$ beträgt (`EVIDENCE-G4-1`: `ppr.rs:528-532`, `719-724`).
- **Null-Norm / Leere Seeds:** Ist die Menge der Seed-Knoten leer oder enthalten alle Seeds gelöschte Knoten, bricht die Funktion sofort ab und gibt eine leere Ergebnisliste `Vec::new()` zurück — **keine Division durch Null, vollkommen definiertes Verhalten**.

---

## 3. Leiden-Algorithmus & Terminierungs-Garantie (G6)

### 3.1 Terminierungs-Garantie (G6.a)
Der Leiden-Community-Detection-Algorithmus in `community.rs` (`detect_communities`) ist streng schleifenbegrenzt:
- Äußere Schleife limitiert auf `config.max_iterations` (Standard: 100) (`EVIDENCE-G6-1`: `community.rs:373`).
- Vorzeitiger Abbruch bei stabiler Konvergenz, sobald in einer Iteration keine Knotenverschiebungen mehr stattfinden (`iter_changed == 0`).
- Bei Erreichen von `max_iterations` ohne vollständige Konvergenz wird kein Fehler geworfen, sondern ein Best-Effort-Ergebnis zurückgegeben und ein `tracing::warn!` emittiert.

### 3.2 Hyperkanten Stern-Expansion (G6.b)
- **Stern-Expansion (Star Expansion):** Über `StarExpansionIterator` wird jede Hyperkante $e$ als künstlicher bipartiter Knoten (`VirtualHyperedgeNode`) repräsentiert (`EVIDENCE-G6-2`: `community.rs:59-114`).
- **Mathematische Korrektheit:** Inzidenzgewichte werden mit der Standard-Kanten-Gewichtung $w^* = 2w / (|e| - 1)$ oder der Kanten-Gewichtung $w$ skaliert.
- **Zero-Copy Invariante:** Der Iterator traversiert referenzbasiert über die Hyperkanten in `GraphInner` ohne Heap-Klonen. Virtuelle Knoten werden bei der finalen Zuordnungs-Generierung gefiltert, sodass nur reale `EntityId`s Community-IDs erhalten.

---

## 4. Bi-Temporale Kanten-Traversierungs-Semantik (G5, G7, G8)

### 4.1 Intervall-Semantik $[valid\_from, valid\_to)$ (G7)
Die Sichtbarkeitsprüfung in `csr/visibility.rs` implementiert ein **halb-offenes Intervall $[valid\_from, valid\_to)$**:

```rust
pub fn is_edge_visible(tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of: TxId) -> bool {
    tx_valid_from.is_none_or(|vf| vf <= as_of) && tx_valid_to.is_none_or(|vt| as_of < vt)
}
```

- **Untere Grenze $as\_of = valid\_from$:** `vf <= as_of` ist `true` $\implies$ **EINGESCHLOSSEN (INCLUDED)** (`EVIDENCE-G7-1`: `csr/visibility.rs:39`).
- **Obere Grenze $as\_of = valid\_to$:** `as_of < vt` ist `false` $\implies$ **AUSGESCHLOSSEN (EXCLUDED)** (`EVIDENCE-G7-1`: `csr/visibility.rs:39`).
- Exakte Grenzfälle werden im Test `test_traverse_at_time_exact_boundary_off_by_one` in `csr/tests/persistence_tests.rs` validiert.

### 4.2 PathRAG Dijkstra (G5)
- In `path_rag/mod.rs` nutzt `PathRAGEngine::find_path` bidirektionalen Dijkstra für gewichtete Graphen.
- **Negative Gewichte:** In `add_edge` werden Gewichte $w < 0.0$ oder $w = \text{NaN/Inf}$ explizit abgelehnt (`ContextraError::InvalidInput`).
- **Hop-Limit:** Die Suche ist durch `max_hops` beschränkt (Standard 4, hartes Limit 100 in Traversal). In `find_path` verhindert `max_steps = max_hops * 1000` Endlosschleifen (`EVIDENCE-G5-1`: `path_rag/mod.rs:260`).

### 4.3 Session-DAG Azyklizität (G8)
- `SessionBranchTree` in `session_dag.rs` verwaltet Verzweigungen der Agenten-Historie.
- **Azyklizität durch Konstruktion:** Knoten-IDs werden als monoton steigende `u64`-Sequenz via `AtomicU64::fetch_add(1)` vergeben. Kanten verlaufen immer von `parent` (existierender kleinerer Index) zu `child` (neu erzeugter größerer Index).
- **Zyklenerkennung:** In `path_to_head` schützt ein `HashSet<NodeIdx>` vor Zyklen, und `MAX_DAG_TRAVERSAL_DEPTH = 10_000` begrenzt die Pfadtiefe (`EVIDENCE-G8-1`: `session_dag.rs:271-289`).

---

## 5. Audit Verdict & Evidence Table

### VERDICT
```text
===============================================================================
CONTEXTRA GRAPH ENGINE ALGORITHMIC AUDIT VERDICT: PASSED (GREEN)
===============================================================================
Crate: contextra-graph (Ring 0)
Scope: csr.rs, ppr.rs, path_rag/mod.rs, community.rs, session_dag.rs, visibility.rs
Tests Status: 213 unit/integration tests PASSED, 5 proptests PASSED
Invariant Enforcement: Zero-Panic, Strict Determinism, Bitemporal [valid_from, valid_to)
===============================================================================
```

### EVIDENCE MARKERS

| Marker | Komponente | Dateipfad & Zeilen | Beschreibung |
|---|---|---|---|
| `EVIDENCE-G1-1` | CSR | `csr/graph_index.rs:809-811` | Bidirektionale Kanten-Konsistenz in `add_bidirectional` |
| `EVIDENCE-G1-2` | CSR | `csr/graph_index.rs:25-78` | Beidseitiges Kanten-Tombstoning in `remove_entity` |
| `EVIDENCE-G2-1` | PPR | `ppr.rs:881-888` | L1-Norm-Konvergenzkriterium in Dense Power Iteration |
| `EVIDENCE-G2-2` | PPR | `ppr.rs:518-524`, `788-796` | Dämpfungsfaktor $\alpha = 0.85$ (Teleport $0.15$) |
| `EVIDENCE-G3-1` | PPR | `ppr.rs:567-575` | Sparse Forward Push Residual-Bedingung $r[u]/\text{degree}(u) > \epsilon$ |
| `EVIDENCE-G4-1` | PPR | `ppr.rs:528-532`, `719-724` | Definiertes Verhalten bei leeren/ungültigen Seed-Mengen |
| `EVIDENCE-G5-1` | PathRAG | `path_rag/mod.rs:260` | Dijkstra Hop-Limit & Schutzzähler gegen Endlosschleifen |
| `EVIDENCE-G6-1` | Leiden | `community.rs:373` | Schleifenbegrenzung `max_iterations` für Terminierung |
| `EVIDENCE-G6-2` | Leiden | `community.rs:59-114` | StarExpansionIterator für Hyperkanten-Expansion |
| `EVIDENCE-G7-1` | Bi-Temporal | `csr/visibility.rs:39` | Halboffenes Intervall $[valid\_from, valid\_to)$ |
| `EVIDENCE-G8-1` | Session-DAG | `session_dag.rs:271-289` | Azyklizität & Zyklenerkennung in `path_to_head` |
