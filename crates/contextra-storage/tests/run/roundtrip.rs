//! # Round-Trip-Tests für Run (SPEC §9.5, §11)
//!
//! Verifiziert, dass mit RunBuilder geschriebene Daten durch RunReader
//! exakt identisch gelesen werden können.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use contextra_storage::codec::tag::{Tag, TagKind};
use contextra_storage::run::blob_ref::BlobRef;
use contextra_storage::run::block::BlockSecurity;
use contextra_storage::run::block_cache::BlockCache;
use contextra_storage::run::builder::{RunBuilder, RunBuilderOptions};
use contextra_storage::run::reader::RunReader;
use contextra_storage_ports::{File, ReadReq};
use std::io;
use std::sync::Arc;

/// Hilfs-Dateityp für In-Memory-Tests ohne Dateisystem.
#[derive(Clone)]
pub struct MemoryFile(pub Arc<Vec<u8>>);

impl File for MemoryFile {
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize> {
        let off_usize = usize::try_from(off).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        if off_usize >= self.0.len() {
            return Ok(0);
        }
        let avail = &self.0[off_usize..];
        let n = std::cmp::min(buf.len(), avail.len());
        buf[..n].copy_from_slice(&avail[..n]);
        Ok(n)
    }

    fn read_batch(&self, reqs: &mut [ReadReq<'_>]) -> io::Result<()> {
        for req in reqs.iter_mut() {
            let mut total = 0usize;
            while total < req.buf.len() {
                let off = req.offset.saturating_add(total as u64);
                let n = self.read_at(&mut req.buf[total..], off)?;
                if n == 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF in read_batch"));
                }
                total += n;
            }
        }
        Ok(())
    }

