# BASELINE.md — `contextra-text` Baseline-Befund

**Datum:** 2026-11-11
**Commit:** HEAD (882aada72)
**Rust-Version:** 1.89.0 (`rust-toolchain.toml`)

## 1. Ausgangsmessungen

| Befehl | Ergebnis | Dauer | Bemerkung |
|---|---|---|---|
| `cargo check -p contextra-text --all-targets --locked` | GRÜN | ~1s | 0 Fehler, Warnings bzgl. deprecated TextScanner/TextPurger in Tests |
| `cargo test -p contextra-text --locked` | GRÜN (1 Flaky Test bei parallelem Gesamtlauf) | ~87s | `test_high_frequency_term_resident_index_performance` einzeln/wiederholt GRÜN |
| `cargo check -p contextra-text --no-default-features --locked` | GRÜN | ~0.3s | Autarker Kern kompiliert fehlerfrei |
| `cargo test -p contextra-text --test golden_oracle_tests` | GRÜN | 0.06s | 4 passed, 1 ignored (generate_golden_corpus), SHA256-Manifest verifiziert |
| `cargo check --workspace --lib --bins --locked --keep-going` | 3211 Fehler-Zeilen in `/tmp/ws_lib_baseline.txt` | ~30s | Externe Baseline (bekannte rote Targets laut BUILD_REPARATUR.md) |
| `cargo check --workspace --all-targets --locked --keep-going` | 23 sortierte Fehlerkategorien in `/tmp/ws_all_baseline.txt` | ~45s | Externe Baseline |

## 2. Brücken- und Modularitäts-Baseline

- `tools/count_bridge_refs.py -c contextra-text`: 16 direkte Bridge-Refs außerhalb `compat/` (Hotspots: `src/inverted/tests/mock.rs`, `search_tests.rs`, `tx_tests.rs`).
- `tools/gate_layering.py --crate contextra-text`: GRÜN (Exit 0).
- `tools/zero_panic_analyzer.py --crate contextra-text`: 0 Prod-Panics.
- `tools/unsafe_boundary_auditor.py --crate contextra-text`: 5 Unsafe-Vorkommen im Repo-Suchlauf (in `tests/alloc_profiler.rs`, im Kern `#![forbid(unsafe_code)]` strikt gesetzt).

## 3. Golden Files

- `tests/golden/MANIFEST.sha256`:
  - `bm25_ranking.json`
  - `clock_reference.json`
  - `doc_id.bin`
  - `doc_id.json`
  - `doc_key.bin`
  - `posting_list_v2.bin`
  - `tokenizer_default.json`
  - `tokenizer_german.json`
Alle SHA256-Prüfsummen und Invarianten stimmen überein.
