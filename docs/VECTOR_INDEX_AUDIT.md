# Audit: `contextra-vector` – Schwachstellen, Lücken, Optimierungspotenzial

**Datum:** 2026-11-02  
**Prüfer:** ANN-Forscher/-Ingenieur (HNSW/DiskANN/ACORN), autonome Analyse  
**Basis:** Quellcode-Stand `crates/contextra-vector/` (alle `src/`- und `tests/`-Dateien)  
**Normativ:** `docs/SPEC.md` 4.3.0; bei Widerspruch gilt SPEC.

---

## Zusammenfassung (Executive Summary)

Die HNSW-Implementierung ist strukturell solide und erfüllt viele Kernanforderungen: Tombstone-Modell, Rebuilding in zwei Phasen, Snapshot-Isolation, deterministische Seeded-RNG, ACORN-γ-Augmentierung. Jedoch bestehen **sieben kritische Lücken** und **zwölf Optimierungspotenziale**, die Recall, Sicherheit (Zeroizing) und SPEC-Konformität gefährden. Das Gelöschte-Knoten-Modell stoppt derzeit bei Tombstones; weder Surrogat-Injektion noch Nachbarschafts-Reparatur (neighbor repair = Reconnect-of-Orphans) sind in Produktion implementiert. Die Löschaussage-Schicht ist im Sinne des Erasure-Reports als **„nicht erfasst"** einzustufen.

---

## 1. Graphkonnektivität und Recall unter Löschung

### 1.1 Tombstone-Semantik ohne Nachbarschafts-Reparatur (KRITISCH)

