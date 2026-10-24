// ─── Macro: Shared CRUD Methods ─────────────────────────────────────────────
//
// This macro generates the common CRUD, search, and scan methods that are
// shared between PyContextra (default collection facade) and PyCollection.

#[macro_export]
macro_rules! contextra_crud_methods {
    ($struct_type:ty) => {
        #[pymethods]
        #[allow(deprecated)]
        impl $struct_type {
            /// Inserts a document with an embedding and optional metadata.
            #[pyo3(signature = (id, vector, metadata=None))]
            pub fn insert<'py>(
                &self,
                py: Python<'py>,
                id: &pyo3::Bound<'py, pyo3::types::PyAny>,
                vector: PyReadonlyArray1<'py, f32>,
                metadata: Option<pyo3::Bound<'py, pyo3::types::PyDict>>,
            ) -> PyResult<()> {
                let id_str = validate_id_obj(id)?;
                let rt = &self.runtime;
                let v = vector.as_slice().map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("Invalid vector: {}", e))
                })?;
                validate_vector(v)?;
                let m = opt_dict_to_json(metadata.as_ref())?;
                let v_owned = v.to_vec();
                run_blocking_ffi(py, &self.poisoned, || rt.block_on(self.inner.insert(&id_str, &v_owned, m)).map_err(contextra_err))
            }

            /// Retrieves a document by its user-provided string or numeric ID.
            pub fn get<'py>(
                &self,
                py: Python<'py>,
                id: &pyo3::Bound<'py, pyo3::types::PyAny>,
            ) -> PyResult<Option<PyDocument>> {
                let id_str = validate_id_obj(id)?;
                let rt = &self.runtime;
                let doc = run_blocking_ffi(py, &self.poisoned, || rt.block_on(self.inner.get(&id_str)).map_err(contextra_err))?;
                match doc {
                    Some(d) => Ok(Some(doc_to_py(py, d)?)),
                    None => Ok(None),
                }
            }

            /// Updates an existing document.
            #[pyo3(signature = (id, vector, metadata=None))]
            pub fn update<'py>(
                &self,
                py: Python<'py>,
                id: &pyo3::Bound<'py, pyo3::types::PyAny>,
                vector: PyReadonlyArray1<'py, f32>,
                metadata: Option<pyo3::Bound<'py, pyo3::types::PyDict>>,
            ) -> PyResult<()> {
                let id_str = validate_id_obj(id)?;
                let rt = &self.runtime;
                let v = vector.as_slice().map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("Invalid vector: {}", e))
                })?;
                validate_vector(v)?;
                let m = opt_dict_to_json(metadata.as_ref())?;
                let v_owned = v.to_vec();
                run_blocking_ffi(py, &self.poisoned, || rt.block_on(self.inner.update(&id_str, &v_owned, m)).map_err(contextra_err))
            }

            /// Upserts a document (inserts if missing, updates if exists).
            #[pyo3(signature = (id, vector, metadata=None))]
            pub fn upsert<'py>(
                &self,
                py: Python<'py>,
                id: &pyo3::Bound<'py, pyo3::types::PyAny>,
                vector: PyReadonlyArray1<'py, f32>,
                metadata: Option<pyo3::Bound<'py, pyo3::types::PyDict>>,
            ) -> PyResult<()> {
                let id_str = validate_id_obj(id)?;
                let rt = &self.runtime;
                let v = vector.as_slice().map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("Invalid vector: {}", e))
                })?;
                validate_vector(v)?;
                let m = opt_dict_to_json(metadata.as_ref())?;
                let v_owned = v.to_vec();
                run_blocking_ffi(py, &self.poisoned, || rt.block_on(self.inner.upsert(&id_str, &v_owned, m)).map_err(contextra_err))
            }

            /// Deletes a document by its ID.
            pub fn delete<'py>(
                &self,
                py: Python<'py>,
                id: &pyo3::Bound<'py, pyo3::types::PyAny>,
            ) -> PyResult<()> {
                let id_str = validate_id_obj(id)?;
                let rt = &self.runtime;
                run_blocking_ffi(py, &self.poisoned, || rt.block_on(self.inner.delete(&id_str)).map_err(contextra_err))
            }

            /// Performs semantic k-NN search over the embeddings.
            #[pyo3(signature = (vector, k))]
            pub fn search<'py>(
                &self,
                py: Python<'py>,
                vector: PyReadonlyArray1<'py, f32>,
                k: usize,
            ) -> PyResult<Vec<PySearchResult>> {
                if k == 0 || k > 1000 {
                    return Err(PyValueError::new_err(format!(
                        "Search k must be between 1 and 1000. Got: {}",
                        k
                    )));
                }
                let rt = &self.runtime;
                let v = vector.as_slice().map_err(|e| {
                    PyValueError::new_err(format!("Invalid vector: {}", e))
                })?;
                validate_vector(v)?;
                let v_owned = v.to_vec();
                let results = run_blocking_ffi(py, &self.poisoned, || rt.block_on(self.inner.search(&v_owned, k)).map_err(contextra_err))?;
                results_to_py(py, results)
            }

            /// Performs semantic search and returns results as FlatBuffer-encoded bytes (PyBytes).
            ///
            /// Returns FlatBuffer binary IPC payload copied into Python PyBytes.
            #[pyo3(signature = (vector, k))]
            pub fn search_fb<'py>(
                &self,
                py: Python<'py>,
                vector: PyReadonlyArray1<'py, f32>,
                k: usize,
            ) -> PyResult<Bound<'py, PyBytes>> {
                if k == 0 || k > 1000 {
                    return Err(PyValueError::new_err(format!(
                        "Search k must be between 1 and 1000. Got: {}",
                        k
                    )));
                }
                let rt = &self.runtime;
                let v = vector.as_slice().map_err(|e| {
                    PyValueError::new_err(format!("Invalid vector: {}", e))
                })?;
                validate_vector(v)?;
                let v_owned = v.to_vec();
                let results = run_blocking_ffi(py, &self.poisoned, || rt.block_on(self.inner.search(&v_owned, k)).map_err(contextra_err))?;

                let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);
                let mut res_offsets = Vec::with_capacity(results.len());

                for r in results {
                    let id_off = builder.create_string(&r.id);
                    let meta_str = r.metadata.map(|m| m.to_string()).unwrap_or_default();
                    let meta_off = builder.create_string(&meta_str);

                    let doc_res = contextra_core::ipc::ScoredDocument::create(
                        &mut builder,
                        &contextra_core::ipc::ScoredDocumentArgs {
                            id: Some(id_off),
                            score: r.score,
                            metadata: Some(meta_off),
                            embedding: None,
                        },
                    );
                    res_offsets.push(doc_res);
                }

                let results_vec_off = builder.create_vector(&res_offsets);
                let response = contextra_core::ipc::SearchResponse::create(
                    &mut builder,
                    &contextra_core::ipc::SearchResponseArgs {
                        results: Some(results_vec_off),
                        total_hits: res_offsets.len() as u32,
                        processing_time_ms: 0.0,
                    },
                );

                builder.finish(response, None);
                let data = builder.finished_data();
                Ok(PyBytes::new(py, data))
            }

            /// Performs hybrid search combining BM25, vector search, and graph traversal results.
            #[allow(clippy::too_many_arguments)]
            #[pyo3(signature = (text, vector, k, vector_weight=None, text_weight=None, graph_weight=None))]
            pub fn hybrid_search<'py>(
                &self,
                py: Python<'py>,
                text: &str,
                vector: PyReadonlyArray1<'py, f32>,
                k: usize,
                vector_weight: Option<f32>,
                text_weight: Option<f32>,
                graph_weight: Option<f32>,
            ) -> PyResult<Vec<PySearchResult>> {
                validate_query_text(text)?;
                if k == 0 || k > 1000 {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Search k must be between 1 and 1000. Got: {}",
                        k
                    )));
                }
                let weights = match (vector_weight, text_weight, graph_weight) {
                    (Some(v), Some(t), Some(g)) => {
                        Some(contextra_core::FusionWeights::new(v, t, g)
                            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?)
                    }
                    (None, None, None) => None,
                    _ => return Err(pyo3::exceptions::PyValueError::new_err(
                        "Must specify either all three weights (vector_weight, text_weight, graph_weight) or none"
                    )),
                };
                let rt = &self.runtime;
                let v = vector.as_slice().map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("Invalid vector: {}", e))
                })?;
                validate_vector(v)?;
                let text_owned = text.to_string();
                let v_owned = v.to_vec();
                let results = run_blocking_ffi(py, &self.poisoned, || {
                    rt.block_on(self.inner.hybrid_search_with_weights(&text_owned, &v_owned, k, None, weights.as_ref()))
                        .map_err(contextra_err)
                })?;
                results_to_py(py, results)
            }

            /// Performs hybrid search and returns results as FlatBuffer-encoded bytes (PyBytes).
            ///
            /// Returns FlatBuffer binary IPC payload copied into Python PyBytes.
            #[allow(clippy::too_many_arguments)]
            #[pyo3(signature = (text, vector, k, vector_weight=None, text_weight=None, graph_weight=None))]
            pub fn hybrid_search_fb<'py>(
                &self,
                py: Python<'py>,
                text: &str,
                vector: PyReadonlyArray1<'py, f32>,
                k: usize,
                vector_weight: Option<f32>,
                text_weight: Option<f32>,
                graph_weight: Option<f32>,
            ) -> PyResult<Bound<'py, PyBytes>> {
                validate_query_text(text)?;
                if k == 0 || k > 1000 {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Search k must be between 1 and 1000. Got: {}",
                        k
                    )));
                }
                let weights = match (vector_weight, text_weight, graph_weight) {
                    (Some(v), Some(t), Some(g)) => {
                        Some(contextra_core::FusionWeights::new(v, t, g)
                            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?)
                    }
                    (None, None, None) => None,
                    _ => return Err(pyo3::exceptions::PyValueError::new_err(
                        "Must specify either all three weights (vector_weight, text_weight, graph_weight) or none"
                    )),
                };
                let rt = &self.runtime;
                let v = vector.as_slice().map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!("Invalid vector: {}", e))
                })?;
                validate_vector(v)?;
                let text_owned = text.to_string();
                let v_owned = v.to_vec();
                let results = run_blocking_ffi(py, &self.poisoned, || {
                    rt.block_on(self.inner.hybrid_search_with_weights(&text_owned, &v_owned, k, None, weights.as_ref()))
                        .map_err(contextra_err)
                })?;

                let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);
                let mut res_offsets = Vec::with_capacity(results.len());

                for r in results {
                    let id_off = builder.create_string(&r.id);
                    let meta_str = r.metadata.map(|m| m.to_string()).unwrap_or_default();
                    let meta_off = builder.create_string(&meta_str);

                    let doc_res = contextra_core::ipc::ScoredDocument::create(
                        &mut builder,
                        &contextra_core::ipc::ScoredDocumentArgs {
                            id: Some(id_off),
                            score: r.score,
                            metadata: Some(meta_off),
                            embedding: None,
                        },
                    );
                    res_offsets.push(doc_res);
                }

                let results_vec_off = builder.create_vector(&res_offsets);
                let response = contextra_core::ipc::SearchResponse::create(
                    &mut builder,
                    &contextra_core::ipc::SearchResponseArgs {
                        results: Some(results_vec_off),
                        total_hits: res_offsets.len() as u32,
                        processing_time_ms: 0.0,
                    },
                );

                builder.finish(response, None);
                let data = builder.finished_data();
                Ok(PyBytes::new(py, data))
            }

            /// Creates a bidirectional relationship between two documents.
            pub fn relate<'py>(
                &self,
                py: Python<'py>,
                from: &pyo3::Bound<'py, pyo3::types::PyAny>,
                to: &pyo3::Bound<'py, pyo3::types::PyAny>,
                label: &str,
            ) -> PyResult<()> {
                let from_str = validate_id_obj(from)?;
                let to_str = validate_id_obj(to)?;
                validate_label(label)?;
                let rt = &self.runtime;
                let label_owned = label.to_string();
                run_blocking_ffi(py, &self.poisoned, || {
                    rt.block_on(self.inner.relate(&from_str, &to_str, &label_owned))
                        .map_err(contextra_err)
                })
            }

            /// Scans documents matching a given key prefix.
            #[pyo3(signature = (prefix="", limit=None))]
            pub fn scan_prefix(
                &self,
                py: Python<'_>,
                prefix: &str,
                limit: Option<usize>,
            ) -> PyResult<Vec<(String, PyObject)>> {
                let rt = &self.runtime;
                let prefix_owned = prefix.to_string();
                let results = run_blocking_ffi(py, &self.poisoned, || {
                    rt.block_on(self.inner.scan_prefix(&prefix_owned, limit))
                        .map_err(contextra_err)
                })?;
                let mut py_res = Vec::with_capacity(results.len());
                for (k, v) in results {
                    py_res.push((k, json_to_py(py, &v)?));
                }
                Ok(py_res)
            }

            /// Performs a range scan of documents.
            ///
            /// Accepts optional string keys for start and end bounds (inclusive).
            /// Pass `None` for unbounded.
            #[pyo3(signature = (start=None, end=None, limit=None))]
            pub fn scan(
                &self,
                py: Python<'_>,
                start: Option<&str>,
                end: Option<&str>,
                limit: Option<usize>,
            ) -> PyResult<Vec<(String, PyObject)>> {
                let rt = &self.runtime;
                let start_bytes: Option<Vec<u8>> = start.map(|s| s.as_bytes().to_vec());
                let end_bytes: Option<Vec<u8>> = end.map(|s| s.as_bytes().to_vec());

                let results = run_blocking_ffi(py, &self.poisoned, || {
                    use std::ops::Bound;
                    let start_bound = match &start_bytes {
                        Some(b) => Bound::Included(b.as_slice()),
                        None => Bound::Unbounded,
                    };
                    let end_bound = match &end_bytes {
                        Some(b) => Bound::Included(b.as_slice()),
                        None => Bound::Unbounded,
                    };
                    rt.block_on(self.inner.scan(start_bound, end_bound, limit))
                        .map_err(contextra_err)
                })?;
                let mut py_res = Vec::with_capacity(results.len());
                for (k, v) in results {
                    py_res.push((k, json_to_py(py, &v)?));
                }
                Ok(py_res)
            }
        }
    };
}

