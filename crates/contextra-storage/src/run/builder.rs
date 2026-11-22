//! # RunBuilder: Schreiben eines unveränderlichen Runs
//!
//! Schreibt alle Blöcke in normativer Reihenfolge nach SPEC §9.5:
//! `DataBlock* ExtensionBlock* RangeDelBlock? FilterBlock PrefixFilterBlock? IndexBlock MetaBlock DirectoryBlock Footer`
//! Keine Integer-Casts (M-02 = 0).

use super::blob_ref::BlobRef;
use super::block::{block_kind, encode_block, BlockSecurity};
use super::data::{DataBlockBuilder, TARGET_BLOCK_BYTES};
use super::directory::DirectoryBlock;
use super::error::RunError;
use super::filter::{FilterBlockBuilder, PrefixFilterBlockBuilder};
use super::footer::Footer;
use super::index::{shortest_separator, IndexBlock, IndexEntry};
use super::meta::MetaBlock;
use super::rangedel::{RangeDelBlock, RangeMarker};
use crate::codec::tag::{Tag, TagKind};
use std::collections::BTreeMap;

/// Konfiguration für den Bau eines Runs.
#[derive(Debug, Clone)]
pub struct RunBuilderOptions {
    /// Space-Kennung.
    pub space_id: u16,
    /// Eindeutige Run-ID aus der Hochwassermarke.
    pub run_id: u64,
    /// Feste Headerlänge je Eintrag.
    pub header_len: u16,
    /// Anzahl der geführten Gewichte (1..=4).
    pub weights_k: u8,
    /// Optionale Präfix-Bloom-Filter-Länge.
    pub prefix_filter_len: Option<usize>,
    /// Erstellungszeit in Millisekunden seit Unix-Epoche.
    pub created_ms: u64,
}

/// Builder zur Konstruktion einer vollständigen Run-Datei.
pub struct RunBuilder<'a> {
    options: RunBuilderOptions,
    security: BlockSecurity<'a>,
    out: Vec<u8>,
    current_offset: u64,

    // Datenblöcke
    current_block: DataBlockBuilder,
    block_weights: Vec<u64>,
    block_max_stamp: u64,

    // Aggregierte Strukturen
    index: IndexBlock,
    filter_builder: FilterBlockBuilder,
    prefix_filter_builder: Option<PrefixFilterBlockBuilder>,
    rangedel: RangeDelBlock,
    extensions: Vec<(u16, Vec<u8>, bool)>,

    // Metadaten
    entries_count: u64,
    min_stamp: u64,
    max_stamp: u64,
    min_key: Option<Vec<u8>>,
    max_key: Option<Vec<u8>>,
    range_markers: u32,
    erase_markers: u32,
    blobs: BTreeMap<u64, u64>,
}

impl<'a> RunBuilder<'a> {
    /// Erstellt einen neuen RunBuilder.
    pub fn new(options: RunBuilderOptions, security: BlockSecurity<'a>) -> Self {
        let prefix_builder = options
            .prefix_filter_len
            .map(PrefixFilterBlockBuilder::new);

        let k = usize::from(options.weights_k);
        let initial_weights = vec![0u64; k];

        Self {
            options,
            security,
            out: Vec::new(),
            current_offset: 0,
            current_block: DataBlockBuilder::new(),
            block_weights: initial_weights,
            block_max_stamp: 0,
            index: IndexBlock::new(),
            filter_builder: FilterBlockBuilder::new(),
            prefix_filter_builder: prefix_builder,
            rangedel: RangeDelBlock::new(),
            extensions: Vec::new(),
            entries_count: 0,
            min_stamp: u64::MAX,
            max_stamp: 0,
            min_key: None,
            max_key: None,
            range_markers: 0,
            erase_markers: 0,
            blobs: BTreeMap::new(),
        }
    }

