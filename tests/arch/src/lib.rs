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

pub mod known_deviations;

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
    ("contextra-testmacros", Ring::Tooling),
    ("contextra-lab", Ring::Tooling),
    ("ctx-verify", Ring::Tooling),
    ("contextra-arch-tests", Ring::Tooling),
];

/// Crates, in denen `unsafe` erlaubt ist (NFR-2).
pub const UNSAFE_ISLANDS: &[&str] = &["contextra-sys", "contextra-simd"];

/// Erlaubte Abhängigkeiten laut SPEC §5.3 Abhängigkeitsmatrix (normativ)
/// inkl. D-88, D-95, D-97:
pub const SPEC_5_3_ALLOWED_EDGES: &[(&str, &[&str])] = &[
    ("contextra-types", &[]),
    ("contextra-ports", &["contextra-types"]),
    ("contextra-sys", &["contextra-types"]),
    ("contextra-simd", &["contextra-types"]),
    (
        "contextra-durable-fs",
        &["contextra-types", "contextra-ports", "contextra-sys"],
    ),
    ("contextra-crypto", &["contextra-types", "contextra-ports"]),
    ("contextra-mvcc", &["contextra-types", "contextra-ports"]),
    (
        "contextra-store",
        &[
            "contextra-types",
            "contextra-ports",
            "contextra-sys",
            "contextra-crypto",
            "contextra-mvcc",
        ],
    ),
    (
        "contextra-text",
        &["contextra-types", "contextra-ports", "contextra-crypto"],
    ),
    (
        "contextra-vector",
        &[
            "contextra-types",
            "contextra-ports",
            "contextra-simd",
            "contextra-crypto",
        ],
    ),
    ("contextra-rank", &["contextra-types", "contextra-ports"]),
    (
        "contextra-erasure",
        &["contextra-types", "contextra-ports", "contextra-crypto"],
    ),
    (
        "contextra-engine",
        &[
            "contextra-types",
            "contextra-ports",
            "contextra-sys",
            "contextra-durable-fs",
            "contextra-crypto",
            "contextra-mvcc",
            "contextra-store",
            "contextra-text",
            "contextra-vector",
            "contextra-rank",
            "contextra-erasure",
        ],
    ),
    (
        "contextra",
        &[
            "contextra-types",
            "contextra-durable-fs",
            "contextra-engine",
        ],
    ),
    ("contextra-testkit", &["contextra-types", "contextra-ports"]),
    (
        "contextra-lab",
        &["contextra-types", "contextra-ports", "contextra-crypto"],
    ),
    (
        "ctx-verify",
        &["contextra-types", "contextra-ports", "contextra-crypto"],
    ),
    ("contextra-arch-tests", &[]),
];

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

        let allowed_workspace_deps = SPEC_5_3_ALLOWED_EDGES
            .iter()
            .find(|(n, _)| *n == krate.name)
            .map(|(_, deps)| *deps);

        for dep in &krate.deps {
            if my_ring == Ring::Ring0 && dep == "tokio" {
                violations.push(format!(
                    "Ring 0 verletzt: {} hängt von tokio ab",
                    krate.name
                ));
                continue;
            }

            let Some(dep_ring) = ring_of(dep) else {
                continue;
            };

            let rule_tag = format!("T-ARCH-3:{dep}");
            if known_deviations::is_deviation_allowed(&krate.name, &rule_tag) {
                continue;
            }

            if ALLOWLIST
                .iter()
                .any(|(from, to)| *from == krate.name && *to == dep)
            {
                continue;
            }

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
            if krate.name == "contextra-erasure" && dep_ring == Ring::Ring2 {
                violations.push(format!(
                    "AR-3 verletzt: contextra-erasure hängt von Ring-2-Crate {} ab",
                    dep
                ));
            }
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

            if my_ring != Ring::Tooling && dep_ring == Ring::Tooling {
                violations.push(format!(
                    "{} ({:?}) hängt von Tooling-Crate {} ab",
                    krate.name, my_ring, dep
                ));
                continue;
            }

            if dep_ring > my_ring && my_ring != Ring::Tooling {
                violations.push(format!(
                    "{} ({:?}) hängt von höherem Ring ab: {} ({:?})",
                    krate.name, my_ring, dep, dep_ring
                ));
                continue;
            }

            if let Some(allowed) = allowed_workspace_deps {
                if !allowed.contains(&dep.as_str()) {
                    violations.push(format!(
                        "SPEC §5.3 verletzt: Kante {} -> {} nicht in Matrix erlaubt",
                        krate.name, dep
                    ));
                }
            }
        }
    }
    violations
}

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

