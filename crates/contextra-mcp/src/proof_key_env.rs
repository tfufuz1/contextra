// FILE-CONTEXT
// STAND:       2026-10-13
// ZWECK:       Einziger Resolver für DeletionProof-Schlüssel (CONTEXTRA_DELETION_PROOF_KEY / CONTEXTRA_PROOF_KEY sowie Key-Files).
// INVARIANTEN: Bevorzugt Key-Files und CONTEXTRA_DELETION_PROOF_KEY; Mindestqualität (len >= 32, Entropie >= 16 distinct, keine Null/Wiederholung/Dummy-Schlüssel) wird erzwungen; Loggt Schlüsselmaterial NIEMALS.

use std::fmt;
use std::sync::Once;
use zeroize::Zeroizing;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Quelle des aufgelösten Schlüssels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofKeySource {
    PrimaryFile,
    LegacyFile,
    PrimaryEnv,
    LegacyEnv,
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
    pub error: Option<String>,
}

impl fmt::Debug for ProofKeyResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProofKeyResolution")
            .field("key_set", &self.key.is_some())
            .field("source", &self.source)
            .field("has_conflict", &self.has_conflict)
            .field("used_legacy", &self.used_legacy)
            .field("error", &self.error)
            .finish()
    }
}

static DEPRECATION_WARN_ONCE: Once = Once::new();
static CONFLICT_WARN_ONCE: Once = Once::new();
static RAW_ENV_WARN_ONCE: Once = Once::new();

