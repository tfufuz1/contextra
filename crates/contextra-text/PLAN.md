# Implementierungsplan: contextra-text

**Ebene:** L2 · **Ziel-Zustand:** BM25/BM25F, WAND, deutsche Morphologie und Komposita-Zerlegung. Persistenzneutral. Keine globalen Wörterbücher oder Statistiken über Partitionen hinweg.
**Basis:** SPEC R14 §2.1, §2.7, §11.7, §11.5, Z-19, AT-7, R1, R2; R15 §2.3.

## 1. Ist-Befund (gemessen)

- 31 Quelldateien, 8 970 Zeilen. Module: `bm25`, `contract`, `data`, `domain` (Verzeichnis), `error`, `inverted` (Verzeichnis), `memory_index`, `morphology` (Verzeichnis), `posting_list`, `tokenizer` (1028), `traits`, `types`, `wand` (890), `inverted/scanner.rs`, `inverted/purge.rs`.
- **105 Treffer verbotener Muster im Produktionscode.**
- `Cargo.toml`: `bincode`, `blake3`, `parking_lot`, `futures`, `serde`, `serde_json`, `regex`, `itoa`, `unicode-segmentation`, `contextra-ports`, `contextra-types`.
- `inverted/scanner.rs` und `inverted/purge.rs` enthalten `block_on` (verboten, §13.3).
- `inverted/` enthält laut Befund Logik für Löschung (`purge`). Sie gehört nach SPEC zu `PartitionSink::rewrite_without`.
- Altpfad laut §13.3: `commit_lock` und globale `meta:stats`/`pl:` (Schlüsselpräfixe). Vor Umbau im Code suchen.
- `contract.rs` enthält `Epoch`, der laut §13.3 durch `types::Epoch` ersetzt werden muss.

## 2. Entfernen

| Ziel | Begründung |
|---|---|
| `commit_lock` | Altpfad (§13.3), Schreibpfad kommt über `PartitionSink` |
| `meta:stats`- und `pl:`-Schlüssel | globale Statistik, verboten (§13.3); ersetzt durch partitionsbezogene Statistik |
| `text::contract::Epoch` | ersetzt durch `types::Epoch` |
| `block_on` in `inverted/scanner.rs`, `inverted/purge.rs` | nur `engine::kernel` |
| `inverted/purge.rs` als Löschlogik | ersetzt durch `rewrite_without` (R14 §13.2 Nr. 9) |
| `bincode`, `blake3` (unkeyed), `serde`, `serde_json`, `futures`, `parking_lot`, `tokio` | verboten bzw. nicht nötig |
| `memory_index.rs`, falls nur Testhilfe | prüfen, nach Test-Modul verschieben |

## 3. Ändern

- **`tokenizer.rs` (1028 Zeilen):** trennen in `tokenizer/split.rs` (Unicode-Segmentierung), `tokenizer/normalize.rs` (Umlaute, ß, Groß-/Kleinschreibung), `tokenizer/stop.rs` (Stoppwörter, konfigurierbar).
- **`wand.rs` (890):** WAND-Iteration in `wand/iter.rs` und Scoring in `wand/score.rs` trennen. Scoring nutzt BM25F-Gewichte pro Feld.
- **`bm25.rs`:** Parameter `k1`, `b` pro Partition, nicht global. Abfragezeit-`df` (§13.3).
- **`posting_list.rs`:** kanonische Codierung über `types`, nicht `bincode`. Delta-Codierung der Doc-IDs.
- **`morphology/`:** Lexikon und Zerlegung als Daten, nicht als Code-Konstanten. Lizenz der Wörterbuchdaten prüfen.
- **`contract.rs`:** `PartitionSink` und `PartitionSource` nach Vorbild `vector`.
- **`inverted/scanner.rs`:** Scanner läuft als Wartungsjob (Lane `idle`), nicht im Lesepfad.

## 4. Implementieren

1. **Partitionsstatistik:** `N`, `avgdl`, `df` je Term, je Partition. Abfragezeit-`df` = Summe über Partitionen der Abfrage (§13.3, R2).
2. **BM25F:** Gewichte pro Feld (Titel, Body, Metadaten), konfiguriert, versioniert.
3. **WAND:** Top-k mit Schwellwert, deterministische Sortierung bei gleichen Scores (Tie-Break per Doc-ID).
4. **Deutsche Morphologie:** Stemming und Lemmatisierung über Lexikon, Komposita-Zerlegung (z. B. „Datenschutzbeauftragter“ in Teile). Messung über Testkorpus; Fehlerrate fließt in Messbericht (Z-19).
5. **Ingest-Pfad:** Dokument → Tokens → Postings → `PartitionSink::apply(batch, epoch)`.
6. **Löschpfad:** `rewrite_without(clusters)` erzeugt neue Generation ohne Postings der Cluster. Statistik wird mit neu berechnet.
7. **Konformanz** `SegmentOwner` für Text (R2).

## 5. Tests

- **Relevanz-Regression:** Testkorpus mit bekannten Relevanzurteilen; Schwelle als Test.
- **Morphologie-Tests:** Umlaute, Komposita, Flexion, Gegenbeispiele.
- **Löschprobe:** nach `rewrite_without` liefert Suche keine Treffer des gelöschten Clusters; `verify_absent` bestätigt.
- **Seitenkanal-Test (AT-7):** Seltene Komposita dürfen über Partitionen hinweg nicht in globalen Statistiken auftauchen; Test prüft Metriken auf Leakage.
- **Fuzz-Ziel** für Tokenizer und Posting-Decoder.

## 6. Abhängigkeiten nach Umbau

Erlaubt: `contextra-ports` (`retrieval`), `contextra-types`, `bytes`, `itoa`, `regex` (nur Tokenizer-Muster, begrenzt), `unicode-segmentation`, `thiserror`.
Verboten: `bincode`, `blake3` (unkeyed), `serde*`, `futures`, `parking_lot`, `tokio`, `block_on`, `std::fs`, `memmap2`.

## 7. Gate und Reihenfolge

1. `commit_lock` und globale Schlüssel entfernen; Tests anpassen.
2. `text::contract::Epoch` → `types::Epoch`.
3. `block_on` und `inverted/purge.rs` entfernen.
4. Partitionsstatistik, dann BM25F und WAND.
5. Morphologie-Daten klären (Lizenz).
6. **G-R1**, **G-R2**, **G-R3**.

**Risiko:** Lexikon-Daten sind Lizenzfragen. Ohne Klärung dürfen sie nicht in das Repository.
