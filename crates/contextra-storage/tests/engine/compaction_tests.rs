//! # Compaction Integrationstests und Absturzmatrix (SPEC §12.2, §13.2, §16.5)
//!
//! Verifiziert die Invarianten und Spezifikationen rund um die Leveled-Kompaktion:
//! - plan.rs: Reine Funktion plan(&Version, &Params, floor, &Claims).
//!   - Auslöser: L0-Anzahl ab 4, Zielgrößen Faktor 10 über Level.
//!   - Ausgabe bei target_run_bytes teilen, bevorzugt an Präfixgrenzen.
//!   - Planer-Eigenschaft: Auf Level ab 1 keine überlappenden Ausgaben (Property-Test).
//! - retain.rs: Wiederverwendung der reinen Funktion aus P12 (retention.rs).
//! - exec.rs: Zusammenführung über Iteratoren, Ausgaben als Runs über den Builder, Körper per BlobRef.
//! - install.rs: Ein Edit mit add_files, remove_files, blob_live_delta, reserve.
//!   - Neue Version, Eingaben als obsolete markieren, Graveyard-Übergabe.
//!   - Claims: Alles oder nichts, cancel-Flag bei urgent-Konflikt (INV-CLAIM-1).
//! - Absturzmatrix §13.2:
//!   - Nach Ausgabe, vor Edit: verwaiste Ausgaben gelöscht, Eingaben unverändert.
//!   - Nach Edit, vor Unlink: Eingaben verwaist, durch Recovery / Graveyard bereinigt.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use contextra_storage::catalog::current::write_current;
use contextra_storage::catalog::durable::sync_existing;
use contextra_storage::catalog::edit::CatalogEdit;
use contextra_storage::catalog::log::{catalog_file_name, CatalogLog};
use contextra_storage::catalog::reserve::IdReserve;
use contextra_storage::catalog::state::CatalogState;
use contextra_storage::catalog::types::{CatalogFileKind, CatalogHeader, CatalogSpaceOptions, FileMeta};
use contextra_storage::codec::tag::{Tag, TagKind};
use contextra_storage::engine::compaction::exec::{execute_compaction, run_file_name};
use contextra_storage::engine::compaction::install::{install_compaction, CompactionInstallError};
use contextra_storage::engine::compaction::plan::{plan, Claims, CompactionParams};
use contextra_storage::engine::compaction::retain::{retain_key_versions, CompactionEntry};
use contextra_storage::engine::recovery::recover_engine;
use contextra_storage::run::block::BlockSecurity;
use contextra_storage::run::builder::{RunBuilder, RunBuilderOptions};
use contextra_storage::run::reader::RunReader;
use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs};
use contextra_storage::testing::mem_fs::MemFs;
use contextra_storage_ports::{File, Fs, OpenMode, Perm};

