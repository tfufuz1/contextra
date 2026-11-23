//! Port-Traits (SP-PORTS §2). Jede Fähigkeit hat genau einen Trait.
pub mod anchor;
pub mod authority;
pub mod cold;
pub mod crypto;
pub mod engine;
pub mod fs;
pub mod memory;
pub mod observe;
pub mod spawn;

pub use anchor::{Anchor, AnchorStrength, AnchorValue, Clock, Entropy};
pub use authority::{Authority, FourEyesProof, Principal};
pub use crypto::{Checksum, Cipher, Kdf, Mac};
pub use fs::{File, FileRole, Fs, FsCaps, IoCtx};
pub use memory::{MemoryBudget, ResourceClass, ResourcePermit};
