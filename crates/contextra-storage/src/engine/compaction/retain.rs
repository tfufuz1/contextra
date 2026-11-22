//! # Compaction Retention (rein)
//!
//! SPEC §12.2, §16.4 (INV-RET-1):
//! Reine Funktion zur Versionenbereinigung je Schlüssel unter Berücksichtigung
//! der Untergrenze (floor), Tombstones und optionalem CompactionFilter.
//! Keine Integer-Casts (M-02 = 0), keine Panics (M-08 = 0).

use crate::codec::tag::{Tag, TagKind};
use crate::run::blob_ref::BlobRef;
use contextra_storage_ports::{CompactionFilter, Decision, Stamp};

/// Repräsentiert eine Version eines Schlüssels während der Kompaktion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionEntry {
    /// Nutzerschlüssel.
    pub key: Vec<u8>,
    /// Tag (Stamp + Kind).
    pub tag: Tag,
    /// Header des Werts (z. B. Metadaten).
    pub header: Vec<u8>,
    /// Körper des Werts (oder leer bei PutRef / Delete).
    pub body: Vec<u8>,
    /// Ausgelagerte Blob-Referenz (falls PutRef).
    pub blob_ref: Option<BlobRef>,
}

impl CompactionEntry {
    /// Liefert den Stempel dieser Version.
    pub fn stamp(&self) -> Stamp {
        self.tag.stamp()
    }

    /// Liefert die Art der Operation (Put, PutRef, Delete, etc.).
    pub fn kind(&self) -> Result<TagKind, ()> {
        self.tag.kind().map_err(|_| ())
    }
}

/// Ergebnis der Aufbewahrungsprüfung für die Versionen eines Schlüssels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainResult {
    /// Überlebende Versionen des Schlüssels.
    pub retained: Vec<CompactionEntry>,
    /// Anzahl verworfener älterer Versionen.
    pub dropped_count: usize,
    /// Gesamtanzahl freizugebender Blob-Bytes aus verworfenen Versionen.
    pub dropped_blob_bytes: u64,
    /// Gibt an, ob ein Löschmarker physisch entfernt wurde.
    pub tombstone_dropped: bool,
}

/// Reine Aufbewahrungsfunktion nach SPEC §12.2 und §16.4 (INV-RET-1).
///
/// Regeln:
/// 1. Behalte alle Versionen mit `stamp > floor`.
/// 2. Behalte die neueste Version mit `stamp <= floor`.
/// 3. Verwerfe alle älteren Versionen mit `stamp <= floor`.
/// 4. Löschmarker (`Delete`) mit `stamp <= floor` dürfen nur verworfen werden,
///    wenn `can_drop_tombstone == true` (unterhalb überlappt kein Run und nichts unter floor).
/// 5. Ein `CompactionFilter` wird nur auf Versionen mit `stamp <= floor` angewandt
///    und ist nur in `forgetting`-Spaces wirksam.
pub fn retain_key_versions(
    entries: Vec<CompactionEntry>,
    floor: Stamp,
    can_drop_tombstone: bool,
    filter: Option<&dyn CompactionFilter>,
    now_ms: u64,
    is_forgetting: bool,
) -> RetainResult {
    let mut retained = Vec::with_capacity(entries.len());
    let mut found_floor_version = false;
    let mut dropped_count: usize = 0;
    let mut dropped_blob_bytes: u64 = 0;
    let mut tombstone_dropped = false;

    for mut entry in entries {
        let stamp = entry.stamp();
        let kind_res = entry.kind();

        if stamp > floor {
            // Regel 1: Alle Versionen über dem Floor bleiben strikt erhalten (INV-RET-1)
            retained.push(entry);
        } else if !found_floor_version {
            // Regel 2: Die neueste Version <= floor prüfen
            found_floor_version = true;

            // EraseRange-Marker bleiben immer erhalten bis erase_done (§14)
            if matches!(kind_res, Ok(TagKind::EraseRange)) {
                retained.push(entry);
                continue;
            }

            // Löschmarker (Delete / RangeDelete)
            if matches!(kind_res, Ok(TagKind::Delete | TagKind::RangeDelete)) {
                if can_drop_tombstone {
                    // Tombstone darf physisch getilgt werden
                    tombstone_dropped = true;
                    dropped_count = dropped_count.saturating_add(1);
                } else {
                    retained.push(entry);
                }
                continue;
            }

            // CompactionFilter für forgetting-Spaces (SD-41, SD-48)
            if is_forgetting {
                if let Some(f) = filter {
                    match f.decide(&entry.key, &entry.header, stamp, now_ms) {
                        Decision::Keep => {
                            retained.push(entry);
                        }
                        Decision::Drop => {
                            if let Some(ref bref) = entry.blob_ref {
                                dropped_blob_bytes =
                                    dropped_blob_bytes.saturating_add(u64::from(bref.len));
                            }
                            dropped_count = dropped_count.saturating_add(1);
                        }
                        Decision::DropBody { new_header } => {
                            if let Some(ref bref) = entry.blob_ref {
                                dropped_blob_bytes =
                                    dropped_blob_bytes.saturating_add(u64::from(bref.len));
                            }
                            if let Some(h) = new_header {
                                entry.header = h;
                            }
                            entry.body.clear();
                            entry.blob_ref = None;
                            // Nach DropBody wird der Eintrag als normaler Put geführt
                            entry.tag = Tag::new(stamp, TagKind::Put);
                            retained.push(entry);
                        }
                    }
                    continue;
                }
            }

            retained.push(entry);
        } else {
            // Regel 3: Ältere Versionen <= floor verwerfen
            if let Some(ref bref) = entry.blob_ref {
                dropped_blob_bytes = dropped_blob_bytes.saturating_add(u64::from(bref.len));
            }
            dropped_count = dropped_count.saturating_add(1);
        }
    }

    RetainResult {
        retained,
        dropped_count,
        dropped_blob_bytes,
        tombstone_dropped,
    }
}
