# CLAUDE.md — contextra-types

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Kanonische Domänentypen: IDs (teilweise), Budgets, Filter, Fehlertypen. Laut Spec **nicht in der Zielstruktur**: REM-30 entfernt das Crate, Inhalte gehen nach `contextra-ids` und `contextra-ports`.

## Eckdaten (Messung 21.11.2026)
- Ebene: **keine Zielebene** (REM-30). Ist-Abhängigkeit: `contextra-ids`.
- Umfang: 7.532 Zeilen `src`, 14 Testdateien, 188 `#[test]`.
- Features: `default`, `docid-128`, `auto-extraction-opt-out`.
- Build: ok.
- Genutzt von: fast allen Crates (contextra, ports, graph, mcp, rank, simd, testkit, text, vector, checkpoint, db, mvcc, bench).

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-types
```

## Hinweise
- **Entscheidung ausstehend (REM-30):** Auflösung ist ein Breaking Change für den halben Workspace. Vorher mit dem Nutzer klären, nicht eigenmächtig verschieben.
- Bis dahin: Neue Typen bevorzugt in `contextra-ids` oder `contextra-ports` anlegen, damit der Umzug einfacher wird. Nur mit Notiz.
- 133 `unwrap`/`expect`/`panic`-Treffer (grob) und 17 `ANCHOR`/`FILE-CONTEXT`-Dateien.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
