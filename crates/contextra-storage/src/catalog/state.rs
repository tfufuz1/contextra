//! # Katalog-Zustand und Übergangsfunktion
//!
//! SPEC §9.7: Der Zustand ist eine reine Reduktion (SD-05) über das Edit-Log.
//! apply_edit(state, edit) -> state ist eine reine Funktion (P-2, INV-CAT-2).
//! Metadaten ermöglichen Auswahl ohne Dateizugriff (H-S5).

use crate::catalog::edit::CatalogEdit;
use crate::catalog::types::{
    CatalogError, CatalogFileKind, CatalogSpaceOptions, ErasePendingMeta, ErasedRangeMeta,
    FileMeta, HoldMeta,
};
use std::collections::{HashMap, HashSet};

/// Aktueller Zustand aller aktiven Dateien und Metadaten des Speicherkerns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogState {
    /// Aktive Dateien indiziert nach (FileKind, id).
    pub files: HashMap<(CatalogFileKind, u64), FileMeta>,
    /// Lebendbytes je Blob-Nummer für GC-Planung ohne Datei-Öffnung (§9.6).
    pub blob_live_bytes: HashMap<u64, u64>,
    /// Geflushter Stamp je Space.
    pub space_flushed: HashMap<u16, u64>,
    /// Untere Grenze für aktive Redo-Segmente.
    pub redo_floor: u64,
    /// Commit-Stamp der unteren Redo-Grenze.
    pub redo_floor_stamp: u64,
    /// Höchste gesehene Redo-Segment-ID (INV-LOG-8).
    pub redo_hwm: u64,
    /// ID-Hochwassermarke für run_id (INV-KEY-2).
    pub run_id_hwm: u64,
    /// ID-Hochwassermarke für blob_no (INV-KEY-2).
    pub blob_no_hwm: u64,
    /// ID-Hochwassermarke für segment_id (INV-KEY-2).
    pub segment_id_hwm: u64,
    /// ID-Hochwassermarke für tmp_id (INV-KEY-2).
    pub tmp_id_hwm: u64,
    /// Minimale Schlüsselgeneration nach KEK-Rotation.
    pub keys_generation_min: u64,
    /// Registrierte Spaces.
    pub spaces: HashMap<u16, (Vec<u8>, CatalogSpaceOptions)>,
    /// Persistente Consumer-Cursor: name -> (consumed_stamp, max_lag_bytes).
    pub consumers: HashMap<Vec<u8>, (u64, u64)>,
    /// Nachweis vernichteter Domänenschlüssel (§15.3).
    pub domain_retired: HashMap<u32, u64>,
    /// Ausstehende physische Löschaufträge.
    pub erase_pending: Vec<ErasePendingMeta>,
    /// Abgeschlossene Löschaufträge.
    pub erase_done: HashSet<u64>,
    /// Im Blob-Sweep abgearbeitete Löschaufträge.
    pub erase_swept: HashSet<u64>,
    /// Gelöschte Bereiche zur Filterung.
    pub erased_ranges: Vec<ErasedRangeMeta>,
    /// Dauerhafte Holds nach §7.8.2.
    pub holds: HashMap<u64, HoldMeta>,
    /// Schlüsselepoch je Space.
    pub key_epochs: HashMap<u16, u32>,
}

