//! Harness module for generating context packs bound to task cards (context-card).

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub struct TaskCardData {
    pub id: String,
    pub title: String,
    pub krate: String,
    pub tier: i64,
    pub risk: String,
    pub goal: String,
    pub non_goals: Vec<String>,
    pub scope: Vec<String>,
    pub forbidden: Vec<String>,
    pub invariants: Vec<String>,
    pub acceptance: Vec<String>,
    pub evidence_required: Vec<String>,
    pub budget_files: Option<i64>,
    pub budget_lines: Option<i64>,
    pub budget_iterations: Option<i64>,
}

fn sanitize_path(path_str: &str) -> Result<PathBuf, String> {
    if path_str.starts_with('-') {
        return Err(format!("Invalider Pfad mit führendem Minus: {}", path_str));
    }
    let p = Path::new(path_str);
    for comp in p.components() {
        if comp == std::path::Component::ParentDir {
            return Err(format!("Verbotenes '..' Segment im Pfad: {}", path_str));
        }
    }
    Ok(p.to_path_buf())
}

pub fn parse_task_card(path: &Path) -> Result<TaskCardData, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Kann Task-Karte {} nicht lesen: {}", path.display(), e))?;

    let val: toml::Value = content
        .parse()
        .map_err(|e| format!("Kann TOML in {} nicht parsen: {}", path.display(), e))?;

    let id = val
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let title = val
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let krate = val
        .get("crate")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tier = val.get("tier").and_then(|v| v.as_integer()).unwrap_or(0);
    let risk = val
        .get("risk")
        .and_then(|v| v.as_str())
        .unwrap_or("none")
        .to_string();
    let goal = val
        .get("goal")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let parse_string_vec = |key: &str| -> Vec<String> {
        val.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };

    let non_goals = parse_string_vec("non_goals");
    let scope = parse_string_vec("scope");
    let forbidden = parse_string_vec("forbidden");
    let invariants = parse_string_vec("invariants");
    let acceptance = parse_string_vec("acceptance");
    let evidence_required = parse_string_vec("evidence_required");

    let budget_files = val
        .get("budget")
        .and_then(|b| b.get("files"))
        .and_then(|v| v.as_integer());
    let budget_lines = val
        .get("budget")
        .and_then(|b| b.get("lines"))
        .and_then(|v| v.as_integer());
    let budget_iterations = val
        .get("budget")
        .and_then(|b| b.get("iterations"))
        .and_then(|v| v.as_integer());

    Ok(TaskCardData {
        id,
        title,
        krate,
        tier,
        risk,
        goal,
        non_goals,
        scope,
        forbidden,
        invariants,
        acceptance,
        evidence_required,
        budget_files,
        budget_lines,
        budget_iterations,
    })
}

pub fn check_freshness(root: &Path) -> i32 {
    let repo_state_path = root.join(".jules/context/REPO_STATE.md");
    if !repo_state_path.exists() {
        return 2; // missing
    }

    let content = match fs::read_to_string(&repo_state_path) {
        Ok(c) => c,
        Err(_) => return 2,
    };

    let current_head = crate::context_pack::get_head_commit(root);

    for line in content.lines() {
        if let Some(sha) = line.strip_prefix("Frische-Anker: HEAD muss `") {
            let sha = sha.trim_end_matches("` sein").trim();
            if sha == current_head {
                return 0; // fresh
            } else {
                return 3; // stale
            }
        }
    }

    3 // stale/unknown
}

