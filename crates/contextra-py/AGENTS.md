# AGENTS.md — contextra-py
> Ring 4 · stable · Quelle: capabilities.toml · Spec: III.24 / K.21 / L.14

## 1. Zweck
PyO3 Python-Bindings für Contextra. Übersetzt die Rust-APIs (`contextra-db`, `contextra-types`) für Python.
Verwaltet den FFI-Panic-Schutz, GIL-Freigabe, Modul-Initialisierung und Sub-Interpreter-Isolierung.
Stellt sicher, dass keine Rust-Panics über die CPython FFI-Grenze hinweg entweichen.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | `#![deny(unsafe_code)]`, PyO3-Modulregistrierung `_contextra` |
| `bindings/common.rs` | FFI-Hilfsfunktionen, `run_blocking_ffi`, Panic-Containment und Error-DTO |
| `bindings/functions.rs` | Modulweiter Einstiegspunkt `open()` und Testhaken `_trigger_panic_for_test` |
| `bindings/db.rs`, `bindings/collection.rs` | PyO3-Klassen `PyContextra` und `PyCollection` für Datenbank-CRUD |
| `bindings/document.rs`, `bindings/search_result.rs` | PyO3-Klassen `PyDocument` und `PySearchResult` |
| `bindings/db_stats.rs`, `bindings/storage_stats.rs`, `bindings/vector_index_stats.rs` | Statistische PyO3-Klassen für Observability |
| `bindings/runtime_state.rs` | Verwaltet per-Interpreter Tokio-Runtime (`CONTEXTRA_WORKER_THREADS`) |
| `bindings/hyperedge.rs`, `bindings/crud_macros.rs` | Hyperkanten-Argumentvalidierung und Makros für CRUD-Methoden |
| `kv_links.rs` | String-Parsing für `LinkRelation` ohne Allokationen |

## 3. Invarianten

- **INV-PY-PANIC-BOUNDARY**: Rust-Panics dürfen NIEMALS die FFI-Grenze nach Python überschreiten (`run_blocking_ffi` fängt Panics ab und konvertiert in PyErr).
  *Prüfung*: `cargo xtask check-ffi-panic-boundary`
- **INV-PY-GIL-RELEASE**: Zeitintensive Datenbank-I/O oder Vektorsuchen geben den GIL frei (`py.allow_threads`).
  *Prüfung*: `cargo test -p contextra-py`
- **INV-PY-SUBINTERPRETER-GUARD**: Verhindert unzulässigen globalen Zustand bei CPython Sub-Interpretern.
  *Prüfung*: `cargo test -p contextra-py`

## 4. Verboten / Anti-Patterns

- **Keine rohen Rust-Panics**: Verwende niemals `panic!`, `unwrap()` oder `expect()` direkt im FFI-Pfad ohne `run_blocking_ffi`.
- **Kein GIL-Holding während I/O**: Führe keine blockierenden Vektorsuchen oder Disk-Reads durch, während der GIL gehalten wird.
- **Keine fehlende Input-Validierung**: Alle Eingabestrings und Vektoren müssen mit `validate_id`, `validate_vector` etc. vor der FFI-Übergabe geprüft werden.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Der per-Interpreter Tokio-Runtime-Pool wird in `PyRuntimeState` isoliert gehalten.
- Python-Aufrufe sperren keine globalen Rust-Mutexes über GIL-Freigabe-Grenzen hinweg.
- Worker-Threads werden über die Umgebungsvariable `CONTEXTRA_WORKER_THREADS` konfiguriert und gedeckelt.

## 6. Verifikation

```bash
cargo xtask check-ffi-panic-boundary
cargo test -p contextra-py
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-py
```

## 7. Bekannte Lücken / SOLL

- Das Crate liegt aus historischen Panic-Profile-Gründen (`unwind`) außerhalb der Root `default-members`.
