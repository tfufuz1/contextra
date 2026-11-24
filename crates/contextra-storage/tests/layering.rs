use std::fs;
use std::path::Path;

fn scan_rust(root: &Path, violations: &mut Vec<String>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") { continue; }
            scan_rust(&path, violations)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path)?;
            for forbidden in ["contextra_store", "contextra_ports", "std::fs::", "std::time::SystemTime", "std::time::Instant"] {
                if source.contains(forbidden) {
                    violations.push(format!("{} imports forbidden layer `{forbidden}`", path.display()));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn core_has_no_legacy_or_adapter_dependencies() -> std::io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    scan_rust(&root, &mut violations)?;
    assert!(violations.is_empty(), "layering violations: {violations:#?}");
    Ok(())
}
