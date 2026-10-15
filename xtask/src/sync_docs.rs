//! Gate G-04: sync-docs --check
//! Vergleicht capabilities.toml mit der in der Spezifikation (§4.1) festgelegten Ringtabelle
//! und stellt sicher, dass jedes Workspace-Member (und geplante Crates) korrekt eingetragen ist.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CapabilitiesPartial {
    crates: BTreeMap<String, CrateEntry>,
}

#[derive(Debug, Deserialize)]
struct CrateEntry {
    ring: String,
    #[serde(default)]
    planned: bool,
}

/// Erwartete normative Ringzuordnung gemäß Spezifikation §4.1
pub fn canonical_spec_rings() -> &'static [(&'static str, &'static str)] {
    &[
        // Ring 0
        ("contextra-types", "Ring 0"),
        ("contextra-ports", "Ring 0"),
        ("contextra-wire", "Ring 0"),
        ("contextra-sys", "Ring 0"),
        ("contextra-durable-fs", "Ring 0"),
        ("contextra-simd", "Ring 0"),
        ("contextra-crypto-core", "Ring 0"),
        ("contextra-mvcc", "Ring 0"),
        ("contextra-text", "Ring 0"),
        ("contextra-rank", "Ring 0"),
        ("contextra-vector", "Ring 0"),
        ("contextra-adapt", "Ring 0"),
        ("contextra-graph", "Ring 0"),
        ("contextra-crypto-kv", "Ring 0"),
        ("contextra-proof-verify", "Ring 0"),
        ("contextra-audit-export", "Ring 0"),
        ("contextra-avv-generator", "Ring 0"),
        ("contextra-crypto", "Ring 0"), // Monolithisches Crypto bis Split
        ("contextra-core", "Ring 0"),   // Monolithisches Core bis Split

        // Ring 1
        ("contextra-store", "Ring 1"),
        ("contextra-checkpoint", "Ring 1"),
        ("contextra-kvcache", "Ring 1"),

        // Ring 2
        ("contextra-infer-candle", "Ring 2"),
        ("contextra-infer-ollama", "Ring 2"),
        ("contextra-infer-onnx", "Ring 2"),
        ("contextra-sandbox", "Ring 2"),

        // Ring 3
        ("contextra-privacy", "Ring 3"),
        ("contextra-engine", "Ring 3"),
        ("contextra-erasure-evidence", "Ring 3"),
        ("contextra-db", "Ring 3"),
        ("contextra-cognition", "Ring 3"),
        ("contextra-router", "Ring 3"),
        ("contextra-agent", "Ring 3"),

        // Ring 4
        ("contextra", "Ring 4"),
        ("contextra-mcp", "Ring 4"),
        ("contextra-py", "Ring 4"),
        ("contextra-license", "Ring 4"),

        // Tooling
        ("contextra-testkit", "Tooling"),
        ("contextra-bench", "Tooling"),
        ("xtask", "Tooling"),
        ("xtask-heavy", "Tooling"),
        ("contextra-crypto-fuzz", "Tooling"),
        ("contextra-db-fuzz", "Tooling"),
        ("contextra-mcp-fuzz", "Tooling"),
        ("contextra-store-fuzz", "Tooling"),
        ("contextra-text-fuzz", "Tooling"),
        ("contextra-vector-fuzz", "Tooling"),
        ("contextra-wire-fuzz", "Tooling"),
    ]
}

pub fn verify_capabilities_against_spec(root: &Path) -> Result<(), Vec<String>> {
    let caps_path = root.join("capabilities.toml");
    let content = match fs::read_to_string(&caps_path) {
        Ok(c) => c,
        Err(e) => return Err(vec![format!("Failed to read capabilities.toml: {}", e)]),
    };

    let parsed: CapabilitiesPartial = match toml::from_str(&content) {
        Ok(p) => p,
        Err(e) => return Err(vec![format!("Failed to parse capabilities.toml: {}", e)]),
    };

    let mut errors = Vec::new();
    let spec_map: BTreeMap<&str, &str> = canonical_spec_rings().iter().copied().collect();

    for (crate_name, expected_ring) in &spec_map {
        match parsed.crates.get(*crate_name) {
            Some(entry) => {
                let actual_ring = entry.ring.trim();
                if actual_ring != *expected_ring {
                    errors.push(format!(
                        "Crate '{}': expected ring '{}' according to Spec §4.1, but found '{}' in capabilities.toml",
                        crate_name, expected_ring, actual_ring
                    ));
                }
            }
            None => {
                errors.push(format!(
                    "Crate '{}' specified in Spec §4.1 is missing from capabilities.toml",
                    crate_name
                ));
            }
        }
    }

    for (crate_name, entry) in &parsed.crates {
        if !spec_map.contains_key(crate_name.as_str()) {
            errors.push(format!(
                "Crate '{}' (ring '{}') found in capabilities.toml is not recognized in Spec §4.1 ring table",
                crate_name, entry.ring
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_docs_spec_verification_passes() {
        let root = crate::find_root_dir();
        let res = verify_capabilities_against_spec(&root);
        assert!(res.is_ok(), "G-04 check failed: {:?}", res.err());
    }

    #[test]
    fn test_sync_docs_spec_verification_fails_on_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let caps_path = temp_dir.path().join("capabilities.toml");

        let mut content = String::from("[version]\nschema = \"2\"\nspec = \"v4\"\n\n");
        for (crate_name, ring) in canonical_spec_rings() {
            let actual_ring = if *crate_name == "contextra-store" {
                "Ring 4" // Wrong ring for testing
            } else {
                *ring
            };
            content.push_str(&format!(
                "[crates.{}]\nring = '{}'\nmaturity = 'stable'\ndescription = 'test'\npath = '{}'\n\n",
                crate_name, actual_ring, crate_name
            ));
        }

        fs::write(&caps_path, content).unwrap();

        let res = verify_capabilities_against_spec(temp_dir.path());
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("contextra-store")));
    }
}
