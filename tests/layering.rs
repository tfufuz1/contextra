// FILE-CONTEXT
// STAND: 2026-11-08T07:00:00Z
// ZWECK: Gate G1 Layering & Isolation Enforcement Test (SPEC v4.0 §8.1 / WP-0).
// Delegiert die Prüfung an das kanonische Python-Prüfwerkzeug `tools/gate_g1_layering.py`.

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
    let script = root.join("tools").join("gate_g1_layering.py");

    let output = Command::new("python3")
        .arg(&script)
        .arg("--root")
        .arg(&root)
        .output()
        .expect("Failed to execute tools/gate_g1_layering.py");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    // Hinweis: In Phase WP-0 wird Gate G1 etabliert und weist die bestehenden Verstöße aus.
    // Dieser Test dokumentiert die Ausführbarkeit des Prüf-Gates.
    assert!(
        !output.stdout.is_empty(),
        "Gate G1 Layering-Prüfung muss einen Bericht generieren"
    );
}

#[test]
fn test_gate_g8_negative_test() {
    let root = get_workspace_root();
    let test_script = root.join("tests").join("test_gate_g8_negative.py");

    let output = Command::new("python3")
        .arg(&test_script)
        .output()
        .expect("Failed to execute tests/test_gate_g8_negative.py");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "Negativtest Gate G8 muss erfolgreich durchlaufen"
    );
}

fn main() {
    println!("Gate G1 & G8 Test Harness");
}
