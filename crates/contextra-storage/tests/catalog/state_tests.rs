#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::catalog::edit::CatalogEdit;
use contextra_storage::catalog::state::{apply_edit, CatalogState};
use contextra_storage::catalog::types::*;

#[test]
fn test_catalog_state_apply_edit_pure_and_snapshot() {
    let state0 = CatalogState::default();

    let mut edit1 = CatalogEdit::default();
    edit1.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 1,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 10,
        bytes: 4096,
        entries: 20,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: b"k10".to_vec(),
        max_key: b"k50".to_vec(),
    });
    edit1.add_files.push(FileMeta {
        kind: CatalogFileKind::Blob,
        id: 10,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 10,
        bytes: 65536,
        entries: 5,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: Vec::new(),
        max_key: Vec::new(),
    });
    edit1.reserve = Some((256, 256, 256, 256, 1));
    edit1.space_flushed.push((1, 10));

    // INV-CAT-2: Ein Edit ist ein Record (P-2); er wird ganz oder gar nicht angewendet
    let state1 = apply_edit(&state0, &edit1).expect("apply edit1 succeeds");
    assert_eq!(state1.files.len(), 2);
    assert_eq!(state1.blob_live_bytes.get(&10), Some(&65536));
    assert_eq!(state1.run_id_hwm, 256);
    assert_eq!(state1.space_flushed.get(&1), Some(&10));

    // Snapshot erzeugen und auf leeren Zustand anwenden
    let snapshot_edit = state1.to_snapshot_edit();
    let recovered_state = apply_edit(&state0, &snapshot_edit).expect("apply snapshot succeeds");
    assert_eq!(recovered_state.files, state1.files);
    assert_eq!(recovered_state.blob_live_bytes, state1.blob_live_bytes);
    assert_eq!(recovered_state.run_id_hwm, state1.run_id_hwm);
}

#[test]
fn test_catalog_state_metadata_selection_without_file_access() {
    let mut state = CatalogState::default();

    let mut edit = CatalogEdit::default();
    edit.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 1,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 10,
        bytes: 1000,
        entries: 10,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: b"alpha".to_vec(),
        max_key: b"delta".to_vec(),
    });
    edit.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 2,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 11,
        max_stamp: 20,
        bytes: 2000,
        entries: 20,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: b"hotel".to_vec(),
        max_key: b"omega".to_vec(),
    });
    edit.add_files.push(FileMeta {
        kind: CatalogFileKind::Blob,
        id: 50,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 20,
        bytes: 50000,
        entries: 15,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: Vec::new(),
        max_key: Vec::new(),
    });

    state = apply_edit(&state, &edit).unwrap();

    // 1. Schlüsselbereichs-Filterung (ohne Dateizugriff)
    let in_range = state.runs_in_range(1, b"beta", b"gamma");
    assert_eq!(in_range.len(), 1);
    assert_eq!(in_range[0].id, 1);

    let in_range2 = state.runs_in_range(1, b"mike", b"tango");
    assert_eq!(in_range2.len(), 1);
    assert_eq!(in_range2[0].id, 2);

    let in_range_empty = state.runs_in_range(1, b"echo", b"foxtrot");
    assert!(in_range_empty.is_empty());

    // 2. Stamp-Filterung
    let by_stamp = state.runs_by_stamp_range(1, 15, 25);
    assert_eq!(by_stamp.len(), 1);
    assert_eq!(by_stamp[0].id, 2);

    // 3. Blobs & Gesamtgröße
    assert_eq!(state.live_blobs().len(), 1);
    assert_eq!(state.total_bytes(), 53000);
    assert_eq!(state.blob_live_bytes_for(50), Some(50000));
}

#[test]
fn test_blob_live_delta_zero_removes_blob() {
    let mut state = CatalogState::default();
    let mut e1 = CatalogEdit::default();
    e1.add_files.push(FileMeta {
        kind: CatalogFileKind::Blob,
        id: 99,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 1,
        bytes: 1000,
        entries: 1,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: Vec::new(),
        max_key: Vec::new(),
    });
    state = apply_edit(&state, &e1).unwrap();
    assert!(state.is_file_live(CatalogFileKind::Blob, 99));

    // Delta reduziert Lebendbytes auf 0
    let mut e2 = CatalogEdit::default();
    e2.blob_live_delta.push((99, -1000));
    state = apply_edit(&state, &e2).unwrap();

    // SPEC §9.7: live_bytes == 0 => Blob im selben Edit entfernt
    assert!(!state.is_file_live(CatalogFileKind::Blob, 99));
    assert_eq!(state.blob_live_bytes_for(99), None);
}
