//! `contextra-base`: the single vocabulary layer of the storage stack.
//!
//! Rules (enforced by lints and `cargo deny`):
//! * no `unsafe` (OS access lives in `contextra-sys`),
//! * no I/O, no async runtime, no crypto *implementations*,
//! * exactly one definition of every shared type below.
#![forbid(unsafe_code)]

pub mod assurance;
pub mod canon;
pub mod receipt;
pub mod secret;
pub mod time;

pub use assurance::{KeyAssurance, KeysInBackups, Profile, ProfileError};
pub use receipt::{CompletionStatus, ReceiptBody, Residual};
pub use secret::Key32;
pub use time::{Clock, Entropy, UnixSeconds};
