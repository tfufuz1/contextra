//! # Begrenzter Clusterschlüssel-Cache (SPEC §4, §16.4)
//!
//! Der Kern hält Clusterschlüssel nur in einem begrenzten Cache. Verdrängte,
//! entfernte und vernichtete Schlüssel werden beim Verwerfen genullt: `Secret32`
//! überschreibt beim `Drop` den Speicher best-effort (SD-56, ohne `unsafe`).
//!
//! Verdrängt wird in Einfügereihenfolge (FIFO). Die Reihenfolge ist deterministisch,
//! damit Tests sie festlegen können. Ein Cache mit Kapazität `0` speichert nichts.

use super::cluster_keys::ClusterId;
use contextra_ports::persistence::Secret32;

/// Ein Eintrag: Clusterschlüssel je Epoche.
struct Entry {
    cluster: ClusterId,
    epoch: u32,
    key: Secret32,
}

/// Begrenzter Cache für Clusterschlüssel.
pub struct KeyCache {
    capacity: usize,
    entries: Vec<Entry>,
}

impl KeyCache {
    /// Legt einen leeren Cache mit der angegebenen Kapazität an.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Vec::new(),
        }
    }

    /// Anzahl der gehaltenen Schlüssel.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true`, wenn der Cache leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Sucht den Schlüssel eines Clusters in einer Epoche.
    pub fn get(&self, cluster: ClusterId, epoch: u32) -> Option<&Secret32> {
        self.entries
            .iter()
            .find(|e| e.cluster == cluster && e.epoch == epoch)
            .map(|e| &e.key)
    }

    /// Fügt einen Schlüssel ein. Ersetzt einen vorhandenen Eintrag derselben Epoche
    /// (der alte Wert wird genullt). Verdrängt bei voller Kapazität den ältesten Eintrag.
    pub fn insert(&mut self, cluster: ClusterId, epoch: u32, key: Secret32) {
        if self.capacity == 0 {
            // Schlüssel wird hier verworfen und dabei genullt.
            return;
        }
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|e| e.cluster == cluster && e.epoch == epoch)
        {
            existing.key = key;
            return;
        }
        if self.entries.len() >= self.capacity {
            // Der verdrängte Eintrag wird beim Verwerfen genullt.
            self.entries.remove(0);
        }
        self.entries.push(Entry {
            cluster,
            epoch,
            key,
        });
    }

    /// Entfernt alle Epochen eines Clusters (nach `destroy`).
    pub fn invalidate_cluster(&mut self, cluster: ClusterId) {
        self.entries.retain(|e| e.cluster != cluster);
    }

    /// Entfernt alle Schlüssel und nullt sie.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