    fn write_all_at(&self, _buf: &[u8], _off: u64) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "readonly"))
    }

    fn len(&self) -> io::Result<u64> {
        Ok(self.0.len() as u64)
    }

    fn set_len(&self, _len: u64) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "readonly"))
    }

    fn sync_data(&self) -> io::Result<()> {
        Ok(())
    }

    fn sync_all(&self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_basic_roundtrip() {
    let opts = RunBuilderOptions {
        space_id: 1,
        run_id: 42,
        header_len: 4,
        weights_k: 1,
        prefix_filter_len: Some(4),
        created_ms: 1_700_000_000_000,
    };

    let mut builder = RunBuilder::new(opts, BlockSecurity::Plain);

    let t1 = Tag::new(10, TagKind::Put);
    let t2 = Tag::new(20, TagKind::Put);
    let t3 = Tag::new(30, TagKind::PutRef);

    builder.add(b"key:001", t1, b"hdr_value_1").unwrap();
    builder.add(b"key:002", t2, b"hdr_value_2").unwrap();

    let bref = BlobRef::new(100, 4096, 500, 0x12345678, 1);
    let mut ref_val = b"hdr_".to_vec();
    ref_val.extend_from_slice(&bref.encode());
    builder.add(b"key:003", t3, &ref_val).unwrap();

    let bytes = builder.finish().unwrap();
    let file_len = bytes.len() as u64;
    let file = Arc::new(MemoryFile(Arc::new(bytes)));

    let reader = RunReader::open(file, file_len, BlockSecurity::Plain, None).unwrap();

    assert_eq!(reader.meta().run_id, 42);
    assert_eq!(reader.meta().entries, 3);
    assert_eq!(reader.meta().min_stamp, 10);
    assert_eq!(reader.meta().max_stamp, 30);
    assert_eq!(reader.meta().min_key, b"key:001");
    assert_eq!(reader.meta().max_key, b"key:003");

    // Abfragen
    let e1 = reader.get(b"key:001").unwrap().unwrap();
    assert_eq!(e1.user_key, b"key:001");
    assert_eq!(e1.tag, t1);
    assert_eq!(e1.value, b"hdr_value_1");

    let e2 = reader.get(b"key:002").unwrap().unwrap();
    assert_eq!(e2.tag, t2);
    assert_eq!(e2.value, b"hdr_value_2");

    let e3 = reader.get(b"key:003").unwrap().unwrap();
    assert_eq!(e3.tag, t3);
    assert_eq!(e3.value, ref_val);

    // Nicht vorhandener Schlüssel
    assert!(reader.get(b"key:999").unwrap().is_none());
    assert!(reader.get(b"abc").unwrap().is_none());
    assert!(reader.get(b"zzz").unwrap().is_none());

    // Präfix-Filter
    assert!(reader.prefix_may_contain(b"key:").unwrap());
    assert!(!reader.prefix_may_contain(b"other:").unwrap());
}

#[test]
fn test_multiblock_roundtrip_with_cache() {
    let opts = RunBuilderOptions {
        space_id: 1,
        run_id: 101,
        header_len: 0,
        weights_k: 1,
        prefix_filter_len: Some(3),
        created_ms: 1000,
    };

    let mut builder = RunBuilder::new(opts, BlockSecurity::Plain);

    // 500 Einträge erzeugen mehrere Blöcke
    for i in 0..500 {
        let k = format!("k_{:05}", i);
        let v = format!("value_{:05}", i);
        let tag = Tag::new((i + 1) as u64, TagKind::Put);
        builder.add(k.as_bytes(), tag, v.as_bytes()).unwrap();
    }

    let bytes = builder.finish().unwrap();
    let file_len = bytes.len() as u64;
    let file = Arc::new(MemoryFile(Arc::new(bytes)));

    let cache = Arc::new(BlockCache::new(10));
    let reader = RunReader::open(file, file_len, BlockSecurity::Plain, Some(cache.clone())).unwrap();

    assert!(reader.index().entries.len() > 1, "Sollte mehrere Datenblöcke haben");

    // Alle Einträge prüfen
    for i in 0..500 {
        let k = format!("k_{:05}", i);
        let v = format!("value_{:05}", i);
        let entry = reader.get(k.as_bytes()).unwrap().unwrap();
        assert_eq!(entry.value, v.as_bytes());
        assert_eq!(entry.tag.stamp(), (i + 1) as u64);
    }

    // Cache-Hit verifizieren
    assert!(cache.get(101, 0).is_some());
}

#[test]
fn test_rangedel_coverage_in_reader() {
    let opts = RunBuilderOptions {
        space_id: 1,
        run_id: 202,
        header_len: 0,
        weights_k: 1,
        prefix_filter_len: None,
        created_ms: 2000,
    };

    let mut builder = RunBuilder::new(opts, BlockSecurity::Plain);

    let t1 = Tag::new(10, TagKind::Put);
    let t2 = Tag::new(20, TagKind::Put);

    builder.add(b"user_a", t1, b"active").unwrap();
    builder.add(b"user_b", t2, b"active").unwrap();

    // RangeDelete [user_a, user_c) mit Stamp 15 verdeckt user_a (10 < 15), aber nicht user_b (20 > 15)
    let del_tag = Tag::new(15, TagKind::RangeDelete);
    builder.add_range_delete(b"user_a".to_vec(), Some(b"user_c".to_vec()), del_tag);

    let bytes = builder.finish().unwrap();
    let file_len = bytes.len() as u64;
    let file = Arc::new(MemoryFile(Arc::new(bytes)));

    let reader = RunReader::open(file, file_len, BlockSecurity::Plain, None).unwrap();

    assert!(reader.get(b"user_a").unwrap().is_none());
    assert!(reader.get(b"user_b").unwrap().is_some());
}

#[test]
fn test_read_batch_operation() {
    let opts = RunBuilderOptions {
        space_id: 1,
        run_id: 303,
        header_len: 0,
        weights_k: 1,
        prefix_filter_len: None,
        created_ms: 3000,
    };

    let mut builder = RunBuilder::new(opts, BlockSecurity::Plain);

    for i in 0..200 {
        let k = format!("batch_key_{:04}", i);
        let v = vec![b'x'; 200];
        builder.add(k.as_bytes(), Tag::new(i as u64 + 1, TagKind::Put), &v).unwrap();
    }

    let bytes = builder.finish().unwrap();
    let file_len = bytes.len() as u64;
    let file = Arc::new(MemoryFile(Arc::new(bytes)));

    let reader = RunReader::open(file, file_len, BlockSecurity::Plain, None).unwrap();
    let num_blocks = reader.index().entries.len();
    assert!(num_blocks >= 2);

    let indices: Vec<usize> = (0..num_blocks).collect();
    let payloads = reader.read_batch(&indices).unwrap();
    assert_eq!(payloads.len(), num_blocks);
}