/// Validiert die Mindestqualität des Schlüsselmaterials.
///
/// Invarianten:
/// - Mindestlänge: >= 32 Bytes
/// - Kein Nullschlüssel (nicht alle Bytes zero / ascii '0')
/// - Keine Wiederholungsmuster (Chunk-Längen 1..=16)
/// - Ausreichende Entropie (mindestens 16 unterschiedliche Byte-Werte)
/// - Keine bekannten Dokumentations-/Testschlüssel
pub fn validate_proof_key_quality(key_bytes: &[u8]) -> Result<(), String> {
    if key_bytes.len() < 32 {
        return Err(
            "deletion proof key quality check failed: key length must be at least 32 bytes"
                .to_string(),
        );
    }

    if key_bytes.iter().all(|&b| b == 0) || key_bytes.iter().all(|&b| b == b'0') {
        return Err("deletion proof key quality check failed: null key rejected".to_string());
    }

    for chunk_len in 1..=16 {
        if key_bytes.len() % chunk_len == 0 {
            let chunk = &key_bytes[..chunk_len];
            if key_bytes.chunks(chunk_len).all(|c| c == chunk) {
                return Err(
                    "deletion proof key quality check failed: repeating pattern rejected"
                        .to_string(),
                );
            }
        }
    }

    let mut seen = [false; 256];
    let mut distinct = 0usize;
    for &b in key_bytes {
        if !seen[b as usize] {
            seen[b as usize] = true;
            distinct += 1;
        }
    }
    if distinct < 16 {
        return Err("deletion proof key quality check failed: insufficient entropy (less than 16 distinct bytes)".to_string());
    }

    if let Ok(key_str) = std::str::from_utf8(key_bytes) {
        let trimmed = key_str.trim();
        let known_bad = [
            "00000000000000000000000000000000",
            "12345678901234567890123456789012",
            "abcdefghijklmnopqrstuvwxyz123456",
            "secret_key_12345678901234567890",
            "change_me_in_production_32bytes!",
            "YOUR_DELETION_PROOF_KEY_HERE___",
            "0x00000000000000000000000000000000",
        ];
        for &bad in &known_bad {
            if trimmed.eq_ignore_ascii_case(bad) {
                return Err(
                    "deletion proof key quality check failed: known test/default key value rejected"
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}

/// Lädt das Schlüsselmaterial sicher aus einer Schlüsseldatei.
///
/// Prüft Dateityp (kein Symlink), auf Unix strikte Dateirechte (0600) und Eigentümer (Prozess-UID/Root).
pub fn load_proof_key_from_file(path_str: &str) -> Result<Zeroizing<String>, String> {
    let path = std::path::Path::new(path_str);
    let symlink_meta = std::fs::symlink_metadata(path)
        .map_err(|e| format!("deletion proof key file error: failed to read file metadata: {e}"))?;

    if symlink_meta.file_type().is_symlink() {
        return Err("deletion proof key file error: path is a symbolic link".to_string());
    }

    #[cfg(unix)]
    {
        let mode = symlink_meta.permissions().mode() & 0o777;
        if mode != 0o600 {
            return Err(format!(
                "deletion proof key file error: file permissions too permissive (0{:o}); required 0600",
                mode
            ));
        }

        let file_uid = symlink_meta.uid();
        let current_uid = std::env::var("USER_ID")
            .or_else(|_| std::env::var("UID"))
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1000);
        if file_uid != current_uid && file_uid != 0 {
            return Err(format!(
                "deletion proof key file error: key file ownership mismatch (file UID {file_uid}, running as UID {current_uid})"
            ));
        }
    }

    let raw_bytes = std::fs::read(path)
        .map_err(|e| format!("deletion proof key file error: failed to read key file: {e}"))?;

    let key_str = match std::str::from_utf8(&raw_bytes) {
        Ok(s) => s.trim().to_string(),
        Err(_) => {
            return Err("deletion proof key file error: key file is not valid UTF-8".to_string());
        }
    };

    validate_proof_key_quality(key_str.as_bytes())?;

    Ok(Zeroizing::new(key_str))
}

/// Reine Hilfsfunktion zur Auflösung des DeletionProof-Schlüssels aus Textwerten ohne Zugriff auf die Umgebung.
/// Leerstrings oder rein aus Whitespace bestehende Strings zählen als `None`.
pub fn resolve_proof_key(primary: Option<String>, legacy: Option<String>) -> ProofKeyResolution {
    let clean_primary = primary
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let clean_legacy = legacy
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let (chosen, source, has_conflict, used_legacy) = match (clean_primary, clean_legacy) {
        (Some(p), Some(l)) => {
            if p == l {
                (Some(p), ProofKeySource::BothSame, false, false)
            } else {
                (Some(p), ProofKeySource::BothDifferentConflict, true, false)
            }
        }
        (Some(p), None) => (Some(p), ProofKeySource::PrimaryEnv, false, false),
        (None, Some(l)) => (Some(l), ProofKeySource::LegacyEnv, false, true),
        (None, None) => (None, ProofKeySource::PrimaryEnv, false, false),
    };

    if let Some(k) = chosen {
        match validate_proof_key_quality(k.as_bytes()) {
            Ok(()) => ProofKeyResolution {
                key: Some(Zeroizing::new(k)),
                source: Some(source),
                has_conflict,
                used_legacy,
                error: None,
            },
            Err(err) => ProofKeyResolution {
                key: None,
                source: Some(source),
                has_conflict,
                used_legacy,
                error: Some(err),
            },
        }
    } else {
        ProofKeyResolution {
            key: None,
            source: None,
            has_conflict: false,
            used_legacy: false,
            error: None,
        }
    }
}

/// Liest Schlüsseldateien oder Umgebungsvariablen (`CONTEXTRA_DELETION_PROOF_KEY_FILE`,
/// `CONTEXTRA_PROOF_KEY_FILE`, `CONTEXTRA_DELETION_PROOF_KEY`, `CONTEXTRA_PROOF_KEY`),
/// löst den Schlüssel auf und emittiert einmalige Warnungen bei Deprecation, Konflikt oder Env-Nutzung.
pub fn resolve_proof_key_from_env() -> ProofKeyResolution {
    let primary_file = std::env::var("CONTEXTRA_DELETION_PROOF_KEY_FILE").ok();
    let legacy_file = std::env::var("CONTEXTRA_PROOF_KEY_FILE").ok();
    let primary_env = std::env::var("CONTEXTRA_DELETION_PROOF_KEY").ok();
    let legacy_env = std::env::var("CONTEXTRA_PROOF_KEY").ok();

    if let Some(file_path) = primary_file
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        match load_proof_key_from_file(file_path) {
            Ok(key) => {
                return ProofKeyResolution {
                    key: Some(key),
                    source: Some(ProofKeySource::PrimaryFile),
                    has_conflict: false,
                    used_legacy: false,
                    error: None,
                };
            }
            Err(err) => {
                return ProofKeyResolution {
                    key: None,
                    source: Some(ProofKeySource::PrimaryFile),
                    has_conflict: false,
                    used_legacy: false,
                    error: Some(err),
                };
            }
        }
    }

    if let Some(file_path) = legacy_file
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        DEPRECATION_WARN_ONCE.call_once(|| {
            tracing::warn!(
                "CONTEXTRA_PROOF_KEY_FILE ist veraltet (deprecated). Bitte CONTEXTRA_DELETION_PROOF_KEY_FILE verwenden."
            );
        });
        match load_proof_key_from_file(file_path) {
            Ok(key) => {
                return ProofKeyResolution {
                    key: Some(key),
                    source: Some(ProofKeySource::LegacyFile),
                    has_conflict: false,
                    used_legacy: true,
                    error: None,
                };
            }
            Err(err) => {
                return ProofKeyResolution {
                    key: None,
                    source: Some(ProofKeySource::LegacyFile),
                    has_conflict: false,
                    used_legacy: true,
                    error: Some(err),
                };
            }
        }
    }

    let res = resolve_proof_key(primary_env, legacy_env);

    if res.used_legacy {
        DEPRECATION_WARN_ONCE.call_once(|| {
            tracing::warn!(
                "CONTEXTRA_PROOF_KEY ist veraltet (deprecated). Bitte CONTEXTRA_DELETION_PROOF_KEY_FILE verwenden."
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

    if res.key.is_some() {
        RAW_ENV_WARN_ONCE.call_once(|| {
            tracing::warn!(
                "DeletionProof-Schlüssel über Umgebungsvariable gesetzt. Die Verwendung von Schlüsseldateien (CONTEXTRA_DELETION_PROOF_KEY_FILE) wird dringend empfohlen."
            );
        });
    }

    res
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    const VALID_PRIM_KEY: &str = "primary_deletion_proof_key_32bytes_valid!";
    const VALID_LEG_KEY: &str = "legacy_deletion_proof_key_32bytes_valid!!";

    #[test]
    fn test_resolve_only_primary() {
        let res = resolve_proof_key(Some(VALID_PRIM_KEY.into()), None);
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some(VALID_PRIM_KEY));
        assert_eq!(res.source, Some(ProofKeySource::PrimaryEnv));
        assert!(!res.has_conflict);
        assert!(!res.used_legacy);
        assert!(res.error.is_none());
    }

    #[test]
    fn test_resolve_only_legacy() {
        let res = resolve_proof_key(None, Some(VALID_LEG_KEY.into()));
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some(VALID_LEG_KEY));
        assert_eq!(res.source, Some(ProofKeySource::LegacyEnv));
        assert!(!res.has_conflict);
        assert!(res.used_legacy);
        assert!(res.error.is_none());
    }

    #[test]
    fn test_resolve_both_same() {
        let res = resolve_proof_key(Some(VALID_PRIM_KEY.into()), Some(VALID_PRIM_KEY.into()));
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some(VALID_PRIM_KEY));
        assert_eq!(res.source, Some(ProofKeySource::BothSame));
        assert!(!res.has_conflict);
        assert!(!res.used_legacy);
        assert!(res.error.is_none());
    }

    #[test]
    fn test_resolve_both_different() {
        let res = resolve_proof_key(Some(VALID_PRIM_KEY.into()), Some(VALID_LEG_KEY.into()));
        assert_eq!(res.key.as_deref().map(|s| s.as_str()), Some(VALID_PRIM_KEY));
        assert_eq!(res.source, Some(ProofKeySource::BothDifferentConflict));
        assert!(res.has_conflict);
        assert!(!res.used_legacy);
        assert!(res.error.is_none());
    }

    #[test]
    fn test_resolve_neither() {
        let res = resolve_proof_key(None, None);
        assert!(res.key.is_none());
        assert_eq!(res.source, None);
        assert!(!res.has_conflict);
        assert!(!res.used_legacy);
        assert!(res.error.is_none());
    }

    #[test]
    fn test_resolve_empty_string_treated_as_unset() {
        let res1 = resolve_proof_key(Some("  ".into()), Some(VALID_LEG_KEY.into()));
        assert_eq!(res1.key.as_deref().map(|s| s.as_str()), Some(VALID_LEG_KEY));
        assert_eq!(res1.source, Some(ProofKeySource::LegacyEnv));
        assert!(res1.used_legacy);

        let res2 = resolve_proof_key(Some("  ".into()), Some("   ".into()));
        assert!(res2.key.is_none());
        assert_eq!(res2.source, None);
    }

    #[test]
    fn test_debug_impl_does_not_leak_key() {
        let res = resolve_proof_key(Some(VALID_PRIM_KEY.into()), None);
        let debug_output = format!("{:?}", res);
        assert!(!debug_output.contains(VALID_PRIM_KEY));
        assert!(debug_output.contains("key_set: true"));
    }

    #[test]
    fn test_quality_validation_rejects_weak_keys() {
        assert!(validate_proof_key_quality(b"short_key").is_err());
        assert!(validate_proof_key_quality(&[0u8; 32]).is_err());
        assert!(validate_proof_key_quality(b"00000000000000000000000000000000").is_err());
        assert!(validate_proof_key_quality(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_err());
        assert!(validate_proof_key_quality(b"12341234123412341234123412341234").is_err());
        assert!(validate_proof_key_quality(b"12345678901234567890123456789012").is_err());
        assert!(validate_proof_key_quality(VALID_PRIM_KEY.as_bytes()).is_ok());
    }
}
