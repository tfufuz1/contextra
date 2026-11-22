//! # Storage Engine
//!
//! Normative Referenzen: SPEC §10, §16 (INV-POISON-1).
//! INV-POISON-1: Ein vergiftetes Handle verweigert Schreiben und Wartung, liest weiter; health() liefert Poisoned.
//! - `memtable`: `RwLock<BTreeMap<InternalKey, Slot>>` mit 64-Op-Unterbatches (H-S7)
//! - `version`: Unveränderliche Versionen mit Nanosekunden-Austausch
//! - `views`: Sichtbarkeitsprotokoll und Leseordnung (INV-VIS-1, INV-VIS-2, INV-VIS-3)
//! - `retention`: Reine Versionenaufbewahrung nach §12.2 (INV-RET-1)
//! - `registry`: Sichten, Halteregeln, Konsumenten, Untergrenze und Claims (INV-CLAIM-1)
//! - `committer`: Flat-Combining Group-Commit mit Leiterwahl und Gegendruck (E-4, SD-49, SD-53)

pub mod committer;
pub mod memtable;
pub mod registry;
pub mod retention;
pub mod version;
pub mod views;

pub use committer::{
    BackpressureLevel, BackpressureSource, Change, CommitError, Committer, Durability, Guard,
    GuardPrewarmer, Op, Receipt, RedoWriter, Ticket,
};
pub use memtable::{Memtable, Slot, MEMTABLE_SUBBATCH_LIMIT};
pub use registry::{Claim, Consumer, Hold, Registry, RegistryError};
pub use retention::{retain, RetainItem};
pub use version::{BlobHandle, FileHandle, SpaceState, Version, VersionState};
pub use views::{create_view, View};
