# Implementierungsplan: contextra-vector

**Ebene:** L2 · **Ziel-Zustand:** HNSW, DiskANN, SQ8, RaBitQ, ACORN. Persistenzneutral: keine Dateien, keine Schlüssel, keine Zeitquelle. Persistenz über `PartitionSink`/`PartitionSource` aus dem Modul `contract`.
**Basis:** SPEC R14 §2.1, §2.7, §11.5, §11.6, §16, DEC-23, DEC-24, FIX-12, FIX-19, R1, R2; R15 §2.3.

## 1. Ist-Befund (gemessen)

- 52 Quelldateien, 16 903 Zeilen (das größte Retrieval-Crate). Module: `acorn`, `candidate_stream`, `casting`, `compute_pool`, `contract`, `diskann` (Verzeichnis), `distance`, `error`, `fixed_hasher`, `hnsw` (Verzeichnis), `partial_rebuild`, `persistence` (Verzeichnis), `quantize` (Verzeichnis), `quantize_rabitq`, `types`.
- **112 Treffer verbotener Muster im Produktionscode** (`memmap2`, `blake3`, `bincode`, `std::fs`, `tokio`, `DISKANN_INTEGRITY_KEY`).
- `Cargo.toml`: `memmap2` ist Standard, `blake3`, `parking_lot`, `rand`, `roaring`, `zeroize`, `contextra-sys` optional.
- Features: `experimental` aktiviert `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`. `default = ["std"]`.
- `contract.rs` existiert, enthält laut Befund aber eigene Persistenz-Typen. Inhalt vor Umbau lesen.
- `TxId` in `types.rs:97`.
- Verweis auf `contextra-sys` (`mmap`-Feature) bleibt nur für `mmap`-Pfad nötig, und der ist laut NZ-9 unzulässig, außer unversiegelt in Indizes; Entscheidung: nicht verwenden.

## 2. Entfernen

| Ziel | Begründung |
|---|---|
| `memmap2` in `Cargo.toml` und Code | NZ-9, R14 §13.3 |
| `std::fs`-Zugriffe in `persistence/`, `diskann/`, `hnsw/` | Persistenzneutralität (R1) |
| `DISKANN_INTEGRITY_KEY` und jeder feste Schlüssel | verboten (§13.3); AEAD unter Generationsschlüssel |
| `blake3`-Inhalts-Hashes | nur keyed, über `PartitionSink` |
| `bincode`, `tokio`, `scc`, `block_on` (wo vorhanden) | verboten in L2 |
| `TxId` in `types.rs` | ersetzt durch `Epoch` |
| `contextra-sys`-Abhängigkeit | nicht nötig nach Entfernung von `mmap` |
| Feature `mmap` (leer) | entfernen |
| Feature `inspect`, `graph`, `docid-128`, falls ungenutzt | nach `grep` entscheiden |

## 3. Ändern

- **`contract.rs`:** Vertrag der Persistenz:
  - `PartitionSink`: `write_block(block)`, `seal_generation(g)`, `apply(batch, epoch)`, `rewrite_without(cluster_set)`.
  - `PartitionSource`: `read_block(id)`, `list_generations()`.
  - Trait-Objekte werden von `engine::projection` implementiert.
- **`hnsw/vector_index_impl.rs` (826):** Schreibpfad über `PartitionSink`, Lesepfad über `search_at(epoch)`.
- **`hnsw/types.rs` (972), `hnsw/core_rebuild.rs` (1202):** Rebuild-Schritte in Blöcke zerlegen, die vom Sink geschrieben werden. Rebuild-Kosten begrenzen (`index_rebuild_bytes`).
- **`diskann/build.rs` (817):** Build über Sink; Ergebnis ist eine versiegelte Generation.
- **`quantize/` und `quantize_rabitq`:** SQ8 und RaBitQ-Codes sind **abgeleitet** (DEC-23). Sie werden aus dem kanonischen F16-Vektor erzeugt, nicht gespeichert als alleinige Wahrheit.
- **`distance`:** nutzt `DistanceFn` aus `contract`; Implementierung kommt aus `contextra-simd` (Injektion durch `engine`).
- **`fixed_hasher.rs`:** prüfen, ob Hash-Funktion nicht-keyed ist. Wenn ja, für Index-Tabellen OK (keine Inhaltsabhängigkeit nach außen), sonst ersetzen.

