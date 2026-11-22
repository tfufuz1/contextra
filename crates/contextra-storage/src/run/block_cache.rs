//! # BlockCache: CLOCK-Seitenersetzungsalgorithmus für Daten- und Metadatenblöcke
//!
//! Gemäß SPEC §11, §17.1:
//! Genau EINE CLOCK-Implementierung.
//! Schlüssel: `(file_id: u64, block_no: u32)`.
//! Der Cache hält KEINE Körper (nur Blöcke).
//! `invalidate_file(file_id)` zum schnellen Invalidieren bei Dateilöschung.
//! Keine Integer-Casts (M-02 = 0).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Eindeutiger Cache-Schlüssel für einen Block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockKey {
    /// Eindeutige ID der Run-Datei.
    pub file_id: u64,
    /// Fortlaufende Blocknummer im Run.
    pub block_no: u32,
}

impl BlockKey {
    /// Erstellt einen neuen Block-Schlüssel.
    pub const fn new(file_id: u64, block_no: u32) -> Self {
        Self { file_id, block_no }
    }
}

struct ClockSlot {
    key: BlockKey,
    value: Arc<Vec<u8>>,
    referenced: AtomicBool,
}

struct ClockInner {
    slots: Vec<Option<ClockSlot>>,
    capacity: usize,
    hand: usize,
}

/// Thread-sicherer CLOCK-Block-Cache.
#[derive(Clone)]
pub struct BlockCache {
    inner: Arc<Mutex<ClockInner>>,
}

impl BlockCache {
    /// Erstellt einen neuen Block-Cache mit der angegebenen Slot-Kapazität.
    pub fn new(capacity: usize) -> Self {
        let cap = std::cmp::max(1, capacity);
        let mut slots = Vec::with_capacity(cap);
        for _ in 0..cap {
            slots.push(None);
        }
        Self {
            inner: Arc::new(Mutex::new(ClockInner {
                slots,
                capacity: cap,
                hand: 0,
            })),
        }
    }

    /// Ruft einen Block aus dem Cache ab und setzt das Referenz-Bit.
    pub fn get(&self, file_id: u64, block_no: u32) -> Option<Arc<Vec<u8>>> {
        let key = BlockKey::new(file_id, block_no);
        let guard = self.inner.lock().ok()?;

        for slot_opt in &guard.slots {
            if let Some(slot) = slot_opt {
                if slot.key == key {
                    slot.referenced.store(true, Ordering::Relaxed);
                    return Some(Arc::clone(&slot.value));
                }
            }
        }
        None
    }

    /// Fügt einen Block in den Cache ein oder aktualisiert ihn.
    pub fn insert(&self, file_id: u64, block_no: u32, data: Arc<Vec<u8>>) {
        let key = BlockKey::new(file_id, block_no);
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        // 1. Prüfen, ob Schlüssel bereits existiert
        for slot_opt in guard.slots.iter_mut() {
            if let Some(slot) = slot_opt {
                if slot.key == key {
                    slot.value = data;
                    slot.referenced.store(true, Ordering::Relaxed);
                    return;
                }
            }
        }

        // 2. Leeren Slot suchen
        for slot_opt in guard.slots.iter_mut() {
            if slot_opt.is_none() {
                *slot_opt = Some(ClockSlot {
                    key,
                    value: data,
                    referenced: AtomicBool::new(true),
                });
                return;
            }
        }

        // 3. CLOCK-Verdrängung
        let cap = guard.capacity;
        let mut steps = 0usize;
        let max_steps = cap.saturating_mul(2);

        while steps < max_steps {
            let hand = guard.hand;
            guard.hand = (hand.saturating_add(1)) % cap;
            steps = steps.saturating_add(1);

            let evict = if let Some(Some(slot)) = guard.slots.get(hand) {
                if slot.referenced.swap(false, Ordering::Relaxed) {
                    false
                } else {
                    true
                }
            } else {
                true
            };

            if evict {
                if let Some(slot_opt) = guard.slots.get_mut(hand) {
                    *slot_opt = Some(ClockSlot {
                        key,
                        value: data,
                        referenced: AtomicBool::new(true),
                    });
                    return;
                }
            }
        }

        // Fallback: ersten Slot überschreiben
        if let Some(slot_opt) = guard.slots.get_mut(0) {
            *slot_opt = Some(ClockSlot {
                key,
                value: data,
                referenced: AtomicBool::new(true),
            });
        }
    }

    /// Entfernt alle gecachten Blöcke einer bestimmten Datei (SPEC §11).
    pub fn invalidate_file(&self, file_id: u64) {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        for slot_opt in guard.slots.iter_mut() {
            if let Some(slot) = slot_opt {
                if slot.key.file_id == file_id {
                    *slot_opt = None;
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_cache_basic_and_eviction() {
        let cache = BlockCache::new(2);

        cache.insert(1, 0, Arc::new(vec![1, 2, 3]));
        cache.insert(1, 1, Arc::new(vec![4, 5, 6]));

        assert!(cache.get(1, 0).is_some());
        assert!(cache.get(1, 1).is_some());

        // Dritter Eintrag verdrängt
        cache.insert(1, 2, Arc::new(vec![7, 8, 9]));
        assert!(cache.get(1, 2).is_some());

        // Invalidate file
        cache.invalidate_file(1);
        assert!(cache.get(1, 0).is_none());
        assert!(cache.get(1, 1).is_none());
        assert!(cache.get(1, 2).is_none());
    }
}
