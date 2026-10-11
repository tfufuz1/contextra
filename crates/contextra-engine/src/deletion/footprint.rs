// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: ErasureFootprint & StorageScope Typen für das Garantiemodell (WP-1.1).

use contextra_types::TenantId;
use serde::{Deserialize, Serialize};

/// Speicherscope für den Footprint (untenanted vs. tenant-spezifisch).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageScope {
    Untenanted,
    Tenant(TenantId),
}

/// Schlüsselbereich mit expliziten Einzel-Keys und Präfixen.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyScope {
    pub explicit: Vec<Vec<u8>>,
    pub prefixes: Vec<Vec<u8>>,
}

/// Repräsentiert den Erasure Footprint für eine Löschoperation inklusive BLAKE3-Hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErasureFootprint {
    pub storage: StorageScope,
    pub keys: KeyScope,
    pub hash: [u8; 32],
}

impl ErasureFootprint {
    /// Erstellt einen neuen `ErasureFootprint` und berechnet automatisch dessen Hash.
    pub fn new(storage: StorageScope, keys: KeyScope) -> Self {
        let mut footprint = Self {
            storage,
            keys,
            hash: [0u8; 32],
        };
        footprint.hash = footprint.compute_hash();
        footprint
    }

    /// Berechnet den BLAKE3-basierten, längenpräfixierten Hash über den Footprint.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // 1. StorageScope
        match &self.storage {
            StorageScope::Untenanted => {
                hasher.update(&[0u8]);
            }
            StorageScope::Tenant(tenant_id) => {
                hasher.update(&[1u8]);
                hasher.update(&tenant_id.inner().to_le_bytes());
            }
        }

        // 2. KeyScope explicit keys (längenpräfixiert)
        let explicit_len = u64::try_from(self.keys.explicit.len()).unwrap_or(0);
        hasher.update(&explicit_len.to_le_bytes());
        for key in &self.keys.explicit {
            let key_len = u64::try_from(key.len()).unwrap_or(0);
            hasher.update(&key_len.to_le_bytes());
            hasher.update(key);
        }

        // 3. KeyScope prefixes (längenpräfixiert)
        let prefixes_len = u64::try_from(self.keys.prefixes.len()).unwrap_or(0);
        hasher.update(&prefixes_len.to_le_bytes());
        for prefix in &self.keys.prefixes {
            let prefix_len = u64::try_from(prefix.len()).unwrap_or(0);
            hasher.update(&prefix_len.to_le_bytes());
            hasher.update(prefix);
        }

        *hasher.finalize().as_bytes()
    }
}
