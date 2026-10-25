# Contextra — Projekt-Basislinie (Baseline)

## 1. Metadaten

- **Commit-Hash:** `922f4098f8a3c5ea1423b622d80618cb0045fa7e`
- **Toolchain:** Rust 1.89 (`rust-toolchain.toml`: `channel = "1.89"`)
- **Datum:** 2026-10-25
- **Zählwerkzeug:** `tools/loc.py`

---

## 2. Zählregeln (Line Counting Rules)

Gemäß Spezifikation misst `tools/loc.py` ausschließlich **Code-Zeilen** von Rust-Quellcode-Dateien:
1. **Zielbereich:** Alle Crate-Verzeichnisse unter `crates/`.
2. **Dateifilter:** Nur Dateien mit Endung `.rs`.
3. **Ausschlussfilter:**
   - Pfade, die `/tests/` oder `/benches/` enthalten.
   - Dateien/Pfade unter `target/`.
   - Dateinamen, die auf `_test.rs` oder `_tests.rs` enden.
4. **Zeilendefinition (Code-Zeile):** Eine Zeile gilt als Code-Zeile, wenn sie nach dem Parsen von Whitespace, einzeiligen Kommentaren (`//`) und verschachtelten Blockkommentaren (`/* ... */`) noch effektiven Rust-Code enthält. Leere Zeilen und reine Kommentarzeilen werden nicht gezählt.

---

## 3. Summen der Kern-Crates (Kernprodukt)

| Kern-Crate | Zweck im Kernprodukt | Code-Zeilen (LOC) | Dateianzahl |
|---|---|---|---|
| `contextra` | Facade Crate & Builder | 850 | 6 |
| `contextra-types` | Domänenmodelle & Fehlertypen | 4.248 | 17 |
| `contextra-ports` | Abstraktionsschnittstellen & Trait-Ports | 2.589 | 21 |
| `contextra-mvcc` | Multi-Version Concurrency Control | 2.475 | 5 |
| `contextra-crypto` | Cryptographic Primitives & Keyrings | 6.105 | 20 |
| `contextra-vector` | HNSW-Vektorindex & Distanzmetriken | 11.314 | 43 |
| `contextra-text` | Invertierter BM25-Index & Tokenizer | 4.008 | 22 |
| `contextra-store` | LSM-Engine, WAL & SSTable Storage | 15.584 | 66 |
| `contextra-checkpoint` | Persistent Checkpoints & State Snapshots | 3.377 | 11 |
| `contextra-engine` | Engine Orchestrator & Collection Drivers | 12.891 | 65 |
| `contextra-db` | High-Level Datenbank-API Handle | 2.081 | 9 |
| `contextra-mcp` | Model Context Protocol Server & Tools | 6.035 | 27 |
| **Summe Kernprodukt** | **12 Kern-Crates** | **71.557** | **312** |

*(Hinweis: Gesamtsumme über alle 33 Workspace-Crates unter `crates/` beträgt **124.776** Code-Zeilen in 515 Dateien).*

---

## 4. Gate-Stand Audit (`governance/gates.toml`)

Nachfolgend ist der Ist-Stand aller 44 in `governance/gates.toml` deklarierten Gates erfasst. Gemäß Zielvereinbarung wurden keine Gates oder Quellcodedateien außerhalb des Geltungsbereichs verändert.

