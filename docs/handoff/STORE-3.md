# Übergabebericht `STORE-3`: SSTable: Format, Builder, Reader, Bloom, Block-Cache

## 1. Übersicht & Ziel
In diesem Auftrag wurde die SSTable-Schicht von `contextra-store` (`src/sstable/**`) als defensiv dekodierendes, robustes und abwärtskompatibles Tabellenformat für das autarke Storage-Bundle (Klasse B) gehärtet.

## 2. Umgesetzte Maßnahmen
- **Builder (`src/sstable/builder.rs`):**
  - Streaming-Aufbau mit konfigurierbarer Blockgröße (`with_block_size`) und Bloom-Filter-Parametern (`with_bloom_fpr`, `set_bloom_filter`).
  - Strikte Prüfung der Sortierreihenfolge (`key < prev_key` wird abgewiesen) sowie streng absteigender Sequenznummern bei gleichen Schlüsseln.
  - Fail-Closed-Vergiftung des Handles bei I/O- und fsync-Fehlern (`poisoned: bool`, `INV-FSYNC-1`).
  - Atomare Fertigstellung (`file.sync_all()` -> atomares Rename via `vfs.rename()` -> `vfs.sync_dir()`).
  - `Drop`-Guard auf `SstableBuilder`: Unvollständige oder fehlgeschlagene temporäre SSTables werden deterministisch gelöscht.
- **Reader & Search (`src/sstable/reader.rs`, `block_search.rs`, `reader_ext.rs`, `stream.rs`):**
  - Strikte Validierung aller Offsets, Längen, Restart-Punkte und Varints gegen Block- und Dateigrenzen.
  - Verhinderung von Puffer-Überläufen: Einträge dürfen `offsets_start` niemals überschreiten.
  - Validierung von Index-Offsets und Schlüsseln auf monotone Ordnung während des Öffnens (`open_with_vfs`).
  - CRC32-Prüfsummenprüfung schlägt bei Bit-Rot deterministisch fehl (`ContextraError::ChecksumMismatch`).
- **Bloom-Filter (`src/sstable/bloom.rs`):**
  - Doppel-Hashing mit festem, dokumentiertem Seed-Verhalten über Blake3-Digest.
  - Zero-Length-Guard (`num_bits == 0`) in `insert` und `may_contain` verhindert Division durch 0.
- **Block-Cache (`src/sstable/block_cache.rs`):**
  - Sharded LRU mit Byte-Budget.
  - Hit- und Miss-Zähler (`AtomicU64`) mit `hits()`, `misses()`, `hit_ratio()` und `reset_metrics()`.
  - Introspektion für `current_bytes()` und `total_capacity_bytes()`.
  - Cache-Poisoning-Ausschluss: Nur geprüfte, intakte Blöcke werden nach CRC-Verifikation im Cache abgelegt.
- **Benchmarks & Tests (`benches/sstable_bench.rs`, `tests/chaos_bitflip_sstable.rs`):**
  - Benchmark-Suite um `streaming_build`, `point_lookup_with_cache`, `point_lookup_without_cache` und `sequential_scan` erweitert.
  - Mutationsmatrix-Test für Trailer/Footer-Bitflips in `chaos_bitflip_sstable.rs` ergänzt.

## 3. Invarianten
- **INV-FORMAT-1:** Deterministiche Header-, Magic- (`MFSX`) und Versionsprüfung (v0 bis v4 unterstützt).
- **INV-FSYNC-1:** Fehlerhafte fsync- oder Schreiboperationen vergiften das Builder-Handle unwiderruflich; temporäre Dateien werden im Fehlerfall gelöscht.

## 4. Scope-Einhaltung
Alle Änderungen erfolgten ausschließlich innerhalb der zugewiesenen OWNS-Pfade:
- `crates/contextra-store/src/sstable/**`
- `crates/contextra-store/tests/chaos_bitflip_sstable.rs`
- `crates/contextra-store/benches/sstable_bench.rs`
- `docs/handoff/STORE-3.md`

Keine geschützten Dateien (`Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `governance/**` etc.) wurden modifiziert.
Offene Punkte für STORE-5: In `src/lib.rs` verwaistes `pub mod kv;` entfernen und Konvergenzabgleich durchführen.
