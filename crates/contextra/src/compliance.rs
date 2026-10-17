// FILE-CONTEXT
// ZWECK: GDPR Compliance facade module (AVV draft generation & Art. 30 processing register export).
// INVARIANTEN: Zero-panic in production code; sanitize inputs via escape_markdown; do NOT use test sources in production.

use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, AvvContext, TechnicalMeasure,
};
use contextra_core::error::ContextraError;
use contextra_db::Contextra;
use contextra_types::markdown::escape_markdown;
use contextra_types::TenantId;
use serde::{Deserialize, Serialize};

/// Review header prepended to generated compliance drafts.
pub const REVIEW_HEADER: &str = "<!-- Contextra AVV Draft Review Header: Legal review required prior to execution. -->\n";

/// Parameters for generating an AVV (Auftragsverarbeitungsvertrag) draft document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvvParams {
    /// Controller / Responsible Party Name (e.g., Kanzlei / Praxis)
    pub controller_name: String,
    /// Processor / Operator Name
    pub processor_name: String,
    /// Tenant Identifier
    pub tenant_id: TenantId,
    /// Technical and organizational measures (TOMs). If `None` or empty, defaults will be used.
    pub technical_measures: Option<Vec<TechnicalMeasure>>,
    /// List of subprocessors
    pub subprocessors: Vec<String>,
    /// SLA for deletion in days after contract termination
    pub deletion_sla_days: u32,
}

/// Metadata associated with an AVV draft document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvvMetadata {
    /// Associated tenant identifier
    pub tenant_id: TenantId,
    /// Configured deletion SLA in days
    pub deletion_sla_days: u32,
    /// Generation timestamp in Unix nanoseconds
    pub generated_at_nanos: u64,
}

/// Generated AVV draft containing review header, Markdown document, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvvDraft {
    /// Review header comment
    pub review_header: String,
    /// Full rendered Markdown text including the review header
    pub markdown: String,
    /// Metadata associated with the draft
    pub metadata: AvvMetadata,
}

/// Time range filter for audit export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start timestamp in Unix nanoseconds
    pub start_nanos: u64,
    /// End timestamp in Unix nanoseconds
    pub end_nanos: u64,
}

/// Exported processing register (Art. 30 DSGVO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterExport {
    /// Formatted JSON payload
    pub content_json: String,
    /// Formatted Markdown document
    pub content_markdown: String,
    /// Number of register entries exported
    pub entry_count: usize,
}

/// Compliance-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum ComplianceError {
    /// Indicates that no production data source for the processing register is available.
    #[error("ComplianceSourceUnavailable: No production ProcessingRegisterSource implementation available")]
    ComplianceSourceUnavailable,

    /// Error encountered during AVV generation.
    #[error("AVV generator error: {0}")]
    AvvGenerator(#[from] contextra_avv_generator::AvvGeneratorError),

    /// Error encountered during audit export.
    #[error("Audit export error: {0}")]
    AuditExport(#[from] contextra_audit_export::AuditExportError),
}

impl From<ComplianceError> for ContextraError {
    fn from(err: ComplianceError) -> Self {
        match err {
            ComplianceError::ComplianceSourceUnavailable => ContextraError::capability_unsupported(
                "export_processing_register",
                err.to_string(),
            ),
            ComplianceError::AvvGenerator(e) => ContextraError::InvalidInput(e.to_string()),
            ComplianceError::AuditExport(e) => ContextraError::Internal(e.to_string()),
        }
    }
}

impl Contextra {
    /// Generates an AVV (Auftragsverarbeitungsvertrag) draft document according to Art. 28 DSGVO.
    ///
    /// Escapes hostile inputs in parameter fields using `escape_markdown` before rendering.
    /// Includes the standard review header in the draft output.
    pub fn generate_avv_draft(&self, params: &AvvParams) -> Result<AvvDraft, ContextraError> {
        let escaped_controller = escape_markdown(&params.controller_name);
        let escaped_processor = escape_markdown(&params.processor_name);
        let escaped_subprocessors: Vec<String> = params
            .subprocessors
            .iter()
            .map(|s| escape_markdown(s))
            .collect();

        let measures = match &params.technical_measures {
            Some(m) if !m.is_empty() => m
                .iter()
                .map(|tm| TechnicalMeasure {
                    name: escape_markdown(&tm.name),
                    description: escape_markdown(&tm.description),
                    reference_article: escape_markdown(&tm.reference_article),
                })
                .collect(),
            _ => default_technical_measures(),
        };

        let avv_ctx = AvvContext {
            controller_name: escaped_controller,
            processor_name: escaped_processor,
            tenant_id: params.tenant_id,
            technical_measures: measures,
            subprocessors: escaped_subprocessors,
            deletion_sla_days: params.deletion_sla_days,
        };

        let body_md = render_avv_markdown(&avv_ctx)
            .map_err(|e| ContextraError::from(ComplianceError::AvvGenerator(e)))?;

        let full_md = format!("{}{}", REVIEW_HEADER, body_md);

        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        Ok(AvvDraft {
            review_header: REVIEW_HEADER.trim().to_string(),
            markdown: full_md,
            metadata: AvvMetadata {
                tenant_id: params.tenant_id,
                deletion_sla_days: params.deletion_sla_days,
                generated_at_nanos: now_nanos,
            },
        })
    }

    /// Exports the Art. 30 DSGVO processing register for the given time range.
    ///
    /// # Note
    /// Currently returns `ComplianceSourceUnavailable` because no production `ProcessingRegisterSource`
    /// implementation is available. In-memory test sources are strictly prohibited in production code.
    pub async fn export_processing_register(
        &self,
        _range: TimeRange,
    ) -> Result<RegisterExport, ContextraError> {
        Err(ContextraError::from(
            ComplianceError::ComplianceSourceUnavailable,
        ))
    }
}
