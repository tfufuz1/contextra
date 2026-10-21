// FILE-CONTEXT
// ZWECK: Dynamische Selbsttest-Prüfung aller in Fassaden-Code verwendeten cfg(feature = ...)-Marker.
// INVARIANTEN: Jedes im Code der Fassade (contextra) abgefragte Cargo-Feature MUSS in crates/contextra/Cargo.toml [features] deklariert sein.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[test]
fn test_facade_features_declared_in_cargo_toml() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path)
        .expect("Cargo.toml von contextra muss lesbar sein");

    let parsed: toml::Value = toml::from_str(&content)
        .expect("Cargo.toml von contextra muss gültiges TOML sein");

    let declared_features: HashSet<String> = parsed
        .get("features")
        .and_then(|f| f.as_table())
        .expect("[features] Tabelle muss in Cargo.toml existieren")
        .keys()
        .cloned()
        .collect();

    let src_dir = manifest_dir.join("src");
    let mut scanned_features = HashSet::new();

    // Scanne dynamisch alle .rs Quelldateien in crates/contextra/src/
    for entry in walkdir(src_dir.as_path()) {
        if entry.extension().map_or(false, |ext| ext == "rs") {
            let src = fs::read_to_string(&entry)
                .unwrap_or_else(|_| panic!("Konnte Quelldatei {:?} nicht lesen", entry));

            // Re-Muster zum Extrahieren von feature = "..." und cfg!(feature = "...")
            for cap in re_find_features(&src) {
                scanned_features.insert(cap);
            }
        }
    }

    assert!(
        !scanned_features.is_empty(),
        "Es wurden keine feature = \"...\" Marker im Quellcode gefunden."
    );

    for feat in &scanned_features {
        assert!(
            declared_features.contains(feat),
            "Feature '{feat}' wird im Fassaden-Code (src/) verwendet, ist aber in crates/contextra/Cargo.toml [features] nicht deklariert!"
        );
    }
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

fn re_find_features(code: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut rest = code;
    while let Some(idx) = rest.find("feature") {
        rest = &rest[idx + "feature".len()..];
        if rest.starts_with('=') || rest.trim_start().starts_with('=') {
            if let Some(first_quote) = rest.find('"') {
                let after_quote = &rest[first_quote + 1..];
                if let Some(second_quote) = after_quote.find('"') {
                    let feat = &after_quote[..second_quote];
                    results.push(feat.to_string());
                    rest = &after_quote[second_quote + 1..];
                }
            }
        }
    }
    results
}
