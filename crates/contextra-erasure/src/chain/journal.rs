//! Durables Saga-Journal (SPEC §5.3 fsync-Anker, §5.5 Recovery).
//!
//! Dateiformat (neu, versioniert, INV-1):
//! - Kopf: `b"CTXCHAIN"` + Version `0x01`.
//! - Frame: `u32` Länge (BE) | Payload | 16 Byte BLAKE3-Prüfsumme des Payloads.
//!
//! Ein abgerissener Frame am Dateiende ist ein nicht bestätigter Schreibvorgang und wird
//! verworfen. Eine Prüfsummenabweichung in einem vollständigen Frame ist Korruption und
//! führt zu einem Fehler (fail-closed).

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use contextra_ids::ErasureId;
use parking_lot::Mutex;

use crate::error::ErasureError;
use crate::types_local::OperatorRef;

use super::ports::Journal;
use super::retention::{RetentionAttestation, RetentionCode};
use super::state::{PendingReason, SagaState};
use super::target::EraseTarget;

const MAGIC: &[u8; 8] = b"CTXCHAIN";
const FORMAT_VERSION: u8 = 1;
const HEADER_LEN: usize = 9;
const CHECKSUM_LEN: usize = 16;
const NO_RESUME: u8 = 0;

/// Ein Journal-Eintrag. Der Zustand ist nur wahr, wenn der Eintrag durabel ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalRecord {
    /// Laufende Sequenznummer.
    pub seq: u64,
    /// Saga-Kennung.
    pub erasure_id: ErasureId,
    /// Löschziel.
    pub target: EraseTarget,
    /// Aufbewahrungs-Attestierung.
    pub attestation: RetentionAttestation,
    /// Zustand nach diesem Eintrag.
    pub state: SagaState,
    /// Bei `Pending`: Zustand, in den nach Auflösung zurückgekehrt wird.
    pub resume: Option<SagaState>,
}

fn state_code(s: SagaState) -> u8 {
    match s {
        SagaState::Requested => 1,
        SagaState::Gated => 2,
        SagaState::KeyRevoked => 3,
        SagaState::Purging => 4,
        SagaState::Verifying => 5,
        SagaState::Completed => 6,
        SagaState::Restricted => 7,
        SagaState::Lifted => 8,
        SagaState::Pending(_) => 9,
        SagaState::Refused => 10,
    }
}

fn reason_code(r: PendingReason) -> u8 {
    match r {
        PendingReason::DiskReserveLow => 1,
        PendingReason::CompactionRunning => 2,
        PendingReason::IoBackoff => 3,
    }
}

fn state_from(code: u8, reason: u8) -> Result<SagaState, ErasureError> {
    let pending = || match reason {
        1 => Ok(PendingReason::DiskReserveLow),
        2 => Ok(PendingReason::CompactionRunning),
        3 => Ok(PendingReason::IoBackoff),
        _ => Err(ErasureError::Corrupt),
    };
    Ok(match code {
        1 => SagaState::Requested,
        2 => SagaState::Gated,
        3 => SagaState::KeyRevoked,
        4 => SagaState::Purging,
        5 => SagaState::Verifying,
        6 => SagaState::Completed,
        7 => SagaState::Restricted,
        8 => SagaState::Lifted,
        9 => SagaState::Pending(pending()?),
        10 => SagaState::Refused,
        _ => return Err(ErasureError::Corrupt),
    })
}

fn encode_payload(rec: &JournalRecord) -> Result<Vec<u8>, ErasureError> {
    let target = rec.target.encode()?;
    let by = rec.attestation.attested_by.as_bytes();
    let by_len = u16::try_from(by.len()).map_err(|_| ErasureError::Corrupt)?;
    let target_len = u16::try_from(target.len()).map_err(|_| ErasureError::Corrupt)?;

    let mut out = Vec::with_capacity(64 + target.len() + by.len());
    out.extend_from_slice(&rec.seq.to_be_bytes());
    out.extend_from_slice(&rec.erasure_id.0);
    out.push(state_code(rec.state));
    out.push(match rec.state {
        SagaState::Pending(r) => reason_code(r),
        _ => 0,
    });
    out.push(rec.resume.map_or(NO_RESUME, state_code));
    out.push(rec.attestation.code.to_u8());
    out.extend_from_slice(&rec.attestation.attested_at.0.to_be_bytes());
    out.extend_from_slice(&by_len.to_be_bytes());
    out.extend_from_slice(by);
    out.extend_from_slice(&target_len.to_be_bytes());
    out.extend_from_slice(&target);
    Ok(out)
}

