#![allow(unused_imports, dead_code, unused_variables)]
use crate::*;
use std::path::{Path, PathBuf};
use std::process;

pub fn run_check_commit_diff_integrity(args: &[String]) -> i32 {
    let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
    if let Err(e) = check_commit_diff_integrity::run_check_commit_diff_integrity(extra_args) {
        eprintln!("❌ check-commit-diff-integrity failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_toc_integrity(args: &[String]) -> i32 {
    let targets: Vec<PathBuf> = args.iter().skip(2).map(PathBuf::from).collect();
    if let Err(e) = check_toc_integrity::run_check_toc_integrity(&targets) {
        eprintln!("❌ check-toc-integrity failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_agents_freshness(args: &[String]) -> i32 {
    if let Err(e) = check_agents_freshness::run_check_agents_freshness() {
        eprintln!("❌ check-agents-freshness failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_manifest_completeness(args: &[String]) -> i32 {
    if let Err(e) = check_manifest_completeness::run() {
        eprintln!("❌ check-manifest-completeness failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_crate_references(args: &[String]) -> i32 {
    if let Err(e) = gates::check_crate_references::run_check_crate_references() {
        eprintln!("❌ check-crate-references failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_unsafe_islands(args: &[String]) -> i32 {
    let strict = args.iter().any(|arg| arg == "--strict");
    match check_unsafe_islands::run_check_unsafe_islands(strict) {
        Ok(passed) => {
            if !passed {
                return 1;
            }
        }
        Err(e) => {
            eprintln!("❌ check-unsafe-islands failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_ring_layering(args: &[String]) -> i32 {
    let strict = args.iter().any(|arg| arg == "--strict");
    match check_ring_layering::run_check_ring_layering(strict) {
        Ok(passed) => {
            if !passed {
                return 1;
            }
        }
        Err(e) => {
            eprintln!("❌ check-ring-layering failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_ring_layering_full(args: &[String]) -> i32 {
    let strict = args.iter().any(|arg| arg == "--strict");
    match check_ring_layering::run_check_ring_layering_full(strict) {
        Ok(passed) => {
            if !passed {
                return 1;
            }
        }
        Err(e) => {
            eprintln!("❌ check-ring-layering-full failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_duplicate_core_primitives(args: &[String]) -> i32 {
    match check_duplicate_core_primitives::run_check_duplicate_core_primitives() {
        Ok(passed) => {
            if !passed {
                return 1;
            }
        }
        Err(e) => {
            eprintln!("❌ check-duplicate-core-primitives failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_ring0_async_purity(args: &[String]) -> i32 {
    match check_ring0_async_purity::run_check_ring0_async_purity() {
        Ok(passed) => {
            if !passed {
                return 1;
            }
        }
        Err(e) => {
            eprintln!("❌ check-ring0-async-purity failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_module_reachability(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    match check_module_reachability::run_check_module_reachability(&root) {
        Ok(res) => {
            if !res.warnings.is_empty() {
                println!("⚠️ check-module-reachability Übergangswarnung(en):");
                for w in &res.warnings {
                    println!("  {}", w);
                }
            }
            if !res.errors.is_empty() {
                eprintln!(
                                "❌ check-module-reachability failed: {} unerreichbare/mehrfach deklarierte Datei(en) gefunden:",
                                res.errors.len()
                            );
                for e in &res.errors {
                    eprintln!("  {}", e);
                }
                return 1;
            }
            println!("✅ check-module-reachability: alle Module erreichbar");
        }
        Err(e) => {
            eprintln!("❌ check-module-reachability failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_audit_tool_evidence(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    match check_audit_tool_evidence::run_check_audit_tool_evidence(&root) {
        Ok(violations) => {
            if !violations.is_empty() {
                eprintln!(
                    "❌ check-audit-tool-evidence failed: {} violation(s) found:",
                    violations.len()
                );
                for v in &violations {
                    eprintln!(
                        "  {}:{}: [{}] {}",
                        v.file_path, v.line_num, v.verdict_text, v.reason
                    );
                }
                return 1;
            }
            println!("✅ check-audit-tool-evidence: all audit verdicts verified with evidence");
        }
        Err(e) => {
            eprintln!("❌ check-audit-tool-evidence failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_audit_verdict_independence(args: &[String]) -> i32 {
    if !check_audit_verdict_independence::run_check_audit_verdict_independence() {
        return 1;
    }
    0
}

pub fn run_check_ffi_panic_boundary(args: &[String]) -> i32 {
    let workspace_root = std::path::PathBuf::from(
        std::env::var("CARGO_WORKSPACE_DIR").unwrap_or_else(|_| ".".to_string()),
    );
    if !check_ffi_panic_boundary::run_check_ffi_panic_boundary(&workspace_root) {
        return 1;
    }
    0
}

pub fn run_lint_unsafe_slices(args: &[String]) -> i32 {
    if !lint_unsafe_slice_bounds::run_lint_unsafe_slice_bounds() {
        return 1;
    }
    0
}

pub fn run_check_mutation_score_gate(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
    if let Err(e) = check_mutation_score_gate::run_check_mutation_score_gate(extra_args, &root) {
        eprintln!("❌ check-mutation-score-gate failed: {}", e);
        return 1;
    }
    0
}

pub fn run_mutation_score_record(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
    if let Err(e) = record_mutation_score::run_record_mutation_score(extra_args, &root) {
        eprintln!("❌ mutation-score-record failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_compile(args: &[String]) -> i32 {
    if !check_compile::run_check_compile() {
        return 1;
    }
    0
}

pub fn run_check_phantom_files(args: &[String]) -> i32 {
    if !check_phantom_files::run_check_phantom_files() {
        return 1;
    }
    0
}

pub fn run_check_audit_duplication(args: &[String]) -> i32 {
    if let Err(e) = check_audit_duplication::run_check_audit_duplication(0.85) {
        eprintln!("❌ check-audit-duplication failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_recall_stability(args: &[String]) -> i32 {
    let success = check_recall_stability::run_check_recall_stability(&args[2..]);
    if !success {
        return 1;
    }
    0
}

pub fn run_check_doc_references(args: &[String]) -> i32 {
    if let Err(e) = check_doc_references::run_check_doc_references() {
        eprintln!("❌ check-doc-references failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_duplicate_intent(args: &[String]) -> i32 {
    if let Err(e) = check_duplicate_intent::check_duplicate_intent() {
        eprintln!("❌ check-duplicate-intent failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_commit_messages(args: &[String]) -> i32 {
    if let Err(e) = check_commit_messages::check_commit_messages() {
        eprintln!("❌ check-commit-messages failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_duplicate_symbols_cross_file(args: &[String]) -> i32 {
    if let Err(e) = check_duplicate_symbols_cross_file::run() {
        eprintln!("❌ check-duplicate-symbols-cross-file failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_duplicate_symbols(args: &[String]) -> i32 {
    let cross_module = args.iter().any(|arg| arg == "--cross-module");
    let changed_files = get_changed_rs_files_from_git_diff().unwrap_or_default();
    match check_duplicate_symbols::check_duplicate_symbols_with_options(
        &changed_files,
        cross_module,
    ) {
        Ok(duplicates) => {
            if !duplicates.is_empty() {
                eprintln!(
                    "❌ check-duplicate-symbols failed: {} Duplikat(e) gefunden",
                    duplicates.len()
                );
                for d in &duplicates {
                    let second_file = d.duplicate_file.as_deref().unwrap_or(&d.file);
                    eprintln!(
                        "  {}:{} und {}:{} — doppeltes {} '{}'",
                        d.file,
                        d.first_line,
                        second_file,
                        d.duplicate_line,
                        d.symbol_kind,
                        d.symbol_name
                    );
                }
                return 1;
            }
            println!("✅ check-duplicate-symbols: keine Duplikate gefunden");
        }
        Err(e) => {
            eprintln!("❌ check-duplicate-symbols failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_workflow_commands(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    if !check_workflow_commands::run_check_workflow_commands(&root) {
        return 1;
    }
    0
}

pub fn run_check_placeholder_refs(args: &[String]) -> i32 {
    if let Err(e) = check_placeholder_refs::run() {
        eprintln!("❌ check-placeholder-refs failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_bandit_latency_budget(args: &[String]) -> i32 {
    let status = std::process::Command::new("cargo")
        .args(["run", "--quiet", "-p", "xtask-heavy", "--", "check-bandit-latency-budget"])
        .status();
    match status {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("❌ Failed to execute xtask-heavy check-bandit-latency-budget: {}", e);
            1
        }
    }
}

pub fn run_check_action_pinning(args: &[String]) -> i32 {
    match check_action_pinning::run_check_action_pinning(Path::new(".github/workflows")) {
        Ok(violations) => {
            if !violations.is_empty() {
                eprintln!(
                    "❌ check-action-pinning failed: {} violation(s) found:",
                    violations.len()
                );
                for v in &violations {
                    eprintln!("  {}", v);
                }
                return 1;
            }
            println!("✅ No violations found (check-action-pinning)");
        }
        Err(e) => {
            eprintln!("❌ check-action-pinning error: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_flatbuffers_drift(args: &[String]) -> i32 {
    if let Err(e) = check_flatbuffers_drift::check_flatbuffers_drift() {
        eprintln!("{}", e);
        return 1;
    }
    0
}

pub fn run_regenerate_flatbuffers(args: &[String]) -> i32 {
    if let Err(e) = check_flatbuffers_drift::regenerate_flatbuffers() {
        eprintln!("{}", e);
        return 1;
    }
    0
}

pub fn run_check_adr_deadlines(args: &[String]) -> i32 {
    if let Err(e) = check_adr_deadlines::check_adr_deadlines() {
        eprintln!("❌ check-adr-deadlines failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_vetoes(args: &[String]) -> i32 {
    if let Err(e) = check_vetoes::check_vetoes() {
        eprintln!("❌ check-vetoes failed: {}", e);
        return 1;
    }
    0
}

pub fn run_validate_tags(args: &[String]) -> i32 {
    let fix = args.iter().any(|arg| arg == "--fix");
    let success = crate::run_validate_tags(fix);
    if success {
        0
    } else {
        1
    }
}

pub fn run_check_jules_context_freshness(args: &[String]) -> i32 {
    let success = crate::run_check_jules_context_freshness();
    if !success {
        return 1;
    }
    0
}

pub fn run_check_review_coverage(args: &[String]) -> i32 {
    let tags = crate::scan_tags("crates");
    let success = crate::run_check_review_coverage(&tags);
    if !success {
        return 1;
    }
    0
}

pub fn run_check_agents_integrity(args: &[String]) -> i32 {
    let success = check_agents_integrity::run_check_agents_integrity();
    if !success {
        return 1;
    }
    0
}

pub fn run_check_consistency(args: &[String]) -> i32 {
    let success = crate::run_check_consistency();
    if !success {
        return 1;
    }
    0
}

pub fn run_check_dag(args: &[String]) -> i32 {
    if !crate::run_check_dag() {
        return 1;
    }
    0
}

pub fn run_check_type_registry(args: &[String]) -> i32 {
    let type_name = match args.get(2) {
        Some(s) if !s.trim().is_empty() => s.as_str(),
        _ => {
            eprintln!("Verwendung: cargo xtask check-type-registry <TypName>");
            return 2;
        }
    };
    let success = check_type_registry::run_check_type_registry(type_name);
    if !success {
        return 1;
    }
    0
}

pub fn run_check_stale_tags(args: &[String]) -> i32 {
    let threshold_days = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--threshold-days="))
        .and_then(|val| val.parse::<i64>().ok())
        .unwrap_or(60);
    let strict = args.iter().any(|arg| arg == "--strict");
    if let Err(e) = check_stale_tags::run_check_stale_tags(threshold_days, strict) {
        eprintln!("❌ check-stale-tags failed: {}", e);
        return 1;
    }
    0
}

pub fn run_check_max_results_unbound(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let include_tests = args.iter().any(|arg| arg == "--include-tests");
    match check_max_results_unbound::run_check_max_results_unbound_with_options(
        &root,
        include_tests,
    ) {
        Ok(violations) => {
            if !violations.is_empty() {
                eprintln!(
                    "❌ check-max-results-unbound failed: {} violation(s) found:",
                    violations.len()
                );
                for v in &violations {
                    eprintln!("  {}:{} — {}", v.file_path, v.line_num, v.line_content);
                }
                return 1;
            }
            println!("✅ No violations found (check-max-results-unbound)");
        }
        Err(e) => {
            eprintln!("❌ check-max-results-unbound error: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_toctou_defaults(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let include_tests = args.iter().any(|arg| arg == "--include-tests");
    match check_toctou_trait_defaults::run_check_toctou_trait_defaults_with_options(
        &root,
        include_tests,
    ) {
        Ok(violations) => {
            if !violations.is_empty() {
                eprintln!(
                    "❌ check-toctou-defaults failed: {} violation(s) found:",
                    violations.len()
                );
                for v in &violations {
                    eprintln!(
                        "  {}:{} — trait '{}' has default get+put without lock",
                        v.file_path, v.start_line, v.trait_name
                    );
                }
                return 1;
            }
            println!("✅ No violations found (check-toctou-defaults)");
        }
        Err(e) => {
            eprintln!("❌ check-toctou-defaults error: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_nan_hot_loop(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let include_tests = args.iter().any(|arg| arg == "--include-tests");
    match check_nan_validation_in_hot_loop::run_check_nan_validation_in_hot_loop_with_options(
        &root,
        include_tests,
    ) {
        Ok(violations) => {
            if !violations.is_empty() {
                eprintln!(
                    "❌ check-nan-hot-loop failed: {} violation(s) found:",
                    violations.len()
                );
                for v in &violations {
                    eprintln!("  {}:{} — {}", v.file_path, v.line_num, v.line_content);
                }
                return 1;
            }
            println!("✅ No violations found (check-nan-hot-loop)");
        }
        Err(e) => {
            eprintln!("❌ check-nan-hot-loop error: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_result_dropped_io(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let include_tests = args.iter().any(|arg| arg == "--include-tests");
    match check_result_dropped_on_io::run_check_result_dropped_on_io_with_options(
        &root,
        include_tests,
    ) {
        Ok(violations) => {
            if !violations.is_empty() {
                eprintln!(
                    "❌ check-result-dropped-io failed: {} violation(s) found:",
                    violations.len()
                );
                for v in &violations {
                    eprintln!("  {}:{} — {}", v.file_path, v.line_num, v.line_content);
                }
                return 1;
            }
            println!("✅ No violations found (check-result-dropped-io)");
        }
        Err(e) => {
            eprintln!("❌ check-result-dropped-io error: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_coverage_gate(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let cov_file_arg = args.get(2).map(PathBuf::from);
    let cov_path = cov_file_arg.unwrap_or_else(|| root.join("coverage.json"));
    match check_coverage_gate::run_check_coverage_gate_file(&cov_path) {
        Ok(results) => {
            let mut failed = false;
            println!(
                "{:<20} | {:<15} | {:<12} | Status",
                "Crate", "Coverage (%)", "Threshold (%)"
            );
            println!("{:-<20}-|-{:-<15}-|-{:-<12}-|--------", "", "", "");

            for r in &results {
                let status = if r.passed { "✅ PASS" } else { "❌ FAIL" };
                if !r.passed {
                    failed = true;
                }
                println!(
                    "{:<20} | {:<15.2} | {:<12.2} | {}",
                    r.crate_name, r.actual_coverage, r.threshold, status
                );
            }

            if failed {
                eprintln!("❌ check-coverage-gate failed: one or more crates below threshold.");
                return 1;
            } else {
                println!("✅ check-coverage-gate passed");
            }
        }
        Err(e) => {
            eprintln!("❌ check-coverage-gate error: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_marker_drift(args: &[String]) -> i32 {
    let target_path = args.get(2).map(PathBuf::from);
    if let Err(e) = generate_markers::run_check_marker_drift(target_path.as_deref()) {
        eprintln!("❌ check-marker-drift failed: {}", e);
        return 1;
    }
    println!("✅ check-marker-drift passed");
    0
}

pub fn run_check_ring_capabilities_consistency(args: &[String]) -> i32 {
    match check_ring_capabilities_consistency::run_check_ring_capabilities_consistency() {
        Ok(findings) => {
            if !findings.is_empty() {
                eprintln!("❌ check-ring-capabilities-consistency failed: {} inconsistent crate(s) found:", findings.len());
                for f in &findings {
                    eprintln!(
                        "  {} (capabilities.toml: {}, ring_layering: {:?})",
                        f.crate_name, f.declared_in_capabilities_toml, f.declared_in_ring_layering
                    );
                }
                return 1;
            }
            println!("✅ check-ring-capabilities-consistency passed");
        }
        Err(e) => {
            eprintln!("❌ check-ring-capabilities-consistency failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_check_veto_deadlines(args: &[String]) -> i32 {
    match veto_deadline_gate::run_check_veto_deadlines(chrono::Utc::now()) {
        Ok(entries) => {
            if entries.is_empty() {
                println!("✅ check-veto-deadlines: keine anstehenden VETO-Review-Fristen");
            } else {
                println!("⚠️ check-veto-deadlines: Anstehende VETO-Review-Fristen:");
                for entry in &entries {
                    println!(
                        "  VETO ID: {}, Feature: {}, Status: {}, Due: {:?}",
                        entry.veto_id, entry.feature_id, entry.status, entry.conditional_review_due
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("❌ check-veto-deadlines failed: {}", e);
            return 1;
        }
    }
    0
}
