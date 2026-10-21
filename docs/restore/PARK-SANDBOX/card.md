# RESTORE-KARTE: PARK-SANDBOX

## Metadata
- **Park-ID:** PARK-SANDBOX
- **Status:** parked
- **Crate:** `crates/contextra-sandbox`
- **Parkierte Pfade:**
  - `crates/contextra-sandbox/` (ca. 3.680 Zeilen)
  - `crates/contextra-engine/src/merge_operator_adapter.rs` -> `docs/restore/PARK-SANDBOX/engine/merge_operator_adapter.rs.parked` (269 Zeilen)
- **Gesamtumfang:** ~3.949 Zeilen
- **BASE_SHA:** `0a629bfb605071274d06c1a9ff6a45054934c153`
- **Pre-Tag:** `park/PARK-SANDBOX/pre`

## Wiring Patch & Gelöste Kanten
- **Wiring Patch:** `docs/restore/PARK-SANDBOX/wiring.patch`
- **Gelöste Aufrufstellen / Entkoppelte Kanten:**
  1. `crates/contextra-engine/Cargo.toml`: Optionale Abhängigkeit `contextra-sandbox = { workspace = true, optional = true }` und Feature `sandbox = ["dep:contextra-sandbox"]` entfernt.
  2. `crates/contextra-engine/src/lib.rs`: `#[cfg(feature = "sandbox")] pub mod merge_operator_adapter;` und `#[cfg(feature = "sandbox")] pub use merge_operator_adapter::WasmMergeOperator;` entfernt.
  3. `crates/contextra-engine/src/merge_operator_adapter.rs`: Nach `docs/restore/PARK-SANDBOX/engine/merge_operator_adapter.rs.parked` verschoben.

## Entfernte Features, Tools, Workflows & xtask-Stellen
- **Entfernte Features:** `sandbox` in `crates/contextra-engine/Cargo.toml`.
- **Anmerkungen zu unberührten Dateien (Sammelauftrag X1 / Allowlist-Schutz):**
  - `crates/contextra-mcp/src/sandbox.rs`: MCP-eigene Isolationsschicht, kein Bezug auf `contextra-sandbox` Crate (unberührt).
  - `crates/contextra-db/Cargo.toml`: Feature `sandbox = []` wird durch Sammelauftrag X1 / Arbeitsauftrag bereinigt.

## Abhängigkeiten & Schnittstellen
- **Ports & Types:** `contextra_ports::MergeOperatorHost`, `contextra_store::MergeOperator`, `contextra_types::ContextraError`.

## Tests & Manifest
- **Manifest:** `docs/restore/PARK-SANDBOX/tests.manifest`
- **Verschobene / Parkierte Tests:** 13 Test-Funktionen (7 Integrationstest-Dateien in `crates/contextra-sandbox/tests/` sowie Inline-Tests in `merge_operator_adapter.rs.parked`).

## Aktivierungsregel & Abnahme-Gate
- **Aktivierungsregel (R-SANDBOX v1.8):** WASM-Merge-Operatoren oder benutzerdefinierte WASM-Funktionen werden in der Runtime benötigt.
- **Abnahme-Gate:**
  - `rg -n 'contextra_sandbox|contextra-sandbox' crates/contextra-engine` ist leer.
  - `merge_operator_adapter.rs` liegt nur als `.rs.parked` unter `docs/restore/PARK-SANDBOX/engine/`.

## Aufwandsschätzung Wiederherstellung
- **Aufwand:** S (Small)
- **Begründung:** Sauber isolierte WASM-Grenzschicht; Wiederherstellung erfordert lediglich Re-Import des Crates, Re-Aktivierung des Features in `contextra-engine` und Rück-Anwendung von `wiring.patch`.

## Bekannte Drift
- Keine.

## Wiederherstellungsschritte
1. Erstelle Recovery-Branch: `git checkout -b restore/PARK-SANDBOX`.
2. Stelle Quellcode wieder her: `git restore --source=0a629bfb605071274d06c1a9ff6a45054934c153 -- crates/contextra-sandbox`.
3. Binde Crate in `Cargo.toml` / `capabilities.toml` (falls zutreffend) wieder ein.
4. Führe isolierten Cargo-Check aus: `cargo check -p contextra-sandbox`.
5. Wende Wiring-Patch rückwärts an: `git apply -R --3way docs/restore/PARK-SANDBOX/wiring.patch`.
6. Benenne parkierte Dateien um: `git mv docs/restore/PARK-SANDBOX/engine/merge_operator_adapter.rs.parked crates/contextra-engine/src/merge_operator_adapter.rs`.
7. Führe Tests und Abnahme-Gate aus: `cargo test -p contextra-sandbox && cargo test -p contextra-engine`.
8. Setze Tag: `git tag restore/PARK-SANDBOX/done`.
