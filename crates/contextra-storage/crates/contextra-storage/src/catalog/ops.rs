//! P1-Teilmenge der Catalog-Op-Kodierung; Tags bleiben bis G-F provisorisch.

use contextra_storage_ports::{ClusterId, ClusterLifecycle, Error, Protection, SpaceId, Stamp};

const TAG_SPACE_ADD: u16 = 1;
const TAG_CLUSTER_REGISTER: u16 = 2;
const TAG_FILES_ADD: u16 = 3;
const TAG_FILES_REMOVE: u16 = 4;
const TAG_REPLAY_FLOOR: u16 = 5;
const TAG_REDO_MARK: u16 = 6;
const TAG_RESERVE_STAMPS: u16 = 7;
const TAG_RESERVE_IDS: u16 = 8;
const TAG_PURGE_BATCH: u16 = 9;
const TAG_UNLINK_ACK: u16 = 10;
const TAG_PURGE_COMPLETE: u16 = 11;
const TAG_CLUSTER_STATE: u16 = 12;
const TAG_RESERVE_SPACE_IDS: u16 = 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind { Run, Blob }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub file_id: u64,
    pub space: SpaceId,
    pub cluster: Option<ClusterId>,
    pub kind: FileKind,
    pub level: u8,
    pub bytes: u64,
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogOp {
    SpaceAdd { space: SpaceId, protection: Protection },
    ClusterRegister { cluster: ClusterId, protection: Protection },
    ClusterState { cluster: ClusterId, state: ClusterLifecycle },
    FilesAdd { file_id: u64, space: SpaceId, cluster: Option<ClusterId>, kind: FileKind, level: u8, bytes: u64, min_stamp: Stamp, max_stamp: Stamp },
    FilesRemove { file_id: u64 },
    ReplayFloor { segment: u64, offset: u64 },
    RedoMark { segment: u64, offset: u64, stamp: Stamp },
    ReserveStamps { through: Stamp },
    ReserveIds { through: u64 },
    ReserveSpaceIds { through: u64 },
    /// Atomically swaps one live file incarnation and pins the old file in the
    /// persistent graveyard until its unlink and parent-directory sync ack.
    PurgeBatch { ticket: [u8; 16], remove_file_id: u64, replacement: Option<FileEntry> },
    UnlinkAck { ticket: [u8; 16], file_id: u64 },
    PurgeComplete { ticket: [u8; 16] },
}

impl CatalogOp {
    pub fn encode(&self, out: &mut Vec<u8>) -> Result<(), Error> {
        let mut payload = Vec::new();
        let tag = match self {
            Self::SpaceAdd { space, protection } => {
                put_u64_be(&mut payload, space.0);
                payload.push(protection_to_wire(*protection));
                TAG_SPACE_ADD
            }
            Self::ClusterRegister { cluster, protection } => {
                payload.extend_from_slice(cluster.as_bytes());
                payload.push(protection_to_wire(*protection));
                TAG_CLUSTER_REGISTER
            }
            Self::ClusterState { cluster, state } => {
                payload.extend_from_slice(cluster.as_bytes());
                match state {
                    ClusterLifecycle::Live => payload.push(1),
                    ClusterLifecycle::Retired { at } => { payload.push(2); put_u64(&mut payload, *at); }
                    ClusterLifecycle::Shredded { at } => { payload.push(3); put_u64(&mut payload, *at); }
                    ClusterLifecycle::Purged { at } => { payload.push(4); put_u64(&mut payload, *at); }
                }
                TAG_CLUSTER_STATE
            }
            Self::FilesAdd { file_id, space, cluster, kind, level, bytes, min_stamp, max_stamp } => {
                if min_stamp > max_stamp { return Err(Error::Limit { limit: "catalog file stamp range" }); }
                put_u64(&mut payload, *file_id);
                put_u64_be(&mut payload, space.0);
                match cluster {
                    Some(id) => { payload.push(1); payload.extend_from_slice(id.as_bytes()); }
                    None => payload.push(0),
                }
                payload.push(match kind { FileKind::Run => 1, FileKind::Blob => 2 });
                payload.push(*level);
                put_u64(&mut payload, *bytes);
                put_u64(&mut payload, *min_stamp);
                put_u64(&mut payload, *max_stamp);
                TAG_FILES_ADD
            }
            Self::FilesRemove { file_id } => { put_u64(&mut payload, *file_id); TAG_FILES_REMOVE }
            Self::ReplayFloor { segment, offset } => { put_u64(&mut payload, *segment); put_u64(&mut payload, *offset); TAG_REPLAY_FLOOR }
            Self::RedoMark { segment, offset, stamp } => { put_u64(&mut payload, *segment); put_u64(&mut payload, *offset); put_u64(&mut payload, *stamp); TAG_REDO_MARK }
            Self::ReserveStamps { through } => { put_u64(&mut payload, *through); TAG_RESERVE_STAMPS }
            Self::ReserveIds { through } => { put_u64(&mut payload, *through); TAG_RESERVE_IDS }
            Self::ReserveSpaceIds { through } => { put_u64_be(&mut payload, *through); TAG_RESERVE_SPACE_IDS }
            Self::PurgeBatch { ticket, remove_file_id, replacement } => {
                payload.extend_from_slice(ticket);
                put_u64(&mut payload, *remove_file_id);
                encode_optional_file(&mut payload, replacement.as_ref())?;
                TAG_PURGE_BATCH
            }
            Self::UnlinkAck { ticket, file_id } => {
                payload.extend_from_slice(ticket);
                put_u64(&mut payload, *file_id);
                TAG_UNLINK_ACK
            }
            Self::PurgeComplete { ticket } => { payload.extend_from_slice(ticket); TAG_PURGE_COMPLETE }
        };
        let length = u32::try_from(payload.len()).map_err(|_| Error::Limit { limit: "catalog op length" })?;
        put_u16(out, tag);
        put_u32(out, length);
        out.extend_from_slice(&payload);
        Ok(())
    }

