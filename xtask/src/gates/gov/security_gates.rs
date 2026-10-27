// Contextra — Security Gates Implementation
// Gate 1: no-security-stub (A1, INV-WIRING-2)
// Gate 2: no-literal-secrets (INV-TENANT-1, VETO-F10)
// Gate 3: security-ports (INV-WIRING-2, INV-DELETION-PORTS-1, INV-KMS-1)
// Gate 4: key-purposes (INV-KEYPURPOSE-1)

use crate::core::ctx::Context;
use crate::core::gate::{Gate, GateSpec, Outcome};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// ── Gate 1: NoSecurityStubGate ───────────────────────────────────────────────

pub struct NoSecurityStubGate;

impl Gate for NoSecurityStubGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "no-security-stub".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH1".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["A1".to_string(), "INV-WIRING-2".to_string()],
            why: "Keine Sicherheitsstubs in Produktion: kein cfg(not(feature))-Ersatz für Sicherheitscode (SPEC §17.3, A1).".to_string(),
            owner: "MP-04".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_no_security_stub(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "no-security-stub" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_no_security_stub(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Outcome::Pass;
    }

    for entry in WalkDir::new(&crates_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        if rel_path.contains("/tests/")
            || rel_path.contains("/benches/")
            || rel_path.contains("/examples/")
        {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("cfg(not(feature = \"encryption-at-rest\"))")
                || trimmed.contains("no_crypto_stubs")
            {
                violations.push(format!(
                    "{}:{} - Security stub / feature negation found: '{}'",
                    rel_path, line_num, trimmed
                ));
            }

            if trimmed.contains("pub fn verify")
                && (trimmed.contains("Ok(true)") || trimmed.contains("return Ok(true)"))
            {
                violations.push(format!(
                    "{}:{} - Unconditional Ok(true) verification stub found",
                    rel_path, line_num
                ));
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "no-security-stub check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 2: NoLiteralSecretsGate ─────────────────────────────────────────────

pub struct NoLiteralSecretsGate;

impl Gate for NoLiteralSecretsGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "no-literal-secrets".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH1".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["INV-TENANT-1".to_string(), "VETO-F10".to_string()],
            why: "Keine Literal-Schlüssel, Mandanten-Fallbacks oder Hardcoded-Secrets (SPEC §17.3).".to_string(),
            owner: "MP-04".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_no_literal_secrets(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "no-literal-secrets" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_no_literal_secrets(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Outcome::Pass;
    }

    for entry in WalkDir::new(&crates_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        if rel_path.contains("/tests/")
            || rel_path.contains("/benches/")
            || rel_path.contains("/examples/")
        {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("default_test_deletion_proof_key") {
                violations.push(format!(
                    "{}:{} - Hardcoded literal proof key found: '{}'",
                    rel_path, line_num, trimmed
                ));
            }

            if trimmed.contains("unwrap_or(TenantId::SYSTEM)")
                || trimmed.contains("unwrap_or_else(|| TenantId::SYSTEM)")
            {
                violations.push(format!(
                    "{}:{} - Mandant fallback to TenantId::SYSTEM found: '{}'",
                    rel_path, line_num, trimmed
                ));
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "no-literal-secrets check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 3: SecurityPortsGate ────────────────────────────────────────────────

pub struct SecurityPortsGate;

#[derive(Debug, Deserialize)]
struct SecurityPortEntry {
    name: String,
    #[serde(rename = "crate", default)]
    krate: String,
    #[serde(default)]
    trait_path: String,
    #[serde(default)]
    class: String,
    #[serde(default)]
    prod_impl: String,
    #[serde(default)]
    wiring_test: String,
}

#[derive(Debug, Deserialize)]
struct SecurityPortsToml {
    #[serde(default, rename = "port")]
    ports: Vec<SecurityPortEntry>,
}

impl Gate for SecurityPortsGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "security-ports".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH1".to_string(),
            kind: "builtin".to_string(),
            enforces: vec![
                "INV-WIRING-2".to_string(),
                "INV-DELETION-PORTS-1".to_string(),
                "INV-KMS-1".to_string(),
            ],
            why: "Erzwingt Sicherheitskontrollen an Port-Schnittstellen gegen governance/security_ports.toml.".to_string(),
            owner: "MP-04".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_security_ports(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "security-ports" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_security_ports(root: &Path) -> Outcome {
    let config_path = root.join("governance/security_ports.toml");
    if !config_path.exists() {
        return Outcome::Error {
            message: "governance/security_ports.toml does not exist".to_string(),
        };
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            return Outcome::Error {
                message: format!("Failed to read governance/security_ports.toml: {}", e),
            }
        }
    };

    let parsed: SecurityPortsToml = match toml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            return Outcome::Error {
                message: format!("Failed to parse governance/security_ports.toml: {}", e),
            }
        }
    };

    let mut violations = Vec::new();

    for port in &parsed.ports {
        if port.prod_impl.is_empty() {
            violations.push(format!(
                "Port '{}' ({}) has empty prod_impl",
                port.name, port.krate
            ));
            continue;
        }

        // Validate implementation against governance/security_ports.toml specification
        let impl_str = &port.prod_impl;

        // If prod_impl references a concrete implementation path e.g. "contextra_crypto::kms::K1Provider"
        if impl_str.contains("::") {
            let parts: Vec<&str> = impl_str.split("::").collect();
            if let (Some(target_crate), Some(target_symbol)) = (parts.first(), parts.last()) {
                let clean_crate = target_crate.replace('_', "-");
                let crate_dir = root.join("crates").join(&clean_crate);

                // Verify that target crate directory exists
                if !crate_dir.exists() {
                    violations.push(format!(
                        "Port '{}' specifies prod_impl '{}', but target crate '{}' does not exist",
                        port.name, impl_str, clean_crate
                    ));
                } else {
                    // Scan target crate for struct / impl symbol declaration
                    let mut found_symbol = false;
                    let src_dir = crate_dir.join("src");
                    if src_dir.exists() {
                        for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
                            if entry.file_type().is_file()
                                && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
                            {
                                if let Ok(code) = fs::read_to_string(entry.path()) {
                                    if code.contains(target_symbol) {
                                        found_symbol = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !found_symbol {
                        violations.push(format!(
                            "Port '{}' specifies prod_impl '{}', but symbol '{}' was not found in crate '{}'",
                            port.name, impl_str, target_symbol, clean_crate
                        ));
                    }
                }
            }
        } else if impl_str.contains("CapabilityUnsupported") || impl_str.contains("per-layer") {
            // Documented valid non-concrete implementations per governance/security_ports.toml
        } else {
            violations.push(format!(
                "Port '{}' specifies invalid or unrecognized prod_impl '{}'",
                port.name, impl_str
            ));
        }

        // Verify wiring test existence on disk if specified and not TBD
        if !port.wiring_test.is_empty() && !port.wiring_test.starts_with("TBD") {
            let test_path = root.join(&port.wiring_test);
            let alt_test_path = root.join("crates/contextra-ports").join(&port.wiring_test);
            if !test_path.exists() && !alt_test_path.exists() {
                violations.push(format!(
                    "Port '{}' references wiring_test '{}' which does not exist on disk",
                    port.name, port.wiring_test
                ));
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "security-ports check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 4: KeyPurposesGate ──────────────────────────────────────────────────

pub struct KeyPurposesGate;

#[derive(Debug, Deserialize)]
struct KeyPurposeEntry {
    label: Option<String>,
    purpose: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KeyPurposesToml {
    #[serde(default, rename = "purpose")]
    purposes: Vec<KeyPurposeEntry>,
    #[serde(default, rename = "key")]
    keys: Vec<KeyPurposeEntry>,
}

impl Gate for KeyPurposesGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "key-purposes".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["INV-KEYPURPOSE-1".to_string()],
            why: "Überprüft die korrekte Zweckbindung von Kryptoschlüsseln gegen governance/key_purposes.toml.".to_string(),
            owner: "MP-04".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_key_purposes(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "key-purposes" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_key_purposes(root: &Path) -> Outcome {
    let config_path = root.join("governance/key_purposes.toml");
    if !config_path.exists() {
        return Outcome::Error {
            message: "governance/key_purposes.toml does not exist".to_string(),
        };
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            return Outcome::Error {
                message: format!("Failed to read governance/key_purposes.toml: {}", e),
            }
        }
    };

    let parsed: KeyPurposesToml = match toml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            return Outcome::Error {
                message: format!("Failed to parse governance/key_purposes.toml: {}", e),
            }
        }
    };

    let mut registered_labels = HashSet::new();
    for entry in parsed.purposes.iter().chain(parsed.keys.iter()) {
        if let Some(ref l) = entry.label {
            registered_labels.insert(l.clone());
        }
        if let Some(ref p) = entry.purpose {
            registered_labels.insert(p.clone());
        }
    }

    let mut violations = Vec::new();
    let label_regex = match regex::Regex::new(r#"contextra/v1/[a-zA-Z0-9_/-]+"#) {
        Ok(r) => r,
        Err(e) => {
            return Outcome::Error {
                message: format!("Regex compilation failed: {}", e),
            }
        }
    };

    let crates_dir = root.join("crates");
    if crates_dir.exists() {
        for entry in WalkDir::new(&crates_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };

            if rel_path.contains("/tests/")
                || rel_path.contains("/benches/")
                || rel_path.contains("/examples/")
            {
                continue;
            }

            let code = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in code.lines().enumerate() {
                let line_num = idx + 1;
                for cap in label_regex.captures_iter(&line) {
                    if let Some(m) = cap.get(0) {
                        let label = m.as_str();
                        if !registered_labels.contains(label) {
                            violations.push(format!(
                                "{}:{} - Unregistered HKDF key purpose label '{}'",
                                rel_path, line_num, label
                            ));
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "key-purposes check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_no_security_stub_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
        fs::write(root.join("crates/test-crate/src/lib.rs"), "pub fn ok() -> bool { true }").unwrap();
        assert_eq!(scan_no_security_stub(root), Outcome::Pass);

        fs::write(
            root.join("crates/test-crate/src/stub.rs"),
            "#[cfg(not(feature = \"encryption-at-rest\"))]\npub fn stub() {}",
        )
        .unwrap();
        match scan_no_security_stub(root) {
            Outcome::Fail { reason } => assert!(reason.contains("Security stub")),
            _ => panic!("Expected Fail for stub pattern"),
        }
    }

    #[test]
    fn test_no_literal_secrets_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
        fs::write(root.join("crates/test-crate/src/lib.rs"), "pub fn ok() {}").unwrap();
        assert_eq!(scan_no_literal_secrets(root), Outcome::Pass);

        fs::write(
            root.join("crates/test-crate/src/secret.rs"),
            "let k = \"default_test_deletion_proof_key\";",
        )
        .unwrap();
        match scan_no_literal_secrets(root) {
            Outcome::Fail { reason } => assert!(reason.contains("Hardcoded literal proof key")),
            _ => panic!("Expected Fail for literal secret"),
        }
    }

    #[test]
    fn test_security_ports_gate() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let toml_content = r#"
            [[port]]
            name = "KmsProvider"
            crate = "contextra-ports"
            prod_impl = "contextra_crypto::kms::K1Provider"
            wiring_test = "ports/tests/kms_wiring.rs"
        "#;
        fs::create_dir_all(root.join("governance")).unwrap();
        fs::create_dir_all(root.join("crates/contextra-crypto/src/kms")).unwrap();
        fs::create_dir_all(root.join("ports/tests")).unwrap();

        fs::write(root.join("governance/security_ports.toml"), toml_content).unwrap();
        fs::write(root.join("crates/contextra-crypto/src/kms/k1.rs"), "pub struct K1Provider;").unwrap();
        fs::write(root.join("ports/tests/kms_wiring.rs"), "// test").unwrap();

        assert_eq!(scan_security_ports(root), Outcome::Pass);
    }

    #[test]
    fn test_key_purposes_gate() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        let toml_content = r#"
            [[purpose]]
            label = "contextra/v1/wal"
            purpose = "wal"
        "#;
        fs::create_dir_all(root.join("governance")).unwrap();
        fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
        fs::write(root.join("governance/key_purposes.toml"), toml_content).unwrap();

        fs::write(
            root.join("crates/test-crate/src/lib.rs"),
            "let label = \"contextra/v1/wal\";",
        )
        .unwrap();

        assert_eq!(scan_key_purposes(root), Outcome::Pass);

        fs::write(
            root.join("crates/test-crate/src/bad.rs"),
            "let label = \"contextra/v1/unregistered_abc\";",
        )
        .unwrap();

        match scan_key_purposes(root) {
            Outcome::Fail { reason } => assert!(reason.contains("Unregistered HKDF key purpose")),
            _ => panic!("Expected Fail for unregistered purpose label"),
        }
    }
}
