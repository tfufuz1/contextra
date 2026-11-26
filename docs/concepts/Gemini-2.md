## 1. High-Level Architektur & Schichtenmodell

Das System (Arbeitsname **AethelDB**) ist eine in Rust entwickelte, eingebettete und verteilt skalierbare Hybrid-Datenbank-Engine. Anstatt Vektoren, Graphen und Volltext in getrennten Engines zu synchronisieren, betrachtet AethelDB jede Informationseinheit als ein **unveränderliches, inhaltsadressiertes Objekt (`UnifiedRecord`)**, über das drei physisch gekoppelte, aber spezialisierte Index-Projektionen gelegt werden.

  

### Architektur-Schichten

1. **Query & Agent Protocol Layer:** Stellt eine eingebettete Rust-API sowie eine Netzwerkschnittstelle über **Apache Arrow Flight** und gRPC bereit. Akzeptiert deklarative Hybrid-Queries (kombiniertes Vektor-, Graph- und BM25-Retrieval mit explizitem Token-Budget) sowie Git-ähnliche Versionsbefehle (`BRANCH`, `COMMIT`, `DIFF`, `MERGE`, `TIME_TRAVEL`).
    
2. **Token-Aware Hybrid Query Planner & Optimizer:** Ein kostenbasierter Optimizer, der Hybrid-Abfragen anhand von Kardinalitätsschätzungen (RoaringBitmap-Dichte, Posting-Listen-Länge, Graph-Verzweigungsgrad) in einen optimalen Ausführungsplan übersetzt (Pre-Filtering vs. Post-Filtering) und das Ergebnis anschließend auf Context-Window-Limits und Positions-Biases von LLMs zuschneidet.
    
3. **Version & Transaction Manager (MVCC + Merkle-DAG):** Verwaltet Branch-Referenzen, Snapshot-Isolation über Epoch-Based Reclamation, Optimistic Concurrency Control (OCC) und Ephemeral Sandboxes für isolierte Agenten-Sessions.
    
4. **Unified Multi-Model Index & MemTable Layer:** Hält aktive, noch nicht auf Disk geschriebene Mutationen eines Branches in lock-freien In-Memory-Strukturen (Concurrent Adaptive Radix Tree, In-Memory-HNSW-Delta, Dynamic Adjacency Map, In-Memory Inverted Index).
    
5. **Storage, Buffer Pool & Async I/O Engine:** Verwaltet unveränderliche, spaltenorientierte Multi-Modal-Segmente (**IMMS**) auf NVMe-SSDs über asynchrones Direct I/O (`io_uring`), einen lock-freien Buffer Pool sowie Zero-Copy-Deserialisierung (`rkyv` und Apache Arrow).
    

### Kern-Datenmodell (`UnifiedRecord`)

Jeder Datensatz (Dokument-Chunk, Entität, Relation oder Agenten-Beobachtung) wird beim Ingest normalisiert und erhält eine globale 128-Bit-ID auf Basis einer **Hybrid Logical Clock (HLC)**:

  

Rust

```
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct EntityId {
    pub hlc_timestamp: u64, // Physische Zeit (48 Bit) + Logischer Zähler (16 Bit)
    pub node_and_seq: u64,  // Worker-ID (16 Bit) + Sequenznummer (48 Bit)
}

pub struct UnifiedRecord {
    pub id: EntityId,
    pub payload: Bytes,                   // Zstd-komprimierter Rohtext / JSON-Properties
    pub summary_ref: Option<EntityId>,    // Verweis auf verdichteten Abstraktions-Knoten
    pub vector: Option<AlignedVector>,    // FP16 / SQ8 Embedding-Vektor
    pub edges: SmallVec<[GraphEdge; 4]>,  // Ausgehende typisierte Relationen
    pub valid_time: TemporalRange,        // Bitemporale Gültigkeit (valid_from..valid_to)
    pub provenance: ProvenanceRecord,     // Kryptografischer Herkunfts- & Vertrauensnachweis
}
```

