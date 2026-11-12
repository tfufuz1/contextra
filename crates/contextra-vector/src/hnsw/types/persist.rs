// FILE-CONTEXT
// ZWECK: Persistierungs- und Mmap-Ladelogik fuer GenericHnswIndex ueber write_atomic_with.
// INVARIANTEN: Persistierung schreibt Header v2 atomar und schuetzt vor unvollstaendigen Zustaenden.
// HOTSPOTS: GenericHnswIndex::save_internal, GenericHnswIndex::load_mmap_from_instance
// STAND: TS:2026-11-12T00:00:00Z

use std::sync::atomic::{AtomicU64, Ordering};

use super::data::VectorData;
use super::index::GenericHnswIndex;
use crate::distance::DistanceKernel;
use crate::error::{Result, VectorError};

impl<K: DistanceKernel> GenericHnswIndex<K> {
    #[cfg(any(feature = "mmap", feature = "workspace"))]
    pub fn load_mmap_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let mmap_index = crate::persistence::MmapIndex::open(path)?;
        self.load_mmap_from_instance(mmap_index)
    }

    #[cfg(any(feature = "mmap", feature = "workspace"))]
    pub fn load_mmap(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        self.load_mmap_sync(path)
    }

    pub(crate) fn load_mmap_from_instance(
        &self,
        mmap_index: crate::persistence::MmapIndex,
    ) -> Result<()> {
        let ep = if mmap_index.header.entry_point() >= 0 {
            Some(mmap_index.header.entry_point() as usize)
        } else {
            None
        };

        let max_layer = if let Some(e) = ep {
            let record = mmap_index.get_node_record(e)?;
            record.max_layer as u64
        } else {
            0
        };

        {
            self.inner.hot.set_entry_point(ep);
            self.inner.hot.set_ram_entry_point(None);
            self.inner.hot.max_layer.store(max_layer, Ordering::SeqCst);
        }

        *self.inner.cold.sq8_bias.write() = mmap_index.header.sq8_bias();

        if mmap_index.header.is_quantized() {
            let dim = self.inner.cold.config.dimension;
            let header = &mmap_index.header;

            let mut mins = Vec::new();
            let mut maxes = Vec::new();

            if header.version() == 2
                && header.quant_calibration_offset() > 0
                && header.quant_calibration_len() as usize >= dim * 4 * 2
            {
                let offset = header.quant_calibration_offset() as usize;
                let cal_bytes = mmap_index
                    .mmap
                    .get(offset..offset + dim * 4 * 2)
                    .ok_or_else(|| {
                        VectorError::Storage("Calibration segment out of bounds".into())
                    })?;

                for i in 0..dim {
                    let min_bytes = &cal_bytes[i * 4..(i + 1) * 4];
                    mins.push(f32::from_le_bytes(min_bytes.try_into().map_err(|_| {
                        VectorError::Storage("Invalid calibration min float".into())
                    })?));
                }
                for i in 0..dim {
                    let max_bytes = &cal_bytes[(dim + i) * 4..(dim + i + 1) * 4];
                    maxes.push(f32::from_le_bytes(max_bytes.try_into().map_err(|_| {
                        VectorError::Storage("Invalid calibration max float".into())
                    })?));
                }
            } else {
                let q_min = header.q_min();
                let q_max = header.q_max();
                mins = vec![q_min; dim];
                maxes = vec![q_max; dim];
            }

            let mut scales = Vec::with_capacity(dim);
            let mut inv_scales = Vec::with_capacity(dim);

            for i in 0..dim {
                let mut range = maxes[i] - mins[i];
                if range.abs() < f32::EPSILON {
                    range = 1e-6;
                }
                scales.push(255.0 / range);
                inv_scales.push(range / 255.0);
            }

            let mut q_guard = self.inner.cold.quantizer.write();
            *q_guard = Some(crate::quantize::ScalarQuantizer {
                mins,
                maxes,
                scales,
                inv_scales,
                dimension: dim,
                total_queries: AtomicU64::new(0),
                out_of_range_queries: AtomicU64::new(0),
            });
        }

        let mut guard = self.inner.cold.mmap_index.write();
        self.inner
            .hot
            .last_tx_id
            .store(mmap_index.header.last_tx_id(), Ordering::SeqCst);
        *guard = Some(mmap_index);
        Ok(())
    }

    // ANCHOR[contextra-vector:HNSW-SAVE-INTERNAL]
    pub(crate) fn save_internal(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let inner = &self.inner;
        let dest = path.as_ref();

        use std::io::{Seek, Write};

        let nodes = inner.hot.nodes.read();
        let entry_point = inner.hot.get_entry_point();
        let q_guard = inner.cold.quantizer.read();

        let node_count = nodes.len();
        let nodes_offset = crate::persistence::HnswHeader::SIZE as u64;
        let vectors_offset =
            nodes_offset + (node_count * crate::persistence::NodeRecord::SIZE) as u64;

        let (q_min, q_max) = if let Some(q) = q_guard.as_ref() {
            (
                q.mins.first().copied().unwrap_or(0.0),
                q.maxes.first().copied().unwrap_or(0.0),
            )
        } else {
            (0.0, 0.0)
        };

        let bias = *inner.cold.sq8_bias.read();

        crate::persistence::atomic::write_atomic_with(dest, |file| {
            let mut writer = std::io::BufWriter::new(file);

            let mut header = crate::persistence::HnswHeader::new_v2_with_bias(
                inner.cold.config.dimension as u32,
                inner.cold.config.m as u32,
                inner.cold.config.distance_metric as u8,
                if inner.cold.config.quantize { 1 } else { 0 },
                q_min,
                q_max,
                node_count as u64,
                entry_point.map(|i| i as i64).unwrap_or(-1),
                nodes_offset,
                0,
                inner.hot.last_tx_id.load(Ordering::SeqCst),
                0,
                0,
                bias.mean_bias,
                bias.variance_bias,
            );

            writer.write_all(&header.to_bytes())?;

            let mut node_records = Vec::with_capacity(node_count);
            for _ in 0..node_count {
                node_records.push(crate::persistence::NodeRecord {
                    doc_id: 0,
                    max_layer: 0,
                    vector_offset: 0,
                    connections_offset: 0,
                });
            }
            for record in &node_records {
                writer.write_all(&record.to_bytes())?;
            }

            let mut current_pos = vectors_offset;
            for (i, node) in nodes.iter().enumerate() {
                node_records[i].doc_id = node.doc_id.inner();
                node_records[i].max_layer = node.max_layer as u8;
                node_records[i].vector_offset = current_pos;

                match &node.vector {
                    VectorData::F32(v) => {
                        for &val in v.iter() {
                            writer.write_all(&val.to_le_bytes())?;
                        }
                        current_pos += (v.len() * 4) as u64;
                    }
                    VectorData::U8(v) => {
                        writer.write_all(v.as_slice())?;
                        current_pos += v.len() as u64;
                    }
                }
            }

            let connections_offset = (current_pos + 3) & !3;
            header.set_connections_offset(connections_offset);

            if connections_offset > current_pos {
                let padding = [0u8; 4];
                writer.write_all(&padding[..(connections_offset - current_pos) as usize])?;
            }

            let mut conn_pos = connections_offset;
            for (i, node) in nodes.iter().enumerate() {
                node_records[i].connections_offset = conn_pos;
                let num_layers = (node.max_layer + 1) as u8;
                writer.write_all(&[num_layers])?;
                conn_pos += 1;

                for layer in 0..num_layers as usize {
                    let layer_conns =
                        inner
                            .hot
                            .get_ram_node_connections(i, layer, inner.cold.config.m);
                    let len = layer_conns.len() as u32;
                    writer.write_all(&len.to_le_bytes())?;
                    for &conn in layer_conns.iter() {
                        writer.write_all(&conn.to_le_bytes())?;
                    }
                    conn_pos += 4 + (len as u64) * 4;
                }
            }

            let cal_offset = conn_pos;
            let mut cal_len = 0u32;

            if let Some(ref q) = *q_guard {
                let dim = q.mins.len();
                cal_len = (dim * 4 * 2) as u32;
                for &m in &q.mins {
                    writer.write_all(&m.to_le_bytes())?;
                }
                for &m in &q.maxes {
                    writer.write_all(&m.to_le_bytes())?;
                }
            }

            header = crate::persistence::HnswHeader::new_v2_with_bias(
                inner.cold.config.dimension as u32,
                inner.cold.config.m as u32,
                inner.cold.config.distance_metric as u8,
                if inner.cold.config.quantize { 1 } else { 0 },
                q_min,
                q_max,
                node_count as u64,
                entry_point.map(|i| i as i64).unwrap_or(-1),
                nodes_offset,
                connections_offset,
                inner.hot.last_tx_id.load(Ordering::SeqCst),
                cal_offset,
                cal_len,
                bias.mean_bias,
                bias.variance_bias,
            );

            writer.flush()?;
            let raw_file = writer
                .into_inner()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            raw_file.seek(std::io::SeekFrom::Start(0))?;
            raw_file.write_all(&header.to_bytes())?;
            raw_file.seek(std::io::SeekFrom::Start(nodes_offset))?;
            for record in &node_records {
                raw_file.write_all(&record.to_bytes())?;
            }

            Ok(())
        })
        .map_err(|e| VectorError::Storage(e.to_string()))
    }

    pub fn save_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let _lock = self.inner.hot.write_mutex.lock();
        self.save_internal(path)
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        self.save_sync(path)
    }
}
