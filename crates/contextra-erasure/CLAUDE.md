# CLAUDE.md — contextra-erasure

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).
> **Sicherheitskritisch:** Löschung ist ein Compliance-Pfad. Änderungen nur mit Tests und klarer Notiz.

## Rolle
Löschkern: Löschkette (Saga), ReadGate, Tresor, Messung, Schichtschnittstelle, Scrub-Planung. Spec: **Kapitel 7** (Löschkette v4.1) und Kapitel 4 (Entkopplung von Löschung und Versionierung, Seal-Modell).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1, „neu aufgebaut“, REM-10).
- Umfang: 5.687 Zeilen `src`, 4 Testdateien, 36 `#[test]`.
- Features: `default`.
- Build: ok.
- Genutzt von: `contextra-db`, `contextra-store`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-erasure
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-erasure -- -D warnings
```

## Hinweise
- SPEC §11.1 definiert Löschtests. Die Testabdeckung hier (36 Tests) ist im Vergleich zum Umfang dünn. Vor Erweiterungen Testmatrix prüfen (`tools/test_coverage_matrix.py`).
- Ehrliche Grenzen und Restfenster stehen in SPEC §7.9. Nicht schönen.
- Ist-Bestand: 42 `unwrap`/`expect`/`panic`-Treffer (grob), 3 `Instant`/`SystemTime` (G-12 verletzt, Zeit sollte über `Clock` laufen).
- Keine Löschung ohne Ledger-Eintrag (§7.10). Keine Formulierung wie „unwiderruflich“ oder „DSGVO-konform“ ohne Messwert (REQ-DOC-2).

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
