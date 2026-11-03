# AGENTS – contextra-vector

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.10, 9.8, D-18, D-19, D-62, D-77, D-86, D-88, E-L3, E-L15). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** HNSW-Vektorindex mit Arena, löschbaren Slots und verschlüsseltem Snapshot.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`, `simd`, `crypto`. **Nicht** `sys`, **nicht** `mvcc` (D-88). Implementiert `VectorIndex` und `ErasureLayer`.

## Fachprofil (Arbeitsrolle)

ANN-Forscher und -Ingenieur (HNSW nach Malkov/Yashunin, DiskANN/FreshDiskANN, gefilterte Suche wie ACORN). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Graphkonnektivität und Recall unter Löschung: Tombstones, Nachbarschaftsreparatur (Nachbarn gelöschter Knoten verbinden, nicht nur entfernen), Wahl des Einstiegsknotens, Stapelreparatur im Wartungsfenster.
- Surrogat-Injektion: ein Platzhalter, der die Navigation erhält, aber keinen Vektorinhalt des Ziels trägt; Zeroizing aller Repräsentationen; Rekonstruierbarkeit des Originals aus Nachbarn bewerten und im Bericht ausweisen.
- Recall- und Latenzmessung mit Ground Truth, deterministischer Aufbau (Seed über `Rng`-Port, feste Reihenfolge).

## Normative Anker

- **Parameter (9.8):** `M = 16`, `efConstruction = 200`, Standard `efSearch = 64`; Dimension ≤ 4096 (D-87).
- **Löschen ist kein Tombstone-Bit (D-77):** (1) **Vor `Gated`**: Arena-Vektoren der Zielslots werden in allen Repräsentationen durch ein Navigationssurrogat ersetzt (Mittel der Vektoren der nicht gegateten Nachbarn; ohne solche: Vektor des Einstiegsknotens); kein Zielvektor-Byte bleibt (T-VEC-DEL-1). (2) **Im Purge-Fenster**: Batch-Kantenreparatur in einem linearen Durchlauf; danach keine Referenz auf Zielslots, Slots in die Freiliste (T-VEC-DEL-2). (3) **Deterministischer Rebuild** (Einfügung in `doc_tag`-Reihenfolge) bei Tombstone-Quote ≥ `vector_rebuild_ratio` (0,2) oder ≥ 10 % reparaturgelöschter Slots; `vector_strict_rebuild` (Standard aus). Der Graph nach Rebuild hängt nur von der Menge der lebenden Slots ab (T-VEC-REBUILD-INDEP).
- **Arena (D-86):** je Slot ein zusammenhängender, zeroisierbarer Bereich mit **allen** Repräsentationen; keine Repräsentation außerhalb der Arena (T-VEC-ARENA-1). `MADV_DONTDUMP` und `mlock` best effort über einen Port aus `ports`, implementiert in `engine` (D-63, D-88).
- **Snapshot (D-19, F.8):** je Slot ein Envelope unter dem DEK des Dokuments mit Vektor **und** Bindung `(cluster_tag, doc_tag, chunk)`; Kanten unter dem Dateischlüssel (D-85); nur über `Vfs` (D-94). Pfad `vector/<tenant_tag>/<coll_tag>/…` (D-75).
- **Schlüsselreichweite (D-69):** `vector_payload = full`, `vector_edges = none` bis zum Rebuild; Restrisiko `IndexTopologyInference` (E-L3, E-L15). **Vor einer Löschaussage für diese Schicht müssen Zeroizing und Reparatur gemessen belegt sein; bis dahin steht die Schicht im Bericht als „nicht erfasst“.**
- **Suche (D-62):** gegatete Kandidaten verwerfen; `ef` verdoppelt sich ab `max(ef, k)` bis `ef_max = 4096`; `SearchPage.exhausted` ist ehrlich. Metadatenfilter bleiben in v1 nachgelagert.

## Verboten

`sys` und `mvcc` als Abhängigkeit, `std::fs`, `memmap2`/`mmap`, Nullvektoren als Löschung, Quantisierungs-Nebenarrays, die Features `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `partial-index-rebuild`, `graph` (D-84), regionales HNSW-Rewiring (VETO-F02), nicht deterministischer Aufbau.

## Abnahme

`T-VEC-DEL-1`, `T-VEC-DEL-2`, `T-VEC-REBUILD-INDEP`, `T-VEC-ARENA-1`, `T-RAW-3`, `T-KEYCOV-1`; `BM-VEC` (Recall@10 ≥ 0,95, p99 ≤ 15 ms), `BM-VEC-DEL` (Recall ≥ 0,95 nach 5/10/20 % Löschung, Reparaturdurchlauf ≤ 60 s je 10⁶ Knoten), `BM-VEC-REBUILD`. Der Umfang außerhalb des Soll (DiskANN, Quantisierung, partielle Rebuilds; F-36) wird zurückgebaut, nicht gepflegt.

## Arbeitspakete

WP-0.3 und WP-1.4 (Build, ohne `std::fs`/`mmap`), WP-8.1 (Arena, Surrogat, Reparatur, Rebuild, Snapshot). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-vector --locked
cargo test --locked --test tk_conformance_vector_index
cargo test -p contextra-arch-tests --locked
```
