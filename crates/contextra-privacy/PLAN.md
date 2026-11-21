# contextra-privacy — Implementierungsspezifikation

**Ebene:** E1 (Kern)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Datenschutz: Entity-Erkennung als Warnung (`entity_recognizer`: Muster, Prüfziffern, Listen), `data_class`, Egress-Guard (`egress_guard`, fail-closed), Verarbeitungsregister (`processing_registry`), `guarded_payload` (SPEC 15, 3.1, REQ-PRV-3, REQ-PRV-4). Ziel-Umfang: 3.500 Zeilen (SPEC 3.1). Ist: 9.278 Zeilen laut Zielspalte (Altbestand).
Abhängigkeiten nach Anhang E: `ids`, `env`. Das Crate hat keine Abhängigkeit auf `ports` (Anhang E).

## 2. Vertrag
Keine Port-Implementierung nach SPEC 3.4. Egress-Guard wird von `contextra-mcp` und `contextra` genutzt (SPEC 15, 6.9). Der Egress-Guard ist fail-closed (SPEC 15).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ids`, `env`.
**Ist (Cargo.toml):** `contextra-ids`, `contextra-env` (erlaubt), dazu `tokio` (**normale Abhängigkeit**, in `deny.toml` als Wrapper eingetragen, Übergang E29), `scc`, `rand`, `regex`, `zeroize`, `serde`, `serde_json`, `thiserror`, `tracing`, `parking_lot`, `blake3`.
**Ist-Abweichung:** `tokio` als normale Abhängigkeit eines E1-Crates. Das ist Teil von Punkt 10 der CLAUDE.md. Durch `deny.toml` gedeckt, aber nicht durch SPEC 15 gefordert.
**Features:** `default = [\"std\"]`. Ist-Abweichung: `std` ist nicht R3-begründet (nur `durable-fs`, `simd`, `rank` sind genannt, SPEC 3.2). Verbleib offen.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/entity_recognizer.rs` | Erkennung | Soll |
| `src/data_class.rs` | Datenklassen | Soll |
| `src/egress_guard.rs` | Guard | Soll (fail-closed) |
| `src/processing_registry.rs` | Verarbeitungsregister | Soll |
| `src/guarded_payload.rs` | geschützte Nutzlast | Soll |
| `src/egress_gateway.rs`, `src/egress_vault.rs` | Gateway, Tresor für Egress | nicht in SPEC 15 genannt, offen (`vault` nach REM-01 prüfen) |
| `src/audit_trace.rs`, `src/context_edit_audit.rs` | Audit-Spuren | nicht in SPEC 15 genannt, offen |
| `src/bulk_exfiltration_detector.rs` | Erkennung von Massenabfluss | nicht in SPEC 15 genannt, offen |
| `src/types.rs` | Typen | zu prüfen |
| `src/contract.rs` | Selbstdeklaration | REM-14 |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-20 | in SPEC 15 genannt, Inhalt in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| REM-14 | ANCHOR-Dateien (21), `contract.rs` | `grep -rl "ANCHOR\[" crates/contextra-privacy` leer |

## 6. Umbau
1. `egress_gateway.rs`, `egress_vault.rs` und Audit-Module nach SPEC-Zuordnung prüfen (Soll offen).
2. Abhängigkeit `tokio` klären (Punkt 10 der CLAUDE.md), siehe Offene Punkte.
3. `unwrap`/`expect`/`panic!` (39 Treffer) nach R6 abbauen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** `tokio` in Produktcode eines E1-Crates: entfernen oder Wrapper behalten (Punkt 10 CLAUDE.md).
- **O-2** Module `egress_gateway`, `egress_vault`, `audit_trace`, `context_edit_audit`, `bulk_exfiltration_detector`: nicht in SPEC 15. Verbleib entscheiden.
- **O-3** `std`-Feature: R3-Begründung fehlt.
- **O-4** Inhalt von REM-20 nach SPEC 12 bestätigen.

## 7. Tests
- Erkennungsgrenzen, Guard fail-closed (SPEC 15).
- Ist: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | kein `unsafe`, `forbid` in `lib.rs` | eingehalten |
| G-05 | `tokio` im normalen Baum (kein Netzwerk-Crate der Sperrliste) | Sperrliste nicht betroffen; `tokio` offen (Punkt 10) |
| G-09 | 21 Dateien mit ANCHOR, `contract.rs` | verletzt (REM-14) |
| G-12 | 2 Treffer | verletzt |
| G-13 | 39 Treffer | Abbau nach R6 |
