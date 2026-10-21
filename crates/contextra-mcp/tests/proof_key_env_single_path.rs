//! Integration tests for MCP proof key resolution single path loading without HOME fallback.

use contextra_mcp::proof_key_env::{resolve_proof_key, resolve_proof_key_from_env};
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use tempfile::NamedTempFile;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_signer_loader_single_path_mcp() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");

    let secret = "K#8xP$mQ2!vL9@zW4&nR7*yF1^tU5(bC";
    let tmp = NamedTempFile::new().expect("Failed to create temp file");
    let path = tmp.path().to_path_buf();

    {
        let mut f = File::create(&path).expect("Failed to open file for writing");
        f.write_all(secret.as_bytes()).expect("Write failed");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .expect("Failed to set 0600 permissions");
    }

    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY_FILE", path.to_str().unwrap());

    let res = resolve_proof_key_from_env();
    assert!(res.key.is_some());
    assert_eq!(res.key.unwrap().as_str(), secret);

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
}

#[test]
fn test_mcp_pure_resolver() {
    let secret = "primary_deletion_proof_key_32bytes_valid!";
    let res = resolve_proof_key(Some(secret.to_string()), None);
    assert!(res.key.is_some());
    assert_eq!(res.key.unwrap().as_str(), secret);
}
