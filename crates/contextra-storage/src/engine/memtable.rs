//! # Memtable: In-Memory Schreibpuffer
//!
//! Normative Referenzen: SPEC §10.1, §6.2, §16.1 (INV-LOCK-1), Hypothese H-S7.
//! Keine Integer-Casts (M-02 = 0).

use std::collections::BTreeMap;
use crate::codec::tag::{InternalKey, Tag, TagKind};
use crate::sync::{AtomicU64, AtomicUsize, Ordering, RankedRwLock};

/// Maximale Anzahl an Operationen je Schreibsperre (H-S7).
pub const MEMTABLE_SUBBATCH_LIMIT: usize = 64;

/// Slot für einen Memtable-Eintrag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    /// Header-Bytes (je Space feste Länge).
    pub header: Vec<u8>,
    /// Körper-Bytes.
    pub body: Vec<u8>,
    /// End-Schlüssel für Bereichsoperationen (RangeDelete / EraseRange).
    pub end_key: Option<Vec<u8>>,
}

impl Slot {
    /// Erstellt einen Slot für ein normales `Put`.
    pub fn put(header: Vec<u8>, body: Vec<u8>) -> Self {
        Self {
            header,
            body,
            end_key: None,
        }
    }

    /// Erstellt einen Slot für ein punktuelles `Delete`.
    pub fn delete() -> Self {
        Self {
            header: Vec::new(),
            body: Vec::new(),
            end_key: None,
        }
    }

    /// Erstellt einen Slot für ein Bereichslöschen (`RangeDelete`).
    pub fn range_delete(end_key: Option<Vec<u8>>) -> Self {
        Self {
            header: Vec::new(),
            body: Vec::new(),
            end_key,
        }
    }

    /// Erstellt einen Slot für physisches Bereichslöschen (`EraseRange`).
    pub fn erase_range(end_key: Option<Vec<u8>>) -> Self {
        Self {
            header: Vec::new(),
            body: Vec::new(),
            end_key,
        }
    }

    /// Berechnet die belegten Bytes (Schlüssel, Header, Körper, End-Schlüssel) für `memtable_bytes`.
    pub fn byte_len(&self, user_key_len: usize) -> usize {
        let mut total = user_key_len;
        total = total.saturating_add(self.header.len());
        total = total.saturating_add(self.body.len());
        if let Some(ref end) = self.end_key {
            total = total.saturating_add(end.len());
        }
        total
    }
}

/// Aktiver oder eingefrorener Memtable-Schreibpuffer.
///
/// Rang 4 Sperre nach SPEC §16.1 (INV-LOCK-1).
pub struct Memtable {
    inner: RankedRwLock<BTreeMap<InternalKey, Slot>, 4>,
    bytes: AtomicUsize,
    min_stamp: AtomicU64,
    max_stamp: AtomicU64,
}

impl Default for Memtable {
    fn default() -> Self {
        Self::new()
    }
}

impl Memtable {
    /// Erstellt eine neue, leere Memtable.
    pub fn new() -> Self {
        Self {
            inner: RankedRwLock::new(BTreeMap::new()),
            bytes: AtomicUsize::new(0),
            min_stamp: AtomicU64::new(u64::MAX),
            max_stamp: AtomicU64::new(0),
        }
    }

    /// Liefert die Summe der Bytes (Schlüssel, Header, Körper).
    pub fn memtable_bytes(&self) -> usize {
        self.bytes.load(Ordering::Acquire)
    }

    /// Liefert den kleinsten enthaltenen Stamp.
    pub fn min_stamp(&self) -> u64 {
        let s = self.min_stamp.load(Ordering::Acquire);
        if s == u64::MAX {
            0
        } else {
            s
        }
    }

    /// Liefert den größten enthaltenen Stamp.
    pub fn max_stamp(&self) -> u64 {
        self.max_stamp.load(Ordering::Acquire)
    }

    /// Fügt Einträge in Unterbatches von höchstens 64 Operationen je Sperre ein (H-S7).
    ///
    /// INV-LOCK-1: Erwirbt Rang 4 Sperre.
    pub fn insert_batch(&self, entries: &[(InternalKey, Slot)]) {
        if entries.is_empty() {
            return;
        }

        // H-S7: Schreibzugriffe in Unterbatches von höchstens 64 Operationen je Sperre
        for chunk in entries.chunks(MEMTABLE_SUBBATCH_LIMIT) {
            let mut guard = self.inner.write();
            let mut added_bytes = 0usize;

            for (k, slot) in chunk {
                let op_bytes = slot.byte_len(k.user_key.len());
                let stamp = k.tag.stamp();

                let cur_min = self.min_stamp.load(Ordering::Relaxed);
                if stamp < cur_min {
                    self.min_stamp.store(stamp, Ordering::Relaxed);
                }
                let cur_max = self.max_stamp.load(Ordering::Relaxed);
                if stamp > cur_max {
                    self.max_stamp.store(stamp, Ordering::Relaxed);
                }

                added_bytes = added_bytes.saturating_add(op_bytes);
                guard.insert(k.clone(), slot.clone());
            }

            self.bytes.fetch_add(added_bytes, Ordering::Release);
        }
    }

    /// Sucht den neuesten Eintrag für einen Nutzerschlüssel bis einschließlich `max_stamp`.
    ///
    /// Ignoriert Einträge mit Stamp > `max_stamp` (INV-VIS-2).
    pub fn get(&self, user_key: &[u8], max_stamp: u64) -> Option<(Tag, Slot)> {
        let guard = self.inner.read();

        // Da InternalKey nach Nutzerschlüssel aufsteigend und Tag absteigend sortiert ist,
        // beginnen alle Einträge für user_key ab (user_key, Tag(u64::MAX)).
        let start_key = InternalKey::new(user_key.to_vec(), Tag(u64::MAX));

        for (k, slot) in guard.range(start_key..) {
            if k.user_key.as_slice() != user_key {
                break;
            }
            let stamp = k.tag.stamp();
            if stamp <= max_stamp {
                return Some((k.tag, slot.clone()));
            }
        }

        None
    }

    /// Prüft, ob ein Schlüssel durch einen aktiven Bereichslöschmarker (RangeDelete/EraseRange)
    /// mit Stamp <= `max_stamp` abgedeckt ist.
    ///
    /// Liefert den höchsten Stamp eines abdeckenden Markers zurück.
    pub fn is_range_deleted(&self, user_key: &[u8], max_stamp: u64) -> Option<u64> {
        let guard = self.inner.read();
        let mut highest_marker_stamp: Option<u64> = None;

        for (k, slot) in guard.iter() {
            let stamp = k.tag.stamp();
            if stamp > max_stamp {
                continue;
            }

            let is_range_marker = match k.tag.kind() {
                Ok(TagKind::RangeDelete | TagKind::EraseRange) => true,
                _ => false,
            };

            if is_range_marker {
                let start = k.user_key.as_slice();
                if start <= user_key {
                    let in_range = match &slot.end_key {
                        Some(end) => user_key < end.as_slice(),
                        None => true,
                    };

                    if in_range {
                        highest_marker_stamp = match highest_marker_stamp {
                            Some(s) if s >= stamp => Some(s),
                            _ => Some(stamp),
                        };
                    }
                }
            }
        }

        highest_marker_stamp
    }

    /// Liefert eine Momentaufnahme aller Einträge.
    pub fn snapshot_entries(&self) -> Vec<(InternalKey, Slot)> {
        let guard = self.inner.read();
        guard.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Liefert die Anzahl der Einträge in der Memtable.
    pub fn entry_count(&self) -> usize {
        let guard = self.inner.read();
        guard.len()
    }
}