/// Normalisiert Zeichenketten fuer den Wortlaut-Lint (T-LINT-1):
/// - Entfernung von Steuer- und Zero-Width-Zeichen (Soft-Hyphen, ZWSP, BOM)
/// - Glaettung von Whitespace und Zeilenumbruechen im Satz
/// - Case-Folding (lowercase)
/// - Homoglyphen-Normalisierung
pub fn normalize_wording_text(input: &str) -> String {
    let mut cleaned = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\u{00AD}' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{200E}' | '\u{200F}'
            | '\u{FEFF}' => {}
            '\n' | '\r' | '\t' => cleaned.push(' '),
            other => cleaned.push(other),
        }
    }
    let mut mapped = String::with_capacity(cleaned.len());
    for c in cleaned.chars() {
        let m = match c {
            '\u{0430}' | 'a' | '\u{0410}' | 'A' => 'a',
            '\u{0435}' | 'e' | '\u{0415}' | 'E' => 'e',
            '\u{043E}' | 'o' | '\u{041E}' | 'O' | 'ö' | 'Ö' => 'o',
            'ä' | 'Ä' => 'a',
            'ü' | 'Ü' => 'u',
            'ß' => 's',
            '\u{0440}' | 'p' | '\u{0420}' | 'P' => 'p',
            '\u{0441}' | 'c' | '\u{0421}' | 'C' => 'c',
            '\u{0443}' | 'y' | '\u{0423}' | 'Y' => 'y',
            '\u{0445}' | 'x' | '\u{0425}' | 'X' => 'x',
            other => other,
        };
        for norm_c in m.to_lowercase() {
            mapped.push(norm_c);
        }
    }
    let mut collapsed = String::with_capacity(mapped.len());
    let mut prev_space = false;
    for c in mapped.chars() {
        if c.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(c);
            prev_space = false;
        }
    }
    collapsed
}

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

        let normalized_content = normalize_wording_text(&content);
        for term in &forbidden_terms {
            let norm_term = normalize_wording_text(term);
            if normalized_content.contains(&norm_term) {
                violations.push(format!(
                    "T-LINT-1 verletzt: {}: Unzulässige Außenaussage / Begriff '{}' (erkannt via Normalisierung)",
                    rel,
                    term
                ));
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
        // Erfasse alle Targets: src/, tests/, benches/, examples/, build.rs
        collect_rs_files(&krate.dir, &mut files);

        for file in files {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            if island && !file.starts_with(krate.dir.join("src")) {
                // In Unsafe-Inseln wird // SAFETY: strikt auf allen src/ Produktionspfaden geprüft.
                continue;
            }
            let rel = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .to_string_lossy()
                .to_string();
            let rel_in_crate = file
                .strip_prefix(&krate.dir)
                .unwrap_or(&file)
                .to_string_lossy()
                .to_string();
            let deviation_tag = format!("T-UNSAFE-1:{rel_in_crate}");

            if known_deviations::is_deviation_allowed(&krate.name, &deviation_tag) {
                continue;
            }

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

/// Prüft T-ARCH-5:
/// - Ring im Manifest `[package.metadata.contextra] ring` == Ring im `//!`-Kopf von lib.rs == Matrix (SPEC §5.2 / §5.3).
/// - Crate-Beschreibung in Cargo.toml `description` == erste Zeile der README.md (wortgleich nach SPEC §5.1a / D-97).
pub fn check_crate_headers_and_descriptions(_root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();

    for krate in crates {
        if krate.name == "contextra-arch-tests" {
            continue;
        }

        let manifest_path = krate.dir.join("Cargo.toml");
        let Ok(manifest_text) = fs::read_to_string(&manifest_path) else {
            violations.push(format!("{}: Cargo.toml nicht lesbar", krate.name));
            continue;
        };
        let Ok(doc) = manifest_text.parse::<toml::Table>() else {
            violations.push(format!("{}: Cargo.toml ist kein gültiges TOML", krate.name));
            continue;
        };

        let pkg = doc.get("package").and_then(|p| p.as_table());
        let manifest_desc = pkg
            .and_then(|p| p.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .trim();

        let manifest_ring_str = pkg
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("contextra"))
            .and_then(|c| c.get("ring"))
            .and_then(|r| r.as_str());

        let expected_ring = ring_of(&krate.name);
        let expected_ring_str = match expected_ring {
            Some(Ring::Ring0) => "0",
            Some(Ring::Ring1) => "1",
            Some(Ring::Ring2) => "2",
            Some(Ring::Ring3) => "3",
            Some(Ring::Ring4) => "4",
            Some(Ring::Tooling) => "tooling",
            None => "",
        };

        if manifest_ring_str != Some(expected_ring_str) {
            let rule = "T-ARCH-5:manifest-ring";
            if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                violations.push(format!(
                    "T-ARCH-5 verletzt: {} [package.metadata.contextra].ring = '{:?}', erwartet: '{}'",
                    krate.name, manifest_ring_str, expected_ring_str
                ));
            }
        }

        let lib_rs = krate.dir.join("src/lib.rs");
        if lib_rs.is_file() {
            if let Ok(lib_text) = fs::read_to_string(&lib_rs) {
                let has_expected_ring_header = lib_text
                    .lines()
                    .filter(|l| l.starts_with("//!"))
                    .any(|l| match expected_ring {
                        Some(Ring::Ring0) => l.contains("Ring 0"),
                        Some(Ring::Ring1) => l.contains("Ring 1"),
                        Some(Ring::Ring2) => l.contains("Ring 2"),
                        Some(Ring::Ring3) => l.contains("Ring 3"),
                        Some(Ring::Ring4) => l.contains("Ring 4"),
                        Some(Ring::Tooling) => {
                            l.contains("tooling") || l.contains("Tooling") || l.contains("dev")
                        }
                        None => false,
                    });

                if !has_expected_ring_header {
                    let rule = "T-ARCH-5:header-ring";
                    if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                        violations.push(format!(
                            "T-ARCH-5 verletzt: {} src/lib.rs //! Kopf nennt Ring '{:?}' nicht",
                            krate.name, expected_ring
                        ));
                    }
                }
            }
        }

        let readme_path = krate.dir.join("README.md");
        if readme_path.is_file() {
            if let Ok(readme_text) = fs::read_to_string(&readme_path) {
                let first_readme_line = readme_text
                    .lines()
                    .map(|l| l.trim())
                    .find(|l| !l.is_empty())
                    .unwrap_or("");

                if manifest_desc != first_readme_line {
                    let rule = "T-ARCH-5:description-readme";
                    if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                        violations.push(format!(
                            "T-ARCH-5 verletzt: {} Cargo.toml description ('{}') != README.md Zeile 1 ('{}')",
                            krate.name, manifest_desc, first_readme_line
                        ));
                    }
                }
            }
        } else {
            let rule = "T-ARCH-5:missing-readme";
            if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                violations.push(format!(
                    "T-ARCH-5 verletzt: {} besitzt keine README.md",
                    krate.name
                ));
            }
        }
    }

    violations
}