/// Lesehilfe ohne Indexzugriff.
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ErasureError> {
        let end = self.pos.checked_add(n).ok_or(ErasureError::Corrupt)?;
        let s = self.buf.get(self.pos..end).ok_or(ErasureError::Corrupt)?;
        self.pos = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, ErasureError> {
        Ok(*self.take(1)?.first().ok_or(ErasureError::Corrupt)?)
    }
    fn u16(&mut self) -> Result<u16, ErasureError> {
        let mut a = [0u8; 2];
        a.copy_from_slice(self.take(2)?);
        Ok(u16::from_be_bytes(a))
    }
    fn u64(&mut self) -> Result<u64, ErasureError> {
        let mut a = [0u8; 8];
        a.copy_from_slice(self.take(8)?);
        Ok(u64::from_be_bytes(a))
    }
}

fn decode_payload(payload: &[u8]) -> Result<JournalRecord, ErasureError> {
    let mut r = Reader {
        buf: payload,
        pos: 0,
    };
    let seq = r.u64()?;
    let mut id = [0u8; 16];
    id.copy_from_slice(r.take(16)?);
    let state_c = r.u8()?;
    let reason_c = r.u8()?;
    let state = state_from(state_c, reason_c)?;
    let resume_c = r.u8()?;
    let resume = if resume_c == NO_RESUME {
        None
    } else {
        Some(state_from(resume_c, 0).map_err(|_| ErasureError::Corrupt)?)
    };
    let code = RetentionCode::from_u8(r.u8()?).ok_or(ErasureError::Corrupt)?;
    let attested_at = contextra_ids::Timestamp(r.u64()?);
    let by_len = usize::from(r.u16()?);
    let by = OperatorRef::try_new(r.take(by_len)?.to_vec()).map_err(|_| ErasureError::Corrupt)?;
    let target_len = usize::from(r.u16()?);
    let target = EraseTarget::decode(r.take(target_len)?)?;
    if r.pos != payload.len() {
        return Err(ErasureError::Corrupt);
    }
    Ok(JournalRecord {
        seq,
        erasure_id: ErasureId(id),
        target,
        attestation: RetentionAttestation {
            code,
            attested_by: by,
            attested_at,
        },
        state,
        resume,
    })
}

fn checksum(payload: &[u8]) -> Result<[u8; CHECKSUM_LEN], ErasureError> {
    let digest = blake3::hash(payload);
    let mut c = [0u8; CHECKSUM_LEN];
    c.copy_from_slice(
        digest
            .as_bytes()
            .get(..CHECKSUM_LEN)
            .ok_or(ErasureError::Corrupt)?,
    );
    Ok(c)
}

/// Dateibasiertes Journal mit fsync nach jedem Eintrag.
pub struct FileJournal {
    path: PathBuf,
    file: Mutex<File>,
}

impl FileJournal {
    /// Öffnet das Journal oder legt es mit Kopf an.
    pub fn open(path: &Path) -> Result<Self, ErasureError> {
        let exists = path.exists();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;
        if !exists || file.metadata()?.len() == 0 {
            let mut header = Vec::with_capacity(HEADER_LEN);
            header.extend_from_slice(MAGIC);
            header.push(FORMAT_VERSION);
            file.write_all(&header)?;
            file.sync_all()?;
        }
        Ok(Self {
            path: path.to_path_buf(),
            file: Mutex::new(file),
        })
    }

    /// Pfad des Journals.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Journal for FileJournal {
    fn append(&self, record: &JournalRecord) -> Result<(), ErasureError> {
        let payload = encode_payload(record)?;
        let len = u32::try_from(payload.len()).map_err(|_| ErasureError::Corrupt)?;
        let mut frame = Vec::with_capacity(4 + payload.len() + CHECKSUM_LEN);
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(&payload);
        frame.extend_from_slice(&checksum(&payload)?);

        let mut file = self.file.lock();
        file.write_all(&frame)?;
        // fsync-Anker: erst nach sync_data gilt der Zustand als durabel.
        file.sync_data()?;
        Ok(())
    }

