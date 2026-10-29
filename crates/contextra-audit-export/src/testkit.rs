//! Test kit utilities for `contextra-audit-export`.

use crate::{
    AuditExportError, ControllerContact, DeletionProofSummary, EgressEventSummary,
    ProcessingRegisterEntry, ProcessingRegisterSource, RetentionPeriod, ThirdCountryTransfer,
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

    /// Creates a default [`ControllerContact`] instance for sample data.
    pub fn sample_controller_contact() -> ControllerContact {
        ControllerContact {
            controller_name: "Musterfirma GmbH".to_string(),
            controller_contact:
                "Datenschutzbeauftragter, Hauptstraße 1, 10115 Berlin, dsb@musterfirma.de"
                    .to_string(),
            representative_contact: Some("Max Mustermann, Geschäftsführung".to_string()),
            dpo_contact: Some("Dr. Anna Schmidt, dsb@musterfirma.de".to_string()),
        }
    }

    /// Creates an [`InMemoryProcessingRegisterSource`] pre-populated with synthetic sample data for `tenant_id`.
    pub fn with_sample_data_for(tenant_id: TenantId) -> Self {
        let mut entry1 = ProcessingRegisterEntry {
            tenant_id,
            controller_contact: Self::sample_controller_contact(),
            processing_purpose: "Kontextsensitive Vektorsuche".to_string(),
            data_subject_categories: vec!["Kunden".to_string(), "Mitarbeiter".to_string()],
            data_categories: vec!["Protokolldaten".to_string(), "Search Queries".to_string()],
            recipient_categories: vec!["Auftragsverarbeiter".to_string(), "Interne IT".to_string()],
            third_country_transfers: vec![ThirdCountryTransfer {
                destination_country: "Keine (Verarbeitung im EWR)".to_string(),
                safeguard_guarantee: "Nicht zutreffend (EWR-Hosting)".to_string(),
            }],
            retention_periods: vec![RetentionPeriod {
                data_category: "Search Queries & Protokolle".to_string(),
                retention_period: "30 Tage nach Abwicklung, danach automatische Löschung"
                    .to_string(),
            }],
            technical_organizational_measures: vec![
                "Kryptographischer Löschnachweis (Ed25519 Signatur)".to_string(),
                "Mandantentrennung & Zugriffskontrolle".to_string(),
                "Verschlüsselung at rest (AES-256-GCM-SIV)".to_string(),
            ],
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
        entry1.sort_internal_collections();

        let mut entry2 = ProcessingRegisterEntry {
            tenant_id,
            controller_contact: Self::sample_controller_contact(),
            processing_purpose: "Dokumenten-Graph-Relationierung".to_string(),
            data_subject_categories: vec!["Kunden".to_string()],
            data_categories: vec!["Entitäten".to_string(), "Metadaten".to_string()],
            recipient_categories: vec!["Interne Fachabteilung".to_string()],
            third_country_transfers: vec![],
            retention_periods: vec![RetentionPeriod {
                data_category: "Entitäten & Metadaten".to_string(),
                retention_period: "Löschung bei Mandantenkündigung".to_string(),
            }],
            technical_organizational_measures: vec![
                "Integritätssicherung via SHA-256 / HMAC".to_string(),
                "Rollenbasierte Zugriffskontrolle (RBAC)".to_string(),
            ],
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
        entry2.sort_internal_collections();

        // Placed in canonical sorted order by purpose ("Dokumenten-Graph..." < "Kontextsensitive...")
        Self {
            entries: vec![entry2, entry1],
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
