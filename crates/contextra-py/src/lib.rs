// FILE-CONTEXT
// REVIEW-PASS[1/2] Systematischer Review der contextra-py PyO3 FFI Bindings (PRÜFER-KONTEXT: FRESH) (TS: 2026-09-16T16:47:30Z) (SESSION: 29edb6a4)
// STAND: 2026-09-10T19:22:55Z (SESSION: 0b2ff57d)
// ZWECK: PyO3 FFI bindings bridging Contextra embedded vector DB functionality to Python.
// INVARIANTEN: Zero Rust panics cross FFI boundary; GIL released during block_on async calls; Tokio Runtime bound per interpreter module state.
// NICHT-OFFENSICHTLICH: Per-interpreter Tokio runtime instance attached to Python module state (`PyRuntimeState`) to enforce sub-interpreter isolation (PEP 684).
// HOTSPOTS: [160-205] contextra_err mapping, [270-650] CRUD & search methods FFI boundary validation.
// SIEHE AUCH: crates/contextra-db/AGENTS.md

//! # Contextra Python Bindings (`contextra-py`)
//!
//! Dieses Crate stellt die offizielle Python-Brücke (FFI = Foreign Function Interface)
//! für die eingebettete 4-Signal Hybrid-Search Vektordatenbank **Contextra** bereit.
//! Es verbindet das Python-Ökosystem via PyO3 mit dem Rust-Kern (`contextra-db`, `contextra-store`).
//!
//! ## Ring-Zugehörigkeit & Architektur
//!
//! - **Ring / Klasse:** Ring 4 (Ränder / FFI-Bindings). Laut SPEC §5.2 / CLAUDE.md ist
//!   `contextra-py` zwar formell archiviert bzw. an der Peripherie angesiedelt, wird jedoch
//!   weiterhin als vollwertiges Workspace-Mitglied geführt.
//! - **Abhängigkeiten (eingehend):** Keine Workspace-Crates hängen von `contextra-py` ab.
//!   Die Nutzung erfolgt ausschließlich durch Python-Clients / User-Space.
//! - **Abhängigkeiten (ausgehend):** `contextra-types`, `contextra-store`, `contextra-db`,
//!   `contextra-rank`, `contextra-adapt`, `contextra-wire` sowie Third-Party Crates wie
//!   `pyo3`, `numpy`, `tokio`, `parking_lot`, `flatbuffers`, `serde_json`, `pythonize`.
//!
//! ## Wichtige Invarianten für Entwickler
//!
//! 1. **Keine Panics über FFI-Grenzen (ADR-N04):** Rust-Panics dürfen unter keinen Umständen
//!    in den Python-Interpreter durchschlagen (dies würde den gesamten Prozess crashen).
//!    Alle Aufrufe werden über `run_blocking_ffi` abgesichert und in strukturierte Python-Exceptions
//!    konvertiert.
//! 2. **GIL-Release (Global Interpreter Lock):** Während I/O-lastiger oder rechenintensiver
//!    Rust-Aufrufe (`block_on`) wird der Python GIL via `py.allow_threads` freigegeben, damit
//!    andere Python-Threads nicht blockiert werden.
//! 3. **Sub-Interpreter-Isolation (PEP 684):** Die Tokio-Runtime (`PyRuntimeState`) wird
//!    pro Python-Modulinstanz verwaltet, um globale Zustände zu vermeiden.
//! 4. **Zero-Copy Vektor-Umschlag:** Input-Vektoren werden über `numpy::PyReadonlyArray1<f32>`
//!    als `&[f32]` ohne Kopie direkt an die Speicher-/Indexschichten übergeben.

#![forbid(unsafe_code)]

use pyo3::prelude::*;
use std::sync::Arc;

pub mod bindings;
pub mod kv_links;

pub use bindings::*;

