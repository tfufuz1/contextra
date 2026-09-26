//! Gate: check-ring0-async-purity
//! Prüft alle Ring-0-Crates auf unerlaubte Async-Runtime-Dependencies in [dependencies].

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::check_ring_layering::{get_crate_ring, Ring};

pub const DENY_LIST: &[&str] = &["tokio", "async-std", "smol", "futures-executor"];

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AsyncException {
    pub crate_name: String,
    pub dependency: String,
    pub reason: String,
    pub adr_reference: String,
}

#[derive(Debug, Deserialize, Default)]
struct ExceptionsFile {
    #[serde(rename = "exceptions", default)]
    exceptions: Vec<RawException>,
}

#[derive(Debug, Deserialize)]
struct RawException {
    #[serde(alias = "crate")]
    crate_name: String,
    dependency: String,
    reason: String,
    adr_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncPurityViolation {
    pub crate_name: String,
    pub dependency: String,
    pub is_allowlisted: bool,
    pub reason: Option<String>,
    pub adr_reference: Option<String>,
}

pub fn load_allowlist(allowlist_path: &Path) -> Result<Vec<AsyncException>, String> {
    if !allowlist_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(allowlist_path)
        .map_err(|e| format!("Failed to read allowlist file {:?}: {}", allowlist_path, e))?;
    let parsed: ExceptionsFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse allowlist file {:?}: {}", allowlist_path, e))?;

    let mut exceptions = Vec::with_capacity(parsed.exceptions.len());
    for r in parsed.exceptions {
        let is_valid_adr = r.adr_reference != "ADR-TODO"
            && r.adr_reference
                .strip_prefix("ADR-")
                .is_some_and(|rest| rest.chars().next().is_some_and(|c| c.is_ascii_digit()));

        if !is_valid_adr {
            return Err(format!(
                "Invalid adr_reference '{}' for crate '{}' and dependency '{}': must start with 'ADR-' followed by a digit and cannot be 'ADR-TODO'",
                r.adr_reference, r.crate_name, r.dependency
            ));
        }

        exceptions.push(AsyncException {
            crate_name: r.crate_name,
            dependency: r.dependency,
            reason: r.reason,
            adr_reference: r.adr_reference,
        });
    }

    Ok(exceptions)
}

/// Liest ein Cargo.toml und prüft NUR den [dependencies]-Abschnitt auf verbotene Dependencies.
pub fn check_cargo_toml_dependencies(
    crate_name: &str,
    cargo_toml_content: &str,
    allowlist: &[AsyncException],
) -> Result<Vec<AsyncPurityViolation>, String> {
    let toml_val: toml::Value = toml::from_str(cargo_toml_content)
        .map_err(|e| format!("Failed to parse Cargo.toml for '{}': {}", crate_name, e))?;

    let mut violations = Vec::new();

    if let Some(deps_table) = toml_val.get("dependencies").and_then(|d| d.as_table()) {
        for &deny_dep in DENY_LIST {
            if deps_table.contains_key(deny_dep) {
                let allow_entry = allowlist
                    .iter()
                    .find(|e| e.crate_name == crate_name && e.dependency == deny_dep);

                violations.push(AsyncPurityViolation {
                    crate_name: crate_name.to_string(),
                    dependency: deny_dep.to_string(),
                    is_allowlisted: allow_entry.is_some(),
                    reason: allow_entry.map(|e| e.reason.clone()),
                    adr_reference: allow_entry.map(|e| e.adr_reference.clone()),
                });
            }
        }
    }

    Ok(violations)
}

pub fn run_check_ring0_async_purity() -> Result<bool, String> {
    println!("=== Running xtask check-ring0-async-purity ===");

    let root_dir = crate::find_root_dir();
    let allowlist_path = root_dir.join("xtask/ring0-async-exceptions.toml");
    let allowlist = load_allowlist(&allowlist_path)?;

    // Alle Ring 0 Crates ermitteln
    let workspace_crates = crate::get_workspace_crates();
    let mut ring0_crates: Vec<String> = workspace_crates
        .iter()
        .filter_map(|c| {
            if get_crate_ring(&c.name) == Some(Ring::Ring0) {
                Some(c.name.clone())
            } else {
                None
            }
        })
        .collect();

    // Deduplizieren und sortieren
    let ring0_set: HashSet<_> = ring0_crates.drain(..).collect();
    ring0_crates = ring0_set.into_iter().collect();
    ring0_crates.sort();

    let mut all_violations = Vec::new();

    for crate_name in &ring0_crates {
        let crate_path = if crate_name == "contextra-vector" {
            root_dir.join("crates/contextra-vector/Cargo.toml")
        } else {
            root_dir.join("crates").join(crate_name).join("Cargo.toml")
        };

        if !crate_path.exists() {
            eprintln!(
                "⚠️ Warning: Cargo.toml for Ring 0 crate '{}' not found at {:?}",
                crate_name, crate_path
            );
            continue;
        }

        let content = fs::read_to_string(&crate_path).map_err(|e| {
            format!(
                "Failed to read Cargo.toml for '{}' at {:?}: {}",
                crate_name, crate_path, e
            )
        })?;

        let violations = check_cargo_toml_dependencies(crate_name, &content, &allowlist)?;
        all_violations.extend(violations);
    }

    if all_violations.is_empty() {
        println!("✅ check-ring0-async-purity: 0 Verstöße gefunden.");
        return Ok(true);
    }

    let allowlisted: Vec<_> = all_violations.iter().filter(|v| v.is_allowlisted).collect();
    let unallowlisted: Vec<_> = all_violations
        .iter()
        .filter(|v| !v.is_allowlisted)
        .collect();

    if !allowlisted.is_empty() {
        println!(
            "⚠️ check-ring0-async-purity: {} dokumentierte Allowlist-Ausnahme(n) gefunden:",
            allowlisted.len()
        );
        for v in &allowlisted {
            println!(
                "  ALLOWLISTED [{}] Crate '{}' -> Dependency '{}' ({})",
                v.adr_reference.as_deref().unwrap_or("ADR-TODO"),
                v.crate_name,
                v.dependency,
                v.reason.as_deref().unwrap_or("No reason specified")
            );
        }
    }

    if !unallowlisted.is_empty() {
        eprintln!(
            "❌ check-ring0-async-purity: {} UNDOKUMENTIERTE Async-Verstöße in Ring 0 gefunden:",
            unallowlisted.len()
        );
        for v in &unallowlisted {
            eprintln!(
                "  ASYNC-VIOLATION: Ring 0 Crate '{}' hat verbotene Dependency '{}' in [dependencies]",
                v.crate_name, v.dependency
            );
        }
        return Ok(false);
    }

    println!("✅ check-ring0-async-purity: Alle Verstöße sind auf der Allow-List dokumentiert.");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unallowed_tokio_dependency_fails() {
        let cargo_toml = r#"
[package]
name = "contextra-test-ring0"
version = "0.1.0"

[dependencies]
tokio = { workspace = true }
serde = "1"

[dev-dependencies]
proptest = "1"
"#;

        let allowlist: Vec<AsyncException> = Vec::new();
        let violations =
            check_cargo_toml_dependencies("contextra-test-ring0", cargo_toml, &allowlist).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].crate_name, "contextra-test-ring0");
        assert_eq!(violations[0].dependency, "tokio");
        assert!(!violations[0].is_allowlisted);
    }

    #[test]
    fn test_clean_cargo_toml_passes() {
        let cargo_toml = r#"
[package]
name = "contextra-clean-ring0"
version = "0.1.0"

[dependencies]
serde = "1"
thiserror = "1"

[dev-dependencies]
tokio = { workspace = true, features = ["full"] }
criterion = "0.5"
"#;

        let allowlist: Vec<AsyncException> = Vec::new();
        let violations =
            check_cargo_toml_dependencies("contextra-clean-ring0", cargo_toml, &allowlist).unwrap();

        assert!(
            violations.is_empty(),
            "Expected 0 violations when tokio is only in [dev-dependencies], got {:?}",
            violations
        );
    }
}
