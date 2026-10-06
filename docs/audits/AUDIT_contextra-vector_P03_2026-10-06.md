# Review: Mission P03 — HNSW/DiskANN Korrektheits- und Speichersicherheitsprüfung

## 1. Zusammenfassung

Die Sicherheits- und Korrektheitsprüfung von `crates/contextra-vector/` (Ring 0) bestätigt, dass die Crate-Boundary durch `#![forbid(unsafe_code)]` in `crates/contextra-vector/src/lib.rs` geschützt ist und sämtliche SIMD-Intrinsics strikt nach `contextra-simd` sowie Memory-Mapping nach `contextra-sys` ausgelagert sind. Der HNSW-Graph baut seine Schichten und Nachbarlisten über eine Diversity-Heuristik auf und hält In-Memory-Kanten bidirektional synchron. Allerdings wurden vier kritische/hohe Schwachstellen identifiziert:
1. In `hnsw/arena.rs` führt das Fehlen von Fehlertoleranz bei Speicherallokationsfehlern zu Panik oder Deadlock.
2. In `hnsw/deletion.rs` führt ein vorzeitiger Abbruch des Verification-Budgets bei der Ghost-Pointer-Prüfung zu falsch-positiver Bestätigung der Zeigerfreiheit.
3. In `quantize/mod.rs` fällt die Quantisierer-Schulung bei invaliden/NaN-Eingaben unbemerkt auf ein Platzhalter-Objekt mit Null-Skalierung zurück, was nachfolgende Distanzberechnungen korrumpiert.
4. In `diskann/persistence.rs` führt der Crash-Recovery-Pfad bei unvollständigen `write_incremental`-Operationen zur doppelten Indexierung un-deduplizierter Vektoren.

---

## 2. Geprüfte Dateien

- `crates/contextra-vector/src/lib.rs` — Crate Root, Sicherheits-Grenzdefinition (`#![forbid(unsafe_code)]`).
- `crates/contextra-vector/src/hnsw/vector_index_impl.rs` — Haupt-Facade für `HnswIndex` und Delegate an den Hot/Cold Core.
- `crates/contextra-vector/src/hnsw/core_insert.rs` — HNSW-Knoten-Einfügepfad (`compute_insert`, `apply_insert`).
- `crates/contextra-vector/src/hnsw/core_search.rs` — Greedy Graph Search & Multi-Layer Traversal.
- `crates/contextra-vector/src/hnsw/core_rebuild.rs` — Heuristische Nachbarauswahl (`select_neighbors_heuristic_with_batch`) & Rebuild-Logik.
- `crates/contextra-vector/src/hnsw/deletion.rs` — Graph-Reparatur und Tombstone-Löschung (`do_delete`, `verify_no_ghost_pointers`).
- `crates/contextra-vector/src/hnsw/arena.rs` — Contiguous Arena-Allocator & Backlink-Tabellen für Nachbarlisten.
- `crates/contextra-vector/src/hnsw/types.rs` — Datenstrukturen (`HnswNode`, `HnswHotCore`, `HnswColdCore`).
- `crates/contextra-vector/src/quantize/mod.rs` — 8-Bit Skalar-Quantisierung (`ScalarQuantizer`).
- `crates/contextra-vector/src/quantize_rabitq.rs` — RaBitQ Random Rotation Quantizer.
- `crates/contextra-vector/src/distance.rs` — Safe Abstraktionsschicht für SIMD-Distanzfunktionen.
- `crates/contextra-vector/src/diskann/persistence.rs` — Persistence & Delta-Flushing für DiskANN.
- `crates/contextra-vector/src/diskann/format.rs` — Header-, Footer- & WAL-Binärformate für DiskANN.
- `crates/contextra-vector/src/persistence/mmap.rs` — Zero-Copy Mmap-Lesepfad für persistierte HNSW-SSTables.

---

## 3. Extrahierte Invarianten

