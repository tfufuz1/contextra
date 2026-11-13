# AGENTS.md — `contextra-text`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-text` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Contextra — BM25, Block-Max WAND, Inverted Index, Tokenizer and German Morphology for Hybrid Search
- **src:** 38 Dateien, 7110 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-env`, `contextra-ids`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 12):** `bincode`, `blake3`, `bytes`, `futures`, `itoa`, `parking_lot`, `regex`, `serde`, `serde_json`, `thiserror`, `tracing`, `unicode-segmentation`
- **Reverse-Abhängigkeiten (3):** `contextra-bench`, `contextra-cognition`, `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (11):** `bm25f`, `default`, `docid-128`, `experimental`, `inspect`, `ports-compat`, `serde`, `std`, `tracing`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (38): `AnalyzerPipeline`, `BM25`, `BM25F`, `BM25MorphIndex`, `Bm25CandidateStream`, `Bm25Scorer`, `BoundedTopK`, `CompactStats`, `DefaultTokenizer`, `DocId`, `DocKey`, `Epoch`, `FieldWeight`, `GermanCompoundSplitter`, `GermanMorphTokenizer` … (+23)
- enum (6): `Error`, `Language`, `NormalizationPolicy`, `OverflowPolicy`, `ParsedTextTarget`, `TextError`
- trait (10): `Analyzer`, `Clock`, `DomainVocabulary`, `Index`, `Maintain`, `MorphologicalTokenizer`, `PostingStore`, `Rng`, `TokenFilter`, `Tokenizer`
- fn (102): `as_slice`, `as_u64`, `block_max_wand_search`, `blocks`, `clear`, `commit`, `commit_lock`, `commit_stats`, `compact_terms`, `compile_protected_regex`, `contains`, `current_epoch`, `decode_compact`, `default_english`, `default_german` … (+87)
- type (4): `BoxFuture`, `DocIdRaw`, `FieldId`, `Result`
- const (11): `BLOCK_SIZE`, `BM25_B`, `BM25_K1`, `DEFAULT_STREAM_BATCH_SIZE`, `MAX_DOC_BYTES`, `MAX_DOC_TERMS`, `MAX_POSTINGS_PER_TERM`, `MAX_SEARCH_K`, `MAX_STAGED_TRANSACTIONS`, `MAX_TEXT_BYTES`, `POSTING_LIST_V2_MAGIC`
- static (4): `LEGAL_COMPOUND_STEMS`, `LEGAL_PROTECTED_TERMS`, `MEDICAL_COMPOUND_STEMS`, `MEDICAL_PROTECTED_TERMS`
- mod (22): `bm25`, `compat`, `contract`, `domain`, `error`, `inverted`, `inverted_ports`, `legal_de`, `medical_de`, `memory_index`, `morph_index`, `morphology`, `posting_list`, `purge`, `scanner` … (+7)
- pub use: `bm25::{score_term_bm25f, FieldId, FieldWeight, BM25, BM25F}`, `compound_splitter::{GermanCompoundSplitter, MorphologicalTokenizer}`, `contract::{Clock, DocKey, Epoch, Error as ContractError, Result as ContractResult, Rng}`, `crate::compat::morph_index`, `crate::compat::purge`, `crate::compat::scanner`, `crate::compat::scorer::Bm25Scorer`, `domain::{DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary}`, `error::{Result as LocalResult, TextError}`, `index_struct::InvertedIndex` … (+22)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
