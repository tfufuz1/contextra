// FILE-CONTEXT
// ZWECK: Schreiben des DiskANN-Index-Dateiformats DANN v1 (WP-VEC-07).
// INVARIANTEN: Deterministische Header- & Knoten-Serialisierung; Footer-HMAC-Abschluss.
// STAND: 2026-11-11

use std::io::Write;
use std::sync::atomic::Ordering;

use super::super::format::{
    DiskAnnFooter, DiskAnnHeader, DISKANN_FOOTER_MAGIC, DISKANN_INTEGRITY_KEY, DISKANN_MAGIC,
    DISKANN_VERSION,
};
use super::super::integrity::DiskAnnHmac;
use super::super::types::DiskAnnIndex;
use crate::error::{Result, VectorError};
use crate::types::DocId;

impl DiskAnnIndex {
    pub(crate) fn write_to_path_sync(
        &self,
        path: &std::path::Path,
        graph: &[Vec<u32>],
        vectors: &[Vec<f32>],
        ids: &[DocId],
    ) -> Result<()> {
        use std::io::Write;
        use std::sync::atomic::Ordering;

        let n = vectors.len();
        let mut file = crate::diskann::io::create_truncate(path)?;

        let quantizer_opt = {
            let q_guard = self.inner.quantizer.read();
            q_guard.clone()
        };
        let (q_min, q_max, quantized) = if let Some(ref q) = quantizer_opt {
            (
                q.mins().first().copied().unwrap_or(0.0),
                q.maxes().first().copied().unwrap_or(0.0),
                1,
            )
        } else {
            (0.0, 0.0, 0)
        };

        let header = DiskAnnHeader {
            magic: *DISKANN_MAGIC,
            version: DISKANN_VERSION,
            node_count: n as u64,
            dimension: self.inner.config.dimension as u32,
            max_degree: self.inner.config.max_degree as u32,
            sector_size: self.inner.config.sector_size as u32,
            entry_point: 0,
            metric: self.inner.config.distance_metric as u8,
            quantized,
            q_min,
            q_max,
        };

        let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY)?;

        let header_bytes = header.to_bytes();
        hmac.update(&header_bytes);
        file.write_all(&header_bytes).map_err(VectorError::from)?;
        let padding = vec![
            0u8;
            self.inner.config.sector_size
                - (DiskAnnHeader::SIZE % self.inner.config.sector_size)
        ];
        hmac.update(&padding);
        file.write_all(&padding).map_err(VectorError::from)?;

        for i in 0..n {
            let mut node_buf = Vec::new();

            if let Some(ref q) = quantizer_opt {
                let qv = q.quantize(&vectors[i])?;
                node_buf.extend_from_slice(&qv);
            } else {
                for &val in &vectors[i] {
                    node_buf.extend_from_slice(&val.to_le_bytes());
                }
            }

            let neighbors = &graph[i];
            node_buf.extend_from_slice(&(neighbors.len() as u32).to_le_bytes());
            for &neighbor in neighbors {
                node_buf.extend_from_slice(&neighbor.to_le_bytes());
            }
            let padding_count = self.inner.config.max_degree - neighbors.len();
            node_buf.extend_from_slice(&vec![0u8; padding_count * 4]);
            node_buf.extend_from_slice(&ids[i].inner().to_le_bytes());

            let used = node_buf.len();
            let node_size = self.inner.node_size_bytes.load(Ordering::SeqCst) as usize;
            if used < node_size {
                node_buf.extend_from_slice(&vec![0u8; node_size - used]);
            }

            hmac.update(&node_buf);
            file.write_all(&node_buf).map_err(VectorError::from)?;
        }

        let computed_hmac = hmac.finalize();
        let footer = DiskAnnFooter {
            magic: *DISKANN_FOOTER_MAGIC,
            hmac: computed_hmac,
        };
        file.write_all(&footer.to_bytes())
            .map_err(VectorError::from)?;

        file.sync_all().map_err(VectorError::from)?;
        Ok(())
    }
}
