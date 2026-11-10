# BASELINE `contextra-privacy` — Vorher-Messung

Datum: 2026-11-10
Commit: HEAD (`f0eda1cb9`) / Branch: `jules-11352313584082305808-a7e1ac90`
Auftrag: `privacy — Doku-Korrektur und Feature-Hygiene für contextra-privacy`

---

## 1. Ausgangslage & Befunde
- **Crate:** `crates/contextra-privacy`
- **Autarkie:** Crate besitzt in `Cargo.toml` 0 interne Abhängigkeiten zu `contextra-*`.
- **Falscher Docstring:** `src/lib.rs:16` behauptet fälschlich: „Ausgehend: Hängt ab von contextra-types, contextra-ports“.
- **Feature-Audit:** Deklariert sind `std`, `serde`, `tracing`, `inspect`, `experimental`. `inspect` und `experimental` besitzen keine `cfg`-Nutzung und werden nirgends im Workspace weitergeleitet.
- **Clippy-Vorabbefund:** `cargo clippy -p contextra-privacy --all-targets --locked -- -D warnings` scheitert in `tests/pattern_recognizer_and_property_tests.rs:303, 314, 325` an `std::time::Instant::now` (Clippy P28-Lint `clippy::disallowed_methods`). In diesem Test wird die lineare Laufzeit / ReDoS-Resistenz gemessen. Dieser Test benötigt `#[allow(clippy::disallowed_methods)]` zur Bereinigung.

---

## 2. Gemessene Kennzahlen (Vorher-Zustand)

| Prüfung | Befehl | Ergebnis / Dauer | Status |
|---|---|---|---|
| No-Default Build | `cargo check -p contextra-privacy --no-default-features --locked` | 1.18s | ✅ PASSED |
| Default Build | `cargo check -p contextra-privacy --locked` | 0.42s | ✅ PASSED |
| Tests | `cargo test -p contextra-privacy --locked` | 149 Tests (Unit, Doc, Integration) in ~2.0s | ✅ PASSED |
| Standalone Export (Gate G3) | `python3 tools/export_crate.py contextra-privacy` | Standalone-Build & Check in temporärem Verzeichnis | ✅ PASSED |
| Zero-Panic Analyzer | `python3 tools/zero_panic_analyzer.py --crate contextra-privacy` | 0 Prod-Verstöße | ✅ PASSED |
| Determinism Guard | `python3 tools/determinism_guard.py --crate contextra-privacy` | 0 Nicht-deterministische Stellen in Prod | ✅ PASSED |
| Gate G1 (Dependencies) | `python3 tools/gate_dependencies.py` | 0 Verstöße für contextra-privacy | ✅ PASSED |
| Clippy (All Targets) | `cargo clippy -p contextra-privacy --all-targets --locked -- -D warnings` | 3 Fehler in `pattern_recognizer_and_property_tests.rs` (`Instant::now`) | ❌ FAILED (Vorabbefund) |

---

## 3. Vorabbefund-Korrektur
`pattern_recognizer_and_property_tests.rs:298`:
Attribut `#[allow(clippy::disallowed_methods)]` wird hinzugefügt, um Clippy für den ReDoS-Laufzeittest freizugeben.
