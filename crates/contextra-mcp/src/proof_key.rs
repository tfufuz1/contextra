//! Modul zur Handhabung und Bereitstellung von Proof-Schlüsseln über einen lokalen Port-Trait.
//!
//! Die eigentliche Implementierung der Lösch- und Beweisschlüsselverwaltung
//! gehört zur `contextra-erasure`-Domäne (Konsument: `ERASE-ADAPT-1`) und wurde
//! nach `docs/handoff/MCP-1/adapter_src/` exportiert.
//!
//! Dieser Port stellt sicher, dass der MCP-Server ohne Abhängigkeit zur
//! Löschdomäne lauffähig bleibt.

use crate::protocol::McpError;
use zeroize::Zeroizing;

/// Port-Trait für die Bereitstellung eines Lösch-/Beweisschlüssels.
pub trait ProofKeyProvider: Send + Sync {
    /// Liefert den konfigurierten Proof-Schlüssel als zeroizenden String.
    fn get_proof_key(&self) -> Result<Zeroizing<String>, McpError>;
}

/// Standard-Provider, der den Schlüssel aus Umgebungsvariablen liest.
#[derive(Debug, Clone, Copy, Default)]
pub struct EnvProofKeyProvider;

impl ProofKeyProvider for EnvProofKeyProvider {
    #[allow(deprecated)]
    fn get_proof_key(&self) -> Result<Zeroizing<String>, McpError> {
        proof_key_from_env()
    }
}

/// Liest den Proof-Schlüssel aus den Prozess-Umgebungsvariablen
/// (`CONTEXTRA_CANONICAL_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`).
#[deprecated(note = "Proof key handling moves to contextra-erasure per ERASE-ADAPT-1")]
pub fn proof_key_from_env() -> Result<Zeroizing<String>, McpError> {
    let res = crate::proof_key_env::resolve_proof_key_from_env();
    res.key
        .ok_or_else(|| McpError::invalid_params("deletion proof key not configured"))
}

/// Statische Hilfsfunktion für den Default-Provider.
pub fn default_proof_key_provider() -> impl ProofKeyProvider {
    EnvProofKeyProvider
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    struct MockProofKeyProvider(Option<String>);

    impl ProofKeyProvider for MockProofKeyProvider {
        fn get_proof_key(&self) -> Result<Zeroizing<String>, McpError> {
            self.0
                .as_ref()
                .map(|k| Zeroizing::new(k.clone()))
                .ok_or_else(|| McpError::invalid_params("mock key not configured"))
        }
    }

    #[test]
    fn test_mock_proof_key_provider() {
        let provider = MockProofKeyProvider(Some("test_secret_123".into()));
        let key = provider.get_proof_key().unwrap(); // #[cfg(test)]
        assert_eq!(key.as_str(), "test_secret_123");

        let empty_provider = MockProofKeyProvider(None);
        assert!(empty_provider.get_proof_key().is_err());
    }
}
