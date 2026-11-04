//! # LSM-Konfiguration (Ring 2)
//!
//! Konfiguriert Schwellenwerte für MemTable-Größe, Flush-Intervalle und WAL-Pfade.
//!
//! Verschlüsselung ist nicht abschaltbar (D-05): Der `KeyManager` wird immer
//! über `LsmStorage::open_with_key_manager` injiziert. `LsmConfig` selbst enthält
//! keine Passphrase (SPEC D-05, P5).

use crate::compaction::CompactionConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// WAL-Dauerhaftigkeitsmodus.
///
/// Nur `Full` ist zulässig (D-05). `WalNoHmac` und `MemoryOnly` wurden entfernt,
/// da sie die WAL-Integrität abschwächen würden (F-21, D-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum DurabilityMode {
    /// Vollständige Persistenz mit HMAC-Kette (einzige zulässige Variante, D-05).
    #[default]
    Full,
    /// Nur für In-Memory-Tests ohne Dateisystem-I/O.
    MemoryOnly,
}



#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DurabilityConfigError {
    #[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")]
    IncompatibleCombination {
        mode: DurabilityMode,
        feature: &'static str,
        reason: &'static str,
    },
}

impl From<DurabilityConfigError> for contextra_types::ContextraError {
    fn from(err: DurabilityConfigError) -> Self {
        contextra_types::ContextraError::DurabilityConfig(err.to_string())
    }
}

/// Konfiguration des LSM-Speichers.
///
/// Verschlüsselung ist Pflicht (D-05): Der `KeyManager` wird separat über
/// `LsmStorage::open_with_key_manager` injiziert. Diese Konfiguration enthält
/// kein Schlüsselmaterial.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LsmConfig {
    /// Pfad zum Datenverzeichnis.
    pub path: PathBuf,
    /// Maximale MemTable-Größe vor Flush.
    pub memtable_size_limit: usize,
    /// Maximale RAM-Nutzung in MB.
    pub max_ram_mb: u64,
    /// Transaktions-Timeout.
    pub tx_timeout: Duration,
    /// Kompaktierungskonfiguration.
    pub compaction: CompactionConfig,
    /// Zeitfenster in Mikrosekunden für Gruppen-Commit-Batching.
    /// 0 = kein Batching (sofortiger Einzel-Commit).
    pub group_commit_window_micros: u64,
    /// Anzahl der Block-Cache-Shards.
    pub block_cache_shards: usize,
    /// WAL-Dauerhaftigkeitsmodus. Standard: `Full` (D-05).
    pub durability_mode: DurabilityMode,
    /// Maximale Kapazität der SSI-Validierungstabelle.
    pub ssi_max_tracked_keys: usize,
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("contextra_data"),
            memtable_size_limit: 64 * 1024 * 1024,
            max_ram_mb: 2048,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            group_commit_window_micros: 500,
            block_cache_shards: 64,
            durability_mode: DurabilityMode::default(),
            ssi_max_tracked_keys: 1_000_000,
        }
    }
}
