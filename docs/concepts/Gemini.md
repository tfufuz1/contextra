Im Folgenden wird die vollständige Systemarchitektur und technische Spezifikation für **KortexDB** beschrieben – eine von Grund auf in Rust entwickelte, transaktionale Multi-Model-Storage-Engine für autonome KI-Agenten und Retrieval-Augmented Generation (RAG).

  

## 1. High-Level Architektur & Schichtenmodell

KortexDB kombiniert eine **Log-Structured Merge-Tree (LSM)**-Architektur mit unveränderlichen (immutable), inhaltsadressierten **Copy-on-Write (CoW) Segmenten** und einem **Merkle-DAG-Katalog**. Während LSM-Trees hohen Schreibdurchsatz für kontinuierliche Agenten-Beobachtungen garantieren, ermöglichen unveränderliche Segmente in Kombination mit kryptografischen Hashes (`BLAKE3`) ein verzögerungsfreies Zero-Copy-Branching und Lock-free Snapshot Isolation.

  

### Schichtenmodell

|**Schicht**|**Verantwortung & Kernkomponenten**|
|---|---|
|**1. Client & Agent Protocol**|Embedded Rust-Crate (`kortex-core`), gRPC (`tonic`) sowie Apache Arrow Flight. Native Unterstützung für Ephemeral-Branch-Sessions, Provenance-Header und Token-Budgets.|
|**2. Token-Aware Query Planner**|Kostenbasierter Hybrid-Optimizer. Zerlegt Anfragen in Graph-, Vektor-, Volltext- und Temporal-Prädikate, fusioniert Rankings via Reciprocal Rank Fusion (RRF) und packt Ergebnisse kontextoptimiert.|
|**3. MVCC & Merkle-DAG Engine**|Verwaltet Branch-Referenzen (`main`, `ephemeral/<session>`), Snapshot Isolation (SI), 3-Wege-Semantic-Merges, Time-Travel-Snapshots sowie Reachability-Garbage-Collection.|
|**4. Unified Multi-Model Index**|Co-lokalisiert Dense Vectors (HNSW / DiskANN), Dual-CSR Property-Graphs und FST-basierte Inverted Indexes über einen gemeinsamen 128-Bit-Primärschlüsselraum (`NodeId`).|
|**5. Async Storage & Buffer Pool**|`io_uring`-Datenpfad mit `O_DIRECT`, lock-freier SIEVE/ARC-Buffer-Pool, `rkyv` Zero-Copy-Deserialisierung und gruppenkommittiertes Write-Ahead-Log (WAL).|

### Datenfluss: Write-Path & Read-Path

**Write-Path (ACID-konform & Lock-free):**

  

1. **Session & Snapshot-Bindung:** Eine Schreibtransaktion pinnt den aktuellen `CommitHash` des Ziel-Branches und generiert einen monoton steigenden Timestamp über eine **Hybrid Logical Clock (`HLC`)**.
    
2. **Group-Commit WAL:** Alle Mutationen (Chunk-Text, Embedding-Vektor, Graph-Kanten, Provenance-Metadaten, Token-Count) werden via `rkyv` serialisiert, mit `BLAKE3` prüfsummiert und über `io_uring` gebündelt (`fsync`-Fenster $\le 2\text{ ms}$) ins WAL geschrieben.
    
3. **In-Memory Multi-Model MemTable:** Parallel zum WAL landen die Deltas in einer lock-freien, epoch-verwalteten SkipList (`crossbeam-epoch`) sowie einem leichtgewichtigen In-Memory-Delta-HNSW und inkrementellen Posting-Listen.
    
4. **Asynchroner Flush & Merkle-Commit:** Erreicht die MemTable 64 MiB, serialisiert ein Hintergrund-Worker sie in ein unveränderliches L0-Segment (`.kseg`), berechnet dessen `BLAKE3`-Hash und aktualisiert den Branch-Pointer atomar per Compare-And-Swap (`CAS`) auf einen neuen `CommitNode` im Merkle-DAG.
    

