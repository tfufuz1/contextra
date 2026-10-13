//! Harness Modul zur Erzeugung der Schnittstellenspezifikation (generate-interface-spec).

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct CrateMeta {
    ring: usize,
    status: &'static str,
    purpose: &'static str,
}

fn get_ring_meta() -> HashMap<&'static str, CrateMeta> {
    let mut m = HashMap::new();
    m.insert("contextra-types", CrateMeta { ring: 0, status: "Fertig", purpose: "Kanonische Domänenmodelle, IDs, Budgets, Filter-AST und Fehlertypen." });
    m.insert("contextra-ports", CrateMeta { ring: 0, status: "Fertig", purpose: "dyn-kompatible Trait-Definitionen (Ports) für Storage, Indizes, Embedder und System-Uhren." });
    m.insert("contextra-mvcc", CrateMeta { ring: 0, status: "Fertig", purpose: "SeqLog, SnapshotRegistry und transaktionaler TxBuffer (MVCC-Isolation)." });
    m.insert("contextra-wire", CrateMeta { ring: 0, status: "Fertig", purpose: "FlatBuffers-IPC-Generat und Zero-Copy-Adapter (Unsafe-Insel)." });
    m.insert("contextra-sys", CrateMeta { ring: 0, status: "Fertig", purpose: "System-Abstraktionen für mmap, mlock und Win32-ACLs (Unsafe-Insel)." });
    m.insert("contextra-simd", CrateMeta { ring: 0, status: "Fertig", purpose: "AVX2/AVX-512/NEON SIMD-Distanzberechnungskerne mit Laufzeit-Dispatch (Unsafe-Insel)." });
    m.insert("contextra-crypto", CrateMeta { ring: 0, status: "In Migration", purpose: "AES-256-GCM-SIV, WAL-HMAC-Integritätsketten, Zeroize und Deletion-Proofs." });
    m.insert("contextra-vector", CrateMeta { ring: 0, status: "In Migration", purpose: "HNSW- und DiskANN-Vektorindex-Implementierungen." });
    m.insert("contextra-text", CrateMeta { ring: 0, status: "Fertig", purpose: "BM25/BM25F-Volltextindexierung und deutsche Kompositazerlegung." });
    m.insert("contextra-graph", CrateMeta { ring: 0, status: "Fertig", purpose: "Compressed Sparse Row (CSR) Graph, Forward-Push PPR, Leiden und Hyperkanten." });
    m.insert("contextra-rank", CrateMeta { ring: 0, status: "In Migration", purpose: "4-Signal-Retrieval-Fusion, Score-Kalibrierung und Drift-Messung." });
    m.insert("contextra-adapt", CrateMeta { ring: 0, status: "Fertig", purpose: "LinUCB-Bandit-Mathematik, Lyapunov-Drift-Wächter und PID-Regler." });
    m.insert("contextra-core", CrateMeta { ring: 0, status: "Fertig (Strangler Facade)", purpose: "Re-Export-Fassade über types, ports, mvcc und wire zur Abwärtskompatibilität." });
    m.insert("contextra-audit-export", CrateMeta { ring: 0, status: "Fertig", purpose: "Generator für Verarbeitungsverzeichnisse gemäß DSGVO Art. 30." });
    m.insert("contextra-avv-generator", CrateMeta { ring: 0, status: "Fertig", purpose: "Template-Generator für Auftragsverarbeitungsverträge (AVV)." });
    m.insert("contextra-license", CrateMeta { ring: 0, status: "Fertig", purpose: "Lizenzierungs- und Aktivierungsprüfung für Feature-Ringe." });
    m.insert("contextra-store", CrateMeta { ring: 1, status: "Fertig", purpose: "LSM-Tree Storage Engine mit WAL-Group-Commit und HMAC-Integritätsprüfung." });
    m.insert("contextra-kvcache", CrateMeta { ring: 1, status: "Fertig", purpose: "Verschlüsselter Prefix-Radix-Baum und KV-Cache-Segment-Verwaltung." });
    m.insert("contextra-checkpoint", CrateMeta { ring: 1, status: "Fertig", purpose: "Time-Travel-Registry und Checkpoint-Verwaltung ohne globalen Zustand." });
    m.insert("contextra-infer-candle", CrateMeta { ring: 2, status: "In Migration", purpose: "Native GGUF-Modell-Inferenz via Candle." });
    m.insert("contextra-infer-ollama", CrateMeta { ring: 2, status: "In Migration", purpose: "HTTP-Inferenz-Client für Ollama mit Contextual-Chunk-Prefixing." });
    m.insert("contextra-infer-onnx", CrateMeta { ring: 2, status: "In Migration", purpose: "ONNX-Embeddings und Reranking via ort." });
    m.insert("contextra-sandbox", CrateMeta { ring: 2, status: "Fertig", purpose: "WASM-Ausführungs-Isolation mit Fuel- und Wall-Clock-Limits via Wasmtime." });
    m.insert("contextra-engine", CrateMeta { ring: 3, status: "In Migration", purpose: "Collection-LSM-Anbindung, Multi-Index-Pläne und Schreibtransaktionen." });
    m.insert("contextra-cognition", CrateMeta { ring: 3, status: "In Migration", purpose: "Hintergrund-Kompaktierung, Synthese und Konsolidierungs-Scheduler." });
    m.insert("contextra-privacy", CrateMeta { ring: 3, status: "In Migration", purpose: "Cloud-Egress Privacy Gateway, PII-Vault, Surrogat-Tokenisierung und DLP." });
    m.insert("contextra-router", CrateMeta { ring: 3, status: "In Migration", purpose: "SLM-Profil-Routing und MCP-Dispatch." });
    m.insert("contextra-agent", CrateMeta { ring: 3, status: "Fertig", purpose: "Agenten-Workflow-Engine mit auditierbarer State-Machine und DLQ." });
    m.insert("contextra-db", CrateMeta { ring: 3, status: "In Migration (Legacy-Fassade)", purpose: "Monolithische Übergangsfassade für Speicher-, Such- und Indexzugriffe." });
    m.insert("contextra", CrateMeta { ring: 4, status: "Fertig", purpose: "Öffentliche Haupt-Fassade und Composition Root für Rust-Anwendungen." });
    m.insert("contextra-mcp", CrateMeta { ring: 4, status: "In Migration", purpose: "Stdio-JSON-RPC MCP-Server-Protokoll-Adapter." });
    m.insert("contextra-py", CrateMeta { ring: 4, status: "In Migration", purpose: "PyO3 Python-FFI-Bindings." });
    m.insert("contextra-testkit", CrateMeta { ring: 99, status: "Fertig (Tooling)", purpose: "Deterministische Test-Utilities (ManualClock, InMemoryStorageEngine, FaultVfs)." });
    m
}

