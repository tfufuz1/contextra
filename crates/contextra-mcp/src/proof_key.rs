//! Modul zur Handhabung und Validierung von DeletionProof-Schlüsseln.
//!
//! # Vorrangregeln
//! - `CONTEXTRA_DELETION_PROOF_KEY_FILE` / `CONTEXTRA_DELETION_PROOF_KEY` ist der kanonische Schlüssel.
//! - `CONTEXTRA_PROOF_KEY_FILE` / `CONTEXTRA_PROOF_KEY` ist ein veralteter (deprecated) Alias.
//! - Sind beide Variablen gesetzt und enthalten unterschiedliche Werte, gewinnt die primäre Quelle
//!   und ein Konflikt-Warning wird ausgegeben.

// FILE-CONTEXT
// STAND:       2026-10-13
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
    #[error("{0}")]
    InvalidKey(String),
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

    let (chosen_key, used_deprecated_alias, has_conflict) = match (clean_primary, clean_alias) {
        (Some(p), Some(a)) => {
            if p == a {
                (p, false, false)
            } else {
                (p, false, true)
            }
        }
        (Some(p), None) => (p, false, false),
        (None, Some(a)) => (a, true, false),
        (None, None) => return Err(ProofKeyError::MissingKey),
    };

    crate::proof_key_env::validate_proof_key_quality(chosen_key.as_bytes())
        .map_err(ProofKeyError::InvalidKey)?;

    Ok(ResolvedProofKey {
        key: Zeroizing::new(chosen_key.to_string()),
        used_deprecated_alias,
        has_conflict,
    })
}

/// Liest den DeletionProof-Schlüssel aus Schlüsseldateien oder Prozess-Umgebungsvariablen
/// (`CONTEXTRA_DELETION_PROOF_KEY_FILE`, `CONTEXTRA_PROOF_KEY_FILE`, `CONTEXTRA_DELETION_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`),
/// erzwingt die Mindestqualität (len >= 32, hohe Entropie, keine Muster)
/// und gibt bei Warnzuständen einmalige `tracing::warn!`-Meldungen aus.
pub fn deletion_proof_key_from_env() -> Result<Zeroizing<String>, McpError> {
    let res = crate::proof_key_env::resolve_proof_key_from_env();
    if let Some(key) = res.key {
        Ok(key)
    } else if let Some(err_msg) = res.error {
        Err(McpError::invalid_params(err_msg))
    } else {
        Err(McpError::invalid_params(
            "deletion proof key not configured",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_CANONICAL_KEY: &str = "canonical_proof_key_32bytes_valid!";
    const VALID_ALIAS_KEY: &str = "deprecated_alias_key_32bytes_valid!!";

    #[test]
    fn test_resolve_only_primary_set() {
        let res = resolve_deletion_proof_key(Some(VALID_CANONICAL_KEY), None);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.key.as_str(), VALID_CANONICAL_KEY);
            assert!(!res.used_deprecated_alias);
            assert!(!res.has_conflict);
        }
    }

    #[test]
    fn test_resolve_only_alias_set() {
        let res = resolve_deletion_proof_key(None, Some(VALID_ALIAS_KEY));
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.key.as_str(), VALID_ALIAS_KEY);
            assert!(res.used_deprecated_alias);
            assert!(!res.has_conflict);
        }
    }

    #[test]
    fn test_resolve_both_same() {
        let res = resolve_deletion_proof_key(Some(VALID_CANONICAL_KEY), Some(VALID_CANONICAL_KEY));
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.key.as_str(), VALID_CANONICAL_KEY);
            assert!(!res.used_deprecated_alias);
            assert!(!res.has_conflict);
        }
    }

    #[test]
    fn test_resolve_both_different_conflict() {
        let res = resolve_deletion_proof_key(Some(VALID_CANONICAL_KEY), Some(VALID_ALIAS_KEY));
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.key.as_str(), VALID_CANONICAL_KEY);
            assert!(!res.used_deprecated_alias);
            assert!(res.has_conflict);
        }
    }

    #[test]
    fn test_resolve_empty_or_whitespace_primary_falls_back_to_alias() {
        let res1 = resolve_deletion_proof_key(Some(""), Some(VALID_ALIAS_KEY));
        assert!(res1.is_ok());
        if let Ok(res1) = res1 {
            assert_eq!(res1.key.as_str(), VALID_ALIAS_KEY);
            assert!(res1.used_deprecated_alias);
            assert!(!res1.has_conflict);
        }

        let res2 = resolve_deletion_proof_key(Some("   \t\n "), Some(VALID_ALIAS_KEY));
        assert!(res2.is_ok());
        if let Ok(res2) = res2 {
            assert_eq!(res2.key.as_str(), VALID_ALIAS_KEY);
            assert!(res2.used_deprecated_alias);
            assert!(!res2.has_conflict);
        }
    }

    #[test]
    fn test_resolve_neither_set_returns_missing_error() {
        let res1 = resolve_deletion_proof_key(None, None);
        assert_eq!(res1, Err(ProofKeyError::MissingKey));

        let res2 = resolve_deletion_proof_key(Some("  "), Some("   "));
        assert_eq!(res2, Err(ProofKeyError::MissingKey));
    }

    #[test]
    fn test_resolve_weak_key_returns_invalid_key_error() {
        let res = resolve_deletion_proof_key(Some("short_key"), None);
        assert!(matches!(res, Err(ProofKeyError::InvalidKey(_))));
    }

    #[test]
    fn test_deletion_proof_key_from_env() {
        // Without env var set
        std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY_FILE");
        std::env::remove_var("CONTEXTRA_PROOF_KEY_FILE");
        std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
        std::env::remove_var("CONTEXTRA_PROOF_KEY");
        assert!(deletion_proof_key_from_env().is_err());

        // With valid env var set
        std::env::set_var("CONTEXTRA_DELETION_PROOF_KEY", VALID_CANONICAL_KEY);
        let res = deletion_proof_key_from_env();
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.as_str(), VALID_CANONICAL_KEY);
        }
        std::env::remove_var("CONTEXTRA_DELETION_PROOF_KEY");
    }
}
