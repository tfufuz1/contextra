//! Architekturprüfungen für das Contextra-Workspace.
//!
//! - Ring-Regel (SPEC §3.2 Regel 1, NFR-3, T-ARCH-3): Eine Abhängigkeit darf nie auf einen höheren Ring
//!   zeigen. Ring 0 darf `tokio` nicht enthalten. Tooling-Crates dürfen nicht von Produktcrates
//!   abhängen. AR-1..AR-4 werden geprüft.
//! - Unsafe-Regel (SPEC §3.2 Regel 6, NFR-2, T-UNSAFE-1): `unsafe` steht nur in den Unsafe-Inseln. Jeder
//!   `unsafe`-Block dort trägt einen `// SAFETY:`-Kommentar. Alle übrigen Crates setzen
//!   `forbid(unsafe_code)`.
//! - T-ARCH-1: contextra-types hat keine I/O-, async-, Thread-, Zeit-, Zufalls-Abhängigkeit.
//! - T-ARCH-2: Nur `TenantScopedStorage` ruft den rohen StorageEngine/LsmStorage auf.
//! - T-ARCH-4: Keine Netzwerk-Crates im normalen Produktions-Abhängigkeitsbaum.
//! - T-LINT-1: Wortlaut-Lint über Quellen, Doku, README (keine unzulässigen Werbeaussagen).
//! - T-LINT-2: Kein `std::fs` / `tokio::fs` im Crate `contextra-store`.
//! - T-CFG-1/2, T-DEF-1: Feature-Hygiene, Verbot von `unexpected_cfgs`-Allowances, keine Sicherheitsfeatures hinter cfg.

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

