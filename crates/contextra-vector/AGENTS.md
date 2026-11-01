# AGENTS.md — `contextra-vector`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.10](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-vector` stellt die HNSW-Vektorsuche für dichte Einbettungen (f32) bereit. Es garantiert vollständige physische Löschbarkeit über Slot-Zeroing im Speicher, verschlüsselte Snapshot-Chunks auf Festplatte und periodischen Batch-Rebuild.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 2 (Vektorindexierung)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-simd`, `contextra-crypto`
- **Verbotene Abhängigkeiten:** Keine Abhängigkeit zu `contextra-store`, `contextra-text` oder `contextra-engine`.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`; Distanzberechnungen delegieren an `contextra-simd`).
  - Keine Klartext-Vektoren auf persistenten Speichermedien.
  - Keine direkte Zeitmessung (`Instant::now` ist verboten; `Clock`-Port nutzen).

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.10)

| Modul | Inhalt & Verantwortung |
|---|---|
| `arena` | Speicherarena für f32-Vektorslots fester Dimension; sicheres Überschreiben bei Freigabe. |
| `hnsw` | Hierarchical Navigable Small World Graph: Aufbau, Multi-Layer-Traversierung, k-NN-Suche. Altes `deletion.rs` und Ad-hoc-Purger entfallen. |
| `docmap` | Bidirektionale Zuordnung `doc_tag ↔ slot` für gezieltes Adressieren bei Löschvorgängen. |
| `snapshot` | Persistente, verschlüsselte Snapshot-Chunks (SPEC §9.8) unter Verwendung von AES-256-GCM-SIV. |
| `rebuild` | Vollständiger Batch-Graph-Rebuild bei Überschreiten der Tombstone-Schwelle (`vector_rebuild_ratio`). |
| `layer` | Implementierung des `ErasureLayer`-Ports für Vektorindizes. |
| `distance` | Abstraktion der Distanzfunktionen, delegiert an SIMD-Kernel in `contextra-simd`. |

> [!IMPORTANT]
> **Bereinigung veralteter Module (SPEC §8.10):**
> Die Module `acorn`, `diskann`, `quantize*`, `partial_rebuild`, `candidate_stream` und `compute_pool` sind nicht Teil des v1-Kerns und werden vollständig entfernt.

## 4. Kritische Invarianten
1. **Zweistufige Slot-Bereinigung:**
   - Beim Erase eines Dokuments überschreibt `zero_slot` die Slot-Bytes im RAM mittels `zeroize`, setzt ein Tombstone-Bit und entfernt den Eintrag aus der `DocMap`.
   - Ein Slot darf **erst dann** wiederverwendet werden, wenn sowohl das RAM-Zeroing als auch der Rewrite der Snapshot-Chunks auf der Festplatte erfolgreich abgeschlossen sind.
2. **Keine Tombstones in Ergebnissen:** Vektorsuchen filtern Tombstone-Slots strikt heraus.
3. **Persistenz-Verschlüsselung:** Vektordaten werden auf dem Datenträger ausnahmslos als AEAD-verschlüsselte Chunk-Snapshots abgelegt.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-1.2: Workspace-Bereinigung & Entfernen toter Module**
  - Rückbau von `diskann`, `acorn`, `quantize` und Hilfspools.
- **[ ] WP-4.2: HNSW Slot-Zeroing & Batch-Rebuild (D-18, D-19)**
  - Implementierung von `zero_slot` mit Zeroize.
  - Implementierung des `ErasureLayer`-Ports für `contextra-vector`.
  - Batch-Rebuild-Trigger basierend auf `vector_rebuild_ratio`.
- **[ ] WP-4.3: Snapshot-Rewrite & Chunk-Verschlüsselung**
  - Persistierung und Rewrite von Chunk-Dateien ohne gelöschte Slots.

## 6. Verifikation & Benchmarks
- **T-VEC-1:** Recall@10 $\ge 0.95$ im Vergleich zur exakten Skalar-Brute-Force-Suche.
- **T-VEC-2:** Löschverifikation: Slot genullt, Chunk-Datei neu geschrieben, Rohscan findet keine Vektorbytes.
- **T-DOCMAP-1:** Konsistenzprüfung zwischen `DocMap`, Graph und Arena nach Rebuild.
- **Benchmarks:** B-VEC-BUILD, B-VEC-SEARCH, B-VEC-ERASE.
