use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossFileDuplicate {
    pub dir: String,
    pub symbol_kind: String,
    pub symbol_name: String,
    pub file1: String,
    pub line1: usize,
    pub file2: String,
    pub line2: usize,
}

#[derive(Debug, Clone)]
struct SymbolOccurrence {
    file: String,
    line: usize,
    kind: String,
    cfg_attr: Option<String>,
}

fn get_pub_item_info(
    item: &syn::Item,
) -> Option<(
    &'static str,
    &syn::Ident,
    &syn::Visibility,
    &[syn::Attribute],
)> {
    match item {
        syn::Item::Const(i) => Some(("const", &i.ident, &i.vis, &i.attrs)),
        syn::Item::Enum(i) => Some(("enum", &i.ident, &i.vis, &i.attrs)),
        syn::Item::Fn(i) => Some(("fn", &i.sig.ident, &i.vis, &i.attrs)),
        syn::Item::Static(i) => Some(("static", &i.ident, &i.vis, &i.attrs)),
        syn::Item::Struct(i) => Some(("struct", &i.ident, &i.vis, &i.attrs)),
        syn::Item::Trait(i) => Some(("trait", &i.ident, &i.vis, &i.attrs)),
        syn::Item::Type(i) => Some(("type", &i.ident, &i.vis, &i.attrs)),
        _ => None,
    }
}

fn extract_cfg_attr(attrs: &[syn::Attribute]) -> Option<String> {
    let cfgs: Vec<String> = attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .map(|a| quote::quote!(#a).to_string())
        .collect();
    if cfgs.is_empty() {
        None
    } else {
        Some(cfgs.join(" && "))
    }
}

/// Scans a single directory for top-level public symbol duplicates across different `.rs` files.
pub fn scan_directory_for_cross_file_duplicates(
    dir: &Path,
) -> Result<Vec<CrossFileDuplicate>, String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    let mut rs_files: Vec<PathBuf> = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
            rs_files.push(p);
        }
    }

    if rs_files.len() < 2 {
        return Ok(Vec::new());
    }

    rs_files.sort();

    let mut symbol_occurrences: HashMap<String, Vec<SymbolOccurrence>> = HashMap::new();

    for file_path in &rs_files {
        let file_str = file_path.to_string_lossy().to_string();
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let syn_file = match syn::parse_file(&content) {
            Ok(f) => f,
            Err(_) => continue,
        };

        for item in syn_file.items {
            if let Some((kind, ident, vis, attrs)) = get_pub_item_info(&item) {
                if !matches!(vis, syn::Visibility::Inherited) {
                    let symbol_name = ident.to_string();
                    if symbol_name != "_" {
                        let line = ident.span().start().line;
                        let cfg_attr = extract_cfg_attr(attrs);

                        symbol_occurrences
                            .entry(symbol_name)
                            .or_default()
                            .push(SymbolOccurrence {
                                file: file_str.clone(),
                                line,
                                kind: kind.to_string(),
                                cfg_attr,
                            });
                    }
                }
            }
        }
    }

    let mut duplicates = Vec::new();
    let dir_str = dir.to_string_lossy().to_string();

    for (symbol_name, occs) in symbol_occurrences {
        // Group by cfg_attr
        let mut cfg_groups: HashMap<Option<String>, Vec<SymbolOccurrence>> = HashMap::new();
        for occ in occs {
            cfg_groups
                .entry(occ.cfg_attr.clone())
                .or_default()
                .push(occ);
        }

        for (_cfg, group_occs) in cfg_groups {
            // Check if occurrences span multiple distinct files
            let mut file_map: HashMap<String, Vec<&SymbolOccurrence>> = HashMap::new();
            for occ in &group_occs {
                file_map.entry(occ.file.clone()).or_default().push(occ);
            }

            if file_map.len() > 1 {
                let mut sorted_files: Vec<String> = file_map.keys().cloned().collect();
                sorted_files.sort();

                let first_file = &sorted_files[0];
                let first_occ = &file_map[first_file][0];

                for other_file in &sorted_files[1..] {
                    for other_occ in &file_map[other_file] {
                        duplicates.push(CrossFileDuplicate {
                            dir: dir_str.clone(),
                            symbol_kind: first_occ.kind.clone(),
                            symbol_name: symbol_name.clone(),
                            file1: first_occ.file.clone(),
                            line1: first_occ.line,
                            file2: other_occ.file.clone(),
                            line2: other_occ.line,
                        });
                    }
                }
            }
        }
    }

    duplicates.sort_by(|a, b| {
        a.file1
            .cmp(&b.file1)
            .then_with(|| a.line1.cmp(&b.line1))
            .then_with(|| a.file2.cmp(&b.file2))
            .then_with(|| a.line2.cmp(&b.line2))
            .then_with(|| a.symbol_name.cmp(&b.symbol_name))
    });

    Ok(duplicates)
}

/// Recursively scans workspace root directories for cross-file duplicate public top-level symbols in the same module directory.
pub fn scan_workspace_for_cross_file_duplicates(
    root_dirs: &[&Path],
) -> Result<Vec<CrossFileDuplicate>, String> {
    let mut dirs_to_scan: Vec<PathBuf> = Vec::new();

    for root in root_dirs {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Some(parent) = path.parent() {
                    dirs_to_scan.push(parent.to_path_buf());
                }
            }
        }
    }

    dirs_to_scan.sort();
    dirs_to_scan.dedup();

    let mut all_duplicates = Vec::new();

    for dir in dirs_to_scan {
        if let Ok(dups) = scan_directory_for_cross_file_duplicates(&dir) {
            all_duplicates.extend(dups);
        }
    }

    all_duplicates.sort_by(|a, b| {
        a.dir
            .cmp(&b.dir)
            .then_with(|| a.file1.cmp(&b.file1))
            .then_with(|| a.line1.cmp(&b.line1))
            .then_with(|| a.file2.cmp(&b.file2))
            .then_with(|| a.line2.cmp(&b.line2))
            .then_with(|| a.symbol_name.cmp(&b.symbol_name))
    });

    Ok(all_duplicates)
}

/// Main entry point for `xtask check-duplicate-symbols-cross-file`.
#[allow(dead_code)]
pub fn run() -> Result<(), String> {
    let root_dirs = [
        Path::new("crates"),
        Path::new("xtask"),
        Path::new("benchmarks"),
    ];

    let duplicates = scan_workspace_for_cross_file_duplicates(&root_dirs)?;

    if duplicates.is_empty() {
        println!("✅ check-duplicate-symbols-cross-file: keine dateiübergreifenden Duplikate im selben Modulverzeichnis gefunden.");
        Ok(())
    } else {
        eprintln!(
            "⚠️ check-duplicate-symbols-cross-file: {} dateiübergreifende(s) Duplikat(e) im selben Modulverzeichnis gefunden:",
            duplicates.len()
        );
        for d in &duplicates {
            eprintln!(
                "  [{}] {}:{} und {}:{} — doppeltes {} '{}'",
                d.dir, d.file1, d.line1, d.file2, d.line2, d.symbol_kind, d.symbol_name
            );
        }

        eprintln!("⚠️ WARNUNG: Gefundene Duplikate verstoßen gegen Governance GOV-D. Bitte in einem Folge-PR beheben.");
        Ok(())
    }
}
