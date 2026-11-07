//! Persistentes Append-Log des Löschjournals journal.log (SPEC §11.6, Anhang F.1 kind=6, F.9, ER-29).

use super::record::{JournalEntry, JournalRecord, RecordCode};
use crate::crypto_local::Dek;
use crate::crypto_local::SecretKey;
use crate::crypto_local::{open_record, seal_record};
use crate::error::ErasureError;
use crate::saga::{EraseSaga, SagaState};
use crate::types_local::ErasureId;
use contextra_ports::vfs::Vfs;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const FILE_HEADER_MAGIC: &[u8; 8] = b"CTXFILE\0";
const FILE_KIND_JOURNAL: u8 = 6;
const JOURNAL_GENESIS_CHAIN: [u8; 32] = [0u8; 32];

fn vfs_err(e: contextra_ports::ContextraError) -> ErasureError {
    ErasureError::Io(std::io::Error::other(e.to_string()))
}

/// Dauerhaftes Journal auf Basis von Vfs (SPEC §11.6, Anhang F.1, F.9).
pub struct DurableJournal {
    vfs: Arc<dyn Vfs>,
    path: PathBuf,
    derived_key: [u8; 32],
    records: Vec<JournalRecord>,
    chain_tail: [u8; 32],
}

impl DurableJournal {
    /// Öffnet ein existierendes Journal oder legt ein neues an.
    pub fn open_or_create(
        vfs: Arc<dyn Vfs>,
        path: &Path,
        derived_key: &[u8; 32],
    ) -> Result<Self, ErasureError> {
        let exists = vfs.exists(path).map_err(vfs_err)?;
        if !exists {
            // Header anlegen (F.1 Dateikopf kind=6, key_mode=1)
            let mut header = [0u8; 64];
            if let Some(dest) = header.get_mut(0..8) {
                dest.copy_from_slice(FILE_HEADER_MAGIC);
            }
            if let Some(dest) = header.get_mut(8..10) {
                dest.copy_from_slice(&1u16.to_be_bytes());
            }
            if let Some(dest) = header.get_mut(10) {
                *dest = FILE_KIND_JOURNAL;
            }
            if let Some(dest) = header.get_mut(11..15) {
                dest.copy_from_slice(&[1, 0, 1, 1]);
            }
            if let Some(dest) = header.get_mut(39) {
                *dest = 1; // key_mode = 1 (abgeleiteter Schlüssel)
            }

            // Header-MAC über Bytes [0..48)
            let header48 = header.get(0..48).ok_or(ErasureError::Corrupt)?;
            let hmac = blake3::keyed_hash(derived_key, header48);
            if let (Some(dest), Some(src)) = (header.get_mut(48..64), hmac.as_bytes().get(0..16)) {
                dest.copy_from_slice(src);
            }

            if let Some(parent) = path.parent() {
                vfs.create_dir_all(parent).map_err(vfs_err)?;
            }
            vfs.write_atomic(path, &header).map_err(vfs_err)?;

            return Ok(Self {
                vfs,
                path: path.to_path_buf(),
                derived_key: *derived_key,
                records: Vec::new(),
                chain_tail: JOURNAL_GENESIS_CHAIN,
            });
        }

        // Datei lesen und parsen
        let bytes = vfs.read(path).map_err(vfs_err)?;
        if bytes.len() < 64 {
            return Err(ErasureError::Corrupt);
        }

        let header = bytes.get(0..64).ok_or(ErasureError::Corrupt)?;
        let body = bytes.get(64..).ok_or(ErasureError::Corrupt)?;

        if header.get(0..8) != Some(FILE_HEADER_MAGIC) {
            return Err(ErasureError::Corrupt);
        }
        if header.get(10).copied() != Some(FILE_KIND_JOURNAL) {
            return Err(ErasureError::Corrupt);
        }

        // Header-MAC prüfen
        let header48 = header.get(0..48).ok_or(ErasureError::Corrupt)?;
        let expected_hmac = blake3::keyed_hash(derived_key, header48);
        let tag = header.get(48..64).ok_or(ErasureError::Corrupt)?;
        let expected_tag = expected_hmac
            .as_bytes()
            .get(0..16)
            .ok_or(ErasureError::Corrupt)?;
        if tag != expected_tag {
            return Err(ErasureError::Corrupt);
        }

        // Einträge parsen (F.9): len(u32) || ct || aead_tag(16) || chain(32)
        let mut records = Vec::new();
        let mut offset = 0;
        let mut chain_prev = JOURNAL_GENESIS_CHAIN;

        let dek = Dek(SecretKey::new(*derived_key));
        let key_binding = b"ctx/journal/v1";

        while offset < body.len() {
            if offset + 4 > body.len() {
                return Err(ErasureError::Corrupt);
            }

            let len_slice = body.get(offset..offset + 4).ok_or(ErasureError::Corrupt)?;
            let mut len_arr = [0u8; 4];
            len_arr.copy_from_slice(len_slice);
            let len =
                usize::try_from(u32::from_be_bytes(len_arr)).map_err(|_| ErasureError::Corrupt)?;

            let total_len = 4 + len + 16 + 32;
            if offset + total_len > body.len() {
                // Unvollständiger Block am Dateiende -> Torn-Tail-Erkennung / Fail-Closed
                return Err(ErasureError::Corrupt);
            }

            let entry_slice = body
                .get(offset..offset + total_len)
                .ok_or(ErasureError::Corrupt)?;
            let len_and_ct_and_tag = entry_slice
                .get(..4 + len + 16)
                .ok_or(ErasureError::Corrupt)?;
            let chain = entry_slice
                .get(4 + len + 16..total_len)
                .ok_or(ErasureError::Corrupt)?;

            // Hash-Kette prüfen: chain = SHA-256(chain_prev || len || ct || aead_tag)
            let mut hasher = Sha256::new();
            hasher.update(chain_prev);
            hasher.update(len_and_ct_and_tag);
            let expected_chain = hasher.finalize();

            if chain != expected_chain.as_slice() {
                return Err(ErasureError::Corrupt);
            }

            let mut chain_arr = [0u8; 32];
            chain_arr.copy_from_slice(chain);
            chain_prev = chain_arr;

            // Envelope entschlüsseln (ct + aead_tag ist das Envelope)
            let envelope_slice = entry_slice
                .get(4..4 + len + 16)
                .ok_or(ErasureError::Corrupt)?;
            let plain = open_record(&dek, key_binding, envelope_slice)
                .map_err(|_| ErasureError::Corrupt)?;

            let rec = JournalRecord::decode(&plain)?;
            records.push(rec);

            offset += total_len;
        }

        Ok(Self {
            vfs,
            path: path.to_path_buf(),
            derived_key: *derived_key,
            records,
            chain_tail: chain_prev,
        })
    }

