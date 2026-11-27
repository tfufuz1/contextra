//! Tamper-evidentes, append-only Audit-Ledger (§11, WP-10).
//!
//! Datei `<ledger_path>/ledger.log` außerhalb des Datenverzeichnisses.

use crate::erasure::journal::{
    decode_exceptions, encode_exceptions, ByteReader,
};
use crate::io_seam::StoreIo;
use contextra_ports::ledger::{
    LedgerAnchor, LedgerDraft, LedgerEntry, LedgerHead, LedgerSink, LedgerVerification,
};
use contextra_ports::vfs::{VfsFile, VfsOpenOptions};
use contextra_ports::BoxFuture;
use contextra_types::{ErasureError, Guarantee, Timestamp};
use parking_lot::Mutex;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const LEDGER_MAGIC: &[u8; 8] = b"CTXLDG01";
const LEDGER_VERSION: u16 = 1;
const LEDGER_HEADER_LEN: usize = 16;
const LEDGER_FILE_NAME: &str = "ledger.log";

/// Normalisiert einen Pfad rein lexikalisch (ohne Dateisystemzugriff, §11.1).
fn normalize_lexical(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            c => out.push(c),
        }
    }
    out
}

/// Prüft, ob `ledger_path` außerhalb von `data_dir` liegt (§11.1).
pub(crate) fn validate_external_ledger_path(
    ledger_path: &Path,
    data_dir: &Path,
) -> Result<(), ErasureError> {
    let norm_ledger = normalize_lexical(ledger_path);
    let norm_data = normalize_lexical(data_dir);
    if norm_ledger.starts_with(&norm_data) {
        return Err(ErasureError::InvalidTarget(
            "ledger_path must reside outside the store data directory",
        ));
    }
    Ok(())
}

fn encode_guarantee(g: Guarantee) -> u8 {
    let mut b = 0u8;
    if g.gated {
        b |= 1 << 0;
    }
    if g.purged {
        b |= 1 << 1;
    }
    if g.crypto_erased {
        b |= 1 << 2;
    }
    b
}

fn decode_guarantee(b: u8) -> Result<Guarantee, ErasureError> {
    Guarantee::new((b & 1) != 0, (b & 2) != 0, (b & 4) != 0)
}

fn encode_payload_without_mac(
    index: u64,
    draft: &LedgerDraft,
    prev_hash: &[u8; 32],
) -> Result<Vec<u8>, ErasureError> {
    let mut out = Vec::with_capacity(192);
    out.extend_from_slice(&index.to_le_bytes());
    out.extend_from_slice(&draft.erasure_id_hmac);
    out.push(encode_guarantee(draft.guarantee));
    out.extend_from_slice(&draft.layers_root);
    encode_exceptions(&draft.exceptions, &mut out)?;
    out.extend_from_slice(&draft.plan_hash);
    out.extend_from_slice(&draft.policy_version.to_le_bytes());
    out.extend_from_slice(&draft.completed_at.as_u64().to_le_bytes());
    out.extend_from_slice(prev_hash);
    Ok(out)
}

fn encode_entry_payload(entry: &LedgerEntry) -> Result<Vec<u8>, ErasureError> {
    let draft = LedgerDraft {
        erasure_id_hmac: entry.erasure_id_hmac,
        guarantee: entry.guarantee,
        layers_root: entry.layers_root,
        exceptions: entry.exceptions.clone(),
        plan_hash: entry.plan_hash,
        policy_version: entry.policy_version,
        completed_at: entry.completed_at,
    };
    let mut out = encode_payload_without_mac(entry.index, &draft, &entry.prev_hash)?;
    out.extend_from_slice(&entry.entry_mac);
    Ok(out)
}

