use contextra_core::types::domain::{DocId, LinkRelation};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::bindings::collection::PyCollection;
use crate::bindings::common::*;

/// Parses a string into a `LinkRelation` (case-insensitive, allocation-free).
pub fn parse_link_relation(s: &str) -> PyResult<LinkRelation> {
    if s.eq_ignore_ascii_case("elaborates") {
        Ok(LinkRelation::Elaborates)
    } else if s.eq_ignore_ascii_case("contradicts") {
        Ok(LinkRelation::Contradicts)
    } else if s.eq_ignore_ascii_case("supersedes") {
        Ok(LinkRelation::Supersedes)
    } else if s.eq_ignore_ascii_case("references") {
        Ok(LinkRelation::References)
    } else {
        Err(PyValueError::new_err(format!(
            "Unknown link relation '{}'. Valid relations are: 'elaborates', 'contradicts', 'supersedes', 'references'",
            s
        )))
    }
}

fn relation_as_str(r: LinkRelation) -> &'static str {
    match r {
        LinkRelation::Elaborates => "elaborates",
        LinkRelation::Contradicts => "contradicts",
        LinkRelation::Supersedes => "supersedes",
        LinkRelation::References => "references",
    }
}

#[pymethods]
impl PyCollection {
    /// Stores a non-vector key-value entry directly in storage without modifying indices.
    pub fn put_kv<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
        value: &Bound<'py, PyDict>,
    ) -> PyResult<()> {
        let id_str = validate_id_obj(id)?;
        let json_val = dict_to_json(value)?;
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.put_kv(&id_str, &json_val))
                .map_err(contextra_err)
        })
    }

    /// Stores a non-vector key-value entry directly in storage only if the key does not already exist.
    pub fn put_kv_if_absent<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
        value: &Bound<'py, PyDict>,
    ) -> PyResult<()> {
        let id_str = validate_id_obj(id)?;
        let json_val = dict_to_json(value)?;
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.put_kv_if_absent(&id_str, &json_val))
                .map_err(contextra_err)
        })
    }

    /// Retrieves a non-vector key-value entry directly from storage.
    pub fn get_kv<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
    ) -> PyResult<Option<PyObject>> {
        let id_str = validate_id_obj(id)?;
        let rt = &self.runtime;
        let opt_val = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.get_kv(&id_str))
                .map_err(contextra_err)
        })?;
        match opt_val {
            Some(val) => Ok(Some(json_to_py(py, &val)?)),
            None => Ok(None),
        }
    }

    /// Creates a bidirectional relationship between two documents atomically.
    pub fn relate_bidirectional<'py>(
        &self,
        py: Python<'py>,
        from: &Bound<'py, PyAny>,
        to: &Bound<'py, PyAny>,
        label: &str,
    ) -> PyResult<()> {
        let from_str = validate_id_obj(from)?;
        let to_str = validate_id_obj(to)?;
        validate_label(label)?;
        let rt = &self.runtime;
        let label_owned = label.to_string();
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(
                self.inner
                    .relate_bidirectional(&from_str, &to_str, &label_owned),
            )
            .map_err(contextra_err)
        })
    }

    /// Links two memories together with a specific relation (Zettelkasten A-MEM).
    pub fn link_memories<'py>(
        &self,
        py: Python<'py>,
        from: &Bound<'py, PyAny>,
        to: &Bound<'py, PyAny>,
        relation: &str,
    ) -> PyResult<()> {
        let from_str = validate_id_obj(from)?;
        let to_str = validate_id_obj(to)?;
        let rel = parse_link_relation(relation)?;
        let from_doc_id = DocId::from_key(&from_str).map_err(contextra_err)?;
        let to_doc_id = DocId::from_key(&to_str).map_err(contextra_err)?;
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.link_memories(from_doc_id, to_doc_id, rel))
                .map_err(contextra_err)
        })
    }

    /// Retrieves all memory links for a given document.
    pub fn get_links<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
    ) -> PyResult<Vec<PyObject>> {
        let id_str = validate_id_obj(id)?;
        let doc_id = DocId::from_key(&id_str).map_err(contextra_err)?;
        let rt = &self.runtime;
        let links = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.get_links(doc_id))
                .map_err(contextra_err)
        })?;

        let mut py_res = Vec::with_capacity(links.len());
        for link in links {
            let dict = PyDict::new(py);
            dict.set_item("target", link.target.inner())?;
            dict.set_item("relation", relation_as_str(link.relation))?;
            dict.set_item("created_at_tx", link.created_at_tx.inner())?;
            py_res.push(dict.into_any().unbind());
        }
        Ok(py_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link_relation_valid_cases() {
        assert_eq!(
            parse_link_relation("elaborates").expect("should parse"),
            LinkRelation::Elaborates
        );
        assert_eq!(
            parse_link_relation("Elaborates").expect("should parse"),
            LinkRelation::Elaborates
        );
        assert_eq!(
            parse_link_relation("ELABORATES").expect("should parse"),
            LinkRelation::Elaborates
        );

        assert_eq!(
            parse_link_relation("contradicts").expect("should parse"),
            LinkRelation::Contradicts
        );
        assert_eq!(
            parse_link_relation("Contradicts").expect("should parse"),
            LinkRelation::Contradicts
        );

        assert_eq!(
            parse_link_relation("supersedes").expect("should parse"),
            LinkRelation::Supersedes
        );
        assert_eq!(
            parse_link_relation("SuperSedes").expect("should parse"),
            LinkRelation::Supersedes
        );

        assert_eq!(
            parse_link_relation("references").expect("should parse"),
            LinkRelation::References
        );
        assert_eq!(
            parse_link_relation("REFERENCES").expect("should parse"),
            LinkRelation::References
        );
    }

    #[test]
    fn test_parse_link_relation_invalid_cases() {
        assert!(parse_link_relation("unknown").is_err());
        assert!(parse_link_relation("").is_err());
        assert!(parse_link_relation("related").is_err());
    }
}