### End-to-End Datenflüsse

- **Write-Path (Ingest bis Commit):**
    
    1. **Normalisierung & Hashing:** Der Ingestion-Worker berechnet tokenisierte Term-Frequenzen, optional quantisierte Vektoren sowie den `Blake3`-Inhalts-Hash inklusive `ProvenanceRecord`.
        
    2. **Group-Commit WAL:** Mehrere nebenläufige Writes werden in einem lock-freien Ringbuffer gebündelt und über einen einzigen `io_uring`-Batch-Write (`O_DIRECT` + `fdatasync`) sequenziell in das Write-Ahead-Log geschrieben.
        
    3. **Branch-MemTable-Insert:** Nach erfolgreichem WAL-Persist werden Payload, Vektor, Graph-Kanten und Text-Postings atomar in die aktive **Branch-MemTable** eingetragen und unter einer segment-lokalen `InternalRowId` (`u32`) verknüpft.
        
    4. **Background-Flush & Merkle-Commit:** Erreicht die MemTable ihren Schwellenwert (z. B. 64 MiB), wird sie im Hintergrund in ein unveränderliches **Immutable Multi-Modal Segment (IMMS)** serialisiert. Ein neuer `CommitNode` wird im Merkle-DAG erzeugt und der Branch-`HEAD` per atomarem Pointer-Swap aktualisiert.
        
- **Read-Path (Single-Transaction Hybrid Query):**
    
    1. **Snapshot-Pinning:** Der Reader pinnt die aktuelle Epoch (`crossbeam_epoch::pin()`) und lädt wait-free den `BranchSnapshot` (aktuelle MemTable + Liste aktiver IMMS-Segmente).
        
    2. **Parallele Filter- & Kandidatengenerierung:** Graph-Traversal erzeugt pro Segment ein `RoaringBitmap` zulässiger `InternalRowId`s. Vektor-Suche (HNSW/Vamana) und Volltext-Suche (Block-Max WAND) nutzen dieses Bitmap direkt während der Traversierung zum Pruning.
        
    3. **Score-Fusion & Decay:** Die Teil-Rankings werden über **Reciprocal Rank Fusion (RRF)** und **Temporal Decay** auf der `InternalRowId` zusammengeführt, ohne Payload-Daten zu kopieren.
        
    4. **Context-Window-Optimierung:** Nur die final selektierten Top-$K$-Einträge werden via Zero-Copy aus dem Buffer Pool aufgelöst, semantisch dedupliziert und U-förmig für das LLM angeordnet.
        

## 2. Physisches Storage- & Index-Design

Reine B+Trees leiden unter hoher Write-Amplification bei vielen kleinen Agenten-Updates; klassische LSM-Trees hingegen fragmentieren Graph-Adjazenzlisten und HNSW-Nachbarschaften über viele SSTables. AethelDB kombiniert deshalb einen **Content-Addressed LSM-Tree** mit **Immutable Multi-Modal Segments (IMMS)**.

  

### Aufbau eines Immutable Multi-Modal Segments (IMMS)

Jedes IMMS ist eine selbstbeschreibende, unveränderliche Datei (benannt nach dem `Blake3`-Hash seines Footers), unterteilt in 4-KiB- oder 16-KiB-Pages. Innerhalb eines Segments werden alle Modalitäten über eine dichte, segment-lokale `InternalRowId` (`0..N-1`) indiziert – **ohne Duplikation von Texten oder Metadaten**.

  