## 4. Implementieren

1. **Modi A/B/C** (DEC-24, FIX-19), über Konfiguration:
   - **A (Mikroindex):** bis `micro_index_max_vectors` 50 000, je Cluster.
   - **B (Partitionsindex):** Packgruppe = Partition, bis `max_partition_vectors` 500 000.
   - **C (Cluster-Index):** bis `cluster_index_max_vectors` 2 Mio., für personenbezogene Akten.
   - Größere Akten: `Limit{ClusterIndexVectors}` mit `hint()` (Risiko 14).
2. **Kanonischer Vektor F16** (`emb_precision`), F32 optional (DEC-23).
3. **Generationsschlüssel** je Partitionsgeneration (`PartitionGeneration`-Scope, §4.9).
4. **Zeitreise:** `search_at(epoch)` liest nur gepinnte Generationen (DEC-28). Erase überstimmt.
5. **`rewrite_without(clusters)`:** erzeugt neue Generation ohne die Vektoren der Cluster; Ergebnis `RewriteOutcome`, **kein** Löschbeweis (FIX-05). Der Beweis kommt aus `EraseAttestation` (§11.2).
6. **ACORN-Filter:** Selektivitätsschwelle `filter_selectivity` 5 %. Darunter Vollscan über begrenzte Menge (`scan_fallback_max_vectors` 1 Mio.).
7. **Konformanzsuite** `SegmentOwner` (R2) für Vektor.
8. **Rerank:** `rerank_depth` = 4 × k, gegen F16-Original, nicht gegen Codes (FIX-12).

## 5. Tests

- **Recall-Messung** gegen Brute-Force (`H-xx` aus §16), mit Streuung und Hardware-Angabe.
- **Löschprobe:** nach `rewrite_without` findet `verify_absent` keinen Vektor des gelöschten Clusters.
- **Zeitreise-Test:** `search_at(e)` für gepinnte `e` und nach Erase.
- **Rebuild-Kosten-Test:** Rebuild bleibt unter `index_rebuild_bytes`.
- **Fuzz-Ziel** `fuzz_hnsw_persistence` (existiert bereits in `fuzz/fuzz_targets/`) und `hnsw_insert_search`: Ergebnis prüfen, nicht nur Absturzfreiheit.
- Benchmarks (`hnsw_bench`, `diskann_build`, …) bleiben, werden in den Messbericht aufgenommen.

## 6. Abhängigkeiten nach Umbau

Erlaubt: `contextra-ports` (`retrieval`), `contextra-types`, `ahash`, `roaring`, `subtle`, `zeroize`, `rand` (nur Tests und Konstruktion, nicht Entropiequelle), `thiserror`, `parking_lot` (nur wenn zwingend; sonst `std::sync`).
Verboten: `memmap2`, `blake3` (unkeyed), `bincode`, `tokio`, `scc`, `std::fs`, `block_on`, `contextra-sys`.

## 7. Gate und Reihenfolge

1. Feature `mmap` und `memmap2` entfernen.
2. `contract.rs` neu aufsetzen; `PartitionSink` und `PartitionSource`.
3. Schreibpfad von HNSW und DiskANN auf den Sink umstellen; Tests anpassen.
4. Modi A/B/C.
5. `rewrite_without` und `verify_absent`-Schnittstelle.
6. **G-R1:** `check_forbidden_apis.sh` ohne Treffer in `vector`. **G-R2:** Konformanz, `O-Segments`/`O-Files` ohne Treffer. **G-R3:** Differential `Index ∪ Delta ≡ Vollscan`.

**Risiko:** `DISKANN_INTEGRITY_KEY` ist ein Schlüssel im Code. Das ist ein Produktionsfehler, nicht nur Refactoring-Arbeit; er wird vor allen anderen Änderungen entfernt.
