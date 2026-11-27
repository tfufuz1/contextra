# Implementierungsspezifikation `contextra-vector` (Löschkette – HNSW als `ErasureLayer`)

*Ring 0 · Konzept-WP 7 · Abhängig von: 02, 03, 10 · Index: `00-INDEX.md` (K-11, K-27, K-37)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

`HnswPurger` (synchron, hält `write_mutex` über die gesamte Laufzeit, verwirft Fehler) wird durch `HnswErasure` ersetzt. G1: sofortige Maskierung. G3: **RAM-Entfernung der Knoten samt Vektorbytes**; optionaler Datei-Rewrite, falls eine HNSW-Datei existiert. Embeddings liegen zusätzlich im LSM (Dokumentkörper) und werden vom Store entfernt.

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `contextra-types`, `-ports`, `-mvcc`, `-sys` optional, `parking_lot`, `tracing`. Keine Kante auf `store` |
| Purger | `hnsw/purge.rs`: `HnswPurger::purge(&self, ctx, target: &ScanTarget)` (`:38`), `write_mutex.lock()` (`:42`), `let _ = arena.update_backlink(…)` (`:133`). Rückgabe `PurgeStats` |
| Maskierung | `HnswIndex.inner.cold.deleted_nodes: RwLock<RoaringTreemap>` (`types.rs:206`), gelesen in Suche (`types.rs:506,551`). Adressiert **Knoten-IDs**, nicht `DocId` (K-27) |
| Entfernen | `hnsw/deletion.rs`: `remove_with_repair(&mut self, id: &u64) -> Result<bool>` (`:311`), `remove(&self, doc_id: DocId) -> Result<bool>` (`:405`) |
| Persistenz | `save_internal` (`types.rs:750`): `std::fs::File::create(&temp_path)` (`:761`), `save`/`save_sync` (`:953,958`), `load_mmap`/`load_mmap_sync` (`:645,650`). Nutzer: **kein Engine-Aufrufer** (K-37). K-11: Fehler beim Verzeichnis-Öffnen werden verschluckt |
| Rebuild | Produktion baut HNSW aus dem LSM neu auf (`Collection::load_index`, `engine/src/collection/mod.rs:585`) |
| Scanner | `hnsw/scanner.rs` (`LayerScanner` für `WitnessLayer::HnswIndex`) ✔ |
| Teilrebuild | `partial_rebuild.rs` (`should_trigger_partial_rebuild`, `PartialRebuildConfig`) |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `src/hnsw/erasure.rs` | **NEU**: `HnswErasure`, `DocNodeMap`-Zugriff, Cursor |
| `src/hnsw/purge.rs` | Alt-Purger `#[deprecated]`, nach WP-7.4 gelöscht; `let _ = update_backlink` entfällt ersatzlos mit der Datei |
| `src/hnsw/types.rs` | (a) `erased_docs: RwLock<Arc<BTreeSet<DocId>>>` im `cold`-Teil, (b) Suche filtert (§5), (c) `save_internal`/`save` auf `Vfs` (§6), (d) `pub(crate) fn remove_nodes_batch` |
| `src/hnsw/arena.rs` | `zeroize_slot(node)` (Vektor-, Quantisierungs- und Nachbarbytes überschreiben, §4.3) |
| `src/hnsw/scanner.rs` | Nadeln aus `ResolvedScope` (§8) |
| `src/lib.rs` | `pub use hnsw::erasure::HnswErasure` |

## 4. Layer-Verhalten

### 4.1 Zuordnung Dokument → Knoten

Der Layer braucht `DocId → node_id`. Der Index hält die Zuordnung bereits (Einfügepfad referenziert Dokument-IDs in Knoten, `deletion.rs:405 remove(doc_id)`). **[?]** Struktur und Lock im Quelltext lesen (`hnsw/types.rs`, Suche `doc_id`/`id_map`). Fehlt eine O(1)-Zuordnung, wird `DocNodeMap` (`BTreeMap<DocId, SmallNodeList>` bzw. `ahash`) angelegt und beim Insert/Remove gepflegt. Eine Kollision (zwei Dokumente, ein Knoten) ist nicht zulässig.

### 4.2 `on_gate(scope)` (G1, < 10 ms)

1. `erased_docs` per Copy-on-Write erweitern (Zeiger-Swap).
2. **Nicht** in `deleted_nodes` eintragen (K-27: Dokumentsicht ist zusätzlich; Knoten werden erst in `purge` markiert).
3. Idempotent; wiederholter Aufruf mit gleichem Scope ändert nichts.

### 4.3 `purge(scope, cx)` (G3, wiederaufnehmbar)

Einheit = **ein Knoten**. Arbeitsliste = `DocNodeMap`-Auflösung der Dokumentsicht (sortiert nach `node_id`), Cursor = `[ver u8][work_hash 8][done u64 LE]` (wie 06 §5.3).

