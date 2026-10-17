// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: ErasureFootprint, FprRamFootprint & StorageScope Typen für das Garantiemodell (WP-1.1, C1).

use contextra_types::{ContextraError, DocId, TenantId};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

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

/// RAM-Footprint für In-Memory-Strukturen und flüchtige Indizes.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FprRamFootprint {
    pub doc_ids: Vec<DocId>,
    pub term_bytes: Vec<Vec<u8>>,
    pub graph_entities: Vec<String>,
    pub relation_keys: Vec<Vec<u8>>,
}

impl std::fmt::Debug for FprRamFootprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FprRamFootprint")
            .field("doc_ids_count", &self.doc_ids.len())
            .field("term_bytes_count", &self.term_bytes.len())
            .field("graph_entities_count", &self.graph_entities.len())
            .field("relation_keys_count", &self.relation_keys.len())
            .finish()
    }
}

impl FprRamFootprint {
    /// Berechnet den BLAKE3-basierten Hash über den RAM-Footprint.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        let doc_len = u64::try_from(self.doc_ids.len()).unwrap_or(0);
        hasher.update(&doc_len.to_le_bytes());
        for id in &self.doc_ids {
            hasher.update(&id.inner().to_le_bytes());
        }

        let term_len = u64::try_from(self.term_bytes.len()).unwrap_or(0);
        hasher.update(&term_len.to_le_bytes());
        for term in &self.term_bytes {
            let len = u64::try_from(term.len()).unwrap_or(0);
            hasher.update(&len.to_le_bytes());
            hasher.update(term);
        }

        let graph_len = u64::try_from(self.graph_entities.len()).unwrap_or(0);
        hasher.update(&graph_len.to_le_bytes());
        for entity in &self.graph_entities {
            let len = u64::try_from(entity.len()).unwrap_or(0);
            hasher.update(&len.to_le_bytes());
            hasher.update(entity.as_bytes());
        }

        let rel_len = u64::try_from(self.relation_keys.len()).unwrap_or(0);
        hasher.update(&rel_len.to_le_bytes());
        for rel in &self.relation_keys {
            let len = u64::try_from(rel.len()).unwrap_or(0);
            hasher.update(&len.to_le_bytes());
            hasher.update(rel);
        }

        *hasher.finalize().as_bytes()
    }

    /// Überschreibt und löscht Inhalte sicher aus dem Speicher.
    pub fn zeroize_contents(&mut self) {
        self.doc_ids.clear();
        for t in &mut self.term_bytes {
            t.zeroize();
        }
        self.term_bytes.clear();
        for g in &mut self.graph_entities {
            let mut bytes = std::mem::take(g).into_bytes();
            bytes.zeroize();
        }
        self.graph_entities.clear();
        for r in &mut self.relation_keys {
            r.zeroize();
        }
        self.relation_keys.clear();
    }
}

/// Repräsentiert den Erasure Footprint für eine Löschoperation inklusive BLAKE3-Hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErasureFootprint {
    pub storage: StorageScope,
    pub keys: KeyScope,
    pub hash: [u8; 32],
    pub ram: FprRamFootprint,
    pub ram_hash: [u8; 32],
    pub observed_empty: bool,
}

impl ErasureFootprint {
    /// Erstellt einen neuen `ErasureFootprint` und berechnet automatisch dessen Hash.
    pub fn new(storage: StorageScope, keys: KeyScope) -> Self {
        let ram = FprRamFootprint::default();
        let ram_hash = ram.compute_hash();
        let mut footprint = Self {
            storage,
            keys,
            hash: [0u8; 32],
            ram,
            ram_hash,
            observed_empty: false,
        };
        footprint.hash = footprint.compute_hash();
        footprint
    }

    /// Funktionale Konstruktor-Methode für Dokument-Scope.
    pub fn for_document(tenant_id: TenantId, collection_name: &str, doc_id: DocId) -> Self {
        Self::fuer_document(tenant_id, collection_name, doc_id)
    }

