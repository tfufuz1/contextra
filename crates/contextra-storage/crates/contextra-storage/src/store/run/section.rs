//! Entry-level ClusterSection builder, sealed point/range reader, and raw CRC verifier (SP-RUN WP-07.3..07.5).
#![forbid(unsafe_code)]

use super::block::{self, BlockEntry, MAX_DECODED_BLOCK_BYTES, TARGET_BLOCK_BYTES};
use super::copy::RawSectionChecksumVerifier;
use super::filter::{BloomFilter, MAX_FILTER_BYTES};
use super::rangedel::{self, RangeMarker};
use super::skip_scan::{block_skip_scan, BlockSkipQuery};
use super::subindex::{SubIndex, SubIndexEntry};
use crate::codec::checksums::crc32c;
use crate::codec::seal::{seal_frame, unseal_frame, Aad, SealMode};
use crate::keys::cache::Key32Handle;
use contextra_storage_ports::traits::{Checksum, Cipher, Entropy, Mac};
use contextra_storage_ports::{BranchId, ClusterId, Error, Key32, SpaceId, Stamp};
use zeroize::Zeroizing;

const SECTION_MAGIC: [u8; 8] = *b"CTXSEC01";
const SECTION_HEADER_LEN: usize = 8 + 1 + 4 + 4; // magic + seal_mode + key_epoch + block_count
const SECTION_TRAILER_LEN: usize = 8 + 4 + 8 + 4 + 8 + 4 + 4; // subindex(off,len) + filter(off,len) + rangedel(off,len) + crc

pub struct SectionSealContext<'a> {
    pub instance_id: &'a [u8; 16],
    pub space: SpaceId,
    pub cluster: ClusterId,
    pub key_epoch: u32,
    pub mode: SealMode,
    pub key: Option<&'a Key32>,
    pub cipher: &'a dyn Cipher,
    pub mac: &'a dyn Mac,
    pub checksum: &'a dyn Checksum,
    pub entropy: &'a dyn Entropy,
}

pub struct EncodedSection {
    pub bytes: Vec<u8>,
    pub block_count: u32,
    pub entry_count: u64,
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
}

#[derive(Debug, Clone, Copy)]
pub struct SectionLayout {
    pub mode: SealMode,
    pub key_epoch: u32,
    pub block_count: u32,
    pub subindex_offset: u64,
    pub subindex_len: u32,
    pub filter_offset: u64,
    pub filter_len: u32,
    pub rangedel_offset: u64,
    pub rangedel_len: u32,
}

/// Format-aware CRC verifier for `copy_sections` that validates section framing
/// and CRC32C without any decryption key (SPEC §9.5.1, INV-SEAL-1).
pub struct RunChecksumVerifier;

impl RawSectionChecksumVerifier for RunChecksumVerifier {
    fn verify_raw_checksums(&self, section: &[u8]) -> Result<(), Error> {
        let _ = parse_section_layout(section)?;
        Ok(())
    }
}

