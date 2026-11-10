# AGENTS.md — `contextra-text`

> Ring 0 / Klasse A · Unsafe: verboten (`#![forbid(unsafe_code)]`) · Rolle: Inverted Index, BM25/BM25F, Block-Max-WAND, Tokenizer, Morphologie.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-text --no-default-features --locked` |
| Contextra-Abhängigkeiten (Kern) | Keine direkten Abhängigkeiten im Kern (`PostingStore` entkoppelt) |
| Kompatibilitätsschicht | `compat/` (hinter `types-compat` / `ports-compat` / `workspace`) |
| Features | `std`, `serde`, `tracing`, `inspect`, `experimental`, `workspace`, `types-compat`, `ports-compat`, `bm25f`, `docid-128` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Modul-Karte

| Pfad | Verantwortung |
|---|---|
| `storage.rs` | Lokales `PostingStore`-Trait (11 exakt genutzte Kern-Methoden) |
| `error.rs` | Lokaler `TextError` (thiserror) und Result-Typ |
| `types.rs`, `contract.rs`, `traits.rs` | Autarke Identifikatoren (`DocId`, `DocKey`), Traits (`Index`, `Maintain`, `Clock`, `Rng`) |
| `bm25.rs`, `wand.rs`, `posting_list.rs`, `stream.rs` | Scoring, Block-Max-WAND, V2-Delta-Varint-Postings, Kandidatenstrom über `PostingStore` |
| `tokenizer.rs`, `morphology/`, `domain/` | Analyse, Komposita, Domänenvokabular |
| `memory_index.rs` | In-Memory MVCC Volltextindex |
| `compat/` (`storage`, `morph_index`, `purge`, `scanner`, `scorer`) | Ports-Brücke (`StorageEngine -> PostingStore`), `Bm25Scorer`, `BM25MorphIndex`, `TextPurger`, `TextScanner` |

## 3. Verträge & Invarianten

1. **Statistik je Tenant** (IDF, Dokumentlängen): keine mandantenübergreifende Posting-Liste oder Statistik.
2. **BM25-Referenzwerte** sind Test-Orakel (`tests/bm25_reference_values.rs`, `tests/golden_oracle_tests.rs`).
3. **Format- und Bytegleichheit (INV-1)**: V2 Delta-Varint-Kodierung bleibt unverändert und byteweise gesichert.
4. **Determinismus (INV-5)**: Gleiche Eingabe liefert identische Rangfolge, stabiler Tie-Break nach ID aufsteigend.
5. **Zero-Panic**: Volle Fehlerbehandlung via `TextError`. Keine Panics im Produktionspfad.
