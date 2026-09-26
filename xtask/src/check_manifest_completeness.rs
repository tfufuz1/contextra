//! Gate: check-manifest-completeness
//!
//! Prüft die Vollständigkeit und Symmetrie zwischen `Cargo.toml` (Workspace Members)
//! und `capabilities.toml` (`[crates.X]`-Blöcke).
//!
//! Invarianten:
//! - **INV-MANIFEST-COMPLETE**: Jeder registrierte Workspace-Member in `Cargo.toml`
//!   muss einen entsprechenden `[crates.X]`-Block in `capabilities.toml` aufweisen.
//! - **INV-CRATE-REACHABILITY**: Jeder `[crates.X]`-Block in `capabilities.toml`
//!   muss einem tatsächlich registrierten Workspace-Member entsprechen.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Ausnahmeliste für Nicht-Standard-Workspace-Mitglieder gemäß §22.3.
/// - `contextra-bench`: Befindet sich unter `benchmarks/` und hat kein `crates/`-Präfix.
/// - `xtask`: Ist das CI/Governance-Tooling selbst und nicht Teil der Produkt-Crates in `capabilities.toml`.
pub const EXEMPT_CRATES: &[&str] = &["contextra-bench", "xtask"];

/// Parst die Workspace-Mitglieder aus der Root-`Cargo.toml` und löst für jeden
/// Elementpfad den Crate-Namen aus dem jeweiligen `Cargo.toml` (`[package] name`) auf.
pub fn extract_workspace_member_crates(
    root: &Path,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let root_cargo_path = root.join("Cargo.toml");
    let content = fs::read_to_string(&root_cargo_path)
        .map_err(|e| format!("Fehler beim Lesen von {}: {}", root_cargo_path.display(), e))?;

    let root_toml: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Fehler beim Parsen von {}: {}", root_cargo_path.display(), e))?;

    let members = root_toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .ok_or_else(|| {
            format!(
                "Fehlendes oder ungültiges '[workspace] members'-Array in {}",
                root_cargo_path.display()
            )
        })?;

    let mut member_crates = BTreeMap::new();

    for member_val in members {
        let member_rel_path = member_val.as_str().ok_or_else(|| {
            format!(
                "Ungültiger Pfad-Eintrag in '[workspace] members' in {}",
                root_cargo_path.display()
            )
        })?;

        let member_cargo_path = root.join(member_rel_path).join("Cargo.toml");
        if !member_cargo_path.exists() {
            return Err(format!(
                "Workspace-Mitglied-Manifest existiert nicht: {}",
                member_cargo_path.display()
            ));
        }

        let member_content = fs::read_to_string(&member_cargo_path).map_err(|e| {
            format!(
                "Fehler beim Lesen von Workspace-Mitglied {}: {}",
                member_cargo_path.display(),
                e
            )
        })?;

        let member_toml: toml::Value = toml::from_str(&member_content).map_err(|e| {
            format!(
                "Fehler beim Parsen von Workspace-Mitglied {}: {}",
                member_cargo_path.display(),
                e
            )
        })?;

        let crate_name = member_toml
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| {
                format!(
                    "Fehlender '[package] name'-Eintrag in {}",
                    member_cargo_path.display()
                )
            })?
            .to_string();

        member_crates.insert(crate_name, PathBuf::from(member_rel_path));
    }

    Ok(member_crates)
}

/// Parst `capabilities.toml` und extrahiert alle Schlüssel unter `[crates]`.
pub fn extract_capabilities_crates(root: &Path) -> Result<BTreeSet<String>, String> {
    let capabilities_path = root.join("capabilities.toml");
    let content = fs::read_to_string(&capabilities_path)
        .map_err(|e| format!("Fehler beim Lesen von {}: {}", capabilities_path.display(), e))?;

    let capabilities_toml: toml::Value = toml::from_str(&content).map_err(|e| {
        format!("Fehler beim Parsen von {}: {}", capabilities_path.display(), e)
    })?;

    let crates_table = capabilities_toml
        .get("crates")
        .and_then(|c| c.as_table())
        .ok_or_else(|| {
            format!(
                "Fehlende oder ungültige '[crates]'-Tabelle in {}",
                capabilities_path.display()
            )
        })?;

    let crate_keys: BTreeSet<String> = crates_table.keys().cloned().collect();
    Ok(crate_keys)
}

