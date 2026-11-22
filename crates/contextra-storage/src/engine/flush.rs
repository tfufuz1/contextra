//! # Flush-Modul (Schreibübergabe in Runs)
//!
//! SPEC §12.1, §16.4, §18 (INV-FLUSH-1):
//! Ablauf je Space:
//! 0. Redo bis max_stamp per sync_data sichern (INV-FLUSH-1)
//! 1. frozen Memtable -> Run bauen (.building), Körper ggf. in Blob schreiben
//! 2. Run und Blob(s) sync_all, sync_dir
//! 3. Edit {add_files, blob_live_delta, space_flushed, redo, reserve} via Synced comitten
//! 4. Neue Version veröffentlichen, frozen entfernen
//! 5. Redo-Segmente unterhalb redo_floor an Graveyard übergeben
//! Keine Integer-Casts (M-02 = 0), keine Panics (M-08 = 0).

use crate::blob::record::BlobRecord;
use crate::blob::writer::BlobWriter;
use crate::catalog::durable::sync_existing;
use crate::catalog::edit::CatalogEdit;
use crate::catalog::log::CatalogLog;
use crate::catalog::reserve::IdReserve;
use crate::catalog::state::{apply_edit, CatalogState};
use crate::catalog::types::{CatalogFileKind, CatalogSpaceOptions, FileMeta};
use crate::codec::tag::{Tag, TagKind};
use crate::engine::compaction::exec::run_file_name;
use crate::engine::compaction::retain::{retain_key_versions, CompactionEntry};
use crate::redo::driver::segment_file_name;
use crate::redo::parse_segment_id;
use crate::run::blob_ref::BlobRef;
use crate::run::block::BlockSecurity;
use crate::run::builder::{RunBuilder, RunBuilderOptions};
use contextra_storage_ports::{Cipher, File, Fs, Kdf, KeyDomain, KeyProvider, Mac, OpenMode, Perm, Stamp};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Schnittstelle zum Redo-Committer für Schritt 0 (INV-FLUSH-1).
pub trait CommitterSync: Send + Sync {
    /// Liefert den aktuell dauerhaft synchronisierten Redo-Stempel.
    fn durable_stamp(&self) -> Stamp;
    /// Synchronisiert das Redo-Log bis mindestens `up_to_stamp`.
    fn sync_data(&self, up_to_stamp: Stamp) -> io::Result<Stamp>;
}

/// Ein Eintrag in der eingefrorenen Memtable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemtableEntry {
    /// Nutzerschlüssel.
    pub key: Vec<u8>,
    /// Tag (Stamp + Kind).
    pub tag: Tag,
    /// Header des Eintrags.
    pub header: Vec<u8>,
    /// Körper des Eintrags (Klartext).
    pub body: Vec<u8>,
    /// Optionale Verschlüsselungsdomäne bei domain_sealed.
    pub domain: Option<u32>,
}

/// Eingefrorene Memtable zur Schreibübergabe.
#[derive(Debug, Clone)]
pub struct FrozenMemtable {
    /// Zugehöriger Space.
    pub space_id: u16,
    /// Höchster Stempel aller enthaltenen Operationen.
    pub max_stamp: Stamp,
    /// Einträge sortiert nach Schlüssel.
    pub entries: Vec<MemtableEntry>,
}

/// Fehler bei der Flush-Ausführung.
#[derive(Debug)]
pub enum FlushError {
    /// E/A-Fehler.
    Io(io::Error),
    /// Verletzung von INV-FLUSH-1 (Redo vor Run-Bau nicht dauerhaft gesichert).
    FlushInvViolation(&'static str),
    /// Katalogfehler.
    Catalog(crate::catalog::types::CatalogError),
}

impl fmt::Display for FlushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Flush E/A-Fehler: {e}"),
            Self::FlushInvViolation(msg) => write!(f, "INV-FLUSH-1 Verletzung: {msg}"),
            Self::Catalog(e) => write!(f, "Katalogfehler im Flush: {e:?}"),
        }
    }
}

impl std::error::Error for FlushError {}

impl From<io::Error> for FlushError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<crate::catalog::types::CatalogError> for FlushError {
    fn from(e: crate::catalog::types::CatalogError) -> Self {
        Self::Catalog(e)
    }
}