    fn load(&self) -> Result<Vec<JournalRecord>, ErasureError> {
        let mut bytes = Vec::new();
        File::open(&self.path)?.read_to_end(&mut bytes)?;
        let body = match bytes.get(..HEADER_LEN) {
            Some(h)
                if h.get(..MAGIC.len()) == Some(&MAGIC[..])
                    && h.get(MAGIC.len()) == Some(&FORMAT_VERSION) =>
            {
                bytes.get(HEADER_LEN..).ok_or(ErasureError::Corrupt)?
            }
            _ => return Err(ErasureError::Corrupt),
        };

        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos < body.len() {
            let Some(len_bytes) = body.get(pos..pos + 4) else {
                break;
            };
            let mut l = [0u8; 4];
            l.copy_from_slice(len_bytes);
            let len = usize::try_from(u32::from_be_bytes(l)).map_err(|_| ErasureError::Corrupt)?;
            let payload_start = pos + 4;
            let frame_end = payload_start + len + CHECKSUM_LEN;
            let Some(frame) = body.get(payload_start..frame_end) else {
                // Abgerissener Frame am Ende: nicht bestätigt, verwerfen.
                break;
            };
            let (payload, stored) = frame.split_at(len);
            if checksum(payload)?.as_slice() != stored {
                return Err(ErasureError::Corrupt);
            }
            out.push(decode_payload(payload)?);
            pos = frame_end;
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use contextra_ids::{CollectionId, DocId, TenantId, Timestamp};

    use super::*;

    fn rec(seq: u64, state: SagaState) -> JournalRecord {
        JournalRecord {
            seq,
            erasure_id: ErasureId([7; 16]),
            target: EraseTarget::Document {
                tenant: TenantId(1),
                collection: CollectionId(2),
                doc: DocId(3),
            },
            attestation: RetentionAttestation {
                code: RetentionCode::NoObligation,
                attested_by: OperatorRef::try_new(b"op-1".to_vec()).expect("op"),
                attested_at: Timestamp(99),
            },
            state,
            resume: None,
        }
    }

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ctx-chain-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        dir.join("chain.journal")
    }

    #[test]
    fn append_und_load_erhalten_reihenfolge_und_inhalt() {
        let p = tmp("roundtrip");
        let j = FileJournal::open(&p).expect("open");
        j.append(&rec(1, SagaState::Requested)).expect("a1");
        j.append(&rec(2, SagaState::Gated)).expect("a2");
        let reopened = FileJournal::open(&p).expect("reopen");
        let loaded = reopened.load().expect("load");
        assert_eq!(
            loaded,
            vec![rec(1, SagaState::Requested), rec(2, SagaState::Gated)]
        );
    }

    #[test]
    fn pending_mit_resume_bleibt_erhalten() {
        let p = tmp("pending");
        let j = FileJournal::open(&p).expect("open");
        let mut r = rec(1, SagaState::Pending(PendingReason::DiskReserveLow));
        r.resume = Some(SagaState::Purging);
        j.append(&r).expect("append");
        assert_eq!(j.load().expect("load"), vec![r]);
    }

    #[test]
    fn abgerissener_letzter_frame_wird_verworfen() {
        let p = tmp("torn");
        let j = FileJournal::open(&p).expect("open");
        j.append(&rec(1, SagaState::Requested)).expect("a1");
        j.append(&rec(2, SagaState::Gated)).expect("a2");
        let mut bytes = std::fs::read(&p).expect("read");
        bytes.truncate(bytes.len() - 5);
        std::fs::write(&p, &bytes).expect("write");
        let loaded = FileJournal::open(&p).expect("open").load().expect("load");
        assert_eq!(loaded, vec![rec(1, SagaState::Requested)]);
    }

    #[test]
    fn prüfsummenfehler_im_vollstaendigen_frame_ist_korruption() {
        let p = tmp("corrupt");
        let j = FileJournal::open(&p).expect("open");
        j.append(&rec(1, SagaState::Requested)).expect("a1");
        let mut bytes = std::fs::read(&p).expect("read");
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        std::fs::write(&p, &bytes).expect("write");
        assert!(matches!(
            FileJournal::open(&p).expect("open").load(),
            Err(ErasureError::Corrupt)
        ));
    }

    #[test]
    fn falscher_kopf_ist_korruption() {
        let p = tmp("header");
        std::fs::write(&p, b"NOTACHAIN").expect("write");
        assert!(matches!(
            FileJournal::open(&p).expect("open").load(),
            Err(ErasureError::Corrupt)
        ));
    }
}
