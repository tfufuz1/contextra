# Audit-Bericht: HNSW Slots, mmap-Overlay, Entry-Point

**Datum:** 2026-10-06
**Crate:** `contextra-vector`
**Task-Karte:** `.jules/tasks/T-2026-0214.toml`

---

## 1. Gelesene Dateien (Evidence)

Die folgenden Dateien wurden für diesen Analysebericht gelesen und auditiert:
- `.jules/PREAMBLE.md`
- `AGENTS.md`
- `crates/contextra-vector/AGENTS.md`
- `.jules/tasks/W1-05.toml`
- `.jules/tasks/T-2026-0214.toml`
- `crates/contextra-vector/src/hnsw/types.rs`
- `crates/contextra-vector/src/hnsw/arena.rs`
- `crates/contextra-vector/src/hnsw/core_search.rs`
- `crates/contextra-vector/src/hnsw/core_insert.rs`
- `crates/contextra-vector/src/hnsw/core_rebuild.rs`
- `crates/contextra-vector/src/hnsw/deletion.rs`
- `crates/contextra-vector/src/hnsw/vector_index_impl.rs`
- `crates/contextra-vector/src/persistence/mmap.rs`

---

## 2. Antworten auf Analyse-Fragen (Q1–Q3)

### Q1. mmap-Overlay

- **Status:** Belegt (Aussage stimmt vollständig).
- **Belegende Dateien & Zeilen:**
  - `crates/contextra-vector/src/persistence/mmap.rs:6` (Doc-Kommentar: `MmapIndex hält read-only FD`)
  - `crates/contextra-vector/src/persistence/mmap.rs:56` (`contextra_sys::mmap_readonly(&file)`)
  - `crates/contextra-vector/src/hnsw/types.rs:88-101` (`nodes: RwLock<Vec<HnswNode>>`, `arena: HnswArena`)
  - `crates/contextra-vector/src/hnsw/types.rs:136` (`mmap_index: RwLock<Option<MmapIndex>>`, `deleted_nodes: RwLock<RoaringTreemap>`)
  - `crates/contextra-vector/src/hnsw/types.rs:434` (`load_mmap_from_instance`)
  - `crates/contextra-vector/src/hnsw/types.rs:496-646` (`HnswIndex::save`: atomares Schreiben in `.hnsw.tmp` und Rename)
  - `crates/contextra-vector/src/hnsw/core_rebuild.rs:379-381` (Code-Kommentar: `Memory-mapped bytes (mmap_index) remain read-only and unchanged...`)
- **Begründung:**
  1. *RAM-Exklusivität des Overlays:* Der via `MmapIndex::open` geladene mmap-Speicher ist strikt schreibgeschützt (`mmap_readonly`). Alle Laufzeitmutationen (Einfügen neuer Vektoren, Aktualisieren von Nachbarlisten/Backlinks, Anlegen von Tombstones in `deleted_nodes`) geschehen ausschließlich im volatilen Arbeitsspeicher in `hot.nodes`, `hot.arena` und `cold.deleted_nodes`. Es gibt kein persistentes In-Place-Overlay für mmap-Dateien auf der Festplatte.
  2. *Neustart-Verhalten:* Beim Prozessneustart gehen ungespeicherte RAM-Strukturen (`nodes`, `arena`, `deleted_nodes`) vollständig verloren. Nur wenn explizit `HnswIndex::save()` aufgerufen wird, schreibt die Engine ein neues vollständiges HNSW-File via temporärer Datei (`.tmp`) und benennt diese atomar um (`atomic_replace`).
  3. *Korrektheit des Code-Kommentars:* Der Kommentar in `core_rebuild.rs:379-381` ist **korrekt**. Da der mmap-Speicher read-only geöffnet ist, bleiben die mmap-Bytes unverändert; das Nullen von RAM-Vektorslots im Prozessspeicher modifiziert niemals die mmap-Bytes auf dem Datenträger.

---

### Q2. Slot-Wiederverwendung

- **Status:** Belegt (Aussage stimmt: Freigabe und Wiederbelegung bei existierenden In-Edges ist möglich).
- **Belegende Dateien & Zeilen:**
  - `crates/contextra-vector/src/hnsw/arena.rs:136-150` (`allocate_node`: Reaktivierung von Indizes aus `free_list`)
  - `crates/contextra-vector/src/hnsw/arena.rs:356-361` (`free_node`: Einfügen freigegebener `ram_idx` in `free_list`)
  - `crates/contextra-vector/src/hnsw/core_rebuild.rs:383-410` (`do_delete_node_idx`: Eintrag in `deleted_nodes`, RAM-Zeroing, Aufruf `free_node`)
  - `crates/contextra-vector/src/hnsw/core_insert.rs:431-438` (`apply_insert`: Überschreiben von `nodes[ram_idx]` und Austragen aus `deleted_nodes`)
  - `crates/contextra-vector/src/hnsw/core_search.rs:69-80` (`search_layer_with_context`: Traversal liest Nachbarkanten aus `resolve_connections`)
  - `crates/contextra-vector/src/hnsw/deletion.rs:49-240` (`remove_with_graph_repair`: Synchrone Kantenbereinigung nur bei dieser Methode)
