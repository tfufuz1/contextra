#![forbid(unsafe_code)]
#![allow(
    unused_imports,
    dead_code,
    clippy::if_same_then_else,
    clippy::needless_range_loop,
    clippy::collapsible_if,
    clippy::duplicate_mod
)]

extern crate self as xtask;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::NaiveDate;
use regex::Regex;
use serde::Serialize;

pub mod agent_lifecycle;
pub mod artifact_header;
pub mod audit_integrity_check;
pub mod bench_compile;
pub mod bench_download;
pub mod check_action_pinning;
pub mod check_adr_deadlines;
pub mod check_agents_freshness;
pub mod check_agents_integrity;
pub mod check_audit_duplication;
pub mod check_audit_tool_evidence;
pub mod check_audit_verdict_independence;
pub mod check_commit_diff_integrity;
pub mod check_commit_messages;
pub mod check_compile;
pub mod check_coverage_gate;
pub mod check_doc_references;
pub mod check_duplicate_core_primitives;
pub mod check_duplicate_intent;
pub mod check_duplicate_symbols;
pub mod check_duplicate_symbols_cross_file;
pub mod check_ffi_panic_boundary;
pub mod check_flatbuffers_drift;
pub mod check_jules_context_freshness;
pub mod check_manifest_completeness;
pub mod check_max_results_unbound;
pub mod check_module_reachability;
pub mod check_mutation_score_gate;
pub mod check_nan_validation_in_hot_loop;
pub mod check_orphan_modules;
pub mod check_phantom_files;
pub mod check_placeholder_refs;
pub mod check_recall_stability;
pub mod check_result_dropped_on_io;
pub mod check_ring0_async_purity;
pub mod check_ring_capabilities_consistency;
pub mod check_ring_layering;
pub mod check_stale_tags;
pub mod check_toc_integrity;
pub mod check_toctou_trait_defaults;
pub mod check_type_registry;
pub mod check_unsafe_islands;
pub mod check_vetoes;
pub mod check_workflow_commands;
pub mod claim;
pub mod cli;
pub mod commit_health;
pub mod context_pack;
pub mod crate_context;
pub mod env_validate;
pub mod feature_matrix;
pub mod gate_check;
pub mod gates;
pub mod gen_arch_docs;
pub mod gen_feature_catalog;
pub mod gen_prompter_data;
pub mod generate_adr;
pub mod generate_diagnostics;
pub mod generate_markers;
pub mod generators;
pub mod harness;
pub mod hotspot_report;
pub mod init_audit_fix;
pub mod jules_preflight;
pub mod jules_submit_gate;
pub mod lint_unsafe_slice_bounds;
pub mod loom_run;
pub mod migrate_docid_128;
pub mod panic_inventory;
pub mod post_merge_report;
pub mod proof;
pub mod py_test;
pub mod record_mutation_score;
pub mod reproducible_build;
pub mod security_scan;
pub mod session_history;
pub mod session_init;
pub mod shell_commit_audit;
pub mod tag_health;
pub mod validate_pr_checklist;
pub mod veto_deadline_gate;
pub mod workspace_index;
pub mod workspace_verify;
pub mod ws_cache;

pub use check_jules_context_freshness::run_check_jules_context_freshness;

static ROOT_DIR_CACHE: Mutex<Option<HashMap<PathBuf, PathBuf>>> = Mutex::new(None);
static WORKSPACE_CRATES_CACHE: Mutex<Option<HashMap<PathBuf, Vec<CrateInfo>>>> = Mutex::new(None);

pub fn find_root_dir() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if let Ok(guard) = ROOT_DIR_CACHE.lock() {
        if let Some(ref cache) = *guard {
            if let Some(cached) = cache.get(&current_dir) {
                return cached.clone();
            }
        }
    }

    let root = find_root_dir_uncached(&current_dir);

    if let Ok(mut guard) = ROOT_DIR_CACHE.lock() {
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(current_dir, root.clone());
    }

    root
}

fn find_root_dir_uncached(current_dir: &Path) -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(manifest_dir);
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_path) {
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

    let mut dir = current_dir.to_path_buf();
    loop {
        let cargo_path = dir.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                if content.contains("[workspace]") && content.contains("members") {
                    return dir;
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }

    if Path::new("Cargo.toml").exists()
        && std::fs::read_to_string("Cargo.toml")
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

// -----------------------------------------------------------------------------
// Core Helper Structures and Functions
// -----------------------------------------------------------------------------

pub fn chrono_or_today_with_tags(tags: &[TagItem]) -> String {
    let mut latest_tag_date = String::new();
    for tag in tags {
        if tag.timestamp.len() >= 10 {
            let date_part = &tag.timestamp[..10];
            if date_part.chars().filter(|c| *c == '-').count() == 2
                && date_part > latest_tag_date.as_str()
            {
                latest_tag_date = date_part.to_string();
            }
        }
    }

    let mut system_today = String::new();
    if let Ok(output) = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                system_today = s;
            }
        }
    }

    if system_today.is_empty() {
        system_today = get_git_file_last_modified("WORKING_STATE.md")
            .unwrap_or_else(|_| "2026-09-02".to_string());
    }

    if !latest_tag_date.is_empty() && latest_tag_date > system_today {
        latest_tag_date
    } else {
        system_today
    }
}

