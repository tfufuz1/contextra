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
        let id_str = doc_id.to_string();

        let (explicit, prefixes) = if collection_name == "default" {
            let user_key = id_str.as_bytes().to_vec();
            let doc_map_key = [b"__docid:", id_str.as_bytes()].concat();
            let rel_prefix = [b"__rel:", id_str.as_bytes()].concat();
            let txt_dl_prefix = [b"__txt:default:dl:", id_str.as_bytes()].concat();
            let txt_fw_prefix = [b"__txt:default:fw:", id_str.as_bytes()].concat();
            let txt_tbs_prefix = [b"__txt:default:tbs:", id_str.as_bytes()].concat();

            (
                vec![user_key, doc_map_key],
                vec![rel_prefix, txt_dl_prefix, txt_fw_prefix, txt_tbs_prefix],
            )
        } else {
            let user_key = format!("__col:{}:\x00{}", collection_name, id_str).into_bytes();
            let doc_map_typed = format!("__col:{}:\x01{}", collection_name, id_str).into_bytes();
            let doc_map_legacy = [b"__docid:", id_str.as_bytes()].concat();

            let rel_typed_prefix = format!("__col:{}:\x02{}", collection_name, id_str).into_bytes();
            let rel_legacy_prefix = [b"__rel:", id_str.as_bytes()].concat();
            let txt_dl_prefix = format!("__txt:{}:dl:{}", collection_name, id_str).into_bytes();
            let txt_fw_prefix = format!("__txt:{}:fw:{}", collection_name, id_str).into_bytes();
            let txt_tbs_prefix = format!("__txt:{}:tbs:{}", collection_name, id_str).into_bytes();

            (
                vec![user_key, doc_map_typed, doc_map_legacy],
                vec![
                    rel_typed_prefix,
                    rel_legacy_prefix,
                    txt_dl_prefix,
                    txt_fw_prefix,
                    txt_tbs_prefix,
                ],
            )
        };

        let keys = KeyScope { explicit, prefixes };

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
            b"__maintenance_intent:".to_vec(),
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

    /// Prüft, ob ein gegebener physischer oder logischer Storage-Schlüssel zu diesem Footprint gehört.
    pub fn matches(&self, key: &[u8]) -> bool {
        if self.keys.explicit.iter().any(|k| k.as_slice() == key)
            || self.keys.prefixes.iter().any(|p| key.starts_with(p.as_slice()))
        {
            return true;
        }

        for doc_id in &self.ram.doc_ids {
            let id_bytes = doc_id.to_string().into_bytes();
            if key.starts_with(b"__txt:")
                || key.starts_with(b"__col:")
                || key.starts_with(b"__rel:")
                || key.starts_with(b"__docid:")
            {
                if key.ends_with(&id_bytes) {
                    return true;
                }
                let id_colon = [id_bytes.as_slice(), b":"].concat();
                let colon_id = [b":", id_bytes.as_slice()].concat();
                if key.windows(id_colon.len()).any(|w| w == id_colon.as_slice())
                    || key.windows(colon_id.len()).any(|w| w == colon_id.as_slice())
                {
                    return true;
                }
            }
        }

        false
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
