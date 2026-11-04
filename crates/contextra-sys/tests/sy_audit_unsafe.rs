#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Audit-Tests fuer Unsafe-Integritaet in contextra-sys (SY-01, SY-02, T-UNSAFE-1).

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if manifest_dir.ends_with("crates/contextra-sys") {
        manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(&manifest_dir)
            .to_path_buf()
    } else {
        manifest_dir
    }
}

/// SPEC: T-UNSAFE-1
/// T-UNSAFE-1 & SY-01: Prüft, dass unsafe ausschließlich in sys und simd vorkommt
/// und alle anderen Crates #![forbid(unsafe_code)] erzwingen.
#[test]
fn sy_audit_01_t_unsafe_1_workspace_scan() {
    let root = workspace_root();
    let crates_dir = root.join("crates");

    if !crates_dir.exists() {
        return;
    }

    let entries = fs::read_dir(&crates_dir).expect("read crates dir");
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let crate_name = entry.file_name().to_string_lossy().to_string();
        let lib_rs = entry.path().join("src").join("lib.rs");
        if !lib_rs.exists() {
            continue;
        }

        let content = fs::read_to_string(&lib_rs).expect("read lib.rs");

        if crate_name == "contextra-sys" || crate_name == "contextra-simd" {
            assert!(
                content.contains("#![allow(unsafe_code)]")
                    || content.contains("#![deny(unsafe_op_in_unsafe_fn)]"),
                "Unsafe-Insel {crate_name} muss unsafe_op_in_unsafe_fn oder allow(unsafe_code) definieren"
            );
        } else {
            assert!(
                content.contains("#![forbid(unsafe_code)]"),
                "Crate {crate_name} muss #![forbid(unsafe_code)] tragen!"
            );
        }
    }
}

/// SPEC: T-UNSAFE-1
/// SY-01: Jeder unsafe-Block in contextra-sys trägt einen // SAFETY:-Kommentar.
#[test]
fn sy_audit_01_each_unsafe_block_has_safety_comment() {
    let root = workspace_root();
    let src_dir = root.join("crates/contextra-sys/src");
    let entries = fs::read_dir(&src_dir).expect("read sys/src");

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("read src file");
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("unsafe {") || trimmed == "unsafe" {
                let start = idx.saturating_sub(5);
                let window = &lines[start..=idx];
                let has_safety = window.iter().any(|l| l.contains("// SAFETY:"));
                assert!(
                    has_safety,
                    "Unsafe-Block in {}:{} besitzt keinen // SAFETY:-Kommentar:\n{}",
                    path.display(),
                    idx + 1,
                    line
                );
            }
        }
    }
}

/// SPEC: T-UNSAFE-1
/// SY-01: Jede pub unsafe fn in contextra-sys hat einen '# Safety'-Abschnitt in der Dokumentation.
#[test]
fn sy_audit_01_pub_unsafe_fn_has_safety_doc() {
    let root = workspace_root();
    let src_dir = root.join("crates/contextra-sys/src");
    let entries = fs::read_dir(&src_dir).expect("read sys/src");

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("read src file");
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub unsafe fn") {
                let start = idx.saturating_sub(15);
                let doc_window = &lines[start..idx];
                let has_safety_doc = doc_window.iter().any(|l| l.contains("# Safety"));
                assert!(
                    has_safety_doc,
                    "pub unsafe fn in {}:{} hat keinen '# Safety'-Abschnitt:\n{}",
                    path.display(),
                    idx + 1,
                    line
                );
            }
        }
    }
}

/// SPEC: T-UNSAFE-1
/// SY-02: Kein Crate mit forbid(unsafe_code) ruft eine unsafe fn aus contextra-sys auf.
#[test]
fn sy_audit_02_no_forbid_crate_calls_unsafe_fn_from_sys() {
    let root = workspace_root();
    let sys_lib =
        fs::read_to_string(root.join("crates/contextra-sys/src/lib.rs")).expect("read lib.rs");

    for line in sys_lib.lines() {
        let trimmed = line.trim();
        assert!(
            !trimmed.starts_with("pub unsafe fn"),
            "contextra-sys exportiert pub unsafe fn: {trimmed}"
        );
    }
}

/// SPEC: T-UNSAFE-1
/// Drift-Prüfung Quelltext vs. docs/unsafe-register.md
#[test]
fn sy_audit_01_unsafe_register_drift_check() {
    let root = workspace_root();
    let register_path = root.join("docs/unsafe-register.md");
    if !register_path.exists() {
        return;
    }

    let register_content = fs::read_to_string(&register_path).expect("read register");

    let src_dir = root.join("crates/contextra-sys/src");
    let mut actual_unsafe_count = 0;
    for entry in fs::read_dir(&src_dir).expect("read src").flatten() {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            let code = fs::read_to_string(entry.path()).expect("read rs");
            for line in code.lines() {
                let trimmed = line.trim();
                if trimmed.contains("unsafe {") || trimmed == "unsafe" {
                    actual_unsafe_count += 1;
                }
            }
        }
    }

    assert!(
        actual_unsafe_count <= 57,
        "Anzahl unsafe-Blöcke ({actual_unsafe_count}) darf Basislimit (57) nicht überschreiten"
    );
    assert!(
        register_content.contains(&format!("**{actual_unsafe_count}**")),
        "Register-Zählung stimmt nicht mit tatsächlicher Anzahl {actual_unsafe_count} überein"
    );
}
