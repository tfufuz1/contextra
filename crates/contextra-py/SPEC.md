# Spezifikation: `contextra-py`

**Ring:** Ring 4 (Boundary & Composition Roots)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.4
**Klasse:** C (Python Native Bindings)
**MSRV:** 1.89.0
**Code-Disziplin:** Safe FFI Facade, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-py` stellt PyO3-basierte Python-Bindings für Contextra bereit:
- **Scope:** Export von `Contextra`, `Collection`, `Document`, Vektorsuche und Transaktionsverwaltung an das Python-Ökosystem.
- **FFI-Sicherheit:** Übersetzung von Rust-`Result<T, ContextraError>` in Python-Exceptions ohne Panic über die C-ABI-Grenze hinweg.
- **Feature-Schema:** Unterstützt Standard (64-Bit DocId) und `docid-128` (128-Bit DocId) transparent.

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-py`:*
- **LOC:** 2.235 LOC über 16 Rust-Dateien.
- **Typen-Inventar:** 0 Traits, 8 Structs, 0 Enums, 61 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 3 interne Crates (`contextra-types`, `contextra-engine`, `contextra-db`).
- **Audit-Befund:** Saubere Trennung; pyo3 ist hinter Feature/Target-Builds separierbar.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `lib.rs` | PyO3-Modulinitialisierung (`contextra`) | `pymodule`, Exception-Registrierung |
| `db.rs` | PyContextra Wrapper-Klasse | `PyContextra`, `PyDatabaseConfig` |
| `collection.rs` | PyCollection Wrapper für CRUD und Hybridsuche | `PyCollection`, `PySearchResult` |
| `document.rs` | Konvertierung zwischen Python-Dicts und `Document` | `PyDocument`, `PyMetadata` |
| `error.rs` | Übersetzung von `ContextraError` in PyErr-Klassen | `PyContextraError`, `PyStorageError` |

---

## 4. Invarianten (INV-PY-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-PY-NO-PANIC-OVER-ABI` | Panics werden niemals über die Python C-ABI propagiert; alle Pfade nutzen `catch_unwind` bzw. `PyResult`. | `tests/test_py_exceptions.py` |
| `INV-PY-GIL-RELEASE` | Rechenintensive und blocking I/O Operationen (z. B. HNSW-Suche, Kompaktierung) geben den Python GIL frei (`py.allow_threads`). | `src/collection.rs` |
| `INV-PY-DOCID-PARITY` | Feature `docid-128` schaltet die ID-Breite auf 128-Bit synchron mit dem Rust-Kern um. | `cargo test -p contextra-py --features docid-128` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.4:
1. Feature `docid-128 = ["contextra-types/docid-128"]` strikt entkoppeln.
2. Unabhängige `cargo check`- und `maturin`-Buildpfade etablieren.