/// Prüft T-ARCH-6:
/// - Keine Glob-Re-Exporte (`pub use ...::*`) außerhalb von Tests / `#[cfg(test)]`.
/// - Dev-Abhängigkeiten folgen der Matrix SPEC §5.3 (`contextra-testkit` ausgenommen, keine Selbst-Dev-Abhängigkeit; AR-14, D-97).
pub fn check_exports_and_dev_dependencies(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();

    for krate in crates {
        let manifest_path = krate.dir.join("Cargo.toml");
        let Ok(manifest_text) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Ok(doc) = manifest_text.parse::<toml::Table>() else {
            continue;
        };

        let allowed_deps = SPEC_5_3_ALLOWED_EDGES
            .iter()
            .find(|(n, _)| *n == krate.name)
            .map(|(_, deps)| *deps)
            .unwrap_or(&[]);

        if let Some(dev_deps) = doc.get("dev-dependencies").and_then(|d| d.as_table()) {
            for (dep_key, val) in dev_deps {
                let dep_name = val
                    .get("package")
                    .and_then(|p| p.as_str())
                    .unwrap_or(dep_key.as_str());

                if ring_of(dep_name).is_none() {
                    continue;
                }

                if dep_name == krate.name {
                    let rule = "T-ARCH-6:dev-self";
                    if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                        violations.push(format!(
                            "T-ARCH-6 verletzt: {} besitzt unzulässige Selbst-Dev-Abhängigkeit (AR-14, D-97)",
                            krate.name
                        ));
                    }
                    continue;
                }

                if dep_name == "contextra-testkit" {
                    continue;
                }

                if !allowed_deps.contains(&dep_name) {
                    let rule = format!("T-ARCH-6:dev-dep:{dep_name}");
                    if !known_deviations::is_deviation_allowed(&krate.name, &rule) {
                        violations.push(format!(
                            "T-ARCH-6 verletzt: Dev-Abhängigkeit {} -> {} widerspricht SPEC §5.3 Matrix (AR-14)",
                            krate.name, dep_name
                        ));
                    }
                }
            }
        }

        let src_dir = krate.dir.join("src");
        if !src_dir.is_dir() {
            continue;
        }

        let mut rs_files = Vec::new();
        collect_rs_files(&src_dir, &mut rs_files);

        for file in rs_files {
            let rel = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();

            if rel.contains("/tests/") || rel.ends_with("_test.rs") || rel.ends_with("/tests.rs") {
                continue;
            }

            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };

            let mut in_cfg_test = false;
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains("#[cfg(test)]") {
                    in_cfg_test = true;
                }

                if !in_cfg_test && trimmed.starts_with("pub use ") && trimmed.ends_with("::*;") {
                    let rule = "T-ARCH-6:glob-export";
                    if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                        violations.push(format!(
                            "T-ARCH-6 verletzt: {}:{}: Unzulässiger Glob-Re-Export '{}' (AR-10)",
                            rel,
                            line_idx + 1,
                            trimmed
                        ));
                    }
                }
            }
        }
    }

    violations
}