// ANCHOR[contextra-py:MOD-001]
/// Initialisiert das native PyO3-Modul `_contextra` für den Python-Interpreter.
///
/// # Zweck
/// Diese Funktion wird beim Import des Moduls in Python (`import _contextra`) automatisch
/// von der Python C-API aufgerufen. Sie initialisiert den pro Interpreter isolierten
/// Tokio-Threadpool, registriert Kernklassen, Exceptions und Modulfunktionen.
///
/// # Ablauf in Einzelschritten:
/// 1. **Sub-Interpreter Prüfung:** Stellt sicher, dass das Modul im Haupt-Interpreter oder
///    in einer kompatiblen Umgebung geladen wird (`check_subinterpreter_guard`).
/// 2. **Metadaten:** Setzt die Modulversion `__version__`.
/// 3. **Worker-Threads ermitteln:** Liest optional `CONTEXTRA_WORKER_THREADS` aus der Umgebung.
/// 4. **Tokio Runtime erstellen:** Erzeugt eine dedizierte Multi-Thread Tokio-Runtime
///    (`contextra-py-worker`), die asynchrone Rust-Operationen synchron kapselt.
/// 5. **Runtime-State binden:** Speichert die Runtime in `_runtime_state` im Python-Modulobjekt,
///    sodass Sub-Interpreter isoliert bleiben.
/// 6. **Klassen & Funktionen exportieren:** Registriert `open`, `PyContextra`, `PyCollection`,
///    Such- und Dokumentenmodelle sowie Statistik-Klassen.
/// 7. **Custom Exceptions registrieren:** Exportiert `ContextraError`, `TenantError`,
///    `NotFoundError`, etc. in den Python-Namespace.
///
/// # Fehlerfälle
/// - Gibt `PyRuntimeError` zurück, falls die Tokio-Runtime nicht erzeugt werden kann (z.B.
///   bei fehlenden OS-Ressourcen für Thread-Erstellung).
/// - Gibt `PyErr` zurück, falls die Typ- oder Klassenregistrierung im Modul scheitert.
#[pymodule]
fn _contextra(_py: Python<'_>, m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    check_subinterpreter_guard(_py)?;
    m.add("__version__", "0.1.0")?;

    // NOTE: Pro Interpreter wird eine eigene Tokio-Runtime erzeugt (PEP 684),
    // um Threadpool-Lecks und gegenseitige Blockaden zwischen Sub-Interpretern zu verhindern.
    let worker_threads = parse_worker_threads_env();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .thread_name("contextra-py-worker")
        .enable_all()
        .build()
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to create tokio runtime for contextra-py: {}",
                e
            ))
        })?;

    let runtime = Arc::new(rt);
    let state = PyRuntimeState {
        runtime,
        worker_threads,
    };
    m.add("_runtime_state", Py::new(_py, state)?)?;

    m.add_function(wrap_pyfunction!(open, m)?)?;
    #[cfg(feature = "test-hooks")]
    m.add_function(wrap_pyfunction!(_trigger_panic_for_test, m)?)?;
    m.add_class::<PyContextra>()?;
    m.add_class::<PyCollection>()?;
    m.add_class::<PySearchResult>()?;
    m.add_class::<PyDocument>()?;
    m.add_class::<PyVectorIndexStats>()?;
    m.add_class::<PyStorageStats>()?;
    m.add_class::<PyDbStats>()?;

    // Exceptions
    m.add("ContextraError", _py.get_type::<ContextraError>())?;
    m.add("TenantError", _py.get_type::<TenantError>())?;
    m.add("NotFoundError", _py.get_type::<NotFoundError>())?;
    m.add("IntegrityError", _py.get_type::<IntegrityError>())?;
    m.add("ContextraIOError", _py.get_type::<ContextraIOError>())?;
    m.add("ContextraIndexError", _py.get_type::<ContextraIndexError>())?;
    m.add("ContextraValueError", _py.get_type::<ContextraValueError>())?;
    m.add(
        "ContextraCryptoError",
        _py.get_type::<ContextraCryptoError>(),
    )?;
    m.add(
        "ContextraInternalError",
        _py.get_type::<ContextraInternalError>(),
    )?;

    Ok(())
}
