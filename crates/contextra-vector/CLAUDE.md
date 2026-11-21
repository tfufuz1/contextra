# CLAUDE.md — contextra-vector

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Vektorindex: HNSW, SQ8, exakte Suche, ACORN-artige Traversierung, Filter (SPEC §6.3). Die Entscheidung „DiskANN und RaBitQ raus“ (ADR-04) steht im Soll-Zustand.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1, ADR-04). Ziel laut Spec: ~11.000 Zeilen.
- Umfang: **18.227 Zeilen `src`**, 104 Testdateien, 188 `#[test]`.
- Features: `default`, `std`, `simd`, `serde`, `tracing`, `inspect`, `experimental`, `mmap`, `diskann`, `quantize-rabitq`, `workspace`, `ports-compat`, `docid-128`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`, `experimental-all`, `partial-index-rebuild`.
- Build: ok.
- Abhängigkeiten (Ist): durable-fs, env, ids, ports, simd, types.
- Genutzt von: `contextra-db`, `contextra-bench`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-vector
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-vector --no-default-features   # G-01
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-vector -- -D warnings
```

## Ist-Abweichungen zur Spec
- **Feature-Ballast:** `diskann`, `quantize-rabitq`, `experimental-*`, `graph`, `workspace`, `ports-compat` gehören laut Spec raus (ADR-04, R7).
- **`compat/`-Verzeichnis** vorhanden (G-11).
- **`unsafe`** (mmap, SIMD) in Übergangsphase laut `tests/unsafe_islands.rs`. Ziel: in `contextra-simd` bzw. `contextra-sys`.
- **184 Dateien** mit `ANCHOR`/`FILE-CONTEXT` (REM-14). Größter Anteil im Repo. Nicht neu erzeugen.
- 162 `unwrap`/`expect`/`panic`-Treffer (grob, inkl. Tests).
- Umfang 18.227 gegen Ziel 11.000.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
