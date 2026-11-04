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
//! ## Kernkomponenten (Soll-Module nach SPEC 3.1 & 8.3)
//! 1. [`sync`]: Synchronisation auf Speichermedien (`fsync`, `fdatasync`, `F_FULLFSYNC`, `flush_dir`).
//! 2. [`rename`]: Dauerhaftes Umbenennen (`rename_durable`).
//! 3. [`memlock`]: Speicherhaertung (`lock_pages`, `mark_dont_dump`, `secure_zero_volatile`).
//! 4. [`hardening`]: Prozesshaertung (`apply_process_hardening`, `RLIMIT_CORE`, `PR_SET_DUMPABLE`).
//! 5. [`perm`]: Dateiberechtigungen und Eigentuemer-Verifikation (`set_mode`, `verify_owner_only`).

pub mod durability;
pub mod error;
pub mod fs_probe;
pub mod hardening;
pub mod memlock;
pub mod perm;
pub mod rename;
pub mod sync;

// Oeffentliche Re-Exporte gemaess Spezifikation 03-contextra-sys.md & SPEC 3.1
pub use durability::{
    durable_flush_dir, durable_flush_file, flush_dir, flush_dir_sync, flush_file, flush_file_sync,
    rename_durable, rename_durable_sync, write_all_durable, FlushKind,
};
pub use error::{SyncFailure, SysError, UnsupportedKind};
pub use fs_probe::{fs_class, fs_class_spec, FsClass, FsKind};
pub use hardening::{
    apply_process_hardening, ensure_fd_limit, lock_pages, mark_dont_dump, secure_zero_volatile,
    set_fd_limit, HardeningPolicy, HardeningReport,
};
pub use perm::{set_mode, set_restrictive_file_acl, verify_file_acl_owner_only, verify_owner_only};
