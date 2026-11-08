# Spezifikation: `contextra-text` (Klasse A / SPEC v4.0 §11.3)

## 1. Zweck und Baukasten-Profil
`contextra-text` ist ein autarker Klasse-A-Baustein für Volltextindexierung, morphologische Textanalyse und BM25/BM25F-Relevanzbewertung mit Block-Max-WAND-Pruning.

* **Klasse:** A (Autarker Baustein, 0 interne Pflicht-Abhängigkeiten)
* **Ring:** Ring 0 / Ring 1
* **Lizenz:** MIT OR Apache-2.0
* **Safety:** `#![forbid(unsafe_code)]`

## 2. Modulstruktur
- `traits`: Normative Schnittstellen gemäß SPEC §4.3 (`Index<Id, Query>` und `Maintain`).
- `memory_index`: Autarker, speicherbasierter In-Memory-Index (`MemoryTextIndex`) für Standalone-Betrieb.
- `error`: Typisierter Fehlertyp `TextError` (`#[non_exhaustive]`) und `Result<T>`.
- `types`: Autarke Basistypen (`DocId`, `TextHit`, `MAX_SEARCH_K`).
- `tokenizer`: Unicode-Wortsegmentierung, Lowercasing, Stoppwortfilterung und Schutz atomarer Token.
- `morphology`: Deutsche Sprachmorphologie (Komposita-Zerlegung, Umlaut-Normalisierung).
- `domain`: Domänenspezifische Vokabulare für Recht und Medizin.
- `bm25`: Mathematische BM25- und BM25F-Relevanzberechnung.
- `posting_list`: Delta-VByte komprimierte Posting-Listen.
- `wand`: Block-Max-WAND dynamisches Pruning.
- `inverted`: Persistenter Inverted-Index (hinter `ports-compat` / `types-compat` für Workspace-Integration).

## 3. Invarianten
- `INV-TEXT-WAND-1`: Der WAND-Algorithmus liefert exakt dieselben Top-K-Treffer wie eine vollständige Auswertung aller Posting-Listen.
- `INV-ZERO-PANIC`: Keine Panics oder unhandled unwraps im Produktionscode.
- `INV-DETERMINISM`: Deterministische Sortierung von Suchergebnissen (Score absteigend, DocId aufsteigend).
- `INV-PURGE-ISOLATION`: Löschlogik und Kaskadierung verbleiben in `contextra-erasure`; der Kern bietet ausschließlich neutrale Wartung (`compact`).

## 4. Garantien & Nicht-Ziele
- **Garantien:** Deterministische BM25-Scores, thread-sichere Lese- und Schreibzugriffe, sublineare WAND-Traversierung.
- **Nicht-Ziele:** Keine mandantenübergreifenden Sagas oder DSGVO-Zertifikate im Bausteinkern.
