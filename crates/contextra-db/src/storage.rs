//! Gate-first tenant-scoped read paths.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use contextra_erasure::{GateDecision, RangeGate, ReadGate};
use contextra_ids::{CollectionId, DocId, SubjectRef, TenantId};
use contextra_privacy::{DataClass, Validated};
use contextra_types::{ContextraError, Result};

use crate::lineage::LineageRef;

/// Stored opaque payload at the DB boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Class declared at the write boundary.
    pub class: DataClass,
    /// Already sealed payload bytes.
    pub payload: Vec<u8>,
    /// Required exactly for D-class records.
    pub lineage: Option<LineageRef>,
}

/// Evidence that the caller wrote the required restricted-read audit record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GateAudit(());

impl GateAudit {
    /// Constructs evidence after the audit sink committed the entry.
    pub fn recorded() -> Self { Self(()) }
}

/// Candidate returned by a retrieval signal before final fusion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HybridCandidate {
    /// Candidate document.
    pub doc_id: DocId,
    /// Signal score.
    pub score: f32,
}

/// The only DB-level storage access point. Every exposed read checks the current
/// erasure gate before touching or returning a document.
pub struct TenantScopedStorage {
    tenant: TenantId,
    collection: CollectionId,
    gate: Arc<dyn ReadGate>,
    records: BTreeMap<DocId, Document>,
    key_locks: BTreeSet<DocId>,
    subject_index: BTreeMap<[u8; 16], BTreeSet<DocId>>,
}

impl TenantScopedStorage {
    /// Creates a tenant- and collection-scoped access boundary.
    pub fn new(tenant: TenantId, collection: CollectionId, gate: Arc<dyn ReadGate>) -> Self {
        Self {
            tenant,
            collection,
            gate,
            records: BTreeMap::new(),
            key_locks: BTreeSet::new(),
            subject_index: BTreeMap::new(),
        }
    }

    /// Writes validated corporate knowledge.
    pub fn remember_k(&mut self, doc: DocId, payload: Validated<Vec<u8>>) -> Result<()> {
        self.insert(doc, Document { class: DataClass::K, payload: payload.into_inner(), lineage: None })
    }

    /// Writes a personal-data unit.
    pub fn remember_p(&mut self, doc: DocId, subject: &SubjectRef, payload: Vec<u8>) -> Result<()> {
        self.insert(doc, Document { class: DataClass::P, payload, lineage: None })?;
        self.subject_index.entry(*subject.as_bytes()).or_default().insert(doc);
        Ok(())
    }

    /// Writes a derived item with mandatory, validated lineage.
    pub fn remember_d(&mut self, doc: DocId, payload: Vec<u8>, lineage: LineageRef) -> Result<()> {
        self.insert(doc, Document { class: DataClass::D, payload, lineage: Some(lineage) })
    }

    /// Installs a coordinator-owned E-class record. Normal callers cannot write E.
    pub(crate) fn remember_e(&mut self, doc: DocId, payload: Vec<u8>) -> Result<()> {
        self.insert(doc, Document { class: DataClass::E, payload, lineage: None })
    }

    fn insert(&mut self, doc: DocId, record: Document) -> Result<()> {
        if self.key_locks.contains(&doc) { return Err(ContextraError::Erased); }
        if record.class == DataClass::D && record.lineage.is_none() {
            return Err(ContextraError::PolicyViolation("derived data requires lineage".into()));
        }
        if record.class != DataClass::D && record.lineage.is_some() {
            return Err(ContextraError::PolicyViolation("only derived data may carry lineage".into()));
        }
        if self.records.contains_key(&doc) {
            return Err(ContextraError::Conflict("document id already occupied".into()));
        }
        self.records.insert(doc, record);
        Ok(())
    }

    /// Locks a unit for the duration of erasure steps 2 through 5.
    pub fn lock_for_erasure(&mut self, doc: DocId) { self.key_locks.insert(doc); }

    /// Resolves the DocDek units affected by a subject erasure without scanning records.
    pub fn documents_for_subject(&self, subject: &SubjectRef) -> Vec<DocId> {
        self.subject_index
            .get(subject.as_bytes())
            .map_or_else(Vec::new, |docs| docs.iter().copied().collect())
    }

