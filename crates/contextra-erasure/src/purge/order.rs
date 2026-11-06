//! Schichtreihenfolge und Priorisierung beim physischen Purge (SPEC §11.2, D-39, D-61, ER-20).
//!
//! Feste Tabelle `LayerId → Priorität` gemäß ER-20:
//!   a) Snapshots widerrufen
//!   b) Textsegment-Rewrite
//!   c) Memtable-Drain, Flush, WAL-Segmente entfernen
//!   d) SSTable-Rewrite, F-Slots nullen, unlink
//!   e) Vektor-Kantenreparatur
//!   f) D-Slots der Zielmenge entfernen
//!   g) Cache-Eviction

/// Priorität einer Schicht beim Purge (niedrigere Zahl = früher ausgeführt).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PurgePriority(pub u8);

impl PurgePriority {
    /// Snapshot-Widerruf (höchste Priorität).
    pub const SNAPSHOT_REVOKE: Self = Self(0);
    /// Textsegment-Rewrite.
    pub const TEXT_REWRITE: Self = Self(1);
    /// WAL-Segmente und Memtable.
    pub const WAL_FLUSH: Self = Self(2);
    /// SSTable-Rewrite.
    pub const SSTABLE_REWRITE: Self = Self(3);
    /// Vektor-Kantenreparatur.
    pub const VECTOR_REPAIR: Self = Self(4);
    /// Slot-Bereinigung (D-Slots).
    pub const SLOT_CLEANUP: Self = Self(5);
    /// Cache-Eviction (niedrigste Priorität).
    pub const CACHE_EVICT: Self = Self(6);
}

/// Zuordnung einer Schicht zu ihrer Purge-Priorität.
#[derive(Clone, Debug)]
pub struct LayerPurgeOrder {
    /// Schicht-ID (16 Bytes).
    pub layer_id: [u8; 16],
    /// Priorität.
    pub priority: PurgePriority,
}

impl LayerPurgeOrder {
    /// Erstellt eine neue Zuordnung.
    pub fn new(layer_id: [u8; 16], priority: PurgePriority) -> Self {
        Self { layer_id, priority }
    }
}

/// Sortiert Schichten nach Purge-Priorität (niedrigste Zahl zuerst).
pub fn sort_by_priority(mut layers: Vec<LayerPurgeOrder>) -> Vec<LayerPurgeOrder> {
    layers.sort_by_key(|l| l.priority);
    layers
}
