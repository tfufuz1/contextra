# AUDIT REPORT: H1 CsrGraph RCU-Konsistenz vs. geplanter Hyperkanten-Sekundärindex

**Stand:** 2026-09-16
**Crate:** `crates/contextra-graph` (Layer 2)
**Datei:** `crates/contextra-graph/src/csr.rs`
**Modus:** AUDIT (Read-Only Analysis)
**Claim-ID:** `contextra-graph` audit-readonly

---

## Executive Summary

Dieser Audit untersucht die RCU-Architektur von `CsrGraph` (`GraphInner` / `ArcSwap<GraphInner>`) in `crates/contextra-graph/src/csr.rs` bezüglich der Vorbereitung und Bereitschaft für einen künftigen Hyperkanten-Sekundärindex (Architektur-Review v11.2 §H1).

**Ergebnis:** Die zentralen Annahmen aus §H1 bezüglich RCU-Snapshot-Konsistenz und der Einbettung sekundärer Indizes direkt in `GraphInner` wurden **BELEGT**. Es gibt exakt ein zentrales `ArcSwap<GraphInner>`-Handle für atomare Publikationen. Der vorgeschlagene Hyperkanten-Sekundärindex *muss* zwingend als Feld innerhalb von `GraphInner` platziert werden, um Snapshot-Inversionen und Split-Brain-Zustände bei Leser-Queries zu verhindern. Allerdings offenbaren die Details bezüglich `#[derive(Clone)]` und `estimate_memory_bytes()` konkrete Skalierungs- und Budgetierungs-Invarianten, die bei der Erweiterung berücksichtigt werden müssen.

---

## Detailed Findings (Fragen 1 bis 6)

### 1. Feldstruktur von `GraphInner` & `Clone`-Semantik bei Hyperkanten-Erweiterung

* **Status:** **BELEGT**
* **Fundstelle:** `crates/contextra-graph/src/csr.rs:223–279`

`GraphInner` leitet in Zeile 223 explizit `#[derive(Clone)]` ab. Die Struktur umfasst im aktuellen Codebase 22 Felder (bzw. 23 bei aktiviertem Feature-Flag `edge-reinforcement-learning`):

```rust
#[derive(Clone)]
pub(crate) struct GraphInner {
    // Entity-Mapping & Status (5 Felder)
    pub(crate) id_map: HashMap<EntityId, InternalIndex>,          // Z. 226
    pub(crate) reverse_map: Vec<EntityId>,                          // Z. 228
    pub(crate) entities: Vec<Option<Entity>>,                       // Z. 230
    pub(crate) communities: HashMap<EntityId, u64>,                 // Z. 232
    pub(crate) communities_loaded: bool,                            // Z. 234

    // CSR-Haupt-Arrays & Gewichte (9 Felder)
    pub(crate) offsets: Vec<usize>,                                 // Z. 238
    pub(crate) targets: Vec<InternalIndex>,                         // Z. 240
    pub(crate) weights: Vec<f32>,                                   // Z. 242
    pub(crate) tx_valid_froms: Vec<Option<TxId>>,                   // Z. 244
    pub(crate) tx_valid_tos: Vec<Option<TxId>>,                     // Z. 246
    pub(crate) business_valid_froms: Vec<Option<i64>>,              // Z. 248
    pub(crate) business_valid_tos: Vec<Option<i64>>,                // Z. 250
    pub(crate) source_doc_ids: Vec<Option<DocId>>,                  // Z. 252
    pub(crate) out_weight_sums: Vec<f32>,                          // Z. 254

    // Sekundär-Indizes & Staging-Puffer (8 Felder)
    pub(crate) doc_to_edges: ahash::AHashMap<DocId, HashSet<(EntityId, EntityId)>>, // Z. 257
    staged_entities: ahash::AHashMap<(TxId, EntityId), Entity>,    // Z. 260
    staged_edges: ahash::AHashMap<(TxId, EntityId), Vec<StagedEdgePayload>>, // Z. 262
    staged_removals: ahash::AHashMap<TxId, Vec<(EntityId, EntityId)>>,       // Z. 264
    pub(crate) pending_edges: HashMap<InternalIndex, Vec<EdgePayload>>,      // Z. 266
    pub(crate) tombstoned_edges: HashSet<(InternalIndex, InternalIndex)>,   // Z. 268
    pending_edge_count: usize,                                      // Z. 270
    is_dirty: bool,                                                 // Z. 272

    // Feature-gated RL Adjazenzcache (1 Feld)
    #[cfg(feature = "edge-reinforcement-learning")]
    pub(crate) edge_store: HashMap<EntityId, Vec<Edge>>,           // Z. 278
}
```

