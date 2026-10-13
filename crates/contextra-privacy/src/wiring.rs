//! `wiring` — Standard-Verdrahtungs-Einstiegspunkt für `contextra-privacy`.
//!
//! Stellt den zentralen Typ [`PrivacyWiring`] sowie die Konfiguration [`PrivacyWiringConfig`] bereit,
//! welche die Privacy-, DLP-, Egress- und Audit-Komponenten bündeln:
//! `EgressGateway` (`DefaultEgressClassifier`), `EgressGuard`, `EgressVault`, `BulkExfiltrationDetector`,
//! `ProcessingRegistry` (`ProcessingActivityRecord` / `ProcessingRegistryExport`) und `AuditTrace` (`compute_audit_trace`).
//! Dies dient als kanonischer Anbindungspunkt für Fassaden (Ring 4 / `contextra::builder`) und MCP (`contextra-mcp`).

use crate::audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace};
use crate::bulk_exfiltration_detector::BulkExfiltrationDetector;
use crate::egress_gateway::DefaultEgressClassifier;
use crate::egress_guard::EgressGuard;
use crate::egress_vault::EgressVault;
use crate::processing_registry::{
    generate_registry, render_markdown, ProcessingActivityRecord, ProcessingRegistryExport,
};
use contextra_ports::Clock;
use contextra_types::ContextraError;
use std::sync::Arc;

/// Konfigurationsstruktur für den Aufbau der Privacy-Verdrahtung.
#[derive(Clone)]
pub struct PrivacyWiringConfig {
    /// EgressVault für Layer-1 DLP-Regelprüfungen.
    pub egress_vault: EgressVault,
    /// EgressGuard für Layer-4 k-NN Bulk-Exfiltrationsprüfungen (optional).
    pub egress_guard: Option<EgressGuard>,
    /// BulkExfiltrationDetector für sitzungsbezogenes Rate-Limiting.
    pub bulk_detector: Arc<BulkExfiltrationDetector>,
    /// Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO).
    pub processing_records: Vec<ProcessingActivityRecord>,
}

impl Default for PrivacyWiringConfig {
    fn default() -> Self {
        let vault = EgressVault::try_default()
            .unwrap_or_else(|_| EgressVault::new(vec![]).expect("empty vault always succeeds"));
        Self {
            egress_vault: vault,
            egress_guard: None,
            bulk_detector: Arc::new(BulkExfiltrationDetector::new(
                100 * 1024 * 1024,
                std::time::Duration::from_secs(60),
            )),
            processing_records: Vec::new(),
        }
    }
}

/// Kanonischer Bündelungs-Typ für alle Privacy- & Egress-Komponenten.
///
/// Wird von der Fassade (`contextra::builder::ContextraBuilder`) und dem MCP-Server
/// (`contextra-mcp`) als Composition Root für Egress-Gateway, DLP-Vault, Guard, Bulk-Detector,
/// Verarbeitungsverzeichnis (Art. 30) und Audit Trace verwendet.
#[derive(Clone)]
pub struct PrivacyWiring {
    config: PrivacyWiringConfig,
    gateway: DefaultEgressClassifier,
}

impl PrivacyWiring {
    /// Erzeugt eine neue `PrivacyWiring`-Instanz aus der angegebenen Konfiguration.
    ///
    /// # Errors
    /// Gibt `ContextraError::InvalidInput` zurück, wenn erforderliche Konfigurationsparameter ungültig sind.
    pub fn build(config: PrivacyWiringConfig) -> Result<Self, ContextraError> {
        let gateway = config.egress_vault.clone();
        Ok(Self { config, gateway })
    }

    /// Gibt eine Referenz auf den konfigurierten `EgressVault` zurück.
    pub fn egress_vault(&self) -> &EgressVault {
        &self.config.egress_vault
    }

    /// Gibt eine optionale Referenz auf den `EgressGuard` zurück.
    pub fn egress_guard(&self) -> Option<&EgressGuard> {
        self.config.egress_guard.as_ref()
    }

    /// Gibt eine Referenz auf den `BulkExfiltrationDetector` zurück.
    pub fn bulk_detector(&self) -> &BulkExfiltrationDetector {
        &self.config.bulk_detector
    }

    /// Gibt eine Referenz auf das zusammengestellte `DefaultEgressClassifier` Gateway zurück.
    pub fn gateway(&self) -> &DefaultEgressClassifier {
        &self.gateway
    }

    /// Generiert den zusammenfassenden `ProcessingRegistryExport` für Art. 30 DSGVO Verzeichnisse.
    pub fn generate_processing_registry(&self) -> ProcessingRegistryExport {
        generate_registry(&self.config.processing_records)
    }

    /// Rendert das Verzeichnis von Verarbeitungstätigkeiten als Markdown-Tabelle.
    pub fn render_processing_registry_markdown(&self) -> String {
        let export = self.generate_processing_registry();
        render_markdown(&export)
    }

    /// Erzeugt einen deterministischen 32-Byte BLAKE3 Hash-Trace für eine Egress-Klassifikationsentscheidung (INV-EGRESS-AUDIT-1).
    pub async fn compute_classification_trace(
        &self,
        payload: &str,
        clock: &dyn Clock,
    ) -> ([u8; 32], String) {
        let (classification, trace) = self.gateway.classify_with_trace(payload, clock).await;
        let rule_id = extract_rule_id(&classification);
        let manual_trace = compute_audit_trace(payload, &rule_id, clock.now_unix_nanos());
        debug_assert_eq!(trace, manual_trace);
        (trace, rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTestClock(u64);
    impl Clock for MockTestClock {
        fn now_unix_nanos(&self) -> u64 {
            self.0
        }
        fn monotonic_nanos(&self) -> u64 {
            self.0
        }
    }

    #[tokio::test]
    async fn test_privacy_wiring_build_and_components() {
        let config = PrivacyWiringConfig::default();
        let wiring = PrivacyWiring::build(config).expect("build default wiring should succeed");

        assert!(wiring.egress_guard().is_none());
        assert_ne!(format!("{:?}", wiring.egress_vault()), "");

        let registry = wiring.generate_processing_registry();
        assert!(registry.records.is_empty());

        let clock = MockTestClock(1_000_000);
        let (trace, rule_id) = wiring
            .compute_classification_trace("Safe text payload", &clock)
            .await;
        assert_ne!(trace, [0u8; 32]);
        assert_eq!(rule_id, "ALLOW");
    }
}
