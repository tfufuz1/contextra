//! Architekturprüfungen für das Contextra-Workspace.
//!
//! - Ring-Regel (SPEC §3.2 Regel 1, NFR-3): Eine Abhängigkeit darf nie auf einen höheren Ring
//!   zeigen. Ring 0 darf `tokio` nicht enthalten. Tooling-Crates dürfen nicht von Produktcrates
//!   abhängen.
//! - Unsafe-Regel (SPEC §3.2 Regel 6, NFR-2): `unsafe` steht nur in den Unsafe-Inseln. Jeder
//!   `unsafe`-Block dort trägt einen `// SAFETY:`-Kommentar. Alle übrigen Crates setzen
//!   `forbid(unsafe_code)`.
//!
//! Die Ring-Zuordnung folgt `docs/REDUCTION_SPEC.md` Abschnitt 4.1. `engine` gilt als Ring 2
//! (Entscheidung D-R3).

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ring {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
    Ring4 = 4,
    Tooling = 99,
}

/// Ring-Zuordnung je Workspace-Crate (Spezifikation 4.1).
pub const RING_TABLE: &[(&str, Ring)] = &[
    ("contextra-types", Ring::Ring0),
    ("contextra-ports", Ring::Ring0),
    ("contextra-sys", Ring::Ring0),
    ("contextra-simd", Ring::Ring0),
    ("contextra-mvcc", Ring::Ring0),
    ("contextra-crypto", Ring::Ring0),
    ("contextra-vector", Ring::Ring0),
    ("contextra-text", Ring::Ring0),
    ("contextra-rank", Ring::Ring0),
    ("contextra-store", Ring::Ring1),
    ("contextra-engine", Ring::Ring2),
    ("contextra-db", Ring::Ring3),
    ("contextra", Ring::Ring4),
    ("contextra-testkit", Ring::Tooling),
    ("contextra-bench", Ring::Tooling),
];

/// Crates, in denen `unsafe` erlaubt ist (NFR-2).
pub const UNSAFE_ISLANDS: &[&str] = &["contextra-sys", "contextra-simd"];

/// Ausnahmen von der Ring-Regel. Für v1 leer. Jeder Eintrag braucht eine Begründung und ein Ziel.
pub const ALLOWLIST: &[(&str, &str)] = &[];

#[derive(Debug, Clone)]
pub struct Krate {
    pub name: String,
    pub dir: PathBuf,
    /// Namen der normalen Abhängigkeiten (`[dependencies]`, nicht dev/build).
    pub deps: Vec<String>,
}

pub fn ring_of(name: &str) -> Option<Ring> {
    RING_TABLE.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
}

