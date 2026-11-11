# contextra-engine — Umsetzungs- & Migrationsplan (v4.1 Baukasten-Architektur)

> Abgeleitet aus `docs/SPEC.md` (v3/v4.1 Gesamtspezifikation) und den Entkopplungsleitlinien für Klasse-C-Komponenten (Ebene E2 Integration).

## 1. Status der Roadmap-Schritte

| Schritt | Inhalt | Status | Beleg / Nachweis |
|---|---|---|---|
| **S-00 / S-01** | Tooling & Layering-Gates Baseline | Abgeschlossen | `capabilities.fragment.toml` Schema v3 |
| **S-02 / S-03** | Golden-Korpus, Differenz-Harness, Metadaten-Bereinigung | Abgeschlossen | Golden-Exporte & Invarianten-Tests |
| **S-33** | `rank`: Fusion & Scoring-Entkopplung | Abgeschlossen | `src/fusion.rs` delegiert an `contextra-rank` |
| **S-37** | `store`: Speicherkern-Entkopplung (`StoreError`, `compat/`) | Abgeschlossen | Generische Bindung über `Backends` / `Collection<S, V>` |
| **S-50 / S-51** | Lizenz- und Erasure-Ports nach `contextra-license` / `contextra-erasure` | **Teilweise / Offen** | Erasure aus Engine-Kern entfernt (WP-14); Lizenz-Ports verbleiben in `contextra-ports::license` und wurden in `src/license_seam.rs` gekapselt. Feature `license` folgt nach formeller Abgabe durch `ports`. Beleg: `grep -rn 'contextra_ports::license' crates/contextra-engine/` (nur `src/license_seam.rs`). |
| **S-60** | Entkopplung Regler (`AdaptiveDecayController`) | Vorbereitet (ENG-B) | Übergabe an Parallelauftrag ENG-B |
| **S-61** | Chunker: Extraktion/Parität mit `contextra-ingest` | Vorgemerkt (ENG-B) | Übergabe an Parallelauftrag ENG-B |
| **S-63** | `cognition` Invertierung (Entfernung von `contextra-engine` aus normal tree) | Abhängigkeit isoliert | `s64_public_surface.rs` schützt Konsumentenpfade |
| **S-64** | `contextra-engine` Baustein-Entkopplung, Seam-Isolierung, Doku | **Abgeschlossen** | S-64 (ENG-A): Seam `license_seam.rs`, Bereinigung ungenutzter Deps, Schema v3 Fragment, Konsumenten-Anker |

## 2. Durchgeführte Maßnahmen (S-64 / ENG-A)

1. **Konsumenten-Anker (`tests/s64_public_surface.rs`):**
   - Pinnt alle öffentlichen Re-Exporte von `src/lib.rs` und Pfade, die von `contextra-db` und `contextra-cognition` konsumiert werden, zur Compile-Zeit ab.

2. **Lizenz-Naht isolieren (`src/license_seam.rs`):**
   - Kapselt alle Verwendungen von `contextra_ports::license::*` an einer Nahtstelle für die spätere Migration zu `contextra-license`.
   - Semantik-Test: `tests/s64_license_default_semantics.rs`.

3. **Abhängigkeits- und Feature-Hygiene:**
   - Ungenutzte Runtime-Abhängigkeiten (`blake3`, `hmac`, `bincode`, `zeroize`, `thiserror`) entfernt; `sha2` in Dev-Dependencies überführt.
   - Alle 11 Features (`onnx`, `sandbox`, `experimental-diskann`, `adaptive-candidate-pool-sizing`, `exp-extraction`, `entity-extraction`, `reranking`, `background-maintenance`, `graph-connectivity-health`, `edge-reinforcement-learning`, `auto-extraction-opt-out`) auf Quell- und Forwarding-Bedarf geprüft und einzeln verifiziert.

4. **Metadaten Schema v3 & Neutralität:**
   - `capabilities.fragment.toml` auf Schema v3 standardisiert.
   - Compliance-Aussagen in Dokumentation/Docstrings neutralisiert (INV-7).
   - Fehlerabbildung in `docs/decoupling/ERRORS.md` und Test `tests/s64_error_passthrough.rs` dokumentiert.

## 3. Offene Punkte für Folgeaufträge

- **Feature `license` & `contextra-license`**: Sobald `contextra-ports` die Lizenz-Traits nach `contextra-license` übergibt, kann `src/license_seam.rs` auf die neue Crate umgestellt werden.
- **Root-`capabilities.toml` Update**: `allowed_internal` (Entfernung von `kvcache`, Aufnahme von `testkit` in `dev_allowed`) durch Z / F0.
- **ENG-B Komponenten**: `chunker/**`, `temporal_filter.rs`, `decay_controller.rs` und zugehörige Tests verbleiben bei ENG-B.
