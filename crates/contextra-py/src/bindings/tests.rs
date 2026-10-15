#[cfg(test)]
mod tests {
    use super::super::*;
    use contextra_core::ContextraError;
    use numpy::PyArrayMethods;
    use pyo3::exceptions::*;
    use pyo3::prelude::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_parse_worker_threads_env_clamping() {
        std::env::set_var("CONTEXTRA_WORKER_THREADS", "0");
        assert_eq!(parse_worker_threads_env(), 1);

        std::env::set_var("CONTEXTRA_WORKER_THREADS", "1000");
        assert_eq!(parse_worker_threads_env(), 256);

        std::env::set_var("CONTEXTRA_WORKER_THREADS", "invalid");
        let val = parse_worker_threads_env();
        assert!((1..=256).contains(&val));
    }

    #[test]
    fn test_py_runtime_state_initialization() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .unwrap();
        let state = PyRuntimeState {
            runtime: std::sync::Arc::new(rt),
            worker_threads: 2,
        };
        assert_eq!(state.worker_threads, 2);
    }

    #[test]
    fn test_validate_id_guards() {
        pyo3::prepare_freethreaded_python();
        assert!(validate_id("").is_err());
        assert!(validate_id("   ").is_err());
        assert!(validate_id("id\0null").is_err());
        assert!(validate_id("valid_id").is_ok());
    }

    #[test]
    fn test_validate_id_obj_numeric_bounds() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let valid_int = 12345i64.into_pyobject(py)?;
            assert_eq!(validate_id_obj(&valid_int)?, "12345");

            let neg_int = (-10i64).into_pyobject(py)?;
            assert!(validate_id_obj(&neg_int).is_err());

            let valid_str = "doc_99".into_pyobject(py)?;
            assert_eq!(validate_id_obj(&valid_str)?, "doc_99");

            Ok(())
        })
    }

    #[test]
    fn test_validate_collection_name_guards() {
        pyo3::prepare_freethreaded_python();
        assert!(validate_collection_name("").is_err());
        assert!(validate_collection_name("   ").is_err());
        assert!(validate_collection_name("name\0null").is_err());
        assert!(validate_collection_name("valid_name").is_ok());
    }

    #[test]
    fn test_validate_db_path_guards() {
        pyo3::prepare_freethreaded_python();
        assert!(validate_db_path("").is_err());
        assert!(validate_db_path("   ").is_err());
        assert!(validate_db_path("path\0null").is_err());
        assert!(validate_db_path("valid_path").is_ok());
    }

    #[test]
    fn test_validate_query_text_guards() {
        pyo3::prepare_freethreaded_python();
        assert!(validate_query_text("").is_err());
        assert!(validate_query_text("   ").is_err());
        assert!(validate_query_text("query\0null").is_err());
        assert!(validate_query_text("valid query").is_ok());
    }

    #[test]
    fn test_validate_batch_size_guards() {
        assert!(validate_batch_size(0).is_err());
        assert!(validate_batch_size(1).is_ok());
        assert!(validate_batch_size(100).is_ok());
        assert!(validate_batch_size(MAX_BATCH_SIZE).is_ok());
        assert!(validate_batch_size(MAX_BATCH_SIZE + 1).is_err());
    }

    #[test]
    fn test_validate_vector_guards() {
        assert!(validate_vector(&[]).is_err());
        assert!(validate_vector(&[1.0, 2.0, 3.0]).is_ok());
        assert!(validate_vector(&[f32::NAN]).is_err());
        assert!(validate_vector(&[f32::INFINITY]).is_err());
    }

    #[test]
    fn test_py_err_mapping_all_error_kinds() {
        pyo3::prepare_freethreaded_python();

        let err_not_found = contextra_err(ContextraError::NotFound("item".into()));
        Python::with_gil(|py| {
            assert!(err_not_found.is_instance_of::<PyKeyError>(py));
        });

        let err_conflict = contextra_err(ContextraError::Conflict("conflict".into()));
        Python::with_gil(|py| {
            assert!(err_conflict.is_instance_of::<PyRuntimeError>(py));
        });

        let err_policy = contextra_err(ContextraError::PolicyViolation("policy".into()));
        Python::with_gil(|py| {
            assert!(err_policy.is_instance_of::<PyPermissionError>(py));
        });

        let err_invalid = contextra_err(ContextraError::InvalidInput("invalid".into()));
        Python::with_gil(|py| {
            assert!(err_invalid.is_instance_of::<ContextraValueError>(py));
        });

        let err_not_impl = contextra_err(ContextraError::CapabilityUnsupported {
            capability: "feature".into(),
            reason: "not supported".into(),
        });
        Python::with_gil(|py| {
            assert!(err_not_impl.is_instance_of::<pyo3::exceptions::PyNotImplementedError>(py));
        });
    }

    #[test]
    fn test_contextra_err_attributes_set() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::NotFound("test_key".into()));
        Python::with_gil(|py| {
            let bound_val = err.value(py);
            if let Ok(kind_attr) = bound_val.getattr("kind") {
                let kind_str: String = kind_attr.extract()?;
                assert_eq!(kind_str, "NotFound");
            } else {
                return Err(PyValueError::new_err(
                    "kind attribute missing on Contextra error object",
                ));
            }
            if let Ok(msg_attr) = bound_val.getattr("message") {
                let msg_str: String = msg_attr.extract()?;
                assert!(msg_str.contains("test_key"));
            } else {
                return Err(PyValueError::new_err(
                    "message attribute missing on Contextra error object",
                ));
            }
            Ok(())
        })
    }

    fn _simulate_panic_for_test() -> ! {
        panic!("Simulated Rust core panic");
    }

    #[test]
    fn test_trigger_panic_helper_resume_unwind_containment() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let poison = AtomicBool::new(false);
            let res = functions::trigger_panic_helper(
                py,
                &poison,
                Some("Test resume_unwind message".to_string()),
            );
            assert!(res.is_err());
            assert!(poison.load(Ordering::SeqCst));
            if let Err(py_err) = res {
                assert!(py_err.is_instance_of::<PyRuntimeError>(py));
                let msg = py_err.value(py).to_string();
                assert!(msg.contains("Rust panic caught at FFI boundary"));
                assert!(msg.contains("Test resume_unwind message"));
            }
        });
    }

    #[test]
    fn test_run_blocking_ffi_panic_containment() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let poison = AtomicBool::new(false);
            let res: PyResult<i32> = run_blocking_ffi(py, &poison, || {
                _simulate_panic_for_test();
            });
            assert!(res.is_err());
            assert!(poison.load(Ordering::SeqCst));
            if let Err(py_err) = res {
                assert!(py_err.is_instance_of::<PyRuntimeError>(py));
                let bound_val = py_err.value(py);
                let msg: String = bound_val.to_string();
                assert!(msg.contains("Rust panic caught at FFI boundary"));
                assert!(msg.contains("Simulated Rust core panic"));
            }

            let res_after: PyResult<i32> = run_blocking_ffi(py, &poison, || Ok(42));
            assert!(res_after.is_err());
            if let Err(py_err) = res_after {
                let msg = py_err.value(py).to_string();
                assert!(msg.contains("engine poisoned after previous panic"));
            }
        });
    }

    #[test]
    fn test_run_blocking_ffi_success() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let poison = AtomicBool::new(false);
            let res: PyResult<i32> = run_blocking_ffi(py, &poison, || Ok(42));
            assert!(matches!(res, Ok(42)));
            assert!(!poison.load(Ordering::SeqCst));
        });
    }

    #[test]
    fn test_validate_id_length_and_empty() {
        assert!(validate_id("").is_err());
        assert!(validate_id("valid_id").is_ok());

        let long_id = "a".repeat(MAX_ID_LENGTH + 1);
        assert!(validate_id(&long_id).is_err());

        let max_id = "a".repeat(MAX_ID_LENGTH);
        assert!(validate_id(&max_id).is_ok());
    }

    #[test]
    fn test_validate_label_length_and_empty() {
        pyo3::prepare_freethreaded_python();
        assert!(validate_label("").is_err());
        assert!(validate_label("   ").is_err());
        assert!(validate_label("label\0null").is_err());
        assert!(validate_label("valid_label").is_ok());

        let long_label = "l".repeat(MAX_LABEL_LENGTH + 1);
        assert!(validate_label(&long_label).is_err());

        let max_label = "l".repeat(MAX_LABEL_LENGTH);
        assert!(validate_label(&max_label).is_ok());
    }

    #[test]
    fn test_validate_vector_nan_inf() {
        assert!(validate_vector(&[1.0, 2.0, 3.0]).is_ok());
        assert!(validate_vector(&[1.0, f32::NAN, 3.0]).is_err());
        assert!(validate_vector(&[1.0, f32::INFINITY, 3.0]).is_err());
        assert!(validate_vector(&[1.0, f32::NEG_INFINITY, 3.0]).is_err());
    }

    #[test]
    fn test_validate_db_path_query_text_and_batch_size() {
        pyo3::prepare_freethreaded_python();

        // validate_db_path
        assert!(validate_db_path("").is_err());
        assert!(validate_db_path("   ").is_err());
        assert!(validate_db_path("path\0null").is_err());
        assert!(validate_db_path("/tmp/contextra_db").is_ok());

        // validate_query_text
        assert!(validate_query_text("").is_err());
        assert!(validate_query_text("   ").is_err());
        assert!(validate_query_text("query\0null").is_err());
        assert!(validate_query_text("valid query text").is_ok());

        let long_query = "q".repeat(MAX_ID_LENGTH + 1);
        assert!(validate_query_text(&long_query).is_err());
        let max_query = "q".repeat(MAX_ID_LENGTH);
        assert!(validate_query_text(&max_query).is_ok());

        // validate_batch_size
        assert!(validate_batch_size(0).is_err());
        assert!(validate_batch_size(100).is_ok());
        assert!(validate_batch_size(MAX_BATCH_SIZE).is_ok());
        assert!(validate_batch_size(MAX_BATCH_SIZE + 1).is_err());
    }

    #[test]
    fn test_py_err_io_and_index_mappings() {
        pyo3::prepare_freethreaded_python();
        let io_err = ContextraError::Io(std::io::Error::other("disk error"));
        let py_io_err: PyErr = contextra_err(io_err);
        Python::with_gil(|py| {
            assert!(py_io_err.is_instance_of::<ContextraIOError>(py));
        });

        let idx_err = ContextraError::Index("hnsw broken".into());
        let py_idx_err: PyErr = contextra_err(idx_err);
        Python::with_gil(|py| {
            assert!(py_idx_err.is_instance_of::<ContextraIndexError>(py));
        });
    }

    #[test]
    fn test_gil_not_held_during_blocking_ops() {
        pyo3::prepare_freethreaded_python();
        let temp_dir =
            std::env::temp_dir().join(format!("contextra_gil_test_{}", std::process::id()));
        let db_path = temp_dir.to_str().unwrap();

        Python::with_gil(|py| {
            register_contextra_module(py);
            let db =
                open(py, db_path, 128, None, None, None, None).expect("failed to open test db");

            // Insert sample documents with prefixes
            let dummy_v = vec![0.1f32; 128];
            for i in 0..10 {
                let py_v = numpy::PyArray1::from_vec(py, dummy_v.clone()).readonly();
                let doc_id_a = format!("pref_a_{}", i);
                let doc_id_b = format!("pref_b_{}", i);
                db.insert(py, &doc_id_a.into_pyobject(py).unwrap(), py_v, None)
                    .unwrap();
                let py_v2 = numpy::PyArray1::from_vec(py, dummy_v.clone()).readonly();
                db.insert(py, &doc_id_b.into_pyobject(py).unwrap(), py_v2, None)
                    .unwrap();
            }

            // Spawn N Python threads calling scan_prefix concurrently
            let threading = py.import("threading").expect("failed to import threading");
            let time = py.import("time").expect("failed to import time");

            let db_py = Py::new(py, db).unwrap();
            let errors = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

            let num_threads = 4;
            let mut threads = Vec::new();

            for i in 0..num_threads {
                let db_ref = db_py.clone_ref(py);
                let errors_clone = errors.clone();

                let worker_code =
                    pyo3::types::PyCFunction::new_closure(py, None, None, move |_args, _kwargs| {
                        Python::with_gil(|py| {
                            let prefix = if i % 2 == 0 { "pref_a_" } else { "pref_b_" };
                            let bound_db = db_ref.bind(py);
                            let db_borrow = bound_db.borrow();
                            for _ in 0..10 {
                                let res = db_borrow.scan_prefix(py, prefix, None);
                                match res {
                                    Ok(l) => {
                                        if l.len() != 10 {
                                            errors_clone
                                                .lock()
                                                .unwrap()
                                                .push(format!("Expected 10 docs, got {}", l.len()));
                                        }
                                    }
                                    Err(e) => {
                                        errors_clone
                                            .lock()
                                            .unwrap()
                                            .push(format!("scan_prefix error: {}", e));
                                    }
                                }
                            }
                        });
                        Ok::<(), pyo3::PyErr>(())
                    })
                    .unwrap();

                let kwargs = pyo3::types::PyDict::new(py);
                kwargs.set_item("target", worker_code).unwrap();
                let t = threading.call_method("Thread", (), Some(&kwargs)).unwrap();
                t.call_method0("start").unwrap();
                threads.push(t);
            }

            let start_time: f64 = time.call_method0("time").unwrap().extract().unwrap();

            for t in threads {
                t.call_method1("join", (4.0,)).unwrap();
                let is_alive: bool = t.call_method0("is_alive").unwrap().extract().unwrap();
                assert!(
                    !is_alive,
                    "Thread timed out; GIL was likely held during blocking operations"
                );
            }

            let end_time: f64 = time.call_method0("time").unwrap().extract().unwrap();
            let elapsed = end_time - start_time;
            assert!(elapsed < 5.0, "Test took {:.2}s, expected < 5.0s", elapsed);

            let err_list = errors.lock().unwrap();
            assert!(
                err_list.is_empty(),
                "Errors during thread execution: {:?}",
                *err_list
            );
        });

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    fn register_contextra_module(py: Python<'_>) {
        if let Ok(sys) = py.import("sys") {
            if let Ok(modules) = sys.getattr("modules") {
                if modules.get_item("_contextra").is_err() {
                    let m = pyo3::wrap_pymodule!(crate::_contextra)(py);
                    let _ = modules.set_item("_contextra", m);
                }
            }
        }
    }

    #[test]
    fn test_tenant_isolation_two_tenants() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            register_contextra_module(py);
            let temp_dir =
                std::env::temp_dir().join(format!("contextra_tenant_iso_{}", std::process::id()));
            let db_path = temp_dir.to_str().unwrap();

            // 1. Open database with tenant 1001
            let db_t1 = open(py, db_path, 128, None, None, None, Some(1001)).expect("open db_t1");
            assert_eq!(db_t1.tenant_id(), 1001);

            // 2. Derive tenant 2002 via with_tenant
            let db_t2 = db_t1.with_tenant(2002).expect("with_tenant 2002");
            assert_eq!(db_t2.tenant_id(), 2002);

            // 3. Verify invalid tenant ID (0) fails
            assert!(db_t1.with_tenant(0).is_err());

            // 4. Tenant 1001 inserts a document in default collection
            let dummy_v = vec![0.1f32; 128];
            let py_v1 = numpy::PyArray1::from_vec(py, dummy_v.clone()).readonly();
            let doc_id = "shared_doc_id";
            db_t1
                .insert(py, &doc_id.into_pyobject(py).unwrap(), py_v1, None)
                .expect("t1 insert");

            // Tenant 1001 sees the document
            let doc_t1 = db_t1
                .get(py, &doc_id.into_pyobject(py).unwrap())
                .expect("t1 get");
            assert!(doc_t1.is_some());

            // Tenant 2002 does NOT see tenant 1001's document
            let doc_t2 = db_t2
                .get(py, &doc_id.into_pyobject(py).unwrap())
                .expect("t2 get");
            assert!(
                doc_t2.is_none(),
                "Tenant 2002 must not see Tenant 1001's document"
            );

            let py_v_search = numpy::PyArray1::from_vec(py, dummy_v.clone()).readonly();
            let search_t2 = db_t2.search(py, py_v_search, 10).expect("t2 search");
            assert!(search_t2.is_empty(), "Tenant 2002 search must be empty");

            // 5. Test named collections
            let col_t1 = db_t1.collection("shared_catalog", py).expect("col_t1");
            assert_eq!(col_t1.tenant_id(), 1001);

            let col_t2 = col_t1
                .with_tenant(2002, py)
                .expect("col_t1 with_tenant 2002");
            assert_eq!(col_t2.tenant_id(), 2002);

            let py_v2 = numpy::PyArray1::from_vec(py, dummy_v.clone()).readonly();
            col_t1
                .insert(py, &"col_doc_id".into_pyobject(py).unwrap(), py_v2, None)
                .expect("col_t1 insert");

            assert!(col_t1
                .get(py, &"col_doc_id".into_pyobject(py).unwrap())
                .unwrap()
                .is_some());
            assert!(
                col_t2
                    .get(py, &"col_doc_id".into_pyobject(py).unwrap())
                    .unwrap()
                    .is_none(),
                "Tenant 2002 collection must not see Tenant 1001's collection document"
            );

            let _ = std::fs::remove_dir_all(temp_dir);
        });
    }
}
