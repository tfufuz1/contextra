//! # Sichtbarkeits- und View-Protokoll
//!
//! Normative Referenzen: SPEC §10.3, §16.3 (INV-VIS-1, INV-VIS-2, INV-VIS-3).
//! Keine Integer-Casts (M-02 = 0).

use crate::codec::tag::TagKind;
use crate::engine::memtable::Slot;
use crate::engine::version::{Version, VersionState};
use crate::sync::{Arc, AtomicU64, Ordering};

/// Eine konsistente Point-in-Time-Sicht auf den Speicherzustand.
///
/// `View = (Arc<Version>, stamp)` nach SPEC §10.3.
/// Ein Leser ignoriert alle Einträge mit Stamp > `stamp` (INV-VIS-2).
#[derive(Clone)]
pub struct View {
    /// Die beim Erzeugen der View festgehaltene Version.
    pub version: Arc<Version>,
    /// Der Sichtbarkeitsstempel dieser View.
    pub stamp: u64,
}

impl View {
    /// Erstellt eine neue View aus einer Version und einem Stamp.
    pub fn new(version: Arc<Version>, stamp: u64) -> Self {
        Self { version, stamp }
    }

    /// Liest den Wert für einen Schlüssel in einem Space.
    ///
    /// Ignoriert alle Einträge mit Stamp > `self.stamp` (INV-VIS-2, INV-VIS-3).
    pub fn get(&self, space_id: u16, user_key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        let space = self.version.get_space(space_id)?;

        let mut best_entry: Option<(u64, TagKind, Slot)> = None;

        // 1. Zuerst in der aktiven Memtable suchen
        if let Some((tag, slot)) = space.active.get(user_key, self.stamp) {
            let s = tag.stamp();
            if let Ok(kind) = tag.kind() {
                best_entry = Some((s, kind, slot));
            }
        }

        // 2. In den gefrorenen Memtables suchen (neueste zuerst)
        for frozen in space.frozen.iter().rev() {
            if let Some((tag, slot)) = frozen.get(user_key, self.stamp) {
                let s = tag.stamp();
                if let Ok(kind) = tag.kind() {
                    match &best_entry {
                        Some((best_s, _, _)) if *best_s >= s => {}
                        _ => {
                            best_entry = Some((s, kind, slot));
                        }
                    }
                }
            }
        }

        // 3. Prüfen, ob ein Bereichslöschmarker (RangeDelete/EraseRange) den Eintrag verdeckt
        let mut rd_stamp = space.active.is_range_deleted(user_key, self.stamp);
        for frozen in space.frozen.iter().rev() {
            if let Some(s) = frozen.is_range_deleted(user_key, self.stamp) {
                rd_stamp = match rd_stamp {
                    Some(cur) if cur >= s => Some(cur),
                    _ => Some(s),
                };
            }
        }

        if let Some(del_stamp) = rd_stamp {
            if let Some((entry_stamp, _, _)) = best_entry {
                if del_stamp >= entry_stamp {
                    return None;
                }
            } else {
                return None;
            }
        }

        match best_entry {
            Some((_, TagKind::Put, slot)) => Some((slot.header, slot.body)),
            Some((_, TagKind::Delete, _)) => None,
            _ => None,
        }
    }
}

/// Erzeugt eine neue View gemäß INV-VIS-3:
///
/// INV-VIS-3 Leseordnung:
/// Ein Leser lädt ZUERST `visible_stamp` (Acquire) und DANACH die Version (Rang 3).
///
/// Beweis nach SPEC §16.3:
/// Sei e ein Eintrag mit Stamp <= v. Weil v per Acquire geladen wurde, gilt für den
/// Schreiber der Gruppe mit v: sein Einfügen von e (Schritt a) happens-before das Laden (Release/Acquire).
/// Zum Zeitpunkt des Einfügens stand e in der damaligen aktiven Memtable M der damaligen Version V0.
/// Versionen wachsen nur monoton. V wurde NACH dem Laden von v gelesen, ist also mindestens
/// so neu wie jede Version, die zum Zeitpunkt des Releases galt; also enthält V e in M oder
/// einer gefrorenen Memtable.
pub fn create_view(visible_stamp: &AtomicU64, state: &VersionState) -> View {
    // 1. ZUERST visible_stamp per Acquire laden (INV-VIS-3)
    let stamp = visible_stamp.load(Ordering::Acquire);
    // 2. DANACH Version unter Rang 3 lesen (INV-VIS-3)
    let version = Arc::clone(&*state.read());
    View::new(version, stamp)
}
