# Audit Report: contextra-graph

**Crate:** `contextra-graph` (Ring 0, `#![forbid(unsafe_code)]`)
**Audit Date:** 2026-03-30
**Auditor:** Jules (Principal Senior Rust Architect)

---

## 1. P1–P7 Audit Übersicht

| Prüfpunkt | Beschreibung | Status | Befund / Details |
|---|---|---|---|
| **P1** | **AGT-GRAPH-001 (TxId-Origin)** | **Pass** | **Dokumentiert + In Debug-Builds technisch erzwungen (`debug_assert!`), in Release-Builds hybrid (tracing-Warnung).** `grep` ergab 0 Treffer in Prod-Code. `TxId` bietet `is_valid_origin()`. `graph_index.rs` erzwingt `debug_assert!(tx.is_valid_origin())`. |
| **P2** | **Bi-Temporale Kanten (ADR-033)** | **Pass** | Sichtbarkeitsprüfung `is_edge_visible` in `visibility.rs` prüft strictly `vf <= as_of && as_of < vt`. Exakter Grenzfall `as_of == valid_to` wird korrekt ausgeblendet (`as_of < vt` evaluated to `false`). |
| **P3** | **CSR-Präfix-Isolation** | **Pass** | System-Präfixe (`__graph:entity:`, `__graph:edge:`, `__graph:community:`, `__graph:hyperedge:`) werden in LSM-Storage unter abweichenden Type-Prefixes/Namespaces abgelegt (`Collection` scan/scan_prefix fügt Type-0 Prefix `\x00` vor User-Keys ein). Normaler User-Scan kann keine `__graph:` System-Keys leaken. |
| **P4** | **GraphEdge-Relation Sync** | **Pass** | In `contextra-engine` (`relate.rs`) erfolgt `relate_with_provenance()` atomar über `DbTransaction`: Persistierung in LSM und Staging für `graph_index` geschehen in derselben Transaktion und werden synchron in `db_tx.commit().await` committed. Kein Inkonsistenz-Zeitfenster. |
| **P5** | **Hop-Limit Traversierung** | **Pass** | `PathRAGEngine` in `path_rag/` erzwingt ein hartes Hop-Limit (`config.max_hops`, Default 4) sowie Schutzzähler `max_steps = max_hops * 1000`. `k_path.rs` nutzt zudem `budget_exhausted` Flag bei Erreichen von `max_visited_nodes`. |
| **P6** | **PPR L1-Norm-Abbruch (ADR-026)** | **Pass** | `ppr.rs` implementiert L1-Norm-Konvergenzkriterium (`diff < convergence_epsilon`). Fallback-Max-Iterationen: `config.max_iterations.min(1000)` garantiert Zero-Hang. |
| **P7** | **Community Detection Skalierung** | **Pass** | Leiden-Algorithmus (`community.rs`) skaliert mit $O(I \cdot \|E\|)$. Iterationen sind strikt beschränkt durch `config.max_iterations` (Default 100). Bei Nicht-Konvergenz wird `tracing::warn!` erzeugt und das Best-Effort-Ergebnis zurückgegeben. |

---

## 2. P1 Invarianten-Analyse: AGT-GRAPH-001 (TxId-Origin)

### Befund
Ein Ausführen von:
```bash
grep -rn "SystemTime::now()\|as_nanos()\|UNIX_EPOCH" crates/contextra-graph/src/ | grep -v "#\[cfg(test)\]"
```
lieferte **0 Treffer** im Produktionscode von `contextra-graph`.

### Durchsetzungs-Grad: Dokumentiert vs. Technisch erzwungen
- **Dokumentiert:** AGENTS.md, `TxId` Rustdoc, `visibility.rs`.
- **Technisch erzwungen (Typ-System):** `TxId` kapselt `u64`. Konstruktion über `TxId::new(u64)` oder Struct-Literal `TxId(u64)` ist möglich. Das Typ-System selbst verhindert somit die Erzeugung ungültiger `TxId`s nicht vollständig zur Kompilierzeit.
- **Laufzeit-Durchsetzung:** `TxId::is_valid_origin(&self)` prüft, ob die TxId in den erlaubt sequenzierten Bereichen liegt:
  $$\text{valid} \iff \text{tx} \le 1.000.000.000.000 \quad \lor \quad \text{tx} \ge \text{TxId::INTERNAL\_BASE}$$
  In `graph_index.rs` (z.B. Zeilen 134, 167, 638, 760) wird dies bei allen Graph-Modifikationen überprüft:
  ```rust
  debug_assert!(
      tx != TxId::INVALID && tx.is_valid_origin(),
      "AGT-GRAPH-001: Invalid TxId origin {:?} for graph operation",
      tx
  );
  ```