/// Liest `crates/*/Cargo.toml` aus dem Workspace-Root und sammelt Name und normale Abhängigkeiten.
pub fn load_crates(root: &Path) -> Result<Vec<Krate>, String> {
    let crates_dir = root.join("crates");
    let entries = fs::read_dir(&crates_dir)
        .map_err(|e| format!("crates-Verzeichnis nicht lesbar ({}): {e}", crates_dir.display()))?;
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Verzeichniseintrag: {e}"))?;
        let dir = entry.path();
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let text = fs::read_to_string(&manifest)
            .map_err(|e| format!("{} nicht lesbar: {e}", manifest.display()))?;
        let doc: toml::Table = text
            .parse()
            .map_err(|e| format!("{} ist kein gültiges TOML: {e}", manifest.display()))?;
        let name = doc
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| format!("{}: [package].name fehlt", manifest.display()))?
            .to_string();
        let deps = doc
            .get("dependencies")
            .and_then(|d| d.as_table())
            .map(|t| {
                t.iter()
                    .map(|(key, val)| {
                        // Umbenannte Abhängigkeiten: `alias = { package = "echter-name" }`
                        val.get("package")
                            .and_then(|p| p.as_str())
                            .unwrap_or(key.as_str())
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.push(Krate { name, dir, deps });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Prüft die Ring-Regeln. Gibt eine Liste von Verstößen zurück, leer bedeutet bestanden.
pub fn check_layering(crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    for krate in crates {
        let Some(my_ring) = ring_of(&krate.name) else {
            violations.push(format!("{}: nicht in RING_TABLE eingetragen", krate.name));
            continue;
        };
        for dep in &krate.deps {
            if my_ring == Ring::Ring0 && dep == "tokio" {
                violations.push(format!("Ring 0 verletzt: {} hängt von tokio ab", krate.name));
                continue;
            }
            if ALLOWLIST.iter().any(|(from, to)| *from == krate.name && *to == dep) {
                continue;
            }
            let Some(dep_ring) = ring_of(dep) else {
                continue; // externe Crates sind hier nicht relevant
            };
            if my_ring == Ring::Tooling {
                continue;
            }
            if dep_ring == Ring::Tooling {
                violations.push(format!(
                    "{} ({:?}) hängt von Tooling-Crate {} ab",
                    krate.name, my_ring, dep
                ));
            } else if dep_ring > my_ring {
                violations.push(format!(
                    "{} ({:?}) hängt von höherem Ring ab: {} ({:?})",
                    krate.name, my_ring, dep, dep_ring
                ));
            }
        }
    }
    violations
}

/// Prüft die Unsafe-Regel. Gibt eine Liste von Verstößen zurück, leer bedeutet bestanden.
pub fn check_unsafe(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    for krate in crates {
        let island = UNSAFE_ISLANDS.contains(&krate.name.as_str());
        let lib_rs = krate.dir.join("src").join("lib.rs");
        if !island {
            match fs::read_to_string(&lib_rs) {
                Ok(text) if text.contains("forbid(unsafe_code)") => {}
                Ok(_) => violations.push(format!(
                    "{}: src/lib.rs setzt kein forbid(unsafe_code)",
                    krate.name
                )),
                Err(e) => violations.push(format!("{}: src/lib.rs nicht lesbar: {e}", krate.name)),
            }
        }
        let mut files = Vec::new();
        collect_rs_files(&krate.dir.join("src"), &mut files);
        for file in files {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            let lines: Vec<&str> = text.lines().collect();
            for (idx, line) in lines.iter().enumerate() {
                let code = line.trim();
                if code.starts_with("//") {
                    continue;
                }
                let is_block = code.contains("unsafe {") || code.ends_with("unsafe");
                let is_fn = code.contains("unsafe fn") || code.contains("unsafe impl") || code.contains("unsafe trait");
                if !is_block && !is_fn {
                    continue;
                }
                let rel = file.strip_prefix(root).unwrap_or(&file).display();
                if !island {
                    violations.push(format!("{}:{}: unsafe in Nicht-Insel {}", rel, idx + 1, krate.name));
                    continue;
                }
                if is_block {
                    // SAFETY-Kommentare stehen bei mehrzeiligen Begründungen bis zu 6 Zeilen darüber.
                    let start = idx.saturating_sub(6);
                    let has_safety = lines
                        .get(start..idx)
                        .map(|ctx| ctx.iter().any(|l| l.contains("SAFETY:")))
                        .unwrap_or(false);
                    if !has_safety {
                        violations.push(format!("{}:{}: unsafe-Block ohne // SAFETY:", rel, idx + 1));
                    }
                }
            }
        }
    }
    violations
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn krate(name: &str, deps: &[&str]) -> Krate {
        Krate {
            name: name.to_string(),
            dir: PathBuf::from("/nonexistent"),
            deps: deps.iter().map(|d| d.to_string()).collect(),
        }
    }

    #[test]
    fn erkennt_aufwaerts_abhaengigkeit() {
        let v = check_layering(&[krate("contextra-types", &["contextra-store"])]);
        assert_eq!(v.len(), 1, "{v:?}");
    }

    #[test]
    fn erkennt_tokio_in_ring0() {
        let v = check_layering(&[krate("contextra-ports", &["tokio"])]);
        assert_eq!(v.len(), 1, "{v:?}");
    }

    #[test]
    fn erlaubt_abwaerts_abhaengigkeit() {
        let v = check_layering(&[krate("contextra-db", &["contextra-engine", "contextra-types"])]);
        assert!(v.is_empty(), "{v:?}");
    }
}
