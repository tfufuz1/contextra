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
