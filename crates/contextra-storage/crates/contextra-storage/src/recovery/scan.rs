//! Zwei-Durchlauf-Scan über Redo: erst Integrität/Heads und Gate-Set, danach Replay.

use crate::store::redo::frame::{DecodedFrame, FrameKind};
use crate::store::redo::reader::{RedoReader, ScanResult};
use crate::store::redo::torn_tail::TailState;
use contextra_storage_ports::{ClusterId, Error, Key32, SpaceId, Stamp};
use contextra_storage_ports::traits::Mac;
use std::collections::BTreeSet;

pub struct RecoveryScan<'a> {
    pub frames: Vec<DecodedFrame<'a>>,
    pub retired_clusters: BTreeSet<ClusterId>,
    pub tail: TailState,
    pub valid_bytes: usize,
    pub last_stamp: Stamp,
    pub last_mac: [u8; 32],
}

pub trait GateScopeResolver {
    fn clusters_in_space(&self, space: SpaceId) -> Result<Vec<ClusterId>, Error>;
}

/// Pass 1 checks framing, CRC, chain MAC and stamp monotonicity. Pass 2 extracts
/// the durable retired-cluster set before callers expose or replay data frames.
pub fn scan<'a>(
    reader: &RedoReader,
    bytes: &'a [u8],
    key: &Key32,
    mac: &dyn Mac,
    previous_mac: [u8; 32],
    previous_stamp: Stamp,
) -> Result<RecoveryScan<'a>, Error> {
    scan_with_resolver(reader, bytes, key, mac, previous_mac, previous_stamp, None)
}

pub fn scan_with_resolver<'a>(
    reader: &RedoReader,
    bytes: &'a [u8],
    key: &Key32,
    mac: &dyn Mac,
    previous_mac: [u8; 32],
    previous_stamp: Stamp,
    resolver: Option<&dyn GateScopeResolver>,
) -> Result<RecoveryScan<'a>, Error> {
    let result = reader.scan(bytes, key, mac, previous_mac, previous_stamp)?;
    let retired_clusters = collect_retired(&result, resolver)?;
    Ok(RecoveryScan {
        frames: result.frames,
        retired_clusters,
        tail: result.tail,
        valid_bytes: result.valid_bytes,
        last_stamp: result.last_stamp,
        last_mac: result.last_mac,
    })
}

fn collect_retired(result: &ScanResult<'_>, resolver: Option<&dyn GateScopeResolver>) -> Result<BTreeSet<ClusterId>, Error> {
    let mut retired = BTreeSet::new();
    for frame in &result.frames {
        if frame.head.kind != FrameKind::Gate { continue; }
        for op in &frame.payloads {
            // Gate bodies are canonical plaintext control payloads. Unknown or
            // malformed encodings fail closed before any user payload replay.
            parse_gate(op.bytes, &mut retired, resolver)?;
        }
    }
    Ok(retired)
}

fn parse_gate(bytes: &[u8], retired: &mut BTreeSet<ClusterId>, resolver: Option<&dyn GateScopeResolver>) -> Result<(), Error> {
    let mut c = Cursor { bytes, offset: 0 };
    let scope = c.u8()?;
    let _ticket = c.array::<16>()?;
    let _level = c.u8()?;
    let count = c.u16()? as usize;
    match scope {
        1 => {
            if count == 0 { return Err(Error::Corrupt { place: "redo gate cluster header" }); }
            for _ in 0..count { retired.insert(ClusterId::from_wire(c.array::<16>()?)); }
        }
        2 => {
            if count != 0 { return Err(Error::Corrupt { place: "redo gate space header" }); }
            let space = SpaceId(c.u64_be()?);
            let resolver = resolver.ok_or(Error::Unsupported { what: "space gate requires catalog resolver" })?;
            retired.extend(resolver.clusters_in_space(space)?);
        }
        _ => return Err(Error::Unsupported { what: "redo gate scope" }),
    }
    if !c.is_empty() { return Err(Error::Corrupt { place: "redo gate trailing bytes" }); }
    Ok(())
}

struct Cursor<'a> { bytes: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "redo gate length" })?;
        let slice = self.bytes.get(self.offset..end).ok_or(Error::Corrupt { place: "redo gate truncated" })?;
        self.offset = end;
        Ok(slice)
    }
    fn u8(&mut self) -> Result<u8, Error> {
        self.array::<1>()?.first().copied().ok_or(Error::Corrupt { place: "recovery byte" })
    }
    fn u16(&mut self) -> Result<u16, Error> { Ok(u16::from_le_bytes(self.array()?)) }
    fn u64_be(&mut self) -> Result<u64, Error> { Ok(u64::from_be_bytes(self.array()?)) }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| Error::Corrupt { place: "redo gate field" })
    }
    fn is_empty(&self) -> bool { self.offset == self.bytes.len() }
}
