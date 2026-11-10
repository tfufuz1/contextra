//! Compatibility adapters for `contextra-types` and `contextra-ports`.

use crate::{Clock, FeatureRing, LicenseError, LicenseGate, OpenFastGate, TenantId};

impl From<FeatureRing> for contextra_ports::license::FeatureRing {
    fn from(ring: FeatureRing) -> Self {
        match ring {
            FeatureRing::Fast => contextra_ports::license::FeatureRing::Fast,
            FeatureRing::Sovereign => contextra_ports::license::FeatureRing::Sovereign,
            FeatureRing::Compliance => contextra_ports::license::FeatureRing::Compliance,
        }
    }
}

impl From<contextra_ports::license::FeatureRing> for FeatureRing {
    fn from(ring: contextra_ports::license::FeatureRing) -> Self {
        match ring {
            contextra_ports::license::FeatureRing::Fast => FeatureRing::Fast,
            contextra_ports::license::FeatureRing::Sovereign => FeatureRing::Sovereign,
            contextra_ports::license::FeatureRing::Compliance => FeatureRing::Compliance,
            _ => FeatureRing::Compliance,
        }
    }
}

impl From<LicenseError> for contextra_ports::license::LicenseError {
    fn from(err: LicenseError) -> Self {
        match err {
            LicenseError::NotActivated(ring) => {
                contextra_ports::license::LicenseError::NotActivated(ring.into())
            }
            LicenseError::InvalidSignature => {
                contextra_ports::license::LicenseError::InvalidSignature
            }
            LicenseError::Expired(ts) => contextra_ports::license::LicenseError::Expired(ts),
        }
    }
}

impl From<contextra_ports::license::LicenseError> for LicenseError {
    fn from(err: contextra_ports::license::LicenseError) -> Self {
        match err {
            contextra_ports::license::LicenseError::NotActivated(ring) => {
                LicenseError::NotActivated(ring.into())
            }
            contextra_ports::license::LicenseError::InvalidSignature => {
                LicenseError::InvalidSignature
            }
            contextra_ports::license::LicenseError::Expired(ts) => LicenseError::Expired(ts),
        }
    }
}

impl From<TenantId> for contextra_types::TenantId {
    fn from(id: TenantId) -> Self {
        contextra_types::TenantId(id.0)
    }
}

impl From<contextra_types::TenantId> for TenantId {
    fn from(id: contextra_types::TenantId) -> Self {
        TenantId(id.0)
    }
}

impl contextra_ports::license::LicenseGate for OpenFastGate {
    fn check_ring(
        &self,
        ring: contextra_ports::license::FeatureRing,
    ) -> std::result::Result<(), contextra_ports::license::LicenseError> {
        LicenseGate::check_ring(self, ring.into()).map_err(Into::into)
    }
}

impl<T: contextra_ports::clock::Clock + ?Sized> Clock for T {
    fn now_unix_nanos(&self) -> u64 {
        contextra_ports::clock::Clock::now_unix_nanos(self)
    }
}
