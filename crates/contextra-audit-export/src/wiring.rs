//! Wiring entry point for `contextra-audit-export` (W3-01 / Rule 7).

use crate::{
    AuditRegisterExporter, DeletionProofSummary, EgressEventSummary, ProcessingRegisterEntry,
    ProcessingRegisterSource,
};
use contextra_ports::clock::Clock;
use contextra_types::{ContextraError, TenantId};
use std::fmt;
use std::sync::Arc;

/// Entry point wiring structure for managing audit register exports with a [`Clock`] port.
pub struct AuditExportWiring<S> {
    exporter: AuditRegisterExporter<S>,
    clock: Arc<dyn Clock>,
}

impl<S: fmt::Debug> fmt::Debug for AuditExportWiring<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuditExportWiring")
            .field("exporter", &self.exporter)
            .field("clock", &"<dyn Clock>")
            .finish()
    }
}

impl<S: ProcessingRegisterSource> AuditExportWiring<S> {
    /// Builds a new [`AuditExportWiring`] with the given source and clock.
    pub fn build(source: S, clock: Arc<dyn Clock>) -> Result<Self, ContextraError> {
        Ok(Self {
            exporter: AuditRegisterExporter::new(source),
            clock,
        })
    }

    /// Returns a reference to the underlying exporter.
    pub fn exporter(&self) -> &AuditRegisterExporter<S> {
        &self.exporter
    }

    /// Returns a reference to the injected clock port.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Creates a new [`ProcessingRegisterEntry`] stamped with the current time from the injected [`Clock`].
    pub fn create_entry(
        &self,
        tenant_id: TenantId,
        processing_purpose: impl Into<String>,
        data_categories: Vec<String>,
        legal_basis: impl Into<String>,
        egress_events: Vec<EgressEventSummary>,
        deletion_proofs: Vec<DeletionProofSummary>,
    ) -> ProcessingRegisterEntry {
        ProcessingRegisterEntry {
            tenant_id,
            processing_purpose: processing_purpose.into(),
            data_categories,
            legal_basis: legal_basis.into(),
            egress_events,
            deletion_proofs,
            generated_at: self.clock.now_unix_nanos(),
        }
    }

    /// Exports processing register entries for `tenant_id` as a formatted JSON string.
    pub fn export_json(&self, tenant_id: TenantId) -> Result<String, ContextraError> {
        self.exporter
            .export_json(tenant_id)
            .map_err(|e| ContextraError::Internal(e.to_string()))
    }

    /// Exports processing register entries for `tenant_id` as a Markdown document.
    pub fn export_markdown(&self, tenant_id: TenantId) -> Result<String, ContextraError> {
        self.exporter
            .export_markdown(tenant_id)
            .map_err(|e| ContextraError::Internal(e.to_string()))
    }
}

impl AuditExportWiring<crate::testkit::InMemoryProcessingRegisterSource> {
    /// Creates an in-memory audit export wiring instance pre-populated with sample data for `tenant_id`.
    pub fn with_sample_data_for(tenant_id: TenantId, clock: Arc<dyn Clock>) -> Self {
        Self {
            exporter: AuditRegisterExporter::with_sample_data_for(tenant_id),
            clock,
        }
    }

    /// Appends a processing register entry to the underlying in-memory register source.
    pub fn add_entry(&mut self, entry: ProcessingRegisterEntry) {
        self.exporter.add_entry(entry);
    }
}
