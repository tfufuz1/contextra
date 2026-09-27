use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateContextData {
    pub crate_name: String,
    pub ring: String,
    pub maturity: String,
    pub loc: usize,
    pub declared_dependencies: Vec<String>,
    pub open_ai_tags: Vec<String>,
    pub recent_commits: Vec<String>,
    pub agents_md_excerpt: Option<String>,
}

fn count_crate_loc(crate_dir: &Path) -> usize {
    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        return 0;
    }

    let mut total_lines = 0;
    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            total_lines += content.lines().count();
        }
    }
    total_lines
}

fn extract_declared_contextra_deps(crate_dir: &Path) -> Vec<String> {
    let cargo_toml = crate_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Vec::new();
    }

    let mut deps = Vec::new();
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        if let Ok(val) = toml::from_str::<toml::Value>(&content) {
            if let Some(dependencies) = val.get("dependencies").and_then(|d| d.as_table()) {
                for (dep_name, _) in dependencies {
                    if dep_name.starts_with("contextra-") {
                        deps.push(dep_name.clone());
                    }
                }
            }
        }
    }
    deps.sort();
    deps
}

fn scan_crate_ai_tags(root: &Path, crate_dir: &Path) -> Vec<String> {
    if !crate_dir.exists() {
        return Vec::new();
    }

    let mut tags = Vec::new();
    for entry in WalkDir::new(crate_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            let relative_path = path.strip_prefix(root).unwrap_or(path);
            for (line_idx, line) in content.lines().enumerate() {
                if line.contains("AI-TAG[") {
                    tags.push(format!(
                        "{}:{}: {}",
                        relative_path.display(),
                        line_idx + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    tags.sort();
    tags
}

fn read_agents_md_excerpt(crate_dir: &Path) -> Option<String> {
    let agents_path = crate_dir.join("AGENTS.md");
    if agents_path.is_file() {
        if let Ok(content) = fs::read_to_string(&agents_path) {
            let lines: Vec<&str> = content.lines().take(40).collect();
            return Some(lines.join("\n"));
        }
    }
    None
}

fn parse_capabilities_info(root: &Path, crate_name: &str) -> (String, String) {
    let cap_path = root.join("capabilities.toml");
    if cap_path.is_file() {
        if let Ok(content) = fs::read_to_string(&cap_path) {
            if let Ok(val) = toml::from_str::<toml::Value>(&content) {
                if let Some(crate_block) = val
                    .get("crates")
                    .and_then(|c| c.get(crate_name))
                    .and_then(|b| b.as_table())
                {
                    let ring = crate_block
                        .get("ring")
                        .and_then(|r| r.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let maturity = crate_block
                        .get("maturity")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    return (ring, maturity);
                }
            }
        }
    }
    ("Unknown".to_string(), "Unknown".to_string())
}

pub fn run_crate_context(
    crate_name: &str,
    format: OutputFormat,
    output: Option<&Path>,
) -> Result<String, String> {
    let root = crate::find_root_dir();
    let crate_dir = root.join("crates").join(crate_name);

    if !crate_dir.exists() && !root.join(crate_name).exists() {
        return Err(format!(
            "Crate-Verzeichnis für '{}' existiert nicht.",
            crate_name
        ));
    }

    let actual_crate_dir = if crate_dir.exists() {
        crate_dir
    } else {
        root.join(crate_name)
    };

    let (ring, maturity) = parse_capabilities_info(&root, crate_name);
    let loc = count_crate_loc(&actual_crate_dir);
    let declared_dependencies = extract_declared_contextra_deps(&actual_crate_dir);
    let open_ai_tags = scan_crate_ai_tags(&root, &actual_crate_dir);
    let agents_md_excerpt = read_agents_md_excerpt(&actual_crate_dir);

    let relative_crate_path = actual_crate_dir
        .strip_prefix(&root)
        .unwrap_or(&actual_crate_dir)
        .display()
        .to_string();

    let recent_commits = Command::new("git")
        .current_dir(&root)
        .args([
            "log",
            "-5",
            "--format=%H|%aI|%s",
            "--",
            &relative_crate_path,
        ])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let data = CrateContextData {
        crate_name: crate_name.to_string(),
        ring,
        maturity,
        loc,
        declared_dependencies,
        open_ai_tags,
        recent_commits,
        agents_md_excerpt,
    };

    let result_str = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Fehler bei JSON-Serialisierung: {}", e))?,
        OutputFormat::Markdown => {
            let mut md = String::new();
            md.push_str(&format!("# Crate Context: {}\n\n", data.crate_name));
            md.push_str(&format!("- **Ring:** {}\n", data.ring));
            md.push_str(&format!("- **Maturity:** {}\n", data.maturity));
            md.push_str(&format!("- **Lines of Code (LOC):** {}\n\n", data.loc));

            md.push_str("## Declared Contextra Dependencies\n");
            if data.declared_dependencies.is_empty() {
                md.push_str("*(None)*\n\n");
            } else {
                for dep in &data.declared_dependencies {
                    md.push_str(&format!("- `{}`\n", dep));
                }
                md.push('\n');
            }

            md.push_str("## Open AI Tags\n");
            if data.open_ai_tags.is_empty() {
                md.push_str("*(None)*\n\n");
            } else {
                for tag in &data.open_ai_tags {
                    md.push_str(&format!("- `{}`\n", tag));
                }
                md.push('\n');
            }

            md.push_str("## Recent Commits\n");
            if data.recent_commits.is_empty() {
                md.push_str("*(None)*\n\n");
            } else {
                for commit in &data.recent_commits {
                    md.push_str(&format!("- `{}`\n", commit));
                }
                md.push('\n');
            }

            md.push_str("## AGENTS.md Excerpt\n");
            match &data.agents_md_excerpt {
                Some(excerpt) if !excerpt.trim().is_empty() => {
                    md.push_str("```markdown\n");
                    md.push_str(excerpt);
                    md.push_str("\n```\n\n");
                }
                _ => {
                    md.push_str("*(None)*\n\n");
                }
            }

            md
        }
    };

    if let Some(out_path) = output {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Kann Verzeichnis {} nicht erstellen: {}",
                    parent.display(),
                    e
                )
            })?;
        }
        fs::write(out_path, &result_str)
            .map_err(|e| format!("Kann Datei {} nicht schreiben: {}", out_path.display(), e))?;
    }

    Ok(result_str)
}
