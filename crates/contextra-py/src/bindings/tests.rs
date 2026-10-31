#[cfg(test)]
mod tests {
    use super::super::*;
    use contextra_types::{ContextraError, DocId, LinkRelation};
    use pyo3::exceptions::*;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;
    use std::sync::atomic::{AtomicBool, Ordering};

    // ─── Worker Threads Env Parsing ─────────────────────────────────────────────

    #[test]
    fn test_parse_worker_threads_env_clamping_low() {
        std::env::set_var("CONTEXTRA_WORKER_THREADS", "0");
        assert_eq!(parse_worker_threads_env(), 1);
    }

    #[test]
    fn test_parse_worker_threads_env_clamping_high() {
        std::env::set_var("CONTEXTRA_WORKER_THREADS", "1000");
        assert_eq!(parse_worker_threads_env(), 256);
    }

    #[test]
    fn test_parse_worker_threads_env_invalid_fallback() {
        std::env::set_var("CONTEXTRA_WORKER_THREADS", "not_a_number");
        let val = parse_worker_threads_env();
        assert!((1..=256).contains(&val));
    }

    #[test]
    fn test_parse_worker_threads_env_valid_value() {
        std::env::set_var("CONTEXTRA_WORKER_THREADS", "8");
        assert_eq!(parse_worker_threads_env(), 8);
    }

    // ─── Runtime State ──────────────────────────────────────────────────────────

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

    // ─── ID Validation ──────────────────────────────────────────────────────────

    #[test]
    fn test_validate_id_empty() {
        assert!(validate_id("").is_err());
    }

    #[test]
    fn test_validate_id_whitespace() {
        assert!(validate_id("   ").is_err());
        assert!(validate_id("\t\n").is_err());
    }

    #[test]
    fn test_validate_id_null_byte() {
        assert!(validate_id("doc\0id").is_err());
    }

    #[test]
    fn test_validate_id_max_len_exceeded() {
        let long_id = "a".repeat(MAX_ID_LENGTH + 1);
        assert!(validate_id(&long_id).is_err());
    }

    #[test]
    fn test_validate_id_max_len_exact() {
        let max_id = "a".repeat(MAX_ID_LENGTH);
        assert!(validate_id(&max_id).is_ok());
    }

    #[test]
    fn test_validate_id_valid_alphanumeric() {
        assert!(validate_id("doc_123_abc-XYZ").is_ok());
    }

