# Eingefrorene Außenfläche (STORE-MOD-B: SSTable)

Dieser Schnittstellenvertrag listet alle Typen, Funktionen, Konstanten und Trait-Definitionen auf, die von Modulen außerhalb von `src/sstable/` konsumiert werden. Alle Signaturen, Sichtbarkeiten und Verhaltensweisen bleiben strikt erhalten.

## 1. Öffentliche Symbole (`contextra_store::sstable::*`)

### Typen & Structs
1. `BlockCache`
   - Signatur: `pub struct BlockCache { ... }`
   - Methoden:
     - `pub fn new(capacity_bytes: usize) -> Self`
     - `pub fn with_shards(capacity_bytes: usize, num_shards: usize) -> Self`
     - `pub fn get(&self, file_id: u64, offset: u64) -> Option<Arc<Vec<u8>>>`
     - `pub fn insert(&self, file_id: u64, offset: u64, block: Arc<Vec<u8>>)`
     - `pub fn remove_file(&self, file_id: u64)`
     - `pub fn clear(&self)`
     - `pub fn entry_count(&self) -> usize`
     - `pub fn size_bytes(&self) -> usize`
   - Fundstellen: `src/scanner/store_scan_view.rs`, `src/lsm/engine.rs`, `src/compaction/engine.rs`, `src/compaction/tests.rs`, `tests/golden_corpus.rs`, `tests/sstable_audit.rs`, `benches/block_cache_bench.rs`, `contextra-engine/tests/wasm_merge_operator.rs`.

2. `BlockCacheBackend`
   - Trait: `pub trait BlockCacheBackend: Send + Sync { ... }`
   - Implementierungen: `LruBlockCacheBackend`, `QuickCacheBlockCacheBackend`, `SieveCacheBackend`

3. `SstableBuilder`
   - Signatur: `pub struct SstableBuilder { ... }`
   - Methoden:
     - `pub fn create(path: &Path) -> Result<Self>`
     - `pub fn create_with_block_size(path: &Path, block_size: usize) -> Result<Self>`
     - `pub fn create_v3_legacy(path: &Path) -> Result<Self>`
     - `pub fn create_with_options(path: &Path, opts: SstableBuilderOptions) -> Result<Self>`
     - `pub async fn add(&mut self, key: &[u8], value: &[u8], seq: u64, tx_id: u64) -> Result<()>`
     - `pub async fn finish(self) -> Result<SstableMetadata>`
   - Fundstellen: `src/scanner/sstable.rs`, `src/compaction/engine.rs`, `src/lsm/mod.rs`, `tests/golden_corpus.rs`, `tests/sstable_audit.rs`.

4. `SstableReader`
   - Signatur: `pub struct SstableReader { ... }`
   - Methoden:
     - `pub async fn open(path: &Path, cache: Arc<BlockCache>) -> Result<Self>`
     - `pub async fn open_with_opts(...) -> Result<Self>`
     - `pub async fn get(&self, key: &[u8]) -> Result<Option<(Bytes, u64, u64)>>`
     - `pub async fn get_with_stats(&self, key: &[u8]) -> Result<Option<...>>`
     - `pub fn first_key(&self) -> &Bytes`
     - `pub fn last_key(&self) -> &Bytes`
     - `pub fn format_version(&self) -> u32`
     - `pub fn path(&self) -> &Path`
   - Fundstellen: `src/scanner/sstable.rs`, `src/scanner/store_scan_view.rs`, `src/lsm/engine.rs`, `src/compaction/adaptive.rs`, `src/compaction/engine.rs`, `tests/golden_corpus.rs`.

5. `BloomFilter`
   - Signatur: `pub struct BloomFilter { ... }`
   - Methoden:
     - `pub fn build_from_keys(keys: &[&[u8]], fpr: f64) -> Self`
     - `pub fn from_bytes(bytes: &[u8]) -> Result<Self>`
     - `pub fn as_bytes(&self) -> &[u8]`
     - `pub fn may_contain(&self, key: &[u8]) -> bool`
   - Fundstellen: `tests/sstable_block_bloom_fpr.rs`, `tests/amplification_benchmark.rs`, `tests/sstable_audit.rs`, `fuzz/fuzz_targets/fuzz_sstable_block_footer.rs`.

6. `BlockBuilder` & `SstableMetadata`
   - Re-exportiert aus `builder.rs`.

7. `SstableStream`
   - Re-exportiert aus `stream.rs`.

### Funktionen
1. `create_block_cache(capacity_mb: usize) -> Arc<BlockCache>`
2. `create_block_cache_with_shards(capacity_mb: usize, num_shards: usize) -> Arc<BlockCache>`
3. Block-Suchfunktionen:
   - `binary_search_first_index_in_block`
   - `binary_search_index_in_block`
   - `binary_search_entry_in_block`
   - `block_binary_search`
   - `get_entry_at_index`
   - `get_entry_key_at_index`
4. Trailer Parsing:
   - `parse_block_trailer`

### Konstanten
1. `SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D;` ("MFSX")
2. `SSTABLE_MAGIC_LEGACY: u32 = 0x4D46_5354;` ("MFST")
3. `BLOCK_CACHE_SHARDS: usize = 64;`
