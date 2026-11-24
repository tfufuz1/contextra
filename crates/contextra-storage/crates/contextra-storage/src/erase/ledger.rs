//! Canonical append-only ledger records (SPEC §9.7, INV-LED-1/2).
//!
//! Selectors are supplied only as already AEAD-sealed bytes. This codec never
//! accepts ClusterId, SpaceId, keys, or user payloads.

use crate::codec::varint::{varint_get, varint_put};
use contextra_storage_ports::{traits::{File, FileRole, IoCtx, Mac}, Error, Key32, Ticket, TombId};

pub const MAX_LEDGER_RECORD_BYTES: usize = 256 * 1024;
pub const MAX_INTENT_TOMBS: usize = 4096;
pub const MAX_LEDGER_BYTES: u64 = 64 * 1024 * 1024;

fn ledger_ctx(purpose: &'static str) -> IoCtx { IoCtx { role: FileRole::Ledger, purpose } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LedgerEventKind {
    EraseIntent = 1, GateApplied, KeyDestroyIntent, KeysDestroyed, PurgeProgress,
    UnlinkProgress, ExtAck, PurgeVerified, Completed, RestrictionSet, RestrictionLifted,
    BranchEraseDone, SpaceDropIntent, SpaceScopeFrozen, SpaceDropped, RestoreReconciled,
    ReceiptIssued, TombEpochOpened, TombEpochClosed, TombEpochRetireIntent, TombEpochRetired,
}

