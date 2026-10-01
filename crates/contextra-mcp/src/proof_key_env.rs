// FILE-CONTEXT
// STAND:       2026-10-01
// ZWECK:       Einziger Resolver für DeletionProof-Schlüssel (CONTEXTRA_DELETION_PROOF_KEY / CONTEXTRA_PROOF_KEY)
// INVARIANTEN: Bevorzugt CONTEXTRA_DELETION_PROOF_KEY; Loggt Schlüsselmaterial NIEMALS.

use std::fmt;
use std::sync::Once;
use zeroize::Zeroizing;

/// Quelle des aufgelösten Schlüssels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofKeySource {
    Primary,
    Legacy,
    BothSame,
    BothDifferentConflict,
}

/// Ergebnis der Schlüssel-Auflösung.
#[derive(Clone, PartialEq, Eq)]
pub struct ProofKeyResolution {
    pub key: Option<Zeroizing<String>>,
    pub source: Option<ProofKeySource>,
    pub has_conflict: bool,
    pub used_legacy: bool,
}

impl fmt::Debug for ProofKeyResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProofKeyResolution")
            .field("key_set", &self.key.is_some())
            .field("source", &self.source)
            .field("has_conflict", &self.has_conflict)
            .field("used_legacy", &self.used_legacy)
            .finish()
    }
}

static DEPRECATION_WARN_ONCE: Once = Once::new();
static CONFLICT_WARN_ONCE: Once = Once::new();

/// Reine Hilfsfunktion zur Auflösung des DeletionProof-Schlüssels ohne Zugriff auf die Prozess-Umgebung.
/// Leerstrings oder rein aus Whitespace bestehende Strings zählen als `None`.
pub fn resolve_proof_key(primary: Option<String>, legacy: Option<String>) -> ProofKeyResolution {
    let clean_primary = primary
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let clean_legacy = legacy
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    match (clean_primary, clean_legacy) {
        (Some(p), Some(l)) => {
            if p == l {
                ProofKeyResolution {
                    key: Some(Zeroizing::new(p)),
                    source: Some(ProofKeySource::BothSame),
                    has_conflict: false,
                    used_legacy: false,
                }
            } else {
                ProofKeyResolution {
                    key: Some(Zeroizing::new(p)),
                    source: Some(ProofKeySource::BothDifferentConflict),
                    has_conflict: true,
                    used_legacy: false,
                }
            }
        }
        (Some(p), None) => ProofKeyResolution {
            key: Some(Zeroizing::new(p)),
            source: Some(ProofKeySource::Primary),
            has_conflict: false,
            used_legacy: false,
        },
        (None, Some(l)) => ProofKeyResolution {
            key: Some(Zeroizing::new(l)),
            source: Some(ProofKeySource::Legacy),
            has_conflict: false,
            used_legacy: true,
        },
        (None, None) => ProofKeyResolution {
            key: None,
            source: None,
            has_conflict: false,
            used_legacy: false,
        },
    }
}

/// Liest beide Umgebungsvariablen (`CONTEXTRA_DELETION_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`),
/// löst den Schlüssel per `resolve_proof_key` auf und emittiert einmalig Warnungen bei Deprecation oder Konflikt.
pub fn resolve_proof_key_from_env() -> ProofKeyResolution {
    let primary = std::env::var("CONTEXTRA_DELETION_PROOF_KEY").ok();
    let legacy = std::env::var("CONTEXTRA_PROOF_KEY").ok();

    let res = resolve_proof_key(primary, legacy);

    if res.used_legacy {
        DEPRECATION_WARN_ONCE.call_once(|| {
            tracing::warn!(
                "CONTEXTRA_PROOF_KEY ist veraltet (deprecated). Bitte CONTEXTRA_DELETION_PROOF_KEY verwenden."
            );
        });
    }

    if res.has_conflict {
        CONFLICT_WARN_ONCE.call_once(|| {
            tracing::warn!(
                "Konflikt: Sowohl CONTEXTRA_DELETION_PROOF_KEY als auch CONTEXTRA_PROOF_KEY sind mit unterschiedlichen Werten gesetzt. CONTEXTRA_DELETION_PROOF_KEY wird bevorzugt."
            );
        });
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_only_primary() {
        let res = resolve_proof_key(Some("prim_key".into()), None);
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some("prim_key"));
        assert_eq!(res.source, Some(ProofKeySource::Primary));
        assert!(!res.has_conflict);
        assert!(!res.used_legacy);
    }

    #[test]
    fn test_resolve_only_legacy() {
        let res = resolve_proof_key(None, Some("leg_key".into()));
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some("leg_key"));
        assert_eq!(res.source, Some(ProofKeySource::Legacy));
        assert!(!res.has_conflict);
        assert!(res.used_legacy);
    }

    #[test]
    fn test_resolve_both_same() {
        let res = resolve_proof_key(Some("same_key".into()), Some("same_key".into()));
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some("same_key"));
        assert_eq!(res.source, Some(ProofKeySource::BothSame));
        assert!(!res.has_conflict);
        assert!(!res.used_legacy);
    }

    #[test]
    fn test_resolve_both_different() {
        let res = resolve_proof_key(Some("prim_key".into()), Some("leg_key".into()));
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some("prim_key"));
        assert_eq!(res.source, Some(ProofKeySource::BothDifferentConflict));
        assert!(res.has_conflict);
        assert!(!res.used_legacy);
    }

    #[test]
    fn test_resolve_neither() {
        let res = resolve_proof_key(None, None);
        assert!(res.key.is_none());
        assert_eq!(res.source, None);
        assert!(!res.has_conflict);
        assert!(!res.used_legacy);
    }

    #[test]
    fn test_resolve_empty_string_treated_as_unset() {
        let res1 = resolve_proof_key(Some("  ".into()), Some("leg_key".into()));
        assert_eq!(res1.key.as_deref().map(|s| s.as_str()), Some("leg_key"));
        assert_eq!(res1.source, Some(ProofKeySource::Legacy));
        assert!(res1.used_legacy);

        let res2 = resolve_proof_key(Some("  ".into()), Some("   ".into()));
        assert!(res2.key.is_none());
        assert_eq!(res2.source, None);
    }

    #[test]
    fn test_debug_impl_does_not_leak_key() {
        let res = resolve_proof_key(Some("secret_password_123".into()), None);
        let debug_output = format!("{:?}", res);
        assert!(!debug_output.contains("secret_password_123"));
        assert!(debug_output.contains("key_set: true"));
    }
}