/// Ring-Zuordnung je Workspace-Crate (SPEC §5.2).
pub const RING_TABLE: &[(&str, Ring)] = &[
    ("contextra-types", Ring::Ring0),
    ("contextra-ports", Ring::Ring0),
    ("contextra-sys", Ring::Ring1),
    ("contextra-simd", Ring::Ring1),
    ("contextra-durable-fs", Ring::Ring1),
    ("contextra-crypto", Ring::Ring1),
    ("contextra-mvcc", Ring::Ring2),
    ("contextra-store", Ring::Ring2),
    ("contextra-vector", Ring::Ring2),
    ("contextra-text", Ring::Ring2),
    ("contextra-rank", Ring::Ring2),
    ("contextra-erasure", Ring::Ring3),
    ("contextra-engine", Ring::Ring3),
    ("contextra", Ring::Ring4),
    ("contextra-testkit", Ring::Tooling),
    ("contextra-lab", Ring::Tooling),
    ("ctx-verify", Ring::Tooling),
    ("contextra-arch-tests", Ring::Tooling),
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
    let entries = fs::read_dir(&crates_dir).map_err(|e| {
        format!(
            "crates-Verzeichnis nicht lesbar ({}): {e}",
            crates_dir.display()
        )
    })?;
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

/// Prüft die Ring-Regeln (T-ARCH-3). Gibt eine Liste von Verstößen zurück, leer bedeutet bestanden.
/// Bildet AR-1, AR-2, AR-3, AR-4 ab und meldet unbekannte Crates fail-closed.
pub fn check_layering(crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    for krate in crates {
        let Some(my_ring) = ring_of(&krate.name) else {
            violations.push(format!(
                "{}: nicht in RING_TABLE eingetragen (fail-closed)",
                krate.name
            ));
            continue;
        };
        for dep in &krate.deps {
            if my_ring == Ring::Ring0 && dep == "tokio" {
                violations.push(format!(
                    "Ring 0 verletzt: {} hängt von tokio ab",
                    krate.name
                ));
                continue;
            }
            // AR-2: store kennt weder text, vector noch erasure
            if krate.name == "contextra-store"
                && (dep == "contextra-text"
                    || dep == "contextra-vector"
                    || dep == "contextra-erasure")
            {
                violations.push(format!(
                    "AR-2 verletzt: contextra-store hängt von {} ab",
                    dep
                ));
            }
            // AR-3: erasure kennt keine Ring-2-Crates
            if krate.name == "contextra-erasure" {
                if let Some(dep_ring) = ring_of(dep) {
                    if dep_ring == Ring::Ring2 {
                        violations.push(format!(
                            "AR-3 verletzt: contextra-erasure hängt von Ring-2-Crate {} ab",
                            dep
                        ));
                    }
                }
            }
            // AR-4: lab und ctx-verify hängen nicht an store, engine, erasure
            if (krate.name == "contextra-lab" || krate.name == "ctx-verify")
                && (dep == "contextra-store"
                    || dep == "contextra-engine"
                    || dep == "contextra-erasure")
            {
                violations.push(format!(
                    "AR-4 verletzt: {} hängt von {} ab",
                    krate.name, dep
                ));
            }

            if ALLOWLIST
                .iter()
                .any(|(from, to)| *from == krate.name && *to == dep)
            {
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

/// Prüft T-ARCH-1: contextra-types hat keine I/O-, async-, Thread-, Zeit-, Zufalls-Abhängigkeit.
pub fn check_types_purity(root: &Path) -> Vec<String> {
    let types_manifest = root.join("crates/contextra-types/Cargo.toml");
    let mut violations = Vec::new();
    if !types_manifest.exists() {
        violations.push("crates/contextra-types/Cargo.toml existiert nicht".to_string());
        return violations;
    }
    let text = match fs::read_to_string(&types_manifest) {
        Ok(t) => t,
        Err(e) => {
            violations.push(format!("Cargo.toml von contextra-types nicht lesbar: {e}"));
            return violations;
        }
    };
    let doc: toml::Table = match text.parse() {
        Ok(d) => d,
        Err(e) => {
            violations.push(format!("Cargo.toml von contextra-types ungültig: {e}"));
            return violations;
        }
    };
    let forbidden_dep_keywords = [
        "tokio",
        "async-std",
        "futures",
        "mio",
        "socket2",
        "reqwest",
        "hyper",
        "rand",
        "fastrand",
        "tempfile",
    ];
    if let Some(deps) = doc.get("dependencies").and_then(|d| d.as_table()) {
        for (dep_name, _) in deps {
            for forbidden in &forbidden_dep_keywords {
                if dep_name == forbidden {
                    violations.push(format!(
                        "T-ARCH-1 verletzt: contextra-types hat Abhängigkeit zu '{dep_name}'"
                    ));
                }
            }
        }
    }

    // Quellcode-Scan nach verbotenen use-Imports in Produktionscode (src/**/*.rs ohne Tests)
    let src_dir = root.join("crates/contextra-types/src");
    let mut rs_files = Vec::new();
    collect_rs_files(&src_dir, &mut rs_files);

    let forbidden_use_patterns = [
        "std::fs",
        "std::net",
        "std::thread",
        "tokio",
        "futures",
        "async_trait",
    ];

    for file in rs_files {
        if file.components().any(|c| c.as_os_str() == "tests") {
            continue;
        }
        let content = match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for pattern in &forbidden_use_patterns {
                if trimmed.starts_with("use ") && trimmed.contains(pattern) {
                    let rel = file.strip_prefix(root).unwrap_or(&file).display();
                    violations.push(format!(
                        "T-ARCH-1 verletzt: {}:{}: verbotener Import '{}'",
                        rel,
                        line_idx + 1,
                        pattern
                    ));
                }
            }
        }
    }

    violations
}

/// Prüft T-ARCH-4: Keine verbotenen Netzwerk-Crates im normalen Produktions-Abhängigkeitsbaum.
pub fn check_network_dependencies(crates: &[Krate]) -> Vec<String> {
    let forbidden_network_crates = [
        "reqwest",
        "hyper",
        "hyper-util",
        "hyper-rustls",
        "tonic",
        "h2",
        "ureq",
        "curl",
        "isahc",
        "surf",
        "attohttpc",
    ];
    let mut violations = Vec::new();
    for krate in crates {
        let Some(my_ring) = ring_of(&krate.name) else {
            continue;
        };
        if my_ring == Ring::Tooling {
            continue;
        }
        for dep in &krate.deps {
            for forbidden in &forbidden_network_crates {
                if dep == forbidden {
                    violations.push(format!(
                        "T-ARCH-4 verletzt: Produktions-Crate '{}' hängt von Netzwerk-Crate '{}' ab",
                        krate.name, dep
                    ));
                }
            }
        }
    }
    violations
}

/// Prüft T-ARCH-2: Nur TenantScopedStorage ruft den rohen StorageEngine/LsmStorage auf.
/// Ausnahmen: contextra-store selbst (Interna, Replay, Tests) sowie TenantScopedStorage-Konstruktion in contextra-engine.
pub fn check_raw_storage_isolation(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    for krate in crates {
        if krate.name == "contextra-store" || krate.name == "contextra-testkit" {
            continue;
        }
        let src_dir = krate.dir.join("src");
        let mut rs_files = Vec::new();
        collect_rs_files(&src_dir, &mut rs_files);

        for file in rs_files {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();

            // Erlaubte Stellen in contextra-engine: Verdrahtung in contextra_impl, lifecycle, collections
            let is_engine_wiring = krate.name == "contextra-engine"
                && (rel.contains("contextra_impl")
                    || rel.contains("lifecycle")
                    || rel.contains("collections")
                    || rel.contains("lib.rs"));

            if is_engine_wiring {
                continue;
            }

            for (line_idx, line) in content.lines().enumerate() {
                let code = line.trim();
                if code.starts_with("//") {
                    continue;
                }
                if code.contains("LsmStorage::") || code.contains("raw_storage.") {
                    violations.push(format!(
                        "T-ARCH-2 verletzt: {}:{}: Direkter Aufruf des rohen Stores ohne TenantScopedStorage: '{}'",
                        rel, line_idx + 1, code
                    ));
                }
            }
        }
    }
    violations
}

/// Prüft T-LINT-1: Wortlaut-Lint über src/**, README, docs auf verbotene Begriffe.
pub const WORDING_ALLOWLIST: &[(&str, &str)] = &[
    (
        "docs/SPEC.md",
        "Spezifikation definiert selbst die Liste verbotener Begriffe und Zitate",
    ),
    (
        "docs/DELETION-DSGVO.md",
        "Rechtsanalyse zitiert verbotene Begriffe im Negativkontext",
    ),
    (
        "docs/IMPLEMENTATION_SPEC_PER_CRATE.md",
        "Zitiert verbotene Begriffe zur Testbeschreibung",
    ),
    ("tests/arch/AGENTS.md", "Erklärt den Wortlaut-Lint"),
    (
        "tests/arch/src/lib.rs",
        "Implementierung des Wortlaut-Linters",
    ),
    ("tests/arch/tests/arch.rs", "Testsuite des Wortlaut-Linters"),
];

pub fn check_wording(root: &Path) -> Vec<String> {
    let forbidden_terms = [
        "DSGVO-konform",
        "garantiert gelöscht",
        "100 % sicher",
        "100% sicher",
        "unwiederbringlich",
        "rechtskonform",
    ];

    let mut violations = Vec::new();

    let mut files_to_scan = Vec::new();
    let crates_dir = root.join("crates");
    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let src_dir = entry.path().join("src");
            if src_dir.is_dir() {
                collect_all_files(&src_dir, &mut files_to_scan);
            }
        }
    }
    let docs_dir = root.join("docs");
    if docs_dir.is_dir() {
        collect_all_files(&docs_dir, &mut files_to_scan);
    }
    for file_name in &["README.md", "README_DEV.md", "PLAN.md"] {
        let p = root.join(file_name);
        if p.exists() {
            files_to_scan.push(p);
        }
    }

    for file in files_to_scan {
        let rel = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .to_string();
        if WORDING_ALLOWLIST
            .iter()
            .any(|(allowed_path, _)| rel == *allowed_path || rel.starts_with(allowed_path))
        {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            for term in &forbidden_terms {
                if trimmed.contains(term) {
                    violations.push(format!(
                        "T-LINT-1 verletzt: {}:{}: Unzulässige Außenaussage / Begriff '{}'",
                        rel,
                        line_idx + 1,
                        term
                    ));
                }
            }
        }
    }

    violations
}

/// Prüft T-LINT-2: Kein `std::fs` oder `tokio::fs` im Crate contextra-store (nur contextra_ports::Vfs).
pub fn check_store_fs_usage(root: &Path) -> Vec<String> {
    let mut violations = Vec::new();
    let store_src = root.join("crates/contextra-store/src");
    let mut rs_files = Vec::new();
    collect_rs_files(&store_src, &mut rs_files);

    for file in rs_files {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        for (line_idx, line) in content.lines().enumerate() {
            let code = line.trim();
            if code.starts_with("//") {
                continue;
            }
            if code.contains("std::fs") || code.contains("tokio::fs") {
                violations.push(format!(
                    "T-LINT-2 verletzt: {}:{}: Direkte Dateisystemnutzung '{}' statt Vfs",
                    rel,
                    line_idx + 1,
                    code
                ));
            }
        }
    }
    violations
}

/// Prüft T-CFG-1, T-CFG-2 und T-DEF-1.
pub fn check_feature_hygiene(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();

    let forbidden_security_feature_substrings = [
        "no-encryption",
        "no_encryption",
        "disable-encryption",
        "without-crypto",
        "opt-out-crypto",
    ];

    for krate in crates {
        let manifest_path = krate.dir.join("Cargo.toml");
        let Ok(manifest_text) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Ok(doc) = manifest_text.parse::<toml::Table>() else {
            continue;
        };

        let mut declared_features = std::collections::BTreeSet::new();
        if let Some(features) = doc.get("features").and_then(|f| f.as_table()) {
            for (f_name, _) in features {
                declared_features.insert(f_name.clone());
                for forbidden in &forbidden_security_feature_substrings {
                    if f_name.contains(forbidden) {
                        violations.push(format!(
                            "T-DEF-1 verletzt: Crate '{}' deklariert verbotenes Sicherheits-Opt-out Feature '{}'",
                            krate.name, f_name
                        ));
                    }
                }
            }
        }

        declared_features.insert("default".to_string());
        declared_features.insert("test".to_string());

        let mut rs_files = Vec::new();
        collect_rs_files(&krate.dir, &mut rs_files);

        for file in rs_files {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();

            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }

                if trimmed.contains("allow(unexpected_cfgs)") {
                    violations.push(format!(
                        "T-CFG-1 verletzt: {}:{}: '#[allow(unexpected_cfgs)]' ist verboten",
                        rel,
                        line_idx + 1
                    ));
                }

                if let Some(pos) = trimmed.find("feature = \"") {
                    let after = &trimmed[pos + 11..];
                    if let Some(end_pos) = after.find('"') {
                        let feat = &after[..end_pos];
                        if !declared_features.contains(feat) {
                            violations.push(format!(
                                "T-CFG-1 verletzt: {}:{}: Undeklariertes Feature '{}' in Crate '{}'",
                                rel, line_idx + 1, feat, krate.name
                            ));
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Prüft die Unsafe-Regel (T-UNSAFE-1).
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
                let is_fn = code.contains("unsafe fn")
                    || code.contains("unsafe impl")
                    || code.contains("unsafe trait");
                if !is_block && !is_fn {
                    continue;
                }
                let rel = file.strip_prefix(root).unwrap_or(&file).display();
                if !island {
                    violations.push(format!(
                        "{}:{}: unsafe in Nicht-Insel {}",
                        rel,
                        idx + 1,
                        krate.name
                    ));
                    continue;
                }
                if is_block {
                    let start = idx.saturating_sub(6);
                    let has_safety = lines
                        .get(start..idx)
                        .map(|ctx| ctx.iter().any(|l| l.contains("SAFETY:")))
                        .unwrap_or(false);
                    if !has_safety {
                        violations.push(format!(
                            "{}:{}: unsafe-Block ohne // SAFETY:",
                            rel,
                            idx + 1
                        ));
                    }
                }
            }
        }
    }
    violations
}

fn collect_all_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(&path, out);
        } else {
            out.push(path);
        }
    }
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn krate(name: &str, deps: &[&str]) -> Krate {
        Krate {
            name: name.to_string(),
            dir: PathBuf::from("/nonexistent"),
            deps: deps.iter().map(|d| d.to_string()).collect(),
        }
    }

    // T-ARCH-3: Positiv und Negativ
    #[test]
    fn t_arch_3_positiv_abwaerts_erlaubt() {
        let v = check_layering(&[krate(
            "contextra-engine",
            &["contextra-types", "contextra-store"],
        )]);
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn t_arch_3_negativ_aufwaerts_verboten() {
        let v = check_layering(&[krate("contextra-types", &["contextra-store"])]);
        assert_eq!(v.len(), 1, "{v:?}");
    }

    #[test]
    fn t_arch_3_negativ_ring0_tokio() {
        let v = check_layering(&[krate("contextra-ports", &["tokio"])]);
        assert_eq!(v.len(), 1, "{v:?}");
    }

    #[test]
    fn t_arch_3_negativ_unbekanntes_crate_fail_closed() {
        let v = check_layering(&[krate("unbekanntes-crate", &["contextra-types"])]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(
            v.first().is_some_and(|msg| msg.contains("fail-closed")),
            "Erwartet fail-closed Meldung"
        );
    }

    // T-ARCH-4: Positiv und Negativ
    #[test]
    fn t_arch_4_positiv_kein_netzwerk() {
        let v = check_network_dependencies(&[krate(
            "contextra-store",
            &["contextra-types", "parking_lot"],
        )]);
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn t_arch_4_negativ_netzwerk_in_produktion() {
        let v = check_network_dependencies(&[krate("contextra-store", &["reqwest"])]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v.first().is_some_and(|msg| msg.contains("T-ARCH-4")));
    }

    // T-LINT-1: Positiv und Negativ
    #[test]
    fn t_lint_1_positiv_erlaubte_begriffe() {
        let tmp = std::env::temp_dir().join("t_lint_1_pos");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let f = tmp.join("README.md");
        fs::write(&f, "Contextra entfernt Daten messtechnisch nachweisbar.").unwrap();
        let v = check_wording(&tmp);
        assert!(v.is_empty(), "{v:?}");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_lint_1_negativ_verbotene_begriffe() {
        let tmp = std::env::temp_dir().join("t_lint_1_neg");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let f = tmp.join("README.md");
        fs::write(
            &f,
            "Dieses Produkt ist 100 % sicher und garantiert gelöscht.",
        )
        .unwrap();
        let v = check_wording(&tmp);
        assert!(!v.is_empty(), "Verbotene Begriffe müssen erkannt werden");
        let _ = fs::remove_dir_all(&tmp);
    }

    // T-ARCH-1: Positiv und Negativ
    #[test]
    fn t_arch_1_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_arch_1_pos");
        let _ = fs::remove_dir_all(&tmp);
        let crate_dir = tmp.join("crates/contextra-types");
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let cargo_toml = crate_dir.join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"contextra-types\"\n[dependencies]\nthiserror = \"2\"\n",
        )
        .unwrap();
        let rs_file = src_dir.join("lib.rs");
        fs::write(&rs_file, "pub struct MyType;\n").unwrap();
        let v = check_types_purity(&tmp);
        assert!(v.is_empty(), "{v:?}");

        // Negativfall: tokio hinzugefügt
        fs::write(
            &cargo_toml,
            "[package]\nname = \"contextra-types\"\n[dependencies]\ntokio = \"1\"\n",
        )
        .unwrap();
        let v_neg = check_types_purity(&tmp);
        assert!(!v_neg.is_empty(), "Tokio in types muss erkannt werden");
        let _ = fs::remove_dir_all(&tmp);
    }

    // T-ARCH-2: Positiv und Negativ
    #[test]
    fn t_arch_2_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_arch_2_test");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/contextra-vector");
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "pub struct VectorStore;\n").unwrap();
        let k = Krate {
            name: "contextra-vector".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_raw_storage_isolation(&tmp, std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        // Negativfall: direkter LsmStorage Aufruf
        fs::write(&lib_rs, "pub fn evil() { LsmStorage::open(); }\n").unwrap();
        let v_neg = check_raw_storage_isolation(&tmp, &[k]);
        assert!(
            !v_neg.is_empty(),
            "Direkter LsmStorage Aufruf muss erkannt werden"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    // T-LINT-2: Positiv und Negativ
    #[test]
    fn t_lint_2_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_lint_2_test");
        let _ = fs::remove_dir_all(&tmp);
        let src = tmp.join("crates/contextra-store/src");
        fs::create_dir_all(&src).unwrap();
        let lib_rs = src.join("lib.rs");
        fs::write(&lib_rs, "use contextra_ports::Vfs;\n").unwrap();
        let v = check_store_fs_usage(&tmp);
        assert!(v.is_empty(), "{v:?}");

        // Negativfall: std::fs
        fs::write(&lib_rs, "use std::fs::File;\n").unwrap();
        let v_neg = check_store_fs_usage(&tmp);
        assert!(!v_neg.is_empty(), "std::fs muss erkannt werden");
        let _ = fs::remove_dir_all(&tmp);
    }

    // T-UNSAFE-1: Positiv und Negativ
    #[test]
    fn t_unsafe_1_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_unsafe_1_test");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/contextra-types");
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "#![forbid(unsafe_code)]\npub struct Safe;\n").unwrap();
        let k = Krate {
            name: "contextra-types".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_unsafe(&tmp, std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        // Negativfall: unsafe in Nicht-Insel
        fs::write(
            &lib_rs,
            "#![forbid(unsafe_code)]\npub fn bad() { unsafe { } }\n",
        )
        .unwrap();
        let v_neg = check_unsafe(&tmp, &[k]);
        assert!(
            !v_neg.is_empty(),
            "Unsafe in Nicht-Insel muss erkannt werden"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    // T-DEF-1 / T-CFG-1: Positiv und Negativ
    #[test]
    fn t_def_1_negativ_sicherheits_opt_out() {
        let tmp = std::env::temp_dir().join("t_def_1_neg");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let cargo_toml = tmp.join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"test-krate\"\n[features]\nno-encryption = []\n",
        )
        .unwrap();
        let k = Krate {
            name: "test-krate".to_string(),
            dir: tmp.clone(),
            deps: vec![],
        };
        let v = check_feature_hygiene(&tmp, &[k]);
        assert!(v.iter().any(|msg| msg.contains("T-DEF-1")), "{v:?}");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_cfg_1_negativ_unexpected_cfgs() {
        let tmp = std::env::temp_dir().join("t_cfg_1_neg");
        let _ = fs::remove_dir_all(&tmp);
        let src = tmp.join("src");
        fs::create_dir_all(&src).unwrap();
        let cargo_toml = tmp.join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"test-krate\"\n[features]\n",
        )
        .unwrap();
        let rs_file = src.join("lib.rs");
        fs::write(&rs_file, "#![allow(unexpected_cfgs)]\n").unwrap();
        let k = Krate {
            name: "test-krate".to_string(),
            dir: tmp.clone(),
            deps: vec![],
        };
        let v = check_feature_hygiene(&tmp, &[k]);
        assert!(
            v.iter().any(|msg| msg.contains("allow(unexpected_cfgs)")),
            "{v:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }
}