    /// Fügt einen Schlüssel-Wert-Eintrag hinzu.
    pub fn add(&mut self, key: &[u8], tag: Tag, value: &[u8]) -> Result<(), RunError> {
        let stamp = tag.stamp();
        if stamp < self.min_stamp {
            self.min_stamp = stamp;
        }
        if stamp > self.max_stamp {
            self.max_stamp = stamp;
        }
        if stamp > self.block_max_stamp {
            self.block_max_stamp = stamp;
        }

        update_key_bounds(&mut self.min_key, &mut self.max_key, key);

        // Filter aktualisieren
        self.filter_builder.add_key(key);
        if let Some(pfb) = &mut self.prefix_filter_builder {
            pfb.add_key(key);
        }

        // Gewicht 0 = logische Bytes (Schlüssel + Header + Körper)
        let mut entry_logical_bytes = u64::try_from(key.len().saturating_add(value.len())).unwrap_or(0);

        // PutRef analysieren: BlobRef ab header_len
        if let Ok(TagKind::PutRef) = tag.kind() {
            let hlen = usize::from(self.options.header_len);
            if let Some(ref_slice) = value.get(hlen..) {
                if let Ok(bref) = BlobRef::decode(ref_slice) {
                    if !bref.is_shredded() {
                        let total_b = self.blobs.entry(bref.blob_no).or_insert(0);
                        *total_b = total_b.saturating_add(u64::from(bref.len));
                    }
                    entry_logical_bytes = u64::try_from(key.len().saturating_add(hlen))
                        .unwrap_or(0)
                        .saturating_add(u64::from(bref.len));
                }
            }
        }

        if let Some(w0) = self.block_weights.get_mut(0) {
            *w0 = w0.saturating_add(entry_logical_bytes);
        }

        // Prüfung, ob aktueller Block geflusht werden muss
        let approx_entry_len = key.len().saturating_add(value.len()).saturating_add(16);
        if !self.current_block.is_empty()
            && self.current_block.estimated_size().saturating_add(approx_entry_len)
                > TARGET_BLOCK_BYTES
        {
            self.flush_current_data_block(Some(key))?;
        }

        self.current_block.add(key, tag, value);
        self.entries_count = self.entries_count.saturating_add(1);

        // Großer Eintrag erhält eigenen Block (SPEC §9.5)
        if approx_entry_len > TARGET_BLOCK_BYTES {
            self.flush_current_data_block(None)?;
        }

        Ok(())
    }

    /// Fügt einen Bereichslöschungs-Marker hinzu.
    pub fn add_range_delete(&mut self, start: Vec<u8>, end: Option<Vec<u8>>, tag: Tag) {
        let stamp = tag.stamp();
        if stamp < self.min_stamp {
            self.min_stamp = stamp;
        }
        if stamp > self.max_stamp {
            self.max_stamp = stamp;
        }

        update_key_bounds(&mut self.min_key, &mut self.max_key, &start);
        if let Some(e) = &end {
            update_key_bounds(&mut self.min_key, &mut self.max_key, e);
        }

        match tag.kind() {
            Ok(TagKind::EraseRange) => self.erase_markers = self.erase_markers.saturating_add(1),
            _ => self.range_markers = self.range_markers.saturating_add(1),
        }

        self.rangedel.add(RangeMarker { start, end, tag });
    }

    /// Fügt einen Erweiterungsblock hinzu.
    pub fn add_extension(&mut self, kind: u16, data: Vec<u8>, required: bool) {
        self.extensions.push((kind, data, required));
    }

    fn flush_current_data_block(&mut self, next_key: Option<&[u8]>) -> Result<(), RunError> {
        if self.current_block.is_empty() {
            return Ok(());
        }

        let last_key = self.current_block.last_key().unwrap_or(&[]).to_vec();
        let entries_in_block = self.current_block.entries_count();
        let separator = shortest_separator(&last_key, next_key);

        let mut old_block = DataBlockBuilder::new();
        std::mem::swap(&mut self.current_block, &mut old_block);
        let payload = old_block.finish();

        let encoded = encode_block(
            &payload,
            block_kind::DATA,
            self.options.run_id,
            self.current_offset,
            self.options.space_id,
            &self.security,
        );

        let block_len = u32::try_from(encoded.len())
            .map_err(|_| RunError::Limit("Datenblock überschreitet u32".into()))?;

        self.index.add(IndexEntry::new(
            separator,
            self.current_offset,
            block_len,
            entries_in_block,
            self.block_weights.clone(),
            self.block_max_stamp,
        ));

        self.out.extend_from_slice(&encoded);
        self.current_offset = self.current_offset.saturating_add(u64::from(block_len));

        // Block-Gewichte und max_stamp zurücksetzen
        let k = usize::from(self.options.weights_k);
        self.block_weights = vec![0u64; k];
        self.block_max_stamp = 0;

        Ok(())
    }

    fn write_block_to_dir(
        &mut self,
        directory: &mut DirectoryBlock,
        kind: u16,
        payload: &[u8],
        required: u8,
    ) -> Result<(), RunError> {
        let encoded = encode_block(
            payload,
            kind,
            self.options.run_id,
            self.current_offset,
            self.options.space_id,
            &self.security,
        );
        let len = u32::try_from(encoded.len())
            .map_err(|_| RunError::Limit("Block überschreitet u32".into()))?;
        directory.add(kind, self.current_offset, len, required);
        self.out.extend_from_slice(&encoded);
        self.current_offset = self.current_offset.saturating_add(u64::from(len));
        Ok(())
    }

