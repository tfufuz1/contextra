# Übergabebericht MISSION `TEXT-2`: Relevanz & Pruning: BM25/BM25F, Posting-Listen, WAND

## 1. Übersicht & Zielerreichung
Mission `TEXT-2` stellt das mathematisch exakte Scoring (BM25, BM25F), die defensive Dekodierung und Delta-Varint-Kompression für Posting-Listen sowie das dynamische Pruning via Block-Max WAND und iteratives Candidate Streaming im Crate `contextra-text` (Klasse A) her.

## 2. Soll/Ist-Vergleich
- **Scoring (BM25 / BM25F):**
  - Robertson-Spärck-Jones IDF-Formel $\ln(1 + \frac{N - n + 0.5}{n + 0.5})$ garantiert $\ge 0$ für alle $df \le N$.
  - Längennormalisierung und Grenzfallbehandlung bei $N = 0$, leeren Feldern, $df = 0$, $tf = 0$ und $avgdl = 0$ liefert stets finite, nicht-negative Werte ohne NaN oder Infinity.
  - Parameter $k_1 \ge 0.0$, $b \in [0.0, 1.0]$ und Feldgewichte werden strikt validiert.
- **Posting-Listen:**
  - Delta-Codierung + VByte (Varint) mit 64er-Block-Max-Metadaten (`PostingBlockInfo` mit `max_doc_id`, `max_tf`, `min_doc_len`).
  - Defensiver Decoder (`decode_compact`): Weist unvollständige, abgeschnittene, überlange Daten, ungültige VByte-Shifts, Puffer-Überläufe sowie nicht-monoton fallende oder duplizierte DocId-Deltas mit `Error::Storage` ab.
  - Round-Trip-Invariante `decode(encode(x)) == x` erfüllt und verifiziert.
- **WAND:**
  - `INV-TEXT-WAND-1` sichergestellt: Liefert exakt dieselben Top-K wie die vollständige Auswertung.
  - Totalordnung und deterministischer Tie-Break (Score absteigend, DocId aufsteigend).
  - Keine Unterschätzung der Schwellenwerte bei Gleitkomma-Berechnungen.
- **Streaming:**
  - `Bm25CandidateStream` bietet lazy Kandidatenabruf (`next_batch`, `next_doc`) ohne Vorab-Materialisierung.
- **Determinismus & Lint-Bereinigung:**
  - P28-Bereinigung: Ersetzen von `HashMap` und `HashSet` durch `BTreeMap` und `BTreeSet` in `posting_list.rs`, `wand.rs` und `stream.rs`.
  - Zero-Panic Doctrine: Ungeprüfte Indexierungen und fehlerhafte Typ-Casts behoben.
- **Eingefrorener Crate-Vertrag:**
  - `src/contract.rs` byte-identisch angelegt und als `pub mod contract;` in `src/lib.rs` eingebunden.

## 3. Verifikationsergebnisse & Messwerte
- `cargo fmt --check -p contextra-text`: Grün.
- `cargo test -p contextra-text --test posting_roundtrip_tests`: 7/7 bestanden.
- `cargo test -p contextra-text --test wand_differential`: 2/2 bestanden.
- `cargo test -p contextra-text --lib`: Alle 107 Unit-Tests bestanden.
- `cargo bench -p contextra-text --no-run`: Baut fehlerfrei.
- `python3 tools/zero_panic_analyzer.py --crate contextra-text --fail-on-prod`: 0 Prod-Verstöße.
- `python3 tools/export_crate.py contextra-text --out /tmp/export-contextra-text`: Gate G3 Standalone-Export erfolgreich.

## 4. Funktionsinventar-Diff
- Keine Funktionen unbegründet entfernt (Diff-Veränderungen basieren ausschließlich auf Zeilenverschiebungen durch das Einfügen von `mod contract;` und zusätzlichen Typsicherheitsprüfungen).
- Neu hinzugekommen: Trait `contract::Clock`, `contract::Rng`, Structs `DocKey`, `Epoch`, `Error` sowie umfassende Testfunktionen in `tests/posting_roundtrip_tests.rs`.
