#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Audit-Tests fuer Crate-Hygiene, Rueckbau und Abhaengigkeiten (SY-03, SY-04, SY-11, T-ARCH-4).

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

/// SY-03 & SY-04: Grep im gesamten Crate contextra-sys nach verbotenen Begriffen
/// (windows, mmap, acl_win32, memmap2) ergibt 0 Treffer.
#[test]
fn sy_audit_03_04_grep_forbidden_terms_is_zero() {
    let root = workspace_root();
    let src_dir = root.join("crates/contextra-sys/src");

    let forbidden_patterns = ["windows", "mmap", "acl_win32", "memmap2"];

    for entry in fs::read_dir(&src_dir).expect("read src").flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("read file");
        for (line_no, line) in content.lines().enumerate() {
            let lower = line.to_lowercase();
            for pattern in &forbidden_patterns {
                if lower.contains(pattern) {
                    panic!(
                        "Verbotenes Muster '{pattern}' gefunden in {}:{}:\n{}",
                        path.display(),
                        line_no + 1,
                        line
                    );
                }
            }
        }
    }

    // Auch in Cargo.toml pruefen
    let cargo_toml =
        fs::read_to_string(root.join("crates/contextra-sys/Cargo.toml")).expect("read Cargo.toml");
    for pattern in &["memmap2", "windows-sys"] {
        assert!(
            !cargo_toml.contains(pattern),
            "Cargo.toml enthaelt verbotene Abhaengigkeit: {pattern}"
        );
    }
}

/// SY-11 & T-ARCH-4: Keine Netzwerk- oder Zeitzugriffe; Single-Node-Garantie.
#[test]
fn sy_audit_11_t_arch_4_no_network_or_time_dependencies() {
    let root = workspace_root();
    let cargo_toml =
        fs::read_to_string(root.join("crates/contextra-sys/Cargo.toml")).expect("read Cargo.toml");

    let network_crates = [
        "tokio-tungstenite",
        "reqwest",
        "hyper",
        "tungstenite",
        "tonic",
    ];
    for dep in &network_crates {
        assert!(
            !cargo_toml.contains(dep),
            "contextra-sys darf keine Netzwerkabhaengigkeit ({dep}) enthalten (SY-11/T-ARCH-4)"
        );
    }
}
