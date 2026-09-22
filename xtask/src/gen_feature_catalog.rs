//! Subcommand `gen-feature-catalog`
//! Generiert aus den Cargo.toml [features]-Sektionen aller Workspace-Crates
//! eine Markdown-Übersicht unter `docs/generated/feature-catalog.md`.

use std::collections::BTreeMap;
use std::fs;

pub fn run_gen_feature_catalog() -> Result<(), String> {
    println!("=== Running xtask gen-feature-catalog ===");

    let root = crate::find_root_dir();
    let workspace_crates = crate::get_workspace_crates();

    let mut catalog: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for crate_info in &workspace_crates {
        let crate_cargo_path = root.join(&crate_info.path).join("Cargo.toml");
        if !crate_cargo_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&crate_cargo_path)
            .map_err(|e| format!("Fehler beim Lesen von {}: {}", crate_cargo_path.display(), e))?;

        let toml_val: toml::Value = toml::from_str(&content)
            .map_err(|e| format!("Fehler beim Parsen von {}: {}", crate_cargo_path.display(), e))?;

        let mut features_map = BTreeMap::new();

        if let Some(features) = toml_val.get("features").and_then(|f| f.as_table()) {
            for (feature_name, feature_deps) in features {
                let deps = if let Some(arr) = feature_deps.as_array() {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                } else {
                    Vec::new()
                };
                features_map.insert(feature_name.clone(), deps);
            }
        }

        catalog.insert(crate_info.name.clone(), features_map);
    }

    let mut markdown = String::new();
    markdown.push_str("# MemFuse — Feature Catalog\n\n");
    markdown.push_str("> **Hinweis**: Diese Datei ist autogeneriert durch `cargo xtask gen-feature-catalog`.\n");
    markdown.push_str("> Sie listet alle verfuegbaren Cargo Feature Flags aller Workspace-Crates auf.\n\n");

    for (crate_name, features) in &catalog {
        markdown.push_str(&format!("## Crate `{}`\n\n", crate_name));

        if features.is_empty() {
            markdown.push_str("*Keine expliziten Feature Flags deklariert.*\n\n");
        } else {
            markdown.push_str("| Feature Flag | Aktivierte Abhaengigkeiten / Flags |\n");
            markdown.push_str("| :--- | :--- |\n");
            for (feat_name, feat_deps) in features {
                let deps_str = if feat_deps.is_empty() {
                    "`default` / keine weiteren Flags".to_string()
                } else {
                    feat_deps
                        .iter()
                        .map(|d| format!("`{}`", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                markdown.push_str(&format!("| `{}` | {} |\n", feat_name, deps_str));
            }
            markdown.push('\n');
        }
    }

    let out_dir = root.join("docs/generated");
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)
            .map_err(|e| format!("Fehler beim Erstellen von {}: {}", out_dir.display(), e))?;
    }

    let out_path = out_dir.join("feature-catalog.md");
    fs::write(&out_path, markdown)
        .map_err(|e| format!("Fehler beim Schreiben von {}: {}", out_path.display(), e))?;

    println!("✅ Feature-Katalog erfolgreich generiert: {}", out_path.display());
    Ok(())
}
