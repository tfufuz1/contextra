// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Reverse-Sweep Subsystem & FprSweepSource Implementation (WP-1.1, C1.4).

use contextra_types::{CollectionId, ContextraError};

use crate::deletion::footprint::ErasureFootprint;
use crate::deletion::registry::KeyFamilyRegistry;

/// Trait für gestreamte Sweep-Quellen ohne vollständige Materialisierung im Speicher.
pub trait FprSweepSource {
    fn for_each_key(&self, f: &mut dyn FnMut(&[u8]) -> bool) -> Result<(), ContextraError>;
}

/// Nadeln für die Rückwärtssuche in der Key-Landschaft.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FprNeedles {
    pub doc_id_bytes: Vec<u8>,
    pub collection_id: Option<CollectionId>,
    pub collection_name: String,
    pub tenant_prefix: Vec<u8>,
}

impl std::fmt::Debug for FprNeedles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FprNeedles")
            .field("doc_id_bytes_len", &self.doc_id_bytes.len())
            .field("has_collection_id", &self.collection_id.is_some())
            .field("collection_name_len", &self.collection_name.len())
            .field("tenant_prefix_len", &self.tenant_prefix.len())
            .finish()
    }
}

/// Tiefe des Reverse-Sweeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepDepth {
    Standard,
    Deep,
}

/// Unregistrierter Rückstand außerhalb des zertifizierten Footprints.
#[derive(Clone, PartialEq, Eq)]
pub struct FprUnregisteredResidue {
    pub family_guess: Option<String>,
    pub count: usize,
    pub hash: [u8; 32],
}

impl std::fmt::Debug for FprUnregisteredResidue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FprUnregisteredResidue")
            .field("family_guess", &self.family_guess)
            .field("count", &self.count)
            .finish()
    }
}

/// Gesamtreport des Reverse-Sweeps (ohne physische Schlüssel-Bytes im Debug).
#[derive(Clone, PartialEq, Eq)]
pub struct FprSweepReport {
    pub scanned_keys_count: usize,
    pub hard_matches_count: usize,
    pub ambiguous_matches_count: usize,
    pub unregistered_residues: Vec<FprUnregisteredResidue>,
}

impl std::fmt::Debug for FprSweepReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FprSweepReport")
            .field("scanned_keys_count", &self.scanned_keys_count)
            .field("hard_matches_count", &self.hard_matches_count)
            .field("ambiguous_matches_count", &self.ambiguous_matches_count)
            .field(
                "unregistered_residues_count",
                &self.unregistered_residues.len(),
            )
            .finish()
    }
}

/// Führt den zwei-dimensionalen Reverse-Sweep gemäß Spezifikation C1.4 aus.
pub fn reverse_sweep<S: FprSweepSource>(
    source: &S,
    needles: &FprNeedles,
    footprint: &ErasureFootprint,
    registry: &KeyFamilyRegistry,
    _depth: SweepDepth,
) -> Result<FprSweepReport, ContextraError> {
    let mut scanned_keys_count = 0usize;
    let mut hard_matches_count = 0usize;
    let mut ambiguous_matches_count = 0usize;
    let mut unregistered_residues = Vec::new();

    source.for_each_key(&mut |raw_key| {
        scanned_keys_count += 1;

        // 1. Tenant-Filterung
        let key = if !needles.tenant_prefix.is_empty() {
            if raw_key.starts_with(&needles.tenant_prefix) {
                &raw_key[needles.tenant_prefix.len()..]
            } else {
                return true; // Nicht tenant-relevant, weiter iterieren
            }
        } else {
            raw_key
        };

        // 2. Prüfen, ob der Schlüssel bereits durch den Footprint abgedeckt war
        let covered_by_footprint = footprint.keys.explicit.iter().any(|e| e.as_slice() == key)
            || footprint
                .keys
                .prefixes
                .iter()
                .any(|p| key.starts_with(p.as_slice()));

        if covered_by_footprint {
            return true;
        }

        // 3. Prüfen auf Nadel-Matches (Dokument-ID, Collection-Name)
        let matches_doc = !needles.doc_id_bytes.is_empty()
            && key
                .windows(needles.doc_id_bytes.len())
                .any(|w| w == needles.doc_id_bytes.as_slice());

        let matches_col = !needles.collection_name.is_empty()
            && key
                .windows(needles.collection_name.len())
                .any(|w| w == needles.collection_name.as_bytes());

        let is_needle_hit = matches_doc || matches_col;

        // 4. Strukturierte Muster-Prüfung gegen die Registry
        let matches_registered_family = registry.all_families().iter().any(|family| {
            if key.starts_with(family.prefix_pattern.as_bytes()) {
                return true;
            }
            if family.prefix_pattern.contains(':') {
                let prefix_head = family
                    .prefix_pattern
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .as_bytes();
                if !prefix_head.is_empty() && key.starts_with(prefix_head) {
                    return true;
                }
            }
            false
        });

        if is_needle_hit {
            if matches_registered_family {
                hard_matches_count += 1;
            } else {
                // Regeln C1.4: Nadel < 16 Byte => AmbiguousMatch, >= 16 Byte => HardMatch
                let needle_len = if matches_doc {
                    needles.doc_id_bytes.len()
                } else {
                    needles.collection_name.len()
                };

                if needle_len < 16 {
                    ambiguous_matches_count += 1;
                } else {
                    hard_matches_count += 1;
                }
            }

            // Unregistrierten Rückstand erfassen
            let family_guess = if let Some(colon_pos) = key.iter().position(|&b| b == b':') {
                String::from_utf8(key[..colon_pos].to_vec()).ok()
            } else {
                None
            };

            let key_hash = *blake3::hash(key).as_bytes();
            unregistered_residues.push(FprUnregisteredResidue {
                family_guess,
                count: 1,
                hash: key_hash,
            });
        }

        true
    })?;

    Ok(FprSweepReport {
        scanned_keys_count,
        hard_matches_count,
        ambiguous_matches_count,
        unregistered_residues,
    })
}