impl TryFrom<u8> for LedgerEventKind {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        use LedgerEventKind::*;
        Ok(match value { 1 => EraseIntent, 2 => GateApplied, 3 => KeyDestroyIntent, 4 => KeysDestroyed,
            5 => PurgeProgress, 6 => UnlinkProgress, 7 => ExtAck, 8 => PurgeVerified, 9 => Completed,
            10 => RestrictionSet, 11 => RestrictionLifted, 12 => BranchEraseDone, 13 => SpaceDropIntent,
            14 => SpaceScopeFrozen, 15 => SpaceDropped, 16 => RestoreReconciled, 17 => ReceiptIssued,
            18 => TombEpochOpened, 19 => TombEpochClosed, 20 => TombEpochRetireIntent, 21 => TombEpochRetired,
            _ => return Err(Error::Unsupported { what: "ledger event tag" }),
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct LedgerEvent {
    pub kind: LedgerEventKind,
    pub tomb_epoch: u32,
    pub ticket: Ticket,
    /// Canonically sorted, unique tomb pseudonyms; never clear cluster IDs.
    pub tombs: Vec<TombId>,
    pub evidence_digest: [u8; 32],
    /// Optional selector encrypted under LedgerScopeKey and bound by its AAD.
    pub scope_ciphertext: Vec<u8>,
}

#[derive(Clone)]
pub struct LedgerRecord {
    pub sequence: u64,
    pub previous_hash: [u8; 32],
    pub clock_sample: u64,
    pub event: LedgerEvent,
    pub mac: [u8; 32],
}

/// Single-owner durable ledger appender. It has no internal lock: ownership is
/// held by the urgent ledger lane, as required by SPEC §3.4 and §9.7.
pub struct LedgerWriter<'a> {
    file: &'a dyn File,
    key: &'a Key32,
    mac_impl: &'a dyn Mac,
    next_sequence: u64,
    head: [u8; 32],
    poisoned: bool,
}

impl<'a> LedgerWriter<'a> {
    pub fn recover(file: &'a dyn File, key: &'a Key32, mac_impl: &'a dyn Mac) -> Result<(Self, Vec<LedgerRecord>), Error> {
        let size = file.size()?;
        if size > MAX_LEDGER_BYTES { return Err(Error::Limit { limit: "ledger size" }); }
        let mut bytes = vec![0; usize::try_from(size).map_err(|_| Error::Limit { limit: "ledger size" })?];
        if !bytes.is_empty() {
            let read = file.read_at(0, &mut bytes, ledger_ctx("ledger_replay"))?;
            if read != bytes.len() { return Err(Error::Corrupt { place: "ledger short read" }); }
        }
        let mut offset = 0usize; let mut seq = 1u64; let mut head = [0; 32]; let mut records = Vec::new();
        while offset < bytes.len() {
            match decode_record(&bytes[offset..], seq, head, key, mac_impl) {
                Ok((record, used)) => { head = record.mac; seq = seq.checked_add(1).ok_or(Error::Exhausted)?; offset += used; records.push(record); }
                // The current File port deliberately has no truncate API. A
                // torn tail therefore requires ledger rollover/recovery
                // orchestration before this handle may append again.
                Err(Error::Corrupt { place: "ledger record truncated" }) => return Err(Error::Corrupt { place: "ledger torn tail requires rollover" }),
                Err(error) => return Err(error),
            }
        }
        Ok((Self { file, key, mac_impl, next_sequence: seq, head, poisoned: false }, records))
    }

    pub fn append(&mut self, clock_sample: u64, event: LedgerEvent) -> Result<LedgerRecord, Error> {
        if self.poisoned { return Err(Error::Poisoned { cause: "ledger writer is poisoned" }); }
        let record = LedgerRecord { sequence: self.next_sequence, previous_hash: self.head, clock_sample, event, mac: [0; 32] };
        let encoded = match encode_record(&record, self.key, self.mac_impl) { Ok(bytes) => bytes, Err(error) => return Err(error) };
        let tag: [u8; 32] = encoded[encoded.len() - 32..].try_into().map_err(|_| Error::Corrupt { place: "ledger encoded MAC" })?;
        if let Err(error) = self.file.append_prealloc(&encoded, ledger_ctx("ledger_append")).and_then(|_| self.file.sync_data(ledger_ctx("ledger_sync"))) {
            self.poisoned = true;
            return Err(error);
        }
        self.head = tag; self.next_sequence = self.next_sequence.checked_add(1).ok_or(Error::Exhausted)?;
        Ok(LedgerRecord { mac: tag, ..record })
    }

    pub fn head(&self) -> [u8; 32] { self.head }
    pub fn next_sequence(&self) -> u64 { self.next_sequence }
    pub fn is_poisoned(&self) -> bool { self.poisoned }
}

pub fn encode_record(record: &LedgerRecord, key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error> {
    let body = encode_body(record.sequence, &record.previous_hash, record.clock_sample, &record.event)?;
    let tag = mac.tag(key, &body)?;
    let len = body.len().checked_add(tag.len()).ok_or(Error::Limit { limit: "ledger record length" })?;
    if len > MAX_LEDGER_RECORD_BYTES { return Err(Error::Limit { limit: "ledger record <= 256 KiB" }); }
    let mut out = Vec::with_capacity(4 + len);
    out.extend_from_slice(&(len as u32).to_le_bytes()); out.extend_from_slice(&body); out.extend_from_slice(&tag);
    Ok(out)
}

pub fn decode_record(input: &[u8], expected_sequence: u64, previous_hash: [u8; 32], key: &Key32, mac: &dyn Mac) -> Result<(LedgerRecord, usize), Error> {
    let len = u32::from_le_bytes(input.get(..4).ok_or(Error::Corrupt { place: "ledger record length" })?.try_into().map_err(|_| Error::Corrupt { place: "ledger record length" })?) as usize;
    if len < 8 + 32 + 8 + 1 + 4 + 16 + 1 + 32 + 1 + 32 || len > MAX_LEDGER_RECORD_BYTES { return Err(Error::Corrupt { place: "ledger record length" }); }
    let total = len.checked_add(4).ok_or(Error::Corrupt { place: "ledger record length" })?;
    let encoded = input.get(4..total).ok_or(Error::Corrupt { place: "ledger record truncated" })?;
    let (body, received) = encoded.split_at(encoded.len() - 32);
    let actual = mac.tag(key, body)?;
    if actual.as_slice() != received { return Err(Error::Corrupt { place: "ledger chain MAC" }); }
    let mut c = Cursor { input: body, offset: 0 };
    let sequence = c.u64()?; let prev = c.array32()?; let clock_sample = c.u64()?;
    if sequence != expected_sequence || prev != previous_hash { return Err(Error::Corrupt { place: "ledger chain" }); }
    let event = decode_event(&mut c)?;
    if !c.done() { return Err(Error::Corrupt { place: "ledger trailing bytes" }); }
    Ok((LedgerRecord { sequence, previous_hash: prev, clock_sample, event, mac: actual }, total))
}

fn encode_body(sequence: u64, previous_hash: &[u8; 32], clock: u64, event: &LedgerEvent) -> Result<Vec<u8>, Error> {
    if event.tombs.len() > MAX_INTENT_TOMBS || event.scope_ciphertext.len() > MAX_LEDGER_RECORD_BYTES { return Err(Error::Limit { limit: "ledger event bounds" }); }
    let mut tombs = event.tombs.clone(); tombs.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    if tombs != event.tombs || tombs.windows(2).any(|w| w[0].as_bytes() == w[1].as_bytes()) { return Err(Error::Limit { limit: "ledger tomb IDs must be sorted and unique" }); }
    let mut out = Vec::new(); out.extend_from_slice(&sequence.to_le_bytes()); out.extend_from_slice(previous_hash); out.extend_from_slice(&clock.to_le_bytes());
    out.push(event.kind as u8); out.extend_from_slice(&event.tomb_epoch.to_le_bytes()); out.extend_from_slice(event.ticket.as_bytes());
    varint_put(&mut out, tombs.len() as u64); for tomb in tombs { out.extend_from_slice(tomb.as_bytes()); }
    out.extend_from_slice(&event.evidence_digest); varint_put(&mut out, event.scope_ciphertext.len() as u64); out.extend_from_slice(&event.scope_ciphertext);
    Ok(out)
}

fn decode_event(c: &mut Cursor<'_>) -> Result<LedgerEvent, Error> {
    let kind = LedgerEventKind::try_from(c.byte()?)?; let tomb_epoch = c.u32()?; let ticket = Ticket::from_bytes(c.array16()?);
    let count = usize::try_from(c.varint()?).map_err(|_| Error::Corrupt { place: "ledger tomb count" })?;
    if count > MAX_INTENT_TOMBS { return Err(Error::Limit { limit: "ledger tomb count" }); }
    let mut tombs = Vec::with_capacity(count); for _ in 0..count { tombs.push(TombId::from_bytes(c.array32()?)); }
    if tombs.windows(2).any(|w| w[0].as_bytes() >= w[1].as_bytes()) { return Err(Error::Corrupt { place: "ledger tomb ordering" }); }
    let evidence_digest = c.array32()?; let scope_len = usize::try_from(c.varint()?).map_err(|_| Error::Corrupt { place: "ledger scope length" })?;
    if scope_len > MAX_LEDGER_RECORD_BYTES { return Err(Error::Limit { limit: "ledger scope length" }); }
    Ok(LedgerEvent { kind, tomb_epoch, ticket, tombs, evidence_digest, scope_ciphertext: c.take(scope_len)?.to_vec() })
}

struct Cursor<'a> { input: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
 fn take(&mut self,n:usize)->Result<&'a[u8],Error>{let e=self.offset.checked_add(n).ok_or(Error::Corrupt{place:"ledger cursor"})?;let v=self.input.get(self.offset..e).ok_or(Error::Corrupt{place:"ledger truncated"})?;self.offset=e;Ok(v)}
 fn byte(&mut self)->Result<u8,Error>{Ok(self.take(1)?[0])} fn u32(&mut self)->Result<u32,Error>{Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(|_|Error::Corrupt{place:"ledger u32"})?))} fn u64(&mut self)->Result<u64,Error>{Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(|_|Error::Corrupt{place:"ledger u64"})?))}
 fn array16(&mut self)->Result<[u8;16],Error>{self.take(16)?.try_into().map_err(|_|Error::Corrupt{place:"ledger array16"})} fn array32(&mut self)->Result<[u8;32],Error>{self.take(32)?.try_into().map_err(|_|Error::Corrupt{place:"ledger array32"})}
 fn varint(&mut self)->Result<u64,Error>{let (v,n)=varint_get(&self.input[self.offset..]).map_err(|_|Error::Corrupt{place:"ledger varint"})?;self.offset+=n;Ok(v)} fn done(&self)->bool{self.offset==self.input.len()}
}
