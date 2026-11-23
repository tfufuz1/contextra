//! Konfiguration und Validierung (SP-FACADE, E-35). Validate prüft die Grenzkette.
use contextra_storage_ports::limits::*;
use contextra_storage_ports::Error;

#[derive(Debug, Clone)]
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
        }
    }
}

impl Config {
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