**Read-Path (Zero-Copy & Lock-free):**

  

1. **Snapshot-Pinning:** Der Leser erwirbt ein unveränderliches `Arc<CommitNode>` ohne jegliche Reader-Locks.
    
2. **Prädikat-Pushdown & Tri-Modal Scan:** Der Query Planner wertet Graph-Traversierungen und Provenance-/Zeitfilter zuerst aus, erzeugt eine kompakte `RoaringBitmap` zulässiger Kandidaten und führt anschließend die Vektor- und BM25-Suche direkt auf den gefilterten Segmenten aus.
    
3. **Zero-Copy Hydration:** Seiten aus dem SIEVE-Buffer-Pool (oder via `io_uring` `IORING_OP_READ_FIXED` nachgeladen) werden ohne Parsing-Overhead direkt als `rkyv::Archived<T>`-Referenzen im Speicher interpretiert.
    
4. **RRF, Decay & Context Packing:** Die Treffer werden über Reciprocal Rank Fusion zusammengeführt, mit Time-Decay und Provenance-Konfidenz gewichtet, dedupliziert und für das Ziel-LLM arrangiert.
    

## 2. Physisches Storage- & Index-Design

Um Synchronisationsprobleme und Speicherduplikation zwischen separaten Systemen auszuschließen, existiert in KortexDB jedes Wissenselement physisch exakt einmal.

  

### Kanonische Adressierung (`NodeId`)

Jeder Chunk, jede Entität und jede Agenten-Beobachtung erhält eine eindeutige 128-Bit-`NodeId` (UUIDv7-kompatibel: 48-Bit-Unix-Millisekunden + 80-Bit-Entropie für zeitliche Lokalität). Alle Indexstrukturen verweisen ausschließlich auf diese `NodeId` bzw. auf einen segmentlokalen `u32`-Slot-Index.

  

Rust

```
#[repr(C)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[rkyv(derive(Debug))]
pub struct NodeId(pub u128);

#[repr(C)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SegmentManifest {
    pub segment_hash: [u8; 32],       // BLAKE3 über alle Datenblöcke
    pub node_count: u32,
    pub payload_block_offset: u64,    // Spaltenorientierter Text & Provenance
    pub vector_zone_offset: u64,      // PQ-Codes + DiskANN L0 Adjacency
    pub graph_csr_offset: u64,        // Forward & Reverse CSR Arrays
    pub text_fst_offset: u64,         // FST Term Dictionary + SIMD-BP128 Postings
    pub min_hlc: u64,
    pub max_hlc: u64,
}
```

### Aufbau eines Multi-Model-Segments (`.kseg`)

Jedes Segment ist auf 4-KiB-Speicherseiten ausgerichtet (`PAGE_SIZE = 4096`), um `O_DIRECT`-DMA-Transfers ohne Kernel-Page-Cache-Kopien zu ermöglichen:

  

1. **Primary Payload & Provenance Zone:** Speichert Nutzdaten (Rohtext, hierarchische Zusammenfassungen, kryptografische Provenance-Ketten, Token-Counts) in Zstandard-komprimierten (mit trainiertem Dictionary) 16-KiB-Blöcken.
    
2. **Vector Index Zone (Tiered HNSW + DiskANN):**
    
    - **RAM-resident:** Obere Graph-Ebenen ($L_1 \dots L_{\max}$) sowie 16-Byte **Product Quantization (PQ)**-Codes aller Vektoren liegen dauerhaft im Buffer Pool. Dies reduziert den RAM-Bedarf bei 1536-dimensionalen `f32`-Vektoren von ~6 KiB auf 16 Bytes pro Vektor (Faktor 384).
        
    - **NVMe-resident:** Die Basisebene ($L_0$) mit Adjazenzlisten und Vollpräzisionsvektoren (`f16`/`f32`) liegt seitenausgerichtet auf SSD und wird während der Beam-Search per gebündelten `io_uring`-Reads (`IOPOLL`) nachgeladen.
        
