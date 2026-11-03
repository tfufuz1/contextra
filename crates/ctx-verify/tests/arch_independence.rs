//! Architecture independence tests per SPEC AR-4, §8.16, §8.17.
//!
//! AR-4: `lab` und `ctx-verify` hängen nicht an `store`, `engine`, `erasure`
//! und nicht an den Produktions-Decodern von `crypto`. Sie haben eigene
//! Implementierungen der Formate (Anhang F) und Primitive (unabhängiger Prüfer).

use std::fs;
use std::path::Path;

/// SPEC: AR-4, T-ARCH-4
#[test]
fn t_arch_4_no_core_production_dependencies() {
    let cargo_toml_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path).expect("failed to read Cargo.toml");

    let forbidden_crates = [
        "contextra-store",
        "contextra-engine",
        "contextra-erasure",
        "contextra-crypto",
        "contextra-text",
        "contextra-vector",
        "contextra-rank",
        "contextra-mvcc",
        "contextra-durable-fs",
        "contextra-sys",
        "contextra-simd",
        "contextra-ports",
        "contextra-types",
    ];

    // Check dependency sections
    for line in content.lines() {
        let trimmed = line.trim();
        for forbidden in &forbidden_crates {
            if trimmed.starts_with(forbidden) {
                panic!("Forbidden dependency found in Cargo.toml: {}", forbidden);
            }
        }
    }
}

/// SPEC: AR-4, T-ARCH-4
#[test]
fn t_arch_4_no_core_production_imports_in_source() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in fs::read_dir(src_dir).expect("failed to read src dir") {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let code = fs::read_to_string(&path).expect("failed to read source file");
            for forbidden in &[
                "contextra_store",
                "contextra_engine",
                "contextra_erasure",
                "contextra_crypto",
                "contextra_text",
                "contextra_vector",
                "contextra_rank",
                "contextra_mvcc",
                "contextra_ports",
                "contextra_types",
                "contextra_durable_fs",
            ] {
                assert!(
                    !code.contains(forbidden),
                    "ctx-verify source {:?} must not import '{}' (AR-4 violation)",
                    path,
                    forbidden
                );
            }
        }
    }
}
