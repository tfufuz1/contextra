//! # Gemeinsame FFI-Hilfsfunktionen und Exception-Handling (`common`)
//!
//! Dieses Modul enthält grundlegende Bausteine für die PyO3-Schnittstelle:
//! - Definition spezifischer Python-Exceptions (`ContextraError`, `TenantError`, etc.).
//! - Validierungsroutinen für Dokument-IDs, Labels, Pfade und Vektordaten.
//! - Panikabsicherung an der FFI-Grenze (`run_blocking_ffi`) mit Poisoning-Schutz.
//! - GIL-freigebende asynchrone Kapselung (`py.allow_threads`).
//! - Serialisierung und Deserialisierung zwischen Python-Typen und Rust-Typen via `pythonize` und `serde_json`.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## FFI-Fehlerbehandlung (ADR-N04)
//! Rust-Fehler (`ContextraError`) werden systematisch in semantisch passende Python-Exceptions
//! transformiert, inklusive strukturierter Attribute wie `kind`, `message` und `details`.

use pyo3::exceptions::{
    PyIOError, PyIndexError, PyKeyError, PyPermissionError, PyRuntimeError, PyValueError,
};
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::bindings::document::PyDocument;
use crate::bindings::search_result::PySearchResult;

// ─── Custom Exceptions ──────────────────────────────────────────────────────

// ANCHOR[contextra-py:EXC-001]
pyo3::create_exception!(_contextra, ContextraError, pyo3::exceptions::PyException);
pyo3::create_exception!(_contextra, TenantError, ContextraError);
pyo3::create_exception!(_contextra, NotFoundError, ContextraError);
pyo3::create_exception!(_contextra, IntegrityError, ContextraError);
pyo3::create_exception!(_contextra, ContextraIOError, PyIOError);
pyo3::create_exception!(_contextra, ContextraIndexError, PyIndexError);
pyo3::create_exception!(_contextra, ContextraValueError, PyValueError);
pyo3::create_exception!(_contextra, ContextraCryptoError, ContextraError);
pyo3::create_exception!(_contextra, ContextraInternalError, PyRuntimeError);

// ─── Shared Constants ───────────────────────────────────────────────────────

/// Minimale Anzahl an Worker-Threads für die Tokio-Laufzeitumgebung.
pub const MIN_WORKER_THREADS: usize = 1;
/// Maximale Anzahl an Worker-Threads für die Tokio-Laufzeitumgebung.
pub const MAX_WORKER_THREADS: usize = 256;

/// Maximale Länge für Dokumenten-IDs (1024 Zeichen bzw. Bytes).
pub const MAX_ID_LENGTH: usize = 1024;
/// Maximale Länge für Collection-Namen (64 Zeichen).
pub const MAX_COLLECTION_NAME_LENGTH: usize = 64;
/// Maximale Länge für Relations-Labels (256 Zeichen).
pub const MAX_LABEL_LENGTH: usize = 256;
/// Maximale Batch-Größe für Massenoperationen (10.000 Einträge).
pub const MAX_BATCH_SIZE: usize = 10_000;

// ─── Shared Helper Functions ────────────────────────────────────────────────

// ANCHOR[contextra-py:HLP-001]
/// Konvertiert ein Python-Dictionary (`PyDict`) in einen `serde_json::Value`.
///
/// # Zweck
/// Wandelt dynamische Metadaten aus Python in das interne JSON-Format von Contextra um.
///
/// # Fehlerfälle
/// Liefert `PyValueError`, falls das Dictionary nicht JSON-serialisierbare Objekte enthält.
pub fn dict_to_json(d: &pyo3::Bound<'_, pyo3::types::PyDict>) -> PyResult<serde_json::Value> {
    depythonize(d).map_err(|e| PyValueError::new_err(format!("Metadata error: {}", e)))
}

