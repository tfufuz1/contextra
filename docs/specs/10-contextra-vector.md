# `contextra-vector` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|HNSW-Vektorindex mit Arena, löschbaren Slots und verschlüsseltem Snapshot.|
|Ring|2|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports`, `simd`, `crypto` (SPEC 5.3; **nicht** `sys`, **nicht** `mvcc`, D-88 (2)); Verbraucher: `engine`|
|Stufe / Gates|S2; G-G (Vektorindex), G-B (Rückbau, ein `Vfs`), G-A (Build, F-32)|
|Arbeitspakete|WP-8.1 (Arena, Surrogat, Kantenreparatur, Rebuild, Snapshot), WP-0.3 (`mmap_readonly`-Aufrufer), WP-1.2 (Features), WP-1.4 (`Vfs`)|
|SPEC-Anker|8.10, 9.8, D-18, D-19, D-62, D-63, D-77, D-84, D-85, D-86, D-88, D-94, E-L3, E-L15, F.8, F-23, F-31, F-32, F-36, T-VEC-*|

## 1. Ist-Zustand (HEAD `0ede89a26`) [V]

|Punkt|Befund|
|---|---|
|Umfang|13.624 Zeilen `src`; außerhalb Soll (F-31, F-36): `diskann/*` (4.122), `quantize/*` (1.116), `quantize_rabitq.rs` (506), `partial_rebuild.rs` (363), `acorn/*` und `hnsw/acorn_filtered.rs` (213 + 297), `hnsw/sq8_bias.rs` (108), `compute_pool.rs` (207), `hnsw/adaptive_ef.rs` (229, Teil bleibt: D-62). Rest im Soll-Umfang: `hnsw/` (`core_insert` 446, `core_search` 737, `core_rebuild` 1.058, `arena` 406, `types` 832, `vector_index_impl` 568, `config` 210, `batch` 89), `persistence/` (`header` 437, `mmap` 221, `node` 82), `distance.rs` (249), `candidate_stream.rs` (170).|
|Build|`persistence/mmap.rs:24` und `diskann/persistence.rs:291` rufen `contextra_sys::mmap_readonly` (`unsafe fn`) aus einem `forbid(unsafe_code)`-Crate auf ⇒ Fehler E0133 (F-32). Der Build-Bruch ist durch D-94 (kein `mmap`) zu beheben, nicht durch `unsafe`.|
|Dateizugriff|54 Treffer `std::fs|tokio::fs|File::open|File::create|OpenOptions|memmap2` in `src` (F-29: 51, F-34: 54); Datei `.hnsw` mit `HNSW_MAGIC = 0x484E5357`, `HNSW_VERSION = 2`, 84-Byte-Header, **unverschlüsselt**, Felder `q_min`/`q_max` (Altfeld), `sq8_bias_*`, `last_tx_id`.|
|Abhängigkeiten|`mvcc` (7 Treffer) und `sys` (2 Treffer) – beide gegen D-88 (2) und Matrix 5.3 (F-27, F-35); `memmap2`, `roaring`, `rand`, `serde`, `subtle`, `zeroize`, `blake3`, `ahash`, `parking_lot`|
|Löschen|`HnswArena::zeroize_and_free_node` (Zeroisierung eines Knotens, `arena.rs:298`), `free_node`; Tombstone-/Freilistenlogik in `hnsw/*` (21 Treffer); `core_rebuild.rs` (1.058 Z.) mit Rebuild-Schwelle `rebuild_threshold` und exponentiellem Backoff (`backoff_*`). **Keine** Batch-Kantenreparatur, **kein** Surrogat (D-77), **keine** Slot-Bindung unter DEK (D-19).|
|Quantisierung|`HnswConfig::quantize` (SQ8), `quantize/*`, `sq8_bias.rs`, RaBitQ: legen Vektor-Repräsentationen **außerhalb** einer zusammenhängenden Slot-Arena ab (verstößt gegen D-86)|
|Identität|`DocId` in 192 Zeilen; Port `VectorIndex` (Ist): `insert(tx, DocId, &[f32])`, `insert_batch`, `search(&[f32], k)`, `search_at(.., seq)` (Standard `CapabilityUnsupported`), `all_doc_ids`|
|Features|`docid-128`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`, `partial-index-rebuild` (mit Warnkommentar im Manifest) – alle zu entfernen (D-84); 6 `[[test]]`- und mehrere `[[bench]]`-Einträge hängen an diesen Features|
|Tests|49 Testdateien, u. a. `hnsw_snapshot_delete_test.rs`, `deletion_ghost_pointer_proptest.rs`, `reconstruction_attack_test.rs`, `hnsw_seed_determinism.rs`, `k02_snapshot_rebuild_determinism.rs`, `recall_vs_brute_force.rs`, `mmap_malformed_data_test.rs`; 11 Benchmarks (u. a. `hnsw_bench.rs`, `hnsw_delete_search_bench.rs`); 2 Fuzz-Ziele|
|Manifest|`ring = "0"` (Soll 2); `lib.rs` ohne Ringangabe (F-35)|

## 2. Verantwortung und Abgrenzung

`vector` ist die Schicht **S2**: ein HNSW-Index (`M = 16`, `efConstruction = 200`, Standard `efSearch = 64`, 9.8) über einer **Arena mit festen, zeroisierbaren Slots**. Es implementiert `VectorIndex` und `ErasureLayer` (Schichten `vector_payload`, `key_coverage = full`, und `vector_edges`, `key_coverage = none`, D-69). Es kennt **kein** `mvcc` und **kein** `sys`: Sichtbarkeit und Snapshot-Widerruf (D-61) bleiben in `engine`/`erasure`; das Sperren der Arena-Seiten (`mlock`, `MADV_DONTDUMP`, D-63) geschieht über den Port `ArenaGuard` (Datei 02, PO-11), den `engine` mit `sys` implementiert. Datei-I/O nur über `Vfs` (positionelles Lesen, kein `mmap`, D-94), Dateischlüssel nur über `FileKeyring`, DEKs nur über `RecordSealer`.

**Grenzen (ausgewiesen):** Metadatenfilter nachgelagert (D-62); nach einer Löschung weist der Bericht `IndexTopologyInference` bis zum nächsten Rebuild aus (Reparaturkanten sind eine Funktion der Zielnachbarschaft, D-77); Zeitreisen gibt es für den Index nicht (siehe Abschnitt 10).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|SPEC|
|---|---|---|
|`arena`|`Arena`: feste Slots, je Slot **ein** zusammenhängender, zeroisierbarer Bereich mit allen Repräsentationen (v1: f32), Freiliste, Slot-Tabelle `slot → (cluster_tag, doc_tag, chunk)`|D-86, 9.8|
|`graph`|Adjazenzlisten je Level (Slotnummern), Einfügen (`insert`), Suche (`search`), Einstiegsknoten, deterministische Levelzuweisung|9.8|
|`delete`|`surrogate` (Navigationssurrogat), `repair` (Batch-Kantenreparatur in **einem** Durchlauf), Freiliste|D-77 (1), (2)|
|`rebuild`|deterministischer Rebuild in `doc_tag`-Reihenfolge, Quoten `vector_rebuild_ratio` und 10 %-Regel, `vector_strict_rebuild`|D-77 (3)|
|`search`|Kandidatenstrom, `ef`-Verdopplung bis `ef_max = 4096` bei gegateten Kandidaten|D-62|
|`snapshot`|`.vsn` lesen/schreiben (F.8) über `Vfs`; Slot-Abschnitt unter DEK je Dokument, Graph-Abschnitt unter dem Dateischlüssel|D-19, D-85, F.8|
|`layer`|`VectorLayer: ErasureLayer` (`resolve`, `on_gate`, `purge`, `witness`, `key_coverage`)|8.2, 11.1|
|`config`|`VectorConfig` (Dimension ≤ 4096, `M`, `ef_c`, `ef_s`, Metrik, `rebuild_ratio`, `strict_rebuild`)|5.7, D-87|
|`error`|`VectorError` → `types::Error`|7.5|

Entfallen: `diskann/*`, `quantize/*`, `quantize_rabitq.rs`, `partial_rebuild.rs`, `acorn/*`, `hnsw/acorn_filtered.rs`, `hnsw/sq8_bias.rs`, `compute_pool.rs` (nur behalten bei belegtem Kernkonsumenten, P8), `persistence/mmap.rs`, Altheader `.hnsw` (`HNSW_MAGIC`, `q_min`/`q_max`). Die Ist-Verzeichnisse `hnsw/` und `persistence/` werden zu den Modulen oben; Modulform nach D-90.

## 4. Schnittstellen [E]

```rust
// ports::VectorIndex (statisch dispatcht, D-03); Ziel-Signaturen
pub trait VectorIndex: Send + Sync + 'static {
    async fn insert(&self, ctx: &VecCtx, doc: DocHandle, chunk: u32, vec: &[f32]) -> Result<Slot>;   // Slot-Bindung im Speicher
    async fn remove(&self, ctx: &VecCtx, doc: DocHandle, chunk: u32) -> Result<()>;                  // logisch (Slot → Tombstone)
    async fn search(&self, ctx: &VecCtx, q: &[f32], k: usize, ef: usize) -> Result<CandidateStream<Hit>>;  // Strom
}
pub struct VecCtx { tenant: TenantTag, coll: CollTag }          // ein Index je (Mandant, Collection), D-75

// Löschpfad (D-77); nur über VectorLayer erreichbar
impl VectorLayer {
    async fn on_gate(&self, s: &LayerScope) -> Result<()>;      // Surrogat in ALLEN Repräsentationen der Zielslots (siehe L-17)
    async fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> Result<PurgeOutcome>;  // Batch-Kantenreparatur, Freiliste, Rebuild nach Quote, Snapshot neu
    fn key_coverage(&self) -> KeyCoverage;                       // Full (Payload) / None (Kanten): zwei Schichtkennungen
}
```

Arena-Slot (D-86), Layout **[E]** (Größenordnung 768 × f32 = 3.072 B Nutzlast je Slot):

```
slot := repr_f32[dim] ‖ (reserviert für spätere Repräsentationen: SQ8, PQ, binär; v1 leer, Größe im Header)
Freiliste: Slots im Zustand Free tragen nur Nullen
```

Snapshot `.vsn` (F.8): Header `dim(u16) ‖ metric(u8) ‖ M(u16) ‖ ef_c(u16) ‖ slot_count(u32) ‖ entry_slot(u32) ‖ max_level(u8)`; Slot-Abschnitt `slot(u32) ‖ envelope` (Klartext: Repräsentationen ‖ `cluster_tag ‖ doc_tag ‖ chunk(u32)`); Graph-Abschnitt als Adjazenzlisten je Level unter dem Dateischlüssel (Block-AEAD). Slots auf der Freiliste tragen keinen Envelope.

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|VC-01|Keine Abhängigkeit zu `sys` und `mvcc`; kein `memmap2`, kein `std::fs`/`tokio::fs`, kein `File::open`/`OpenOptions` auf Klartext. Lesen und Schreiben ausschließlich über `Vfs` (positionelles Lesen). Grep = 0.|D-88, D-94, D-04, F-13|T-LINT-2, T-ARCH-3, T-ARCH-6|
|VC-02|Arena: je Slot ein **zusammenhängender** Bereich mit **allen** Repräsentationen; **keine** Repräsentation außerhalb der Arena (kein Quantisierungs-Nebenarray). Gelöschte Slots werden in allen Repräsentationen überschrieben (Nullen) und erst danach in die Freiliste gestellt.|D-86|T-VEC-ARENA-1|
|VC-03|Löschung vor `Gated`: Arena-Vektoren der Zielslots werden durch ein **Navigationssurrogat** ersetzt (Mittel der Vektoren der nicht gegateten Nachbarn; ohne solche Nachbarn: Vektor des Einstiegsknotens). Kein Zielvektor-Byte bleibt in der Arena. Der Nullvektor ist **kein** zulässiges Surrogat.|D-77 (1), D-18|T-VEC-DEL-1|
|VC-04|Purge-Fenster: **Batch-Kantenreparatur** in einem linearen Durchlauf über alle Adjazenzlisten: Kanten auf Zielslots entfernen, Ersatzkanten mit der Standard-Auswahlheuristik aus den Nachbarn der Zielknoten wählen; danach existiert keine Referenz auf Zielslots, die Slots gehen in die Freiliste. Der Durchlauf ist speichergebunden (≤ 60 s je 10⁶ Knoten, BM-VEC-DEL).|D-77 (2)|T-VEC-DEL-2, BM-VEC-DEL|
|VC-05|Deterministischer Rebuild (Einfügung in `doc_tag`-Reihenfolge, deterministische Konfliktauflösung) bei Tombstone-Quote ≥ `vector_rebuild_ratio` (0,2) **oder** ≥ 10 % reparaturgelöschten Slots seit dem letzten Rebuild; mit `vector_strict_rebuild` (Standard aus) im selben Purge-Fenster, spätestens zum Ende von `erase_sla`. **Eigenschaft:** Der Graph nach Rebuild hängt nur von der Menge der lebenden Slots ab. Levelzuweisung und Tie-Breaks sind Funktionen von `doc_tag ‖ chunk`, nicht vom Einfügezeitpunkt [E].|D-77 (3), D-86|T-VEC-REBUILD-INDEP, BM-VEC-REBUILD|
|VC-06|Suche: Gegatete Kandidaten werden verworfen; `ef` wird ab `max(ef, k)` verdoppelt bis `ef_max = 4096`; liefert die Suche danach weniger als `k` gültige Treffer, ist die Seite erschöpft (`exhausted` setzt `engine`). Nie stilles Verkürzen. Dimension ≤ 4096, `k ≤ 1000`.|D-62, D-87|T-HYB-1, T-SEARCH-2, T-LIMIT-1|
|VC-07|Snapshot: Slot-Abschnitt je Slot ein Envelope v2 unter dem **DEK des Dokuments** mit Vektor **und** Bindung `(cluster_tag, doc_tag, chunk)`; Graph-Abschnitt unter dem Dateischlüssel (Block-AEAD, zufällig, `F`-Slot **vor** dem ersten Block, I-9b); Pfad `vector/<tenant_tag_hex>/<coll_tag_hex>/<file_id_hex>.vsn`. Nach Schlüsselvernichtung sind weder Vektor noch Zuordnung lesbar.|D-19, D-85, D-75, F.8|T-RAW-3, T-KEYCOV-1, T-FK-1..3|
|VC-08|Laden: Ein Slot, dessen Envelope wegen fehlenden DEK-Slots nicht geöffnet werden kann (`Erased`), wird als gelöscht markiert und in die nächste Kantenreparatur aufgenommen; jeder andere Öffnungsfehler ⇒ `Corrupt` (fatal für die Datei, P2). Unbekannte `format_version`/`alg_ids` ⇒ `Corrupt`.|P2, D-42, F.1|T-FMT-1, T-VEC-DEL-2|
|VC-09|Arena-Seiten: `MADV_DONTDUMP` und `mlock` (best effort) über den Port `ArenaGuard`; der Zustand erscheint in `health()` und im Bericht (`ProcessMemory`).|D-63, D-88|T-DUMP-1|
|VC-10|Je (Mandant, Collection) **ein** Index mit eigener Arena und eigenem Snapshot; Mandanten-Offboarding und Collection-Löschung entfernen die Dateien (`unlink` nach `F`-Slot-Nullung) und geben die Arena frei (Zeroisierung).|D-75, 11.10|T-TENANT-1|
|VC-11|Keine Features `diskann`, `rabitq`, `predicate-augmented-search`, `graph`, `partial-index-rebuild`, `docid-128`; keine Quantisierung in v1 (Layout bleibt erweiterbar, VC-02).|D-84, D-86|T-CFG-1, T-ARCH-3|
|VC-12|`DocId` entfällt; Slot-Tabelle und Treffer tragen `DocHandle`/`DocTag` (blind). Keine Klartextkennung im Index (HR-4, I-11).|D-08, D-70|T-DOCREF-1, T-LOG-1|
|VC-13|Positive Eigenschaft `recall`: Recall@10 ≥ 0,95 bei 1 M × 768 (p99 ≤ 15 ms) und nach Löschung von 5/10/20 % der Slots mit Batch-Kantenreparatur ≥ 0,95.|9.8, BM-VEC, BM-VEC-DEL|BM-VEC, BM-VEC-DEL|
|VC-14|Ring 2, Abhängigkeiten `types`, `ports`, `simd`, `crypto` im Kopf und Manifest; `unsafe` verboten (`forbid`); keine Panics, kein `unwrap`/`expect` (Ist: 76 `unwrap` in `src`, Ratchet sinkend, Ziel 0).|D-91, P6, Q-03|T-ARCH-5, Clippy|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`diskann/*` (4.122 Z.), Features `experimental-diskann`, `[[test]]`/`[[bench]]`-Einträge dazu|entfernen|WP-1.2|Grep, T-CFG-1|
|`quantize/*`, `quantize_rabitq.rs`, `hnsw/sq8_bias.rs`, `HnswConfig::quantize`, `quantizer_*`|entfernen|WP-1.2|Grep, T-VEC-ARENA-1|
|`partial_rebuild.rs`, Feature `partial-index-rebuild` (inkl. Warnkommentar im Manifest), `partial_rebuild_*`-Tests|entfernen (Ersatz: Batch-Kantenreparatur, VC-04)|WP-1.2|Grep|
|`acorn/*`, `hnsw/acorn_filtered.rs`, `experimental-predicate-augmented-search`|entfernen (Filter nachgelagert, D-62)|WP-1.2|Grep|
|`persistence/mmap.rs`, `diskann/persistence.rs:291`, Abhängigkeit `contextra-sys`, `memmap2`|entfernen; Snapshot über `Vfs` (VC-07)|WP-0.3, WP-1.4|`cargo check --locked`|
|Abhängigkeit `contextra-mvcc` (7 Treffer)|entfernen; Sichtbarkeit in `engine`|WP-1.5|T-ARCH-3|
|Altheader `.hnsw` (`HNSW_MAGIC`, `q_min`/`q_max`, `sq8_bias_*`, `last_tx_id`)|durch Format F.8 ersetzen; Altdateien werden nicht gelesen (P3)|WP-8.1|T-FMT-1|
|`DocId`, `docid-128`, `graph`|durch `DocHandle`; Features entfernen|WP-3.1, WP-1.2|Grep|
|`hnsw/core_rebuild.rs` (Rebuild mit Backoff, `rebuild_threshold`)|auf VC-05 umbauen; exponentielles Backoff entfällt (Auslöser sind Quote und `erase_sla`, nicht Zeit)|WP-8.1|T-VEC-REBUILD-INDEP|
|`compute_pool.rs`|prüfen (Kernkonsument?), sonst entfernen|WP-8.1|Review|
|`ring = "0"`, `lib.rs`-Kopf ohne Ring|Ring 2; Kopf nach Q-01|WP-1.5|T-ARCH-5|

## 7. Arbeitspakete

1. WP-0.3: Build grün machen, indem `persistence/mmap.rs` auf `Vfs`-Lesen umgestellt oder entfernt wird (D-94); danach `Cargo.lock` erneuern.
2. WP-1.2: Rückbau nach Abschnitt 6 (Features, DiskANN, Quantisierung, ACORN, RaBitQ, Teilrebuild). Umfangsziel: ca. 6.300 Zeilen (46 %, F-36) entfallen; Budget 5.8 profitiert.
3. WP-8.1 (rot): (a) `T-VEC-DEL-1`: nach `on_gate` enthält die Arena kein Byte eines Zielvektors (Bytevergleich gegen gesicherte Vektoren); (b) `T-VEC-DEL-2`: nach `repair` keine Referenz auf Zielslots; Recall-Grenze; (c) `T-VEC-REBUILD-INDEP`: zwei Einfügereihenfolgen derselben Lebendmenge ⇒ identischer Graph; (d) `T-VEC-ARENA-1`.
4. Grün: `arena` → `graph` → `delete` → `rebuild` → `snapshot` → `layer`.
5. Messung: `BM-VEC`, `BM-VEC-DEL`, `BM-VEC-REBUILD` auf Referenzhardware (14.8); Protokolle vor G-G.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-VEC-DEL-1/2|Surrogat sofort, kein Zielvektor-Byte in der Arena; Kantenreparatur ohne Referenz auf Zielslots, Recall-Grenze|
|T-VEC-REBUILD-INDEP|Graph nach Rebuild hängt nur von der Menge der lebenden Slots ab|
|T-VEC-ARENA-1|Alle Repräsentationen je Slot zusammenhängend und überschrieben|
|T-RAW-3|Embeddings: MK1/MK2 über Snapshot, Arena-Dump (Lab), `tmp/`|
|T-KEYCOV-1|`vector_payload = full`, `vector_edges = none` stimmen mit der Messung überein|
|T-FK-1..3|Dateischlüssel des `.vsn`: Anlage vor Block 0, Vernichtung nach Swap vor `unlink`|
|T-HYB-1, T-SEARCH-2|`ef`-Verdopplung bis 4096, `exhausted` korrekt|
|T-FMT-1|F.8-Header, Golden-Datei; Fuzz auf den Snapshot-Parser (bestehende Fuzz-Ziele angepasst)|
|T-DUMP-1|Arena-Seiten `MADV_DONTDUMP` (zusammen mit `engine`)|
|T-LIMIT-1|Dimension > 4096, `k` > 1000 ⇒ `Limit::*`|
|Property|Determinismus (`hnsw_seed_determinism`, `k02_snapshot_rebuild_determinism` bleiben angepasst); `deletion_ghost_pointer_proptest` bleibt (Löschen ⇒ keine Geisterzeiger)|
|BM-VEC, BM-VEC-DEL, BM-VEC-REBUILD|p99 ≤ 15 ms (Recall@10 ≥ 0,95, 1 M × 768); Recall ≥ 0,95 nach Löschung; Rebuild-Dauer dokumentiert, Recall ≥ Ausgang − 0,01|
|`reconstruction_attack_test.rs`|bleibt als Lab-nahe Prüfung der Surrogat-Eigenschaft: Aus dem Surrogat lässt sich der Zielvektor nicht rekonstruieren (Nachbarmittel); Ergebnis fließt in `IndexTopologyInference`|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: Grep `std::fs|tokio::fs|memmap2|File::open|OpenOptions|contextra_sys|contextra_mvcc|DocId|quantiz|diskann|rabitq|acorn` in `vector` = 0; `cargo tree -p contextra-vector` ohne `sys`/`mvcc`; alle `T-VEC-*` und `BM-VEC*` mit Protokoll; Lab belegt `key_coverage` (G-G, G-D).

## 10. Offene Punkte

- **L-17 (neu): D-77 (1) gegen P16.** D-77 verlangt das Surrogat **vor** `Gated` in allen Repräsentationen der Zielslots; P16 und D-30 verlangen, dass alles bis `Gated` O(1) bezüglich der Zielgröße ist (BM-ERASE-O1: Zeit(10⁶)/Zeit(10²) ≤ 1,2). Das Ersetzen von 10⁶ Vektoren (≈ 3 GB bei 768 × f32) ist speichergebunden linear, nicht O(1). Vorschlag **[E]**: Für `Document` sofort (wenige Slots); für `Subject`/`Collection`/`Tenant` hält das `ReadGate` die Zielslots sofort unsichtbar (Slotfilter über die Gate-Menge), die Zeroisierung der Arena läuft als **erster** Schritt des Purge-Fensters unmittelbar nach `Gated`; die Zwischenzeit steht als `ProcessMemory` (Klartext im Prozessspeicher, nicht auf Platte) im Bericht. Die Entscheidung berührt D-77, 11.2 Nr. 3 und BM-ERASE-O1 und gehört vor WP-8.1 in die SPEC (Anhang H).
- **L-18 (neu): Slot-Kennung und Port `ArenaGuard`.** SPEC 8.2 nennt den Port nicht; Datei 02 (PO-11) führt ihn als Entwurf. Signatur und Besitz (`engine` implementiert mit `sys`) sind mit WP-3.1 festzulegen; ohne Port bleibt VC-09 unerfüllbar (D-88).
- **Zeitreisen und Snapshot-Isolation.** Wie bei `text` (Datei 09, L-15): Der Index spiegelt den aktuellen Stand; der Ist-Pfad `search_at` mit `CapabilityUnsupported` entfällt. Das Verhalten von Suchtreffern gegenüber offenen Snapshots (Treffer nicht in der Snapshot-Version vorhanden) filtert `engine` über `ReadGate`/MVCC-Sicht beim Laden des Dokuments.
- **Wiederholbarkeit des Reparaturdurchlaufs:** „Standard-Auswahlheuristik“ (D-77 (2)) ist die HNSW-Heuristik nach Malkov/Yashunin; die Parameter (`M`, `M_max0`) und die Reihenfolge der Verarbeitung (Slot aufsteigend) legt WP-8.1 fest, damit der Durchlauf **deterministisch** ist (Voraussetzung für Lab-Differenzprüfung).
- **Surrogat und Rekonstruktion:** Das Nachbarmittel ist eine Funktion der Nachbarvektoren; ob es Rückschlüsse auf den Zielvektor erlaubt, prüft `reconstruction_attack_test.rs` (Ist, am Altdesign). Ergebnis bestimmt, ob `IndexTopologyInference` ausreicht oder ein Zufallsanteil nötig ist.
