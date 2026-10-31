//! # Freistehende FFI-Funktionen (`functions`)
//!
//! Dieses Modul exportiert freistehende Funktionen für den Python-Namespace,
//! allen voran die zentrale Einstiegsfunktion `open()`.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Funktionen
//! - `open(path, ...)`: Öffnet oder erstellt eine Contextra-Datenbank auf der Festplatte.
//! - `trigger_panic_helper` / `_trigger_panic_for_test`: Interne Testhooks zur Absicherung
//!   der FFI-Panikgrenze (ADR-N04).

use contextra_db::{Contextra, ContextraConfig};
use pyo3::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::bindings::common::*;
use crate::bindings::db::PyContextra;
use crate::bindings::runtime_state::get_runtime;

// ANCHOR[contextra-py:FN-001]
/// Öffnet oder erstellt eine Contextra-Datenbank am angegebenen Dateipfad.
///
/// # Parameter
/// - `path`: Dateipfad zum Datenbankverzeichnis (wird validiert, keine Null-Bytes).
/// - `dimension`: Vektordimension (Standard: 768, passend für Standardmodelle wie `nomic-embed-text`; Schranke [1, 10000]).
/// - `max_elements`: Optionale Obergrenze für Elemente im HNSW-Index (> 0).
/// - `encryption_passphrase`: Optionale Passphrase zur transparenten Verschlüsselung der Datenbank.
/// - `distance_metric`: Distanzmetrik für Vektoren ("cosine", "euclidean"/"l2", "dot"/"dotproduct").
///
/// # Ablauf in Schritten:
/// 1. Pfad und Vektordimension prüfen (`validate_db_path`, Bounds-Check).
/// 2. Tokio-Runtime für den aktuellen Interpreter abrufen (`get_runtime`).
/// 3. Datenbank-Konfiguration (`ContextraConfig`) mit Dimension, Passphrase und Metrik befüllen.
/// 4. Atomares Vergiftungsflag (`poisoned`) initialisieren.
/// 5. Asynchron `Contextra::open_with_config` unter GIL-Release und Panic-Schutz aufrufen.
/// 6. `PyContextra`-Handle erzeugen und an Python zurückgeben.
///
/// # Fehlerfälle
/// - `PyValueError` bei ungültigem Pfad, Dimension außerhalb [1, 10000] oder unbekannter Metrik.
/// - `ContextraIOError` / `ContextraError` bei Dateisystem- oder Speicherfehlern.
#[pyfunction]
#[pyo3(signature = (path, dimension=768, max_elements=None, encryption_passphrase=None, distance_metric=None))]
pub fn open(
    py: Python<'_>,
    path: &str,
    dimension: usize,
    max_elements: Option<usize>,
    encryption_passphrase: Option<String>,
    distance_metric: Option<String>,
) -> PyResult<PyContextra> {
    validate_db_path(path)?;
    if dimension == 0 || dimension > 10_000 {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Dimension must be between 1 and 10000. Got: {}",
            dimension
        )));
    }
    let rt = get_runtime(py)?;
    let worker_threads = rt.metrics().num_workers();
    let mut config = ContextraConfig {
        dimension,
        encryption_passphrase,
        ..Default::default()
    };

    if let Some(me) = max_elements {
        if me == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "max_elements must be greater than 0",
            ));
        }
        config.max_elements = me;
    }

    if let Some(dm) = distance_metric {
        config.distance_metric = match dm.to_lowercase().as_str() {
            "cosine" => contextra_db::DistanceMetric::Cosine,
            "euclidean" | "l2" => contextra_db::DistanceMetric::Euclidean,
            "dot" | "dotproduct" => contextra_db::DistanceMetric::DotProduct,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unsupported distance metric: {}",
                    dm
                )))
            }
        };
    }

    let poisoned = Arc::new(AtomicBool::new(false));
    let path_string = path.to_string();
    let db = run_blocking_ffi(py, &poisoned, || {
        rt.block_on(Contextra::open_with_config(path_string, config))
            .map_err(contextra_err)
    })?;

    Ok(PyContextra::new(Arc::new(db), rt, worker_threads, poisoned))
}

// ANCHOR[contextra-py:FN-002]
/// Interner Hilfsaufruf zum gezielten Auslösen eines Panics innerhalb von `run_blocking_ffi`.
///
/// # Zweck
/// Dient rein als Testhook zur Verifikation der FFI-Panik-Isolation (`catch_unwind`).
///
/// # Panics
/// Löst bewusst einen Panic-Unwind via `std::panic::resume_unwind` aus.
#[doc(hidden)]
pub(crate) fn trigger_panic_helper(
    py: Python<'_>,
    poisoned: &AtomicBool,
    message: Option<String>,
) -> PyResult<()> {
    let msg = message.unwrap_or_else(|| "Test panic for FFI isolation".to_string());
    run_blocking_ffi(py, poisoned, move || -> PyResult<()> {
        std::panic::resume_unwind(Box::new(msg));
    })
}

// ANCHOR[contextra-py:FN-003]
/// Python-Funktion zum Testen der Panik-Isolation (nur bei Feature `test-hooks`).
///
/// # Zweck
/// Wird von der Python-Testsuite aufgerufen, um zu verifizieren, dass ein Rust-Panic
/// sauber als `PyRuntimeError` im Python-Interpreter ankommt.
#[doc(hidden)]
#[cfg(feature = "test-hooks")]
#[pyfunction]
#[pyo3(signature = (message=None))]
pub fn _trigger_panic_for_test(py: Python<'_>, message: Option<String>) -> PyResult<()> {
    let dummy_poison = AtomicBool::new(false);
    trigger_panic_helper(py, &dummy_poison, message)
}
