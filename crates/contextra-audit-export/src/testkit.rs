//! Test kit utilities for `contextra-audit-export`.

use crate::{
    AuditExportError, DeletionProofSummary, EgressEventSummary, ProcessingRegisterEntry,
    ProcessingRegisterSource,
};
use contextra_types::TenantId;

/// In-memory implementation of [`ProcessingRegisterSource`] for testing purposes.
#[derive(Debug, Default, Clone)]
pub struct InMemoryProcessingRegisterSource {
    entries: Vec<ProcessingRegisterEntry>,
}

impl InMemoryProcessingRegisterSource {
    /// Creates a new empty [`InMemoryProcessingRegisterSource`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an [`InMemoryProcessingRegisterSource`] pre-populated with synthetic sample data for `tenant_id`.
    pub fn with_sample_data_for(tenant_id: TenantId) -> Self {
        let entry1 = ProcessingRegisterEntry {
            tenant_id,
            processing_purpose: "Kontextsensitive Vektorsuche".to_string(),
            data_categories: vec!["Protokolldaten".to_string(), "Search Queries".to_string()],
            legal_basis: "Art. 6 Abs. 1 lit. b DSGVO".to_string(),
            egress_events: vec![EgressEventSummary {
                event_id: "egress-evt-101".to_string(),
                destination: "https://llm.provider.example/v1/embeddings".to_string(),
                timestamp_nanos: 1_700_000_000_000_000_000,
                detail: "Outbound query embedding vector calculation".to_string(),
            }],
            deletion_proofs: vec![DeletionProofSummary {
                proof_id: "del-proof-201".to_string(),
                scope: "doc_id:98765".to_string(),
                timestamp_nanos: 1_700_000_050_000_000_000,
                verified: true,
            }],
            generated_at: 1_700_000_100_000_000_000,
        };

        let entry2 = ProcessingRegisterEntry {
            tenant_id,
            processing_purpose: "Dokumenten-Graph-Relationierung".to_string(),
            data_categories: vec!["Entitäten".to_string(), "Metadaten".to_string()],
            legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),
            egress_events: vec![],
            deletion_proofs: vec![DeletionProofSummary {
                proof_id: "del-proof-202".to_string(),
                scope: "entity_id:54321".to_string(),
                timestamp_nanos: 1_700_000_080_000_000_000,
                verified: true,
            }],
            generated_at: 1_700_000_100_000_000_000,
        };

        Self {
            entries: vec![entry1, entry2],
        }
    }

    /// Appends a custom [`ProcessingRegisterEntry`] record.
    pub fn add_entry(&mut self, entry: ProcessingRegisterEntry) {
        self.entries.push(entry);
    }
}

impl ProcessingRegisterSource for InMemoryProcessingRegisterSource {
    fn collect_entries(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<ProcessingRegisterEntry>, AuditExportError> {
        let matched = self
            .entries
            .iter()
            .filter(|e| e.tenant_id == tenant_id)
            .cloned()
            .collect();
        Ok(matched)
    }
}
