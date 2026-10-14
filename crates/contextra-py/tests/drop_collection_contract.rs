// FILE-CONTEXT (STAND 2026-10-13, ZWECK: Integrationstest für drop_collection Vertrag, INVARIANTEN: INV-PY-PANIC-BOUNDARY, INV-DELETION-1)

#![forbid(unsafe_code)]

use contextra_db::{Contextra, ContextraConfig};
use numpy::PyArrayMethods;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::runtime::Runtime;

use _contextra::bindings::db::PyContextra;

fn setup_py_contextra(name: &str) -> (PyContextra, PathBuf, Arc<Runtime>) {
    let dir =
        std::env::temp_dir().join(format!("contextra_py_test_{}_{}", name, std::process::id()));
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime"),
    );

    let path_str = dir.to_str().expect("Path UTF-8").to_string();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = rt.block_on(async {
        Contextra::open_with_config(path_str, config)
            .await
            .expect("Failed to open Contextra DB")
    });

    let poisoned = Arc::new(AtomicBool::new(false));
    let py_db = PyContextra::new(Arc::new(db), rt.clone(), 2, poisoned);
    (py_db, dir, rt)
}

#[test]
fn test_drop_collection_docstring_contract() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let (py_db, dir, _rt) = setup_py_contextra("docstring");

        let py_db_bound = Bound::new(py, py_db).expect("Bound PyContextra");
        let py_attr = py_db_bound
            .getattr("drop_collection")
            .expect("getattr drop_collection");
        let doc_attr = py_attr.getattr("__doc__").expect("getattr __doc__");
        let doc: String = doc_attr.extract().expect("extract doc string");

        // Must NOT claim unconstrained physical deletion or HMAC-SHA256 stub
        assert!(
            !doc.contains("removing all its data from storage"),
            "Docstring must not claim unconstrained physical deletion without limitations"
        );
        assert!(
            !doc.contains("HMAC-SHA256 v2 with a stub integrity warning"),
            "Docstring must not claim outdated HMAC-SHA256 stub semantics"
        );

        // Must contain accurate non-guarantee and logical deletion statements
        assert!(
            doc.contains("Limitations & Non-Guarantees"),
            "Docstring must explain limitations and non-guarantees"
        );
        assert!(
            doc.contains("logical LSM deletion"),
            "Docstring must specify logical LSM deletion semantics"
        );
        assert!(
            doc.contains("active MemTable"),
            "Docstring must specify active MemTable tombstoning"
        );

        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn test_drop_collection_proof_status_and_scope_fields() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let (py_db, dir, _rt) = setup_py_contextra("proof_fields");

        // Create a test collection and insert a document first
        let col = py_db
            .collection("col_test", py)
            .expect("Collection creation");
        let vec = numpy::PyArray1::from_vec(py, vec![0.1f32, 0.2, 0.3, 0.4]);
        let vec_bound = vec.readonly();
        let doc_id = PyString::new(py, "doc1").into_any();
        col.insert(py, &doc_id, vec_bound, None)
            .expect("Document insert");

        // Execute drop_collection with a valid 32-byte proof key
        let key_bytes = PyBytes::new(py, &[0x55u8; 32]);
        let res = py_db.drop_collection("col_test", Some(&key_bytes), py);

        match res {
            Ok(proof_json) => {
                let val: serde_json::Value =
                    serde_json::from_str(&proof_json).expect("Proof JSON must be valid");
                let obj = val.as_object().expect("Proof JSON must be an object");
                assert_eq!(
                    obj.get("proof_status").and_then(|v| v.as_str()),
                    Some("issued"),
                    "Proof JSON must include additive proof_status = issued"
                );
                assert!(
                    obj.contains_key("scope"),
                    "Proof JSON must include scope field"
                );
            }
            Err(err) => {
                let err_msg = err.to_string();
                assert!(
                    err_msg.contains("Capability unsupported: deletion_proof")
                        || err_msg.contains("encryption-at-rest"),
                    "If drop_collection fails due to unconfigured feature, error must be CapabilityUnsupported: {}",
                    err_msg
                );
            }
        }

        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn test_drop_collection_short_proof_key_fails_closed() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let (py_db, dir, _rt) = setup_py_contextra("short_key");

        // Pass short 16-byte proof key
        let secret_key_str = "secret_short_123";
        let key_bytes = PyBytes::new(py, secret_key_str.as_bytes());
        let res = py_db.drop_collection("default", Some(&key_bytes), py);

        assert!(res.is_err(), "Short proof key must return PyErr");
        let err = match res {
            Err(e) => e,
            Ok(_) => panic!("Expected error for short key"),
        };
        let err_msg = err.to_string();

        assert!(
            err_msg.contains("at least 32 bytes"),
            "Error message must state min key length requirement"
        );
        assert!(
            !err_msg.contains(secret_key_str),
            "Error message must never leak secret key material"
        );

        let _ = std::fs::remove_dir_all(&dir);
    });
}
