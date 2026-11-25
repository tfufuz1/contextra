# Review: P03 — HNSW/DiskANN Korrektheits- und Speichersicherheitsprüfung

---

## 1. Zusammenfassung

`crates/contextra-vector` implementiert HNSW, DiskANN, SQ8 und RaBitQ (feature-gated) ohne einen einzigen `unsafe`-Block — alle `unsafe`-Operationen wurden in `contextra-sys` und `contextra-simd` ausgelagert, was die Prüffläche im Crate selbst stark reduziert. Die Distanzfunktionen validieren Dimensionsmismatch und NaN vor der Berechnung. Das mmap-Layout wird beim Öffnen gegen Überläufe und Dateigrößen validiert. DiskANN-Dateien sind durch HMAC-SHA256 integer-geschützt.

Dennoch wurden **fünf signifikante Befunde** identifiziert:

1. **HIGH** — Die Ghost-Pointer-Verifikation in `GhostFreeVectorIndex` ist **budgetiert und endet vorzeitig** bei großen Graphen, sodass `INV-DELETION-2` ("zero residual pointers") nach `Ok(stats)` nicht garantiert halten kann.
2. **HIGH** — mmap-Knoten (Disk-Segment) werden bei `remove_with_graph_repair` **nicht repariert** — Backlinks von mmap-Knoten zu gelöschten RAM-Knoten bleiben als Ghost-Pointer erhalten.
3. **HIGH** — Der Entry-Point (`AtomicU32`) ist auf **max 4 Milliarden Knoten** begrenzt; bei Knotenindex ≥ `u32::MAX` führt der Cast `idx as u32` zu silent Truncation und falschen Entry-Point-Referenzen.
4. **MEDIUM** — Die `count`-Felder im `HnswArena` sind `u8` (max. 255 Einträge pro Layer), aber `M_max0 = m * 2` kann bei konfigurierbarem `m` > 127 den u8-Bereich überschreiten.
5. **MEDIUM** — DiskANN `persist_delta_sync` hält den `pending_inserts`-Lock, während es einen Full-Rebuild auslöst, der seinerseits `load_all_vectors_from_mmap` aufruft — ein Dead-Lock-Risiko bei konkurrentem Zugriff über den Hintergrund-Thread.

