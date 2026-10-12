# Audit-Bericht: FFI-Boundary & Bindings `contextra-py`

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect für Contextra
**Ziel:** PyO3 Python-Bindings (`crates/contextra-py/src/`)
**Git HEAD:** `cf65dbcba86807c380a9dcbfd66523cb41528d18`
**Claim Ticket:** `ffi-audit`

---

## 1. Prüftabelle P1–P5

| Prüfpunkt | Thema | Status | Severity | Beleg (Datei:Zeile) & Befund |
|---|---|---|---|---|
| **P1** | **FFI-Panic-Boundary** | PASS | OK | `crates/contextra-py/src/bindings/common.rs:200-240`<br>`run_blocking_ffi` fängt Panics via `py.allow_threads` + `std::panic::catch_unwind` ab. Keine Rust-Panic überschreitet die FFI-Grenze; setzt `poisoned = true` und gibt PyRuntimeError zurück. |
| **P2** | **Panic-Profil** | PASS | OK | `Cargo.toml:202`<br>`[profile.release]` ist explizit auf `panic = "unwind"` konfiguriert (AGI/AGT-PY-d5d2be30, ADR-N04), um Panic-Containment an der FFI-Grenze zu gewährleisten. |
| **P3** | **NumPy-Vektor-Konversion** | PASS | OK | `crates/contextra-py/src/bindings/crud_macros.rs:18,48,78`<br>Nutzung von `PyReadonlyArray1<'py, f32>` erzwingt 1D-Arrays und Typ `f32`. `validate_vector` prüft auf Leere/NaN/Inf. Dimensionsprüfung erfolgt im Kern (`contextra-db`). |
| **P4** | **GIL-Management** | PASS | OK | `crates/contextra-py/src/bindings/common.rs:218`<br>`py.allow_threads` gibt den Python GIL während blockierender Rust Tokio-Runtime Aufrufe (`block_on`) frei. Deadlock-Prävention bei nebenläufigen Aufrufen bestanden (`test_gil_not_held_during_blocking_ops`). |
| **P5** | **Error-Mapping-Vollständigkeit** | PASS | OK | `crates/contextra-py/src/bindings/common.rs:280-320`<br>`contextra_err` mappt alle 22+ `ContextraError`-Varianten über `ContextraErrorDto` auf Python-Exceptions (`PyKeyError`, `PyRuntimeError`, `ContextraIOError`, `ContextraValueError`, `ContextraIndexError`, `ContextraCryptoError` etc.) inkl. Attribute (`kind`, `message`, `details`). |

---

## 2. Detaillierte Befunde nach Prüfpunkten

### (1) FFI-Panic-Boundary-Ergebnis (P1)
- **Implementierung:** In `crates/contextra-py/src/bindings/common.rs` ist `run_blocking_ffi` als generische FFI-Sicherheitskontrolle implementiert.
- **Panic Catching:** `std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))` umschließt jeden Rust-Aufruf.
- **Engine Poisoning:** Bei einer gefangenen Panic wird eine atomare `poisoned`-Flag (`AtomicBool`) auf `true` gesetzt. Folgeaufrufe auf derselben Instanz werden sofort mit `PyRuntimeError("engine poisoned after previous panic...")` abgewiesen.
- **Test-Nachweis:**
  - `test_run_blocking_ffi_panic_containment` in `bindings/tests.rs` (fängt simulierte Panics und verifiziert Poisoning).
  - `test_trigger_panic_helper_resume_unwind_containment` in `bindings/tests.rs`.
  - `cargo xtask check-ffi-panic-boundary` lief ohne Befund (`py-ffi-panic.log`).

### (2) Panic-Profil-Nachweis (P2)
- **Konfiguration:** `Cargo.toml` definiert Workspace-weit `[profile.release] panic = "unwind"`. Standalone-Binaries nutzen separat `[profile.release-abort]`.
- **Begründung:** Gemäß AGT-PY-d5d2be30 & ADR-N04 ist `panic = "unwind"` zwingend erforderlich, damit PyO3-Extension-Modules Panics vor der C-Grenze abfangen und kontrolliert in `PyErr` übersetzen können.

