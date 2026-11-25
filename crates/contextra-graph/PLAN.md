# Implementierungsplan: contextra-graph

**Ebene:** L2 · **Ziel-Zustand:** CSR-Graph, Hyperkanten, PPR und PathRAG. Persistenzneutral. Keine Session-Zustände, kein Verstärkungslernen, keine Löschlogik außerhalb von `rewrite_without`.
**Basis:** SPEC R14 §2.1, §2.7, §11.8, §11.9, NZ-11, NZ-22, DEC-16, DEC-24, §13.3 (`session_dag`, `edge_reinforcement`, `GraphPurger` gelöscht), R1, R3, Z-17; R15 §2.3.

## 1. Ist-Befund (gemessen)

- 77 Quelldateien, 22 406 Zeilen. Größtes Crate nach Zeilen mit Ausnahme von `engine`.
- Module: `apprh`, `arc_slice`, `cascade` (1092), `community`, `consistency_enforcement`, `csr`, `edge_reinforcement`, `edge_reinforcement_buffer`, `entity_extraction`, `error`, `hyperedge` (796), `hyperedge_suggest`, `path_rag`, `percolation`, `ppr`, `ppr_stream`, `provenance`, `provider`, `purger`, `session_dag` (1008), `test_doubles`, `tl_hfd`, `types`, `contract`.
- **219 Treffer verbotener Muster im Produktionscode** (höchster Wert nach `engine`).
- `Cargo.toml`: `bincode`, `blake3`, `scc`, `arc-swap`, `serde`, `serde_json`, `parking_lot`, `tracing`. Laut §2.7 verboten: `bincode`, `blake3`, `scc`, `serde`, `parking_lot`.
- Verweis `purger.rs` auf nicht existierendes `contextra-erasure-spi` (R14 §13.2 Nr. 9). Ersatz: `PartitionSink::rewrite_without`.
- `edge_reinforcement` und `edge_reinforcement_buffer` sind Lernlogik (NZ-22, §18: `edge-reinforcement-learning` nicht freigegeben).
- `session_dag` ist Session-Zustand (NZ-22).
- `entity_extraction`: Extraktion gehört zu `ingest` oder ist ein Port (`TextGenerator`), nicht in den Graph.
- `provider` und `provenance`: Herkunftsangaben. Prüfen, ob personenbezogene Information enthalten, und Verbindung zu §11.13 (Person → Cluster) klären.

## 2. Entfernen

| Ziel | Begründung |
|---|---|
| `session_dag.rs` | NZ-22, §13.3 |
| `edge_reinforcement.rs`, `edge_reinforcement_buffer.rs` | NZ-22, §18 (nicht freigegeben) |
| `purger.rs` | ersetzt durch `PartitionSink::rewrite_without` (R14 §13.2 Nr. 9) |
| `entity_extraction.rs` | Verantwortung von `ingest`/`TextGenerator` (NZ-23) |
| `test_doubles.rs` in Produktionsbuild | nur `#[cfg(test)]` |
| `scc`, `arc-swap`, `blake3`, `bincode`, `serde`, `serde_json`, `parking_lot` | verboten bzw. nicht nötig |
| `block_on` in `benches/*` und `fuzz/*` | in Benchmarks zulässig, aber Produktionscode muss frei sein |

**Vor Löschung:** `grep -rn 'session_dag\|edge_reinforcement\|GraphPurger' crates/` über alle Crates, insbesondere `engine` (dort gibt es 44 Dateien mit verwandten Namen).

## 3. Ändern

