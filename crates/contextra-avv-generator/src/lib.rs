#![forbid(unsafe_code)]

//! `contextra-avv-generator` — Generator für Auftragsverarbeitungsverträge (AVV) nach Art. 28 DSGVO.
//!
//! # Zweck & Architektur
//! Diese Crate erstellt standardkonforme, strukturierte AVV-Dokumentenvorlagen im Markdown-Format.
//! Sie verknüpft die formellen rechtlichen Mindestanforderungen nach Art. 28 Abs. 3 DSGVO
//! mit den real implementierten technischen Sicherheitsgarantien des Contextra-Kerns:
//! - **Kryptografischer Löschbeweis (Ed25519):** Mathematischer Nachweis der physischen Löschung (Art. 17 DSGVO).
//! - **Zero-Egress-Default / Egress-Gateway:** Strikt kontrollierter ausgehender Datenverkehr (Art. 32 DSGVO).
//! - **Kryptografische Mandantentrennung:** Sichere Isolation von Vektoren und Indizes pro [`TenantId`].
//! - **Verschlüsselter KV-Cache:** Schutz flüchtiger Kontexte im Arbeitsspeicher mit rotierenden Schlüsseln.
//!
//! # Einbindung & Abhängigkeiten
//! - **Ring-Einstufung:** Ring 4 (Erweiterung / `[E]`).
//! - **Abhängigkeiten (normal):**
//!   - `contextra-types`: Stellt fundamentale Domänentypen wie [`TenantId`] bereit.
//!   - `serde`: Serialisierung und Deserialisierung des Kontexts [`AvvContext`].
//!   - `thiserror`: Strukturierte Fehlerbehandlung über [`AvvError`].
//! - **Wird eingebunden von:**
//!   - `contextra-privacy`: Re-exportiert den Generator (historisch / Migrationspfad).
//!   - `contextra`: Haupt-Crate (optional via Feature-Flag `avv-generator`).
//!
//! # Sicherheit & Garantien
//! Gemäß `#![forbid(unsafe_code)]` enthält diese Crate keinerlei `unsafe`-Blöcke.
//! Alle generierten Vorlagen enthalten eingangs einen rechtlichen Hinweis (Disclaimer),
//! dass sie einen technischen Vertragsentwurf darstellen und keine Rechtsberatung ersetzen.

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

// ANCHOR[contextra-avv-generator:CTX-001]
/// Kontext- und Konfigurationsparameter zur Generierung einer AVV-Vorlage gemäß Art. 28 DSGVO.
///
/// Enthält alle variablen Vertragsbestandteile, wie Namen der Vertragsparteien,
/// die Mandanten-ID ([`TenantId`]), die technischen Sicherheitsmaßnahmen (TOM),
/// autorisierte Unterauftragsverarbeiter und Fristen zur Datenlöschung.
///
/// # Beispiel
/// ```rust
/// use contextra_avv_generator::{AvvContext, default_technical_measures, render_avv_markdown};
/// use contextra_types::TenantId;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let ctx = AvvContext::new(
///     "Musterkanzlei AG",
///     "Contextra Hosting GmbH",
///     TenantId(100),
///     default_technical_measures(),
///     vec!["EU-Cloud-Region-Frankfurt".to_string()],
///     14,
/// );
///
/// let markdown = render_avv_markdown(&ctx)?;
/// assert!(markdown.contains("Musterkanzlei AG"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AvvContext {
    /// Name oder Bezeichnung des Verantwortlichen (Auftraggebers, z. B. Kanzlei, Praxis, Unternehmen).
    pub controller_name: String,
    /// Name oder Bezeichnung des Auftragsverarbeiters (Auftragnehmers, Betreiber des Contextra-Systems).
    pub processor_name: String,
    /// Eindeutige Mandantenkennung ([`TenantId`]) zur Personalisierung und Trennung im Speicher.
    pub tenant_id: TenantId,
    /// Liste der technischen und organisatorischen Maßnahmen (TOM) gemäß Art. 32 DSGVO.
    pub technical_measures: Vec<TechnicalMeasure>,
    /// Liste der genehmigten Unterauftragsverarbeiter (Subunternehmer).
    pub subprocessors: Vec<String>,
    /// Vereinbarte Frist (SLA = Service Level Agreement) für die Löschung in Tagen nach Vertragsende (1..=365).
    pub deletion_sla_days: u32,
    /// Optionales explizites Vertragsdatum (z. B. "2026-10-29"). Bei `None` wird ein Standardtext verwendet.
    #[serde(default)]
    pub contract_date: Option<String>,
    /// Kennzeichnung, ob Unterauftragsverarbeiter grundsätzlich gestattet sind.
    ///
    /// NOTE: Wenn `Some(true)` gesetzt ist, aber `subprocessors` leer ist,
    /// schlägt [`validate`](Self::validate) mit [`AvvError::MissingSubprocessors`] fehl.
    #[serde(default)]
    pub subprocessors_allowed: Option<bool>,
}

impl Default for AvvContext {
    /// Erstellt eine standardmäßige Vorbelegung des Kontexts.
    ///
    /// Setzt leere Namen, `TenantId(0)`, die Standard-TOMs ([`default_technical_measures`]),
    /// eine leere Subprozessor-Liste und eine 14-tägige Löschfrist.
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
    // ANCHOR[contextra-avv-generator:CTX-002]
    /// Erzeugt einen neuen [`AvvContext`] mit den Pflichtparametern und Standard-Optionen.
    ///
    /// # Parameter
    /// - `controller_name`: Name des Auftraggebers.
    /// - `processor_name`: Name des Auftragnehmers.
    /// - `tenant_id`: Mandanten-ID.
    /// - `technical_measures`: Liste technischer und organisatorischer Maßnahmen.
    /// - `subprocessors`: Liste der Unterauftragsverarbeiter.
    /// - `deletion_sla_days`: Löschfrist in Tagen (zulässig: 1..=365).
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

