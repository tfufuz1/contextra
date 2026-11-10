# ADR-0117: Erweiterung der CI-Workflows und Matrix-Prüfungen

**Status:** Angenommen
**Datum:** 10. November 2026
**Autor:** Principal Rust Engineer (Auftrag F0)
**Bezug:** SPEC v2 §7 (S-01), §8 (B11, B14, G-L3, G-C1)
**Label:** `protected-path`

---

## Kontext
Die CI-Workflows (`.github/workflows/rust-ci.yml`, `tsan.yml`, `loom-gate.yml`, `mutation-testing.yml`) steuern die kontinuierliche Verifikation und Regression-Gates des Contextra-Workspace.
Nach SPEC §7 (S-01) und B11/B14 sind folgende Erweiterungen vorgeschrieben:
1. `rust-ci.yml`: Ausführung von `cargo check --workspace --all-targets --locked` bei PRs, Matrix-Zeile für `docid-128` für alle weiterleitenden Crates sowie Verifikation des neuen Layering-Gates (G-L1).
2. Pfadfilter in `tsan.yml` und `loom-gate.yml`: Erweiterung um `mvcc`, `kvcache`, `checkpoint`, `ids`, `env`.
3. Mutation Testing Crate-Liste in `mutation-testing.yml` und `rust-ci.yml`: Aufnahme von `mvcc` und `kvcache`.

## Entscheidung
1. `.github/workflows/rust-ci.yml` führt nun die Jobs `layering-gate` und `check-docid-128` aus und schließt `mvcc` und `kvcache` in den Cross-Platform-Core-Test ein.
2. `.github/workflows/tsan.yml` und `loom-gate.yml` überwachen nun auch Änderungen in den Entkopplungs- und Concurrency-Crates (`mvcc`, `kvcache`, `checkpoint`, `ids`, `env`).
3. `.github/workflows/mutation-testing.yml` bezieht `mvcc` und `kvcache` in den wöchentlichen Schedule-Lauf ein.

## Konsequenzen
- Concurrency- und Layering-Regeln werden automatisch bei jedem relevanten PR geprüft.
- Feature-Matrix und `docid-128` Breitenumschaltung werden vor Drift geschützt.