/// Prüft die Symmetrie zwischen Workspace-Mitgliedern und Capabilities-Einträgen.
pub fn validate_manifest_completeness(
    member_crates: &BTreeMap<String, PathBuf>,
    capabilities_crates: &BTreeSet<String>,
    exemptions: &[&str],
) -> Result<(), String> {
    let mut missing_capabilities = Vec::new();
    let mut orphan_capabilities = Vec::new();

    // c. INV-MANIFEST-COMPLETE: Jeder Workspace-Member muss in capabilities.toml stehen (außer Ausnahmen)
    for (crate_name, member_path) in member_crates {
        if exemptions.contains(&crate_name.as_str()) {
            continue;
        }
        if !capabilities_crates.contains(crate_name) {
            missing_capabilities.push(format!(
                "  - {} (Pfad: {})",
                crate_name,
                member_path.display()
            ));
        }
    }

    // d. INV-CRATE-REACHABILITY: Jeder capabilities.toml-Eintrag muss einem Workspace-Member entsprechen
    for cap_crate in capabilities_crates {
        if !member_crates.contains_key(cap_crate) {
            orphan_capabilities.push(format!("  - {}", cap_crate));
        }
    }

    if missing_capabilities.is_empty() && orphan_capabilities.is_empty() {
        Ok(())
    } else {
        let mut err_msg = String::new();
        err_msg.push_str("Manifest-Vollständigkeitsprüfung fehlgeschlagen:\n");

        if !missing_capabilities.is_empty() {
            err_msg.push_str(&format!(
                "\n❌ INV-MANIFEST-COMPLETE: {} Workspace-Mitglied(er) ohne [crates.X]-Eintrag in capabilities.toml:\n{}\n",
                missing_capabilities.len(),
                missing_capabilities.join("\n")
            ));
        }

        if !orphan_capabilities.is_empty() {
            err_msg.push_str(&format!(
                "\n❌ INV-CRATE-REACHABILITY: {} verwaiste [crates.X]-Einträge in capabilities.toml ohne Workspace-Mitglied:\n{}\n",
                orphan_capabilities.len(),
                orphan_capabilities.join("\n")
            ));
        }

        Err(err_msg)
    }
}

/// Einstiegspunkt für den xtask-Gate-Check.
pub fn run() -> Result<(), String> {
    let root = crate::find_root_dir();
    let member_crates = extract_workspace_member_crates(&root)?;
    let capabilities_crates = extract_capabilities_crates(&root)?;

    validate_manifest_completeness(&member_crates, &capabilities_crates, EXEMPT_CRATES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_completeness_matching_synthetic() {
        let mut members = BTreeMap::new();
        members.insert(
            "contextra-core".to_string(),
            PathBuf::from("crates/contextra-core"),
        );
        members.insert(
            "contextra-store".to_string(),
            PathBuf::from("crates/contextra-store"),
        );

        let mut capabilities = BTreeSet::new();
        capabilities.insert("contextra-core".to_string());
        capabilities.insert("contextra-store".to_string());

        let result = validate_manifest_completeness(&members, &capabilities, EXEMPT_CRATES);
        assert!(result.is_ok());
    }

    #[test]
    fn test_manifest_completeness_missing_capability() {
        let mut members = BTreeMap::new();
        members.insert(
            "contextra-core".to_string(),
            PathBuf::from("crates/contextra-core"),
        );
        members.insert(
            "contextra-audit-export".to_string(),
            PathBuf::from("crates/contextra-audit-export"),
        );

        let mut capabilities = BTreeSet::new();
        capabilities.insert("contextra-core".to_string());

        let result = validate_manifest_completeness(&members, &capabilities, EXEMPT_CRATES);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("INV-MANIFEST-COMPLETE"));
        assert!(err.contains("contextra-audit-export"));
        assert!(!err.contains("INV-CRATE-REACHABILITY"));
    }

    #[test]
    fn test_manifest_completeness_orphan_capability() {
        let mut members = BTreeMap::new();
        members.insert(
            "contextra-core".to_string(),
            PathBuf::from("crates/contextra-core"),
        );

        let mut capabilities = BTreeSet::new();
        capabilities.insert("contextra-core".to_string());
        capabilities.insert("contextra-legacy-orphan".to_string());

        let result = validate_manifest_completeness(&members, &capabilities, EXEMPT_CRATES);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("INV-CRATE-REACHABILITY"));
        assert!(err.contains("contextra-legacy-orphan"));
        assert!(!err.contains("INV-MANIFEST-COMPLETE"));
    }

    #[test]
    fn test_manifest_completeness_exemptions() {
        let mut members = BTreeMap::new();
        members.insert(
            "contextra-core".to_string(),
            PathBuf::from("crates/contextra-core"),
        );
        members.insert(
            "contextra-bench".to_string(),
            PathBuf::from("benchmarks/contextra-bench"),
        );

        let mut capabilities = BTreeSet::new();
        capabilities.insert("contextra-core".to_string());

        let result = validate_manifest_completeness(&members, &capabilities, EXEMPT_CRATES);
        assert!(
            result.is_ok(),
            "Exempted crates like contextra-bench must not trigger INV-MANIFEST-COMPLETE"
        );
    }
}