    pub fn decode(input: &[u8]) -> Result<(Self, usize), Error> {
        let mut cursor = Cursor::new(input);
        let tag = cursor.u16()?;
        let len = cursor.u32()? as usize;
        let payload = cursor.take(len)?;
        let mut p = Cursor::new(payload);
        let op = match tag {
            TAG_SPACE_ADD => Self::SpaceAdd { space: SpaceId(p.u64_be()?), protection: protection_from_wire(p.u8()?)? },
            TAG_CLUSTER_REGISTER => Self::ClusterRegister { cluster: ClusterId::from_wire(p.array()?), protection: protection_from_wire(p.u8()?)? },
            TAG_CLUSTER_STATE => {
                let cluster = ClusterId::from_wire(p.array()?);
                let state = match p.u8()? {
                    1 => ClusterLifecycle::Live,
                    2 => ClusterLifecycle::Retired { at: p.u64()? },
                    3 => ClusterLifecycle::Shredded { at: p.u64()? },
                    4 => ClusterLifecycle::Purged { at: p.u64()? },
                    _ => return Err(Error::Unsupported { what: "catalog cluster lifecycle" }),
                };
                Self::ClusterState { cluster, state }
            }
            TAG_FILES_ADD => {
                let file_id = p.u64()?;
                let space = SpaceId(p.u64_be()?);
                let cluster = match p.u8()? {
                    0 => None,
                    1 => Some(ClusterId::from_wire(p.array()?)),
                    _ => return Err(Error::Unsupported { what: "catalog cluster option" }),
                };
                let kind = match p.u8()? { 1 => FileKind::Run, 2 => FileKind::Blob, _ => return Err(Error::Unsupported { what: "catalog file kind" }) };
                let level = p.u8()?;
                let bytes = p.u64()?;
                let min_stamp = p.u64()?;
                let max_stamp = p.u64()?;
                if min_stamp > max_stamp { return Err(Error::Corrupt { place: "catalog file stamp range" }); }
                Self::FilesAdd { file_id, space, cluster, kind, level, bytes, min_stamp, max_stamp }
            }
            TAG_FILES_REMOVE => Self::FilesRemove { file_id: p.u64()? },
            TAG_REPLAY_FLOOR => Self::ReplayFloor { segment: p.u64()?, offset: p.u64()? },
            TAG_REDO_MARK => Self::RedoMark { segment: p.u64()?, offset: p.u64()?, stamp: p.u64()? },
            TAG_RESERVE_STAMPS => Self::ReserveStamps { through: p.u64()? },
            TAG_RESERVE_IDS => Self::ReserveIds { through: p.u64()? },
            TAG_RESERVE_SPACE_IDS => Self::ReserveSpaceIds { through: p.u64_be()? },
            TAG_PURGE_BATCH => Self::PurgeBatch { ticket: p.array()?, remove_file_id: p.u64()?, replacement: decode_optional_file(&mut p)? },
            TAG_UNLINK_ACK => Self::UnlinkAck { ticket: p.array()?, file_id: p.u64()? },
            TAG_PURGE_COMPLETE => Self::PurgeComplete { ticket: p.array()? },
            _ => return Err(Error::Unsupported { what: "catalog op tag" }),
        };
        if !p.is_empty() { return Err(Error::Corrupt { place: "catalog op trailing bytes" }); }
        Ok((op, cursor.offset()))
    }
}

