# CLAUDE.md — contextra-ids

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Kanonische Identitäts- und Tenant-Typen. Laut Spec (ADR-33, ADR-52) die **einzige Definition von Limits** (`MAX_SEARCH_K` etc. gehören hierher) und Aufnahmeort der Typen aus `contextra-types` (REM-30).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E0** (SPEC §3.1). Keine internen Abhängigkeiten.
- Umfang: 1.181 Zeilen `src`, 5 Testdateien, 35 `#[test]`.
- Features: `default`, `std`, `docid-128`.
- Build: `cargo check -p contextra-ids` ok.
- Genutzt von (Ist): env, types, checkpoint, crypto, erasure, graph, mvcc, privacy, store, text, vector.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-ids
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-ids --no-default-features   # G-01
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-ids -- -D warnings
```

## Hinweise
- Änderungen hier wirken auf fast alle Crates. Vorher Dependents prüfen (Abschnitt Eckdaten).
- R4: Jeder Typ genau einmal definiert. Keine Duplikate aus `types` übernehmen, ohne sie dort zu entfernen.
- R7: Keine neuen Features mit `compat`, `workspace` oder `types-compat` im Namen.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
