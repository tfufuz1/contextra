//! Environment observation and attestation TLV encoding (§C8, §C9, WP-1.1).
//!
//! # Observation vs Guarantee (INV-D10)
//!
//! **IMPORTANT**: Environment observation provides empirical system state inspection
//! for evidence generation. Residual risks (CoW filesystems, TRIM/discard, block encryption,
//! active swap, core dump generation) are reported as **observed**, NEVER as **guaranteed**.
//! Software cannot guarantee physical SSD remanence erasure or underlying firmware/hardware
//! behavior without cryptographic erasure (S4).
//!
//! Process-level hardening (`harden_runtime`, e.g., setting `RLIMIT_CORE=0` or `PR_SET_DUMPABLE=0`)
//! is deliberately **omitted** from this module, as process state modification requires
//! architecture-level ADR approval and baseline policy updates.

#![forbid(unsafe_code)]

use std::path::Path;

pub use contextra_sys::env_observe::{
    self, cow_fs_from_fs_kind_and_options, observe, parse_core_pattern, parse_dm_uuid,
    parse_limits, parse_mountinfo, parse_proc_swaps, parse_rotational, EnvFsKind, EnvObservation,
    EnvOsKind, EnvTri,
};

/// Encodes an empirical [`EnvObservation`] snapshot along with security flags into canonical TLV bytes (§C9).
///
/// The binary format uses deterministic tag ordering:
/// - Tag u16 BE (2 bytes)
/// - Length u32 BE (4 bytes, fixed 1 byte for u8 values)
/// - Value u8 (1 byte)
///
/// Tag Mapping:
/// - `0x0001`: `os` ([`EnvOsKind`])
/// - `0x0002`: `fs_kind` ([`EnvFsKind`])
/// - `0x0003`: `cow_fs` ([`EnvTri`])
/// - `0x0004`: `discard_enabled` ([`EnvTri`])
/// - `0x0005`: `block_encrypted` ([`EnvTri`])
/// - `0x0006`: `rotational` ([`EnvTri`])
/// - `0x0007`: `swap_active` ([`EnvTri`])
/// - `0x0008`: `core_dumps_possible` ([`EnvTri`])
/// - `0x0009`: `secure_buffers` (`bool`, 1 or 0)
/// - `0x000A`: `secure_delete_applied` (`bool`, 1 or 0)
#[must_use]
pub fn encode_observation_tlv(
    obs: &EnvObservation,
    secure_buffers: bool,
    secure_delete_applied: bool,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(70);

    let fields: [(u16, u8); 10] = [
        (0x0001, obs.os.to_u8()),
        (0x0002, obs.fs_kind.to_u8()),
        (0x0003, obs.cow_fs.to_u8()),
        (0x0004, obs.discard_enabled.to_u8()),
        (0x0005, obs.block_encrypted.to_u8()),
        (0x0006, obs.rotational.to_u8()),
        (0x0007, obs.swap_active.to_u8()),
        (0x0008, obs.core_dumps_possible.to_u8()),
        (0x0009, u8::from(secure_buffers)),
        (0x000A, u8::from(secure_delete_applied)),
    ];

    for (tag, val) in fields {
        buf.extend_from_slice(&tag.to_be_bytes());
        buf.extend_from_slice(&1u32.to_be_bytes());
        buf.push(val);
    }

    buf
}

/// Samples system environment observation for `path` and returns encoded TLV bytes (§C8, §C9).
///
/// Parameters `secure_buffers` and `secure_delete_applied` default to `false`.
/// Any unreadable or unparseable source yields [`EnvTri::Unknown`](contextra_sys::env_observe::EnvTri::Unknown).
///
/// # Observation vs Guarantee (INV-D10)
///
/// Environment observations are strictly empirical and never constitute physical guarantees.
/// Hardware remanence (e.g. SSD flash wear-leveling) remains outside software bounds without S4.
#[must_use]
pub fn observe_environment(
    path: &Path,
    secure_buffers: bool,
    secure_delete_applied: bool,
) -> Vec<u8> {
    let obs = env_observe::observe(path);
    encode_observation_tlv(&obs, secure_buffers, secure_delete_applied)
}