|**Bereich im Segment**|**Physisches Format**|**Zugriffs- & Kompressionsstrategie**|
|---|---|---|
|**1. ID-Mapping (`EntityId -> RowId`)**|Sortiertes Array + Ribbon-Filter|Binäre Suche / Interpolation Search im RAM; Ribbon-Filter (10 Bit/Key) verhindert unnötige Disk-Reads.|
|**2. Columnar Payload Store**|Apache Arrow IPC Pages|Zstd-Kompression mit segment-trainiertem Dictionary; Speicherung von Rohtext, Properties und `ProvenanceRecord`.|
|**3. Vektor-Index (Tiered Vamana/HNSW)**|DiskANN-inspiriertes Page-Layout|Obere Graph-Layer + SQ8-Vektoren im RAM; Layer-0-Knoten (SQ8-Vektor + `[u32; R]` Nachbarn) gepackt in 4-KiB-Disk-Pages; FP16-Vektoren in separatem Blob für Re-Ranking.|
|**4. Property-Graph (Bidirectional CSR)**|`rkyv`-archivierte Compressed Sparse Row|Zwei CSR-Strukturen (`ForwardCSR` & `ReverseCSR`) für $O(1)$-Hop-Lookups; Kanten-Properties spaltenorientiert für SIMD-Filter.|
|**5. Volltext-Index (Inverted Index)**|FST-Dictionary + SIMD-BP128 Postings|Term-Lexikon als Finite State Transducer (FST); Posting-Listen in 128er-Blöcken mit Block-Max-BM25-Metadaten (WAND).|

### Kopplung von Vektor, Graph und Text auf Disk

- **Vektor-Layout (Minimierung von Random-IOPS):** In klassischen HNSW-Implementierungen auf Disk erfordert jeder Graph-Hop einen Read für die Adjazenzliste und bis zu $R$ weitere Reads für die Nachbar-Vektoren. AethelDB nutzt auf Disk das **Vamana-Co-Location-Layout**: Jeder Knoten auf Layer 0 belegt einen festen Slot innerhalb einer 4-KiB-Page, der seinen 8-Bit-quantisierten Vektor (`SQ8`) **und** seine `R` Nachbar-`InternalRowId`s direkt nebeneinander enthält. Ein einzelner NVMe-Read lädt damit den gesamten lokalen Nachbarschafts-Kontext. Das finale Re-Ranking der Top-Kandidaten greift einmalig sequenziell auf den unkomprimierten FP16-Vektor-Blob zu.
    
- **Graph-Layout (Bidirectional CSR):** Kanten werden zweifach als **Compressed Sparse Row** abgelegt, um eingehende und ausgehende Traversals symmetrisch zu beschleunigen:
    
    Rust
    
    ```
    #[repr(C)]
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct CsrGraphIndex {
        pub row_offsets: Box<[u64]>,      // Index = source InternalRowId, Wert = Start in targets
        pub edge_types: Box<[u16]>,       // Komprimierte Relationstyp-IDs (SIMD-scanbar)
        pub target_rows: Box<[u32]>,      // Ziel-InternalRowId im selben Segment
        pub edge_record_rows: Box<[u32]>, // Verweis auf Kanten-Properties & Provenance im Payload Store
    }
    ```
    
    _Hinweis:_ Verweist eine Kante auf eine `EntityId` in einem älteren Segment, löst ein segmentübergreifender **Cross-Segment Edge Resolver** die Ziel-ID über die Segment-Ribbon-Filter auf.
    

### Ausführung hybrider Queries ohne Daten-Duplikation

Bei einer hybriden Query (z. B. _„Finde Chunks zu 'OAuth Token Refresh' innerhalb von 2 Hops um Entität `AuthService`, kombiniert aus Vektor-Ähnlichkeit und BM25“_) koordiniert der Query Planner drei Phasen:

  

1. **Graph-Filter-Bitmask ($B_{\text{graph}}$):** Der Graph-Executor traversiert `ForwardCSR` / `ReverseCSR` ausgehend von `AuthService` (unter Prüfung von Kanten-Typen und zeitlicher Gültigkeit) und setzt die erreichten `InternalRowId`s in einem komprimierten `RoaringBitmap` $B_{\text{graph}}$ pro Segment.
    
