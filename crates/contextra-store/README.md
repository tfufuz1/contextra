# contextra-store

Transaktionaler LSM-Tree, WAL-Durability, Leveled SSTables und Purge-Primitive.

- **Ring:** Ring 2 (Speicherschicht)
- **Normativ:** [docs/SPEC.md §8.8](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-store` ist der blinde, transaktionale Key-Value-Speicherkern von Contextra. Er operiert ausschließlich auf `KeyBuf`, `Seq`, opaken Bytes und `Matcher`:
- `wal`: Write-Ahead-Log mit HMAC-SHA256-Kette, CRC32 und Gruppen-Commit.
- `memtable`: Skip-List mit allen Versionen; `remove_range` für Purge.
- `sstable`: Block-AEAD-SSTables (AES-256-GCM-SIV), Index, Bloom-Filter, LRU-Cache mit Zeroize.
- `manifest`: Append-Log für atomaren Dateitausch (`ReplaceFiles{old, new}`).
- `erase`: Purge-Ablauf (FI-S20..S27, SPEC §11.4): `plan`, `drain`, `rewrite_file`, `swap`, `unlink_verify`.
- `scoped`: `TenantScopedStorage` erzwingt Mandantentrennung und `ReadGate`-Prüfung.

## Invarianten
- `#![forbid(unsafe_code)]`
- I/O ausschließlich über den `Vfs`-Port (`durable-fs` wird nicht direkt referenziert).
- WAL-First Consistency (I-1): Frame(s) → Gruppen-Commit → `sync_data` WAL → Memtable → Sichtbarkeit.
- Follower-Commits sind cancel-sicher (I-14, D-C01).
- SSTable-Rewrite behält Original-Sequenzen (`seq`, `tx_id`, `flags`) bei (I-2).
- Pflichtverschlüsselung ohne Opt-Out (D-05).