Pro Batch (`cx.max_units`, Default 128 Knoten):
1. Kurzer `write_mutex`-Abschnitt **nur** für den Zeiger-/Strukturwechsel; Berechnungen und Datei-I/O laufen **außerhalb** (anders als heute `purge.rs:42`).
2. `remove_nodes_batch(batch)`: je Knoten Nachbarn per Reverse-Adjazenz neu verbinden (Aufwand O(Grad), nicht O(Layer × N)); **jedes** `update_backlink`-Ergebnis wird mit `?` propagiert (B-09: `let _ =` entfällt). Scheitert der Umbau eines Nachbarn, wird der Batch zurückgerollt (Strukturänderungen erst nach vollständigem Erfolg des Batch auf eine Kopie angewandt, dann Swap).
3. `arena.zeroize_slot(node)` für **jeden** entfernten Knoten: Vektorbytes (f32/SQ8), Nachbarlisten und ggf. mmap-Abbildung (Knoten in mmap-Segmenten werden nie nur markiert, sondern Segment-Rewrite §6). Nach dem Zeroisieren Knoten in `deleted_nodes` (Wiederverwendungsschutz).
4. Entry-Point-Fix: war der Entry Point betroffen, neuen wählen (höchste Ebene unter den Verbleibenden). Leerer Index ⇒ Entry Point `None`.
5. Budget: `cx.budget.acquire(IoClass::Erase, bytes)` mit `bytes` = Summe Knotengröße; nur bei Datei-Rewrite relevant (RAM-Entfernung zählt 0 I/O-Bytes, aber `max_units` begrenzt CPU).
6. Fortschritt: nach jedem Batch `cx.progress.record(WitnessLayer::hnsw(), LayerProgress{cursor, units_done, bytes_freed})`. `units_committed` = Knoten, die **vollständig** entfernt und bestätigt sind.

### 4.4 Recall-Guard

Vor dem ersten Batch: Stichprobe `n = min(64, Restknoten)` (Ziel-Vektoren aus dem Rest, Suche k=10, Recall gegen exakte Nachbarn aus der Stichprobe via `distance`). Nach dem letzten Batch erneut. `recall_before - recall_after > cfg.max_recall_drop` (Default 0.05) ⇒ `Partial-Rebuild` einplanen: `HnswErasure` signalisiert über `LayerError{ kind: Retryable, detail: "recall_guard" }` **nicht**; stattdessen setzt es `needs_partial_rebuild` und der Engine-Worker ruft `partial_rebuild` (existiert) mit dem Gate-Filter. Die Löschung ist davon unabhängig abgeschlossen.

## 5. Suchpfad (G1)

`core_search.rs`/`acorn_filtered.rs`: Kandidaten mit `node ∈ deleted_nodes` **oder** `doc ∈ erased_docs` werden verworfen und **nicht** als Ergebnis gezählt. Über-Fetch: `ef_search` und `k` werden um `masked_ratio` erhöht (`ceil(k / (1 - ratio))`, höchstens 4×), damit Top-k nach dem Maskieren gefüllt bleibt. Ohne aktive Maskierung ändert sich nichts (leere Menge, ein Load). `gate.check_doc` wird bei der Hydration durch die Engine erneut geprüft (10 §7); die Vektorschicht prüft nur ihre eigene RAM-Menge.

## 6. Datei-I/O über `Vfs` (K-11)

Gilt, **wenn** `save`/`load_mmap` genutzt werden (heute kein Engine-Aufrufer, K-37).

1. `save_internal(&self, vfs: &dyn Vfs, path)`: Temp-Datei `*.hnsw.tmp` über `vfs.open(write_create)`, Schreiben, `sync_all`, `vfs.rename(tmp, final)` (**`rename` hat Default `Unsupported`**, K-17: der Aufrufer muss ein `Vfs` mit `rename` übergeben; sonst `Err(Unsupported)`, kein Fallback auf `std::fs`), `vfs.sync_dir(parent)`. Fehler beim Öffnen/Synchronisieren des Verzeichnisses werden **propagiert**.
2. Rewrite beim Erase: neues Segment ohne Scope-Knoten schreiben (Knoten-IDs dicht neu vergeben **nicht** nötig; Löcher als Tombstone-Kennzeichen), `rename`, alte Mappings nach Reader-Drain freigeben (`Arc<Mmap>`-Refcount = 1), alte Datei an **Pending-Delete** melden. Die Meldung läuft über einen neuen Port-Hook `PurgeCx.progress.register_obsolete_file(path)` **[?]** (nicht in 02: Alternative: Rückgabe `PurgeOutcome.obsolete_files: Vec<PathBuf>`, vom Treiber an `ErasureKernel` weitergereicht; **Entscheidung O-V2**).
3. `load_mmap` liest nach Erase nur die neue Datei; ein Neustart zwischen `rename` und Entfernen der alten Datei findet beide, die alte steht auf der Pending-Delete-Liste.
4. Existiert **keine** HNSW-Datei (Normalfall), ist dieser Abschnitt ein No-op; G3 = RAM-Bereinigung + Store-Purge der Embeddings (K-37).

## 7. Lock-Disziplin