    /// Fügt einen neuen Record an und synchronisiert (`fsync`).
    pub fn append_record(&mut self, rec: &JournalRecord) -> Result<(), ErasureError> {
        let plain = rec.encode();
        let dek = Dek(SecretKey::new(self.derived_key));
        let key_binding = b"ctx/journal/v1";

        let envelope = seal_record(&dek, rec.erasure_id, key_binding, &plain);
        if envelope.len() < 16 {
            return Err(ErasureError::Crypto("Seal failed".to_string()));
        }

        let (ct, tag) = envelope.split_at(envelope.len() - 16);
        let ct_len = u32::try_from(ct.len())
            .map_err(|_| ErasureError::LimitExceeded("Ciphertext too large".to_string()))?;

        // F.9 Block: len(u32) || ct || tag(16) || chain(32)
        let mut block = Vec::with_capacity(4 + ct.len() + 16 + 32);
        block.extend_from_slice(&ct_len.to_be_bytes());
        block.extend_from_slice(ct);
        block.extend_from_slice(tag);

        let mut hasher = Sha256::new();
        hasher.update(self.chain_tail);
        hasher.update(&block);
        let next_chain = hasher.finalize();

        block.extend_from_slice(&next_chain);

        // Vfs-Schreibzugriff und fsync
        let mut cur_bytes = self.vfs.read(&self.path).unwrap_or_default();
        cur_bytes.extend_from_slice(&block);
        self.vfs
            .write_atomic(&self.path, &cur_bytes)
            .map_err(vfs_err)?;

        self.records.push(rec.clone());
        let mut next_arr = [0u8; 32];
        next_arr.copy_from_slice(&next_chain);
        self.chain_tail = next_arr;

        Ok(())
    }

    /// Liefert alle Records.
    pub fn records(&self) -> &[JournalRecord] {
        &self.records
    }

