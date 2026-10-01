//! Modul zur Überprüfung der Konsistenz und Wahrheit von Bootstrap-Dokumenten (doc-truth).

use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocTruthFinding {
    pub file: String,
    pub line: usize,
    pub id: String,
    pub severity: String, // "error" | "warn" | "info"
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize)]
pub struct DocTruthJsonReport {
    pub gate: String,
    pub status: String, // "pass" | "fail" | "error" | "not_applicable"
    pub summary: String,
    pub findings: Vec<DocTruthFinding>,
}

/// Einstiegspunkt gemäß Modulvertrag.
pub fn run_doc_truth(args: &[String]) -> i32 {
    let mut root_dir: Option<PathBuf> = None;
    let mut json = false;
    let mut _strict = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--json" => {
                json = true;
            }
            "--strict" => {
                _strict = true;
            }
            _ => {}
        }
        i += 1;
    }

    let root = match root_dir {
        Some(r) => r,
        None => doc_truth_find_root(),
    };

    let mut findings = Vec::new();

    // Run checks
    doc_truth_check_paths_pub(&root, &mut findings);
    doc_truth_check_crates_pub(&root, &mut findings);
    doc_truth_check_commands_pub(&root, &mut findings);
    doc_truth_check_unsafe_islands_pub(&root, &mut findings);
    doc_truth_check_crate_table_pub(&root, &mut findings);

    // Sort findings deterministically
    findings.sort();

    let has_errors = findings.iter().any(|f| f.severity == "error");

    let status = if has_errors { "fail" } else { "pass" };

    let exit_code = if status == "fail" { 1 } else { 0 };

    if json {
        let report = DocTruthJsonReport {
            gate: "doc-truth".to_string(),
            status: status.to_string(),
            summary: format!(
                "doc-truth: {} error(s), {} warning(s), {} info(s)",
                findings.iter().filter(|f| f.severity == "error").count(),
                findings.iter().filter(|f| f.severity == "warn").count(),
                findings.iter().filter(|f| f.severity == "info").count()
            ),
            findings,
        };
        if let Ok(out) = serde_json::to_string_pretty(&report) {
            println!("{}", out);
        }
    } else {
        println!("=== xtask doc-truth ===");
        if findings.is_empty() {
            println!("✅ Alle Dokumente sind wahr und konsistent zum Quellcode.");
        } else {
            for f in &findings {
                let icon = match f.severity.as_str() {
                    "error" => "❌",
                    "warn" => "⚠️",
                    _ => "ℹ️",
                };
                println!(
                    "{} [doc-truth/{}] {}:{} — {}\n   Invariante/ADR: {}\n   Fix: {}",
                    icon, f.id, f.file, f.line, f.message, f.id, f.fix
                );
            }
            println!("\nStatus: {}", status.to_uppercase());
        }
    }

    exit_code
}

