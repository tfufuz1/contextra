//! Ziel- und Bereichsbeschreibungen für Bereinigung und Verifikation.

#![forbid(unsafe_code)]

use contextra_types::{CollectionId, DocId, TenantId};
use serde::{Deserialize, Serialize};

/// Zielbeschreibung des Scans inklusive kryptographischem Fingerabdruck.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScanTarget {
    /// Bezeichner des Scan-Ziels (z. B. Tenant, Collection, Präfix).
    pub target_id: String,
    /// BLAKE3 Hash des Scan-Ziels.
    pub target_hash: [u8; 32],
}

impl ScanTarget {
    /// Erzeugt ein neues ScanTarget aus Bezeichner und Hash.
    pub fn new(target_id: impl Into<String>, target_hash: [u8; 32]) -> Self {
        Self {
            target_id: target_id.into(),
            target_hash,
        }
    }

    /// Erzeugt ein ScanTarget aus einem KeyScope.
    pub fn from_scope(scope: &KeyScope) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(scope.to_canonical_string().as_bytes());
        Self {
            target_id: scope.to_canonical_string(),
            target_hash: *hasher.finalize().as_bytes(),
        }
    }
}

/// Kanonischer Bereichsdeskriptor für physische Bereinigungs- und Scanoperationen.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyScope {
    /// Vollständiger Mandanten-Scope.
    Tenant {
        /// Mandanten-Identifikator.
        tenant: TenantId,
    },
    /// Collection-Scope innerhalb eines Mandanten.
    Collection {
        /// Mandanten-Identifikator.
        tenant: TenantId,
        /// Collection-Identifikator.
        collection: CollectionId,
    },
    /// Einzelnes Dokument innerhalb einer Collection.
    Document {
        /// Mandanten-Identifikator.
        tenant: TenantId,
        /// Collection-Identifikator.
        collection: CollectionId,
        /// Dokumenten-Identifikator.
        doc_id: DocId,
    },
    /// Benutzerdefinierter Byte-Präfix oder Schlüsselbereich.
    Prefix {
        /// Mandanten-Identifikator.
        tenant: TenantId,
        /// Präfix-Bytes.
        prefix: Vec<u8>,
    },
}

impl KeyScope {
    /// Erzeugt einen kanonischen String zur eindeutigen Identifikation des Scopes.
    pub fn to_canonical_string(&self) -> String {
        match self {
            Self::Tenant { tenant } => format!("tenant:{}", tenant.as_u64()),
            Self::Collection { tenant, collection } => {
                format!("coll:{}:{}", tenant.as_u64(), collection.inner())
            }
            Self::Document {
                tenant,
                collection,
                doc_id,
            } => {
                format!(
                    "doc:{}:{}:{}",
                    tenant.as_u64(),
                    collection.inner(),
                    doc_id.inner()
                )
            }
            Self::Prefix { tenant, prefix } => {
                format!("prefix:{}:{:02x?}", tenant.as_u64(), prefix)
            }
        }
    }

    /// Gibt den zugeordneten Mandanten zurück.
    pub fn tenant(&self) -> TenantId {
        match self {
            Self::Tenant { tenant } => *tenant,
            Self::Collection { tenant, .. } => *tenant,
            Self::Document { tenant, .. } => *tenant,
            Self::Prefix { tenant, .. } => *tenant,
        }
    }
}