### (3) NumPy-Vektor-Konversion & Dimensionsprüfung (P3)
- **Typenbindung:** FFI-Methoden (`insert`, `update`, `upsert`, `search`, `hybrid_search`, `insert_many`, `upsert_many`) nutzen `PyReadonlyArray1<'py, f32>`. Falsche Dimensionen (z. B. 2D-Arrays) oder falsche dtypes (z. B. `float64`) werden direkt durch das PyO3/NumPy-Typsystem vor der Rust-Ausführung als Python `TypeError` / `PyValueError` abgewiesen.
- **Sicherheitsprüfung:** `validate_vector(&[f32])` prüft auf leere Vektoren sowie auf das Vorhandensein von `NaN`- oder `Inf`-Werten und gibt bei Verstoß `ContextraValueError` zurück.
- **Dimensionsabgleich:** Der Abgleich der Vektordimension gegen die konfigurierte Embedding-Dimension der Collection erfolgt in `contextra-db` / `Collection::insert`. Bei Abweichung wird `ContextraError::InvalidInput` erzeugt und über `contextra_err` sauber in `ContextraValueError` übersetzt.

### (4) GIL-Management (P4)
- **Freigabe:** `run_blocking_ffi` führt `py.allow_threads(...)` vor dem Eintritt in synchrone/blockierende Async-Operationen (`rt.block_on(...)`) aus.
- **Nebenläufigkeit:** Der CPython GIL wird während I/O- und Vektorsuch-Operationen freigegeben. Anderen Python-Threads wird die Ausführung erlaubt.
- **Test-Nachweis:** `test_gil_not_held_during_blocking_ops` in `bindings/tests.rs` startet 4 parallele Python-Worker-Threads mit `scan_prefix` und verifiziert, dass kein Thread blockiert oder in Timeouts läuft.

### (5) Error-Mapping-Vollständigkeit (P5)
- **Mapping:** `contextra_err(e: ContextraError) -> PyErr` wandelt den Fehler zuerst in einen DTO (`ContextraErrorDto`) um.
- **Exception-Hierarchie:**
  - `NotFound` -> `PyKeyError`
  - `Conflict`, `Transaction`, `TransactionTimeout` -> `PyRuntimeError`
  - `PolicyViolation`, `NamespaceViolation`, `Sandbox`, `MemoryLimitExceeded`, `SandboxTimeout` -> `PyPermissionError`
  - `InvalidInput`, `Serialization`, `Json`, `ParseError`, `Bincode`, `InvalidSequenceNumber`, `CheckpointNotFound`, `LimitExceeded` -> `ContextraValueError`
  - `Storage`, `Io`, `WalCorruption`, `ChecksumMismatch` -> `ContextraIOError`
  - `Index`, `HnswConnectivityDegraded`, `Text` -> `ContextraIndexError`
  - `Crypto` -> `ContextraCryptoError`
  - `MemoryBudgetExceeded` -> `PyMemoryError`
  - `CapabilityUnsupported` -> `PyNotImplementedError`
  - `Internal`, `Cluster` -> `ContextraInternalError`
  - Standard-Fallback -> `ContextraError`
- **Attribute:** Attacht dynamisch `kind`, `message` und optionale `details`-Attribute auf dem `PyErr`-Objekt.
- **Test-Nachweis:** `test_py_err_mapping_all_error_kinds`, `test_contextra_err_attributes_set` und `test_py_err_io_and_index_mappings` in `bindings/tests.rs`.

---

## 3. VERDICT & EVIDENCE

```
================================================================================
VERDICT: PASSED
================================================================================
Das FFI-Boundary-Audit für `crates/contextra-py/src/` bestätigt vollständige
FFI-Panicsicherheit (P1), die Einhaltung des Unwind-Panic-Profils (P2),
robuste NumPy-Typ- und Vektorvalidierung (P3), korrekte GIL-Freigabe während
I/O-Operationen (P4) sowie ein lückenloses Error-Mapping auf strukturierte
Python-Exceptions (P5).

[EVIDENCE: P1_FFI_PANIC_BOUNDARY_OK]
[EVIDENCE: P2_PANIC_PROFILE_UNWIND_OK]
[EVIDENCE: P3_NUMPY_VECTOR_CONVERSION_OK]
[EVIDENCE: P4_GIL_MANAGEMENT_OK]
[EVIDENCE: P5_ERROR_MAPPING_COMPLETE_OK]
[EVIDENCE: TEST_PASSED_RUST_UNIT_TESTS_OK]
[EVIDENCE: TEST_PASSED_PYTEST_67_PASSED]
================================================================================
```
