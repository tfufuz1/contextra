//! # RunReader: Lesepfad für unveränderliche Runs
//!
//! Gemäß SPEC §9.5, §11:
//! Öffnen: Footer -> Directory -> Meta -> Index.
//! Filter und Datenblöcke werden LAZY geladen.
//! Lesen mit `read_at` über `File`. Kein `mmap`.
//! `get`, Bereichsiteration, Präfixfilter-Prüfung und `read_batch`.
//! Vor jedem Zugriff werden Offset und Länge gegen Dateilänge und Überlappung geprüft.
//! Keine Integer-Casts (M-02 = 0).

use super::block::{block_kind, decode_block, BlockSecurity};
use super::block_cache::BlockCache;
use super::data::{DataBlockReader, DataEntry};
use super::directory::DirectoryBlock;
use super::error::RunError;
use super::filter::{FilterBlockReader, PrefixFilterBlockReader};
use super::footer::Footer;
use super::index::IndexBlock;
use super::meta::MetaBlock;
use super::rangedel::RangeDelBlock;
use contextra_storage_ports::{File, ReadReq};
use std::sync::{Arc, Mutex};

/// Leser für einen unveränderlichen Run.
pub struct RunReader<'a> {
    file: Arc<dyn File>,
    file_len: u64,
    security: BlockSecurity<'a>,
    block_cache: Option<Arc<BlockCache>>,
    directory: DirectoryBlock,
    meta: MetaBlock,
    index: IndexBlock,
    rangedel: Option<RangeDelBlock>,

    // Lazy Filter
    filter: Mutex<Option<FilterBlockReader>>,
    prefix_filter: Mutex<Option<PrefixFilterBlockReader>>,
}

impl<'a> RunReader<'a> {
    /// Öffnet einen Run und lädt Footer, Directory, Meta und Index.
    pub fn open(
        file: Arc<dyn File>,
        file_len: u64,
        security: BlockSecurity<'a>,
        block_cache: Option<Arc<BlockCache>>,
    ) -> Result<Self, RunError> {
        if file_len < u64::try_from(Footer::LEN).unwrap_or(28) {
            return Err(RunError::Corrupt("Datei zu kurz für Run-Footer".into()));
        }

        // 1. Footer lesen (letzte 28 B)
        let footer_offset = file_len.saturating_sub(u64::try_from(Footer::LEN).unwrap_or(28));
        let mut footer_buf = [0u8; Footer::LEN];
        read_exact_at(file.as_ref(), &mut footer_buf, footer_offset)?;
        let footer = Footer::decode(&footer_buf)?;

        // 2. Directory lesen
        let dir_bytes = read_block_raw(file.as_ref(), footer.directory_offset, footer.directory_len, file_len)?;
        let dir_payload = decode_block(
            &dir_bytes,
            block_kind::DIRECTORY,
            0, // run_id wird aus Meta geladen
            footer.directory_offset,
            0,
            &security,
        )?;
        let directory = DirectoryBlock::decode(&dir_payload, file_len)?;

        // 3. MetaBlock lesen (Pflicht)
        let meta_entry = directory
            .find(block_kind::META)
            .ok_or_else(|| RunError::Corrupt("MetaBlock fehlt im Directory".into()))?;
        let meta_raw = read_block_raw(file.as_ref(), meta_entry.offset, meta_entry.len, file_len)?;
        let meta_payload = decode_block(
            &meta_raw,
            block_kind::META,
            0,
            meta_entry.offset,
            0,
            &security,
        )?;
        let meta = MetaBlock::decode(&meta_payload)?;

        // 4. IndexBlock lesen (Pflicht)
        let index_entry = directory
            .find(block_kind::INDEX)
            .ok_or_else(|| RunError::Corrupt("IndexBlock fehlt im Directory".into()))?;
        let index_raw = read_block_raw(file.as_ref(), index_entry.offset, index_entry.len, file_len)?;
        let index_payload = decode_block(
            &index_raw,
            block_kind::INDEX,
            meta.run_id,
            index_entry.offset,
            meta.space_id,
            &security,
        )?;
        let k = usize::from(meta.weights_k);
        let index = IndexBlock::decode(&index_payload, k)?;

        // 5. RangeDelBlock beim Öffnen vollständig laden (falls vorhanden)
        let rangedel = if let Some(rd_entry) = directory.find(block_kind::RANGE_DEL) {
            let rd_raw = read_block_raw(file.as_ref(), rd_entry.offset, rd_entry.len, file_len)?;
            let rd_payload = decode_block(
                &rd_raw,
                block_kind::RANGE_DEL,
                meta.run_id,
                rd_entry.offset,
                meta.space_id,
                &security,
            )?;
            Some(RangeDelBlock::decode(&rd_payload)?)
        } else {
            None
        };

        Ok(Self {
            file,
            file_len,
            security,
            block_cache,
            directory,
            meta,
            index,
            rangedel,
            filter: Mutex::new(None),
            prefix_filter: Mutex::new(None),
        })
    }

