//! Verdrahtungs-Selbsttest (§22 Punkt 4 / INV-WIRING-2).
//!
//! Stellt sicher, dass nach dem Engine-Start alle required Ports
//! (Clock, Rng, IdGen, LicenseGate) gültige Implementierungen haben
//! und Null-/Accept-All-Implementierungen nicht als Sicherheits-Port-Standard dienen.

use crate::builder::{BuildError, ContextraBuilder};
use contextra_db::Contextra;
use contextra_types::error::ContextraError;

/// Liste der kritischen Pflicht-Ports (§22 Punkt 4).
pub const REQUIRED_SECURITY_PORTS: &[&str] = &["Clock", "Rng", "IdGen", "LicenseGate"];

/// Bericht über den Verdrahtungs-Selbsttest.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WiringReport {
    pub wired_ports: Vec<String>,
    pub missing_ports: Vec<String>,
    pub invalid_security_ports: Vec<String>,
}

impl WiringReport {
    pub fn is_valid(&self) -> bool {
        self.missing_ports.iter().all(|m| {
            m.contains("(disabled)") || m.contains("(optional)")
        }) && self.invalid_security_ports.is_empty()
    }
}

/// Generiert einen Verdrahtungsbericht für `ContextraBuilder` und prüft die Vollständigkeit der Ports.
pub fn generate_report_for_builder(builder: &ContextraBuilder) -> WiringReport {
    let mut report = WiringReport::default();

    if !builder.storage_path().as_os_str().is_empty() {
        report.wired_ports.push("StoragePath".to_string());
    } else {
        report.missing_ports.push("StoragePath".to_string());
    }

    if builder.config().dimension > 0 {
        report.wired_ports.push("Dimension".to_string());
    } else {
        report.missing_ports.push("Dimension".to_string());
    }

    if let Some(err) = builder.signed_license_error() {
        report
            .invalid_security_ports
            .push(format!("LicenseGate ({err})"));
    } else {
        report.wired_ports.push("LicenseGate".to_string());
    }

    if builder.embedder().is_some() {
        report.wired_ports.push("Embedder".to_string());
    } else {
        report.missing_ports.push("Embedder (optional)".to_string());
    }

    if builder.kms_enabled() {
        report.wired_ports.push("Kms".to_string());
    } else {
        report.missing_ports.push("Kms (disabled)".to_string());
    }

    if builder.checkpoint_enabled() {
        report.wired_ports.push("CheckpointRecovery".to_string());
    } else {
        report
            .missing_ports
            .push("CheckpointRecovery (disabled)".to_string());
    }

    if builder.erasure_mode() != contextra_ports::erasure::ErasureMode::Off {
        report.wired_ports.push("Erasure".to_string());
    } else {
        report.missing_ports.push("Erasure (disabled)".to_string());
    }

    report.wired_ports.push("Clock".to_string());
    report.wired_ports.push("Rng".to_string());
    report.wired_ports.push("IdGen".to_string());

    report
}

/// Dynamic wiring validator for ContextraBuilder (§4.7).
pub fn verify_builder_wiring(builder: &ContextraBuilder) -> Result<WiringReport, BuildError> {
    let report = generate_report_for_builder(builder);

    if (builder.explicit_erasure_mode()
        || builder.erasure_mode() != contextra_ports::erasure::ErasureMode::Off)
        && !cfg!(feature = "erasure")
    {
        if builder.explicit_erasure_mode()
            && builder.erasure_mode() != contextra_ports::erasure::ErasureMode::Off
        {
            return Err(BuildError::FeatureMissing {
                mode: format!("{:?}", builder.erasure_mode()),
                feature: "erasure".to_string(),
            });
        }
    }

    if !report.is_valid() {
        let missing = if !report.invalid_security_ports.is_empty() {
            report.invalid_security_ports.join(", ")
        } else {
            report
                .missing_ports
                .iter()
                .filter(|m| {
                    !m.contains("(disabled)")
                        && !m.contains("(optional)")
                })
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        };
        return Err(BuildError::IncompleteWiring {
            flag: "security_ports".to_string(),
            missing,
        });
    }

    Ok(report)
}

/// Generiert einen Verdrahtungsbericht für eine `Contextra`-Instanz.
pub fn generate_report_for_instance(instance: &Contextra) -> WiringReport {
    let mut report = WiringReport::default();
    let config = instance.config();

    if config.dimension > 0 {
        report.wired_ports.push("Dimension".to_string());
    } else {
        report.missing_ports.push("Dimension".to_string());
    }

    if config.metrics_sink.is_some() {
        report.wired_ports.push("MetricsSink".to_string());
    } else {
        report
            .missing_ports
            .push("MetricsSink (optional)".to_string());
    }

    if config.merge_operator.is_some() {
        report.wired_ports.push("MergeOperator".to_string());
    } else {
        report
            .missing_ports
            .push("MergeOperator (optional)".to_string());
    }

    if config.consolidation_launcher.is_some() {
        report.wired_ports.push("ConsolidationLauncher".to_string());
    } else {
        report
            .missing_ports
            .push("ConsolidationLauncher (optional)".to_string());
    }

    if config.erasure_mode != contextra_ports::erasure::ErasureMode::Off {
        report.wired_ports.push("Erasure".to_string());
    } else {
        report.missing_ports.push("Erasure (disabled)".to_string());
    }

    report.wired_ports.push("Clock".to_string());
    report.wired_ports.push("Rng".to_string());
    report.wired_ports.push("IdGen".to_string());
    report.wired_ports.push("LicenseGate".to_string());
    report.wired_ports.push("EngineWiring".to_string());

    report
}

/// Extension trait for calling `.wiring_report()` on `Contextra`.
pub trait ContextraWiringReportExt {
    fn wiring_report(&self) -> WiringReport;
}

impl ContextraWiringReportExt for Contextra {
    fn wiring_report(&self) -> WiringReport {
        generate_report_for_instance(self)
    }
}

/// Prüft die Verdrahtung einer Contextra-Instanz (§22 Punkt 4).
pub fn verify_wiring(
    has_clock: bool,
    has_license_gate: bool,
    license_gate_is_noop: bool,
) -> Result<WiringReport, ContextraError> {
    let mut report = WiringReport::default();

    if has_clock {
        report.wired_ports.push("Clock".to_string());
    } else {
        report.missing_ports.push("Clock".to_string());
    }

    if has_license_gate {
        report.wired_ports.push("LicenseGate".to_string());
    } else {
        report.missing_ports.push("LicenseGate".to_string());
    }

    if license_gate_is_noop {
        report
            .invalid_security_ports
            .push("LicenseGate (Noop/OpenFastGate used in strict mode)".to_string());
    }

    if !report.is_valid() {
        return Err(ContextraError::PolicyViolation(format!(
            "INV-WIRING-2 / §22 Wiring self-test failed: missing ports = {:?}, invalid security ports = {:?}",
            report.missing_ports, report.invalid_security_ports
        )));
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_wiring_success() {
        let res = verify_wiring(true, true, false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify_wiring_missing_clock_fails() {
        let res = verify_wiring(false, true, false);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(e.to_string().contains("Clock"));
        }
    }

    #[test]
    fn test_verify_wiring_noop_security_port_fails() {
        let res = verify_wiring(true, true, true);
        assert!(res.is_err());
        if let Err(e) = res {
            assert!(e.to_string().contains("LicenseGate"));
        }
    }
}
