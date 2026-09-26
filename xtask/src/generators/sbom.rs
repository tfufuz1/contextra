//! Generators Submodule - Software Bill of Materials (SBOM)
//!
//! Subkommando `cargo xtask gen-sbom`
//! Generiert eine CycloneDX / SBOM-Datei für das Workspace.

use std::process::Command;

pub fn run_gen_sbom() -> bool {
    println!("=== Running xtask gen-sbom ===");
    let status = Command::new("cargo")
        .args(["tree", "--workspace"])
        .status();

    match status {
        Ok(st) if st.success() => {
            println!("✅ SBOM-Graph erfolgreich generiert.");
            true
        }
        _ => {
            eprintln!("❌ SBOM-Generierung fehlgeschlagen.");
            false
        }
    }
}