**Analyse der `Clone`-Kosten & Zero-Copy-Invarianten:**
Ein zusätzliches Feld `hyperedge_index: AHashMap<EntityId, Vec<HyperEdgeId>>` bricht die `Clone`-Semantik von `GraphInner` **nicht**, da `AHashMap`, `EntityId` und `Vec` alle `Clone` implementieren.
**Aber:** Da `GraphInner` bei jedem RCU-Publication-Schritt geklont wird (sowohl in `InnerWriteGuard::drop` in Z. 688 als auch in `compact_async` in Z. 1461: `Arc::new(inner_writer.clone())`), führt ein zusätzliches `AHashMap`-Feld bei jedem Compaction- und Schreib-Flush-Zyklus zu einer **vollständigen Deep-Copy** der HashMap samt aller gekoppelten Vektoren. Bei großen Hypergraphen vervielfacht dies die Re-Allokations- und Klonkosten bei jedem RCU-Swap.

---

### 2. Atomarität & Publikation über `ArcSwap<GraphInner>`

* **Status:** **BELEGT**
* **Fundstelle:** `crates/contextra-graph/src/csr.rs:688, 701, 789–791, 1461`

Die H1-Grundannahme ist **vollständig korrekt**: Es existiert genau eine einzige `ArcSwap`-Instanz für den gesamten Graphzustand auf der `CsrGraph`-Struktur:

```rust
// Z. 701 in struct CsrGraph:
inner: Arc<ArcSwap<GraphInner>>,
```

Es gibt keine separaten oder vorgezogenen `ArcSwap`-Instanzen für Teilzustände. Alle Leser erwerben Zugriff ausschließlich über `inner_read()`:

```rust
// Z. 789-791 in CsrGraph::inner_read:
pub(crate) fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>> {
    self.inner.load()
}
```

Das Publishing eines neuen `GraphInner`-Zustands an Leser erfolgt an exakt zwei Stellen im Codebase über `arc_swap.store(...)`:
1. **RAII-Drop von Schreibsperren:** In `InnerWriteGuard::drop` (Z. 688): `self.arc_swap.store(Arc::new((*self.guard).clone()));`
2. **Asynchrone Kompaktierung:** In `compact_async()` im `spawn_blocking`-Block (Z. 1461): `arc_swap.store(Arc::new(inner_writer.clone()));`

**Konsequenz für Hyperkanten:** Wäre ein Hyperkanten-Index als externe Struktur außerhalb von `GraphInner` definiert, gäbe es zwei getrennte Zustandsanzeigen (`ArcSwap<GraphInner>` und z.B. `ArcSwap<HyperedgeIndex>`). Ein Leser könnte bei parallelen Swaps einen neuen CSR-Zustand sehen, aber noch den alten Hyperkanten-Index (oder umgekehrt). §H1 ist somit unanfechtbar belegt: Der Hyperkanten-Index **muss** Feld von `GraphInner` sein, um Atomatität über den einzigen `ArcSwap<GraphInner>` zu garantieren.

---

### 3. Analyse des Marker-Kommentars `TODO(IP-08-BUDGET-COUPLING)`

* **Status:** **BELEGT**
* **Fundstelle:** `crates/contextra-graph/src/csr.rs:1445`

Der Marker existiert exakt an Zeile 1445 in `compact_async()`:

```rust
// Z. 1435-1446 in CsrGraph::compact_async:
if let Some(max_mb) = self.config.max_compaction_peak_memory_mb {
    let estimated_bytes = snapshot.estimate_memory_bytes();
    let estimated_peak_bytes = estimated_bytes * 2;
    let max_bytes = max_mb * 1024 * 1024;
    if estimated_peak_bytes > max_bytes {
        tracing::warn!(
            estimated_peak_mb = estimated_peak_bytes / (1024 * 1024),
            max_compaction_peak_memory_mb = max_mb,
            "compact_async deferred due to compaction memory budget constraint"
        );
        // TODO(IP-08-BUDGET-COUPLING): Connect to global ResourceTracker when cross-crate tracker handle is integrated.
        return Ok(());
    }
}
```

**Analyse des TODOs:**
1. Die Kopplung zwischen der lokalen Speicherschätzung `estimate_memory_bytes()` und dem lokalen Schwellenwert `max_compaction_peak_memory_mb` **ist im Code bereits vorhanden und aktiv** (Z. 1436–1443).
2. Was laut TODO fehlt, ist die Anbindung an den **globalen, cross-crate `ResourceTracker`**, damit Compactions auch dann aufgeschoben werden, wenn systemweit (z.B. durch HNSW-Index-Rebuilds in `contextra-index` oder LSM-MemTable Flushes in `contextra-store`) der Speicher knapp wird.
3. **Unterschätzungs-Risiko:** `estimate_memory_bytes()` berücksichtigt aktuell nur die flachen Vektor- und Delta-Puffer-Längen, ignoriert jedoch die Kapazitäten und Hash-Overheads von `id_map`, `communities`, `doc_to_edges`, `staged_entities`, `staged_edges`, `staged_removals` und `tombstoned_edges`. Bei Hinzufügen eines Hyperkanten-Sekundärindex (`AHashMap`) würde diese Unterschätzung noch deutlicher wiegen, falls der Hyperkanten-Index nicht explizit in `estimate_memory_bytes()` eingerechnet wird.

