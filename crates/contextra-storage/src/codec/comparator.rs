//! # Vergleich interner Schlüssel (`cmp`, SPEC §6.2, Freeze-Liste §10.12)
//!
//! Ordnung eines Eintrags:
//! 1. Cluster **aufsteigend** (16 Byte, opak),
//! 2. Nutzerschlüssel **bytelexikografisch aufsteigend**,
//! 3. Tag **absteigend** (neueste Version zuerst).
//!
//! Die Felder werden einzeln verglichen. Ein `memcmp` über eine Konkatenation ist
//! ausdrücklich ausgeschlossen (§6.2), weil es Schlüssel mit gleichem Präfix und
//! unterschiedlicher Länge falsch ordnen würde.

use super::tag::Tag;
use std::cmp::Ordering;

/// Cluster-Kennung: 16 Byte, opak (SPEC §6.1).
pub type ClusterBytes = [u8; 16];

/// `ClusterId::NONE`: „ohne Cluster“, nur in nicht geclusterten Spaces (§6.1).
pub const CLUSTER_NONE: ClusterBytes = [0u8; 16];

/// Eintrag für die Ordnung nach §6.2. Leiht den Nutzerschlüssel aus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryKey<'a> {
    /// Cluster des Eintrags.
    pub cluster: ClusterBytes,
    /// Nutzerschlüssel (1 ≤ Länge ≤ `max_key_bytes`).
    pub user_key: &'a [u8],
    /// Tag `(stamp << 8) | kind`.
    pub tag: Tag,
}

impl Ord for EntryKey<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cluster
            .cmp(&other.cluster)
            .then_with(|| self.user_key.cmp(other.user_key))
            // Tag absteigend: größerer Tag (neuerer Stamp) kommt zuerst.
            .then_with(|| other.tag.0.cmp(&self.tag.0))
    }
}

impl PartialOrd for EntryKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::tag::TagKind;

    fn key<'a>(cluster: u8, user_key: &'a [u8], stamp: u64) -> EntryKey<'a> {
        EntryKey {
            cluster: [cluster; 16],
            user_key,
            tag: Tag::new(stamp, TagKind::Put),
        }
    }

    #[test]
    fn cluster_hat_vorrang_vor_schluessel() {
        // Cluster 1 mit großem Schlüssel steht vor Cluster 2 mit kleinem Schlüssel.
        assert!(key(1, b"zzz", 1) < key(2, b"a", 1));
    }

    #[test]
    fn schluessel_bytelexikografisch_nicht_konkateniert() {
        // Konkatenation "a"+"bc" == "ab"+"c" würde gleich sein; einzelner Vergleich nicht.
        assert!(key(0, b"a", 1) < key(0, b"ab", 1));
        assert!(key(0, b"ab", 1) < key(0, b"b", 1));
        // Präfix kommt vor der Verlängerung.
        assert!(key(0, b"k", 1) < key(0, b"k\x00", 1));
    }

    #[test]
    fn tag_absteigend_neueste_version_zuerst() {
        assert!(key(0, b"k", 7) < key(0, b"k", 1));
    }

    #[test]
    fn gleiche_eintraege_sind_gleich() {
        assert_eq!(key(3, b"k", 5).cmp(&key(3, b"k", 5)), Ordering::Equal);
    }

    #[test]
    fn ordnung_ist_total_und_antisymmetrisch_auf_stichprobe() {
        let samples = [
            key(0, b"a", 1),
            key(0, b"a", 9),
            key(0, b"ab", 1),
            key(1, b"", 1),
            key(1, b"a", 3),
        ];
        for x in &samples {
            for y in &samples {
                assert_eq!(x.cmp(y), y.cmp(x).reverse());
            }
        }
    }
}
