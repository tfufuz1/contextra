#![forbid(unsafe_code)]

//! `contextra-license`
//!
//! License and activation gate module for Contextra feature rings (§14.6, §15).

pub use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};

pub mod signed_gate;
pub use signed_gate::{
    constant_time_eq_32, derive_local_installation_id_hash, verify_step_1_fast_ring,
    verify_step_2_presence, verify_step_3_installation, verify_step_4_signature,
    verify_step_5_expiration, verify_step_6_ring_permission, verify_step_7_success, LicensePayload,
    SignedActivation, SignedLicenseGate, SignedLicenseOptions, StepVerificationError,
    MAX_PAYLOAD_BYTES,
};
