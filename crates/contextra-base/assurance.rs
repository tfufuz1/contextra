//! Trust modelling for key providers: closed enums instead of booleans.
use crate::time::UnixSeconds;

/// How credibly a provider can prove key destruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAssurance {
    /// TPM 2.0 / HSM with a modular destruction attestation.
    Hardware,
    /// Remote key service with a deletion API.
    Kms,
    /// Passphrase-protected key file. Development only.
    FilePassphrase,
}

/// Whether key copies exist in backups, and until when.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeysInBackups {
    /// Keys are never exported.
    Excluded,
    /// Copies exist but are destroyed at the given time (backup horizon).
    BoundedUntil(UnixSeconds),
    /// Copies exist with no known horizon.
    Unbounded,
}

/// Deployment profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Personal-data spaces require `Hardware` or `Kms`.
    Production,
    /// Any provider is accepted; receipts are capped.
    Development,
}

/// Refusal to open a personal-data space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileError {
    /// Production profile with insufficient key assurance.
    InsufficientAssurance(KeyAssurance),
}

impl core::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InsufficientAssurance(a) => {
                write!(f, "production profile refuses personal data with {a:?} keys")
            }
        }
    }
}
impl std::error::Error for ProfileError {}

impl Profile {
    /// Gate executed when a personal-data space is opened.
    pub fn admit_personal_data(self, assurance: KeyAssurance) -> Result<(), ProfileError> {
        match (self, assurance) {
            (Self::Production, KeyAssurance::FilePassphrase) => {
                Err(ProfileError::InsufficientAssurance(assurance))
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_rejects_file_keys() {
        assert!(Profile::Production.admit_personal_data(KeyAssurance::FilePassphrase).is_err());
        assert!(Profile::Production.admit_personal_data(KeyAssurance::Kms).is_ok());
        assert!(Profile::Production.admit_personal_data(KeyAssurance::Hardware).is_ok());
        assert!(Profile::Development.admit_personal_data(KeyAssurance::FilePassphrase).is_ok());
    }
}