    /// Liefert die Run-Metadaten.
    pub const fn meta(&self) -> &MetaBlock {
        &self.meta
    }

    /// Liefert den Index-Block.
    pub const fn index(&self) -> &IndexBlock {
        &self.index
    }

    /// Punktabfrage nach Nutzerschlüssel.
    pub fn get(&self, user_key: &[u8]) -> Result<Option<DataEntry>, RunError> {
        // Schlüsselgrenzen prüfen
        if !self.meta.min_key.is_empty() && user_key < self.meta.min_key.as_slice() {
            return Ok(None);
        }
        if !self.meta.max_key.is_empty() && user_key > self.meta.max_key.as_slice() {
            return Ok(None);
        }

        // Bloom-Filter prüfen (lazy geladen)
        if !self.check_filter(user_key)? {
            return Ok(None);
        }

        // Block über Index lokalisieren
        let block_idx = match self.index.search(user_key) {
            Some(idx) => idx,
            None => return Ok(None),
        };

        let data_payload = self.load_data_block(block_idx)?;
        let reader = DataBlockReader::new(&data_payload)?;
        let entry_opt = reader.get(user_key)?;

        if let Some(entry) = entry_opt {
            // Bereichslöschungen prüfen
            if let Some(rd) = &self.rangedel {
                if rd.is_deleted(user_key, entry.tag.stamp()) {
                    return Ok(None);
                }
            }
            return Ok(Some(entry));
        }

        Ok(None)
    }

    /// Prüft den Präfix-Bloom-Filter (lazy geladen).
    pub fn prefix_may_contain(&self, prefix: &[u8]) -> Result<bool, RunError> {
        let mut guard = self.prefix_filter.lock().map_err(|_| {
            RunError::Corrupt("Prefix-Filter-Sperre vergiftet".into())
        })?;

        if guard.is_none() {
            if let Some(entry) = self.directory.find(block_kind::PREFIX_FILTER) {
                let raw = read_block_raw(self.file.as_ref(), entry.offset, entry.len, self.file_len)?;
                let payload = decode_block(
                    &raw,
                    block_kind::PREFIX_FILTER,
                    self.meta.run_id,
                    entry.offset,
                    self.meta.space_id,
                    &self.security,
                )?;
                let p_reader = PrefixFilterBlockReader::decode(&payload, prefix.len())?;
                *guard = Some(p_reader);
            }
        }

        if let Some(pf) = guard.as_ref() {
            Ok(pf.may_contain_prefix(prefix))
        } else {
            Ok(true)
        }
    }