2. **Adaptive Hybrid-Vektorsuche:** Der Optimizer berechnet die Selektivität $\sigma = \vert{}B_{\text{graph}}\vert{} / N_{\text{segment}}$:
    
    - **Hohe Selektivität ($\sigma < 0.05$):** Graph-Navigation im HNSW wäre langsamer und riskiert disjunkte Subgraphen. Stattdessen iteriert die Engine direkt über die gesetzten Bits in $B_{\text{graph}}$ und berechnet die Distanzen per **AVX-512 / NEON SIMD** über den SQ8-Vektor-Blob (Exact Filtered Scan).
        
    - **Mittlere Selektivität ($0.05 \le \sigma \le 0.65$):** Die Engine nutzt **ACORN-inspiriertes 2-Hop-Lookahead-Pruning**. Während der HNSW-Suche werden Nachbarn $v \notin B_{\text{graph}}$ nicht für die Ergebnisliste gewertet, aber deren unmittelbare Nachbarn $w \in N(v)$ geprüft, falls $w \in B_{\text{graph}}$. Das verhindert, dass die Suche an Filter-Grenzen stecken bleibt.
        
3. **Bitmask-geführtes BM25 & Reciprocal Rank Fusion (RRF):** Der Volltext-Executor evaluiert die Query-Terme mittels **Block-Max WAND** und überspringt jeden 128er-Posting-Block, dessen ID-Intervall keinen Schnittpunkt mit $B_{\text{graph}}$ aufweist. Die resultierenden Ranglisten werden über RRF kombiniert:
    
    $$\text{Score}_{\text{RRF}}(r) = \frac{w_{\text{vec}}}{k + \text{rank}_{\text{vec}}(r)} + \frac{w_{\text{bm25}}}{k + \text{rank}_{\text{bm25}}(r)} + w_{\text{graph}} \cdot \text{HopScore}(r)$$
    
    Erst nach der Fusion werden die finalen Top-$K$-`InternalRowId`s dereferenziert, um die Payload-Texte aus dem Arrow-Store zu lesen.
    

## 3. Versionierungs- & Concurrency-Engine

### Merkle-DAG & Content-Addressed Manifeste

Jeder konsistente Datenbankzustand wird durch einen unveränderlichen `CommitNode` im Merkle-DAG repräsentiert, dessen Primärschlüssel der kryptografische `Blake3`-Hash seines Inhalts ist:

  

Rust

```
pub type ContentHash = [u8; 32]; // Blake3 Digest

pub struct CommitNode {
    pub commit_id: ContentHash,
    pub parents: SmallVec<[ContentHash; 2]>, // 1 Parent regulär, 2 Parents bei Merge
    pub entity_tree_root: ContentHash,       // Root des CoW Merkle-ART (EntityId -> SegmentLocation)
    pub active_segments: Arc<[SegmentMeta]>, // Unveränderliche Liste der IMMS-Segmente dieses Commits
    pub hlc_timestamp: u64,
    pub author_agent_id: u64,
    pub commit_kind: CommitKind,             // Standard, AgentCheckpoint, Merge, Compaction
}
```

- **Copy-on-Write Merkle-ART (Adaptive Radix Tree):** Der Primärindex `EntityId -> (SegmentId, InternalRowId)` ist als persistent-unveränderlicher Adaptive Radix Tree implementiert. Jeder innere Knoten speichert neben den Kind-Pointern den `Blake3`-Hash seines Teilbaums. Wird ein Datensatz eingefügt oder aktualisiert, werden ausschließlich die Knoten entlang des Pfades von der Wurzel zum Blatt kopiert (Path Copying, $O(\log_{256} N)$), während alle unveränderten Teilbäume geteilt bleiben.
    

### Zero-Copy Branching & Time-Travel

- **Zero-Copy Branching:** Ein Branch (`main`, `agent/task-891/hypothesis-a`) ist lediglich ein atomarer Zeiger in einer Branch-Tabelle (`BranchRef { name, head: AtomicPtr<CommitNode>, policy }`). Das Erstellen eines Branches dauert wenige Nanosekunden und verbraucht **0 Bytes** zusätzlichen Storage.
    
