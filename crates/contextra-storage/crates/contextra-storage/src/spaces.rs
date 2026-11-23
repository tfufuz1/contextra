//! Space-Beschreibung (SP-FACADE). Platzhalter; Space-Liste folgt SP-CLUSTER.
use contextra_storage_ports::{Protection, SpaceId};

#[derive(Debug, Clone)]
pub struct SpaceSpec {
    pub id: SpaceId,
    pub protection: Protection,
    pub name: String,
}
