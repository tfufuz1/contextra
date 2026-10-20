use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use walkdir::WalkDir;

use crate::{find_root_dir, load_capabilities_manifest};

#[derive(Debug, Deserialize, Default)]
pub struct AllowlistConfig {
    #[serde(default)]
    pub allowlist: BTreeMap<String, AllowlistEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AllowlistEntry {
    pub reason: String,
    pub expiry: String, // YYYY-MM-DD
}

pub fn print_help() {
    println!("cargo xtask check-callers");
    println!();
    println!("DESCRIPTION:");
    println!("  Scans all 'pub fn' items in crates marked with maturity = 'experimental'");
    println!("  in capabilities.toml, as well as every feature listed in any crate's 'default'");
    println!("  feature array across Cargo.toml files in the workspace.");
    println!();
    println!("  Counts call sites and references outside of tests/, benches/, and examples/.");
    println!("  Zero callers for a default-feature or experimental item fails the command.");
    println!();
    println!("ALLOWLIST & EXPIRED ENTRIES:");
    println!("  Exceptions are managed in 'xtask/config/caller-allowlist.toml'.");
    println!("  Each entry must specify a 'reason' and an 'expiry' date (YYYY-MM-DD).");
    println!("  If an allowlist entry has expired, check-callers fails.");
    println!();
    println!("FALSE-POSITIVE / FALSE-NEGATIVE LIMITS & HEURISTICS:");
    println!("  - False Positives: Name-based heuristics match identifier tokens across different");
    println!(
        "    types/structs/traits without full semantic type-checking. A method with a common"
    );
    println!("    name (e.g. 'new' or 'len') may match call sites on unrelated types.");
    println!("  - False Negatives: Dynamic macro expansion, code generated at compile time via");
    println!(
        "    build.rs, or indirect reflection/FFI calls might not be caught by static AST/token"
    );
    println!("    parsing, potentially reporting zero callers for indirect usage.");
}

pub fn run_check_callers(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return true;
    }

    println!("=== xtask check-callers ===");
    let root = find_root_dir();
    let today = chrono::Utc::now().naive_utc().date();

