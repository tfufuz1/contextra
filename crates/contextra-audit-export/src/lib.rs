//! `contextra-audit-export` — GDPR Article 30 Processing Register export generator.
//!
//! # Architecture Role (Building Block, Class A)
//! This crate provides an independent, autarkic export tool for generating processing registers
//! (Verzeichnis von Verarbeitungstätigkeiten nach Art. 30 DSGVO) based on neutral audit records.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod bsi_mapping;
pub mod csv_template;
pub mod error;
pub mod markdown_template;
pub mod testkit;

pub use bsi_mapping::{bsi_mapping_table, render_bsi_mapping_markdown, BsiMappingEntry};
pub use csv_template::render_register_csv;
pub use error::AuditExportError;
pub use markdown_template::render_register_markdown;

use core::fmt;
use serde::{Deserialize, Serialize};

/// Canonical local tenant identifier for data isolation in compliance audit exports.
///
/// Implements standard traits and conversions to standard primitive types (`u64`).
/// When compiled with feature `types-compat`, transparent conversions to/from
/// `contextra_types::TenantId` are provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TenantId(pub u64);

impl TenantId {
    /// Reserved identifier for system-internal contexts.
    pub const SYSTEM: Self = Self(0);

    /// Constructs a valid user `TenantId`.
    ///
    /// # Errors
    /// Returns [`AuditExportError::InvalidTenant`] if `id == 0`, as tenant ID 0 is strictly
    /// reserved for system context.
    pub fn try_new(id: u64) -> Result<Self, AuditExportError> {
        if id == 0 {
            Err(AuditExportError::InvalidTenant(
                "TenantId(0) is reserved for system context".to_string(),
            ))
        } else {
            Ok(Self(id))
        }
    }

    /// Constructs a `TenantId` without validation.
    #[inline]
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying 64-bit integer value.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the underlying 64-bit integer value.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Checks if this tenant is the reserved system tenant (ID 0).
    #[inline]
    pub const fn is_system(self) -> bool {
        self.0 == 0
    }
}

impl Default for TenantId {
    #[inline]
    fn default() -> Self {
        Self::SYSTEM
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TenantId({})", self.0)
    }
}

impl From<TenantId> for u64 {
    #[inline]
    fn from(t: TenantId) -> Self {
        t.0
    }
}

impl TryFrom<u64> for TenantId {
    type Error = AuditExportError;

    #[inline]
    fn try_from(id: u64) -> Result<Self, Self::Error> {
        Self::try_new(id)
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::TenantId> for TenantId {
    #[inline]
    fn from(t: contextra_types::TenantId) -> Self {
        Self(t.as_u64())
    }
}

#[cfg(feature = "types-compat")]
impl From<TenantId> for contextra_types::TenantId {
    #[inline]
    fn from(t: TenantId) -> Self {
        contextra_types::TenantId(t.0)
    }
}

/// Contact details for the data controller, representative, and data protection officer (Art. 30 Abs. 1 lit. a DSGVO).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerContact {
    /// Name or company name of the data controller (Verantwortlicher).
    pub controller_name: String,
    /// Contact details of the data controller (e.g., address, email, phone).
    pub controller_contact: String,
    /// Optional representative of the controller (Vertreter des Verantwortlichen).
    pub representative_contact: Option<String>,
    /// Optional Data Protection Officer (Datenschutzbeauftragter / DSB).
    pub dpo_contact: Option<String>,
}

impl ControllerContact {
    /// Validates mandatory fields in controller contact details.
    pub fn validate(&self) -> Result<(), AuditExportError> {
        if self.controller_name.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "controller_contact.controller_name".to_string(),
            });
        }
        if self.controller_contact.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "controller_contact.controller_contact".to_string(),
            });
        }
        Ok(())
    }
}

/// Third country transfer record including legal safeguard guarantees (Art. 30 Abs. 1 lit. e DSGVO).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ThirdCountryTransfer {
    /// Destination third country or international organization.
    pub destination_country: String,
    /// Legal safeguards / guarantees (e.g., EU Standard Contractual Clauses, Adequacy Decision).
    pub safeguard_guarantee: String,
}

impl ThirdCountryTransfer {
    /// Validates mandatory fields in third country transfer details.
    pub fn validate(&self) -> Result<(), AuditExportError> {
        if self.destination_country.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "third_country_transfer.destination_country".to_string(),
            });
        }
        if self.safeguard_guarantee.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "third_country_transfer.safeguard_guarantee".to_string(),
            });
        }
        Ok(())
    }
}

/// Retention period definition per category of personal data (Art. 30 Abs. 1 lit. f DSGVO).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct RetentionPeriod {
    /// Category of personal data associated with this retention policy.
    pub data_category: String,
    /// Time limit or rule for erasure / retention period (Löschfrist).
    pub retention_period: String,
}

impl RetentionPeriod {
    /// Validates mandatory fields in retention period rules.
    pub fn validate(&self) -> Result<(), AuditExportError> {
        if self.data_category.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "retention_period.data_category".to_string(),
            });
        }
        if self.retention_period.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "retention_period.retention_period".to_string(),
            });
        }
        Ok(())
    }
}

