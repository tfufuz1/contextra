# Spezifikation: `contextra-text` (Klasse A / SPEC v4.0 §11.3)

## 1. Zweck und Baukasten-Profil
`contextra-text` ist ein autarker Klasse-A-Baustein für Volltextindexierung, morphologische Textanalyse und BM25/BM25F-Relevanzbewertung mit Copy-on-Write Snapshot-Isolation und Block-Max-WAND-Pruning.

* **Klasse:** A (Autarker Baustein, 0 interne Pflicht-Abhängigkeiten im Standalone-Build)
* **Ring:** Ring 0 / Ring 1
* **Lizenz:** MIT OR Apache-2.0
* **Safety:** `#![forbid(unsafe_code)]`
* **Rust-Version:** 1.89

## 2. Modulstruktur
- `contract`: Eingefrorener Vertrag (`Error`, `Result`, `DocKey`, `Epoch`, `Clock`, `Rng`).
- `traits`: Normative Schnittstellen gemäß SPEC §4.3 (`Index<DocKey, &str>` und `Maintain`).
- `memory_index`: Autarker, thread-sicherer In-Memory-Index (`MemoryTextIndex`, `IndexSnapshot`) mit Copy-on-Write Snapshot-Isolation.
- `error`: Typisierter Fehlertyp `TextError` (`#[non_exhaustive]`) und `Result<T>`.
- `types`: Autarke Basistypen (`DocId`, `TextHit`, `MAX_SEARCH_K`).
- `tokenizer`: Unicode-Wortsegmentierung, Lowercasing, Stoppwortfilterung und Schutz atomarer Token.
- `morphology`: Deutsche Sprachmorphologie (Komposita-Zerlegung, Umlaut-Normalisierung).
- `domain`: Domänenspezifische Vokabulare für Recht und Medizin.
- `bm25`: Mathematische BM25- und BM25F-Relevanzberechnung.
- `posting_list`: Delta-VByte komprimierte Posting-Listen.
- `wand`: Block-Max-WAND dynamisches Pruning.
- `inverted`: Persistenter Inverted-Index (hinter `ports-compat` / `types-compat` für Workspace-Integration).

## 3. Invariantentabelle mit Test-Zuordnung

| Invariante | Beschreibung | Verifikations-Test |
|---|---|---|
| `INV-TEXT-1` | Snapshot-Konsistenz: Lesezugriffe sehen konsistente Wörterbücher, Postinglisten und Korpusstatistiken (`N`, `avgdl`, `df`). | `memory_index::tests::test_memory_text_index_crud_and_compact`, `tests/snapshot_concurrency_test.rs` |
| `INV-TEXT-WAND-1` | Block-Max WAND dynamisches Pruning liefert exakt dieselben Top-K Treffer wie Brute-Force-Scoring. | `tests/wand_differential.rs`, `tests/wand_tie_break_threshold.rs` |
| `INV-ZERO-PANIC` | Kein `unwrap()`, `expect()`, `panic!` im Produktionspfad; Arithmetik checked/saturating. | `tools/zero_panic_analyzer.py`, `tests/bm25_edge_cases.rs` |
| `INV-DETERMINISM` | Deterministische Sortierung von Suchergebnissen (Score absteigend, DocId aufsteigend). | `tests/bm25_candidate_stream.rs`, `tests/wand_tie_break_threshold.rs` |
| `INV-PURGE-ISOLATION` | Neutrale Löschung per Tombstone; physische Bereinigung nur über `Maintain::compact()`; keine DSGVO/Witness-Typen im Kern. | `memory_index::tests::test_memory_text_index_crud_and_compact`, `tests/tombstone_read_your_writes.rs` |
| `INV-FSYNC-1` | Fail-closed Kapazitäts- und Eingabegrenzen (`MAX_DOC_BYTES`, `MAX_SEARCH_K`). | `tests/tx_tests.rs::test_upsert_document_oversized_text_returns_error` |

## 4. Garantien & Nicht-Ziele
- **Garantien:** Deterministische BM25-Scores, Copy-on-Write Snapshot-Isolation ohne Leser-Blockierung von Schreibern, sublineare WAND-Traversierung.
- **Nicht-Ziele:** Keine mandantenübergreifenden Sagas oder DSGVO-Zertifikate im Bausteinkern (Auslagerung nach `contextra-erasure-adapters`).