/// Reine Zustandsübergangsfunktion nach SPEC §9.7 (P-2).
///
/// INV-CAT-2: Ein Edit ist ein Record; er wird ganz oder gar nicht angewendet.
pub fn apply_edit(state: &CatalogState, edit: &CatalogEdit) -> Result<CatalogState, CatalogError> {
    let mut next = state.clone();

    // 1. Dateien hinzufügen
    for file in &edit.add_files {
        next.files.insert((file.kind, file.id), file.clone());
        if file.kind == CatalogFileKind::Blob {
            next.blob_live_bytes.insert(file.id, file.bytes);
        }
    }

    // 2. Dateien entfernen
    for (kind, id) in &edit.remove_files {
        next.files.remove(&(*kind, *id));
        if *kind == CatalogFileKind::Blob {
            next.blob_live_bytes.remove(id);
        }
    }

    // 3. Blob Live Delta anwenden
    for (blob_no, delta) in &edit.blob_live_delta {
        let current = next.blob_live_bytes.get(blob_no).copied().unwrap_or(0);
        let updated = if *delta >= 0 {
            let d_u64 = u64::try_from(*delta).unwrap_or(0);
            current.saturating_add(d_u64)
        } else {
            let abs_d = delta.checked_abs().and_then(|v| u64::try_from(v).ok()).unwrap_or(0);
            current.saturating_sub(abs_d)
        };

        if updated == 0 {
            // SPEC §9.7: live_bytes == 0 => Blob im selben Edit entfernt
            next.blob_live_bytes.remove(blob_no);
            next.files.remove(&(CatalogFileKind::Blob, *blob_no));
        } else {
            next.blob_live_bytes.insert(*blob_no, updated);
        }
    }

    // 4. Blob Replace (Sweep g -> g+1)
    for (blob_no, _old_gen, new_gen) in &edit.blob_replace {
        if let Some(f) = next.files.get_mut(&(CatalogFileKind::Blob, *blob_no)) {
            f.generation = *new_gen;
        }
    }

    // 5. Space flushed
    for (space, stamp) in &edit.space_flushed {
        next.space_flushed.insert(*space, *stamp);
    }

    // 6. Redo Grenzen
    if let Some((floor, floor_st, hwm)) = edit.redo {
        next.redo_floor = floor;
        next.redo_floor_stamp = floor_st;
        next.redo_hwm = hwm;
    }

    // 7. Reserve (INV-KEY-2: Hochwassermarken steigen streng monoton, nie zurück)
    if let Some((r_hwm, b_hwm, s_hwm, t_hwm, k_gen)) = edit.reserve {
        next.run_id_hwm = next.run_id_hwm.max(r_hwm);
        next.blob_no_hwm = next.blob_no_hwm.max(b_hwm);
        next.segment_id_hwm = next.segment_id_hwm.max(s_hwm);
        next.tmp_id_hwm = next.tmp_id_hwm.max(t_hwm);
        next.keys_generation_min = next.keys_generation_min.max(k_gen);
    }

    // 8. Spaces
    for (id, name, opts) in &edit.spaces_add {
        next.spaces.insert(*id, (name.clone(), opts.clone()));
    }
    for id in &edit.spaces_drop {
        next.spaces.remove(id);
    }

    // 9. Consumer
    for (name, stamp, max_lag) in &edit.consumer_set {
        next.consumers.insert(name.clone(), (*stamp, *max_lag));
    }
    for name in &edit.consumer_drop {
        next.consumers.remove(name);
    }

    // 10. Domain retired
    for (dom, stamp) in &edit.domain_retired {
        next.domain_retired.insert(*dom, *stamp);
    }

    // 11. Erase pending, done, swept
    for ep in &edit.erase_pending_add {
        next.erase_pending.push(ep.clone());
    }
    for id in &edit.erase_done {
        next.erase_done.insert(*id);
    }
    for id in &edit.erase_swept {
        next.erase_swept.insert(*id);
    }

    // 12. Erased ranges
    for er in &edit.erased_ranges_add {
        next.erased_ranges.push(er.clone());
    }
    for rem in &edit.erased_ranges_remove {
        next.erased_ranges.retain(|e| e != rem);
    }

    // 13. Holds
    for h in &edit.hold_set {
        next.holds.insert(h.hold_id, h.clone());
    }
    for id in &edit.hold_drop {
        next.holds.remove(id);
    }

    // 14. Key epoch
    for (space, ep) in &edit.key_epoch {
        next.key_epochs.insert(*space, *ep);
    }

    Ok(next)
}