    /// Funktionale Konstruktor-Methode für Dokument-Scope (deutsche Bezeichnung).
    pub fn fuer_document(tenant_id: TenantId, collection_name: &str, doc_id: DocId) -> Self {
        let storage = StorageScope::Tenant(tenant_id);
        let user_key = [
            format!("__col:{}:\x00", collection_name).as_bytes(),
            doc_id.to_string().as_bytes(),
        ]
        .concat();
        let doc_map_key = [b"__docid:", doc_id.to_string().as_bytes()].concat();
        let rel_prefix = [b"__rel:", doc_id.to_string().as_bytes()].concat();

        let keys = KeyScope {
            explicit: vec![user_key, doc_map_key],
            prefixes: vec![rel_prefix],
        };

        let mut ram = FprRamFootprint::default();
        ram.doc_ids.push(doc_id);
        let ram_hash = ram.compute_hash();

        let mut footprint = Self {
            storage,
            keys,
            hash: [0u8; 32],
            ram,
            ram_hash,
            observed_empty: false,
        };
        footprint.hash = footprint.compute_hash();
        footprint
    }

    /// Funktionale Konstruktor-Methode für Collection-Scope.
    pub fn for_collection(tenant_id: TenantId, collection_name: &str) -> Self {
        Self::fuer_collection(tenant_id, collection_name)
    }

    /// Funktionale Konstruktor-Methode für Collection-Scope (deutsche Bezeichnung).
    pub fn fuer_collection(tenant_id: TenantId, collection_name: &str) -> Self {
        let storage = StorageScope::Tenant(tenant_id);
        let col_data = format!("__col:{}:\x00", collection_name).into_bytes();
        let txt_data = format!("__txt:{}:", collection_name).into_bytes();
        let col_idx = [b"__col_idx:\x00", collection_name.as_bytes()].concat();

        let keys = KeyScope {
            explicit: vec![col_idx],
            prefixes: vec![col_data, txt_data],
        };

        let ram = FprRamFootprint::default();
        let ram_hash = ram.compute_hash();

        let mut footprint = Self {
            storage,
            keys,
            hash: [0u8; 32],
            ram,
            ram_hash,
            observed_empty: false,
        };
        footprint.hash = footprint.compute_hash();
        footprint
    }

    /// Funktionale Konstruktor-Methode für Tenant-Scope.
    pub fn for_tenant(tenant_id: TenantId) -> Self {
        Self::fuer_tenant(tenant_id)
    }

    /// Funktionale Konstruktor-Methode für Tenant-Scope (deutsche Bezeichnung).
    pub fn fuer_tenant(tenant_id: TenantId) -> Self {
        let storage = StorageScope::Tenant(tenant_id);
        let prefixes = vec![
            b"__col:".to_vec(),
            b"__col_idx:".to_vec(),
            b"__txt:".to_vec(),
            b"__graph:".to_vec(),
            b"__rel:".to_vec(),
            b"__docid:".to_vec(),
            b"__tx_intent:".to_vec(),
            b"__kv_spill:".to_vec(),
            b"__session_dag:".to_vec(),
            b"__meta:".to_vec(),
        ];

        let keys = KeyScope {
            explicit: Vec::new(),
            prefixes,
        };

        let ram = FprRamFootprint::default();
        let ram_hash = ram.compute_hash();

        let mut footprint = Self {
            storage,
            keys,
            hash: [0u8; 32],
            ram,
            ram_hash,
            observed_empty: false,
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

    /// Zeroisiert sensible Schlüssel- und Speichermengen.
    pub fn zeroize_contents(&mut self) {
        for k in &mut self.keys.explicit {
            k.zeroize();
        }
        self.keys.explicit.clear();
        for p in &mut self.keys.prefixes {
            p.zeroize();
        }
        self.keys.prefixes.clear();
        self.ram.zeroize_contents();
    }
}

/// Prüft nach "Fail-Closed" Regel: Wenn Handle Daten hatte (`scanned_count > 0`), aber 0 Schlüssel gelöscht wurden, wird ein Fehler zurückgegeben.
pub fn check_nonempty_or_observed_empty(
    scanned_count: usize,
    deleted_count: usize,
) -> Result<(), ContextraError> {
    if scanned_count > 0 && deleted_count == 0 {
        Err(ContextraError::Storage(
            "Fail-closed: handle contained scanned entries but 0 keys were deleted".into(),
        ))
    } else {
        Ok(())
    }
}
