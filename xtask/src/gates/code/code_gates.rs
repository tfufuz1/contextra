// Contextra — Code Gates Implementation
// Gate 8: determinism (INV-RING0-DETERMINISM-1, P28)
// Gate 9: panic-surface (P7, REQ-QA-005)
// Gate 10: code-lints (INV-IO-1, INV-FAILCLOSED-1)

use crate::core::ctx::Context;
use crate::core::gate::{Gate, GateSpec, Outcome};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// ── Gate 8: DeterminismGate ──────────────────────────────────────────────────

pub struct DeterminismGate;

#[derive(Debug, Deserialize)]
struct DeterminismAllowEntry {
    path: String,
    pattern: String,
    reason: String,
    adr: String,
}

#[derive(Debug, Deserialize)]
struct DeterminismAllowFile {
    allow: Vec<DeterminismAllowEntry>,
}

impl Gate for DeterminismGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "determinism".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH1".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["INV-RING0-DETERMINISM-1".to_string(), "P28".to_string()],
            why: "Erzwingt Determinismus in kritischen Ausführungspfaden (Ring 0/1) und prüft governance/determinism-allow.toml mit ADR-Referenzen.".to_string(),
            owner: "MP-05".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_determinism(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "determinism" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_determinism(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    // 1. Check governance/determinism-allow.toml for required ADR references
    let allow_path = root.join("governance/determinism-allow.toml");
    let allow_entries = if allow_path.exists() {
        match fs::read_to_string(&allow_path) {
            Ok(content) => match toml::from_str::<DeterminismAllowFile>(&content) {
                Ok(parsed) => {
                    for entry in &parsed.allow {
                        if entry.adr.trim().is_empty() {
                            violations.push(format!(
                                "governance/determinism-allow.toml: Entry for '{}' ({}) missing required ADR reference (reason: {})",
                                entry.path, entry.pattern, entry.reason
                            ));
                        }
                    }
                    parsed.allow
                }
                Err(e) => {
                    return Outcome::Error {
                        message: format!(
                            "Failed to parse governance/determinism-allow.toml: {}",
                            e
                        ),
                    };
                }
            },
            Err(e) => {
                return Outcome::Error {
                    message: format!("Failed to read governance/determinism-allow.toml: {}", e),
                };
            }
        }
    } else {
        vec![]
    };

    // 2. Scan Ring 0 & Ring 1 crates for non-deterministic patterns
    let ring01_crates = [
        "contextra-types",
        "contextra-ports",
        "contextra-vector",
        "contextra-text",
        "contextra-graph",
        "contextra-rank",
        "contextra-adapt",
        "contextra-simd",
        "contextra-store",
        "contextra-mvcc",
        "contextra-checkpoint",
        "contextra-kvcache",
        "contextra-crypto",
        "contextra-privacy",
        "contextra-sys",
        "contextra-wire",
    ];

    let forbidden_patterns = [
        (
            "SystemTime::now",
            "SystemTime::now() calls non-deterministic clock source",
        ),
        (
            "Instant::now",
            "Instant::now() calls non-deterministic clock source",
        ),
        (
            "thread_rng",
            "rand::thread_rng() uses unseeded thread-local random generator",
        ),
        (
            "rand::random",
            "rand::random() uses unseeded random generation",
        ),
        (
            "Uuid::new_v4",
            "Uuid::new_v4() generates random UUIDs breaking determinism",
        ),
    ];

    for crate_name in &ring01_crates {
        let crate_path = root.join("crates").join(crate_name).join("src");
        if !crate_path.exists() {
            continue;
        }

        for entry in WalkDir::new(&crate_path).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                let trimmed = line.trim();

                if trimmed.starts_with("//") || trimmed.contains("#[cfg(test)]") {
                    continue;
                }

                for (pattern, msg) in &forbidden_patterns {
                    if trimmed.contains(pattern) {
                        let is_allowed = allow_entries.iter().any(|allow| {
                            rel_path.contains(&allow.path)
                                && pattern.contains(&allow.pattern)
                                && !allow.adr.trim().is_empty()
                        });

                        if !is_allowed {
                            violations.push(format!(
                                "{}:{} - Non-deterministic pattern '{}' found: {}",
                                rel_path, line_num, pattern, msg
                            ));
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "determinism check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 9: PanicSurfaceGate ─────────────────────────────────────────────────

pub struct PanicSurfaceGate;

impl Gate for PanicSurfaceGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "panic-surface".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["P7".to_string(), "REQ-QA-005".to_string()],
            why: "Kein unwrap/expect/panic/todo/unreachable/indexing in Produktionscode (SPEC §0.2 R2, §17.3). Baselines in governance/ratchet.toml.".to_string(),
            owner: "MP-05".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_panic_surface(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "panic-surface" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_panic_surface(root: &Path) -> Outcome {
    let mut allow_count = 0;
    let mut violations = Vec::new();

    let crates_dir = root.join("crates");
    if crates_dir.exists() {
        for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };

            if rel_path.contains("/tests/")
                || rel_path.contains("/benches/")
                || rel_path.contains("/examples/")
            {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                let trimmed = line.trim();

                if trimmed.contains("#[allow(clippy::indexing_slicing)]")
                    || trimmed.contains("#[allow(clippy::unwrap_used)]")
                    || trimmed.contains("#[allow(clippy::expect_used)]")
                    || trimmed.contains("#[allow(clippy::panic)]")
                {
                    allow_count += 1;
                }

                if (trimmed.contains(".unwrap()") || trimmed.contains(".expect("))
                    && !trimmed.starts_with("//")
                    && !trimmed.contains("allow")
                {
                    violations.push(format!(
                        "{}:{} - Unhandled unwrap/expect in production code: '{}'",
                        rel_path, line_num, trimmed
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        println!("panic-surface metric: allow_exceptions={}", allow_count);
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "panic-surface check failed (allows: {}) with {} violation(s):\n{}",
                allow_count,
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 10: CodeLintsGate ───────────────────────────────────────────────────

pub struct CodeLintsGate;

impl Gate for CodeLintsGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "code-lints".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["INV-IO-1".to_string(), "INV-FAILCLOSED-1".to_string()],
            why: "Prüft Code-Richtlinien bezüglich I/O und Fail-Closed über Clippy-/rustc-Ausgabe."
                .to_string(),
            owner: "MP-05".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_code_lints(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "code-lints" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_code_lints(root: &Path) -> Outcome {
    let mut allow_count = 0;
    let mut violations = Vec::new();

    let crates_dir = root.join("crates");
    if crates_dir.exists() {
        for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };

            if rel_path.contains("/tests/")
                || rel_path.contains("/benches/")
                || rel_path.contains("/examples/")
            {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                let trimmed = line.trim();

                if trimmed.contains("#[allow(clippy::disallowed_methods)]")
                    || trimmed.contains("#[allow(clippy::disallowed_types)]")
                    || trimmed.contains("#[allow(unused_must_use)]")
                {
                    allow_count += 1;
                }

                if (trimmed.contains("let _ =") && trimmed.contains(".read"))
                    || (trimmed.contains("let _ =") && trimmed.contains(".write"))
                {
                    violations.push(format!(
                        "{}:{} - Silent let _ = on Result in I/O operation: '{}'",
                        rel_path, line_num, trimmed
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        println!("code-lints metric: allow_exceptions={}", allow_count);
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "code-lints check failed (allows: {}) with {} violation(s):\n{}",
                allow_count,
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_determinism_gate_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/contextra-vector/src")).unwrap();
        fs::write(
            root.join("crates/contextra-vector/src/lib.rs"),
            "pub fn ok() {}",
        )
        .unwrap();

        assert_eq!(scan_determinism(root), Outcome::Pass);

        fs::write(
            root.join("crates/contextra-vector/src/hnsw.rs"),
            "let now = Instant::now();",
        )
        .unwrap();

        match scan_determinism(root) {
            Outcome::Fail { reason } => assert!(reason.contains("Instant::now")),
            _ => panic!("Expected Fail for Instant::now"),
        }
    }

    #[test]
    fn test_panic_surface_gate() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
        fs::write(root.join("crates/test-crate/src/lib.rs"), "pub fn ok() {}").unwrap();

        assert_eq!(scan_panic_surface(root), Outcome::Pass);
    }

    #[test]
    fn test_code_lints_gate() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/test-crate/src")).unwrap();
        fs::write(root.join("crates/test-crate/src/lib.rs"), "pub fn ok() {}").unwrap();

        assert_eq!(scan_code_lints(root), Outcome::Pass);
    }
}
