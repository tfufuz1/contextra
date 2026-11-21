# contextra-testkit — Implementierungsspezifikation

**Ebene:** E3, nur Dev-Abhängigkeit (SPEC 15, 3.2)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Prüfwerkzeuge für Tests: FaultVfs, Canary-Scanner (T-ENC-01), Mock-Store, Mock-Uhr, Schedule-Exploration (`loom`), Mock-Embedder, Mock-Gate. Enthält die Datei mock.rs aus CMP-3 (SPEC 15, F-27). Ziel-Umfang: 7.000 Zeilen (SPEC 3.1). Ist: 7.389 Zeilen laut Zielspalte (Altbestand). Nur Dev-Abhängigkeit aller Crates außer `bench`, `server`, `kms` (Anhang E).

## 2. Vertrag
Keine Ports implementiert. Bietet Mocks und Fault-Injection für die Tests der Ziel-Crates. Ist: `src/fault_vfs.rs`, `src/in_memory_graph.rs`, `src/in_memory_store.rs`, `src/in_memory_text.rs`, `src/conformance/*` (Clock, Failure, Graph, KV-Ports, Metrics, Rng, Routing-Ports, Storage, Text, Vector, VFS).

## 3. Abhängigkeiten
**Erlaubt (Anhang E, dev):** alle außer `bench`, `server`, `kms`.
**Ist (Cargo.toml):** `contextra-ports` (erlaubt), `contextra-types` (**verboten**, REM-30, 14 Dateien).
**Features:** Ist `default`, `testkit`, `erasure-conformance`. Soll (SPEC 15): keine Features außer Kapitel 15. `erasure-conformance` ist nicht genannt, offen.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/fault_vfs.rs` | FaultVfs | Soll (SPEC 15, Absturzmatrix in `durable-fs`) |
| `src/in_memory_store.rs` | Mock-Store | Soll (Mock-Store) |
| `src/in_memory_graph.rs`, `src/in_memory_text.rs` | Mocks | Soll, zu prüfen |
| `src/conformance/*` | Konformitätsprüfungen je Port | Soll (Testwerkzeug), Umfang gegen SPEC 15 prüfen |
| `src/conformance/clock.rs`, `rng.rs` | Mock-Uhr, Zufall | Soll (Mock-Uhr). Hinweis: `Clock` liegt in `env`, Mock-Uhr nur hier |
| Canary-Scanner, Mock-Embedder, Mock-Gate | nicht gefunden | Soll, fehlt im Ist (prüfen) |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| CMP-3 | Keine Entfernung. Datei mock.rs (aus CMP-3) ist neu anzulegen | nach Umbau: `test -f crates/contextra-testkit/src/mock.rs` |
| REM-30 | Import `contextra-types` (14 Dateien) | `grep -rln "contextra_types" crates/contextra-testkit` leer |
| REM-14 | ANCHOR-Dateien (22) | `grep -rl "ANCHOR\[" crates/contextra-testkit` leer |
| Zeitquelle | `Instant::now` in 1 Datei außerhalb `env` (G-12) | Fundstelle auf `Clock` umstellen |

## 6. Umbau
1. Import `contextra-types` lösen (REM-30). Typen aus `ports::dto` bzw. `ids`.
2. Canary-Scanner (T-ENC-01), Mock-Embedder, Mock-Gate anlegen (SPEC 15).
3. Schedule-Exploration mit `loom` anbinden.
4. Datei `tests/layering.rs` fehlt (Ist-Abweichung, G-03, Stand 21.11.2026: `cargo test -p contextra-testkit --test layering` meldet „no test target“). Anlegen, sobald die Schichtregeln als Rust-Test festgelegt sind (Offener Punkt O-3).
5. Selbstdeklarationen entfernen (nach Freigabe).

**Offene Punkte (Entscheidung nötig):**
- **O-1** Feature `erasure-conformance`: entfernen oder in SPEC 15 aufnehmen.
- **O-2** Umfang der `conformance/`-Module gegen SPEC 15 festlegen.

## 7. Tests
- G-03: `tests/layering.rs` (SPEC 11.6, 13.6 SYNC-2). Ist-Abweichung: Datei fehlt.
- Ist-Befund: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | `forbid(unsafe_code)` in 3 Dateien | eingehalten |
| G-07 | Import `contextra_types` (14) | verletzt (REM-30) |
| G-09 | 22 Dateien mit ANCHOR | verletzt (REM-14) |
| G-12 | 1 Treffer außerhalb `env` | verletzt |
