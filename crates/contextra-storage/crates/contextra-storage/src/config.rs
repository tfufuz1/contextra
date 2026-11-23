//! Konfiguration und Validierung (SP-FACADE, E-35). Validate prüft die Grenzkette.
use contextra_storage_ports::limits::*;
use contextra_storage_ports::traits::{Fs, Mac};
use contextra_storage_ports::{Error, Key32};
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

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
    /// Injected environment and root are required for persistent open.
    pub root: Option<PathBuf>,
    pub fs: Option<Arc<dyn Fs>>,
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
            .field("root_configured", &self.root.is_some())
            .field("fs_configured", &self.fs.is_some())
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
            erase_reserve_bytes: 256 * MIB as u64,
            max_lag_bytes: MAX_LAG_BYTES,
            key_grace_secs: KEY_GRACE_SECS,
            root: None,
            fs: None,
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

    pub fn with_catalog_auth(mut self, key: Arc<Key32>, mac: Arc<dyn Mac>) -> Self {
        self.catalog_key = Some(key);
        self.catalog_mac = Some(mac);
        self.catalog_integrity = crate::catalog::CatalogIntegrity::Authenticated;
        self
    }

    pub fn validate(&self) -> Result<(), Error> {
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
        Ok(())
    }
}
