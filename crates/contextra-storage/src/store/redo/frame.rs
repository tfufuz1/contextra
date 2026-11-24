//! Provisorischer, deterministischer Redo-Frame-Codec (WP-05.1).

use contextra_storage_ports::{
    traits::Mac, BranchId, CallerMeta, ClusterId, Error, Key32, Origin, SpaceId, Stamp,
    STAMP_MAX,
};

const MAGIC: &[u8; 4] = b"R8F1";
const FIXED_HEAD_BYTES: usize = 4 + 8 + 1 + 2 + 16 + 8 + 1 + 1 + 8 + 4;
const TAIL_BYTES: usize = 4 + 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind { Change, Gate, Control, Reserve }

#[derive(Debug, Clone, Copy)]
pub struct OpHead {
    pub space: SpaceId,
    pub cluster: ClusterId,
    pub branch: BranchId,
    pub kind: u8,
    pub key_epoch: u32,
    pub op_uid: [u8; 16],
    pub payload_len: u32,
}

#[derive(Debug, Clone)]
pub struct FrameHead {
    pub stamp: Stamp,
    pub kind: FrameKind,
    pub idem: [u8; 16],
    pub epoch: u64,
    pub flags: u8,
    pub caller: CallerMeta,
    pub ttl_bucket: u32,
    pub ops: Vec<OpHead>,
}

#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub head: FrameHead,
    /// One sealed or control payload per op head, in the same order.
    pub payloads: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct OpPayload<'a> {
    pub head: OpHead,
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct DecodedFrame<'a> {
    pub head: FrameHead,
    pub payloads: Vec<OpPayload<'a>>,
    pub chain_mac: [u8; 32],
}

impl FrameKind {
    fn to_wire(self) -> u8 {
        match self { Self::Change => 1, Self::Gate => 2, Self::Control => 3, Self::Reserve => 4 }
    }

    fn from_wire(value: u8) -> Result<Self, Error> {
        match value {
            1 => Ok(Self::Change), 2 => Ok(Self::Gate), 3 => Ok(Self::Control), 4 => Ok(Self::Reserve),
            _ => Err(Error::Unsupported { what: "redo frame kind" }),
        }
    }
}

