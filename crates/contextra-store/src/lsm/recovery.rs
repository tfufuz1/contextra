//! # Crash-Recovery des LSM-Trees (Ring 1)
//!
//! ZWECK: Stellt den letzten konsistenten Zustand durch Manifest- und WAL-Replay wieder her.
//! Zerlegt in kohärente Untermodule:
//! - [`salt`]: Salt-Generierung, atomare Speicherung und Verzeichnisprüfung.
//! - [`startup`]: Initialer Start, Replay, Rekonstruktion und Hintergrundtasks.
//! - [`rollback`]: Rollback-Intent-Verarbeitung und selektive SSTable-Bereinigung.

pub(crate) mod salt;
pub(crate) mod startup;
pub(crate) mod rollback;

// Re-Exporte fuer Rueckwaertskompatibilitaet und Aussenflaeche
pub(crate) use salt::{
    directory_is_pristine, generate_crypto_salt, generate_crypto_salt_with_source,
    next_temp_file_counter, write_salt_atomically,
};
pub use rollback::{INTENT_MAGIC, MIN_ENTRIES_FOR_SSTABLE_REBUILD};
