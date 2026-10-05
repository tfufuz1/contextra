//! Modul zur Handhabung und Validierung von DeletionProof-Schlüsseln.
//!
//! # Vorrangregeln
//! - `CONTEXTRA_DELETION_PROOF_KEY` ist der kanonische Schlüssel.
//! - `CONTEXTRA_PROOF_KEY` ist ein veralteter (deprecated) Alias.
//! - Sind beide Variablen gesetzt und enthalten unterschiedliche Werte, gewinnt `CONTEXTRA_DELETION_PROOF_KEY`
//!   und ein Konflikt-Warning wird ausgegeben.

// FILE-CONTEXT
// STAND:       2026-09-30
// ZWECK:       Lösen und Validieren des DeletionProof Schlüssel-Materials aus Umgebungsvariablen.
// INVARIANTEN: CONTEXTRA_DELETION_PROOF_KEY ist kanonisch; CONTEXTRA_PROOF_KEY ist ein deprecated Alias.
//              Schlüsselwerte werden NIEMALS geloggt. Zero-Panic im Produktionspfad.

use crate::protocol::McpError;
use zeroize::Zeroizing;

/// Ergebnis des reinen Schlüssel-Resolvers.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ResolvedProofKey {
    /// Das aufgelöste Schlüsselmaterial als zeroizendes String-Wrapper.
    pub key: Zeroizing<String>,
    /// Gibt an, ob der deprecated Alias `CONTEXTRA_PROOF_KEY` verwendet wurde.
    pub used_deprecated_alias: bool,
    /// Gibt an, ob sowohl der primäre Schlüssel als auch der Alias mit unterschiedlichen Werten gesetzt waren.
    pub has_conflict: bool,
}

/// Fehler beim Auflösen des DeletionProof-Schlüssels.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ProofKeyError {
    #[error("deletion proof key not configured")]
    MissingKey,
}

/// Reine Hilfsfunktion zur Auflösung des DeletionProof-Schlüssels ohne Zugriff auf die Prozess-Umgebung.
///
/// # Vorrangregeln
/// - `primary` (`CONTEXTRA_DELETION_PROOF_KEY`) ist der kanonische Schlüssel.
/// - `alias` (`CONTEXTRA_PROOF_KEY`) ist ein veralteter (deprecated) Alias.
/// - Wenn beide gesetzt und unterschiedlich sind, gewinnt `primary`, und `has_conflict` wird auf `true` gesetzt.
/// - Leere Strings oder Strings, die nur Whitespace enthalten, zählen als nicht gesetzt.
#[allow(dead_code)]
pub(crate) fn resolve_deletion_proof_key(
    primary: Option<&str>,
    alias: Option<&str>,
) -> Result<ResolvedProofKey, ProofKeyError> {
    let clean_primary = primary.map(|s| s.trim()).filter(|s| !s.is_empty());
    let clean_alias = alias.map(|s| s.trim()).filter(|s| !s.is_empty());

    match (clean_primary, clean_alias) {
        (Some(p), Some(a)) => {
            if p == a {
                Ok(ResolvedProofKey {
                    key: Zeroizing::new(p.to_string()),
                    used_deprecated_alias: false,
                    has_conflict: false,
                })
            } else {
                Ok(ResolvedProofKey {
                    key: Zeroizing::new(p.to_string()),
                    used_deprecated_alias: false,
                    has_conflict: true,
                })
            }
        }
        (Some(p), None) => Ok(ResolvedProofKey {
            key: Zeroizing::new(p.to_string()),
            used_deprecated_alias: false,
            has_conflict: false,
        }),
        (None, Some(a)) => Ok(ResolvedProofKey {
            key: Zeroizing::new(a.to_string()),
            used_deprecated_alias: true,
            has_conflict: false,
        }),
        (None, None) => Err(ProofKeyError::MissingKey),
    }
}

// TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
/// Liest den DeletionProof-Schlüssel aus den Prozess-Umgebungsvariablen
/// (`CONTEXTRA_DELETION_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`), ruft die reine Auflösungsfunktion auf
/// und gibt bei Warnzuständen einmalige `tracing::warn!`-Meldungen aus.
pub fn deletion_proof_key_from_env() -> Result<Zeroizing<String>, McpError> {
    let res = crate::proof_key_env::resolve_proof_key_from_env();
    res.key
        .ok_or_else(|| McpError::invalid_params("deletion proof key not configured"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_only_primary_set() {
        let res = resolve_deletion_proof_key(Some("secret_key_123"), None).unwrap();
        assert_eq!(res.key.as_str(), "secret_key_123");
        assert!(!res.used_deprecated_alias);
        assert!(!res.has_conflict);
    }

    #[test]
    fn test_resolve_only_alias_set() {
        let res = resolve_deletion_proof_key(None, Some("deprecated_alias_key")).unwrap();
        assert_eq!(res.key.as_str(), "deprecated_alias_key");
        assert!(res.used_deprecated_alias);
        assert!(!res.has_conflict);
    }

    #[test]
    fn test_resolve_both_same() {
        let res = resolve_deletion_proof_key(Some("identical_key"), Some("identical_key")).unwrap();
        assert_eq!(res.key.as_str(), "identical_key");
        assert!(!res.used_deprecated_alias);
        assert!(!res.has_conflict);
    }

    #[test]
    fn test_resolve_both_different_conflict() {
        let res =
            resolve_deletion_proof_key(Some("canonical_key"), Some("legacy_alias_key")).unwrap();
        assert_eq!(res.key.as_str(), "canonical_key");
        assert!(!res.used_deprecated_alias);
        assert!(res.has_conflict);
    }

    #[test]
    fn test_resolve_empty_or_whitespace_primary_falls_back_to_alias() {
        let res1 = resolve_deletion_proof_key(Some(""), Some("alias_key")).unwrap();
        assert_eq!(res1.key.as_str(), "alias_key");
        assert!(res1.used_deprecated_alias);
        assert!(!res1.has_conflict);

        let res2 = resolve_deletion_proof_key(Some("   \t\n "), Some("alias_key")).unwrap();
        assert_eq!(res2.key.as_str(), "alias_key");
        assert!(res2.used_deprecated_alias);
        assert!(!res2.has_conflict);
    }

    #[test]
    fn test_resolve_neither_set_returns_missing_error() {
        let res1 = resolve_deletion_proof_key(None, None);
        assert_eq!(res1, Err(ProofKeyError::MissingKey));

        let res2 = resolve_deletion_proof_key(Some("  "), Some("   "));
        assert_eq!(res2, Err(ProofKeyError::MissingKey));
    }

    #[test]
    fn test_deletion_proof_key_from_env() {
        // Without env var set
        std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
        std::env::remove_var("CONTEXTRA_PROOF_KEY");
        assert!(deletion_proof_key_from_env().is_err());

        // With env var set
        std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", "unit_test_proof_key_12345");
        let res = deletion_proof_key_from_env().unwrap();
        assert_eq!(res.as_str(), "unit_test_proof_key_12345");
        std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    }
}