fn get_ring_names() -> HashMap<usize, &'static str> {
    let mut m = HashMap::new();
    m.insert(0, "Ring 0 — Foundation & Pure Numerics/Types");
    m.insert(1, "Ring 1 — Persistence & Cache");
    m.insert(2, "Ring 2 — External Adapters & Sandboxing (Leaf Engines)");
    m.insert(3, "Ring 3 — Orchestration & Cognition");
    m.insert(4, "Ring 4 — Boundary & Composition Roots");
    m.insert(99, "Tooling (Cross-Ring)");
    m
}

const CRATE_ORDER: &[&str] = &[
    "contextra-types", "contextra-ports", "contextra-mvcc", "contextra-wire", "contextra-sys",
    "contextra-simd", "contextra-crypto", "contextra-vector", "contextra-text", "contextra-graph",
    "contextra-rank", "contextra-adapt", "contextra-core", "contextra-audit-export",
    "contextra-avv-generator", "contextra-license",
    "contextra-store", "contextra-kvcache", "contextra-checkpoint",
    "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx", "contextra-sandbox",
    "contextra-engine", "contextra-cognition", "contextra-privacy", "contextra-router",
    "contextra-agent", "contextra-db",
    "contextra", "contextra-mcp", "contextra-py",
    "contextra-testkit",
];

const UNSAFE_ISLANDS: &[&str] = &["contextra-simd", "contextra-sys", "contextra-wire"];