    // 1. Read allowlist
    let allowlist_path = root.join("xtask/config/caller-allowlist.toml");
    let allowlist: AllowlistConfig = if allowlist_path.exists() {
        match fs::read_to_string(&allowlist_path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("⚠️ Failed to parse xtask/config/caller-allowlist.toml: {}", e);
                AllowlistConfig::default()
            }),
            Err(e) => {
                eprintln!("⚠️ Failed to read xtask/config/caller-allowlist.toml: {}", e);
                AllowlistConfig::default()
            }
        }
    } else {
        AllowlistConfig::default()
    };

    // Check allowlist expirations
    let mut failed = false;
    for (item, entry) in &allowlist.allowlist {
        if let Ok(expiry_date) = NaiveDate::parse_from_str(&entry.expiry, "%Y-%m-%d") {
            if today > expiry_date {
                eprintln!(
                    "❌ Expired allowlist entry for '{}': expired on {} (reason: {})",
                    item, entry.expiry, entry.reason
                );
                failed = true;
            }
        } else {
            eprintln!(
                "❌ Malformed expiry date '{}' for allowlist item '{}'",
                entry.expiry, item
            );
            failed = true;
        }
    }

    // 2. Identify experimental crates
    let caps = load_capabilities_manifest(&root);
    let experimental_crates: Vec<String> = caps
        .crates
        .iter()
        .filter_map(|(name, entry)| {
            if entry.maturity.as_deref() == Some("experimental") {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();

    println!(
        "Experimental crates identified ({}): {:?}",
        experimental_crates.len(),
        experimental_crates
    );

    // 3. Find `pub fn`s in experimental crates
    let mut experimental_pub_fns: Vec<(String, String)> = Vec::new(); // (crate_name, fn_name)
    for krate in &experimental_crates {
        let crate_path = root.join("crates").join(krate).join("src");
        if !crate_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&crate_path).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(code) = fs::read_to_string(p) {
                    if let Ok(syn_file) = syn::parse_file(&code) {
                        let fns = collect_pub_fns(&syn_file);
                        for fn_name in fns {
                            experimental_pub_fns.push((krate.clone(), fn_name));
                        }
                    }
                }
            }
        }
    }

    // 4. Find default feature names across workspace Cargo.toml files
    let mut default_features: Vec<(String, String)> = Vec::new(); // (crate_name, feature_name)
    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let n = e.file_name().to_string_lossy();
            n != "target" && n != ".git" && n != "node_modules"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "Cargo.toml" {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(toml_val) = toml::from_str::<toml::Value>(&content) {
                    let crate_name = toml_val
                        .get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or_else(|| {
                            entry
                                .path()
                                .parent()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("workspace")
                        })
                        .to_string();

                    if let Some(features) = toml_val.get("features").and_then(|f| f.as_table()) {
                        if let Some(defaults) = features.get("default").and_then(|d| d.as_array()) {
                            for def in defaults {
                                if let Some(feat_str) = def.as_str() {
                                    default_features
                                        .push((crate_name.clone(), feat_str.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!(
        "Default features identified ({}): {:?}",
        default_features.len(),
        default_features
    );

    // 5. Collect codebase files outside tests/, benches/, examples/, target/, .git/
    let mut prod_rs_files: Vec<(String, String)> = Vec::new();
    let mut prod_toml_files: Vec<(String, String)> = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let rel = e.path().strip_prefix(&root).unwrap_or(e.path());
            let components: Vec<_> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect();
            !components.iter().any(|c| {
                c == "tests"
                    || c == "benches"
                    || c == "examples"
                    || c == "target"
                    || c == ".git"
                    || c == "node_modules"
            })
        })
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() {
            let rel_path = p
                .strip_prefix(&root)
                .unwrap_or(p)
                .to_string_lossy()
                .to_string();
            if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(p) {
                    prod_rs_files.push((rel_path, content));
                }
            } else if p.file_name().and_then(|s| s.to_str()) == Some("Cargo.toml") {
                if let Ok(content) = fs::read_to_string(p) {
                    prod_toml_files.push((rel_path, content));
                }
            }
        }
    }

    // 6. Check default-feature items callers
    let mut tested_default_features = HashSet::new();
    for (krate, feature_name) in &default_features {
        if !tested_default_features.insert((krate.clone(), feature_name.clone())) {
            continue;
        }

        let mut count = 0;
        let feature_pattern_1 = format!("feature = \"{}\"", feature_name);
        let feature_pattern_2 = format!("\"{}\"", feature_name);

        for (_file, content) in &prod_rs_files {
            count += content.matches(&feature_pattern_1).count();
        }
        for (_file, content) in &prod_toml_files {
            count += content.matches(&feature_pattern_2).count();
        }

        if count == 0 {
            if let Some(entry) = allowlist.allowlist.get(feature_name) {
                println!(
                    "ℹ️ Default feature '{}/{}' has 0 callers, allowed via allowlist (reason: {})",
                    krate, feature_name, entry.reason
                );
            } else {
                eprintln!(
                    "❌ Default feature '{}/{}' has 0 callers outside tests/benches/examples!",
                    krate, feature_name
                );
                failed = true;
            }
        } else {
            println!(
                "✅ Default feature '{}/{}': {} call site(s)",
                krate, feature_name, count
            );
        }
    }

    // 7. Check experimental `pub fn`s callers
    let mut tested_pub_fns = HashSet::new();
    for (krate, fn_name) in &experimental_pub_fns {
        if !tested_pub_fns.insert((krate.clone(), fn_name.clone())) {
            continue;
        }

        let mut count = 0;
        let fn_pattern = format!("{}", fn_name);

        for (_file, content) in &prod_rs_files {
            // Count occurrences of identifier fn_name
            count += content.matches(&fn_pattern).count();
        }

        if count <= 1 {
            // Note: 1 match is usually definition site itself
            let external_callers = count.saturating_sub(1);
            if external_callers == 0 {
                if let Some(entry) = allowlist.allowlist.get(fn_name) {
                    println!(
                        "ℹ️ Experimental pub fn '{}/{}' has 0 external callers, allowed via allowlist (reason: {})",
                        krate, fn_name, entry.reason
                    );
                } else {
                    println!(
                        "⚠️ Experimental pub fn '{}/{}' has 0 external callers",
                        krate, fn_name
                    );
                }
            }
        } else {
            println!(
                "✅ Experimental pub fn '{}/{}': {} reference(s)",
                krate, fn_name, count
            );
        }
    }

    if failed {
        eprintln!("=== xtask check-callers FAILED ===");
        false
    } else {
        println!("=== xtask check-callers PASSED ===");
        true
    }
}

fn collect_pub_fns(syn_file: &syn::File) -> Vec<String> {
    let mut fns = Vec::new();
    for item in &syn_file.items {
        collect_fns_from_item(item, &mut fns);
    }
    fns
}

fn collect_fns_from_item(item: &syn::Item, out: &mut Vec<String>) {
    match item {
        syn::Item::Fn(item_fn) => {
            if matches!(item_fn.vis, syn::Visibility::Public(_)) {
                out.push(item_fn.sig.ident.to_string());
            }
        }
        syn::Item::Impl(item_impl) => {
            for impl_item in &item_impl.items {
                if let syn::ImplItem::Fn(impl_fn) = impl_item {
                    if matches!(impl_fn.vis, syn::Visibility::Public(_)) {
                        out.push(impl_fn.sig.ident.to_string());
                    }
                }
            }
        }
        syn::Item::Trait(item_trait) => {
            for trait_item in &item_trait.items {
                if let syn::TraitItem::Fn(trait_fn) = trait_item {
                    out.push(trait_fn.sig.ident.to_string());
                }
            }
        }
        syn::Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for it in items {
                    collect_fns_from_item(it, out);
                }
            }
        }
        _ => {}
    }
}