// ANCHOR[contextra-py:HLP-002]
/// Konvertiert ein optionales Python-Dictionary in ein optionales `serde_json::Value`.
///
/// # Rückgabe
/// - `Ok(Some(Value))` wenn ein Dictionary übergeben wurde.
/// - `Ok(None)` wenn `None` übergeben wurde.
pub fn opt_dict_to_json(
    metadata: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Option<serde_json::Value>> {
    match metadata {
        Some(d) => Ok(Some(dict_to_json(d)?)),
        None => Ok(None),
    }
}

// ANCHOR[contextra-py:VAL-001]
/// Validiert, dass ein Beziehungs-Label nicht leer ist, keine Null-Bytes enthält und die Maximallänge nicht überschreitet.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError` bei ungültigem Label.
pub fn validate_label(label: &str) -> PyResult<()> {
    if label.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Relationship label cannot be empty or whitespace-only",
        ));
    }
    if label.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Relationship label cannot contain null bytes",
        ));
    }
    if label.len() > MAX_LABEL_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Relationship label exceeds maximum length of {} bytes. Got: {}",
            MAX_LABEL_LENGTH,
            label.len()
        )));
    }
    Ok(())
}

// ANCHOR[contextra-py:VAL-002]
/// Validiert, dass eine Dokumenten-ID nicht leer ist, keine Null-Bytes enthält und maximal 1024 Zeichen lang ist.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError` bei ungültiger ID.
pub fn validate_id(id: &str) -> PyResult<()> {
    if id.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Document ID cannot be empty or whitespace-only",
        ));
    }
    if id.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Document ID cannot contain null bytes",
        ));
    }
    if id.len() > MAX_ID_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Document ID exceeds maximum length of {} bytes. Got: {}",
            MAX_ID_LENGTH,
            id.len()
        )));
    }
    Ok(())
}

// ANCHOR[contextra-py:VAL-003]
/// Validiert, dass ein Collection-Name nicht leer ist, keine Null-Bytes enthält und maximal 64 Zeichen lang ist.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError` bei ungültigem Namen.
pub fn validate_collection_name(name: &str) -> PyResult<()> {
    if name.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Collection name cannot be empty or whitespace-only",
        ));
    }
    if name.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Collection name cannot contain null bytes",
        ));
    }
    if name.len() > MAX_COLLECTION_NAME_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Collection name exceeds maximum length of {} bytes. Got: {}",
            MAX_COLLECTION_NAME_LENGTH,
            name.len()
        )));
    }
    Ok(())
}

// ANCHOR[contextra-py:VAL-004]
/// Validiert, dass ein Datenbank-Dateipfad nicht leer ist und keine Null-Bytes enthält.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError` bei fehlerhaftem Pfadstring.
pub fn validate_db_path(path: &str) -> PyResult<()> {
    if path.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Database path cannot be empty or whitespace-only",
        ));
    }
    if path.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Database path cannot contain null bytes",
        ));
    }
    Ok(())
}

// ANCHOR[contextra-py:VAL-005]
/// Validiert, dass ein Volltext-Suchquery nicht leer ist und keine Null-Bytes enthält.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError` bei ungültigem Suchtext.
pub fn validate_query_text(text: &str) -> PyResult<()> {
    if text.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Search query text cannot be empty or whitespace-only",
        ));
    }
    if text.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Search query text cannot contain null bytes",
        ));
    }
    if text.len() > MAX_ID_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Query text exceeds maximum length of {} bytes. Got: {}",
            MAX_ID_LENGTH,
            text.len()
        )));
    }
    Ok(())
}

// ANCHOR[contextra-py:VAL-006]
/// Prüft, dass eine Batch-Größe größer 0 ist und das Limit von 10.000 Elementen nicht überschreitet.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError` bei leerer Batch oder Überschreitung des Limits.
pub fn validate_batch_size(size: usize) -> PyResult<()> {
    if size == 0 {
        return Err(ContextraValueError::new_err("Batch cannot be empty"));
    }
    if size > MAX_BATCH_SIZE {
        return Err(ContextraValueError::new_err(format!(
            "Batch size {} exceeds maximum allowed limit of {}",
            size, MAX_BATCH_SIZE
        )));
    }
    Ok(())
}