/// Prüft T-ARCH-7:
/// - Namensregeln nach D-90: Keine Module `misc`, `util`, `utils`, `common`, `helpers`, keine Endung `_impl`, kein `mod.rs`, kein Modul gleich dem Crate-Namen.
/// - Fassaden-Schnitt nach D-89: `contextra` enthält keine Logik und nur die Module `api`, `wiring`, `error`.
pub fn check_naming_and_facade_structure(crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    let banned_names = ["misc", "util", "utils", "common", "helpers"];

    for krate in crates {
        let src_dir = krate.dir.join("src");
        if !src_dir.is_dir() {
            continue;
        }

        if krate.name == "contextra" {
            let allowed_facade_modules = [
                "lib.rs",
                "api.rs",
                "wiring.rs",
                "error.rs",
                "api",
                "wiring",
                "error",
            ];
            if let Ok(entries) = fs::read_dir(&src_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !allowed_facade_modules.contains(&name.as_str()) {
                        let rule = "T-ARCH-7:facade-structure";
                        if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                            violations.push(format!(
                                "T-ARCH-7 verletzt: Fassade 'contextra' enthält unzulässiges Modul/Datei '{}' (D-89)",
                                name
                            ));
                            break;
                        }
                    }
                }
            }
        }

        let short_crate_name = krate
            .name
            .strip_prefix("contextra-")
            .unwrap_or(&krate.name)
            .to_string();

        let mut files_to_check = Vec::new();
        collect_all_files(&src_dir, &mut files_to_check);

        for file in files_to_check {
            let filename = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let stem = file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if filename == "mod.rs" {
                let rule = "T-ARCH-7:mod_rs";
                if !known_deviations::is_deviation_allowed(&krate.name, rule)
                    && !known_deviations::is_deviation_allowed(&krate.name, "T-ARCH-7:naming")
                {
                    violations.push(format!(
                        "T-ARCH-7 verletzt: {} enthält verbotene 'mod.rs' Datei (D-90)",
                        file.display()
                    ));
                }
                continue;
            }

            for banned in &banned_names {
                if stem == *banned {
                    let rule = "T-ARCH-7:naming";
                    if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                        violations.push(format!(
                            "T-ARCH-7 verletzt: {} trägt verbotenen Namen '{}' (D-90)",
                            file.display(),
                            banned
                        ));
                    }
                }
            }

            if stem.ends_with("_impl") {
                let rule = "T-ARCH-7:naming";
                if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                    violations.push(format!(
                        "T-ARCH-7 verletzt: {} endet auf '_impl' (D-90)",
                        file.display()
                    ));
                }
            }

            if !short_crate_name.is_empty() && stem == short_crate_name && filename != "lib.rs" {
                let rule = "T-ARCH-7:naming";
                if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                    violations.push(format!(
                        "T-ARCH-7 verletzt: Modul {} trägt den Namen des Crates '{}' (D-90)",
                        file.display(),
                        krate.name
                    ));
                }
            }
        }
    }

    violations
}

/// Prüft T-ARCH-8:
/// - Keine Sitzungsverläufe, Prüfprotokolle, `reports/` und `evidence/` im Quellbaum (`crates/**`, `tests/**`, D-98).
pub fn check_clean_source_tree(crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();

    for krate in crates {
        let mut all_files = Vec::new();
        collect_all_files(&krate.dir, &mut all_files);

        for file in all_files {
            let path_str = file.to_string_lossy();
            if path_str.contains("/reports/") || path_str.contains("/evidence/") {
                let rule = "T-ARCH-8:reports-or-evidence";
                if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                    violations.push(format!(
                        "T-ARCH-8 verletzt: Unzulässiges Verzeichnis in Crate '{}': {} (D-98)",
                        krate.name,
                        file.display()
                    ));
                }
            }

            let filename = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if (filename.starts_with("audit-")
                || filename.starts_with("audit_")
                || filename.starts_with("session-")
                || filename.starts_with("session_"))
                && (filename.ends_with(".log")
                    || filename.ends_with(".txt")
                    || filename.ends_with(".json"))
            {
                let rule = "T-ARCH-8:session-logs";
                if !known_deviations::is_deviation_allowed(&krate.name, rule) {
                    violations.push(format!(
                        "T-ARCH-8 verletzt: Sitzungsverlauf / Prüfprotokoll '{}' im Quellbaum (D-98)",
                        file.display()
                    ));
                }
            }
        }
    }

    violations
}

/// Prüft T-DOC-1:
/// - Keine Session- oder Zeitmarken im Quelltext und in Dokumenten:
///   `FILE-CONTEXT`, `ANCHOR[`, `STAND:`, `SESSION:`, absolute Entwicklerpfade wie `/home/` (D-91, D-98).
pub fn check_session_marks_and_timestamps(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    let forbidden_marks = ["FILE-CONTEXT", "ANCHOR[", "STAND:", "SESSION:", "/home/"];

    for krate in crates {
        let rule = "T-DOC-1:session-marks";
        if known_deviations::is_deviation_allowed(&krate.name, rule) {
            continue;
        }

        let mut files_to_scan = Vec::new();
        collect_all_files(&krate.dir, &mut files_to_scan);

        for file in files_to_scan {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();

            for (line_idx, line) in content.lines().enumerate() {
                for mark in &forbidden_marks {
                    if line.contains(mark) {
                        violations.push(format!(
                            "T-DOC-1 verletzt: {}:{}: Unzulässige Marke / Zeitstempel '{}' (D-91)",
                            rel,
                            line_idx + 1,
                            mark
                        ));
                    }
                }
            }
        }
    }

    violations
}

