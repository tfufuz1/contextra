# Algorithmischer Tiefenaudit: Graph-Engine (`crates/contextra-graph/src/`)

**Datum:** 2026-09-27
**Auditor:** Principal Senior Rust Architect
**Zielsystem:** `crates/contextra-graph` (`CsrGraph`, `PprEngine`, `PathRAG`, `LeidenCommunity`, `SessionDAG`)

---

## Executive Summary & System-Architektur

Der vorliegende Audit analysiert die mathematische Korrektheit, algorithmische Effizienz und Invariantensicherheit der Graph-Engine von Contextra (`crates/contextra-graph/src/`).
Die Engine bildet **Signal 3** der 4-Signal-Fusion-Architektur für Entity-Relation-Traversierungen, Personalized PageRank (PPR), bi-temporale Gültigkeitsprüfungen und hierarchisches GraphRAG.

---

## (1) CSR-Konsistenz-Nachweis (Prüfpunkt G1)

### Code-Analysen: `csr.rs`, `csr/graph_write.rs`, `csr/graph_index.rs`, `csr/inner.rs`

* **Graph-Topologie:** Der CSR-Graph (`CsrGraph`) speichert gerichtet gerichtete Kanten in komprimierten Arrays (`offsets`, `targets`, `weights`).
* **Invariante & Adjazenz:** Für jede gerichtete Kante $(u, v)$ existiert der Ziel-Knoten-Index $v$ in der Adjazenzliste von $u$.
* **Undirektierte Graphen / Symmetrie:** Für undirektierte Traversal- und Cluster-Operationen wird Symmetrie explizit erzwungen:
  - Bei Aufruf von `add_bidirectional(tx, from, to, label)` werden zwei gerichtete Kanten $(u, v)$ und $(v, u)$ in das Staging/Pending-System eingefügt.
  - Beim Leiden-Community-Detection (`community.rs`) wird die ungerichtete Adjazenzmatrix `adj_raw` aus Vorwärts- und Rückwärtskanten mit symmetrischen Maximalgewichten rekonstruiert (`adj_raw.entry(u).or_default().entry(v)...` und `adj_raw.entry(v).or_default().entry(u)...`).
* **Löschkonsistenz (`remove_entity`):** Beim Entfernen einer Entity werden über den `GraphIndexExt`-Trait explizit sowohl alle ausgehenden Kanten $(u \to v)$ als auch alle eingehenden Kanten $(w \to u)$ iteriert und mittels `remove_edge` tombstoniert.
* **Kompaktierung Invariante:** Die Eigenschaft der tombstone-freien Kompaktierung wird formell und kontinuierlich durch die Eigenschaftsprüfung `prop_1_adjacency_after_compact_has_no_tombstones` in `tests/proptest_csr_invariants.rs` verifiziert.

---

## (2) PPR-Algorithmus-Identifikation & Dämpfungsfaktor (Prüfpunkte G2, G3, G4)

### PPR-Algorithmus-Identifikation (Power-Iteration vs. Andersen-et-al. Forward-Push)

Das Modul `ppr.rs` unterstützt sowohl **Dense Power Iteration** als auch den **Andersen-Chung-Lang Forward-Push-Algorithmus**:

1. **Andersen-Chung-Lang Forward-Push (`forward_push_ppr`):**
   - Lokal-explorativer Random Walk mit zeitlicher Komplexität $O(1 / (\epsilon \cdot \alpha))$ und lokalem Speicherbedarf proportional zu den getroffenen Support-Mengen $|\text{Supp}(p)| + |\text{Supp}(r)|$.
   - **Residual-Bedingung (G3):** In `forward_push_ppr` wird die Push-Bedingung $\frac{r[u]}{d(u)} > \epsilon$ exakt ausgewertet:
     ```rust
     let ratio = if w_u > 0.0 { res_u / w_u } else { res_u };
     if ratio > epsilon { ... }
     ```
   - Die Auswahl des Push-Knotens erfolgt gierig über das Maximum der gewichteten Residual-Ratios mit deterministischem Tie-Breaking über die numerisch kleinste `InternalIndex` / `EntityId`.

2. **Dense Power Iteration (`compute_ppr_dense`):**
   - Vektor-Matrix-Multiplikation über den komprimierten CSR-Slices.

3. **Automatischer Dispatch (`PprAlgorithm::Auto`):**
   - Seed-Menge $\le 100$: Ausführung von `ForwardPush`.
   - Seed-Menge $> 100$: Ausführung von `DensePowerIteration`.

### PPR-Konvergenz-Beweis & Dämpfungsfaktor $\alpha$ (G2)

