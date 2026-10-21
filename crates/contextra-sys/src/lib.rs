#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Contextra Unsafe System Abstraction Crate (Ring 0 - Unsafe-Insel)"]

pub mod acl_win32;
pub mod aligned;
pub mod clock;
pub mod determinism;
pub mod durability;
pub mod env_observe;
pub mod hardening;
pub mod mlock;
pub mod mmap;
pub mod posix;
pub mod rng;
// TODO(WP-S1 / PR-3 Schritt 3): `crates/contextra-sys/src/crypto_rng.rs` implementieren (`pub mod crypto_rng;`).
// Implementierung von `OsCryptoRng` unter Verwendung des OS-Entropie-Syscalls (z.B. via `getrandom`),
// um echten kryptographischen Zufall in Ring 0 bereitzustellen und den `CryptoRng`-Trait zu erfüllen.
pub mod vault;
pub mod wiring;

pub use wiring::SysWiring;

pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use aligned::{AlignedBuf, ALIGNMENT};
pub use determinism::{SeededRng, SystemClock, SystemIdGen, SystemRng};
pub use durability::{
    flush_dir, flush_file, fs_class, rename_durable, FlushKind, FsClass, OsSyscallBackend,
    SyscallBackend,
};
pub use env_observe::*;
// TODO: pub use crypto_rng::OsCryptoRng;
pub use hardening::{harden_process, ProcessHardeningStatus};
pub use mlock::{mem_lock, mem_unlock};
pub use mmap::mmap_readonly;
pub use posix::{reopen_and_dup2, statvfs_available_bytes};
pub use vault::LockedRegions;
