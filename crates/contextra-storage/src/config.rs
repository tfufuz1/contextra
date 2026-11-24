//! Konfiguration und Validierung (SP-FACADE, E-35, E-74..E-81). Validate prüft die Grenzkette.
use contextra_storage_ports::limits::*;
use contextra_storage_ports::traits::{Anchor, Clock, Entropy, Fs, Mac};
use contextra_storage_ports::{Error, Key32, KeyProvider};
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

pub const DEFAULT_VIEW_LEASE_NANOS: u64 = 60 * 1_000_000_000;
pub const DEFAULT_VIEW_LEASE_MAX_NANOS: u64 = 600 * 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpenProfile {
    Development,
    #[default]
    Production,
}

#[derive(Clone)]
pub struct Config {
    pub max_body_bytes: usize,
    pub max_change_bytes: usize,
    pub max_group_bytes: usize,
    pub redo_segment_bytes: u64,
    pub max_clusters: u64,
    pub max_open_files: usize,
    pub erase_reserve_bytes: u64,
    pub max_lag_bytes: u64,
    pub key_grace_secs: u64,
    pub open_profile: OpenProfile,
    pub view_lease_nanos: u64,
    pub view_lease_max_nanos: u64,
    /// Injected environment and root are required for persistent open.
    pub root: Option<PathBuf>,
    pub fs: Option<Arc<dyn Fs>>,
    pub anchor: Option<Arc<dyn Anchor>>,
    pub clock: Option<Arc<dyn Clock>>,
    pub entropy: Option<Arc<dyn Entropy>>,
    pub key_provider: Option<Arc<dyn KeyProvider>>,
    pub catalog_key: Option<Arc<Key32>>,
    pub catalog_mac: Option<Arc<dyn Mac>>,
    pub catalog_integrity: crate::catalog::CatalogIntegrity,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("max_body_bytes", &self.max_body_bytes)
            .field("max_change_bytes", &self.max_change_bytes)
            .field("max_group_bytes", &self.max_group_bytes)
            .field("redo_segment_bytes", &self.redo_segment_bytes)
            .field("max_clusters", &self.max_clusters)
            .field("max_open_files", &self.max_open_files)
            .field("erase_reserve_bytes", &self.erase_reserve_bytes)
            .field("max_lag_bytes", &self.max_lag_bytes)
            .field("key_grace_secs", &self.key_grace_secs)
            .field("open_profile", &self.open_profile)
            .field("view_lease_nanos", &self.view_lease_nanos)
            .field("view_lease_max_nanos", &self.view_lease_max_nanos)
            .field("root_configured", &self.root.is_some())
            .field("fs_configured", &self.fs.is_some())
            .field("anchor_configured", &self.anchor.is_some())
            .field("clock_configured", &self.clock.is_some())
            .field("entropy_configured", &self.entropy.is_some())
            .field("key_provider_configured", &self.key_provider.is_some())
            .field("catalog_key_configured", &self.catalog_key.is_some())
            .field("catalog_integrity", &self.catalog_integrity)
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_body_bytes: MAX_BODY_BYTES,
            max_change_bytes: MAX_CHANGE_BYTES,
            max_group_bytes: MAX_GROUP_BYTES,
            redo_segment_bytes: REDO_SEGMENT_BYTES,
            max_clusters: DEFAULT_MAX_CLUSTERS,
            max_open_files: DEFAULT_MAX_OPEN_FILES,
            erase_reserve_bytes: MIN_ERASE_RESERVE_BYTES,
            max_lag_bytes: MAX_LAG_BYTES,
            key_grace_secs: KEY_GRACE_SECS,
            open_profile: OpenProfile::Production,
            view_lease_nanos: DEFAULT_VIEW_LEASE_NANOS,
            view_lease_max_nanos: DEFAULT_VIEW_LEASE_MAX_NANOS,
            root: None,
            fs: None,
            anchor: None,
            clock: None,
            entropy: None,
            key_provider: None,
            catalog_key: None,
            catalog_mac: None,
            catalog_integrity: crate::catalog::CatalogIntegrity::Plain,
        }
    }
}

impl Config {
    pub fn with_filesystem(mut self, root: PathBuf, fs: Arc<dyn Fs>) -> Self {
        self.root = Some(root);
        self.fs = Some(fs);
        self
    }

    pub fn with_anchor(mut self, anchor: Arc<dyn Anchor>) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    pub fn with_entropy(mut self, entropy: Arc<dyn Entropy>) -> Self {
        self.entropy = Some(entropy);
        self
    }

    pub fn with_key_provider(mut self, provider: Arc<dyn KeyProvider>) -> Self {
        self.key_provider = Some(provider);
        self
    }

    pub fn with_catalog_auth(mut self, key: Arc<Key32>, mac: Arc<dyn Mac>) -> Self {
        self.catalog_key = Some(key);
        self.catalog_mac = Some(mac);
        self.catalog_integrity = crate::catalog::CatalogIntegrity::Authenticated;
        self
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.max_body_bytes == 0 || self.max_body_bytes > MAX_BODY_BYTES {
            return Err(Error::Limit {
                limit: "max_body_bytes must be 1..=4 MiB",
            });
        }
        if self.max_change_bytes == 0 || self.max_change_bytes > MAX_CHANGE_BYTES {
            return Err(Error::Limit {
                limit: "max_change_bytes must be 1..=8 MiB",
            });
        }
        if self.max_group_bytes == 0 || self.max_group_bytes as u64 > self.redo_segment_bytes {
            return Err(Error::Limit {
                limit: "max_group_bytes must fit one redo segment",
            });
        }
        if self.redo_segment_bytes == 0 || self.redo_segment_bytes % REDO_ALIGN != 0 {
            return Err(Error::Limit {
                limit: "redo_segment_bytes must be nonzero and REDO_ALIGN aligned",
            });
        }
        if self.max_clusters == 0 || self.max_clusters > 64_000_000 {
            return Err(Error::Limit {
                limit: "max_clusters must be in 1..=64 million",
            });
        }
        if self.max_lag_bytes == 0 {
            return Err(Error::Limit {
                limit: "max_lag_bytes must be nonzero",
            });
        }
        if self.max_body_bytes.saturating_mul(2) > self.max_change_bytes {
            return Err(Error::Limit {
                limit: "max_body_bytes <= max_change_bytes / 2 (E-35)",
            });
        }
        if (self.max_change_bytes as u64).saturating_mul(8) > self.redo_segment_bytes {
            return Err(Error::Limit {
                limit: "max_change_bytes <= redo_segment_bytes / 8 (E-35)",
            });
        }
        if self.max_open_files == 0 {
            return Err(Error::Limit {
                limit: "max_open_files >= 1",
            });
        }
        if self.erase_reserve_bytes < MIN_ERASE_RESERVE_BYTES {
            return Err(Error::Limit {
                limit: "erase_reserve_bytes must be at least 2 GiB",
            });
        }
        if self.view_lease_nanos == 0 || self.view_lease_nanos > self.view_lease_max_nanos {
            return Err(Error::Limit {
                limit: "view_lease_nanos must be 1..=view_lease_max_nanos",
            });
        }
        Ok(())
    }
}
