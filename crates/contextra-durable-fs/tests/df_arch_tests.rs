//! Architecture and hygiene tests for `contextra-durable-fs` (T-ARCH-4, T-LINT-2, DF-01, DF-04).

use contextra_ports::vfs::Vfs;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn Error>>;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_t_arch_4_no_network_no_tokio_fs_no_block_on() -> TestResult {
    let root = crate_root();
    let src_dir = root.join("src");
    let mut violations = Vec::new();

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    for file in files {
        let content = fs::read_to_string(&file)?;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains("tokio::fs") {
                violations.push(format!(
                    "{}:{}: forbidden tokio::fs usage: {}",
                    file.display(),
                    idx + 1,
                    trimmed
                ));
            }
            if trimmed.contains("block_on") {
                violations.push(format!(
                    "{}:{}: forbidden block_on usage: {}",
                    file.display(),
                    idx + 1,
                    trimmed
                ));
            }
            if trimmed.contains("std::net")
                || trimmed.contains("reqwest")
                || trimmed.contains("hyper")
            {
                violations.push(format!(
                    "{}:{}: forbidden network usage: {}",
                    file.display(),
                    idx + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "T-ARCH-4 / DF-04 violations detected:\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[test]
fn test_t_lint_2_single_vfs_trait_no_second_vfs() -> TestResult {
    let root = crate_root();
    let src_dir = root.join("src");
    let mut violations = Vec::new();

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    for file in files {
        let content = fs::read_to_string(&file)?;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // Searching for trait definition: `trait Vfs ` or `pub trait Vfs `
            if (trimmed.starts_with("pub trait Vfs") || trimmed.starts_with("trait Vfs"))
                && !trimmed.contains("contextra_ports")
            {
                violations.push(format!(
                    "{}:{}: duplicate trait Vfs defined: {}",
                    file.display(),
                    idx + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "T-LINT-2 / DF-01 violations (second Vfs trait defined):\n{}",
        violations.join("\n")
    );
    Ok(())
}

#[test]
fn test_df_01_real_vfs_implements_ports_vfs() {
    let vfs = contextra_durable_fs::real_vfs::RealVfs::new();
    fn assert_implements_vfs<T: Vfs + 'static>(_val: &T) {}
    assert_implements_vfs(&vfs);
}

#[test]
fn test_clean_removal_no_realfs_no_dead_modules() -> TestResult {
    let root = crate_root();
    let lib_rs = root.join("src/lib.rs");
    let content = fs::read_to_string(&lib_rs)?;

    let mut violations = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if trimmed.contains("RealFs") {
            violations.push(format!("lib.rs:{}: legacy alias RealFs found", idx + 1));
        }
        if trimmed.contains("mod health;") || trimmed.contains("pub mod health") {
            violations.push(format!("lib.rs:{}: dead module health found", idx + 1));
        }
        if trimmed.contains("mod fault_injector;") || trimmed.contains("pub mod fault_injector") {
            violations.push(format!(
                "lib.rs:{}: dead module fault_injector found",
                idx + 1
            ));
        }
        if trimmed.contains("mod journal;") || trimmed.contains("pub mod journal") {
            violations.push(format!("lib.rs:{}: dead module journal found", idx + 1));
        }
    }

    assert!(
        violations.is_empty(),
        "Legacy artifacts must be completely removed from lib.rs:\n{}",
        violations.join("\n")
    );
    Ok(())
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}
