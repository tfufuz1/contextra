#![cfg(not(loom))]

use std::fs;
use std::path::{Path, PathBuf};

fn check_content(relative_path: &str, content: &str, violations: &mut Vec<String>) {
    if content.contains("no_crypto_stubs") {
        violations.push(format!(
            "{relative_path}: contains forbidden stub module name 'no_crypto_stubs'"
        ));
    }

    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub fn verify")
            || trimmed.starts_with("fn verify")
            || trimmed.starts_with("pub(crate) fn verify")
        {
            let snippet: String = lines
                .iter()
                .skip(i)
                .take(15)
                .copied()
                .collect::<Vec<&str>>()
                .join("\n");

            if snippet.contains("Ok(true)") {
                violations.push(format!(
                    "{relative_path}:{}: verify function contains unverified Ok(true) return snippet:\n{}",
                    i + 1,
                    snippet
                ));
            }
        }
    }
}

fn visit_dirs(
    dir: &Path,
    manifest_dir: &Path,
    violations: &mut Vec<String>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, manifest_dir, violations)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = fs::read_to_string(&path)?;
                let relative_path = path
                    .strip_prefix(manifest_dir)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                check_content(&relative_path, &content, violations);
            }
        }
    }
    Ok(())
}

#[test]
fn test_no_crypto_stubs_or_fake_verify_fallback_guard() -> Result<(), String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let src_dir = manifest_dir.join("src");

    let mut violations = Vec::new();
    visit_dirs(&src_dir, &manifest_dir, &mut violations).map_err(|e| e.to_string())?;

    if !violations.is_empty() {
        return Err(format!(
            "Fallback guard detected stub or unverified proof implementation:\n{}",
            violations.join("\n")
        ));
    }

    Ok(())
}

#[test]
fn test_guard_detects_mutation_fake_verify() {
    let fake_code = r#"
        pub fn verify(&self) -> Result<bool> {
            Ok(true)
        }
    "#;
    let mut violations = Vec::new();
    check_content("src/deletion/fake.rs", fake_code, &mut violations);
    assert!(
        !violations.is_empty(),
        "Guard MUST detect unverified Ok(true) in fn verify"
    );
    assert!(violations[0].contains("verify function contains unverified Ok(true)"));
}

#[test]
fn test_guard_detects_mutation_no_crypto_stubs() {
    let stub_code = r#"
        mod no_crypto_stubs {
            // stub
        }
    "#;
    let mut violations = Vec::new();
    check_content("src/lib.rs", stub_code, &mut violations);
    assert!(
        !violations.is_empty(),
        "Guard MUST detect no_crypto_stubs module name"
    );
    assert!(violations[0].contains("contains forbidden stub module name 'no_crypto_stubs'"));
}
