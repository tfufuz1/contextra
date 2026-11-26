# Spezifikation: **Mnemosyne** – Hybride Rust-Datenbank-Engine für KI-Agenten und RAG

**Arbeitstitel:** Mnemosyne  
**Version:** 1.0 – Architektur- und Systemspezifikation  
**Annahmen:** Embedded Single-Node-Engine mit optionaler Content-Addressable-Object-Store-Replikation; Mehrbenutzerbetrieb über Transaktionen; Fokus auf lokale NVMe/SSD, `io_uring`-fähige Linux-Systeme.  
**Nicht-Ziele:** Vollständig verteilter Multi-Writer-Cluster, SQL-Vollständigkeit, exakte Garantie semantischer Wahrheit von LLM-Ausgaben.

---

## 0. Logisches Datenmodell und Grundbegriffe

Mnemosyne speichert alle Modelle in einem gemeinsamen, kanonischen Record-Modell. Indizes sind abgeleitete, unveränderliche Projektionen – keine duplizierten Primärdaten.

**Kernobjekte:**

| Objekt | Bedeutung |
|---|---|
| `RecordId` | Stabile 128-Bit-ID eines logischen Datensatzes |
| `LogicalKey` | Fachlicher Schlüssel, z. B. `entity:person:42`, `edge:works_at:7`, `chunk:doc:99:3` |
| `Record` | Kanonischer Datensatz mit Typ, Properties, optionalem Dense-Vektor, Textpayload, Provenance, Temporalität |
| `Commit` | Logischer Versionsknoten im Merkle-DAG |
| `Manifest` | Physische Sicht eines Commits: Segmentliste, Index-Watermarks, Statistiken |
| `Branch` | Benannter Ref auf `(commit_id, manifest_id)` |
| `Segment` | Unveränderliche physische Datei bzw. Dateigruppe |
| `ProvenanceNode` | Kryptografischer Herkunftsnachweis für Record, Chunk oder Kante |
| `ContextPackage` | Token-optimiertes Query-Ergebnis für LLM-Konsum |

Jeder Record ist **bitemporal**: `valid_from`/`valid_to` für fachliche Gültigkeit, `tx_time`/`commit_id` für Systemzeit. Löschungen sind Tombstones.

---

## 1. High-Level-Architektur & Schichtenmodell

### 1.1 Schichten

