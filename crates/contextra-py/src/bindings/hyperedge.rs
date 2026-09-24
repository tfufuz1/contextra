use pyo3::prelude::*;

use crate::bindings::collection::PyCollection;
use crate::bindings::common::*;

/// Validates arguments for creating an n-ary hyperedge.
pub fn validate_hyperedge_args(
    predicate: &str,
    participants: &[(&str, &str)],
    source_doc_id: Option<&str>,
) -> PyResult<()> {
    if participants.len() < 2 {
        return Err(ContextraValueError::new_err(format!(
            "relate_n_ary requires at least 2 participants, found {}",
            participants.len()
        )));
    }
    if participants.len() > 64 {
        return Err(ContextraValueError::new_err(format!(
            "relate_n_ary supports at most 64 participants, found {}",
            participants.len()
        )));
    }

    validate_label(predicate)?;

    for &(doc_id, role) in participants {
        validate_id(doc_id)?;
        validate_label(role)?;
    }

    if let Some(src_id) = source_doc_id {
        validate_id(src_id)?;
    }

    Ok(())
}

#[pymethods]
impl PyCollection {
    /// Legt eine n-äre Hyperkante an. `participants` = [(doc_id, role), ...], mindestens 2.
    #[pyo3(signature = (predicate, participants, source_doc_id=None))]
    pub fn relate_n_ary<'py>(
        &self,
        py: Python<'py>,
        predicate: &str,
        participants: Vec<(Bound<'py, PyAny>, String)>,
        source_doc_id: Option<Bound<'py, PyAny>>,
    ) -> PyResult<u64> {
        let mut extracted_participants = Vec::with_capacity(participants.len());
        for (id_obj, role) in &participants {
            let doc_id = validate_id_obj(id_obj)?;
            extracted_participants.push((doc_id, role.clone()));
        }

        let extracted_source_doc_id = match source_doc_id.as_ref() {
            Some(obj) => Some(validate_id_obj(obj)?),
            None => None,
        };

        let part_refs: Vec<(&str, &str)> = extracted_participants
            .iter()
            .map(|(d, r)| (d.as_str(), r.as_str()))
            .collect();

        validate_hyperedge_args(predicate, &part_refs, extracted_source_doc_id.as_deref())?;

        let rt = &self.runtime;
        let predicate_owned = predicate.to_string();
        let source_doc_id_owned = extracted_source_doc_id;

        let hyperedge_id = run_blocking_ffi(py, &self.poisoned, || {
            let part_slices: Vec<(&str, &str)> = extracted_participants
                .iter()
                .map(|(d, r)| (d.as_str(), r.as_str()))
                .collect();

            rt.block_on(self.inner.relate_n_ary(
                &predicate_owned,
                &part_slices,
                source_doc_id_owned.as_deref(),
            ))
            .map_err(contextra_err)
        })?;

        Ok(hyperedge_id.inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hyperedge_args_success() {
        pyo3::prepare_freethreaded_python();
        let participants = vec![("doc-1", "author"), ("doc-2", "reviewer")];
        assert!(validate_hyperedge_args("collaborated", &participants, None).is_ok());
        assert!(validate_hyperedge_args("collaborated", &participants, Some("src-doc")).is_ok());
    }

    #[test]
    fn test_validate_hyperedge_args_participant_count_bounds() {
        pyo3::prepare_freethreaded_python();
        // < 2 participants
        let single = vec![("doc-1", "author")];
        assert!(validate_hyperedge_args("pred", &single, None).is_err());

        let empty: Vec<(&str, &str)> = vec![];
        assert!(validate_hyperedge_args("pred", &empty, None).is_err());

        // > 64 participants
        let many_vec: Vec<(String, String)> = (0..65)
            .map(|i| (format!("doc-{}", i), "role".to_string()))
            .collect();
        let many: Vec<(&str, &str)> = many_vec
            .iter()
            .map(|(d, r)| (d.as_str(), r.as_str()))
            .collect();
        assert!(validate_hyperedge_args("pred", &many, None).is_err());

        // Exactly 64 participants is valid
        let max64_vec: Vec<(String, String)> = (0..64)
            .map(|i| (format!("doc-{}", i), "role".to_string()))
            .collect();
        let max64: Vec<(&str, &str)> = max64_vec
            .iter()
            .map(|(d, r)| (d.as_str(), r.as_str()))
            .collect();
        assert!(validate_hyperedge_args("pred", &max64, None).is_ok());
    }

    #[test]
    fn test_validate_hyperedge_args_invalid_labels_and_ids() {
        pyo3::prepare_freethreaded_python();
        let valid = vec![("doc-1", "author"), ("doc-2", "reviewer")];

        // Empty predicate
        assert!(validate_hyperedge_args("", &valid, None).is_err());
        assert!(validate_hyperedge_args("   ", &valid, None).is_err());

        // Empty role
        let empty_role = vec![("doc-1", "author"), ("doc-2", "")];
        assert!(validate_hyperedge_args("pred", &empty_role, None).is_err());

        // Empty doc id
        let empty_id = vec![("doc-1", "author"), ("", "reviewer")];
        assert!(validate_hyperedge_args("pred", &empty_id, None).is_err());

        // Invalid source_doc_id
        assert!(validate_hyperedge_args("pred", &valid, Some("")).is_err());
    }
}