    #[test]
    fn test_validate_id_obj_str_valid() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let s = "doc_str_99".into_pyobject(py)?;
            assert_eq!(validate_id_obj(&s)?, "doc_str_99");
            Ok(())
        })
    }

    #[test]
    fn test_validate_id_obj_int_valid() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let val = 12345i64.into_pyobject(py)?;
            assert_eq!(validate_id_obj(&val)?, "12345");
            Ok(())
        })
    }

    #[test]
    fn test_validate_id_obj_int_zero() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let val = 0i64.into_pyobject(py)?;
            assert_eq!(validate_id_obj(&val)?, "0");
            Ok(())
        })
    }

    #[test]
    fn test_validate_id_obj_int_negative() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let neg = (-10i64).into_pyobject(py)?;
            assert!(validate_id_obj(&neg).is_err());
            Ok(())
        })
    }

    #[test]
    fn test_validate_id_obj_invalid_type() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let float_val = 3.14f64.into_pyobject(py)?;
            assert!(validate_id_obj(&float_val).is_err());
            Ok(())
        })
    }

    // ─── Collection Name Validation ────────────────────────────────────────────

    #[test]
    fn test_validate_collection_name_empty() {
        assert!(validate_collection_name("").is_err());
    }

    #[test]
    fn test_validate_collection_name_whitespace() {
        assert!(validate_collection_name("   ").is_err());
    }

    #[test]
    fn test_validate_collection_name_null_byte() {
        assert!(validate_collection_name("col\0name").is_err());
    }

    #[test]
    fn test_validate_collection_name_max_len_exceeded() {
        let long_col = "c".repeat(MAX_COLLECTION_NAME_LENGTH + 1);
        assert!(validate_collection_name(&long_col).is_err());
    }

    #[test]
    fn test_validate_collection_name_max_len_exact() {
        let max_col = "c".repeat(MAX_COLLECTION_NAME_LENGTH);
        assert!(validate_collection_name(&max_col).is_ok());
    }

    #[test]
    fn test_validate_collection_name_valid() {
        assert!(validate_collection_name("my_collection_1").is_ok());
    }

    // ─── Label Validation ───────────────────────────────────────────────────────

    #[test]
    fn test_validate_label_empty() {
        assert!(validate_label("").is_err());
    }

    #[test]
    fn test_validate_label_whitespace() {
        assert!(validate_label("   ").is_err());
    }

    #[test]
    fn test_validate_label_null_byte() {
        assert!(validate_label("label\0null").is_err());
    }

    #[test]
    fn test_validate_label_max_len_exceeded() {
        let long_label = "l".repeat(MAX_LABEL_LENGTH + 1);
        assert!(validate_label(&long_label).is_err());
    }

    #[test]
    fn test_validate_label_max_len_exact() {
        let max_label = "l".repeat(MAX_LABEL_LENGTH);
        assert!(validate_label(&max_label).is_ok());
    }

    #[test]
    fn test_validate_label_valid() {
        assert!(validate_label("knows_person").is_ok());
    }

    // ─── Database Path Validation ───────────────────────────────────────────────

    #[test]
    fn test_validate_db_path_empty() {
        assert!(validate_db_path("").is_err());
    }

    #[test]
    fn test_validate_db_path_whitespace() {
        assert!(validate_db_path("   ").is_err());
    }

    #[test]
    fn test_validate_db_path_null_byte() {
        assert!(validate_db_path("path\0null").is_err());
    }

    #[test]
    fn test_validate_db_path_valid() {
        assert!(validate_db_path("/tmp/contextra_db_test").is_ok());
    }

    // ─── Query Text Validation ──────────────────────────────────────────────────

    #[test]
    fn test_validate_query_text_empty() {
        assert!(validate_query_text("").is_err());
    }

    #[test]
    fn test_validate_query_text_whitespace() {
        assert!(validate_query_text("   ").is_err());
    }

    #[test]
    fn test_validate_query_text_null_byte() {
        assert!(validate_query_text("query\0null").is_err());
    }

    #[test]
    fn test_validate_query_text_max_len_exceeded() {
        let long_q = "q".repeat(MAX_ID_LENGTH + 1);
        assert!(validate_query_text(&long_q).is_err());
    }

    #[test]
    fn test_validate_query_text_max_len_exact() {
        let max_q = "q".repeat(MAX_ID_LENGTH);
        assert!(validate_query_text(&max_q).is_ok());
    }

    #[test]
    fn test_validate_query_text_valid() {
        assert!(validate_query_text("natural language query text").is_ok());
    }

    // ─── Batch Size Validation ──────────────────────────────────────────────────

    #[test]
    fn test_validate_batch_size_zero() {
        assert!(validate_batch_size(0).is_err());
    }

    #[test]
    fn test_validate_batch_size_one() {
        assert!(validate_batch_size(1).is_ok());
    }

    #[test]
    fn test_validate_batch_size_max() {
        assert!(validate_batch_size(MAX_BATCH_SIZE).is_ok());
    }

    #[test]
    fn test_validate_batch_size_overflow() {
        assert!(validate_batch_size(MAX_BATCH_SIZE + 1).is_err());
    }

    // ─── Vector Validation ──────────────────────────────────────────────────────

    #[test]
    fn test_validate_vector_empty() {
        assert!(validate_vector(&[]).is_err());
    }

    #[test]
    fn test_validate_vector_valid() {
        assert!(validate_vector(&[0.1, 0.2, -0.5, 1.0]).is_ok());
    }

    #[test]
    fn test_validate_vector_nan() {
        assert!(validate_vector(&[1.0, f32::NAN, 2.0]).is_err());
    }

    #[test]
    fn test_validate_vector_pos_inf() {
        assert!(validate_vector(&[1.0, f32::INFINITY, 2.0]).is_err());
    }

    #[test]
    fn test_validate_vector_neg_inf() {
        assert!(validate_vector(&[1.0, f32::NEG_INFINITY, 2.0]).is_err());
    }

    // ─── Exception Mapping Tests ────────────────────────────────────────────────

    #[test]
    fn test_err_mapping_not_found() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::NotFound("item_1".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<NotFoundError>(py));
        });
    }

    #[test]
    fn test_err_mapping_conflict() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::Conflict("conflict_1".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<IntegrityError>(py));
        });
    }

    #[test]
    fn test_err_mapping_tenant_mismatch() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::NamespaceViolation("tenant_mismatch".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<TenantError>(py));
        });
    }

    #[test]
    fn test_err_mapping_policy_violation() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::PolicyViolation("policy_1".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<PyPermissionError>(py));
        });
    }

    #[test]
    fn test_err_mapping_invalid_input() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::InvalidInput("invalid_1".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraValueError>(py));
        });
    }

    #[test]
    fn test_err_mapping_storage() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::Storage("storage_err".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraIOError>(py));
        });
    }

    #[test]
    fn test_err_mapping_io() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::Io(std::io::Error::other("disk_err")));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraIOError>(py));
        });
    }

    #[test]
    fn test_err_mapping_index() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::Index("index_corrupt".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraIndexError>(py));
        });
    }

    #[test]
    fn test_err_mapping_crypto() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::Crypto("key_error".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraCryptoError>(py));
        });
    }

    #[test]
    fn test_err_mapping_memory_budget() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::MemoryBudgetExceeded("oom".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<pyo3::exceptions::PyMemoryError>(py));
        });
    }

    #[test]
    fn test_err_mapping_capability_unsupported() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::CapabilityUnsupported {
            capability: "feature_x".into(),
            reason: "not supported".into(),
        });
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<pyo3::exceptions::PyNotImplementedError>(py));
        });
    }

    #[test]
    fn test_err_mapping_internal() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::Internal("internal_bug".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraInternalError>(py));
        });
    }

    #[test]
    fn test_err_mapping_serialization() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::Serialization("serde_fail".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraValueError>(py));
        });
    }

    #[test]
    fn test_err_mapping_checksum_mismatch() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::ChecksumMismatch("bad_crc".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraIOError>(py));
        });
    }

    #[test]
    fn test_err_mapping_wal_corruption() {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::WalCorruption("bad_wal".into()));
        Python::with_gil(|py| {
            assert!(err.is_instance_of::<ContextraIOError>(py));
        });
    }

    #[test]
    fn test_contextra_err_attributes_kind_and_message() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        let err = contextra_err(ContextraError::NotFound("missing_doc_key".into()));
        Python::with_gil(|py| {
            let bound_val = err.value(py);
            let kind: String = bound_val.getattr("kind")?.extract()?;
            assert_eq!(kind, "NotFound");

            let msg: String = bound_val.getattr("message")?.extract()?;
            assert!(msg.contains("missing_doc_key"));
            Ok(())
        })
    }

    // ─── FFI Blocking & Panic Isolation ───────────────────────────────────────

    #[test]
    fn test_run_blocking_ffi_success() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let poison = AtomicBool::new(false);
            let res: PyResult<i32> = run_blocking_ffi(py, &poison, || Ok(100));
            assert!(matches!(res, Ok(100)));
            assert!(!poison.load(Ordering::SeqCst));
        });
    }

    #[test]
    fn test_run_blocking_ffi_panic_containment() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let poison = AtomicBool::new(false);
            let res: PyResult<i32> = run_blocking_ffi(py, &poison, || {
                panic!("Simulated Rust core panic payload");
            });
            assert!(res.is_err());
            assert!(poison.load(Ordering::SeqCst));
            if let Err(py_err) = res {
                assert!(py_err.is_instance_of::<PyRuntimeError>(py));
                let msg = py_err.value(py).to_string();
                assert!(msg.contains("Rust panic caught at FFI boundary"));
                assert!(msg.contains("Simulated Rust core panic payload"));
            }
        });
    }

    #[test]
    fn test_run_blocking_ffi_blocked_after_poisoning() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let poison = AtomicBool::new(true);
            let res: PyResult<i32> = run_blocking_ffi(py, &poison, || Ok(42));
            assert!(res.is_err());
            if let Err(py_err) = res {
                let msg = py_err.value(py).to_string();
                assert!(msg.contains("engine poisoned after previous panic"));
            }
        });
    }

    #[test]
    fn test_trigger_panic_for_test_not_exported_without_test_hooks_feature() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            if let Ok(module) = PyModule::import(py, "_contextra") {
                #[cfg(not(feature = "test-hooks"))]
                {
                    assert!(!module.hasattr("_trigger_panic_for_test").unwrap_or(false));
                }
                #[cfg(feature = "test-hooks")]
                {
                    assert!(module.hasattr("_trigger_panic_for_test").unwrap_or(false));
                }
            }
        });
    }

    #[test]
    fn test_trigger_panic_helper_resume_unwind_containment() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let poison = AtomicBool::new(false);
            let res = functions::trigger_panic_helper(
                py,
                &poison,
                Some("Test panic unwinding".to_string()),
            );
            assert!(res.is_err());
            assert!(poison.load(Ordering::SeqCst));
        });
    }

    // ─── Link Relation Parsing ─────────────────────────────────────────────────

    #[test]
    fn test_parse_link_relation_elaborates() {
        assert_eq!(
            crate::kv_links::parse_link_relation("elaborates").unwrap(),
            LinkRelation::Elaborates
        );
    }

    #[test]
    fn test_parse_link_relation_contradicts() {
        assert_eq!(
            crate::kv_links::parse_link_relation("contradicts").unwrap(),
            LinkRelation::Contradicts
        );
    }

    #[test]
    fn test_parse_link_relation_supersedes() {
        assert_eq!(
            crate::kv_links::parse_link_relation("supersedes").unwrap(),
            LinkRelation::Supersedes
        );
    }

    #[test]
    fn test_parse_link_relation_references() {
        assert_eq!(
            crate::kv_links::parse_link_relation("references").unwrap(),
            LinkRelation::References
        );
    }

    #[test]
    fn test_parse_link_relation_case_insensitive() {
        assert_eq!(
            crate::kv_links::parse_link_relation("ELABORATES").unwrap(),
            LinkRelation::Elaborates
        );
        assert_eq!(
            crate::kv_links::parse_link_relation("RefErenCes").unwrap(),
            LinkRelation::References
        );
    }

    #[test]
    fn test_parse_link_relation_invalid() {
        assert!(crate::kv_links::parse_link_relation("invalid_relation").is_err());
        assert!(crate::kv_links::parse_link_relation("").is_err());
    }

    // ─── Conversions & Helpers ─────────────────────────────────────────────────

    #[test]
    fn test_doc_to_py_conversion() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let doc = contextra_db::Document {
                id: "doc_test_1".into(),
                metadata: Some(serde_json::json!({"key": "val"})),
            };
            let py_doc = doc_to_py(py, doc)?;
            assert_eq!(py_doc.id, "doc_test_1");
            assert!(py_doc.metadata.is_some());
            Ok(())
        })
    }

    #[test]
    fn test_doc_to_py_conversion_no_metadata() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let doc = contextra_db::Document {
                id: "doc_test_2".into(),
                metadata: None,
            };
            let py_doc = doc_to_py(py, doc)?;
            assert_eq!(py_doc.id, "doc_test_2");
            assert!(py_doc.metadata.is_none());
            Ok(())
        })
    }

    #[test]
    fn test_results_to_py_conversion() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let results = vec![contextra_db::SearchResult {
                id: "doc_search_1".into(),
                score: 0.95,
                metadata: Some(serde_json::json!({"score": 95})),
            }];
            let py_res = results_to_py(py, results)?;
            assert_eq!(py_res.len(), 1);
            assert_eq!(py_res[0].id, "doc_search_1");
            assert!((py_res[0].score - 0.95).abs() < 1e-5);
            Ok(())
        })
    }

    #[test]
    fn test_dict_to_json_conversion() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("foo", "bar")?;
            dict.set_item("num", 42)?;
            let json_val = dict_to_json(&dict)?;
            assert_eq!(json_val["foo"], "bar");
            assert_eq!(json_val["num"], 42);
            Ok(())
        })
    }

    #[test]
    fn test_opt_dict_to_json_some() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("tag", "test")?;
            let res = opt_dict_to_json(Some(&dict))?;
            assert!(res.is_some());
            assert_eq!(res.unwrap()["tag"], "test");
            Ok(())
        })
    }

    #[test]
    fn test_opt_dict_to_json_none() -> PyResult<()> {
        let res = opt_dict_to_json(None)?;
        assert!(res.is_none());
        Ok(())
    }

    #[test]
    fn test_json_to_py_conversion() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let json_val = serde_json::json!({
                "str": "hello",
                "int": 123
            });
            let py_obj = json_to_py(py, &json_val)?;
            let bound_dict = py_obj.bind(py).downcast::<PyDict>()?;
            let str_val: String = bound_dict.get_item("str")?.unwrap().extract()?;
            assert_eq!(str_val, "hello");
            Ok(())
        })
    }

    // ─── Open DB Parameter Guards ──────────────────────────────────────────────

    #[test]
    fn test_open_db_invalid_dimension_zero() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let res = open(py, "/tmp/dummy_path", 0, None, None, None);
            assert!(res.is_err());
        });
    }

    #[test]
    fn test_open_db_invalid_dimension_too_large() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let res = open(py, "/tmp/dummy_path", 10_001, None, None, None);
            assert!(res.is_err());
        });
    }

    #[test]
    fn test_open_db_invalid_max_elements_zero() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let res = open(py, "/tmp/dummy_path", 128, Some(0), None, None);
            assert!(res.is_err());
        });
    }

    #[test]
    fn test_open_db_invalid_distance_metric() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let res = open(
                py,
                "/tmp/dummy_path",
                128,
                None,
                None,
                Some("invalid_metric".into()),
            );
            assert!(res.is_err());
        });
    }

    // ─── GIL Concurrency Unit Test ─────────────────────────────────────────────

    #[test]
    fn test_gil_not_held_during_blocking_ops() {
        use numpy::PyArrayMethods;
        pyo3::prepare_freethreaded_python();
        let temp_dir =
            std::env::temp_dir().join(format!("contextra_gil_test_{}", std::process::id()));
        let db_path = temp_dir.to_str().unwrap();

        Python::with_gil(|py| {
            let db = open(py, db_path, 128, None, None, None).expect("failed to open test db");

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
                            for _ in 0..10 {
                                let res: PyResult<PyObject> =
                                    db_ref.call_method1(py, "scan_prefix", (prefix,));
                                match res {
                                    Ok(res_obj) => {
                                        let list = res_obj.extract::<Vec<(String, PyObject)>>(py);
                                        if let Ok(l) = list {
                                            if l.len() != 10 {
                                                errors_clone.lock().unwrap().push(format!(
                                                    "Expected 10 docs, got {}",
                                                    l.len()
                                                ));
                                            }
                                        } else {
                                            errors_clone.lock().unwrap().push(
                                                "Failed to extract scan_prefix result".into(),
                                            );
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

                let t = threading.call_method1("Thread", (worker_code,)).unwrap();
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
}
