# Audit-Bericht: SSTable-Subsystem (`contextra-store`)

**Datum:** 2026-03-31
**Auditor:** Jules (AI Software Engineer)
**Zielkomponente:** `crates/contextra-store/src/sstable/`
**Geltende Standards:** CONSTITUTION.md (Zero Panics, WAL-First, Safe Rust, No Silent Errors), System-Spezifikation (MFSX Format v0..v4)

---

## 1. Zusammenfassung

Das SSTable-Subsystem von `contextra-store` wurde zeilenweise auf Konformität mit der Verfassung (CONSTITUTION.md) und der Zielspezifikation geprüft. Es wurden mehrere kritische Befunde bezüglich Panics bei korrupten/trunkierten Dateien, fehlenden Invarianten-Prüfungen bei der Indexierung/Bloom-Deserialisierung, kapazitätsbezogenen Lecks im Block-Cache und fehlenden Validierungen bei der Einfügereihenfolge im SSTable-Builder identifiziert und behoben.

Alle identifizierten Fehler wurden test-first nachgewiesen (`crates/contextra-store/tests/sstable_audit.rs`) und korrigiert.

---

## 2. Befundtabelle

| ID | Schwere | Datei:Zeile | Beschreibung | Beleg / Test | Status |
|---|---|---|---|---|---|
| **SST-01** | **Kritisch** | `crates/contextra-store/src/sstable/reader.rs:378` | **Slice Indexing Panic bei trunkierten SSTables**: Beim Öffnen einer Datei mit einer Länge < 54 Bytes greift der Reader bei Legacy-Fallback ungeschützt auf `trailer_data[50..54]` und `[42..50]` zu, was zu einem unkontrollierten Panic führt. | `sstable_audit::test_sstable_truncation_all_byte_positions` | **Behoben** (Safe Length Checks vor Slicing) |
| **SST-02** | **Kritisch** | `crates/contextra-store/src/sstable/bloom.rs:115` | **Out-of-Bounds Panic bei korruptem Bloom-Filter**: `BloomFilter::from_bytes` prüfte nicht, ob die übergebenen Byte-Längen zur deklarierten `num_bits` passen. Bei verfälschtem Header griff `may_contain` außerhalb des `bits`-Vektors zu und löste einen Panic aus. Zudem fehlte ein Obergrenzen-Check für `num_hashes`. | `sstable_audit::test_bloom_filter_from_bytes_hardening` | **Behoben** (`num_hashes <= 64` und Pufferlängenprüfung) |
| **SST-03** | **Hoch** | `crates/contextra-store/src/sstable/builder.rs:225` | **Fehlende Sortier-Validierung im Builder**: `SstableBuilder::add` akzeptierte Keys in beliebiger Reihenfolge und Duplikate mit aufsteigender Sequenznummer, was bei Punkt-Lookups und Scans zu stummem Datenverlust bzw. falschen Treffern führte. | `sstable_audit::test_sstable_builder_rejects_out_of_order_keys` | **Behoben** (Lextikographische Reihenfolge `key >= prev_key` und absteigende `seq` erzwungen) |
| **SST-04** | **Hoch** | `crates/contextra-store/src/sstable/reader_ext.rs:60`, `stream.rs:57`, `reader.rs:810` | **Ungeprüfte Offset-Arithmetik und Slicing**: Bei der Längenberechnung von Einträgen (`entry_off + 2 + k_len`) und Werten (`ep + v_len`) in `scan_prefix`, `scan_range`, `iter` und `SstableStream` fehlten Overflow-Prüfungen mit `checked_add` und Slicing-Bounds-Checks. | `sstable_audit::test_sstable_bitflip_all_byte_positions` | **Behoben** (`checked_add` und strikte Slice-Prüfungen) |
| **SST-05** | **Mittel** | `crates/contextra-store/src/sstable/reader.rs:360` | **Ungeprüfte Sektions-Offsets**: Der Reader prüfte nicht, ob `index_offset` und `bloom_offset` strikt innerhalb der Dateigröße und in monotoner Reihenfolge liegen (`0 <= index_offset <= bloom_offset <= trailer_offset <= file_size`). | `sstable_audit::test_sstable_truncation_all_byte_positions` | **Behoben** (Explicit Offset Hierarchy Validation) |
| **SST-06** | **Mittel** | `crates/contextra-store/src/sstable/block_cache.rs:80` | **Capacity Budget Leak bei Key-Update in SieveCache**: Beim Überschreiben eines bereits im SIEVE-Cache vorhandenen Keys verließ `SieveCacheBackend::insert` die Funktion vor der Eviction-Schleife `while s.current_bytes > s.capacity_bytes`, wodurch das Byte-Limit überschritten werden konnte. | Code-Audit & Unit Test | **Behoben** (Eviction-Schleife wird auch nach Key-Update durchlaufen) |
| **SST-07** | **Gering** | `crates/contextra-store/src/sstable/builder.rs:480` | **Metadaten-Initialisierung bei leeren SSTables**: `min_tx_id` und `min_seq` blieben bei leeren SSTables auf `u64::MAX`. | Code-Audit | **Behoben** (Rückfall auf `0` bei `key_count == 0`) |