/// Encodes sorted entries and optional range-delete markers into a single
/// position-independent `ClusterSection` (all offsets section-relative).
pub fn encode_cluster_section(
    ctx: &SectionSealContext<'_>,
    entries: &[BlockEntry],
    range_markers: &[RangeMarker],
) -> Result<EncodedSection, Error> {
    if entries.is_empty() {
        return Err(Error::Limit {
            limit: "cluster section requires at least one entry",
        });
    }

    let mut out = Vec::new();
    out.extend_from_slice(&SECTION_MAGIC);
    out.push(seal_mode_to_wire(ctx.mode));
    out.extend_from_slice(&ctx.key_epoch.to_le_bytes());
    let block_count_pos = out.len();
    out.extend_from_slice(&0u32.to_le_bytes());

    let mut bloom_salt = [0u8; 16];
    ctx.entropy.fill(&mut bloom_salt)?;
    let mut filter = BloomFilter::new(entries.len().max(1) as u32, 0.01, bloom_salt)?;
    for e in entries {
        filter.insert(&e.key);
    }

    let mut subindex_entries = Vec::new();
    let mut idx = 0usize;
    let mut section_min_stamp = Stamp::MAX;
    let mut section_max_stamp = 0u64;

    while idx < entries.len() {
        let mut chunk_end = idx;
        let mut approx_bytes = 64usize;
        while chunk_end < entries.len() {
            let e = &entries[chunk_end];
            let e_bytes = 32 + e.key.len() + e.header.len() + e.body.len();
            if chunk_end > idx && approx_bytes.saturating_add(e_bytes) > TARGET_BLOCK_BYTES {
                break;
            }
            approx_bytes = approx_bytes.saturating_add(e_bytes);
            chunk_end += 1;
        }
        let chunk = &entries[idx..chunk_end];
        let raw_block = block::encode(chunk, MAX_DECODED_BLOCK_BYTES)?;
        let mut block_uid = [0u8; 16];
        ctx.entropy.fill(&mut block_uid)?;
        let mut nonce = [0u8; 24];
        ctx.entropy.fill(&mut nonce)?;

        let aad = Aad::Block {
            instance: ctx.instance_id,
            space: ctx.space,
            cluster: ctx.cluster,
            key_epoch: ctx.key_epoch,
            block_uid: &block_uid,
        };
        let sealed_block = seal_frame(
            ctx.mode,
            ctx.key_epoch,
            ctx.key,
            &nonce,
            aad,
            &raw_block,
            ctx.cipher,
            ctx.mac,
            ctx.checksum,
        )?;

        let block_offset = out.len() as u64;
        let sealed_len = u32::try_from(sealed_block.len())
            .map_err(|_| Error::Limit { limit: "sealed block length" })?;
        out.extend_from_slice(&sealed_len.to_le_bytes());
        out.extend_from_slice(&sealed_block);

        let mut min_s = Stamp::MAX;
        let mut max_s = 0u64;
        for e in chunk {
            let s = e.stamp();
            if s < min_s {
                min_s = s;
            }
            if s > max_s {
                max_s = s;
            }
        }
        if min_s < section_min_stamp {
            section_min_stamp = min_s;
        }
        if max_s > section_max_stamp {
            section_max_stamp = max_s;
        }
        let last_key = Zeroizing::new(chunk[chunk.len() - 1].key.to_vec());
        subindex_entries.push(SubIndexEntry {
            last_key,
            offset: block_offset,
            len: sealed_len + 4,
            entries: chunk.len() as u32,
            weight_sum: [chunk.len() as u64, 0, 0, 0],
            min_stamp: min_s,
            max_stamp: max_s,
            block_uid,
        });
        idx = chunk_end;
    }

    let block_count = u32::try_from(subindex_entries.len())
        .map_err(|_| Error::Limit { limit: "section block count" })?;
    out[block_count_pos..block_count_pos + 4].copy_from_slice(&block_count.to_le_bytes());

    let subindex = SubIndex::new(subindex_entries)?;
    let raw_subindex = subindex.encode(MAX_DECODED_BLOCK_BYTES)?;
    let subindex_uid = meta_uid(1);
    let mut nonce = [0u8; 24];
    ctx.entropy.fill(&mut nonce)?;
    let sealed_subindex = seal_frame(
        ctx.mode,
        ctx.key_epoch,
        ctx.key,
        &nonce,
        Aad::Block {
            instance: ctx.instance_id,
            space: ctx.space,
            cluster: ctx.cluster,
            key_epoch: ctx.key_epoch,
            block_uid: &subindex_uid,
        },
        &raw_subindex,
        ctx.cipher,
        ctx.mac,
        ctx.checksum,
    )?;
    let subindex_offset = out.len() as u64;
    let subindex_len = u32::try_from(sealed_subindex.len())
        .map_err(|_| Error::Limit { limit: "sealed subindex length" })?;
    out.extend_from_slice(&sealed_subindex);

    let raw_filter = filter.encode()?;
    let filter_uid = meta_uid(2);
    ctx.entropy.fill(&mut nonce)?;
    let sealed_filter = seal_frame(
        ctx.mode,
        ctx.key_epoch,
        ctx.key,
        &nonce,
        Aad::Block {
            instance: ctx.instance_id,
            space: ctx.space,
            cluster: ctx.cluster,
            key_epoch: ctx.key_epoch,
            block_uid: &filter_uid,
        },
        &raw_filter,
        ctx.cipher,
        ctx.mac,
        ctx.checksum,
    )?;
    let filter_offset = out.len() as u64;
    let filter_len = u32::try_from(sealed_filter.len())
        .map_err(|_| Error::Limit { limit: "sealed filter length" })?;
    out.extend_from_slice(&sealed_filter);

    let raw_rangedel = rangedel::encode(range_markers, MAX_DECODED_BLOCK_BYTES)?;
    let rangedel_uid = meta_uid(3);
    ctx.entropy.fill(&mut nonce)?;
    let sealed_rangedel = seal_frame(
        ctx.mode,
        ctx.key_epoch,
        ctx.key,
        &nonce,
        Aad::Block {
            instance: ctx.instance_id,
            space: ctx.space,
            cluster: ctx.cluster,
            key_epoch: ctx.key_epoch,
            block_uid: &rangedel_uid,
        },
        &raw_rangedel,
        ctx.cipher,
        ctx.mac,
        ctx.checksum,
    )?;
    let rangedel_offset = out.len() as u64;
    let rangedel_len = u32::try_from(sealed_rangedel.len())
        .map_err(|_| Error::Limit { limit: "sealed rangedel length" })?;
    out.extend_from_slice(&sealed_rangedel);

    out.extend_from_slice(&subindex_offset.to_le_bytes());
    out.extend_from_slice(&subindex_len.to_le_bytes());
    out.extend_from_slice(&filter_offset.to_le_bytes());
    out.extend_from_slice(&filter_len.to_le_bytes());
    out.extend_from_slice(&rangedel_offset.to_le_bytes());
    out.extend_from_slice(&rangedel_len.to_le_bytes());
    let crc = crc32c(&out);
    out.extend_from_slice(&crc.to_le_bytes());

    Ok(EncodedSection {
        bytes: out,
        block_count,
        entry_count: entries.len() as u64,
        min_stamp: section_min_stamp,
        max_stamp: section_max_stamp,
    })
}