pub fn chrono_or_today() -> String {
    chrono_or_today_with_tags(&[])
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TagItem {
    pub file_path: String,
    pub line_num: usize,
    pub tag_type: String,
    pub raw: String,
    pub timestamp: String,
    pub category: Option<String>,
    pub severity: Option<String>,
    pub id: Option<String>,
    pub session: Option<String>,
    pub status: Option<String>,
    pub description: String,
    pub is_resolved: bool,
}

#[derive(Debug, Default)]
pub struct CapabilitiesManifest {
    pub crates: HashMap<String, CapabilityCrateEntry>,
}

#[derive(Debug, Clone)]
pub struct CapabilityCrateEntry {
    pub ring: Option<String>,
    pub maturity: Option<String>,
    pub description: Option<String>,
}

pub fn load_capabilities_manifest(root_dir: &Path) -> CapabilitiesManifest {
    let cap_path = root_dir.join("capabilities.toml");
    let content = fs::read_to_string(&cap_path).unwrap_or_default();
    let toml_val: toml::Value =
        toml::from_str(&content).unwrap_or(toml::Value::Table(Default::default()));

    let mut manifest = CapabilitiesManifest::default();
    if let Some(crates_table) = toml_val.get("crates").and_then(|c| c.as_table()) {
        for (name, val) in crates_table {
            let ring = val
                .get("ring")
                .and_then(|r| r.as_str())
                .map(ToString::to_string);
            let maturity = val
                .get("maturity")
                .and_then(|m| m.as_str())
                .map(ToString::to_string);
            let description = val
                .get("description")
                .and_then(|d| d.as_str())
                .map(ToString::to_string);
            manifest.crates.insert(
                name.clone(),
                CapabilityCrateEntry {
                    ring,
                    maturity,
                    description,
                },
            );
        }
    }

    manifest
}

#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub path: String,
    pub layer: u8,
    pub loc: usize,
    pub status: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub ring: String,
    pub maturity: String,
}

pub fn scan_tags<P: AsRef<Path>>(root: P) -> Vec<TagItem> {
    let mut tags = Vec::new();

    let file_context_re = Regex::new(r"//\s*FILE-CONTEXT").unwrap();
    let stand_re = Regex::new(r"//\s*STAND:\s*(.+)").unwrap();
    let zweck_re = Regex::new(r"//\s*ZWECK:\s*(.+)").unwrap();
    let tag_re = Regex::new(
        r"//\s*(ANCHOR|DEBT|AI-TAG)\[([A-Z0-9_\-]+)\](?:\[([A-Za-z0-9_\-]+)\])?\s*(.*?)(?:\s*\(ID:\s*([A-Za-z0-9_\-]+)\))?(?:\s*\(TS:\s*([^\)]+)\))?(?:\s*\(SESSION:\s*([A-Za-z0-9_\-]+)\))?(?:\s*\(STATUS:\s*([A-Za-z0-9_\-]+)\))?$",
    )
    .unwrap();

    let root_path = root.as_ref();
    let walk_dir = walkdir::WalkDir::new(root_path);

    for entry in walk_dir
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != ".git"
                && name != ".cargo"
                && name != "node_modules"
                && name != "dist"
                && name != "build"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file()
            && (path.extension().and_then(|s| s.to_str()) == Some("rs")
                || path.extension().and_then(|s| s.to_str()) == Some("md")
                || path.extension().and_then(|s| s.to_str()) == Some("toml"))
        {
            let rel_path = path
                .strip_prefix(root_path)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if let Ok(content) = fs::read_to_string(path) {
                for (idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    if file_context_re.is_match(trimmed)
                        || stand_re.is_match(trimmed)
                        || zweck_re.is_match(trimmed)
                    {
                        let tag_type = if file_context_re.is_match(trimmed) {
                            "FILE-CONTEXT".to_string()
                        } else if stand_re.is_match(trimmed) {
                            "STAND".to_string()
                        } else {
                            "ZWECK".to_string()
                        };

                        tags.push(TagItem {
                            file_path: rel_path.clone(),
                            line_num: idx + 1,
                            tag_type,
                            raw: trimmed.to_string(),
                            timestamp: "".to_string(),
                            category: None,
                            severity: None,
                            id: None,
                            session: None,
                            status: None,
                            description: trimmed.to_string(),
                            is_resolved: false,
                        });
                        continue;
                    }

                    if let Some(caps) = tag_re.captures(trimmed) {
                        let tag_kind = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let cat_or_domain = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                        let sev = caps.get(3).map(|m| m.as_str());
                        let desc = caps.get(4).map(|m| m.as_str()).unwrap_or("").trim();
                        let id = caps.get(5).map(|m| m.as_str().to_string());
                        let ts = caps
                            .get(6)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        let session = caps.get(7).map(|m| m.as_str().to_string());
                        let status = caps.get(8).map(|m| m.as_str().to_string());

                        let is_resolved = status.as_deref() == Some("DONE")
                            || status.as_deref() == Some("RESOLVED")
                            || status.as_deref() == Some("CLOSED");

                        tags.push(TagItem {
                            file_path: rel_path.clone(),
                            line_num: idx + 1,
                            tag_type: tag_kind.to_string(),
                            raw: trimmed.to_string(),
                            timestamp: ts,
                            category: Some(cat_or_domain.to_string()),
                            severity: sev.map(|s| s.to_string()),
                            id,
                            session,
                            status,
                            description: desc.to_string(),
                            is_resolved,
                        });
                    }
                }
            }
        }
    }

    tags
}

