# Baseline-Bericht contextra-text (S-32)

## 1. Messungen Vorher (HEAD ca103a3c7)

- **Referenzen auf `types|ports|testkit` außerhalb `compat/` (G-D):** 67
  - `wand.rs`: 20
  - `error.rs`: 15
  - `inverted/tests/mock.rs`: 10
  - `types.rs`: 8
  - `lib.rs`: 4 (im Test-Modul)
  - `inverted/tests/search_tests.rs`: 4
  - `inverted/index_struct.rs`: 2
  - `inverted/tests/tx_tests.rs`: 2
  - `stream.rs`: 1
  - `posting_list.rs`: 1
- **Referenzen innerhalb `compat/`:** 16 (in `scorer.rs` (6), `purge.rs` (3), `scanner.rs` (3), `morph_index.rs` (2), `storage.rs` (2))
- **Gesamt-Referenzen in `src/`:** 83

## 2. Nachweis des Hohlkerns

In `crates/contextra-text/src/lib.rs` sind die zentralen Module:
- `pub mod inverted;`
- `pub mod stream;`
- `pub mod wand;`
- `pub mod compat;`
mit `#[cfg(feature = "types-compat")]` gegatet.

Bei `cargo doc -p contextra-text --no-default-features --no-deps` erzeugt rustdoc 4 Warnungen über unaufgelöste Links:
- `Bm25Scorer`
- `TextIndex`
- `InvertedIndex`
- `Bm25CandidateStream`
Ohne Default-Features fehlen dem Crate die Hauptfunktionalitäten des Invertierten Index und der WAND-Suche.

## 3. Kompilierungs- und Teststatus

- `cargo check -p contextra-text --no-default-features`: GRÜN
- Feature-Matrix: alle 10 Features (`std`, `serde`, `tracing`, `inspect`, `experimental`, `workspace`, `types-compat`, `ports-compat`, `bm25f`, `docid-128`) bauen mit `--no-default-features --features <f>` sauber.
- `cargo test -p contextra-text --no-default-features`: GRÜN (75 unittests, 1 doctest, alle Integrationstests).
- `cargo test -p contextra-text` (mit Default-Features):
  - Unit- und Integrationstests laufen.
- Stand Workspace:
  - `cargo check --workspace --all-targets --keep-going`: 2 Target-Fehler in `contextra-store` (`MemtableScanner.runtime` ungenutztes Feld unter lib und lib test).
  - Konsumentenprüfung: `contextra-engine`, `contextra-agent` nutzen `contextra-text`.
