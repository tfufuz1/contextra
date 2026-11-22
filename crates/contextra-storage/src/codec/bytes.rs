//! # Bytes Ausschnitt
//!
//! Schlanker `Bytes`-Typ als `Arc<[u8]>`-Ausschnitt (SPEC §7.2)
//! für kopiefreie Rückgaben aus Block-Caches.

use std::ops::Deref;
use std::sync::Arc;

/// Kopiefreier Byte-Ausschnitt auf einem geteilten Datenblock.
#[derive(Clone, PartialEq, Eq)]
pub struct Bytes {
    data: Arc<[u8]>,
    start: usize,
    len: usize,
}

impl Bytes {
    /// Erstellt einen neuen `Bytes`-Ausschnitt aus einem `Arc<[u8]>`.
    pub fn from_arc(data: Arc<[u8]>, start: usize, len: usize) -> Option<Self> {
        let end = start.checked_add(len)?;
        if end <= data.len() {
            Some(Self { data, start, len })
        } else {
            None
        }
    }

    /// Erstellt `Bytes` aus einem vorhandenen Slice durch Klonen.
    pub fn copy_from_slice(slice: &[u8]) -> Self {
        let data: Arc<[u8]> = Arc::from(slice);
        let len = data.len();
        Self { data, start: 0, len }
    }

    /// Liefert einen Teilausschnitt ohne Kopie.
    pub fn slice(&self, range_start: usize, range_len: usize) -> Option<Self> {
        let new_start = self.start.checked_add(range_start)?;
        let end = new_start.checked_add(range_len)?;
        if end <= self.start + self.len {
            Some(Self {
                data: Arc::clone(&self.data),
                start: new_start,
                len: range_len,
            })
        } else {
            None
        }
    }

    /// Liefert die Länge in Bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Prüft, ob der Ausschnitt leer ist.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        let end = self.start + self.len;
        match self.data.get(self.start..end) {
            Some(s) => s,
            None => &[],
        }
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bytes(len={})", self.len)
    }
}