fn decode_entry_payload(payload: &[u8]) -> Result<(LedgerEntry, [u8; 32]), ErasureError> {
    if payload.len() < 32 {
        return Err(ErasureError::LedgerCorrupt);
    }
    let without_mac_len = payload.len() - 32;
    let without_mac = payload
        .get(..without_mac_len)
        .ok_or(ErasureError::LedgerCorrupt)?;
    let entry_hash = *blake3::hash(without_mac).as_bytes();

    let mut r = ByteReader::new(payload);
    let index = r.read_u64().map_err(|_| ErasureError::LedgerCorrupt)?;
    let erasure_id_hmac = r.read_arr32().map_err(|_| ErasureError::LedgerCorrupt)?;
    let g_byte = r.read_u8().map_err(|_| ErasureError::LedgerCorrupt)?;
    let guarantee = decode_guarantee(g_byte).map_err(|_| ErasureError::LedgerCorrupt)?;
    let layers_root = r.read_arr32().map_err(|_| ErasureError::LedgerCorrupt)?;
    let exceptions = decode_exceptions(&mut r).map_err(|_| ErasureError::LedgerCorrupt)?;
    let plan_hash = r.read_arr32().map_err(|_| ErasureError::LedgerCorrupt)?;
    let policy_version = r.read_u32().map_err(|_| ErasureError::LedgerCorrupt)?;
    let completed_nanos = r.read_u64().map_err(|_| ErasureError::LedgerCorrupt)?;
    let prev_hash = r.read_arr32().map_err(|_| ErasureError::LedgerCorrupt)?;
    let entry_mac = r.read_arr32().map_err(|_| ErasureError::LedgerCorrupt)?;
    if !r.is_empty() {
        return Err(ErasureError::LedgerCorrupt);
    }

    Ok((
        LedgerEntry {
            index,
            erasure_id_hmac,
            guarantee,
            layers_root,
            exceptions,
            plan_hash,
            policy_version,
            completed_at: Timestamp::new(completed_nanos),
            prev_hash,
            entry_mac,
        },
        entry_hash,
    ))
}

/// Durables, kryptographisch verkettetes Audit-Ledger (§11).
pub(crate) struct StoreLedger {
    io: StoreIo,
    path: PathBuf,
    ledger_key: [u8; 32],
    file: Arc<Mutex<Box<dyn VfsFile>>>,
    inner: tokio::sync::Mutex<LedgerState>,
    corrupt: AtomicBool,
    anchor: Option<Arc<dyn LedgerAnchor>>,
}

struct LedgerState {
    entries: Vec<LedgerEntry>,
    head: LedgerHead,
    offset: u64,
}

impl StoreLedger {
    /// Öffnet oder initialisiert `<ledger_dir>/ledger.log` und prüft die gesamte Kette.
    pub async fn open(
        io: StoreIo,
        ledger_dir: PathBuf,
        ledger_key: [u8; 32],
        anchor: Option<Arc<dyn LedgerAnchor>>,
    ) -> Result<Self, ErasureError> {
        io.create_dir_all(&ledger_dir)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        let path = ledger_dir.join(LEDGER_FILE_NAME);
        let exists = io
            .exists(&path)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;

        if !exists {
            let io_c = io.clone();
            let path_c = path.clone();
            let dir_c = ledger_dir.clone();
            let file = tokio::task::spawn_blocking(move || -> Result<Box<dyn VfsFile>, ErasureError> {
                let opts = VfsOpenOptions {
                    read: true,
                    write: true,
                    create: true,
                    ..Default::default()
                };
                let f = io_c
                    .open(&path_c, &opts)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                let mut hdr = [0u8; LEDGER_HEADER_LEN];
                hdr[..8].copy_from_slice(LEDGER_MAGIC);
                hdr[8..10].copy_from_slice(&LEDGER_VERSION.to_le_bytes());
                f.write_at(&hdr, 0)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                f.sync_all()
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                io_c.vfs()
                    .sync_dir(&dir_c)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                Ok(f)
            })
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))??;

