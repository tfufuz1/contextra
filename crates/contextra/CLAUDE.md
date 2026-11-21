# CLAUDE.md — contextra (Fassade)

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Öffentliche Fassade und API: Konfigurationsladen, Builder, Agent-Gedächtnis, Query-Rewriter-Anbindung, Performance-Profile. SPEC §6.8 (Fassade, Fehler, Schnittstellen). Die Crate-Verträge sind in SPEC §15.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E2** (SPEC §3.1). Ziel laut Spec: ~1.500 Zeilen.
- Umfang: 2.040 Zeilen `src` (Ist), 14 Testdateien, 16 `#[test]`.
- Features: `default`, `embedded`.
- **Build: BROKEN.** `cargo check -p contextra` → 20 Fehler (E0425, E0432, E0433).
- Abhängigkeiten (Ist): `contextra-db`, `contextra-ports`, `contextra-types`.
- Genutzt von: `contextra-mcp` (bricht damit ebenfalls).

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo check -p contextra      # aktuell rot
RUSTUP_TOOLCHAIN=stable cargo check -p contextra-db   # Ursache liegt dort
```

## Fehlerursache
Typen werden aus `contextra_db` importiert, die dort fehlen. Betroffene Dateien:
- `src/agent_memory.rs`, `src/builder.rs`, `src/lib.rs`, `src/query_rewriter.rs`, `src/performance_profile.rs`.
- Außerdem Warnung: `contextra-license` wurde entfernt, `LicenseGate` wird noch referenziert (`performance_profile.rs`). Die Lizenz-Seams (S-50/S-64) sind nicht entschieden.

Vor Reparatur: Entscheidung, ob `contextra-db` die API liefern soll (Ziel) oder die Fassade vorübergehend gestubbt wird. Nicht still auf dem Ist-Stand „reparieren“.

## Ist-Abweichungen zur Spec
- Umfang 2.040 vs. Ziel 1.500.
- 20 `ANCHOR`/`FILE-CONTEXT`-Dateien (REM-14).
- Der Name `contextra-engine` steht noch in Kommentaren (REM-19, Crate entfernt).

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