    /// Lädt mehrere Datenblöcke gebündelt über `read_batch` (SPEC §11).
    pub fn read_batch(&self, block_indices: &[usize]) -> Result<Vec<Vec<u8>>, RunError> {
        let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(block_indices.len());
        let mut reqs: Vec<ReadReq<'_>> = Vec::with_capacity(block_indices.len());

        for &idx in block_indices {
            let entry = self
                .index
                .entries
                .get(idx)
                .ok_or_else(|| RunError::Corrupt("Block-Index außerhalb der Grenzen".into()))?;

            let len = usize::try_from(entry.len)
                .map_err(|_| RunError::Limit("Blocklänge übersteigt Adressraum".into()))?;

            buffers.push(vec![0u8; len]);
        }

        for (i, buf) in buffers.iter_mut().enumerate() {
            let idx = *block_indices.get(i).ok_or(RunError::UnexpectedEof)?;
            let entry = self.index.entries.get(idx).ok_or(RunError::UnexpectedEof)?;
            reqs.push(ReadReq {
                offset: entry.offset,
                buf: buf.as_mut_slice(),
            });
        }

        self.file.read_batch(&mut reqs)?;

        let mut results = Vec::with_capacity(buffers.len());
        for (i, raw) in buffers.into_iter().enumerate() {
            let idx = *block_indices.get(i).ok_or(RunError::UnexpectedEof)?;
            let entry = self.index.entries.get(idx).ok_or(RunError::UnexpectedEof)?;
            let payload = decode_block(
                &raw,
                block_kind::DATA,
                self.meta.run_id,
                entry.offset,
                self.meta.space_id,
                &self.security,
            )?;
            results.push(payload);
        }

        Ok(results)
    }

    fn check_filter(&self, user_key: &[u8]) -> Result<bool, RunError> {
        let mut guard = self.filter.lock().map_err(|_| {
            RunError::Corrupt("Filter-Sperre vergiftet".into())
        })?;

        if guard.is_none() {
            if let Some(entry) = self.directory.find(block_kind::FILTER) {
                let raw = read_block_raw(self.file.as_ref(), entry.offset, entry.len, self.file_len)?;
                let payload = decode_block(
                    &raw,
                    block_kind::FILTER,
                    self.meta.run_id,
                    entry.offset,
                    self.meta.space_id,
                    &self.security,
                )?;
                let reader = FilterBlockReader::decode(&payload)?;
                *guard = Some(reader);
            }
        }

        if let Some(f) = guard.as_ref() {
            Ok(f.may_contain(user_key))
        } else {
            Ok(true)
        }
    }

    /// Lädt einen Datenblock anhand seines Index.
    pub fn load_data_block(&self, block_idx: usize) -> Result<Arc<Vec<u8>>, RunError> {
        let block_no = u32::try_from(block_idx).unwrap_or(0);
        if let Some(cache) = &self.block_cache {
            if let Some(data) = cache.get(self.meta.run_id, block_no) {
                return Ok(data);
            }
        }

        let entry = self
            .index
            .entries
            .get(block_idx)
            .ok_or_else(|| RunError::Corrupt("Index-Eintrag fehlt".into()))?;

        let raw = read_block_raw(self.file.as_ref(), entry.offset, entry.len, self.file_len)?;
        let payload = decode_block(
            &raw,
            block_kind::DATA,
            self.meta.run_id,
            entry.offset,
            self.meta.space_id,
            &self.security,
        )?;
        let arc_data = Arc::new(payload);

        if let Some(cache) = &self.block_cache {
            cache.insert(self.meta.run_id, block_no, Arc::clone(&arc_data));
        }

        Ok(arc_data)
    }
}

fn read_block_raw(file: &dyn File, offset: u64, len: u32, file_len: u64) -> Result<Vec<u8>, RunError> {
    let len_u64 = u64::from(len);
    let end = offset
        .checked_add(len_u64)
        .ok_or_else(|| RunError::Limit("Offset-Überlauf beim Blocklesen".into()))?;

    if end > file_len {
        return Err(RunError::Corrupt(format!(
            "Block {offset}..{end} ragt über Dateiende {file_len} hinaus"
        )));
    }

    let len_usize = usize::try_from(len)
        .map_err(|_| RunError::Limit("Blocklänge übersteigt Adressraum".into()))?;
    let mut buf = vec![0u8; len_usize];
    read_exact_at(file, &mut buf, offset)?;
    Ok(buf)
}

fn read_exact_at(file: &dyn File, mut buf: &mut [u8], mut offset: u64) -> Result<(), RunError> {
    while !buf.is_empty() {
        let n = file.read_at(buf, offset)?;
        if n == 0 {
            return Err(RunError::UnexpectedEof);
        }
        offset = offset.saturating_add(u64::try_from(n).unwrap_or(0));
        buf = match buf.get_mut(n..) {
            Some(rem) => rem,
            None => break,
        };
    }
    Ok(())
}