- **I-1 (Crate-Safety Boundary):** `contextra-vector` erzwingt 100% Safe Rust über `#![forbid(unsafe_code)]`. SIMD-Intrinsics sind strikt nach `contextra-simd` und Low-Level Mmap an `contextra-sys` delegiert.
- **I-2 (HNSW Diversity Heuristic):** Nachbarlisten-Pruning begrenzt Kanten auf $M$ bzw. $M_{\max}$ ($2M$ auf Layer 0) unter Anwendung der Heuristik von Malkov & Yashunin (Erhaltung der räumlichen Diversität gegenüber reinem Top-$M$ Nearest Neighbor Pruning).
- **I-3 (Symmetrische Kanten-Synchrizität):** Wenn Knoten $A$ Knoten $B$ als Nachbarn wählt, erhält $B$ im selben Commit-Schritt einen Backlink-Eintrag zu $A$, sodass keine einseitigen Phantom-Edges entstehen.
- **I-4 (INV-DELETION-2 / Ghost Pointer Absence):** Nach der physischen Löschung eines Knotens existieren im verbliebenen Graphen keinerlei Referenzen/Zeiger mehr auf die gelöschte Knoten-ID.
- **I-5 (P24 / Lokale Löschkosten):** Graph-Reparatur bei Löschungen ist lokal auf das 2.-Ordnungs-Nachbarschafts-Umfeld begrenzt.
- **I-6 (NaN/Inf Input Validation):** Sämtliche Vektoren und Suchabfragen werden vor Distanzberechnungen auf NaN und Unendlich-Werte geprüft, um Heap-Korruption bei Top-$k$-Suchen zu verhindern.
- **I-7 (DiskANN Crash Consistency & HMAC Integrity):** Disk-backed Vektor-Indizes werden mittels atomarem `rename` plus Parent-Directory `fsync` geschrieben und beim Laden gegen HMAC-SHA256 Prüfsummen validiert.
- **I-8 (Zero-Panic Guarantee):** Kein Panik-Aufruf (`unwrap`, `expect`, Slicing Out-of-Bounds) darf bei fehlerhaften/korrupten Eingaben oder Speicherallokationsfehlern im Produktionspfad ausgelöst werden.

---

## 4. Befunde

### [CRITICAL] F-01 — Panic / Deadlock-Anfälligkeit in Arena Allocation bei Speicherallokationsfehlern
- **Ort:** `crates/contextra-vector/src/hnsw/arena.rs:387` in `apply_insert`
- **Invariante betroffen:** I-8 (Zero-Panic Guarantee), I-3 (Symmetrische Kanten-Synchrizität)
- **Beleg:**
  ```rust
  let ram_idx = self
      .hot
      .arena
      .allocate_node(prepared.new_layer, m, &prepared.final_connections)
      .unwrap_or_else(|_| prepared.new_idx.saturating_sub(mmap_count));
  ```
- **Angriffs-/Fehlerszenario:**
  1. `allocate_node` schlägt fehl (z. B. wenn `raw_capacity == 0` oder Memory-Limit erreicht ist) und gibt `Err(ContextraError::InvalidInput)` zurück.
  2. `unwrap_or_else` fängt den Fehler ab und weist `ram_idx` den Ersatzwert `prepared.new_idx.saturating_sub(mmap_count)` zu.
  3. Nachfolgend versucht `apply_insert`, `nodes[ram_idx]` zu beschreiben. Wenn `ram_idx >= nodes.len()`, wird ein `push` ausgeführt. Allerdings wurde in der Arena kein tatsächlicher Speicher-Slot reserviert.
  4. Spätere Aufrufe von `get_ram_node_connections` greifen mit Slicing auf ungepflegte Arena-Offsets zu, was zu Slicing-Panics oder Out-of-Bounds Indexing führt.
- **Auswirkung:** Silent State Corruption im Arena-Allocator und darauffolgende Thread-Panics im Produktions-Insert-Pfad unter Speicherdruck.
- **Empfehlung:** `allocate_node` muss Fehler explizit über `Result<usize>` an den Aufrufer weiterleiten. Bei Fehler darf der Knoten nicht unvollständig in den Graphen eingefügt werden; die Transaktion muss sauber abgebrochen werden.

---

### [HIGH] F-02 — Unvollständige Ghost-Pointer-Verifikation bei Löschungen durch vorzeitigen Budget-Abbruch
- **Ort:** `crates/contextra-vector/src/hnsw/deletion.rs:335-345` / `verify_no_ghost_pointers`
- **Invariante betroffen:** I-4 (INV-DELETION-2 / Ghost Pointer Absence)
- **Beleg:**
  ```rust
  for i in 0..total_nodes {
      if inspected_nodes >= budget {
          break;
      }
      ...
  ```
- **Angriffs-/Fehlerszenario:**
  1. Ein Knoten $T$ mit hohem Grad wird aus dem HNSW-Graphen gelöscht.
  2. `delete_node` führt die Kantenreparatur aus.
  3. Zur Verifikation wird `verify_no_ghost_pointers` aufgerufen.
  4. Die Iteration startet bei Node Index `i = 0` und prüft Knoten sequentiell. Sobald `inspected_nodes >= budget` erreicht ist, bricht die Schleife mit `break` ab.
  5. Falls verbleibende Ghost-Pointer auf $T$ bei Knoten mit Index `i > budget` existieren, werden diese nie inspiziert.
  6. Die Funktion gibt `ghost_pointers = 0` zurück, und die Löschung wird als "erfolgreich verifiziert" protokolliert, obwohl noch Geisterzeiger im Graphen existieren.