pub fn parse_section_layout(section: &[u8]) -> Result<SectionLayout, Error> {
    if section.len() < SECTION_HEADER_LEN + SECTION_TRAILER_LEN {
        return Err(Error::Corrupt {
            place: "cluster section too small",
        });
    }
    if section[..8] != SECTION_MAGIC {
        return Err(Error::Unsupported {
            what: "cluster section magic",
        });
    }
    let crc_offset = section.len() - 4;
    let expected_crc = u32::from_le_bytes(
        section[crc_offset..]
            .try_into()
            .map_err(|_| Error::Corrupt {
                place: "cluster section crc",
            })?,
    );
    if crc32c(&section[..crc_offset]) != expected_crc {
        return Err(Error::Corrupt {
            place: "cluster section crc",
        });
    }

    let mode = wire_to_seal_mode(section[8])?;
    let key_epoch = u32::from_le_bytes(
        section[9..13]
            .try_into()
            .map_err(|_| Error::Corrupt {
                place: "cluster section key_epoch",
            })?,
    );
    let block_count = u32::from_le_bytes(
        section[13..17]
            .try_into()
            .map_err(|_| Error::Corrupt {
                place: "cluster section block_count",
            })?,
    );

    let t = section.len() - SECTION_TRAILER_LEN;
    let subindex_offset = read_u64(section, t)?;
    let subindex_len = read_u32(section, t + 8)?;
    let filter_offset = read_u64(section, t + 12)?;
    let filter_len = read_u32(section, t + 20)?;
    let rangedel_offset = read_u64(section, t + 24)?;
    let rangedel_len = read_u32(section, t + 32)?;

    let trailer_start = t as u64;
    let sub_end = subindex_offset
        .checked_add(u64::from(subindex_len))
        .ok_or(Error::Corrupt {
            place: "section subindex overflow",
        })?;
    let fil_end = filter_offset
        .checked_add(u64::from(filter_len))
        .ok_or(Error::Corrupt {
            place: "section filter overflow",
        })?;
    let rng_end = rangedel_offset
        .checked_add(u64::from(rangedel_len))
        .ok_or(Error::Corrupt {
            place: "section rangedel overflow",
        })?;
    if subindex_offset < SECTION_HEADER_LEN as u64
        || sub_end > filter_offset
        || fil_end > rangedel_offset
        || rng_end > trailer_start
    {
        return Err(Error::Corrupt {
            place: "cluster section offsets",
        });
    }

    Ok(SectionLayout {
        mode,
        key_epoch,
        block_count,
        subindex_offset,
        subindex_len,
        filter_offset,
        filter_len,
        rangedel_offset,
        rangedel_len,
    })
}

pub struct SectionReadContext<'a> {
    pub instance_id: &'a [u8; 16],
    pub space: SpaceId,
    pub cluster: ClusterId,
    pub key_handle: Option<&'a Key32Handle>,
    pub cipher: &'a dyn Cipher,
    pub mac: &'a dyn Mac,
    pub checksum: &'a dyn Checksum,
}