3. **Property-Graph Zone (Dual Compressed Sparse Row – CSR):**
    
    - Kanten sind als unveränderliche **Forward-CSR** (`src_slot -> [edge_type_id, dst_slot, valid_from, valid_to, prop_offset]`) und **Reverse-CSR** (`dst_slot -> ...`) abgelegt.
        
    - Da `rkyv` exakte Speicher-Layouts garantiert, entspricht eine 1-Hop-Nachbarschaftsabfrage einem Zero-Copy-Slice-Zugriff (`&[ArchivedEdge]`) in $O(1)$ ohne Pointer-Chasing.
        
4. **Full-Text Index Zone (FST + Block-Max WAND):**
    
    - Das Term-Vokabular ist als **Finite State Transducer (`fst`)** gespeichert, der normalisierte Tokens auf Byte-Offsets der Posting-Listen abbildet.
        
    - Posting-Listen nutzen **SIMD-BP128**-Bitpacking für `u32`-Slots und quantisierte Termfrequenzen für hardwarebeschleunigtes **Block-Max WAND / BM25**-Scoring.
        

### Hybride Query-Ausführung in einer Transaktion

Klassische Systeme führen getrennte Top-$K$-Suchen durch und schneiden die Ergebnisse nachträglich (Post-Filtering), was bei selektiven Graph-Filtern zu massivem Recall-Verlust führt. KortexDB nutzt einen **adaptiven Tri-Modal Join**:

  

1. **Pre-Filtering via Graph & Metadaten:** Eine Multi-Hop-Traversierung auf den CSR-Arrays sowie Filter auf Provenance und `valid_time` erzeugen eine `RoaringBitmap` gültiger Kandidaten (`CandidateMask`).
    
