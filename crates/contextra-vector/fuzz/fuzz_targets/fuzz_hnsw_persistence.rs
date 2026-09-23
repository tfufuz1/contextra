#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};

#[derive(Arbitrary, Debug)]
struct HnswPersistInput {
    n_vectors: u8,           // 1..=30 (klein halten für Geschwindigkeit)
    dimension: u8,           // 4..=64 (mod 4 für SIMD-Alignment)
    raw_floats: Vec<u8>,     // Rohdaten → reinterpret als f32-Vektoren
    corrupt_offset_pct: u8,  // Offset für Datei-Korruption nach persist()
    corrupt_bytes: [u8; 8],  // Zu injizierende Bytes
    do_corrupt: bool,        // Ob überhaupt korrupt gemacht werden soll
}

fuzz_target!(|input: HnswPersistInput| {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return,
    };

    rt.block_on(async {
        // 1. dim = (input.dimension as usize % 61) + 4 -> [4..64], aligned to mod 4 == 0
        let raw_dim = (input.dimension as usize % 61) + 4;
        let dim = ((raw_dim / 4) * 4).max(4);

        // 2. n = (input.n_vectors as usize % 30) + 1 -> [1..30]
        let n = (input.n_vectors as usize % 30) + 1;

        // 3. Extract vectors from raw_floats: 4 bytes -> 1 f32, sanitize NaN/Inf -> 0.0
        let float_chunks: Vec<f32> = input
            .raw_floats
            .chunks_exact(4)
            .map(|chunk| {
                let f = f32::from_le_bytes(chunk.try_into().unwrap());
                if f.is_nan() || f.is_infinite() {
                    0.0
                } else {
                    f
                }
            })
            .collect();

        if float_chunks.len() < dim {
            return;
        }

        let mut vectors = Vec::with_capacity(n);
        for chunk in float_chunks.chunks(dim) {
            if chunk.len() == dim {
                vectors.push(chunk.to_vec());
                if vectors.len() == n {
                    break;
                }
            }
        }

        if vectors.is_empty() {
            return;
        }

        let config = HnswConfig {
            dimension: dim,
            max_elements: 100,
            m: 16,
            ef_construction: 64,
            ef_search: 32,
            distance_metric: DistanceMetric::Cosine,
            rebuild_threshold: 0.9,
            quantize: false,
            quantizer_recalibration_sample_size: 100,
            quantizer_drift_threshold: 0.10,
        };

        let index = match HnswIndex::try_new(config.clone()) {
            Ok(idx) => idx,
            Err(_) => return,
        };

        let tx = TxId::new(1);
        for (i, vec) in vectors.iter().enumerate() {
            let _ = index.insert(tx, DocId::new((i + 1) as u64), vec).await;
        }
        let _ = index.commit(tx).await;

        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("fuzz_hnsw_{}_{}.hnsw", std::process::id(), id));

        // 5. Save index
        if index.save(&path).await.is_err() {
            let _ = std::fs::remove_file(&path);
            return;
        }

        // 6. Corrupt file if requested
        if input.do_corrupt {
            if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(&path) {
                if let Ok(len) = file.metadata().map(|m| m.len()) {
                    if len > 0 {
                        let offset = (len * (input.corrupt_offset_pct as u64 % 100)) / 100;
                        let _ = file.seek(SeekFrom::Start(offset)); // INTENTIONAL-DROP
                        let _res_write = file.write_all(&input.corrupt_bytes);
                        let _res_sync = file.sync_all();
                    }
                }
            }
        }

        // 7. Load index - must not panic
        let reloaded = match HnswIndex::try_new(config) {
            Ok(idx) => idx,
            Err(_) => {
                let _ = std::fs::remove_file(&path);
                return;
            }
        };

        // 8. Search on reloaded index if load succeeded - must not panic
        if reloaded.load_mmap(&path).await.is_ok() {
            let query = vec![0.0f32; dim];
            let _ = reloaded.search(&query, 5).await;
        }

        let _ = std::fs::remove_file(&path);
    });
});
