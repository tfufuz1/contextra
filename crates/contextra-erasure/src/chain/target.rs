//! Löschziele nach SPEC §5.1 und ihre kanonische Kodierung.

use contextra_ids::{CollectionId, DocId, SubjectRef, TenantId};

use crate::error::ErasureError;
use crate::vault::slot::SlotType;

/// Art eines Löschziels (SPEC §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetKind {
    /// Einzeldokument (Klasse K, `DocumentDEK`).
    Document,
    /// Betroffener (Klasse P, `SubjectKey`, O(1)).
    Subject,
    /// Collection (`CollectionKEK`, O(1)).
    Collection,
    /// Mandant (`TenantKEK`, O(1)).
    Tenant,
}

/// Löschziel. Die Variante bestimmt, welcher Schlüssel vernichtet wird.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EraseTarget {
    /// Einzelnes Dokument; vernichtet den `DocumentDEK`.
    Document {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: CollectionId,
        /// Dokument.
        doc: DocId,
    },
    /// Betroffener über alle Collections; vernichtet den `SubjectKey`.
    Subject {
        /// Mandant.
        tenant: TenantId,
        /// Betroffenen-Referenz (128 Bit Zufall).
        subject: SubjectRef,
    },
    /// Collection; vernichtet die `CollectionKEK`.
    Collection {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: CollectionId,
    },
    /// Mandant; vernichtet die `TenantKEK`.
    Tenant {
        /// Mandant.
        tenant: TenantId,
    },
}

impl Clone for EraseTarget {
    fn clone(&self) -> Self {
        match self {
            Self::Document {
                tenant,
                collection,
                doc,
            } => Self::Document {
                tenant: *tenant,
                collection: *collection,
                doc: *doc,
            },
            // `SubjectRef` (aus contextra-ids) implementiert kein Clone; die 16 Byte werden kopiert.
            Self::Subject { tenant, subject } => Self::Subject {
                tenant: *tenant,
                subject: SubjectRef(subject.0),
            },
            Self::Collection { tenant, collection } => Self::Collection {
                tenant: *tenant,
                collection: *collection,
            },
            Self::Tenant { tenant } => Self::Tenant { tenant: *tenant },
        }
    }
}

const TAG_DOCUMENT: u8 = 1;
const TAG_SUBJECT: u8 = 2;
const TAG_COLLECTION: u8 = 3;
const TAG_TENANT: u8 = 4;

/// Domain-Separator für die Vault-Tag-Ableitung.
const VAULT_TAG_DOMAIN: &[u8] = b"contextra.erasure.target.v1";

impl EraseTarget {
    /// Art des Ziels.
    pub const fn kind(&self) -> TargetKind {
        match self {
            Self::Document { .. } => TargetKind::Document,
            Self::Subject { .. } => TargetKind::Subject,
            Self::Collection { .. } => TargetKind::Collection,
            Self::Tenant { .. } => TargetKind::Tenant,
        }
    }

    /// Vault-Slot-Ebene, auf der der Schlüssel liegt.
    pub const fn slot_type(&self) -> SlotType {
        match self {
            Self::Document { .. } => SlotType::Document,
            Self::Subject { .. } => SlotType::Subject,
            Self::Collection { .. } => SlotType::Collection,
            Self::Tenant { .. } => SlotType::Tenant,
        }
    }

    /// Kanonische, versionierte Kodierung (Journal, Duplikaterkennung, Vault-Tag).
    pub fn encode(&self) -> Result<Vec<u8>, ErasureError> {
        let mut out = Vec::with_capacity(48);
        match self {
            Self::Document {
                tenant,
                collection,
                doc,
            } => {
                out.push(TAG_DOCUMENT);
                out.extend_from_slice(&tenant.0.to_be_bytes());
                out.extend_from_slice(&collection.0.to_be_bytes());
                let doc_json =
                    serde_json::to_vec(doc).map_err(|e| ErasureError::Json(e.to_string()))?;
                let len = u16::try_from(doc_json.len()).map_err(|_| ErasureError::Corrupt)?;
                out.extend_from_slice(&len.to_be_bytes());
                out.extend_from_slice(&doc_json);
            }
            Self::Subject { tenant, subject } => {
                out.push(TAG_SUBJECT);
                out.extend_from_slice(&tenant.0.to_be_bytes());
                out.extend_from_slice(&subject.0);
            }
            Self::Collection { tenant, collection } => {
                out.push(TAG_COLLECTION);
                out.extend_from_slice(&tenant.0.to_be_bytes());
                out.extend_from_slice(&collection.0.to_be_bytes());
            }
            Self::Tenant { tenant } => {
                out.push(TAG_TENANT);
                out.extend_from_slice(&tenant.0.to_be_bytes());
            }
        }
        Ok(out)
    }