2. **Kostenbasierter Algorithmus-Switch:**
    
    - **Hohe Selektivität (`CandidateMask.len() < 5_000`):** Umschaltung auf einen exakten **SIMD-Brute-Force-Scan** (`AVX-512` / `NEON` Dot-Product) ausschließlich über die maskierten Kandidaten plus exaktes BM25-Scoring. Resultat: 100 % Recall ohne Graph-Verzerrung.
        
    - **Niedrige Selektivität (`CandidateMask.len() >= 5_000`):** Ausführung einer **Filtered-HNSW / ACORN Beam Search**, bei der Nachbarknoten gegen `CandidateMask` geprüft werden und der Suchstrahl dynamisch skaliert wird ($ef' = \min(ef_{\max}, \lceil ef / \text{Selektivität} \rceil)$).
        
3. **Score-Fusion (RRF + Temporal + Provenance):**
    
    $$\text{FinalScore}(d) = \left( \frac{w_v}{k + r_v(d)} + \frac{w_t}{k + r_t(d)} + \frac{w_g}{k + r_g(d)} \right) \cdot e^{-\lambda (t_{\text{now}} - t_{\text{valid}}(d))} \cdot C_{\text{prov}}(d)$$
    

## 3. Versionierungs- & Concurrency-Engine

KortexDB behandelt den gesamten Datenbankzustand als unveränderlichen **Merkle-DAG** über einem **Copy-on-Write Prolly Tree** (Probabilistic B+Tree), wodurch Git-Semantik nativ auf Storage-Ebene entsteht.

  

Rust

```
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CommitNode {
    pub commit_hash: [u8; 32],         // BLAKE3(manifest_root || parents || hlc || meta)
    pub parents: Vec<[u8; 32]>,         // 1 Parent (regulär) oder 2 Parents (Merge-Commit)
    pub manifest_root: [u8; 32],        // Root-Hash des Prolly Trees (Segment-Mapping)
    pub hlc_timestamp: u64,
    pub agent_session_id: Option<u128>,
    pub provenance_sig: [u8; 64],       // Ed25519-Signatur des Commits
}
```

### Zusammenspiel von MVCC, Merkle-DAG und Git-Operationen

- **Zero-Copy-Branching (`CREATE BRANCH`):** Ein Branch ist lediglich ein benannter atomarer Pointer (`ArcSwap<CommitNode>`) in einer lock-freien Hash-Tabelle (`papaya::HashMap`). Das Abzweigen eines Branches (`CREATE BRANCH exp_1 FROM main`) kopiert 0 Bytes Nutzdaten und benötigt $< 1\,\mu\text{s}$.
    
- **Point-in-Time-Queries (Time-Travel):** Queries akzeptieren `AS OF COMMIT <hash>` oder `AS OF TIMESTAMP <hlc>`. Die Engine traversiert die unveränderlichen `CommitNode`-Elternzeiger (beschleunigt durch einen Commit-Skip-Index), lädt den zugehörigen `manifest_root` und öffnet einen Snapshot über exakt jene `.kseg`-Dateien, die zu diesem Zeitpunkt gültig waren.
    
- **Sublineare Diffs (`DIFF commit_a..commit_b`):** Da der Prolly Tree inhaltsadressiert ist, besitzen unveränderte Teilbäume identische `BLAKE3`-Hashes. Beim Diff zweier Commits werden identische Subtrees in $O(\log N)$ übersprungen – es werden ausschließlich tatsächlich modifizierte Blatt-Segmente inspiziert.
    
- **Atomare Rollbacks (`RESET BRANCH`):** Ein Rollback setzt den Branch-Pointer per WAL-protokolliertem `CAS` auf einen früheren `CommitHash` zurück. Laufende Leser auf alten Snapshots arbeiten dank `crossbeam-epoch` ungestört zu Ende.
    
- **Reachability Garbage Collection (Vacuum):** Ein asynchroner GC-Prozess markiert ausgehend von allen aktiven Branch-Heads, Tags und nicht abgelaufenen Ephemeral-Branches alle erreichbaren `CommitHash`- und `SegmentHash`-Knoten. Nicht mehr referenzierte `.kseg`-Dateien werden sicher vom Dateisystem entfernt.
    

## 4. LLM- & Agenten-Spezialfunktionen

KortexDB verlagert Schutzmechanismen gegen typische Fehlermuster autonomer Agenten direkt in die Storage- und Query-Schicht.

  

### 1. Kryptografisches Provenance-Tracking (Anti-Halluzination)

Jeder Chunk und jede Graph-Kante besitzt einen unveränderlichen `ProvenanceRecord`. Synthetisiert ein Agent aus drei Quellen eine neue Schlussfolgerung, speichert KortexDB die `NodeId`s der Ausgangsfakten in `derivation_chain` und berechnet die Konfidenz konservativ über die Kette hinweg:

  

$$C_{\text{derived}} = \alpha_{\text{decay}} \cdot \min_{p \in \text{Parents}} C(p)$$

Queries können harte Herkunftsschranken erzwingen (z. B. `FILTER provenance.max_derivation_depth <= 1 AND provenance.confidence >= 0.85`). Dadurch wird verhindert, dass ein Agent eigene, unbestätigte Halluzinationen aus früheren Läufen abruft und als Fakten verstärkt (_Recursive Model Collapse_).

  

### 2. Token-Aware Query Planner & „Lost-in-the-Middle“-Optimierung

Statt starrer `TOP K`-Limits optimiert der Planner die Ergebnismenge auf ein exaktes Token-Budget (`MAX_TOKENS`) für das jeweilige Zielmodell (Token-Counts werden beim Ingest via `tiktoken-rs` vorberechnet):

  

1. **Maximal Marginal Relevance (MMR) Deduplizierung:** Redundante Chunks mit einer Kosinus-Ähnlichkeit $> 0.92$ zu bereits gewählten Treffern werden unterdrückt, um die Informationsdichte pro Token zu maximieren.
    
2. **Hierarchische Abstraktionsstufen:** Jeder Chunk speichert neben dem `raw_text` eine kompakte `extractive_summary` sowie strukturierte `entity_triples`. Reicht das Token-Budget nicht für alle Treffer im Volltext, erhalten die Top-Treffer den `raw_text`, während ergänzende Kontext-Nachbarn automatisch auf `extractive_summary` herunterskaliert werden.
    
3. **U-Curve Reordering (Anti-Lost-in-the-Middle):** Da Transformer-Modelle Informationen am **Anfang** und am **Ende** des Context-Windows signifikant zuverlässiger verarbeiten als in der Mitte, sortiert der Planner die finalen $N$ Chunks nicht linear absteigend `[1, 2, 3, 4, 5, 6]`, sondern U-förmig: **`[1, 3, 5, 6, 4, 2]`**. Die beiden relevantesten Anker-Fakten stehen somit an den Aufmerksamkeits-Maxima des Prompts.
    

### 3. Ephemeral Sandbox Branches & Guarded Merge Gates

Jede Agenten-Session startet automatisch in einem isolierten **Ephemeral Branch** (`ephemeral/<session_id>`) mit konfigurierbarem TTL und Speicherlimit:

- Fehlerhafte Tool-Calls, Endlosschleifen oder halluzinierte Graph-Kanten isolieren sich vollständig im Sandbox-Branch. Schlägt die Task-Validierung fehl, genügt ein `DROP BRANCH` in $O(1)$.
    
- Der Merge in `main` durchläuft ein **Declarative Merge Gate**, das Schema-Integrität, Mindest-Provenance-Tiefe und Widerspruchsfreiheit prüft, bevor der Merge-Commit atomar sichtbar wird.
    

### 4. Temporales Indexing & Automatische Widerspruchserkennung

- **Bi-Temporale Datenhaltung:** Jede Fakt-Kante führt `valid_time` (fachliche Gültigkeit in der realen Welt) und `transaction_time` (Systemzeitpunkt der Speicherung) sowie eine domänenspezifische Halbwertszeit $t_{1/2}$ für das Time-Decay-Scoring.
    
- **Write-Time Contradiction Detection:** Ist ein Prädikat im Graph-Schema als funktional deklariert (z. B. `(Kunde_42, aktueller_tarif, "Basic")`) und fügt ein Agent `(Kunde_42, aktueller_tarif, "Enterprise")` mit überlappender `valid_time` ein, erkennt die Engine den Konflikt beim Schreibzugriff:
    
    - _Auto-Supersede-Modus:_ Die alte Kante erhält `valid_time.end = now` und wird über eine `SUPERSEDED_BY`-Kante mit dem neuen Fakt verknüpft (lückenlose Historie bleibt über Time-Travel abfragbar).
        
    - _Strict-Guard-Modus:_ Der Write gibt ein strukturiertes `ContradictionError`-Objekt mit beiden Provenance-Ketten an den Agenten zurück, damit dieser den Widerspruch aktiv auflöst.
        

## 5. Konkreter Rust-Technologie-Stack & Architektur-Trade-offs

### Empfohlener Rust-Crate-Stack

|**Subsystem**|**Empfohlene Crates**|**Technischer Einsatzzweck**|
|---|---|---|
|**Async & Storage I/O**|`tokio`, `tokio-uring` / `monoio`|Trennung von Control-Plane (`tokio`) und `io_uring` `O_DIRECT`-Datenpfad mit registrierten DMA-Puffern.|
|**Zero-Copy & Formate**|`rkyv`, `bytecheck`, `arrow`, `parquet`|Direkter Speicherzugriff auf SSTable-Header, CSR-Graphen und WAL-Frames ohne Deserialisierung; Arrow für Query-Output.|
|**Lock-Free Concurrency**|`crossbeam-epoch`, `papaya`, `arc-swap`|Sichere Speicherfreigabe ohne GC-Pausen; atomare Branch-Pointer-Updates; lock-freie Hash-Maps.|
|**SIMD & Bitmaps**|`simsimd`, `roaring`, `std::arch`|Hardwarebeschleunigte Vektordistanzen (`AVX-512` / `NEON`) und rasante Mengenoperationen auf Kandidaten-IDs.|
|**Text & Tokenizing**|`fst`, `tiktoken-rs`, `zstd`|Kompakte Term-Dictionaries auf Disk, präzises Ingest-Token-Counting und Dictionary-Kompression für Chunks.|
|**Kryptografie & RPC**|`blake3`, `ed25519-dalek`, `tonic`|SIMD-paralleles Hashing für den Merkle-DAG, Commit-Signaturen und gRPC-Schnittstelle.|

### Ehrliche Abwägung technischer Zielkonflikte

1. **Write-Amplification vs. Suchlatenz (LSM-Compaction von Multi-Model-Segmenten):**
    
    - _Zielkonflikt:_ Das vollständige Neubauen von HNSW-Graphen und FST-Indizes bei jedem LSM-Compaction-Schritt erzeugt hohe CPU- und I/O-Write-Amplification.
        
    - _Architekturentscheidung:_ **Asymmetrische Tiered Compaction**. In L0 und L1 (kleine, kurzlebige Segmente bis 64 MiB) verzichtet KortexDB auf den Aufbau eines HNSW-Graphen und nutzt stattdessen einen linearen SIMD-PQ-Scan. Erst bei der Compaction in kalte Ebenen ($\ge \text{L2}$, ab 256 MiB) wird ein vollständiger DiskANN-Graph konstruiert. Dies senkt die Write-Amplification um Faktor ~5 bei minimalem Latenzanstieg ($< 0.4\text{ ms}$) in L0/L1.
        
2. **RAM-Verbrauch vs. NVMe-Latenz (HNSW vs. DiskANN):**
    
    - _Zielkonflikt:_ Ein reiner In-Memory-HNSW-Index benötigt inklusive Graph-Adjazenzlisten rund das 1,5- bis 2-Fache der Rohvektorgröße im RAM – bei 100 Millionen Vektoren sind das über 900 GiB RAM.
        
    - _Architekturentscheidung:_ **Hybrid PQ-in-RAM + Full-Vector-on-NVMe**. Im RAM verbleiben nur die 16-Byte-PQ-Codes und die oberen Graph-Ebenen (~2,5 GiB für 100 Mio. Vektoren). Die präzisen Vektoren der Basisebene werden während der Beam Search via `io_uring` gebündelt von NVMe nachgeladen, was die p99-Latenz unter 3 ms hält und die Hardwarekosten drastisch reduziert.
        
3. **Merge-Konflikte bei Graphen und Vektoren:**
    
    - _Zielkonflikt:_ Während Quellcode zeilenbasiert gemerged werden kann, sind binäre HNSW-Adjazenzlisten und hochdimensionale Vektorräume nicht sinnvoll auf Byte-Ebene mergebar.
        
    - _Architekturentscheidung:_ **Logisches Delta-Replay statt binärem Index-Merge**. Beim Merge eines Sandbox-Branches (`LCA -> Sandbox`) extrahiert KortexDB ausschließlich die logischen `(NodeId, Vector, Edge)`-Mutationen aus dem Branch-Diff. Nicht-funktionale Graph-Kanten folgen einer kommutativen CRDT-Semantik (_Add-Wins Set_ mit Tombstone-Tracking), funktionale Kanten prüfen Schema-Kardinalitäten, und neue Vektoren werden direkt in die Delta-MemTable des Ziel-Branches eingespielt (typischerweise hunderte bis wenige tausend Vektoren, Dauer $< 10\text{ ms}$).