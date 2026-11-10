//! # Key-Value Segmentspeicher für contextra-store (Ring 1)
//!
//! Reiner KV-Segmentspeicher ohne Krypto-/Shredding-/Vernichtungssemantik.
//! Vernichtungs- und Nachweislogik liegt außerhalb des Storage-Kerns in `contextra-erasure`.

use contextra_types::Result;

/// Neutrales Segment eines KV-Speichers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvSegment {
    /// Eindeutige Segment-ID.
    pub id: u64,
    /// Nutzdaten des Segments.
    pub data: Vec<u8>,
}

impl KvSegment {
    /// Erzeugt ein neues KV-Segment.
    #[must_use]
    pub fn new(id: u64, data: Vec<u8>) -> Self {
        Self { id, data }
    }
}

/// Speicherbereich für KV-Segmente.
#[derive(Debug, Default)]
pub struct KvSegmentStore {
    segments: std::collections::BTreeMap<u64, KvSegment>,
}

impl KvSegmentStore {
    /// Erzeugt einen neuen leeren Segmentspeicher.
    #[must_use]
    pub fn new() -> Self {
        Self {
            segments: std::collections::BTreeMap::new(),
        }
    }

    /// Speichert ein Segment.
    pub fn put(&mut self, segment: KvSegment) -> Result<()> {
        self.segments.insert(segment.id, segment);
        Ok(())
    }

    /// Liest ein Segment anhand seiner ID.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&KvSegment> {
        self.segments.get(&id)
    }

    /// Entfernt ein Segment anhand seiner ID (reine neutrale Speicheroperation).
    pub fn remove(&mut self, id: u64) -> Option<KvSegment> {
        self.segments.remove(&id)
    }
}
