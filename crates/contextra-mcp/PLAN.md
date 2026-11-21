# contextra-mcp — Implementierungsspezifikation

**Ebene:** E3 (Adapter)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
MCP-Server über stdio: Tool-Schemas aus den Fassadentypen, Principals aus Konfiguration (ADR-38), Routing, Validierung, Protokoll (SPEC 15, 3.2). Ziel-Umfang: 2.500 Zeilen (SPEC 3.1). Ist: 7.815 Zeilen laut Zielspalte (Altbestand).
Abhängigkeiten nach Anhang E: `contextra`, `ports`, `privacy`.

## 2. Vertrag
Implementiert keine Ports. Konsumiert die Fassade `contextra` (öffentliche API nach 6.8) und den Egress-Guard aus `privacy`. Binary: `src/bin/contextra-mcp-server.rs`.

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `contextra`, `ports`, `privacy`.
**Ist (Cargo.toml):** `contextra`, `contextra-types` (**verboten**, R1, REM-30, 10 Dateien), `contextra-ports`, `contextra-rank` (**nicht erlaubt**), `contextra-crypto` (**nicht erlaubt**), `contextra-privacy`.
**Ist-Abweichung:** `rank` und `crypto` dürfen von `mcp` nicht direkt verwendet werden. Der Zugriff läuft über die Fassade.
**Features:** Ist `default`, `test-utils`. Soll: keine Features außer SPEC 15. `test-utils` ist in SPEC 15 nicht genannt.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/bin/contextra-mcp-server.rs` | Einstieg | Soll |
| `src/server.rs`, `src/server_dispatch.rs`, `src/server_tools.rs`, `src/protocol.rs` | Server, Protokoll | Soll |
| `src/routing.rs`, `src/validation.rs`, `src/config.rs` | Routing, Prüfung, Konfiguration | Soll (ADR-38) |
| `src/tools_crud.rs` | Tool-Implementierung | zu prüfen gegen SPEC 15 (Tool-Schemas) |
| `src/egress_gateway.rs`, `src/egress_guard.rs`, `src/bulk_exfiltration_detector.rs` | Egress-Logik | **Doppelung mit `contextra-privacy`** (dort `egress_guard`, `bulk_exfiltration_detector`). Entfernen oder in privacy nutzen (REM-21, REM-22 prüfen) |
| `src/prompt_injection.rs`, `src/sandbox.rs`, `src/explain.rs`, `src/io.rs` | Hilfen | nicht in SPEC 15 genannt, offen |
| `src/tests.rs` | Tests im Produktcode | Verbleib offen |
| `src/lib.rs` | Wurzel | Soll |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-21, REM-22 | in SPEC 15 genannt, Inhalt in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| Doppelung Egress | `egress_guard.rs`, `egress_gateway.rs`, `bulk_exfiltration_detector.rs` | `grep -rln "egress_guard" crates/contextra-mcp` nach Entscheidung leer |
| REM-30 | Import `contextra-types` (10 Dateien) | `grep -rln "contextra_types" crates/contextra-mcp` leer |
| REM-14 | ANCHOR-Dateien (27) | `grep -rl "ANCHOR\[" crates/contextra-mcp` leer |

## 6. Umbau
1. Abhängigkeiten `contextra-rank` und `contextra-crypto` entfernen. Benötigte Funktionen über die Fassade beziehen.
2. `contextra-types` entfernen (REM-30).
3. Egress-Doppelung auflösen (Entscheidung, siehe O-1).
4. `unwrap`/`expect`/`panic!` (113 Treffer) nach R6 abbauen.
5. Testschalter `test-utils` prüfen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** Egress-Logik: in `mcp` behalten oder nur über `privacy` nutzen?
- **O-2** `prompt_injection.rs`, `sandbox.rs`: nicht in SPEC 15.
- **O-3** Feature `test-utils`: entfernen oder in SPEC 15 aufnehmen.
- **O-4** Inhalt von REM-21 und REM-22 nach SPEC 12 bestätigen.

## 7. Tests
T-SEC-04, Schema-Roundtrip (SPEC 15). Ist: `src/tests.rs`. Zählung nicht erfolgt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-03 | `rank`, `crypto` nicht erlaubt; `types` verboten | verletzt |
| G-07 | Import `contextra_types` (10) | verletzt (REM-30) |
| G-09 | 27 Dateien mit ANCHOR | verletzt (REM-14) |
| G-13 | 113 Treffer | Abbau nach R6 |
