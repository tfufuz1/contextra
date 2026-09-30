//! Module implementing ratchet baseline checks, auto-tightening and show commands.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RatchetConfig {
    pub unwrap_legacy_grep: u64,
    pub unwrap_exact: u64,
    pub expect_exact: u64,
    pub allow_attrs: u64,
    pub ignore_attrs: u64,
    pub forbid_unsafe_crates: u64,
    pub unsafe_blocks: BTreeMap<String, u64>,
}

pub fn ratchet_parse_args(args: &[String]) -> (String, String, Option<String>, bool, bool) {
    let mut root = String::new();
    let mut command = String::from("check");
    let mut adr = None;
    let mut json = false;
    let mut init = false;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "check" | "update" | "show" => {
                command = args[idx].clone();
                idx += 1;
            }
            "--root" => {
                if idx + 1 < args.len() {
                    root = args[idx + 1].clone();
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            "--adr" => {
                if idx + 1 < args.len() {
                    adr = Some(args[idx + 1].clone());
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            "--json" => {
                json = true;
                idx += 1;
            }
            "--init" => {
                init = true;
                idx += 1;
            }
            _ => {
                idx += 1;
            }
        }
    }

    if root.is_empty() {
        if let Ok(out) = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            if out.status.success() {
                root = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
    }

    (root, command, adr, json, init)
}

pub fn ratchet_count_legacy_unwrap(root: &str) -> u64 {
    let crates_dir = Path::new(root).join("crates");
    if !crates_dir.exists() {
        return 0;
    }

    let mut count = 0;
    for entry in WalkDir::new(crates_dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let path_str = path.to_string_lossy().replace('\\', "/");
        if path_str.contains("/tests") || path_str.contains("/test") || path_str.contains("/bench")
        {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for line in content.lines() {
            if line.contains("#[cfg(test)]") {
                continue;
            }
            if line.contains(".unwrap()") || line.contains(".expect(") {
                count += 1;
            }
        }
    }

    count
}

pub fn ratchet_measure_repo(root: &str) -> RatchetConfig {
    let unwrap_legacy_grep = ratchet_count_legacy_unwrap(root);

    let mut unwrap_exact = 0;
    let mut expect_exact = 0;
    let mut allow_attrs = 0;
    let mut ignore_attrs = 0;
    let mut forbid_unsafe_crates = 0;
    let mut unsafe_blocks: BTreeMap<String, u64> = BTreeMap::new();

    let unsafe_islands = [
        "contextra-simd",
        "contextra-sys",
        "contextra-wire",
    ];
    for island in unsafe_islands {
        unsafe_blocks.insert(island.to_string(), 0);
    }

    let crates_dir = Path::new(root).join("crates");
    if crates_dir.exists() {
        // Find crates
        if let Ok(entries) = fs::read_dir(&crates_dir) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let crate_name = entry.file_name().to_string_lossy().to_string();
                let crate_path = entry.path();
                let mut crate_has_forbid = false;

                for file_entry in WalkDir::new(&crate_path).into_iter().flatten() {
                    if !file_entry.file_type().is_file() {
                        continue;
                    }
                    let p = file_entry.path();
                    if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                        continue;
                    }
                    let path_str = p.to_string_lossy().replace('\\', "/");
                    if path_str.contains("/tests/")
                        || path_str.contains("/benches/")
                        || path_str.contains("/examples/")
                    {
                        continue;
                    }

                    let content = match fs::read_to_string(p) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    if content.contains("#![forbid(unsafe_code)]") {
                        crate_has_forbid = true;
                    }

                    if let Ok(syn_file) = syn::parse_file(&content) {
                        // Traverse AST for exact counts
                        struct AstVisitor<'a> {
                            unwrap_cnt: &'a mut u64,
                            expect_cnt: &'a mut u64,
                            allow_cnt: &'a mut u64,
                            ignore_cnt: &'a mut u64,
                            unsafe_cnt: &'a mut u64,
                        }

                        impl<'a> syn::visit::Visit<'a> for AstVisitor<'a> {
                            fn visit_item_mod(&mut self, i: &'a syn::ItemMod) {
                                // Ignore #[cfg(test)] modules
                                for attr in &i.attrs {
                                    if attr.path().is_ident("cfg") {
                                        if let Ok(nested) = attr.parse_args::<syn::Ident>() {
                                            if nested == "test" {
                                                return;
                                            }
                                        }
                                    }
                                }
                                syn::visit::visit_item_mod(self, i);
                            }

                            fn visit_attribute(&mut self, attr: &'a syn::Attribute) {
                                if attr.path().is_ident("allow") {
                                    *self.allow_cnt += 1;
                                }
                                if attr.path().is_ident("ignore") {
                                    *self.ignore_cnt += 1;
                                }
                                syn::visit::visit_attribute(self, attr);
                            }

                            fn visit_expr_method_call(&mut self, i: &'a syn::ExprMethodCall) {
                                if i.method == "unwrap" {
                                    *self.unwrap_cnt += 1;
                                } else if i.method == "expect" {
                                    *self.expect_cnt += 1;
                                }
                                syn::visit::visit_expr_method_call(self, i);
                            }

                            fn visit_expr_unsafe(&mut self, i: &'a syn::ExprUnsafe) {
                                *self.unsafe_cnt += 1;
                                syn::visit::visit_expr_unsafe(self, i);
                            }
                        }

                        use syn::visit::Visit;
                        let mut crate_unsafe = 0;
                        let mut visitor = AstVisitor {
                            unwrap_cnt: &mut unwrap_exact,
                            expect_cnt: &mut expect_exact,
                            allow_cnt: &mut allow_attrs,
                            ignore_cnt: &mut ignore_attrs,
                            unsafe_cnt: &mut crate_unsafe,
                        };
                        visitor.visit_file(&syn_file);

                        if unsafe_islands.contains(&crate_name.as_str()) {
                            *unsafe_blocks.entry(crate_name.clone()).or_insert(0) += crate_unsafe;
                        }
                    }
                }

                if crate_has_forbid {
                    forbid_unsafe_crates += 1;
                }
            }
        }
    }

    RatchetConfig {
        unwrap_legacy_grep,
        unwrap_exact,
        expect_exact,
        allow_attrs,
        ignore_attrs,
        forbid_unsafe_crates,
        unsafe_blocks,
    }
}

