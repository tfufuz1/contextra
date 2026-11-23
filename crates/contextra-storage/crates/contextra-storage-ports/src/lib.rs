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
    CallerMeta, Change, Durability, Guard, Op, Origin, Precondition, Protection, Receipt,
};
pub use error::{Error, ErrorClass};
pub use ids::{BranchId, ClusterId, IdemToken, Redacted, SpaceId, Stamp, STAMP_MAX};
pub use keys::{Key32, KeyLookup, KeyProbe, KeyProvider};
