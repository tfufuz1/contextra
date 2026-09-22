//! Gate: check-ring0-async-purity
//! Validiert die Ring-0-Crates gegen eine Deny-Liste von Async-Runtime-Dependencies im [dependencies]-Abschnitt:
//!   - tokio
//!   - async-std
//!   - smol
//!   - futures-executor

use crate::check_ring_layering::{get_crate_ring, Ring};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

pub const DENY_LIST: &[&str] = &["tokio", "async-std", "smol", "futures-executor"];

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct AsyncExceptionEntry {
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub dependency: String,
    pub reason: String,
    pub adr: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AsyncExceptionsConfig {
    #[serde(default)]
    pub exceptions: Vec<AsyncExceptionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurityViolation {
    pub crate_name: String,
    pub dependency: String,
    pub is_allowlisted: bool,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    name: String,
    manifest_path: String,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: HashSet<String>,
}

pub fn check_crate_async_purity(
    crate_name: &str,
    manifest_toml_str: &str,
    allowlist: &[AsyncExceptionEntry],
) -> Result<Vec<PurityViolation>, String> {
    let toml_val: toml::Value =
        toml::from_str(manifest_toml_str).map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let mut violations = Vec::new();

    if let Some(deps_table) = toml_val.get("dependencies").and_then(|d| d.as_table()) {
        for dep_name in deps_table.keys() {
            if DENY_LIST.contains(&dep_name.as_str()) {
                let is_allowlisted = allowlist
                    .iter()
                    .any(|e| e.crate_name == crate_name && e.dependency == *dep_name);

                violations.push(PurityViolation {
                    crate_name: crate_name.to_string(),
                    dependency: dep_name.clone(),
                    is_allowlisted,
                });
            }
        }
    }

    Ok(violations)
}

pub fn load_allowlist<P: AsRef<Path>>(path: P) -> Result<AsyncExceptionsConfig, String> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(format!("Allowlist file does not exist: {}", path_ref.display()));
    }
    let content = fs::read_to_string(path_ref)
        .map_err(|e| format!("Failed to read allowlist file {}: {}", path_ref.display(), e))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse allowlist TOML {}: {}", path_ref.display(), e))
}

pub fn run_check_ring0_async_purity() -> Result<bool, String> {
    println!("=== Running xtask check-ring0-async-purity ===");

    let root_dir = crate::find_root_dir();
    let allowlist_path = root_dir.join("xtask/ring0-async-exceptions.toml");

    let allowlist_config = load_allowlist(&allowlist_path)?;

    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("Failed to execute cargo metadata: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "cargo metadata command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let metadata: CargoMetadata =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse cargo metadata: {}", e))?;

    let mut all_violations = Vec::new();

    for pkg in &metadata.packages {
        // Only inspect workspace members
        let is_member = metadata.workspace_members.iter().any(|m| m.contains(&pkg.name))
            || metadata.workspace_members.contains(&pkg.name);

        if !is_member {
            continue;
        }

        let ring = match get_crate_ring(&pkg.name) {
            Some(r) => r,
            None => continue,
        };

        if ring != Ring::Ring0 {
            continue;
        }

        let manifest_content = fs::read_to_string(&pkg.manifest_path)
            .map_err(|e| format!("Failed to read manifest for {}: {}", pkg.name, e))?;

        let violations =
            check_crate_async_purity(&pkg.name, &manifest_content, &allowlist_config.exceptions)?;

        all_violations.extend(violations);
    }

    let allowlisted: Vec<_> = all_violations.iter().filter(|v| v.is_allowlisted).collect();
    let unallowlisted: Vec<_> = all_violations.iter().filter(|v| !v.is_allowlisted).collect();

    if !allowlisted.is_empty() {
        println!(
            "⚠️ check-ring0-async-purity: {} dokumentierte Allowlist-Ausnahme(n) gefunden:",
            allowlisted.len()
        );
        for v in &allowlisted {
            let entry = allowlist_config
                .exceptions
                .iter()
                .find(|e| e.crate_name == v.crate_name && e.dependency == v.dependency);
            let adr = entry.map(|e| e.adr.as_str()).unwrap_or("TODO");
            let reason = entry.map(|e| e.reason.as_str()).unwrap_or("");
            println!(
                "  ALLOWLISTED [{}] {} -> {} ({})",
                adr, v.crate_name, v.dependency, reason
            );
        }
    }

    if !unallowlisted.is_empty() {
        eprintln!(
            "❌ check-ring0-async-purity: {} UNDOKUMENTIERTE Async-Purity-Verstöße in Ring 0 gefunden:",
            unallowlisted.len()
        );
        for v in &unallowlisted {
            eprintln!(
                "  PURITY-VIOLATION: Crate '{}' -> verletzte Dependency '{}'",
                v.crate_name, v.dependency
            );
        }
        return Ok(false);
    }

    println!("✅ check-ring0-async-purity: Ring-0 Async Purity Gate PASSED.");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_crate_async_purity_with_unallowed_tokio_fails() {
        let fixture_toml = r#"
[package]
name = "memfuse-test-ring0"
version = "0.1.0"

[dependencies]
memfuse-core = { path = "../memfuse-core" }
tokio = "1.0"

[dev-dependencies]
tokio = { version = "1.0", features = ["full"] }
"#;

        let allowlist: Vec<AsyncExceptionEntry> = vec![];

        let violations =
            check_crate_async_purity("memfuse-test-ring0", fixture_toml, &allowlist).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].crate_name, "memfuse-test-ring0");
        assert_eq!(violations[0].dependency, "tokio");
        assert!(!violations[0].is_allowlisted);
    }

    #[test]
    fn test_check_crate_async_purity_clean_passes() {
        let fixture_toml = r#"
[package]
name = "memfuse-test-ring0"
version = "0.1.0"

[dependencies]
memfuse-core = { path = "../memfuse-core" }
ahash = "0.8"

[dev-dependencies]
tokio = { version = "1.0", features = ["full"] }
"#;

        let allowlist: Vec<AsyncExceptionEntry> = vec![];

        let violations =
            check_crate_async_purity("memfuse-test-ring0", fixture_toml, &allowlist).unwrap();

        assert!(violations.is_empty());
    }
}
