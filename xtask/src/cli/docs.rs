#![allow(unused_imports, dead_code, unused_variables)]
use crate::*;
use std::path::{Path, PathBuf};
use std::process;

pub fn run_sync_docs(args: &[String]) -> i32 {
    let check_only = args.iter().any(|arg| arg == "--check");
    let success = crate::run_sync_docs(check_only);
    if !success {
        return 1;
    }
    0
}

pub fn run_gen_arch_docs(args: &[String]) -> i32 {
    let check_only = args.iter().any(|arg| arg == "--check");
    if let Err(e) = gen_arch_docs::run_gen_arch_docs(check_only) {
        eprintln!("❌ gen-arch-docs failed: {}", e);
        return 1;
    }
    0
}

pub fn run_gen_feature_catalog(args: &[String]) -> i32 {
    if let Err(e) = gen_feature_catalog::run_gen_feature_catalog() {
        eprintln!("❌ gen-feature-catalog failed: {}", e);
        return 1;
    }
    0
}

pub fn run_generate_adr(args: &[String]) -> i32 {
    let title = args.get(2).map(|s| s.as_str()).unwrap_or("Untitled");
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    match generate_adr::run_generate_adr(title, dry_run) {
        Ok(res) => {
            println!(
                "✅ ADR-{:03} erfolgreich in {} registriert",
                res.number,
                res.path.display()
            );
        }
        Err(e) => {
            eprintln!("❌ generate-adr failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_consolidate_adrs(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    if let Err(e) = generate_adr::consolidate_decisions(&root) {
        eprintln!("❌ consolidate-adrs failed: {}", e);
        return 1;
    }
    0
}
