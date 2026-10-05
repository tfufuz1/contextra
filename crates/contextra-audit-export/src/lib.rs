//! `contextra-audit-export` — GDPR Article 30 Processing Register export generator.
//!
//! # Architecture Role (Ring 4)
//! This crate provides an external interface for generating automated processing registers
//! (Verzeichnis von Verarbeitungstätigkeiten nach Art. 30 DSGVO) based on egress gateway logs
//! and cryptographic deletion proofs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod bsi_mapping;
pub mod error;
pub mod markdown_template;
pub mod testkit;

pub use bsi_mapping::{bsi_mapping_table, render_bsi_mapping_markdown, BsiMappingEntry};
pub use error::AuditExportError;
pub use markdown_template::{render_art30_register_markdown, render_register_markdown};

use contextra_types::TenantId;
use serde::{Deserialize, Serialize};

/// Summary of an egress gateway event recorded for audit verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgressEventSummary {
    /// Unique identifier of the egress event.
    pub event_id: String,
    /// Target destination or external endpoint.
    pub destination: String,
    /// Timestamp in Unix nanoseconds when egress occurred.
    pub timestamp_nanos: u64,
    /// Additional context or classification metadata.
    pub detail: String,
}

/// Summary of a cryptographic deletion proof (GDPR Art. 17 / Art. 30 compliance).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionProofSummary {
    /// Unique identifier of the deletion proof.
    pub proof_id: String,
    /// Target resource scope or deleted item identifier.
    pub scope: String,
    /// Timestamp in Unix nanoseconds when the proof was issued.
    pub timestamp_nanos: u64,
    /// Verification status of the deletion proof.
    pub verified: bool,
}

/// A record entry in the GDPR Article 30 processing register.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessingRegisterEntry {
    /// Associated tenant identifier (`contextra_types::TenantId`).
    pub tenant_id: TenantId,
    /// Specific purpose of processing (Verarbeitungszweck).
    pub processing_purpose: String,
    /// Categories of personal data processed.
    pub data_categories: Vec<String>,
    /// Statutory legal basis for processing (e.g. Art. 6 Abs. 1 lit. b DSGVO).
    pub legal_basis: String,
    /// Linked egress gateway event summaries.
    pub egress_events: Vec<EgressEventSummary>,
    /// Linked cryptographic deletion proof summaries.
    pub deletion_proofs: Vec<DeletionProofSummary>,
    /// Generation timestamp in Unix nanoseconds (`Clock`-compatible).
    pub generated_at: u64,
}

/// Complete processing register record according to GDPR Article 30 (Art. 30 Abs. 1 lit. a–g DSGVO).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Art30ProcessingRegisterEntry {
    /// Associated tenant identifier (`contextra_types::TenantId`).
    pub tenant_id: TenantId,
    /// Name and contact details of controller/representative/DPO (Art. 30 Abs. 1 lit. a DSGVO).
    pub controller_details: String,
    /// Purposes of the processing (Art. 30 Abs. 1 lit. b DSGVO).
    pub processing_purpose: String,
    /// Categories of data subjects (Art. 30 Abs. 1 lit. c DSGVO).
    pub data_subject_categories: Vec<String>,
    /// Categories of personal data (Art. 30 Abs. 1 lit. c DSGVO).
    pub data_categories: Vec<String>,
    /// Categories of recipients to whom personal data have been/will be disclosed (Art. 30 Abs. 1 lit. d DSGVO).
    pub recipient_categories: Vec<String>,
    /// Transfers of personal data to a third country or international organization, if applicable (Art. 30 Abs. 1 lit. e DSGVO).
    pub third_country_transfers: Option<String>,
    /// Envisaged time limits for erasure of data categories (Art. 30 Abs. 1 lit. f DSGVO).
    pub erasure_deadlines: String,
    /// Technical and organizational security measures (Art. 30 Abs. 1 lit. g / Art. 32 Abs. 1 DSGVO).
    pub technical_organizational_measures: Vec<String>,
    /// Statutory legal basis for processing (e.g. Art. 6 Abs. 1 lit. b DSGVO).
    pub legal_basis: String,
    /// Linked egress gateway event summaries.
    pub egress_events: Vec<EgressEventSummary>,
    /// Linked cryptographic deletion proof summaries.
    pub deletion_proofs: Vec<DeletionProofSummary>,
    /// Generation timestamp in Unix nanoseconds (`Clock`-compatible).
    pub generated_at: u64,
}

/// Abstract data source for retrieving processing register records.
pub trait ProcessingRegisterSource {
    /// Collects processing register entries for the specified tenant.
    fn collect_entries(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<ProcessingRegisterEntry>, AuditExportError>;
}

/// Renders a list of processing register entries as a formatted JSON string.
pub fn render_register_json(
    entries: &[ProcessingRegisterEntry],
) -> Result<String, AuditExportError> {
    serde_json::to_string_pretty(entries).map_err(AuditExportError::from)
}

/// Exporter for generating GDPR Article 30 processing register exports from a [`ProcessingRegisterSource`].
#[derive(Debug, Clone)]
pub struct AuditRegisterExporter<S> {
    source: S,
}

impl<S> AuditRegisterExporter<S> {
    /// Creates a new [`AuditRegisterExporter`] backed by the specified source.
    pub fn new(source: S) -> Self {
        Self { source }
    }

    /// Returns a reference to the underlying processing register source.
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Returns a mutable reference to the underlying processing register source.
    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }
}

impl AuditRegisterExporter<testkit::InMemoryProcessingRegisterSource> {
    /// Creates an exporter pre-populated with synthetic sample processing register entries for `tenant_id`.
    pub fn with_sample_data_for(tenant_id: TenantId) -> Self {
        Self::new(testkit::InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id))
    }

    /// Appends a processing register entry to the underlying in-memory register source.
    pub fn add_entry(&mut self, entry: ProcessingRegisterEntry) {
        self.source.add_entry(entry);
    }
}

impl<S: ProcessingRegisterSource> AuditRegisterExporter<S> {
    /// Exports the processing register entries for `tenant_id` as a formatted JSON string.
    pub fn export_json(&self, tenant_id: TenantId) -> Result<String, AuditExportError> {
        let entries = self.source.collect_entries(tenant_id)?;
        render_register_json(&entries)
    }

    /// Exports the processing register entries for `tenant_id` as a Markdown document.
    pub fn export_markdown(&self, tenant_id: TenantId) -> Result<String, AuditExportError> {
        let entries = self.source.collect_entries(tenant_id)?;
        render_register_markdown(&entries)
    }
}
