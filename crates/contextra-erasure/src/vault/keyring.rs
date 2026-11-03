//! FileKeyring-Implementierung (SPEC §11.5, D-85, I-9b, I-17).

use crate::error::ErasureError;

/// Stub-Implementierung für WP-4.1.
pub struct VaultKeyring;

impl VaultKeyring {
    /// Erstellt eine neue Keyring-Instanz.
    pub fn new() -> Self {
        Self
    }
}

impl Default for VaultKeyring {
    fn default() -> Self {
        Self::new()
    }
}
