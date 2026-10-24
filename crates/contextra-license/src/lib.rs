#![forbid(unsafe_code)]

//! `contextra-license`
//!
//! License and activation gate module for Contextra feature rings (§14.6, §15).

pub use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};

pub mod signed_gate;
pub use signed_gate::{
    derive_local_installation_id_hash, LicensePayload, SignedActivation, SignedLicenseGate,
};