pub fn doc_truth_find_root() -> PathBuf {
    if let Ok(out) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if out.status.success() {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path_str.is_empty() {
                return PathBuf::from(path_str);
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Target governance files to scan for path/crate/command references.
fn doc_truth_target_files(root: &Path) -> Vec<(String, PathBuf, bool)> {
    let mut files = Vec::new();

    let docs = [
        "AGENTS.md",
        ".jules/SESSION_BOOTSTRAP.md",
        ".jules/JULES_CONTEXT.md",
        ".jules/COMMON_LLM_ERRORS.md",
        ".jules/SCHEDULED_AUDIT.md",
        ".jules/AUDIT_INTAKE_PROTOCOL.md",
        ".jules/ISSUE_AUTOMATION_POLICY.md",
    ];

    for doc in docs {
        let p = root.join(doc);
        if p.exists() {
            files.push((doc.to_string(), p, false)); // false = blocking/error if fails
        }
    }

    // Crate AGENTS.md (warnings only, non-blocking)
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.file_name() == "AGENTS.md" {
            if let Ok(rel) = entry.path().strip_prefix(root) {
                files.push((
                    rel.to_string_lossy().to_string(),
                    entry.path().to_path_buf(),
                    true,
                ));
            }
        }
    }

    files
}

/// Check 1a: File path backtick references exist in repo.
pub fn doc_truth_check_paths_pub(root: &Path, findings: &mut Vec<DocTruthFinding>) {
    let target_files = doc_truth_target_files(root);
    let path_re = Regex::new(r"`([^`]+)`").unwrap();

    for (rel_file, full_path, is_warn_only) in target_files {
        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_dir = full_path.parent();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            if line.contains("<!-- doc-ref-ignore -->") {
                continue;
            }

            for cap in path_re.captures_iter(line) {
                let token = cap.get(1).map_or("", |m| m.as_str()).trim();

                // Determine if token looks like a file/directory path
                if !doc_truth_is_path_candidate(token) {
                    continue;
                }

                if !doc_truth_path_exists(root, file_dir, token) {
                    findings.push(DocTruthFinding {
                        file: rel_file.clone(),
                        line: line_num,
                        id: "DOC-TRUTH-PATH".to_string(),
                        severity: if is_warn_only { "warn" } else { "error" }.to_string(),
                        message: format!(
                            "Referenzierter Pfad '{}' existiert nicht im Repository.",
                            token
                        ),
                        fix: format!(
                            "Pfade in '{}' korrigieren oder '<!-- doc-ref-ignore -->' anhängen.",
                            rel_file
                        ),
                    });
                }
            }
        }
    }
}

fn doc_truth_is_path_candidate(token: &str) -> bool {
    let trimmed = token.trim_matches(|c: char| c == '\'' || c == '"' || c == '(' || c == ')');
    if trimmed.is_empty() || trimmed.contains(' ') || trimmed.starts_with("http") {
        return false;
    }

    // Check extensions or directory structure
    if trimmed.ends_with('/') || trimmed.contains('/') {
        if trimmed == "./" || trimmed == "../" || trimmed.starts_with('/') {
            return false;
        }
        return true;
    }

    let valid_exts = [
        ".rs", ".toml", ".md", ".json", ".txt", ".yml", ".yaml", ".fbs", ".html",
    ];
    if valid_exts.iter().any(|ext| trimmed.ends_with(ext))
        && trimmed != ".rs"
        && trimmed != ".toml"
        && trimmed != ".md"
    {
        return true;
    }

    trimmed == "justfile"
}

fn doc_truth_path_exists(root: &Path, file_dir: Option<&Path>, candidate: &str) -> bool {
    let clean = candidate.trim_start_matches("./");

    // Glob/Wildcard check
    if clean.contains('*') {
        if let Some(pos) = clean.find('*') {
            let dir_part = &clean[..pos].trim_end_matches('/');
            if dir_part.is_empty() || root.join(dir_part).exists() {
                return true;
            }
            if let Some(fd) = file_dir {
                if fd.join(dir_part).exists() || fd.join("src").join(dir_part).exists() {
                    return true;
                }
            }
        }
        return false;
    }

    // Relative to doc file directory or doc file directory's src/
    if let Some(fd) = file_dir {
        if fd.join(clean).exists() || fd.join("src").join(clean).exists() {
            return true;
        }
    }

    // Direct path relative to root
    if root.join(clean).exists() {
        return true;
    }

    // Relative to crates/ or benchmarks/
    if root.join("crates").join(clean).exists() || root.join("benchmarks").join(clean).exists() {
        return true;
    }

    false
}

/// Check 1b: Crate names exist in workspace.
pub fn doc_truth_check_crates_pub(root: &Path, findings: &mut Vec<DocTruthFinding>) {
    let workspace_crates = doc_truth_get_workspace_crates(root);
    let target_files = doc_truth_target_files(root);
    let crate_re = Regex::new(r"\bcontextra-[a-z0-9-]+\b").unwrap();

    for (rel_file, full_path, is_warn_only) in target_files {
        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            if line.contains("<!-- crate-ref-ignore -->") {
                continue;
            }

            for cap in crate_re.captures_iter(line) {
                let crate_name = cap.get(0).unwrap().as_str();

                if !workspace_crates.contains(crate_name) {
                    findings.push(DocTruthFinding {
                        file: rel_file.clone(),
                        line: line_num,
                        id: "DOC-TRUTH-CRATE".to_string(),
                        severity: if is_warn_only { "warn" } else { "error" }.to_string(),
                        message: format!("Referenzierter Crate-Name '{}' existiert nicht im Workspace.", crate_name),
                        fix: format!("Crate-Namen in '{}' korrigieren oder '<!-- crate-ref-ignore -->' anhängen.", rel_file),
                    });
                }
            }
        }
    }
}

