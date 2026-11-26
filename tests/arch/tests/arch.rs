//! Führt die Architekturprüfungen gegen das echte Workspace aus. Ohne diese Datei würden die
//! Prüfungen in keiner Pipeline laufen (Spezifikation N1).

use contextra_arch_tests::{check_layering, check_unsafe, load_crates};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn ring_regeln_gelten() -> Result<(), String> {
    let crates = load_crates(&workspace_root())?;
    let violations = check_layering(&crates);
    assert!(violations.is_empty(), "Ring-Verstöße:\n{}", violations.join("\n"));
    Ok(())
}

#[test]
fn unsafe_nur_in_inseln() -> Result<(), String> {
    let root = workspace_root();
    let crates = load_crates(&root)?;
    let violations = check_unsafe(&root, &crates);
    assert!(violations.is_empty(), "Unsafe-Verstöße:\n{}", violations.join("\n"));
    Ok(())
}
