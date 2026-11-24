#![allow(dead_code)]
// Origin: crates/contextra-store/src
// Commit: f92caabaa65d74648d64b96c5fffcfd42f922814

pub const LEGACY_SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D;
pub const LEGACY_SSTABLE_MAGIC_LEGACY: u32 = 0x4D46_5354;
pub const LEGACY_INTENT_MAGIC: &[u8; 8] = b"MFRLBK\0\0";
pub const LEGACY_WAL_V2_HEADER: [u8; 4] = *b"MFW2";
pub const LEGACY_WAL_V3_HEADER: [u8; 4] = *b"MFW3";
