# CLAUDE.md — contextra-mvcc

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Multi-Version Concurrency Control: Sequenz-Log, Transaktionspuffer, Versionskatalog. **Ist-Name `contextra-mvcc`.** Laut Spec (ADR-06) wird das Crate zu **`contextra-version`** umbenannt, mit Versionskatalog, Snapshots und Aufbewahrung.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1, ADR-06).
- Umfang: 4.000 Zeilen `src`, 24 Testdateien, 127 `#[test]`.
- Features: `default`, `workspace`, `types-compat`, `ports-compat`, `docid-128`.
- Build: ok.
- Genutzt von: `contextra-db`, `contextra-store`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-mvcc
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-mvcc -- -D warnings
```

## Ist-Abweichungen zur Spec
- **Umbenennung ausstehend** (ADR-06). Vorher mit dem Nutzer klären, weil der Name in `Cargo.toml` (Workspace und Dependents) überall steht.
- **R7 verletzt:** Features `types-compat` und `ports-compat`, und das Verzeichnis `src/compat` (G-11).
- **`contract.rs`** vorhanden (REM-14, entfällt).
- 21 `unwrap`/`expect`/`panic`-Treffer (grob) und 32 `ANCHOR`/`FILE-CONTEXT`-Dateien.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