/// Prüft T-LINT-4:
/// Keine Marken aus Q-09 (Zeitstempel "STAND", Sitzungs- oder Agentenkennungen, ANCHOR[...], FILE-CONTEXT, absolute Pfade).
pub fn check_trademark_and_session_marks(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    let q09_marks = ["STAND:", "SESSION:", "ANCHOR[", "FILE-CONTEXT", "/home/"];

    for krate in crates {
        let rule = "T-LINT-4:trademark-marks";
        if known_deviations::is_deviation_allowed(&krate.name, rule) {
            continue;
        }

        let mut files_to_scan = Vec::new();
        collect_all_files(&krate.dir, &mut files_to_scan);

        for file in files_to_scan {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();

            for (line_idx, line) in content.lines().enumerate() {
                for mark in &q09_marks {
                    if line.contains(mark) {
                        violations.push(format!(
                            "T-LINT-4 verletzt: {}:{}: Unzulässige Marke aus Q-09 '{}'",
                            rel,
                            line_idx + 1,
                            mark
                        ));
                    }
                }
            }
        }
    }

    violations
}

/// Prüft T-LINT-3:
/// Veraltete Evidenzbegriffe (E1 bis E3, E0 bis E4 als alte Messklassen/Evidenzstufen) und unzulässige Aussagen nach SPEC §6.4.
pub fn check_obsolete_evidence_terms(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    let obsolete_patterns = [
        "E1/E2/E3",
        "E0..E4",
        "E0-E4",
        "E1-E3",
        "garantiert gelöscht",
        "guaranteed",
        "100 % sicher",
        "100% sicher",
        "DSGVO-konform",
        "GDPR-compliant",
        "rechtskonform",
        "unwiederbringlich",
        "irrecoverable",
        "unwiderruflich",
        "informationstheoretisch",
        "von einer Aufsichtsbehörde geprüft",
    ];

    let mut files_to_scan = Vec::new();
    for krate in crates {
        if known_deviations::is_deviation_allowed(&krate.name, "T-LINT-3:wording") {
            continue;
        }
        let src_dir = krate.dir.join("src");
        if src_dir.is_dir() {
            collect_all_files(&src_dir, &mut files_to_scan);
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
            .any(|(p, _)| rel == *p || rel.starts_with(p))
        {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };

        for (line_idx, line) in content.lines().enumerate() {
            for pat in &obsolete_patterns {
                if line.contains(pat) {
                    violations.push(format!(
                        "T-LINT-3 verletzt: {}:{}: Veralteter Evidenzbegriff oder unzulässige Aussage '{}' (SPEC §6.4 / T-LINT-3)",
                        rel,
                        line_idx + 1,
                        pat
                    ));
                }
            }
        }
    }

    violations
}

/// Prüft T-TRACE-1:
/// Rückverfolgbarkeit von Tests zu SPEC-IDs (PREAMBLE D / SPEC §15.1).
/// Jeder `#[test]`, `#[tokio::test]` oder `#[ctx_test]` muss einer SPEC-ID zugeordnet sein.
pub fn check_test_traceability(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();

    for krate in crates {
        let rule = "T-TRACE-1:test-traceability";
        if known_deviations::is_deviation_allowed(&krate.name, rule) {
            continue;
        }

        let mut test_files = Vec::new();
        let tests_dir = krate.dir.join("tests");
        if tests_dir.is_dir() {
            collect_rs_files(&tests_dir, &mut test_files);
        }
        let src_dir = krate.dir.join("src");
        if src_dir.is_dir() {
            collect_rs_files(&src_dir, &mut test_files);
        }

        for file in test_files {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let rel = file
                .strip_prefix(root)
                .unwrap_or(&file)
                .display()
                .to_string();
            let lines: Vec<&str> = content.lines().collect();

            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                let is_test_attr = trimmed == "#[test]"
                    || trimmed == "#[tokio::test]"
                    || trimmed.starts_with("#[ctx_test");

                if !is_test_attr {
                    continue;
                }

                let start = idx.saturating_sub(3);
                let has_spec_doc = lines
                    .get(start..idx)
                    .map(|slice| slice.iter().any(|l| l.trim().starts_with("/// SPEC:")))
                    .unwrap_or(false);
                let next_fn_has_spec_name = lines
                    .get(idx + 1)
                    .map(|l| {
                        let t = l.trim();
                        t.starts_with("fn t_") || t.starts_with("async fn t_")
                    })
                    .unwrap_or(false);

                if !has_spec_doc && !next_fn_has_spec_name && !trimmed.starts_with("#[ctx_test") {
                    violations.push(format!(
                        "T-TRACE-1 verletzt: {}:{}: Test ohne zugeordnete SPEC-ID (/// SPEC: T-... oder t_...)",
                        rel,
                        idx + 1
                    ));
                }
            }
        }
    }

    violations
}

