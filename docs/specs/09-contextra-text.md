# `contextra-text` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|BM25-Volltextindex als verschlüsselte Segmente mit deutschem Analyzer.|
|Ring|2|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports`, `crypto` (SPEC 5.3, D-88 (3)); Verbraucher: `engine` (**nicht** `store`, `vector`, `erasure`, AR-2/AR-3)|
|Stufe / Gates|S1; G-E (Textindex), G-D (Schicht `ErasureLayer`)|
|Arbeitspakete|WP-7.1 (Segmentindex, Analyzer, BMW, Positionen, Entkopplung, Rewrite), WP-1.1/1.2 (Rückbau), WP-0.3|
|SPEC-Anker|8.9, 9.7, D-20, D-49, D-62, D-69, D-71, D-75, D-81, D-85, E-L5, E-L15, F.7, T-TEXT-*, BM-TEXT, BM-ERASE-WA-TEXT|

## 1. Ist-Zustand (HEAD `0ede89a26`) [V]

|Punkt|Befund|
|---|---|
|Umfang|7.151 Zeilen `src` (inkl. Inline-Tests ca. 2.400): `wand.rs` (885), `inverted/index_struct.rs` (699), `bm25.rs` (683), `tokenizer.rs` (550), `posting_list.rs` (548), `morphology/*` (ca. 1.300 inkl. Tests), `stream.rs` (184), `domain/*` (ca. 320), `lib.rs` (339)|
|Architektur|**termgeordneter Index im Key-Value-Speicher**: `InvertedIndex` legt Postings über `StorageEngine` ab (33 Treffer `StorageEngine`), Statistik als `TextIndexMetadata` unter dem Schlüssel `meta:stats` (`total_docs`, `total_tokens`, `avg_doc_len_x1000`). Das widerspricht D-20 (der LSM hält keine Postings) und E-L5 (Zähler überleben die Löschung ihres Beitrags nicht).|
|Identität|`DocId` in 146 Zeilen; `TextIndex`-Port (`ports/src/text_index.rs`): `search(&str, k)`, `search_at(.., seq_no)` (Standard: `CapabilityUnsupported`), `insert(tx, DocId, &str)`. Soll: `DocHandle`, Bindung nur als Envelope unter dem DEK (D-69, D-70).|
|Analyzer|`Language { English, German, Custom }` mit **Fallback auf `English`** bei unbekanntem Code (`from_iso`); `GermanMorphTokenizer`, `GermanCompoundSplitter` (Wörterbuch `src/data/german_words.txt`, Trie), `normalize_umlauts`, Stoppwörter. Es gibt **keinen** Snowball-Stemmer, keine NFKC-Normalisierung (kein `unicode-normalization` im Manifest), kein `ß→ss`-Faltungsgoldenset. `unicode-segmentation` ist vorhanden (UAX #29).|
|Scoring|`bm25.rs`: Standard `k1 = 1.5` (Soll **1,2**, 9.7), `b = 0.75`; `BM25F` (Feature `bm25f`); `wand.rs` (WAND) ohne Blockmetadaten für Block-Max-WAND (D-81); `posting_list.rs` ohne 128er-Blöcke, ohne Positionen als Format|
|Features|`bm25f`, `docid-128` (beide zu entfernen, D-84); 7 `cfg(feature…)`-Stellen|
|Domäne|`domain/legal_de.rs`, `domain/medical_de.rs`, `DomainVocabulary`: ohne SPEC-Anker (D-81 kennt nur `de/1` und `und/1`)|
|Dateisystem|0 Treffer `std::fs` (positiv); aber auch kein Segment-, Datei- oder `Vfs`-Bezug|
|Manifest|`bincode`, `serde_json`, `regex`, `futures`, `itoa`, `bytes`, `parking_lot`, `blake3`; **`crypto` fehlt** (F-35); `ring = "0"`; `lib.rs` nennt Ring 0 und Verbraucher `contextra-db`, `-router`, `-agent`, `-bench`|
|Tests|21 Testdateien (u. a. `wand_differential.rs`, `index_purge_proptests.rs`, `bm25_reference_values.rs`, `purge_error_injection_tests.rs`, `write_amplification.rs`), 2 Fuzz-Ziele (`fuzz_bm25_tokenize`, `fuzz_tokenizer_smoke`); Tests gegen den KV-Index werden mit dem Umbau hinfällig|

## 2. Verantwortung und Abgrenzung

`text` ist die Schicht **S1**: ein **abgeleiteter, segmentierter, verschlüsselter** BM25-Index (D-20). Es implementiert `TextIndex` und `ErasureLayer` (Schicht `text_postings`, `key_coverage = unlinked`, D-69). Die Quelle der Wahrheit für Dokumenttext bleibt der Dokumentspeicher (`store`); `text` hält davon abgeleitete Strukturen und kennt `store` nicht (AR-2). Datei-I/O nur über `Vfs`, Dateischlüssel nur über `FileKeyring` (Ports); Kryptoprimitive aus `crypto` (Block-AEAD, Envelope, `term_tag`).

**Grenzen (ausgewiesen, nicht geschönt):** keine Präfix-, Wildcard- oder Regex-Suche (Terme liegen nur als `term_tag` vor, 9.7); Metadatenfilter nachgelagert (D-62); ein Wörterbuchangriff gegen `term_tag` setzt `K_idx` voraus, daher Restrisiko `TextIndexUntilPurge` bis zum Segment-Rewrite (D-69, E-L15).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|SPEC|
|---|---|---|
|`analyzer`|Trait `Analyzer` (statisch dispatcht), `de1` (UAX #29, NFKC, Case-Folding, `ß→ss`, Snowball-German, Stoppwörter, wörterbuchbasierter Dekompounder), `und1` (neutral); `analyzer_id`, `analyzer_version`, `lang`|D-81|
|`buffer`|Textpuffer: Zugriff auf `TextBuf`-Records über den Port (Cluster des Dokuments, `full`)|9.7|
|`segment`|`SegmentBuilder`, `SegmentReader` (F.7): Wörterbuch (`term_tag`), Postings in 128er-Blöcken (VByte-Delta), Positionen (optional), Dokumenttabelle mit Bindungs-Envelopes, Lösch-Bitmap, Statistikblock|F.7|
|`merge`|Segment-Merge (Auswahl, Neuaufbau, adaptive Zielgröße 8–32 MiB)|D-71 (b)|
|`score`|BM25 (`k1 = 1,2`, `b = 0,75`), globale Statistik aus Segmentsummen, Block-Max-WAND (`wand`), Phrasenprüfung über Positionen|9.7, D-81|
|`search`|Kandidatenstrom (`Stream`), Bindung nur für Top-k-Kandidaten entschlüsseln, `ReadGate`-Filter mit Nachladen|D-62, D-69|
|`layer`|`TextLayer: ErasureLayer` (`resolve`, `on_gate`, `purge`, `witness`, `key_coverage = Unlinked`)|8.2, 11.1|
|`error`|`TextError` → `types::Error`|7.5|

Entfallen: `inverted/*` (KV-Index), `posting_list.rs` und `stream.rs` in der Ist-Form (neu in `segment`/`search`), `domain/*`, `bm25::BM25F`, `Language::Custom`, `morph_index.rs`, `GermanMorphTokenizer` als eigener Typ (aufgehen in `analyzer::de1`). Die Wörterbuchdatei `german_words.txt` bleibt **nur**, wenn der Dekompounder sie nutzt und ihre Lizenz im Evidenzbündel steht (HR-12) [E].

## 4. Schnittstellen [E]

```rust
// ports::TextIndex (statisch dispatcht, D-03); Ziel-Signaturen, Ist-Trait siehe Datei 02
pub trait TextIndex: Send + Sync + 'static {
    async fn index(&self, ctx: &TextCtx, doc: DocHandle, seq: Seq, text: &Analyzed) -> Result<()>;   // in den Puffer; Bindung unter DEK
    async fn remove(&self, ctx: &TextCtx, doc: DocHandle, seq: Seq) -> Result<()>;                  // Bitmap/Pufferzeile
    async fn search(&self, ctx: &TextCtx, q: &TextQuery, k: usize) -> Result<CandidateStream<DocHandle>>; // Strom, kein Vec
    async fn stats(&self, ctx: &TextCtx) -> Result<TextStats>;
}
pub struct TextCtx { tenant: TenantTag, coll: CollTag, analyzer: AnalyzerId }   // Analyzer je Collection unveränderlich (D-81)
pub struct TextQuery { terms: Vec<TermTag>, phrases: Vec<Vec<TermTag>>, }       // ≤ 256 Terme (D-87)
```

`search` liefert einen **Strom** `DocHandle`-Kandidaten mit Score (absteigend); das Nachladen bis `k` gültige Treffer (D-62) steuert `engine` (Verbraucher des `ReadGate`), `text` setzt den Postings-Strom fort. `search_at` (Zeitreise) ist kein Merkmal der Textschicht: Sichtbarkeit lösen `engine` und `ReadGate` auf (D-88 (2) gilt sinngemäß); der Ist-Standardzweig `CapabilityUnsupported` entfällt [Lücke → siehe Abschnitt 10].

```rust
// Segmentkopf (F.7) – Layout big-endian, wörtlich aus Anhang F
pub struct SegmentHeader { bucket: u8, bucket_count: u8, analyzer_id: u8, analyzer_version: u16,
                           lang: [u8; 3], positions: bool, doc_count: u32, term_count: u32, total_len: u64 }
// Bindung (D-69): Envelope v2 unter dem DEK, AAD = "ctx/tbind/v1" ‖ file_id ‖ u32(doc_ord)
pub struct Binding { cluster: ClusterTag, doc: DocTag }
```

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|TX-01|Der Textindex ist abgeleitet und segmentiert; der LSM hält **keine** Postings. Der KV-Index (`inverted/*`, Schlüssel `meta:stats`) entfällt vollständig.|D-20, F-23|T-TEXT-1, Grep|
|TX-02|Segmente: `text/<tenant_tag_hex>/<coll_tag_hex>/<file_id_hex>.seg`, unveränderlich, Block-AEAD unter zufälligem Dateischlüssel (`FileKeyring::create`, **vor** dem ersten Block, I-9b). Kein Klartextname, keine Collection-Namen im Pfad.|D-75, D-85, F.7|T-RAW-6, T-FK-1|
|TX-03|Terme liegen **nur** als `term_tag = tag("ctx/term/v1", analyzer_id ‖ lang ‖ normalisierter_term)` vor; nie ein Klartextterm in einer Datei, im Wörterbuch oder in Logs.|D-81, 9.7|T-RAW-4|
|TX-04|Postings tragen nur segmentlokale `doc_ord`. Die Bindung `doc_ord → (cluster_tag, doc_tag)` liegt je Dokument als Envelope unter dem DEK des Dokuments (AAD `ctx/tbind/v1 ‖ file_id ‖ doc_ord`). Nach Schlüsselvernichtung sind Postings `unlinked`; fehlt der DEK einer Bindung, gilt der Treffer als gelöscht und wird übersprungen.|D-69|T-TEXT-4, T-KEYCOV-1|
|TX-05|Ein Dokument liegt zu jedem Zeitpunkt in genau einem Segment **oder** im Textpuffer. Update: Eintrag im alten Segment per Bitmap als gelöscht markieren und neu in den Puffer; Merge baut Segmente neu.|D-49, 9.7|T-TEXT-1/2|
|TX-06|Segmentheader trägt `bucket`, `bucket_count` (v1: `1`), `analyzer_id`, `analyzer_version`, `lang`, `positions`. Bucket-Funktion `cluster_tag[0] mod bucket_count`. Segmentzielgröße adaptiv: 8 MiB bis 1 Mio. Chunks je Collection, danach 32 MiB. Über `bucket_count > 1` und Segmentgrößen entscheidet G-E anhand BM-ERASE-WA-TEXT (D-71 d), nicht per Annahme.|D-71, F.7|BM-ERASE-WA-TEXT, T-FMT-1|
|TX-07|Analyzer `de/1` und `und/1`. Analyzer, Version und Sprache sind je Collection unveränderlich; ein Wechsel ist Neuaufbau in einer neuen Collection. Unbekannte Analyzer-ID ⇒ `Config` (kein stiller Fallback auf Englisch, P2).|D-81|T-TEXT-ANALYZER-1|
|TX-08|Postings in 128er-Blöcken mit Blockmetadaten (`max_doc_ord`, `max_tf_norm`) für Block-Max-WAND; Positionen (Collection-Option, Standard an) für Phrasensuche. BM25 mit `k1 = 1,2`, `b = 0,75` und **globalen** Statistiken aus Segmentsummen.|D-81, 9.7|T-TEXT-PHRASE-1, BM-TEXT|
|TX-09|**Erase:** alle Segmente, die Zieldokumente enthalten, werden im **ersten Purge-Fenster** ohne die Zieldokumente neu geschrieben; df, Längensummen, Blockmaxima und Segmentstatistik werden **exakt neu berechnet** (E-L5); kein Zähler überlebt die Löschung seines Beitrags. Das alte Segment geht durch Dateischlüssel-Vernichtung (`FileKeyring::destroy`) und `unlink`. Bericht: `segments_rewritten`, `bytes_rewritten`.|D-69, D-71 c, E-L5, 11.2 Nr. 5b|T-TEXT-3, BM-ERASE-WA-TEXT|
|TX-10|`TextLayer::resolve` ist O(Schichtstruktur) und listet die zielgrößenabhängige Segmentmenge **nicht** vor `Gated`; die Auflistung erfolgt im Purge (11.1). `on_gate` ist leer (keine Sofortmaßnahme nötig, Gate filtert), `key_coverage = Unlinked`.|11.1, D-69|T-KEYCOV-1|
|TX-11|`witness`: MK1 (kein Eintrag des Ziels in Wörterbuch, Dokumenttabelle, Bitmap-Zuordnung, Pufferzeilen) und MK2 (Rohscan der Segmentdateien inkl. alter Segmente und `tmp/`) über `Scanner<Armed>`; rein lesend.|12.1, I-19|T-RAW-1..6|
|TX-12|Suche: Query-Terme ≤ 256 (`Limit::QueryTooLarge`), `k ≤ 1000`; Kandidatenstrom statt Vektor; Bindung nur für Top-k-Kandidaten entschlüsseln. Fehlschlag beim Entschlüsseln einer Bindung wegen fehlenden Slots ⇒ überspringen; jeder andere Fehler ⇒ `Corrupt`.|D-62, D-87|T-HYB-1, T-SEARCH-2|
|TX-13|Merge verwirft keine Bindung und keinen Eintrag eines gegateten Bereichs der Variante `Restricted` (I-18); für `Erased` gilt der Gate-Verwurf beim Neuschreiben (D-78) als Teilpurge, der Zeuge bleibt Pflicht.|I-18, D-78|T-COMPACT-RESTRICT-1|
|TX-14|Keine Features `bm25f`/`docid-128`, keine Domänenvokabulare, kein `Language::Custom`. Ring 2 und Abhängigkeiten `types`, `ports`, `crypto` im Kopf und Manifest; Abhängigkeiten ohne Verwendung (`regex`, `serde_json`, `bincode`, `futures`) nur bei belegter Verwendung.|D-84, P8, D-91|T-CFG-1, T-ARCH-5|
|TX-15|Neue Abhängigkeiten (`unicode-normalization`, Snowball-Stemmer) nur mit benanntem Test und Lizenzprüfung (HR-12); bevorzugt reine Rust-Crates ohne Netzwerk- oder `unsafe`-Anteil im Baum (AR-7, AR-8).|HR-12|`cargo-deny`|
|TX-16|Keine Panics, kein `unwrap`/`expect` (Ist-Muster u. a. in `from_iso`: `unwrap_or("")`), keine Indexierung per `[]`; keine Klartextterme in Fehlern.|P6, HR-4|Clippy, T-LOG-1|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`inverted/*` (KV-Index, `meta:stats`, `TextIndexMetadata`), `index_struct.rs`, `morph_index.rs`|durch `segment`/`search`/`buffer` ersetzen; Altdaten werden nicht migriert (kein Legacy-Lesepfad, P3)|WP-7.1|T-TEXT-1, Grep `meta:stats` = 0|
|`DocId` (146 Treffer), `docid-128`|`DocHandle`/`DocTag`; Feature entfernen|WP-3.1, WP-7.1|Grep|
|`bm25f`, `BM25F`, `FieldId`, `FieldWeight`, `score_term_bm25f`|entfernen|WP-1.2|T-CFG-1|
|`domain/*`, `DomainVocabulary`, `LegalDomainVocabulary`, `MedicalDomainVocabulary`|entfernen (kein Kernkonsument, P8)|WP-1.2|Grep|
|`Language::{English, German, Custom}` mit Fallback|durch `AnalyzerId` ersetzen (TX-07)|WP-7.1|T-TEXT-ANALYZER-1|
|`BM25_K1 = 1.5`|auf `1,2` ändern; Referenzwerte (`bm25_reference_values.rs`) neu erzeugen|WP-7.1|BM-TEXT, Referenztest|
|`wand.rs` (885 Z.)|zu Block-Max-WAND umbauen; Differentialtest gegen Vollscan bleibt|WP-7.1|`wand_differential.rs` (angepasst)|
|`stream.rs`, `Bm25CandidateStream`|auf `CandidateStream<DocHandle>` abbilden|WP-7.1|T-HYB-1|
|Testdateien zum KV-Index (`index_purge_proptests`, `purge_error_injection_tests`, `tombstone_*`, `write_amplification`)|durch Segment-Entsprechungen ersetzen oder entfernen|WP-7.1|T-TEXT-*|
|Manifest: `ring = "0"`, fehlendes `crypto`, tote Abhängigkeiten; `lib.rs`-Kopf (Ring 0, `contextra-db`, …)|TX-14; Kopf nach Q-01|WP-1.5|T-ARCH-5|

## 7. Arbeitspakete

1. WP-7.1 (rot, Reihenfolge nach Gate-Abhängigkeit): (a) Goldenset `T-TEXT-ANALYZER-1` (Komposita, Umlaute, `ß`, NFKC-Fälle) gegen `de/1`; (b) Format-Roundtrip `segment` mit Golden-Datei (F.7); (c) `T-TEXT-4`: Postings sind nach Vernichtung des DEK `unlinked`; (d) `T-TEXT-3`: Rewrite entfernt Zieldokumente und berechnet Statistiken exakt neu.
2. Grün: `analyzer` → `segment` (Builder/Reader) → `buffer` → `score` (BMW, Positionen) → `merge` → `search` → `layer`.
3. G0-Verbund: `BM-ERASE-WA-TEXT` (1/10 GiB, Betroffener mit 10³ Dokumenten über 12 Monate; Segmentgrößen 8/32 MiB; `bucket_count` ∈ {1, 4, 16}); Entscheidung D-71 d protokollieren.
4. Abhängigkeiten: `ports::TextIndex`, `FileKeyring`, `RecordSealer` (Datei 02), `types` (Tags, `DocHandle`), `crypto` (Block-AEAD) müssen vorab stehen (WP-3.1, WP-3.2); `engine` verdrahtet (WP-5.1).
5. Fuzz: Segmentparser (F.7), Analyzer-Eingaben; bestehende Fuzz-Ziele bleiben als Analyzer-Fuzz.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-TEXT-1..4|Segmentbildung, Merge, Rewrite je Zielsegment, Statistiken exakt neu (E-L5); T-TEXT-4: Postings `unlinked`|
|T-TEXT-ANALYZER-1|Goldenset Komposita, Umlaute, `ß` für `de/1`|
|T-TEXT-PHRASE-1|Phrasensuche über Positionen|
|T-KEYCOV-1|`key_coverage` der Schicht = `unlinked` stimmt mit Messung überein|
|T-RAW-1..6|MK1/MK2 für Segmente, alte Segmente, `tmp/`; Dateinamen nur Tags (T-RAW-6); Tags im Index (T-RAW-4)|
|T-FK-1..3|Dateischlüssel vor erstem Block, Vernichtung nach Swap vor `unlink`, freigegebene Blöcke nicht entschlüsselbar|
|T-HYB-1, T-SEARCH-2|Nachladen bis `k` oder Erschöpfung; `exhausted` korrekt (Zusammenspiel mit `engine`)|
|T-COMPACT-RESTRICT-1|Merge verwirft `Restricted`-Daten nie|
|T-CRASH-1/2 (Anteil)|Absturz im Segment-Rewrite: altes Segment bleibt gültig|
|T-FMT-1|F.7-Header, Golden-Datei|
|BM-TEXT|BM25 top-10 auf 1 Mio. Chunks (`de/1`, BMW): p99 ≤ 30 ms|
|BM-ERASE-WA-TEXT|Aufwand ≤ 2 × Σ Größe der berührten Segmente; Entscheidung `bucket_count` (D-71 d)|
|Property|`proptest`: BM25-Invarianten (`proptest_bm25_invariants.rs` bleibt), BMW = Vollscan bei beliebigen Eingaben|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: Grep `inverted|meta:stats|DocId|bm25f|docid-128|Language::` in `text` = 0; Segmentdateien tragen nur Tags und `file_id`; `BM-TEXT` und `BM-ERASE-WA-TEXT` protokolliert, Entscheidung D-71 d im Evidenzbündel (G-E); Zeilenbudget: `text` darf nach dem Umbau nicht über dem Ist-Umfang (7.151) liegen, Ziel ≤ 5.000 ohne Tests (Budget 5.8: Kernreduktion ≥ 30 %) [E].

## 10. Offene Punkte

- **L-15 (neu): Zeitreise im Textindex.** SPEC 9.7 beschreibt Segmente und Puffer ohne Versionssicht; D-11 verlangt Zeitreisen für Dokumente, 7.2 nennt `search_text` ohne `at`. Offen, ob Textsuche Zeitreisen unterstützt. Vorschlag **[E]**: nein in v1 (Suche immer auf dem aktuellen Stand, `get` mit `at` für Dokumente); der Ist-Pfad `search_at` mit `CapabilityUnsupported` entfällt. Entscheidung per Anhang H.
- **L-16 (neu): Analyzer-Abhängigkeiten.** `de/1` verlangt NFKC, Case-Folding, Snowball-German und Wörterbuch-Dekompounder (D-81). Das Manifest enthält dafür nur `unicode-segmentation`; Quelle für Stemmer und Normalisierung (Crate oder Eigenbau) und die Lizenz der Wörterbuchdatei sind offen (TX-15). Ohne Klärung sperrt der Punkt T-TEXT-ANALYZER-1.
- **Schlüssel `K_idx` im Segment:** `term_tag` setzt `K_idx` voraus; `text` erhält ihn über einen Port (`TagSource`?) oder über `crypto::IdxKey`. SPEC 5.3 erlaubt `text→crypto`; die Übergabe des Schlüssels aus `engine` ist festzulegen (WP-7.1).
- **BM-ERASE-WA-TEXT** kann `bucket_count > 1` erzwingen; dann ist F.7 unverändert (Header-Feld vorhanden, D-71 a), aber `resolve` und `merge` müssen Buckets kennen. Der Umfang ist bis G-E zurückgestellt.
- **Statistik und E-L5:** Das Neuberechnen aller Statistiken im Rewrite genügt nicht, wenn die **globale** Statistik (`avg_doc_len`, `df`) im Speicher gecacht wird; Cache-Eviction in 11.2 Nr. 5g muss den Statistik-Cache einschließen.