fn doc_truth_get_workspace_crates(root: &Path) -> HashSet<String> {
    let mut crates = HashSet::new();

    // Read root Cargo.toml members
    if let Ok(content) = fs::read_to_string(root.join("Cargo.toml")) {
        let member_re = Regex::new(r#""(?:crates|benchmarks)/([a-z0-9-]+)""#).unwrap();
        for cap in member_re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                crates.insert(m.as_str().to_string());
            }
        }
    }

    // Read capabilities.toml
    if let Ok(content) = fs::read_to_string(root.join("capabilities.toml")) {
        let cap_re = Regex::new(r"\[crates\.([a-z0-9-]+)\]").unwrap();
        for cap in cap_re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                crates.insert(m.as_str().to_string());
            }
        }
    }

    // Always include xtask
    crates.insert("xtask".to_string());
    crates
}

/// Check 1c: cargo xtask / cargo run commands point to existing commands.
pub fn doc_truth_check_commands_pub(root: &Path, findings: &mut Vec<DocTruthFinding>) {
    let valid_commands = doc_truth_get_xtask_commands(root);
    let target_files = doc_truth_target_files(root);

    let cmd_re = Regex::new(
        r"(?:cargo\s+xtask|cargo\s+run\s+--manifest-path\s+xtask/Cargo\.toml\s+--)\s+([a-z0-9-]+)",
    )
    .unwrap();

    for (rel_file, full_path, is_warn_only) in target_files {
        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let is_planned = line.contains("<!-- harness:planned -->");

            for cap in cmd_re.captures_iter(line) {
                let cmd_name = cap.get(1).unwrap().as_str();

                if !valid_commands.contains(cmd_name) {
                    if is_planned {
                        findings.push(DocTruthFinding {
                            file: rel_file.clone(),
                            line: line_num,
                            id: "DOC-TRUTH-CMD-PLANNED".to_string(),
                            severity: "info".to_string(),
                            message: format!(
                                "Geplantes xtask Kommando '{}' ist noch nicht implementiert.",
                                cmd_name
                            ),
                            fix: "Keine Aktion erforderlich (geplantes Kommando).".to_string(),
                        });
                    } else {
                        findings.push(DocTruthFinding {
                            file: rel_file.clone(),
                            line: line_num,
                            id: "DOC-TRUTH-CMD".to_string(),
                            severity: if is_warn_only { "warn" } else { "error" }.to_string(),
                            message: format!("Kommando 'cargo xtask {}' existiert nicht in xtask.", cmd_name),
                            fix: "Kommando-Namen korrigieren oder '<!-- harness:planned -->' anhängen.".to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn doc_truth_get_xtask_commands(root: &Path) -> HashSet<String> {
    let mut cmds = HashSet::new();

    // 1. Scan xtask/src/main.rs for match arms
    let main_rs = root.join("xtask/src/main.rs");
    if let Ok(content) = fs::read_to_string(&main_rs) {
        let arm_re1 = Regex::new(r#""([a-z0-9-]+)"\s*=>"#).unwrap();
        let arm_re2 = Regex::new(r#"Some\("([a-z0-9-]+)"\)"#).unwrap();

        for cap in arm_re1.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                cmds.insert(m.as_str().to_string());
            }
        }
        for cap in arm_re2.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                cmds.insert(m.as_str().to_string());
            }
        }
    }

    // 2. Scan xtask/src/harness/*.rs
    let harness_dir = root.join("xtask/src/harness");
    if harness_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&harness_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let cmd_name = stem.replace('_', "-");
                        cmds.insert(cmd_name);
                    }
                }
            }
        }
    }

    cmds
}

