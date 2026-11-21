# CLAUDE.md — contextra-text

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Textsuche: BM25 (mit Feld-Gewichtung BM25F), Block-Max WAND, invertierter Index, Tokenizer, deutsche Morphologie (Stemmer, Stoppwörter, Komposita-Zerlegung). SPEC §6.4.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1). Ziel laut Spec: ~10.000 Zeilen.
- Umfang: 9.535 Zeilen `src`, 32 Testdateien, 160 `#[test]`.
- Features: `default`, `std`, `serde`, `tracing`, `inspect`, `experimental`, `workspace`, `types-compat`, `ports-compat`, `bm25f`, `docid-128`.
- Build: ok.
- Abhängigkeiten (Ist): env, ids, ports, types.
- Genutzt von: `contextra-db`, `contextra-bench`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-text
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-text --no-default-features   # G-01
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-text -- -D warnings
```

## Ist-Abweichungen zur Spec
- **R7 verletzt:** `types-compat`, `ports-compat`, `workspace`; `src/compat/`-Verzeichnis (G-11).
- **G-12 verletzt:** 4 `Instant`/`SystemTime`-Aufrufe außerhalb `contextra-env`.
- **`unsafe`:** Nur in einer Testdatei (`alloc_profiler.rs`, laut Anhang B). Gilt als Test-Ausnahme nach ADR-53, Eintrag in `tests/unsafe_islands.rs` prüfen.
- 46 `unwrap`/`expect`/`panic`-Treffer (grob).
- Deutsche Sprachregeln („Straße“/„Strasse“, Komposita) sind Produktversprechen (README). Änderungen an Tokenizer mit Tests belegen.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
