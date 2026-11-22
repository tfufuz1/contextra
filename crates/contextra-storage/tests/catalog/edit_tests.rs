#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::catalog::edit::*;
use contextra_storage::catalog::types::*;

#[test]
fn test_catalog_edit_all_fields_tlv_roundtrip() {
    let mut edit = CatalogEdit::default();

    // 0x0001: add_files
    edit.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 101,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 10,
        max_stamp: 20,
        bytes: 8192,
        entries: 50,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 5,
        blob_no_max: 8,
        min_key: b"a".to_vec(),
        max_key: b"z".to_vec(),
    });

    // 0x0002: remove_files
    edit.remove_files.push((CatalogFileKind::Run, 99));

    // 0x0003: blob_live_delta
    edit.blob_live_delta.push((5, -100));

    // 0x0004: blob_replace
    edit.blob_replace.push((5, 1, 2));

    // 0x0005: space_flushed
    edit.space_flushed.push((1, 20));

    // 0x0006: redo
    edit.redo = Some((10, 10, 50));

    // 0x0007: reserve
    edit.reserve = Some((256, 256, 256, 256, 1));

    // 0x0008: spaces_add
    edit.spaces_add.push((1, b"default".to_vec(), CatalogSpaceOptions {
        header_len: 8,
        weights_k: 2,
        flags: 1,
        separate_threshold: 4096,
        prefix_filter: 4,
        locality_prefix: 2,
    }));

    // 0x0009: spaces_drop
    edit.spaces_drop.push(2);

    // 0x000A: consumer_set
    edit.consumer_set.push((b"indexer".to_vec(), 15, 1024 * 1024));

    // 0x000B: consumer_drop
    edit.consumer_drop.push(b"old_consumer".to_vec());

    // 0x000C: domain_retired
    edit.domain_retired.push((42, 100));

    // 0x000D: erase_pending_add
    edit.erase_pending_add.push(ErasePendingMeta {
        id: 77,
        space: 1,
        range: CatalogKeyRange {
            start: b"del_start".to_vec(),
            end: Some(b"del_end".to_vec()),
        },
        level: 2,
    });

    // 0x000E: erase_done
    edit.erase_done.push(77);

    // 0x000F: erase_swept
    edit.erase_swept.push(77);

    // 0x0010: erased_ranges_add
    edit.erased_ranges_add.push(ErasedRangeMeta {
        space: 1,
        range: CatalogKeyRange { start: b"k1".to_vec(), end: None },
        stamp_s: 77,
    });

    // 0x0011: erased_ranges_remove
    edit.erased_ranges_remove.push(ErasedRangeMeta {
        space: 1,
        range: CatalogKeyRange { start: b"k1".to_vec(), end: None },
        stamp_s: 77,
    });

    // 0x0012: hold_set
    edit.hold_set.push(HoldMeta {
        hold_id: 1,
        stamp: 10,
        owner: b"backup-job".to_vec(),
        expires_ms: 1700000000000,
    });

    // 0x0013: hold_drop
    edit.hold_drop.push(1);

    // 0x0014: key_epoch
    edit.key_epoch.push((1, 3));

    let encoded = edit.encode();
    assert!(!encoded.is_empty());

    let decoded = CatalogEdit::decode(&encoded).expect("decode all fields succeeds");
    assert_eq!(decoded, edit);
}

#[test]
fn test_catalog_optional_and_unknown_fields() {
    // Unbekanntes optionales Feld (Bit 0x8000 gesetzt): muss ignoriert werden
    let mut payload = Vec::new();
    let opt_field_id = 0x9FFF_u16; // 0x8000 | 0x1FFF
    payload.extend_from_slice(&opt_field_id.to_le_bytes());
    payload.push(3); // len varint = 3
    payload.extend_from_slice(b"xyz");

    // Gültiges Feld anhängen (0x0009 spaces_drop)
    let drop_edit = CatalogEdit { spaces_drop: vec![5], ..Default::default() };
    payload.extend_from_slice(&drop_edit.encode());

    let decoded = CatalogEdit::decode(&payload).expect("optional unknown field is skipped");
    assert_eq!(decoded.spaces_drop, vec![5]);

    // Unbekanntes Pflichtfeld (Bit 0x8000 nicht gesetzt): muss zu Unsupported führen
    let mut bad_payload = Vec::new();
    let req_field_id = 0x0FFF_u16;
    bad_payload.extend_from_slice(&req_field_id.to_le_bytes());
    bad_payload.push(2);
    bad_payload.extend_from_slice(b"ab");

    let err = CatalogEdit::decode(&bad_payload).unwrap_err();
    assert_eq!(err, CatalogError::UnsupportedField(req_field_id));
}
