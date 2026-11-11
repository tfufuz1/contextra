//! SSTable Manifest for append-only tracking of active SSTable sets.
// FILE-CONTEXT
// STAND: 2026-09-11
// ZWECK: Append-Only Manifest-Protokolldatei zur Verfolgung gültiger SSTables für Crash-Safety.
// INVARIANTEN: fsync NACH jedem Manifest-Eintrag; Add erst nach fsync der SSTable-Datei.

mod apply;
mod cleanup;
mod core;
mod entry;
mod handle;
mod recovery;
mod rollover;

#[cfg(test)]
mod tests;

pub use core::Manifest;
pub use entry::ManifestEntry;

/// Maximum allowed payload size for a single manifest entry (1 MB).
pub const MAX_MANIFEST_ENTRY_SIZE: u32 = 1024 * 1024;

/// Default file size threshold (64 KB) to trigger MANIFEST rollover.
pub const DEFAULT_ROLLOVER_THRESHOLD_BYTES: u64 = 64 * 1024;

pub use core::{CURRENT_MANIFEST_VERSION, MANIFEST_HEADER_MAGIC};
