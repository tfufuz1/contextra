//! G-03 (SPEC §11.6, §13.6 SYNC-2): Schichtregeln aus Anhang E.
//!
//! Der Test ruft `tools/check_spec_sync.py layering --ratchet` auf. Bekannte Befunde
//! stehen in `layering_baseline.txt`; ein neuer Befund lässt den Test fehlschlagen.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn layering_ratchet_has_no_new_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = root.join("tools/check_spec_sync.py");
    let out = Command::new("python3")
        .arg(&script)
        .arg("layering")
        .arg("--ratchet")
        .current_dir(&root)
        .output()
        .expect("python3 konnte nicht gestartet werden");
    assert!(
        out.status.success(),
        "neue Schichtbefunde gegenüber layering_baseline.txt:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
