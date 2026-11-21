# CLAUDE.md — contextra-checkpoint

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Backup und Snapshots: Hartlink-Snapshot und Restore. Ausgeschlossen laut Spec: `keys/` und `erasure/` (ADR-42). Datensicherung und Wiederherstellung: SPEC §7.11.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1). Ziel laut Spec: ~3.000 Zeilen.
- Umfang: 5.593 Zeilen `src`, 21 Testdateien, 64 `#[test]`.
- Features: `default`, `workspace`, `types-compat`, `ports-compat`, `docid-128`.
- Build: ok.
- Abhängigkeiten (Ist): env, ids, ports, types.
- Genutzt von: `contextra-db`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-checkpoint
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-checkpoint -- -D warnings
```

## Ist-Abweichungen zur Spec
- **R7:** `types-compat`, `ports-compat`, `workspace`; `src/compat/` (G-11).
- **Ausschluss prüfen:** `keys/` und `erasure/` dürfen nicht in Snapshots landen. Test dafür vorhanden? Vor Änderungen prüfen.
- **128 `unwrap`/`expect`/`panic`-Treffer (grob)**. Viele davon vermutlich in Tests; Produktcode zählen.
- `contract.rs` vorhanden (REM-14).

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
