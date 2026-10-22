# Restore-Karte: PARK-AGENT

## Metadaten
- **ID**: PARK-AGENT
- **Status**: `parked`
- **Crates und Pfade**: `crates/contextra-agent`
- **Größe**: 10.169 Zeilen (37 Dateien)
- **BASE_SHA**: `0a629bfb605071274d06c1a9ff6a45054934c153`
- **Tag**: `park/PARK-AGENT/pre`

## Wiring-Patch & Aufrufstellen
- **Wiring-Patch**: keine Kanten (entfällt)
- **Gelöste Aufrufstellen**: Keine Kanten in behaltenen Crates. `contextra-agent` hat keine Konsumenten in verbleibenden Core/Engine/Store Crates.

## Relevante Stellen in verbleibenden Konfigurationsdateien (Sammelauftrag X1)
Die folgenden Stellen verweisen auf `contextra-agent` und werden vom Sammelauftrag X1 bereinigt:
- `Cargo.toml`: `members` (`"crates/contextra-agent"`) und `[workspace.dependencies]` (`contextra-agent = { path = "crates/contextra-agent" }`)
- `capabilities.toml`: `[crates.contextra-agent]` Deklaration und Testkommandos
- `deny.toml`: `skip = ["contextra-agent"]`
- `xtask/src/check_ring_layering.rs`: Zuordnung Ring 3 Order
- `Cargo.lock`: Eintrag für `contextra-agent`
- `ARCHITECTURE.md`, `WORKING_STATE.md`, `docs/FEATURES_SPECIFICATION.md`: Architekturdokumentation

## Abhängigkeiten (Ports & Types)
`contextra-agent` nutzt u. a. folgende Ports und Typen:
- `contextra-ports`: `Clock`, `Rng`, `IdGenerator`
- `contextra-types`: `ContextraError`, `Result`, `TenantId`, `TxId`
- Externe Crate-Abhängigkeiten (vormals): `contextra-checkpoint`, `contextra-db`, `contextra-graph`, `contextra-router`, `contextra-store`

## Tests
- **`tests.manifest`**: `docs/restore/PARK-AGENT/tests.manifest`
- **Anzahl verschobener Tests**: 0 (Candidate hatte keine Tests in behaltenen Crates)

## Wiederherstellungsvoraussetzungen & Abnahme
- **Wiederherstellungsreihenfolge**: `agent` nutzt `router`, `graph` und `checkpoint`; Wiederherstellung erst in v1.6 (R-PY-AGENT) nach diesen drei Crates.
- **Aktivierungsregel**: Nach Wiederherstellung von `contextra-router`, `contextra-graph` und `contextra-checkpoint`.
- **Abnahme-Gate**: `cargo check -p contextra-agent --locked` & `cargo test -p contextra-agent --locked` bestanden.

## Aufwandsschätzung Wiederherstellung
- **Schätzung**: S (Small)
- **Begründung**: Keine Schnittkanten oder Patches in behaltenem Code erforderlich. Es müssen lediglich das Crate-Verzeichnis aus `BASE_SHA` wiederhergestellt und die Workspace-Manifeste re-integriert werden.

## Bekannte Drift
*(derzeit leer)*

## Wiederherstellungsschritte
1. Branch `restore/PARK-AGENT` erstellen.
2. `git restore --source=0a629bfb605071274d06c1a9ff6a45054934c153 -- crates/contextra-agent`
3. Manifeste (`Cargo.toml`, `capabilities.toml`, `deny.toml`, `xtask/src/check_ring_layering.rs`) re-integrieren.
4. `cargo check -p contextra-agent` isoliert ausführen und Abhängigkeiten prüfen.
5. (Wiring-Patch entfällt)
6. (`*.rs.parked` Entpacken entfällt)
7. `cargo test -p contextra-agent --locked` ausführen und verifizieren.
8. Tag `restore/PARK-AGENT/done` setzen.
