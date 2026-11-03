//! Tags und TagPreimage (SPEC 9.3, D-21, HR-4, TY-06, TY-07).
//!
//! Alle Tags sind `[u8; 16]`-Newtypes mit Eq/Ord/Hash/Copy.
//! Debug zeigt höchstens 4 Byte Hex zur Vermeidung von Informationsabfluss (HR-4).

use core::fmt;
use serde::{Deserialize, Serialize};

macro_rules! define_tag {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name([u8; 16]);

        impl $name {
            /// Erzeugt ein neues Tag aus rohen 16 Bytes.
            #[inline]
            pub const fn new(bytes: [u8; 16]) -> Self {
                Self(bytes)
            }

            /// Liefert die rohen 16 Bytes als Slice.
            #[inline]
            pub const fn as_bytes(&self) -> &[u8; 16] {
                &self.0
            }

            /// Liefert das Tag als Bytezugriff.
            #[inline]
            pub const fn to_bytes(self) -> [u8; 16] {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // Höchstens 4 Byte Hex laut HR-4
                write!(
                    f,
                    "{}({:02x}{:02x}{:02x}{:02x}..)",
                    stringify!($name),
                    self.0[0],
                    self.0[1],
                    self.0[2],
                    self.0[3]
                )
            }
        }
    };
}

define_tag!(TenantTag, "16-Byte Mandanten-Tag (SPEC D-21).");
define_tag!(CollTag, "16-Byte Collection-Tag (SPEC D-21).");
define_tag!(SubjectTag, "16-Byte Betroffenen-Tag (SPEC D-21).");
define_tag!(DocTag, "16-Byte Dokument-Tag (SPEC D-21).");
define_tag!(TermTag, "16-Byte Term-Tag (SPEC D-21).");
define_tag!(NameTag, "16-Byte Namens-Tag für Kataloge (SPEC D-21, 9.1).");

/// 16-Byte Cluster-Tag für Dokumenten-Clustering (SPEC D-70, TY-06).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClusterTag([u8; 16]);

impl ClusterTag {
    /// Erzeugt ein ClusterTag aus 16 rohen Bytes.
    #[inline]
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes des ClusterTags.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// cluster_tag = subject_tag bei zugeordneten Dokumenten (D-70).
    #[inline]
    pub const fn from_subject(s: SubjectTag) -> Self {
        Self(*s.as_bytes())
    }

    /// Unzugeordnetes Dokument: 0x00¹² ‖ doc_tag[0..4] (D-70, TY-06).
    #[inline]
    pub fn unassigned(d: DocTag) -> Self {
        let mut b = [0u8; 16];
        let doc_bytes = d.as_bytes();
        b[12] = doc_bytes[0];
        b[13] = doc_bytes[1];
        b[14] = doc_bytes[2];
        b[15] = doc_bytes[3];
        Self(b)
    }
}

impl fmt::Debug for ClusterTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClusterTag({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// Längenpräfigierte Vorform zur Tag-Bildung (SPEC 9.3, TY-07).
/// Verhindert Kollisionen durch unterschiedliche Feldaufteilungen: `label ‖ (u32 len ‖ feld)*`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TagPreimage(Vec<u8>);

impl TagPreimage {
    /// Erzeugt eine neue Vorform mit einem Domain-Separation-Label.
    pub fn new(label: &'static [u8]) -> Self {
        let mut buf = Vec::with_capacity(label.len() + 32);
        buf.extend_from_slice(label);
        Self(buf)
    }

    /// Fügt ein Feld mit vorangestellter 32-Bit-Länge (big-endian) hinzu (TY-07).
    pub fn field(mut self, f: &[u8]) -> Self {
        let len = f.len() as u32;
        self.0.extend_from_slice(&len.to_be_bytes());
        self.0.extend_from_slice(f);
        self
    }

    /// Liefert die serialisierten Bytes der Vorform.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
