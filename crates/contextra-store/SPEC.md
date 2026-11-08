# SPEC — `contextra-store`

**Normativitätsklasse:** Klasse B (Storage Bundle)
**Ring:** Ring 1
**Status:** In Entwicklung (Welle 3, Auftrag A)
**Normative Referenz:** `docs/SPEC.md` v4.0 (§2, §3, §4, §5)

---

## 1. Zweck & Systemgrenzen
`contextra-store` stellt ein eigenständiges, persistentes Log-Structured Merge-Tree (LSM) Key-Value-Speichertriebwerk bereit.

### Zielgarantien:
- **Atomarität & Durability:** Zweiphasiger WAL-Flush mit deterministischer Integritätsprüfung (HMAC-Kette).
- **Concurrency & Snapshot Isolation:** MVCC-fähige MemTable mit Versionsketten; deterministisches Sharding.
- **Speichersicherheit:** `#![forbid(unsafe_code)]` ohne Ausnahme im gesamten Produktionscode.
- **Entkopplung:** Keine Rechts- oder Löschsagas (DSGVO/GDPR, Beweise, Berichte) im Speicherbaustein. Neutrale Wartung über `reclaim()` und `compact()`.

---

## 2. Invarianten (Normativ)

### INV-STORE-1: Durability & Fsync (INV-FSYNC-1)
- Datei- und Verzeichnis-Syncs dürfen nicht verworfen werden (`let _ =`). Jeder Fehler bei `fsync` führt zur Vergiftung des Handles und wird fail-closed als `ContextraError::Storage` propagiert.
- Atomare Dateiupdates erfolgen über: temporäre Datei schreiben → `fsync` der Datei → atomarer `rename` → `fsync` des übergeordneten Verzeichnisses.

### INV-STORE-2: Zweiphasiger WAL-Flush & Open-Heal
- Der WAL-Flush erfolgt zweiphasig: Schreiben des Payloads mit anschließender Aktualisierung der HMAC-Kette.
- Bei unvollständigen Schreibvorgängen am Dateiende (Torn Writes) garantiert die Engine Open-Heal: Abgetrennte Enden werden bis zum letzten gültigen TxEnd-Eintrag erkannt und getrimmt.

### INV-STORE-3: MemTable Out-of-Order Sequenzen
- Die MemTable nimmt Mutationen nebenläufig über Shards auf. Versionen pro Schlüssel werden strikt nach `seq_no` sortiert gehalten (`binary_search_by_key`).
- Höhere Sequenznummern überschatten ältere; Out-of-Order Schreibvorgänge dürfen keine Versionskorruption verursachen.

### INV-STORE-4: Tombstone-Disziplin
- Tombstones werden durch Setzen des höchstwertigen Bits (`TOMBSTONE_BIT = 1 << 63`) in der Sequenznummer repräsentiert.
- Bei allen Sequenzvergleichen (`max_seq`, Snapshot-Checks) ist Bit 63 vorher zu maskieren (`seq & !TOMBSTONE_BIT`).

### INV-STORE-5: Single-Load Snapshot Isolation
- Bei `get_at_seq` oder `scan_prefix_at` wird die committed Sequenznummer genau einmal zu Beginn der Operation geladen.
- Flush-before-Visible: Bevor eine neue SSTable in den aktiven Lesepfad übernommen wird, müssen alle darin enthaltenen Commits im Manifest persistiert sein.

---

## 3. Schnittstellen & Wartung

### Neutrale physische Wartung
- `LsmStorage::reclaim(&self) -> Result<u64>`:
  Flusht anstehende MemTable-Inhalte auf Platte und kürzt verwaiste WAL-Schwänze (`wal.truncate_uncommitted_tail()`).
- `LsmStorage::compact(&self) -> Result<()>`:
  Stößt die Zusammenführung und Bereinigung von SSTables über die Compaction-Engine an.

### Introspektion (`inspect` Feature)
- `LsmStorage::sstables_snapshot(&self) -> Vec<Arc<SstableReader>>`
- `LsmStorage::memtables_snapshot(&self) -> (Arc<MemTable>, Vec<Arc<MemTable>>)`
  Stellen unveränderliche Momentaufnahmen für Scan-Adapter bereit.

---

## 4. Fehlermodell
Alle öffentlichen Operationen geben `contextra_types::Result<T>` zurück.
- Fehlerarten:
  - `ContextraError::Storage`: I/O-Fehler, Prüfsummenfehler, Formatfehler, vergiftete Locks.
  - `ContextraError::InvalidInput`: Ungültige Schlüssel-, Batch- oder Transaktionsparameter.
- Mutex-Poisoning führt nie zum Panic, sondern stets zu `ContextraError::Storage`.
