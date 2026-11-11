# Format-Abdeckung `contextra-store` (PROOF-1)

Dieses Dokument erfasst alle persistenten Dateiformate, Header-Layouts, Magic-Bytes und Versionskonstanten in `contextra-store` basierend auf der tatsächlichen Quelltext-Verifikation.

---

## 1. Tabelle der persistenten Formate

| Format | Quelle (Datei:Konstante/Zeile) | Magic-Bytes / Konstante | Header-/Footer-Layout | Vorhandene Golden-Dateien | Lücke / Handlungsbedarf |
|---|---|---|---|---|---|
| **Manifest (v1)** | `src/manifest/core.rs:22-23` | `MANIFEST_HEADER_MAGIC = b"MFMN"`<br>`CURRENT_MANIFEST_VERSION = 1` | Header: 4B Magic (`MFMN`) + 1B Version (`0x01`). Einträge: Bincode/Framed TLV. | `manifest_v1.manifest` | Writer-Gleichheit bei Version 1, Leser-Kompatibilität bei Version >= 2. |
| **SSTable V4 (Standard)** | `src/sstable/block_cache.rs:451`<br>`src/sstable/builder.rs:549` | `SSTABLE_MAGIC_MFSX = 0x5853_464D` (LE: `b"MFSX"`) | 54-Byte Trailer am Dateiende: Data-Blocks, Index-Block (mit CRC32), Bloom-Filter (mit CRC32), Offsets und Magic. | `sstable_v4.sst` | Multiblock-SSTable mit aktivem Bloom-Filter als Golden ergänzen. |
| **SSTable V3 (Legacy)** | `src/sstable/block_cache.rs:452`<br>`src/sstable/reader.rs:439` | `SSTABLE_MAGIC_LEGACY = 0x4D46_5354` (LE: `b"MFST"`) | 44-Byte Legacy Trailer am Dateiende (`MFST`). | `sstable_legacy.sst` | Vollständig abgedeckt durch Lesetest in `golden_corpus.rs`. |
| **WAL V3 (Current)** | `src/wal/encode.rs:69`<br>`src/wal/flusher.rs:240` | `WAL_V3_HEADER = *b"MFW3"` | 4B Header `MFW3`. Jeder Eintrag: Seq(u64) + Len(u32) + Crc(u32) + Payload + HMAC(32B). | `wal_with_hmac.log` | Deterministischer WAL ohne HMAC (`wal_no_hmac.log`) sowie beschädigter Tail (`wal_corrupt_tail.log`) fehlen noch. |
| **WAL V2 (Legacy)** | `src/wal/encode.rs:66` | `WAL_V2_HEADER = *b"MFW2"` | 4B Header `MFW2`. HMAC ohne `tx_id`. | Keine | Durch Replay-Parser abgedeckt. |
| **Rollback Intent** | `src/lsm/recovery.rs:720` | `INTENT_MAGIC = b"MFRLBK\0\0"` | 8B Magic `b"MFRLBK\0\0"` + 8B `tx_id` (u64 LE) = 16 Bytes fest. | `rollback_tx_42.intent` | Vollständig abgedeckt. |
| **Tenant Key Codec** | `src/tenant_codec.rs:18-20` | Präfix `TenantId` (8 Bytes Big-Endian) | Key = `TenantId::to_be_bytes()` + raw key. | Keine | Testvektoren in `tenant_codec_vectors.json` fehlen. |

---

## 2. Liste der genutzten öffentlichen API-Pfade

Da STORE-A interne Module umbauen kann, werden in den Beweistests ausschließlich Pfade genutzt, die in `src/lib.rs` öffentlich re-exportiert sind:

- `contextra_store::manifest::{Manifest, ManifestEntry, CURRENT_MANIFEST_VERSION, MANIFEST_HEADER_MAGIC}`
- `contextra_store::sstable::{SstableBuilder, SstableReader, BlockCache, SSTABLE_MAGIC_MFSX, SSTABLE_MAGIC_LEGACY}`
- `contextra_store::wal::{Wal, WalOp, WalVersion, WAL_V3_HEADER, WAL_V2_HEADER}`
- `contextra_store::tenant_codec::TenantKeyCodec`
- `contextra_store::StoreError`, `contextra_store::StoreResult`
- `contextra_types::{TxId, TenantId}`
