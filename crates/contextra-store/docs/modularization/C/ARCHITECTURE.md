# Architektur-Dokumentation: STORE-MOD-C (Manifest MFMN & MemTable)

## 1. Manifest-Dateilayout (`MFMN` v2 / v1)

Das Manifest ist ein append-only Protokoll für SSTable-Generationen und WAL-Checkpoints.

| Byte-Offset | Feld | Typ | Beschreibung |
|---|---|---|---|
| `0..4` | Magic | `[u8; 4]` | `b"MFMN"` (Manifest Magic Number) |
| `4..5` | Version | `u8` | `1` (Legacy) oder `2` (Aktuell) |
| `5..` | Log Frames | Folge von Frames | Siehe Frame-Struktur unten |

### Frame-Struktur je Manifest-Eintrag
| Feld | Typ | Beschreibung |
|---|---|---|
| `len` | `u32` (LE) | Gesamtlänge des Payloads (ohne die 4 Längenbytes) |
| `op_tag` | `u8` | Operationscode: `0=Add`, `1=Remove`, `2=WalCheckpoint`, `3=Replace`, `4=RollbackComplete` |
| `payload` | variabel | Operationsspezifische Nutzlast |
| `crc32` | `u32` (LE) | CRC32-Prüfsumme über `op_tag` + `payload` |

---

## 2. Recovery-Algorithmus (Schritte)

1. **Header-Validierung:**
   - Prüfung auf Magic `b"MFMN"`.
   - Versionsprüfung: `version <= CURRENT_MANIFEST_VERSION`.
2. **Sequentielles Frame-Replay:**
   - Einlesen der Frame-Länge `len`.
   - Bei unvollständigem Frame am Dateiende (Tail-Truncation durch Stromausfall): sicherer Abbruch und Beibehaltung der bis dahin gelesenen Einträge.
   - CRC32-Validierung je Frame: Fail-closed bei Mismatch.
3. **Zustandsrekonstruktion:**
   - `reconstruct_valid_sstables`: Bildet die Menge aktiver SSTables.
   - `reconstruct_dead_sstables`: Identifiziert endgültig ersetzte oder gelöschte SSTables zur Vermeidung von Wiederauferstehung nach Crash (INV-STORE-1).

---

## 3. MemTable-Shard-Modell

- **Partitionierung:** 16 Shards (`SHARD_COUNT = 16`).
- **Standard-Keys:** 2-Byte-Präfix Range-Sharding (`(b0 << 8) | b1 % 16`).
- **System-Keys:** Hash-Sharding (`ahash`/FNV) auf den Suffix nach dem reservierten Präfix (z. B. `__col:`, `__meta:`).
- **MVCC & Tombstones:**
  - Jede Version speichert `(seq_no, value, tx_id)`.
  - Bit 63 (`TOMBSTONE_BIT = 1u64 << 63`) markiert Löschung.
  - Sequenzen bleiben aufsteigend sortiert für O(log N) Snapshot-Lookups.
