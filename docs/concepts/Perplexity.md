Perplexity
# Spezifikation: **Helix** — Unified Multi-Model Knowledge Engine

Helix ist eine in Rust entwickelte, eingebettet bis clusterfähige hybride Datenbank-Engine für KI-Agenten und RAG-Systeme. Sie speichert Dokumente, Dense Vektoren, Property-Graphen und Volltextindizes in einem gemeinsamen versionierten Storage-Kern, sodass hybride Queries, Branching, Time-Travel und Provenance ohne externe Synchronisation funktionieren. Die Architektur nutzt eine LSM-artige, versionierte Basis mit Arrow-kompatiblen Columnar-Blöcken; Arrow ist eine etablierte, native Rust-Implementierung für In-Memory- und Parquet-Datenformate. [arrow.apache](https://arrow.apache.org/rust/arrow/index.html)

## 1. High-Level Architektur

### Schichtenmodell

| Schicht | Verantwortung | Kernelemente |
|---|---|---|
| API / Query Language | Deklarative hybride Queries, Transaktionen, Branch-Operationen | `helixql`, gRPC/HTTP, embedded API |
| Planner / Optimizer | Token-Budget, Kostenmodell, Hybrid-Ranking, Indexauswahl | Token-aware Planner, RRF-Engine |
| Knowledge Services | Provenance, Konflikterkennung, Temporalität, Summarization | Fact Ledger, Conflict Resolver |
| Execution | Vektor-, Graph- und Text-Scans, Joins, Filter | Vector Executor, Graph Traversal, Inverted Index |
| Versioning | MVCC, Branches, Commits, Merge, Time-Travel | Merkle-DAG, Snapshot Manifest |
| Storage Core | Persistenz, Caching, WAL, Compaction | LSM/SST, Buffer Pool, io_uring |

Die Engine ist bewusst **nicht** drei Datenbanken hinter einem Gateway. Stattdessen existiert ein gemeinsames logisches Datenmodell: Jede Entität, Kante, Chunk, Fakt und Vektor ist eine versionierte Row in einem gemeinsamen Keyspace mit konsistenten Commit-Metadaten.

### Logisches Datenmodell

```text
Node {
  node_id: Uuid,
  kind: Entity | Document | Chunk | Claim | Agent | Tool,
  properties: TypedMap,
  valid_time: [Time, Time),
  transaction_time: CommitId,
  provenance: ProvenanceRef,
  embedding: VectorRef?,
  text_ref: TextRef?,
}

Edge {
  edge_id: Uuid,
  src: NodeId,
  dst: NodeId,
  relation: RelationId,
  properties: TypedMap,
  confidence: f32,
  evidence: Vec<EvidenceRef>,
  valid_time: [Time, Time),
  transaction_time: CommitId,
}

Chunk {
  chunk_id: Uuid,
  document_id: DocumentId,
  text: TextBlob,
  token_count: u32,
  embedding: VectorRef,
  lexical_terms: InvertedIndexRef,
  provenance: ProvenanceRef,
}
```

Ein Chunk ist damit gleichzeitig Quelle für Vektor- und Textsuche sowie evidenzbares Objekt im Graphen. Eine Graphkante kann etwa `Claim --supported_by--> Chunk` sein; dadurch wird Retrieval nicht nur „ähnlich“, sondern nachvollziehbar begründet.

### Write-Path

1. Ein Agent oder Connector sendet eine Transaktion: Dokumente, Chunks, Embeddings, Entities, Kanten und Fakten.
2. Der **Write Coordinator** validiert Schema, Provenance, Konfidenz, Zeitintervalle und Referenzen.
3. Der **Conflict Detector** prüft semantische und temporale Widersprüche gegen den aktuellen Branch.
4. Änderungen werden als versionierte Write-Batches in den **Memtable** geschrieben; vor dem Acknowledgement landen die zugehörigen WAL-Records dauerhaft auf dem Storage Device.
5. Der Commit erzeugt einen neuen **Merkle-Commit** im Branch-DAG und aktualisiert Branch-Pointer atomar.
6. Background Services führen Flush, Compaction, Vektorindex-Rebuild bzw. inkrementelle Index-Merges aus.

LSM-Engines schreiben zunächst in eine Memtable, loggen Writes in ein WAL und flushen anschließend immutable Sorted Runs; Background-Compaction begrenzt dabei Read- und Space-Amplification.  Helix übernimmt dieses Grundmuster, erweitert es jedoch um Graph-, Vektor- und Text-Indexfragmente als Teil desselben Commits. [crates](https://crates.io/crates/lsm-db)

### Read-Path

1. Query wird geparst und in einen physischen Plan übersetzt.
2. Der Planner bestimmt ein Token-Budget, Relevanzgewichte, Graph-Filter, Zeitfilter und erforderliche Provenance-Tiefe.
3. Vektor-, BM25- und Graph-Executors arbeiten gegen **denselben MVCC-Snapshot**.
4. Ergebnisse werden fusioniert, dedupliziert, evidenzstabilisiert und in ein hierarchisches Kontextpaket umgewandelt.
5. Der Response enthält Antwortkandidaten, Evidenzpfade, Konfidenz, Konflikte und ggf. eine komprimierte Zusammenfassung.

## 2. Physisches Storage- und Index-Design

### Storage-Layout

Helix verwendet einen **hybriden LSM/Core-Design**:

- **LSM-Kern** für hochvolumige, append-lastige Daten: Chunks, Embeddings, Kanten, inverted postings, Provenance-Events.
- **Copy-on-Write B+Tree** für kleine, hot Metadata-Strukturen: Branch-Pointer, Schema, ACLs, Entity-Registry, aktive Index-Roots.
- **Immutable Segment Files** als Einheit für Recovery, Compaction, Replication und Merkle-Hashing.

Ein Segment enthält mehrere komplementäre Bereiche:

```text
segment_<commit>_<level>.hxs
├── row_block/          Arrow RecordBatches: Nodes, Edges, Chunks
├── vector_block/       quantisierte Dense Vectors + Row-IDs
├── graph_block/        CSR/CSC adjacency + edge properties
├── text_block/         term dictionary, postings, positions
├── provenance_block/   content hashes, source refs, signatures
├── bloom/              per-column Bloom/Ribbon filters
├── zone_map/           min/max statistics, temporal ranges
└── footer/             checksums, schema, index roots
```

Arrow bietet eine native Rust-Implementierung mit RecordBatches, IPC und Parquet-Unterstützung; Helix nutzt Arrow vor allem für columnare Row- und Vector-Blöcke, nicht zwingend für jedes Indexformat. [arrow.apache](https://arrow.apache.org/rust/arrow/index.html)

### Vektoren: HNSW plus DiskANN

Helix führt zwei komplementäre ANN-Strukturen:

| Index | Einsatz | Storage | Trade-off |
|---|---|---|---|
| In-Memory HNSW | Hot Working Set, niedrige Latenz, interaktive Agenten | RAM | Hoher RAM-Bedarf, Rebuild-/Merge-Kosten |
| DiskANN/Vamana | Große Korpora, kosteneffiziente Persistenz | SSD/NVMe | Höhere Latenz bei Cold Reads, weniger RAM |
| IVF-PQ | Optional für sehr große, niedrigpräzise Vorfilter | Disk | Schnell, aberApproximation und Re-ranking nötig |

Ein `rust-diskann`-artiger Vamana-Index unterstützt dynamische Insert/Delete-Operationen und ist explizit für disk-basierte ANN-Suche ausgelegt.  Helix verwendet ihn als Standard für große Korpora; HNSW dient als RAM-Cache für aktive Namespaces und kürzlich geschriebene Chunks. [docs](https://docs.rs/crate/rust-diskann/0.4.2)

**Vektorlayout:**

```text
vector_id -> {
  fp16_or_int8_codes: [u8],
  residual_or_pq_codes: [u8],
  norm: f32,
  model_id: ModelVersionId,
  dimension: u16,
  commit_id: CommitId,
  tombstone: bool,
}
```

Vektoren werden nicht als Opaque Blob im Graphen gespeichert, sondern über `VectorRef` an Chunks gebunden. Dadurch kann derselbe Chunk mehrere Embeddings besitzen — etwa unterschiedliche Modelle, Dimensionen oder Revisionsstände — ohne Text oder Graph-Struktur zu duplizieren.

### Graph: Property Graph im LSM

Graphen werden als **adjacency-first storage** gespeichert:

- `OUT(edge_sort_key) -> edge_id, dst, relation, properties`
- `IN(edge_sort_key) -> edge_id, src, relation, properties`
- `EDGE_BY_ID -> edge_record`
- `ENTITY_BY_KEY -> node_id`
- `CLAIM_BY_SUBJECT_PREDICATE -> claim_id`

Der Sort Key ist:

```text
(branch_epoch, src, relation, valid_from, edge_id)
```

Dadurch sind gerichtete Traversals, zeitliche Filter und Multi-Hop-Reasoning effizient. Für 1-Hop-Nachbarschaften wird ein CSR-Block pro Segment erzeugt; für dynamische Updates bleibt der LSM-Weg primär, während Compaction CSR-Blöcke konsolidiert.

### Volltext: Inverted Index mit BM25

Der Textindex ist kein separater Suchdienst, sondern ein versionierter Teil des Segments:

```text
term -> postings {
  chunk_id,
  tf,
  positions: delta-encoded,
  field_id,
  commit_id,
  doc_len,
}
```

BM25 wird über Segment-Statistiken berechnet: Term Frequency, Document Frequency, durchschnittliche Dokumentlänge und Feldgewichte. Positionsdaten erlauben Phrase- und Proximity-Queries; ein optionaler Sparse-Embedding-Index ergänzt lexikalische Suche für Out-of-Vocabulary- und semantische Lücken.

### Hybride Query ohne Duplikation

Eine typische Query lautet:

```helixql
MATCH (c:Company)-[:SUPPLIES*1..3]->(p:Product)
WHERE p.region = "EU"
  AND c.valid_at(now())
CALL vector_search(
  query = $question,
  index = "default",
  candidates = 200
)
CALL bm25_search(
  query = $question,
  fields = ["title", "body"],
  candidates = 200
)
FUSION rrf(k = 60)
SELECT chunk, evidence_path, confidence
TOKEN_BUDGET 6000
```

Der Executor führt folgende Schritte aus:

1. Graph-Pruning: Kandidaten-Chunk-IDs durch Graph-Filter einschränken.
2. Vektorsuche: ANN über HNSW/DiskANN, optional mit Graph-Filter als Post- oder Pre-Filter.
3. Lexikale Suche: BM25 über den Inverted Index.
4. Fusion: Reciprocal Rank Fusion:

\[
\mathrm{RRF}(d) = \sum_{i=1}^{k} \frac{w_i}{\mathrm{rrf\_k} + \mathrm{rank}_i(d)}
\]

5. Re-Ranking: Cross-Encoder optional, Confidence- und Provenance-Boost.
6. Context Assembly: Deduplizierung, Token-Budget, Hierarchie und Positionierung.

Alle Executors lesen aus demselben Commit-Snapshot. Dadurch ist eine hybride Query transaktional konsistent: Sie sieht keine neu geschriebene Kante, aber einen alten Vektor für denselben Chunk.

## 3. Versionierungs- und Concurrency-Engine

### MVCC-Modell

Helix verwendet **Snapshot Isolation** mit optionaler Serialisierbarkeit:

- Jede Row besitzt `(branch_id, commit_id, valid_time, tombstone)`.
- Leser erhalten einen stabilen `ReadSnapshot(commit_id)`.
- Writer nutzen Optimistic Concurrency Control: Read-Set-Validierung vor Commit.
- Bei Serialisierbarkeit werden Write-Write-, Read-Write- und Graph-Topologie-Konflikte geprüft.
- Graph-Traversals erhalten ein deterministisches Snapshot-Verhalten: Eine Kante ist entweder vollständig im Snapshot oder nicht sichtbar.

LSM-basierte Rust-Engines verwenden üblicherweise monotone Sequence Numbers, MVCC-Snapshots und First-Committer-Wins-Konflikterkennung; Helix verallgemeinert dieses Modell auf Graphen, Vektoren und Textindizes. [github](https://github.com/maxmccutcheon59/spindle)

### Merkle-DAG und Branches

Jeder Commit ist ein DAG-Knoten:

```text
Commit {
  commit_id: Hash,
  parents: Vec<Hash>,
  branch_id: BranchId,
  schema_version: u64,
  segment_manifest: SegmentManifestHash,
  index_roots: {
    vector: Hash,
    graph: Hash,
    text: Hash,
    provenance: Hash,
  },
  stats: CommitStats,
  signature: Option<Signature>,
}
```

Ein Branch-Pointer ist lediglich ein atomar ersetzter Commit-Hash:

```text
main -> commit_9f3a
agent_session_42 -> commit_71bb
```

Daraus ergeben sich:

- **Zero-Copy Branching:** Ein neuer Branch kopiert nur den Commit-Pointer, nicht Daten oder Indizes.
- **Time-Travel:** Queries binden einen Commit-Hash und lesen ausschließlich in diesem Snapshot sichtbare Versionen.
- **Diff:** Zwei Commits werden über Segment-Manifeste und Merkle-Roots verglichen; nur veränderte Segmente und Keys werden materialisiert.
- **Rollback:** Ein Branch wird auf einen früheren Commit-Hash gesetzt; alte Commits bleiben bis zur Garbage Collection erhalten.
- **Merge:** Drei-Wege-Merge über gemeinsamen Ancestor, semantische Merge-Regeln und Konfliktauflösung.

### Merge-Semantik

| Objekt | Merge-Regel | Konfliktfall |
|---|---|---|
| Dokument/Chunk | Content-hash-basiert | Identischer Hash: automatisch zusammenführen |
| Node | Property-Level-Merge | Unterschiedliche Werte: Policy oder manuelle Entscheidung |
| Edge | Topologie + Confidence | Widersprüchliche Relation: Konflikt-Claim erzeugen |
| Vektor | Modell- und Chunk-ID-gebunden | Unterschiedliche Embeddings: Versionierung, kein stiller Overwrite |
| Fakt | Temporaler Geltungsbereich | Widerspruch: Assertion, Supersession oder Quarantine |
| Index | Deterministische Rebuild/Merge | Inkonsistente Root: Commit wird abgelehnt |

Ein Merge erzeugt niemals einen „stillschweigend korrekten“ Graphen. Bei Konflikten entstehen explizite `Conflict`-Nodes mit Alternativen, Evidenz und Confidence.

### Rust-Concurrency

- **Lock-free Read Path:** Immutable Segmente und atomische Branch-Pointer erlauben parallele Reads ohne globale Locks.
- **Sharded Write Coordinator:** Partitionierung nach Namespace und Entity-Hash reduziert Contention.
- **Wait-free Snapshot Lookup:** `ArcSwap<BranchHead>` bzw. RCU-artiger Head-Pointer.
- **Crossbeam-Skiplist Memtable:** Lock-freie Inserts, versionierte Values.
- **DashMap / ShardMap:** Nur für begrenzte, hot Metadata-Strukturen.
- **Tokio + io_uring:** Asynchrone I/O-Pipelines für WAL, Segment-Flush, Cache-Prefetch und Remote Storage; Tokio bietet eine optionale `io-uring`-Integration unter Linux. [doc.servo](https://doc.servo.org/tokio/index.html)
- **Rayon / work-stealing:** CPU-lastige ANN-Graph-Builds, Compaction und BM25-Scoring.

Rusts Ownership-Modell verhindert Data Races zur Compile Time; `Send`/`Sync`-Bounds dokumentieren, welche Executor-Strukturen safely über Tasks geteilt werden dürfen. Unsafe Code ist auf wenige, auditierbare Module beschränkt: SIMD, mmap, io_uring-Wrapper und optionaler Allocator.

## 4. LLM- und Agenten-Spezialfunktionen

### Provenance als First-Class Citizen

Jeder Chunk, jede Kante und jeder abgeleitete Fakt trägt einen Provenance-Record:

```text
Provenance {
  content_hash: Blake3Hash,
  source_uri: Option<Url>,
  source_fingerprint: SourceFingerprint,
  extractor: ExtractorId,
  model_id: Option<ModelVersionId>,
  prompt_hash: Option<Hash>,
  confidence: Confidence,
  verified_by: Vec<VerifierId>,
  chain: Vec<ProvenanceRef>,
}
```

Kryptografische Integrität entsteht durch:

- **Content Hash:** Blake3 über normalisierten Chunk-Inhalt.
- **Merkle-Root:** Aggregation über Segment- und Commit-Ebene.
- **Signature:** Optionale Signatur durch Connector, Verarbeiter oder Reviewer.
- **Evidence Chain:** `Claim -> SupportingChunk -> SourceDocument -> OriginalURI`.
- **Confidence Decomposition:** Extraktions-, Quellen-, Konsistenz- und Verifikations-Score.

Eine Antwort darf nur dann als `grounded` markiert werden, wenn jede relevante Assertion mindestens einen gültigen Evidence-Pfad besitzt. Fehlende Evidenz erzeugt nicht automatisch eine Löschung, sondern ein `Unverified`-Flag und eine niedrigere Retrieval-Priorität.

### Token-aware Query Planner

Der Planner optimiert nicht nur Latenz, sondern **Informationsdichte pro Token**.

**Ziel:**

\[
\max \left( \sum_i \mathrm{value}(r_i) - \lambda \cdot \mathrm{tokens}(r_i) - \mu \cdot \mathrm{redundancy}(R) \right)
\]

unter der Nebenbedingung:

\[
\sum_i \mathrm{tokens}(r_i) \leq \mathrm{token\_budget}
\]

Der Planner berücksichtigt:

- Relevanz aus Vektor-, BM25- und Graph-Score.
- Informationsdichte: neue Entitäten, Relationen, Zahlen, Zitate und seltene Begriffe.
- Redundanz: Embedding-/Jaccard-/shingle-basierte Near-Duplicate-Erkennung.
- Position: Wichtige Evidenz wird an den Anfang und das Ende des Kontexts gestellt, um Lost-in-the-Middle-Effekte zu reduzieren.
- Hierarchie: Zusammenfassung auf Dokument-, Abschnitts- und Claim-Ebene.
- Budget: Harte Token-Grenze pro Agenten-Call.

**Context Pack Format:**

```text
1. Task contract
2. High-confidence answer skeleton
3. Primary evidence chunks
4. Contradictions and caveats
5. Supplementary evidence
6. Raw source pointers
```

Der Planner kann beispielsweise 70 Prozent des Budgets für diverse High-Confidence-Chunks, 20 Prozent für Gegenbelege und 10 Prozent für Rohquellen-Pointer reservieren.

### Ephemeral Agent Branches

Jede Agenten-Session erhält standardmäßig einen ephemeralen Branch:

```text
session:agent_42 -> ephemeral_commit
```

Regeln:

- Tool-Outputs, Zwischenhypothesen und halluzinationsverdächtige Assertions landen zunächst nur im Session-Branch.
- Der Branch besitzt ein TTL, ein Kostenbudget und eine maximale Write-Rate.
- Vor dem Merge laufen Validierungsregeln: Quellenpflicht, Konfidenz-Schwellen, Schema-Checks, Zitierprüfung und Konfliktauflösung.
- Fehlgeschlagene Sessions werden verworfen, ohne `main` zu beeinflussen.
- Erfolgreiche Sessions werden als Reviewable Merge vorgeschlagen, nicht automatisch publiziert.

Dadurch wird die Wissensbasis zum **staged knowledge graph**: Agenten dürfen explorieren, aber der globale Zustand bleibt kontrollierbar.

### Temporales Wissen und Konflikte

Helix unterscheidet zwei Zeitachsen:

- **Valid Time:** Wann eine Aussage in der realen Welt gilt.
- **Transaction Time:** Wann die Engine sie gespeichert hat.

Ein Fakt kann beispielsweise lauten:

```text
CEO_OF(CompanyA, PersonB)
valid_time = [2024-01-01, 2026-05-01)
```

Wird ein neuer CEO eingefügt, schließt Helix nicht einfach den alten Fakt, sondern erzeugt eine Supersession-Relation mit Evidenz. Widersprüchliche Aussagen ohne klare zeitliche Ordnung werden als `Conflict`-Node gespeichert.

**Time-Decay:**

\[
\mathrm{weight}(e, t) = \mathrm{base\_confidence}(e) \cdot e^{-\lambda(t - t_\mathrm{last\_verified})}
\]

Aktualität ist dabei kein Ersatz für Korrektheit: Ein alter, verifizierter Fakt kann höher gewichtet werden als ein neuer, unbelegter Agenten-Claim.

### Halluzinations- und Tool-Fehlerabwehr

- **Claim Extraction:** LLM-Outputs werden vor dem Commit in atomare, typisierte Claims zerlegt.
- **Citation Binding:** Jede Assertion muss mindestens eine Chunk-Referenz besitzen.
- **Entailment Check:** Optionaler NLI-/Cross-Encoder prüft, ob Evidenz die Claim tatsächlich stützt.
- **Entity Resolution:** Kanonische Entity-IDs verhindern, dass dieselbe Entität unter mehreren Namen divergiert.
- **Schema Constraints:** Kardinalitäten, Typen, erlaubte Relationen und zeitliche Konsistenz werden erzwungen.
- **Tool Call Replay:** Nichtdeterministische Tool-Outputs werden mit Input-Hash, Tool-Version, Zeitstempel und Output-Hash versioniert.
- **Quarantine Zone:** Niedrigkonfidente, widersprüchliche oder unbelegte Inhalte sind abfragbar, aber nicht Teil des Default-Grounding-Kontexts.

## 5. Rust-Stack und Trade-offs

### Empfohlener Stack

| Bereich | Empfehlung | Begründung |
|---|---|---|
| Runtime | `tokio`, optional `io-uring` | Asynchrones I/O; Tokio unterstützt io_uring unter Linux.  [doc.servo](https://doc.servo.org/tokio/index.html) |
| Storage | Eigener LSM-Kern oder `lsm-db`-Prinzipien | WAL, Memtable, SSTables, Compaction und Block Cache sind bewährte Grundbausteine.  [crates](https://crates.io/crates/lsm-db) |
| WAL | Eigene WAL mit CRC32C, Group Commit, `fdatasync` | Kontrolle über Durability, Recovery und Commit-Latenz |
| Columnar | `arrow`, `parquet` | Native Rust-Implementierung, columnare Arrays, IPC und Parquet.  [arrow.apache](https://arrow.apache.org/rust/arrow/index.html) |
| Zero-Copy | `rkyv` für interne Hot Structures, Arrow für Tabellen | Schnelle Deserialisierung; sorgfältiger Einsatz wegen Validierung und Versionierung |
| Vektorindex | Eigener HNSW + `rust-diskann`-artiger Vamana-Index | RAM-optimierte Hot Search und disk-effiziente Large Corpus Search.  [docs](https://docs.rs/crate/rust-diskann/0.4.2) |
| Concurrency | `crossbeam`, `parking_lot`, `arc-swap`, `dashmap` | Lock-freie Strukturen, atomare Head-Pointer, begrenzte Locks |
| Hashing | `blake3` | Schnelles Content- und Merkle-Hashing |
| Compression | `zstd`, optional `lz4` | Kompromiss aus CPU und Speicherplatz |
| Quantisierung | `half`, `simsimd`, `rand` | FP16/INT8, SIMD-Scoring, reproduzierbare Sampling-Policies |
| Observability | `tracing`, `metrics`, `opentelemetry` | Latenz, Cache-Hit-Rate, Compaction, WAL-Backlog, Query-Quality |

### Durability-Modell

Helix bietet drei Durability-Profile:

| Profil | WAL-Verhalten | Anwendungsfall |
|---|---|---|
| `relaxed` | Group Commit, z. B. 5–20 ms | Bulk Ingest, Caches, abgeleitete Daten |
| `transactional` | WAL Flush pro Commit, Group Commit | Standard für Agenten-Wissen |
| `strict` | `fdatasync` vor Acknowledgement, optional fsync auf Verzeichnis | Finanz-, Compliance- und Auditdaten |

Nach einem Crash werden WAL-Records geprüft, replays und unvollständige Segmente verworfen. Commit-Pointer werden erst nach erfolgreichem Flush und Manifest-Update sichtbar.

### Kompromisse

| Zielkonflikt | Entscheidung | Begründung |
|---|---|---|
| Write- vs. Read-Amplification | LSM für Bulk, CoW B+Tree für Metadata | Schnelle Ingests ohne langsame Metadata-Updates |
| RAM vs. Latenz | Tiered ANN: HNSW-Cache plus DiskANN | Hot Data bleibt schnell, Cold Data bleibt kosteneffizient |
| Frische vs. Index-Qualität | Incremental Index + Background Rebuild | Neue Chunks sofort abfragbar, optimale Graph-Struktur folgt später |
| Merge-Automatisierung vs. Korrektheit | Policy-gesteuerte, reviewbare Merges | Vektor- und Graphkonflikte dürfen nicht still verschmolzen werden |
| Zero-Copy vs. Sicherheit | `rkyv` nur für trusted internal formats | Externe Eingaben erhalten Validierung und Versionierung |
| Token-Budget vs. Recall | Budgetierte, diversitätsbewusste Auswahl | Verhindert Redundanz, ohne relevante Evidenz zu verlieren |
| MVCC vs. Serialisierbarkeit | Snapshot Isolation default, Serializable opt-in | Gute Standardlatenz, strenge Konsistenz nur wo nötig |

### Garbage Collection

Commits, die von keinem Branch, Snapshot, Subscription oder Retention Policy referenziert werden, werden asynchron entfernt. Vor der Löschung prüft Helix:

- Aktive Time-Travel-Queries.
- Pinning durch Agenten-Sessions.
- Replication-Lags.
- Compliance- und Audit-Retention.
- Abhängige Provenance-Records.

## 6. Betrieb und Erweiterbarkeit

Helix unterstützt mehrere Deployment-Modi:

- **Embedded:** In-Process-Bibliothek für Agenten-Runtimes.
- **Single-Node Service:** gRPC/HTTP mit lokalem NVMe-Storage.
- **Shared-Nothing Cluster:** Sharding nach Namespace/Entity, Replikation per Raft oder Quorum-WAL.
- **Object-Storage-Tier:** S3-kompatible Segmente für große, kalte Korpora.

Kernmetriken umfassen p99 Query-Latenz, ANN-Recall, BM25-Latenz, Graph-Traversal-Tiefe, WAL-Flush-Latenz, Compaction-Backlog, Cache-Hit-Rate, Merge-Konfliktrate, Provenance-Coverage und Token-Nutzung pro Antwort.

Das Ergebnis ist eine Engine, in der Retrieval, Reasoning, Versionierung und Evidenz nicht nachträglich integriert werden, sondern im selben transaktionalen Datenkern entstehen. Damit können Agenten sicher experimentieren, Wissen versioniert weiterentwickeln und Antworten mit überprüfbaren Quellen liefern.

---