pub fn extract_agents_sections(agents_content: &str, file_rel_path: &str) -> String {
    let target_headers = [
        "Invarianten",
        "Verboten",
        "Nebenläufigkeit",
        "Verifikation",
        "Bekannte Lücken",
    ];

    let mut result = String::new();
    let lines: Vec<&str> = agents_content.lines().collect();
    let mut in_target_section = false;
    let mut section_start = 0;
    let mut current_header = "";
    let mut total_extracted_lines = 0;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let header_text = trimmed.trim_start_matches('#').trim();
            let clean_header = header_text
                .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c.is_whitespace());
            let matches_header = target_headers.iter().any(|&h| clean_header.starts_with(h));

            if matches_header {
                if in_target_section {
                    let section_lines = &lines[section_start..idx];
                    let line_range = format!("{}: {}-{}", file_rel_path, section_start + 1, idx);
                    result.push_str(&format!(
                        "### {}\n*Quelle: `{}`*\n",
                        current_header, line_range
                    ));
                    for l in section_lines {
                        if total_extracted_lines >= 150 {
                            result.push_str("... [gekürzt, Limit 150 Zeilen erreicht]\n");
                            return result;
                        }
                        result.push_str(l);
                        result.push('\n');
                        total_extracted_lines += 1;
                    }
                    result.push('\n');
                }
                in_target_section = true;
                section_start = idx + 1;
                current_header = clean_header;
            } else if in_target_section {
                let section_lines = &lines[section_start..idx];
                let line_range = format!("{}: {}-{}", file_rel_path, section_start + 1, idx);
                result.push_str(&format!(
                    "### {}\n*Quelle: `{}`*\n",
                    current_header, line_range
                ));
                for l in section_lines {
                    if total_extracted_lines >= 150 {
                        result.push_str("... [gekürzt, Limit 150 Zeilen erreicht]\n");
                        return result;
                    }
                    result.push_str(l);
                    result.push('\n');
                    total_extracted_lines += 1;
                }
                result.push('\n');
                in_target_section = false;
            }
        }
    }

    if in_target_section && section_start < lines.len() {
        let end_idx = lines.len();
        let section_lines = &lines[section_start..end_idx];
        let line_range = format!("{}: {}-{}", file_rel_path, section_start + 1, end_idx);
        result.push_str(&format!(
            "### {}\n*Quelle: `{}`*\n",
            current_header, line_range
        ));
        for l in section_lines {
            if total_extracted_lines >= 150 {
                result.push_str("... [gekürzt, Limit 150 Zeilen erreicht]\n");
                return result;
            }
            result.push_str(l);
            result.push('\n');
            total_extracted_lines += 1;
        }
        result.push('\n');
    }

    result
}

pub fn map_risk_to_gates_and_tools(risk: &str) -> (Vec<&'static str>, Vec<(&'static str, bool)>) {
    // Returns (mandatory_gates, required_tools)
    match risk {
        "sec" => (
            vec!["mutants-diff", "fuzz-smoke"],
            vec![("cargo-mutants", false), ("cargo-fuzz", false)],
        ),
        "crash" => (
            vec!["wal-replay-verify"],
            vec![("cargo", true), ("git", true)],
        ),
        "simd" => (vec!["simd-safety"], vec![("cargo", true)]),
        _ => (vec!["standard-verification"], vec![("cargo", true)]),
    }
}

pub fn check_scope_collision_with_branch(scope: &[String], branch_files: &[String]) -> bool {
    for s in scope {
        let clean_scope = s
            .trim_end_matches("/*")
            .trim_end_matches("*.rs")
            .trim_end_matches('*')
            .trim_end_matches('/');
        for bf in branch_files {
            if bf == s || bf.starts_with(clean_scope) {
                return true;
            }
        }
    }
    false
}

pub fn run_context_card(args: &[String]) -> i32 {
    run(args)
}