**Befund:** `do_delete` in [`core_rebuild.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L394-L474) setzt ausschließlich ein Tombstone-Bit und aktualisiert den Entry-Point. **Keine Kante der verbleibenden Nachbarn des gelöschten Knotens wird repariert.** Die Nachbarn von `v_del` verlieren damit alle Verbindungen zu `v_del` erst beim nächsten Traversal, nicht sofort.

**Konsequenz:** Nach Löschung von `k` gut-vernetzten Knoten entstehen Inseln (disconnected components) in den unteren Layern, besonders bei Layer 0. Jeder traversale Absteig landet mit Wahrscheinlichkeit > 0 an einem Tombstone, das `visited_dead_nodes`-Zähler steigt monoton.

**Aktueller Workaround:** `try_relink_pruned_neighbors` in [`core_search.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_search.rs#L124-L151) führt **lazy pruning** durch: Wenn ein traversierter Knoten tote Nachbarn hat, werden diese live aus der Kantenliste gelöscht. Dies ist **keine Reparatur** (kein Reconnect zu Überlebenden), sondern nur eine Schrumpfung des Kantengraphen.

**Algorithmusvergleich (Malkov/Yashunin, FreshDiskANN §4):**  
Korrekte Nachbarschafts-Reparatur verbindet die Nachbarn des gelöschten Knotens miteinander (oder mit dem besten Überlebenden) via `SelectNeighborsHeuristic`. Nur so bleibt der Small-World-Charakter erhalten.

**Fehlender Test:** Kein Test prüft, ob nach Löschung von 5/20 % der Knoten der Recall@10 über einer Gate-Schwelle bleibt. `partial_rebuild_recall.rs` prüft nur, dass `rebuild_region` den Recall nicht schlechter macht als reines Tombstoning – **kein absoluter Gate-Wert**.

**SPEC-ID:** T-VEC-DEL-1, T-VEC-DEL-2 (beide referenzieren diese Eigenschaft, sind aber nicht im Code implementiert).

**Empfehlung:**
1. `do_delete` → nach Tombstone-Setzen: Nachbarn von `v_del` in allen Layern abrufen, paarweise via `SelectNeighborsHeuristic` verbinden (FreshDiskANN-Reparaturschema).
2. Recall@10-Gate-Tests bei 1/5/20 % gelöschten Knoten mit absolutem Schwellwert ≥ 0.85.

---

### 1.2 Entry-Point-Wahl bei Löschung (MITTEL)

**Befund:** [`do_delete`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L403-L471) wählt als neuen Entry-Point den Knoten mit dem **höchsten `max_layer`**. Der Scan ist `O(N)` und hält `nodes.read()`, `mmap_guard.read()` und `deleted_nodes.read()` gleichzeitig offen.

**Problem 1:** Kein Kriterium prüft die Konnektivität des Ersatz-Entry-Points. Ein Knoten auf Layer 5 mit nur 1 verbleibenden Nachbarn ist ein schlechter Entry-Point.

**Problem 2:** Bei häufigen Löschungen in der höchsten Schicht wird der Scan immer wieder ausgeführt (O(N) amortisiert schlecht). Bei 1 M Knoten verursacht jede EP-Deletion einen Lock auf die gesamte Knotenliste.

**Empfehlung:** Kandidaten für den EP in einer Small Priority Queue nach `max_layer DESC, connectivity_estimate DESC` halten. Für den Lock: Entry-Point-Kandidaten als separate sortierte Struktur verwalten.

---

### 1.3 Surrogat-Injektion: Nicht implementiert (KRITISCH)

**Befund:** Die Spezifikation (AGENTS.md, Rollenbeschreibung) fordert die Möglichkeit, **inhaltslose Platzhalterknoten** zu injizieren, die Navigation erhalten aber keinen Vektorinhalt tragen. Die Implementierung kennt dieses Konzept nicht:

- `VectorData` hat nur `F32` und `U8` – kein `Surrogate`-Variant.
- `zeroize_and_free_node` in [`arena.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/arena.rs#L298-L340) löscht Kantenlisten, gibt den Slot aber zur **Wiederverwendung** frei – es bleibt kein navigierbarer Platzhalter übrig.
- `GhostFreeVectorIndex` und `remove_with_graph_repair` werden in Tests referenziert ([`reconstruction_attack_test.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/tests/reconstruction_attack_test.rs#L3), [`deletion_ghost_pointer_proptest.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/tests/deletion_ghost_pointer_proptest.rs#L3)), existieren aber **nicht im Produktionscode** (`src/`). Der Build würde scheitern.

**Bedeutung für Löschaussage:** Eine Löschaussage „Vektorbytes sind zeroized und nicht rekonstruierbar" kann nicht belegt werden, solange:
- Der Vektorspeicher im `HnswNode.vector` nach `do_delete` **nicht zeroized** wird.
- Die Kantenlisten im Arena nach Soft-Delete **weiterhin die alten Nachbar-Indizes** enthalten.
- Kein Surrogat-Knoten die Navigation übernimmt, bevor die Bytes gelöscht werden.

**Schicht im Erasure-Report:** **„nicht erfasst"** – Zeroizing und Reparatur sind weder implementiert noch gemessen.

**Empfehlung (Surrogat-Injektion nach Malkov/DiskANN-Muster):**
```
enum VectorData {
    F32(Vec<f32>),
    U8(Vec<u8>),
    Surrogate,           // Neu: navigierbar, kein Inhalt
}
```
Beim Löschen:
1. `HnswNode.vector` → `VectorData::Surrogate` + Zeroize der alten Bytes.
2. Kantenlisten des Surrogat-Knotens bleiben erhalten (Navigation funktioniert).
3. Surrogat-Knoten wird in `deleted_nodes` eingetragen → nicht in Suchergebnissen sichtbar.
4. Nach Nachbarschafts-Reparatur (Schritt 1.1): Surrogat-Knoten kann aus dem Graphen entfernt werden.

---

### 1.4 `rebuild_region` repariert nicht, sondern nur pruned (HOCH)

**Befund:** [`rebuild_region_sync`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L628-L723) iteriert alle RAM-Knoten und **entfernt Tombstone-Kanten** aus den Kantenlisten. Es werden jedoch **keine neuen Kanten** zwischen den Überlebenden aufgebaut.

```rust
// Nur Pruning – kein Reconnect:
for &neighbor_u32 in layer_slice {
    if !tombstoned_set.contains(&neighbor_u32) {
        kept.push(neighbor_u32);   // ← kein SelectNeighbors, kein Reconnect
    }
}
```

**Recall-Folge:** Der Recall-Test in [`partial_rebuild_recall.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/tests/partial_rebuild_recall.rs#L150-L163) lässt einen Recall-Verlust von 15 Prozentpunkten zu (`recall_after_rebuild >= recall_before - 0.15`). Das ist für einen Produktions-Vektorindex mit Löschanforderungen kein hinreichender Gate-Wert.

**Empfehlung:** Nach dem Pruning-Schritt: Für jeden überlebenden Knoten mit unter `M/2` verbleibenden Kanten → `SelectNeighborsHeuristic` auf dem Subgraph der Region ausführen und Kanten auffüllen.

---

### 1.5 Stapel-Reparatur im Wartungsfenster: Nicht geplant (MITTEL)

**Befund:** Es gibt keinen Mechanismus für geplante Batch-Repair-Läufe (Maintenance-Window-Repair). Der Rebuild erfolgt nur on-demand (`trigger_rebuild_async`) oder via `partial_rebuild` (Feature-Flag). Ein regelmäßiger Wartungs-Scan auf Knoten mit Konnektivitäts-Score < Schwellwert fehlt.

---

## 2. Sicherheit: Zeroizing und Rekonstruierbarkeit

### 2.1 Vektorbytes werden nach Soft-Delete nicht zeroized (KRITISCH)

**Befund:** `do_delete` in [`core_rebuild.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L394) setzt nur:
```rust
self.cold.deleted_nodes.write().insert(idx as u64);
self.hot.deleted_count.fetch_add(1, Ordering::SeqCst);
```
`HnswNode.vector` (F32 oder U8) bleibt vollständig im RAM. Die Kantenliste in der Arena bleibt ebenfalls vollständig erhalten.

**Rekonstruierbarkeit:** Ein Angreifer mit Lese-Zugriff auf den `nodes`-Vec (z.B. über einen unsicheren Snapshot oder Memory-Dump) kann gelöschte Vektoren uneingeschränkt rekonstruieren. Die Kantenlisten ermöglichen zusätzlich die Rekonstruktion des Nachbarschaftsgraphen.

**SPEC-Bezug:** D-77 verbietet Nullvektoren als Löschung – aber Zeroizing der tatsächlichen Bytes ist nirgends spezifiziert oder implementiert.

**Test `reconstruction_attack_test.rs`:** Prüft korrekt, dass Suchergebnisse den gelöschten Knoten nicht zurückgeben – aber **nicht**, ob die Vektorbytes im RAM tatsächlich gelöscht sind. Die Assertion `nearest_deleted_score <= nearest_baseline_score * 5.0` ist außerdem sehr locker (5×-Faktor).

**Empfehlung:**
1. `do_delete` → nach Tombstone-Setzen: `use zeroize::Zeroize; node.vector.zeroize()` (dafür `VectorData` muss `Zeroize` implementieren).
2. Arena: `zeroize_and_free_node` existiert bereits in [`arena.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/arena.rs#L298) – wird aber **nicht von `do_delete` aufgerufen**.
3. Test: Direkter Speicher-Zugriff prüfen (`nodes[idx].vector` nach Deletion = leer/zeroized).

---

### 2.2 Zeroize bei Rebuild: Vektorbytes gelöschter Knoten überleben Phase 1 (HOCH)

**Befund:** [`rebuild_phase1_snapshot_and_build`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L726-L836) klont alle Knoten:
```rust
all.push((node.doc_id, node.vector.clone(), node.committed_tx, false));
```
Gelöschte Knoten werden optional als `is_deleted=true` übergeben und in den neuen Index insertiert und dann `do_delete` aufgerufen. Die geklonten `Vec<f32>` oder `Vec<u8>` der gelöschten Knoten landen im Heap und werden nach dem Rebuild nicht explizit zeroized.

**Empfehlung:** Nach Abschluss von Phase 2 (`rebuild_phase2_merge_and_swap`): Die alte `nodes`-Liste (die `mem::take`-Kopie) explizit droppen und zeroizen, bevor der Drop-Destruktor greift.

---

### 2.3 Persistenz: Gelöschte Knoten werden in HNSW-Snapshot mitgeschrieben (HOCH)

**Befund:** [`save_internal`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/types.rs#L621) schreibt **alle Knoten** in die `.hnsw`-Datei, inklusive tombstone'd Knoten:
```rust
for (i, node) in nodes.iter().enumerate() {
    // Keine Filterung auf deleted_nodes!
    node_records[i].doc_id = node.doc_id.inner();
    ...
    writer.write_all(&val.to_le_bytes())  // Vektorbytes gelöschter Knoten
}
```
Das bedeutet: Gelöschte Vektoren sind auf Disk persistent, auch nach `save()`. Die Tombstone-Liste (`deleted_nodes`) wird nicht mitgespeichert.

**Konsequenz:** Nach Reload via `load_mmap` ist der Tombstone-Zustand verloren – alle Knoten erscheinen aktiv. Das ist ein Korrektheitsproblem zusätzlich zum Datenschutzproblem.

**Empfehlung:**
1. `save_internal` → Tombstone-Liste mit persistieren (als Bitmap-Feld im Header oder als Anhang).
2. Gelöschte Knoten beim Schreiben zeroizen oder überspringen (letzteres erfordert Anpassung des Offset-Schemas).
3. Test: Load/Save-Roundtrip prüft, dass gelöschte DocIds nicht in Suchergebnissen erscheinen.

---

### 2.4 `LAST_REBUILD_MAP`: Pointer-als-Key Anti-Pattern (MITTEL)

**Befund:** In [`core_rebuild.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L13-L36):
```rust
static LAST_REBUILD_MAP: parking_lot::Mutex<Option<AHashMap<usize, u64>>> = ...;
fn get_last_rebuild_nanos(core: &HnswIndexCore) -> Option<u64> {
    let key = core as *const _ as usize;  // Pointer als Key!
```
Raw-Pointer als HashMap-Key ist konzeptionell fragil: Wenn zwei `Arc<HnswIndexCore>` denselben Heap-Slot erhalten (nach Drop + Neuallokation), kann der Backoff-Timer eines früheren Index auf einen neuen Index angewendet werden. Das `Drop`-Impl bereinigt korrekt, aber zwischen Drop und Bereinigung besteht ein kleines Zeitfenster.

**Besser:** UUID oder AtomicU64-ID pro Instanz als stabiler Key.

---

## 3. ACORN-Filtersuche

### 3.1 ACORN: Nur Layer 0, keine Quantisierungsunterstützung (HOCH)

**Befund:** [`acorn_filtered.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/acorn_filtered.rs#L213-L216):
```rust
let connections = self.inner.resolve_connections(current.index, 0, &ctx)?;
// Layer ist hard-kodiert auf 0
```
Der Layer-Descend (Layers 1..max_layer) wird korrekt ohne Filter durchgeführt, dann aber Layer 0 mit γ-Budget. Das ist das ACORN-Schema. Jedoch: Bei quantisierten Vektoren wird `query_quantized = None` gesetzt (kein quantisierter Query-Pfad in ACORN). Alle Distanzen werden via `resolve_dist(e, query, None, &ctx)` berechnet – also immer asymmetrisch/dequantisiert, niemals symmetric SQ8.

**Empfehlung:** Query für ACORN-Suche ebenfalls quantisieren und symmetric_dist verwenden, um Hot-Path-Distanzberechnungen zu beschleunigen.

### 3.2 ACORN: `visited`-Vec hat O(total_nodes) Speicherbedarf (MITTEL)

**Befund:**
```rust
let mut visited = vec![false; total_nodes];
```
Bei 1 M Knoten: 1 MB Allocation pro Query. Bei hohen QPS (Queries per Second) verursacht das erheblichen GC-Druck.

**Empfehlung:** `AHashSet<usize>` mit `with_capacity(ef_acorn * 4)` oder Bitvec mit lazy Clearing via Generation-Counter.

### 3.3 `valid_matches` Heap: `max_valid_capacity` zu groß (NIEDRIG)

```rust
let max_valid_capacity = ef_acorn.max(k.saturating_mul(10));
```
Bei `ef_acorn = 1000` und `k = 10` wird ein Heap für bis zu 1000 Elemente gehalten, obwohl nur top-k gebraucht werden. Ein fixierter `k * gamma`-Wert wäre tighter.

---

## 4. Recall-/Latenz-Messung und Gates

### 4.1 Kein absolutes Recall@k-Gate bei Löschwellen (KRITISCH)

**Befund:** Es existieren Tests für:
- Determinismus (`determinism_test.rs`, `hnsw_layer_determinism.rs`)
- Partial-Rebuild-Recall (aber mit 15 %-Toleranz, s.o.)
- ACORN-Recall (`acorn_naive_recall.rs`)

Fehlend:
- **Recall@10-Gate nach 1 % gelöschten Knoten** (Erwartung ≥ 0.90)
- **Recall@10-Gate nach 5 % gelöschten Knoten** (Erwartung ≥ 0.85)
- **Recall@10-Gate nach 20 % gelöschten Knoten** (Erwartung ≥ 0.70)
- Ground-Truth via Brute-Force auf `F32` mit exakter Distanz

Solche Gates sind Voraussetzung für jede Löschaussage nach SPEC 1.1.

**Empfehlung:** Dedizierter Test `tests/recall_after_deletion_gates.rs`:
```rust
#[test]
fn recall_gate_1_percent_deletion() { ... assert!(recall >= 0.90); }
#[test]  
fn recall_gate_5_percent_deletion() { ... assert!(recall >= 0.85); }
#[test]
fn recall_gate_20_percent_deletion() { ... assert!(recall >= 0.70); }
```

---

### 4.2 Determinismus: Layer-Seed wird nicht per-Insert propagiert (MITTEL)

**Befund:** [`random_layer`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L91-L96):
```rust
pub(super) fn random_layer(&self) -> usize {
    let r = self.cold.rng.next_unit_f64() as f32;
    ...
}
```
Der `layer_seed`-`AtomicU64` in `HnswHotCore` wird zwar beim Init gesetzt, aber `random_layer` verwendet `self.cold.rng` direkt. Der `layer_seed` Atomic ist dead code (wird nur via `set_layer_seed` geschrieben, nie in `random_layer` gelesen). Dies ist inkonsistent und könnte Verwirrung stiften.

---

## 5. Optimierungspotenziale

### 5.1 `do_delete`: O(N)-Scan für EP-Ersatz (bereits in 1.2)

### 5.2 `search_layer`: Dead-Node-Filter via `deleted_snapshot` pro Nachbar (MITTEL)

```rust
for &neighbor_u32 in connections.iter() {
    let neighbor = neighbor_u32 as usize;
    if deleted_snapshot.contains(neighbor as u64) {
        self.cold.visited_dead_nodes.fetch_add(1, Ordering::Relaxed);
        has_dead_neighbors = true;
    }
    if visited.insert(neighbor) { ... }
```
Gelöschte Nachbarn werden trotzdem durch `visited.insert` geprüft (wasted work). Besser: `continue` vor `visited.insert` wenn `deleted_snapshot.contains`.

**Aktuell:** Gelöschte Nachbarn landen nicht in `results`, weil `deleted_snapshot` in der Ergebnis-Filterung nochmals geprüft wird. Aber Distanzberechnung `resolve_dist` für tote Nachbarn wird nicht aufgerufen (korrekt – `visited.insert` ist vor `resolve_dist`). Dennoch: Atomic-Increment + Bitmap-Lookup für jeden toten Nachbarn ist teuer bei hoher Tombstone-Rate.

### 5.3 `select_neighbors_heuristic_with_batch`: Relaxation-Berechnung unnötig komplex (NIEDRIG)

```rust
let std_dev = variance.sqrt();
let relaxation = if self.cold.config.quantize {
    (0.1 * (1.0 / (1.0 + std_dev))).clamp(0.02, 0.2)
} else { 0.0 };
```
`std_dev` wird für F32-Vektoren nie genutzt (relaxation = 0). Die Varianz-Berechnung ist unnötiger Overhead im F32-Pfad.

### 5.4 `HnswArena::allocate_node`: Free-List-Suche O(|free_list|) (MITTEL)

```rust
for (i, &freed_idx) in free_list.iter().enumerate() {
    if freed_idx < capacities.len() && capacities[freed_idx] >= node_capacity {
        found_free_index = Some(i);
        break;
    }
}
```
Bei vielen gelöschten Knoten wächst die Free-List. Ein binärer Heap (sortiert nach Kapazität) würde O(log N) Reuse ermöglichen.

### 5.5 `save_internal`: Tombstones nicht persistiert (bereits Sicherheitsproblem 2.3)

### 5.6 `rebuild_phase1`: Gesamter Nodes-Vec geklont (MITTEL)

```rust
all.push((node.doc_id, node.vector.clone(), node.committed_tx, false));
```
Bei 1 M Knoten mit 768-dim F32-Vektoren: ~3 GB Heap-Allokation für den Clone. Streaming-Rebuild (chunk-weise) oder In-place-Rebuild würden den Peak-RAM signifikant reduzieren.

### 5.7 `AHashSet` vs `Vec<bool>` in `search_layer` (NIEDRIG)

```rust
let mut visited = AHashSet::with_capacity(ef.saturating_mul(4));
```
Für kleine `ef`-Werte ist `AHashSet` gut. Bei großen `ef` (> 1000) würde ein dichter `Vec<bool>[total_nodes]` (wie in ACORN) schneller sein. Die Wahl hängt vom `ef`-Wert ab; eine adaptive Strategie wäre besser.

---

## 6. Verstöße gegen Architekturinvarianten

### 6.1 `std::fs` direkt in `hnsw/types.rs` und `persistence/mmap.rs` (HOCH)

**Befund:** [`save_internal`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/types.rs#L632) und [`MmapIndex::open`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/mmap.rs#L21) verwenden direkt `std::fs::File::create`, `std::fs::read`, `std::fs::rename`.

**SPEC/AGENTS.md-Verletzung:** „I/O ausschließlich über den Port `Vfs` (D-04). Kein `std::fs` und kein `tokio::fs` außerhalb der Implementierung in `contextra-durable-fs`."

**Schweregrad:** Architekturverletzung. Betrifft direkt `contextra-vector` (Ring 2), das `std::fs` direkt verwendet. DiskANN-Code (`diskann/build.rs`, `diskann/persistence.rs`) verletzt dieselbe Invariante.

### 6.2 `contextra_mvcc` als Abhängigkeit von Ring 2 (ZU PRÜFEN)

**Befund:** [`hnsw/types.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/types.rs#L5):
```rust
use contextra_mvcc::TxBuffer;
```
Laut AGENTS.md: „Verboten: `sys` und `mvcc` (D-88)". `contextra-vector` (Ring 2) importiert `contextra-mvcc`.

**Schweregrad:** Klar dokumentierter Verstoß gegen D-88.

---

## 7. Fehlende Implementierungen (Test-Code referenziert nicht-existente Symbole)

| Symbol | Referenziert in | Status |
|---|---|---|
| `GhostFreeVectorIndex` | `reconstruction_attack_test.rs`, `deletion_ghost_pointer_proptest.rs` | **Nicht in `src/` definiert** |
| `remove_with_graph_repair` | `reconstruction_attack_test.rs`, `deletion_ghost_pointer_proptest.rs` | **Nicht in `src/` definiert** |
| `GraphRepairStats.verified_no_ghost_pointers` | `reconstruction_attack_test.rs` | **Nicht in `src/` definiert** |

Diese Tests würden den Build brechen. Sie beschreiben eine Funktion (`remove_with_graph_repair`), die:
1. Den Knoten tombstonet
2. Nachbarschaft repariert
3. Ghost-Pointer-Freiheit verifiziert

Diese Funktion ist korrekt konzipiert (per Test-Spec), aber **komplett unimplementiert**.

---

## 8. Quantisierung (SQ8)

### 8.1 `train()` vs `try_train()`: `unwrap_or_else` verschluckt Fehler (MITTEL)

```rust
pub fn train(batch: &[&[f32]], dimension: usize) -> Self {
    Self::try_train(batch, dimension).unwrap_or_else(|_| Self { ... })
}
```
Fehlerursache (z.B. NaN in Trainingsdaten) wird nicht geloggt. Der stille Fallback-Quantisierer (mins=0, maxes=1) führt zu schlechtem Recall ohne Warnung.

**Empfehlung:** `unwrap_or_else` → `inspect_err(|e| tracing::error!(...))`.

### 8.2 Quantisierungsversatz bei Rebuild: Dequantisierung → Requantisierung Verlust (MITTEL)

Im Rebuild ([`core_rebuild.rs:820-828`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_rebuild.rs#L820-L828)):
```rust
VectorData::U8(v) => {
    let dequantized = q.dequantize(&v)?;
    new_index.inner.do_insert(doc_id, &dequantized, committed_tx)?;
}
```
Dequantisierung → `do_insert` → Requantisierung mit ggf. neu kalibrierten Grenzen. Jeder Rebuild-Zyklus akkumuliert Quantisierungsfehler. Bei 10 Rebuilds und SQ8 ist der kumulative Fehler messbar.

**Empfehlung:** Originale F32-Vektoren im Index mitführen (als optionalen `original_f32`-Buffer) oder bei Rebuild aus der Quelle re-trainieren. Alternativ: Metriken für kumulative Drift nach Rebuild-Zyklen.

---

## 9. Offene Messpflichten (Erasure-Report-Schichten)

| Messung | Status | Gate-Wert |
|---|---|---|
| MK1: Vektorbytes nach Deletion zeroized | **Nicht erfasst** | Alle Bytes = 0x00 |
| MK2: Kantenlisten nach Deletion zeroized | **Nicht erfasst** | Alle Einträge = 0 |
| MK3: Rekonstruierbarkeit aus Nachbarn | **Nicht erfasst** | Cosine-Sim < 0.5 |
| Recall@10 nach 1 % Deletion | **Nicht erfasst** | ≥ 0.90 |
| Recall@10 nach 5 % Deletion | **Nicht erfasst** | ≥ 0.85 |
| Recall@10 nach 20 % Deletion | **Nicht erfasst** | ≥ 0.70 |
| Deterministischer Rebuild (Seed) | Implementiert | Bit-identisch ✓ |
| Snapshot-Isolation bei Rebuild | Implementiert | ✓ |

> [!CAUTION]
> Die Löschaussage für den Vektorindex ist derzeit **vollständig unbelegt**. Kein einziger der sechs Erasure-Messpunkte ist erfasst. Bis MK1-MK3 und die Recall-Gates implementiert und grün sind, darf keine externe Aussage über die Löschfähigkeit des Vektorindex gemacht werden.

---

## 10. Priorisierte Empfehlungsliste

| Priorität | Befund | Aufwand |
|---|---|---|
| P0 | `GhostFreeVectorIndex` + `remove_with_graph_repair` implementieren | Hoch |
| P0 | Vektorbytes nach `do_delete` zeroizen (MK1) | Niedrig |
| P0 | Recall@k-Gates bei 1/5/20 % Deletion | Mittel |
| P1 | Nachbarschafts-Reparatur (Reconnect nach Deletion) | Hoch |
| P1 | Tombstone-Persistenz in `save_internal` | Mittel |
| P1 | `std::fs` → Vfs-Port-Umstieg (D-04-Konformität) | Hoch |
| P1 | `contextra_mvcc`-Import aus Ring 2 entfernen (D-88) | Mittel |
| P2 | Surrogat-Knoten-Konzept (`VectorData::Surrogate`) | Hoch |
| P2 | Arena-Zeroize bei Deletion (MK2) | Niedrig |
| P2 | Rekonstruierbarkeits-Test (MK3) | Niedrig |
| P3 | EP-Ersatz O(1) statt O(N) | Mittel |
| P3 | ACORN: `visited` Vec statt `vec![false; N]` | Niedrig |
| P3 | Free-List → BinaryHeap | Niedrig |

---

*Ende des Audits. Normativ: `docs/SPEC.md` 4.3.0.*
