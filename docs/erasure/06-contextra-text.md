# Implementierungsspezifikation `contextra-text` (Löschkette – Textindex als `ErasureLayer`)

*Ring 0 · Konzept-WP 6 · Abhängig von: 02, 03 (A-1), 10 · Index: `00-INDEX.md` (K-26, K-30, K-35, K-39, A-1, A-3)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

`TextPurger` (synchron, `futures::executor::block_on`, committet selbst) wird durch `TextErasure` ersetzt: ein asynchroner, wiederaufnehmbarer `ErasureLayer` für `WitnessLayer::text()` (A-3, `LAYER_TEXT = "TextIndex"`). Er entfernt Postings, korrigiert die BM25-Statistik, räumt RAM-Strukturen und liefert den Kandidatenfilter für die Suche.

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `contextra-types`, `contextra-ports` **optional** (`types-compat`, Default an), `parking_lot`, `tracing`, `futures = "0.3"` (`Cargo.toml`). Kante auf `store` fehlt und darf nicht entstehen (K-36) |
| Purger | `inverted/purge.rs`: `block_on_async` (`:30-34`) an `:126,158,172,216,246,261,303,323,350,395,401,427,446,463`; `purge_document_report` entfernt `dl:` (`:122`), `fw:` (`:167`), Postings, `plb:`-Listen (`:261-303`), schreibt `meta:stats` (`:392`) und committet selbst (`:401`). `purge_collection_report` nutzt Präfix `__txt:{ns}:` (`:418-422`) |
| Schlüssel | Präfix `__txt:default:` bzw. `__txt:{namespace}:` (`index_struct.rs:67-69`). Unterschlüssel: `pl:{term}:` (`key_term_prefix`), `plb:{term}` (`key_batch_posting_list`, gebündelte Liste), `dl:{id}`, `fw:{id}` (`key_with_id`), `meta:stats` (`self.key("meta:stats")`). Postings-Bündel liegen **nicht** im dokumentgebundenen Schlüssel-Scope (K-30) |
| RAM | `resident_index: ResidentPostingIndex` (`ensure_term_loaded`, `index_struct.rs:~165-200`), Statistik atomar (`load_stats`, `:121`), Morphologie-/Trie-Caches |
| Scanner | `inverted/scanner.rs` implementiert `LayerScanner` für `WitnessLayer::TextIndex`, scannt LSM-Schlüssel **und** RAM, Positivkontrolle per Kanarien-Dokument |
| Port | `StorageEngine` (`ports/src/storage.rs`): `get`, `put`, `delete`, `delete_prefix`, `commit(tx) -> Result<()>` (**ohne Sequenz**), `last_seq_no()`, `begin` nicht über Port (TxIds kommen vom Aufrufer) |
| Ergebnis K-26 | Der Purger committet selbst und arbeitet über `StorageEngine`; er muss über `PrivilegedRead` lesen, weil das Gate den Lesezugriff auf `fw:` sperrt |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `src/erasure.rs` | **NEU** (hinter `types-compat`): `TextErasure<S>`, `TextTxSource`, `TextErasureConfig` |
| `src/inverted/purge.rs` | `block_on_async` und alle Aufrufe **löschen**. Der Typ `TextPurger` und `text_purger()` werden `#[deprecated]` und entfallen nach WP-6.4 (Index: `Purger` ist deprecated). Die reinen Rechenteile (Term-Entfernen aus `PostingList`, Statistik-Neuberechnung) wandern als `pub(crate)`-Funktionen nach `erasure.rs` |
| `src/inverted/index_struct.rs` | `pub(crate)` Zugriffe ergänzen: `drop_resident_terms(&[String])`, `reset_trie_caches()`, `recompute_stats_after(removed: &RemovedDocStats)`; Kandidatenfilter (§7) |
| `src/inverted/scanner.rs` | `NeedleScan` auf Scope-Nadeln ausrichten (§8); `ScanTarget` nur noch für den Altpfad |
| `src/lib.rs` | `pub use erasure::{TextErasure, TextErasureConfig, TextTxSource}` |
| `tests/text_erasure_conformance.rs` | **NEU** (13 §5: Konformanz C1–C7) |

## 4. Typen