    /// Führt die Reduktion nach Completed (R13) via atomarem Dateiaustausch durch.
    pub fn reduce_completed(&mut self) -> Result<(), ErasureError> {
        let mut completed_ids = std::collections::BTreeSet::new();
        for r in &self.records {
            if r.record_code == RecordCode::Completed {
                completed_ids.insert(r.erasure_id);
            }
        }

        let mut reduced = Vec::new();
        for r in &self.records {
            if completed_ids.contains(&r.erasure_id) {
                if r.record_code == RecordCode::Completed {
                    reduced.push(r.clone());
                }
            } else {
                reduced.push(r.clone());
            }
        }

        // Erstelle neues Journal über atomaren Austausch
        let mut header = [0u8; 64];
        if let Some(dest) = header.get_mut(0..8) {
            dest.copy_from_slice(FILE_HEADER_MAGIC);
        }
        if let Some(dest) = header.get_mut(8..10) {
            dest.copy_from_slice(&1u16.to_be_bytes());
        }
        if let Some(dest) = header.get_mut(10) {
            *dest = FILE_KIND_JOURNAL;
        }
        if let Some(dest) = header.get_mut(11..15) {
            dest.copy_from_slice(&[1, 0, 1, 1]);
        }
        if let Some(dest) = header.get_mut(39) {
            *dest = 1;
        }
        let header48 = header.get(0..48).ok_or(ErasureError::Corrupt)?;
        let hmac = blake3::keyed_hash(&self.derived_key, header48);
        if let (Some(dest), Some(src)) = (header.get_mut(48..64), hmac.as_bytes().get(0..16)) {
            dest.copy_from_slice(src);
        }

        let mut new_bytes = Vec::new();
        new_bytes.extend_from_slice(&header);

        let mut new_chain = JOURNAL_GENESIS_CHAIN;
        let dek = Dek(SecretKey::new(self.derived_key));
        let key_binding = b"ctx/journal/v1";

        for r in &reduced {
            let plain = r.encode();
            let envelope = seal_record(&dek, r.erasure_id, key_binding, &plain);
            let (ct, tag) = envelope.split_at(envelope.len() - 16);
            let ct_len = u32::try_from(ct.len()).map_err(|_| ErasureError::Corrupt)?;

            let mut block = Vec::with_capacity(4 + ct.len() + 16 + 32);
            block.extend_from_slice(&ct_len.to_be_bytes());
            block.extend_from_slice(ct);
            block.extend_from_slice(tag);

            let mut hasher = Sha256::new();
            hasher.update(new_chain);
            hasher.update(&block);
            let next_chain = hasher.finalize();

            block.extend_from_slice(&next_chain);
            new_bytes.extend_from_slice(&block);

            let mut next_arr = [0u8; 32];
            next_arr.copy_from_slice(&next_chain);
            new_chain = next_arr;
        }

        // write_atomic garantiert tmp + fsync + rename + dir-fsync
        self.vfs
            .write_atomic(&self.path, &new_bytes)
            .map_err(vfs_err)?;

        self.records = reduced;
        self.chain_tail = new_chain;

        Ok(())
    }

    /// Rekonstruiert den Zustand aller noch nicht abgeschlossenen Sagas aus dem Journal (SPEC §11.2, §11.6).
    pub fn recover_open_sagas(&self) -> Vec<EraseSaga> {
        let mut latest_state: BTreeMap<[u8; 16], (SagaState, u64)> = BTreeMap::new();
        for r in &self.records {
            let state = r.record_code.to_saga_state();
            latest_state.insert(r.erasure_id, (state, r.timestamp));
        }

        let mut recovered = Vec::new();
        for (id_bytes, (state, _ts)) in latest_state {
            if !state.is_terminal() {
                recovered.push(EraseSaga::with_state(ErasureId::new(id_bytes), state));
            }
        }
        recovered
    }
}

/// In-Memory-Journal-Implementierung (SPEC §11.6, ER-29).
pub struct AppendLog {
    entries: Vec<JournalEntry>,
}

impl AppendLog {
    /// Erstellt ein leeres Log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Fügt einen Eintrag hinzu.
    pub fn append(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    /// Liefert alle Einträge.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Anzahl der Einträge.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Prüft ob das Log leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Rekonstruiert den Zustand aller noch nicht abgeschlossenen Sagas aus dem Journal (SPEC §11.2, §11.6).
    pub fn recover_open_sagas(&self) -> Vec<EraseSaga> {
        let mut latest_state: BTreeMap<[u8; 16], (SagaState, u64)> = BTreeMap::new();
        for e in &self.entries {
            latest_state.insert(e.erasure_id, (e.state, e.timestamp));
        }

        let mut recovered = Vec::new();
        for (id_bytes, (state, _ts)) in latest_state {
            if !state.is_terminal() {
                recovered.push(EraseSaga::with_state(ErasureId::new(id_bytes), state));
            }
        }
        recovered
    }
}

impl Default for AppendLog {
    fn default() -> Self {
        Self::new()
    }
}
