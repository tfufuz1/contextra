# CLAUDE.md — contextra-privacy

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Datenschutz am Ausgang: Egress-Guard (keine sensiblen Daten verlassen das Haus ungeschützt), DLP-Erkennung (IBAN, Kreditkarte, Steuer-/Ausweisnummern, E-Mail, Telefon), Platzhalter-Ersetzung und verlustfreie Wiederherstellung, Verarbeitungsregister. SPEC §6.9, REQ-PRV-3/4, REM-20.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1). Ziel laut Spec: ~3.500 Zeilen.
- Umfang: 5.726 Zeilen `src`, 14 Testdateien, 115 `#[test]`.
- Features: `default`, `std`, `serde`, `tracing`.
- Build: ok.
- Abhängigkeiten (Ist): env, ids.
- Genutzt von: `contextra-db`, `contextra-mcp`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-privacy
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-privacy -- -D warnings
```

## Hinweise
- Egress-Guard ist sicherheitsrelevant. Jede Änderung an Erkennungsregeln mit Gegenbeispielen testen (falsch-positive und falsch-negative Fälle).
- Entity-Erkennung ist laut Spec nur Warnung, keine Garantie. Doku entsprechend formulieren (REQ-DOC-2).
- Vor Bestätigung bestimmter Werte: keine Zusicherungen wie „DSGVO-konform“ ohne Messung.

## Ist-Abweichungen zur Spec
- 39 `unwrap`/`expect`/`panic`-Treffer (grob), 2 `Instant`/`SystemTime` außerhalb `env` (G-12).
- `contract.rs` vorhanden (REM-14), 21 Dateien mit Selbstdeklarationen.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