/// Kodiert einen vollständigen Frame einschließlich CRC und Ketten-MAC.
/// `previous` ist der MAC des vorherigen Frames; der erste Frame nutzt Nullen.
pub fn encode_frame(frame: &EncodedFrame, previous: &[u8; 32], key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error> {
    if frame.head.stamp > STAMP_MAX { return Err(Error::Limit { limit: "stamp exceeds 56 bits" }); }
    if frame.head.ops.len() != frame.payloads.len() || frame.head.ops.len() > u16::MAX as usize {
        return Err(Error::Limit { limit: "redo op count" });
    }
    let mut body = Vec::new();
    body.extend_from_slice(MAGIC);
    body.extend_from_slice(&frame.head.stamp.to_le_bytes());
    body.push(frame.head.kind.to_wire());
    body.extend_from_slice(&(frame.head.ops.len() as u16).to_le_bytes());
    body.extend_from_slice(&frame.head.idem);
    body.extend_from_slice(&frame.head.epoch.to_le_bytes());
    body.push(frame.head.flags);
    body.push(origin_to_wire(frame.head.caller.origin));
    body.extend_from_slice(&frame.head.caller.caller_seq.to_le_bytes());
    body.extend_from_slice(&frame.head.ttl_bucket.to_le_bytes());
    for (op, payload) in frame.head.ops.iter().zip(&frame.payloads) {
        let payload_len = u32::try_from(payload.len()).map_err(|_| Error::Limit { limit: "redo payload length" })?;
        if payload_len != op.payload_len { return Err(Error::Limit { limit: "redo payload/head mismatch" }); }
        body.extend_from_slice(&op.space.0.to_be_bytes());
        body.extend_from_slice(op.cluster.as_bytes());
        body.extend_from_slice(&op.branch.0.to_le_bytes());
        body.push(op.kind);
        body.extend_from_slice(&op.key_epoch.to_le_bytes());
        body.extend_from_slice(&op.op_uid);
        body.extend_from_slice(&payload_len.to_le_bytes());
        body.extend_from_slice(payload);
    }
    let crc = crate::codec::checksums::crc32c(&body);
    body.extend_from_slice(&crc.to_le_bytes());
    let mut authenticated = Vec::with_capacity(previous.len() + body.len());
    authenticated.extend_from_slice(previous);
    authenticated.extend_from_slice(&body);
    let chain_mac = mac.tag(key, &authenticated)?;
    let frame_len = body.len().checked_add(chain_mac.len()).ok_or(Error::Limit { limit: "redo frame length" })?;
    let frame_len = u32::try_from(frame_len).map_err(|_| Error::Limit { limit: "redo frame length" })?;
    if u64::from(frame_len) > contextra_storage_ports::limits::REDO_SEGMENT_BYTES / 8 {
        return Err(Error::Limit { limit: "redo frame <= segment / 8" });
    }
    let mut output = Vec::with_capacity(4 + frame_len as usize);
    output.extend_from_slice(&frame_len.to_le_bytes());
    output.extend_from_slice(&body);
    output.extend_from_slice(&chain_mac);
    Ok(output)
}

/// Prüft Grenzen, CRC und Ketten-MAC, bevor Payloads als Borrowed-Slices geliefert werden.
pub fn decode_frame<'a>(input: &'a [u8], previous: &[u8; 32], key: &Key32, mac: &dyn Mac) -> Result<DecodedFrame<'a>, Error> {
    if input.len() < 4 { return Err(Error::Corrupt { place: "redo frame length" }); }
    let length_bytes: [u8; 4] = input.get(..4).ok_or(Error::Corrupt { place: "redo frame length" })?
        .try_into().map_err(|_| Error::Corrupt { place: "redo frame length" })?;
    let length = u32::from_le_bytes(length_bytes) as usize;
    if length.checked_add(4) != Some(input.len())
        || length < FIXED_HEAD_BYTES + TAIL_BYTES
        || length > (contextra_storage_ports::limits::REDO_SEGMENT_BYTES / 8) as usize
    {
        return Err(Error::Corrupt { place: "redo frame length" });
    }
    let mac_start = input.len().checked_sub(32).ok_or(Error::Corrupt { place: "redo frame length" })?;
    let crc_start = mac_start.checked_sub(4).ok_or(Error::Corrupt { place: "redo frame length" })?;
    let signed = input.get(4..mac_start).ok_or(Error::Corrupt { place: "redo frame bounds" })?;
    let received_mac: [u8; 32] = input.get(mac_start..).ok_or(Error::Corrupt { place: "redo frame bounds" })?
        .try_into().map_err(|_| Error::Corrupt { place: "redo frame MAC" })?;
    let mut authenticated = Vec::with_capacity(previous.len() + signed.len());
    authenticated.extend_from_slice(previous);
    authenticated.extend_from_slice(signed);
    if mac.tag(key, &authenticated)? != received_mac { return Err(Error::Corrupt { place: "redo chain MAC" }); }
    let crc_bytes: [u8; 4] = input.get(crc_start..mac_start).ok_or(Error::Corrupt { place: "redo frame CRC" })?
        .try_into().map_err(|_| Error::Corrupt { place: "redo frame CRC" })?;
    let payload_region = input.get(4..crc_start).ok_or(Error::Corrupt { place: "redo frame bounds" })?;
    if crate::codec::checksums::crc32c(payload_region) != u32::from_le_bytes(crc_bytes) {
        return Err(Error::Corrupt { place: "redo frame CRC" });
    }
    let mut cursor = Cursor::new(payload_region);
    if cursor.take(4)? != MAGIC { return Err(Error::Unsupported { what: "redo frame magic/version" }); }
    let stamp = cursor.u64()?;
    let kind = FrameKind::from_wire(cursor.u8()?)?;
    let op_count = cursor.u16()? as usize;
    let idem = cursor.array::<16>()?;
    let epoch = cursor.u64()?;
    let flags = cursor.u8()?;
    let origin = origin_from_wire(cursor.u8()?)?;
    let caller_seq = cursor.u64()?;
    let ttl_bucket = cursor.u32()?;
    let mut ops = Vec::with_capacity(op_count);
    let mut payloads = Vec::with_capacity(op_count);
    for _ in 0..op_count {
        let space = SpaceId(cursor.u64_be()?);
        let cluster = ClusterId::from_wire(cursor.array::<16>()?);
        let branch = BranchId(cursor.u32()?);
        let op_kind = cursor.u8()?;
        let key_epoch = cursor.u32()?;
        let op_uid = cursor.array::<16>()?;
        let payload_len = cursor.u32()?;
        let payload = cursor.take(payload_len as usize)?;
        let op = OpHead { space, cluster, branch, kind: op_kind, key_epoch, op_uid, payload_len };
        ops.push(op);
        payloads.push(OpPayload { head: op, bytes: payload });
    }
    if !cursor.is_empty() { return Err(Error::Corrupt { place: "redo frame trailing bytes" }); }
    let head = FrameHead { stamp, kind, idem, epoch, flags, caller: CallerMeta { origin, caller_seq }, ttl_bucket, ops };
    Ok(DecodedFrame { head, payloads, chain_mac: received_mac })
}

struct Cursor<'a> { bytes: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self { Self { bytes, offset: 0 } }
    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "redo cursor overflow" })?;
        let bytes = self.bytes.get(self.offset..end).ok_or(Error::Corrupt { place: "redo frame truncated" })?;
        self.offset = end;
        Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| Error::Corrupt { place: "redo fixed field" })
    }
    fn u8(&mut self) -> Result<u8, Error> {
        self.array::<1>()?.first().copied().ok_or(Error::Corrupt { place: "redo fixed field" })
    }
    fn u16(&mut self) -> Result<u16, Error> { Ok(u16::from_le_bytes(self.array()?)) }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> Result<u64, Error> { Ok(u64::from_le_bytes(self.array()?)) }
    fn u64_be(&mut self) -> Result<u64, Error> { Ok(u64::from_be_bytes(self.array()?)) }
    fn is_empty(&self) -> bool { self.offset == self.bytes.len() }
}

fn origin_to_wire(origin: Origin) -> u8 {
    match origin { Origin::User => 0, Origin::Maintenance => 1, Origin::Replication => 2, Origin::Migration => 3 }
}
fn origin_from_wire(value: u8) -> Result<Origin, Error> {
    match value { 0 => Ok(Origin::User), 1 => Ok(Origin::Maintenance), 2 => Ok(Origin::Replication), 3 => Ok(Origin::Migration), _ => Err(Error::Unsupported { what: "redo caller origin" }) }
}
