# AGENTS.md — `contextra-py`

> **Ring 4** · Klasse Rand `[R]` · Reifegrad `stable` · Unsafe: nein · Rolle: PyO3-Python-Bindings (`_contextra`, abi3-py38) über der Rust-API.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-py --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `db`, `types`, `wire` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `types`, `ports`, `contextra` |
| Features | `docid-128`, `test-hooks` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Dep auf `db` statt auf die Fassade** (dokumentierte Ausnahme `ANCHOR:ARCH:DAG-003`).
- Workspace-Mitglied (nicht in `default-members`); Release-Profil workspace-weit `panic = "unwind"` für FFI. Frühere Angabe „nicht im Workspace“ ist überholt.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `bindings/` (`db`, `collection`, `document`, `search_result`, `hyperedge`, `crud_macros`, `runtime_state`, …) | PyO3-Klassen |
| `kv_links.rs`, `lib.rs` | Modulregistrierung |

## 4. Verträge & Invarianten

1. **GIL freigeben** bei I/O und Rechenlast (`py.allow_threads`).
2. **Panics an der Grenze abfangen** und in `PyErr` übersetzen; keine Unwinds nach Python.
3. `ContextraError` wird über `ContextraErrorDto` auf Python-Exceptions abgebildet (`NotFound→KeyError` etc.).
4. Vektoren als `PyReadonlyArray1<f32>` (Zero-Copy).
5. Globale Zustände (Runtime) isolieren (Sub-Interpreter, PEP 684).

## 5. Ports

Keine eigenen Ports.

## 6. Nebenläufigkeit & Performance

- Eigene Tokio-Runtime, unabhängig von Python-Threads; kein Lock über GIL-Grenzen.

## 7. Anti-Patterns

- `block_on` bei gehaltenem GIL.
- `Vec<f32>` statt NumPy-Array.

## 8. Tests & Messung

```bash
cargo test -p contextra-py --locked
cargo clippy -p contextra-py --all-targets -- -D warnings
```

- 13 Testdateien; CI `contextra-py-ci.yml`, Wheels `build-wheels.yml`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | auf Fassade `contextra` umstellen |