    /// Returns a record only after the current gate state allows it.
    pub fn get(&self, doc: DocId, audit: Option<GateAudit>) -> Result<Option<&Document>> {
        self.read_allowed(doc, audit)?;
        Ok(self.records.get(&doc))
    }

    /// Old snapshots share the current gate; they never reopen erased data.
    pub fn get_at_seq(&self, doc: DocId, _seq: u64, audit: Option<GateAudit>) -> Result<Option<&Document>> {
        self.get(doc, audit)
    }

    /// Scans records, omitting erased targets and enforcing audit for restrictions.
    pub fn scan(&self, audit: Option<GateAudit>) -> Result<Vec<(DocId, &Document)>> {
        self.records.iter().filter_map(|(doc, record)| match self.read_allowed(*doc, audit) {
            Ok(()) => Some(Ok((*doc, record))),
            Err(ContextraError::Erased) => None,
            Err(error) => Some(Err(error)),
        }).collect()
    }

    /// Filters retrieval candidates before ranking and deterministically tops up to `k`.
    pub fn hybrid_search(&self, mut candidates: Vec<HybridCandidate>, k: usize, audit: Option<GateAudit>) -> Result<Vec<HybridCandidate>> {
        candidates.sort_by(|left, right| right.score.total_cmp(&left.score).then_with(|| left.doc_id.cmp(&right.doc_id)));
        let mut allowed = Vec::with_capacity(k);
        for candidate in candidates {
            if allowed.len() == k { break; }
            match self.read_allowed(candidate.doc_id, audit) {
                Ok(()) => allowed.push(candidate),
                Err(ContextraError::Erased) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(allowed)
    }

    /// Compaction decision derived from the range gate, preserving restrictions.
    pub fn compaction_range(&self, start: DocId, end: DocId) -> RangeGate {
        self.gate.check_range(&self.key(start), &self.key(end))
    }

    /// Scoped tenant identity.
    pub const fn tenant(&self) -> TenantId { self.tenant }
    /// Scoped collection identity.
    pub const fn collection(&self) -> CollectionId { self.collection }

    fn read_allowed(&self, doc: DocId, audit: Option<GateAudit>) -> Result<()> {
        match self.gate.check(&self.key(doc)) {
            GateDecision::Open => Ok(()),
            GateDecision::Erased => Err(ContextraError::Erased),
            GateDecision::Restricted if audit.is_some() => Ok(()),
            GateDecision::Restricted => Err(ContextraError::RestrictedReadAuditRequired),
        }
    }

    fn key(&self, doc: DocId) -> Vec<u8> {
        let mut key = Vec::with_capacity(32);
        key.extend_from_slice(b"t:");
        key.extend_from_slice(&self.tenant.0.to_be_bytes());
        key.extend_from_slice(b":c:");
        key.extend_from_slice(&self.collection.0.to_be_bytes());
        key.extend_from_slice(b":d:");
        key.extend_from_slice(&doc.to_be_bytes());
        key
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use contextra_erasure::SagaGate;
    use contextra_ids::{CollectionId, DocId, TenantId};
    use contextra_privacy::Validated;

    use super::{HybridCandidate, TenantScopedStorage};

    #[test]
    fn erased_documents_are_never_returned_or_ranked() -> Result<(), Box<dyn std::error::Error>> {
        let gate = Arc::new(SagaGate::new());
        let mut storage = TenantScopedStorage::new(TenantId(7), CollectionId(3), gate.clone());
        let erased = DocId::from_key("erased")?;
        let live = DocId::from_key("live")?;
        let erased_payload = Validated::new_unchecked(b"erased".to_vec());
        let live_payload = Validated::new_unchecked(b"live".to_vec());
        assert!(storage.remember_k(erased, erased_payload).is_ok());
        assert!(storage.remember_k(live, live_payload).is_ok());
        assert!(gate.activate_key(&storage.key(erased)).is_ok());

        assert!(storage.get(erased, None).is_err());
        let results = storage.hybrid_search(
            vec![
                HybridCandidate { doc_id: erased, score: 2.0 },
                HybridCandidate { doc_id: live, score: 1.0 },
            ],
            1,
            None,
        );
        assert!(matches!(results, Ok(results) if results == vec![HybridCandidate { doc_id: live, score: 1.0 }]));
        Ok(())
    }
}