- **Point-in-Time Time-Travel:** Da alle IMMS-Segmente und Merkle-ART-Knoten streng unveränderlich sind, erfordert eine historische Query (`SELECT ... AT COMMIT <hash>` oder `AS OF HLC <ts>`) keinerlei Rekonstruktion aus Undo-Logs. Der Query Planner lädt den `CommitNode` des Zielzeitpunkts und führt die hybride Abfrage exakt auf dessen `active_segments` und `entity_tree_root` aus.
    

### Lock-Free Concurrency & MVCC-Transaktionen in Rust

1. **Wait-Free Reads via `arc-swap` & Epoch-Based Reclamation:**
    
    - Der aktive Zustand jedes Branches liegt hinter einem `arc_swap::ArcSwap<BranchState>`.
        
    - Ein Reader ruft `let guard = crossbeam_epoch::pin();` auf und lädt den `BranchState` komplett lock-free. Selbst unter maximaler Schreiblast blockieren Reader und Writer sich gegenseitig zu keinem Zeitpunkt.
        
2. **Optimistic Concurrency Control (OCC) / Serializable Snapshot Isolation (SSI):**
    
    - Eine Schreibtransaktion öffnet einen lokalen `WriteBatch` auf Basis von `start_commit`.
        
    - Während der Transaktion zeichnet ein leichtgewichtiger **Conflict Tracker** das Read-Set (gelesene `EntityId`-Präfixe und traversierte Graph-Neighborhood-Hashes) sowie das Write-Set auf.
        
    - Beim Commit vergleicht der Transaction Coordinator das Read-Set gegen alle Commits zwischen `start_commit` und dem aktuellen Branch-`HEAD` (mittels `Blake3`-Diff auf dem Merkle-ART in Mikrosekunden). Liegt kein Konflikt vor, wird der neue `CommitNode` per `compare_exchange` als neuer `HEAD` installiert.
        
3. **Reachability-basierte Garbage Collection:**
    
    - Ein Background-Compactor verschmilzt kleine IMMS-Segmente zu größeren (Leveled Compaction). Alte Segmente und verwaiste Merkle-Knoten werden nicht sofort gelöscht, sondern in eine **Retire-Queue** gestellt.
        
    - Physisches `unlink()` erfolgt erst, wenn (a) kein Branch-`HEAD` und kein konfigurierter Time-Travel-Retention-Anchor mehr auf den `CommitNode` verweist und (b) `crossbeam_epoch` bestätigt, dass alle aktiven Reader-Threads die betroffene Epoch verlassen haben.
        

### Semantischer 3-Wege-Merge für Vektoren, Graphen & Text

Beim Zusammenführen eines Agenten-Branches (`MERGE agent_branch INTO main`) sucht die Engine zunächst den **Lowest Common Ancestor (LCA)** im Merkle-DAG und überspringt dank der Merkle-Hashes alle unveränderten Teilbäume in $O(1)$. Für divergierende Teilbäume greifen modalitätsspezifische Merge-Regeln:

  

- **Property-Graph (CRDT OR-Set-Semantik):** Das Hinzufügen neuer Knoten und Kanten ist kommutativ und wird konfliktlos vereinigt. Ein echter Konflikt entsteht nur, wenn beide Branches dasselbe funktionale Attribut einer Kante (z. B. `weight` oder `status`) unterschiedlich mutiert haben.
    
- **Vektoren & Text-Chunks (Semantische Äquivalenzprüfung):** Haben beide Branches denselben Chunk-Key modifiziert, berechnet AethelDB die Kosinus-Ähnlichkeit zwischen den neuen Embeddings $\vec{v}_{\text{main}}$ und $\vec{v}_{\text{branch}}$. Liegt $\cos(\vec{v}_{\text{main}}, \vec{v}_{\text{branch}}) \ge 0.96$ (inhaltlich identische Paraphrase), wählt die Engine deterministisch den Eintrag mit höherem `ProvenanceRecord`-Konfidenzwert. Bei echter semantischer Divergenz wird ein typisiertes `MergeConflict`-Objekt erzeugt, das einem Supervisor-Agenten zur Auflösung vorgelegt wird.
    