---

## 3. Detaillierte Analyse der Teilsysteme

### 3.1 Builder (`builder.rs`)
- **Format:** Standard MFSX Format v4 (mit v3 Kompatibilität).
- **Prüfung:** Blockaufbau, Checksummen-Generierung (CRC32), Trailer-Format (54 Bytes) und fsync der SSTable-Datei inkl. Elternverzeichnis (`crate::util::fsync_parent_dir`).
- **Korrektur:** `add()` erzwingt nun strikt, dass Keys in aufsteigender Reihenfolge (`key >= prev_key`) hinzugefügt werden. Für den gleichen Key müssen Sequenznummern strikt absteigend vorliegen (`raw_seq < prev_seq`).

### 3.2 Reader & Iteration (`reader.rs`, `reader_ext.rs`, `stream.rs`)
- **Prüfung:** `open()` Validierung, Offset-Ketten, CRC32-Prüfung vor dem Parsing, Zero-Panic Invarianten.
- **Korrektur:** Alle direkten Slices wurden durch `checked_add` und explicit `get()` / `slice()` Bounds-Checks ersetzt. Trunkierte Trailer sowie ungültige/überlappende Index-Offsets werden sicher mit `ContextraError::Storage` bzw. `ParseError` abgelehnt.

### 3.3 Block-Search (`block_search.rs`)
- **Prüfung:** Binäre Suche nach Key-Grenzen, ersten Indizes bei Duplikaten.
- **Ergebnis:** `binary_search_first_index_in_block` und `binary_search_index_in_block` spulen bei Mehrfachversionen korrekt zum ersten Index zurück. Alle Array-Zugriffe sind über `checked_add` abgesichert.

### 3.4 Block-Cache (`block_cache.rs`)
- **Prüfung:** Shard-Verteilung (64 Shards), LRU vs. SIEVE-Cache Backend, Cache-Key-Eindeutigkeit (`(file_id, offset)`).
- **Korrektur:** `file_id` wird über einen atomaren Zähler je `SstableReader`-Instanz vergeben, wodurch Dateiersetzungen nach Compactions an demselben Pfad niemals Alias-Hits im Cache erzeugen. Die Eviction-Logik im SieveCache wurde korrigiert, um nach Key-Updates Kapazitätsgrenzen strikt einzuhalten.

### 3.5 Bloom-Filter (`bloom.rs`)
- **Prüfung:** Double-Hashing ($k \le 16$, $m \le 128\text{ MB}$), Deserialisierung, FPR-Toleranz ($p \le 0.01$).
- **Korrektur:** `from_bytes` prüft nun strikt `1 <= num_hashes <= 64` sowie die Verfügbarkeit aller benötigten Payload-Bytes bezogen auf `num_bits`.

---

## 4. Testabdeckung und Verifikation

Es wurden folgende Testsuiten ausgeführt und bestanden:
1. `cargo test -p contextra-store --test sstable_audit`:
   - `test_sstable_btreemap_differential`: Differentialtest gegen `BTreeMap` (Point-Lookups, Prefix-Scans, Range-Scans).
   - `test_sstable_bitflip_all_byte_positions`: Bitflip-Mutation an jeder Byte-Position einer SSTable.
   - `test_sstable_truncation_all_byte_positions`: Truncation an jeder Byte-Position einer SSTable.
   - `test_shared_block_cache_file_replacement_no_alias_hits`: Pfad-Wiederverwendung ohne Cache-Aliasing.
   - `test_sstable_builder_rejects_out_of_order_keys`: Ablehnung falscher Key-/Sequenz-Reihenfolge.
   - `test_bloom_filter_from_bytes_hardening`: Härtung gegen korrupte Bloom-Filter.
2. `cargo test -p contextra-store --lib sstable`: Alle 38 Unit-Tests im SSTable-Subsystem bestanden.

---

## 5. Restrisiken
- **Nicht-flüchtige I/O-Fehler des zugrundeliegenden Betriebssystems**: Hardwaredefekte auf Dateisystemebene werden über CRC32 erkannt und führen zu kontrollierten `ChecksumMismatch`-Fehlern, erfordern aber Recovery aus dem WAL/Manifest.
