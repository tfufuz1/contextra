# contextra-graph — Implementierungsspezifikation

**Ebene:** E1, optional (SPEC 3.1, 3.2, 9)


## 1. Zweck und Ebene
Entitätsgraph: reduzierter CSR-Graph mit Seal-Provenienz und PathRAG (SPEC 15, 3.1, ADR-21 bis ADR-27, Kapitel 6.10). Optional. Ob es im Ziel bleibt, entscheidet G-GRAPH und REM-31 (SPEC 12.4). Ziel-Umfang: 6.000 Zeilen (SPEC 3.1). Ist: 34.611 Zeilen laut Zielspalte (Altbestand).
Abhängigkeiten nach Anhang E: `ports`, `ids`, `env`, `durable-fs`.

## 2. Vertrag
Implementiert (SPEC 3.4): `GraphIndex` und `ScrubTarget` für den Entitätsgraphen. Ist: `GraphIndex` ist in `contextra-ports` (`graph_index.rs`) definiert, `ScrubTarget` fehlt dort. Ist-Abweichung: Das Ports-Crate führt außerdem `GraphCollectionMutation` und `CommunityResolver` (siehe PLAN von `contextra-ports`, O-5).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`, `env`, `durable-fs`.
**Ist (Cargo.toml):** erlaubt sind `contextra-ids`, `contextra-env`, `contextra-ports`. **Verboten:** `contextra-types` (R1, REM-30, 83 Dateien). Weitere: `bytes`, `parking_lot`, `tracing`, `ahash`, `serde`, `serde_json`, `bincode`, `blake3`, `arc-swap`, `scc`, `thiserror`.
**Ist-Abweichung:** `contextra-durable-fs` fehlt in Ist.
**Features:** Ist `default`, `workspace`, `session-dag`, `entity-extraction`, `hyperedge-suggest`, `apprh-diffusion`, `k-path-diffusion`, `graph-connectivity-health`, `edge-reinforcement-learning`, `docid-128`, `test-support`. Soll (SPEC 15): keine Features außer SPEC 15. Ist-Abweichung: alle genannten außer `docid-128` sind in SPEC 15 nicht genannt. `workspace` verletzt CMP-10.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/apprh/*` (diffusion, gate_monitor, params, selector, shadow) | APPRH-Diffusion | nicht in SPEC 15, offen (ADR-21 bis ADR-27 prüfen) |
| `src/community.rs`, `src/cascade.rs` | Gemeinschaften, Kaskaden | nicht in SPEC 15, offen |
| `src/arc_slice.rs` | Hilfstyp | zu prüfen |
| `src/compat.rs`, `src/compat/*` (graph_index, collection_mutation, algo_error_bridge, graph_index_ext, error) | Kompatibilitätsmodul | **entfernen** (R7, G-11). Inhalt in Soll-Module |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

Soll-Module nach 6.10 und SPEC 15: CSR-Graph, Seal-Provenienz, PathRAG. Im Ist nicht eindeutig zugeordnet, prüfen.

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-31 / SPEC 12.4 | Graph-Reduktion, nur wenn G-GRAPH grün | Entscheidung nach 12.4, offen |
| CMP-10 | Feature `workspace` und Compat-Namen | `grep -n "compat\|workspace" crates/contextra-graph/Cargo.toml` leer |
| REM-30 | Import `contextra-types` (83 Dateien) | `grep -rln "contextra_types" crates/contextra-graph` leer |
| G-11 | `compat/` und `compat.rs` | `find crates/contextra-graph -name "compat*"` leer |
| REM-14 | ANCHOR-Dateien (125) | `grep -rl "ANCHOR\[" crates/contextra-graph` leer |

## 6. Umbau
1. Entscheidung über Verbleib (G-GRAPH, REM-31) vor jedem Umbau. Ohne Entscheidung kein Umbau.
2. `compat/` auflösen, Typen aus `contextra-types` verlagern.
3. Features nach Entscheidung reduzieren.
4. `unwrap`/`expect`/`panic!` (741 Treffer) nach R6 abbauen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** Verbleib des Crates: G-GRAPH und REM-31 (SPEC 12.4). Ohne Entscheidung ist kein Umbau vertretbar.
- **O-2** Features `apprh-diffusion`, `k-path-diffusion`, `edge-reinforcement-learning`, `hyperedge-suggest`, `session-dag`, `entity-extraction`, `graph-connectivity-health`: nicht in SPEC 15.
- **O-3** Module `apprh/` und `community.rs`: Zuordnung in SPEC nicht eindeutig.

## 7. Tests
- T-GR-01 bis T-GR-12 (SPEC 15). T-GR-12: `db` ohne Feature `graph` kompiliert ohne Graph-Code.
- Ist: `src/community/tests.rs`. Zählung nicht erfolgt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-03 | `contextra-types` (R1) | verletzt (REM-30) |
| G-07 | Import `contextra_types` in 83 Dateien | verletzt |
| G-09 | 125 Dateien mit ANCHOR | verletzt (REM-14) |
| G-11 | `compat/`, Feature `workspace` | verletzt |
| G-12 | 6 Treffer | verletzt |
| G-13 | 741 Treffer | Abbau nach R6 |
