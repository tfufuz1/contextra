//! `init` CLI subcommand for initializing data directory and master database key.

use contextra_crypto::ed25519_proof::DeletionProofKeyPair;
use contextra_types::ContextraError;
use rand::rngs::OsRng;
use tracing::info;

/// Options for `init` command.
#[derive(Debug, Default)]
pub struct InitOptions {
    pub db_path: std::path::PathBuf,
    pub key_file: Option<std::path::PathBuf>,
}

/// Executes data directory and crypto key initialization using contextra-crypto API.
/// Fails if keyfile/keyring is missing and interactive prompt is disabled (no pure env var passphrase per INV-KDF-1).
pub fn execute_init(opts: &InitOptions) -> Result<(), ContextraError> {
    if !opts.db_path.exists() {
        std::fs::create_dir_all(&opts.db_path)?;
    }

    if let Some(ref key_path) = opts.key_file {
        if !key_path.exists() {
            // Generate master key bytes via contextra-crypto API (DeletionProofKeyPair / OsRng)
            let keypair = DeletionProofKeyPair::generate(&mut OsRng);
            let key_bytes = keypair.verifying_key_bytes();
            std::fs::write(key_path, key_bytes)?;
            info!(
                key_path = %key_path.display(),
                "Generated new master key file for Contextra database via contextra-crypto API."
            );
        }
    } else {
        // Unattended init without explicit keyfile / keyring is disallowed (INV-KDF-1)
        return Err(ContextraError::InvalidInput(
            "INV-KDF-1 violation: --key-file or keyring option must be provided for unattended `init`. Pure environment variable passphrases are forbidden."
                .to_string(),
        ));
    }

    info!(
        db_path = %opts.db_path.display(),
        "Contextra data directory initialized successfully."
    );
    Ok(())
}