pub fn calculate_crate_loc<P: AsRef<Path>>(dir: P) -> usize {
    let mut total = 0;
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != "tests"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(path) {
                total += content.lines().filter(|l| !l.trim().is_empty()).count();
            }
        }
    }
    total
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VisitStatus {
    Visiting,
    Visited(u8),
}

pub fn compute_crate_layers(crates: &mut [CrateInfo]) -> Result<(), String> {
    let crate_map: std::collections::HashMap<String, Vec<String>> = crates
        .iter()
        .map(|c| (c.name.clone(), c.dependencies.clone()))
        .collect();

    let mut status_map: std::collections::HashMap<String, VisitStatus> =
        std::collections::HashMap::new();
    let mut path: Vec<String> = Vec::new();

    fn get_layer(
        name: &str,
        crate_map: &std::collections::HashMap<String, Vec<String>>,
        status_map: &mut std::collections::HashMap<String, VisitStatus>,
        path: &mut Vec<String>,
    ) -> Result<u8, String> {
        if let Some(&status) = status_map.get(name) {
            match status {
                VisitStatus::Visited(layer) => return Ok(layer),
                VisitStatus::Visiting => {
                    let cycle_start = path.iter().position(|p| p == name).unwrap_or(0);
                    let mut cycle_path = path[cycle_start..].to_vec();
                    cycle_path.push(name.to_string());
                    return Err(format!(
                        "Dependency cycle detected: {}",
                        cycle_path.join(" -> ")
                    ));
                }
            }
        }

        let deps = match crate_map.get(name) {
            Some(deps) => deps,
            None => return Ok(0),
        };

        status_map.insert(name.to_string(), VisitStatus::Visiting);
        path.push(name.to_string());

        let mut max_dep_layer: Option<u8> = None;
        for dep in deps {
            if crate_map.contains_key(dep) {
                let dep_layer = get_layer(dep, crate_map, status_map, path)?;
                max_dep_layer = Some(max_dep_layer.map_or(dep_layer, |m| m.max(dep_layer)));
            }
        }

        path.pop();

        let calculated_layer = match max_dep_layer {
            None => 0,
            Some(max_l) => max_l
                .checked_add(1)
                .ok_or_else(|| format!("Layer overflow for crate '{}'", name))?,
        };

        status_map.insert(name.to_string(), VisitStatus::Visited(calculated_layer));
        Ok(calculated_layer)
    }

    let mut calculated_layers = Vec::with_capacity(crates.len());
    for c in crates.iter() {
        let layer = get_layer(&c.name, &crate_map, &mut status_map, &mut path)?;
        calculated_layers.push(layer);
    }

    for (c, layer) in crates.iter_mut().zip(calculated_layers) {
        c.layer = layer;
    }

    Ok(())
}

pub fn get_workspace_crates() -> Vec<CrateInfo> {
    let root_dir = find_root_dir();
    if let Ok(guard) = WORKSPACE_CRATES_CACHE.lock() {
        if let Some(ref cache) = *guard {
            if let Some(cached) = cache.get(&root_dir) {
                return cached.clone();
            }
        }
    }
    let capabilities = load_capabilities_manifest(&root_dir);
    let crates = get_workspace_crates_from_root(&root_dir, &capabilities);
    if let Ok(mut guard) = WORKSPACE_CRATES_CACHE.lock() {
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(root_dir, crates.clone());
    }
    crates
}

