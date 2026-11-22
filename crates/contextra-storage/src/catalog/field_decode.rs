//! # Feld-Dekodierer für Katalog-TLV
//!
//! SPEC §9.7: Spezifische Dekodierer für alle 20 Katalog-Felder.

use crate::catalog::codec::*;
use crate::catalog::edit::CatalogEdit;
use crate::catalog::types::{
    CatalogError, CatalogFileKind, CatalogSpaceOptions, ErasePendingMeta, ErasedRangeMeta,
    FileMeta, HoldMeta,
};

pub(crate) fn decode_add_files(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let kind_u8 = read_u8(&mut p)?;
        let kind = CatalogFileKind::from_u8(kind_u8)
            .ok_or(CatalogError::InvalidFormat("Ungültige FileKind"))?;
        let id = read_u64(&mut p)?;
        let space = read_u16(&mut p)?;
        let level = read_u8(&mut p)?;
        let tier = read_u8(&mut p)?;
        let flags = read_u16(&mut p)?;
        let min_stamp = read_u64(&mut p)?;
        let max_stamp = read_u64(&mut p)?;
        let bytes = read_u64(&mut p)?;
        let entries = read_u64(&mut p)?;
        let generation = read_u32(&mut p)?;
        let segment_kind = read_u16(&mut p)?;
        let blob_no_min = read_u64(&mut p)?;
        let blob_no_max = read_u64(&mut p)?;
        let min_key = read_vec(&mut p)?;
        let max_key = read_vec(&mut p)?;
        edit.add_files.push(FileMeta {
            kind, id, space, level, tier, flags, min_stamp, max_stamp, bytes, entries,
            generation, segment_kind, blob_no_min, blob_no_max, min_key, max_key,
        });
    }
    Ok(())
}

pub(crate) fn decode_remove_files(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let k_u8 = read_u8(&mut p)?;
        let kind = CatalogFileKind::from_u8(k_u8).ok_or(CatalogError::InvalidFormat("Ungültige FileKind"))?;
        let id = read_u64(&mut p)?;
        edit.remove_files.push((kind, id));
    }
    Ok(())
}

pub(crate) fn decode_blob_live_delta(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let b = read_u64(&mut p)?;
        let d = read_i64(&mut p)?;
        edit.blob_live_delta.push((b, d));
    }
    Ok(())
}

pub(crate) fn decode_blob_replace(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let b = read_u64(&mut p)?;
        let ga = read_u32(&mut p)?;
        let gn = read_u32(&mut p)?;
        edit.blob_replace.push((b, ga, gn));
    }
    Ok(())
}

pub(crate) fn decode_space_flushed(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let s = read_u16(&mut p)?;
        let st = read_u64(&mut p)?;
        edit.space_flushed.push((s, st));
    }
    Ok(())
}

pub(crate) fn decode_redo(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let f = read_u64(&mut p)?;
    let fs = read_u64(&mut p)?;
    let h = read_u64(&mut p)?;
    edit.redo = Some((f, fs, h));
    Ok(())
}

pub(crate) fn decode_reserve(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let r = read_u64(&mut p)?;
    let b = read_u64(&mut p)?;
    let s = read_u64(&mut p)?;
    let t = read_u64(&mut p)?;
    let k = read_u64(&mut p)?;
    edit.reserve = Some((r, b, s, t, k));
    Ok(())
}

pub(crate) fn decode_spaces_add(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let id = read_u16(&mut p)?;
        let name = read_vec(&mut p)?;
        let header_len = read_u8(&mut p)?;
        let weights_k = read_u8(&mut p)?;
        let flags = read_u16(&mut p)?;
        let separate_threshold = read_u32(&mut p)?;
        let prefix_filter = read_u8(&mut p)?;
        let locality_prefix = read_u8(&mut p)?;
        edit.spaces_add.push((id, name, CatalogSpaceOptions {
            header_len, weights_k, flags, separate_threshold, prefix_filter, locality_prefix,
        }));
    }
    Ok(())
}

pub(crate) fn decode_spaces_drop(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        edit.spaces_drop.push(read_u16(&mut p)?);
    }
    Ok(())
}

pub(crate) fn decode_consumer_set(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let name = read_vec(&mut p)?;
        let stamp = read_u64(&mut p)?;
        let max_lag = read_u64(&mut p)?;
        edit.consumer_set.push((name, stamp, max_lag));
    }
    Ok(())
}

pub(crate) fn decode_consumer_drop(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        edit.consumer_drop.push(read_vec(&mut p)?);
    }
    Ok(())
}

pub(crate) fn decode_domain_retired(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let d = read_u32(&mut p)?;
        let st = read_u64(&mut p)?;
        edit.domain_retired.push((d, st));
    }
    Ok(())
}

pub(crate) fn decode_erase_pending_add(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let id = read_u64(&mut p)?;
        let space = read_u16(&mut p)?;
        let (range, c_r) = get_range(p)?;
        let (_, r) = p.split_at(c_r);
        p = r;
        let level = read_u8(&mut p)?;
        edit.erase_pending_add.push(ErasePendingMeta { id, space, range, level });
    }
    Ok(())
}

pub(crate) fn decode_erase_done(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        edit.erase_done.push(read_u64(&mut p)?);
    }
    Ok(())
}

pub(crate) fn decode_erase_swept(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        edit.erase_swept.push(read_u64(&mut p)?);
    }
    Ok(())
}

pub(crate) fn decode_erased_ranges_add(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let space = read_u16(&mut p)?;
        let (range, c_r) = get_range(p)?;
        let (_, r) = p.split_at(c_r);
        p = r;
        let stamp_s = read_u64(&mut p)?;
        edit.erased_ranges_add.push(ErasedRangeMeta { space, range, stamp_s });
    }
    Ok(())
}

pub(crate) fn decode_erased_ranges_remove(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let space = read_u16(&mut p)?;
        let (range, c_r) = get_range(p)?;
        let (_, r) = p.split_at(c_r);
        p = r;
        let stamp_s = read_u64(&mut p)?;
        edit.erased_ranges_remove.push(ErasedRangeMeta { space, range, stamp_s });
    }
    Ok(())
}

pub(crate) fn decode_hold_set(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let hold_id = read_u64(&mut p)?;
        let stamp = read_u64(&mut p)?;
        let owner = read_vec(&mut p)?;
        let expires_ms = read_u64(&mut p)?;
        edit.hold_set.push(HoldMeta { hold_id, stamp, owner, expires_ms });
    }
    Ok(())
}

pub(crate) fn decode_hold_drop(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        edit.hold_drop.push(read_u64(&mut p)?);
    }
    Ok(())
}

pub(crate) fn decode_key_epoch(edit: &mut CatalogEdit, mut p: &[u8]) -> Result<(), CatalogError> {
    let (cnt_u64, c) = get_varint(p)?;
    let (_, r) = p.split_at(c);
    p = r;
    let cnt = usize::try_from(cnt_u64).map_err(|_| CatalogError::InvalidFormat("Überlauf"))?;
    for _ in 0..cnt {
        let s = read_u16(&mut p)?;
        let ep = read_u32(&mut p)?;
        edit.key_epoch.push((s, ep));
    }
    Ok(())
}