| Gate-Name | Typ / Kind | Befehl / Aufruf | Ergebnis | Fehlerbeschreibung / Ursache |
|---|---|---|---|---|
| `fmt` | `cmd` | `cargo fmt --check` | **FAIL** | Unformatierte Zeilen in `xtask/src/core/cmd.rs`, `constitution.rs` etc. |
| `clippy-lintset` | `cmd` | `cargo clippy --workspace --all-targets --locked` | **FAIL** | `xtask` Kompilierfehler: `harness_generated.rs` fehlt. |
| `ring-dag` | `builtin` | `cargo xtask run-gate ring-dag` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `ring0-async` | `builtin` | `cargo xtask run-gate ring0-async` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `capabilities-consistency` | `builtin` | `cargo xtask run-gate capabilities-consistency` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `unsafe-inventory` | `builtin` | `cargo xtask run-gate unsafe-inventory` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `public-api` | `cmd` | `cargo public-api` | **FAIL** | Tool `cargo-public-api` nicht im PATH installiert. |
| `feature-matrix` | `builtin` | `cargo xtask run-gate feature-matrix` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `wired` | `builtin` | `cargo xtask run-gate wired` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `security-ports` | `builtin` | `cargo xtask run-gate security-ports` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `key-purposes` | `builtin` | `cargo xtask run-gate key-purposes` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `witness-mint` | `builtin` | `cargo xtask run-gate witness-mint` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `deletion-types-once` | `builtin` | `cargo xtask run-gate deletion-types-once` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `deletion-honesty` | `builtin` | `cargo xtask run-gate deletion-honesty` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `doc-truth` | `builtin` | `cargo xtask run-gate doc-truth` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `symbol-uniqueness` | `builtin` | `cargo xtask run-gate symbol-uniqueness` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `code-lints` | `builtin` | `cargo xtask run-gate code-lints` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `ratchet` | `builtin` | `cargo xtask run-gate ratchet` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `determinism` | `builtin` | `cargo xtask run-gate determinism` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `protected-paths` | `builtin` | `cargo xtask run-gate protected-paths` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `gate-weakening` | `builtin` | `cargo xtask run-gate gate-weakening` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `test-integrity` | `builtin` | `cargo xtask run-gate test-integrity` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `scope-guard` | `builtin` | `cargo xtask run-gate scope-guard` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `provenance` | `builtin` | `cargo xtask run-gate provenance` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `claims-in-pr` | `builtin` | `cargo xtask run-gate claims-in-pr` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `veto-guard` | `builtin` | `cargo xtask run-gate veto-guard` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `adr-index` | `builtin` | `cargo xtask run-gate adr-index` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `deny` | `cmd` | `cargo deny check` | **FAIL** | Bans-, Licences- und Advisory-Prüfungen fehlgeschlagen. |
| `audit` | `cmd` | `cargo audit` | **FAIL** | 33 Sicherheitswarnungen/Vulnerabilities in Abhängigkeiten gefunden. |
| `sbom` | `builtin` | `cargo xtask run-gate sbom` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `format-corpus` | `cmd` | `cargo check --tests` | **PASS** | Erfolgreich ausgeführt (Ausgabe in `gate_format_corpus.txt` gespeichert). |
| `merge-result-build` | `cmd` | `cargo check --workspace --all-targets --locked` | **FAIL** | `xtask` Kompilierfehler (`harness_generated.rs` fehlt). |
| `gate-selftests` | `builtin` | `cargo xtask run-gate gate-selftests` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `catalog-sync` | `builtin` | `cargo xtask run-gate catalog-sync` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `evidence-complete` | `builtin` | `cargo xtask run-gate evidence-complete` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `ci-hygiene` | `builtin` | `cargo xtask run-gate ci-hygiene` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `fuzz-matrix` | `xtask-heavy` | `cargo xtask fuzz-matrix` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `fuzz-regression` | `xtask-heavy` | `cargo xtask fuzz-regression` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `miri` | `cmd` | `cargo miri test` | **FAIL** | Nightly-Toolchain / Miri-Komponente nicht installiert. |
| `loom-inventory` | `xtask-heavy` | `cargo xtask loom-inventory` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `mutation` | `cmd` | `cargo mutants` | **FAIL** | Tool `cargo-mutants` nicht im PATH installiert. |
| `chaos` | `xtask-heavy` | `cargo xtask chaos` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `bench-regression` | `xtask-heavy` | `cargo xtask bench-regression` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
| `platform-matrix` | `builtin` | `cargo xtask run-gate platform-matrix` | **FAIL** | `xtask` lässt sich nicht kompilieren (`harness_generated.rs` fehlt). |
