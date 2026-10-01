use std::path::{Path, PathBuf};

fn find_router_cargo_toml() -> PathBuf {
    // Attempt 1: Relative to CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = Path::new(&manifest_dir).join("../crates/contextra-router/Cargo.toml");
        if path.exists() {
            return path;
        }
        let direct_path = Path::new(&manifest_dir).join("crates/contextra-router/Cargo.toml");
        if direct_path.exists() {
            return direct_path;
        }
    }

    // Attempt 2: Search upwards from current directory
    let mut curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        let candidate = curr.join("crates/contextra-router/Cargo.toml");
        if candidate.exists() {
            return candidate;
        }
        if !curr.pop() {
            break;
        }
    }

    panic!("Could not find crates/contextra-router/Cargo.toml from current location");
}

#[test]
fn test_router_ring0_and_db_deps_are_dev_only() {
    let manifest_path = find_router_cargo_toml();
    let content = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", manifest_path.display(), e));

    let parsed: toml::Table = toml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse TOML in {}: {}", manifest_path.display(), e));

    let forbidden_deps = ["contextra-db", "contextra-vector", "contextra-graph", "contextra-text"];

    if let Some(deps_value) = parsed.get("dependencies") {
        if let Some(deps) = deps_value.as_table() {
            for forbidden in &forbidden_deps {
                assert!(
                    !deps.contains_key(*forbidden),
                    "Forbidden dependency '{}' found in [dependencies] section of {}. Ring 0 crates and contextra-db must remain dev-dependencies only in contextra-router.",
                    forbidden,
                    manifest_path.display()
                );
            }
        }
    }
}