pub fn get_workspace_crates_from_root(
    root_dir: &Path,
    capabilities: &CapabilitiesManifest,
) -> Vec<CrateInfo> {
    let cargo_path = root_dir.join("Cargo.toml");
    let root_cargo = fs::read_to_string(&cargo_path).unwrap_or_default();
    let toml_val: toml::Value =
        toml::from_str(&root_cargo).expect("Failed to parse root Cargo.toml");

    let members = match toml_val
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
    {
        Some(m) => m,
        None => return Vec::new(),
    };

    let mut crates = Vec::new();

    for member in members {
        let path_str = match member.as_str() {
            Some(s) => s,
            None => continue,
        };
        if path_str == "xtask" {
            continue;
        }

        let crate_cargo_path = root_dir.join(path_str).join("Cargo.toml");
        if !crate_cargo_path.exists() {
            continue;
        }

        let crate_cargo_content = fs::read_to_string(&crate_cargo_path).unwrap_or_default();
        let crate_toml: toml::Value =
            toml::from_str(&crate_cargo_content).unwrap_or(toml::Value::Table(Default::default()));

        let name = crate_toml
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or_default()
            .to_string();

        let cargo_desc = crate_toml
            .get("package")
            .and_then(|p| p.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        let mut dependencies = Vec::new();
        if let Some(deps) = crate_toml.get("dependencies").and_then(|d| d.as_table()) {
            for (dep_name, dep_val) in deps {
                if dep_name.starts_with("contextra-") {
                    dependencies.push(dep_name.clone());
                } else if let Some(table) = dep_val.as_table() {
                    if table.contains_key("path") {
                        dependencies.push(dep_name.clone());
                    }
                }
            }
        }
        dependencies.sort();
        dependencies.dedup();

        let crate_dir = root_dir.join(path_str);
        let loc = calculate_crate_loc(&crate_dir);

        let (ring, maturity, description, status) =
            if let Some(entry) = capabilities.crates.get(&name) {
                let ring = entry
                    .ring
                    .clone()
                    .unwrap_or_else(|| "unklassifiziert".to_string());
                let maturity = entry
                    .maturity
                    .clone()
                    .unwrap_or_else(|| "unklassifiziert".to_string());
                let desc = entry.description.clone().unwrap_or_else(|| {
                    if cargo_desc.is_empty() {
                        "unklassifiziert".to_string()
                    } else {
                        cargo_desc.clone()
                    }
                });
                let st = match maturity.as_str() {
                    "stable" => "🟢 stable".to_string(),
                    "experimental" => "🟡 experimental".to_string(),
                    "deprecated" => "🔴 deprecated".to_string(),
                    _ => "🔴 unklassifiziert".to_string(),
                };
                (ring, maturity, desc, st)
            } else {
                (
                    "unklassifiziert".to_string(),
                    "unklassifiziert".to_string(),
                    if cargo_desc.is_empty() {
                        "unklassifiziert".to_string()
                    } else {
                        cargo_desc.clone()
                    },
                    "🔴 unklassifiziert".to_string(),
                )
            };

        let status = if name == "contextra-embed" {
            "🧊 Optional".to_string()
        } else {
            status
        };

        crates.push(CrateInfo {
            name,
            path: path_str.to_string(),
            layer: 0,
            loc,
            status,
            description,
            dependencies,
            ring,
            maturity,
        });
    }

    let _ = compute_crate_layers(&mut crates);
    crates
}

pub fn render_updated_markdown(
    original: &str,
    start_marker: &str,
    end_marker: &str,
    new_content: &str,
) -> String {
    let mut result = String::with_capacity(original.len() + new_content.len());
    let mut in_section = false;
    let mut section_found = false;

    for line in original.lines() {
        if line.contains(start_marker) {
            result.push_str(line);
            result.push('\n');
            result.push_str(new_content);
            if !new_content.ends_with('\n') {
                result.push('\n');
            }
            in_section = true;
            section_found = true;
            continue;
        }

        if in_section {
            if line.contains(end_marker) {
                in_section = false;
                result.push_str(line);
                result.push('\n');
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !section_found {
        result.clear();
        result.push_str(original);
        if !original.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(start_marker);
        result.push('\n');
        result.push_str(new_content);
        if !new_content.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(end_marker);
        result.push('\n');
    }

    result
}

pub fn update_markdown_section(
    file_path: &Path,
    start_marker: &str,
    end_marker: &str,
    new_content: &str,
) -> Result<bool, String> {
    let original = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let updated = render_updated_markdown(&original, start_marker, end_marker, new_content);

    if original != updated {
        fs::write(file_path, updated).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn generate_session_continuity_section(tags: &[TagItem]) -> String {
    let root = find_root_dir();
    generate_session_continuity_section_from_root(&root, tags)
}

pub fn generate_session_continuity_section_from_root(root_path: &Path, tags: &[TagItem]) -> String {
    let mut section = String::new();
    let current_session_id =
        env::var("JULES_SESSION_ID").unwrap_or_else(|_| "2c814094".to_string());
    section.push_str(&format!(
        "- **Standard-Session-ID:** `{}` (automatische Kontinuität, falls kein Override per Flag)\n",
        current_session_id
    ));

    let active_tags: Vec<&TagItem> = tags
        .iter()
        .filter(|t| {
            !t.is_resolved
                && (t.tag_type == "DEBT" || t.tag_type == "ANCHOR" || t.tag_type == "AI-TAG")
        })
        .collect();

    section.push_str(&format!(
        "- **Offene contextra_tags (DEBT/ANCHOR/AI-TAG):** {} (siehe Tabelle unten)\n",
        active_tags.len()
    ));

    let claims_path = root_path.join(".jules/claims.toml");
    if claims_path.exists() {
        if let Ok(content) = fs::read_to_string(&claims_path) {
            let claim_count = content
                .lines()
                .filter(|l| l.trim().starts_with("[[claim]]"))
                .count();
            section.push_str(&format!(
                "- **Aktive Task-Claims (.jules/claims.toml):** {}\n",
                claim_count
            ));
        }
    } else {
        section.push_str("- **Aktive Task-Claims (.jules/claims.toml):** 0\n");
    }

    section
}

pub fn tokenize_title(title: &str) -> std::collections::HashSet<String> {
    title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_string())
        .collect()
}

pub fn jaccard_similarity(
    s1: &std::collections::HashSet<String>,
    s2: &std::collections::HashSet<String>,
) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    let intersection = s1.intersection(s2).count();
    let union = s1.union(s2).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

pub fn extract_scope(title: &str) -> String {
    if let Some(start) = title.find('(') {
        if let Some(end) = title.find(')') {
            if start < end {
                return title[start + 1..end].to_lowercase();
            }
        }
    }
    "allgemein".to_string()
}

pub fn generate_full_working_state(tags: &[TagItem], crates: &[CrateInfo]) -> String {
    let root = find_root_dir();
    generate_full_working_state_from_root(&root, tags, crates)
}

pub fn generate_full_working_state_from_root(
    root_path: &Path,
    tags: &[TagItem],
    crates: &[CrateInfo],
) -> String {
    let mut ws = String::new();
    let today = chrono_or_today_with_tags(tags);

    ws.push_str("# Contextra — Globaler Arbeitsstand (WORKING_STATE.md)\n\n");
    ws.push_str(&format!(
        "> **Stand:** {} | **Format:** v2 (Automatisierte Generierung via `cargo xtask sync-docs`)\n\n",
        today
    ));

    ws.push_str("## 1. Systemstatus & Architektur\n");
    ws.push_str(
        "Contextra ist eine hochperformante, einbettbare Vektor- und Graph-Engine für Rust.\n\n",
    );

    ws.push_str("## 2. Session-Kontinuität\n");
    ws.push_str(&generate_session_continuity_section_from_root(
        root_path, tags,
    ));
    ws.push('\n');

    ws.push_str("## 3. Crate-Inventar & Ringe\n");
    ws.push_str(&generate_crate_inventory_section(crates));
    ws.push('\n');

    ws.push_str("## 4. Invarianten & Qualitäts-Gates\n");
    ws.push_str(&generate_invariants_table_section(crates));
    ws.push('\n');

    ws.push_str("## 5. DAG & Topologie\n");
    ws.push_str(&generate_dag_topology_section(crates));
    ws.push('\n');

    ws.push_str("## 6. KI-Tags & Schulden-Register\n");
    ws.push_str(&generate_ai_tags_section(tags));
    ws.push('\n');

    ws.push_str("## 7. Chronik & Modifikations-Historie\n");
    ws.push_str(&generate_changelog(tags));

    ws
}

pub fn generate_invariants_table_section(_crates: &[CrateInfo]) -> String {
    let mut s = String::new();
    s.push_str("| Invariante / Gate | Beschreibung | Ziel-Ebene | status |\n");
    s.push_str("| :--- | :--- | :--- | :--- |\n");
    s.push_str("| **Zero-Panic (P7)** | Kein unwrap/expect in Produktionscode | Ring 0, Ring 1 | 🟢 Aktiv |\n");
    s.push_str(
        "| **Ring-Layering** | Strikter gerichteter Azyklischer Graph | Alle Crates | 🟢 Aktiv |\n",
    );
    s.push_str("| **Async Purity** | Keine Async-Blockaden in Ring 0 | Ring 0 | 🟢 Aktiv |\n");
    s.push_str(
        "| **Determinismus** | Reproduzierbarkeit via Port-Injektion | Ring 0 | 🟢 Aktiv |\n",
    );
    s
}

pub fn generate_changelog(tags: &[TagItem]) -> String {
    let mut s = String::new();
    s.push_str("| Datum | Komponente | Beschreibung | Tag-ID | Session |\n");
    s.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

    let mut resolved_tags: Vec<&TagItem> = tags
        .iter()
        .filter(|t| t.is_resolved || t.status.as_deref() == Some("DONE"))
        .collect();

    resolved_tags.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    for t in resolved_tags.iter().take(20) {
        let date = if t.timestamp.len() >= 10 {
            &t.timestamp[..10]
        } else {
            "2026-09-02"
        };
        let tag_id = t.id.as_deref().unwrap_or("-");
        let session = t.session.as_deref().unwrap_or("-");
        s.push_str(&format!(
            "| {} | `{}` | {} | `{}` | `{}` |\n",
            date, t.file_path, t.description, tag_id, session
        ));
    }

    s
}

pub fn generate_ai_tags_section(tags: &[TagItem]) -> String {
    let mut s = String::new();
    s.push_str("| Datei | Zeile | Typ | Schweregrad | Beschreibung | Status |\n");
    s.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");

    let mut open_tags: Vec<&TagItem> = tags.iter().filter(|t| !t.is_resolved).collect();
    open_tags.sort_by_key(|t| severity_weight(t.severity.as_deref()));

    for t in open_tags.iter().take(30) {
        let sev = t.severity.as_deref().unwrap_or("INFO");
        let st = t.status.as_deref().unwrap_or("OPEN");
        s.push_str(&format!(
            "| `{}` | {} | `{}` | **{}** | {} | {} |\n",
            t.file_path, t.line_num, t.tag_type, sev, t.description, st
        ));
    }

    s
}

pub fn generate_dag_topology_section(crates: &[CrateInfo]) -> String {
    let mut s = String::new();
    let mut layers: BTreeMap<u8, Vec<&CrateInfo>> = BTreeMap::new();
    for c in crates {
        layers.entry(c.layer).or_default().push(c);
    }

    for (layer, layer_crates) in layers {
        s.push_str(&format!("### Layer {} (Rang {})\n", layer, layer));
        for c in layer_crates {
            let deps = if c.dependencies.is_empty() {
                "keine".to_string()
            } else {
                c.dependencies.join(", ")
            };
            s.push_str(&format!(
                "- **`{}`** (Ring: `{}`) -> Abhaengigkeiten: {}\n",
                c.name, c.ring, deps
            ));
        }
        s.push('\n');
    }

    s
}

pub fn format_loc(loc: usize) -> String {
    if loc >= 1000 {
        format!("{:.1}k", loc as f64 / 1000.0)
    } else {
        loc.to_string()
    }
}

pub fn generate_crate_inventory_section(crates: &[CrateInfo]) -> String {
    let mut s = String::new();
    s.push_str("| Crate | Pfad | Layer | LOC | Status | Ring | Reife | Beschreibung |\n");
    s.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");

    for c in crates {
        s.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | `{}` | `{}` | {} |\n",
            c.name,
            c.path,
            c.layer,
            format_loc(c.loc),
            c.status,
            c.ring,
            c.maturity,
            c.description
        ));
    }

    s
}

pub fn severity_weight(sev: Option<&str>) -> usize {
    match sev {
        Some("CRITICAL") | Some("FATAL") => 0,
        Some("HIGH") | Some("ERROR") => 1,
        Some("MEDIUM") | Some("WARNING") => 2,
        Some("LOW") | Some("INFO") => 3,
        _ => 4,
    }
}

pub fn run_sync_docs(check_only: bool) -> bool {
    println!("=== xtask sync-docs ===");
    let root = find_root_dir();

    let tags = scan_tags(&root);
    let crates = get_workspace_crates();

    let new_ws = generate_full_working_state(&tags, &crates);
    let ws_path = root.join("WORKING_STATE.md");

    if check_only {
        if ws_path.exists() {
            let current = fs::read_to_string(&ws_path).unwrap_or_default();
            if current.trim() != new_ws.trim() {
                eprintln!("❌ WORKING_STATE.md ist nicht synchron! Bitte `cargo xtask sync-docs` ausfuehren.");
                return false;
            }
        }
        println!("✅ WORKING_STATE.md ist synchron.");
        true
    } else {
        if let Err(e) = fs::write(&ws_path, new_ws) {
            eprintln!("❌ Fehler beim Schreiben von WORKING_STATE.md: {}", e);
            return false;
        }
        println!("✅ WORKING_STATE.md verfiziert und aktualisiert.");
        true
    }
}

pub fn check_readme_crate_count(actual_count: usize, readme_content: &str) -> bool {
    let re = Regex::new(r"Insgesamt\s+(\d+)\s+Crates").unwrap();
    if let Some(caps) = re.captures(readme_content) {
        if let Ok(claimed) = caps[1].parse::<usize>() {
            return claimed == actual_count;
        }
    }
    true
}

pub fn check_crate_agents(crates: &[CrateInfo], root_dir: &Path) -> bool {
    let mut ok = true;
    for c in crates {
        let agents_path = root_dir.join(&c.path).join("AGENTS.md");
        if !agents_path.exists() {
            println!("⚠️ Crate `{}` hat keine eigene AGENTS.md", c.name);
            ok = false;
        }
    }
    ok
}

pub fn check_no_orphan_adr_files(root_dir: &Path) -> bool {
    let adr_dir = root_dir.join("docs").join("decisions");
    if !adr_dir.exists() {
        return true;
    }

    let mut ok = true;
    for entry in walkdir::WalkDir::new(adr_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name != "README.md" && !name.starts_with("ADR-") {
                eprintln!("❌ Unbekannte ADR-Datei ohne ADR-Präfix: {}", name);
                ok = false;
            }
        }
    }
    ok
}

pub fn check_adr_consistency_dir(root_dir: &Path) -> bool {
    let adr_dir = root_dir.join("docs").join("decisions");
    if !adr_dir.exists() {
        return true;
    }

    let mut ok = true;
    for entry in walkdir::WalkDir::new(adr_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(path) {
                if !check_adr_consistency(&content) {
                    eprintln!("❌ Inkonsistente ADR-Struktur in `{}`", path.display());
                    ok = false;
                }
            }
        }
    }
    ok
}

pub fn check_adr_consistency(decisions: &str) -> bool {
    let has_status = decisions.contains("# Status")
        || decisions.contains("## Status")
        || decisions.contains("Status:");
    let has_kontext = decisions.contains("# Kontext")
        || decisions.contains("## Kontext")
        || decisions.contains("Kontext:");
    let has_entscheidung = decisions.contains("# Entscheidung")
        || decisions.contains("## Entscheidung")
        || decisions.contains("Entscheidung:");

    has_status && has_kontext && has_entscheidung
}

pub fn get_changed_rs_files_from_git_diff() -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Git diff Befehl fehlgeschlagen".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files = stdout
        .lines()
        .filter(|l| l.ends_with(".rs"))
        .map(ToString::to_string)
        .collect();

    Ok(files)
}

