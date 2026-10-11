// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: KeyFamilyRegistry & Footprint Modell (WP-1.1, C1).
// INVARIANTEN: INV-D1: Enumeration vollständig (Footprint(scope) aus Registry, kein Ad-hoc Scan).

use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_types::{CollectionId, DocId, TenantId};
use serde::{Deserialize, Serialize};

/// Identifikator einer Key-Familie im Storage-System.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyFamily {
    /// Dokumentdaten in Collection: `__col:{col_name}:\x00{doc_id}`
    CollectionDocData,
    /// Collection-Index-Metadaten: `__col_idx:\x00{col_name}`
    CollectionIndex,
    /// Text-Inverted-Index: `__txt:{col_name}:\x00...`
    TextIndex,
    /// Graph-Knoten, Kanten und Communities: `__graph:...`
    GraphData,
    /// Relationale Zuordnungen / Fremdschlüssel: `__rel:...`
    Relations,
    /// Interne DocId-Mappings: `__docid:...`
    DocIdMapping,
    /// Transaktions-Intents: `__tx_intent:...`
    TxIntent,
    /// KV-Cache-Einträge: `__kv_spill:...`
    KvCache,
    /// System-Metadaten: `__meta:...`
    SystemMeta,
}

/// Spezifiziert den Zielbereich einer Löschung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurgeScopeTarget {
    /// Einzeldokument innerhalb einer Collection
    Document {
        tenant_id: TenantId,
        collection_id: CollectionId,
        collection_name: String,
        doc_id: DocId,
    },
    /// Gesamte Collection eines Mandanten
    Collection {
        tenant_id: TenantId,
        collection_id: CollectionId,
        collection_name: String,
    },
    /// Vollständiger Mandant
    Tenant {
        tenant_id: TenantId,
    },
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
}

/// Zentrale Registry aller Key-Familien zur Beseitigung von Präfix-Drift.
pub struct KeyFamilyRegistry;

impl KeyFamilyRegistry {
    /// Liefert das präzise Präfix für Dokumentendaten einer Collection: `__col:{name}:\x00`
    pub fn collection_data_prefix(name: &str) -> Vec<u8> {
        format!("__col:{}:\x00", name).into_bytes()
    }

    /// Liefert den exakten Schlüssel für Collection-Index-Metadaten: `__col_idx:\x00{name}`
    pub fn collection_idx_key(name: &str) -> Vec<u8> {
        [b"__col_idx:\x00", name.as_bytes()].concat()
    }

    /// Liefert das präzise Präfix für den Text-Index einer Collection: `__txt:{name}:`
    pub fn text_data_prefix(name: &str) -> Vec<u8> {
        format!("__txt:{}:", name).into_bytes()
    }

    /// Liefert das Präfix für Graph-Daten: `__graph:`
    pub fn graph_data_prefix() -> &'static [u8] {
        b"__graph:"
    }

    /// Liefert das Präfix für relationale Verknüpfungen: `__rel:`
    pub fn relations_prefix() -> &'static [u8] {
        b"__rel:"
    }

    /// Liefert das Präfix für DocId-Mappings: `__docid:`
    pub fn docid_prefix() -> &'static [u8] {
        b"__docid:"
    }

    /// Liefert das Präfix für Transaktions-Intents: `__tx_intent:`
    pub fn tx_intent_prefix() -> &'static [u8] {
        b"__tx_intent:"
    }

    /// Ermittelt den vollständigen Footprint für ein gegebenes Löschziel (INV-D1).
    pub fn enumerate(target: PurgeScopeTarget) -> Footprint {
        match target {
            PurgeScopeTarget::Collection {
                ref collection_name,
                ..
            } => {
                let mut prefixes = Vec::new();
                let mut exact_keys = Vec::new();

                // Collection doc data (mit \x00 Trenner)
                prefixes.push(Self::collection_data_prefix(collection_name));
                // Text-Index Daten
                prefixes.push(Self::text_data_prefix(collection_name));
                // Collection-Metadaten-Index
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
                // Ein Mandant erstreckt sich über alle Key-Familien innerhalb seines Scopes
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

        let footprint = KeyFamilyRegistry::enumerate(target);

        // Matched Collection doc key with \x00
        let doc_key = b"__col:test_col:\x00doc_123";
        assert!(footprint.matches(doc_key));

        // Matched text index
        let txt_key = b"__txt:test_col:terms:hello";
        assert!(footprint.matches(txt_key));

        // Matched exact col idx key
        let idx_key = b"__col_idx:\x00test_col";
        assert!(footprint.matches(idx_key));

        // Does NOT match other collection
        let other_key = b"__col:other_col:\x00doc_123";
        assert!(!footprint.matches(other_key));
    }
}
