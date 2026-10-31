#![forbid(unsafe_code)]

//! `contextra-avv-generator`
//!
//! Generates AVV (Auftragsverarbeitungsvertrag) template documents according to Art. 28 GDPR / DSGVO,
//! referencing Contextra's technical guarantees (deletion proof, egress gateway, multi-tenant isolation,
//! encrypted KV-cache).

pub mod assistance;
pub mod audit;
pub mod confidentiality;
pub mod data_subject_rights;
pub mod error;
pub mod instructions;
pub mod template;
pub mod termination;

pub use error::{AvvError, AvvGeneratorError};

use contextra_types::TenantId;
use serde::{Deserialize, Serialize};

/// Context parameter container for generating an AVV document according to Art. 28 DSGVO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AvvContext {
    /// Controller / Responsible Party Name (e.g. Kanzlei / Praxis / Mandant)
    pub controller_name: String,
    /// Processor / Operator Name (e.g. Contextra Operator)
    pub processor_name: String,
    /// Tenant Identifier for personalisation
    pub tenant_id: TenantId,
    /// Technical and organizational measures implemented
    pub technical_measures: Vec<TechnicalMeasure>,
    /// List of approved subprocessors (if any)
    pub subprocessors: Vec<String>,
    /// SLA for deletion in days after contract termination
    pub deletion_sla_days: u32,
    /// Optional date of the contract / agreement (e.g. "2026-10-29").
    #[serde(default)]
    pub contract_date: Option<String>,
    /// Flag indicating whether subprocessors are permitted/authorized.
    /// If `Some(true)` while `subprocessors` list is empty, [`validate`](Self::validate) returns [`AvvError::MissingSubprocessors`].
    #[serde(default)]
    pub subprocessors_allowed: Option<bool>,
}

impl Default for AvvContext {
    fn default() -> Self {
        Self {
            controller_name: String::new(),
            processor_name: String::new(),
            tenant_id: TenantId(0),
            technical_measures: default_technical_measures(),
            subprocessors: Vec::new(),
            deletion_sla_days: 14,
            contract_date: None,
            subprocessors_allowed: None,
        }
    }
}

impl AvvContext {
    /// Creates a new [`AvvContext`] with mandatory parameters and default optional fields.
    pub fn new(
        controller_name: impl Into<String>,
        processor_name: impl Into<String>,
        tenant_id: TenantId,
        technical_measures: Vec<TechnicalMeasure>,
        subprocessors: Vec<String>,
        deletion_sla_days: u32,
    ) -> Self {
        Self {
            controller_name: controller_name.into(),
            processor_name: processor_name.into(),
            tenant_id,
            technical_measures,
            subprocessors,
            deletion_sla_days,
            contract_date: None,
            subprocessors_allowed: None,
        }
    }

    /// Sets an explicit contract date for the AVV context.
    pub fn with_contract_date(mut self, contract_date: impl Into<String>) -> Self {
        self.contract_date = Some(contract_date.into());
        self
    }

    /// Sets whether subprocessors are authorized/permitted.
    pub fn with_subprocessors_allowed(mut self, allowed: bool) -> Self {
        self.subprocessors_allowed = Some(allowed);
        self
    }

    /// Validates all fields in the [`AvvContext`] against legal and operational constraints.
    pub fn validate(&self) -> Result<(), AvvError> {
        if self.controller_name.trim().is_empty() {
            return Err(AvvError::EmptyControllerName);
        }
        if self.processor_name.trim().is_empty() {
            return Err(AvvError::EmptyProcessorName);
        }
        if let Some(ref date) = self.contract_date {
            if date.trim().is_empty() {
                return Err(AvvError::EmptyContractDate);
            }
        }
        if self.deletion_sla_days == 0 {
            return Err(AvvError::InvalidDeletionSla {
                days: 0,
                reason: "Löschfrist muss mindestens 1 Tag betragen.",
            });
        }
        if self.deletion_sla_days > 365 {
            return Err(AvvError::InvalidDeletionSla {
                days: self.deletion_sla_days,
                reason: "Löschfrist darf maximal 365 Tage betragen.",
            });
        }
        if self.subprocessors_allowed == Some(true) && self.subprocessors.is_empty() {
            return Err(AvvError::MissingSubprocessors);
        }
        for subp in &self.subprocessors {
            if subp.trim().is_empty() {
                return Err(AvvError::InvalidSubprocessor(
                    "Name des Unterauftragsverarbeiters darf nicht leer sein.".to_string(),
                ));
            }
        }
        for measure in &self.technical_measures {
            if measure.name.trim().is_empty() {
                return Err(AvvError::InvalidTechnicalMeasure(
                    "Name der technischen Maßnahme darf nicht leer sein.".to_string(),
                ));
            }
            if measure.description.trim().is_empty() {
                return Err(AvvError::InvalidTechnicalMeasure(
                    "Beschreibung der technischen Maßnahme darf nicht leer sein.".to_string(),
                ));
            }
            if measure.reference_article.trim().is_empty() {
                return Err(AvvError::InvalidTechnicalMeasure(
                    "Referenzartikel der technischen Maßnahme darf nicht leer sein.".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Description of a Technical and Organizational Measure (TOM) according to Art. 32 DSGVO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TechnicalMeasure {
    /// Name of measure (e.g. "Kryptografischer Löschbeweis (Ed25519)")
    pub name: String,
    /// Detailed technical description of measure
    pub description: String,
    /// Reference article (e.g. "Art. 32 Abs. 1 lit. b & Art. 17 DSGVO")
    pub reference_article: String,
}

/// Renders the AVV document in Markdown format using the provided [`AvvContext`].
pub fn render_avv_markdown(ctx: &AvvContext) -> Result<String, AvvError> {
    ctx.validate()?;
    template::render(ctx)
}

/// Renders the AVV document in Markdown format using the provided [`AvvContext`] and explicit date parameter.
pub fn render_avv_markdown_with_date(ctx: &AvvContext, date: &str) -> Result<String, AvvError> {
    ctx.validate()?;
    template::render_with_date(ctx, date)
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