pub fn get_git_file_last_modified(file_path: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=short", file_path])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }

    Ok("2026-09-02".to_string())
}

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let len_a = a.chars().count();
    let len_b = b.chars().count();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        matrix[i][0] = i;
    }
    for j in 0..=len_b {
        matrix[0][j] = j;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len_a][len_b]
}

pub struct SentenceItem {
    pub text: String,
    pub index: usize,
}

pub fn split_into_sentences(content: &str) -> Vec<SentenceItem> {
    let mut sentences = Vec::new();

    let mut start = 0;
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if (c == '.' || c == '!' || c == '?') && i + 1 < len && chars[i + 1].is_whitespace() {
            let s = content[start..=i].trim();
            if !s.is_empty() {
                sentences.push(SentenceItem {
                    text: s.to_string(),
                    index: sentences.len(),
                });
            }
            start = i + 1;
        }
        i += 1;
    }

    if start < content.len() {
        let s = content[start..].trim();
        if !s.is_empty() {
            sentences.push(SentenceItem {
                text: s.to_string(),
                index: sentences.len(),
            });
        }
    }

    sentences
}

pub fn sentences_are_similar(s1: &SentenceItem, s2: &SentenceItem) -> bool {
    if s1.text == s2.text {
        return true;
    }
    let dist = levenshtein_distance(&s1.text, &s2.text);
    let max_len = s1.text.len().max(s2.text.len());
    if max_len == 0 {
        return true;
    }
    (dist as f64 / max_len as f64) < 0.2
}