#[derive(Debug, Clone)]
struct SpecItem {
    kind: String,
    name: String,
    generics: String,
    bounds: String,
    sig: String,
    body: Option<String>,
    doc: String,
    file: String,
}

fn strip_tests(src: &str) -> &str {
    let cfg_test_re = Regex::new(r"\n#\[cfg\(test\)\]\s*\nmod\s+tests\b").unwrap();
    if let Some(m) = cfg_test_re.find(src) {
        return &src[..m.start()];
    }
    let mod_tests_re = Regex::new(r"\nmod\s+tests\s*\{").unwrap();
    if let Some(m) = mod_tests_re.find(src) {
        return &src[..m.start()];
    }
    src
}

fn find_matching_brace(src: &str, start_idx: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in src[start_idx..].char_indices() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(start_idx + i);
            }
        }
    }
    None
}

fn get_preceding_doc(src: &str, start_idx: usize) -> (Vec<String>, Vec<String>) {
    let before = &src[..start_idx];
    let lines_before: Vec<&str> = before.lines().collect();
    let mut doc_lines = Vec::new();
    let mut attr_lines = Vec::new();
    let mut i = lines_before.len();

    while i > 0 {
        i -= 1;
        let line = lines_before[i].trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("///") {
            doc_lines.insert(0, line[3..].trim().to_string());
            continue;
        }
        if line.starts_with("#[") {
            attr_lines.insert(0, line.to_string());
            continue;
        }
        if line.starts_with("//!") {
            continue;
        }
        break;
    }

    (doc_lines, attr_lines)
}