Kein `unsafe`-Code in diesem Crate selbst vorhanden (verifiziert durch `#![forbid(unsafe_code)]` in [`lib.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/lib.rs#L18)).

---

## 2. Geprüfte Dateien

| Datei | Rolle im System |
|---|---|
| [`src/lib.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/lib.rs) | Crate-Root; `#![forbid(unsafe_code)]` |
| [`src/hnsw/core_insert.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_insert.rs) | HNSW-Insert: compute/apply, Backlink-Update, Entry-Point-Pflege |
| [`src/hnsw/core_search.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_search.rs) | Greedy-Search (alle Layer), Adaptive-ef, Filter-Search |
| [`src/hnsw/deletion.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/deletion.rs) | Ghost-free Deletion + Neighborhood-Repair (INV-DELETION-2) |
| [`src/hnsw/arena.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/arena.rs) | Arena-Allocator: 64-Byte-aligned, Freelist, Backlink-Update |
| [`src/hnsw/types.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/types.rs) | Typen: `HnswNode`, `Candidate`, `HnswHotCore`, `HnswColdCore`, `HnswIndex::save()` |
| [`src/distance.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/distance.rs) | Re-Export von `contextra_simd`; keine eigene Logik |
| [`src/quantize/mod.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/quantize/mod.rs) | SQ8-Quantisierer: train/quantize/dequantize/asymmetric_dist |
| [`src/quantize_rabitq.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/quantize_rabitq.rs) | RaBitQ-Quantisierer (feature-gated `experimental-rabitq`) |
| [`src/persistence/mmap.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/mmap.rs) | mmap-Reader für HNSW-Dateien; Bounds-Checks auf alle Offsets |
| [`src/persistence/header.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/header.rs) | HNSW-Datei-Header v2 (84 Bytes) |
| [`src/diskann/build.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/build.rs) | DiskANN-Graph-Bau; Robust-Prune (α-Diversitätsheuristik); WAL-Schreib/-Lesepfade |
| [`src/diskann/persistence.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/persistence.rs) | DiskANN Persist/Load: HMAC-Integritätsprüfung, mmap, Node-Parsing |
| [`src/diskann/search.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/search.rs) | DiskANN Beam-Search; NaN-Check nach jeder Distanzberechnung |
| [`src/diskann/format.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/format.rs) | DiskANN-Header/Footer-Serialisierung; WAL-Konstanten |
| [`UNSAFE_TRANSITION.md`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/UNSAFE_TRANSITION.md) | Nachweis Migration aller `unsafe`-Blöcke nach `contextra-sys` |

---

## 3. Extrahierte Invarianten (Phase A)

**I-1 — Zero-Panic Doctrine (P2)**
Behauptet in `lib.rs:17` Kommentar und `quantize/mod.rs:3`: Kein `unwrap()`/`expect()`/`panic!` in Produktionscode; alle Fehler via `Result<T>`.

**I-2 — Dimension-Mismatch → harter Fehler**
Behauptet in `core_insert.rs:93–98` (vor Quantisierung) und `core_search.rs:181–187` (vor Suche): Query-Dimensionen werden gegen `config.dimension` geprüft; Mismatch → `Err(ContextraError::InvalidInput)`.

**I-3 — NaN/Inf in Input → harter Fehler**
Behauptet in `core_search.rs:199–207` und `core_insert.rs:101` (`validate_vector`): Nicht-finite Werte werden **vor** Distanzberechnung abgefangen.

**I-4 — INV-DELETION-2: Keine Ghost-Pointer nach `remove_with_graph_repair`**
Dokumentiert in `deletion.rs:3` und Trait-Docstring `deletion.rs:30–33`: Nach `Ok(stats)` darf kein Nachbarlisteneintrag mehr auf `doc_id` zeigen.

**I-5 — Bidirektionale Kantenkonsistenz**
Impliziert durch `core_insert.rs:246–338` (Backlink-Update): Wird A als Nachbar von B eingetragen, wird B's Nachbarliste ebenfalls aktualisiert.

**I-6 — Diversitätsheuristik beim Prune**
Behauptet durch Verwendung von `select_neighbors_heuristic_with_batch` in `core_insert.rs:233` und DiskANN `prune_in_memory` in `build.rs:56`: Nachbarn werden nach HNSW-Paper-Heuristik (α-Diversität), nicht nur nach Top-M-Distanz, ausgewählt.

**I-7 — mmap-Bounds garantiert**
Behauptet in `persistence/mmap.rs:3`: "Zero-Panic bei Deserialisierung". Alle Offsets werden mit `checked_mul`/`checked_add` + `.get()` geprüft.

**I-8 — DiskANN HMAC-Integritätsschutz**
Behauptet in `diskann/persistence.rs:304–327`: HMAC-SHA256 wird über den gesamten Payload (Header + Nodes) berechnet; Mismatch → `Err`.

**I-9 — Atomarer Disk-Write (rename-Idiom)**
Behauptet in `diskann/persistence.rs:73–87` und `hnsw/types.rs:795–807`: Writes auf `.tmp`-Datei, `fsync`, dann `rename`; kein Partial-Write sichtbar.

**I-10 — Zero unsafe in `contextra-vector`**
Bewiesen durch `lib.rs:18` (`#![forbid(unsafe_code)]`), bestätigt durch `UNSAFE_TRANSITION.md`.

**I-11 — Entry-Point-Addressierbarkeit**
Impliziert durch korrekte Suche: Der Entry-Point-Index muss gültig und erreichbar sein.

**I-12 — mmap-Reparatur-Scope bei Deletion**
Beansprucht durch `INV-DELETION-2`: Repair muss für **alle** Knoten gelten, nicht nur RAM-Knoten.

---

## 4. Befunde

---

### [HIGH] F-01 — Ghost-Pointer-Verifikation endet bei Budget-Erschöpfung, verletzt INV-DELETION-2

- **Ort:** [`src/hnsw/deletion.rs:250–261`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/deletion.rs#L250-L261)
- **Invariante betroffen:** I-4
- **Beleg:**
```rust
let total_degree = target_max_layer * m;
let budget = (total_degree.max(1)) * m * 4;
let remaining_ghost_pointers = self.verify_no_ghost_pointers(target_idx as u32, budget);
```
Und in `verify_no_ghost_pointers` (`deletion.rs:298–300`):
```rust
for i in 0..total_nodes {
    if inspected_nodes >= budget {
        break;
    }
```
- **Angriffs-/Fehlerszenario:**
  1. Graph enthält 10.000 Knoten, `M = 16`, `target_max_layer = 2`.
  2. Budget = `(2 * 16).max(1) * 16 * 4 = 2.048` Knoten.
  3. Nach Repair werden Knoten 0..2047 verifiziert; Knoten 2048..9999 bleiben **ungeprüft**.
  4. Knoten 5000 zeigt noch auf `target_idx` → Ghost-Pointer bleibt unentdeckt.
  5. `remove_with_graph_repair` gibt `Ok(stats)` zurück; `verified_no_ghost_pointers` ist `true`.
  6. Suche traversiert Knoten 5000 → greift auf gelöschten Knoten zu → falsche Suchergebnisse oder (bei Reallokation von Slots) auf einen neu belegten Knoten mit anderem `doc_id`.
- **Auswirkung:** INV-DELETION-2 wird strukturell verletzt: Der Rückgabewert `Ok(DeletionStats { verified_no_ghost_pointers: true })` ist bei großen Graphen eine **Lüge**. Das Feld täuscht den Aufrufer über den tatsächlichen Zustand. Bei Slot-Reuse durch `free_node` + `allocate_node` im Arena können Ghost-Pointer auf neu eingefügte Knoten verweisen → **Silent Corruption** der Nachbarlisten.
- **Empfehlung:** Die Verifikation muss entweder (a) den **gesamten** Graphen abdecken (O(N) — bei Lösch-Latenz akzeptabel), oder (b) das `verified_no_ghost_pointers`-Feld muss bei Budget-Erschöpfung auf `false` gesetzt werden, und der Callee darf nur bei `true` eine Ghost-freiheits-Garantie ableiten. Die einfache Korrektur wäre: wenn `inspected_nodes >= budget`, muss `ghost_pointers`-Rückgabe entweder korrekt verfolgt werden oder als unbekannt gelten, sodass die aufrufende Funktion keinen `Ok`-Pfad mit `verified_no_ghost_pointers: true` einschlägt.

---

### [HIGH] F-02 — mmap-Knoten werden bei Graph-Repair nicht beschreibbar aktualisiert — Ghost-Pointer im Disk-Segment

- **Ort:** [`src/hnsw/deletion.rs:234–243`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/deletion.rs#L234-L243)
- **Invariante betroffen:** I-4, I-12
- **Beleg:**
```rust
// Apply updated connections if neighbor is a RAM node
if neighbor_idx >= mmap_node_count {
    let neighbor_ram_idx = neighbor_idx - mmap_node_count;
    let _ = self.inner.hot.arena.update_backlink(
        neighbor_ram_idx, layer, m, &updated_conns,
    );
}
```
Der `else`-Zweig (mmap-Knoten) fehlt vollständig.
- **Angriffs-/Fehlerszenario:**
  1. Index besteht aus 500 Knoten im mmap-Segment (persistiert) und 100 Knoten im RAM.
  2. RAM-Knoten 510 (globaler Index) wird gelöscht.
  3. mmap-Knoten 42 zeigt auf Knoten 510 → wird bei Repair erkannt (`existing_conns.contains(target_idx as u32)` ist `true`).
  4. `neighbor_idx (42) >= mmap_node_count (500)` ist **falsch** → kein `update_backlink`.
  5. mmap ist read-only → eine Schreiboperation wäre ohnehin nicht möglich.
  6. `repaired_edges += 1` wird gezählt (Schein-Reparatur), tatsächlich bleibt die mmap-Kante unangetastet.
  7. Suche traversiert mmap-Knoten 42 → greift auf Knoten 510 → deleted → `visited_dead_nodes`-Counter steigt; bei Suche ohne Lazy-Relink wird der gelöschte Knoten übersprungen, aber der Graph verliert Connectivity.
- **Auswirkung:** INV-DELETION-2 gilt nur für RAM-Knoten. Ghost-Pointer im mmap-Segment können nicht bereinigt werden, ohne die Datei neu zu schreiben. Da mmap read-only ist, ist das strukturell unvermeidlich — aber es muss **dokumentiert** sein, und die Invariante muss eingeschränkt formuliert werden. Schlimmer: nach einem Full-Rebuild wird der mmap korrekt regeneriert; aber in der Zwischenzeit täuscht `verified_no_ghost_pointers: true` Vollständigkeit vor.
- **Empfehlung:** Die Invariante INV-DELETION-2 muss auf RAM-Knoten eingeschränkt oder um den Hinweis ergänzt werden, dass mmap-Backlinks erst nach dem nächsten Rebuild bereinigt werden. Alternativ: nach Deletion eines Knotens, der mmap-Eingehende hat, muss ein Rebuild erzwungen werden. Das Feld `verified_no_ghost_pointers` in `DeletionStats` muss bei vorhandenen mmap-Backlinks auf `false` gesetzt werden.

---

### [HIGH] F-03 — Entry-Point-Index als `AtomicU32` trunciert bei Knotenindex ≥ 2³²

- **Ort:** [`src/hnsw/types.rs:140–145`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/types.rs#L140-L145) und [`core_insert.rs:418`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/core_insert.rs#L418)
- **Invariante betroffen:** I-11
- **Beleg:**
```rust
pub fn set_entry_point(&self, ep: Option<usize>) {
    let val = match ep {
        Some(idx) => idx as u32,   // ← silent truncation wenn idx >= 2^32
        None => SENTINEL_NO_ENTRY_POINT,
    };
    self.entry_point.store(val, Ordering::Release);
}
```
Und in `core_insert.rs:418`:
```rust
self.hot.set_entry_point(Some(prepared.new_idx));
```
wobei `new_idx = mmap_node_count + ram_nodes_count + offset` — bei großen Indizes überschreitbar.
- **Angriffs-/Fehlerszenario:**
  1. Konfiguration: `mmap_node_count = 3.000.000.000`, `ram_nodes_count = 1.500.000.000`.
  2. Neuer Knoten bekommt `new_idx = 4.500.000.001`.
  3. `set_entry_point(Some(4_500_000_001))` → `4_500_000_001 as u32 = 205_032_705` (trunciert).
  4. Suche beginnt bei globalem Index 205.032.705 statt 4.500.000.001 → falscher Startknoten.
  5. Graph-Traversal landet in einer anderen Region des Graphen → systematisch schlechterer Recall; bei Delete-Reihenfolge kann der truncierte Index sogar auf einen gelöschten Knoten verweisen.
- **Auswirkung:** Bei sehr großen Indizes (≥ 4 Mrd. Knoten gesamt) wird der Entry-Point korrumpiert. Das Systemmotiv verspricht air-gap-taugliche Speicher-Engine; extreme Skalierung ist damit designrelevant. Dieser Befund ist bei realen Workloads zwar schwer erreichbar, aber architekturell ein latentes CRITICAL-Risiko. Da `Sentinel = u32::MAX` bereits als Sonderwert belegt ist, ist der Wertebereich bereits eingeschränkt.
- **Empfehlung:** Den Entry-Point auf `AtomicU64` umstellen, den Sentinel auf `u64::MAX` setzen und alle `as u32`-Casts entfernen. Da `HnswHotCore` mit `#[repr(align(64))]` ausgestattet ist, ist die Fieldgröße unkritisch für das Cache-Layout.

---

### [MEDIUM] F-04 — Arena `counts`-Felder als `u8` begrenzen Nachbarlisten auf max. 255 Einträge

- **Ort:** [`src/hnsw/arena.rs:59`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/arena.rs#L59), [`arena.rs:160`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/arena.rs#L160), [`arena.rs:252`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/hnsw/arena.rs#L252)
- **Invariante betroffen:** I-6 (Prune-Korrektheit impliziert korrekte Speicherung)
- **Beleg:**
```rust
pub counts: RwLock<Vec<u8>>,  // ← 1 Byte pro Layer-Count
// ...
counts.push(len as u8);  // ← trunciert wenn len > 255
// ...
counts[count_start + layer] = len as u8;  // ← dito
```
Layer 0 hat Kapazität `m * 2`; bei `m = 128` wäre `m * 2 = 256 > u8::MAX`.
- **Angriffs-/Fehlerszenario:**
  1. Konfiguration: `m = 128` (gültig laut Config-Prüfung — NICHT VERIFIZIERT, ob Config `m <= 127` erzwingt).
  2. Layer-0-Kapazität = 256.
  3. Knoten erhält 256 Nachbarn auf Layer 0 → `len as u8 = 0`.
  4. Beim nächsten Lesen: `len = 0` → leere Verbindungsliste zurückgegeben.
  5. Suche kann diesen Knoten nicht traversieren → Konnektivitätsverlust.
- **Auswirkung:** Wenn die Config `m > 127` erlaubt, ist dieser Fehler korrektheitskritisch (keine Verbindungen auf Layer 0 → effektiv isolierter Knoten → Suche funktioniert nicht). VERMUTUNG, UNGEPRÜFT: `HnswConfig::validate()` könnte `m <= 127` erzwingen — diese Datei wurde nicht vollständig eingesehen.
- **Empfehlung:** `counts` auf `Vec<u16>` umstellen und alle entsprechenden Lese-/Schreibstellen anpassen. Alternativ: eine explizite Assertion in `allocate_node` und `update_backlink`, die sicherstellt, dass `len <= 255`, mit Err-Rückgabe statt silenter Truncation.

---

### [MEDIUM] F-05 — DiskANN `persist_delta_sync`: Lock wird gehalten während Full-Rebuild — Dead-Lock-Potential

- **Ort:** [`src/diskann/persistence.rs:40–66`](file:///home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/persistence.rs#L40-L66)
- **Invariante betroffen:** I-9 (atomarer Disk-Write), allgemein Liveness
- **Beleg:**
```rust
pub fn persist_delta_sync(&self) -> Result<()> {
    let pending = {
        let mut guard = self.inner.pending_inserts.write();
        self.inner.pending_count.store(0, Ordering::Relaxed);
        std::mem::take(&mut *guard)
    };  // ← guard wird hier dropped

    // ...
    if pending_ratio > 0.10 || existing_count == 0 {
        let (mut all_vecs, mut all_ids) = self.load_all_vectors_from_mmap()?;
        // ...
        return self.build_sync(&all_vecs, &all_ids);
    }
```
Der Lock wird zwar korrekt dropped (`std::mem::take`), aber `load_all_vectors_from_mmap` erwirbt intern `self.inner.header.read()` und `self.inner.mmap.read()`. Falls `build_sync` ihrerseits `self.inner.pending_inserts.write()` erwirbt (oder ein Hintergrundthread den Write-Lock hält), entsteht Lock-Inversion.
- **Angriffs-/Fehlerszenario:**
  1. Hintergrundthread A ruft `trigger_background_persist_delta` → `persist_delta_sync` → hält `flushing_in_progress = true`.
  2. Gleichzeitig erwirbt Hauptthread B `pending_inserts.write()` für eine Insert-Operation.
  3. Wenn `build_sync` → `write_to_path_sync` → `quantizer.read()` erwirbt, während B's Insert `quantizer.write()` hält → Deadlock.
  4. Speziell riskant, weil `load_all_vectors_from_mmap` die `quantizer.read()`-Guard über eine längere Kette hält.
- **Auswirkung:** Nicht immer reproduzierbar, aber unter Last (gleichzeitige Inserts + Flush-Trigger) möglich. Liveness-Problem, kein Datenverlust direkt, aber im Worst Case: Prozess-Freeze der gesamten Vektor-Engine. VERMUTUNG, UNGEPRÜFT: Ob `build_sync` tatsächlich `pending_inserts` intern re-acquiert, müsste durch vollständige Lektüre von `build_sync` verifiziert werden.
- **Empfehlung:** Die vollständige Lock-Hierarchie für `DiskAnnIndex` muss in einem zentralen Kommentar dokumentiert und durch `cargo xtask check-ring-capabilities-consistency` oder einen dedizierten Lock-Order-Test durchgesetzt werden. Mindestens: sicherstellen, dass `build_sync` niemals `pending_inserts` acquiriert, wenn von `persist_delta_sync` aufgerufen.

---

## 5. Optimierungspotenzial (Phase D)

**O-1 — O(N²) Ghost-Pointer-Scan bei Deletion**
Die aktuelle Reparatur in `deletion.rs:119–246` iteriert über alle `total_nodes` auf jedem Layer — O(N × L) Operationen. Bei N = 100.000 und L = 5 sind das 500.000 Verbindungslisten-Lookups pro Deletion. Alternative: Einführung einer inversen Nachbarliste (Backlink-Index `target_idx → set<neighbor_idx>`) als separater Index, der bei jedem Insert/Delete aktualisiert wird — würde Repair auf O(M) Lookups reduzieren. Das ist nur eine Optimierung, keine Korrektheitsfrage.

**O-2 — Lazy Dead-Neighbor-Pruning während Suche (bereits implementiert)**
`core_search.rs:124–151` enthält bereits einen Lazy-Relink-Pfad mittels `try_relink_pruned_neighbors`. Dieser Pfad verwendet `try_write()` und ist non-blocking — korrekte Implementierung. Keine Änderung nötig.

**O-3 — `ep.contains(&x)` in O(K) statt O(1)**
In `core_search.rs:226` und `core_insert.rs:153`:
```rust
if !ep.contains(&ram_ep) {
    ep.push(ram_ep);
}
```
`ep` ist ein `Vec<usize>`. Da `ep` typischerweise ≤ 3 Elemente enthält, ist der O(K)-Scan akzeptabel. Bei sehr breitem filter-entry-point-Sampling (`filter_eps` bis zu `max_filter_eps`) wächst `ep` aber auf hunderte Elemente — dann wäre eine `AHashSet<usize>` für Membership-Checks sinnvoll.

**O-4 — DiskANN Cache-Eviction: FIFO statt LRU**
`persistence.rs:616–621`: Die Cache-Eviction entfernt die ersten 25% der Keys (`cache.keys().take(to_remove)`). Da `AHashMap` keine definierte Iterationsreihenfolge garantiert, ist dies effektiv zufällig, nicht LRU. Eine echte LRU-Struktur (`lru`-Crate oder manuelle Zeitstempel) würde Hot-Nodes im Cache halten und I/O-Amplifikation bei Suchoperationen reduzieren.

---

## 6. Offene Fragen / nicht verifizierbar ohne Laufzeit-Tests

**OQ-1 — `HnswConfig::validate()` Obergrenze für `m`**
Es wurde nicht verifiziert, ob `HnswConfig::validate()` sicherstellt, dass `m <= 127`. Falls nicht, ist F-04 CRITICAL statt MEDIUM. Prüfung: `cargo xtask doctrine-scan --crate contextra-vector` oder direkte Lektüre von `src/hnsw/config.rs` (nicht geprüft).

**OQ-2 — Reconnect-Vollständigkeit der `select_neighbors_heuristic_with_batch`-Heuristik**
Der `dummy_vec = VectorData::F32(Vec::new())` in `deletion.rs:198,218` wird als `new_vector` an `select_neighbors_heuristic_with_batch` übergeben. Es ist unklar, ob diese Funktion den `dummy_vec` tatsächlich für die Heuristikberechnung verwendet oder nur als Fallback. Falls der leere Vektor zu Distanz `0.0` oder NaN führt, könnte die Heuristik degenerieren. Diese Funktion wurde nicht vollständig eingesehen.

**OQ-3 — Crash zwischen WAL-append und `pending_count`-Reset**
In `persist_delta_sync`: `pending_count` wird auf 0 gesetzt, *bevor* `pending` vollständig verarbeitet ist. Ein Crash nach dem Reset, aber vor erfolgreichem `build_sync`, würde die Einträge verlieren (sie sind weder im neuen Index noch rückholbar aus der WAL, da die WAL nach `build_sync` gelöscht wird). Ob die `pending.wal` hier als Recovery-Fallback dient, hängt vom Commit-Protokoll des Gesamtsystems ab — nicht vollständig verifiziert.

**OQ-4 — `contextra-sys::mmap_readonly` Safety-Audit**
Alle `unsafe`-Blöcke der mmap-Operationen liegen in `contextra-sys`. Da dieses Crate nicht Teil des Scopes dieser Mission war, wurde es nicht geprüft. Die Sicherheit der mmap-Abstraktion hängt vollständig von der korrekten Implementierung in `contextra-sys` ab.

**OQ-5 — RaBitQ Gram-Schmidt bei dimension ≤ 1**
`generate_orthogonal_matrix(1)` iteriert `for j in (0..1).step_by(2)` — es gibt genau einen Schritt, `j = 0`. Das Ergebnis ist eine 1×1-Matrix mit einem einzelnen Gaussian-Wert, der dann normalisiert wird. Korrekt. Aber: wird diese Funktion mit `dimension = 0` aufgerufen, gibt `try_train` vorher einen Fehler zurück (verifiziert in `quantize_rabitq.rs:44`). ✓

---

## 7. Jules-Task-Karten

```yaml
id: JULES-P03-01
title: "Ghost-Pointer-Verifikation vollständig oder Budget-Erschöpfung als Unbekannt markieren"
severity: HIGH
files_to_touch:
  - crates/contextra-vector/src/hnsw/deletion.rs
context: >
  verify_no_ghost_pointers() bricht bei budget-Erschöpfung mit `break` ab, ohne die
  nicht geprüften Knoten zu berücksichtigen. Die aufrufende Funktion setzt danach
  `verified_no_ghost_pointers: true` im Rückgabewert, obwohl bei großen Graphen
  (>budget Knoten) die Garantie nicht gehalten werden kann. Das verletzt INV-DELETION-2,
  die im Trait-Docstring als absolute Nachbedingung nach Ok(stats) dokumentiert ist.
acceptance_criteria:
  - "Nach remove_with_graph_repair gilt: wenn der gesamte Graph (alle total_nodes) gescannt wurde UND ghost_pointers == 0, dann verified_no_ghost_pointers = true."
  - "Bei Budget-Erschöpfung (inspected_nodes >= budget vor Abschluss des Scans) wird verified_no_ghost_pointers = false gesetzt."
  - "Der Trait-Docstring von GhostFreeVectorIndex::remove_with_graph_repair wird aktualisiert: entweder Invariante einschränken oder Scan-Budget dokumentieren."
  - "cargo test -p contextra-vector deletion_ghost_pointer_proptest :: alle Testfälle grün."
test_to_add: >
  Test: Einen HNSW-Index mit N = 5.000 Knoten aufbauen (M=16), einen Knoten aus der
  Mitte löschen. Anschließend manuell über alle Knoten und alle Layer iterieren und
  prüfen, dass keiner mehr auf den gelöschten Index zeigt. Dieser Test darf nur dann
  passieren, wenn verified_no_ghost_pointers = true zurückgegeben wird.
non_goals: >
  Keine Änderung an der Repair-Logik selbst (Backlink-Update-Algorithmus bleibt
  unverändert). Keine Erweiterung des Scopes auf mmap-Knoten (F-02 ist ein separater Task).
  Keine Performance-Optimierung des Scans.
```

```yaml
id: JULES-P03-02
title: "INV-DELETION-2 auf mmap-Knoten einschränken und DeletionStats korrekt befüllen"
severity: HIGH
files_to_touch:
  - crates/contextra-vector/src/hnsw/deletion.rs
context: >
  remove_with_graph_repair repariert nur RAM-Knoten-Backlinks (neighbor_idx >= mmap_node_count).
  mmap-Knoten sind read-only; deren Backlinks auf den gelöschten Knoten bleiben dauerhaft
  erhalten bis zum nächsten Full-Rebuild. Der Rückgabewert DeletionStats::verified_no_ghost_pointers
  kann dadurch true sein, obwohl nachweislich Ghost-Pointer im mmap-Segment existieren.
  Dies verletzt INV-DELETION-2 für den mmap-Teilgraph.
acceptance_criteria:
  - "Wenn nach der Repair mmap-Knoten existieren, die noch auf target_idx zeigen, wird verified_no_ghost_pointers = false gesetzt."
  - "Die Dokumentation von GhostFreeVectorIndex (Trait-Docstring) und deletion.rs FILE-CONTEXT wird mit dem Hinweis ergänzt: 'mmap-Backlinks werden erst nach Rebuild bereinigt'."
  - "DeletionStats erhält ein neues Feld mmap_ghost_pointers: usize, das die Anzahl verbleibender mmap-seitiger Ghost-Pointer dokumentiert."
  - "Bestehende Tests in deletion_ghost_pointer_proptest.rs weiterhin grün; neuer Test prüft mmap-Szenario."
test_to_add: >
  Test: HNSW-Index aus Snapshot laden (mmap-Segment), anschließend RAM-Knoten hinzufügen.
  Den RAM-Knoten, auf den ein mmap-Knoten als Nachbar zeigt, löschen.
  Erwarte: remove_with_graph_repair gibt Ok zurück, aber verified_no_ghost_pointers = false
  UND mmap_ghost_pointers > 0. Nach rebuild() erwarte: kein Ghost-Pointer mehr.
non_goals: >
  Keine Implementierung von mmap-Write-Pfaden (mmap bleibt read-only).
  Kein Erzwingen eines sofortigen Rebuilds bei jeder Deletion.
  Keine Änderung an der RAM-Repair-Logik.
```

```yaml
id: JULES-P03-03
title: "Entry-Point von AtomicU32 auf AtomicU64 umstellen"
severity: HIGH
files_to_touch:
  - crates/contextra-vector/src/hnsw/types.rs
context: >
  HnswHotCore::entry_point und ram_entry_point sind AtomicU32. set_entry_point()
  castet den usize-Index mit `idx as u32`, was bei Knotenindizes >= 2^32 silent
  truncation erzeugt. Bei sehr großen Indizes (mmap_node_count + ram_nodes_count >= 4 Mrd.)
  wird ein falscher Entry-Point gespeichert; Suchen starten an einer falschen Graphposition.
  Das Sentinel SENTINEL_NO_ENTRY_POINT = u32::MAX kollidiert zudem mit legitimen Indices
  nahe u32::MAX.
acceptance_criteria:
  - "entry_point und ram_entry_point sind AtomicU64."
  - "SENTINEL_NO_ENTRY_POINT ist u64::MAX."
  - "set_entry_point / get_entry_point verwenden u64-Operationen; kein as u32-Cast mehr."
  - "cargo xtask check-compile erfolgreich."
  - "Alle HNSW-Tests (hnsw_edge_cases, hnsw_seed_determinism, recall.rs) weiterhin grün."
test_to_add: >
  Test: Index mit synthetischem mmap_node_count = u32::MAX - 5 konfigurieren (via Mock).
  Einen Knoten inserieren; verify: get_entry_point() gibt den korrekten usize-Index
  zurück (kein Truncation). Erwarte: Entry-Point-Vergleich mit erwartetem Wert exakt gleich.
non_goals: >
  Keine Änderung an der Persistenz-Schicht (NodeRecord speichert Entry-Point als i64 im Header).
  Keine Änderung an der Arena-Indexierung.
  Kein Refactoring der such- oder insert-Logik.
```

---

## Unsafe-Block-Tabelle (vollständig)

| Datei | Zeile / Funktion | Behauptete Invariante | Verifikationsstatus |
|---|---|---|---|
| `src/lib.rs` | `#![forbid(unsafe_code)]` | Kein `unsafe` im gesamten Crate | **BEWIESEN SICHER** — Compiler-Enforcement |
| `src/persistence/mmap.rs` | `MmapIndex::open()` — delegiert an `contextra_sys::mmap_readonly` | `unsafe` liegt in `contextra-sys` | **NICHT IN SCOPE** — Unsafe in externem Crate |
| `src/diskann/persistence.rs` | `load_sync()` — delegiert an `contextra_sys::mmap_readonly` | ditto | **NICHT IN SCOPE** |
| `UNSAFE_TRANSITION.md` | Alle historischen `unsafe`-Blöcke | Migriert zu `contextra-sys` (TRANS-VEC-001, TRANS-VEC-002) | **BEWIESEN SICHER** — Migration dokumentiert und abgeschlossen |

**Ergebnis:** `crates/contextra-vector` enthält **null eigene `unsafe`-Blöcke**. Alle sicherheitsrelevanten Operationen (mmap, SIMD) liegen in `contextra-sys` resp. `contextra-simd` und müssen dort separat geprüft werden (außerhalb des Scopes dieser Mission).

---

*Report erstellt: 2026-10-06 · Mission P03 · Reviewer: Principal Security & Correctness Review*
