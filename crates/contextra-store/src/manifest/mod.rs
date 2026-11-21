//! SSTable Manifest for append-only tracking of active SSTable sets.

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

pub mod version;
pub use version::Version;