            return Ok(Self {
                io,
                path,
                ledger_key,
                file: Arc::new(Mutex::new(file)),
                inner: tokio::sync::Mutex::new(LedgerState {
                    entries: Vec::new(),
                    head: LedgerHead {
                        index: 0,
                        hash: [0u8; 32],
                    },
                    offset: LEDGER_HEADER_LEN as u64,
                }),
                corrupt: AtomicBool::new(false),
                anchor,
            });
        }

        let bytes = io
            .read(&path)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        let (entries, head, offset, is_corrupt) = Self::parse_and_verify(&bytes, &ledger_key);

        let io_c = io.clone();
        let path_c = path.clone();
        let file = tokio::task::spawn_blocking(move || -> Result<Box<dyn VfsFile>, ErasureError> {
            let opts = VfsOpenOptions {
                read: true,
                write: true,
                ..Default::default()
            };
            io_c.open(&path_c, &opts)
                .map_err(|e| ErasureError::Io(e.to_string()))
        })
        .await
        .map_err(|e| ErasureError::Io(e.to_string()))??;

        Ok(Self {
            io,
            path,
            ledger_key,
            file: Arc::new(Mutex::new(file)),
            inner: tokio::sync::Mutex::new(LedgerState {
                entries,
                head,
                offset,
            }),
            corrupt: AtomicBool::new(is_corrupt),
            anchor,
        })
    }

    pub fn is_corrupt(&self) -> bool {
        self.corrupt.load(Ordering::Acquire)
    }

    pub fn publish_anchor_best_effort(&self, head: LedgerHead) {
        if let Some(ref anchor) = self.anchor {
            let anchor = Arc::clone(anchor);
            tokio::spawn(async move {
                if let Err(e) = anchor.publish(&head).await {
                    tracing::warn!(error = %e, index = head.index, "LedgerAnchor::publish failed");
                }
            });
        }
    }

    fn parse_and_verify(
        bytes: &[u8],
        ledger_key: &[u8; 32],
    ) -> (Vec<LedgerEntry>, LedgerHead, u64, bool) {
        if bytes.len() < LEDGER_HEADER_LEN
            || bytes.get(..8) != Some(LEDGER_MAGIC.as_slice())
            || bytes
                .get(8..10)
                .and_then(|s| <[u8; 2]>::try_from(s).ok())
                .map(u16::from_le_bytes)
                != Some(LEDGER_VERSION)
        {
            return (
                Vec::new(),
                LedgerHead {
                    index: 0,
                    hash: [0u8; 32],
                },
                0,
                true,
            );
        }

        let mut pos = LEDGER_HEADER_LEN;
        let mut entries = Vec::new();
        let mut expected_prev_hash = [0u8; 32];
        let mut expected_index = 0u64;
        let mut head = LedgerHead {
            index: 0,
            hash: [0u8; 32],
        };

        while pos < bytes.len() {
            let Some(len_bytes) = bytes.get(pos..pos + 4).and_then(|s| <[u8; 4]>::try_from(s).ok()) else {
                return (entries, head, pos as u64, true);
            };
            let body_len = u32::from_le_bytes(len_bytes) as usize;
            let Some(crc_bytes) = bytes.get(pos + 4..pos + 8).and_then(|s| <[u8; 4]>::try_from(s).ok()) else {
                return (entries, head, pos as u64, true);
            };
            let expected_crc = u32::from_le_bytes(crc_bytes);
            let frame_end = pos + 8 + body_len;
            let Some(body) = bytes.get(pos + 8..frame_end) else {
                return (entries, head, pos as u64, true);
            };
            if crc32fast::hash(body) != expected_crc {
                return (entries, head, pos as u64, true);
            }
            if body.first().copied() != Some(1) {
                return (entries, head, pos as u64, true);
            }
            let payload = body.get(1..).unwrap_or(&[]);
            let Ok((entry, entry_hash)) = decode_entry_payload(payload) else {
                return (entries, head, pos as u64, true);
            };

            if entry.index != expected_index || entry.prev_hash != expected_prev_hash {
                return (entries, head, pos as u64, true);
            }

            let without_mac = payload.get(..payload.len().saturating_sub(32)).unwrap_or(&[]);
            let Ok(expected_mac) = contextra_crypto::ledger_mac(ledger_key, without_mac) else {
                return (entries, head, pos as u64, true);
            };
            if entry.entry_mac != expected_mac {
                return (entries, head, pos as u64, true);
            }

            head = LedgerHead {
                index: entry.index,
                hash: entry_hash,
            };
            expected_prev_hash = entry_hash;
            expected_index += 1;
            entries.push(entry);
            pos = frame_end;
        }

        (entries, head, pos as u64, false)
    }
}

