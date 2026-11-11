# Format- und Abdeckungsinventar — `contextra-vector`

Stand: 2026-11-11
Basis-Commit: `f92caabaa` (Stand `origin/main`)

## 1. Übersicht der Persistenz-Formate

| Format | Magische Bytes / Konstante | Version | Quellcode-Definition | Aktueller Golden-Stand | Deckungslücke |
|---|---|---|---|---|---|
| **HNSW v2 Raw** | `0x484E5357` (`"HNSW"`) | `2` (`HNSW_VERSION`) | `crates/contextra-vector/src/persistence/header.rs:14,16` | Vorhanden: `tests/golden/hnsw_raw.bin` | Keine für Standard; Tombstone-Zustand fehlt |
| **HNSW v2 SQ8** | `0x484E5357` (`"HNSW"`) | `2` (`HNSW_VERSION`) | `crates/contextra-vector/src/persistence/header.rs:14,16` | Vorhanden: `tests/golden/hnsw_sq8.bin` | Keine für Standard |
| **HNSW v1 Legacy** | `0x484E5357` (`"HNSW"`) | `1` | `crates/contextra-vector/src/persistence/header.rs:125` | Getestet in Unit-Tests (`persistence/tests.rs`) | Synthetisches Fixture für Header/Fallback vorhanden |
| **HNSW mit Tombstones** | `0x484E5357` (`"HNSW"`) | `2` | `crates/contextra-vector/src/hnsw/deletion.rs`, `persistence/` | Fehlt | Muss in `tests/golden/hnsw_tombstones.bin` erzeugt werden |
| **HNSW mit `docid-128`** | `0x484E5357` (`"HNSW"`) | `2` | Feature `docid-128`, `src/types.rs` | Fehlt | Spezifischer Test mit Feature-Gate erforderlich |
| **DiskANN Header (`DANN`)** | `b"DANN"` (`DISKANN_MAGIC`) | `1` (`DISKANN_VERSION`) | `crates/contextra-vector/src/diskann/format.rs:8,9` | Fehlt | Zu erzeugen mit Feature `experimental-diskann` |
| **DiskANN Footer (`DANF`)** | `b"DANF"` (`DISKANN_FOOTER_MAGIC`) | 1 (implizit) | `crates/contextra-vector/src/diskann/format.rs:10` | Fehlt | In DiskANN-Datei enthalten |
| **DiskANN Pending WAL (`PWAL`)** | `b"PWAL"` (`PENDING_WAL_MAGIC`) | `1` (`PENDING_WAL_VERSION`) | `crates/contextra-vector/src/diskann/format.rs:13,14` | Fehlt | Zu erzeugen via DiskAnnIndex WAL API |
| **DiskANN Tombstone WAL (`TWAL`)** | `b"TWAL"` (`TOMBSTONE_WAL_MAGIC`) | `1` (`TOMBSTONE_WAL_VERSION`) | `crates/contextra-vector/src/diskann/format.rs:17,18` | Fehlt | Zu erzeugen via DiskAnnIndex WAL API |
| **HNSW NodeRecord (Mmap)** | Struktur: `doc_id`, `layer_count`, `vector_offset`, `connections_offset` | – | `crates/contextra-vector/src/persistence/node.rs:13` | In `hnsw_raw.bin` / `hnsw_sq8.bin` eingebettet | Mmap-Reader-Validierung in `proof_hnsw_reader_compat.rs` |

## 2. Detailliertes Header-Layout

### 2.1 HNSW Header (`HnswHeader`)
- Größe: 84 Bytes (`HnswHeader::SIZE`)
- Layout:
  - `[0..4]`: `magic` (`0x484E5357` u32 LE)
  - `[4..6]`: `version` (`u16` LE, 1 oder 2)
  - `[6..10]`: `dimension` (`u32` LE)
  - `[10..14]`: `m` (`u32` LE)
  - `[14]`: `metric` (`u8`: 0=Cosine, 1=Euclidean, 2=DotProduct)
  - `[15]`: `quantized` (`u8`: 0=Nein, 1=Ja)
  - `[16..20]`: `q_min` (`f32` LE)
  - `[20..24]`: `q_max` (`f32` LE)
  - `[24..32]`: `node_count` (`u64` LE)
  - `[32..40]`: `entry_point` (`i64` LE)
  - `[40..48]`: `nodes_offset` (`u64` LE)
  - `[48..56]`: `connections_offset` (`u64` LE)
  - `[56..64]`: `last_tx_id` (`u64` LE)
  - `[64..72]`: `quant_calibration_offset` (`u64` LE)
  - `[72..76]`: `quant_calibration_len` (`u32` LE)
  - `[76..80]`: `sq8_bias_mean` (`f32` LE)
  - `[80..84]`: `sq8_bias_variance` (`f32` LE)

### 2.2 DiskANN Header (`DiskAnnHeader`) & Footer (`DiskAnnFooter`)
- Header-Größe: 40 Bytes
  - `[0..4]`: `magic` (`b"DANN"`)
  - `[4..6]`: `version` (`u16` LE = 1)
  - `[6..14]`: `node_count` (`u64` LE)
  - `[14..18]`: `dimension` (`u32` LE)
  - `[18..22]`: `max_degree` (`u32` LE)
  - `[22..26]`: `sector_size` (`u32` LE)
  - `[26..30]`: `entry_point` (`u32` LE)
  - `[30]`: `metric` (`u8`)
  - `[31]`: `quantized` (`u8`)
  - `[32..36]`: `q_min` (`f32` LE)
  - `[36..40]`: `q_max` (`f32` LE)
- Footer-Größe: 36 Bytes
  - `[0..4]`: `magic` (`b"DANF"`)
  - `[4..36]`: `hmac` (`[u8; 32]`, BLAKE3-Keyed MAC über Index-Daten mit `DISKANN_INTEGRITY_KEY`)

### 2.3 DiskANN WAL Formate (`PWAL` & `TWAL`)
- Header: 5 Bytes (`[MAGIC: 4 bytes] [VERSION: 1 byte = 0x01]`)
- Entry `PWAL`:
  - `doc_id`: 8 Bytes (bzw. 16 Bytes bei `docid-128`)
  - `dim`: 4 Bytes (`u32` LE)
  - `embedding`: `dim * 4` Bytes (`f32` LE)
  - `hmac`: 32 Bytes
- Entry `TWAL`:
  - `doc_id`: 8 Bytes (bzw. 16 Bytes bei `docid-128`)
  - `hmac`: 32 Bytes

## 3. Öffentliche Pfade (`src/lib.rs`) für VEC-A Kompatibilität
Die folgenden Re-Exports sind stabil und dürfen in den Beweistests genutzt werden:
- `candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE}`
- `compute_pool::ComputePool`
- `contract::{BlockTransform, ContractClock, DistanceFn, ContractError, ExternalId, ContractIndex, ContractMaintain, ContractMetric, ContractResult, ContractRng, ScoredPoint}`
- `diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex}` (unter `#[cfg(feature = "experimental-diskann")]`)
- `hnsw::{HnswConfig, HnswIndex, RebuildStatus}`
- `persistence::{HnswHeader, MmapIndex}`
- `quantize::ScalarQuantizer`
- `quantize_rabitq::RaBitQQuantizer` (unter `#[cfg(feature = "experimental-rabitq")]`)
- `types::{Clock, DistanceMetric, DocId, Maintain, MetricsSink, Rng, ScoredDocument, TxId}`
- `distance::{compute_distance, validate_vector, ScalarKernel, DistanceKernel, ...}` (in `src/distance.rs`, Modul `pub mod distance`)
