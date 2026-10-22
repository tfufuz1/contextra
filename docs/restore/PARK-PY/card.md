# Restore-Karte: PARK-PY

- **ID:** PARK-PY
- **Status:** parked
- **Crates und Pfade:**
  - `crates/contextra-py/` (entfernt)
  - `.github/workflows/contextra-py-ci.yml` (nach `docs/restore/PARK-PY/workflows/contextra-py-ci.yml` verschoben)
  - `.github/workflows/build-wheels.yml` (nach `docs/restore/PARK-PY/workflows/build-wheels.yml` verschoben)
  - `.github/workflows/publish-pypi.yml` (nach `docs/restore/PARK-PY/workflows/publish-pypi.yml` verschoben)
- **Größe in Zeilen:** 8.359 Zeilen (inkl. `Cargo.lock` & `pyproject.toml`) / ca. 2.764 Rust- & Python-Codezeilen in `crates/contextra-py/`
- **BASE_SHA:** `0a629bfb605071274d06c1a9ff6a45054934c153`
- **Tag:** `park/PARK-PY/pre` (Push optional/sofern Rechte vorhanden)
- **Wiring-Patch:** `docs/restore/PARK-PY/wiring.patch` (Keine Code-Kanten in behaltenen Crates vorhanden)

## Gelöste Aufrufstellen
Es gab keine Code-Kanten zu `contextra-py` in den behaltenen Rust-Crates (`crates/*`).

## Mit entfernte / geparkte Artefakte
- **Workflows:**
  - `.github/workflows/contextra-py-ci.yml`
  - `.github/workflows/build-wheels.yml`
  - `.github/workflows/publish-pypi.yml`
- **Referenzen in nicht berührten Dateien (vom Sammelauftrag X1 zu bereinigen):**
  - Wurzel-`Cargo.toml` (`"crates/contextra-py"`, Workspace Dependency)
  - `justfile` (`test-py`, `build-py`)
  - `.github/workflows/pr-gate.yml`, `.github/workflows/coverage.yml` (`--exclude contextra-py`, Clippy/Test Steps)
  - `xtask/src/py_test.rs`, `xtask/src/check_ffi_panic_boundary.rs`, `xtask/src/check_unsafe_islands.rs`, `xtask/src/jules_submit_gate.rs`

## Abhängigkeiten
- **Geparktes Crate `contextra-py` nutzte:**
  - `contextra-db` / `contextra-engine`
  - `contextra-types`
  - `contextra-ports`
  - `contextra-router` (geparkt)
  - `contextra-adapt` (geparkt)
  - PyO3 (`pyo3` v0.20+) FFI Bindings

## tests.manifest
- `docs/restore/PARK-PY/tests.manifest` (0 Inline/Test-Dateien aus behaltenen Crates verschoben)

## Aktivierungsregel und Abnahme-Gate
- **Aktivierungsregel:** Wiederherstellung geplant für v1.6 (R-PY-AGENT).
- **Abnahme-Gate:**
  - `maturin build` / `maturin develop` in `crates/contextra-py` baut erfolgreich.
  - Pytest-Suite in `crates/contextra-py/tests/` verifiziert GIL-Concurrency, Panic-Isolation und MCP-Server-Integration ohne Regressionen.

## Aufwandsschätzung der Wiederherstellung
- **Größe:** M (Medium)
- **Begründung:** Das Crate selbst ist isoliert und stellt PyO3 FFI Bindings bereit. Bei der Wiederherstellung in v1.6 müssen ggf. API-Änderungen an `contextra-db` / `contextra-engine` nachgezogen werden.

## Bekannte Drift
- Keine (anfangs leer).

## Wiederherstellungsschritte
1. Git-Branch `restore/PARK-PY` erstellen.
2. Crate und Workflows wiederherstellen: `git restore --source=0a629bfb605071274d06c1a9ff6a45054934c153 -- crates/contextra-py .github/workflows/contextra-py-ci.yml .github/workflows/build-wheels.yml .github/workflows/publish-pypi.yml`
3. Manifeste, Workflows, `Cargo.toml`, `justfile` und `xtask` über Sammel-Restore bzw. Kartenliste ergänzen.
4. Isolierter Check: `cargo check -p contextra-py`.
5. Wiring-Patch anwenden (sofern vorhanden): `git apply -R --3way docs/restore/PARK-PY/wiring.patch`.
6. Parked Tests (falls vorhanden) zurück aus `docs/restore/PARK-PY/tests/` nach `crates/` verschieben.
7. Test-Suite und FFI-Gate ausführen (`maturin develop && pytest`).
8. Tag `restore/PARK-PY/done` setzen.
