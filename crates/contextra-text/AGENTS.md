# AGENTS.md — contextra-text
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.29

## 1. Zweck
Lexikalische Volltextsuch-Engine des Contextra-Systems (Signal 2 der 4-Signal-Fusion). Implementiert transaktionale Inverted Indices, BM25/BM25F-Scoring, Block-Max WAND (BMW) Abfrageoptimierung sowie eine morphologische Tokenisierungs-Pipeline für die DACH-Region (`GermanCompoundSplitter`). Implementiert `TextIndex` aus `contextra-ports`.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | Modul-Deklaration, `#![forbid(unsafe_code)]` Crate-Boundary |
| `bm25.rs` | `BM25` Scorer — Score-Berechnung (IDF, TF, doc_len) |
| `tokenizer.rs` | `Tokenizer` Trait, `DefaultTokenizer`, `GermanMorphTokenizer` |
| `morphology/` | `GermanCompoundSplitter`, Umlaut-Normalisierung, `Trie`, `stopwords.rs`, `passthrough.rs`, `metrics.rs` |
| `posting_list.rs` | Postings-Listen-Strukturen und Delta-Dekodierung |
| `wand.rs` | Block-Max WAND (BMW) Query-Execution & Block-Max Score Bounds |
| `stream.rs` | Term- & Dokument-Streaming-Iteratoren für Inverted Index Search |
| `inverted/` | Transaktionale Inverted Index Kernstrukturen (`index_struct.rs`, `morph_index.rs`, `types.rs`) |
| `domain/` | Domänenspezifische Vokabulare und Tokenizer (`legal_de.rs`, `medical_de.rs`) |
| `data/german_words.txt` | Eingebettetes Wörterbuch für deutsche Komposita-Zerlegung (via `include_str!`) |

## 3. Invarianten

- **UTF-8 Safety:** String-Slices und Token-Grenzen dürfen NIE als Byte-Offsets auf UTF-8-Slices angewendet werden ohne Boundary-Validierung (`is_char_boundary`). Slices an falschen Byte-Grenzen führen zu Panics. (`cargo test -p contextra-text`)
- **Determinismus der Tokenisierung:** Query- und Ingestion-Pfade MÜSSEN identische Tokenisierungsketten durchlaufen. Diskrepanzen führen zu Silent Recall Drops. (`cargo test -p contextra-text tokenizer`)
- **MVCC Isolation bei Block-Max WAND:** Non-latest searches in `wand.rs` validieren Posting-Existenz unter historischen Sequenznummern (`storage.get_at_seq(&pl_doc_key, seq)`), um Term-Leaks aus Zukunftstransaktionen zu verhindern. (`cargo test -p contextra-text wand`)
- **INV-TEXT-SAFE:** 100% Safe Rust mit `#![forbid(unsafe_code)]` im Crate-Root. (`cargo xtask check-agents-integrity`)

## 4. Verboten / Anti-Patterns

```rust
// ❌ FALSCH — Byte-Offsets direkt für UTF-8 Substrings nutzen:
let token = &text[start_bytes..end_bytes];
// ✅ KORREKT — char_indices() nutzen oder is_char_boundary() vor Slice prüfen.

// ❌ FALSCH — tokio::spawn oder async runtime in contextra-text nutzen:
tokio::spawn(async move { ... });
// ✅ KORREKT — Ring 0 ist rein synchron (P26).

// ❌ FALSCH — Query-Text mit einfachem split_whitespace() parsen:
let terms = query.split_whitespace();
// ✅ KORREKT — Denselben Tokenizer wie bei Ingestion verwenden (self.tokenizer.tokenize(query)).
```

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- **Ring-0 Sync-Reinheit (P26):** `contextra-text` besitzt keine `tokio`-Abhängigkeit und führt Indexierung & Suche synchron durch.
- Inverted-Index Mutexes / Caches (z. B. DF-Counts, Avg-Doc-Len) nutzen feingranulare `parking_lot::RwLock` Sperren.
- Keine Mutexe über I/O-Grenzen halten.

## 6. Verifikation

```bash
cargo test -p contextra-text --locked
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-text
cargo xtask check-ring0-async-purity
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- `domain/legal_de.rs` und `domain/medical_de.rs` bieten domänenspezifische Spezialisierungen, die bei der Tokenizer-Konfiguration explizit ausgewählt werden müssen.