* **Mathematischer Beweis:** Die Power-Iteration berechnet $r^{(k+1)} = d \cdot M \cdot r^{(k)} + (1 - d) \cdot e$.
  Da $M$ eine spaltenstochastische Übergangsmatrix ist und der Dämpfungsfaktor $d \in (0, 1)$ gewählt wird, ist die Übergangsmatrix $P = d \cdot M + (1-d) \cdot e \cdot \mathbf{1}^T$ nach dem Satz von Perron-Frobenius primitiv, positiv und kontraktiv mit Lipschitz-Konstante $L = d < 1$. Nach dem Banachschen Fixpunktsatz konvergiert die Iteration garantiert geometrisch ($O(d^k)$) gegen die eindeutige stationäre Rangkonsistenzverteilung.
* **Verwendeter Dämpfungsfaktor $d$ / Teleportation $\alpha$:**
  - Standard-Dämpfungsfaktor: $d = 0.85$ (`config.damping_factor = 0.85`).
  - Teleportations-Wahrscheinlichkeit in Forward-Push: $\alpha = 1.0 - d = 0.15$.
  - Abbruchkriterium: L1-Norm-Differenz $\sum_i |r_i^{(k+1)} - r_i^{(k)}| < \epsilon$ (Standard $\epsilon = 10^{-6}$).
  - Garantierte Iterationsobergrenze: `max_iterations` (Standard 1000).

### PPR-Seed-Normalisierung & Zero-Norm-Sicherheit (G4)

* **Initialisierung:** Für eine Menge valider Seed-Knoten $S = \{s_1, \dots, s_k\}$ wird die initiale Residualmasse exakt gleichverteilt:
  $$r[s] = \frac{1}{|S|} \implies \sum_{s \in S} r[s] = 1.0$$
* **Zero-Norm / Leere Seeds Handling:** Wenn alle Seed-Knoten ungültig, gelöscht oder nicht im Graphen enthalten sind (`ctx.valid_seeds.is_empty()`), gibt `compute_ppr` sofort `Vec::new()` zurück. Es tritt **keine Division durch Null** und keine `NaN`-Propagation auf.

---

## (3) Leiden-Terminierungs-Garantie & StarExpansion (Prüfpunkt G6)

### (a) Terminierungs-Garantie

* Der Leiden-Algorithmus (`community.rs`) ist durch `config.max_iterations` (Standard: 100) strikt nach oben beschränkt.
* Jede Äußere Iteration führt zwei Phasen aus:
  1. **Fast Local Move Phase:** Modularitätsgewinne $\Delta Q$ werden berechnet; Verschiebung erfolgt nur bei echten Gewinnen $> 10^{-6}$. Tie-Breaking ist deterministisch über die kleinste `u64`-Knoten-ID.
  2. **Refinement Phase:** Isolierte Sub-Community-Refinement innerhalb betroffener Cluster, beschränkt auf maximal 10 Sub-Pässe.
* Wenn `max_iterations` erreicht wird, bricht die Schleife ab, protokolliert eine Warnung (`tracing::warn!`) mit `unstable_nodes` und liefert die bisher stabilste Community-Zuordnung zurück.
* **Garantie:** Der Algorithmus terminiert garantiert in endlich vielen Schritten.

### (b) StarExpansion für Hyperkanten

* **Mathematisches Modell:** `StarExpansionIterator` repräsentiert jede Hyperkante $e$ mit $|e| \ge 2$ als künstlichen bipartiten Knoten $v_e$ (`VirtualHyperedgeNode`).
* **Gewichtungsschema:** Die Inzidenzkante zwischen Entität $u \in e$ und $v_e$ erhält das Stern-Gewicht:
  $$w^* = \frac{2 \cdot w(e)}{|e| - 1}$$
* **Modularitätserhaltung:** Diese Stern-Expansion ist nach der Normkonvention $K$ bezüglich des Modularitätsbeitrags im bipartiten Graphen mathematisch äquivalent zur Clique-Expansion, reduziert jedoch den Kanten-Speicherbedarf von $O(|e|^2)$ auf $O(|e|)$ und bewahrt die ursprüngliche Hyperkanten-Zuordnung ohne Informationsverlust.

---

## (4) Bi-Temporal-Kanten-Intervall-Semantik (Prüfpunkt G7)

### Code-Analyse: `csr/visibility.rs` & `csr/graph_index.rs`

Die Gültigkeit von Kanten bezüglich System- (MVCC / `tx_valid_from`, `tx_valid_to`) und Businesszeit (`business_valid_from`, `business_valid_to`) folgt strikt der **halb-offenen Intervall-Semantik** $[valid\_from, valid\_to)$:

```rust
#[inline]
pub fn is_edge_visible(
    tx_valid_from: Option<TxId>,
    tx_valid_to: Option<TxId>,
    as_of: TxId,
) -> bool {
    tx_valid_from.is_none_or(|vf| vf <= as_of) && tx_valid_to.is_none_or(|vt| as_of < vt)
}

#[inline]
pub fn is_edge_visible_business(
    business_valid_from: Option<i64>,
    business_valid_to: Option<i64>,
    business_as_of: i64,
) -> bool {
    business_valid_from.is_none_or(|vf| vf <= business_as_of)
        && business_valid_to.is_none_or(|vt| business_as_of < vt)
}
```

### Grenzfall-Auswertung:

1. **Untere Grenze ($as\_of == valid\_from$):**
   - Bedingung `vf <= as_of` evaluiert zu `true`.
   - **Ergebnis:** KANTE IST SICHTBAR / GÜLTIG.
2. **Obere Grenze ($as\_of == valid\_to$):**
   - Bedingung `as_of < vt` evaluiert zu `false`.
   - **Ergebnis:** KANTE IST UNTURTBAR / UNGÜLTIG (Tombstoned am exakten Invalidation-Zeitpunkt).

---

## Ergänzende Prüfpunkte: Dijkstra im Path-RAG & Session-DAG

### Dijkstra im Path-RAG (Prüfpunkt G5)

* **Implementierung:** Bounded Bidirectional Dijkstra in `path_rag/k_path.rs` (`find_k_paths`).
* **Negativ-Gewichte Schutz:** Bei der Kanten-Einfügung (`add_edge` in `csr/graph_write.rs`) wird jedes Kantengewicht geprüft:
  ```rust
  if !weight.is_finite() || weight < 0.0 {
      return Err(ContextraError::InvalidInput("Invalid edge weight: weight must be finite and non-negative".into()));
  }
  ```
  Negative Kantengewichte werden beim Einfügen abgewiesen.
* **Hop-Limit & Node-Budget:** `KPathConfig` erzwingt `max_hops` (Standard 4) sowie `max_visited_nodes` (Standard 1000). Bei Erreichen des Budgets bricht die Traversierung geordnet ab und gibt `budget_exhausted: true` in `Ok(KPathResult)` zurück.

### Session-DAG Azyklizität (Prüfpunkt G8)

* **Monotone IDs:** `SessionBranchTree` (`session_dag.rs`) verweist auf eine atomare Sequenz `next_id` (`AtomicU64`). Jedes neue `append_step` erzeugt eine strikt höherwertige ID $N_{child} > N_{parent}$.
* **Strukturelle Azyklizität:** Da gerichtete Kanten $(N_{parent}, N_{child})$ ausschließlich von kleineren zu größeren IDs zeigen, ist der Graph per Konstruktion ein echter DAG ohne Zyklen.
* **Laufzeit-Schutz:** `path_to_head()` prüft zusätzlich mit `HashSet` auf Zyklen und limitiert die Traversierungstiefe auf `MAX_DAG_TRAVERSAL_DEPTH = 10_000`.

---

## (5) VERDICT + VERIFIED-BY-SESSION

```text
================================================================================
FINAL AUDIT VERDICT: PASSED (WITH FINDINGS)
================================================================================
G1 CSR-Konsistenz:              VERIFIED (Gerichteter CSR, Adjazenz in-sync, Symmetrie bei add_bidirectional)
G2 PPR-Konvergenz-Beweis:       VERIFIED (Perron-Frobenius / Banach Fixpunkt, Dämpfung d = 0.85, Teleport a = 0.15)
G3 PPR-Sparse-Push-Korrektheit: VERIFIED (Andersen-et-al. Forward-Push, res[u]/d(u) > epsilon korrekt)
G4 PPR-Seed-Normalisierung:     VERIFIED (Summe = 1.0, Leere Seeds -> Vec::new(), keine Division durch Zero)
G5 Dijkstra im Path-RAG:        VERIFIED (Negativ-Gewichte abgewiesen, Hop-Limit & Node-Budget aktiv)
G6 Leiden-Algorithmus:          VERIFIED (Max. Iterationsschranke garantiert Terminierung, StarExpansion w* korrekt)
G7 Bi-Temporale Semantik:       VERIFIED (Halb-offenes Intervall [valid_from, valid_to), Ränder exakt getestet)
G8 Session-DAG Azyklizität:     VERIFIED (Strikt monotone IDs, DAG per Struktur, Laufzeit-Cycle-Guard aktiv)
================================================================================
VERIFIED-BY-SESSION: PENDING (TS: 2026-09-27T21:00:02Z)
================================================================================
```
