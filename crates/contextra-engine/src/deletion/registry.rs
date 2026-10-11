// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: KeyFamilyRegistry & Schlüssel-Präfix-Familien Management (WP-1.1, C1).
// INVARIANTEN: INV-D1: Enumeration vollständig (Footprint(scope) aus Registry, kein Ad-hoc Scan).

use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_types::{CollectionId, DocId, TenantId};
use serde::{Deserialize, Serialize};

use crate::deletion::footprint::{ErasureFootprint, KeyScope, StorageScope};

/// Löscheinheit für eine Key-Familie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EraseUnit {
    PerDocument,
    PerCollection,
    PerTenant,
    Both,
}

/// Repräsentiert einen Eintrag in der `KEYFAMILIES.toml` Registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyFamilyEntry {
    pub name: &'static str,
    pub prefix_pattern: &'static str,
    pub owner_crate: &'static str,
    pub erase_unit: EraseUnit,
    pub scanner: &'static str,
    pub contains_identifier: bool,
}

/// Statische Registry aller bekannten Key-Familien im System (synchron zu KEYFAMILIES.toml).
pub static ALL_KEY_FAMILIES: &[KeyFamilyEntry] = &[
    KeyFamilyEntry {
        name: "CollectionDocData",
        prefix_pattern: "__col:{collection_name}:\x00",
        owner_crate: "contextra-engine",
        erase_unit: EraseUnit::Both,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "CollectionIndex",
        prefix_pattern: "__col_idx:\x00{collection_name}",
        owner_crate: "contextra-engine",
        erase_unit: EraseUnit::PerCollection,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "TextIndex",
        prefix_pattern: "__txt:{collection_name}:",
        owner_crate: "contextra-engine",
        erase_unit: EraseUnit::Both,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "Relations",
        prefix_pattern: "__rel:",
        owner_crate: "contextra-engine",
        erase_unit: EraseUnit::PerDocument,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "DocIdMapping",
        prefix_pattern: "__docid:",
        owner_crate: "contextra-engine",
        erase_unit: EraseUnit::PerDocument,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "GraphData",
        prefix_pattern: "__graph:",
        owner_crate: "contextra-graph",
        erase_unit: EraseUnit::Both,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "TxIntent",
        prefix_pattern: "__tx_intent:",
        owner_crate: "contextra-engine",
        erase_unit: EraseUnit::PerTenant,
        scanner: "TODO",
        contains_identifier: false,
    },
    KeyFamilyEntry {
        name: "KvCache",
        prefix_pattern: "__kv_spill:",
        owner_crate: "contextra-kvcache",
        erase_unit: EraseUnit::PerTenant,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "SessionDag",
        prefix_pattern: "__session_dag:",
        owner_crate: "contextra-graph",
        erase_unit: EraseUnit::PerTenant,
        scanner: "TODO",
        contains_identifier: true,
    },
    KeyFamilyEntry {
        name: "MaintenanceIntent",
        prefix_pattern: "__maintenance_intent:",
        owner_crate: "contextra-cognition",
        erase_unit: EraseUnit::PerTenant,
        scanner: "TODO",
        contains_identifier: false,
    },
];

/// Referenz auf einen Deletion-Scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionScopeRef<'a> {
    Document {
        tenant_id: TenantId,
        collection_id: CollectionId,
        collection_name: &'a str,
        doc_id: DocId,
    },
    Collection {
        tenant_id: TenantId,
        collection_id: CollectionId,
        collection_name: &'a str,
    },
    Tenant {
        tenant_id: TenantId,
    },
}

/// Spezifiziert das Ziel einer Löschung (Eigener Eigentümer-Typ für Purge-Scopes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurgeScopeTarget {
    Document {
        tenant_id: TenantId,
        collection_id: CollectionId,
        collection_name: String,
        doc_id: DocId,
    },
    Collection {
        tenant_id: TenantId,
        collection_id: CollectionId,
        collection_name: String,
    },
    Tenant {
        tenant_id: TenantId,
    },
}

impl<'a> From<&'a PurgeScopeTarget> for DeletionScopeRef<'a> {
    fn from(target: &'a PurgeScopeTarget) -> Self {
        match target {
            PurgeScopeTarget::Document {
                tenant_id,
                collection_id,
                ref collection_name,
                doc_id,
            } => DeletionScopeRef::Document {
                tenant_id: *tenant_id,
                collection_id: *collection_id,
                collection_name,
                doc_id: *doc_id,
            },
            PurgeScopeTarget::Collection {
                tenant_id,
                collection_id,
                ref collection_name,
            } => DeletionScopeRef::Collection {
                tenant_id: *tenant_id,
                collection_id: *collection_id,
                collection_name,
            },
            PurgeScopeTarget::Tenant { tenant_id } => DeletionScopeRef::Tenant {
                tenant_id: *tenant_id,
            },
        }
    }
}

