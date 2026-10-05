#![forbid(unsafe_code)]

//! `contextra-avv-generator`
//!
//! Generates AVV (Auftragsverarbeitungsvertrag) template documents referencing Contextra's
//! technical guarantees (deletion proof, egress gateway, multi-tenant isolation, encrypted KV-cache).

pub mod error;
pub mod template;

pub use error::AvvGeneratorError;

use contextra_types::TenantId;
use serde::{Deserialize, Serialize};

/// Context parameter container for generating an AVV document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvvContext {
    /// Controller / Responsible Party Name (e.g. Kanzlei / Praxis)
    pub controller_name: String,
    /// Processor / Operator Name (Contextra Operator)
    pub processor_name: String,
    /// Tenant Identifier for personalisation
    pub tenant_id: TenantId,
    /// Technical and organizational measures implemented
    pub technical_measures: Vec<TechnicalMeasure>,
    /// List of subprocessors (if any)
    pub subprocessors: Vec<String>,
    /// SLA for deletion in days after contract termination
    pub deletion_sla_days: u32,
}

impl AvvContext {
    /// Creates a new [`AvvContext`] initialized with Contextra's default technical measures and SLA (14 days).
    pub fn new(
        controller_name: impl Into<String>,
        processor_name: impl Into<String>,
        tenant_id: TenantId,
    ) -> Self {
        Self {
            controller_name: controller_name.into(),
            processor_name: processor_name.into(),
            tenant_id,
            technical_measures: default_technical_measures(),
            subprocessors: Vec::new(),
            deletion_sla_days: 14,
        }
    }

    /// Builder pattern entry point for constructing an [`AvvContext`].
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub fn builder(
        controller_name: impl Into<String>,
        processor_name: impl Into<String>,
        tenant_id: TenantId,
    ) -> AvvContextBuilder {
        AvvContextBuilder::new(controller_name, processor_name, tenant_id)
    }

    /// Renders this AVV context into a Markdown document using [`render_avv_markdown`].
    pub fn render(&self) -> Result<String, AvvGeneratorError> {
        render_avv_markdown(self)
    }
}

/// Builder for constructing and customizing an [`AvvContext`].
#[derive(Debug, Clone)]
pub struct AvvContextBuilder {
    ctx: AvvContext,
}

impl AvvContextBuilder {
    /// Creates a new [`AvvContextBuilder`] initialized with default technical measures.
    pub fn new(
        controller_name: impl Into<String>,
        processor_name: impl Into<String>,
        tenant_id: TenantId,
    ) -> Self {
        Self {
            ctx: AvvContext::new(controller_name, processor_name, tenant_id),
        }
    }

    /// Overrides the list of technical measures.
    pub fn with_technical_measures(mut self, measures: Vec<TechnicalMeasure>) -> Self {
        self.ctx.technical_measures = measures;
        self
    }

    /// Appends a technical measure to the list.
    pub fn add_technical_measure(mut self, measure: TechnicalMeasure) -> Self {
        self.ctx.technical_measures.push(measure);
        self
    }

    /// Overrides the list of subprocessors.
    pub fn with_subprocessors(mut self, subprocessors: Vec<String>) -> Self {
        self.ctx.subprocessors = subprocessors;
        self
    }

    /// Appends a subprocessor to the list.
    pub fn add_subprocessor(mut self, subprocessor: impl Into<String>) -> Self {
        self.ctx.subprocessors.push(subprocessor.into());
        self
    }

    /// Sets the SLA for deletion in days.
    pub fn with_deletion_sla_days(mut self, days: u32) -> Self {
        self.ctx.deletion_sla_days = days;
        self
    }

    /// Returns the built [`AvvContext`].
    pub fn build(self) -> AvvContext {
        self.ctx
    }

    /// Directly renders the built [`AvvContext`] into a Markdown document.
    pub fn render(self) -> Result<String, AvvGeneratorError> {
        self.ctx.render()
    }
}

/// Description of a Technical and Organizational Measure (TOM).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TechnicalMeasure {
    /// Name of measure (e.g. "Kryptografischer Löschbeweis (Ed25519)")
    pub name: String,
    /// Detailed technical description of measure
    pub description: String,
    /// Reference article (e.g. "Art. 32 DSGVO")
    pub reference_article: String,
}

/// Renders the AVV document in Markdown format using the provided [`AvvContext`].
pub fn render_avv_markdown(ctx: &AvvContext) -> Result<String, AvvGeneratorError> {
    template::render(ctx)
}

/// Returns the default list of technical measures derived from Contextra's product guarantees.
pub fn default_technical_measures() -> Vec<TechnicalMeasure> {
    vec![
        TechnicalMeasure {
            name: "Kryptografischer Löschbeweis (Ed25519)".to_string(),
            description: "Nach Ausführung einer Löschanforderung wird ein fälschungssicherer, Ed25519-signierter Löschbeleg generiert, der die unwiderrufliche Löschung aller Datenpunkte mathematisch und auditierbar nachweist.".to_string(),
            reference_article: "Art. 32 Abs. 1 lit. b & Art. 17 DSGVO".to_string(),
        },
        TechnicalMeasure {
            name: "Zero-Egress-Default / Egress-Gateway".to_string(),
            description: "Sämtlicher Datenverkehr nach außen ist standardmäßig blockiert. Ausgehende Anfragen verlaufen erzwingend über ein abgesichertes Egress-Gateway mit strikter Whitelist und DLP-Inhaltsprüfung.".to_string(),
            reference_article: "Art. 32 Abs. 1 lit. b DSGVO".to_string(),
        },
        TechnicalMeasure {
            name: "Kryptografische Mandantentrennung".to_string(),
            description: "Vektoren, Indizes und Metadaten sind pro Tenant-ID kryptografisch isoliert. Zugriffe über Mandantengrenzen hinweg werden auf Speicherebene unterbindungssicher verhindert.".to_string(),
            reference_article: "Art. 32 Abs. 1 lit. b DSGVO".to_string(),
        },
        TechnicalMeasure {
            name: "Verschlüsselte Segmente im KV-Cache".to_string(),
            description: "Flüchtige Zwischenspeicher und KV-Cache-Segmente werden im Arbeitsspeicher mit rotierenden Schlüsseln verschlüsselt, um Unbefugten den Zugriff auf Klartextkontexte im Memory-Dump zu verwehren.".to_string(),
            reference_article: "Art. 32 Abs. 1 lit. a DSGVO".to_string(),
        },
    ]
}