`write_mutex` wird höchstens für den Strukturwechsel eines Batch (< 1 ms) gehalten, **nie** über `.await`, nie über Datei-I/O. Reihenfolge `erased_docs`-Swap vor `write_mutex`. Kein Lock-Rang aus dem Store (Crate kennt ihn nicht); Doku-Kommentar nennt die lokale Ordnung `write_mutex < deleted_nodes`.

## 8. Witness

`hnsw/scanner.rs`: `remaining` = (1) Knoten, deren `doc ∈ Scope` im RAM sind (Strukturprüfung), (2) Treffer der Nadeln (Vektorbytes des Scope-Dokuments, **nur** wenn der Treiber die Original-Embeddingbytes in die Nadelmenge gibt; sie stammen aus dem `PrivilegedRead` vor dem Store-Purge und bleiben nur im RAM) in Arena und Datei(en). Positivkontrolle: Kanarien-Vektor einfügen, scannen, entfernen.

## 9. Tests

| ID | Inhalt |
| --- | --- |
| TST-VEC-ERA-1…7 | Konformanz C1–C7 |
| TST-VEC-ERA-8 | Nach `on_gate` liefert `search` den Knoten nie; Top-k bleibt voll (Über-Fetch), Property-Test über 500 Zufallsindizes |
| TST-VEC-ERA-9 | Nach `purge`: `arena` enthält die Vektorbytes nicht mehr (Byte-Scan des Arena-Speichers) |
| TST-VEC-ERA-10 | Fehlerinjektion in `update_backlink` ⇒ `Err`, Struktur unverändert (Rollback des Batch) |
| TST-VEC-ERA-11 | Recall-Guard: Erase von 30 % Knoten, Recall-Abfall ≤ Grenze bei Standardparametern oder Rebuild-Flag gesetzt |
| TST-VEC-ERA-12 | `save`/`load` über `FaultVfs`: Crash nach `rename` vor Entfernen ⇒ nach Neustart nur neue Datei gelesen, alte auf Liste (13 §4) |
| TST-VEC-ERA-13 | `rg "std::fs|tokio::fs" crates/contextra-vector/src/hnsw/{erasure,types}.rs` leer (G-4 erweitert) |
| TST-VEC-ERA-14 | Wiederaufnahme an jedem Cursor; Nachbardokumente bit-identisch erreichbar (Scope-Isolation C3) |

## 10. Arbeitspakete

| WP | Inhalt |
| --- | --- |
| 7.1 | `erased_docs`, Suchfilter, `on_gate`, Konformanz-Skelett |
| 7.2 | `purge` mit `DocNodeMap`, Batch, Zeroize, Entry-Point, Recall-Guard |
| 7.3 | `Vfs`-Persistenz, Segment-Rewrite, Obsolete-Files-Pfad |
| 7.4 | Alt-Purger entfernen, Scanner-Nadeln |

## 11. Definition of Done

Konformanz grün; keine `let _ =`/`std::fs` im Erase-Pfad (G-2, G-4); Arena-Byte-Test grün; Suche ohne aktive Maskierung unverändert schnell (Bench `benchmarks/contextra-bench` ±2 %).

## 12. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-V1 | `DocId → node_id` bereits vorhanden? | Lesen; sonst `DocNodeMap` |
| O-V2 | Obsolete-Datei-Meldung: `ProgressSink`-Methode oder `PurgeOutcome`-Feld | `PurgeOutcome.obsolete_files` (kein Port-Umbau); 02 um das Feld ergänzen (kleiner Nachtrag A-4) |
| O-V3 | `WitnessLayer`-Konstante für HNSW | `WitnessLayer::HnswIndex` existiert ✔ (`hnsw()` als Kurzform im Layer) |
| O-V4 | DiskANN-Index (`experimental-diskann`) | Außerhalb Stufe 1; Feature-Gate liefert `FeatureDisabled` im Witness |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `src/hnsw/erasure.rs` (98 LOC): `HnswErasure` mit `HnswScanner`, Dokumentmenge über `scoped_docs` ✔. **Offen:** nicht im Builder registriert (K-48); **kein** Konformanztest im Crate; Zeile 12 der Datenort-Matrix verlangt zusätzlich: (a) Prüfung, dass das Quantisierungs-Codebook keine Einzelvektor-Information trägt (`TST-VEC-CB-1`, **[?]**), (b) Datei-`save`/`load_mmap` nur über `StoreIo` + Temp/`rename`/`sync_dir` (CP-28), (c) Zeroisierung gelöschter Vektorspeicher im RAM, soweit der Allokator das zulässt, sonst dauerhafte Garantiegrenze „Prozessspeicher“ (Dokumentation und Ledger-Scope-Text, **kein** neuer persistierter Ausnahme-Wert).

**Verbindliche Ergänzungen.** C8 Ergebnisfilter (Suche prüft jede Kandidatenliste gegen `ReadGate`, auch vor `on_gate`); Idempotenz nach Cursor-Reset; `purge` puffert teuren Graph-Repair-Fortschritt intern, damit ein Layer-Neustart (14 §3.4) nicht jedes Mal komplett neu rechnet; `Concurrency`: Suche darf gegen `purge` laufen, Test im Loom-Modell (13).