impl LedgerSink for StoreLedger {
    fn append<'a>(&'a self, e: LedgerDraft) -> BoxFuture<'a, Result<LedgerEntry, ErasureError>> {
        Box::pin(async move {
            if self.is_corrupt() {
                return Err(ErasureError::LedgerCorrupt);
            }
            let mut st = self.inner.lock().await;
            if let Some(existing) = st
                .entries
                .iter()
                .find(|x| x.erasure_id_hmac == e.erasure_id_hmac && x.plan_hash == e.plan_hash)
            {
                return Ok(existing.clone());
            }

            let index = st.entries.len() as u64;
            let prev_hash = st.head.hash;
            let without_mac = encode_payload_without_mac(index, &e, &prev_hash)?;
            let entry_hash = *blake3::hash(&without_mac).as_bytes();
            let entry_mac = contextra_crypto::ledger_mac(&self.ledger_key, &without_mac)
                .map_err(|_| ErasureError::LedgerCorrupt)?;

            let entry = LedgerEntry {
                index,
                erasure_id_hmac: e.erasure_id_hmac,
                guarantee: e.guarantee,
                layers_root: e.layers_root,
                exceptions: e.exceptions,
                plan_hash: e.plan_hash,
                policy_version: e.policy_version,
                completed_at: e.completed_at,
                prev_hash,
                entry_mac,
            };

            let payload = encode_entry_payload(&entry)?;
            let body_len = u32::try_from(1 + payload.len())
                .map_err(|_| ErasureError::InvalidTarget("ledger entry too large"))?;
            let mut body = Vec::with_capacity(1 + payload.len());
            body.push(1u8);
            body.extend_from_slice(&payload);
            let crc = crc32fast::hash(&body);

            let mut frame = Vec::with_capacity(8 + body.len());
            frame.extend_from_slice(&body_len.to_le_bytes());
            frame.extend_from_slice(&crc.to_le_bytes());
            frame.extend_from_slice(&body);

            let write_offset = st.offset;
            let f_arc = Arc::clone(&self.file);
            let frame_len = frame.len() as u64;

            tokio::task::spawn_blocking(move || -> Result<(), ErasureError> {
                let f = f_arc.lock();
                f.write_at(&frame, write_offset)
                    .map_err(|err| ErasureError::Io(err.to_string()))?;
                f.sync_data()
                    .map_err(|err| ErasureError::Io(err.to_string()))?;
                Ok(())
            })
            .await
            .map_err(|err| ErasureError::Io(err.to_string()))??;

            st.offset = write_offset.saturating_add(frame_len);
            st.head = LedgerHead {
                index,
                hash: entry_hash,
            };
            st.entries.push(entry.clone());
            Ok(entry)
        })
    }

    fn head<'a>(&'a self) -> BoxFuture<'a, Result<LedgerHead, ErasureError>> {
        Box::pin(async move {
            if self.is_corrupt() {
                return Err(ErasureError::LedgerCorrupt);
            }
            let st = self.inner.lock().await;
            Ok(st.head)
        })
    }

    fn verify<'a>(&'a self) -> BoxFuture<'a, Result<LedgerVerification, ErasureError>> {
        Box::pin(async move {
            let bytes = self
                .io
                .read(&self.path)
                .await
                .map_err(|e| ErasureError::Io(e.to_string()))?;
            let (entries, head, _offset, is_corrupt) =
                Self::parse_and_verify(&bytes, &self.ledger_key);
            if is_corrupt {
                self.corrupt.store(true, Ordering::Release);
                return Err(ErasureError::LedgerCorrupt);
            }
            Ok(LedgerVerification {
                valid: true,
                entries_checked: entries.len() as u64,
                head,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ledger_append_idempotent_and_tamper_detect() {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().join("data");
        let ledger_dir = dir.path().join("ledger");
        assert!(validate_external_ledger_path(&data_dir.join("l"), &data_dir).is_err());
        assert!(validate_external_ledger_path(&ledger_dir, &data_dir).is_ok());

        let io = StoreIo::default();
        let key = [0x42u8; 32];
        let ledger = StoreLedger::open(io.clone(), ledger_dir.clone(), key, None)
            .await
            .unwrap();

        let draft = LedgerDraft {
            erasure_id_hmac: [0x11; 32],
            guarantee: Guarantee::new(true, true, false).unwrap(),
            layers_root: [0x22; 32],
            exceptions: vec![contextra_types::ErasureException::OutOfScopeExternalCopies],
            plan_hash: [0x33; 32],
            policy_version: 1,
            completed_at: Timestamp::new(12345),
        };

        let e1 = ledger.append(draft.clone()).await.unwrap();
        let e2 = ledger.append(draft).await.unwrap();
        assert_eq!(e1, e2);

        let v = ledger.verify().await.unwrap();
        assert!(v.valid);
        assert_eq!(v.entries_checked, 1);

        // Tamper a byte in ledger.log
        let log_path = ledger_dir.join(LEDGER_FILE_NAME);
        let mut raw = std::fs::read(&log_path).unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0x01;
        std::fs::write(&log_path, &raw).unwrap();

        assert!(matches!(ledger.verify().await, Err(ErasureError::LedgerCorrupt)));
        assert!(ledger.is_corrupt());
    }
}