/// Summary of an egress gateway event recorded for audit verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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
    /// Associated tenant identifier ([`TenantId`]).
    pub tenant_id: TenantId,
    /// Contact details of data controller, representative, and DSB (Art. 30 Abs. 1 lit. a).
    pub controller_contact: ControllerContact,
    /// Specific purpose of processing (Verarbeitungszweck, Art. 30 Abs. 1 lit. b).
    pub processing_purpose: String,
    /// Categories of data subjects (Kategorien betroffener Personen, Art. 30 Abs. 1 lit. c).
    pub data_subject_categories: Vec<String>,
    /// Categories of personal data processed (Kategorien personenbezogener Daten, Art. 30 Abs. 1 lit. c).
    pub data_categories: Vec<String>,
    /// Categories of recipients to whom data is disclosed (Empfängerkategorien, Art. 30 Abs. 1 lit. d).
    pub recipient_categories: Vec<String>,
    /// Third country transfers and safeguards (Drittlandübermittlungen, Art. 30 Abs. 1 lit. e).
    pub third_country_transfers: Vec<ThirdCountryTransfer>,
    /// Retention periods and time limits for erasure (Löschfristen, Art. 30 Abs. 1 lit. f).
    pub retention_periods: Vec<RetentionPeriod>,
    /// General description of technical and organizational measures (TOM, Art. 30 Abs. 1 lit. g / Art. 32).
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

impl ProcessingRegisterEntry {
    /// Validates all mandatory fields required by GDPR Article 30 Abs. 1.
    ///
    /// Returns [`AuditExportError::MissingField`] if any statutory field is missing or empty.
    pub fn validate(&self) -> Result<(), AuditExportError> {
        self.controller_contact.validate()?;

        if self.processing_purpose.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "processing_purpose".to_string(),
            });
        }
        if self.legal_basis.trim().is_empty() {
            return Err(AuditExportError::MissingField {
                field: "legal_basis".to_string(),
            });
        }
        if self.data_subject_categories.is_empty()
            || self
                .data_subject_categories
                .iter()
                .any(|s| s.trim().is_empty())
        {
            return Err(AuditExportError::MissingField {
                field: "data_subject_categories".to_string(),
            });
        }
        if self.data_categories.is_empty()
            || self.data_categories.iter().any(|s| s.trim().is_empty())
        {
            return Err(AuditExportError::MissingField {
                field: "data_categories".to_string(),
            });
        }
        if self.recipient_categories.is_empty()
            || self
                .recipient_categories
                .iter()
                .any(|s| s.trim().is_empty())
        {
            return Err(AuditExportError::MissingField {
                field: "recipient_categories".to_string(),
            });
        }
        for transfer in &self.third_country_transfers {
            transfer.validate()?;
        }
        if self.retention_periods.is_empty() {
            return Err(AuditExportError::MissingField {
                field: "retention_periods".to_string(),
            });
        }
        for period in &self.retention_periods {
            period.validate()?;
        }
        if self.technical_organizational_measures.is_empty()
            || self
                .technical_organizational_measures
                .iter()
                .any(|s| s.trim().is_empty())
        {
            return Err(AuditExportError::MissingField {
                field: "technical_organizational_measures".to_string(),
            });
        }

        Ok(())
    }

    /// Sorts internal lists deterministically in place.
    pub fn sort_internal_collections(&mut self) {
        self.data_subject_categories.sort();
        self.data_categories.sort();
        self.recipient_categories.sort();
        self.third_country_transfers.sort();
        self.retention_periods.sort();
        self.technical_organizational_measures.sort();
        self.egress_events.sort();
        self.deletion_proofs.sort();
    }
}

/// Abstract data source for retrieving processing register records.
pub trait ProcessingRegisterSource {
    /// Collects processing register entries for the specified tenant.
    fn collect_entries(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<ProcessingRegisterEntry>, AuditExportError>;
}

/// Neutral record source trait for streaming audit records (B1, B3).
pub trait RecordSource {
    /// Record item type.
    type Record;

    /// Collects records into a vector.
    fn collect_records(&self) -> Result<Vec<Self::Record>, AuditExportError>;
}

/// Renders a list of processing register entries as a formatted JSON string.
///
/// Entries and internal collections are deterministically sorted prior to serialization.
/// All entries are validated according to Art. 30 Abs. 1 DSGVO.
pub fn render_register_json(
    entries: &[ProcessingRegisterEntry],
) -> Result<String, AuditExportError> {
    let mut sorted_entries = entries.to_vec();
    for entry in &mut sorted_entries {
        entry.validate()?;
        entry.sort_internal_collections();
    }
    sorted_entries.sort_by(|a, b| {
        a.tenant_id
            .as_u64()
            .cmp(&b.tenant_id.as_u64())
            .then_with(|| a.processing_purpose.cmp(&b.processing_purpose))
            .then_with(|| {
                a.controller_contact
                    .controller_name
                    .cmp(&b.controller_contact.controller_name)
            })
    });

    serde_json::to_string_pretty(&sorted_entries).map_err(AuditExportError::from)
}