impl CatalogState {
    /// Erstellt einen vollständigen Snapshot-Edit, der den gesamten Zustand abbildet.
    pub fn to_snapshot_edit(&self) -> CatalogEdit {
        let mut add_files: Vec<FileMeta> = self.files.values().cloned().collect();
        add_files.sort_by_key(|f| (f.kind, f.id));

        let mut space_flushed: Vec<(u16, u64)> = self.space_flushed.iter().map(|(&s, &st)| (s, st)).collect();
        space_flushed.sort_by_key(|&(s, _)| s);

        let mut spaces_add: Vec<(u16, Vec<u8>, CatalogSpaceOptions)> = self
            .spaces
            .iter()
            .map(|(&id, (name, opts))| (id, name.clone(), opts.clone()))
            .collect();
        spaces_add.sort_by_key(|&(id, _, _)| id);

        let mut consumer_set: Vec<(Vec<u8>, u64, u64)> = self
            .consumers
            .iter()
            .map(|(name, &(st, lag))| (name.clone(), st, lag))
            .collect();
        consumer_set.sort_by(|a, b| a.0.cmp(&b.0));

        let mut domain_retired: Vec<(u32, u64)> = self.domain_retired.iter().map(|(&d, &st)| (d, st)).collect();
        domain_retired.sort_by_key(|&(d, _)| d);

        let mut hold_set: Vec<HoldMeta> = self.holds.values().cloned().collect();
        hold_set.sort_by_key(|h| h.hold_id);

        let mut key_epoch: Vec<(u16, u32)> = self.key_epochs.iter().map(|(&s, &e)| (s, e)).collect();
        key_epoch.sort_by_key(|&(s, _)| s);

        CatalogEdit {
            add_files,
            remove_files: Vec::new(),
            blob_live_delta: Vec::new(),
            blob_replace: Vec::new(),
            space_flushed,
            redo: Some((self.redo_floor, self.redo_floor_stamp, self.redo_hwm)),
            reserve: Some((
                self.run_id_hwm,
                self.blob_no_hwm,
                self.segment_id_hwm,
                self.tmp_id_hwm,
                self.keys_generation_min,
            )),
            spaces_add,
            spaces_drop: Vec::new(),
            consumer_set,
            consumer_drop: Vec::new(),
            domain_retired,
            erase_pending_add: self.erase_pending.clone(),
            erase_done: self.erase_done.iter().copied().collect(),
            erase_swept: self.erase_swept.iter().copied().collect(),
            erased_ranges_add: self.erased_ranges.clone(),
            erased_ranges_remove: Vec::new(),
            hold_set,
            hold_drop: Vec::new(),
            key_epoch,
        }
    }

    /// Sucht alle aktiven Runs eines Space, die sich mit [min_key, max_key] überschneiden.
    ///
    /// Funktioniert rein im Speicher ohne jeglichen Dateizugriff (H-S5).
    pub fn runs_in_range(&self, space: u16, min_key: &[u8], max_key: &[u8]) -> Vec<&FileMeta> {
        self.files
            .values()
            .filter(|f| f.kind == CatalogFileKind::Run && f.space == space)
            .filter(|f| {
                if !f.max_key.is_empty() && f.max_key.as_slice() < min_key {
                    return false;
                }
                if !f.min_key.is_empty() && f.min_key.as_slice() > max_key {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Liefert alle aktiven Runs eines Space in einem Stamp-Bereich.
    pub fn runs_by_stamp_range(&self, space: u16, min_stamp: u64, max_stamp: u64) -> Vec<&FileMeta> {
        self.files
            .values()
            .filter(|f| f.kind == CatalogFileKind::Run && f.space == space)
            .filter(|f| f.max_stamp >= min_stamp && f.min_stamp <= max_stamp)
            .collect()
    }

    /// Liefert alle lebendigen Blob-Dateien.
    pub fn live_blobs(&self) -> Vec<&FileMeta> {
        self.files
            .values()
            .filter(|f| f.kind == CatalogFileKind::Blob)
            .collect()
    }

    /// Liefert die Lebendbytes für eine gegebene Blob-Nummer.
    pub fn blob_live_bytes_for(&self, blob_no: u64) -> Option<u64> {
        self.blob_live_bytes.get(&blob_no).copied()
    }

    /// Berechnet die Gesamtbytes aller aktiven Dateien.
    pub fn total_bytes(&self) -> u64 {
        self.files.values().map(|f| f.bytes).fold(0u64, |acc, b| acc.saturating_add(b))
    }

    /// Prüft, ob eine Datei laut Katalog lebendig ist (INV-CAT-1).
    pub fn is_file_live(&self, kind: CatalogFileKind, id: u64) -> bool {
        self.files.contains_key(&(kind, id))
    }
}