// ANCHOR[contextra-py:VAL-007]
/// Validiert, dass ein Vektor nicht leer ist und weder `NaN` noch unendliche Werte (`+/-Inf`) enthält.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError` bei leeren oder unendlichen Gleitkommazahlen.
pub fn validate_vector(vector: &[f32]) -> PyResult<()> {
    if vector.is_empty() {
        return Err(ContextraValueError::new_err("Vector cannot be empty"));
    }
    if vector.iter().any(|x| x.is_nan() || x.is_infinite()) {
        return Err(ContextraValueError::new_err(
            "Vector contains NaN or infinite float values",
        ));
    }
    Ok(())
}

// ANCHOR[contextra-py:VAL-008]
/// Validiert und extrahiert ein Dokument-ID-Objekt aus Python (erlaubt String oder nicht-negativen Integer).
///
/// # Ablauf
/// 1. Prüft, ob das Objekt ein String ist -> Validierung via `validate_id`.
/// 2. Prüft, ob das Objekt ein Integer ist -> Sicherstellung, dass `>= 0` und `< u64::MAX`.
/// 3. Gibt die ID als String zurück.
///
/// # Fehlerfälle
/// Liefert `ContextraValueError`, falls der Typ weder String noch Integer ist oder Schranken verletzt werden.
pub fn validate_id_obj(id_obj: &pyo3::Bound<'_, pyo3::types::PyAny>) -> PyResult<String> {
    if let Ok(id_str) = id_obj.extract::<String>() {
        validate_id(&id_str)?;
        Ok(id_str)
    } else if let Ok(id_int) = id_obj.extract::<i128>() {
        if id_int < 0 {
            return Err(ContextraValueError::new_err(
                "Document ID cannot be a negative integer",
            ));
        }
        #[cfg(not(feature = "docid-128"))]
        if id_int > (u64::MAX as i128) {
            return Err(ContextraValueError::new_err(
                "Document ID integer value exceeds maximum allowed bound (u64::MAX)",
            ));
        }
        Ok(id_int.to_string())
    } else if id_obj.is_instance_of::<pyo3::types::PyInt>() {
        Err(ContextraValueError::new_err(
            "Document ID integer value exceeds maximum allowed bound",
        ))
    } else {
        Err(ContextraValueError::new_err(
            "Document ID must be a string or non-negative integer",
        ))
    }
}

// ANCHOR[contextra-py:SUB-001]
/// Prüft explizit, ob das Crate innerhalb eines CPython Sub-Interpreters geladen wird.
///
/// # Zweck
/// Verhindert fehlerhafte Ausführung in isolierten Sub-Interpretern (Interpreter-ID != 0),
/// wenn keine vollständige Threadpool-Isolation garantiert werden kann.
///
/// # Fehlerfälle
/// Liefert `PyImportError`, falls `interpreter_id != 0`.
pub fn check_subinterpreter_guard(py: Python<'_>) -> PyResult<()> {
    let current_id: Option<i64> = if let Ok(interp_mod) = py.import("_xxsubinterpreters") {
        interp_mod
            .call_method0("get_current")
            .ok()
            .and_then(|id| id.extract().ok())
    } else if let Ok(interp_mod) = py.import("_interpreters") {
        interp_mod
            .call_method0("get_current")
            .ok()
            .and_then(|id| id.extract().ok())
    } else {
        None
    };

    if let Some(id) = current_id {
        if id != 0 {
            return Err(pyo3::exceptions::PyImportError::new_err(
                "contextra does not support loading in sub-interpreters due to per-process Tokio runtime isolation",
            ));
        }
    }
    Ok(())
}