## 4. LLM- & Agenten-Spezialfunktionen

AethelDB verlagert Schutzmechanismen gegen typische LLM-Fehlermuster aus brüchigem Prompt-Engineering direkt in die Storage- und Query-Engine.

  

### 1. Kryptografisches Provenance-Tracking & Trust-Propagation

Um Halluzinationen und „Quellen-Amnesie“ zu verhindern, erzwingt die Engine auf Storage-Ebene ein `ProvenanceRecord` für jeden Chunk und jede Kante:

  

Rust

```
pub enum OriginKind {
    DeterministicIngest { parser_version: u32 },
    LlmExtraction { model_id: u32, prompt_hash: ContentHash },
    ToolExecution { tool_name_hash: u64, invocation_id: EntityId },
    HumanVerified { reviewer_id: u64 },
}

pub struct ProvenanceRecord {
    pub source_uri_hash: ContentHash,
    pub source_byte_span: (u64, u64),     // Exakte Position im Quelldokument
    pub origin: OriginKind,
    pub confidence: f32,                  // [0.0, 1.0]
    pub ed25519_sig: [u8; 64],            // Signatur des erzeugenden Agenten/Workers
}
```

- **Multi-Hop Confidence Decay:** Traversiert ein Agent einen Reasoning-Pfad $P = (e_1, e_2, \dots, e_k)$ über mehrere abgeleitete Kanten, berechnet die Engine automatisch den kumulierten Vertrauenswert:
    
    $$C(P) = \left(\prod_{i=1}^{k} \text{confidence}(e_i)\right) \cdot \gamma^{k-1} \quad (\text{mit } \gamma \in (0, 1])$$
    
- **Query-Level Trust Gates:** Queries können per `REQUIRE MIN_CONFIDENCE 0.85 AND ORIGIN IN (DeterministicIngest, HumanVerified)` unbestätigte LLM-Hypothesen hart ausfiltern.
    

### 2. Token-Aware Query Planner & Context-Window-Optimizer

Queries akzeptieren ein explizites Token-Budget (`BUDGET 4096 TOKENS FOR "o200k_base"`). Die Engine optimiert das Resultset in vier Stufen:

1. **Zero-Overhead Token-Accounting:** Beim Ingest werden die exakten Token-Längen für gängige Tokenizer-Familien im Segment-Header gespeichert – zur Query-Zeit entfällt teures Re-Tokenisieren komplett.
    
2. **Hierarchische Multi-Resolution-Pyramide:** Chunks sind über `SUMMARIZES`-Kanten mit verdichteten Zusammenfassungen verknüpft. Überschreitet die Menge relevanter Detail-Chunks das Token-Budget, ersetzt der Planner automatisch den weniger relevanten „Long Tail“ durch die zugehörigen Eltern-Zusammenfassungen, um die semantische Abdeckung zu maximieren.
    
3. **Submodular MMR-Deduplizierung:** Mittels _Maximal Marginal Relevance_ auf den im RAM liegenden SQ8-Vektoren entfernt der Planner Chunks, deren Ähnlichkeit zu bereits selektierten Top-Treffern $> 0.92$ beträgt.
    
4. **U-Shaped Reordering (Anti-Lost-in-the-Middle):** Da LLMs Informationen am Anfang und Ende des Prompts deutlich zuverlässiger abrufen als in der Mitte, ordnet der Planner die finalen $N$ selektierten Chunks nicht monoton absteigend $[r_1, r_2, r_3, r_4, r_5, r_6]$, sondern **bi-modal U-förmig**: $[r_1, r_3, r_5, \dots, r_6, r_4, r_2]$. Die höchsten Relevanz-Scores liegen dadurch an den Aufmerksamkeits-Rändern des Context Windows.
    

