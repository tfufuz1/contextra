# Baseline: `contextra-types`

**Erhebungsdatum:** 2026-11-10
**Geltungsbereich:** `crates/contextra-types`
**Referenz:** `docs/decoupling/JULES_RAHMEN.md` (R6, R7), `docs/SPEC.md`

---

## 1. Paketübersicht & Metriken

- **Crate-Name:** `contextra-types`
- **Pfad:** `crates/contextra-types`
- **Kategorie / Ebene:** Ebene 2 (`contracts`)
- **Codeumfang:** 4.540 LOC (22 Quelldateien)
- **Interne Abhängigkeiten:** `contextra-ids` (1 interne Abhängigkeit)
- **Externe Abhängigkeiten:** `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes`
- **Dev-Abhängigkeiten:** `proptest`, `criterion`
- **Features:** `docid-128`, `auto-extraction-opt-out` (Standard: `default = []`)

---

## 2. Test- und Kompilierstatus

| Prüfung | Befehl | Status / Dauer | Anmerkungen |
|---|---|---|---|
| No-Default Build | `cargo check -p contextra-types --no-default-features --locked` | Grün (~7.5s) | Keine Fehler |
| Feature `docid-128` | `cargo check -p contextra-types --features docid-128 --locked` | Grün (~3.0s) | Schaltet `DocId` auf 128 Bit um |
| Feature `auto-extraction-opt-out` | `cargo check -p contextra-types --features auto-extraction-opt-out --locked` | Grün (~3.0s) | Extraktions-Opt-out aktiv |
| Unit- & Integrationstests (Default) | `cargo test -p contextra-types --all-targets --locked` | Grün (~43s) | 120 Unit-Tests + 13 Test-Suites bestanden (0 fehlgeschlagen, 1 ignored) |
| Tests mit `docid-128` | `cargo test -p contextra-types --features docid-128 --locked` | Grün (~17s) | 120 Unit-Tests + 13 Test-Suites bestanden |
| Clippy | `cargo clippy -p contextra-types --all-targets --locked -- -D warnings` | Grün (~17s) | Keine Warnungen oder Fehler |

---

## 3. Diagnose-Tools

- **Layering-Gate (G-L1):** `python3 tools/gate_layering.py --crate contextra-types` -> **GRÜN** (Keine unzulässigen Abhängigkeiten).
- **Zero-Panic Analyzer:** `python3 tools/zero_panic_analyzer.py --crate contextra-types` -> **0 Prod-Verstöße**.
- **Determinismus-Guard:** `python3 tools/determinism_guard.py --crate contextra-types` -> **0 Nicht-deterministische Stellen in Prod**.
- **Unsafe-Boundary Auditor:** `python3 tools/unsafe_boundary_auditor.py --crate contextra-types` -> **0 Unsafe außerhalb von Inseln**, `#![forbid(unsafe_code)]` vollständig aktiv.
- **Wire-Format & Golden Tests:** `tests/golden_bytes_suite.rs` und `tests/golden_ids_parity.rs` bestehen fehlerfrei (INV-1 Formatstabilität gewahrt).