---

### 4. Aufrufpfade von `estimate_memory_bytes()`

* **Status:** **BELEGT**
* **Fundstelle:** `crates/contextra-graph/src/csr.rs:310, 1436`

`estimate_memory_bytes()` ist eine Methode auf `GraphInner` (Z. 310) und wird **ausschließlich an einer einzigen Stelle** im gesamten Crate bzw. Workspace aufgerufen:

* **Aufrufpfad:** `CsrGraph::compact_async()` in Zeile 1436:
  `let estimated_bytes = snapshot.estimate_memory_bytes();`

An keiner anderen Stelle (z.B. Metriken, Monitoring oder `stats()`) wird diese Methode verwendet (`stats()` in Z. 1563 berechnet stattdessen eine vereinfachte eigene Summe über CSR-Kernarrays). `estimate_memory_bytes()` dient exakt und ausschließlich dem Budget-Check vor Beginn der Hintergrund-Kompaktierung.

---

### 5. Granularität & Aufbau von `estimate_memory_bytes()`

* **Status:** **BELEGT**
* **Fundstelle:** `crates/contextra-graph/src/csr.rs:310–323`

Die Methode ist per-Feld additiv aufgebaut, was die isolierte Erweiterung um weitere Felder sehr einfach macht:

```rust
pub(crate) fn estimate_memory_bytes(&self) -> usize {
    (self.reverse_map.len() * std::mem::size_of::<EntityId>())
        + (self.entities.len() * std::mem::size_of::<Option<Entity>>())
        + (self.offsets.len() * std::mem::size_of::<usize>())
        + (self.targets.len() * std::mem::size_of::<usize>())
        + (self.weights.len() * std::mem::size_of::<f32>())
        + (self.tx_valid_froms.len() * std::mem::size_of::<Option<TxId>>())
        + (self.tx_valid_tos.len() * std::mem::size_of::<Option<TxId>>())
        + (self.business_valid_froms.len() * std::mem::size_of::<Option<i64>>())
        + (self.business_valid_tos.len() * std::mem::size_of::<Option<i64>>())
        + (self.source_doc_ids.len() * std::mem::size_of::<Option<DocId>>())
        + (self.out_weight_sums.len() * std::mem::size_of::<f32>())
        + (self.pending_edge_count * std::mem::size_of::<EdgePayload>())
}
```

Eine künftige Erweiterung um einen Hyperkanten-Term kann als isolierter Summand (z.B. `+ (self.hyperedge_count * std::mem::size_of::<HyperEdgePayload>())`) angehängt werden.

---

### 6. Bestehender Sekundärindex in `GraphInner` als Architektur-Vorbild

* **Status:** **BELEGT**
* **Fundstelle:** `crates/contextra-graph/src/csr.rs:257, 601–605, 831–836, 1269–1274`

In `GraphInner` existiert bereits heute ein sekundärer, snapshot-konsistenter Index neben den reinen CSR-Arrays:

* **Feld:** `pub(crate) doc_to_edges: ahash::AHashMap<DocId, HashSet<(EntityId, EntityId)>>` (Z. 257).
* **Funktion:** Reverse-Lookup für Dokument-Provenienz & Cascading Invalidation (INV-GRAPH-PROV-1).
* **Konsistenz-Muster:**
  - Wird bei `add_edge()` / `commit()` / `load_edge_direct()` synchron in `GraphInner` befüllt (z.B. Z. 601–605, Z. 1269–1274).
  - Wandert bei jedem `ArcSwap::store` atomar zusammen mit den CSR-Arrays in den neuen RCU-Snapshot.
  - Abfragen über `CsrGraph::edges_for_doc(doc_id)` lesen lock-frei aus dem via `inner_read()` erworbenen `GraphInner`-Snapshot (Z. 831–836).

Dieses `doc_to_edges`-Feld dient als **exaktes technisches Modell** für die vorgeschlagene Einbettung des Hyperkanten-Sekundärindex in `GraphInner`.

---

## Empfehlungen für künftige Hyperkanten-Implementierung

1. **Snapshot-Isolation sichern:** Den Hyperkanten-Index ausnahmslos als Mitgliedsfeld von `GraphInner` deklarieren. Keine externen `ArcSwap`- oder `RwLock`-Handhabungen außerhalb von `GraphInner`.
2. **`estimate_memory_bytes()` anpassen:** Den Hyperkanten-Speicherbedarf in `estimate_memory_bytes()` einrechnen, um Compaction-Budgetüberschreitungen präzise zu verhindern.
3. **`Clone`-Overhead beachten:** Bei der Wahl der Datenstruktur für Hyperkanten (z.B. flache Vektoren vs. verschachtelte `AHashMap`s) beachten, dass `GraphInner` bei jedem Schreib-Commit geklont wird.

---
*Audit abgeschlossen gemäss Audit-Plan.*