- **Auswirkung:** Falsch-positive Verifikationsbestätigung (`verified_no_ghost_pointers: true`). Bei Suchen werden gelöschte Knoten dereferenziert, was zu falschen K-NN-Ergebnissen oder Fehlern beim Laden führt.
- **Empfehlung:** Die Verifikation muss zielgerichtet die Nachbarn der reparierten Knoten durchsuchen (BFS/DFS-Traverse der 2.-Ordnungs-Nachbarschaft), anstatt den linearen Index-Raum `0..total_nodes` ab dem Anfang unvollständig zu scannen.

---

### [HIGH] F-03 — Stiller Rückfall auf Null-Skalierungs-Quantisierer bei Schulungsfehlern
- **Ort:** `crates/contextra-vector/src/quantize/mod.rs:177-185` & `195-204`
- **Invariante betroffen:** I-6 (NaN/Inf Input Validation), I-8 (Zero-Panic Guarantee)
- **Beleg:**
  ```rust
  pub fn train(batch: &[&[f32]], dimension: usize) -> Self {
      Self::try_train(batch, dimension).unwrap_or_else(|_| Self {
          mins: vec![0.0; dimension],
          maxes: vec![1.0; dimension],
          scales: vec![255.0; dimension],
          inv_scales: vec![1.0 / 255.0; dimension],
          dimension,
          total_queries: AtomicU64::new(0),
          out_of_range_queries: AtomicU64::new(0),
      })
  }
  ```
- **Angriffs-/Fehlerszenario:**
  1. Eine Anwendung schult den Skalarquantisierer mit einem Batch, der invalide Dimensionen oder ein NaN-Element enthält.
  2. `try_train` gibt `Err(ContextraError::InvalidInput)` zurück.
  3. `train` verschluckt diesen Fehler stumm via `unwrap_or_else` und gibt ein Dummy-Quantisierer-Objekt mit Standardbereich `[0.0, 1.0]` zurück.
  4. Wenn die echten Vektoren Werte außerhalb von `[0.0, 1.0]` haben (z. B. Einheitsvektoren mit Komponenten in `[-1.0, 1.0]`), werden alle negativen Werte hart auf 0 geklemmt.
- **Auswirkung:** Massiver Recall-Einbruch bei der quantisierten Vektorsuche ohne jegliche Fehlermeldung oder Warnung im Log.
- **Empfehlung:** Die veralteten panik- bzw. verschluckenden `train`- und `train_with_percentiles`-Methoden deprecaten oder entfernen. Ausschließlich `try_train` und `try_train_with_percentiles` verwenden und Fehler an den Aufrufer weiterleiten.

---

### [HIGH] F-04 — Crash-Recovery-Inkonsistenz bei DiskANN Delta-Flushing
- **Ort:** `crates/contextra-vector/src/diskann/persistence.rs:48-94` in `persist_delta_sync`
- **Invariante betroffen:** I-7 (DiskANN Crash Consistency)
- **Beleg:**
  ```rust
  let tmp_path = self.inner.config.index_path.with_extension("delta.tmp");
  self.write_incremental_to_file_sync(&tmp_path, &pending)?;
  ...
  std::fs::rename(&tmp_path, &index_path)...
  let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
  if pending_wal.exists() {
      let _ = std::fs::remove_file(&pending_wal);
  }
  ```
- **Angriffs-/Fehlerszenario:**
  1. `persist_delta_sync` schreibt den inkrementellen Index in `delta.tmp` und benennt `delta.tmp` erfolgreich in `index_path` um.
  2. Direkt nach dem `rename`, aber VOR dem Löschen von `pending.wal` (Zeile 91), stürzt der Prozess ab (Power-Loss / SIGKILL).
  3. Beim Neustart ruft `load_sync` auf. `index_path` existiert, aber `pending.wal` existiert ebenfalls noch.
  4. Da `pending.wal` noch existiert, werden die darin enthaltenen Vektoren beim erneuten `load_sync` erneut in den Speicher geladen und beim nächsten Persistierungszyklus als neue Knoten angehängt.
- **Auswirkung:** Duplizierung von Vektoren auf Disk und im Index nach Crash-Recovery, was zu doppelten Suchergebnissen und Index-Wachstum führt.
- **Empfehlung:** `pending.wal` muss vor dem Abschließen des Delta-Flushes entweder atomar abgeschnitten/gelöscht oder mit einer Sequence-ID versehen werden, die mit dem Header des Haupt-Index synchronisiert ist.

---

## 5. Optimierungspotenzial