```rust
/// Liefert TxIds und Commit; Implementierung in der Engine (kennt TxIdAllocator, Group-Commit).
pub trait TextTxSource: Send + Sync {
    fn begin<'a>(&'a self) -> BoxFuture<'a, Result<TxId, ContextraError>>;
    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<(), ContextraError>>;
}

pub struct TextErasureConfig {
    pub terms_per_tx: usize,          // Default 256, Obergrenze je Commit
    pub fallback_dictionary_scan: bool, // Default true
}

pub struct TextErasure<S: StorageEngine> {
    index: Arc<InvertedIndex<S>>,     // je Collection-Namespace; Engine registriert eine Instanz je Collection
    tx: Arc<dyn TextTxSource>,
    cfg: TextErasureConfig,
    gate: Arc<dyn ReadGate>,          // Kandidatenfilter (§7)
}
impl<S: StorageEngine + 'static> ErasureLayer for TextErasure<S> { /* §5 */ }
```

Ein `TextErasure` je `InvertedIndex` (je Collection). Der Treiber registriert pro betroffener Collection eine Instanz; `expected_layers` enthält `WitnessLayer::text()` **einmal** (10 §4). Nach Layer-Vertrag liefert eine registrierte, aber nicht betroffene Collection `remaining = 0`.

## 5. Layer-Verhalten (normativ)

### 5.1 `on_gate(scope)` (RAM, idempotent, < 10 ms, keine I/O)

1. Dokumentsicht in eine `BTreeSet<DocId>` je Collection übernehmen (`DocScope::{Set,Collection,Tenant}`); `erased_docs: RwLock<Arc<BTreeSet<DocId>>>` per Copy-on-Write ersetzen (Zeiger-Swap < 1 µs).
2. `resident_index`: Posting-Einträge der Dokumente **nicht** hier entfernen (Terme noch unbekannt ohne I/O); stattdessen Markierung `erased_docs` wird von `ResidentPostingIndex`-Lesern gefiltert (§7).
3. Morphologie-/Trie-Caches: `reset_trie_caches()` nur wenn Terme des Scopes eindeutig im Cache stehen können; sonst unverändert (Caches enthalten Terme, keine Dokumentbezüge **[?]** prüfen `morphology/trie.rs`).

### 5.2 `purge(scope, cx)`

Reihenfolge in einem Aufruf (ein Tick, `cx.max_units` Einheiten = Terme):

1. **Arbeitsliste bestimmen** (nur beim ersten Aufruf bzw. wenn `cx.cursor` leer):
   - Für jedes `DocId` der Dokumentsicht dieser Collection: `cx.reader.get(fw:{doc})` (privilegiert, K-26). Decodieren ⇒ Termmenge. Vereinigung als sortierte `BTreeSet<String>` (`terms`).
   - Fehlt `fw:` (Wiederholung nach Store-Purge, siehe §5.4): wenn `fallback_dictionary_scan`, Arbeitsliste = **alle** `plb:`-Terme in Seiten à 1 024 über `cx.reader.scan_prefix(index.key("plb:"), after, 1024)`.
2. **Je Term ab Cursor** (aufsteigend): `plb:{term}` über `cx.reader.get` lesen, `PostingList::decode_compact`, alle `DocId` ∈ Scope entfernen. Ist die Liste leer ⇒ `delete(plb:{term})`, sonst `put(plb:{term}, encode_compact)`. Legacy-Schlüssel `pl:{term}:*` (Einzel-Postings): `scan_prefix(key_term_prefix(term))` und je Treffer `delete`. Jeder Zugriff geht durch `cx.reader` (Lesen) und `index.storage()` (Schreiben).
3. **Commit je `terms_per_tx` Terme**: `tx.begin()`, Writes, `index.storage().commit(tx)`. Fehler ⇒ `rollback`, `Err(LayerError{kind: Retryable|Permanent})`; **kein** Teilerfolg als `Ok`.
4. **Collapse registrieren** (A-1): Nach erfolgreichem Commit `s = storage.last_seq_no().await?` (Obergrenze der Commit-Sequenz, da `commit` keine Sequenz liefert), dann `cx.progress.register_collapse(&[(plb_key, Seq(s)), …])`. Der Kernel lässt ältere Versionen der `plb:`-Schlüssel in der Erase-Compaction fallen (03 §7.2 `CollapseVersions`), weil sie die Dokument-Term-Zuordnung noch enthalten. Fehler ⇒ `Err` (sonst Rest bleibt unbemerkt).
5. **Statistik** (nach der letzten Termseite): `dl:`-Längen der Scope-Dokumente über `cx.reader.get(dl:{doc})` lesen (Summe `removed_len`, Anzahl `removed_docs`), `meta:stats` (N, Summe der Längen ⇒ avgdl) neu berechnen und per `put` + `commit` schreiben; atomar mit den RAM-Zählern (`recompute_stats_after`). Existiert `dl:` nicht mehr (Wiederholung): Statistik nicht erneut verändern (Idempotenz über Markierung `stats_applied` im `LayerProgress.cursor`, §5.3).
6. **RAM**: `drop_resident_terms(terms)` für alle bearbeiteten Terme; `resident_index` lädt sie bei Bedarf neu aus dem LSM.
7. **`dl:`/`fw:` werden von der Schicht nicht gelöscht.** Sie liegen im Schlüssel-Scope (exakte Schlüssel) und fallen durch Memtable-`retain` und Erase-Compaction (03 §7). Ein Delete der Schicht auf gesperrte Schlüssel würde vom Commit-Gate mit `ContextraError::Erased` abgewiesen (K-1). Das ist der Grund für die Reihenfolge Text **vor** Store.
8. `PurgeOutcome{ progress, units_committed = Anzahl committeter Terme, bytes_freed = Summe verkürzter Listenbytes }`. `cursor == None` erst, wenn Arbeitsliste, Statistik und RAM-Schritt abgeschlossen sind.

