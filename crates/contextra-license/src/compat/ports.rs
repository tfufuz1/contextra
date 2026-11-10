//! Port adapter implementations for `contextra-ports`.

use crate::{LicenseGate, SignedLicenseGate};

impl contextra_ports::license::LicenseGate for SignedLicenseGate {
    fn check_ring(
        &self,
        ring: contextra_ports::license::FeatureRing,
    ) -> std::result::Result<(), contextra_ports::license::LicenseError> {
        LicenseGate::check_ring(self, ring.into()).map_err(Into::into)
    }
}
