//! # Reine Retention-Logik
//!
//! Normative Referenzen: SPEC §12.2, §16.4 (INV-RET-1, INV-HOLD-1).
//! INV-HOLD-1: Dauerhafte Holds überleben Neustart und zählen bis Ablauf/hold_drop in die Untergrenze floor.
//! Keine Integer-Casts (M-02 = 0).

use crate::codec::tag::{InternalKey, TagKind};
use crate::engine::memtable::Slot;

/// Ein Eintrag für die Retention-Prüfung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainItem {
    /// Interner Schlüssel mit Nutzerschlüssel und Tag.
    pub key: InternalKey,
    /// Assoziierter Slot (Header, Körper, End-Schlüssel).
    pub slot: Slot,
}

impl RetainItem {
    /// Erstellt ein neues RetainItem.
    pub fn new(key: InternalKey, slot: Slot) -> Self {
        Self { key, slot }
    }

    /// Liefert den Stamp des Eintrags.
    pub fn stamp(&self) -> u64 {
        self.key.tag.stamp()
    }

    /// Prüft, ob es sich um einen Löschmarker handelt (Delete, RangeDelete, EraseRange).
    pub fn is_delete(&self) -> bool {
        match self.key.tag.kind() {
            Ok(TagKind::Delete | TagKind::RangeDelete | TagKind::EraseRange) => true,
            _ => false,
        }
    }
}

/// Reine Funktion `retain(entries, floor, has_lower)` nach SPEC §12.2 und §16.4 (INV-RET-1).
///
/// Regeln:
/// - Je Schlüssel alle Versionen mit Stamp > `floor` behalten.
/// - Die neueste Version mit Stamp <= `floor` behalten.
/// - Ältere Versionen mit Stamp <= `floor` verwerfen.
/// - Löschmarker (Delete, RangeDelete) nur verwerfen, wenn darunter nichts liegt
///   (`has_lower == false`) und nichts unter der Untergrenze liegt.
///
/// Die Eingabesequenz `entries` muss nach `InternalKey` sortiert sein:
/// Nutzerschlüssel bytelexikografisch aufsteigend, Tag absteigend (neueste Version zuerst).
pub fn retain(
    entries: impl IntoIterator<Item = RetainItem>,
    floor: u64,
    has_lower: bool,
) -> Vec<RetainItem> {
    let mut result = Vec::new();

    let mut current_user_key: Option<Vec<u8>> = None;
    let mut kept_at_or_below_floor = false;

    for item in entries {
        let is_new_key = match &current_user_key {
            Some(uk) => uk.as_slice() != item.key.user_key.as_slice(),
            None => true,
        };

        if is_new_key {
            current_user_key = Some(item.key.user_key.clone());
            kept_at_or_below_floor = false;
        }

        let stamp = item.stamp();
        if stamp > floor {
            // Version über floor: IMMER behalten (INV-RET-1)
            result.push(item);
        } else if !kept_at_or_below_floor {
            // Neueste Version mit Stamp <= floor
            kept_at_or_below_floor = true;

            if item.is_delete() {
                // Löschmarker nur behalten, wenn darunter noch Daten liegen (has_lower == true)
                if has_lower {
                    result.push(item);
                }
            } else {
                result.push(item);
            }
        } else {
            // Ältere Versionen unterhalb von floor werden verworfen
        }
    }

    result
}
