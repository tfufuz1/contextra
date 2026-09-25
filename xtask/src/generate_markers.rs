use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_root_dir() -> PathBuf {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(manifest_dir);
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if let Ok(curr) = env::current_dir() {
        let mut dir = curr;
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if Path::new("Cargo.toml").exists()
        && fs::read_to_string("Cargo.toml")
            .unwrap_or_default()
            .contains("members")
    {
        PathBuf::from(".")
    } else if Path::new("../Cargo.toml").exists() {
        PathBuf::from("..")
    } else {
        PathBuf::from(".")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Capabilities {
    pub version: Option<VersionInfo>,
    #[serde(default)]
    pub crates: BTreeMap<String, CapabilityCrateEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct VersionInfo {
    pub schema: Option<String>,
    pub spec: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CapabilityCrateEntry {
    pub ring: Option<String>,
    pub maturity: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub path: Option<String>,
    pub unsafe_island: Option<bool>,
    #[serde(default)]
    pub may_depend_on: Vec<String>,
    pub ci_class: Option<String>,
    #[serde(default)]
    pub spec: Vec<String>,
    pub test: Option<String>,
}

pub fn render_marker_table(capabilities: &Capabilities) -> String {
    let mut out = String::new();
    out.push_str("# Contextra — Capability & Maturity Markers\n\n");
    out.push_str("> **Hinweis**: Diese Datei ist autogeneriert aus `capabilities.toml` durch `cargo xtask generate-markers` (P12).\n");
    out.push_str("> Alle Reifegrad-Marker werden maschinell aus dem Manifest abgeleitet.\n\n");

    out.push_str("| Crate | Ring | Maturity Marker | Capabilities | Beschreibung |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

    for (name, entry) in &capabilities.crates {
        let ring = entry.ring.as_deref().unwrap_or("-");
        let maturity_marker = match entry.maturity.as_deref() {
            Some("stable") => "🟢 stable".to_string(),
            Some("experimental") => "🟡 experimental".to_string(),
            Some("deprecated") => "🔴 deprecated".to_string(),
            Some(other) => format!("🔴 {}", other),
            None => "🔴 unklassifiziert".to_string(),
        };

        let caps_formatted = if entry.capabilities.is_empty() {
            "-".to_string()
        } else {
            entry
                .capabilities
                .iter()
                .map(|c| format!("`{}`", c))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let description = entry.description.as_deref().unwrap_or("-");

        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            name, ring, maturity_marker, caps_formatted, description
        ));
    }

    out
}

pub fn load_capabilities_from_file(path: &Path) -> Result<Capabilities, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read capabilities file '{}': {}", path.display(), e))?;
    toml::from_str::<Capabilities>(&content)
        .map_err(|e| format!("Failed to parse capabilities file '{}': {}", path.display(), e))
}

pub fn run_generate_markers(output_path: Option<&Path>) -> Result<(), String> {
    println!("=== XTask: Generate Capability Markers ===");
    let root = find_root_dir();
    let capabilities_path = root.join("capabilities.toml");

    let capabilities = load_capabilities_from_file(&capabilities_path)?;
    let rendered = render_marker_table(&capabilities);

    let default_out_path = root.join("docs/generated/capability-markers.md");
    let target_path = output_path.unwrap_or(&default_out_path);

    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(target_path, &rendered)
        .map_err(|e| format!("Failed to write markers output to '{}': {}", target_path.display(), e))?;

    println!(
        "✅ Capability markers successfully written to '{}'.",
        target_path.display()
    );
    Ok(())
}

pub fn run_check_marker_drift(output_path: Option<&Path>) -> Result<(), String> {
    println!("=== Gate: Check Capability Marker Drift ===");
    let root = find_root_dir();
    let capabilities_path = root.join("capabilities.toml");

    if !capabilities_path.exists() {
        return Err(format!(
            "❌ Gate failed: 'capabilities.toml' not found at '{}'",
            capabilities_path.display()
        ));
    }

    let default_out_path = root.join("docs/generated/capability-markers.md");
    let target_path = output_path.unwrap_or(&default_out_path);

    if !target_path.exists() {
        return Err(format!(
            "❌ Gate failed: Target markers file not found at '{}'. Run 'cargo xtask generate-markers'.",
            target_path.display()
        ));
    }

    let capabilities = load_capabilities_from_file(&capabilities_path)?;
    let newly_rendered = render_marker_table(&capabilities);

    let existing_content = fs::read_to_string(target_path).map_err(|e| {
        format!(
            "Failed to read existing markers file '{}': {}",
            target_path.display(),
            e
        )
    })?;

    if normalize_text(&newly_rendered) == normalize_text(&existing_content) {
        println!(
            "✅ Gate passed: Capability markers file '{}' is in sync with 'capabilities.toml'.",
            target_path.display()
        );
        Ok(())
    } else {
        let err_msg = format!(
            "❌ Gate failed: Marker drift detected! 'capabilities.toml' does not match '{}'.\n💡 Run 'cargo xtask generate-markers' to update the generated Markdown file.",
            target_path.display()
        );
        eprintln!("{}", err_msg);
        Err(err_msg)
    }
}

fn normalize_text(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string()
}
