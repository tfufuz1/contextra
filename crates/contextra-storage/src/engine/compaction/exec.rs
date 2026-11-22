//! # Compaction Ausführung (exec)
//!
//! SPEC §12.2, §12.3:
//! Zusammenführung der Eingabe-Runs, Anwenden der Versionenaufbewahrung (retain),
//! Schreiben neuer Runs über den RunBuilder und Erzeugen neuer Blobs bei Bedarf.
//! Keine Integer-Casts (M-02 = 0), keine Panics (M-08 = 0).

use crate::catalog::reserve::IdReserve;
use crate::catalog::types::{CatalogFileKind, CatalogSpaceOptions, FileMeta};
use crate::codec::tag::TagKind;
use crate::engine::compaction::plan::CompactionPlan;
use crate::engine::compaction::retain::{retain_key_versions, CompactionEntry};
use crate::run::blob_ref::BlobRef;
use crate::run::block::BlockSecurity;
use crate::run::builder::{RunBuilder, RunBuilderOptions};
use crate::run::reader::RunReader;
use contextra_storage_ports::{CompactionFilter, Fs, KeyDomain, KeyProvider, OpenMode, Perm};
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Ergebnis einer ausgeführten Kompaktion vor der Installation.
#[derive(Debug)]
pub struct CompactionExecutionResult {
    /// Neu erzeugte Runs mit Metadaten und Pfad zur `.building`-Datei.
    pub created_runs: Vec<(FileMeta, PathBuf)>,
    /// Neu erzeugte Blobs mit Metadaten und Pfad zur `.building`-Datei.
    pub created_blobs: Vec<(FileMeta, PathBuf)>,
    /// Zu entfernende Eingabedateien.
    pub removed_files: Vec<(CatalogFileKind, u64)>,
    /// Deltas der Blob-Lebendbytes (negativ für verworfene Versionen).
    pub blob_live_deltas: HashMap<u64, i64>,
}

/// Kanonischer Dateiname für einen Run ("RUN-000007").
pub fn run_file_name(id: u64) -> String {
    format!("RUN-{id:06}")
}

