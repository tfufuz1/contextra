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
    CallerMeta, Change, ClusterLifecycle, Durability, EraseLocality, Guard, Op, Origin,
    Precondition, Protection, Receipt,
};
pub use error::{Error, ErrorClass};
pub use ids::{
    BranchId, ClusterId, DataClass, DurablePosition, EraseRequestId,
    IdemToken, InstanceId, Redacted, SpaceId, Stamp, Ticket, TombId,
    FIRST_USER_SPACE_ID, STAMP_MAX, SYSTEM_SPACE_ID_MAX,
};
pub use keys::{
    BatchId, DestroyBehavior, DestroyReceipt, KekId, Key32, KeyAssurance, KeyCreateResult,
    KeyListEntry, KeyLookup, KeyProvider, KeyProviderInfo, KeyState, KeysInBackups,
    KEY_PROVIDER_BATCH_MAX,
};