    // ANCHOR[contextra-avv-generator:CTX-003]
    /// Setzt ein explizites Vertragsdatum für die AVV-Vorlage (Builder-Muster).
    pub fn with_contract_date(mut self, contract_date: impl Into<String>) -> Self {
        self.contract_date = Some(contract_date.into());
        self
    }

    // ANCHOR[contextra-avv-generator:CTX-004]
    /// Konfiguriert die grundsätzliche Zulässigkeit von Unterauftragsverarbeitern (Builder-Muster).
    pub fn with_subprocessors_allowed(mut self, allowed: bool) -> Self {
        self.subprocessors_allowed = Some(allowed);
        self
    }

    // ANCHOR[contextra-avv-generator:CTX-005]
    /// Validiert alle Kontextfelder gegen rechtliche und logische Mindestanforderungen.
    ///
    /// # Prüfschritte
    /// 1. `controller_name`: Darf nach Trimmen nicht leer sein ([`AvvError::EmptyControllerName`]).
    /// 2. `processor_name`: Darf nach Trimmen nicht leer sein ([`AvvError::EmptyProcessorName`]).
    /// 3. `contract_date`: Falls vorhanden, darf der String nach Trimmen nicht leer sein ([`AvvError::EmptyContractDate`]).
    /// 4. `deletion_sla_days`: Muss zwischen 1 und 365 Tagen liegen ([`AvvError::InvalidDeletionSla`]).
    /// 5. `subprocessors_allowed`: Ist `Some(true)` gesetzt, darf die Liste `subprocessors` nicht leer sein ([`AvvError::MissingSubprocessors`]).
    /// 6. `subprocessors`: Kein Eintrag darf nach Trimmen leer sein ([`AvvError::InvalidSubprocessor`]).
    /// 7. `technical_measures`: Name, Beschreibung und Referenzartikel jeder Maßnahme müssen befüllt sein ([`AvvError::InvalidTechnicalMeasure`]).
    ///
    /// # Fehler
    /// Gibt den jeweils ersten festgestellten Validierungsfehler als [`AvvError`] zurück.
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

// ANCHOR[contextra-avv-generator:TOM-001]
/// Beschreibung einer technischen und organisatorischen Maßnahme (TOM) gemäß Art. 32 DSGVO.
///
/// TOMs beschreiben die Sicherheitsmechanismen, die zum Schutz der Vertraulichkeit,
/// Integrität, Verfügbarkeit und Belastbarkeit der Systeme implementiert wurden.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TechnicalMeasure {
    /// Bezeichnung der Sicherheitsmaßnahme (z. B. "Kryptografischer Löschbeweis (Ed25519)").
    pub name: String,
    /// Detaillierte technische Funktionsbeschreibung der Maßnahme.
    pub description: String,
    /// Referenzierter Artikel der DSGVO (z. B. "Art. 32 Abs. 1 lit. b & Art. 17 DSGVO").
    pub reference_article: String,
}

// ANCHOR[contextra-avv-generator:PUB-001]
/// Rendert das vollständige AVV-Dokument im Markdown-Format anhand des übergebenen [`AvvContext`].
///
/// # Ablauf
/// 1. Validiert den Kontext über [`AvvContext::validate`].
/// 2. Delegiert die Dokumenterstellung an [`template::render`].
///
/// # Fehler
/// - Gibt [`AvvError`] zurück, wenn Validierungsregeln verletzt sind (z. B. leere Namen).
/// - Gibt [`AvvError::RenderError`] zurück, falls beim Text-Rendering ein Schreibfehler auftritt.
pub fn render_avv_markdown(ctx: &AvvContext) -> Result<String, AvvError> {
    ctx.validate()?;
    template::render(ctx)
}

// ANCHOR[contextra-avv-generator:PUB-002]
/// Rendert das AVV-Dokument im Markdown-Format mit einem explizit vorgegebenen Vertragsdatum.
///
/// # Ablauf
/// 1. Validiert den Kontext über [`AvvContext::validate`].
/// 2. Ruft [`template::render_with_date`] mit dem übergebenen `date`-String auf.
///
/// # Fehler
/// Liefert einen Fehler, wenn die Kontextvalidierung fehlschlägt oder das Rendering abbricht.
pub fn render_avv_markdown_with_date(ctx: &AvvContext, date: &str) -> Result<String, AvvError> {
    ctx.validate()?;
    template::render_with_date(ctx, date)
}

// ANCHOR[contextra-avv-generator:PUB-003]
/// Liefert die standardmäßig implementierten technischen Schutzmaßnahmen von Contextra zurück.
///
/// Diese Liste spiegelt die Kern-Sicherheitsfeatures des Contextra-Systems wider:
/// 1. **Kryptografischer Löschbeweis:** Ed25519-signierter Beleg über unwiderrufliche Datenlöschung.
/// 2. **Zero-Egress-Default:** Netzwerkseitige Blockade unautorisierter Abflüsse via Egress-Gateway.
/// 3. **Kryptografische Mandantentrennung:** Speicherebenen-Isolation pro Mandanten-ID ([`TenantId`]).
/// 4. **Verschlüsselte Segmente im KV-Cache:** Key-Value-Cache-Verschlüsselung im RAM gegen Speicherdumps.
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
