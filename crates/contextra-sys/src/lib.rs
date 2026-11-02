#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![doc = "Contextra Unsafe System Abstraction Crate (Ring 1 - Unsafe-Insel)"]

//! # Contextra-Sys: Low-Level-Systemabstraktionen und Unsafe-Insel (Ring 1)
//!
//! Dieses Modul bildet die hardware- und betriebssystemnahe Basis fuer das Contextra-System.
//! Es kapselt alle systemweiten Zugriffe, die betriebssystemspezifische FFI-Aufrufe
//! oder `unsafe`-Code erfordern, in sicheren, typisierten Rust-Schnittstellen.
//!
//! ## Ring-Zugehoerigkeit und Architektur
//! - **Ring 1 (Kern / Unsafe-Insel):** Darf von hoeheren Ringen eingebunden werden.
//! - **Abhaengigkeiten:** `libc`, `tracing`.
//!
//! ## Kernkomponenten
//! 1. [`durability`]: Haltbare Dateisystem-Synchronisation (`flush_file`, `flush_dir`, `rename_durable`).
//! 2. [`fs_probe`]: Dateisystemerkennung fuer die Startprobe (`fs_class`).
//! 3. [`hardening`]: Prozess- und Speicherhaertung (`apply_process_hardening`, `mark_dont_dump`, `lock_pages`).
//! 4. [`perm`]: Dateiberechtigungen und Eigentuemer-Verifikation (`set_mode`, `verify_owner_only`).
//! 5. [`error`]: Fehler- und Statustypen (`SysError`, `SyncFailure`, `UnsupportedKind`).

pub mod durability;
pub mod error;
pub mod fs_probe;
pub mod hardening;
pub mod perm;

// Oeffentliche Re-Exporte gemaess Spezifikation 03-contextra-sys.md
pub use durability::{
    durable_flush_dir, durable_flush_file, flush_dir, flush_dir_sync, flush_file, flush_file_sync,
    rename_durable, rename_durable_sync, write_all_durable, FlushKind,
};
pub use error::{SyncFailure, SysError, UnsupportedKind};
pub use fs_probe::{fs_class, fs_class_spec, FsClass, FsKind};
pub use hardening::{
    apply_process_hardening, ensure_fd_limit, lock_pages, mark_dont_dump, set_fd_limit,
    HardeningPolicy, HardeningReport,
};
pub use perm::{set_mode, set_restrictive_file_acl, verify_file_acl_owner_only, verify_owner_only};
