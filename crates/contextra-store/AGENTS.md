# AGENTS.md — `contextra-store`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.8](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-store` ist der blinde, transaktionale LSM-Tree-Speicherkern von Contextra. Er verwaltet Write-Ahead-Log (WAL), Memtables, persistente SSTables, das Manifest, Hintergrundkompaktionen und **Purge-Primitive**. Er kennt weder Dokumentinhalte noch Volltext- oder Vektorindizes, sondern operiert rein auf `KeyBuf`, `Seq`, opaken Bytes und `Matcher`.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 2 (Speicherschicht)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-crypto`, `contextra-mvcc`.
- **Verbotene Abhängigkeiten:** Keine Abhängigkeit zu `text`, `vector`, `erasure`, `engine` oder `contextra-durable-fs` (I/O ausschließlich über den `Vfs`-Port aus `contextra-ports`!).
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Kein direkter Aufruf von `std::fs` oder `tokio::fs` (D-04: I/O ausschließlich über `Vfs`).
  - Kein `let _ =` auf `sync_*` (vollständige Fehlerpropagation).
  - Kein unverschlüsselter Blockpfad (Verschlüsselung ist Pflicht, D-05).
  - Kein `HashMap`/`HashSet` in Manifest oder Kernpfaden (ausschließlich `BTreeMap`/`BTreeSet`, P28).

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.8)

| Modul | Inhalt & Verantwortung |
|---|---|
| `codec` | Cluster-Codec (SPEC §9.1): `encode_data_key`, `encode_loc_key`, `encode_meta_key`, `parse`, `range_of(Target)`. Ersetzt `tenant_codec.rs`. |
| `scoped` | `TenantScopedStorage` (einziger Zugriffspunkt, T-ARCH-2): Erzwingt Mandantentrennung, prüft `ReadGate` bei jedem Lese- und Scanvorgang. |
| `wal` | Write-Ahead-Log: `frame` (HMAC-Kette, CRC32, Seq, TxId), `io` (Append, Rotate, Seal), `flusher` (Gruppen-Commit), `replay` (eine kanonische Implementierung). |
| `memtable` | Skip-List mit allen Versionen; `remove_range(&Matcher)` zum rückstandslosen Entfernen aller Versionen bei Purge. |
| `sstable` | Block-AEAD-SSTables (AES-256-GCM-SIV, AAD = `file_id ‖ block_idx ‖ version`), Bloom-Filter, Index, LRU-Blockcache mit Zeroize. |
| `manifest` | Transaktionales Append-Log: `AddFile`, `RemoveFile`, `ReplaceFiles{old,new}`, `SetSeq`. `fsync` je Eintrag. |
| `compaction` | Leveled Compaction; `CompactionFilter` verwirft gegatete Bereiche über den Port `ReadGate`; Purge-`Barrier`. |
| `lsm` | `LsmStorage` implementiert `StorageEngine` und `ErasureLayer`. |
| `erase` | Purge-Ablauf (FI-S20..S27, SPEC §11.4): `plan`, `drain`, `rewrite_file`, `swap`, `unlink_verify`. |

## 4. Kritische Invarianten
1. **WAL-First Consistency (I-1):** Schreibreihenfolge ist verbindlich: WAL-Frame erzeugen → Gruppen-Commit → `sync_data` WAL → Memtable eintragen → Sichtbarkeit im Read-Pfad.
2. **Cancel-Safety (I-14, D-C01):** Commit läuft in eigener Task; Abbruch des Aufrufer-Futures ändert den Ausgang nicht.
3. **Ebene A Purge-Disziplin (SPEC §11.4):**
   - SSTable-Rewrite behält Original-Sequenznummern `seq`, `tx_id` und `flags` für alle Nicht-Ziel-Einträge unverändert bei (I-2).
   - `swap` tauscht Dateien atomar im Manifest (`ReplaceFiles`), synchronisiert Datei und Verzeichnis, invalidiert und nullt betroffene Blockcache-Einträge und löscht die Altdatei mit Verzeichnis-Sync und Fehlen-Prüfung (`Vfs::exists`).
4. **Behebung Sicherheitsbefunde (SPEC §4.2):**
   - `#![allow(unexpected_cfgs)]` und die sieben toten `wal-integrity`-Gates in `wal/replay.rs` werden restlos entfernt.
   - Alle `encryption-at-rest`-Gates entfallen; Verschlüsselung ist fundamentaler Kernbestandteil.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-2.1: Snapshot-Sentinel Fix in Compaction**
  - Korrektur in `store/src/lsm/ops/compaction.rs` (Z. 225–231), sodass keine Tombstone-Bits mehr an `prune_through` übergeben werden.
- **[ ] WP-2.3: WAL-Integrität wiederherstellen & Replay vereinheitlichen**
  - Entfernen von `allow(unexpected_cfgs)`; Vereinheitlichung der doppelten Replay-Logik.
- **[ ] WP-2.4: CRC-Fehler bei Offset 4 lokalisieren (Zeitbox 2 Wochen)**
  - Diagnose und Beseitigung des CRC-Fehlers bei verschlüsseltem WAL-Replay.
- **[ ] WP-2.5: Sicherheitsdefaults umsetzen (D-05, D-07)**
  - Entfernen unverschlüsselter Pfade; `KeySet`-Integration; Mandantentrennung via `TenantScopedStorage`.
- **[ ] WP-2.6: Vfs-Migration (D-04)**
  - Vollständige Umstellung aller Datei- und Verzeichnisoperationen auf den `Vfs`-Port.
- **[ ] WP-2.7: Cancel-sicherer Follower-Commit**
  - Entkopplung des Gruppen-Commits von Client-Futures via `oneshot`.
- **[ ] WP-2.8: Lebenszyklus und Drop-Sicherheit**
  - Geordnetes Schließen aller Hintergrundtasks in `close()`; kein Blockieren in `Drop`.
- **[ ] WP-2.9: Zero-Panic & Determinismus**
  - Entfernung aller `unwrap()`/`expect()` in Produktionspfaden; Umstellung von `HashMap` auf `BTreeMap` im Manifest.
- **[ ] WP-2.10: Crash-Matrix (T-CRASH-1..4)**
  - Validierung gegen Kill- und Stromausfall-Simulationen mit `FaultVfs`.
- **[ ] WP-3.7 & WP-3.8: Purge-Barrier, Drain und SSTable-Rewrite**
  - Exklusive `Barrier`, Memtable-Flush, WAL-Drain und atomarer `swap` via Manifest.

## 6. Verifikation & Benchmarks
- **T-DIFF-1:** Differential-Testing gegen `ReferenceModel` ($10^6$ Operationen).
- **T-CRASH-1..4:** Crash-Matrix ohne Datenverlust committeter Transaktionen.
- **T-CANCEL-1:** Abbruchfestigkeit bei parallelen Commits.
- **Benchmarks:** B-INSERT, B-GET, B-SCAN, B-WAL, B-SST, B-ERASE-WA, B-RECOVER.
