//! Module: feature_matrix
//! Reads `[features]` from each `crates/*/Cargo.toml` using `toml::from_str`.
//! Classifies features as `On` (default), `Off` (non-default), or `Dev` (dev-/test- related).
//! Runs `cargo check -p <crate> --no-default-features --features <feature>` (or `cargo test` if `test=true`).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureDefault {
    On,
    Off,
    Dev,
}

impl FeatureDefault {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeatureDefault::On => "On",
            FeatureDefault::Off => "Off",
            FeatureDefault::Dev => "Dev",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureEntry {
    pub crate_name: String,
    pub feature: String,
    pub default: FeatureDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureCheckResult {
    pub entry: FeatureEntry,
    pub passed: bool,
    pub duration_secs: u64,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct CargoTomlPartial {
    package: Option<PackagePartial>,
    features: Option<BTreeMap<String, Vec<String>>>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Deserialize)]
struct PackagePartial {
    name: String,
}

fn find_root_dir() -> PathBuf {
    if let Ok(cargo_manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(cargo_manifest);
        if path.file_name().and_then(|s| s.to_str()) == Some("xtask") {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
        return path;
    }
    let mut curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if curr.join("Cargo.toml").exists() && curr.join("capabilities.toml").exists() {
            return curr;
        }
        if !curr.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

pub fn scan_crate_features(crates_dir: &Path) -> Result<Vec<FeatureEntry>, String> {
    let mut entries = Vec::new();
    if !crates_dir.exists() {
        return Ok(entries);
    }

    let entries_dir = fs::read_dir(crates_dir)
        .map_err(|e| format!("Failed to read crates dir {}: {}", crates_dir.display(), e))?;

    for entry in entries_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let cargo_toml_path = path.join("Cargo.toml");
            if cargo_toml_path.exists() {
                let content = fs::read_to_string(&cargo_toml_path)
                    .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path.display(), e))?;

                let parsed: CargoTomlPartial = toml::from_str(&content).map_err(|e| {
                    format!(
                        "Failed to parse TOML in {}: {}",
                        cargo_toml_path.display(),
                        e
                    )
                })?;

                let crate_name = match parsed.package {
                    Some(pkg) => pkg.name,
                    None => continue,
                };

                let features = match parsed.features {
                    Some(f) => f,
                    None => continue,
                };

                let default_features: HashSet<String> = features
                    .get("default")
                    .map(|vec| vec.iter().cloned().collect())
                    .unwrap_or_default();

                let dev_dep_names: HashSet<String> = parsed
                    .dev_dependencies
                    .map(|map| map.keys().cloned().collect())
                    .unwrap_or_default();

                for (feat_name, feat_deps) in &features {
                    if feat_name == "default" {
                        continue;
                    }

                    let is_dev_name = feat_name.starts_with("dev-")
                        || feat_name.starts_with("test-")
                        || feat_deps.iter().any(|d| {
                            let dep_clean = d.strip_prefix("dep:").unwrap_or(d);
                            dev_dep_names.contains(dep_clean)
                        });

                    let default_status = if is_dev_name {
                        FeatureDefault::Dev
                    } else if default_features.contains(feat_name) {
                        FeatureDefault::On
                    } else {
                        FeatureDefault::Off
                    };

                    entries.push(FeatureEntry {
                        crate_name: crate_name.clone(),
                        feature: feat_name.clone(),
                        default: default_status,
                    });
                }
            }
        }
    }

    entries.sort_by(|a, b| {
        a.crate_name
            .cmp(&b.crate_name)
            .then_with(|| a.feature.cmp(&b.feature))
    });

    Ok(entries)
}

pub fn run_feature_matrix(
    test: bool,
    only: Option<&str>,
) -> Result<Vec<FeatureCheckResult>, String> {
    let root = find_root_dir();
    run_feature_matrix_in_root(&root, test, only)
}

pub fn run_feature_matrix_in_root(
    root: &Path,
    test: bool,
    only: Option<&str>,
) -> Result<Vec<FeatureCheckResult>, String> {
    let crates_dir = root.join("crates");
    let entries = scan_crate_features(&crates_dir)?;

    let mut results = Vec::new();

    println!(
        "{:<25} {:<25} {:<10} {:<10} {:<10}",
        "Crate", "Feature", "Default", "Status", "Duration"
    );
    println!("{}", "-".repeat(85));

    for entry in entries {
        if let Some(crate_filter) = only {
            if entry.crate_name != crate_filter {
                continue;
            }
        }

        // Only test non-default (Off) and Dev features if requested
        let start = Instant::now();
        let subcommand = if test { "test" } else { "check" };

        let output_res = Command::new("cargo")
            .arg(subcommand)
            .arg("-p")
            .arg(&entry.crate_name)
            .arg("--no-default-features")
            .arg("--features")
            .arg(&entry.feature)
            .output();

        let duration_secs = start.elapsed().as_secs();

        match output_res {
            Ok(output) => {
                let passed = output.status.success();
                let error = if !passed {
                    Some(String::from_utf8_lossy(&output.stderr).to_string())
                } else {
                    None
                };

                let status_icon = if passed { "✅" } else { "❌" };
                println!(
                    "{:<25} {:<25} {:<10} {:<10} {}s",
                    entry.crate_name,
                    entry.feature,
                    entry.default.as_str(),
                    status_icon,
                    duration_secs
                );

                results.push(FeatureCheckResult {
                    entry,
                    passed,
                    duration_secs,
                    error,
                });
            }
            Err(e) => {
                return Err(format!("Failed to execute cargo command: {}", e));
            }
        }
    }

    Ok(results)
}