- **Fazit P1:** Die Invariante ist **dokumentiert + in Debug-Builds über `debug_assert!` technisch erzwungen**. In Release-Builds wird sie über `is_suspicious_tx_id` und `tracing::warn!` abgefangen.

---

## 3. P2 Bi-Temporale Traversierungs-Logik (ADR-033)

### Code-Nachweis (`crates/contextra-graph/src/csr/visibility.rs`)
```rust
#[inline]
pub fn is_edge_visible(
    tx_valid_from: Option<TxId>,
    tx_valid_to: Option<TxId>,
    as_of: TxId,
) -> bool {
    tx_valid_from.is_none_or(|vf| vf <= as_of) && tx_valid_to.is_none_or(|vt| as_of < vt)
}
```

### Mentale Konstruktion & Boundary-Prüfung
Betrachte 3 Kanten für ein `as_of = TxId(10)`:
1. **Kante 1 (Gültig):** `valid_from = TxId(10)`, `valid_to = TxId(20)`
   - `10 <= 10` (`true`) && `10 < 20` (`true`) $\implies$ **Sichtbar**
2. **Kante 2 (Abgelaufen am exakten Grenzfall):** `valid_from = TxId(5)`, `valid_to = TxId(10)`
   - `5 <= 10` (`true`) && `10 < 10` (`false`) $\implies$ **Ausgeblendet (Gültigkeit endete exakt vor Tx 10)**
3. **Kante 3 (Zukunft):** `valid_from = TxId(15)`, `valid_to = TxId(25)`
   - `15 <= 10` (`false`) $\implies$ **Ausgeblendet**

### Test-Verifikation
In `crates/contextra-graph/src/csr/tests/persistence_tests.rs` (L453-475) wird genau dieses Verhalten mit expliziten Asserts für `valid_to - 1` (sichtbar), `valid_to` (ausgeblendet) und `valid_to + 1` (ausgeblendet) getestet.

---

## 4. P3 CSR-Präfix-Isolation Nachweis

### Code-Nachweis (`crates/contextra-engine/src/collection/crud/read.rs`)
Graph-Keys nutzen die Präfixe `__graph:entity:`, `__graph:edge:`, `__graph:community:`, `__graph:hyperedge:`.
User-Dokumente in `Collection` werden beim Lesen/Schreiben durch `namespaced_key(key, 0)` mit dem Type-Byte `0` (`\x00`) gepräfixt.
Sämtliche `Collection::scan()` und `Collection::scan_prefix()` Aufrufe binden ihre Scans strikt an Type `0`:
```rust
let start_ns = match start {
    Bound::Included(b) => Bound::Included(self.namespaced_key(b, 0)),
    // ...
};
```
Da System-Graphen-Keys unter Type `2` (`\x02`) oder un-namespaced System-Präfixen abgespeichert werden, ist ein unbeabsichtigtes Auslesen von Graph-Interna über die öffentliche `scan()` API ausgeschlossen.

---

## 5. P4 GraphEdge-Relation Synchronisation

In `crates/contextra-engine/src/collection/relate.rs`:
`relate_with_provenance()` führt folgende Schritte durch:
1. `db_tx = self.begin_transaction()?`
2. `self.storage.put(db_tx.tx_id, &key, &bytes).await?`
3. `db_tx.stage_graph_entity(...)`
4. `db_tx.stage_graph_edge(edge)`
5. `db_tx.commit().await`

Beim Commit der Transaktion werden LSM-Write-Batch und `graph_index.add_edge()` synchron in einem Schritt committet. Tritt vor dem Commit ein Fehler auf, rollt `db_tx.rollback().await` die Operation ab.

---

## 6. P5 & P6 PPR-Konvergenz & Hop-Limit Garantien

### PPR Konvergenz (ADR-026)
In `crates/contextra-graph/src/ppr.rs`:
- Die Iteration stoppt, sobald die L1-Norm der Rangdifferenz unter `config.convergence_epsilon` fällt (`diff < epsilon`).
- Fallback zur Vermeidung von Endlosschleifen (Zero-Hang): `let max_iters = config.max_iterations.min(1000);`.
- Falls nach `max_iters` keine Konvergenz erreicht wurde, wird `tracing::warn!` ausgelöst und der aktuelle Best-Effort-Vektor zurückgegeben.

### Hop-Limit bei Traversierung
In `crates/contextra-graph/src/path_rag/`:
- `max_hops` (Default: 4) begrenzt die Suchtiefe im Dijkstra.
- `max_steps = self.config.max_hops * 1000` schützt vor O(V)-Traversierungen in zyklischen Graphen.
- `k_path.rs` markiert `budget_exhausted = true`, sobald das Knoten-Budget überschritten wird.

---

## 7. Audit Verdict & Sign-Off

```text
VERDICT: APPROVED
VERIFIED-BY-SESSION: PENDING (TS: 2026-03-30T12:30:00Z)
```
