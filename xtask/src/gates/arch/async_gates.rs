// Contextra — Architecture & Async Gates Implementation
// Gate 11: async-io (REQ-ASYNC-2, P26)

use crate::core::ctx::Context;
use crate::core::gate::{Gate, GateSpec, Outcome};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct AsyncIoGate;

impl Gate for AsyncIoGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "async-io".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["REQ-ASYNC-2".to_string(), "P26".to_string()],
            why: "Kein Blockieren im async-Kontext; keine Async-Runtime in Ring 0 (SPEC §17.3)."
                .to_string(),
            owner: "MP-03".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_async_io(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "async-io" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_async_io(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    // 1. Verify Ring 0 crates do not depend on tokio / async-std in non-dev dependencies
    let ring0_crates = [
        "contextra-types",
        "contextra-ports",
        "contextra-sys",
        "contextra-simd",
        "contextra-wire",
        "contextra-mvcc",
        "contextra-crypto",
        "contextra-vector",
        "contextra-text",
        "contextra-rank",
    ];

    for crate_name in &ring0_crates {
        let cargo_toml = root.join("crates").join(crate_name).join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        let content = match fs::read_to_string(&cargo_toml) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut in_dev_deps = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[dev-dependencies]") {
                in_dev_deps = true;
            } else if trimmed.starts_with('[') {
                in_dev_deps = false;
            }

            if !in_dev_deps
                && (trimmed.starts_with("tokio ")
                    || trimmed.starts_with("tokio=")
                    || trimmed.starts_with("async-std"))
            {
                violations.push(format!(
                    "crates/{}/Cargo.toml - Ring 0 crate depends on async runtime: '{}'",
                    crate_name, trimmed
                ));
            }
        }
    }

    // 2. Scan for blocking I/O calls directly inside async fn
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

            let mut in_async_fn = false;
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                let trimmed = line.trim();

                if trimmed.contains("async fn ") || trimmed.contains("pub async fn ") {
                    in_async_fn = true;
                } else if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                    in_async_fn = false;
                }

                if in_async_fn
                    && (trimmed.contains("std::fs::read")
                        || trimmed.contains("std::fs::write")
                        || trimmed.contains("std::thread::sleep"))
                {
                    violations.push(format!(
                        "{}:{} - Blocking I/O inside async fn: '{}'",
                        rel_path, line_num, trimmed
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "async-io check failed with {} violation(s):\n{}",
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
    fn test_async_io_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/contextra-types/src")).unwrap();
        fs::write(
            root.join("crates/contextra-types/src/lib.rs"),
            "pub fn ok() {}",
        )
        .unwrap();
        fs::write(
            root.join("crates/contextra-types/Cargo.toml"),
            "[package]\nname = \"contextra-types\"\n",
        )
        .unwrap();

        assert_eq!(scan_async_io(root), Outcome::Pass);

        fs::write(
            root.join("crates/contextra-types/src/async_bad.rs"),
            "pub async fn bad() { std::fs::read(\"file\"); }",
        )
        .unwrap();

        match scan_async_io(root) {
            Outcome::Fail { reason } => assert!(reason.contains("Blocking I/O inside async fn")),
            _ => panic!("Expected Fail for blocking I/O in async fn"),
        }
    }
}
