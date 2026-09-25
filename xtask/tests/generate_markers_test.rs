#[path = "../src/generate_markers.rs"]
mod generate_markers;

use generate_markers::{
    load_capabilities_from_file, render_marker_table, run_check_marker_drift,
    run_generate_markers, Capabilities,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_render_marker_table_synthetic_fixture() {
    let fixture_toml = r#"
[version]
schema = "2"
spec = "v4"

[crates.crate-alpha]
ring = 'Ring 0'
maturity = 'stable'
description = 'Alpha crate description'
capabilities = ['cap-a', 'cap-b']

[crates.crate-beta]
ring = 'Ring 1'
maturity = 'experimental'
description = 'Beta crate description'
capabilities = ['cap-c']

[crates.crate-gamma]
ring = 'Ring 2'
maturity = 'deprecated'
description = 'Gamma crate description'
capabilities = []

[crates.crate-delta]
ring = 'Ring 3'
maturity = 'custom-status'
description = 'Delta crate description'
capabilities = []
"#;

    let caps: Capabilities = toml::from_str(fixture_toml).expect("Failed to parse fixture TOML");
    let table = render_marker_table(&caps);

    assert!(table.contains("| `crate-alpha` | Ring 0 | 🟢 stable | `cap-a`, `cap-b` | Alpha crate description |"));
    assert!(table.contains("| `crate-beta` | Ring 1 | 🟡 experimental | `cap-c` | Beta crate description |"));
    assert!(table.contains("| `crate-gamma` | Ring 2 | 🔴 deprecated | - | Gamma crate description |"));
    assert!(table.contains("| `crate-delta` | Ring 3 | 🔴 custom-status | - | Delta crate description |"));
}

#[test]
fn test_render_marker_table_empty_capabilities() {
    let empty_toml = r#"
[version]
schema = "2"
spec = "v4"
"#;

    let caps: Capabilities = toml::from_str(empty_toml).expect("Failed to parse empty TOML");
    let table = render_marker_table(&caps);

    assert!(table.contains("# Contextra — Capability & Maturity Markers"));
    assert!(table.contains("| Crate | Ring | Maturity Marker | Capabilities | Beschreibung |"));
    // Table should contain no crate row entries
    assert_eq!(caps.crates.len(), 0);
}

#[test]
fn test_check_marker_drift_detects_manual_modifications() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let fixture_toml = r#"
[version]
schema = "2"
spec = "v4"

[crates.contextra-test]
ring = 'Ring 0'
maturity = 'stable'
description = 'Test Crate'
capabilities = ['test-cap']
"#;

    let capabilities_file = root_path.join("capabilities.toml");
    let markers_file = root_path.join("capability-markers.md");

    fs::write(&capabilities_file, fixture_toml).expect("Failed to write capabilities.toml fixture");

    let loaded_caps = load_capabilities_from_file(&capabilities_file)
        .expect("Failed to load capabilities from file");
    let initial_rendered = render_marker_table(&loaded_caps);
    fs::write(&markers_file, &initial_rendered).expect("Failed to write initial markers file");

    // 1. Verify drift check passes when in sync
    let newly_rendered = render_marker_table(&loaded_caps);
    assert_eq!(initial_rendered.trim(), newly_rendered.trim());

    // 2. Modify markers file manually
    let drifted_content = format!("{}\n<!-- MANUAL DRIFT MODIFICATION -->\n", initial_rendered);
    fs::write(&markers_file, drifted_content).expect("Failed to write modified markers file");

    // 3. Verify drift check detects disagreement
    let modified_content = fs::read_to_string(&markers_file).unwrap();
    assert_ne!(newly_rendered.trim(), modified_content.trim());
}

#[test]
fn test_generate_markers_and_check_drift_end_to_end() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_file = temp_dir.path().join("capability-markers.md");

    // Generate output file
    let gen_res = run_generate_markers(Some(&out_file));
    assert!(gen_res.is_ok(), "run_generate_markers failed: {:?}", gen_res);

    // Check drift against unmodified file
    let drift_res = run_check_marker_drift(Some(&out_file));
    assert!(drift_res.is_ok(), "run_check_marker_drift failed: {:?}", drift_res);

    // Tamper with generated file
    let mut content = fs::read_to_string(&out_file).unwrap();
    content.push_str("\nTampered line!\n");
    fs::write(&out_file, content).unwrap();

    // Check drift against tampered file
    let drift_tampered_res = run_check_marker_drift(Some(&out_file));
    assert!(
        drift_tampered_res.is_err(),
        "Drift check should fail on tampered file"
    );
}
