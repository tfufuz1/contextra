//! # Feld-Kodierer für Katalog-TLV
//!
//! SPEC §9.7: Spezifische Kodierer für alle 20 Katalog-Felder.

use crate::catalog::codec::*;
use crate::catalog::edit::*;

fn write_tlv(out: &mut Vec<u8>, field_id: u16, payload: &[u8]) {
    if payload.is_empty() {
        return;
    }
    out.extend_from_slice(&field_id.to_le_bytes());
    let len_u64 = u64::try_from(payload.len()).unwrap_or(0);
    put_varint(out, len_u64);
    out.extend_from_slice(payload);
}

/// Kodiert den Edit in die TLV-Bytes eines Records (SPEC §9.7).
pub fn encode_edit(edit: &CatalogEdit) -> Vec<u8> {
    let mut out = Vec::new();

    // 0x0001 add_files
    if !edit.add_files.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.add_files.len()).unwrap_or(0));
        for f in &edit.add_files {
            p.push(f.kind.to_u8());
            p.extend_from_slice(&f.id.to_le_bytes());
            p.extend_from_slice(&f.space.to_le_bytes());
            p.push(f.level);
            p.push(f.tier);
            p.extend_from_slice(&f.flags.to_le_bytes());
            p.extend_from_slice(&f.min_stamp.to_le_bytes());
            p.extend_from_slice(&f.max_stamp.to_le_bytes());
            p.extend_from_slice(&f.bytes.to_le_bytes());
            p.extend_from_slice(&f.entries.to_le_bytes());
            p.extend_from_slice(&f.generation.to_le_bytes());
            p.extend_from_slice(&f.segment_kind.to_le_bytes());
            p.extend_from_slice(&f.blob_no_min.to_le_bytes());
            p.extend_from_slice(&f.blob_no_max.to_le_bytes());
            put_bytes(&mut p, &f.min_key);
            put_bytes(&mut p, &f.max_key);
        }
        write_tlv(&mut out, FIELD_ADD_FILES, &p);
    }

    // 0x0002 remove_files
    if !edit.remove_files.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.remove_files.len()).unwrap_or(0));
        for (k, id) in &edit.remove_files {
            p.push(k.to_u8());
            p.extend_from_slice(&id.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_REMOVE_FILES, &p);
    }

    // 0x0003 blob_live_delta
    if !edit.blob_live_delta.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.blob_live_delta.len()).unwrap_or(0));
        for (b, d) in &edit.blob_live_delta {
            p.extend_from_slice(&b.to_le_bytes());
            p.extend_from_slice(&d.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_BLOB_LIVE_DELTA, &p);
    }

    // 0x0004 blob_replace
    if !edit.blob_replace.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.blob_replace.len()).unwrap_or(0));
        for (b, ga, gn) in &edit.blob_replace {
            p.extend_from_slice(&b.to_le_bytes());
            p.extend_from_slice(&ga.to_le_bytes());
            p.extend_from_slice(&gn.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_BLOB_REPLACE, &p);
    }

    // 0x0005 space_flushed
    if !edit.space_flushed.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.space_flushed.len()).unwrap_or(0));
        for (s, st) in &edit.space_flushed {
            p.extend_from_slice(&s.to_le_bytes());
            p.extend_from_slice(&st.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_SPACE_FLUSHED, &p);
    }

    // 0x0006 redo
    if let Some((floor, floor_st, hwm)) = edit.redo {
        let mut p = Vec::new();
        p.extend_from_slice(&floor.to_le_bytes());
        p.extend_from_slice(&floor_st.to_le_bytes());
        p.extend_from_slice(&hwm.to_le_bytes());
        write_tlv(&mut out, FIELD_REDO, &p);
    }

    // 0x0007 reserve
    if let Some((r_hwm, b_hwm, s_hwm, t_hwm, k_gen)) = edit.reserve {
        let mut p = Vec::new();
        p.extend_from_slice(&r_hwm.to_le_bytes());
        p.extend_from_slice(&b_hwm.to_le_bytes());
        p.extend_from_slice(&s_hwm.to_le_bytes());
        p.extend_from_slice(&t_hwm.to_le_bytes());
        p.extend_from_slice(&k_gen.to_le_bytes());
        write_tlv(&mut out, FIELD_RESERVE, &p);
    }

    // 0x0008 spaces_add
    if !edit.spaces_add.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.spaces_add.len()).unwrap_or(0));
        for (id, name, opt) in &edit.spaces_add {
            p.extend_from_slice(&id.to_le_bytes());
            put_bytes(&mut p, name);
            p.push(opt.header_len);
            p.push(opt.weights_k);
            p.extend_from_slice(&opt.flags.to_le_bytes());
            p.extend_from_slice(&opt.separate_threshold.to_le_bytes());
            p.push(opt.prefix_filter);
            p.push(opt.locality_prefix);
        }
        write_tlv(&mut out, FIELD_SPACES_ADD, &p);
    }

    // 0x0009 spaces_drop
    if !edit.spaces_drop.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.spaces_drop.len()).unwrap_or(0));
        for s in &edit.spaces_drop {
            p.extend_from_slice(&s.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_SPACES_DROP, &p);
    }

    // 0x000A consumer_set
    if !edit.consumer_set.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.consumer_set.len()).unwrap_or(0));
        for (name, c_stamp, max_lag) in &edit.consumer_set {
            put_bytes(&mut p, name);
            p.extend_from_slice(&c_stamp.to_le_bytes());
            p.extend_from_slice(&max_lag.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_CONSUMER_SET, &p);
    }

    // 0x000B consumer_drop
    if !edit.consumer_drop.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.consumer_drop.len()).unwrap_or(0));
        for name in &edit.consumer_drop {
            put_bytes(&mut p, name);
        }
        write_tlv(&mut out, FIELD_CONSUMER_DROP, &p);
    }

    // 0x000C domain_retired
    if !edit.domain_retired.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.domain_retired.len()).unwrap_or(0));
        for (d, st) in &edit.domain_retired {
            p.extend_from_slice(&d.to_le_bytes());
            p.extend_from_slice(&st.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_DOMAIN_RETIRED, &p);
    }

    // 0x000D erase_pending_add
    if !edit.erase_pending_add.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.erase_pending_add.len()).unwrap_or(0));
        for ep in &edit.erase_pending_add {
            p.extend_from_slice(&ep.id.to_le_bytes());
            p.extend_from_slice(&ep.space.to_le_bytes());
            put_range(&mut p, &ep.range);
            p.push(ep.level);
        }
        write_tlv(&mut out, FIELD_ERASE_PENDING_ADD, &p);
    }

    // 0x000E erase_done
    if !edit.erase_done.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.erase_done.len()).unwrap_or(0));
        for id in &edit.erase_done {
            p.extend_from_slice(&id.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_ERASE_DONE, &p);
    }

    // 0x000F erase_swept
    if !edit.erase_swept.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.erase_swept.len()).unwrap_or(0));
        for id in &edit.erase_swept {
            p.extend_from_slice(&id.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_ERASE_SWEPT, &p);
    }

    // 0x0010 erased_ranges_add
    if !edit.erased_ranges_add.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.erased_ranges_add.len()).unwrap_or(0));
        for er in &edit.erased_ranges_add {
            p.extend_from_slice(&er.space.to_le_bytes());
            put_range(&mut p, &er.range);
            p.extend_from_slice(&er.stamp_s.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_ERASED_RANGES_ADD, &p);
    }

    // 0x0011 erased_ranges_remove
    if !edit.erased_ranges_remove.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.erased_ranges_remove.len()).unwrap_or(0));
        for er in &edit.erased_ranges_remove {
            p.extend_from_slice(&er.space.to_le_bytes());
            put_range(&mut p, &er.range);
            p.extend_from_slice(&er.stamp_s.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_ERASED_RANGES_REMOVE, &p);
    }

    // 0x0012 hold_set
    if !edit.hold_set.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.hold_set.len()).unwrap_or(0));
        for h in &edit.hold_set {
            p.extend_from_slice(&h.hold_id.to_le_bytes());
            p.extend_from_slice(&h.stamp.to_le_bytes());
            put_bytes(&mut p, &h.owner);
            p.extend_from_slice(&h.expires_ms.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_HOLD_SET, &p);
    }

    // 0x0013 hold_drop
    if !edit.hold_drop.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.hold_drop.len()).unwrap_or(0));
        for h in &edit.hold_drop {
            p.extend_from_slice(&h.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_HOLD_DROP, &p);
    }

    // 0x0014 key_epoch
    if !edit.key_epoch.is_empty() {
        let mut p = Vec::new();
        put_varint(&mut p, u64::try_from(edit.key_epoch.len()).unwrap_or(0));
        for (s, ep) in &edit.key_epoch {
            p.extend_from_slice(&s.to_le_bytes());
            p.extend_from_slice(&ep.to_le_bytes());
        }
        write_tlv(&mut out, FIELD_KEY_EPOCH, &p);
    }

    out
}