```text
┌─────────────────────────────────────────────────────────────┐
│ Client/API: Rust SDK, gRPC/HTTP, Query-DSL, Agent-Session-API│
├─────────────────────────────────────────────────────────────┤
│ Query Engine: Parser, Token-aware Planner, Executor, Fusion │
├─────────────────────────────────────────────────────────────┤
│ Transaction & Versioning: MVCC, Commit-Graph, Branch/Merge  │
├─────────────────────────────────────────────────────────────┤
│ Storage Engine: WAL, Memtable, Segment Store, Manifest/DAG  │
├─────────────────────────────────────────────────────────────┤
│ Index Services: Vector (HNSW/DiskANN), Graph (CSR), Text    │
├─────────────────────────────────────────────────────────────┤
│ Cache/Buffer Pool, Async I/O, Durability, Recovery          │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Write-Path

1. Client startet Transaktion auf einem Branch mit Read-Snapshot `(commit_id, manifest_id)`.
2. Writes landen im Transaktionspuffer und werden als logische Deltas an den WAL angehängt: `tx_id`, `branch`, `parent_commit`, `record_delta`, `provenance_delta`, `schema_hash`, Checksum.
3. Bei `commit`:
   - Optimistische Konfliktprüfung gegen den Branch-Head.
   - Optional: synchrone oder asynchrone Konflikterkennung für Fakten.
   - WAL-`fsync` per Group Commit.
   - Delta in Memtable übernehmen.
   - Neuer `Commit`-Knoten: `H(parents, delta_root, schema_hash, policy_hash)`.
   - Branch-Ref atomar auf neuen Commit zeigen lassen; Verzeichnis-`fsync`.
4. Background-Worker flushen Memtables zu unveränderlichen Segmenten, bauen Vektor-/Graph-/Text-Indizes und erzeugen ein neues `Manifest` für denselben logischen Commit.
5. Query-Snapshots pinnen `(commit_id, manifest_id)`. Indexe sind nur nutzbar, wenn ihr Watermark ≤ Snapshot-Watermark ist; sonst exakter Fallback oder Scan.

### 1.3 Read-Path

1. Branch auflösen → Commit + Manifest pinnen.
2. Query parsen; Planner wählt Zugriffspfad: ANN, BM25, Graph-Traversal, Filter-Scan, RRF-Fusion.
3. Executor arbeitet ausschließlich auf dem gepinnten Snapshot.
4. Ergebnisse werden über `RecordId` vereinigt, dedupliziert, mit Provenance angereichert und token-optimiert.
5. Rückgabe als `ContextPackage` oder klassisches Resultset.

### 1.4 Wichtige Invarianten

- **Unveränderlichkeit:** Segmente und Commits werden nie in-place mutiert.
- **Single Source of Truth:** Kanonische Records liegen in Row-/Columnar-Segmenten; alle Indizes referenzieren nur `RecordId` plus abgeleitete Statistiken.
- **Snapshot-Konsistenz:** Jede Query sieht genau einen Commit und ein Manifest.
- **Tamper-Evidenz:** Commit-DAG und Provenance sind Merkle-basiert; physische Manifeste besitzen eigene Merkle-Roots.
- **Lock-free Reads:** Lesezugriffe benötigen keine globalen Locks; Ref-Updates laufen über `ArcSwap` und Epoch-Reclamation.

---

## 2. Physisches Storage- & Index-Design

### 2.1 Kanonische Speicherung

Die Engine nutzt einen **hybriden LSM-/CoW-Ansatz**: Writes gehen über WAL + Memtable; Flushes erzeugen unveränderliche, columnar orientierte Segmente. Metadaten und Branch-Refs liegen in CoW-Manifesten.

**Segmenttypen:**

| Segment | Inhalt |
|---|---|
| `RowSegment` | Kanonische Records: `RecordId`, `LogicalKey`, Typ, Properties, Temporalität, Tombstone |
| `VectorColumnSegment` | Dense Vectors als `f32`/`f16`-Arrays, spaltenweise, Arrow-kompatibel |
| `GraphAdjacencySegment` | CSR-Out-/In-Adjazenz, Edge-IDs, Property-Spalten |
| `TextPostingSegment` | Invertierter Index: Term → Postings mit `RecordId`, TF, Positionen, Feld |
| `ProvenanceSegment` | Provenance-Knoten, Hashes, Signaturen, Confidence |
| `StatsSegment` | Zonenmaps, Bloom-Filter, BM25-Statistiken, Vektor-Dimensionen |

Vektoren werden **nicht** im HNSW-Index dupliziert. Der HNSW-Knoten speichert `RecordId` + Offset in die mmap-bare `VectorColumnSegment`. DiskANN-PQ-Codes sind abgeleitete Kompressionsstrukturen, keine kanonischen Daten.

### 2.2 Vektorindex

- **HNSW:** In-Memory-/mmap-Graph mit Levels; Knoten = `RecordId`, Offset, Level, Nachbarlisten. Updates erzeugen Delta-HNSW-Segmente; Tombstones markieren Löschungen.
- **DiskANN:** Für große, RAM-limierte Workloads; Vamana-Graph + PQ-Codes; Zugriff über mmap und Buffer Pool.
- **Exakter Fallback:** Für kleine Kandidatenmengen oder wenn Index-Watermark nicht zum Snapshot passt.
- **Hybrid-Filter:** Graph- und Text-Filter liefern Kandidatenmengen; ANN-Suche kann gefiltert oder nachgelagert angewendet werden. Der Planner entscheidet kostenbasiert.

### 2.3 Graphindex

- Knoten und Kanten sind normale Records.
- Pro Segment: CSR-Adjazenz für Out- und In-Edges, sortiert nach `EdgeType` und `RecordId`.
- Multi-Hop-Traversal merged CSR-Segmente + Memtable-Delta; Tombstones werden übersprungen.
- Kanten tragen Provenance und Confidence; Traversal kann `confidence >= x` und `valid_at` filtern.
- Entity-Resolution ist ein separater Index: normalisierte Namen/Aliase → `LogicalKey`.

### 2.4 Volltextindex

- Tokenisierung: Unicode-Segmentierung, optional Stemming, Language-Packs.
- Postings pro Segment: `term -> [(RecordId, tf, positions, field)]`.
- BM25-Statistiken segmentlokal; globale IDF/AvgDL im Manifest.
- Phrasen- und Nähe-Queries über Positionslisten.
- Text- und Graph-Filter teilen sich `RecordId`-Mengen; keine Payload-Duplikation.

### 2.5 Hybride Query-Ausführung

Beispiel-DSL:

```text
HYBRID(
  vector: ANN(embedding, k=80, ef=200),
  text: BM25("rust storage engine", k=80),
  graph: TRAVERSE(start=entity:person:42, depth<=2, edge_types=[works_at, knows]),
  filter: valid_at(now) AND confidence >= 0.7,
  fusion: RRF(k=60),
  budget_tokens: 8000
)
```

Ausführung:

1. Gemeinsamer Snapshot.
2. Kandidatenmengen aus ANN, BM25, Graph-Traversal.
3. Filterung über kanonische Records und Temporalität.
4. Fusion via Reciprocal Rank Fusion: `score = Σ 1/(k + rank_i)`.
5. Dedup über `LogicalKey` und semantische Ähnlichkeit.
6. Token-aware Re-Packing.
7. Rückgabe mit Provenance-Inclusion-Proofs.

### 2.6 Buffer Pool, Caching, I/O

- `mmap` für immutable Segmente; Buffer Pool für Hot Pages und Index-Knoten.
- Bloom-Filter und Zonenmaps pro Segment für Pruning.
- `Arc<Segment>`-Lebenszeiten; Epoch-basierte Reclamation für Reader.
- Async I/O: Netzwerk über `tokio`; Storage-Worker über `io_uring`-fähige Runtime (`monoio`/`glommio`) oder `tokio` mit Blocking-Pool als Fallback.
- Zero-Copy: Arrow IPC/Parquet für Columnar Segmente; `rkyv` für WAL- und Manifest-Metadaten.

### 2.7 WAL, Memtable, Durability

- WAL ist append-only, segmentiert, mit CRC32C und Record-Framing.
- Group Commit: Mehrere Transaktionen teilen einen `fsync`.
- Memtable: sortierte In-Memory-Struktur pro Branch/Snapshot; Flush erzeugt neue Segmente.
- Checkpoints: Manifest + letzter WAL-LSN; Recovery replayt WAL ab Checkpoint, verifiziert Merkle-Roots und baut Memtable neu auf.
- Torn-Write-Erkennung über Checksums; atomare Ref-Updates via Temp-Datei + `rename` + Verzeichnis-`fsync`.

---

## 3. Versionierungs- & Concurrency-Engine

### 3.1 Commit-DAG und Manifeste

- **Commit:** logischer Versionsknoten. Hash = `Blake3(parents, delta_root, schema_hash, policy_hash)`.
- **Manifest:** physische Sicht. Hash = `Blake3(segment_hashes, index_watermarks, stats_root)`.
- **Branch-Ref:** `{commit_id, manifest_id}`. Branching ist O(1): neuer Ref auf existierenden Commit.
- **Compaction** erzeugt neue Segmente und ein neues Manifest für denselben logischen Commit; `commit_id` bleibt stabil, `manifest_id` ändert sich. Alte Manifeste bleiben für Time-Travel erhalten.

### 3.2 MVCC und Sichtbarkeit

- Jeder Record hat Versionen, identifiziert über `(LogicalKey, commit_id)`.
- Sichtbarkeit: Ein Record ist sichtbar, wenn sein Commit Vorfahr des Snapshot-Commits ist und kein Tombstone im selben oder späteren sichtbaren Commit existiert.
- Snapshot Isolation ist Default; optional serialisierbare Validierung über Read-/Write-Sets.
- Konflikterkennung beim Commit: Write-Write auf gleichem `LogicalKey` im selben Branch → Konflikt; Cross-Branch-Merges separat.

### 3.3 Branch, Merge, Time-Travel, Rollback

- **Branch:** Ref erzeugen; keine Datenkopie.
- **Time-Travel:** `AS OF COMMIT <id>` oder `AS OF TX_TIME <timestamp>`; zusätzlich `VALID AT <timestamp>` für fachliche Zeit.
- **Diff:** Vergleich zweier Manifeste/Commits über `RecordId`-Mengen und Property-Hashes; optional semantischer Diff für Vektoren/Graphen.
- **Rollback:** Branch-Ref atomar auf älteren Commit setzen; keine Datenlöschung.
- **Merge:** Drei-Wege-Merge über `LogicalKey`:
  - Property-Konflikte: Hash-Vergleich; Policy `manual`, `prefer_newer`, `prefer_confidence`, `prefer_source`.
  - Vektor-Konflikte: gleicher `LogicalKey`, unterschiedliche Vektoren → Konflikt oder Policy `recompute_embedding`.
  - Graph-Kanten: Union mit Tombstone-Auflösung; doppelte Kanten werden zusammengeführt.
  - Text: Index wird aus gemergten Records neu gebaut; Postings nicht direkt gemergt.
- **Ephemeral Branches:** TTL, automatische GC nach Retention; Merge nur über Validierungs-Gate.

### 3.4 Concurrency in Rust

- Read-Path: `ArcSwap<Snapshot>` + `crossbeam-epoch`; keine Locks.
- Write-Path: Single-Writer pro Branch für Commit-Ref; parallele Transaktionen über WAL und Memtable.
- Caches: `DashMap`/`parking_lot::RwLock` nur für nicht-kritische Metadaten.
- CPU-intensive Index-Builds: `rayon`.
- Keine vollständige Wait-Freedom-Garantie; Reads sind lock-free, Commits nutzen kurze kritische Sektionen und Group Commit.

### 3.5 Durability & Recovery

- ACID: Atomarität über WAL + Commit-Ref; Consistency über Schema- und Provenance-Validierung; Isolation über MVCC; Durability über `fsync` von WAL und Ref.
- Crash Recovery:
  1. Letzten Checkpoint laden.
  2. WAL ab LSN replayen.
  3. Merkle-Roots verifizieren.
  4. Branch-Refs und Manifeste prüfen.
  5. Verwaiste Temp-Dateien löschen.
- Retention-Policy für alte Commits/Manifeste; GC nur für nicht gepinnte Snapshots.

---

## 4. LLM- & Agenten-Spezialfunktionen

### 4.1 Kryptografisches Provenance-Tracking

Jeder Record, Chunk und jede Kante erhält einen `ProvenanceNode`:

```text
ProvenanceNode {
  record_id,
  source_hash,          // Hash des Originaldokuments/Tool-Outputs
  ingestion_commit,
  model_id, prompt_hash, tool_trace_hash,
  confidence,           // 0.0–1.0, kalibrierbar
  signature,            // Ed25519 optional
  parent_provenance,    // Merkle-DAG der Herkunft
}
```

- Query-Ergebnisse enthalten Inclusion-Proofs gegen den Commit-Merkle-Root.
- Clients können prüfen: „Dieser Chunk stammt aus Commit X, Quelle Y, Modell Z.“
- Fakten ohne Provenance können gefiltert werden: `provenance_exists = true`.
- Confidence wird auf Chunk- und Kantenebene propagiert; Graph-Traversal kann Confidence multiplizieren oder min-aggregieren.

### 4.2 Token-aware Query Planner

Der Planner erhält ein Token-Budget und optimiert das Context-Packing:

- **Kostenmodell:** Tokenkosten pro Record/Chunk/Summary; Relevanz aus RRF, BM25, ANN-Distanz, Graph-Nähe, Temporalität, Confidence.
- **Informationsdichte:** Unique-Token-Anteil, Entropie, Redundanz via MinHash/SimHash.
- **Hierarchische Abstraktion:** Records tragen `abstraction_level` (`raw`, `summary`, `entity`, `doc`). Summaries verweisen per Graph-Kante auf Rohchunks.
- **Diversität:** MMR (Maximal Marginal Relevance) gegen redundante Chunks.
- **Budget-Optimierung:** 0/1-Knapsack oder DP: maximiere Nutzen unter Tokenbudget; Summaries als günstige Items mit Verweisen.
- **Lost-in-the-Middle-Mitigation:** Hochrelevante Items an Anfang und Ende; Gruppierung nach Quelle; explizite Zwischenüberschriften; keine rein monotone Sortierung.
- **Output:** `ContextPackage` mit `items[]`, `token_count`, `provenance_proofs[]`, `conflicts[]`, `omitted[]`.

### 4.3 Ephemeral Agent Branches

- Jede Agenten-Session erhält einen eigenen Branch `agent/<session-id>`.
- Tool-Calls und LLM-generierte Fakten werden zunächst nur dort geschrieben.
- Validierungs-Gate vor Merge: Tests, Schema-Checks, Provenance-Pflicht, Confidence-Schwelle, optional LLM-Judge.
- Fehlerhafte Branches werden verworfen; kein Einfluss auf `main`.
- Merge-Policy pro Agent: `manual`, `auto_if_validated`, `quarantine`.
- Quarantäne-Branch für widersprüchliche oder ungeprüfte Fakten.

### 4.4 Veraltetes Wissen & Widersprüche

- **Temporales Indexing:** Bitemporale Records; Query-Filter `valid_at`, `tx_time`.
- **Time-Decay:** Optionaler Score-Faktor `exp(-λ * age)`; λ pro Collection konfigurierbar.
- **Konflikterkennung beim Insert:** Index über `(subject, predicate)`; aktive Objekte mit unterschiedlichen Werten erzeugen `ContradictionSet`.
- **Policies:** `flag_only`, `prefer_newer`, `prefer_higher_confidence`, `prefer_verified_source`, `manual`.
- **Query-Verhalten:** Konflikte werden nicht still aufgelöst, sondern im `ContextPackage` als `conflicts[]` zurückgegeben; der Agent kann sie explizit behandeln.
- **Halluzinations-Schutz:** Unbelegte Fakten erhalten `confidence=0` oder `ungrounded=true`; Query kann sie ausschließen.

### 4.5 Nicht-Determinismus und Reproduzierbarkeit

- Provenance speichert Modell-ID, Prompt-Hash, Tool-Trace-Hash und Seed, sofern verfügbar.
- Agenten-Sessions sind replaybar: Branch + Commit + Provenance + Tool-Logs.
- Deterministische Index-Builds werden angestrebt, aber ANN-Suchen bleiben approximativ; Ergebnisse enthalten `index_watermark` und `recall_hint`.

---

## 5. Rust-Technologie-Stack & Architektur-Trade-offs

### 5.1 Empfohlene Crates

| Bereich | Crates |
|---|---|
| Async/Netzwerk | `tokio`, `tonic`, `hyper`, `tower` |
| Storage-I/O | `monoio` oder `glommio` für `io_uring`; `tokio`-Fallback; `memmap2` |
| Columnar/Zero-Copy | `arrow2`/`arrow`, `parquet2`, `rkyv`, `bytes` |
| Hashing/Krypto | `blake3`, `ed25519-dalek`, `crc32c` |
| Concurrency | `arc-swap`, `crossbeam-epoch`, `crossbeam-channel`, `dashmap`, `parking_lot`, `rayon` |
| Vektorindex | `hnsw_rs` oder eigene HNSW-Implementierung; DiskANN-Komponenten custom |
| Text | `unicode-segmentation`, `rust-stemmers`, `lindera`; Tantivy-Tokenizer wiederverwenden, nicht als separate Engine |
| Kompression | `zstd`, `lz4_flex` |
| Serialisierung | `serde`, `bincode`/`rkyv` |
| Observability | `tracing`, `metrics`, `opentelemetry` |
| Testing | `proptest`, `loom`, `criterion`, `insta` |

### 5.2 Trade-offs

| Zielkonflikt | Abwägung |
|---|---|
| **Write-Amplification vs. Lese-Latenz** | LSM-Flush + Compaction erzeugt Write-Amplification; Tiered Compaction und Zstd mildern. CoW-Manifeste machen Branching billig, halten aber alte Segmente länger am Leben. |
| **HNSW-RAM vs. DiskANN-Latenz** | HNSW liefert niedrige Latenz, kostet RAM; DiskANN spart RAM, erhöht Latenz und Build-Komplexität. Beide werden unterstützt; Planner wählt per Cost Model. |
| **Approximate ANN vs. Determinismus** | ANN ist approximativ und build-abhängig. Exakte Fallbacks und Watermark-Transparenz sind Pflicht; Recall-Hinweise in Ergebnissen. |
| **Graph-Merge vs. Konsistenz** | Kanten-Union ist einfach, Entity-Resolution und Property-Konflikte sind semantisch schwierig. Merge-Policies und manuelle Konfliktbehandlung bleiben notwendig. |
| **Token-Planer-Overhead vs. Kontextqualität** | DP/MMR kostet CPU; bei kleinen Kandidatenmengen greift eine Heuristik. Budget- und Latenzgrenzen sind konfigurierbar. |
| **Provenance-Overhead vs. Vertrauen** | Jeder Record erhält Hashes/Signaturen; das kostet Speicher und CPU. Optionales Signieren und Batch-Merkle-Bäume reduzieren Overhead. |
| **Lock-free Reads vs. Komplexität** | `ArcSwap` + Epoch-Reclamation sind robust, aber fehleranfällig. `loom`-Tests und klare Ownership-Grenzen sind erforderlich. |
| **Asynchrone Index-Builds vs. Query-Korrektheit** | Indizes können hinterherhinken. Queries nutzen nur Indizes mit passendem Watermark; sonst Scan/Fallback. |
| **Retention vs. Time-Travel** | Alte Commits/Manifeste ermöglichen Time-Travel, kosten Speicher. GC nur für nicht gepinnte Snapshots und nach Retention-Policy. |

### 5.3 Empfohlene Implementierungsreihenfolge

1. Kanonische Records, WAL, Memtable, Manifeste, MVCC, Branch/Commit.
2. Exakte Vektorsuche + BM25 + einfache Graph-Adjazenz + RRF.
3. HNSW, CSR-Optimierung, Token-aware Planner, Provenance.
4. Ephemeral Branches, Merge-Policies, Konflikterkennung, Time-Decay.
5. DiskANN, Compaction-Optimierung, Signaturen, Replikation über Object Store.

---

**Fazit:** Mnemosyne vereinigt Vektor-, Graph- und Textmodell über ein gemeinsames, unveränderliches Record- und Commit-Modell. Die Kombination aus LSM-artigem Write-Path, CoW-Manifesten, Merkle-DAG-Commit-Historie und lock-free Read-Snapshots ermöglicht ACID-Durability, Zero-Copy-Branching und echte hybride Queries. Die eingebauten Provenance-, Token-Planer-, Sandbox- und Konflikterkennungsmechanismen adressieren typische LLM- und Agenten-Schwachstellen direkt auf Engine-Ebene – ohne separate Vektor-, Graph- und Suchsysteme synchronisieren zu müssen.