/// Point lookup inside a `ClusterSection`: Filter -> SubIndex -> 1 DataBlock -> RangeDel check.
pub fn get_in_section(
    section: &[u8],
    ctx: &SectionReadContext<'_>,
    branch: BranchId,
    user_key: &[u8],
    stamp_cap: Stamp,
) -> Result<Option<BlockEntry>, Error> {
    let layout = parse_section_layout(section)?;

    // 1. Check Bloom filter
    let filter_bytes = slice_at(section, layout.filter_offset, layout.filter_len)?;
    let filter_pt = unseal_section_part(ctx, &layout, &meta_uid(2), filter_bytes)?;
    let filter = BloomFilter::decode(&filter_pt, MAX_FILTER_BYTES + 64)?;
    if !filter.may_contain(user_key) {
        return Ok(None);
    }

    // 2. Check SubIndex for candidate block
    let sub_bytes = slice_at(section, layout.subindex_offset, layout.subindex_len)?;
    let sub_pt = unseal_section_part(ctx, &layout, &meta_uid(1), sub_bytes)?;
    let subindex = SubIndex::decode(&sub_pt, MAX_DECODED_BLOCK_BYTES, 1_000_000)?;
    let Some(cand) = subindex.candidate(user_key, stamp_cap) else {
        return Ok(None);
    };

    // 3. Unseal ONLY that 1 DataBlock
    let blk_slice = slice_at(section, cand.offset, cand.len)?;
    if blk_slice.len() < 4 {
        return Err(Error::Corrupt {
            place: "section block prefix",
        });
    }
    let blk_frame_len = u32::from_le_bytes([blk_slice[0], blk_slice[1], blk_slice[2], blk_slice[3]]) as usize;
    if blk_frame_len + 4 != blk_slice.len() {
        return Err(Error::Corrupt {
            place: "section block frame length",
        });
    }
    let blk_pt = unseal_section_part(ctx, &layout, &cand.block_uid, &blk_slice[4..])?;
    let entries = block::decode(&blk_pt, MAX_DECODED_BLOCK_BYTES)?;

    let mut best: Option<BlockEntry> = None;
    for entry in entries {
        if entry.branch == branch
            && entry.key.as_slice() == user_key
            && entry.stamp() <= stamp_cap
            && best.as_ref().is_none_or(|b| entry.stamp() > b.stamp())
        {
            best = Some(entry);
        }
    }

    let Some(found) = best else {
        return Ok(None);
    };

    // 4. Apply RangeDel markers if present
    let rng_bytes = slice_at(section, layout.rangedel_offset, layout.rangedel_len)?;
    let rng_pt = unseal_section_part(ctx, &layout, &meta_uid(3), rng_bytes)?;
    let markers = rangedel::decode(&rng_pt, MAX_DECODED_BLOCK_BYTES, 1_000_000)?;
    for m in markers {
        if m.branch == branch
            && user_key >= m.start.as_slice()
            && user_key < m.end.as_slice()
        {
            let m_stamp = m.stamp()?;
            if m_stamp <= stamp_cap && m_stamp >= found.stamp() {
                return Ok(None);
            }
        }
    }

    Ok(Some(found))
}

