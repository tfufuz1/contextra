# Restore Card: PARK-COGNITION

## Metadaten
- **ID:** PARK-COGNITION
- **Status:** parked
- **Crate(n):** `crates/contextra-cognition`
- **Pfade:** `crates/contextra-cognition/`
- **Größe:** 9.146 Zeilen (inkl. Tests; 5.240 Zeilen Crate-Quellcode)
- **BASE_SHA:** `6166e9d93a6fbb2ac42ae2bbec08c74ce31e5a3b`
- **Tag:** `park/PARK-COGNITION/pre` (versucht anzulegen)
- **Wiring-Patch:** `docs/restore/PARK-COGNITION/wiring.patch`
- **Tests Manifest:** `docs/restore/PARK-COGNITION/tests.manifest` (11 Testfunktionen verschoben)

## Beschreibung
Parken von `crates/contextra-cognition` gemäß Park-Verfahren C1. `contextra-cognition` implementiert Memory Consolidation Passes (Structural Consolidation Pass, Generative Synthesis Pass, Context Compaction, Maintenance Scheduler).

## Gelöste Kanten (Wiring)

| Datei:Zeile | Vorher | Nachher |
|---|---|---|
| `crates/contextra-db/Cargo.toml:17` | `contextra-cognition = { workspace = true }` | Abhängigkeit entfernt |
| `crates/contextra-db/src/lib.rs:27-33` | Re-Exports von `contextra_cognition` (`execute_background_consolidation`, `ConsolidationConfig`, `MaintenanceScheduler` etc.) | Re-Exports vollständig entfernt |
| `crates/contextra-mcp/src/lib.rs:23` | Doc-Eintrag für Tool `contextra_consolidate` | Eintrag entfernt |
| `crates/contextra-mcp/src/sandbox.rs:51,119-131` | `TOOL_REGISTRY` Definition und Einstufung von `contextra_consolidate` | Tool aus Registry und Tests entfernt |
| `crates/contextra-mcp/src/server_dispatch.rs:72,404` | Discovery-Schema und Match-Arm für `contextra_consolidate` | Tool-Schema und Dispatch-Knoten entfernt |
| `crates/contextra-mcp/src/server_tools.rs:43,664-754` | Dispatch-Arm und Handler `handle_consolidate` | Handler und Aufrufpfad entfernt |
| `crates/contextra-mcp/tests/campaign_j42_mcp_audit.rs:21` | Tool-Test-Liste mit `contextra_consolidate` | Tool aus Liste entfernt |
| `crates/contextra-mcp/tests/tool_registry_classification.rs:131-149` | Klassifizierungstest für `contextra_consolidate` | Tool aus Erwartungs-Map entfernt |
| `crates/contextra-mcp/tests/tests.rs:97,176,817,1016-1134` | Tool-Aufruf-Tests für `contextra_consolidate` | Tests entfernt |
| `Cargo.toml:35,146` | Workspace member & dep `contextra-cognition` | Einträge gestrichen |
| `capabilities.toml:80-90,123` | Crate-Eintrag `[crates.contextra-cognition]` & dep Verweise | Einträge gestrichen |
| `deny.toml:65` | `contextra-cognition` in Allowed Crates | Eintrag gestrichen |
| `tests/layering.rs:76` | Ring-3 Layering Test mit `contextra-cognition` | Eintrag gestrichen |
| `xtask/src/check_ring_layering.rs:68` | Ring-3 Ordnung mit `contextra-cognition` | Eintrag gestrichen |

## Abhängigkeiten (Imports aus Core)
- `contextra-types` (`ContextraError`, `Result`, `DocId`, `TenantId`, `CollectionId`, `DistanceMetric`)
- `contextra-ports` (`Clock`, `Rng`, `IdGenerator`, `TextEmbeddingEngine`, `SegmentSynthesizer`)
- `contextra-store` (`LsmStorage`)
- `contextra-graph` (`CsrGraph`)
- `contextra-engine` (`Collection`)

## Aktivierungsregel & Abnahme-Gate
- **Aktivierung:** Wiederherstellung erfolgt in Version v1.2 (R-GRAPH) nach `graph`.
- **Abnahme:** `rg -n 'contextra_cognition|contextra-cognition|contextra_consolidate' --glob '!docs/restore/**' --glob '!target' .` liefert 0 Treffer.

## Aufwandsschätzung der Wiederherstellung
- **Aufwand:** M (Medium)
- **Begründung:** Wiederherstellung erfordert Re-Integration in `contextra-db` / `contextra-mcp` und Abstimmung der Schnittstelle zu `contextra-graph` und `contextra-engine`.

## Wiederherstellungsschritte
1. Branch `restore/PARK-COGNITION` erstellen.
2. `git restore --source=6166e9d93a6fbb2ac42ae2bbec08c74ce31e5a3b -- crates/contextra-cognition/` ausführen.
3. Crate in `Cargo.toml`, `capabilities.toml`, `deny.toml`, `tests/layering.rs`, `xtask/src/check_ring_layering.rs` wieder eintragen.
4. `cargo check -p contextra-cognition` isoliert ausführen.
5. `git apply -R --3way docs/restore/PARK-COGNITION/wiring.patch` anwenden.
6. Geparkte Testdateien unter `docs/restore/PARK-COGNITION/tests/crates/contextra-db/*.rs.parked` zurück nach `crates/contextra-db/tests/*.rs` verschieben.
7. Workspace-Check und -Tests ausführen (`cargo test -p contextra-cognition -p contextra-db -p contextra-mcp`).
8. Tag `restore/PARK-COGNITION/done` setzen.
