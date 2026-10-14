use std::fs;
use tempfile::tempdir;
use xtask::harness::deletion_guarantee_gate::run_deletion_guarantee_gate;

#[test]
fn test_clean_fixture_passes() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("crates/contextra-engine/src");
    fs::create_dir_all(&src_dir).unwrap();

    let clean_code = r#"
pub fn clean_function() -> Result<u64, String> {
    Ok(42)
}
"#;
    fs::write(src_dir.join("clean.rs"), clean_code).unwrap();

    let args = vec!["--root".to_string(), dir.path().to_string_lossy().to_string()];
    let exit_code = run_deletion_guarantee_gate(&args);
    assert_eq!(exit_code, 0, "Clean fixture must exit with 0");
}

#[test]
fn test_violations_fixture_fails() {
    let dir = tempdir().unwrap();

    // Violation (a): db_instance_id seed derivation
    let engine_dir = dir.path().join("crates/contextra-engine/src");
    fs::create_dir_all(&engine_dir).unwrap();
    let code_a = r#"
pub fn derive_seed_bad(db_instance_id: u64) {
    let Kdf = KdfManager::derive_seed(db_instance_id);
}
"#;
    fs::write(engine_dir.join("seed.rs"), code_a).unwrap();

    // Violation (b): issue_*witness* returning Ok(None)
    let crypto_dir = dir.path().join("crates/contextra-crypto/src");
    fs::create_dir_all(&crypto_dir).unwrap();
    let code_b = r#"
pub fn issue_residue_witness() -> Result<Option<u64>, String> {
    Ok(None)
}
"#;
    fs::write(crypto_dir.join("witness.rs"), code_b).unwrap();

    // Violation (c): unwrap_or_default in deletion/
    let deletion_dir = dir.path().join("crates/contextra-engine/src/deletion");
    fs::create_dir_all(&deletion_dir).unwrap();
    let code_c = r#"
pub fn delete_op(opt: Option<u64>) {
    let val = opt.unwrap_or_default();
}
"#;
    fs::write(deletion_dir.join("op.rs"), code_c).unwrap();

    // Violation (d): .verify( on Ed25519 in deletion_proof/
    let proof_dir = dir.path().join("crates/contextra-crypto/src/deletion_proof");
    fs::create_dir_all(&proof_dir).unwrap();
    let code_d = r#"
pub fn verify_proof(key: Ed25519Key) {
    key.verify(data);
}
"#;
    fs::write(proof_dir.join("verifier.rs"), code_d).unwrap();

    // Violation (e): let _ = ignoring revoke_group
    let code_e = r#"
pub fn revoke() {
    let _ = revoke_group("tenant_1");
}
"#;
    fs::write(engine_dir.join("revoke.rs"), code_e).unwrap();

    let args = vec!["--root".to_string(), dir.path().to_string_lossy().to_string(), "--json".to_string()];
    let exit_code = run_deletion_guarantee_gate(&args);
    assert_eq!(exit_code, 2, "Violating fixture must exit with 2 (fail-closed)");
}