/// Scans matching blocks in a `ClusterSection` using `block_skip_scan` on SubIndex first.
pub fn scan_section(
    section: &[u8],
    ctx: &SectionReadContext<'_>,
    branch: BranchId,
    start_inclusive: Option<&[u8]>,
    end_exclusive: Option<&[u8]>,
    stamp_cap: Stamp,
) -> Result<Vec<BlockEntry>, Error> {
    let layout = parse_section_layout(section)?;
    let sub_bytes = slice_at(section, layout.subindex_offset, layout.subindex_len)?;
    let sub_pt = unseal_section_part(ctx, &layout, &meta_uid(1), sub_bytes)?;
    let subindex = SubIndex::decode(&sub_pt, MAX_DECODED_BLOCK_BYTES, 1_000_000)?;

    let rng_bytes = slice_at(section, layout.rangedel_offset, layout.rangedel_len)?;
    let rng_pt = unseal_section_part(ctx, &layout, &meta_uid(3), rng_bytes)?;
    let markers = rangedel::decode(&rng_pt, MAX_DECODED_BLOCK_BYTES, 1_000_000)?;

    let query = BlockSkipQuery {
        start_inclusive,
        end_exclusive,
        min_stamp: 1,
        max_stamp: stamp_cap,
    };

    let mut out = Vec::new();
    for blk_meta in block_skip_scan(&subindex, query) {
        let blk_slice = slice_at(section, blk_meta.offset, blk_meta.len)?;
        if blk_slice.len() < 4 {
            return Err(Error::Corrupt {
                place: "section block prefix",
            });
        }
        let blk_pt = unseal_section_part(ctx, &layout, &blk_meta.block_uid, &blk_slice[4..])?;
        let entries = block::decode(&blk_pt, MAX_DECODED_BLOCK_BYTES)?;
        for entry in entries {
            if entry.branch != branch || entry.stamp() > stamp_cap {
                continue;
            }
            if let Some(s) = start_inclusive {
                if entry.key.as_slice() < s {
                    continue;
                }
            }
            if let Some(e) = end_exclusive {
                if entry.key.as_slice() >= e {
                    continue;
                }
            }
            let shadowed = markers.iter().any(|m| {
                m.branch == branch
                    && entry.key.as_slice() >= m.start.as_slice()
                    && entry.key.as_slice() < m.end.as_slice()
                    && m.stamp().is_ok_and(|ms| ms <= stamp_cap && ms >= entry.stamp())
            });
            if !shadowed {
                out.push(entry);
            }
        }
    }
    Ok(out)
}

fn unseal_section_part(
    ctx: &SectionReadContext<'_>,
    layout: &SectionLayout,
    block_uid: &[u8; 16],
    frame: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    let aad = Aad::Block {
        instance: ctx.instance_id,
        space: ctx.space,
        cluster: ctx.cluster,
        key_epoch: layout.key_epoch,
        block_uid,
    };
    match layout.mode {
        SealMode::Plain => {
            let (_, pt) = unseal_frame(
                layout.mode,
                None,
                aad,
                frame,
                ctx.cipher,
                ctx.mac,
                ctx.checksum,
            )?;
            Ok(pt)
        }
        SealMode::Authenticated | SealMode::Encrypted => {
            let handle = ctx.key_handle.ok_or(Error::KeyUnavailable {
                retry_after_secs: None,
            })?;
            handle.with_key(|k| {
                let (_, pt) = unseal_frame(
                    layout.mode,
                    Some(k),
                    aad,
                    frame,
                    ctx.cipher,
                    ctx.mac,
                    ctx.checksum,
                )?;
                Ok(pt)
            })
        }
    }
}

fn slice_at(bytes: &[u8], offset: u64, len: u32) -> Result<&[u8], Error> {
    let start = usize::try_from(offset).map_err(|_| Error::Corrupt {
        place: "section offset overflow",
    })?;
    let end = start
        .checked_add(len as usize)
        .ok_or(Error::Corrupt {
            place: "section slice end overflow",
        })?;
    bytes.get(start..end).ok_or(Error::Corrupt {
        place: "section slice bounds",
    })
}

fn meta_uid(tag: u8) -> [u8; 16] {
    let mut uid = [0u8; 16];
    uid[0] = tag;
    uid
}

fn seal_mode_to_wire(mode: SealMode) -> u8 {
    match mode {
        SealMode::Plain => 1,
        SealMode::Authenticated => 2,
        SealMode::Encrypted => 3,
    }
}

fn wire_to_seal_mode(v: u8) -> Result<SealMode, Error> {
    match v {
        1 => Ok(SealMode::Plain),
        2 => Ok(SealMode::Authenticated),
        3 => Ok(SealMode::Encrypted),
        _ => Err(Error::Unsupported {
            what: "section seal mode",
        }),
    }
}

fn read_u32(bytes: &[u8], at: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(
        bytes
            .get(at..at + 4)
            .ok_or(Error::Corrupt {
                place: "section u32",
            })?
            .try_into()
            .map_err(|_| Error::Corrupt {
                place: "section u32",
            })?,
    ))
}

fn read_u64(bytes: &[u8], at: usize) -> Result<u64, Error> {
    Ok(u64::from_le_bytes(
        bytes
            .get(at..at + 8)
            .ok_or(Error::Corrupt {
                place: "section u64",
            })?
            .try_into()
            .map_err(|_| Error::Corrupt {
                place: "section u64",
            })?,
    ))
}
