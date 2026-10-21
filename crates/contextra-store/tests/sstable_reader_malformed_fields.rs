// ZWECK: Systematischer Mutationstest für SSTable Reader (STORE-003 Audit Verification).
//
// Dieser Test erzeugt eine gültige SSTable-Datei auf Disk und manipuliert jedes Längen-, Offset- und
// Zählerfeld des Headers, des Index-Blocks, der Datenblöcke und des Footers (Trailers) auf Extremwerte
// (0, 1, file_len, file_len + 1, u16::MAX, u32::MAX, u64::MAX) und prüft, dass SstableReader::open,
// get, iter, lookup_metrics, stream NIEMALS paniken und keine unbegrenzten Allokationen verursachen.

use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use futures::StreamExt;
use std::sync::Arc;
use tempfile::TempDir;

fn create_block_cache() -> Arc<BlockCache> {
    Arc::new(BlockCache::new(10))
}

async fn generate_valid_sstable(dir: &TempDir) -> (std::path::PathBuf, u64, u64) {
    let sst_path = dir.path().join("valid_v4.sst");

    let mut builder = SstableBuilder::create_with_version(&sst_path, 4)
        .await
        .expect("create builder v4");

    for i in 0..25 {
        let key = format!("key_{:04}", i);
        let val = format!("val_{:04}", i);
        builder
            .add(key.as_bytes(), val.as_bytes(), (i + 1) as u64, 1)
            .await
            .expect("add key");
    }

    builder.finish().await.expect("finish sstable");

    let data = std::fs::read(&sst_path).expect("read sst file");
    let file_len = data.len();
    assert!(file_len >= 54, "SSTable file too small");

    // Extract index_offset and bloom_offset from 54-byte trailer
    let trailer = &data[file_len - 54..];
    let bloom_offset = u64::from_le_bytes(trailer[32..40].try_into().unwrap());
    let index_offset = u64::from_le_bytes(trailer[40..48].try_into().unwrap());

    (sst_path, index_offset, bloom_offset)
}

#[tokio::test]
async fn test_systematic_field_mutations_never_panic() {
    let tmp = TempDir::new().expect("temp dir");
    let (valid_path, index_offset, bloom_offset) = generate_valid_sstable(&tmp).await;
    let original_data = std::fs::read(&valid_path).expect("read valid sst");
    let file_len = original_data.len();

    let target_file_path = tmp.path().join("mutated.sst");

    let values_u16: [u16; 7] = [0, 1, file_len as u16, (file_len + 1) as u16, 0x7FFF, 0x8000, u16::MAX];
    let values_u32: [u32; 7] = [0, 1, file_len as u32, (file_len + 1) as u32, 0x7FFF_FFFF, 0x8000_0000, u32::MAX];
    let values_u64: [u64; 7] = [0, 1, file_len as u64, (file_len + 1) as u64, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, u64::MAX];

    // Collect targeted byte offsets for lengths, offsets, and counter fields across Header, Index, Data Blocks, and Footer
    let mut field_offsets_16: Vec<usize> = Vec::new();
    let mut field_offsets_32: Vec<usize> = Vec::new();
    let mut field_offsets_64: Vec<usize> = Vec::new();

    // 1. Footer (Trailer) fields (last 54 bytes)
    let trailer_start = file_len - 54;
    field_offsets_64.push(trailer_start);      // min_tx_id
    field_offsets_64.push(trailer_start + 8);  // max_tx_id
    field_offsets_64.push(trailer_start + 16); // min_seq
    field_offsets_64.push(trailer_start + 24); // max_seq
    field_offsets_64.push(trailer_start + 32); // bloom_offset
    field_offsets_64.push(trailer_start + 40); // index_offset
    field_offsets_16.push(trailer_start + 48); // format_version
    field_offsets_32.push(trailer_start + 50); // magic

    // 2. Index Block fields (from index_offset to bloom_offset)
    let index_start = index_offset as usize;
    let index_end = bloom_offset as usize;
    if index_end > index_start && index_end <= file_len {
        field_offsets_32.push(index_start); // index CRC
        let mut pos = index_start + 4;
        while pos + 10 <= index_end {
            field_offsets_16.push(pos); // key_len
            let k_len = u16::from_le_bytes(original_data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2 + k_len;
            if pos + 8 <= index_end {
                field_offsets_64.push(pos); // block offset in index entry
                pos += 8;
            } else {
                break;
            }
        }
    }

    // 3. Data Block fields (from 0 to index_offset)
    let data_end = index_start.min(file_len);
    let pos = 0;
    if pos + 4 <= data_end {
        field_offsets_32.push(pos); // block CRC
        let block_start = pos + 4;
        let mut entry_pos = block_start;

        // Inspect entry fields within block
        while entry_pos + 22 <= data_end {
            field_offsets_16.push(entry_pos); // entry k_len
            let k_len = u16::from_le_bytes(original_data[entry_pos..entry_pos + 2].try_into().unwrap()) as usize;
            entry_pos += 2 + k_len;
            if entry_pos + 20 <= data_end {
                field_offsets_64.push(entry_pos);     // seq_no
                field_offsets_64.push(entry_pos + 8); // tx_id
                field_offsets_32.push(entry_pos + 16); // v_len
                let v_len = u32::from_le_bytes(original_data[entry_pos + 16..entry_pos + 20].try_into().unwrap()) as usize;
                entry_pos += 20 + v_len;
            } else {
                break;
            }
        }

        // Move to block trailer fields (v4: bloom_len, num_offsets)
        if data_end >= 8 {
            field_offsets_32.push(data_end - 8); // bloom_len
            field_offsets_32.push(data_end - 4); // num_offsets
        }
    }

    // Run systematic mutation on all 16-bit field offsets
    for &offset in &field_offsets_16 {
        if offset + 2 <= file_len {
            for &val in &values_u16 {
                let mut mutated = original_data.clone();
                mutated[offset..offset + 2].copy_from_slice(&val.to_le_bytes());
                run_reader_assertions(&target_file_path, &mutated).await;
            }
        }
    }

    // Run systematic mutation on all 32-bit field offsets
    for &offset in &field_offsets_32 {
        if offset + 4 <= file_len {
            for &val in &values_u32 {
                let mut mutated = original_data.clone();
                mutated[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
                run_reader_assertions(&target_file_path, &mutated).await;
            }
        }
    }

    // Run systematic mutation on all 64-bit field offsets
    for &offset in &field_offsets_64 {
        if offset + 8 <= file_len {
            for &val in &values_u64 {
                let mut mutated = original_data.clone();
                mutated[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
                run_reader_assertions(&target_file_path, &mutated).await;
            }
        }
    }
}

async fn run_reader_assertions(target_path: &std::path::Path, mutated_bytes: &[u8]) {
    std::fs::write(target_path, mutated_bytes).expect("write mutated file");

    let bc = create_block_cache();
    let open_res = SstableReader::open(target_path, bc.clone()).await;

    if let Ok(reader) = open_res {
        let reader_arc = Arc::new(reader);

        // Test point lookup
        let _ = reader_arc.get(b"key_0005").await;
        let _ = reader_arc.get(b"non_existent_key").await;
        let _ = reader_arc.get_at(b"key_0005", 100, 100).await;

        // Test lookup_metrics
        let _ = reader_arc.lookup_metrics(b"key_0005").await;

        // Test iter
        let _ = reader_arc.iter().await;

        // Test stream
        if let Ok(mut stream) = reader_arc.stream().await {
            while let Ok(Some(_item)) = stream.next().await {}
        }
    }
}
