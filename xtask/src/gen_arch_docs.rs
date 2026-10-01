//! xtask subcommand: gen-arch-docs
//! Generiert die Crate-Ring-Tabelle und das Mermaid-Abhängigkeitsdiagramm in `ARCHITECTURE.md`
//! zwischen `<!-- BEGIN GENERATED -->` und `<!-- END GENERATED -->`.
//! Unterstützt `--check` für CI-Drift-Prüfung.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    name: String,
    description: Option<String>,
    dependencies: Vec<MetadataDependency>,
    metadata: Option<MetadataExtra>,
}

#[derive(Debug, Deserialize)]
struct MetadataExtra {
    contextra: Option<ContextraMetadata>,
}

#[derive(Debug, Deserialize)]
struct ContextraMetadata {
    ring: String,
}

#[derive(Debug, Deserialize)]
struct MetadataDependency {
    name: String,
    kind: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum RingSortKey {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
    Ring4,
    Tooling,
}

impl RingSortKey {
    fn from_str(s: &str) -> Self {
        match s.trim() {
            "0" | "Ring 0" => RingSortKey::Ring0,
            "1" | "Ring 1" => RingSortKey::Ring1,
            "2" | "Ring 2" => RingSortKey::Ring2,
            "3" | "Ring 3" => RingSortKey::Ring3,
            "4" | "Ring 4" => RingSortKey::Ring4,
            _ => RingSortKey::Tooling,
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            RingSortKey::Ring0 => "Ring 0",
            RingSortKey::Ring1 => "Ring 1",
            RingSortKey::Ring2 => "Ring 2",
            RingSortKey::Ring3 => "Ring 3",
            RingSortKey::Ring4 => "Ring 4",
            RingSortKey::Tooling => "Tooling",
        }
    }
}

pub fn generate_arch_docs_content(json_str: &str) -> Result<String, String> {
    let metadata: CargoMetadata = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse cargo metadata: {}", e))?;

    let workspace_packages: HashMap<String, &MetadataPackage> = metadata
        .packages
        .iter()
        .filter(|p| {
            metadata
                .workspace_members
                .iter()
                .any(|m| m.contains(&p.name))
                || metadata.workspace_members.contains(&p.name)
        })
        .map(|p| (p.name.clone(), p))
        .collect();

    struct CrateDocInfo {
        name: String,
        ring_key: RingSortKey,
        ring_str: String,
        description: String,
        normal_deps: Vec<String>,
    }

    let mut doc_crates: Vec<CrateDocInfo> = Vec::new();

    for (pkg_name, pkg) in &workspace_packages {
        let raw_ring = pkg
            .metadata
            .as_ref()
            .and_then(|m| m.contextra.as_ref())
            .map(|c| c.ring.as_str())
            .unwrap_or("tooling");

        let ring_key = RingSortKey::from_str(raw_ring);
        let ring_str = ring_key.display_name().to_string();
        let description = pkg
            .description
            .clone()
            .unwrap_or_else(|| "Keine Beschreibung".to_string());

        let mut normal_deps = Vec::new();
        for dep in &pkg.dependencies {
            let is_normal = dep.kind.is_none() || dep.kind.as_deref() == Some("normal");
            if is_normal && workspace_packages.contains_key(&dep.name) {
                normal_deps.push(dep.name.clone());
            }
        }
        normal_deps.sort();

        doc_crates.push(CrateDocInfo {
            name: pkg_name.clone(),
            ring_key,
            ring_str,
            description,
            normal_deps,
        });
    }

    doc_crates.sort_by(|a, b| {
        a.ring_key
            .cmp(&b.ring_key)
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut out = String::new();

    out.push_str("## 2. Crate-Inventar (Ist-Zustand)\n\n");
    out.push_str(&format!(
        "Die folgende Tabelle führt alle {} im Workspace definierten Crates auf, eingeordnet in das Ring-Modell:\n\n",
        doc_crates.len()
    ));

    out.push_str("| Crate-Name | Ring | Verantwortlichkeit |\n");
    out.push_str("|---|---|---|\n");

    for c in &doc_crates {
        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            c.name, c.ring_str, c.description
        ));
    }

    out.push_str("\n---\n\n");
    out.push_str("## 3. Abhängigkeitsdiagramm\n\n");
    out.push_str("Das folgende Mermaid-Diagramm bildet die tatsächlichen `[dependencies]` zwischen den Workspace-Crates ab:\n\n");

    out.push_str("```mermaid\ngraph TD\n");

    // BTreeMap to sort mermaid nodes deterministically
    let mut diagram_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in &doc_crates {
        diagram_edges.insert(c.name.clone(), c.normal_deps.clone());
    }

    let sanitize_node = |name: &str| -> String { name.replace('-', "_") };

    for (from_crate, deps) in &diagram_edges {
        let from_id = sanitize_node(from_crate);
        if deps.is_empty() {
            out.push_str(&format!("    {}[{}]\n", from_id, from_crate));
        } else {
            for dep in deps {
                let to_id = sanitize_node(dep);
                out.push_str(&format!(
                    "    {}[{}] --> {}[{}]\n",
                    from_id, from_crate, to_id, dep
                ));
            }
        }
    }

    out.push_str("```\n");

    Ok(out)
}

pub fn run_gen_arch_docs(check_only: bool) -> Result<(), String> {
    println!(
        "=== Running xtask gen-arch-docs (check_only={}) ===",
        check_only
    );

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
    let generated_block = generate_arch_docs_content(&json_str)?;

    let arch_path = Path::new("ARCHITECTURE.md");
    if !arch_path.exists() {
        return Err("ARCHITECTURE.md not found in working directory".to_string());
    }

    let content = fs::read_to_string(arch_path)
        .map_err(|e| format!("Failed to read ARCHITECTURE.md: {}", e))?;

    let start_marker = "<!-- BEGIN GENERATED -->";
    let end_marker = "<!-- END GENERATED -->";

    let (start_idx, end_idx) = match (content.find(start_marker), content.find(end_marker)) {
        (Some(s), Some(e)) => (s, e),
        _ => {
            return Err("Markers '<!-- BEGIN GENERATED -->' and/or '<!-- END GENERATED -->' not found in ARCHITECTURE.md".to_string());
        }
    };

    if start_idx >= end_idx {
        return Err("Marker '<!-- BEGIN GENERATED -->' appears after '<!-- END GENERATED -->' in ARCHITECTURE.md".to_string());
    }

    let before = &content[..start_idx + start_marker.len()];
    let after = &content[end_idx..];

    let new_content = format!("{}\n{}\n{}", before, generated_block.trim(), after);

    if check_only {
        if content.trim() != new_content.trim() {
            eprintln!("❌ ARCHITECTURE.md is out of sync with crate metadata or dependencies!");
            eprintln!("💡 Run `cargo run -p xtask -- gen-arch-docs` to update ARCHITECTURE.md.");
            return Err("ARCHITECTURE.md drift detected".to_string());
        }
        println!("✅ ARCHITECTURE.md is in sync.");
        Ok(())
    } else {
        fs::write(arch_path, new_content)
            .map_err(|e| format!("Failed to write ARCHITECTURE.md: {}", e))?;
        println!("✅ Successfully updated ARCHITECTURE.md.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_arch_docs_content() {
        let mock_json = r#"{
            "packages": [
                {
                    "name": "contextra-core",
                    "description": "Core primitives",
                    "dependencies": [],
                    "metadata": { "contextra": { "ring": "0" } }
                },
                {
                    "name": "contextra-store",
                    "description": "LSM store",
                    "dependencies": [
                        { "name": "contextra-core", "kind": null }
                    ],
                    "metadata": { "contextra": { "ring": "1" } }
                }
            ],
            "workspace_members": ["contextra-core", "contextra-store"]
        }"#;

        let content = generate_arch_docs_content(mock_json).unwrap();
        assert!(content.contains("## 2. Crate-Inventar (Ist-Zustand)"));
        assert!(content.contains("`contextra-core` | Ring 0 | Core primitives"));
        assert!(content.contains("`contextra-store` | Ring 1 | LSM store"));
        assert!(content.contains("```mermaid"));
        assert!(
            content.contains("contextra_store[contextra-store] --> contextra_core[contextra-core]")
        );
    }
}
