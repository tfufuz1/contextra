//! Dokument- und Kontext-Referenzen (SPEC 7.2, 7.3, D-70, I-11).
//!
//! Alle Newtypes sind längenbegrenzt, nicht leer und tauchen nie in Fehlermeldungen auf (I-11).

use core::fmt;
use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Error};
use crate::tags::{ClusterTag, DocTag};

/// Maximale Bytelänge für Referenz-Strings / Byte-Identifier.
pub const MAX_REF_LEN: usize = 256;

macro_rules! define_ref_newtype {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(Vec<u8>);

        impl $name {
            /// Erzeugt eine geprüfte Instanz. Verstoß führt zu `ConfigError` (I-11).
            pub fn try_new(bytes: Vec<u8>) -> Result<Self, Error> {
                if bytes.is_empty() {
                    return Err(Error::Config(ConfigError::EmptyIdentifier));
                }
                if bytes.len() > MAX_REF_LEN {
                    return Err(Error::Config(ConfigError::IdentifierTooLong));
                }
                Ok(Self(bytes))
            }

            /// Liefert die Bytes als Slice.
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            /// Verbraucht den Wrapper und liefert den Vektor.
            #[inline]
            pub fn into_inner(self) -> Vec<u8> {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // Redigierte Ausgabe laut I-11 / HR-4
                write!(f, "{}([REDACTED {} bytes])", stringify!($name), self.0.len())
            }
        }
    };
}

define_ref_newtype!(TenantRef, "Mandanten-Referenz (SPEC 7.3).");
define_ref_newtype!(CollectionRef, "Collection-Referenz (SPEC 7.3).");
define_ref_newtype!(SubjectRef, "Betroffenen-Referenz (SPEC 7.2, 7.3).");
define_ref_newtype!(OperatorRef, "Opake Betreiberkennung (SPEC 7.3).");
define_ref_newtype!(DocKey, "Dokumentenschlüssel (SPEC 7.2).");

/// Eindeutige Dokumentenadresse: Betroffener ist unveränderlicher Teil der Identität (SPEC 7.2, D-70).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct DocRef {
    /// Optionaler Betroffener (Subject).
    pub subject: Option<SubjectRef>,
    /// Eindeutiger Dokumentenschlüssel innerhalb des Clusters.
    pub key: DocKey,
}

/// Opaker Dokumenten-Handle als Suchtreffer (SPEC 7.2, D-70).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocHandle {
    cluster: ClusterTag,
    doc: DocTag,
}

impl DocHandle {
    /// Erzeugt einen neuen `DocHandle`.
    #[inline]
    pub const fn new(cluster: ClusterTag, doc: DocTag) -> Self {
        Self { cluster, doc }
    }

    /// Liefert das Cluster-Tag.
    #[inline]
    pub const fn cluster(&self) -> ClusterTag {
        self.cluster
    }

    /// Liefert das Dokumenten-Tag.
    #[inline]
    pub const fn doc(&self) -> DocTag {
        self.doc
    }
}

impl fmt::Debug for DocHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocHandle({:?}, {:?})", self.cluster, self.doc)
    }
}

/// Mandanten-Kontext für alle mandantenspezifischen Aufrufe (SPEC D-07, VETO-F10).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct TenantContext {
    tenant: TenantRef,
}

impl TenantContext {
    /// Erzeugt einen neuen `TenantContext`.
    pub fn new(tenant: TenantRef) -> Self {
        Self { tenant }
    }

    /// Liefert die Mandanten-Referenz.
    pub fn tenant(&self) -> &TenantRef {
        &self.tenant
    }
}