/// Führt die vollständige 5-Schritte-Schreibübergabe (Flush) nach SPEC §12.1 durch.
pub fn flush_memtable(
    fs: &dyn Fs,
    base_dir: &Path,
    catalog_file: &dyn File,
    catalog_log: &mut CatalogLog,
    catalog_state: &CatalogState,
    committer: &dyn CommitterSync,
    memtable: &FrozenMemtable,
    floor: Stamp,
    id_reserve: &mut IdReserve,
    space_opts: &CatalogSpaceOptions,
    now_ms: u64,
    key_provider: Option<&dyn KeyProvider>,
    cipher: Option<&dyn Cipher>,
    mac: Option<&dyn Mac>,
    kdf: Option<&dyn Kdf>,
    graveyard: &mut Vec<PathBuf>,
) -> Result<CatalogState, FlushError> {
    // Schritt 0: INV-FLUSH-1
    // Ist durable_stamp < max_stamp der Memtable, Redo per sync_data sichern
    let max_s = memtable.max_stamp;
    if committer.durable_stamp() < max_s {
        committer.sync_data(max_s)?;
    }
    let durable_stamp = committer.durable_stamp();
    if durable_stamp < max_s {
        return Err(FlushError::FlushInvViolation(
            "INV-FLUSH-1: durable_stamp unterschreitet max_stamp der Memtable vor Run-Bau",
        ));
    }

    // Schritt 1: Run (und ggf. Blob) bauen (.building)
    let run_id = id_reserve.ensure_run_id()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let run_building = base_dir.join(format!("{}.building", run_file_name(run_id)));

    let has_separate = (space_opts.flags & 1) != 0;
    let is_domain_sealed = (space_opts.flags & 2) != 0;
    let sep_threshold = u64::from(space_opts.separate_threshold);

    let mut blob_opt: Option<(u64, BlobWriter, PathBuf, Box<dyn File>, u64)> = None;

    // Wandle Memtable-Einträge in CompactionEntries für retain um
    let mut comp_entries = Vec::with_capacity(memtable.entries.len());
    for m in &memtable.entries {
        comp_entries.push(CompactionEntry {
            key: m.key.clone(),
            tag: m.tag,
            header: m.header.clone(),
            body: m.body.clone(),
            blob_ref: None,
        });
    }

    let retain_res = retain_key_versions(
        comp_entries,
        floor,
        false, // Im Flush auf L0 dürfen Tombstones nicht getilgt werden
        None,
        now_ms,
        false,
    );

    let mut pfx_len = None;
    if space_opts.prefix_filter > 0 {
        pfx_len = Some(usize::from(space_opts.prefix_filter));
    }

    let mut builder = RunBuilder::new(
        RunBuilderOptions {
            space_id: memtable.space_id,
            run_id,
            header_len: u16::from(space_opts.header_len),
            weights_k: space_opts.weights_k,
            prefix_filter_len: pfx_len,
            created_ms: now_ms,
        },
        BlockSecurity::Plain,
    );

    for entry in retain_res.retained {
        let key = &entry.key;
        let tag = entry.tag;
        let body_len_u64 = u64::try_from(entry.body.len()).unwrap_or(0);
        let separate = (has_separate && body_len_u64 >= sep_threshold)
            || (is_domain_sealed && !entry.body.is_empty());

        if separate {
            // Lazy Initialisierung der Blob-Datei (.building)
            if blob_opt.is_none() {
                let b_no = id_reserve.ensure_blob_no()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let b_path = base_dir.join(format!("BLOB-{b_no:06}.0000.building"));
                let b_file = fs.open(&b_path, OpenMode::CreateNew(Perm::Private))?;
                let mut b_writer = BlobWriter::new(b_no, 0, memtable.space_id, 0);
                b_writer.write_header(&*b_file)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                blob_opt = Some((b_no, b_writer, b_path, b_file, 0));
            }

            let mut bref_opt: Option<BlobRef> = None;
            let domain_val = entry.blob_ref.as_ref().map_or(0, |b| b.domain);

            if is_domain_sealed {
                if let Some(kp) = key_provider {
                    if let Ok(None) = kp.domain_key(KeyDomain(domain_val)) {
                        // Schlüssel vernichtet: BlobRef mit blob_no = 0
                        bref_opt = Some(BlobRef::shredded(domain_val));
                    }
                }
            }

            let final_bref = if let Some(shredded_ref) = bref_opt {
                shredded_ref
            } else if let Some((_, ref mut writer, _, ref b_file, ref mut live_bytes)) = blob_opt {
                let rec = BlobRecord::new_body(
                    domain_val,
                    tag.stamp(),
                    key.clone(),
                    entry.body.clone(),
                );
                let bref_res = writer.append(&**b_file, &rec, None, None, cipher, mac, kdf)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                *live_bytes = live_bytes.saturating_add(u64::from(bref_res.len));
                BlobRef {
                    blob_no: bref_res.blob_no,
                    offset: bref_res.offset,
                    len: bref_res.len,
                    crc32c: bref_res.crc32c,
                    domain: bref_res.domain,
                }
            } else {
                BlobRef::shredded(domain_val)
            };

            let mut val_bytes = entry.header;
            val_bytes.extend_from_slice(&final_bref.encode());
            let putref_tag = Tag::new(tag.stamp(), TagKind::PutRef);
            builder.add(key, putref_tag, &val_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        } else {
            let mut val_bytes = entry.header;
            val_bytes.extend_from_slice(&entry.body);
            builder.add(key, tag, &val_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        }
    }

    let run_bytes = builder.finish()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let run_file = fs.open(&run_building, OpenMode::CreateNew(Perm::Private))?;
    run_file.write_all_at(&run_bytes, 0)?;

    // Schritt 2: sync_all auf Run und Blob, sync_dir
    run_file.sync_all()?;
    let run_final_path = base_dir.join(run_file_name(run_id));
    fs.rename(&run_building, &run_final_path)?;

    let mut add_files = vec![FileMeta {
        kind: CatalogFileKind::Run,
        id: run_id,
        space: memtable.space_id,
        level: 0,
        tier: 0,
        flags: 0,
        bytes: u64::try_from(run_bytes.len()).unwrap_or(0),
        entries: 0,
        generation: 0,
        segment_kind: 0,
        min_stamp: 0,
        max_stamp: max_s,
        min_key: Vec::new(),
        max_key: Vec::new(),
        blob_no_min: 0,
        blob_no_max: 0,
    }];

    let mut blob_live_delta = Vec::new();

    if let Some((b_no, _, b_path, b_file, live_b)) = blob_opt {
        b_file.sync_all()?;
        let b_final_path = base_dir.join(format!("BLOB-{b_no:06}.0000"));
        fs.rename(&b_path, &b_final_path)?;

        add_files.push(FileMeta {
            kind: CatalogFileKind::Blob,
            id: b_no,
            space: memtable.space_id,
            level: 0,
            tier: 0,
            flags: 0,
            bytes: live_b,
            entries: 0,
            generation: 0,
            segment_kind: 0,
            min_stamp: 0,
            max_stamp: max_s,
            min_key: Vec::new(),
            max_key: Vec::new(),
            blob_no_min: b_no,
            blob_no_max: b_no,
        });

        blob_live_delta.push((b_no, i64::try_from(live_b).unwrap_or(0)));
    }

    fs.sync_dir(base_dir)?;

    // Schritt 3: Edit und Catalog-Commit über Synced
    let mut edit = CatalogEdit::default();
    edit.add_files = add_files;
    edit.blob_live_delta = blob_live_delta;
    edit.space_flushed.push((memtable.space_id, max_s));
    edit.redo = Some((
        catalog_state.redo_floor,
        catalog_state.redo_floor_stamp,
        durable_stamp,
    ));
    edit.reserve = Some((
        id_reserve.run_id_hwm,
        id_reserve.blob_no_hwm,
        id_reserve.segment_id_hwm,
        id_reserve.tmp_id_hwm,
        catalog_state.keys_generation_min,
    ));

    let synced_edit = sync_existing(fs, catalog_file, base_dir, edit.clone())?;
    catalog_log.commit(catalog_file, synced_edit, None, cipher, mac, kdf)?;

    // Schritt 4: Neue Version ableiten
    let new_state = apply_edit(catalog_state, &edit)?;

    // Schritt 5: Redo-Segmente unterhalb redo_floor an Graveyard übergeben
    if let Ok(entries) = fs.list_dir(base_dir) {
        for entry in entries {
            if let Some(seg_id) = parse_segment_id(&entry.name) {
                if seg_id < new_state.redo_floor {
                    graveyard.push(base_dir.join(segment_file_name(seg_id)));
                }
            }
        }
    }

    Ok(new_state)
}