// ANCHOR[contextra-py:FFI-001]
/// Führt einen blockierenden Aufruf sicher über die FFI-Grenze hinweg aus (mit Panic-Catch und Poisoning).
///
/// # Zweck & Invariante (ADR-N04 / APM-PY-A):
/// Verhindert, dass unkontrollierte Rust-Panics in den Python-Prozess durchschlagen.
///
/// # Ablauf in Schritten:
/// 1. **Poisoning-Prüfung:** Prüft das atomare `poisoned`-Flag. Falls zuvor ein Panic aufgetreten ist,
///    wird sofort mit `PyRuntimeError` abgebrochen (Fail-Closed).
/// 2. **GIL-Release & Panic Catch:** Gibt mit `py.allow_threads` den Python-GIL frei und führt die
///    Closure `f` gekapselt in `std::panic::catch_unwind` aus.
/// 3. **Ergebnis-Auswertung:**
///    - Bei Erfolg: Rückgabe des Werts.
///    - Bei Panic: Setzt `poisoned = true` via `SeqCst`, extrahiert die Paniknachricht und
///      liefert eine saubere `PyRuntimeError`-Exception an Python zurück.
///
/// # Nebenläufigkeit
/// Durch `py.allow_threads` können andere Python-Threads parallel weiterlaufen, während
/// Rust auf Dateisystem-I/O oder Vektorberechnungen wartet.
pub fn run_blocking_ffi<F, R>(py: Python<'_>, poisoned: &AtomicBool, f: F) -> PyResult<R>
where
    F: FnOnce() -> PyResult<R> + Send,
    R: Send,
{
    // Poison-nach-Panic-Invariante (APM-PY-A): Vor der Ausführung prüfen, ob das Handle vergiftet ist
    if poisoned.load(Ordering::SeqCst) {
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "engine poisoned after previous panic, create a new instance",
        ));
    }

    let panic_result =
        py.allow_threads(|| std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)));

    match panic_result {
        Ok(res) => res,
        Err(panic_payload) => {
            // NOTE: Atomares Vergiften via SeqCst stellt sicher, dass nachfolgende Threads sofort geblockt werden.
            poisoned.store(true, Ordering::SeqCst);
            let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic inside Rust core".to_string()
            };
            Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Rust panic caught at FFI boundary: {}",
                panic_msg
            )))
        }
    }
}

// ANCHOR[contextra-py:CNV-001]
/// Konvertiert einen `serde_json::Value` zurück in ein natives Python-Objekt (`PyObject`).
///
/// # Fehlerfälle
/// Liefert `PyRuntimeError`, falls die Konvertierung scheitert.
pub fn json_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<PyObject> {
    pythonize(py, val)
        .map(|o| o.unbind())
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Metadata error: {}", e)))
}

// ANCHOR[contextra-py:CNV-002]
/// Konvertiert ein `contextra_db::Document` in ein `PyDocument`.
///
/// # Ablauf
/// 1. Metadaten-JSON (falls vorhanden) in ein natives Python-Dictionary überführen.
/// 2. In eine `PyDocument`-Instanz kapseln.
pub fn doc_to_py(py: Python<'_>, d: contextra_db::Document) -> PyResult<PyDocument> {
    let meta_py = match d.metadata {
        Some(ref m) => Some(json_to_py(py, m)?),
        None => None,
    };
    Ok(PyDocument {
        id: d.id,
        metadata: meta_py,
    })
}

// ANCHOR[contextra-py:CNV-003]
/// Konvertiert einen Vektor von internen `contextra_db::SearchResult` in Python-taugliche `PySearchResult`.
///
/// # Ablauf
/// 1. Allokiert einen Vektor mit passender Kapazität.
/// 2. Iteriert über die Treffer, wandelt Metadaten in Python-Objekte um und erzeugt `PySearchResult`.
pub fn results_to_py(
    py: Python<'_>,
    results: Vec<contextra_db::SearchResult>,
) -> PyResult<Vec<PySearchResult>> {
    let mut py_res = Vec::with_capacity(results.len());
    for r in results {
        let meta_py = match r.metadata {
            Some(ref m) => Some(json_to_py(py, m)?),
            None => None,
        };
        py_res.push(PySearchResult {
            id: r.id,
            score: r.score,
            metadata: meta_py,
        });
    }
    Ok(py_res)
}

