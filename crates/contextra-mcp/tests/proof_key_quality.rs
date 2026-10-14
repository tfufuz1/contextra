// FILE-CONTEXT
// STAND:       2026-10-13
// ZWECK:       Integrationstests für DeletionProof-Schlüsselqualität, Key-File-Validierung und Lecksicherheit (F3, Auftrag B12).
// INVARIANTEN: Schwache Schlüssel werden abgelehnt; Key-Files erfordern 0600 Rechte; Fehlertexte enthalten KEINE Schlüsselbytes.

use contextra_mcp::deletion_proof_key_from_env;
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_short_key_rejected() {
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");
    let secret = "too_short_key_123";
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", secret);

    let res = deletion_proof_key_from_env();
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("at least 32 bytes"));
    assert!(
        !err_msg.contains(secret),
        "Error message leaked secret key material!"
    );

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[test]
fn test_null_key_rejected() {
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");
    let secret = "00000000000000000000000000000000";
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", secret);

    let res = deletion_proof_key_from_env();
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("null key rejected")
            || err_msg.contains("known test/default key")
            || err_msg.contains("insufficient entropy")
    );
    assert!(
        !err_msg.contains(secret),
        "Error message leaked secret key material!"
    );

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[test]
fn test_repeating_pattern_key_rejected() {
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");

    // Single char repeating
    let secret1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", secret1);
    let res1 = deletion_proof_key_from_env();
    assert!(res1.is_err());
    let err1 = res1.unwrap_err().to_string();
    assert!(err1.contains("repeating pattern rejected") || err1.contains("insufficient entropy"));
    assert!(
        !err1.contains(secret1),
        "Error message leaked secret key material!"
    );

    // Chunk repeating
    let secret2 = "12341234123412341234123412341234";
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", secret2);
    let res2 = deletion_proof_key_from_env();
    assert!(res2.is_err());
    let err2 = res2.unwrap_err().to_string();
    assert!(err2.contains("repeating pattern rejected") || err2.contains("insufficient entropy"));
    assert!(
        !err2.contains(secret2),
        "Error message leaked secret key material!"
    );

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[test]
fn test_low_entropy_key_rejected() {
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");
    // 10 distinct characters repeated
    let secret = "a1b2c3d4e5a1b2c3d4e5a1b2c3d4e5a1";
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", secret);

    let res = deletion_proof_key_from_env();
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("insufficient entropy") || err_msg.contains("at least 32 bytes"));
    assert!(
        !err_msg.contains(secret),
        "Error message leaked secret key material!"
    );

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[test]
fn test_known_dummy_key_rejected() {
    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
    std::env::remove_var("CONTEXTRA_PROOF_KEY");
    let secret = "change_me_in_production_32bytes!";
    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", secret);

    let res = deletion_proof_key_from_env();
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("known test/default key value rejected"));
    assert!(
        !err_msg.contains(secret),
        "Error message leaked secret key material!"
    );

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
}

#[test]
fn test_key_file_with_permissive_permissions_rejected() {
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
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("Failed to set 0644 permissions");
    }

    std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY_FILE", path.to_str().unwrap());

    let res = deletion_proof_key_from_env();

    #[cfg(unix)]
    {
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("permissions too permissive"));
        assert!(
            !err_msg.contains(secret),
            "Error message leaked secret key material!"
        );
    }

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
}

#[test]
fn test_valid_key_file_with_0600_permissions_accepted() {
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

    let res = deletion_proof_key_from_env();
    assert!(
        res.is_ok(),
        "Expected valid key file to be accepted: {:?}",
        res.err()
    );
    assert_eq!(res.unwrap().as_str(), secret);

    std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
}
