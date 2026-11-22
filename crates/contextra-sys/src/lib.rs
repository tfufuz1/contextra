#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![doc = "Contextra Unsafe System Abstraction Crate (Ring 0 - Unsafe-Insel)"]

pub mod acl_win32;
pub mod durability;
pub mod mlock;
pub mod mmap;
pub mod posix;
pub mod vault;

pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only};
pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable};
pub use mlock::{mem_lock, mem_lock_with_policy, mem_unlock, MemLockPolicy};
pub use mmap::mmap_readonly;
pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit};
pub use vault::LockedRegions;