// ─── Macro: Batch Methods ───────────────────────────────────────────────────

#[macro_export]
macro_rules! contextra_batch_methods {
    ($struct_type:ty) => {
        #[pymethods]
        impl $struct_type {
            /// Inserts multiple documents in a single transaction.
            ///
            /// Each doc is a tuple of `(id: str, vector: np.ndarray, metadata: dict | None)`.
            pub fn insert_many<'py>(
                &self,
                py: Python<'py>,
                docs: Vec<(
                    pyo3::Bound<'py, pyo3::types::PyAny>,
                    PyReadonlyArray1<'py, f32>,
                    Option<pyo3::Bound<'py, pyo3::types::PyDict>>,
                )>,
            ) -> PyResult<()> {
                validate_batch_size(docs.len())?;
                let rt = &self.runtime;
                let mut batch: Vec<(String, Vec<f32>, Option<serde_json::Value>)> =
                    Vec::with_capacity(docs.len());
                for (id_obj, vector, metadata) in &docs {
                    let id_str = validate_id_obj(id_obj)?;
                    let v = vector.as_slice().map_err(|e| {
                        pyo3::exceptions::PyValueError::new_err(format!("Invalid vector: {}", e))
                    })?;
                    validate_vector(v)?;
                    let m = opt_dict_to_json(metadata.as_ref())?;
                    batch.push((id_str, v.to_vec(), m));
                }
                run_blocking_ffi(py, &self.poisoned, || {
                    rt.block_on(self.inner.insert_many(&batch))
                        .map_err(contextra_err)
                })
            }

            /// Upserts multiple documents in a single transaction.
            ///
            /// Each doc is a tuple of `(id: str, vector: np.ndarray, metadata: dict | None)`.
            pub fn upsert_many<'py>(
                &self,
                py: Python<'py>,
                docs: Vec<(
                    pyo3::Bound<'py, pyo3::types::PyAny>,
                    PyReadonlyArray1<'py, f32>,
                    Option<pyo3::Bound<'py, pyo3::types::PyDict>>,
                )>,
            ) -> PyResult<()> {
                validate_batch_size(docs.len())?;
                let rt = &self.runtime;
                let mut batch: Vec<(String, Vec<f32>, Option<serde_json::Value>)> =
                    Vec::with_capacity(docs.len());
                for (id_obj, vector, metadata) in &docs {
                    let id_str = validate_id_obj(id_obj)?;
                    let v = vector.as_slice().map_err(|e| {
                        pyo3::exceptions::PyValueError::new_err(format!("Invalid vector: {}", e))
                    })?;
                    validate_vector(v)?;
                    let m = opt_dict_to_json(metadata.as_ref())?;
                    batch.push((id_str, v.to_vec(), m));
                }
                run_blocking_ffi(py, &self.poisoned, || {
                    rt.block_on(self.inner.upsert_many(&batch))
                        .map_err(contextra_err)
                })
            }
        }
    };
}
