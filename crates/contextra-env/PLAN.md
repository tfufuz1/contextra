# contextra-env — Umsetzungs- und Architekturplan

> **Status:** Vollständig umgesetzt und entkoppelt (E0 Foundation Baustein).
> **Referenz:** `docs/SPEC.md` v2 / §4.2, `crates/contextra-env/SPEC.md`.

## 1. Baustein-Architektur und Entkopplungsstatus

`contextra-env` ist als **Ebene-0-Mikrokern (E0)** entworfen:
- Keine internen Workspace-Abhängigkeiten (`allowed_internal = []`).
- Standalone-Kompilierung über `tools/export_crate.py` (Gate G3) bestanden.
- Vollständig isolierte Traits und Referenzimplementierungen für:
  1. `Clock` & `SystemClock` / `ManualClock` (unter Feature `test-utils`).
  2. `Rng` & `SeededRng` (SplitMix64 deterministischer PRNG).
  3. `IdGen` & `SequentialIdGen` (Atomare ID-Generierung).
  4. `MetricsSink`, `NoopMetricsSink` & `InMemoryMetricsSink` (Telemetrie).

## 2. K13-Feature-Harmonisierung [Abgeschlossen]

- **Vorgabe (SPEC §4.2 / K13):** `default = ["std"]`; `test-utils` wird nur bei Bedarf von Test-Verbrauchern aktiviert.
- **Ist-Korrektur:** In `Cargo.toml` wurde `default = ["std"]` gesetzt, sodass `test-utils` nicht standardmäßig in Produktions-Builds kompiliert wird.
- **Nachweis:** `cargo tree -p contextra-env -e features` bzw. Standalone-Checks zeigen, dass `test-utils` optional ist.

## 3. Kommentierungs- und Dokumentationssystem

Etablierung des einheitlichen Comment-Anchors-Schemas für alle Quellcodedateien in `src/`:
- **Epics:**
  - `epic=env-core`: Modulinitialisierung und Re-Exports (`lib.rs`, `env.rs`).
  - `epic=clock`: Zeitabstraktion und System-/Monotonieuhr (`clock.rs`, `manual_clock.rs`).
  - `epic=rng`: Pseudozufallsgeneratoren und SplitMix64 (`rng.rs`).
  - `epic=id-gen`: Atomare ID-Generierung (`id_gen.rs`).
  - `epic=metrics`: Observability und Metriksenken (`metrics.rs`).
- **Anker-Struktur:**
  - Regionen: `SECTION ...` / `!SECTION ...`
  - Querverweise: `LINK`
  - Inline-Hinweise: `NOTE` und `ANCHOR[id=...,epic=...,seq=...]`

## 4. Definition of Done & Verifikationskriterien

1. **Gate G3 (Standalone-Fähigkeit):** `tools/export_crate.py --out /tmp/test-env contextra-env` baut erfolgreich ohne Workspace.
2. **Unit- & Conformance-Tests:** 15 Unittests + 4 Conformance-Tests + Golden-Differenztest erfolgreich grün.
3. **Modularity Score:** 100/100 auf `tools/decouple_inspector.py`.
4. **Zero Panic / No Unsafe:** `#![forbid(unsafe_code)]` und 0 unhandled Panics.
5. **Erweiterte Kommentierung:** Vollständige Durchdringung aller Dateien mit didaktischer Transparenz.
