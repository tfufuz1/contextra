#![allow(dead_code)]
// Origin: crates/contextra-store/src/manifest/core.rs
// Commit: f92caabaa65d74648d64b96c5fffcfd42f922814
// Lines: 20-30, 85-95, 195-205

pub const LEGACY_MANIFEST_HEADER_MAGIC: &[u8; 4] = b"MFMN";
pub const LEGACY_CURRENT_MANIFEST_VERSION: u8 = 1;

pub fn parse_manifest_header_v1(header: &[u8]) -> Result<u8, &'static str> {
    if header.len() < 5 {
        return Err("Header too short: expected at least 5 bytes");
    }
    if &header[0..4] != LEGACY_MANIFEST_HEADER_MAGIC {
        return Err("Magic mismatch: expected MFMN");
    }
    let version = header[4];
    if version == 0 {
        return Err("Invalid version 0");
    }
    Ok(version)
}

pub fn create_manifest_header(version: u8) -> [u8; 5] {
    let mut h = [0u8; 5];
    h[0..4].copy_from_slice(LEGACY_MANIFEST_HEADER_MAGIC);
    h[4] = version;
    h
}