- **`cascade.rs` (1092):** Kaskadenlogik auf Gate-Frame-Semantik (`INV-ERA-12`) umstellen. Die Kaskade berechnet nur die Menge der abgeleiteten Cluster; das Schreiben übernimmt `storage`. Eine Quelle löscht den Abgeleiteten im selben Gate-Frame.
- **`hyperedge.rs` (796):** Hyperkanten dürfen nicht über Löschdomänen hinweg bestehen (NZ-11, §11.8). Kanten mit Knoten aus mehreren Clustern nur zulässig, wenn alle Cluster in derselben Löschdomäne liegen.
- **`csr/`:** CSR-Format als Zwei-Array-Struktur (Offsets, Targets). Persistenz über `PartitionSink`. Tests in `csr/tests/persistence_tests.rs` auf Sink umstellen.
- **`ppr/` und `ppr_stream`:** PPR mit Zeitbudget und Knotenlimit. Ergebnis nie als Cluster-übergreifender Cache persistieren.
- **`community/`:** Community-Erkennung nur innerhalb einer Partition. Ergebnis ist abgeleitet und wird bei Löschung verworfen.
- **`percolation`, `tl_hfd`:** Prüfen, ob Experimentalcode. Wenn ja, hinter Feature; wenn nicht in §11.8 genannt, entfernen.
- **`arc_slice`, `apprh`:** Hilfsmodule. Nach Verwendung prüfen; `arc_slice` mit `arc-swap` verboten, falls nur dafür genutzt.
- **`contract.rs`:** `PartitionSink`/`PartitionSource`, `GraphIndex` (nur lesend).

## 4. Implementieren

1. **Knoten- und Kantenmodell** mit `ClusterId` je Knoten, Partition je Knoten-Gruppe (DEC-21).
2. **CSR** mit Generationsschlüssel; Rebuild über Sink, nicht in-place.
3. **Traversierung** `traverse*`:
   - Knotenlimit; bei Überschreitung `EraseFanoutExceeded` ohne Mutation (§8.8 Links).
   - Keine E/A auf dem Lesepfad (Z-17), Daten kommen aus Snapshot-Generation.
4. **PPR:** Power Iteration mit Toleranz und Maximalzahl an Iterationen; Ergebnis deterministisch bei gleicher Eingabe.
5. **PathRAG:** Pfadsuche mit Tiefenlimit (`max_link_depth`), Ranking nach Pfadgewicht; Ausgabe mit Provenienz (`provenance`) je Kante.
6. **Hyperkanten-Vorschläge** (`hyperedge_suggest`): nur Vorschläge, nie automatische Persistenz.
7. **`rewrite_without(clusters)`:** entfernt Knoten und Kanten der Cluster, berechnet Community-Ergebnisse neu oder verwirft sie.
8. **Konformanz** `SegmentOwner` für Graph (R2).

## 5. Tests

- **Differential:** `Index ∪ Delta ≡ Vollscan` für Traversierung (G-R3).
- **Fanout-Test:** 4 097 Knoten ⇒ `EraseFanoutExceeded`, Graph unverändert.
- **Löschprobe:** nach `rewrite_without` findet `verify_absent` keinen Knoten und keine Kante des Clusters.
- **Kaskadentest:** Löschung einer Quelle zieht den Abgeleiteten im selben Frame mit.
- **Determinismus-Test:** PPR bei gleicher Eingabe liefert identische Bits.
- **Fuzz-Ziel** `fuzz_graph_persist` (existiert) mit Ergebnisprüfung.
- Bestehende Tests in `csr/tests/` (781, 820 Zeilen), `ppr/tests/` (846, 1193), `community/tests.rs` (827) bleiben; nur Bezüge auf gelöschte Module werden angepasst.

## 6. Abhängigkeiten nach Umbau

Erlaubt: `contextra-ports` (`retrieval`), `contextra-types`, `bytes`, `ahash`, `thiserror`, `tracing` (optional).
Verboten: `bincode`, `blake3`, `scc`, `arc-swap`, `serde`, `serde_json`, `parking_lot`, `tokio`, `block_on`, `std::fs`.

## 7. Gate und Reihenfolge

1. Löschen: `session_dag`, `edge_reinforcement*`, `purger`, `entity_extraction`. Konsumenten in `engine` vorher prüfen.
2. Abhängigkeiten bereinigen, `scc`/`arc-swap` ersetzen durch `std::sync`.
3. `contract.rs` mit Sink und Source.
4. CSR-Persistenz, dann Traversierung, dann PPR und PathRAG.
5. `cascade.rs` auf Gate-Frame-Semantik.
6. **G-R1**, **G-R3**.

**Risiko:** `cascade.rs` ist die Stelle, an der Löschung über Graphkanten entschieden wird. Fehler dort verletzen `INV-ERA-12` und sind schwer zu finden. Eigene Modelltests vor Umbau schreiben.