// ANCHOR[contextra-py:ERR-001]
/// Übersetzt ein typisiertes Rust `ContextraError` in eine strukturierte Python-Exception (`PyErr`).
///
/// # Ablauf
/// 1. Wandelt `ContextraError` in ein Data Transfer Object (`ContextraErrorDto`) um.
/// 2. Mappt die Fehlerkategorie (`kind`) auf die entsprechende Python-Exception-Klasse:
///    - `"NotFound"` -> `NotFoundError`
///    - `"Conflict"`, `"Transaction"` -> `IntegrityError`
///    - `"TenantMismatch"`, `"NamespaceViolation"` -> `TenantError`
///    - `"Storage"`, `"Io"`, `"WalCorruption"` -> `ContextraIOError`
///    - `"Index"`, `"Text"` -> `ContextraIndexError`
///    - `"Crypto"` -> `ContextraCryptoError`
///    - `"InvalidInput"` -> `ContextraValueError`
/// 3. Hängt Attribute (`kind`, `message`, `details`) an das Python-Fehlerobjekt für strukturierte Inspektion an.
pub fn contextra_err(e: contextra_types::ContextraError) -> PyErr {
    let dto = contextra_types::ContextraErrorDto::from(&e);
    Python::with_gil(|py| {
        let py_err = match dto.kind.as_str() {
            "NotFound" => NotFoundError::new_err(dto.message.clone()),
            "Conflict" | "Transaction" | "TransactionTimeout" => {
                IntegrityError::new_err(dto.message.clone())
            }
            "NamespaceViolation" | "TenantMismatch" => TenantError::new_err(dto.message.clone()),
            "PolicyViolation"
            | "NamespaceViolation"
            | "Sandbox"
            | "MemoryLimitExceeded"
            | "SandboxTimeout" => PyPermissionError::new_err(dto.message.clone()),
            "InvalidInput"
            | "Serialization"
            | "Json"
            | "ParseError"
            | "Bincode"
            | "InvalidSequenceNumber"
            | "CheckpointNotFound"
            | "LimitExceeded" => ContextraValueError::new_err(dto.message.clone()),
            "Storage" | "Io" | "WalCorruption" | "ChecksumMismatch" => {
                ContextraIOError::new_err(dto.message.clone())
            }
            "Index" | "HnswConnectivityDegraded" | "Text" => {
                ContextraIndexError::new_err(dto.message.clone())
            }
            "Crypto" => ContextraCryptoError::new_err(dto.message.clone()),
            "MemoryBudgetExceeded" => pyo3::exceptions::PyMemoryError::new_err(dto.message.clone()),
            "CapabilityUnsupported" => {
                pyo3::exceptions::PyNotImplementedError::new_err(dto.message.clone())
            }
            "Internal" | "Cluster" => ContextraInternalError::new_err(dto.message.clone()),
            _ => ContextraError::new_err(dto.message.clone()),
        };
        let value = py_err.value(py);
        if value.setattr("kind", dto.kind).is_err() {
            // Ignore non-fatal attribute attachment error if py_err instance doesn't support setattr
        }
        if value.setattr("message", dto.message).is_err() {
            // Ignore non-fatal attribute attachment error
        }
        if let Some(ref details) = dto.details {
            if let Ok(details_py) = json_to_py(py, details) {
                if value.setattr("details", details_py).is_err() {
                    // Ignore non-fatal attribute attachment error
                }
            }
        }
        py_err
    })
}