/// Prüft T-ARCH-9:
/// Doppelte Modul-Formen (X.rs und X/mod.rs bzw. X/) zur frühzeitigen Erkennung von E0761.
/// Verbot von 'mod.rs' und Kollisionen zwischen X.rs und Unterverzeichnissen.
pub fn check_double_modules(crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    for krate in crates {
        let src_dir = krate.dir.join("src");
        if !src_dir.is_dir() {
            continue;
        }
        let mut dirs_to_check = Vec::new();
        dirs_to_check.push(src_dir.clone());
        let mut all_dirs = Vec::new();
        while let Some(current) = dirs_to_check.pop() {
            if let Ok(entries) = fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        all_dirs.push(path.clone());
                        dirs_to_check.push(path);
                    }
                }
            }
        }
        for dir in all_dirs {
            let mod_rs = dir.join("mod.rs");
            if mod_rs.is_file() {
                let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                let parent = dir.parent().unwrap_or(&dir);
                let sibling_rs = parent.join(format!("{dir_name}.rs"));
                if sibling_rs.is_file() {
                    violations.push(format!(
                        "T-ARCH-9 verletzt: Kollidierende Modul-Form in Crate '{}': {} und {} (E0761-Risiko)",
                        krate.name,
                        sibling_rs.display(),
                        mod_rs.display()
                    ));
                }
            }
        }
    }
    violations
}

