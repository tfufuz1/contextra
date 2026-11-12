# SSTable Architektur & Spezifikation (Ring 1, STORE-MOD-B)

## 1. Übersicht & Zweck
Die SSTable-Komponente (`contextra-store::sstable`) stellt unveränderliche, sortierte Festplattendateien (Sorted String Tables) für das LSM-Tree-Speichertriebwerk bereit. Sie unterstützt wahlfreien Punkt-Lookup mit Bloom-Filter-Beschleunigung, sequentielle Bereichsabfragen (Range Scans), blockbasiertes Caching und atomare Schreibvorgänge.

## 2. Persistiertes Dateilayout (Format V4 `MFSX` vs. Legacy `MFST`)

### 2.1 MFSX (Format V4 - Standard)
```
+--------------------------------------------------------------------------+
| Data Block 0: [Entry 0] ... [Entry N-1] [Offsets] [Num Offsets] [CRC32]  |
+--------------------------------------------------------------------------+
| Data Block 1: ...                                                        |
+--------------------------------------------------------------------------+
| ...                                                                      |
+--------------------------------------------------------------------------+
| Block Index: [(first_key_len, first_key, block_offset)]                  |
+--------------------------------------------------------------------------+
| Bloom Filter: [CRC32 (4B)] [Serialized Bloom Filter Data]                |
+--------------------------------------------------------------------------+
| Trailer (54 Bytes):                                                      |
|   - min_tx_id    (u64, 8B, LE)                                           |
|   - max_tx_id    (u64, 8B, LE)                                           |
|   - min_seq      (u64, 8B, LE)                                           |
|   - max_seq      (u64, 8B, LE)                                           |
|   - bloom_offset (u64, 8B, LE)                                           |
|   - index_offset (u64, 8B, LE)                                           |
|   - version      (u16, 2B, LE = 4)                                       |
|   - magic        (u32, 4B, LE = 0x5853_464D, "MFSX")                     |
+--------------------------------------------------------------------------+
```

### 2.2 MFST (Legacy V3 Format)
```
+--------------------------------------------------------------------------+
| Data Block 0: [Entry 0] ... [Offsets] [Num Offsets]                      |
+--------------------------------------------------------------------------+
| ...                                                                      |
+--------------------------------------------------------------------------+
| Block Index: [(first_key_len, first_key, block_offset)]                  |
+--------------------------------------------------------------------------+
| Trailer (54 Bytes):                                                      |
|   - Reserved / Padding (42 Bytes)                                        |
|   - index_offset (u64, 8B, LE)                                           |
|   - magic        (u32, 4B, LE = 0x4D46_5354, "MFST")                     |
+--------------------------------------------------------------------------+
```

## 3. Lesepfad (Point Lookup & Range Scan)
1. **Trailer Parsing (`SstableFooter::parse`)**:
   - Liest die letzten 54 Bytes der Datei.
   - Prüft Magic (`MFSX` oder `MFST`).
   - Extrahiert `index_offset`, `bloom_offset`, `min_seq`, `max_seq`.
2. **Whole-SSTable Bloom-Filter Pre-Check**:
   - Falls `bloom_filter.may_contain(key) == false`, sofortiger Return `Ok(None)` ohne Disk-I/O.
3. **Index Binary Search**:
   - Binärsuche im Block-Index nach dem Block mit dem größten `first_key <= key`.
4. **Block Cache Lookup**:
   - Abfrage von `(file_id, block_offset)` im geshardeten `BlockCache`.
   - Bei Cache-Miss: Block über VFS laden, CRC32 validieren und in den Cache einfügen.
5. **In-Block Binärsuche**:
   - Zero-Copy-Binärsuche über die blockinternen Offsets nach dem gesuchten Schlüssel.

## 4. Modulare Komponentenstruktur
- `src/sstable/format.rs`: Magic-Bytes, Konstanten und strukturiertes `SstableFooter`-Parsing.
- `src/sstable/reader/`:
  - `open.rs`: Initialisierungs- und Öffnungspfade (VFS/Mmap, Index- und Bloom-Filter-Laden).
  - `block.rs`: Block-Lesen und Zero-Copy-Dekodierung.
  - `lookup.rs`: Punktabfragen (`get`, `get_at`) und Block-Bloom-Prüfung.
  - `iter.rs`: Streaming- und Iterationslogik.
- `src/sstable/block_cache/`:
  - `traits.rs`: `BlockCacheBackend` Abstraktion.
  - `lru.rs`: Standard LRU-Cache.
  - `quick.rs`: Lock-optimiertes S3-FIFO-Backend (`block-cache-v2`).
  - `sieve.rs`: SIEVE-Cache-Backend.
  - `mod.rs`: Geshardete Cache-Fassade (`BlockCache`).
- `src/sstable/builder/`:
  - `block.rs`: Datenblock-Formatierung (`BlockBuilder`).
  - `meta.rs`: Metadaten (`SstableMetadata`).
  - `writer.rs`: SSTable-Erstellung und atomares Schreiben (`SstableBuilder`).
- `src/sstable/bloom.rs`: Whole-SSTable und Per-Block Bloom-Filter.
- `src/sstable/block_search.rs`: Hochoptimierte Binärsuchalgorithmen im Block.
- `src/sstable/stream.rs`: Sequentieller Stream-Iterator über Blöcke.
