# contextra — Implementierungsspezifikation

**Ebene:** E2 (Fassade)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Öffentliche API nach 6.8, Konfigurationsladen, Fehlerabbildung (SPEC 15). Entfernen: REM-02 (Profile), REM-24 (Agentengedächtnis), REM-25 (Query-Rewriter). Ziel-Umfang: 1.500 Zeilen (SPEC 3.1). Ist: 3.807 Zeilen laut Zielspalte (Altbestand).
Abhängigkeiten nach Anhang E: `db`, `ports`, `ids`, `privacy`.

## 2. Vertrag
Öffentliche API nach 6.8 (Schnittstelle für `mcp` und `server`). Der Build ist im Ist rot (CLAUDE.md §2): die Dateien importieren Typen aus `contextra-db`, die dort nicht mehr existieren. Die Liste steht im Abschnitt „Ist-Zustand“ von `CLAUDE.md`.

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `db`, `ports`, `ids`, `privacy`.
**Ist (Cargo.toml):** `contextra-types` (**verboten**, REM-30, 15 Dateien), `contextra-ports`, `contextra-db`, `tokio` (normal, Wrapper in `deny.toml`, Übergang E29), `serde`, `serde_json`, `uuid`, `thiserror`, `tracing`, `zeroize`.
**Ist-Abweichung:** `contextra-ids` und `contextra-privacy` fehlen in Ist, obwohl Anhang E sie verlangt.
**Features:** Ist `default`, `embedded`. Soll: keine Features außer SPEC 15. `embedded` ist nicht genannt, offen.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/lib.rs` | Wurzel, API | Soll |
| `src/builder.rs`, `src/boot.rs` | Aufbau, Start | Soll (Konfigurationsladen), Verbleib zu prüfen |
| `src/collection_profile.rs`, `src/performance_profile.rs` | Profile | **REM-02 (Profile): entfernen** |
| `src/agent_memory.rs` | Agentengedächtnis | **REM-24: entfernen** |
| `src/query_rewriter.rs` | Query-Rewriter (LLM) | **REM-25: entfernen** |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-02 | `collection_profile.rs`, `performance_profile.rs` | `ls crates/contextra/src/*profile*` leer |
| REM-24 | `src/agent_memory.rs`, Zettelkasten-Typen | `grep -rn "MemoryType\|WorkflowState" crates/contextra` leer |
| REM-25 | `src/query_rewriter.rs` | `grep -rn "QueryRewriter\|LlmTextGenerator" crates/contextra` leer |
| REM-30 | Import `contextra-types` (15 Dateien) | `grep -rln "contextra_types" crates/contextra` leer |
| REM-14 | ANCHOR-Dateien (21) | `grep -rl "ANCHOR\[" crates/contextra` leer |

## 6. Umbau
1. Build-Fehler beheben, bevor etwas anderes passiert. Ursache sind fehlende Typen aus `contextra-db` (CLAUDE.md §2). Entscheidung nötig: Stub oder Feature-Gating (CLAUDE.md §6 Punkt 2).
2. REM-02, REM-24, REM-25 entfernen.
3. `contextra-types` lösen (REM-30).
4. Abhängigkeiten `contextra-ids` und `contextra-privacy` ergänzen.
5. `tokio` klären (Punkt 10 CLAUDE.md).
6. `unwrap`/`expect`/`panic!` (1 Treffer) nach R6 abbauen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** Build-Fehler: Stub oder Feature-Gating (CLAUDE.md §6 Punkt 2).
- **O-2** Feature `embedded`: nicht in SPEC 15.
- **O-3** `tokio` als normale Abhängigkeit der Fassade (Punkt 10 CLAUDE.md).

## 7. Tests
API-Oberflächentest, Konfigurationsfehler (T-OPS-03, SPEC 15). Ist: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-01 | Build rot (20 Fehler laut CLAUDE.md §2) | grün |
| G-03 | `types` verboten, `ids`/`privacy` fehlen | verletzt |
| G-07 | Import `contextra_types` (15) | verletzt (REM-30) |
| G-09 | 21 Dateien mit ANCHOR | verletzt (REM-14) |