pub fn check_sentence_overlap(agents_content: &str, jules_content: &str) -> bool {
    let s1 = split_into_sentences(agents_content);
    let s2 = split_into_sentences(jules_content);

    for item1 in &s1 {
        for item2 in &s2 {
            if sentences_are_similar(item1, item2) {
                return true;
            }
        }
    }

    false
}

pub fn run_check_consistency() -> bool {
    println!("=== xtask check-consistency ===");
    let root = find_root_dir();
    let mut ok = true;

    if !check_no_orphan_adr_files(&root) {
        ok = false;
    }

    if !check_adr_consistency_dir(&root) {
        ok = false;
    }

    if ok {
        println!("✅ check-consistency bestanden.");
    } else {
        eprintln!("❌ check-consistency fehlgeschlagen.");
    }

    ok
}

#[derive(Debug, PartialEq, Eq)]
pub enum TagDateStatus {
    Valid,
    Stale(String),
    Malformed(String),
}

pub fn validate_working_state_tag_date(date_str: &str, today: NaiveDate) -> TagDateStatus {
    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(parsed) => {
            let num_days = (today - parsed).num_days();
            if num_days > 90 {
                TagDateStatus::Stale(date_str.to_string())
            } else {
                TagDateStatus::Valid
            }
        }
        Err(_) => TagDateStatus::Malformed(date_str.to_string()),
    }
}