/// Check 1d: Unsafe consistency across capabilities.toml, AGENTS.md, and source code.
pub fn doc_truth_check_unsafe_islands_pub(root: &Path, findings: &mut Vec<DocTruthFinding>) {
    let island_capabilities = doc_truth_get_capabilities_unsafe_islands(root);
    let island_agents = doc_truth_get_agents_unsafe_islands(root);
    let island_actual_code = doc_truth_get_actual_unsafe_crates(root);

    // Verify capabilities.toml == AGENTS.md and actual unsafe crates are subset of declared islands
    if island_capabilities != island_agents || !island_actual_code.is_subset(&island_capabilities) {
        let msg = format!(
            "Inkonsistenz bei Unsafe-Inseln!\n  capabilities.toml: {:?}\n  AGENTS.md: {:?}\n  Code Scan: {:?}",
            island_capabilities, island_agents, island_actual_code
        );
        findings.push(DocTruthFinding {
            file: "capabilities.toml / AGENTS.md".to_string(),
            line: 1,
            id: "DOC-TRUTH-UNSAFE-ISLANDS".to_string(),
            severity: "error".to_string(),
            message: msg,
            fix: "AGENTS.md, capabilities.toml und Quellcode an die tatsächliche Unsafe-Isolierung angleichen.".to_string(),
        });
    }

    // Check that non-island crates enforce forbid/deny unsafe_code
    let workspace_crates = doc_truth_get_workspace_crate_paths(root);
    for (crate_name, crate_path) in workspace_crates {
        if island_capabilities.contains(&crate_name) {
            continue;
        }

        let lib_rs = crate_path.join("src/lib.rs");
        let main_rs = crate_path.join("src/main.rs");

        let mut enforces_forbid = false;
        for entry_file in &[lib_rs, main_rs] {
            if entry_file.exists() {
                if let Ok(content) = fs::read_to_string(entry_file) {
                    if content.contains("#![forbid(unsafe_code)]")
                        || content.contains("#![deny(unsafe_code)]")
                    {
                        enforces_forbid = true;
                        break;
                    }
                }
            }
        }

        if !enforces_forbid {
            findings.push(DocTruthFinding {
                file: format!("{}/src/lib.rs", crate_path.strip_prefix(root).unwrap_or(&crate_path).display()),
                line: 1,
                id: "DOC-TRUTH-FORBID-UNSAFE".to_string(),
                severity: "error".to_string(),
                message: format!("Crate '{}' ist keine Unsafe-Insel, enthält aber kein '#![forbid(unsafe_code)]'.", crate_name),
                fix: format!("'#![forbid(unsafe_code)]' am Anfang von {}/src/lib.rs einfügen.", crate_path.display()),
            });
        }
    }
}

fn doc_truth_get_capabilities_unsafe_islands(root: &Path) -> BTreeSet<String> {
    let mut islands = BTreeSet::new();
    let cap_path = root.join("capabilities.toml");

    if let Ok(content) = fs::read_to_string(&cap_path) {
        let mut current_crate = String::new();
        for line in content.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("[crates.") {
                current_crate = line_trimmed
                    .trim_start_matches("[crates.")
                    .trim_end_matches(']')
                    .to_string();
            } else if line_trimmed == "unsafe_island = true" && !current_crate.is_empty() {
                islands.insert(current_crate.clone());
            }
        }
    }

    islands
}

fn doc_truth_get_agents_unsafe_islands(root: &Path) -> BTreeSet<String> {
    let mut islands = BTreeSet::new();
    let agents_path = root.join("AGENTS.md");

    if let Ok(content) = fs::read_to_string(&agents_path) {
        let crate_re = Regex::new(r"contextra-[a-z0-9-]+").unwrap();
        for line in content.lines() {
            if line.contains("Unsafe-Isolierung") || line.contains("Unsafe-Inseln") {
                for cap in crate_re.captures_iter(line) {
                    islands.insert(cap.get(0).unwrap().as_str().to_string());
                }
            }
        }
    }

    islands
}

fn doc_truth_get_workspace_crate_paths(root: &Path) -> Vec<(String, PathBuf)> {
    let mut crate_paths = Vec::new();

    let crates_dir = root.join("crates");
    if crates_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&crates_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() && path.join("Cargo.toml").exists() {
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    crate_paths.push((name, path));
                }
            }
        }
    }

    crate_paths
}