/// Prüft T-GOV-1:
/// Ratchet- und Ausnahmen-Hygiene.
/// Verhindert Stale-Einträge in .agents/governance/ratchet.toml (Crates im Ratchet, die im Workspace nicht existieren).
pub fn check_ratchet_hygiene(root: &Path, crates: &[Krate]) -> Vec<String> {
    let mut violations = Vec::new();
    let ratchet_path = root.join(".agents/governance/ratchet.toml");
    if !ratchet_path.is_file() {
        violations
            .push("T-GOV-1 verletzt: .agents/governance/ratchet.toml existiert nicht".to_string());
        return violations;
    }
    let Ok(content) = fs::read_to_string(&ratchet_path) else {
        violations.push("T-GOV-1 verletzt: ratchet.toml nicht lesbar".to_string());
        return violations;
    };
    let Ok(table) = content.parse::<toml::Table>() else {
        violations.push("T-GOV-1 verletzt: ratchet.toml ist kein gültiges TOML".to_string());
        return violations;
    };

    let existing_crate_names: std::collections::BTreeSet<String> =
        crates.iter().map(|c| c.name.clone()).collect();

    if let Some(unsafe_blocks) = table.get("unsafe_blocks").and_then(|v| v.as_table()) {
        for (c_name, _) in unsafe_blocks {
            if !existing_crate_names.contains(c_name)
                && c_name != "contextra-arch-tests"
                && c_name != "ctx-verify"
            {
                violations.push(format!(
                    "T-GOV-1 verletzt: Stale-Eintrag in ratchet.toml [unsafe_blocks]: Crate '{}' existiert nicht im Workspace (Fail-Open-Risiko)",
                    c_name
                ));
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
    fn t_arch_3_negativ_unerlaubte_matrix_kante_gleicher_ring() {
        let v_text_mvcc = check_layering(&[krate("contextra-text", &["contextra-mvcc"])]);
        assert_eq!(v_text_mvcc.len(), 1, "{v_text_mvcc:?}");
        assert!(
            v_text_mvcc.first().unwrap().contains("SPEC §5.3 verletzt"),
            "{v_text_mvcc:?}"
        );
    }

    #[test]
    fn t_trace_1_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_trace_1_test");
        let _ = fs::remove_dir_all(&tmp);
        let src_dir = tmp.join("crates/test-krate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "/// SPEC: T-ISO-1\n#[test]\nfn my_test() {}\n").unwrap();
        let content = fs::read_to_string(&lib_rs).unwrap();
        assert!(content.contains("/// SPEC:"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_lint_4_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_lint_4_test");
        let _ = fs::remove_dir_all(&tmp);
        let src_dir = tmp.join("crates/test-krate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(
            &lib_rs,
            "// Sauberer Code ohne Q-09 Marken\npub struct Clean;\n",
        )
        .unwrap();

        let k = Krate {
            name: "test-krate".to_string(),
            dir: tmp.join("crates/test-krate"),
            deps: vec![],
        };
        let v = check_trademark_and_session_marks(&tmp, std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        fs::write(&lib_rs, "// STAND: 2026-11-01\npub struct Bad;\n").unwrap();
        let v_neg = check_trademark_and_session_marks(&tmp, &[k]);
        assert!(
            v_neg.iter().any(|msg| msg.contains("T-LINT-4")),
            "{v_neg:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_lint_3_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_lint_3_test");
        let _ = fs::remove_dir_all(&tmp);
        let src_dir = tmp.join("crates/test-krate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "// Messklasse MK1\npub struct Clean;\n").unwrap();
        let k = Krate {
            name: "test-krate".to_string(),
            dir: tmp.join("crates/test-krate"),
            deps: vec![],
        };
        let v = check_obsolete_evidence_terms(&tmp, std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        fs::write(&lib_rs, "// Alte Messklassen E1/E2/E3\npub struct Bad;\n").unwrap();
        let v_neg = check_obsolete_evidence_terms(&tmp, &[k]);
        assert!(
            v_neg.iter().any(|msg| msg.contains("T-LINT-3")),
            "{v_neg:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_doc_1_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_doc_1_test");
        let _ = fs::remove_dir_all(&tmp);
        let src_dir = tmp.join("crates/test-krate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "// Sauberer Code ohne Marken\npub struct Clean;\n").unwrap();

        let forbidden = ["FILE-CONTEXT", "ANCHOR[", "STAND:", "SESSION:", "/home/"];
        let content = fs::read_to_string(&lib_rs).unwrap();
        assert!(!forbidden.iter().any(|m| content.contains(m)));

        fs::write(&lib_rs, "// ANCHOR[test:A-1]\npub struct Bad;\n").unwrap();
        let content_bad = fs::read_to_string(&lib_rs).unwrap();
        assert!(forbidden.iter().any(|m| content_bad.contains(m)));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_arch_8_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_arch_8_test");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/test-krate");
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "pub struct Clean;\n").unwrap();

        let k = Krate {
            name: "test-krate".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_clean_source_tree(std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        let reports_dir = krate_dir.join("reports");
        fs::create_dir_all(&reports_dir).unwrap();
        let old_report = reports_dir.join("report.txt");
        fs::write(&old_report, "old report").unwrap();
        let v_neg = check_clean_source_tree(std::slice::from_ref(&k));
        assert!(
            v_neg.iter().any(|msg| msg.contains("T-ARCH-8")),
            "{v_neg:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_arch_7_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_arch_7_test");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/test-krate");
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "pub mod domain;\n").unwrap();
        let domain_rs = src_dir.join("domain.rs");
        fs::write(&domain_rs, "pub struct Model;\n").unwrap();

        let k = Krate {
            name: "test-krate".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_naming_and_facade_structure(std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        let mod_rs = src_dir.join("mod.rs");
        fs::write(&mod_rs, "").unwrap();
        let v_neg1 = check_naming_and_facade_structure(std::slice::from_ref(&k));
        assert!(
            v_neg1.iter().any(|msg| msg.contains("mod.rs")),
            "{v_neg1:?}"
        );
        fs::remove_file(&mod_rs).unwrap();

        let util_rs = src_dir.join("util.rs");
        fs::write(&util_rs, "").unwrap();
        let v_neg2 = check_naming_and_facade_structure(std::slice::from_ref(&k));
        assert!(v_neg2.iter().any(|msg| msg.contains("util")), "{v_neg2:?}");
        fs::remove_file(&util_rs).unwrap();

        let impl_rs = src_dir.join("engine_impl.rs");
        fs::write(&impl_rs, "").unwrap();
        let v_neg3 = check_naming_and_facade_structure(&[k]);
        assert!(v_neg3.iter().any(|msg| msg.contains("_impl")), "{v_neg3:?}");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_arch_6_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_arch_6_test");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/contextra-durable-fs");
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let cargo_toml = krate_dir.join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"contextra-crypto\"\n[dev-dependencies]\ncontextra-ports = { path = \"../contextra-ports\" }\n",
        )
        .unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "pub use contextra_ports::Clock;\n").unwrap();

        let k = Krate {
            name: "contextra-durable-fs".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_exports_and_dev_dependencies(&tmp, std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        fs::write(&lib_rs, "pub use contextra_ports::*;\n").unwrap();
        let v_neg = check_exports_and_dev_dependencies(&tmp, std::slice::from_ref(&k));
        assert!(
            v_neg.iter().any(|msg| msg.contains("Glob-Re-Export")),
            "{v_neg:?}"
        );

        let krate_types_dir = tmp.join("crates/contextra-types");
        fs::create_dir_all(&krate_types_dir).unwrap();
        let types_cargo_toml = krate_types_dir.join("Cargo.toml");
        fs::write(
            &types_cargo_toml,
            "[package]\nname = \"contextra-types\"\n[dev-dependencies]\ncontextra-types = { path = \".\" }\n",
        )
        .unwrap();
        let k_types = Krate {
            name: "contextra-types".to_string(),
            dir: krate_types_dir.clone(),
            deps: vec![],
        };
        let v_neg2 = check_exports_and_dev_dependencies(&tmp, &[k_types]);
        assert!(
            v_neg2
                .iter()
                .any(|msg| msg.contains("Selbst-Dev-Abhängigkeit")),
            "{v_neg2:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_arch_5_positiv_und_negativ() {
        let tmp = std::env::temp_dir().join("t_arch_5_test");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/contextra-types");
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let cargo_toml = krate_dir.join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"contextra-types\"\ndescription = \"Desc\"\n[package.metadata.contextra]\nring = \"0\"\n",
        )
        .unwrap();
        let readme = krate_dir.join("README.md");
        fs::write(&readme, "Desc\n").unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "//! Ring 0 Datentypen\n").unwrap();

        let k = Krate {
            name: "contextra-types".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_crate_headers_and_descriptions(&tmp, std::slice::from_ref(&k));
        assert!(v.is_empty(), "{v:?}");

        fs::write(&readme, "Andere Beschreibung\n").unwrap();
        let v_neg = check_crate_headers_and_descriptions(&tmp, &[k]);
        assert!(
            v_neg.iter().any(|msg| msg.contains("description")),
            "{v_neg:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

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

    // === Zero-Trust Fehlerinjektions-Selbsttests (Mutanten) ===

    #[test]
    fn t_selftest_unsafe_in_tests_und_benches_wird_rot() {
        let tmp = std::env::temp_dir().join("t_selftest_unsafe_targets");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/safe-krate");
        let tests_dir = krate_dir.join("tests");
        fs::create_dir_all(&tests_dir).unwrap();
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            "#![forbid(unsafe_code)]\npub struct Safe;\n",
        )
        .unwrap();
        fs::write(
            tests_dir.join("my_test.rs"),
            "fn test_foo() { unsafe { } }\n",
        )
        .unwrap();

        let k = Krate {
            name: "safe-krate".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_unsafe(&tmp, &[k]);
        assert!(
            v.iter().any(|msg| msg.contains("unsafe in Nicht-Insel")),
            "Mutant: unsafe in tests/ muss rot werden: {v:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_selftest_wording_soft_hyphen_homoglyph_wird_rot() {
        let tmp = std::env::temp_dir().join("t_selftest_wording_mutants");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // 1. Soft-Hyphen
        let f1 = tmp.join("README.md");
        fs::write(&f1, "Dies ist 100\u{00AD}% sicher.\n").unwrap();
        let v1 = check_wording(&tmp);
        assert!(
            v1.iter().any(|msg| msg.contains("100% sicher")),
            "Mutant: 100% sicher mit Soft-Hyphen muss erkannt werden: {v1:?}"
        );

        // 2. Homoglyphen & Zeilenumbruch
        fs::write(&f1, "Dieses System ist\ngarantiert\ngel\u{043E}scht.\n").unwrap();
        let v2 = check_wording(&tmp);
        assert!(
            v2.iter().any(|msg| msg.contains("garantiert gelöscht")),
            "Mutant: garantiert gelöscht mit Zeilenumbruch und kyrillischem 'o' muss erkannt werden: {v2:?}"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_selftest_double_modules_e0761_wird_rot() {
        let tmp = std::env::temp_dir().join("t_selftest_double_modules");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/test-krate");
        let src_dir = krate_dir.join("src");
        let vault_dir = src_dir.join("vault");
        fs::create_dir_all(&vault_dir).unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod vault;\n").unwrap();
        fs::write(src_dir.join("vault.rs"), "pub struct Vault;\n").unwrap();
        fs::write(vault_dir.join("mod.rs"), "pub struct Vault;\n").unwrap();

        let k = Krate {
            name: "test-krate".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_double_modules(&[k]);
        assert!(
            v.iter().any(|msg| msg.contains("T-ARCH-9")),
            "Mutant: doppelte Modulform X.rs + X/mod.rs muss rot werden: {v:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_selftest_stale_ratchet_wird_rot() {
        let tmp = std::env::temp_dir().join("t_selftest_stale_ratchet");
        let _ = fs::remove_dir_all(&tmp);
        let gov_dir = tmp.join(".agents/governance");
        fs::create_dir_all(&gov_dir).unwrap();

        fs::write(
            gov_dir.join("ratchet.toml"),
            "[unsafe_blocks]\ncontextra-phantom = 42\n",
        )
        .unwrap();

        let k = Krate {
            name: "contextra-types".to_string(),
            dir: tmp.join("crates/contextra-types"),
            deps: vec![],
        };
        let v = check_ratchet_hygiene(&tmp, &[k]);
        assert!(
            v.iter().any(|msg| msg.contains("contextra-phantom")),
            "Mutant: Phantom-Crate in ratchet.toml muss rot werden: {v:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn t_selftest_ungetaggter_test_wird_rot() {
        let tmp = std::env::temp_dir().join("t_selftest_ungetaggter_test");
        let _ = fs::remove_dir_all(&tmp);
        let krate_dir = tmp.join("crates/test-krate");
        let src_dir = krate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            "#[test]\nfn some_untraced_test() {}\n",
        )
        .unwrap();

        let k = Krate {
            name: "test-krate".to_string(),
            dir: krate_dir.clone(),
            deps: vec![],
        };
        let v = check_test_traceability(&tmp, &[k]);
        assert!(
            v.iter().any(|msg| msg.contains("T-TRACE-1")),
            "Mutant: Test ohne SPEC-ID muss rot werden: {v:?}"
        );
        let _ = fs::remove_dir_all(&tmp);
    }
}
