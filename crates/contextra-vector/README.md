# contextra-vector

Vektorsuche und Indexierung (HNSW, DiskANN, SQ8 Quantisierung).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, §9.2, §9.3, `capabilities.toml`).

## Rolle in einem Satz
`contextra-vector` implementiert speichereffiziente Vektorindexierung und Ähnlichkeitssuche (HNSW, SQ8, ACL-Bitset) mit Mandantenisolation und Tombstone-Pruning.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Vektorindexierung)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-simd` (Spec §6.9, `capabilities.toml`)

*Sicherheits & Lösch-Invarianten:* VETO-F02 (Tombstone-Pruning ohne kryptographische Löschbeweise), INV-TENANT-IDX-1 (mandantenisolierte Vektorräume).