/// Repräsentiert die vollständige Menge an Schlüsselpräfixen und Schichten für einen Löschbereich (INV-D1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Footprint {
    pub target: PurgeScopeTarget,
    pub prefixes: Vec<Vec<u8>>,
    pub exact_keys: Vec<Vec<u8>>,
    pub affected_layers: Vec<DeletionLayer>,
}

impl Footprint {
    /// Prüft, ob ein gegebener physischer Storage-Schlüssel zu diesem Footprint gehört.
    pub fn matches(&self, key: &[u8]) -> bool {
        self.exact_keys.iter().any(|k| k.as_slice() == key)
            || self.prefixes.iter().any(|p| key.starts_with(p.as_slice()))
    }

    /// Konvertiert diesen legacy Footprint in einen Garantiemodell-`ErasureFootprint`.
    pub fn to_erasure_footprint(&self) -> ErasureFootprint {
        let storage = match &self.target {
            PurgeScopeTarget::Tenant { tenant_id }
            | PurgeScopeTarget::Collection { tenant_id, .. }
            | PurgeScopeTarget::Document { tenant_id, .. } => StorageScope::Tenant(*tenant_id),
        };

        let keys = KeyScope {
            explicit: self.exact_keys.clone(),
            prefixes: self.prefixes.clone(),
        };

        ErasureFootprint::new(storage, keys)
    }
}

/// Zentrale Registry aller Key-Familien zur Beseitigung von Präfix-Drift.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyFamilyRegistry;

impl KeyFamilyRegistry {
    /// Erstellt eine neue Instanz der `KeyFamilyRegistry`.
    pub fn new() -> Self {
        Self
    }