    /// Schließt den Run ab und liefert die vollständigen Bytes der Datei.
    pub fn finish(mut self) -> Result<Vec<u8>, RunError> {
        self.flush_current_data_block(None)?;
        let mut directory = DirectoryBlock::new();

        // 2. ExtensionBlocks (SPEC §9.5)
        let extensions = std::mem::take(&mut self.extensions);
        for (kind, ext_data, required) in extensions {
            let req_byte = if required { 1u8 } else { 0u8 };
            self.write_block_to_dir(&mut directory, kind, &ext_data, req_byte)?;
        }

        // 3. RangeDelBlock (optional)
        if !self.rangedel.markers.is_empty() {
            let mut rd = std::mem::take(&mut self.rangedel);
            rd.prepare();
            let payload = rd.encode();
            self.write_block_to_dir(&mut directory, block_kind::RANGE_DEL, &payload, 0)?;
        }

        // 4. FilterBlock (SPEC §9.5)
        let filter_builder = std::mem::take(&mut self.filter_builder);
        let filter_payload = filter_builder.finish();
        self.write_block_to_dir(&mut directory, block_kind::FILTER, &filter_payload, 0)?;

        // 5. PrefixFilterBlock (optional)
        if let Some(pfb) = self.prefix_filter_builder.take() {
            let p_payload = pfb.finish();
            self.write_block_to_dir(&mut directory, block_kind::PREFIX_FILTER, &p_payload, 0)?;
        }

        // 6. IndexBlock (SPEC §9.5, Pflicht = 1)
        let k = usize::from(self.options.weights_k);
        let index_payload = self.index.encode(k);
        self.write_block_to_dir(&mut directory, block_kind::INDEX, &index_payload, 1)?;

        // 7. MetaBlock (SPEC §9.5, Pflicht = 1)
        let meta = MetaBlock {
            space_id: self.options.space_id,
            run_id: self.options.run_id,
            entries: self.entries_count,
            min_stamp: if self.min_stamp == u64::MAX { 0 } else { self.min_stamp },
            max_stamp: self.max_stamp,
            range_markers: self.range_markers,
            erase_markers: self.erase_markers,
            created_ms: self.options.created_ms,
            header_len: self.options.header_len,
            weights_k: self.options.weights_k,
            weigher_version: 0,
            feature_bits: 0,
            min_key: self.min_key.unwrap_or_default(),
            max_key: self.max_key.unwrap_or_default(),
            blobs: self.blobs.into_iter().collect(),
        };
        let meta_payload = meta.encode();
        let encoded_meta = encode_block(
            &meta_payload,
            block_kind::META,
            self.options.run_id,
            self.current_offset,
            self.options.space_id,
            &self.security,
        );
        let meta_len = u32::try_from(encoded_meta.len())
            .map_err(|_| RunError::Limit("Meta-Block überschreitet u32".into()))?;
        directory.add(block_kind::META, self.current_offset, meta_len, 1);
        self.out.extend_from_slice(&encoded_meta);
        self.current_offset = self.current_offset.saturating_add(u64::from(meta_len));

        // 8. DirectoryBlock
        let dir_offset = self.current_offset;
        let dir_payload = directory.encode();
        let encoded_dir = encode_block(
            &dir_payload,
            block_kind::DIRECTORY,
            self.options.run_id,
            dir_offset,
            self.options.space_id,
            &self.security,
        );
        let dir_len = u32::try_from(encoded_dir.len())
            .map_err(|_| RunError::Limit("Directory-Block überschreitet u32".into()))?;
        self.out.extend_from_slice(&encoded_dir);
        self.current_offset = self.current_offset.saturating_add(u64::from(dir_len));

        // 9. Footer (28 B)
        let footer = Footer::new(dir_offset, dir_len);
        let footer_bytes = footer.encode();
        self.out.extend_from_slice(&footer_bytes);

        Ok(self.out)
    }
}

fn update_key_bounds(min_k: &mut Option<Vec<u8>>, max_k: &mut Option<Vec<u8>>, key: &[u8]) {
    match min_k {
        Some(m) if key < m.as_slice() => *m = key.to_vec(),
        None => *min_k = Some(key.to_vec()),
        _ => {}
    }
    match max_k {
        Some(m) if key > m.as_slice() => *m = key.to_vec(),
        None => *max_k = Some(key.to_vec()),
        _ => {}
    }
}
