// FILE-CONTEXT
// STAND: 2026-11-10T12:00:00Z
// ZWECK: Gate G1 Layering & Isolation Enforcement Test (SPEC v4.0 §8.1 / WP-0 / S-01).
// Delegiert die Prüfung an das kanonische Python-Prüfwerkzeug `tools/gate_layering.py`.

use std::path::PathBuf;
use std::process::Command;

fn get_workspace_root() -> PathBuf {
    match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

#[test]
fn test_gate_g1_layering_and_isolation() {
    let root = get_workspace_root();
    let script = root.join("tools").join("gate_layering.py");

    let output = Command::new("python3")
        .arg(&script)
        .arg("--root")
        .arg(&root)
        .arg("--crate")
        .arg("contextra-ids")
        .output()
        .expect("Failed to execute tools/gate_layering.py");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "Gate G-L1 Layering-Prüfung für autarken Kern contextra-ids muss grün sein"
    );
}

#[test]
fn test_gate_layering_negative_test() {
    let root = get_workspace_root();
    let test_script = root.join("tests").join("test_gate_layering_negative.py");

    let output = Command::new("python3")
        .arg(&test_script)
        .output()
        .expect("Failed to execute tests/test_gate_layering_negative.py");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "Negativtest Gate G-L1 muss erfolgreich durchlaufen (Exit 1 bei verbotener Kante beweisen)"
    );
}

fn main() {
    println!("Gate G-L1 Test Harness");
}
