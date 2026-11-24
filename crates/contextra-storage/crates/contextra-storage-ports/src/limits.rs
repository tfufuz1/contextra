//! Grenzwerte und Konstanten (SP-PORTS, SPEC §4.6, §16, E-35/E-66).
//! Die Asserts erzwingen die Kette zur Compile-Zeit.
pub const KIB: usize = 1 << 10;
pub const MIB: usize = 1 << 20;
pub const GIB: u64 = 1 << 30;

pub const REDO_SEGMENT_BYTES: u64 = 64 * MIB as u64;
pub const REDO_ALIGN: u64 = 4096;
pub const REDO_MARK_BYTES: u64 = 8 * MIB as u64;

pub const DEFAULT_MAX_KEY_BYTES: usize = KIB;
pub const MAX_KEY_BYTES: usize = 4 * KIB;
pub const DEFAULT_MAX_HEADER_BYTES: usize = 4 * KIB;
pub const MAX_HEADER_BYTES: usize = 64 * KIB;
pub const MAX_BODY_BYTES: usize = 4 * MIB;
pub const MAX_CHANGE_OPS: usize = 4096;
pub const MAX_CHANGE_BYTES: usize = 8 * MIB;
pub const MAX_GROUP_CHANGES: usize = 1024;
pub const MAX_GROUP_BYTES: usize = 8 * MIB;
pub const MAX_PAGE_BYTES: usize = MIB;
pub const MAX_LAG_BYTES: u64 = GIB;

/// Capacity reserved for erase progress, consumer retention, filesystem
/// metadata headroom, and the dedicated purge scratch slots (SPEC §16).
pub const MIN_ERASE_RESERVE_BYTES: u64 = 2 * GIB;

pub const RUN_BLOCK_TARGET_BYTES: usize = 16 * KIB;
pub const RUN_RESTART_INTERVAL: usize = 16;
pub const MINI_SECTION_MAX_BYTES: usize = 4096;
pub const DIR_PAGE_BYTES: usize = 4096;

pub const CATALOG_ROLLOVER_MIN_BYTES: u64 = 8 * MIB as u64;
pub const STAMP_RESERVE: u64 = 1 << 20;
pub const ID_RESERVE: u64 = 1 << 16;
pub const STAMP_RESERVE_URGENT: u64 = 1 << 10;
pub const ID_RESERVE_URGENT: u64 = 1 << 8;

pub const DEFAULT_MAX_CLUSTERS: u64 = 16_000_000;
pub const HARD_MAX_CLUSTERS: u64 = 64_000_000;
pub const DEFAULT_MAX_OPEN_FILES: usize = 1024;
pub const FD_RESERVE: usize = 256;
pub const MAX_LIVE_FILES: usize = 262_144;
pub const MAX_CLUSTER_LOCAL_CLUSTERS: usize = 1024;
pub const MAX_GATE_CLUSTERS: usize = 4096;
pub const MAX_BRANCHES_PER_CLUSTER: usize = 1024;
pub const DEFAULT_MAX_BRANCH_DEPTH: u8 = 4;
pub const HARD_MAX_BRANCH_DEPTH: u8 = 8;

pub const KEY_GRACE_SECS: u64 = 600;
pub const COHORT_MAX: usize = 1024;

const _: () = assert!(MAX_BODY_BYTES * 2 <= MAX_CHANGE_BYTES);
const _: () = assert!((MAX_CHANGE_BYTES as u64) * 8 <= REDO_SEGMENT_BYTES);
const _: () = assert!((MAX_GROUP_BYTES as u64) * 8 <= REDO_SEGMENT_BYTES);
const _: () = assert!(DEFAULT_MAX_BRANCH_DEPTH <= HARD_MAX_BRANCH_DEPTH);
