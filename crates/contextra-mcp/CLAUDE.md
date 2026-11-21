# CLAUDE.md — contextra-mcp

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
MCP-Server über stdio (ADR-38, SPEC §6.8/§3.2 E3). Macht Contextra für KI-Assistenten (z. B. Claude Desktop) als Werkzeug nutzbar.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E3** (SPEC §3.1). Ziel laut Spec: ~2.500 Zeilen.
- Umfang: 4.650 Zeilen `src` (Ist), 9 Testdateien, 47 `#[test]`.
- Features: `default`, `test-utils`.
- **Build: BROKEN** (transitiv). 21 Fehler, Ursache ist die Fassade `contextra` (siehe deren CLAUDE.md).
- Abhängigkeiten (Ist): `contextra`, `contextra-crypto`, `contextra-ports`, `contextra-privacy`, `contextra-rank`, `contextra-types`.
- Genutzt von: niemand im Workspace.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo check -p contextra-mcp   # aktuell rot, bis contextra gefixt ist
```

## Ist-Abweichungen zur Spec
- `Cargo.toml` hat **keine `description`**. Ergänzen, wenn das Crate angefasst wird.
- 113 `unwrap`/`expect`/`panic`-Treffer (grob) und 26 `ANCHOR`/`FILE-CONTEXT`-Dateien.
- Stdio-Protokoll: Keine Logs auf stdout, sonst bricht der MCP-Kanal. Nur stderr oder Dateien.
- Netzwerk ist im Kern verboten (R8). Dieses Crate ist E3, bleibt aber netzfrei, solange nicht als Feature freigegeben.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
