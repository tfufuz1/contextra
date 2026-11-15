//! `cargo xtask gates`: Architektur- und Hygienepruefungen (Spec 16.6).
//!
//! Umgesetzt: G1, G-DEF, G-HUB, G-UNSAFE, G-NOTOKIO, G-LINT (Konfiguration), G-WORD.
//! Offen (Archiv `archive/xtask/`): G2, G3, G-CRY, G-FS, G-ERA, G-READ, G-NOSERDE-DISK, G-SUPPLY, G-ZUSAGE.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

type Res = Result<Vec<String>, String>;

/// Gate-Name und Funktion.
const GATES: &[(&str, fn(&Ctx) -> Res)] = &[
    ("g1", g1),
    ("g-def", g_def),
    ("g-hub", g_hub),
    ("g-unsafe", g_unsafe),
    ("g-notokio", g_notokio),
    ("g-lint", g_lint),
    ("g-word", g_word),
];

struct Ctx {
    root: PathBuf,
    levels: BTreeMap<String, u32>,
    budgets: BTreeMap<String, u32>,
    isolated: Vec<String>,
}

fn parse_rings(text: &str) -> (BTreeMap<String, u32>, BTreeMap<String, u32>, Vec<String>) {
    let (mut lv, mut bud, mut iso) = (BTreeMap::new(), BTreeMap::new(), Vec::new());
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = s.to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let (k, v) = (k.trim().to_string(), v.trim());
            if let Ok(n) = v.parse::<u32>() {
                match section.as_str() {
                    "levels" => {
                        lv.insert(k, n);
                    }
                    "budgets" => {
                        bud.insert(k, n);
                    }
                    "isolated" => iso.push(k),
                    _ => {}
                }
            }
        }
    }
    (lv, bud, iso)
}

fn metadata(root: &Path) -> Result<serde_json::Value, String> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err("cargo metadata failed".into());
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("metadata json: {e}"))
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                rs_files(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
}

/// G1: Level-Graph gegen rings.toml und cargo metadata; Dev-Abhaengigkeiten zaehlen nicht.
fn g1(c: &Ctx) -> Res {
    let md = metadata(&c.root)?;
    let mut bad = Vec::new();
    for pkg in md["packages"].as_array().ok_or("no packages")? {
        let name = pkg["name"].as_str().unwrap_or_default();
        let Some(&la) = c.levels.get(name) else { continue };
        for d in pkg["dependencies"].as_array().into_iter().flatten() {
            if d["kind"].as_str() == Some("dev") {
                continue;
            }
            let dn = d["name"].as_str().unwrap_or_default();
            if c.isolated.iter().any(|i| i == name) && dn.starts_with("acta-") {
                bad.push(format!("{name}: isolierter Crate haengt von {dn} ab"));
            }
            if let Some(&lb) = c.levels.get(dn) {
                if la <= lb {
                    bad.push(format!("{name} (L{la}) -> {dn} (L{lb}) verletzt level(A) > level(B)"));
                }
            }
        }
    }
    // HR-8: von acta-engine haengt hoechstens acta-node ab.
    for pkg in md["packages"].as_array().into_iter().flatten() {
        let name = pkg["name"].as_str().unwrap_or_default();
        for d in pkg["dependencies"].as_array().into_iter().flatten() {
            if d["name"].as_str() == Some("acta-engine") && d["kind"].as_str() != Some("dev") && name != "acta-node" {
                bad.push(format!("{name}: HR-8 verletzt (nur acta-node darf acta-engine nutzen)"));
            }
        }
    }
    Ok(bad)
}

/// G-DEF: alle Crates in `default-members`.
fn g_def(c: &Ctx) -> Res {
    let md = metadata(&c.root)?;
    let all = md["workspace_members"].as_array().map_or(0, Vec::len);
    let def = md["workspace_default_members"].as_array().map_or(0, Vec::len);
    Ok(if all == def { vec![] } else { vec![format!("{def} von {all} Crates in default-members")] })
}

fn is_test_path(p: &Path) -> bool {
    p.components().any(|c| c.as_os_str() == "tests" || c.as_os_str() == "benches")
        || p.file_name().is_some_and(|n| n == "tests.rs")
}

/// G-HUB: Groessenbudgets (Produktionscode ohne Tests, ohne Leer- und Kommentarzeilen).
fn g_hub(c: &Ctx) -> Res {
    let mut bad = Vec::new();
    for (name, &budget) in &c.budgets {
        let src = c.root.join("crates").join(name).join("src");
        let mut files = Vec::new();
        rs_files(&src, &mut files);
        let mut loc = 0_u32;
        for f in files.iter().filter(|f| !is_test_path(f)) {
            let text = fs::read_to_string(f).map_err(|e| e.to_string())?;
            let mut skip_tests = false;
            for l in text.lines() {
                let t = l.trim();
                if t.starts_with("#[cfg(test)]") {
                    skip_tests = true;
                }
                if skip_tests || t.is_empty() || t.starts_with("//") {
                    continue;
                }
                loc += 1;
            }
        }
        if loc > budget {
            bad.push(format!("{name}: {loc} LOC > Budget {budget} (ADR erforderlich)"));
        }
    }
    Ok(bad)
}

