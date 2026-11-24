// =========================================================================
// ANCHOR[id=sys-lib-main,epic=sys-core,seq=0] Crate-Wurzel und Baukasten-Export
// =========================================================================
// LINK crates/contextra-sys/Cargo.toml
// LINK crates/contextra-sys/SPEC.md
// LINK crates/contextra-sys/PLAN.md
// NOTE: Low-level unsafe system abstractions and FFI island for Contextra (E0 / Ring 0).
// Invariante: Autarkes Crate (0 interne Abhängigkeiten).
// Keine unzulässigen Garantieversprechen (SPEC §6.4). Zero panic in Produktion.

#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![doc = "Contextra Unsafe System Abstraction Crate (Ring 0 / E0 - Unsafe-Insel)"]

//! # Contextra-Sys: Low-Level-Systemabstraktionen und Unsafe-Insel (E0 / Ring 0)
//!
//! Dieses Modul bildet die hardware- und betriebssystemnahe Basis (Ring 0, Ebene E0) für das Contextra-System
//! und externe Konsumenten im Baustein-Ökosystem.
//! Es kapselt alle systemweiten Zugriffe, die betriebssystemspezifische FFI-Aufrufe (Foreign Function Interface,
//! Schnittstelle zu betriebssystemeigenem C-Code) oder `unsafe`-Code erfordern, in sicheren, typisierten Rust-Schnittstellen.
//!
//! ## Baukasten-Einordnung und Architektur
//! - **Ebene E0 (Kern / Unsafe-Insel):** Darf von höheren Schichten eingebunden werden, hängt selbst von keinen anderen
//!   Contextra-Crates ab (autarker Baustein).
//! - **Abhängigkeiten (ausgehend):** Keine internen Workspace-Abhängigkeiten. Externe Crates: `libc` (POSIX),
//!   `windows-sys` (Win32), `memmap2`, `thiserror`, `tracing`.
//! - **Eingehende Abhängigkeiten (Beispiel-Konsumenten):**
//!   - `contextra-vault` (RAII-Tresorpuffer aufbauend auf `sys::vault`)
//!   - `contextra-store` (LSM-Speicher und WAL = Write-Ahead-Log)
//!   - `contextra-vector` (Vektorindex und DiskANN-Speicherabbildung)
//!   - Hinweis: `contextra-durable-fs` ist eine eigenständige Dateisystem-Abstraktion (SPEC v3 S-03R / §6.4).
//!
//! ## Kernkomponenten
//! 1. [`acl_win32`]: Plattformspezifische Zugriffsbeschränkungen (DACL unter Windows, chmod `0600` und UID-Prüfung unter Unix).
//! 2. [`contract`]: Minimale Umwelt-Traits ([`MemoryLocker`]) und I/O-freie Referenzimplementierungen ([`NoopMemoryLocker`]).
//! 3. [`durability`]: Haltbare Dateisystem-Synchronisation (WAL/LSM fsync, fdatasync, F_FULLFSYNC, FlushFileBuffers, Handle-Poisoning).
//! 4. [`error`]: Strukturiertes Fehlermodell (`thiserror`, `#[non_exhaustive]`) mit maschinenlesbarem OS-Errno.
//! 5. [`mlock`]: Festpinnen sensibler Speicherseiten im RAM (`mlock`, `VirtualLock`, `MADV_DONTDUMP`).
//! 6. [`mmap`]: Sichere Nur-Lese-Speicherabbildung (`mmap_readonly`, `SealedFile`, `SealedMap`) für Vektordaten und SSTables.
//! 7. [`posix`]: POSIX-Ressourcenlimits (`rlimit` für Dateideskriptoren) und Deskriptorumleitungen (`dup2`).
//! 8. [`vault`]: RAII-Sicherheitsverwaltung ([`LockedRegions`], [`VaultBuffer`]) für sensible Schlüsselpuffer inklusive sicherem Nullsetzen (Zeroize).

// SECTION Submodule
// ANCHOR[id=sys-submodules,epic=sys-core,seq=1] Deklaration der internen Module
// LINK crates/contextra-sys/src/acl_win32.rs
pub mod acl_win32;
// LINK crates/contextra-sys/src/contract.rs
pub mod contract;
// LINK crates/contextra-sys/src/durability.rs
pub mod durability;
// LINK crates/contextra-sys/src/error.rs
pub mod error;
// LINK crates/contextra-sys/src/mlock.rs
pub mod mlock;
// LINK crates/contextra-sys/src/mmap.rs
pub mod mmap;
// LINK crates/contextra-sys/src/posix.rs
pub mod posix;
// LINK crates/contextra-sys/src/vault.rs
pub mod vault;
// !SECTION Submodule

// SECTION Baukasten-Zielmodule
// ANCHOR[id=sys-building-blocks,epic=sys-core,seq=2] Modulare Baukasten-Namespaces

/// Baukasten-Zielmodul `mem`: Kapselt Memory-Locking, Vault-Speicher und Zeroization.
// LINK crates/contextra-sys/src/mlock.rs
// LINK crates/contextra-sys/src/vault.rs
pub mod mem {
    pub use crate::contract::{MemoryLocker, NoopMemoryLocker, OsMemoryLocker};
    pub use crate::mlock::{
        apply_dont_dump, mem_lock, mem_lock_typed, mem_lock_with_policy, mem_unlock, MemLockPolicy,
    };
    pub use crate::vault::{LockedRegions, VaultBuffer};
}

/// Baukasten-Zielmodul `fsmeta`: Plattformspezifische Datei-ACLs und Eigentümerprüfungen.
// LINK crates/contextra-sys/src/acl_win32.rs
pub mod fsmeta {
    pub use crate::acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
}

/// Baukasten-Zielmodul `map`: Sichere Speicherabbildung (INV-IO-1 / INV-ENCRYPT-REST-2).
// LINK crates/contextra-sys/src/mmap.rs
pub mod map {
    pub use crate::mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed};
}
// !SECTION Baukasten-Zielmodule

// SECTION Public Re-exports
// ANCHOR[id=sys-reexports,epic=sys-core,seq=3] Ergonomische Direktzugriffe
// NOTE: Re-Exports für ergonomischen Direktzugriff durch übergeordnete Ringe (z. B. contextra-store, contextra-vector, contextra-vault).
pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use contract::{MemoryLocker, NoopMemoryLocker, OsMemoryLocker};
pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable, DurableFile};
pub use error::{Error, Result, SysError};
pub use mlock::{
    apply_dont_dump, mem_lock, mem_lock_typed, mem_lock_with_policy, mem_unlock, MemLockPolicy,
};
pub use mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed};
pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit};
pub use vault::{LockedRegions, VaultBuffer};
// !SECTION Public Re-exports