### 5.3 Cursor-Format (≤ 256 Byte, `Cursor::new` prüft)

```
[ver u8 = 1][phase u8][work_hash: 8 Byte BLAKE3-Präfix der Arbeitsliste][done_terms u64 LE][stats_applied u8]   (= 19 Byte, keine Termlängenbegrenzung)
```
`phase`: 0 = Terme, 1 = Statistik, 2 = RAM. `work_hash` verhindert, dass nach Wiederaufnahme eine geänderte Arbeitsliste ab falscher Stelle weiterläuft; bei Abweichung beginnt die Schicht neu (idempotent). Die Arbeitsliste ist sortiert und deterministisch; `done_terms` indiziert sie. Der Cursor enthält keine Terme, daher entfällt jede Termlängenbegrenzung.

### 5.4 Wiederaufnahme und Verifikationsschleife

`Verifying` kann nach `Purging` zurückspringen (Index §4: Zustandsautomat, `attempt++`). Dann existiert `fw:` evtl. nicht mehr (Store hat bereits gepurged). Deshalb §5.2 Schritt 1 Fallback. Der Fallback ist O(Dictionary) und läuft nur bei Restmenge > 0.

### 5.5 Fehlerklassifikation

| Quelle | `LayerErrorKind` |
| --- | --- |
| `StorageEngine`-Fehler `Io`, Lock-Timeout | `Retryable` |
| `decode_compact` schlägt fehl (Korruption) | `Permanent` (Detail `"posting_corrupt"`), Auftrag → `Quarantined`, kein Überspringen |
| Schlüssel außerhalb `ResolvedScope` im Reader (`ScopeViolation`) | `ScopeViolation` |
| `cx.cancel.is_cancelled()` | `Cancelled`; Zustand bleibt durable über den letzten committeten Cursor |

`unwrap_or_default()`, `let _ =`, `.ok();` sind im Pfad verboten (G-2).

## 6. CPU und Abbruch

Dekodieren/Kodieren der Posting-Listen läuft in `tokio::task::spawn_blocking` (je Termseite, Abbruchprüfung vor und nach). Kein `block_on` (G-1). Budget: je Commit `cx.budget.acquire(IoClass::Erase, bytes)` mit `bytes` = Summe der gelesenen + geschriebenen Listenbytes.

## 7. Kandidatenfilter (Suchpfad, G1)

`search_bm25`/`search_bm25_at` und die WAND-/Stream-Pfade (`wand.rs`, `stream.rs`) filtern Kandidaten gegen `erased_docs` (RAM, O(log n)) **und** hydratisieren nur nach erneuter Prüfung `gate.check_doc(tenant, collection, doc)` (Epochenprotokoll `e1`/`e2`, 03 §6.4). Ohne aktive Gates (`gate.is_active() == false`): ein Load, keine Mengenprüfung. Zeitreise (`search_bm25_at`) wird ebenfalls gefiltert (Gate gilt für alle Sequenzen). Der Kandidatenfilter ändert keine Rangfolge-Statistik (BM25 verwendet bis zum Purge die alten N/avgdl; Abweichung ist begrenzt durch SLA und wird dokumentiert).