pub fn validate_tags_items(tags: &[TagItem]) -> bool {
    let mut ok = true;
    for t in tags {
        if t.timestamp.is_empty() {
            eprintln!("❌ Tag ohne Timestamp in `{}:{}`", t.file_path, t.line_num);
            ok = false;
        }
    }
    ok
}

pub fn run_validate_tags(_fix: bool) -> bool {
    println!("=== xtask validate-tags ===");
    let root = find_root_dir();
    let tags = scan_tags(&root);

    let ok = validate_tags_items(&tags);
    if ok {
        println!("✅ validate-tags bestanden.");
    } else {
        eprintln!("❌ validate-tags fehlgeschlagen.");
    }

    ok
}

pub struct DagViolation {
    pub from_crate: String,
    pub from_layer: u8,
    pub to_crate: String,
    pub to_layer: u8,
}

pub fn check_dag_layer_violations(crates: &[CrateInfo]) -> Vec<DagViolation> {
    let layer_map: std::collections::HashMap<&str, u8> =
        crates.iter().map(|c| (c.name.as_str(), c.layer)).collect();

    let mut violations = Vec::new();
    for c in crates {
        for dep in &c.dependencies {
            if let Some(&dep_layer) = layer_map.get(dep.as_str()) {
                if dep_layer > c.layer {
                    violations.push(DagViolation {
                        from_crate: c.name.clone(),
                        from_layer: c.layer,
                        to_crate: dep.clone(),
                        to_layer: dep_layer,
                    });
                }
            }
        }
    }

    violations
}

pub fn run_check_dag() -> bool {
    println!("=== xtask check-dag ===");
    let crates = get_workspace_crates();
    let violations = check_dag_layer_violations(&crates);

    let known_exceptions: &[(&str, &str)] = &[];

    let mut untracked = Vec::new();
    for v in &violations {
        let is_known = known_exceptions
            .iter()
            .any(|(f, t)| *f == v.from_crate && *t == v.to_crate);
        if !is_known {
            eprintln!(
                "❌ DAG-Verstoß: {} (Layer {}) → {} (Layer {})",
                v.from_crate, v.from_layer, v.to_crate, v.to_layer
            );
            untracked.push(v);
        } else {
            println!(
                "⚠️ Bekannter DAG-Verstoß (dokumentiert): {} → {}",
                v.from_crate, v.to_crate
            );
        }
    }

    if untracked.is_empty() {
        println!("=== xtask check-dag PASSED ===");
        true
    } else {
        eprintln!(
            "=== xtask check-dag FAILED: {} undokumentierte Verstöße ===",
            untracked.len()
        );
        false
    }
}

