Module: tokenizer, morphology (compound_splitter, trie, stopwords, passthrough, metrics), inverted (mit scanner, purge, morph_index), bm25, bm25f (Feature), wand, posting_list, memory_index, erasure, contract, domain (medical_de, legal_de) und Datendateien in `data/` und `src/data/`.

Deine Stärken: BM25-Referenzwerte, WAND-Korrektheit (Differenztests in `wand_differential.rs`), Kompositazerlegung mit Fehlerbalance (Precision gegen Recall), Umlaut-Behandlung (ä/ö/ü/ß, ae/oe/ue/ss).

Harte Regeln:
- Snapshot-Isolation: Leser sehen unveränderliche IndexSnapshots. Schreiber blockieren Leser nicht länger als O(1).
- Löschung läuft über Tombstones. Physische Freigabe ausschließlich über `compact`.
- Jede Änderung an Scoring braucht Referenzwerte (`bm25_reference_values.rs`) und Property-Tests.
- Morphologie-Änderungen brauchen Qualitätstests (`domain_vocab_quality.rs`) mit Vorher-Nachher-Vergleich.
- Fuzz-Targets (`fuzz_bm25_tokenize`, `fuzz_tokenizer_smoke`) nach jeder Tokenizer-Änderung ausführen.

Verifikation: `cargo test -p contextra-text --locked`, `cargo test -p contextra-text --all-features`, `cargo bench -p contextra-text`.

Optimierungsfokus: Posting-Kompression, Skip-Listen, Block-Größen für WAND, Stoppwort-Lookup per Hash oder Trie.