/// G-UNSAFE: `unsafe` nur in acta-sys, dort jeder Block mit `// SAFETY:`.
fn g_unsafe(c: &Ctx) -> Res {
    let mut bad = Vec::new();
    let mut files = Vec::new();
    rs_files(&c.root.join("crates"), &mut files);
    for f in files {
        let in_sys = f.components().any(|x| x.as_os_str() == "acta-sys");
        let text = fs::read_to_string(&f).map_err(|e| e.to_string())?;
        let lines: Vec<&str> = text.lines().collect();
        for (i, l) in lines.iter().enumerate() {
            let t = l.trim();
            if t.starts_with("//") || !has_word(t, "unsafe") || t.contains("forbid(unsafe_code)") || t.contains("unsafe_code")
                || t.contains("unsafe_op_in_unsafe_fn")
            {
                continue;
            }
            if !in_sys {
                bad.push(format!("{}:{}: unsafe ausserhalb acta-sys", f.display(), i + 1));
            } else if t.contains("unsafe {") || t.contains("unsafe impl") {
                let ok = (i.saturating_sub(3)..i).filter_map(|k| lines.get(k)).any(|p| p.contains("SAFETY:"));
                if !ok {
                    bad.push(format!("{}:{}: unsafe ohne // SAFETY:", f.display(), i + 1));
                }
            }
        }
    }
    Ok(bad)
}

fn has_word(hay: &str, w: &str) -> bool {
    hay.match_indices(w).any(|(i, _)| {
        let before = hay[..i].chars().next_back().is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        let after = hay[i + w.len()..].chars().next().is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        before && after
    })
}

/// G-NOTOKIO: kein `tokio` in Level 0 bis 7.
fn g_notokio(c: &Ctx) -> Res {
    let mut bad = Vec::new();
    for (name, &lv) in &c.levels {
        if lv > 7 {
            continue;
        }
        let p = c.root.join("crates").join(name).join("Cargo.toml");
        if let Ok(t) = fs::read_to_string(&p) {
            if t.contains("tokio") {
                bad.push(format!("{name} (L{lv}) enthaelt tokio"));
            }
        }
    }
    Ok(bad)
}

/// G-LINT: jede Crate uebernimmt die Workspace-Lints (unwrap/expect/panic/indexing verboten).
fn g_lint(c: &Ctx) -> Res {
    let mut bad = Vec::new();
    for name in c.levels.keys() {
        let p = c.root.join("crates").join(name).join("Cargo.toml");
        let t = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let ok = t.contains("[lints]\nworkspace = true") || (name == "acta-sys" && t.contains("unwrap_used = \"deny\""));
        if !ok {
            bad.push(format!("{name}: Workspace-Lints fehlen"));
        }
    }
    Ok(bad)
}

/// G-WORD: verbotene Formulierungen (Spec 1.6, Liste A) in Quelltext des Workspace.
fn g_word(c: &Ctx) -> Res {
    const LISTE_A: &[&str] = &[
        "dsgvo-konform", "dsgvo-zertifiziert", "erfüllt art. 17 automatisch", "100 % gelöscht", "garantiert gelöscht",
        "garantiert vernichtet", "unwiederbringlich", "vergisst daten", "revisionssicher", "fehlerfrei",
        "löscht aus allen backups", "perfekte antworten",
    ];
    let mut bad = Vec::new();
    let mut files = Vec::new();
    rs_files(&c.root.join("crates"), &mut files);
    for f in files {
        let text = fs::read_to_string(&f).map_err(|e| e.to_string())?.to_lowercase();
        for w in LISTE_A {
            if text.contains(w) {
                bad.push(format!("{}: verbotene Formulierung \"{w}\"", f.display()));
            }
        }
    }
    Ok(bad)
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "gates".into());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let text = match fs::read_to_string(root.join("rings.toml")) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("rings.toml: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (levels, budgets, isolated) = parse_rings(&text);
    let ctx = Ctx { root, levels, budgets, isolated };
    let mut failed = false;
    for (name, f) in GATES {
        if arg != "gates" && arg != *name {
            continue;
        }
        match f(&ctx) {
            Ok(v) if v.is_empty() => println!("[ok]   {name}"),
            Ok(v) => {
                failed = true;
                println!("[FAIL] {name}");
                for l in v {
                    println!("         {l}");
                }
            }
            Err(e) => {
                failed = true;
                println!("[ERR]  {name}: {e}");
            }
        }
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
