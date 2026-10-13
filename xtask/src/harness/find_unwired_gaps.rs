//! Harness Modul zur Lücken- und Verdrahtungsanalyse im Workspace (find-unwired-gaps).

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const UNSAFE_ISLANDS: &[&str] = &["contextra-sys", "contextra-simd", "contextra-wire"];
const SKIP_DIR_NAMES: &[&str] = &["target", "tests", "benches", "examples", ".git"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapSymbol {
    pub name: String,
    pub kind: String,
    pub krate: String,
    pub file: String,
    pub documented: bool,
    pub is_unsafe_island: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUnwiredGapsReport {
    pub files_scanned: usize,
    pub total_pub_symbols: usize,
    pub undocumented_count: usize,
    pub no_caller_count: usize,
    pub undocumented: Vec<GapSymbol>,
    pub no_caller: Vec<GapSymbol>,
}

fn iter_rs_files(repo_root: &Path) -> Vec<PathBuf> {
    let crates_dir = repo_root.join("crates");
    let mut files = Vec::new();
    if !crates_dir.exists() {
        return files;
    }
    walk_dir(&crates_dir, &mut files);
    files
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if !SKIP_DIR_NAMES.contains(&name) {
                walk_dir(&path, files);
            }
        } else if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
}

fn crate_of(repo_root: &Path, file_path: &Path) -> String {
    let crates_dir = repo_root.join("crates");
    if let Ok(rel) = file_path.strip_prefix(&crates_dir) {
        if let Some(first) = rel.components().next() {
            return first.as_os_str().to_string_lossy().to_string();
        }
    }
    "unknown".to_string()
}

fn has_doc_above(src: &str, start_idx: usize) -> bool {
    let before = &src[..start_idx].trim_end_matches('\n');
    let lines: Vec<&str> = before.lines().collect();
    let mut i = lines.len();
    while i > 0 {
        i -= 1;
        let s = lines[i].trim();
        if s.is_empty() || s.starts_with("#[") {
            continue;
        }
        return s.starts_with("///");
    }
    false
}

pub fn run_find_unwired_gaps(args: &[String]) -> i32 {
    let mut repo_root = PathBuf::from(".");
    let mut out_dir = PathBuf::from("docs/gaps");
    let mut json_format = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                if i + 1 < args.len() {
                    repo_root = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--out-dir" => {
                if i + 1 < args.len() {
                    out_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--json" => {
                json_format = true;
            }
            _ => {}
        }
        i += 1;
    }

    let files = iter_rs_files(&repo_root);
    if files.is_empty() {
        eprintln!("[error] find-unwired-gaps: keine crates/**/*.rs gefunden");
        return 1;
    }

    let mut sources: HashMap<PathBuf, String> = HashMap::new();
    for fp in &files {
        if let Ok(content) = fs::read_to_string(fp) {
            sources.insert(fp.clone(), content);
        }
    }

    let pub_fn_re =
        Regex::new(r"\n(\s*)pub(?:\(crate\))?\s+(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)").unwrap();
    let pub_item_re = Regex::new(r"\n(\s*)pub\s+(struct|enum|trait|type|const)\s+(\w+)").unwrap();

    let mut symbols: Vec<GapSymbol> = Vec::new();

    for (fp, src) in &sources {
        let path_str = fp.to_string_lossy();
        if path_str.contains("/tests/") || path_str.ends_with("_test.rs") {
            continue;
        }

        let krate = crate_of(&repo_root, fp);
        let rel_path = fp
            .strip_prefix(&repo_root)
            .unwrap_or(fp)
            .to_string_lossy()
            .to_string();
        let is_island = UNSAFE_ISLANDS.contains(&krate.as_str());

        for m in pub_fn_re.captures_iter(src) {
            if let Some(full_match) = m.get(0) {
                let match_str = full_match.as_str();
                if match_str.contains("pub(crate)") {
                    continue;
                }
            }
            if let Some(name_m) = m.get(2) {
                let name = name_m.as_str().to_string();
                let doc = has_doc_above(src, m.get(0).unwrap().start());
                symbols.push(GapSymbol {
                    name,
                    kind: "fn".to_string(),
                    krate: krate.clone(),
                    file: rel_path.clone(),
                    documented: doc,
                    is_unsafe_island: is_island,
                });
            }
        }

        for m in pub_item_re.captures_iter(src) {
            if let (Some(kind_m), Some(name_m)) = (m.get(2), m.get(3)) {
                let kind = kind_m.as_str().to_string();
                let name = name_m.as_str().to_string();
                let doc = has_doc_above(src, m.get(0).unwrap().start());
                symbols.push(GapSymbol {
                    name,
                    kind,
                    krate: krate.clone(),
                    file: rel_path.clone(),
                    documented: doc,
                    is_unsafe_island: is_island,
                });
            }
        }
    }

    let fn_names: HashSet<String> = symbols
        .iter()
        .filter(|s| s.kind == "fn")
        .map(|s| s.name.clone())
        .collect();

    let call_token_re = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap();
    let mut caller_count: HashMap<String, usize> = HashMap::new();

    for src in sources.values() {
        for cap in call_token_re.captures_iter(src) {
            if let Some(m) = cap.get(1) {
                let token = m.as_str();
                if fn_names.contains(token) {
                    *caller_count.entry(token.to_string()).or_default() += 1;
                }
            }
        }
    }

    let mut undocumented: Vec<GapSymbol> = symbols
        .iter()
        .filter(|s| !s.documented)
        .cloned()
        .collect();

    let mut no_caller: Vec<GapSymbol> = symbols
        .iter()
        .filter(|s| s.kind == "fn" && *caller_count.get(&s.name).unwrap_or(&0) <= 1)
        .cloned()
        .collect();

    undocumented.sort_by(|a, b| {
        a.krate
            .cmp(&b.krate)
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.name.cmp(&b.name))
    });

    no_caller.sort_by(|a, b| {
        a.krate
            .cmp(&b.krate)
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.name.cmp(&b.name))
    });

    let _ = fs::create_dir_all(&out_dir);

    // CSV: undokumentiert_streng.csv
    let undoc_csv_path = out_dir.join("undokumentiert_streng.csv");
    let mut undoc_csv = String::from("crate,kind,name,file,unsafe_insel\n");
    for s in &undocumented {
        undoc_csv.push_str(&format!(
            "{},{},{},{},{}\n",
            s.krate,
            s.kind,
            s.name,
            s.file,
            if s.is_unsafe_island { "ja" } else { "nein" }
        ));
    }
    let _ = fs::write(&undoc_csv_path, undoc_csv);

    // CSV: pub_fn_ohne_aufrufer.csv
    let nocaller_csv_path = out_dir.join("pub_fn_ohne_aufrufer.csv");
    let mut nocaller_csv = String::from("crate,name,file,unsafe_insel\n");
    for s in &no_caller {
        nocaller_csv.push_str(&format!(
            "{},{},{},{}\n",
            s.krate,
            s.name,
            s.file,
            if s.is_unsafe_island { "ja" } else { "nein" }
        ));
    }
    let _ = fs::write(&nocaller_csv_path, nocaller_csv);

    if json_format {
        let report = FindUnwiredGapsReport {
            files_scanned: files.len(),
            total_pub_symbols: symbols.len(),
            undocumented_count: undocumented.len(),
            no_caller_count: no_caller.len(),
            undocumented,
            no_caller,
        };
        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
    } else {
        println!(
            "[ok] {} Dateien, {} pub-Symbole analysiert",
            files.len(),
            symbols.len()
        );
        println!(
            "[ok] {} undokumentiert, {} ohne Aufrufer (CSV in {})",
            undocumented.len(),
            no_caller.len(),
            out_dir.display()
        );

        let mut by_crate_nocaller: HashMap<String, usize> = HashMap::new();
        for s in &no_caller {
            *by_crate_nocaller.entry(s.krate.clone()).or_default() += 1;
        }

        let mut sorted_crates: Vec<(String, usize)> = by_crate_nocaller.into_iter().collect();
        sorted_crates.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\nTop-Crates (ohne Aufrufer):");
        for (c, n) in sorted_crates.iter().take(10) {
            let flag = if UNSAFE_ISLANDS.contains(&c.as_str()) {
                " [UNSAFE-INSEL]"
            } else {
                ""
            };
            println!("  {}: {}{}", c, n, flag);
        }
    }

    0
}