pub fn run_ratchet(args: &[String]) -> i32 {
    let (root, command, adr, json, init) = ratchet_parse_args(args);

    let config_path = Path::new(&root).join("governance/ratchet.toml");
    let measured = ratchet_measure_repo(&root);

    if command == "show" {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "ratchet",
                    "status": "pass",
                    "summary": "Repository Ratchet Metrics",
                    "measured": measured
                })
            );
        } else {
            println!("=== Ratchet Metrics ===");
            println!("  unwrap_legacy_grep: {}", measured.unwrap_legacy_grep);
            println!("  unwrap_exact:       {}", measured.unwrap_exact);
            println!("  expect_exact:       {}", measured.expect_exact);
            println!("  allow_attrs:        {}", measured.allow_attrs);
            println!("  ignore_attrs:       {}", measured.ignore_attrs);
            println!("  forbid_unsafe_crates: {}", measured.forbid_unsafe_crates);
            println!("  unsafe_blocks:");
            for (k, v) in &measured.unsafe_blocks {
                println!("    {k}: {v}");
            }
        }
        return 0;
    }

    if init {
        if config_path.exists() {
            let msg = format!(
                "--init ist nur zulässig, wenn '{}' fehlt.",
                config_path.display()
            );
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "ratchet",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [ratchet]: {msg}");
            }
            return 2;
        }

        let toml_str = match toml::to_string_pretty(&measured) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("Fehler beim Serialisieren von ratchet.toml: {e}");
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "gate": "ratchet",
                            "status": "error",
                            "summary": msg,
                            "findings": []
                        })
                    );
                } else {
                    eprintln!("FEHLER [ratchet]: {msg}");
                }
                return 2;
            }
        };

        if let Err(e) = fs::write(&config_path, toml_str) {
            let msg = format!("Fehler beim Schreiben von '{}': {e}", config_path.display());
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "ratchet",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [ratchet]: {msg}");
            }
            return 2;
        }

        // Also update .github/unwrap_baseline.txt
        let unwrap_baseline_path = Path::new(&root).join(".github/unwrap_baseline.txt");
        let _ = fs::write(
            unwrap_baseline_path,
            format!("{}\n", measured.unwrap_legacy_grep),
        );

        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "ratchet",
                    "status": "pass",
                    "summary": "Ratchet.toml erfolgreich initialisiert.",
                    "findings": []
                })
            );
        } else {
            println!("PASS [ratchet]: Ratchet.toml erfolgreich initialisiert.");
        }
        return 0;
    }

    if !config_path.exists() {
        let msg = format!("Ratchet-Konfiguration '{}' nicht gefunden. Nutze 'ratchet update --init' zum Erzeugen.", config_path.display());
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "ratchet",
                    "status": "error",
                    "summary": msg,
                    "findings": []
                })
            );
        } else {
            eprintln!("FEHLER [ratchet]: {msg}");
        }
        return 2;
    }

    let config_str = match fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Fehler beim Lesen von '{}': {e}", config_path.display());
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "ratchet",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [ratchet]: {msg}");
            }
            return 2;
        }
    };

    let baseline: RatchetConfig = match toml::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Fehler beim Parsen von '{}': {e}", config_path.display());
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "ratchet",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [ratchet]: {msg}");
            }
            return 2;
        }
    };

    if command == "update" {
        // Check if baseline increases occur
        let mut has_increases = false;
        let mut increase_details = Vec::new();

        if measured.unwrap_legacy_grep > baseline.unwrap_legacy_grep {
            has_increases = true;
            increase_details.push(format!(
                "unwrap_legacy_grep: {} -> {}",
                baseline.unwrap_legacy_grep, measured.unwrap_legacy_grep
            ));
        }
        if measured.unwrap_exact > baseline.unwrap_exact {
            has_increases = true;
            increase_details.push(format!(
                "unwrap_exact: {} -> {}",
                baseline.unwrap_exact, measured.unwrap_exact
            ));
        }
        if measured.expect_exact > baseline.expect_exact {
            has_increases = true;
            increase_details.push(format!(
                "expect_exact: {} -> {}",
                baseline.expect_exact, measured.expect_exact
            ));
        }
        if measured.allow_attrs > baseline.allow_attrs {
            has_increases = true;
            increase_details.push(format!(
                "allow_attrs: {} -> {}",
                baseline.allow_attrs, measured.allow_attrs
            ));
        }
        if measured.ignore_attrs > baseline.ignore_attrs {
            has_increases = true;
            increase_details.push(format!(
                "ignore_attrs: {} -> {}",
                baseline.ignore_attrs, measured.ignore_attrs
            ));
        }
        if measured.forbid_unsafe_crates < baseline.forbid_unsafe_crates {
            has_increases = true;
            increase_details.push(format!(
                "forbid_unsafe_crates: {} -> {}",
                baseline.forbid_unsafe_crates, measured.forbid_unsafe_crates
            ));
        }
        for (k, v) in &measured.unsafe_blocks {
            let base_v = baseline.unsafe_blocks.get(k).copied().unwrap_or(0);
            if *v > base_v {
                has_increases = true;
                increase_details.push(format!("unsafe_blocks.{k}: {base_v} -> {v}"));
            }
        }

        if has_increases {
            let mut adr_valid = false;
            if let Some(adr_id) = &adr {
                let adr_path = Path::new(&root)
                    .join("docs/decisions")
                    .join(format!("{adr_id}.md"));
                let mut exists = adr_path.exists();
                if !exists {
                    if let Ok(entries) = WalkDir::new(Path::new(&root).join("docs/decisions"))
                        .max_depth(1)
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()
                    {
                        for entry in entries {
                            if let Some(name) = entry.file_name().to_str() {
                                if name.starts_with(&format!("{adr_id}-")) && name.ends_with(".md")
                                {
                                    exists = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if exists {
                    adr_valid = true;
                }
            }

            if !adr_valid {
                let msg = format!("Ratchet-Erhöhung verweigert. Erhöhungen ({}) sind nur mit '--adr ADR-NNN' UND existierender ADR-Datei zulässig.", increase_details.join(", "));
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "gate": "ratchet",
                            "status": "error",
                            "summary": msg,
                            "findings": []
                        })
                    );
                } else {
                    eprintln!("FEHLER [ratchet]: {msg}");
                }
                return 2;
            }
        }

        // Apply auto-tightening / updates
        let new_config = RatchetConfig {
            unwrap_legacy_grep: measured.unwrap_legacy_grep.min(baseline.unwrap_legacy_grep),
            unwrap_exact: measured.unwrap_exact.min(baseline.unwrap_exact),
            expect_exact: measured.expect_exact.min(baseline.expect_exact),
            allow_attrs: measured.allow_attrs.min(baseline.allow_attrs),
            ignore_attrs: measured.ignore_attrs.min(baseline.ignore_attrs),
            forbid_unsafe_crates: measured
                .forbid_unsafe_crates
                .max(baseline.forbid_unsafe_crates),
            unsafe_blocks: measured
                .unsafe_blocks
                .iter()
                .map(|(k, v)| {
                    let base_v = baseline.unsafe_blocks.get(k).copied().unwrap_or(0);
                    (k.clone(), (*v).min(base_v))
                })
                .collect(),
        };

        let toml_str = match toml::to_string_pretty(&new_config) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("Fehler beim Serialisieren von ratchet.toml: {e}");
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "gate": "ratchet",
                            "status": "error",
                            "summary": msg,
                            "findings": []
                        })
                    );
                } else {
                    eprintln!("FEHLER [ratchet]: {msg}");
                }
                return 2;
            }
        };

        if let Err(e) = fs::write(&config_path, toml_str) {
            let msg = format!("Fehler beim Schreiben von '{}': {e}", config_path.display());
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "ratchet",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [ratchet]: {msg}");
            }
            return 2;
        }

        // Update .github/unwrap_baseline.txt
        let unwrap_baseline_path = Path::new(&root).join(".github/unwrap_baseline.txt");
        let _ = fs::write(
            unwrap_baseline_path,
            format!("{}\n", new_config.unwrap_legacy_grep),
        );

        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "ratchet",
                    "status": "pass",
                    "summary": "Ratchet.toml erfolgreich aktualisiert.",
                    "findings": []
                })
            );
        } else {
            println!("PASS [ratchet]: Ratchet.toml erfolgreich aktualisiert.");
        }
        return 0;
    }

    // Default subcommand: check
    let mut violations = Vec::new();

    if measured.unwrap_legacy_grep > baseline.unwrap_legacy_grep {
        violations.push((
            "RATCHET-UNWRAP-LEGACY",
            format!(
                "unwrap_legacy_grep ({}) überschreitet Baseline ({})",
                measured.unwrap_legacy_grep, baseline.unwrap_legacy_grep
            ),
            "Reduziere unwrap()/expect() Aufrufe im Code.",
        ));
    }
    if measured.unwrap_exact > baseline.unwrap_exact {
        violations.push((
            "RATCHET-UNWRAP-EXACT",
            format!(
                "unwrap_exact ({}) überschreitet Baseline ({})",
                measured.unwrap_exact, baseline.unwrap_exact
            ),
            "Reduziere unwrap() Aufrufe im Code.",
        ));
    }
    if measured.expect_exact > baseline.expect_exact {
        violations.push((
            "RATCHET-EXPECT-EXACT",
            format!(
                "expect_exact ({}) überschreitet Baseline ({})",
                measured.expect_exact, baseline.expect_exact
            ),
            "Reduziere expect() Aufrufe im Code.",
        ));
    }
    if measured.allow_attrs > baseline.allow_attrs {
        violations.push((
            "RATCHET-ALLOW-ATTRS",
            format!(
                "allow_attrs ({}) überschreitet Baseline ({})",
                measured.allow_attrs, baseline.allow_attrs
            ),
            "Entferne neue #[allow(...)] Attribute.",
        ));
    }
    if measured.ignore_attrs > baseline.ignore_attrs {
        violations.push((
            "RATCHET-IGNORE-ATTRS",
            format!(
                "ignore_attrs ({}) überschreitet Baseline ({})",
                measured.ignore_attrs, baseline.ignore_attrs
            ),
            "Entferne neue #[ignore] Attribute.",
        ));
    }
    if measured.forbid_unsafe_crates < baseline.forbid_unsafe_crates {
        violations.push((
            "RATCHET-FORBID-UNSAFE",
            format!(
                "forbid_unsafe_crates ({}) ist unter Baseline ({}) gefallen",
                measured.forbid_unsafe_crates, baseline.forbid_unsafe_crates
            ),
            "Stelle #![forbid(unsafe_code)] in den betroffenen Crates wieder her.",
        ));
    }
    for (k, v) in &measured.unsafe_blocks {
        let base_v = baseline.unsafe_blocks.get(k).copied().unwrap_or(0);
        if *v > base_v {
            violations.push((
                "RATCHET-UNSAFE-BLOCKS",
                format!("unsafe_blocks.{k} ({v}) überschreitet Baseline ({base_v})"),
                "Reduziere unsafe-Blöcke in der Unsafe-Insel.",
            ));
        }
    }

    if violations.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "ratchet",
                    "status": "pass",
                    "summary": "Alle Ratchet-Kennzahlen liegen innerhalb der Baselines.",
                    "findings": []
                })
            );
        } else {
            println!("PASS [ratchet]: Alle Kennzahlen innerhalb der Baselines.");
        }
        return 0;
    }

    let summary = format!(
        "Ratchet-Monotonie-Verstoß: {} Kennzahl(en) überschreiten die Baselines.",
        violations.len()
    );

    if json {
        let findings: Vec<serde_json::Value> = violations
            .iter()
            .map(|(id, msg, fix)| {
                serde_json::json!({
                    "id": id,
                    "severity": "error",
                    "file": "governance/ratchet.toml",
                    "line": 0,
                    "message": msg,
                    "fix": fix
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::json!({
                "gate": "ratchet",
                "status": "fail",
                "summary": summary,
                "findings": findings
            })
        );
    } else {
        eprintln!("VERSTOSS [ratchet]: {summary}");
        for (id, msg, fix) in &violations {
            eprintln!("  WAS [{id}]: {msg}");
            eprintln!(
                "  WARUM: Qualitätseinschränkung / Ratchet-Monotonie verletzend (ADR/Invariante)."
            );
            eprintln!("  FIX: {fix}");
        }
    }

    1
}