pub fn run_check_review_coverage(tags: &[TagItem]) -> bool {
    println!("=== Running xtask check-review-coverage ===");
    let done_anchors: Vec<&TagItem> = tags
        .iter()
        .filter(|t| t.tag_type == "ANCHOR" && t.status.as_deref() == Some("DONE"))
        .collect();

    let mut failed = false;

    for anchor in &done_anchors {
        let is_new_tag = anchor.session.is_some()
            || (anchor.timestamp.as_str() >= "2026-08-29"
                && anchor.timestamp.as_str() != "2026-08-29T00:00:00Z");

        if !is_new_tag {
            continue;
        }

        let anchor_id = match &anchor.id {
            Some(id) => id,
            None => {
                eprintln!(
                    "❌ ANCHOR at {}:{} marked DONE without an ID!",
                    anchor.file_path, anchor.line_num
                );
                failed = true;
                continue;
            }
        };

        let is_unsafe_file = anchor.file_path.contains("distance.rs")
            || anchor.file_path.contains("diskann.rs")
            || anchor.file_path.contains("persistence.rs")
            || anchor.file_path.contains("anti_tamper.rs");

        let is_security = anchor.raw.contains("SECURITY");

        let required_passes = if is_unsafe_file || is_security { 3 } else { 2 };

        let matching_passes: Vec<&TagItem> = tags
            .iter()
            .filter(|t| {
                t.tag_type == "REVIEW-PASS"
                    && t.status.as_deref() == Some("PASS")
                    && t.id.as_deref() == Some(anchor_id.as_str())
            })
            .collect();

        let mut distinct_sessions = std::collections::HashSet::new();

        for pass in matching_passes {
            if let Some(sess) = &pass.session {
                if anchor.session.as_ref() != Some(sess) {
                    distinct_sessions.insert(sess.clone());
                }
            }
        }

        if distinct_sessions.len() < required_passes {
            eprintln!(
                "❌ ANCHOR '{}' in {}:{} has {}/{} required independent REVIEW-PASSes (sessions: {:?})",
                anchor_id,
                anchor.file_path,
                anchor.line_num,
                distinct_sessions.len(),
                required_passes,
                distinct_sessions
            );
            failed = true;
        } else {
            println!(
                "✅ ANCHOR '{}' in {}:{} passed review coverage ({}/{} independent passes)",
                anchor_id,
                anchor.file_path,
                anchor.line_num,
                distinct_sessions.len(),
                required_passes
            );
        }
    }

    if failed {
        eprintln!("=== xtask check-review-coverage FAILED ===");
        false
    } else {
        println!("=== xtask check-review-coverage PASSED ===");
        true
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ContextTagNdjson<'a> {
    #[serde(rename = "crate")]
    pub krate: &'a str,
    pub file: &'a str,
    pub line: usize,
    pub tag_type: &'a str,
    pub domain: Option<&'a str>,
    pub severity: Option<&'a str>,
    pub id: Option<&'a str>,
    pub status: Option<&'a str>,
    pub ts: &'a str,
    pub session: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct ContextTagFilter {
    pub krate: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
}

pub fn extract_crate_name(file_path: &str) -> &str {
    let p = Path::new(file_path);
    let mut components = p.components();
    if let Some(first) = components.next() {
        if first.as_os_str() == "crates" {
            if let Some(second) = components.next() {
                return second.as_os_str().to_str().unwrap_or("");
            }
        } else {
            return first.as_os_str().to_str().unwrap_or("");
        }
    }
    ""
}

pub fn parse_context_tag_args(args: &[String]) -> ContextTagFilter {
    let mut filter = ContextTagFilter::default();
    for arg in args {
        if let Some(val) = arg.strip_prefix("--crate=") {
            filter.krate = Some(val.to_string());
        } else if let Some(val) = arg.strip_prefix("--severity=") {
            filter.severity = Some(val.to_string());
        } else if let Some(val) = arg.strip_prefix("--status=") {
            filter.status = Some(val.to_string());
        }
    }
    filter
}

pub fn filter_tags<'a>(
    tags: &'a [TagItem],
    filter: &ContextTagFilter,
) -> Vec<ContextTagNdjson<'a>> {
    tags.iter()
        .filter_map(|t| {
            let krate = extract_crate_name(&t.file_path);
            if let Some(ref req_crate) = filter.krate {
                if !krate.eq_ignore_ascii_case(req_crate) {
                    return None;
                }
            }
            if let Some(ref req_sev) = filter.severity {
                match &t.severity {
                    Some(sev) if sev.eq_ignore_ascii_case(req_sev) => {}
                    _ => return None,
                }
            }
            if let Some(ref req_status) = filter.status {
                match &t.status {
                    Some(st) if st.eq_ignore_ascii_case(req_status) => {}
                    _ => return None,
                }
            }
            Some(ContextTagNdjson {
                krate,
                file: &t.file_path,
                line: t.line_num,
                tag_type: &t.tag_type,
                domain: t.category.as_deref(),
                severity: t.severity.as_deref(),
                id: t.id.as_deref(),
                status: t.status.as_deref(),
                ts: &t.timestamp,
                session: t.session.as_deref(),
            })
        })
        .collect()
}

pub fn run_context_tags(tags: &[TagItem], args: &[String]) {
    let filter = parse_context_tag_args(args);
    let filtered = filter_tags(tags, &filter);
    for item in filtered {
        if let Ok(json) = serde_json::to_string(&item) {
            println!("{}", json);
        }
    }
}
