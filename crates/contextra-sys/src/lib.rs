#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![doc = "Contextra Unsafe System Abstraction Crate (Ring 0 - Unsafe-Insel)"]

//! # Contextra-Sys: Low-Level-Systemabstraktionen und Unsafe-Insel (Ring 0)
//!
//! Dieses Modul bildet die hardware- und betriebssystemnahe Basis (Ring 0) für das Contextra-System.
//! Es kapselt alle systemweiten Zugriffe, die betriebssystemspezifische FFI-Aufrufe (Foreign Function Interface,
//! Schnittstelle zu betriebssystemeigenem C-Code) oder `unsafe`-Code erfordern, in sicheren, typisierten Rust-Schnittstellen.
//!
//! ## Ring-Zugehörigkeit und Architektur
//! - **Ring 0 (Kern / Unsafe-Insel):** Darf von höheren Ringen eingebunden werden, hängt selbst von keinen anderen
//!   Contextra-Crates ab.
//! - **Abhängigkeiten (ausgehend):** Keine internen Workspace-Abhängigkeiten. Externe Crates: `libc` (POSIX),
//!   `windows-sys` (Win32), `memmap2`, `tracing`.
//! - **Eingehende Abhängigkeiten (wer nutzt contextra-sys):**
//!   - `contextra-db` (Transaktions- und Datenbankverwaltung)
//!   - `contextra-durable-fs` (Dauerhafte Dateisystem-Abstraktionen)
//!   - `contextra-engine` (Such- und Indexierungs-Engine)
//!   - `contextra-store` (LSM-Speicher und WAL = Write-Ahead-Log)
//!   - `contextra-vector` (Vektorindex und DiskANN-Speicherabbildung)
//!
//! ## Kernkomponenten
//! 1. [`acl_win32`]: Plattformspezifische Zugriffsbeschränkungen (DACL unter Windows, chmod `0600` und UID-Prüfung unter Unix).
//! 2. [`durability`]: Haltbare Dateisystem-Synchronisation (WAL/LSM fsync, fdatasync, F_FULLFSYNC, FlushFileBuffers).
//! 3. [`mlock`]: Festpinnen sensibler Speicherseiten im RAM (Verhinderung von Paging/Swapping auf die Festplatte).
//! 4. [`mmap`]: Sichere Nur-Lese-Speicherabbildung (`mmap_readonly`) für Vektordaten und SSTables.
//! 5. [`posix`]: POSIX-Ressourcenlimits (`rlimit` für Dateideskriptoren) und Deskriptorumleitungen (`dup2`).
//! 6. [`vault`]: RAII-Sicherheitsverwaltung ([`LockedRegions`]) für sensible Schlüsselpuffer inklusive sicherem Nullsetzen (Zeroize).

pub mod acl_win32;
pub mod durability;
pub mod mlock;
pub mod mmap;
pub mod posix;
pub mod vault;

// ANCHOR[contextra-sys:LIB-REEXPORT-001]
// Öffentliche Re-Exports für ergonomischen Direktzugriff durch übergeordnete Ringe (z. B. contextra-store, contextra-engine).
pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable};
pub use mlock::{mem_lock, mem_lock_with_policy, mem_unlock, MemLockPolicy};
pub use mmap::mmap_readonly;
pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit};
pub use vault::LockedRegions;