## 8. Witness

`LayerScanner::scan` des Textindex (`inverted/scanner.rs`) bleibt Mess-Quelle. Anpassungen:
- Nadeln: `DocId` in Dezimal, LE-/BE-Bytes, Hex (nur im RAM); **Terme** der Scope-Dokumente (aus der Arbeitsliste, nur im RAM, nie im Journal).
- Positivkontrolle bleibt zwingend (Kanarien-Dokument) und läuft **vor** dem Scan.
- `remaining` = Zahl der `plb:`-Listen/`pl:`-Einträge mit Scope-DocId + RAM-Treffer in `resident_index`. `dl:`/`fw:`-Reste zählt der Store-Scanner (SSTable/WAL), nicht diese Schicht.
- Der Layer meldet in `LayerWitness` `NotMeasured`, wenn die Positivkontrolle fehlschlägt (I-6).

## 9. Verdrahtung (Engine)

`Collection::new`/`wiring` erzeugt je Collection `TextErasure` (Adapter `TextTxSource` über `TxIdAllocator` + `Contextra::next_tx`), registriert sie im `LayerRegistry` des Treibers (10 §4.3). Bis der Treiber existiert (WP-5), ist `TextErasure` über die öffentliche Konformanzsuite getestet (kein toter Code in `text`, dort gilt kein `deny(dead_code)`).

## 10. Tests

| ID | Inhalt |
| --- | --- |
| TST-TEXT-ERA-1…7 | Konformanz C1–C7 aus 13 §5 mit `InMemoryTextIndex`-nahem `InvertedIndex<InMemoryStore>` |
| TST-TEXT-ERA-8 | Canary: Dokument mit eindeutigem Term (`zq9canary…`), Erase, Scan über `plb:`/`pl:`/RAM ⇒ `remaining = 0`, Nachbardokument mit gleichem Term bleibt auffindbar, df korrekt |
| TST-TEXT-ERA-9 | Statistik: N und avgdl nach Erase = Neuberechnung aus Restkorpus (Property-Test, 200 Fälle) |
| TST-TEXT-ERA-10 | Wiederaufnahme nach Abbruch an **jedem** Commit (Crash-Injektion über `FaultVfs`/CrashVfs, 13 §4) liefert dasselbe Ergebnis wie ununterbrochener Lauf |
| TST-TEXT-ERA-11 | Fallback-Scan: `fw:` entfernt, Restposting vorhanden ⇒ Schicht findet und entfernt es |
| TST-TEXT-ERA-12 | Kandidatenfilter: nach `on_gate` liefert `search_bm25` das Dokument nie, auch nicht via `search_bm25_at(seq<gate)` |
| TST-TEXT-ERA-13 | `rg "block_on" crates/contextra-text/src/erasure.rs` leer (G-1) |
| TST-TEXT-ERA-14 | Gate: Delete der Schicht auf `dl:`-Schlüssel wird nicht versucht (Mock-Storage zählt Delete-Aufrufe auf Scope-Schlüssel = 0) |

## 11. Arbeitspakete

| WP | Inhalt |
| --- | --- |
| 6.1 | `TextTxSource`, `TextErasure` mit `on_gate`, Kandidatenfilter, Konformanz-Skelett |
| 6.2 | `purge` (Termliste, Commit, Collapse, Statistik, RAM) |
| 6.3 | Scanner-Nadeln, Positivkontrolle, Tests TST-TEXT-ERA-8…12 |
| 6.4 | Entfernen von `block_on_async`, `TextPurger` deprecated → gelöscht (nach 10, WP-5 grün) |

## 12. Definition of Done

Konformanz grün; Canary grün; `block_on` im Crate nur noch in Tests; G-1/G-2/G-4 grün für `text/src/erasure.rs`; keine neue Kante in `capabilities.toml`.

## 13. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-T1 | Enthalten Morphologie-/Trie-Caches Dokumentbezüge? | Lesen `morphology/trie.rs`; falls nein, Cache unverändert |
| O-T2 | Hat `commit` später eine Sequenz im Rückgabewert? | Dann `last_seq_no()` durch Rückgabe ersetzen |
| O-T3 | Mehrere Collections teilen `meta:stats`? | `meta:stats` ist namespacegebunden (`self.key`), je Instanz getrennt ✔ |
