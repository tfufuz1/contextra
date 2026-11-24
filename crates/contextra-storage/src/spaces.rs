//! Immutable Space configuration and pre-mutation validation (SP-FACADE §4.5).

use crate::codec::format::IntegrityMode;
use contextra_storage_ports::{Durability, Error, Protection, SpaceId, FIRST_USER_SPACE_ID};

pub use contextra_storage_ports::DataClass;

pub const DEFAULT_PURE_THRESHOLD: u64 = 8 * 1024 * 1024;
pub const DEFAULT_VALUE_SEPARATION_MIN: u32 = 1024;
pub const DEFAULT_MAX_KEY_BYTES: u32 = 1024;
pub const MAX_KEY_BYTES: u32 = 4096;
pub const MAX_WEIGHT_SLOTS: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceClass { Durable, Volatile }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpiryBucket { Month, Quarter, Year }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackPolicy {
    None,
    ByTenantAndExpiry { bucket: ExpiryBucket, shards: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealPolicy { ByMode, Cluster }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryPolicy { KeepLatest, KeepUntilFloor, AppendOnly }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression { None }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpaceOptions {
    pub class: SpaceClass,
    pub clustered: bool,
    pub data_class: DataClass,
    pub packing: PackPolicy,
    pub pure_threshold: u64,
    pub seal: SealPolicy,
    pub weight_slots: u8,
    pub history: HistoryPolicy,
    pub value_separation_min: u32,
    pub max_key_bytes: u32,
    pub compression: Compression,
    pub durability_default: Durability,
}

impl Default for SpaceOptions {
    fn default() -> Self {
        Self {
            class: SpaceClass::Durable,
            clustered: true,
            data_class: DataClass::NonPersonal,
            packing: PackPolicy::None,
            pure_threshold: DEFAULT_PURE_THRESHOLD,
            seal: SealPolicy::ByMode,
            weight_slots: MAX_WEIGHT_SLOTS,
            history: HistoryPolicy::KeepUntilFloor,
            value_separation_min: DEFAULT_VALUE_SEPARATION_MIN,
            max_key_bytes: DEFAULT_MAX_KEY_BYTES,
            compression: Compression::None,
            durability_default: Durability::Durable,
        }
    }
}

impl SpaceOptions {
    pub fn volatile() -> Self {
        Self { class: SpaceClass::Volatile, durability_default: Durability::Relaxed, ..Self::default() }
    }

    /// Validates the immutable option set before any catalog, key, or file I/O.
    pub fn validate(&self, integrity: IntegrityMode) -> Result<(), Error> {
        if self.data_class == DataClass::Personal {
            if !self.clustered { return Err(Error::Limit { limit: "personal spaces must be clustered" }); }
            if integrity != IntegrityMode::Encrypted {
                return Err(Error::Limit { limit: "personal data requires encrypted integrity mode" });
            }
            if self.class == SpaceClass::Durable && self.seal != SealPolicy::Cluster {
                return Err(Error::Limit { limit: "durable personal space must seal by cluster" });
            }
        }
        if !self.clustered && self.seal == SealPolicy::Cluster {
            return Err(Error::Limit { limit: "cluster seal requires clustered space" });
        }
        if !self.clustered && self.packing != PackPolicy::None {
            return Err(Error::Limit { limit: "packing requires clustered space" });
        }
        if let PackPolicy::ByTenantAndExpiry { shards, .. } = self.packing {
            if shards == 0 { return Err(Error::Limit { limit: "pack shards must be nonzero" }); }
        }
        if self.pure_threshold == 0 { return Err(Error::Limit { limit: "pure threshold must be nonzero" }); }
        if self.weight_slots > MAX_WEIGHT_SLOTS { return Err(Error::Limit { limit: "weight slots must be <= 4" }); }
        if self.max_key_bytes == 0 || self.max_key_bytes > MAX_KEY_BYTES {
            return Err(Error::Limit { limit: "max key bytes must be 1..=4096" });
        }
        if self.value_separation_min == 0 {
            return Err(Error::Limit { limit: "value separation minimum must be nonzero" });
        }
        if self.class == SpaceClass::Volatile && self.durability_default == Durability::Durable {
            return Err(Error::Limit { limit: "volatile spaces cannot default to durable writes" });
        }
        Ok(())
    }

    pub fn protection_allowed(&self, protection: Protection) -> bool {
        !(protection == Protection::Plain
            && (self.data_class == DataClass::Personal || self.seal == SealPolicy::Cluster))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceSpec {
    pub id: SpaceId,
    pub name: String,
    pub options: SpaceOptions,
}

impl SpaceSpec {
    pub fn validate(&self, integrity: IntegrityMode) -> Result<(), Error> {
        if self.id.0 < FIRST_USER_SPACE_ID { return Err(Error::ReservedSpace); }
        if self.name.is_empty() || self.name.len() > 64 {
            return Err(Error::Limit { limit: "space name length must be 1..=64 bytes" });
        }
        self.options.validate(integrity)
    }
}