    /// Liefert alle in der Registry erfassten Key-Familien.
    pub fn all_families(&self) -> &'static [KeyFamilyEntry] {
        ALL_KEY_FAMILIES
    }

    /// Ermittelt alle für einen gegbenen DeletionScope relevanten Key-Familien.
    pub fn enumerate(&self, scope: &DeletionScopeRef) -> Vec<KeyFamilyEntry> {
        ALL_KEY_FAMILIES
            .iter()
            .filter(|entry| match scope {
                DeletionScopeRef::Document { .. } => {
                    matches!(entry.erase_unit, EraseUnit::PerDocument | EraseUnit::Both)
                }
                DeletionScopeRef::Collection { .. } => {
                    matches!(entry.erase_unit, EraseUnit::PerCollection | EraseUnit::Both)
                }
                DeletionScopeRef::Tenant { .. } => true,
            })
            .cloned()
            .collect()
    }

    /// Zentraler Encoder zur konsistenten Präfix- und Key-Generierung.
    pub fn encode_prefix(&self, family: &str, params: &[&str]) -> Option<Vec<u8>> {
        match family {
            "CollectionDocData" | "__col:" => {
                if params.len() == 1 {
                    Some(format!("__col:{}:\x00", params[0]).into_bytes())
                } else if params.len() >= 2 {
                    Some(format!("__col:{}:\x00{}", params[0], params[1]).into_bytes())
                } else {
                    None
                }
            }
            "CollectionIndex" | "__col_idx:" => {
                if !params.is_empty() {
                    Some([b"__col_idx:\x00", params[0].as_bytes()].concat())
                } else {
                    None
                }
            }
            "TextIndex" | "__txt:" => {
                if !params.is_empty() {
                    Some(format!("__txt:{}:", params[0]).into_bytes())
                } else {
                    None
                }
            }
            "Relations" | "__rel:" => {
                if params.is_empty() {
                    Some(b"__rel:".to_vec())
                } else {
                    Some(format!("__rel:{}", params.join(":")).into_bytes())
                }
            }
            "DocIdMapping" | "__docid:" => {
                if params.is_empty() {
                    Some(b"__docid:".to_vec())
                } else {
                    Some(format!("__docid:{}", params.join(":")).into_bytes())
                }
            }
            "GraphData" | "__graph:" => {
                if params.is_empty() {
                    Some(b"__graph:".to_vec())
                } else {
                    Some(format!("__graph:{}", params.join(":")).into_bytes())
                }
            }
            "TxIntent" | "__tx_intent:" => Some(b"__tx_intent:".to_vec()),
            "KvCache" | "__kv_spill:" => Some(b"__kv_spill:".to_vec()),
            "SessionDag" | "__session_dag:" => Some(b"__session_dag:".to_vec()),
            "MaintenanceIntent" | "__maintenance_intent:" => {
                Some(b"__maintenance_intent:".to_vec())
            }
            _ => None,
        }
    }

    // Statische Helper-Methoden zur Abwärtskompatibilität

    pub fn collection_data_prefix(name: &str) -> Vec<u8> {
        format!("__col:{}:\x00", name).into_bytes()
    }

    pub fn collection_idx_key(name: &str) -> Vec<u8> {
        [b"__col_idx:\x00", name.as_bytes()].concat()
    }

    pub fn text_data_prefix(name: &str) -> Vec<u8> {
        format!("__txt:{}:", name).into_bytes()
    }

    pub fn graph_data_prefix() -> &'static [u8] {
        b"__graph:"
    }

    pub fn relations_prefix() -> &'static [u8] {
        b"__rel:"
    }

    pub fn docid_prefix() -> &'static [u8] {
        b"__docid:"
    }

    pub fn tx_intent_prefix() -> &'static [u8] {
        b"__tx_intent:"
    }

    /// Ermittelt den vollständigen Footprint für ein gegebenes Löschziel (INV-D1).
    pub fn enumerate_legacy(target: PurgeScopeTarget) -> Footprint {
        match target {
            PurgeScopeTarget::Collection {
                ref collection_name,
                ..
            } => {
                let mut prefixes = Vec::new();
                let mut exact_keys = Vec::new();

                prefixes.push(Self::collection_data_prefix(collection_name));
                prefixes.push(Self::text_data_prefix(collection_name));
                exact_keys.push(Self::collection_idx_key(collection_name));

                let affected_layers = vec![
                    DeletionLayer::LsmMemtable,
                    DeletionLayer::SsTableAllLevels,
                    DeletionLayer::WalAllSegments { seq_after: 0 },
                    DeletionLayer::HnswIndex,
                    DeletionLayer::TextIndex,
                    DeletionLayer::CsrGraph,
                    DeletionLayer::KvCacheSegments,
                ];

                Footprint {
                    target,
                    prefixes,
                    exact_keys,
                    affected_layers,
                }
            }
            PurgeScopeTarget::Document {
                ref collection_name,
                doc_id,
                ..
            } => {
                let exact_doc_key = [
                    Self::collection_data_prefix(collection_name),
                    doc_id.to_string().into_bytes(),
                ]
                .concat();

                let affected_layers = vec![
                    DeletionLayer::LsmMemtable,
                    DeletionLayer::SsTableAllLevels,
                    DeletionLayer::WalAllSegments { seq_after: 0 },
                    DeletionLayer::HnswIndex,
                    DeletionLayer::TextIndex,
                ];

                Footprint {
                    target,
                    prefixes: Vec::new(),
                    exact_keys: vec![exact_doc_key],
                    affected_layers,
                }
            }
            PurgeScopeTarget::Tenant { .. } => {
                let prefixes = vec![
                    b"__col:".to_vec(),
                    b"__col_idx:".to_vec(),
                    b"__txt:".to_vec(),
                    b"__graph:".to_vec(),
                    b"__rel:".to_vec(),
                    b"__docid:".to_vec(),
                    b"__tx_intent:".to_vec(),
                ];

                let affected_layers = vec![
                    DeletionLayer::LsmMemtable,
                    DeletionLayer::SsTableAllLevels,
                    DeletionLayer::WalAllSegments { seq_after: 0 },
                    DeletionLayer::HnswIndex,
                    DeletionLayer::TextIndex,
                    DeletionLayer::CsrGraph,
                    DeletionLayer::KvCacheSegments,
                    DeletionLayer::EmbeddingCache,
                ];

                Footprint {
                    target,
                    prefixes,
                    exact_keys: Vec::new(),
                    affected_layers,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_footprint_matching() {
        let target = PurgeScopeTarget::Collection {
            tenant_id: TenantId::try_new(1).unwrap(),
            collection_id: CollectionId::new(42),
            collection_name: "test_col".to_string(),
        };

        let footprint = KeyFamilyRegistry::enumerate_legacy(target);

        let doc_key = b"__col:test_col:\x00doc_123";
        assert!(footprint.matches(doc_key));

        let txt_key = b"__txt:test_col:terms:hello";
        assert!(footprint.matches(txt_key));

        let idx_key = b"__col_idx:\x00test_col";
        assert!(footprint.matches(idx_key));

        let other_key = b"__col:other_col:\x00doc_123";
        assert!(!footprint.matches(other_key));
    }
}