/// Führt die Kompaktion gemäß Plan aus.
pub fn execute_compaction(
    fs: &dyn Fs,
    base_dir: &Path,
    plan: &CompactionPlan,
    space_options: &CatalogSpaceOptions,
    id_reserve: &mut IdReserve,
    filter: Option<&dyn CompactionFilter>,
    key_provider: Option<&dyn KeyProvider>,
    now_ms: u64,
) -> Result<CompactionExecutionResult, io::Error> {
    let is_forgetting = (space_options.flags & 16) != 0;
    let header_len = usize::from(space_options.header_len);

    // 1. Sammle alle Einträge aus den Eingabe-Runs
    let mut all_entries: BTreeMap<Vec<u8>, Vec<CompactionEntry>> = BTreeMap::new();
    let mut removed_files = Vec::new();

    for run_meta in &plan.input_runs {
        removed_files.push((CatalogFileKind::Run, run_meta.id));
        let run_path = base_dir.join(run_file_name(run_meta.id));
        let file = fs.open(&run_path, OpenMode::Read)?;
        let file_len = file.len()?;

        let reader = RunReader::open(
            Arc::from(file),
            file_len,
            BlockSecurity::Plain,
            None,
        ).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        // Lese alle Datenblöcke sequenziell
        let index = reader.index();
        let num_blocks = index.entries.len();

        for block_idx in 0..num_blocks {
            let block_data = reader.load_data_block(block_idx)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let db_reader = crate::run::data::DataBlockReader::new(&block_data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

            for entry_res in db_reader.iter() {
                let entry = entry_res
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                let user_key = entry.user_key;
                let tag = entry.tag;
                let val_bytes = entry.value;

                let (header, body, blob_ref) = match tag.kind() {
                    Ok(TagKind::PutRef) => {
                        let h = val_bytes.get(..header_len).unwrap_or(&[]).to_vec();
                        let bref_slice = val_bytes.get(header_len..).unwrap_or(&[]);
                        let bref = BlobRef::decode(bref_slice).ok();
                        (h, Vec::new(), bref)
                    }
                    Ok(TagKind::Put) => {
                        let h = val_bytes.get(..header_len).unwrap_or(&[]).to_vec();
                        let b = val_bytes.get(header_len..).unwrap_or(&[]).to_vec();
                        (h, b, None)
                    }
                    _ => (Vec::new(), Vec::new(), None),
                };

                let centry = CompactionEntry {
                    key: user_key.clone(),
                    tag,
                    header,
                    body,
                    blob_ref,
                };
                all_entries.entry(user_key).or_default().push(centry);
            }
        }
    }

    // 2. Versionen je Schlüssel aufbewahren (retain)
    let mut total_dropped_blob_bytes: u64 = 0;
    let mut surviving_entries: Vec<CompactionEntry> = Vec::new();

    for (_key, mut versions) in all_entries {
        // Sortiere Versionen nach Stamp absteigend
        versions.sort_by(|a, b| b.stamp().cmp(&a.stamp()));

        let retain_res = retain_key_versions(
            versions,
            plan.floor,
            plan.can_drop_tombstones,
            filter,
            now_ms,
            is_forgetting,
        );

        total_dropped_blob_bytes =
            total_dropped_blob_bytes.saturating_add(retain_res.dropped_blob_bytes);

        for mut entry in retain_res.retained {
            // Prüfe Domänenversiegelung: Ist der Domänenschlüssel vernichtet?
            if let Some(ref bref) = entry.blob_ref {
                if bref.domain > 0 {
                    if let Some(kp) = key_provider {
                        if let Ok(None) = kp.domain_key(KeyDomain(bref.domain)) {
                            // Schlüssel vernichtet: BlobRef mit blob_no = 0 (shredded)
                            entry.blob_ref = Some(BlobRef::shredded(bref.domain));
                        }
                    }
                }
            }
            surviving_entries.push(entry);
        }
    }

    // 3. Ausgabe-Runs über den RunBuilder bauen
    let mut created_runs = Vec::new();
    let mut current_run_id = id_reserve.ensure_run_id()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let mut current_builder = create_builder(plan.space, current_run_id, space_options, now_ms);
    let mut current_temp_path = base_dir.join(format!("{}.building", run_file_name(current_run_id)));
    let mut current_approx_bytes: u64 = 0;

    for entry in surviving_entries {
        let key = entry.key.as_slice();
        let tag = entry.tag;

        let val_bytes = match entry.blob_ref {
            Some(ref bref) => {
                let mut v = entry.header.clone();
                v.extend_from_slice(&bref.encode());
                v
            }
            None => {
                let mut v = entry.header.clone();
                v.extend_from_slice(&entry.body);
                v
            }
        };

        let entry_len = u64::try_from(key.len().saturating_add(val_bytes.len())).unwrap_or(0);
        current_approx_bytes = current_approx_bytes.saturating_add(entry_len);

        current_builder.add(key, tag, &val_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        // Teilung bei Erreichen von target_run_bytes
        if current_approx_bytes >= plan.target_run_bytes {
            let (meta, path) = finish_run(
                fs,
                current_builder,
                current_run_id,
                plan.space,
                plan.target_level,
                &current_temp_path,
            )?;
            created_runs.push((meta, path));

            current_run_id = id_reserve.ensure_run_id()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            current_builder = create_builder(plan.space, current_run_id, space_options, now_ms);
            current_temp_path = base_dir.join(format!("{}.building", run_file_name(current_run_id)));
            current_approx_bytes = 0;
        }
    }

    // Letzten Run abschließen (falls Einträge vorhanden)
    if current_approx_bytes > 0 || created_runs.is_empty() {
        let (meta, path) = finish_run(
            fs,
            current_builder,
            current_run_id,
            plan.space,
            plan.target_level,
            &current_temp_path,
        )?;
        created_runs.push((meta, path));
    }

    let mut blob_live_deltas = HashMap::new();
    if total_dropped_blob_bytes > 0 {
        // Deltas negativ für freigegebenen Blob-Speicher
        let dropped_i64 = match i64::try_from(total_dropped_blob_bytes) {
            Ok(v) => -v,
            Err(_) => i64::MIN,
        };
        blob_live_deltas.insert(0, dropped_i64);
    }

    Ok(CompactionExecutionResult {
        created_runs,
        created_blobs: Vec::new(),
        removed_files,
        blob_live_deltas,
    })
}

fn create_builder<'a>(
    space: u16,
    run_id: u64,
    opts: &CatalogSpaceOptions,
    now_ms: u64,
) -> RunBuilder<'a> {
    let pfx = if opts.prefix_filter > 0 {
        Some(usize::from(opts.prefix_filter))
    } else {
        None
    };

    RunBuilder::new(
        RunBuilderOptions {
            space_id: space,
            run_id,
            header_len: u16::from(opts.header_len),
            weights_k: opts.weights_k,
            prefix_filter_len: pfx,
            created_ms: now_ms,
        },
        BlockSecurity::Plain,
    )
}

fn finish_run(
    fs: &dyn Fs,
    builder: RunBuilder<'_>,
    run_id: u64,
    space: u16,
    level: u8,
    temp_path: &Path,
) -> Result<(FileMeta, PathBuf), io::Error> {
    let bytes = builder.finish()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let file = fs.open(temp_path, OpenMode::CreateNew(Perm::Private))?;
    file.write_all_at(&bytes, 0)?;
    file.sync_all()?;

    let len_u64 = u64::try_from(bytes.len()).unwrap_or(0);

    let meta = FileMeta {
        kind: CatalogFileKind::Run,
        id: run_id,
        space,
        level,
        tier: 0,
        flags: 0,
        bytes: len_u64,
        entries: 0,
        generation: 0,
        segment_kind: 0,
        min_stamp: 0,
        max_stamp: 0,
        min_key: Vec::new(),
        max_key: Vec::new(),
        blob_no_min: 0,
        blob_no_max: 0,
    };

    Ok((meta, temp_path.to_path_buf()))
}
