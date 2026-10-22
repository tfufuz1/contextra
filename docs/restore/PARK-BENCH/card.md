# Restore-Karte: PARK-BENCH

- **ID**: PARK-BENCH
- **Status**: parked
- **Crates und Pfade**: `benchmarks/contextra-bench`
- **Größe in Zeilen**: 8.716 Zeilen (28 Dateien)
- **BASE_SHA**: `0a629bfb605071274d06c1a9ff6a45054934c153`
- **Tag**: `park/PARK-BENCH/pre` (lokal erstellt)

## Wiring-Patch & Gelöste Aufrufstellen
- **Wiring-Patch**: Entfällt (`docs/restore/PARK-BENCH/wiring.patch` entfällt, da keine Kanten im behaltenen Code existieren).
- **Gelöste Aufrufstellen**: None in retained crate code. (Nicht-Allowlist-Referenzen in `Cargo.toml`, `justfile`, `benchmarks/README.md`, `xtask/src/bench_download.rs`, `xtask/src/bench_gate.rs` etc. verbleiben für den Sammelauftrag X1).

## Entfernte Features, Tools, Workflows, xtask-Stellen & Konfigurationen
- **Workflows**:
  - `.github/workflows/bench.yml` (verschoben nach `docs/restore/PARK-BENCH/workflows/bench.yml`)
  - `.github/workflows/bench-regression.yml` (verschoben nach `docs/restore/PARK-BENCH/workflows/bench-regression.yml`)
- **Tools/xtask/Konfigurationen (verbleiben im Workspace für Sammelauftrag X1)**:
  - `Cargo.toml` (`workspace.members`: `benchmarks/contextra-bench`)
  - `justfile` (`bench-regression`, `cargo run -p contextra-bench`)
  - `xtask/src/bench_download.rs`
  - `xtask/src/bench_gate.rs`
  - `xtask/src/check_manifest_completeness.rs`
  - `xtask/src/check_unsafe_islands.rs`
  - `benchmarks/README.md`

## Abhängigkeiten
Das Crate `contextra-bench` hing an:
- `contextra-infer-onnx`
- `contextra-graph`
- `contextra-adapt`
- `contextra-ports` (Clock, Rng, IdGenerator)
- `contextra-types`

## Tests
- **`tests.manifest`**: `docs/restore/PARK-BENCH/tests.manifest`
- **Anzahl verschobener Tests**: 0 (alle Tests verblieben in `benchmarks/contextra-bench`, welches komplett als Verzeichnis gelöscht wurde).

## Aktivierungsregel & Abnahme-Gate
- **Aktivierungsregel**: Das Crate `contextra-bench` wird erst wiederhergestellt, wenn `contextra-infer-onnx`, `contextra-graph` und `contextra-adapt` aus dem Park-Zustand zurückgekehrt sind oder wenn die Benchmark-Komponenten ohne diese Abhängigkeiten auskommen.
- **Abnahme-Gate**:
  - Both workflow files present in `docs/restore/PARK-BENCH/workflows/`.
  - Directory `benchmarks/contextra-bench/` removed.
  - Directory `benchmarks/store_vs_redb/` untouched.

## Aufwandsschätzung der Wiederherstellung
- **Kategorie**: M (Medium)
- **Begründung**: Das Crate besitzt 8.716 LOC und hängt an mehreren geparkten Komponenten (`contextra-infer-onnx`, `contextra-graph`, `contextra-adapt`). Die Wiederherstellung erfordert die Re-Integration der Workspace-Manifest-Einträge, Workflows und xtask-Verbindungen.

## Bekannte Drift
- Keine bekannt.

## Wiederherstellungsschritte
1. Branch `restore/PARK-BENCH` von Hauptbranch erstellen.
2. `git restore --source=0a629bfb605071274d06c1a9ff6a45054934c153 -- benchmarks/contextra-bench/ .github/workflows/bench.yml .github/workflows/bench-regression.yml` ausführen.
3. Manifeste (`Cargo.toml`, `capabilities.toml`), `justfile`, `xtask` Module und Workflows nach obiger Liste ergänzen / wieder eintragen.
4. `cargo check -p contextra-bench` isoliert ausführen und Abhängigkeiten prüfen.
5. (Falls ein `wiring.patch` existierte, `git apply -R --3way docs/restore/PARK-BENCH/wiring.patch` anwenden - entfällt hier).
6. Verschobene `*.rs.parked` Dateien / Funktionen (falls vorhanden) an Ursprungsort zurückverschieben.
7. Vollständiges Testing: `cargo test -p contextra-bench` und Regression-Gate verifizieren.
8. Tag `restore/PARK-BENCH/done` setzen.
