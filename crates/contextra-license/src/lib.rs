#![forbid(unsafe_code)]

//! `contextra-license`
//!
//! License and activation gate module for Contextra feature rings (§14.6, §15).

pub use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};

/// Offener Default: alles unterhalb `Fast` ist immer erlaubt (Open-Source-Ring bleibt frei).
#[derive(Debug, Default, Clone, Copy)]
pub struct OpenFastGate;

impl LicenseGate for OpenFastGate {
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {
        match ring {
            FeatureRing::Fast => Ok(()),
            other => Err(LicenseError::NotActivated(other)),
        }
    }
}