1. **BacklinkTable HashMap Overhead:** `BacklinkTable` in `hnsw/arena.rs` nutzt ein `AHashMap<(usize, usize), Vec<u32>>`. Bei großen Indizes führt dies zu vielen kleinen Heap-Allokationen. Eine flache Slot-basierte Indizierung im Arena-Puffer würde Cache-Misses reduzieren.
2. **RaBitQ Matrix-Orthogonalisierung:** In `quantize_rabitq.rs` verwendet Gram-Schmidt bei der Generierung der Rotationsmatrix $O(D^3)$ Operationen auf der CPU. Für feste Dimensionen könnte eine Householder-Transformation oder Fast Walsh-Hadamard Transformation (FWHT) verwendet werden.

---

## 6. Offene Fragen / nicht verifizierbar ohne Laufzeit-Tests

- **Concurrent Lock Contention unter extremer Thread-Anzahl:** Die Interaktion zwischen `nodes.read()` in `core_search.rs` und `nodes.write()` in `core_insert.rs` lässt sich statisch als korrekt verifizieren (Keine Deadlock-Zyklen gefunden), verlangt jedoch Stress-Tests unter Loom/TSAN bei >128 parallelen Schreibern.

---

## 7. Jules-Task-Karten

```yaml
id: JULES-P03-01
title: Fix Arena Allocation failure handling and remove fallback in apply_insert
severity: CRITICAL
files_to_touch:
  - crates/contextra-vector/src/hnsw/arena.rs
  - crates/contextra-vector/src/hnsw/core_insert.rs
context: >
  In apply_insert (core_insert.rs), allocate_node errors are caught with unwrap_or_else
  and assigned an unallocated fallback index, corrupting internal arena offsets and causing
  panics during neighbor list slicing.
acceptance_criteria:
  - allocate_node errors are propagated as Result to apply_insert callers.
  - No fallback index is assigned if arena allocation fails.
  - Transactions abort cleanly without corrupting the graph state.
test_to_add: >
  Integration test simulating arena allocation error and verifying clean Result::Err propagation.
non_goals: >
  Do not redesign the entire arena buffer layout in this task.
```

```yaml
id: JULES-P03-02
title: Fix ghost pointer verification scan targeted neighborhood traversal in deletion.rs
severity: HIGH
files_to_touch:
  - crates/contextra-vector/src/hnsw/deletion.rs
context: >
  verify_no_ghost_pointers scans total_nodes linearly from index 0 and breaks early when budget
  is reached, falsely certifying deletion when ghost pointers exist beyond the budget boundary.
acceptance_criteria:
  - verify_no_ghost_pointers performs a targeted BFS/DFS scan over the 2nd-order neighborhood.
  - Verification fails if any residual pointer to the deleted node remains anywhere in the neighborhood.
test_to_add: >
  Test case constructing a graph where residual pointers exist outside the initial node range and verifying failure.
non_goals: >
  Do not alter the external DeletionStats return structure.
```

```yaml
id: JULES-P03-03
title: Remove silent dummy fallback in ScalarQuantizer train methods
severity: HIGH
files_to_touch:
  - crates/contextra-vector/src/quantize/mod.rs
context: >
  ScalarQuantizer::train and train_with_percentiles silently swallow errors via unwrap_or_else,
  returning zero-scale dummy quantizers that clamp out-of-bounds inputs and ruin search recall.
acceptance_criteria:
  - Deprecate or remove unwrap_or_else fallbacks in train and train_with_percentiles.
  - Fallback returns an error or caller uses try_train explicitly.
test_to_add: >
  Unit test asserting that training on invalid input (NaN/dimension mismatch) returns Err and does not produce a dummy quantizer.
non_goals: >
  Do not modify the underlying quantization formula or scale math.
```

```yaml
id: JULES-P03-04
title: Synchronize DiskANN pending WAL truncation with index rename recovery
severity: HIGH
files_to_touch:
  - crates/contextra-vector/src/diskann/persistence.rs
context: >
  A crash between index_path atomic rename and pending.wal removal causes re-indexing of
  already persisted vectors upon recovery.
acceptance_criteria:
  - Pending WAL removal or sequence invalidation is atomic relative to index load recovery.
  - Reloading after simulated crash post-rename does not create duplicate vectors.
test_to_add: >
  Fault injection test simulating crash post-rename and asserting zero duplicate vector entries upon reload.
non_goals: >
  Do not modify the DiskANN binary header structure.
```

---

## 8. Unsafe-Code-Inventar

| Datei:Zeile | Behauptete Invariante | Verifikationsstatus |
| --- | --- | --- |
| `crates/contextra-vector/src/lib.rs:18` | `#![forbid(unsafe_code)]` erzwingt Zero Unsafe Code im gesamten Crate. | BEWIESEN SICHER (100% Safe Rust) |

*Hinweis: Historische `unsafe`-Blöcke für SIMD-Operationen wurden vollständig nach `contextra-simd` und Memory-Mapping nach `contextra-sys` (`contextra_sys::mmap_readonly`) ausgelagert.*