### 3. Ephemeral Agent Sandboxes & Speculative Execution

- **Session-Isolation:** Jede autonome Agenten-Session startet standardmäßig auf einem **Ephemeral Branch** (`ephemeral/<session_id>`) mit festem Speicher-Quota und Time-To-Live (TTL).
    
- **Commit-Gate-Validierung:** Alle Zwischenschritte, Tool-Outputs und extrahierten Fakten landen in diesem isolierten Branch. Erst wenn ein Validierungs-Gate (z. B. Schema-Check, Widerspruchsfreiheit, Mindest-Konfidenz) bestanden ist, wird der Branch atomar in `main` gemergt.
    
- **Atomares Verwerfen:** Läuft ein Agent in eine Fehlerschleife oder halluziniert ungültige Relationen, genügt ein `DROP BRANCH` in $O(1)$ – die globale Wissensbasis (`main`) bleibt vollständig unberührt.
    

### 4. Bitemporales Indexing, Time-Decay & Widerspruchserkennung

- **Bitemporales Modell:** Jedes Faktum führt zwei Zeitachsen: `tx_time` (wann die Datenbank es gespeichert hat) und `valid_time` (`valid_from`, `valid_to` – wann es fachlich gültig war).
    
- **Time-Decay-Gewichtung:** Für zeitkritische Abfragen gewichtet der Scorer die hybride Relevanz mit einer konfigurierbaren Halbwertszeit $T_{1/2}$:
    
    $$S_{\text{final}}(r, t_{\text{query}}) = S_{\text{RRF}}(r) \cdot 2^{-\frac{\max(0, t_{\text{query}} - t_{\text{valid}}(r))}{T_{1/2}}}$$
    
- **Write-Time Contradiction Detection:** Beim Einfügen einer neuen Kante `(Subjekt, Prädikat, Objekt_neu)` prüft der Transaction Manager über den `ForwardCSR`-Index in Mikrosekunden, ob für als **funktional** markierte Prädikate (z. B. `CURRENT_STATUS`, `API_ENDPOINT`, `HAS_OWNER`) bereits eine aktive Kante `(Subjekt, Prädikat, Objekt_alt)` mit `valid_to = None` existiert.
    
    - Weicht `Objekt_neu` ab, wird automatisch eine `CONTRADICTS`-Relation angelegt. Ist `ProvenanceRecord.confidence` der neuen Aussage höher oder ihr `valid_from` neuer, schließt die Engine das alte Faktum bitemporal (`valid_to = new.valid_from`), statt es zu löschen – historische Time-Travel-Queries bleiben dadurch konsistent.
        

## 5. Konkreter Rust-Technologie-Stack & Architektur-Trade-offs

### Empfohlener Rust-Crate-Stack

|**Subsystem**|**Empfohlene Crates**|**Technischer Einsatzzweck**|
|---|---|---|
|**Async Runtime & Direct I/O**|`tokio`, `tokio-uring` / `monoio`|`tokio` für Control-Plane, RPC & Background-Tasks; Thread-per-Core `io_uring` (`O_DIRECT`) für lock-freie NVMe-Page-Reads und Group-Commit WAL.|
|**Zero-Copy & Columnar Memory**|`rkyv`, `arrow`, `bytes`|`rkyv` für Zero-Copy-Zugriff auf CSR-Graph-Header, Vamana-Pages & Merkle-ART im Buffer Pool; Apache Arrow für Vektor-Blobs und SIMD-Filterung.|
|**Lock-Free Concurrency**|`crossbeam-epoch`, `arc-swap`, `papaya`, `parking_lot`|Epoch-Based Reclamation für MVCC-Reader, wait-free `HEAD`-Pointer-Swaps, lock-freie Concurrent Hash Maps für aktive MemTables.|
|**Indizes & Bitmaps**|`roaring`, `fst`, `tantivy` (Low-Level Codecs)|`RoaringBitmap` für Graph-Pre-Filtering; `fst` (Finite State Transducer) & SIMD-BP128 Bitpacking für den Block-Max WAND Volltext-Index.|
|**SIMD & Kompression**|`simsimd` / `std::simd`, `zstd`, `lz4_flex`|Hardwarebeschleunigte Kosinus-, L2- und Hamming-Distanzen (AVX-512 / ARM NEON); Zstd-Dictionary-Kompression für Text-Payloads.|
|**Kryptografie & Hashing**|`blake3`, `ed25519-dalek`|SIMD-parallelisiertes Content-Addressing (> 5 GB/s/Core) für den Merkle-DAG; Ed25519-Signaturen für Provenance-Records.|

