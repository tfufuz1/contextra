# RESTORE-KARTE: PARK-ROUTER (B1)

- **ID:** PARK-ROUTER
- **Status:** `parked`
- **Crates und Pfade:** `crates/contextra-router`
- **Größe in Zeilen:** 8755 Zeilen (`git ls-files crates/contextra-router | xargs wc -l`)
- **BASE_SHA:** `6166e9d93a6fbb2ac42ae2bbec08c74ce31e5a3b`
- **Tag:** `park/PARK-ROUTER/pre`

---

## 1. Wiring Patch und Kanten
- **Wiring-Patch:** `docs/restore/PARK-ROUTER/wiring.patch`
- **Liste gelöster Aufrufstellen:**
  - `crates/contextra/src/lib.rs:37`: `pub use contextra_router as router;` (entfernt)
  - `crates/contextra/Cargo.toml:28,91`: Optionale Abhängigkeit `contextra-router` und Feature `router` (entfernt)
  - `crates/contextra-mcp/Cargo.toml:21`: Feature `router` in `contextra` Abhängigkeit (entfernt)
  - `crates/contextra-mcp/src/routing.rs:1..137`: Router-Setup, Handles und Adapter entfallen; `setup_kv_bridge` wurde nach `crates/contextra-mcp/src/server.rs` eingebettet (Datei `routing.rs` gelöscht)
  - `crates/contextra-mcp/src/config.rs:194..237`: `RouterConfig` samt Feldern und Env-Logik (entfernt)
  - `crates/contextra-mcp/src/server.rs:25,95,121`: `routing` Feld und `with_routing` Methode auf `McpServer` (entfernt)
  - `crates/contextra-mcp/src/lib.rs:40,79`: Modul `routing` und Re-Exports `setup_routing`, `RoutingHandle` (entfernt)
  - `crates/contextra-mcp/tests/mcp_test.rs:954..1002`: Tests `test_mcp_boot_without_routing_config` und `test_mcp_boot_with_routing_config` nach `docs/restore/PARK-ROUTER/tests/contextra-mcp/mcp_test_inline.rs.parked` verschoben

---

## 2. Entfernte Features, Tools, Workflows & Konfiguration
- **Features:** `router` Feature in `crates/contextra`.
- **Workflows:** Security check in `.github/workflows/pr-gate.yml` für `sh -c` in `dispatch.rs` entfernt.
- **xtask:** Referenzen in Layering (`xtask/src/check_ring_layering.rs`) und Claims (`xtask/src/claim.rs`).
- **Konfigurationswerte:** Env-Variablen `CONTEXTRA_ROUTER_PROFILES_PATH`, `CONTEXTRA_ROUTER_CALIBRATION_PATH`, `CONTEXTRA_ROUTER_PROFILES_JSON`.

---

## 3. Abhängigkeiten (Ports & Types)
- `contextra-ports`: `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer`, `DriftStatusProvider`
- `contextra-types`: `ContextChunk`, `ContextWindow`, `TokenBudget`, `EntityId`, `ContextraError`

---

## 4. Tests Manifest
- `docs/restore/PARK-ROUTER/tests.manifest` (2 verschobene Inline-Tests)

---

## 5. Aktivierungsregel & Abnahme-Gate
- **Aktivierungsregel:** Wiederherstellung für v1.1 (R-ROUTING) vorausgesetzt R1.1 (`adapt`) und R1.2 (`privacy` + `avv-generator`) sind wiederhergestellt.
- **Abnahme-Gate:**
  - `rg -n 'contextra_router|contextra-router' --glob '!docs/restore/**' --glob '!target' .` liefert 0 Treffer (außer unberührter xtask-Test `router_dev_only_dependency.rs`).
  - `rg -n 'RouterConfig|setup_routing|RoutingHandle' crates` liefert 0 Treffer.

---

## 6. Aufwandsschätzung Wiederherstellung
- **Aufwand:** M (Medium)
- **Begründung:** Sauber isoliertes Crate mit wenigen Anbindungen (hauptsächlich MCP Server Bootstrapping und Facade Re-Export). Bei Wiederherstellung muss der Bridge-Glue in `mcp` neu verdrahtet und `wiring.patch` in umgekehrter Richtung angewendet werden.

---

## 7. Bekannte Drift
- Keine.

---

## 8. Wiederherstellungsschritte
1. Branch `restore/PARK-ROUTER` erstellen.
2. `git restore --source=6166e9d93a6fbb2ac42ae2bbec08c74ce31e5a3b -- crates/contextra-router`.
3. Workspace Manifeste (`Cargo.toml`, `capabilities.toml`, `deny.toml`, `ARCHITECTURE.md`, `tests/layering.rs`, `.github/workflows/pr-gate.yml`) nach Kartenliste ergänzen.
4. `cargo check -p contextra-router` isoliert durchführen.
5. `git apply -R --3way docs/restore/PARK-ROUTER/wiring.patch`.
6. Inline-Tests aus `docs/restore/PARK-ROUTER/tests/contextra-mcp/mcp_test_inline.rs.parked` zurück nach `crates/contextra-mcp/tests/mcp_test.rs` integrieren.
7. Tests & Abnahme-Gate ausführen.
8. Tag `restore/PARK-ROUTER/done` erstellen.
