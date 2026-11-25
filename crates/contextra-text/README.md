# contextra-text

`contextra-text` stellt Volltextsuche, BM25/BM25F-Scoring und deutsche Morphologie bereit (Ring 0).

## Zweck

Indexiert Freitexte über residenten Postinglisten-Index, führt Block-Max WAND Top-k Selektion, BM25F Feldgewichtung und deutsche Kompositazerlegung durch.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Synchroner Text-Kern)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Scorer & Index:** `Bm25Scorer`, `InvertedIndex`, `ResidentPostingIndex`, `PostingList`
- **BM25 / BM25F:** `BM25`, `BM25F`, `FieldId`, `FieldWeight`
- **Tokenizer & Morphologie:** `Tokenizer`, `GermanMorphTokenizer`, `GermanCompoundSplitter`

## Architektur & Verweise

Details zur Volltextsuche finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §7.3.
