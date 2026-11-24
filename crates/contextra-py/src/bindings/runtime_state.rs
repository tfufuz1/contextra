//! # Laufzeitumgebungs-Zustand für Python (`PyRuntimeState`)
//!
//! Dieses Modul verwaltet die per-Interpreter isolierte Tokio-Runtime (`tokio::runtime::Runtime`).
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Hintergrund & Relevanz (PEP 684)
//! In Python 3.12+ (Sub-Interpreters) teilen sich Sub-Interpreter unter Umständen
//! nicht denselben Speicher. Um Datenrennen und Ressourcen-Lecks zu verhindern,
//! wird der asynchrone Tokio-Threadpool an das jeweilige Python-Modul `_contextra` geheftet.
//! So hat jeder Python-Interpreter seine eigene, saubere Async-Laufzeitumgebung.

use pyo3::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::bindings::common::{MAX_WORKER_THREADS, MIN_WORKER_THREADS};

// ANCHOR[contextra-py:RNT-001]
/// Liest und begrenzt die Umgebungsvariable `CONTEXTRA_WORKER_THREADS` auf den Bereich `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]`.
///
/// # Zweck
/// Bestimmt die Anzahl der Hintergrund-Worker-Threads für die Tokio-Laufzeitumgebung.
///
/// # Ablauf
/// 1. Standardwert ermitteln: Halbe Anzahl verfügbarer CPU-Kerne (`available_parallelism`),
///    mindestens jedoch auf `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]` begrenzt (z. B. 2 bis 32).
/// 2. Liest die Umgebungsvariable `CONTEXTRA_WORKER_THREADS`.
/// 3. Parst den String zu `usize` und klemmt den Wert in die Schranken `MIN_WORKER_THREADS..=MAX_WORKER_THREADS`.
/// 4. Liefert bei Fehlern oder fehlendem Eintrag den berechneten Standardwert zurück.
///
/// # Garantien
/// Stellt sicher, dass die Tokio-Runtime niemals mit 0 Threads initialisiert wird (Fail-Safe).
pub fn parse_worker_threads_env() -> usize {
    let default_threads = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        / 2)
    .clamp(MIN_WORKER_THREADS, MAX_WORKER_THREADS);

    std::env::var("CONTEXTRA_WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|val| val.clamp(MIN_WORKER_THREADS, MAX_WORKER_THREADS))
        .unwrap_or(default_threads)
}

// ANCHOR[contextra-py:RNT-002]
/// Kapselt die Tokio-Laufzeitumgebung und die Thread-Konfiguration pro Python-Modul.
///
/// # Felder
/// - `runtime`: Referenzgezählter Handle (`Arc<Runtime>`) auf den Tokio-Multi-Thread-Pool.
/// - `worker_threads`: Anzahl konfigurierter Worker-Threads in diesem Pool.
#[pyclass(name = "RuntimeState")]
#[derive(Clone)]
pub struct PyRuntimeState {
    pub runtime: Arc<Runtime>,
    pub worker_threads: usize,
}

// ANCHOR[contextra-py:RNT-003]
/// Ruft die Tokio-Runtime für den aktuellen Python-Interpreter ab oder initialisiert sie bei Bedarf.
///
/// # Zweck
/// Stellt sicher, dass asynchrone Rust-Operationen in Python synchron ausgeführt werden können.
///
/// # Ablauf in Schritten:
/// 1. Sucht nach dem bereits geladenen Modul `contextra._contextra` oder `_contextra`.
/// 2. Prüft, ob das Modulattribut `_runtime_state` bereits existiert und extrahiert den Handle.
/// 3. Falls noch nicht vorhanden: Ermittelt Worker-Threads via `parse_worker_threads_env()`.
/// 4. Erzeugt eine neue Multi-Thread-Tokio-Runtime mit dem Thread-Präfix `contextra-py-worker`.
/// 5. Bindet die neue Runtime an das Modulattribut `_runtime_state` via `setattr`.
///
/// # Fehlerfälle
/// - `PyRuntimeError` bei Scheitern des Runtime-Builds oder beim Setzen des Modulattributs.
pub fn get_runtime(py: Python<'_>) -> PyResult<Arc<Runtime>> {
    let module = py
        .import("contextra._contextra")
        .or_else(|_| py.import("_contextra"))?;
    if let Ok(state_attr) = module.getattr("_runtime_state") {
        if let Ok(state) = state_attr.extract::<PyRef<'_, PyRuntimeState>>() {
            return Ok(state.runtime.clone());
        }
    }

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
        runtime: runtime.clone(),
        worker_threads,
    };

    let py_state = Py::new(py, state)?;
    module.setattr("_runtime_state", py_state).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Failed to attach '_runtime_state' to '_contextra' module: {}",
            e
        ))
    })?;

    Ok(runtime)
}