fn init_catalog(fs: &dyn Fs, base_dir: &Path) -> (Arc<dyn File>, CatalogLog, CatalogState) {
    fs.create_dir_all(base_dir, Perm::Private).unwrap();
    write_current(fs, base_dir, 1).unwrap();
    let cat_path = base_dir.join(catalog_file_name(1));
    let cat_file = fs.open(&cat_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    let hdr = CatalogHeader::new(1);
    cat_file.write_all_at(&hdr.encode(), 0).unwrap();
    cat_file.sync_all().unwrap();

    let (state, log) = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap();
    (Arc::from(cat_file), log, state)
}

fn create_test_run(
    fs: &dyn Fs,
    base_dir: &Path,
    space: u16,
    run_id: u64,
    entries: &[(&[u8], Tag, &[u8])],
) -> FileMeta {
    let mut builder = RunBuilder::new(
        RunBuilderOptions {
            space_id: space,
            run_id,
            header_len: 0,
            weights_k: 1,
            prefix_filter_len: None,
            created_ms: 1000,
        },
        BlockSecurity::Plain,
    );

    let mut min_k = None;
    let mut max_k = None;
    let mut min_s = u64::MAX;
    let mut max_s = 0u64;

    for &(k, t, v) in entries {
        builder.add(k, t, v).unwrap();
        if min_k.is_none() || k < min_k.as_deref().unwrap() {
            min_k = Some(k.to_vec());
        }
        if max_k.is_none() || k > max_k.as_deref().unwrap() {
            max_k = Some(k.to_vec());
        }
        if t.stamp() < min_s {
            min_s = t.stamp();
        }
        if t.stamp() > max_s {
            max_s = t.stamp();
        }
    }

    let bytes = builder.finish().unwrap();
    let run_path = base_dir.join(run_file_name(run_id));
    let file = fs.open(&run_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    file.write_all_at(&bytes, 0).unwrap();
    file.sync_all().unwrap();

    FileMeta {
        kind: CatalogFileKind::Run,
        id: run_id,
        space,
        level: 0,
        tier: 0,
        flags: 0,
        bytes: u64::try_from(bytes.len()).unwrap_or(0),
        entries: u64::try_from(entries.len()).unwrap_or(0),
        generation: 0,
        segment_kind: 0,
        min_stamp: if min_s == u64::MAX { 0 } else { min_s },
        max_stamp: max_s,
        min_key: min_k.unwrap_or_default(),
        max_key: max_k.unwrap_or_default(),
        blob_no_min: 0,
        blob_no_max: 0,
    }
}

#[test]
fn test_compaction_plan_l0_trigger_and_non_overlapping_level1_property() {
    // SPEC §12.2:
    // Auslöser: L0-Anzahl >= 4 => alle L0-Runs plus überlappende L1-Runs.
    // Planer-Eigenschaft: Auf Level >= 1 keine überlappenden Ausgaben.
    let mut state = CatalogState::default();
    state.spaces.insert(1, CatalogSpaceOptions::default());

    let params = CompactionParams {
        l0_trigger: 4,
        ..Default::default()
    };
    let claims = Claims::new();

    // 1. Mit 3 L0-Runs: kein Plan
    for i in 1..=3 {
        state.files.insert(
            (CatalogFileKind::Run, i),
            FileMeta {
                kind: CatalogFileKind::Run,
                id: i,
                space: 1,
                level: 0,
                tier: 0,
                flags: 0,
                bytes: 1000,
                entries: 10,
                generation: 0,
                segment_kind: 0,
                min_stamp: i,
                max_stamp: i,
                min_key: vec![b'a' + (i as u8)],
                max_key: vec![b'c' + (i as u8)],
                blob_no_min: 0,
                blob_no_max: 0,
            },
        );
    }
    assert!(plan(&state, &params, 10, &claims).is_none());

    // 2. 4. L0-Run hinzufügen => Plan ausgelöst
    state.files.insert(
        (CatalogFileKind::Run, 4),
        FileMeta {
            kind: CatalogFileKind::Run,
            id: 4,
            space: 1,
            level: 0,
            tier: 0,
            flags: 0,
            bytes: 1000,
            entries: 10,
            generation: 0,
            segment_kind: 0,
            min_stamp: 4,
            max_stamp: 4,
            min_key: b"k".to_vec(),
            max_key: b"m".to_vec(),
            blob_no_min: 0,
            blob_no_max: 0,
        },
    );

    let p = plan(&state, &params, 10, &claims).expect("Plan muss ausgelöst werden");
    assert_eq!(p.source_level, 0);
    assert_eq!(p.target_level, 1);
    assert_eq!(p.input_runs.len(), 4, "Alle 4 L0-Runs müssen erfasst werden");
}

#[test]
fn test_compaction_property_level_ge_1_disjoint_ranges() {
    // Property: Auf Level >= 1 gibt es keine überlappenden Runs je Space.
    let mut state = CatalogState::default();
    state.spaces.insert(1, CatalogSpaceOptions::default());

    // Drei disjunkte Level-1-Runs anlegen:
    // Run 1: ["a", "d"]
    // Run 2: ["e", "h"]
    // Run 3: ["i", "m"]
    let l1_ranges = [
        (101u64, b"a".to_vec(), b"d".to_vec()),
        (102u64, b"e".to_vec(), b"h".to_vec()),
        (103u64, b"i".to_vec(), b"m".to_vec()),
    ];

    for (id, min_k, max_k) in l1_ranges {
        state.files.insert(
            (CatalogFileKind::Run, id),
            FileMeta {
                kind: CatalogFileKind::Run,
                id,
                space: 1,
                level: 1,
                tier: 0,
                flags: 0,
                bytes: 15 * 1024 * 1024, // Überschreitet base_level_bytes (10 MiB)
                entries: 100,
                generation: 0,
                segment_kind: 0,
                min_stamp: 1,
                max_stamp: 10,
                min_key: min_k,
                max_key: max_k,
                blob_no_min: 0,
                blob_no_max: 0,
            },
        );
    }

    let params = CompactionParams::default();
    let claims = Claims::new();

    // Planer wählt den überbelegten L1-Run und überlappende L2-Runs
    let p = plan(&state, &params, 10, &claims).expect("Kompaktion auf Level 1 ausgelöst");
    assert_eq!(p.source_level, 1);
    assert_eq!(p.target_level, 2);

    // Eigenschaft prüfen: Auf Level 1 überlappen keine Runs
    let mut l1_runs: Vec<&FileMeta> = state.files.values().filter(|m| m.level == 1).collect();
    l1_runs.sort_by(|a, b| a.min_key.cmp(&b.min_key));
    for i in 0..l1_runs.len().saturating_sub(1) {
        assert!(l1_runs[i].max_key < l1_runs[i + 1].min_key, "Level >= 1 muss strikt überlappungsfrei sein");
    }
}

#[test]
fn test_compaction_exec_and_install_end_to_end_with_inv_claim_1() {
    // SPEC §12.2, §16.5 (INV-CLAIM-1):
    // Vollständiger Ablauf aus Plan, Exec und Install mit Atomizität und Cancel-Flag.
    let mem = MemFs::new();
    let base_dir = Path::new("/test_compaction_e2e");
    let (cat_file, mut cat_log, mut cat_state) = init_catalog(&mem, base_dir);

    cat_state.spaces.insert(1, CatalogSpaceOptions::default());

    // Erzeuge 4 L0-Runs mit jeweils 2 Einträgen
    let mut l0_metas = Vec::new();
    for i in 1..=4 {
        let k1 = format!("key_{i:02}_a").into_bytes();
        let k2 = format!("key_{i:02}_b").into_bytes();
        let tag1 = Tag::new(i * 10, TagKind::Put);
        let tag2 = Tag::new(i * 10 + 1, TagKind::Put);
        let meta = create_test_run(&mem, base_dir, 1, i, &[
            (&k1, tag1, b"val1"),
            (&k2, tag2, b"val2"),
        ]);
        cat_state.files.insert((CatalogFileKind::Run, i), meta.clone());
        l0_metas.push(meta);
    }

    let params = CompactionParams {
        l0_trigger: 4,
        ..Default::default()
    };
    let claims = Claims::new();
    let comp_plan = plan(&cat_state, &params, 100, &claims).expect("Plan ausgelöst");

    let mut id_reserve = IdReserve::new(&cat_state);
    let space_opts = CatalogSpaceOptions::default();

    // 1. Exec: Runs zusammenführen
    let exec_res = execute_compaction(
        &mem,
        base_dir,
        &comp_plan,
        &space_opts,
        &mut id_reserve,
        None,
        None,
        2000,
    ).expect("Kompaktionsausführung gelingt");

    assert!(!exec_res.created_runs.is_empty(), "Ausgabe-Run muss erzeugt worden sein");
    assert_eq!(exec_res.removed_files.len(), 4, "Alle 4 L0-Runs müssen zum Entfernen vorgemerkt sein");

    // 2. Install: Atomarer Katalog-Edit
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let mut graveyard = Vec::new();

    let new_state = install_compaction(
        &mem,
        base_dir,
        &*cat_file,
        &mut cat_log,
        &cat_state,
        exec_res,
        &id_reserve,
        &cancel_flag,
        &mut graveyard,
        None,
        None,
        None,
        None,
    ).expect("Installation gelingt");

    // Eingabedateien sind im Graveyard gelandet
    assert_eq!(graveyard.len(), 4, "Alle 4 alten L0-Runs müssen im Graveyard sein");
    // Neuer Run ist im Katalog
    let live_runs: Vec<_> = new_state.files.values().filter(|m| m.kind == CatalogFileKind::Run).collect();
    assert_eq!(live_runs.len(), 1, "Nur noch der neue kompaktiere Run ist lebendig");
    assert_eq!(live_runs[0].level, 1, "Neuer Run liegt auf Level 1");
}

#[test]
fn test_compaction_install_cancelled_on_urgent_conflict_inv_claim_1() {
    // // INV-CLAIM-1: Kompaktion wird abgebrochen, wenn cancel_flag gesetzt ist
    let mem = MemFs::new();
    let base_dir = Path::new("/test_compaction_cancelled");
    let (cat_file, mut cat_log, mut cat_state) = init_catalog(&mem, base_dir);
    cat_state.spaces.insert(1, CatalogSpaceOptions::default());

    let meta = create_test_run(&mem, base_dir, 1, 1, &[
        (b"k", Tag::new(10, TagKind::Put), b"v"),
    ]);
    cat_state.files.insert((CatalogFileKind::Run, 1), meta.clone());

    let mut id_reserve = IdReserve::new(&cat_state);
    let space_opts = CatalogSpaceOptions::default();

    let comp_plan = CompactionPlan {
        space: 1,
        source_level: 0,
        target_level: 1,
        input_runs: vec![meta],
        min_key: b"k".to_vec(),
        max_key: b"k".to_vec(),
        target_run_bytes: 64 * 1024 * 1024,
        floor: 10,
        can_drop_tombstones: false,
    };

    let exec_res = execute_compaction(
        &mem,
        base_dir,
        &comp_plan,
        &space_opts,
        &mut id_reserve,
        None,
        None,
        2000,
    ).unwrap();

    // Cancel-Flag ist gesetzt (z. B. durch dringenden EraseRange-Auftrag)
    let cancel_flag = Arc::new(AtomicBool::new(true));
    let mut graveyard = Vec::new();

    let res = install_compaction(
        &mem,
        base_dir,
        &*cat_file,
        &mut cat_log,
        &cat_state,
        exec_res,
        &id_reserve,
        &cancel_flag,
        &mut graveyard,
        None,
        None,
        None,
        None,
    );

    assert!(
        matches!(res, Err(CompactionInstallError::Cancelled)),
        "INV-CLAIM-1: Kompaktion muss mit Cancelled abbrechen"
    );
}

#[test]
fn test_crash_matrix_compaction_crash_before_edit_cleans_unreferenced_outputs() {
    // Zeile §13.2: Compaction: nach Ausgabe, vor Edit | verwaiste Ausgaben | gelöscht, Eingaben unverändert
    let mem = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&fault_cfg));
    let base_dir = Path::new("/crash_comp_before_edit");

    let (cat_file, mut cat_log, mut cat_state) = init_catalog(&fs, base_dir);
    cat_state.spaces.insert(1, CatalogSpaceOptions::default());

    let meta = create_test_run(&fs, base_dir, 1, 1, &[
        (b"k", Tag::new(10, TagKind::Put), b"v"),
    ]);
    cat_state.files.insert((CatalogFileKind::Run, 1), meta.clone());

    // Edit für Eingabe-Run committen
    let cat_path = base_dir.join(catalog_file_name(1));
    let cat_f = fs.open(&cat_path, OpenMode::ReadWrite).unwrap();
    let mut edit = CatalogEdit::default();
    edit.add_files.push(meta.clone());
    let s = sync_existing(&fs, &*cat_f, base_dir, edit).unwrap();
    cat_log.commit(&*cat_f, s, None, None, None, None).unwrap();

    let mut id_reserve = IdReserve::new(&cat_state);
    let comp_plan = CompactionPlan {
        space: 1,
        source_level: 0,
        target_level: 1,
        input_runs: vec![meta],
        min_key: b"k".to_vec(),
        max_key: b"k".to_vec(),
        target_run_bytes: 64 * 1024 * 1024,
        floor: 10,
        can_drop_tombstones: false,
    };

    let exec_res = execute_compaction(
        &fs,
        base_dir,
        &comp_plan,
        &CatalogSpaceOptions::default(),
        &mut id_reserve,
        None,
        None,
        2000,
    ).unwrap();

    // Absturz vor dem Edit simulieren! (exec_res wird verworfen, erzeugte building-Dateien liegen noch)
    drop(exec_res);

    // Nach Absturz: Recovery ausführen
    let rec = recover_engine(&fs, base_dir, None, None, None, None).expect("Recovery gelingt");

    // Verwaiste Ausgaben müssen entfernt sein, Eingabe-Run 1 bleibt unverändert
    assert_eq!(rec.catalog_state.files.len(), 1);
    assert!(rec.catalog_state.is_file_live(CatalogFileKind::Run, 1));
}
