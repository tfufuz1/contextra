//! contextra-storage-ports: Verträge, Werttypen, Fehler, Limits. Keine Logik (SP-PORTS).
#![forbid(unsafe_code)]

pub mod change;
pub mod error;
pub mod facade;
pub mod ids;
pub mod keys;
pub mod limits;
pub mod traits;

pub use change::{
    CallerMeta, Change, ClusterLifecycle, Durability, Guard, Op, Origin, Precondition, Protection, Receipt,
};
pub use error::{Error, ErrorClass};
pub use ids::{BatchId, BranchId, ClusterId, IdemToken, Redacted, SpaceId, Stamp, FIRST_USER_SPACE_ID, STAMP_MAX, SYSTEM_SPACE_ID_MAX};
pub use keys::{Key32, KeyCreateFailure, KeyCreateResult, KeyLookup, KeyProbe, KeyProvider, MAX_KEYS_PER_BATCH};