fn clamp_to_char_boundary(src: &str, mut idx: usize) -> usize {
    if idx >= src.len() {
        return src.len();
    }
    while !src.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

fn extract_items(filepath: &Path, crate_root: &Path) -> Vec<SpecItem> {
    let src_raw = match fs::read_to_string(filepath) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let src = strip_tests(&src_raw);
    let rel = filepath
        .strip_prefix(crate_root)
        .unwrap_or(filepath)
        .to_string_lossy()
        .to_string();

    let mut items = Vec::new();

    // PUB TRAIT
    let trait_re =
        Regex::new(r"\n(\s*)pub\s+trait\s+(\w+)((?:<[^{]*?>)?)\s*((?::[^{]*?)?)\s*\{").unwrap();
    for cap in trait_re.captures_iter(src) {
        let m = cap.get(0).unwrap();
        let indent = cap.get(1).unwrap().as_str();
        let name = cap.get(2).unwrap().as_str().to_string();
        let generics = cap.get(3).unwrap().as_str().trim().to_string();
        let bounds = cap.get(4).unwrap().as_str().trim().to_string();

        let brace_start = m.end() - 1;
        let body = if let Some(brace_end) = find_matching_brace(src, brace_start) {
            src[brace_start + 1..brace_end].trim().to_string()
        } else {
            String::new()
        };

        let doc_idx = clamp_to_char_boundary(src, m.start() + indent.len() + 1);
        let (doc, _) = get_preceding_doc(src, doc_idx);

        items.push(SpecItem {
            kind: "trait".to_string(),
            name,
            generics,
            bounds,
            sig: String::new(),
            body: Some(body),
            doc: doc.join(" "),
            file: rel.clone(),
        });
    }

    // PUB STRUCT
    let struct_re = Regex::new(r"\n(\s*)pub\s+struct\s+(\w+)([^;{]*)").unwrap();
    for cap in struct_re.captures_iter(src) {
        let m = cap.get(0).unwrap();
        let indent = cap.get(1).unwrap().as_str();
        let name = cap.get(2).unwrap().as_str().to_string();
        let rest = cap.get(3).unwrap().as_str().trim().to_string();

        let doc_idx = clamp_to_char_boundary(src, m.start() + indent.len() + 1);
        let (doc, _) = get_preceding_doc(src, doc_idx);

        let end_search = clamp_to_char_boundary(src, m.end() + 2000);
        let forward = &src[m.end()..end_search];
        let semi_idx = forward.find(';');
        let brace_idx = forward.find('{');

        let body = if let Some(bi) = brace_idx {
            if semi_idx.map_or(true, |si| bi < si) {
                let abs_brace = m.end() + bi;
                find_matching_brace(src, abs_brace)
                    .map(|be| src[abs_brace + 1..be].to_string())
            } else {
                None
            }
        } else {
            None
        };

        items.push(SpecItem {
            kind: "struct".to_string(),
            name,
            generics: String::new(),
            bounds: String::new(),
            sig: rest,
            body,
            doc: doc.join(" "),
            file: rel.clone(),
        });
    }

    // PUB ENUM
    let enum_re = Regex::new(r"\n(\s*)pub\s+enum\s+(\w+)([^{]*)\{").unwrap();
    for cap in enum_re.captures_iter(src) {
        let m = cap.get(0).unwrap();
        let indent = cap.get(1).unwrap().as_str();
        let name = cap.get(2).unwrap().as_str().to_string();
        let generics = cap.get(3).unwrap().as_str().trim().to_string();

        let doc_idx = clamp_to_char_boundary(src, m.start() + indent.len() + 1);
        let (doc, _) = get_preceding_doc(src, doc_idx);
        let brace_start = m.end() - 1;
        let body = find_matching_brace(src, brace_start)
            .map(|be| src[brace_start + 1..be].trim().to_string());

        items.push(SpecItem {
            kind: "enum".to_string(),
            name,
            generics,
            bounds: String::new(),
            sig: String::new(),
            body,
            doc: doc.join(" "),
            file: rel.clone(),
        });
    }

    // PUB FN
    let fn_re =
        Regex::new(r"\n(\s*)pub(?:\(crate\))?\s+(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)").unwrap();
    for cap in fn_re.captures_iter(src) {
        let m = cap.get(0).unwrap();
        let indent = cap.get(1).unwrap().as_str();
        let match_str = m.as_str();
        if match_str.contains("pub(crate)") {
            continue;
        }

        let name = cap.get(2).unwrap().as_str().to_string();
        let doc_idx = clamp_to_char_boundary(src, m.start() + indent.len() + 1);
        let (doc, _) = get_preceding_doc(src, doc_idx);

        if let Some(paren_start) = src[m.end()..].find('(') {
            let abs_paren_start = m.end() + paren_start;
            let mut depth = 0;
            let mut end_idx = abs_paren_start;
            for (idx, c) in src[abs_paren_start..].char_indices() {
                if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = abs_paren_start + idx;
                        break;
                    }
                }
            }

            let search_end = clamp_to_char_boundary(src, end_idx + 400);
            let forward_search = &src[end_idx + 1..search_end];
            let term_idx = if let Some(idx) = forward_search.find(|c| c == '{' || c == ';') {
                end_idx + 1 + idx
            } else {
                end_idx + 1
            };

            let sig_start = doc_idx;
            let raw_sig = &src[sig_start..term_idx];
            let sig = raw_sig.split_whitespace().collect::<Vec<_>>().join(" ");

            items.push(SpecItem {
                kind: "fn".to_string(),
                name,
                generics: String::new(),
                bounds: String::new(),
                sig,
                body: None,
                doc: doc.join(" "),
                file: rel.clone(),
            });
        }
    }

    // PUB TYPE
    let type_re = Regex::new(r"\n(\s*)pub\s+type\s+(\w+)([^;]*);").unwrap();
    for cap in type_re.captures_iter(src) {
        let m = cap.get(0).unwrap();
        let indent = cap.get(1).unwrap().as_str();
        let name = cap.get(2).unwrap().as_str().to_string();
        let rest = cap.get(3).unwrap().as_str().trim().to_string();
        let doc_idx = clamp_to_char_boundary(src, m.start() + indent.len() + 1);
        let (doc, _) = get_preceding_doc(src, doc_idx);

        items.push(SpecItem {
            kind: "type".to_string(),
            name,
            generics: String::new(),
            bounds: String::new(),
            sig: rest.split_whitespace().collect::<Vec<_>>().join(" "),
            body: None,
            doc: doc.join(" "),
            file: rel.clone(),
        });
    }

    // PUB CONST
    let const_re = Regex::new(r"\n(\s*)pub\s+const\s+(\w+)\s*:\s*([^=;]+)").unwrap();
    for cap in const_re.captures_iter(src) {
        let m = cap.get(0).unwrap();
        let indent = cap.get(1).unwrap().as_str();
        let name = cap.get(2).unwrap().as_str().to_string();
        let ty = cap.get(3).unwrap().as_str().trim().to_string();
        let doc_idx = clamp_to_char_boundary(src, m.start() + indent.len() + 1);
        let (doc, _) = get_preceding_doc(src, doc_idx);

        items.push(SpecItem {
            kind: "const".to_string(),
            name,
            generics: String::new(),
            bounds: String::new(),
            sig: ty,
            body: None,
            doc: doc.join(" "),
            file: rel.clone(),
        });
    }

    items
}

