# contextra-vector

HNSW-Vektorsuche, verschlüsselte Snapshots, Slot-Zeroing und Batch-Rebuild.

- **Ring:** Ring 2 (Vektorindexierung)
- **Normativ:** [docs/SPEC.md §8.10](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-vector` implementiert speichereffiziente Vektorindexierung und k-NN-Ähnlichkeitssuche für dichte f32-Einbettungen:
- `arena`: Speicherarena mit Slot-Zeroing im RAM.
- `hnsw`: Hierarchical Navigable Small World Graph (Multi-Layer-Traversierung).
- `docmap`: Zuordnung `doc_tag ↔ slot`.
- `snapshot`: Persistente Chunk-Dateien (AEAD-verschlüsselt mit AES-256-GCM-SIV).
- `rebuild`: Vollständiger Batch-Rebuild ab Überschreitung der Tombstone-Quote (`vector_rebuild_ratio`).
- `layer`: Implementiert den `ErasureLayer`-Port für Vektordaten.

## Invarianten
- `#![forbid(unsafe_code)]` (Distanzberechnungen delegieren an `contextra-simd`).
- Vektoren verlassen den Prozess nur AEAD-verschlüsselt.
- Slots dürfen erst nach Zeroing im RAM **und** Snapshot-Rewrite auf Platte wiederverwendet werden.
- Suchergebnisse enthalten niemals Tombstones.
- Veraltete Module (`acorn`, `diskann`, `quantize*`) sind vollständig entfernt.
