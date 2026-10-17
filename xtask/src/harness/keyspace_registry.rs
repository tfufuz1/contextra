//! Gates KEYFAMILIES.toml against KeyFamilyRegistry code.

use std::fs;

pub fn run_keyspace_registry(_args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let keyfamilies_path = root.join("docs/deletion/KEYFAMILIES.toml");
    if !keyfamilies_path.exists() {
        eprintln!("❌ KEYFAMILIES.toml not found at {}", keyfamilies_path.display());
        return 1;
    }

    let content = match fs::read_to_string(&keyfamilies_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to read KEYFAMILIES.toml: {}", e);
            return 1;
        }
    };

    let registry_code_path = root.join("crates/contextra-engine/src/deletion/registry.rs");
    let code_content = if registry_code_path.exists() {
        fs::read_to_string(&registry_code_path).unwrap_or_default()
    } else {
        String::new()
    };

    match verify_keyspace_registry(&content, &code_content) {
        Ok(()) => {
            println!("✅ keyspace-registry gate passed");
            0
        }
        Err(e) => {
            eprintln!("❌ keyspace-registry gate failed: {}", e);
            1
        }
    }
}

pub fn verify_keyspace_registry(keyfamilies_toml: &str, registry_code: &str) -> Result<(), String> {
    if keyfamilies_toml.trim().is_empty() {
        return Err("KEYFAMILIES.toml is empty".to_string());
    }

    if registry_code.trim().is_empty() {
        return Err("KeyFamilyRegistry code is empty or missing".to_string());
    }

    // Parse TOML
    let value: toml::Value = toml::from_str(keyfamilies_toml)
        .map_err(|e| format!("Failed to parse KEYFAMILIES.toml: {}", e))?;

    let families = value.get("key_family")
        .or_else(|| value.get("family"))
        .or_else(|| value.get("families"))
        .and_then(|f| f.as_array())
        .or_else(|| value.as_table().and_then(|t| t.values().find_map(|v| v.as_array())));

    let mut found_count = 0;
    if let Some(families_arr) = families {
        for fam in families_arr {
            if let Some(id) = fam.get("id").and_then(|s| s.as_str()).or_else(|| fam.get("name").and_then(|s| s.as_str())) {
                if !registry_code.contains(id) {
                    return Err(format!("Key family '{}' declared in KEYFAMILIES.toml but missing from code registry", id));
                }
                found_count += 1;
            }
        }
    }

    if found_count == 0 {
        return Err("No key families found in KEYFAMILIES.toml".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_keyspace_registry_negative_empty() {
        assert!(verify_keyspace_registry("", "").is_err());
    }

    #[test]
    fn test_verify_keyspace_registry_negative_missing_code() {
        let toml_data = r#"
[[key_family]]
id = "CTX-KEY-VECTOR-01"
name = "Vector Embeddings"
"#;
        assert!(verify_keyspace_registry(toml_data, "").is_err());
    }

    #[test]
    fn test_verify_keyspace_registry_negative_missing_key() {
        let toml_data = r#"
[[key_family]]
id = "CTX-KEY-VECTOR-01"
name = "Vector Embeddings"
"#;
        let code = r#"
pub const REGISTERED: &[&str] = &["CTX-KEY-DOCUMENT-01"];
"#;
        assert!(verify_keyspace_registry(toml_data, code).is_err());
    }

    #[test]
    fn test_verify_keyspace_registry_positive() {
        let toml_data = r#"
[[key_family]]
id = "CTX-KEY-VECTOR-01"
name = "Vector Embeddings"
"#;
        let code = r#"
pub const REGISTERED: &[&str] = &["CTX-KEY-VECTOR-01"];
"#;
        assert!(verify_keyspace_registry(toml_data, code).is_ok());
    }
}
