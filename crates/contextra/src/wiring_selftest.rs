//! Verdrahtungs-Selbsttest (§22 Punkt 4 / INV-WIRING-2).
//!
//! Stellt sicher, dass nach dem Engine-Start alle required Ports
//! (Inferenz / Embedder, Kms, Clock, Observer) gültige Implementierungen haben
//! und Null-/Accept-All-Implementierungen nicht als Sicherheits-Port-Standard dienen.

use contextra_core::error::ContextraError;

/// Liste der kritischen Sicherheits-Ports (vorerst als const laut Spec).
pub const REQUIRED_SECURITY_PORTS: &[&str] = &["Clock", "LicenseGate"];

/// Bericht über den Verdrahtungs-Selbsttest.
#[derive(Debug, Default)]
pub struct WiringReport {
    pub missing_ports: Vec<String>,
    pub invalid_security_ports: Vec<String>,
}

impl WiringReport {
    pub fn is_valid(&self) -> bool {
        self.missing_ports.is_empty() && self.invalid_security_ports.is_empty()
    }
}

/// Prüft die Verdrahtung einer Contextra-Instanz (§22 Punkt 4).
pub fn verify_wiring(
    has_clock: bool,
    has_license_gate: bool,
    license_gate_is_noop: bool,
) -> Result<WiringReport, ContextraError> {
    let mut report = WiringReport::default();

    if !has_clock {
        report.missing_ports.push("Clock".to_string());
    }
    if !has_license_gate {
        report.missing_ports.push("LicenseGate".to_string());
    }

    // Null-/Accept-All-Implementierungen dürfen NICHT als Sicherheits-Port-Standard dienen
    if license_gate_is_noop {
        report.invalid_security_ports.push("LicenseGate (Noop/OpenFastGate used in strict mode)".to_string());
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
        assert!(res.unwrap_err().to_string().contains("Clock"));
    }

    #[test]
    fn test_verify_wiring_noop_security_port_fails() {
        let res = verify_wiring(true, true, true);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("LicenseGate"));
    }
}