### Ehrliche Architektur-Trade-offs

1. **Write-Amplification vs. Multi-Modal Query-Latenz (IMMS-Compaction):**
    
    - _Zielkonflikt:_ Im Gegensatz zu sortierten Key-Value-Paaren lassen sich HNSW-Graphen und CSR-Adjazenzlisten nicht linear durch einfaches Merge-Sort verschmelzen. Beim Compaction-Vorgang mehrerer Segmente müssen `InternalRowId`s neu vergeben, CSR-Offsets neu berechnet und Vamana-Nachbarschaften partiell neu verknüpft werden, was CPU-Last und Write-Amplification erhöht.
        
    - _Lösung:_ Tiered Compaction mit lazy Merging – kleine Segmente werden beim Read parallel über `RoaringBitmap`-Merge abgefragt; eine physische Re-Indizierung erfolgt erst im Hintergrund, wenn die Segment-Anzahl pro Level einen Schwellenwert ($K \ge 8$) überschreitet.
        
2. **RAM-Verbrauch vs. Tail-Latenz (Vektor-Indizes):**
    
    - _Zielkonflikt:_ Ein vollständiger FP32-HNSW-Graph für 100 Millionen 1536-dimensionale Vektoren benötigt über 600 GiB RAM.
        
    - _Lösung:_ Asymmetrisches Tiering. Im RAM verbleiben nur die oberen HNSW-Navigationsebenen sowie SQ8- oder Binary-Quantized Vektoren (Faktor 4x bis 32x Kompression). Layer 0 liegt im Vamana-Co-Location-Layout auf NVMe-SSDs. _Trade-off:_ Die p99-Latenz steigt bei Cold-Cache-Reads um ca. $0.5\text{–}1.5\text{ ms}$ durch NVMe-IOPS, dafür sinkt der RAM-Bedarf um bis zu 90 %.
        
3. **Semantische Merge-Konflikte vs. Agenten-Parallelität:**
    
    - _Zielkonflikt:_ Wenn dutzende Agenten gleichzeitig auf demselben Subgraphen operieren, führt strikte Serializable Snapshot Isolation (SSI) zu häufigen OCC-Retries.
        
    - _Lösung:_ Aufspaltung der Datenmodell-Operationen in **kommutative CRDT-Operationen** (neue Chunks, neue Beobachtungskanten -> konfliktloser Auto-Merge) und **nicht-kommutative State-Mutationen** (Überschreiben funktionaler Kanten), kombiniert mit feingranularem Prefix-Conflict-Tracking im Merkle-ART.
        
4. **CPU-Overhead durch Inline-Provenance & Widerspruchsprüfung:**
    
    - _Zielkonflikt:_ Kryptografisches Hashing, Signaturprüfung und Konsistenzchecks auf dem Write-Path kosten CPU-Zyklen und erhöhen die Commit-Latenz.
        
    - _Lösung:_ `Blake3` arbeitet SIMD-parallelisiert und fällt kaum ins Gewicht; teurere Prüfungen werden zweistufig getrennt: Ein deterministischer, struktureller Check (`Subject + FunctionalPredicate` im CSR-Index in $< 5\,\mu\text{s}$) läuft synchron beim Write, während tiefere semantische NLI-Widerspruchsprüfungen asynchron vor dem Branch-Merge auf dem Ephemeral Branch ausgeführt werden.