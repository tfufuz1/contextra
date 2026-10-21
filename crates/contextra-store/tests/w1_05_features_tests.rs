use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn extract_features_from_cargo_toml(toml_content: &str) -> HashSet<String> {
    let mut features = HashSet::new();
    let mut in_features = false;

    for line in toml_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[features]") {
            in_features = true;
            continue;
        } else if trimmed.starts_with('[') {
            in_features = false;
        }

        if in_features && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(key) = trimmed.split('=').next() {
                features.insert(key.trim().to_string());
            }
        }
    }

    features
}

fn scan_file_for_cfg_features(path: &Path, declared_features: &HashSet<String>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let mut rest = line;
        while let Some(idx) = rest.find("feature = \"") {
            let after = &rest[idx + 11..];
            if let Some(end_idx) = after.find('"') {
                let feat = &after[..end_idx];
                assert!(
                    declared_features.contains(feat),
                    "File {} references undeclared feature '{}' in cfg attribute",
                    path.display(),
                    feat
                );
                rest = &after[end_idx + 1..];
            } else {
                break;
            }
        }
    }
}

fn scan_dir_for_cfg_features(dir: &Path, declared_features: &HashSet<String>) {
    if !dir.exists() {
        return;
    }
    let entries = fs::read_dir(dir).expect("read_dir");
    for entry in entries {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_cfg_features(&path, declared_features);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            scan_file_for_cfg_features(&path, declared_features);
        }
    }
}

#[test]
fn store_features_all_defined() {
    let cargo_path = Path::new("Cargo.toml");
    let content = if cargo_path.exists() {
        fs::read_to_string(cargo_path).expect("read Cargo.toml")
    } else {
        fs::read_to_string("crates/contextra-store/Cargo.toml").expect("read store Cargo.toml")
    };

    let declared_features = extract_features_from_cargo_toml(&content);
    assert!(
        !declared_features.is_empty(),
        "Must extract declared features from store Cargo.toml"
    );

    let src_dir = if Path::new("src").exists() {
        Path::new("src").to_path_buf()
    } else {
        Path::new("crates/contextra-store/src").to_path_buf()
    };

    scan_dir_for_cfg_features(&src_dir, &declared_features);
}