- **Begründung:**
  1. *Wiederbelegung trotz In-Edges:* Standard-Löschungen über `VectorIndex::delete` -> `commit` rufen `do_delete_node_idx` auf. Dabei wird der Knoten in `deleted_nodes` markiert und sein RAM-Slot über `free_node` in die `free_list` eingetragen. Es findet jedoch **keine** Kanten- und Backlink-Bereinigung in den Nachbarlisten anderer aktiver Knoten statt (eine synchrone Bereinigung erfolgt nur bei `remove_with_graph_repair`).
  2. *Mechanismus bei der Wiederbelegung:* Wird anschließend ein neuer Knoten eingefügt, reaktivert `allocate_node` den freigegebenen Slot `ram_idx` aus der `free_list`. In `apply_insert` wird der Slot mit den neuen Dokumentdaten belegt und der Index `global_idx = mmap_count + ram_idx` aus `deleted_nodes` entfernt.
  3. *Auswirkung auf die Suche (Routing vs. Ghost-Ergebnisse):* Nach Austragen aus `deleted_nodes` filtern die Suchiterationen Kanten zu `global_idx` nicht mehr heraus. Knoten, deren In-Edges auf `global_idx` nicht bereinigt wurden, führen die Graphnavigierung nun zu dem neuen Dokument `B`. Es handelt sich dabei zwar nicht um ein gelöschtes Geisterdokument (da `DocId B` aktiv ist), aber um eine **topologisch invalide Kante** im HNSW-Graphen. Traversalpfade nutzen veraltete Nachbarschaftsbeziehungen, was die Suchqualität verschlechtert und Fehlrouting verursacht.

---

### Q3. Entry-Point

- **Status:** Belegt (Aussage stimmt vollständig).
- **Belegende Dateien & Zeilen:**
  - `crates/contextra-vector/src/hnsw/core_rebuild.rs:412-482` (`do_delete_node_idx`: Neuwahl-Logik für `entry_point` und `ram_entry_point`)
  - `crates/contextra-vector/src/hnsw/vector_index_impl.rs:271` (`commit`: Sperren von `write_mutex`)
  - `crates/contextra-vector/src/hnsw/deletion.rs:49` (`remove_with_graph_repair`: Sperren von `write_mutex`)
  - `crates/contextra-vector/src/hnsw/types.rs:88-101` (`write_mutex: Mutex<()>` im `HnswHotCore`)
- **Begründung:**
  1. *Wahl des neuen Entry-Points:* Wenn der gelöschte Knoten `idx` dem aktuellen `entry_point` oder `ram_entry_point` entspricht, wird eine Neuwahl ausgelöst. Der Algorithmus iteriert über alle verbleibenden Knoten (mmap: `0..mmap_node_count` und RAM: `0..nodes.len()`), schließt gelöschte Knoten aus und ermittelt den Knoten mit der **höchsten Layer-Nummer** (`max_layer`). Dieser Knoten wird als neuer `entry_point` gesetzt und `max_layer` entsprechend angepasst.
  2. *Ausführung unter dem globalen Mutex:* Die Neuwahl findet innerhalb von `do_delete_node_idx` statt. Dieser Pfad wird ausschließlich während `VectorIndex::commit` oder `GhostFreeVectorIndex::remove_with_graph_repair` aufgerufen, welche beide zu Beginn die exklusive `write_mutex` sperren.
  3. *O-Abschätzung der Haltezeit:* Der Suchlauf zur Neuwahl iteriert in zwei Schleifen sequentiell über alle mmap-Knoten ($N_{\text{mmap}}$) und alle RAM-Knoten ($N_{\text{RAM}}$). Die Zeitkomplexität beträgt **$O(N)$** mit $N = N_{\text{mmap}} + N_{\text{RAM}}$. Bei großen Indizes (z. B. $N = 10^6$) führt dieser lineare Scan unter gehaltener exklusiver `write_mutex` zu einer spürbaren Blockade aller konkurrierenden Schreib- und Commit-Operationen.

---

## 3. Abgrenzung zu W1-05

Aus der Task-Karte `.jules/tasks/W1-05.toml` geht hervor, welche spezifischen Punkte das Paket W1-05 adressiert und welche explizit ausgeschlossen sind:

### Was W1-05 ändert
- **Op-Normalisierung in `commit()`:** Zusammenfassung von `insert(A)` und `delete(A)` im selben Transaction-Buffer zu einem Netto-Delete (`vector_index_impl.rs`).
- **Fehleratomarität:** Re-Staging von Operationen im Fehlerfall vor der ersten Mutation zur Verhinderung von Ops-Verlust.
- **RAM-Vektor-Löschung:** Nullen von Vektordaten in Prozess-RAM-Slots (`node.vector.fill(0)`), sobald ein RAM-Slot freigegeben wird (`do_delete_node_idx` in `core_rebuild.rs`).

### Was W1-05 NICHT ändert
- **Kein Overlay (W1-04 Scope):** Keine Einführung von persistenten On-Disk-Overlays oder Modifikation von mmap-Bytes.
- **Keine Änderung an `deletion.rs`, `types.rs`, `batch.rs`, `core_insert.rs`, `arena.rs`, `mod.rs`:** Slot-Reuse-Logik in `HnswArena` und fehlende Kantenbereinigung bei Standard-Löschungen bleiben unverändert.
- **Keine Änderung der Entry-Point-Neuwahl:** Der $O(N)$ lineare Scan zur Neuwahl des Entry-Points unter `write_mutex` bleibt bestehen.

---

*Bericht Ende.*