fn collect_crate_items(crate_dir: &Path) -> Vec<SpecItem> {
    let src_dir = crate_dir.join("src");
    let mut items = Vec::new();
    if !src_dir.exists() {
        return items;
    }

    fn walk(dir: &Path, crate_dir: &Path, items: &mut Vec<SpecItem>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default();
                    if name != "tests" && name != "test" {
                        walk(&path, crate_dir, items);
                    }
                } else if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
                    items.extend(extract_items(&path, crate_dir));
                }
            }
        }
    }

    walk(&src_dir, crate_dir, &mut items);
    items
}

struct LibMap {
    doc: String,
    mods: Vec<String>,
    mods_inline: Vec<String>,
    uses: Vec<String>,
}

fn extract_lib_map(crate_dir: &Path) -> LibMap {
    let lib_path = crate_dir.join("src").join("lib.rs");
    if !lib_path.exists() {
        return LibMap {
            doc: String::new(),
            mods: Vec::new(),
            mods_inline: Vec::new(),
            uses: Vec::new(),
        };
    }

    let src = fs::read_to_string(&lib_path).unwrap_or_default();
    let mut doc_lines = Vec::new();

    for line in src.lines() {
        let s = line.trim();
        if s.starts_with("//!") {
            doc_lines.push(s[3..].trim().to_string());
        } else if s.is_empty() && doc_lines.is_empty() {
            continue;
        } else {
            if !doc_lines.is_empty() {
                break;
            }
            if s.starts_with("//") || s.is_empty() {
                continue;
            }
            break;
        }
    }

    let mod_re = Regex::new(r"(?m)^\s*pub mod (\w+);").unwrap();
    let mods = mod_re
        .captures_iter(&src)
        .map(|c| c[1].to_string())
        .collect();

    let mod_inline_re = Regex::new(r"(?m)^\s*pub mod (\w+)\s*\{").unwrap();
    let mods_inline = mod_inline_re
        .captures_iter(&src)
        .map(|c| c[1].to_string())
        .collect();

    let use_re = Regex::new(r"pub use ([^;]+);").unwrap();
    let uses = use_re
        .captures_iter(&src)
        .map(|c| c[1].split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();

    LibMap {
        doc: doc_lines.join(" "),
        mods,
        mods_inline,
        uses,
    }
}

fn parse_cargo_toml(crate_dir: &Path) -> (Vec<String>, Vec<String>, Vec<(String, String)>) {
    let path = crate_dir.join("Cargo.toml");
    let content = fs::read_to_string(path).unwrap_or_default();

    let get_section = |sec_name: &str| -> String {
        let pat = format!(r"\n\[{}\]\n(.*?)(?=\n\[|\Z)", regex::escape(sec_name));
        if let Ok(re) = Regex::new(&pat) {
            if let Some(cap) = re.captures(&content) {
                return cap[1].to_string();
            }
        }
        String::new()
    };

    let deps_sec = get_section("dependencies");
    let feat_sec = get_section("features");

    let dep_re = Regex::new(r"([A-Za-z0-9_\-]+)\s*=").unwrap();
    let mut internal = Vec::new();
    let mut external = Vec::new();

    for line in deps_sec.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        if let Some(cap) = dep_re.captures(l) {
            let dep = cap[1].to_string();
            if dep.starts_with("contextra") {
                internal.push(dep);
            } else {
                external.push(dep);
            }
        }
    }

    internal.sort();
    internal.dedup();
    external.sort();
    external.dedup();

    let feat_re = Regex::new(r"([A-Za-z0-9_\-]+)\s*=\s*(\[.*\])").unwrap();
    let mut features = Vec::new();
    for line in feat_sec.lines() {
        let l = line.trim();
        if let Some(cap) = feat_re.captures(l) {
            features.push((cap[1].to_string(), cap[2].to_string()));
        }
    }

    (internal, external, features)
}