fn protection_to_wire(value: Protection) -> u8 { match value { Protection::Sealed => 1, Protection::Plain => 2 } }
fn protection_from_wire(value: u8) -> Result<Protection, Error> {
    match value { 1 => Ok(Protection::Sealed), 2 => Ok(Protection::Plain), _ => Err(Error::Unsupported { what: "catalog protection" }) }
}
fn put_u16(out: &mut Vec<u8>, value: u16) { out.extend_from_slice(&value.to_le_bytes()); }
fn put_u32(out: &mut Vec<u8>, value: u32) { out.extend_from_slice(&value.to_le_bytes()); }
fn put_u64(out: &mut Vec<u8>, value: u64) { out.extend_from_slice(&value.to_le_bytes()); }
fn put_u64_be(out: &mut Vec<u8>, value: u64) { out.extend_from_slice(&value.to_be_bytes()); }

fn encode_optional_file(out: &mut Vec<u8>, file: Option<&FileEntry>) -> Result<(), Error> {
    let Some(file) = file else { out.push(0); return Ok(()); };
    if file.min_stamp > file.max_stamp { return Err(Error::Limit { limit: "catalog file stamp range" }); }
    out.push(1);
    put_u64(out, file.file_id);
    put_u64_be(out, file.space.0);
    match file.cluster { Some(cluster) => { out.push(1); out.extend_from_slice(cluster.as_bytes()); }, None => out.push(0) }
    out.push(match file.kind { FileKind::Run => 1, FileKind::Blob => 2 });
    out.push(file.level);
    put_u64(out, file.bytes);
    put_u64(out, file.min_stamp);
    put_u64(out, file.max_stamp);
    Ok(())
}

fn decode_optional_file(c: &mut Cursor<'_>) -> Result<Option<FileEntry>, Error> {
    match c.u8()? {
        0 => Ok(None),
        1 => {
            let file_id = c.u64()?;
            let space = SpaceId(c.u64_be()?);
            let cluster = match c.u8()? { 0 => None, 1 => Some(ClusterId::from_wire(c.array()?)), _ => return Err(Error::Unsupported { what: "catalog cluster option" }) };
            let kind = match c.u8()? { 1 => FileKind::Run, 2 => FileKind::Blob, _ => return Err(Error::Unsupported { what: "catalog file kind" }) };
            let level = c.u8()?;
            let bytes = c.u64()?;
            let min_stamp = c.u64()?;
            let max_stamp = c.u64()?;
            if min_stamp > max_stamp { return Err(Error::Corrupt { place: "catalog file stamp range" }); }
            Ok(Some(FileEntry { file_id, space, cluster, kind, level, bytes, min_stamp, max_stamp }))
        }
        _ => Err(Error::Unsupported { what: "catalog optional file" }),
    }
}

struct Cursor<'a> { input: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
    fn new(input: &'a [u8]) -> Self { Self { input, offset: 0 } }
    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "catalog cursor overflow" })?;
        let bytes = self.input.get(self.offset..end).ok_or(Error::Corrupt { place: "catalog op truncated" })?;
        self.offset = end;
        Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| Error::Corrupt { place: "catalog fixed field" })
    }
    fn u8(&mut self) -> Result<u8, Error> { self.array::<1>()?.first().copied().ok_or(Error::Corrupt { place: "catalog fixed field" }) }
    fn u16(&mut self) -> Result<u16, Error> { Ok(u16::from_le_bytes(self.array()?)) }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> Result<u64, Error> { Ok(u64::from_le_bytes(self.array()?)) }
    fn u64_be(&mut self) -> Result<u64, Error> { Ok(u64::from_be_bytes(self.array()?)) }
    fn is_empty(&self) -> bool { self.offset == self.input.len() }
    fn offset(&self) -> usize { self.offset }
}