    /// Dekodiert eine mit [`EraseTarget::encode`] erzeugte Kodierung.
    pub fn decode(bytes: &[u8]) -> Result<Self, ErasureError> {
        let mut r = Reader { buf: bytes, pos: 0 };
        let tag = r.u8()?;
        let target = match tag {
            TAG_DOCUMENT => {
                let tenant = TenantId(r.u64()?);
                let collection = CollectionId(r.u64()?);
                let len = usize::from(r.u16()?);
                let doc: DocId = serde_json::from_slice(r.take(len)?)
                    .map_err(|e| ErasureError::Json(e.to_string()))?;
                Self::Document {
                    tenant,
                    collection,
                    doc,
                }
            }
            TAG_SUBJECT => {
                let tenant = TenantId(r.u64()?);
                let mut subject = [0u8; 16];
                subject.copy_from_slice(r.take(16)?);
                Self::Subject {
                    tenant,
                    subject: SubjectRef(subject),
                }
            }
            TAG_COLLECTION => {
                let tenant = TenantId(r.u64()?);
                let collection = CollectionId(r.u64()?);
                Self::Collection { tenant, collection }
            }
            TAG_TENANT => Self::Tenant {
                tenant: TenantId(r.u64()?),
            },
            _ => return Err(ErasureError::Corrupt),
        };
        if r.pos != bytes.len() {
            return Err(ErasureError::Corrupt);
        }
        Ok(target)
    }

    /// 16-Byte-Tag für die Vault-Slot-Suche (domain-separiertes BLAKE3 über die Kodierung).
    pub fn vault_tag(&self) -> Result<[u8; 16], ErasureError> {
        let enc = self.encode()?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(VAULT_TAG_DOMAIN);
        hasher.update(&enc);
        let digest = hasher.finalize();
        let mut tag = [0u8; 16];
        tag.copy_from_slice(digest.as_bytes().get(..16).ok_or(ErasureError::Corrupt)?);
        Ok(tag)
    }
}

/// Lesehilfe ohne Indexzugriff (Lint `indexing_slicing`).
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ErasureError> {
        let end = self.pos.checked_add(n).ok_or(ErasureError::Corrupt)?;
        let slice = self.buf.get(self.pos..end).ok_or(ErasureError::Corrupt)?;
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, ErasureError> {
        Ok(*self.take(1)?.first().ok_or(ErasureError::Corrupt)?)
    }

    fn u16(&mut self) -> Result<u16, ErasureError> {
        let mut a = [0u8; 2];
        a.copy_from_slice(self.take(2)?);
        Ok(u16::from_be_bytes(a))
    }

    fn u64(&mut self) -> Result<u64, ErasureError> {
        let mut a = [0u8; 8];
        a.copy_from_slice(self.take(8)?);
        Ok(u64::from_be_bytes(a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> EraseTarget {
        EraseTarget::Document {
            tenant: TenantId(3),
            collection: CollectionId(7),
            doc: DocId(42),
        }
    }

    #[test]
    fn kodierung_roundtrip_alle_varianten() {
        let targets = [
            doc(),
            EraseTarget::Subject {
                tenant: TenantId(1),
                subject: SubjectRef([9; 16]),
            },
            EraseTarget::Collection {
                tenant: TenantId(1),
                collection: CollectionId(2),
            },
            EraseTarget::Tenant {
                tenant: TenantId(5),
            },
        ];
        for t in targets {
            let enc = t.encode().expect("encode");
            assert_eq!(EraseTarget::decode(&enc).expect("decode"), t);
        }
    }

    #[test]
    fn dekodierung_lehnt_abgeschnittene_und_unbekannte_eingaben_ab() {
        let enc = doc().encode().expect("encode");
        let truncated = enc.get(..enc.len() - 1).expect("slice");
        assert!(EraseTarget::decode(truncated).is_err());
        assert!(EraseTarget::decode(&[0xFF]).is_err());
        assert!(EraseTarget::decode(&[]).is_err());
    }

    #[test]
    fn vault_tag_ist_deterministisch_und_zielabhaengig() {
        let a = doc().vault_tag().expect("tag");
        assert_eq!(a, doc().vault_tag().expect("tag"));
        let other = EraseTarget::Document {
            tenant: TenantId(3),
            collection: CollectionId(7),
            doc: DocId(43),
        };
        assert_ne!(a, other.vault_tag().expect("tag"));
    }

    #[test]
    fn slot_typ_folgt_schluesselebene() {
        assert_eq!(doc().slot_type(), SlotType::Document);
        assert_eq!(
            EraseTarget::Tenant {
                tenant: TenantId(1)
            }
            .slot_type(),
            SlotType::Tenant
        );
    }
}