fn esc(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ").trim().to_string()
}

fn clip(s: &str, n: usize) -> String {
    let clean = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.chars().count() <= n {
        clean
    } else {
        let truncated: String = clean.chars().take(n.saturating_sub(1)).collect();
        format!("{}…", truncated.trim_end())
    }
}

fn fmt_trait(t: &SpecItem) -> String {
    let body = t.body.as_deref().unwrap_or("").trim_matches('\n');
    let bounds_str = if !t.bounds.is_empty() {
        format!(" {}", t.bounds)
    } else {
        String::new()
    };
    format!(
        "pub trait {}{}{} {{\n{}\n}}",
        t.name, t.generics, bounds_str, body
    )
}

pub fn run_generate_interface_spec(args: &[String]) -> i32 {
    let mut repo_root = PathBuf::from(".");
    let mut out_dir = PathBuf::from("docs");
    let mut stable_name = "CONTEXTRA_INTERFACE_SPECIFICATION.md".to_string();
    let mut no_timestamp_only = false;

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
            "--stable-name" => {
                if i + 1 < args.len() {
                    stable_name = args[i + 1].clone();
                    i += 1;
                }
            }
            "--no-timestamp-only" => {
                no_timestamp_only = true;
            }
            _ => {}
        }
        i += 1;
    }

    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        eprintln!(
            "[error] generate-interface-spec: '{}' nicht gefunden.",
            crates_dir.display()
        );
        return 1;
    }

    let meta_map = get_ring_meta();
    let ring_names = get_ring_names();

    let present_crates: Vec<&str> = CRATE_ORDER
        .iter()
        .copied()
        .filter(|c| crates_dir.join(c).is_dir())
        .collect();

    let mut body_buf = String::new();
    let mut total_stats: HashMap<String, usize> = HashMap::new();

    for cname in &present_crates {
        let cdir = crates_dir.join(cname);
        let meta = meta_map.get(cname).cloned().unwrap_or(CrateMeta {
            ring: 0,
            status: "Unbekannt",
            purpose: "—",
        });

        let items = collect_crate_items(&cdir);
        let lm = extract_lib_map(&cdir);
        let (internal, external, features) = parse_cargo_toml(&cdir);

        let unsafe_flag = if UNSAFE_ISLANDS.contains(cname) {
            "Ja (deklarierte Unsafe-Insel)"
        } else {
            "Nein (`#![forbid(unsafe_code)]`)"
        };

        body_buf.push_str(&format!("\n---\n\n## `{}`\n\n", cname));
        body_buf.push_str("| Attribut | Wert |\n|---|---|\n");
        body_buf.push_str(&format!(
            "| **Ring** | {} — {} |\n",
            meta.ring,
            ring_names.get(&meta.ring).copied().unwrap_or("Unbekannt")
        ));
        body_buf.push_str(&format!("| **Status** | {} |\n", meta.status));
        body_buf.push_str(&format!("| **Zweck** | {} |\n", meta.purpose));
        body_buf.push_str(&format!("| **Unsafe-Code** | {} |\n", unsafe_flag));

        let int_str = if internal.is_empty() {
            "—".to_string()
        } else {
            internal
                .iter()
                .map(|i| format!("`{}`", i))
                .collect::<Vec<_>>()
                .join(", ")
        };
        body_buf.push_str(&format!("| **Interne Abhängigkeiten** | {} |\n", int_str));

        let ext_str = if external.is_empty() {
            "—".to_string()
        } else {
            external
                .iter()
                .map(|e| format!("`{}`", e))
                .collect::<Vec<_>>()
                .join(", ")
        };
        body_buf.push_str(&format!("| **Externe Abhängigkeiten (Auswahl)** | {} |\n\n", ext_str));

        if !lm.doc.is_empty() {
            body_buf.push_str(&format!("### Architekturbeschreibung\n\n{}\n\n", lm.doc));
        }

        body_buf.push_str("### Modul- und Re-Export-Oberfläche (lib.rs)\n\n```rust\n");
        for m in &lm.mods {
            body_buf.push_str(&format!("pub mod {};\n", m));
        }
        for m in &lm.mods_inline {
            body_buf.push_str(&format!("pub mod {} {{ .. }}\n", m));
        }
        for u in &lm.uses {
            body_buf.push_str(&format!("pub use {};\n", u));
        }
        body_buf.push_str("```\n\n");

        if !features.is_empty() {
            body_buf.push_str("### Feature-Flags\n\n| Flag | Aktiviert |\n|---|---|\n");
            for (fn_, fv) in &features {
                body_buf.push_str(&format!("| `{}` | `{}` |\n", fn_, esc(fv)));
            }
            body_buf.push_str("\n");
        }

        let mut traits = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut fns = Vec::new();
        let mut types = Vec::new();
        let mut consts = Vec::new();

        for it in items {
            *total_stats.entry(it.kind.clone()).or_default() += 1;
            match it.kind.as_str() {
                "trait" => traits.push(it),
                "struct" => structs.push(it),
                "enum" => enums.push(it),
                "fn" => fns.push(it),
                "type" => types.push(it),
                "const" => consts.push(it),
                _ => {}
            }
        }

        if !traits.is_empty() {
            traits.sort_by(|a, b| a.name.cmp(&b.name));
            body_buf.push_str(&format!("### Schnittstellen — Public Traits ({})\n\n", traits.len()));
            for t in &traits {
                body_buf.push_str(&format!(
                    "**`{}`** — *{}* (`{}`)\n\n",
                    t.name,
                    clip(&t.doc, 260),
                    t.file
                ));
                body_buf.push_str(&format!("```rust\n{}\n```\n\n", fmt_trait(t)));
            }
        }

        if !structs.is_empty() {
            structs.sort_by(|a, b| a.name.cmp(&b.name));
            body_buf.push_str(&format!("### Öffentliche Datentypen — Structs ({})\n\n", structs.len()));
            body_buf.push_str("| Struct | Felder / Form | Beschreibung | Datei |\n|---|---|---|---|\n");
            for s in &structs {
                let form = if let Some(ref body) = s.body {
                    let bshort = clip(&body.replace('\n', " "), 160);
                    if bshort.is_empty() {
                        "{ }".to_string()
                    } else {
                        format!("{{ {} }}", bshort)
                    }
                } else {
                    "(tuple/unit)".to_string()
                };
                let desc = if s.doc.is_empty() { "—".to_string() } else { clip(&s.doc, 150) };
                body_buf.push_str(&format!("| `{}` | `{}` | {} | `{}` |\n", s.name, esc(&form), desc, s.file));
            }
            body_buf.push_str("\n");
        }

        if !enums.is_empty() {
            enums.sort_by(|a, b| a.name.cmp(&b.name));
            body_buf.push_str(&format!("### Öffentliche Datentypen — Enums ({})\n\n", enums.len()));
            body_buf.push_str("| Enum | Varianten | Beschreibung | Datei |\n|---|---|---|---|\n");
            for e in &enums {
                let variants = clip(&e.body.as_deref().unwrap_or("").replace('\n', " "), 200);
                let desc = if e.doc.is_empty() { "—".to_string() } else { clip(&e.doc, 150) };
                body_buf.push_str(&format!("| `{}` | `{}` | {} | `{}` |\n", e.name, esc(&variants), desc, e.file));
            }
            body_buf.push_str("\n");
        }

        if !types.is_empty() {
            types.sort_by(|a, b| a.name.cmp(&b.name));
            body_buf.push_str(&format!("### Typ-Aliase ({})\n\n", types.len()));
            body_buf.push_str("| Alias | Definition | Beschreibung |\n|---|---|---|\n");
            for t in &types {
                let desc = if t.doc.is_empty() { "—".to_string() } else { clip(&t.doc, 150) };
                body_buf.push_str(&format!("| `{}` | `type {}{}` | {} |\n", t.name, t.name, esc(&t.sig), desc));
            }
            body_buf.push_str("\n");
        }

        if !consts.is_empty() {
            consts.sort_by(|a, b| a.name.cmp(&b.name));
            body_buf.push_str(&format!("### Konstanten ({})\n\n", consts.len()));
            body_buf.push_str("| Konstante | Typ | Beschreibung |\n|---|---|---|\n");
            for c in &consts {
                let desc = if c.doc.is_empty() { "—".to_string() } else { clip(&c.doc, 150) };
                body_buf.push_str(&format!("| `{}` | `{}` | {} |\n", c.name, esc(&c.sig), desc));
            }
            body_buf.push_str("\n");
        }

        if !fns.is_empty() {
            fns.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.name.cmp(&b.name)));
            body_buf.push_str(&format!("### Öffentliche Funktionen ({})\n\n", fns.len()));
            body_buf.push_str("| Signatur | Beschreibung | Datei |\n|---|---|---|\n");
            for f in &fns {
                let desc = if f.doc.is_empty() { "—".to_string() } else { clip(&f.doc, 140) };
                body_buf.push_str(&format!("| `{}` | {} | `{}` |\n", esc(&clip(&f.sig, 260)), desc, f.file));
            }
            body_buf.push_str("\n");
        }
    }

    let now_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let stamp_str = chrono::Utc::now().format("%Y%m%d").to_string();

    let total_symbols: usize = total_stats.values().sum();

    let mut doc_out = String::new();
    doc_out.push_str("# Contextra — Globale Mikrofeingranulare Schnittstellenspezifikation\n\n");
    doc_out.push_str(&format!("**Repository:** `contextra` (Workspace-Root) · **Generiert am:** {}\n", now_str));
    doc_out.push_str("**Erzeugt durch:** `cargo xtask generate-interface-spec` (automatisierte Vollanalyse)\n\n");

    doc_out.push_str("## Umfang der Analyse\n\n| Kennzahl | Wert |\n|---|---|\n");
    doc_out.push_str(&format!("| Analysierte Fach-Crates (`crates/*`) | {} |\n", present_crates.len()));
    doc_out.push_str(&format!("| Extrahierte öffentliche Symbole gesamt | **{}** |\n", total_symbols));
    doc_out.push_str(&format!("| — davon Public Traits | {} |\n", total_stats.get("trait").copied().unwrap_or(0)));
    doc_out.push_str(&format!("| — davon Public Structs | {} |\n", total_stats.get("struct").copied().unwrap_or(0)));
    doc_out.push_str(&format!("| — davon Public Enums | {} |\n", total_stats.get("enum").copied().unwrap_or(0)));
    doc_out.push_str(&format!("| — davon Public Funktionen | {} |\n", total_stats.get("fn").copied().unwrap_or(0)));
    doc_out.push_str(&format!("| — davon Typ-Aliase | {} |\n", total_stats.get("type").copied().unwrap_or(0)));
    doc_out.push_str(&format!("| — davon Konstanten | {} |\n\n", total_stats.get("const").copied().unwrap_or(0)));

    doc_out.push_str(&body_buf);

    let _ = fs::create_dir_all(&out_dir);

    if !no_timestamp_only {
        let (stem, ext) = if let Some(idx) = stable_name.rfind('.') {
            (&stable_name[..idx], &stable_name[idx..])
        } else {
            (stable_name.as_str(), ".md")
        };
        let ts_name = format!("{}_{}{}", stem, stamp_str, ext);
        let _ = fs::write(out_dir.join(ts_name), &doc_out);
    }

    let _ = fs::write(out_dir.join(&stable_name), &doc_out);

    println!(
        "[ok] Schnittstellenspezifikation für {} Crates ({} Symbole) generiert in {}",
        present_crates.len(),
        total_symbols,
        out_dir.display()
    );

    0
}