pub fn run(args: &[String]) -> i32 {
    let root = xtask::find_root_dir();

    let mut card_path_opt: Option<PathBuf> = None;
    let mut check_only = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--card" => {
                if i + 1 < args.len() {
                    match sanitize_path(&args[i + 1]) {
                        Ok(p) => card_path_opt = Some(p),
                        Err(e) => {
                            eprintln!("❌ {}", e);
                            return 1;
                        }
                    }
                    i += 1;
                }
            }
            "--check" => {
                check_only = true;
            }
            _ => {}
        }
        i += 1;
    }

    if card_path_opt.is_none() {
        if let Ok(env_card) = std::env::var("CONTEXTRA_CARD") {
            if !env_card.is_empty() {
                match sanitize_path(&env_card) {
                    Ok(p) => card_path_opt = Some(p),
                    Err(e) => {
                        eprintln!("❌ {}", e);
                        return 1;
                    }
                }
            }
        }
    }

    // Freshness & Self-Healing
    let freshness = check_freshness(&root);

    if check_only {
        return freshness;
    }

    if freshness != 0 {
        // Run refresh-context.sh --agent
        let status = Command::new("bash")
            .current_dir(&root)
            .args([".jules/setup/refresh-context.sh", "--agent"])
            .status();

        match status {
            Ok(s) if s.success() => {}
            _ => {
                eprintln!("⚠️ refresh-context.sh --agent schlug fehl.");
            }
        }
    }

    let repo_state = match fs::read_to_string(root.join(".jules/context/REPO_STATE.md")) {
        Ok(s) => s,
        Err(_) => crate::context_pack::generate_repo_state(&root),
    };

    let mut pack_md = String::new();

    if let Some(card_path) = card_path_opt {
        let full_card_path = if card_path.is_absolute() {
            card_path
        } else {
            root.join(card_path)
        };

        let card = match parse_task_card(&full_card_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ {}", e);
                return 1;
            }
        };

        // (a) Kollisionsbanner
        let mut collisions = Vec::new();

        // Extract remote branches and their modified files from REPO_STATE.md
        let branches_out = Command::new("git")
            .current_dir(&root)
            .args([
                "for-each-ref",
                "--sort=-committerdate",
                "refs/remotes/origin",
                "--format=%(refname:short)",
            ])
            .output()
            .ok()
            .filter(|o| o.status.success());

        if let Some(out) = branches_out {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for b_line in stdout.lines() {
                let ref_name = b_line.trim();
                if ref_name == "origin/main" || ref_name == "origin/HEAD" || ref_name == "origin" {
                    continue;
                }
                let branch_short = ref_name.strip_prefix("origin/").unwrap_or(ref_name);

                let diff_out = Command::new("git")
                    .current_dir(&root)
                    .args(["diff", "--name-only", "origin/main...", ref_name])
                    .output()
                    .ok()
                    .filter(|o| o.status.success());

                if let Some(df) = diff_out {
                    let f_str = String::from_utf8_lossy(&df.stdout);
                    let b_files: Vec<String> = f_str
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if check_scope_collision_with_branch(&card.scope, &b_files) {
                        collisions.push((branch_short.to_string(), b_files));
                    }
                }
            }
        }

        if !collisions.is_empty() {
            pack_md.push_str("# ⚠️ KOLLISIONSDETEKTION!\n");
            pack_md.push_str("> **WARNUNG:** Der Scope dieser Task-Karte überschneidet sich mit aktiven Remote-Branches!\n");
            pack_md.push_str("> **Handlungsanweisung:** Karte überspringen und im PR melden.\n\n");
            for (b_name, b_files) in collisions {
                pack_md.push_str(&format!(
                    "- **Branch `{}`** kollidiert in folgenden Dateien:\n",
                    b_name
                ));
                for f in b_files.iter().take(5) {
                    pack_md.push_str(&format!("  - `{}`\n", f));
                }
            }
            pack_md.push_str("\n---\n\n");
        }

        // (b) Task-Karte Details
        pack_md.push_str(&format!("# CONTEXT PACK: {} — {}\n\n", card.id, card.title));
        pack_md.push_str(&format!(
            "- **Crate:** `{}` (Tier {})\n",
            card.krate, card.tier
        ));
        pack_md.push_str(&format!("- **Risiko:** `{}`\n", card.risk));
        pack_md.push_str(&format!("- **Ziel:** {}\n\n", card.goal));

        if !card.non_goals.is_empty() {
            pack_md.push_str("## Non-Goals\n");
            for ng in &card.non_goals {
                pack_md.push_str(&format!("- {}\n", ng));
            }
            pack_md.push('\n');
        }

        pack_md.push_str("## Scope\n");
        for s in &card.scope {
            pack_md.push_str(&format!("- `{}`\n", s));
        }
        pack_md.push('\n');

        if !card.forbidden.is_empty() {
            pack_md.push_str("## Verboten / Geschützt\n");
            for f in &card.forbidden {
                pack_md.push_str(&format!("- `{}`\n", f));
            }
            pack_md.push('\n');
        }

        if !card.invariants.is_empty() {
            pack_md.push_str("## Invarianten\n");
            for inv in &card.invariants {
                pack_md.push_str(&format!("- {}\n", inv));
            }
            pack_md.push('\n');
        }

        // (c) Risiko -> Pflichtgates und benötigte Tools
        let (mandatory_gates, required_tools) = map_risk_to_gates_and_tools(&card.risk);
        pack_md.push_str("## Gate- & Tool-Anforderungen\n");
        pack_md.push_str(&format!(
            "- **Pflicht-Gates für Risiko `{}`:** {}\n",
            card.risk,
            mandatory_gates.join(", ")
        ));
        pack_md.push_str("- **Tool-Verfügbarkeit:**\n");

        for (tool_name, is_req) in required_tools {
            let available = Command::new("which")
                .arg(tool_name)
                .output()
                .map_or(false, |o| o.status.success());
            let symbol = if available { "✅" } else { "❌" };
            pack_md.push_str(&format!(
                "  - {} `{}` (Erforderlich: {})\n",
                symbol, tool_name, is_req
            ));
            if !available && is_req {
                pack_md.push_str(&format!("    ⚠️ **PROMINENTER HINWEIS:** Erforderliches Tool `{}` für Pflichtgate fehlt!\n", tool_name));
            }
        }
        pack_md.push('\n');

        // (d) Crate AGENTS.md Auszüge
        pack_md.push_str("## AGENTS.md Auszüge\n");
        let crate_agents = root.join("crates").join(&card.krate).join("AGENTS.md");
        let root_agents = root.join("AGENTS.md");

        let mut agents_extracted = String::new();
        if crate_agents.exists() {
            if let Ok(c) = fs::read_to_string(&crate_agents) {
                agents_extracted.push_str(&extract_agents_sections(
                    &c,
                    &format!("crates/{}/AGENTS.md", card.krate),
                ));
            }
        }
        if root_agents.exists() {
            if let Ok(c) = fs::read_to_string(&root_agents) {
                agents_extracted.push_str(&extract_agents_sections(&c, "AGENTS.md"));
            }
        }

        if agents_extracted.is_empty() {
            pack_md.push_str("*(Keine relevanten Abschnitte in AGENTS.md gefunden)*\n\n");
        } else {
            pack_md.push_str(&agents_extracted);
        }

        // (e) Einschlägige Regeldateien aus rules/
        pack_md.push_str("## Einschlägige Regeldateien (rules/)\n");
        let rules_dir = root.join("rules");
        let mut rule_files = Vec::new();
        if rules_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&rules_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                            if name.contains(&card.risk)
                                || card.risk == "sec"
                                || card.risk == "crash"
                            {
                                rule_files.push(format!("rules/{}", name));
                            }
                        }
                    }
                }
            }
        }

        if rule_files.is_empty() {
            pack_md.push_str("*(Keine spezifischen Regeldateien für dieses Risiko)*\n\n");
        } else {
            for rf in rule_files {
                pack_md.push_str(&format!("- `{}`\n", rf));
            }
            pack_md.push('\n');
        }

        // (f) Historie der Scope-Dateien
        pack_md.push_str("## Historie der Scope-Dateien (letzte 5 Commits)\n");
        let mut history_found = false;
        for s in &card.scope {
            let log_out = Command::new("git")
                .current_dir(&root)
                .args(["log", "-5", "--format=%h %cs %s", "--", s])
                .output()
                .ok()
                .filter(|o| o.status.success());

            if let Some(lo) = log_out {
                let stdout = String::from_utf8_lossy(&lo.stdout);
                if !stdout.trim().is_empty() {
                    pack_md.push_str(&format!("### `{}`\n", s));
                    for l in stdout.lines() {
                        pack_md.push_str(&format!("- {}\n", l));
                    }
                    history_found = true;
                }
            }
        }
        if !history_found {
            pack_md.push_str("*(Keine Git-Historie für Scope-Dateien)*\n\n");
        } else {
            pack_md.push('\n');
        }

        // (g) AI-Tags in den Scope-Dateien
        pack_md.push_str("## AI-Tags in Scope-Dateien\n");
        let mut scope_tags = Vec::new();
        for s in &card.scope {
            let scope_path = root.join(s);
            if scope_path.is_file() {
                if let Ok(c) = fs::read_to_string(&scope_path) {
                    for (line_idx, line) in c.lines().enumerate() {
                        if line.contains("AI-TAG[") {
                            scope_tags.push(format!("{}:{}: {}", s, line_idx + 1, line.trim()));
                        }
                    }
                }
            }
        }

        if scope_tags.is_empty() {
            pack_md.push_str("*(Keine AI-Tags im Scope)*\n\n");
        } else {
            for st in scope_tags {
                pack_md.push_str(&format!("- `{}`\n", st));
            }
            pack_md.push('\n');
        }

        // (h) Vorhandene Testdateien der Crate
        pack_md.push_str("## Testdateien der Crate\n");
        let tests_dir = root.join("crates").join(&card.krate).join("tests");
        let mut test_files = Vec::new();
        if tests_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&tests_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.is_file() && p.extension().is_some_and(|e| e == "rs") {
                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                            test_files.push(name.to_string());
                        }
                    }
                }
            }
        }

        if test_files.is_empty() {
            pack_md.push_str("*(Keine gesonderten Integrationstests unter tests/)*\n\n");
        } else {
            test_files.sort();
            for tf in test_files {
                pack_md.push_str(&format!("- `{}`\n", tf));
            }
            pack_md.push('\n');
        }

        // (i) Akzeptanzbefehle wörtlich
        pack_md.push_str("## Akzeptanzbefehle\n```bash\n");
        for cmd in &card.acceptance {
            pack_md.push_str(cmd);
            pack_md.push('\n');
        }
        pack_md.push_str("```\n\n");
    } else {
        pack_md.push_str("# CONTEXT PACK (Keine Karte angegeben)\n");
        pack_md.push_str("> Hinweis: Führe `cargo xtask context-card --card <pfad>` aus, um eine Task-Karte einzubinden.\n\n");
    }

    pack_md.push_str("---\n\n");
    pack_md.push_str(&repo_state);

    let output_path = root.join(".jules/context/CONTEXT_PACK.md");
    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(e) = fs::write(&output_path, &pack_md) {
        eprintln!("❌ Kann CONTEXT_PACK.md nicht schreiben: {}", e);
        return 1;
    }

    println!("✅ CONTEXT_PACK.md erzeugt ({} Bytes)", pack_md.len());
    0
}