fn doc_truth_get_actual_unsafe_crates(root: &Path) -> BTreeSet<String> {
    let mut unsafe_crates = BTreeSet::new();
    let crate_paths = doc_truth_get_workspace_crate_paths(root);

    for (crate_name, crate_path) in crate_paths {
        let src_dir = crate_path.join("src");
        if !src_dir.is_dir() {
            continue;
        }

        let mut crate_has_unsafe = false;
        for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file()
                && p.extension().and_then(|s| s.to_str()) == Some("rs")
                && doc_truth_file_has_unsafe(p)
            {
                crate_has_unsafe = true;
                break;
            }
        }

        if crate_has_unsafe {
            unsafe_crates.insert(crate_name);
        }
    }

    unsafe_crates
}

fn doc_truth_file_has_unsafe(file_path: &Path) -> bool {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let syn_file = match syn::parse_file(&content) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut visitor = DocTruthUnsafeVisitor { has_unsafe: false };
    visitor.visit_file(&syn_file);
    visitor.has_unsafe
}

struct DocTruthUnsafeVisitor {
    has_unsafe: bool,
}

impl<'ast> Visit<'ast> for DocTruthUnsafeVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        for attr in &node.attrs {
            if doc_truth_is_cfg_test(attr) {
                return;
            }
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        for attr in &node.attrs {
            if doc_truth_is_cfg_test(attr) {
                return;
            }
        }
        if node.sig.unsafety.is_some() {
            self.has_unsafe = true;
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.unsafety.is_some() {
            self.has_unsafe = true;
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if node.unsafety.is_some() {
            self.has_unsafe = true;
        }
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_expr_unsafe(&mut self, _node: &'ast syn::ExprUnsafe) {
        self.has_unsafe = true;
    }
}

fn doc_truth_is_cfg_test(attr: &syn::Attribute) -> bool {
    if attr.path().is_ident("cfg") {
        if let Ok(meta) = attr.parse_args::<syn::Ident>() {
            return meta == "test";
        }
    }
    false
}

/// Check 1e: Crate table in .jules/JULES_CONTEXT.md matches workspace members.
pub fn doc_truth_check_crate_table_pub(root: &Path, findings: &mut Vec<DocTruthFinding>) {
    let context_path = root.join(".jules/JULES_CONTEXT.md");
    if !context_path.exists() {
        return;
    }

    let content = match fs::read_to_string(&context_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let active_crates = doc_truth_get_workspace_crates(root);
    let mut table_crates = HashSet::new();

    let row_re = Regex::new(r"\|\s*`([a-z0-9-]+)`\s*\|").unwrap();

    for line in content.lines() {
        if line.contains("<!-- crate-ref-ignore -->") {
            continue;
        }
        if line.starts_with('|') {
            for cap in row_re.captures_iter(line) {
                let name = cap.get(1).unwrap().as_str();
                table_crates.insert(name.to_string());
            }
        }
    }

    // Compare table_crates vs active_crates (excluding xtask)
    for active in &active_crates {
        if active == "xtask" {
            continue;
        }
        if !table_crates.contains(active) {
            findings.push(DocTruthFinding {
                file: ".jules/JULES_CONTEXT.md".to_string(),
                line: 1,
                id: "DOC-TRUTH-CRATE-TABLE-MISSING".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Fehlender Crate '{}' in Crate-Tabelle in JULES_CONTEXT.md.",
                    active
                ),
                fix: format!(
                    "Crate '{}' in Crate-Tabelle von JULES_CONTEXT.md aufnehmen.",
                    active
                ),
            });
        }
    }

    for table_c in &table_crates {
        if !active_crates.contains(table_c) {
            findings.push(DocTruthFinding {
                file: ".jules/JULES_CONTEXT.md".to_string(),
                line: 1,
                id: "DOC-TRUTH-CRATE-TABLE-EXTRA".to_string(),
                severity: "error".to_string(),
                message: format!("Invalider/überzähliger Crate '{}' in Crate-Tabelle in JULES_CONTEXT.md.", table_c),
                fix: format!("Crate '{}' aus Crate-Tabelle in JULES_CONTEXT.md entfernen oder '<!-- crate-ref-ignore -->' anhängen.", table_c),
            });
        }
    }
}
