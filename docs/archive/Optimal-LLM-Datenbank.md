# Claude
# Engram: eine Storage-Engine für LLM-Agenten (Rust)

## 1. Designthese

RocksDB und Co. optimieren für Key-Value-Zugriffe von Programmen. LLM-Agenten arbeiten anders:

| Programm | LLM-Agent |
|---|---|
| exakte Keys | unscharfe Anfragen (semantisch, assoziativ) |
| beliebig große Ergebnisse | **Token-Budget** pro Abfrage (Kontextfenster) |
| deterministisch | spekulativ: probiert, verwirft, korrigiert |
| vertraut den Daten | Daten können **Prompt-Injection** enthalten |
| Daten ändern sich | Daten werden **abgeleitet** (Summaries, Embeddings) und müssen reproduzierbar bleiben |

Die zentrale Idee: Die Datenbank liefert keine Zeilen, sondern **fertig gepackten Kontext** (`assemble`). Das Layout auf der Platte ist darauf ausgelegt.

## 2. Datenmodell

Alles ist ein **unveränderliches, inhaltsadressiertes Objekt** (BLAKE3-Hash), organisiert in einem Merkle-DAG. Objekttypen:

- **Doc**: Text als Baum (Dokument → Abschnitte → Chunks), pro Knoten Token-Zahl
- **Fact**: Triple oder Statement mit Bi-Temporalität (`valid_from/to`, `tx_time`)
- **Episode**: Agenten-Trace (Gedanke, Tool-Call, Ergebnis, Beobachtung)
- **Vec**: Embedding mit `model_id`, mehrere pro Objekt möglich
- **Blob**: Bilder, Dateien, KV-Cache-Tensoren
- **Derived**: Summary, Extraktion usw. mit Lineage (Modell, Prompt-Hash, Input-Hashes)

Jedes Objekt trägt ein **Trust-Label** (`system | user | tool | web | model`), das über Ableitungen propagiert wird. Eine Summary aus Web-Inhalten bleibt `web`-tainted.

## 3. Physisches Layout

**3.1 Token-natives Speichern.** Text liegt zusätzlich als Token-ID-Strom pro Tokenizer-Familie vor (zstd mit trainiertem Wörterbuch). Dadurch gibt es kein Re-Tokenisieren beim Packen, exakte Budget-Berechnung aus dem Index heraus und stabile Präfixe für Prompt-Caching.

**3.2 LOD-Cluster („Level of Detail“).** Ein Chunk, seine Summaries (L1/L2/L3), Embeddings und Metadaten liegen in **einem zusammenhängenden Extent**. Progressive Offenlegung (Outline → Abschnitt → Volltext) ist dann ein Sequential-Read statt vieler Random-Reads.

**3.3 Affinitäts-Clustering statt Key-Order.** Segmente werden nach **Ko-Retrieval** sortiert, nicht nach Key. Die Engine loggt, welche Objekte gemeinsam in einem `assemble` landen (Co-Access-Graph) und kombiniert das mit Embedding-Clustern (IVF-Zellen). Die Compaction ordnet physisch um. Die Sortierung ist `(space, cluster_id, hilbert(time), hash)`: ANN-Nachbarn und Dinge, die zusammen gebraucht werden, liegen auf denselben Seiten. Das ist der größte Unterschied zu LSM-Level-Compaction, die nur nach Key sortiert.

**3.4 Segmente.** Immutable Dateien mit festem Aufbau:

```
[Header][Column blocks: tokens | text | meta | trust]
[LOD blocks][Vector block (PQ + full)][FST/BM25 postings]
[Graph adjacency][Footer: Bloom, min/max, Merkle root, checksums]
```

Die Indizes (Volltext, Vektor, Graph) werden **pro Segment** gebaut. Weil Segmente unveränderlich sind, sind die Indizes automatisch versioniert und lassen sich beim Mergen einfach zusammenführen.

## 4. Versionskontrolle im Storage

Die Haupt-Indexstruktur ist ein **Prolly-Tree** (wie bei Dolt/Noms): history-independent, mit Structural Sharing.

- **Commit** = neue Wurzel-Hash, Atomizität über einen einzigen CAS auf den Branch-Pointer.
- **Branch** ist O(1). Jeder Agent-Run bekommt einen eigenen Branch. Rollback ist ein `reset`, kein Aufräumen.
- **Diff** kostet O(Änderungen), nicht O(Daten). Dadurch sieht man genau, was ein Agent verändert hat.
- **Merge** in drei Stufen: (1) strukturell automatisch, (2) typisierte CRDTs für Memory-Listen und Zähler, (3) ein Callback `resolve(conflict)`, den ein LLM oder Mensch bedient.
- **Time-Travel**: `search(... , at=commit)` funktioniert auf jedem Stand, weil auch Indizes versioniert sind.
- **Spekulative Forks** machen Tree-of-Thought oder Parallel-Agenten billig: N Branches, der beste wird gemergt.

Transaktionen sind Snapshot-Isolation: Writes gehen in den WAL und dann in ein neues Segment bzw. einen neuen Baum-Pfad. Die Wurzel wird erst nach `fsync` atomar umgehängt.

## 5. LLM-native API

```rust
pub struct ContextQuery {
    pub intent: String,                 // Freitext
    pub budget: TokenBudget,            // z.B. 6000 für Tokenizer "claude"
    pub policy: PackPolicy,             // diversity, recency, LOD-Strategie
    pub min_trust: Trust,               // z.B. Trust::User
    pub at: Option<CommitId>,
}

pub struct PackedContext {
    pub blocks: Vec<Block>,             // Text + Provenance + Handle + LOD
    pub tokens_used: u32,
    pub expandable: Vec<Handle>,        // "mehr davon" per Handle
    pub prefix_id: PrefixHash,          // stabiler Präfix für Prompt-Cache
}
```

Wichtige Eigenschaften:

- **`assemble`** macht Hybrid-Retrieval (BM25 + ANN + Graph-Expansion), Rerank und Greedy-Knapsack-Packing unter Token-Budget und Diversität. Je nach Budget wählt es pro Treffer das passende LOD.
- **Kurze Handles** (z. B. `d7k2`, eindeutig per Präfix des Hashs) statt UUIDs, weil LLMs lange IDs schlecht kopieren.
- **Fehler als handlungsleitender Text**: „Branch `run-42` existiert nicht. Vorhanden: …“.
- Eingebauter **MCP-Server** und ein schmales Tool-Set: `search`, `read(handle, lod)`, `remember`, `link`, `branch`, `diff`, `commit`.
- **Cursor mit Token-Budget** statt `LIMIT/OFFSET`.

## 6. Cache-Bewusstsein

- Die Packer-Ausgabe ist **deterministisch und präfix-stabil**: stabile Blöcke vorn, volatile hinten. Das erhöht die Trefferrate beim Provider-Prompt-Caching.
- Für lokale Modelle: **KV-Cache-Blobs** content-adressiert, als Radix-Baum über Token-Präfixe (wie SGLang) und mit Tiering GPU/RAM/NVMe/S3.
- Bei Modellwechsel: Embeddings sind mit `model_id` versioniert und werden **lazy re-embedded** in einem Hintergrundjob, während alte Vektoren weiter nutzbar bleiben.

## 7. Memory-Semantik

- Tiers: Working (Branch-lokal, flüchtig), Episodic (Traces), Semantic (Fakten), Procedural (Skills, Prompts).
- **Konsolidierung** als Compaction-Job: Episoden werden zu Facts und Summaries verdichtet, dedupliziert und mit Decay gewichtet. Jede Summary behält Pointer auf die Quellen, sodass Verdichtung **reversibel** bleibt.
- **Bi-Temporalität**: „Was wusste der Agent am Dienstag?“ ist von „Was galt am Dienstag?“ unterscheidbar.
- **Lineage** erlaubt: „Welche Summaries sind veraltet, weil sich Quelle X geändert hat?“ wird als Invalidierungsgraph ausgewertet.

## 8. Sicherheit

- **Taint-Tracking**: Web- und Tool-Inhalte werden in `assemble` markiert oder gefiltert, was Prompt-Injection über Memory eindämmt.
- Capabilities pro Space und Branch (read/append/merge). Ein Agent darf nur auf seinem Branch schreiben, `main` nur per Merge-Gate.
- **Verschlüsselung pro Space** mit eigenem Key. Löschen (DSGVO) in einem unveränderlichen Store geschieht per **Crypto-Shredding** plus Tombstone und anschließendem Rewrite in der Compaction.

## 9. Engine-Internals (Rust)

- **Thread-per-core** mit `io_uring` (z. B. `glommio` oder `tokio-uring`), O_DIRECT und eigener Buffer-Pool statt mmap (kontrollierte Latenz, keine Page-Fault-Überraschungen).
- Zero-Copy-Zugriff auf Segmente über `rkyv` oder eigenes Layout, `zstd`-Dictionaries, `blake3`, `fst`, `roaring`.
- WAL mit Group-Commit, danach Memtable → Segment. Die Memtable hält auch einen kleinen HNSW für frische Vektoren, bevor sie in den Segment-Index (DiskANN/Vamana oder IVF-PQ) gehen.
- Cold Tier = S3: unveränderliche Segmente lassen sich ohne Koordination hochladen und per Range-Request lesen.
- Zwei Betriebsarten: **eingebettet** (wie SQLite, eine Bibliothek pro Agent) und **Server** (gRPC + MCP, Multi-Tenant).

Crate-Aufteilung: `engram-core` (Objekte, Hashing), `-seg` (Segmentformat), `-tree` (Prolly/Branches), `-index` (BM25/ANN/Graph), `-pack` (Assemble), `-wal`, `-tier`, `-mcp`, `-server`.

## 10. Vergleich mit RocksDB

| Aspekt | RocksDB | Engram |
|---|---|---|
| Sortierung | Key | Affinität/Cluster |
| Versionierung | Snapshots (flüchtig) | Merkle-Commits, Branches, Merge |
| Indizes | extern | pro Segment, versioniert |
| Abfrageergebnis | Zeilen | token-budgetiertes Kontextpaket |
| Ableitungen | keine | Lineage + Invalidierung |
| Vertrauen | keines | Taint-Labels |
| Cache | Block-Cache | Block-Cache + Prompt-/KV-Prefix-Cache |

## 11. Roadmap

1. **MVP**: Objektmodell, Segmentformat, WAL, Prolly-Tree mit Branch/Commit/Diff (eingebettet).
2. Hybrid-Suche (BM25 + ANN) und `assemble` mit Token-Packing.
3. Affinitäts-Compaction und LOD-Cluster, Benchmarks gegen RocksDB + Qdrant + Postgres/pgvector auf Agenten-Workloads.
4. Merge-Strategien, CRDT-Typen, Taint, Verschlüsselung.
5. Server, MCP, S3-Tiering, KV-Cache-Store.

**Risiken**: Affinitäts-Clustering kann bei nicht stationären Workloads thrashen (Gegenmittel: Hysterese, nur nachts umordnen). Prolly-Trees haben Write-Amplification bei vielen Kleinstwrites (Gegenmittel: Batching in der Memtable). Die Qualität von `assemble` hängt am Reranker, deshalb sollte er pluggable sein.

Wenn du möchtest, schreibe ich als Nächstes das Segmentformat als Rust-Structs aus, skizziere den Prolly-Tree-Chunker oder fasse alles als Dokument zusammen.

---
# Gemini

Klassische Storage-Engines wie RocksDB (LSM-Trees) oder B-Tree-Datenbanken sind für unabhängige, punktförmige Key-Value-Mutationen optimiert – genau das Gegenteil dessen, was LLMs und autonome Agenten benötigen. LLM-Workloads bestehen aus **verzweigten, append-lastigen Kontext-DAGs (Directed Acyclic Graphs)**: Identische Präfixe (System-Prompts, RAG-Kontexte, Tool-Definitionen) werden tausendfach geteilt, spekulativ geforkt (Tree-of-Thoughts, MCTS, Agent-Retries) und müssen für minimale Time-To-First-Token (TTFT) als physisch zusammenhängende Blöcke aus Tokens, Vektoren und KV-Cache-Tensoren direkt in den Speicher gestreamt werden.

  

Das Konzept für **AetherStore** – eine in Rust geschriebene, LLM-native Storage-Engine – ersetzt LSM-Trees durch eine **Copy-on-Write (CoW) Merkle-Radix-Arena-Architektur** mit nativer Git-Semantik und Hardware-naher Zero-Copy-Lokalität.

  

## 1. Paradigmenwechsel: Warum RocksDB für LLMs suboptimal ist

|**Architektur-Dimension**|**RocksDB (LSM-Tree)**|**Dedizierte Vector-DB**|**AetherStore (LLM-Native Engine)**|
|---|---|---|---|
|**Primäre Datenstruktur**|Sortierte SSTables + MemTable|HNSW / IVF Graph + Metadata Store|**Persistent CoW Merkle-Radix-DAG**|
|**Branching / Versionierung**|Teure Snapshots; kein echtes Prefix-Sharing bei Forks|Kaum vorhanden (Mutable Collections)|**$O(1)$ Zero-Copy Branching** (Git-like Commits & Rollbacks)|
|**Physische Lokalität**|Keys lexikografisch sortiert; Werte getrennt/fragmentiert|Vektoren getrennt vom eigentlichen Payload-Text|**Co-Located Context Slabs** (Tokens + Embeddings + KV-Cache benachbart)|
|**Write-Verhalten**|Hohe Write-Amplification durch Background-Compaction|In-Place Graph-Updates mit Lock-Contention|**Append-Only Shadow Paging** (kein WAL für Payloads nötig)|
|**I/O & Serialisierung**|Protobuf/JSON Deserialisierung über CPU-Heap|gRPC/REST + Heap-Allokationen|**4KB-aligned Zero-Copy DMA** (`io_uring` & GPUDirect Storage)|

## 2. Physische Datenlokalität: Co-Located Context Slabs

In herkömmlichen Agenten-Stacks liegen der Konversationstext in Postgres, die Embeddings in Qdrant und die vorberechneten Attention-KV-Caches im RAM oder Redis. Bei jedem Agenten-Step entstehen Serialisierungs- und Netzwerk-Overheads.

  

AetherStore organisiert Daten stattdessen in **Context Slabs** (physische 64-KB- bis 2-MB-Arenen, ausgerichtet an OS-HugePages und 4096-Byte-NVMe-Sektoren). Innerhalb eines Slabs werden zusammengehörige Daten physisch direkt hintereinander abgelegt (**Array-of-Structures-of-Arrays Layout**):

  

1. **Token-Block-Alignment (PagedAttention-kompatibel):** Text wird nicht als beliebiger String gespeichert, sondern direkt beim Schreiben tokenisiert und in feste Seiten (z. B. 64 Tokens pro Page) quantisiert. Das eliminiert wiederholte Tokenisierung beim Laden.
    
2. **Inline-Embeddings:** Direkt neben dem Token-Block liegt das quantisierte Embedding (Int8/Binary für schnelle Grobfilterung, FP16 für Rescoring). Ein einziger NVMe-Read lädt Vektor und Text gleichzeitig in den CPU-L2-Cache.
    
3. **KV-Cache-Tensor-Slots:** Für häufig genutzte Präfixe (System-Prompts, Codebases) speichert der Slab direkt angrenzend die vorberechneten Key-Value-Attention-Tensoren (4KB-aligned). Über `io_uring` und NVIDIA GPUDirect Storage (`cuFileRead`) können diese ohne Umweg über den CPU-RAM direkt in den GPU-VRAM gestreamt werden.
    
4. **Trajectory-Packing:** Da ein LLM beim Inferenz-Start den gesamten Pfad von der Wurzel bis zum aktuellen Blatt (Root-to-Leaf Trajectory) benötigt, schreibt der Arena-Allocator aufeinanderfolgende Agenten-Steps sequenziell auf denselben physischen NVMe-Block. Ein einziger Sequential-Read ersetzt hunderte Random-IOPS.
    

## 3. Native Versionskontrolle & Atomare Transaktionen

Agenten arbeiten nicht linear: Sie verwerfen Halluzinationen, testen parallele Tool-Pfade (Monte-Carlo Tree Search) und benötigen exakte Reproduzierbarkeit für Evals und Debugging.

  

- **Content-Addressable Merkle-DAG:** Jeder unveränderliche Token- oder Memory-Block wird über seinen **BLAKE3-Hash** identifiziert. Identische System-Prompts oder Dokument-Chunks existieren physisch nur ein einziges Mal auf der Disk (automatische Deduplizierung).
    
- **$O(1)$ Branching & Time-Travel:** Ein Konversationsverlauf oder Agenten-Zustand ist lediglich ein Zeiger (`CommitId`) auf die Wurzel eines persistenten Radix-Trees (Branching-Faktor 32). Ein `fork()` klont nur diesen 32-Byte-Hash – völlig ohne Daten zu kopieren.
    
- **Atomare Commits ohne klassisches WAL:** Da alle Datenblöcke immutable (Copy-on-Write) sind, schreibt eine Transaktion zuerst die neuen Slabs per `io_uring` auf freie Sektoren. Erst wenn alle Blöcke persistent sind, aktualisiert ein einzelner **atomarer Compare-And-Swap (CAS)** im Ring-Buffer-Superblock den `BranchHead`. Ein Crash während des Schreibens hinterlässt niemals korrupte Zustände; unreferenzierte Slabs werden beim Neustart einfach ignoriert.
    

## 4. Integrierter Semantischer & Struktureller Index

Statt eines externen Vektor-Index nutzt AetherStore ein **Branch-Aware DiskANN-Overlay**:

  

- Index-Knoten verweisen nicht auf abstrakte UUIDs, sondern direkt auf physische `(ArenaId, ByteOffset, CommitMask)`-Tupel.
    
- Die `CommitMask` (ein komprimiertes Roaring-Bitmap) filtert während des Graph-Traversals in Nanosekunden alle Vektoren heraus, die nicht zum aktuellen Agenten-Branch oder Zeitpunkt gehören. So gibt es keine "Phantom-Ergebnisse" aus verworfenen Agenten-Pfaden.
    

## 5. Rust-Systemarchitektur & Core-Implementierung

Um Lock-Contention bei hochparallelen Agenten-Swarms zu vermeiden, nutzt AetherStore eine **Thread-per-Core-Architektur** (analog zu Glommio/Seastar), kombiniert mit `io_uring` für asynchrones Direct-I/O und Epoch-Based Reclamation (`crossbeam-epoch`) für lock-freie Lesezugriffe.

  

Rust

```
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 32-Byte BLAKE3 Content-Addressable Hash für Block-Deduplizierung & Merkle-Verification
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
pub struct ContentHash(pub [u8; 32]);

/// Physische Adresse auf NVMe: Arena-ID + 4KB-aligned Offset + Länge
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct PhysicalExtent {
    pub arena_id: u32,
    pub page_offset: u32, // Multipliziert mit 4096 ergibt exakten Byte-Offset
    pub page_count: u16,
    pub flags: u16,       // Bitflags: HasKvCache, CompressedZstd, QuantizedInt8
}

/// 4096-Byte-aligned Context-Page für Zero-Copy Direct-I/O & GPUDirect Storage.
/// Enthält physisch benachbart: Metadata, Token-IDs, Quantized Embedding & KV-Cache-Ref.
#[repr(C, align(4096))]
pub struct AlignedContextPage {
    pub magic: u32,
    pub token_count: u16,
    pub embedding_dim: u16,
    pub parent_hash: ContentHash,
    pub content_hash: ContentHash,
    /// Vor-tokenisierte IDs (z. B. 64 Tokens * 4 Bytes = 256 Bytes)
    pub token_ids: [u32; 64],
    /// Quantisiertes Int8-Embedding direkt im selben Cache-Line-Bereich (z. B. 768 Dim)
    pub quantized_embedding: [i8; 768],
    /// Optionaler Verweis auf ausgelagerte Attention-KV-Tensoren im High-Speed-Tier
    pub kv_cache_extent: PhysicalExtent,
    /// Restlicher Platz der 4KB-Page für komprimierte Raw-Bytes / Tool-JSON-Payloads
    pub inline_payload: [u8; 2944],
}

/// Ein atomarer Snapshot eines Agenten-Zustands (Git-Commit-Äquivalent)
#[derive(Clone, Debug)]
pub struct AgentCommit {
    pub commit_id: ContentHash,
    pub parent_commit: Option<ContentHash>,
    /// Wurzel des persistenten CoW-Radix-Trees für alle Kontext-Blöcke dieses Branches
    pub context_radix_root: PhysicalExtent,
    /// Roaring-Bitmap-Root für Branch-isolierte Vektorsuche
    pub vector_visibility_root: PhysicalExtent,
    pub timestamp_ns: u64,
}

/// Lock-freier Branch-Zeiger für O(1) Forks und atomare Updates
pub struct BranchHead {
    pub name: String,
    /// Atomarer Pointer auf die aktuelle Commit-Version (Slot-Index im Superblock)
    current_commit_slot: AtomicU64,
}

pub struct AetherEngine {
    // Thread-lokale io_uring Queues & Arena-Allokatoren würden hier verwaltet
    superblock_slots: Arc<Vec<AtomicU64>>,
}

impl AetherEngine {
    /// Erzeugt einen neuen Branch in O(1) ohne jegliches Kopieren von Kontextdaten.
    pub fn fork_branch(&self, source: &BranchHead, new_name: String) -> BranchHead {
        let active_slot = source.current_commit_slot.load(Ordering::Acquire);
        // Refcount für den zugrundeliegenden Merkle-Root wird atomar erhöht
        self.increment_slab_refcount(active_slot);

        BranchHead {
            name: new_name,
            current_commit_slot: AtomicU64::new(active_slot),
        }
    }

    /// Hängt einen neuen Inferenz-/Tool-Step atomar an einen Branch an (Optimistic Concurrency).
    pub fn atomic_append_step(
        &self,
        branch: &BranchHead,
        expected_slot: u64,
        new_page: &AlignedContextPage,
    ) -> Result<u64, StorageError> {
        // 1. Schreibe AlignedContextPage ohne Locks in die aktuelle thread-lokale Arena (Shadow Paging)
        let extent = self.write_page_direct_io(new_page)?;

        // 2. Erzeuge neuen CoW-Radix-Pfad (nur O(log_32 N) Knoten werden neu geschrieben)
        let new_commit_slot = self.build_cow_commit(expected_slot, extent)?;

        // 3. Atomarer Commit via CAS: Schlägt fehl, falls ein anderer Worker denselben Branch mutiert hat
        branch
            .current_commit_slot
            .compare_exchange(
                expected_slot,
                new_commit_slot,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| StorageError::ConcurrentBranchConflict)
    }

    /// Zero-Copy-Zugriff auf eine komplette Root-to-Leaf-Trajektorie als Speicher-Slices
    pub fn read_trajectory_zero_copy(&self, _branch: &BranchHead) -> Vec<&AlignedContextPage> {
        // Mappt die physisch benachbarten Extents direkt via mmap / io_uring Registered Buffers
        // ohne Heap-Allokation oder Deserialisierung in den Adressraum der Inferenz-Engine.
        unimplemented!("Direct-I/O & Registered-Buffer Mapping")
    }

    fn write_page_direct_io(&self, _page: &AlignedContextPage) -> Result<PhysicalExtent, StorageError> {
        Ok(PhysicalExtent { arena_id: 1, page_offset: 0, page_count: 1, flags: 0 })
    }

    fn build_cow_commit(&self, _parent_slot: u64, _extent: PhysicalExtent) -> Result<u64, StorageError> {
        Ok(42)
    }

    fn increment_slab_refcount(&self, _slot: u64) {}
}

#[derive(Debug)]
pub enum StorageError {
    ConcurrentBranchConflict,
    IoFailure,
}
```

## 6. Storage-Tiering & Trajectory-Aware Garbage Collection

Da Agenten bei Tree-Search-Verfahren (z. B. Code-Generierung mit Unit-Test-Feedback) bis zu 90 % aller erkundeten Branches wieder verwerfen, benötigt die Engine eine spezialisierte Bereinigung:

- **Automatisches Multi-Tiering:**
    
    - _Tier 0 (GPU VRAM / Host DRAM):_ Aktive KV-Caches und die obersten Ebenen des Merkle-Radix-Trees.
        
    - _Tier 1 (Lokale NVMe via `io_uring`):_ Vollständige `AlignedContextPages` inklusive Int8-Embeddings und warmen KV-Caches.
        
    - _Tier 2 (Object Storage / S3):_ Abgeschlossene Agenten-Sessions werden asynchron in spaltenorientierte Arrow/Parquet-Archiv-Slabs verdichtet, behalten aber ihre Content-Hashes für spätere Audits.
        
- **Epoch-Based GC & Hot-Path-Defragmentierung:** Wird ein Branch gelöscht (`drop_branch`), dekrementiert die Engine atomar die Referenzzähler entlang des Merkle-Pfads. Sobald ein Slab unter eine Auslastung von 40 % fällt, kopiert ein Hintergrund-Worker die verbliebenen "lebenden" Pfade sequenziell in eine neue Arena (**Hot-Trajectory-Packing**), sodass die physische Leselokalität über Monate hinweg optimal bleibt, ohne jemals die Latenz laufender Inferenz-Requests zu stören.

---

# grok

**Konzept: “AetherStore” – eine LLM-/Agenten-native Storage-Engine in Rust**

### Design-Ziele (explizit gegen RocksDB & Co.)
- **Physische Lokalität zuerst**: Daten, die ein LLM/Agent zusammen braucht (Embedding + Kontext-Chunks + Metadaten + Version-History), liegen auf dem Medium und im RAM so nah beieinander wie möglich.
- **Atomare, versionierte Mutation** ohne klassische B-Tree/LSM-Overheads.
- **Native Unterstützung für die drei Kern-Datenarten von Agents**:
  1. Dense/Sparse Embeddings + quantisierte Varianten
  2. Token-/Text-Chunks mit hierarchischer Kontext-Struktur
  3. Graph-/Relation- und Event-Streams (Trajectories, Tool-Calls, Episodic Memory)
- Schreibpfad extrem write-friendly und lesepfad extrem cache-freundlich (LLM-Inference und Retrieval sind stark leselastig und stark lokal).
- Eingebaute Versionskontrolle (branch/merge/snapshot) auf Storage-Ebene, nicht als Overlay.
- Zero-copy und mmap-first wo sinnvoll, aber mit sicheren Rust-Abstraktionen.

RocksDB (LSM), LMDB, SQLite, und die meisten Vector-DBs sind entweder generisch key-value oder nur für einen Aspekt (Vektoren) optimiert. AetherStore ist von Anfang an um die Zugriffsmuster von RAG + Agent-Memory + Trajectory-Versionierung herum gebaut.

### Kern-Architektur

#### 1. Physisches Layout: “Semantic Slabs” + “Locality Groups”
Statt reiner Key-Value oder reiner LSM:

- **Slab** = festes, page-aligned (meist 64 KiB–2 MiB) physisches Speicherstück.
- Jeder Slab gehört zu einer **Locality Group**. Eine Locality Group fasst alle Daten zusammen, die ein Agent/LLM typischerweise in einem “Arbeitskontext” braucht:
  - Embedding(s) eines Dokuments/Chunks
  - Der eigentliche Text/Token-Stream (komprimiert)
  - Metadaten + Tags
  - Graph-Kanten zu verwandten Chunks
  - Die Version-History dieses Objekts (als delta-chain)

Dadurch wird ein Cache-Miss oder ein Disk-Seek extrem teuer vermieden. Der Storage-Engine-Scheduler hält ganze Locality Groups hot, wenn ein Agent an ihnen arbeitet.

#### 2. Adressierung & Content-Addressability
Jedes logische Objekt hat:
- Einen **logischen Key** (user-defined, z. B. `agent:42/memory/episodic/2026-11-21`)
- Einen **Content-Hash** (BLAKE3) → echte Content-Addressability
- Eine **Version-ID** (monoton steigend oder Merkle-Root)

Das ermöglicht:
- Deduplizierung auf Block-Ebene
- Günstige Snapshots und Branches (nur Root-Zeiger ändern)
- Integritätsprüfung ohne Extra-Layer

#### 3. Versionierung (first-class)
- Jede Mutation erzeugt eine neue Version (Copy-on-Write auf Slab-Ebene oder Delta).
- Es gibt explizite **Branches** (z. B. `main`, `agent-run-7`, `experiment-rlhf`).
- Snapshots sind billig (nur Root-Pointer + Referenzzählung).
- Merge ist möglich (3-way oder operational transform für Text + Graph).
- Garbage Collection läuft asynchron und behält konfigurierbare History-Tiefe pro Locality Group.

Das ist deutlich stärker als “RocksDB + manuelle Versionierung” oder Git-ähnliche Overlays.

#### 4. Daten-Typen (native)
| Typ                  | Speicherung                          | Besonderheit                              |
|----------------------|--------------------------------------|-------------------------------------------|
| Embedding            | quantisiert (int8/fp16/bf16) + optional raw | SIMD-freundliche Layouts, ANN-Index lokal |
| Text/Tokens          | zstd oder spezielles Token-Delta     | Hierarchische Chunks (Sentence → Paragraph → Doc) |
| Graph / Relations    | adjacency lists + edge attributes    | Locality-aware Partitionierung            |
| Event / Trajectory   | append-only log + index              | Zeitreihen + causal ordering              |
| Structured State     | MessagePack / Cap’n Proto            | Schema-evolution-fähig                    |

#### 5. Indexe (multi-modal, co-located)
- **ANN-Index** (HNSW oder IVF + PQ) lebt *innerhalb* der Locality Group oder als Secondary Index mit Pointern zurück in die Slabs.
- **Inverted + Semantic Index** für Keyword + Hybrid-Search.
- **Temporal Index** für Agent-Trajectories und “was wusste der Agent zum Zeitpunkt X”.
- Alle Indexe können versioniert und branched werden.

#### 6. Transaktionen & Atomarität
- Optimistic Concurrency Control + MVCC auf Versionsebene.
- Atomic Multi-Object Updates innerhalb einer Locality Group sind extrem billig (ein Slab-Write + Atomic Root-Update).
- Cross-Group Transaktionen nutzen 2PC oder eine leichte WAL nur für die Commit-Records.

#### 7. Speicherhierarchie
```
L0  – Hot Working Set   (in-memory, pinned Locality Groups)
L1  – Warm              (NVMe, mmap)
L2  – Cold / Archive    (object storage oder HDD, compressed)
```
Der Engine entscheidet automatisch (oder per Hint vom Agenten), welche Groups hot bleiben. LLM-Inference-Pattern (wiederholte Zugriffe auf denselben Kontext) werden erkannt und bevorzugt.

### Rust-Implementierungs-Skizze

**Kern-Crates (vorgeschlagen):**
- `aether-core` – Slab-Allocator, Locality-Group-Manager, Version-Graph
- `aether-index` – HNSW/IVF + inverted indexes (SIMD mit `wide` oder `simsimd`)
- `aether-codec` – Embedding-Quantisierung, Token-Delta, zstd
- `aether-tx` – MVCC + Atomic Commit
- `aether-fs` – mmap + io_uring (Linux) / io_uring-ähnliche Abstraktion
- `aether-api` – high-level Agent-Memory-API (`store_memory`, `retrieve_relevant`, `branch`, `snapshot`, `merge`)

Wichtige Design-Entscheidungen in Rust:
- Arena-Allocator pro Locality Group → fast zero-fragmentation und excellent cache behavior.
- `Arc` + generational indices oder epoch-based reclamation für concurrent readers.
- `io_uring` + `mmap` hybrid: Writes gehen über io_uring, Reads bevorzugt über mmap (zero-copy in den LLM-Tensor-Pfad).
- Kein globaler Lock; Sharding nach Locality-Group-Hash.
- Optional: GPU-offload für ANN-Rebuild und Quantisierung (via `cudarc` oder `wgpu`).

### API-Beispiel (Agent-Perspektive)

```rust
let store = AetherStore::open("agent_memory.aether")?;

// Atomic, versionierter Write einer ganzen Locality Group
store.with_tx(|tx| {
    tx.put_embedding("doc:42", &emb, Quantization::Int8)?;
    tx.put_text("doc:42", chunk_text)?;
    tx.link("doc:42", "doc:17", Relation::Similar)?;
    tx.commit_branch("agent-run-7")
})?;

// Retrieval mit physischer Lokalität
let results = store.retrieve(
    Query::hybrid("was war der Status von Projekt X")
        .with_graph_hops(2)
        .at_version("agent-run-7")
        .limit(12)
)?;
```

### Warum das besser als RocksDB + Vector-DB-Kombination ist
- RocksDB optimiert für generische Key-Value-Writes → schlechte räumliche Lokalität für Embeddings + Text + Graph.
- Separate Vector-DBs (Milvus, Qdrant, …) haben ihren eigenen Speicher und erzwingen Netzwerk oder teure Joins.
- Versionierung ist meist Overlay → teuer und inkonsistent.
- AetherStore macht die Locality Group zur atomaren Einheit und die Version zum first-class Bürger. Dadurch sinken Cache-Misses und I/O dramatisch, und Agenten können sicher experimentieren (Branch → Train/Run → Merge oder Discard).

### Erweiterungen / Roadmap-Ideen
- Native Support für “Memory Consolidation” (LLM-gesteuertes Zusammenfassen alter Versionen).
- Hardware-aware Layout (CXL, persistent memory, GPU-direct storage).
- Built-in “Attention Cache” – der Engine merkt sich, welche Chunks in den letzten Inference-Läufen hot waren.
- Plugin-System für custom Codecs und Similarity-Metriken.

Das Konzept ist bewusst radikal lokalitäts- und versionszentriert. Es opfert etwas generische Key-Value-Flexibilität zugunsten der realen Zugriffsmuster von modernen LLM-Agents. In Rust lässt sich das mit sehr hoher Performance und Speichersicherheit realisieren.

**Detaillierte Sub-Allokation innerhalb eines Slabs**

Die Sub-Allokation ist der Mechanismus, der *innerhalb* eines bereits allokierten Slabs freien Platz findet und an Objekte vergibt. Sie entscheidet maßgeblich über interne Fragmentierung, Allokationsgeschwindigkeit und Cache-Verhalten.

### 1. Ziele der Sub-Allokation

- Sehr schnelle Allokation (idealerweise < 20–30 CPU-Zyklen im Hot-Path)
- Gute räumliche Lokalität der Objekte *innerhalb* des Slabs
- Geringe interne Fragmentierung
- Unterstützung von Objekten stark unterschiedlicher Größe (4 Byte Metadaten bis mehrere hundert KB Text/Embedding)
- Einfache und sichere Integration mit Copy-on-Write
- SIMD-/Cache-Line-freundliche Ausrichtung (besonders für Embeddings)

### 2. Slab-interne Struktur

Jeder Slab beginnt mit dem bereits beschriebenen **Slab-Header** (64–128 Byte). Danach folgt der nutzbare Datenbereich.

Der Datenbereich wird in zwei logische Zonen unterteilt:

```
[ Slab-Header ]
[ Metadata-Zone (optional, wachsend von vorne) ]
[ Object-Zone (wachsend von hinten oder gemischt) ]
[ Free-Space-Verwaltung ]
```

Zwei bewährte Varianten:

**Variante A – Bitmap + lineare Allokation (bevorzugt für Small/Standard-Slabs)**  
**Variante B – Intrusive Free-List + Size-Classes (für größere und stärker fragmentierte Slabs)**

AetherStore verwendet **hybrid**:

- Für Slabs ≤ 256 KiB → primär Bitmap + First-Fit/Next-Fit
- Für größere Slabs → Size-Class-segregierte Free-Lists + Best-Fit

### 3. Objekt-Header (Object Header)

Jedes allokierte Objekt beginnt mit einem kompakten Header (8–16 Byte):

```rust
#[repr(C)]
struct ObjectHeader {
    size: u32,              // tatsächliche Größe inkl. Header
    type_tag: u16,          // Embedding, Text, Edge, Delta, ...
    flags: u16,             // live, pinned, compressed, ...
    // optional:
    // generation: u32,     // für schnelle CoW-Erkennung
    // checksum: u32,       // partial
}
```

Der Header ist so gestaltet, dass die Nutzdaten (besonders Embeddings) auf 32- oder 64-Byte-Grenzen ausgerichtet sind.

### 4. Free-Space-Verwaltung im Detail

#### 4.1 Bitmap-Variante (kleine bis mittlere Slabs)

- Der Slab wird in feste **Granules** unterteilt (typisch 16, 32 oder 64 Byte).
- Ein Bitmap (1 Bit pro Granule) liegt entweder im Slab-Header-Bereich oder am Ende des Slabs.
- Allokation:
  1. Benötigte Anzahl Granules berechnen (inkl. Alignment-Padding).
  2. Im Bitmap nach einer ausreichend langen Folge von Free-Bits suchen (Bit-Scan / hardware `tzcnt`/`lzcnt`).
  3. Bits setzen, Object-Header schreiben, Pointer zurückgeben.

Vorteile: extrem einfach, sehr cache-freundlich, gute Locality.  
Nachteil: etwas höhere interne Fragmentierung durch feste Granule-Größe.

#### 4.2 Free-List-Variante (größere Slab)

Freie Regionen werden als intrusive Liste verwaltet:

```rust
struct FreeRegion {
    size: u32,
    next: u32,          // Offset relativ zum Slab-Anfang
    prev: u32,          // für schnelles Entfernen
}
```

Zusätzlich existieren **Size-Class-Buckets** (z. B. 16, 32, 64, 128, 256, 512, 1K, 2K, 4K, …).  
Eine Allokation sucht zuerst im passenden Bucket, greift sonst auf den nächstgrößeren zurück und splittet bei Bedarf.

Beim Free wird die Region in den passenden Bucket eingehängt und – wenn möglich – mit benachbarten freien Regionen verschmolzen (Coalescing).

### 5. Allokationsstrategien

| Strategie       | Verwendung                          | Bemerkung                                      |
|-----------------|-------------------------------------|------------------------------------------------|
| Next-Fit        | Hot-Path, häufige kleine Allokationen | Sehr schnell, guter Cache-Verhalten           |
| First-Fit       | Standard                            | Guter Kompromiss                               |
| Best-Fit        | Wenn Fragmentierung kritisch        | Teurer, aber weniger Verschnitt                |
| Size-Class      | Immer parallel                      | Reduziert Suche drastisch                      |

Zusätzlich gibt es einen **Thread-Local-Cache** kleiner freier Regionen pro Arena, damit parallele Allokationen innerhalb derselben Locality Group selten den zentralen Free-Space-Lock brauchen.

### 6. Ausrichtung und Embedding-Optimierung

Embeddings werden besonders behandelt:

- Der Object-Header wird so platziert, dass die eigentlichen Float-/Int-Daten auf 64-Byte-Grenzen (Cache-Line) und idealerweise auf 32-Byte-Grenzen (AVX2/AVX-512) liegen.
- Bei quantisierten Embeddings (int8/int4) wird zusätzlich auf 16- oder 32-Byte-Grenzen ausgerichtet.
- Der Allocator kennt den Typ und wählt automatisch die richtige Alignment-Anforderung.

### 7. Freigabe und Coalescing

Beim Free:
1. Object-Header wird als frei markiert (oder sofort überschrieben).
2. Die Region wird in die Free-List / Bitmap eingetragen.
3. Sofortiges oder verzögertes Coalescing mit linken und rechten Nachbarn.
4. Bei Bitmap-Variante einfach die Bits zurücksetzen.

Für CoW-Slabs gilt: Ein Free in einer neuen Version beeinflusst die alte Version nicht. Die alte Version behält ihre eigene Sicht auf den (damals noch belegten) Bereich.

### 8. Fragmentierung und Slab-interne Compaction

Wenn der freie Platz eines Slab zwar insgesamt ausreicht, aber zu stark fragmentiert ist (z. B. größte freie Region < benötigte Größe), wird eine **lokale Compaction** ausgelöst:

- Alle lebenden Objekte werden an den Anfang des Slab dicht kopiert.
- Die Free-Space-Verwaltung wird neu aufgebaut.
- Dies geschieht nur, wenn der Slab nicht mehr von mehreren Versionen referenziert wird (ansonsten wird ein neuer Slab allokiert → CoW).

### 9. Concurrent Sub-Allokation

- Die Free-Space-Struktur einer Arena ist durch einen schmalen Spinlock oder besser durch lock-free Techniken (bei Bitmap) geschützt.
- Sehr kleine Allokationen können über einen thread-lokalen Bump-Pointer oder einen kleinen Cache laufen und nur periodisch mit dem zentralen Free-Space synchronisiert werden.
- Leser (Inference, Retrieval) sehen immer eine konsistente Sicht dank MVCC und Generation-Zählern.

### 10. Zusammenfassung der Design-Entscheidungen

| Aspekt             | Entscheidung                           | Begründung                                    |
| ------------------ | -------------------------------------- | --------------------------------------------- |
| Primäre Verwaltung | Hybrid (Bitmap + Size-Class Free-List) | Geschwindigkeit + Flexibilität                |
| Alignment          | Typabhängig, bis 64 Byte               | SIMD + Cache-Line-Effizienz                   |
| Coalescing         | Sofort bei Free-List, lazy bei Bitmap  | Balance zwischen CPU und Fragmentierung       |
| Compaction         | Lokal + nur wenn nötig                 | Vermeidet teure globale Reorganisation        |
| Concurrent Access  | Thread-local Cache + schmale Locks     | Hoher Parallelitätsgrad innerhalb einer Group |

Die Sub-Allokation ist bewusst relativ einfach gehalten. Komplexe Allokatoren (wie jemalloc) wären hier überdimensioniert und würden die Vorhersagbarkeit und die enge Kopplung an Locality Groups und CoW verschlechtern.

---
# ChatGPT
# **CairnDB – Eine Storage-Engine, die speziell für LLMs und autonome Agenten entwickelt wird**

Architekturvorschlag

Rust

LLM-native Storage

Meine Empfehlung wäre, eine neue Storage-Engine zu entwickeln, die nicht versucht, eine klassische Datenbank für LLMs nachzubauen, sondern die grundlegenden Annahmen über Datenhaltung verändert.

Ich würde sie vorläufig **CairnDB** nennen.

Die zentrale Idee:

**CairnDB speichert nicht einfach Schlüssel und Werte. Es speichert versionierte Wissenszustände, Ereignisse, Beziehungen, Kontext und deren Herkunft so, dass ein LLM sie mit möglichst wenig I/O, möglichst wenig CPU-Aufwand und möglichst wenig unnötigen Tokens wiederverwenden kann.**

RocksDB ist ein sehr guter allgemeiner LSM-Storage. Aber eine LLM-Engine hat andere Anforderungen:

- Ein Agent schreibt ständig kleine Änderungen.
    
- Eine einzelne Operation kann mehrere Datenobjekte verändern.
    
- Ein Agent muss seine letzten Zustände und Entscheidungen rekonstruieren können.
    
- Informationen besitzen Herkunft, Vertrauenswürdigkeit, zeitliche Gültigkeit und Beziehungen.
    
- Dieselben Daten müssen lexikalisch, semantisch, strukturell und zeitlich durchsuchbar sein.
    
- Ein Agent benötigt nicht unbedingt die ähnlichsten 100 Texte, sondern die besten 10 Informationen innerhalb eines begrenzten Token-Budgets.
    
- Unterschiedliche Agenten müssen parallel arbeiten können, ohne sich gegenseitig zu überschreiben.
    
- Jede Änderung muss nachvollziehbar, atomar und gegebenenfalls rückgängig zu machen sein.
    
- Embeddings und Suchindizes müssen sich effizient aktualisieren lassen, ohne den eigentlichen Datenbestand zu gefährden.
    

Daraus folgt eine wichtige Architekturentscheidung:

**Die Datenbank braucht einen gemeinsamen, versionierten Datenkern und mehrere spezialisierte, daraus abgeleitete Zugriffspfade.**

Nicht fünf voneinander unabhängige Datenbanken, die über APIs zusammengeklebt werden.

# **1. Die grundlegende Architekturentscheidung**

Ich würde CairnDB als Kombination aus sechs Konzepten bauen:

# **CairnDB Architecture**

## **LLM / Agent Interface**

Memory API · Context API · Transactions · History · Search

**Context Engine**

Token-aware retrieval, ranking, context assembly

**Transaction Engine**

MVCC, atomic commits, branching, snapshots

**Unified Versioned Data Core**

**Document & Event Store**

**MVCC Object Store**

**Relationship Store**

**Vector & Embedding Store**

**LSM / Sorted Runs**

Point lookup, ranges, immutable segments

**ANN Index**

Semantic similarity and vector filtering

**Full-Text Index**

BM25, terms, positions, exact matching

**Graph & Bitmap Index**

Relationships, metadata, authorization

**Physical Storage Layer**

WAL · Locality-aware Data Pages · Content-addressed Blobs · Immutable Segments · Manifests · Checksums

Das Entscheidende ist, dass die verschiedenen Indizes keine unabhängigen Wahrheiten darstellen.

Der versionierte Datenkern ist die Wahrheit. Vektorsuche, Volltextsuche, Graphindizes und andere Indizes sind optimierte Projektionen dieses Datenbestands.

Damit bekommen wir atomare Änderungen, historische Rekonstruktion und konsistente Abfragen, ohne sämtliche Suchindizes bei jeder Transaktion synchron vollständig neu aufbauen zu müssen.

# **2. Das wichtigste Prinzip: LLM-Daten sind nicht alle gleich**

Eine klassische Key-Value-Datenbank sieht beispielsweise:

```
key   -> value
user1 -> {"name": "Max"}
```

Für eine Agenten-Datenbank reicht dieses Modell nicht.

Ein Agent erzeugt unterschiedliche Informationsklassen mit unterschiedlichen Zugriffsmustern.

Ich würde sechs native Datentypen einführen.

|Datentyp|Zweck|Optimales Zugriffsmuster|
|---|---|---|
|Working State|Laufender Agentenzustand|Sehr schnelle mutable Updates|
|Episodic Memory|Interaktionen, Entscheidungen, Erfahrungen|Append-only, zeitliche Suche|
|Semantic Memory|Wissen, Fakten, Eigenschaften|Schnelle strukturierte und semantische Suche|
|Procedural Memory|Fähigkeiten, Routinen, Workflows|Strukturierte und graphbasierte Suche|
|Artifacts|Dokumente, Code, Bilder, Audio, Ergebnisse|Content-addressed Blob Storage|
|Provenance|Quellen, Begründungen, Vertrauen, Herkunft|Versionierte Referenzen und Relations|

Diese Trennung ist nicht willkürlich. Unterschiedliche Memory-Typen erfordern tatsächlich unterschiedliche Speicher- und Retrievalstrategien. Auch die aktuelle Microsoft-Referenzarchitektur für Agent-Memory unterscheidet beispielsweise semantische, episodische und prozedurale Informationen.

GitHub+1

## **2.1 Working State**

Hier liegt der unmittelbare Zustand eines Agenten.

Beispiel:

```
{
  "agent_id": "agent_42",
  "run_id": "run_981",
  "current_goal": "Analyze repository",
  "step": 17,
  "current_plan": "...",
  "tool_state": {},
  "pending_actions": [],
  "checkpoint": 16
}
```

Anforderungen:

- Sehr kleine Lese- und Schreiboperationen.
    
- Atomare Teiländerungen.
    
- Schnelle Wiederherstellung.
    
- Optimistic Concurrency Control.
    
- Schnelle Checkpoints.
    
- Kein unnötiges Komprimieren bei häufigen Updates.
    

Der besondere Trick: Working State und Event History werden gemeinsam in einer Transaktion verändert.

Beispielsweise:

1. Tool-Aufruf wird protokolliert.
    
2. Agentenstatus wird aktualisiert.
    
3. Nächster Arbeitsschritt wird gespeichert.
    
4. Ein Checkpoint wird angelegt.
    

Alle vier Änderungen werden gemeinsam committed.

Ein Absturz darf nicht dazu führen, dass die Tool-Ausführung existiert, aber der Agentenzustand nicht weiß, dass sie stattgefunden hat.

## **2.2 Episodic Memory**

Hier werden vergangene Ereignisse gespeichert:

```
User request
Agent reasoning summary
Tool invocation
Tool result
Observation
Decision
Outcome
```

Die Ereignisse sollten grundsätzlich unveränderlich sein.

Statt Ereignisse zu überschreiben, entstehen neue Ereignisse, die vorherige ergänzen oder korrigieren.

Vorteile:

- Auditierbarkeit.
    
- Rekonstruktion.
    
- Lernen aus Erfahrungen.
    
- Zeitliche Abfragen.
    
- Wiederverwendung erfolgreicher Lösungswege.
    

## **2.3 Semantic Memory**

Hier liegen konsolidierte Fakten.

Beispiel:

```
{
  "subject": "project_x",
  "predicate": "uses_database",
  "object": "PostgreSQL",
  "confidence": 0.96,
  "source": "episode_123",
  "valid_from": 1795000000,
  "valid_until": null
}
```

Besonders wichtig ist, dass ein Fakt nicht nur einen Wert besitzt.

Er braucht:

- Quelle.
    
- Vertrauenswürdigkeit.
    
- Gültigkeitszeitraum.
    
- Erstellungszeitpunkt.
    
- Version.
    
- Eventuell widersprüchliche Informationen.
    
- Referenz auf die ursprüngliche Information.
    

Ein später erkannter Fehler überschreibt den ursprünglichen Fakt nicht spurlos. Er erzeugt eine neue Version mit entsprechender Korrektur.

## **2.4 Procedural Memory**

Dieser Bereich speichert wiederverwendbare Handlungsabläufe.

Beispiel:

```
Deploy application
   |
   +-- Check repository
   |
   +-- Run tests
   |
   +-- Build image
   |
   +-- Deploy
   |
   +-- Verify health
```

Hier würde ich native Graphbeziehungen und strukturierte Schrittfolgen verwenden.

Eine erfolgreiche Problemlösung könnte dadurch zu einer wiederverwendbaren Fähigkeit werden.

## **2.5 Artifacts**

Hier werden große Informationen gespeichert:

- Quellcode.
    
- Dokumente.
    
- Webseiten.
    
- Bilder.
    
- Audio.
    
- Videos.
    
- Tool-Ergebnisse.
    
- Multimodale Daten.
    

Große Inhalte dürfen nicht vollständig in die primäre Key-Value-Struktur gelangen.

Dafür bauen wir einen dedizierten Blob- und Content-Addressed-Storage.

## **2.6 Provenance**

Dieser Datentyp ist besonders wichtig.

LLMs können falsche Informationen erzeugen. Deshalb muss CairnDB zwischen einer Information und der Quelle dieser Information unterscheiden.

Beispiel:

```
Fact: PostgreSQL is used by project X.

Source:
  document_42
  episode_123

Extracted by:
  agent_7

Model:
  extraction_model_v3

Confidence:
  0.96

Supersedes:
  fact_revision_3

Commit:
  commit_9821
```

Damit lässt sich später feststellen:

- Woher kommt eine Information?
    
- Wer hat sie erstellt?
    
- Welche Quellen unterstützen sie?
    
- Wurde sie bestätigt?
    
- Wurde sie inzwischen korrigiert?
    
- Auf welcher Version basiert eine Agentenentscheidung?
    

# **3. Das Herzstück: Eine neue hybride Storage-Engine**

Hier würde ich mich bewusst von reinem RocksDB-Design entfernen.

Meine Wahl wäre eine Architektur aus:

**Append-only Data Log + MVCC + Locality-aware LSM + Content-addressed Storage.**

Ich würde sie intern als _Cairn Hybrid Storage Engine_ bezeichnen.

Warum?

RocksDB verwendet bereits einen sehr guten Grundmechanismus:

- Memtable.
    
- Write-Ahead Log.
    
- Sortierte SST-Dateien.
    
- Hintergrundkompaktierung.
    

Das ist eine gute Basis für hohe Schreiblast und SSD-Storage.

GitHub+2

Allerdings würde ich die Datenplatzierung, das Versionsmodell, die Indizierung und die Kontextabfrage speziell für Agenten optimieren.

## **3.1 Drei physische Speicherbereiche**

HOT

**Mutable Memory Layer**

RAM + WAL + aktuelle Transaktionsdaten

Kleine Änderungen, aktive Agentenzustände, neue Ereignisse, kürzlich aktualisierte Metadaten.

WARM

**Locality-aware Segment Store**

NVMe SSD + sortierte Runs + Retrieval Bundles

Aktive Dokumente, Memory-Chunks, Indizes, Embeddings, Beziehungen.

COLD

**Immutable Archive Store**

Komprimierte Segmente + große Artefakte

Alte Versionen, historische Events, unveränderte Dokumente und ältere Agentenläufe.

Diese Einteilung ist logisch, nicht einfach eine starre Aufteilung anhand des Alters.

Ein sehr altes Dokument kann noch HOT sein, wenn es permanent verwendet wird.

Daher sollte die Engine Zugriffsfrequenz, Größe, Änderungsrate und Retrieval-Relevanz berücksichtigen.

## **3.2 Locality-aware Storage – Daten physisch zusammenbringen**

Das ist eine meiner wichtigsten Optimierungen.

Normale Datenbanken optimieren häufig anhand von Schlüsseln und allgemeinen Tabellenzugriffen.

CairnDB sollte zusätzlich wissen, welche Datensätze typischerweise gemeinsam gelesen werden.

Beispiel:

Ein Agent arbeitet an einem Repository.

Er benötigt häufig gemeinsam:

```
Repository Metadata
    |
    +-- Current Branch
    |
    +-- File Manifest
    |
    +-- Relevant File Chunks
    |
    +-- Related Decisions
    |
    +-- Previous Tool Results
    |
    +-- Relevant Embeddings
```

Wenn diese Daten an komplett unterschiedlichen Stellen auf der SSD liegen, entstehen unnötige Leseoperationen.

Mein Vorschlag ist eine neue Einheit:

**Retrieval Bundle**

Ein Retrieval Bundle gruppiert physisch zusammengehörige Datensätze.

Beispielsweise:

```
Bundle: repository_42

Page 0:
  Repository Metadata
  Branch Metadata
  File Manifest

Page 1:
  Important File Chunks

Page 2:
  Related Events

Page 3:
  Metadata and Relationships

Page 4:
  Embedding References
```

Die Idee ist nicht, jeden Inhalt redundant in jedem Bundle abzulegen.

Vielmehr enthält es kleine, unmittelbar benötigte Metadaten und Referenzen auf die eigentlichen Daten.

Die eigentlichen Inhalte werden in zusammenhängenden Extents gespeichert.

### **Wie erfolgt die Gruppierung?**

Jedes Objekt bekommt einen `locality_key`.

```
struct LocalityKey {
    tenant_id: TenantId,
    workspace_id: WorkspaceId,
    scope_id: ScopeId,
    entity_id: Option<EntityId>,
}
```

Beim Schreiben berücksichtigt die Engine:

- Gleicher Workspace.
    
- Gleiche Konversation.
    
- Gleiches Dokument.
    
- Gleicher Agentenlauf.
    
- Häufig gemeinsam abgerufene Objekte.
    
- Gleiche Retrieval-Klasse.
    

Die Compaction soll nicht nur nach Key-Reihenfolge arbeiten, sondern diese Lokalitätsinformationen möglichst erhalten.

Wichtig: Bei NVMe-SSDs kontrolliert das Storage-System nicht die tatsächliche Platzierung der NAND-Zellen. Diese übernimmt der Controller. Wir können aber die logische Platzierung in zusammenhängenden Extents, Pages und I/O-Batches kontrollieren und damit unnötige Random Reads reduzieren.

Das ist ein realistischer und wirkungsvoller Ansatz für physische Nähe.

## **3.3 Segment-Format**

Jeder Storage-Segment sollte einen einheitlichen Header besitzen, aber unterschiedliche Record-Typen unterstützen.

Ein möglicher Aufbau:

```
┌──────────────────────────────────┐
│ Segment Header                   │
│ Format Version                   │
│ Segment ID                       │
│ Generation                       │
│ Min Commit / Max Commit           │
│ Locality Group                   │
│ Compression Type                 │
│ Encryption Metadata              │
│ Checksum                         │
├──────────────────────────────────┤
│ Block Directory                  │
├──────────────────────────────────┤
│ Primary Key Records              │
├──────────────────────────────────┤
│ Version Records                  │
├──────────────────────────────────┤
│ Relationship Records             │
├──────────────────────────────────┤
│ Inline Metadata                  │
├──────────────────────────────────┤
│ Record Offset Table              │
├──────────────────────────────────┤
│ Block Checksums                  │
└──────────────────────────────────┘
```

Ich würde drei Ebenen unterscheiden:

**1. Key Layer**

Enthält kompakte Informationen und Verweise auf Daten.

**2. Value Layer**

Enthält kleinere Werte direkt und größere Werte als Referenzen auf Extents.

**3. Blob Layer**

Enthält große, unveränderliche Inhalte.

Das orientiert sich teilweise am Value-Separation-Ansatz von WiscKey. Dort werden Keys und große Values getrennt gespeichert, um unnötige Datenbewegungen während der LSM-Kompaktierung zu reduzieren.

USENIX

CairnDB erweitert dieses Prinzip durch die zusätzliche Lokalitäts- und Versionsdimension.

### **Größen und Layout als erste Implementierungshypothese**

|Element|Vorgeschlagener Startwert|
|---|---|
|Logical Page|16 KiB|
|Data Block|64–256 KiB|
|Inline Value|Bis ca. 128–256 Byte|
|Größere Values|Separater Value-Extent|
|Kompression HOT|LZ4|
|Kompression COLD|Zstd|
|Blob-Referenz|64 Bit Offset + Segment-ID oder kompakter Handle|

Diese Werte sind Startparameter für Benchmarks, keine universell optimalen Konstanten.

Ein wichtiger Punkt: Embeddings werden nicht mit gewöhnlichen Textkompressionsverfahren behandelt. Hier sind quantisierte Vektorformate wesentlich sinnvoller.

# **4. Atomare Transaktionen: Die wichtigste Eigenschaft**

Das System muss Datenänderungen über verschiedene Objekttypen atomar ausführen können.

Beispiel:

Ein Agent hat eine Information erkannt und möchte sie speichern.

Es müssen gleichzeitig folgende Änderungen erfolgen:

```
1. Neues Event speichern
2. Semantic Memory aktualisieren
3. Source/Provenance anlegen
4. Beziehung zwischen beiden erzeugen
5. Neue Version des Working State speichern
6. Embedding-Auftrag registrieren
```

Entweder wird alles committed oder nichts davon.

Dafür würde ich ein eigenes **MVCC Transaction Engine** bauen.

## **4.1 MVCC als Fundament**

Jede Änderung bekommt eine eindeutige Commit-Version.

```
type CommitId = u64;

struct VersionedRecord {
    object_id: ObjectId,

    created_at: CommitId,
    superseded_at: Option<CommitId>,

    payload_ref: PayloadRef,
}
```

Ein Leser startet mit einem Snapshot:

```
let snapshot = db.begin_read().await?;
```

Dieser Snapshot bleibt während seiner gesamten Transaktion unverändert.

Ein paralleler Writer kann unterdessen neue Daten committed haben.

Der Leser sieht weiterhin die alte, konsistente Version.

Das erlaubt:

- Mehrere parallele Leser.
    
- Gleichzeitige Hintergrundkompaktierung.
    
- Atomare Mehrfachänderungen.
    
- Historische Abfragen.
    
- Wiederherstellung früherer Zustände.
    

## **4.2 Commit-Protokoll**

Ich würde einen zentralen Commit-Sequencer verwenden.

Das bedeutet nicht, dass alle Daten synchron nacheinander verarbeitet werden müssen. Die Verarbeitung und Vorbereitung können parallel erfolgen. Lediglich die Veröffentlichung einer Commit-Reihenfolge wird serialisiert.

Ablauf:

Index EngineStorageWALTransaction EngineAgentIndex EngineStorageWALTransaction EngineAgent#chatgpt-mermaid-_r_aa_{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,Segoe UI,Helvetica,Apple Color Emoji,Arial,sans-serif,Segoe UI Emoji,Segoe UI Symbol;font-size:16px;fill:rgb(237, 237, 237);}@keyframes edge-animation-frame{from{stroke-dashoffset:0;}}@keyframes dash{to{stroke-dashoffset:0;}}#chatgpt-mermaid-_r_aa_ .edge-animation-slow{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}#chatgpt-mermaid-_r_aa_ .edge-animation-fast{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}#chatgpt-mermaid-_r_aa_ .error-icon{fill:rgb(48, 48, 48);}#chatgpt-mermaid-_r_aa_ .error-text{fill:rgb(237, 237, 237);stroke:rgb(237, 237, 237);}#chatgpt-mermaid-_r_aa_ .edge-thickness-normal{stroke-width:1px;}#chatgpt-mermaid-_r_aa_ .edge-thickness-thick{stroke-width:3.5px;}#chatgpt-mermaid-_r_aa_ .edge-pattern-solid{stroke-dasharray:0;}#chatgpt-mermaid-_r_aa_ .edge-thickness-invisible{stroke-width:0;fill:none;}#chatgpt-mermaid-_r_aa_ .edge-pattern-dashed{stroke-dasharray:3;}#chatgpt-mermaid-_r_aa_ .edge-pattern-dotted{stroke-dasharray:2;}#chatgpt-mermaid-_r_aa_ .marker{fill:rgb(175, 175, 175);stroke:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ .marker.cross{stroke:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ svg{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,Segoe UI,Helvetica,Apple Color Emoji,Arial,sans-serif,Segoe UI Emoji,Segoe UI Symbol;font-size:16px;}#chatgpt-mermaid-_r_aa_ p{margin:0;}#chatgpt-mermaid-_r_aa_ .actor{stroke:rgb(31, 78, 148);fill:rgb(9, 23, 44);stroke-width:1;}#chatgpt-mermaid-_r_aa_ rect.actor.outer-path[data-look="neo"]{filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-_r_aa_ rect.note[data-look="neo"]{stroke:rgb(58, 132, 63);fill:rgb(48, 48, 48);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-_r_aa_ text.actor>tspan{fill:rgb(237, 237, 237);stroke:none;}#chatgpt-mermaid-_r_aa_ .actor-line{stroke:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ .innerArc{stroke-width:1.5;stroke-dasharray:none;}#chatgpt-mermaid-_r_aa_ .messageLine0{stroke-width:1.5;stroke-dasharray:none;stroke:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ .messageLine1{stroke-width:1.5;stroke-dasharray:2,2;stroke:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ [id$="-arrowhead"] path{fill:rgb(175, 175, 175);stroke:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ .sequenceNumber{fill:#505050;}#chatgpt-mermaid-_r_aa_ [id$="-sequencenumber"]{fill:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ [id$="-crosshead"] path{fill:rgb(175, 175, 175);stroke:rgb(175, 175, 175);}#chatgpt-mermaid-_r_aa_ .messageText{fill:rgb(237, 237, 237);stroke:none;}#chatgpt-mermaid-_r_aa_ .labelBox{stroke:rgba(255, 255, 255, 0.15);fill:rgb(0, 0, 0);filter:none;}#chatgpt-mermaid-_r_aa_ .labelText,#chatgpt-mermaid-_r_aa_ .labelText>tspan{fill:rgb(237, 237, 237);stroke:none;}#chatgpt-mermaid-_r_aa_ .loopText,#chatgpt-mermaid-_r_aa_ .loopText>tspan{fill:rgb(237, 237, 237);stroke:none;}#chatgpt-mermaid-_r_aa_ .sectionTitle,#chatgpt-mermaid-_r_aa_ .sectionTitle>tspan{fill:rgb(237, 237, 237);stroke:none;}#chatgpt-mermaid-_r_aa_ .loopLine{stroke-width:2px;stroke-dasharray:2,2;stroke:rgba(255, 255, 255, 0.15);fill:rgba(255, 255, 255, 0.15);}#chatgpt-mermaid-_r_aa_ .note{stroke:rgb(58, 132, 63);fill:rgb(48, 48, 48);}#chatgpt-mermaid-_r_aa_ .noteText,#chatgpt-mermaid-_r_aa_ .noteText>tspan{fill:rgb(237, 237, 237);stroke:none;font-weight:normal;}#chatgpt-mermaid-_r_aa_ .activation0{fill:rgb(48, 48, 48);stroke:hsl(0, 0%, 8.8235294118%);}#chatgpt-mermaid-_r_aa_ .activation1{fill:rgb(48, 48, 48);stroke:hsl(0, 0%, 8.8235294118%);}#chatgpt-mermaid-_r_aa_ .activation2{fill:rgb(48, 48, 48);stroke:hsl(0, 0%, 8.8235294118%);}#chatgpt-mermaid-_r_aa_ .actorPopupMenu{position:absolute;}#chatgpt-mermaid-_r_aa_ .actorPopupMenuPanel{position:absolute;fill:rgb(9, 23, 44);box-shadow:0px 8px 16px 0px rgba(0,0,0,0.2);filter:drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4));}#chatgpt-mermaid-_r_aa_ .actor-man circle,#chatgpt-mermaid-_r_aa_ line{fill:rgb(9, 23, 44);stroke-width:2px;}#chatgpt-mermaid-_r_aa_ g rect.rect{filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));stroke:rgb(31, 78, 148);}#chatgpt-mermaid-_r_aa_ .node .neo-node{stroke:rgb(31, 78, 148);}#chatgpt-mermaid-_r_aa_ [data-look="neo"].node rect,#chatgpt-mermaid-_r_aa_ [data-look="neo"].cluster rect,#chatgpt-mermaid-_r_aa_ [data-look="neo"].node polygon{stroke:url(#chatgpt-mermaid-_r_aa_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-_r_aa_ [data-look="neo"].swimlane.cluster rect{filter:none;}#chatgpt-mermaid-_r_aa_ [data-look="neo"].node path{stroke:url(#chatgpt-mermaid-_r_aa_-gradient);stroke-width:1px;}#chatgpt-mermaid-_r_aa_ [data-look="neo"].node .outer-path{filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-_r_aa_ [data-look="neo"].node .neo-line path{stroke:rgb(31, 78, 148);filter:none;}#chatgpt-mermaid-_r_aa_ [data-look="neo"].node circle{stroke:url(#chatgpt-mermaid-_r_aa_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-_r_aa_ [data-look="neo"].node circle .state-start{fill:#000000;}#chatgpt-mermaid-_r_aa_ [data-look="neo"].icon-shape .icon{fill:url(#chatgpt-mermaid-_r_aa_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-_r_aa_ [data-look="neo"].icon-shape .icon-neo path{stroke:url(#chatgpt-mermaid-_r_aa_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-_r_aa_ :root{--mermaid-font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,Segoe UI,Helvetica,Apple Color Emoji,Arial,sans-serif,Segoe UI Emoji,Segoe UI Symbol;}Begin TransactionSnapshot + Write SetWrite Events / Facts / StateWrite immutable payload extentsAppend transaction recordsValidate conflictsAppend COMMIT markerDurable commitPublish new MVCC rootCommit SuccessIndex update eventUpdate index deltas

Wichtige Details:

1. Große Payloads werden vor dem Commit-Datensatz dauerhaft geschrieben.
    
2. Der WAL enthält alle notwendigen Transaktionsinformationen und Referenzen.
    
3. Der Commit-Marker wird erst nach erfolgreichem Persistieren der notwendigen Daten geschrieben.
    
4. Nach dem dauerhaften Commit wird der neue Snapshot veröffentlicht.
    
5. Bei einem Crash werden uncommittete Transaktionen verworfen.
    
6. Nicht mehr referenzierte Extents können später eingesammelt werden.
    

Der WAL ist damit die maßgebliche Recovery-Quelle, nicht irgendein zufällig bereits aktualisierter Suchindex.

### **Group Commit**

Einzelne Agentenoperationen dürfen nicht jeweils zwingend einen vollständigen physischen Flush verursachen.

Ich würde mehrere Commit-Anfragen bündeln:

```
Agent A ─┐
Agent B ─┼── Group Commit ── WAL Sync
Agent C ─┘
```

Konfigurierbare Durability-Modi:

```
enum Durability {
    Sync,
    GroupSync,
    Async,
}
```

`Sync` garantiert die dauerhafte Speicherung vor der Rückmeldung.

`GroupSync` bündelt mehrere Commits, gibt aber ebenfalls erst nach gesicherter Persistierung zurück.

`Async` ermöglicht höhere Geschwindigkeit, hat aber ein ausdrücklich definiertes Verlustfenster bei einem Systemausfall.

Diese Unterscheidung ist wichtig, weil bei Agentensystemen nicht alle Daten dieselbe Dauerhaftigkeit benötigen.

Ein temporärer Zwischenzustand kann anders behandelt werden als eine bereits bestätigte Tool-Ausführung oder eine wertvolle langfristige Memory-Änderung.

## **4.3 Optimistic Concurrency Control**

Ich würde nicht für jede Agententransaktion globale Locks verwenden.

Jede Transaktion bekommt:

- Snapshot-Version.
    
- Read Set.
    
- Write Set.
    
- Optional Range-Read Set.
    

Beim Commit werden Konflikte geprüft.

Beispiel:

```
Agent A liest Object 42 auf Version 10.
Agent B verändert Object 42 auf Version 11.
Agent A versucht Version 12 zu schreiben.
```

A erkennt den Konflikt und kann seine Transaktion wiederholen.

Besonders interessant sind Agenten, die an verschiedenen Teilen desselben Workspace arbeiten.

Hier darf nicht jeder neue Event automatisch mit jedem anderen Event kollidieren.

Deshalb würde ich unterschiedliche Konfliktregeln verwenden:

|Operation|Konfliktstrategie|
|---|---|
|Event anhängen|Append-only, normalerweise konfliktarm|
|Working State ändern|Compare-and-swap|
|Semantic Fact ändern|Optimistic Version Check|
|Tool State ändern|Idempotency Key + CAS|
|Graph Edge hinzufügen|Versionsprüfung auf Relation|
|Branch Merge|Explizite Konflikterkennung|

# **5. Versionskontrolle direkt im Storage**

Das ist ein Bereich, in dem CairnDB einen besonders starken Mehrwert bieten könnte.

Ich würde Versionskontrolle nicht als nachträgliche Zusatzfunktion entwickeln.

Sie gehört in die grundlegende Datenarchitektur.

Mein Vorbild ist hier nicht nur klassische Datenbank-MVCC, sondern zusätzlich das konzeptionelle Modell von Git.

## **5.1 Commit-DAG**

Jeder Commit referenziert seinen Vorgänger oder seine Vorgänger.

```
           Commit 1
               |
           Commit 2
               |
           Commit 3
             /   \
        Commit 4  Commit 5
             \   /
           Commit 6
```

Damit entstehen Branches und Merges.

Ein Agent kann einen separaten Arbeitszweig erzeugen:

```
workspace/main
      |
      |
   commit_42
     /   \
    /     \
Agent A   Agent B
  |         |
  43        44
  |         |
  |      Analysis
  |      Finished
  |         |
  45        |
   \       /
    Merge
      |
   commit 46
```

Agent A und Agent B können unabhängig Änderungen vornehmen.

Die Änderungen werden später zusammengeführt.

## **5.2 Copy-on-Write Manifests**

Ein Branch darf nicht alle Daten kopieren.

Stattdessen speichert jeder Commit Referenzen auf bestehende immutable Segmente und neue Änderungen.

Beispiel:

```
Manifest 100
  ├── Object Segment A
  ├── Object Segment B
  ├── Vector Segment A
  └── Graph Segment A

Manifest 101
  ├── Object Segment A
  ├── Object Segment B
  ├── Object Segment C (new)
  ├── Vector Segment A
  └── Graph Segment A
```

Nur die tatsächlich veränderten Inhalte benötigen neue physische Speicherung.

Zusätzlich würde ich eine Merkle-basierte Manifeststruktur verwenden.

Das bringt:

- Schnelle Snapshot-Erstellung.
    
- Strukturierte Diffs.
    
- Verifizierbare Datenintegrität.
    
- Effiziente Branches.
    
- Gemeinsame Nutzung unveränderlicher Daten.
    
- Einfaches Garbage Collection anhand erreichbarer Roots.
    

## **5.3 Content-Addressed Storage**

Unveränderte Inhalte sollten nur einmal innerhalb ihres Sicherheits- und Speicherbereichs gespeichert werden.

Beispiel:

```
Content Hash:
  blake3(content)

Object A ──┐
           ├── Blob Hash X
Object B ──┘
```

Zwei Referenzen auf identische Inhalte zeigen auf dasselbe Blob.

Das spart Speicherplatz, insbesondere bei:

- Agenten-Checkpoints.
    
- Dokumentversionen.
    
- Repository-Snapshots.
    
- Wiederholten Tool-Ergebnissen.
    
- Ähnlichen Kontextartefakten.
    

Wichtig wäre eine tenantgebundene Deduplication, um keine Informationen durch globale Content-Hash-Abfragen zwischen Mandanten preiszugeben.

## **5.4 Versionierte Daten und historische Abfragen**

Die API sollte beispielsweise erlauben:

```
let current = db.get(object_id).await?;

let historical = db
    .get_at(object_id, commit_id)
    .await?;

let diff = db
    .diff(commit_a, commit_b)
    .await?;
```

Zusätzlich:

```
db.branch("experiment")
    .from_commit(commit_id)
    .create()
    .await?;
```

Und:

```
db.merge()
    .source("agent_b")
    .target("main")
    .strategy(MergeStrategy::DetectConflicts)
    .execute()
    .await?;
```

Damit wird Versionierung eine native Funktion der Engine.

Apache Iceberg zeigt beispielsweise, wie Snapshot-Isolation und unveränderliche Datendateien kombiniert werden können. CairnDB würde dieses Prinzip stärker auf einzelne Agentenobjekte, deren Beziehungen und Arbeitszustände übertragen.

Apache Iceberg™

# **6. Das besondere Problem: Vektoren und Suchindizes**

Hier darf CairnDB keinen klassischen Architekturfehler machen.

Ein häufiger Fehler ist, die Vektordatenbank als eigenständige Wahrheit zu behandeln.

Dann entstehen Probleme:

```
Primary DB:
  Record updated

Vector DB:
  Old embedding

Application:
  Inconsistent retrieval
```

Ich würde daher einen anderen Weg wählen.

**Ein gemeinsamer MVCC-Datenkern, mehrere versionierte Suchindizes und ein konsistenter Delta-Pfad.**

## **6.1 Vier Index-Arten**

**1. Vector Index**

Semantische Ähnlichkeit, Embedding Search, ANN

**2. Full-Text Index**

BM25, Begriffe, Phrasen, Positionen

**3. Graph Index**

Entitäten, Zusammenhänge, mehrstufige Relationen

**4. Metadata Index**

Zeit, Typ, Tenant, Status, Berechtigungen, Tags

Alle vier arbeiten mit stabilen Object IDs und Commit-Versionen.

## **6.2 Vector Storage: Hybrid aus Delta und Base Index**

Ich würde nicht alle Vektoren bei jedem Update in einen großen HNSW-Index einfügen.

Stattdessen:

```
                 Vector Query
                      |
               Query Planner
                      |
           +----------+----------+
           |                     |
      Base Index             Delta Index
           |                     |
       IVF / PQ             Exact / Small ANN
           |                     |
           +----------+----------+
                      |
               MVCC Filtering
                      |
                Top Candidates
```

Die Idee:

- Große, relativ stabile Datenbestände bekommen einen kompakten Base Index.
    
- Neue Vektoren landen zunächst in einem schnell aktualisierbaren Delta Index.
    
- Gelöschte und ersetzte Vektoren werden durch versionierte Sichtbarkeitsinformationen ausgefiltert.
    
- Hintergrundprozesse übernehmen die Zusammenführung.
    
- Die eigentlichen Originalvektoren bleiben verfügbar, auch wenn der ANN-Index quantisiert ist.
    

Für unterschiedliche Größenordnungen würde ich unterschiedliche Algorithmen einsetzen.

|Situation|Indexstrategie|
|---|---|
|Wenige Tausend Vektoren|Exakte SIMD-Suche|
|Mittlere Datenmengen|HNSW oder IVF|
|Große Datenmengen|IVF + quantisierte Vektoren|
|Sehr große SSD-Datensätze|DiskANN/Vamana-artige Indizes|

DiskANN ist interessant, weil es ANN-Suche gezielt für SSD-basierten Storage entwickelt und Filter- sowie Aktualisierungsszenarien weiterentwickelt hat.

proceedings.neurips.cc+1

Auch aktuelle Vektorindex-Implementierungen wie Lance unterstützen verschiedene Kombinationen aus IVF, Product Quantization, Scalar Quantization und HNSW. Das würde ich als Benchmark-Referenz heranziehen.

Rust+1

## **6.3 Ein wichtiger Trick: Index Watermarks**

Ein asynchron aktualisierter Index kann trotzdem konsistente Ergebnisse liefern.

Dafür braucht jeder Suchindex einen `applied_commit_id`.

Beispiel:

```
Primary Data:
  Applied through commit 5000

Vector Base Index:
  Applied through commit 4950

Vector Delta:
  Commits 4951–5000

Text Index:
  Applied through commit 4980

Text Delta:
  Commits 4981–5000
```

Eine Query mit Snapshot 5000 durchsucht jeweils:

```
Base Index + Delta since watermark
```

Alle Kandidaten werden anschließend gegen den angefragten MVCC-Snapshot validiert.

Dadurch entstehen keine Suchlücken allein deshalb, weil eine Hintergrundindexierung noch nicht fertig ist.

Für historische Snapshots muss die Engine zusätzlich die Sichtbarkeit der Versionen und den passenden Indexzustand beachten. Daher bekommen immutable Indexsegmente eigene Generationen und Commit-Bereiche.

Das ist einer der zentralen architektonischen Vorteile gegenüber einer einfachen Kombination aus RocksDB und einer externen Vector DB.

## **6.4 Embeddings als versionierte Daten**

Ein Embedding darf niemals einfach nur eine Liste aus Float-Werten sein.

Ich würde folgendes Modell verwenden:

```
struct EmbeddingRecord {
    object_id: ObjectId,
    revision: RevisionId,

    model_id: ModelId,
    model_version: ModelVersion,

    dimensions: u32,
    encoding: VectorEncoding,
    metric: DistanceMetric,

    values_ref: PayloadRef,

    created_at: CommitId,
}
```

Warum?

Ein Dokument kann verschiedene Embeddings besitzen:

- Verschiedene Embedding-Modelle.
    
- Unterschiedliche Dimensionen.
    
- Unterschiedliche Vektorversionen.
    
- Unterschiedliche Normalisierung.
    
- Unterschiedliche Anwendungszwecke.
    

Es muss immer nachvollziehbar sein, welches Modell für welchen Vektor verantwortlich ist.

Wird das Embedding-Modell gewechselt, werden die bisherigen Vektoren nicht überschrieben.

Es entsteht eine neue Vektorversion, die über eine neue Indexgeneration erreichbar wird.

# **7. LLM-native Retrieval Engine**

Hier liegt meines Erachtens das größte Differenzierungspotenzial.

Eine klassische Vector DB beantwortet:

> Welche Dokumente sind meiner Query am ähnlichsten?

Eine speziell für LLMs gebaute Storage-Engine muss stattdessen beantworten:

> Welche Informationen helfen dem Agenten bei seiner aktuellen Aufgabe am meisten, unter Berücksichtigung seines Wissensstands, seiner Berechtigungen, der Zuverlässigkeit der Quellen und seines verfügbaren Token-Budgets?

Das ist ein fundamental anderer Ansatz.

## **7.1 Hybrid Retrieval**

Eine Abfrage sollte mehrere Verfahren kombinieren.

Beispiel:

```
Query:
"Warum ist der letzte Deployment-Versuch gescheitert?"
```

CairnDB führt parallel aus:

1. Semantische Ähnlichkeitssuche.
    
2. Exakte Suche nach Deployment-Fehlercodes.
    
3. Zeitliche Suche nach vorherigen Events.
    
4. Graphsuche nach betroffenen Komponenten.
    
5. Filterung nach Workspace und Agentenberechtigungen.
    

Danach werden die Ergebnisse zusammengeführt.

Ich würde Reciprocal Rank Fusion (RRF) als Ausgangspunkt und anschließend eine konfigurierbare Rankingfunktion einsetzen.

Beispiel:

\[ Score = w_v S_v + w_f S_f + w_t S_t + w_p S_p + w_c S_c \]

Dabei:

- \(S_v\): semantische Ähnlichkeit.
    
- \(S_f\): Full-Text-Relevanz.
    
- \(S_t\): zeitliche Relevanz.
    
- \(S_p\): Provenance-Qualität.
    
- \(S_c\): Kontextbezug.
    

Die Gewichte sollten je nach Agententyp oder Anwendungsfall konfigurierbar sein.

## **7.2 Token-Budget-aware Retrieval**

Das ist eine Funktion, die ich nativ in CairnDB integrieren würde.

Beispiel:

```
let context = db.context()
    .query("Why did the deployment fail?")
    .scope(workspace_id)
    .token_budget(8000)
    .include_sources(true)
    .diversity(0.7)
    .recency_weight(0.3)
    .execute()
    .await?;
```

Der Context Engine stehen beispielsweise folgende Ergebnisse zur Verfügung:

|Information|Relevanz|Tokens|
|---|---|---|
|Fehlerprotokoll|0.99|450|
|Deployment-Konfiguration|0.94|900|
|Frühere Fehlversuche|0.90|650|
|Repository-Dokumentation|0.86|2.000|
|Unrelated Logs|0.35|1.200|

Die Engine muss nun nicht einfach die Top 10 Ergebnisse liefern.

Sie löst vielmehr ein Auswahlproblem unter Token-Beschränkung.

Vereinfacht:

\[ \max \sum_i x_i R_i \]

unter:

\[ \sum_i x_i T_i \leq B \]

Dabei sind:

- \(R_i\): Relevanz eines Kontextobjektes.
    
- \(T_i\): Token-Kosten.
    
- \(B\): verfügbares Token-Budget.
    
- \(x_i\): Auswahl des Objektes.
    

Zusätzlich brauchen wir Abhängigkeiten.

Beispielsweise dürfen mehrere isolierte Code-Chunks nicht ohne den notwendigen Kontext zusammengesetzt werden.

Die Engine sollte deshalb auch Folgendes berücksichtigen:

- Zusammengehörige Chunks.
    
- Dokumenthierarchien.
    
- Quellcode-Kontext.
    
- Redundanz.
    
- Quellenvielfalt.
    
- Informationsalter.
    
- Konflikte zwischen Fakten.
    
- Tokenisierung des verwendeten Modells.
    

Das macht CairnDB zu einer Datenbank, die nicht nur Daten abruft, sondern LLM-Kontext gezielt zusammenstellt.

## **7.3 Native Token-Metadaten**

Jeder Chunk kann folgende Werte besitzen:

```
struct ChunkMetadata {
    document_id: DocumentId,

    byte_start: u64,
    byte_end: u64,

    token_count: u32,
    tokenizer_id: TokenizerId,

    chunk_index: u32,
    parent_chunk: Option<ChunkId>,

    semantic_role: SemanticRole,
}
```

Damit müssen Agenten nicht bei jeder Abfrage Tokenzahlen erneut berechnen.

Da unterschiedliche Modelle unterschiedliche Tokenizer haben, muss die Tokenzahl immer an eine konkrete Tokenizer-Version gebunden sein.

## **7.4 Context Cache**

Eine weitere sinnvolle Funktion wäre ein nativer Context Cache.

Gespeichert werden könnte:

```
Query Hash
Snapshot ID
Agent ID
Model ID
Tokenizer ID
Authorization Scope
Selected Object IDs
Selected Revisions
Token Count
Context Hash
```

Wird derselbe Kontext erneut benötigt und haben sich die zugrunde liegenden Daten nicht verändert, kann die Engine den Kontext wiederverwenden.

Wichtig: Die Cache-Validierung muss auch Snapshot, Berechtigungen und Modellparameter berücksichtigen.

# **8. Eine weitere wichtige Innovation: Temporal und Causal Storage**

Ich würde CairnDB zusätzlich eine native zeitliche und kausale Datenmodellierung geben.

Das ist besonders interessant für Agenten, die über Stunden, Tage oder Monate hinweg arbeiten.

Eine Information besitzt nicht nur eine Version, sondern unterschiedliche Zeitdimensionen.

Beispiel:

```
Fact:
  Server version = 2.4

Transaction Time:
  Wann wurde die Information gespeichert?

Valid Time:
  Wann galt diese Information tatsächlich?

Observation Time:
  Wann hat der Agent die Information erkannt?

Source Time:
  Wann ist die ursprüngliche Quelle entstanden?
```

Das ermöglicht Fragen wie:

- Welche Serverversion war am 12. Oktober aktiv?
    
- Was wusste der Agent zu diesem Zeitpunkt?
    
- Wann wurde der Fehler tatsächlich entdeckt?
    
- Welche Informationen haben zu einer späteren Entscheidung geführt?
    
- Welche Entscheidung beruhte auf inzwischen überholtem Wissen?
    

Dafür würde ich `valid_from`, `valid_until`, `observed_at` und `commit_id` als native Metadaten unterstützen.

## **8.1 Causal Graph**

Über die reine Versionshistorie hinaus können wir auch kausale Zusammenhänge abbilden.

Beispiel:

```
Event A:
Deployment started
       |
       v
Event B:
Database migration failed
       |
       v
Event C:
Application unavailable
       |
       v
Event D:
Rollback performed
       |
       v
Event E:
Deployment succeeded
```

Diese Informationen lassen sich später als strukturierte Erfahrung abrufen.

Es entsteht ein kausal nachvollziehbarer Agentenverlauf.

Das ist wesentlich aussagekräftiger als eine Sammlung unsortierter Chatnachrichten.

# **9. Rust-Implementierung: Wie ich die Engine tatsächlich aufbauen würde**

Ich würde einen klar abgegrenzten Core entwickeln und keine riesige monolithische Implementierung.

Die Module sollten unabhängig benchmarkbar und austauschbar sein.

## **9.1 Crate-Struktur**

```
cairndb/
│
├── cairn-core/
│   ├── ids.rs
│   ├── records.rs
│   ├── versions.rs
│   ├── types.rs
│   └── errors.rs
│
├── cairn-wal/
│   ├── writer.rs
│   ├── reader.rs
│   ├── recovery.rs
│   ├── group_commit.rs
│   └── checksums.rs
│
├── cairn-mvcc/
│   ├── transaction.rs
│   ├── snapshots.rs
│   ├── conflicts.rs
│   └── visibility.rs
│
├── cairn-memtable/
│   ├── memtable.rs
│   ├── active_index.rs
│   └── flush.rs
│
├── cairn-lsm/
│   ├── sorted_runs.rs
│   ├── block_index.rs
│   ├── compaction.rs
│   └── bloom.rs
│
├── cairn-object/
│   ├── object_store.rs
│   ├── blob_store.rs
│   ├── content_addressing.rs
│   └── dedup.rs
│
├── cairn-locality/
│   ├── bundles.rs
│   ├── placement.rs
│   └── compaction_policy.rs
│
├── cairn-vector/
│   ├── vector_store.rs
│   ├── quantization.rs
│   ├── delta_index.rs
│   ├── ivf.rs
│   └── ann.rs
│
├── cairn-search/
│   ├── fulltext.rs
│   ├── hybrid.rs
│   ├── ranking.rs
│   └── query_planner.rs
│
├── cairn-graph/
│   ├── edges.rs
│   ├── adjacency.rs
│   └── traversal.rs
│
├── cairn-context/
│   ├── token_budget.rs
│   ├── context_builder.rs
│   ├── deduplication.rs
│   └── context_cache.rs
│
├── cairn-history/
│   ├── commits.rs
│   ├── branches.rs
│   ├── merge.rs
│   └── diff.rs
│
├── cairn-security/
│   ├── encryption.rs
│   ├── permissions.rs
│   └── tenant_isolation.rs
│
├── cairn-runtime/
│   ├── scheduler.rs
│   ├── workers.rs
│   └── io.rs
│
├── cairn-api/
│   ├── database.rs
│   ├── transactions.rs
│   └── queries.rs
│
└── cairn-bench/
    ├── writes.rs
    ├── reads.rs
    ├── retrieval.rs
    ├── compaction.rs
    └── workloads.rs
```

Ich würde die Core-Engine vollständig in Rust implementieren.

Python, Node.js und andere Schnittstellen kommen später als Bindings hinzu.

Die Storage-Engine darf nicht von einem Python- oder Agenten-Framework abhängig sein.

## **9.2 Welche Rust-Technologien?**

Folgende Komponenten wären meine Ausgangsauswahl:

|Bereich|Technologie / Ansatz|
|---|---|
|Speicherverwaltung|`bytes`, eigene aligned Page Buffers|
|Parallelität|`crossbeam`, `parking_lot`|
|Snapshot-Publishing|`arc-swap`|
|Asynchrone I/O|Tokio als optionale Runtime|
|Linux-I/O|Optional `io_uring`|
|Checksums|CRC32C|
|Content Hashes|BLAKE3|
|Kompression|LZ4, Zstd|
|Bitmap-Indizes|Roaring Bitmaps|
|Vektoren|Eigene SIMD-Kernels|
|Serialization|Eigene binäre Record-Formate|
|Testing|`proptest`, `criterion`, Fault Injection|

Einige dieser Komponenten sollten abstrahiert werden, damit spätere Implementierungen ausgetauscht werden können.

Besonders wichtig wäre eine minimale Abhängigkeitsstruktur im kritischen Storage-Pfad.

Ich würde außerdem nicht überall `async` einsetzen.

Ein synchroner, gut kontrollierter Commit-Kern mit klar definierten I/O-Grenzen kann wesentlich einfacher zu analysieren und zu optimieren sein. Asynchrone Verarbeitung ist besonders für Hintergrundjobs, parallele Reads und Indexaufbau sinnvoll.

## **9.3 Multithreading-Konzept**

Ich würde die Engine folgendermaßen strukturieren:

```
                    API Layer
                        |
                 Transaction Router
                        |
          +-------------+-------------+
          |             |             |
       Shard 0        Shard 1       Shard 2
          |             |             |
       Memtable       Memtable      Memtable
          |             |             |
       WAL Writer     WAL Writer    WAL Writer
          |             |             |
          +-------------+-------------+
                        |
                  Commit Sequencer
                        |
                  Snapshot Manager
                        |
             Background Job Scheduler
                        |
          +-------------+-------------+
          |             |             |
      Compaction     Vector Build    GC
```

Der Commit Sequencer ordnet abgeschlossene Transaktionen eindeutig ein.

Hintergrundoperationen können parallel laufen.

Für Leser würde ich immutable Datenstrukturen, Snapshot-Referenzen und Epoch-basierte Speicherfreigabe einsetzen.

Das vermeidet globale Locks auf jedem Read.

### **Meine Empfehlung zur Shard-Aufteilung**

Nicht einfach nur anhand von Key-Hashing sharden.

Ich würde zunächst entlang logischer Isolationseinheiten partitionieren:

```
Tenant
  |
  Workspace
    |
    Agent / Session / Collection
```

Das hat mehrere Vorteile:

- Datenlokalität.
    
- Effiziente Isolation.
    
- Kleinere Indizes.
    
- Einfachere Löschung.
    
- Gezielte Kompaktierung.
    
- Weniger unnötige Suchraumdurchquerung.
    

Die Anzahl physischer Shards sollte dabei unabhängig von den logischen Namespaces sein.

# **10. Das Dateisystem auf der SSD**

Ich würde nicht alle Daten in eine einzelne gigantische Datei schreiben.

Stattdessen ein kontrolliertes Dateilayout:

```
database/
│
├── MANIFEST
├── SUPERBLOCK
│
├── wal/
│   ├── wal-000001
│   ├── wal-000002
│   └── wal-000003
│
├── data/
│   ├── active/
│   ├── segments/
│   ├── locality_bundles/
│   └── blob_extents/
│
├── indexes/
│   ├── primary/
│   ├── vector/
│   ├── fulltext/
│   ├── graph/
│   └── metadata/
│
├── history/
│   ├── commits/
│   ├── branches/
│   └── manifests/
│
├── checkpoints/
│
└── temporary/
```

## **10.1 Superblock**

Der Superblock enthält Informationen über den zuletzt veröffentlichten Datenbankzustand:

```
struct Superblock {
    format_version: u32,

    generation: u64,

    last_durable_commit: CommitId,

    manifest_root: Hash256,

    wal_position: LogPosition,

    checksum: u32,
}
```

Ich würde zwei Superblock-Slots verwenden.

Eine neue Generation wird vollständig geschrieben und validiert, bevor sie als gültige Root-Generation veröffentlicht wird.

Damit kann ein Crash während der Manifest-Aktualisierung abgefangen werden.

## **10.2 Compaction-Strategie**

Klassische Leveled Compaction ist nicht für jede Datenklasse ideal.

CairnDB sollte mehrere Strategien besitzen:

|Datenklasse|Strategie|
|---|---|
|Kleine häufig veränderte Objekte|Leveled Compaction|
|Append-only Events|Tiered Compaction|
|Immutable Dokumente|Pack / Merge|
|Große Blobs|Kein unnötiges Rewrite|
|Embeddings|Vektor-spezifischer Indexbau|
|Historische Versionen|Retention-aware Compaction|

Besonders interessant wäre eine adaptive Compaction.

Sie berücksichtigt:

- Write Amplification.
    
- Read Amplification.
    
- Aktuelle IOPS.
    
- Datenalter.
    
- Zugriffsfrequenz.
    
- Lokalitätsqualität.
    
- Größe der Segmentgenerationen.
    
- Gepinnte Snapshots und Branches.
    

Das System sollte nicht blind kompaktieren, nur weil ein allgemeiner Level-Schwellenwert erreicht wurde.

Stattdessen priorisiert es Arbeiten, die für die tatsächlichen Lese- und Schreibmuster den größten Nutzen bringen.

Ein Beispiel:

Wenn ein Workspace stark aktiv ist, sollte er möglichst schnell kompakte, lokal organisierte Segmente bekommen.

Wenn ein Workspace seit Monaten unverändert ist, braucht er keine permanenten Hintergrund-Rewrites.

# **11. Sicherheit und Datenintegrität**

Gerade bei agentischen Systemen ist das keine optionale Zusatzfunktion.

Ich würde die folgenden Konzepte von Anfang an in die Engine integrieren.

### **Tenant-Isolation**

Jede Information gehört zu einem definierten Sicherheitsbereich.

```
Tenant
  └── Workspace
       └── Collection
            └── Object
                 └── Revision
```

Berechtigungen müssen bereits bei der Query-Planung durchgesetzt werden, nicht nur nachträglich bei der Rückgabe.

Das gilt insbesondere für ANN-Suchen. Ein unberechtigter Vektor darf nicht über Kandidatenlisten, Trefferzahlen oder Traversal-Verhalten Informationen über fremde Daten offenlegen.

### **Verschlüsselung**

- Verschlüsselung auf Extent- oder Segmentebene.
    
- Separate Schlüssel pro Tenant oder Sicherheitsbereich.
    
- Schlüsselrotation.
    
- Verschlüsselte WAL-Segmente.
    
- Verschlüsselung von Snapshots und Backups.
    
- Sichere Löschung durch Key-Erasure, soweit die Schlüssel- und Datenaufbewahrungsarchitektur dies ermöglicht.
    

### **Garbage Collection**

Die Engine muss erkennen, welche Daten noch erreichbar sind.

Als Roots dienen beispielsweise:

```
Current Branch Heads
Pinned Snapshots
Active Transactions
Historical Retention Points
Backup Manifests
```

Nur wenn ein Datensatz von keinem gültigen Root mehr erreichbar ist, darf er physisch entfernt werden.

Hier ist die Kombination aus immutable Objekten, Manifesten und Referenzzählung beziehungsweise Mark-and-Sweep besonders interessant.

# **12. Die API: Storage für Agenten statt Datenbankverwaltung**

Ich würde CairnDB eine API geben, die sowohl klassische Datenbankoperationen als auch agentenspezifische Funktionen versteht.

Das soll die Verwendung für Agenten-Frameworks extrem einfach machen.

## **12.1 Klassische Transaktionen**

Ein mögliches Rust-API-Design:

```
let db = Cairn::open("./agent-data")?;

let mut tx = db.begin_write().await?;

tx.events().append(event).await?;

tx.memory().put(
    "project.database",
    fact,
).await?;

tx.relationships().connect(
    source,
    target,
    Relation::Supports,
).await?;

tx.state().update(
    agent_id,
    new_state,
).await?;

tx.commit().await?;
```

Alle Änderungen sind Teil derselben Transaktion.

## **12.2 Agent Memory API**

```
let memory = db.memory();

memory.remember()
    .content("The project uses PostgreSQL")
    .scope(workspace_id)
    .kind(MemoryKind::Semantic)
    .source(document_id)
    .confidence(0.96)
    .commit()
    .await?;
```

## **12.3 Hybrid Search API**

```
let results = db.search()
    .query("Why did the deployment fail?")
    .scope(workspace_id)
    .semantic(true)
    .fulltext(true)
    .graph(true)
    .temporal(true)
    .limit(20)
    .execute()
    .await?;
```

## **12.4 Context API**

Das ist das zentrale Interface für LLMs:

```
let context = db.context()
    .for_agent(agent_id)
    .query("Continue the deployment investigation")
    .token_budget(12000)
    .include_history(true)
    .include_provenance(true)
    .include_related(true)
    .execute()
    .await?;
```

Die Antwort kann bereits in einer strukturierten Form zurückkommen:

```
struct ContextResult {
    items: Vec<ContextItem>,

    total_tokens: u32,

    snapshot: CommitId,

    sources: Vec<SourceReference>,

    conflicts: Vec<MemoryConflict>,
}
```

Der Agent muss somit nicht selbst mehrere Datenbanken abfragen und das Ergebnis aus unterschiedlichen Strukturen zusammensetzen.

## **12.5 Versionierungs-API**

```
db.history()
    .branch("agent-experiment")
    .create()
    .await?;

db.history()
    .diff("main", "agent-experiment")
    .await?;

db.history()
    .merge("agent-experiment", "main")
    .execute()
    .await?;
```

Damit bekommt man ein tatsächlich agententaugliches Storage-System.

# **13. Warum nicht einfach RocksDB, redb, Fjall oder LanceDB?**

Die Konkurrenz sollte man ernst nehmen.

Eine neue Datenbank muss einen echten technischen Mehrwert besitzen.

|System|Stärken|Was CairnDB zusätzlich bieten soll|
|---|---|---|
|RocksDB|Ausgereiftes LSM, WAL, Kompaktierung|LLM-native Datenmodelle, Context Engine, Versionierungs-DAG|
|redb|ACID, MVCC, Copy-on-Write-B+Trees|Spezialisierte Retrieval-Indizes und append-first Blob-Storage|
|Fjall|Rust, LSM, Keyspaces, Transaktionen, Value Separation|Einheitliches versionsübergreifendes LLM-Retrieval|
|FASTER|Cache-optimierter Index, Hybrid Log|Native Provenance, Token-Budget-Retrieval und Memory-Lifecycle|
|LanceDB|Rust, Vektoren, Metadaten, multimodale Daten, Versionierung|Agentenorientierte Transaktionen, Event- und Working-State-Modell|
|CairnDB|Eigene Architektur|Alle diese Konzepte als zusammenhängendes System|

Einige wichtige Details:

- **redb** bietet bereits ACID-Transaktionen, MVCC und Copy-on-Write-B+Trees. Es ist eine gute Referenz für Transaktionsdesign.
    
    Rust+1
    
- **Fjall** ist besonders interessant, weil es eine native Rust-Implementierung eines LSM-Ansatzes mit Cross-Keyspace-Atomizität und optionaler Value Separation bietet.
    
    Rust
    
- **FASTER** kombiniert einen cacheoptimierten Index mit einem persistenten Hybrid-Log. Diese Trennung zwischen Index und Datenlog ist eine wichtige Inspirationsquelle.
    
    microsoft.github.io
    
- **LanceDB** hat bereits viele AI-relevante Fähigkeiten, einschließlich Vektorsuche, Full-Text, multimodaler Daten und Versionierung. Es wäre eine besonders wichtige Vergleichsbasis für den Retrieval-Teil.
    
    Rust
    

Meine Schlussfolgerung:

Ich würde keine dieser Datenbanken einfach forken und um ein Agenten-API erweitern.

Ich würde ihre besten Ideen übernehmen, insbesondere:

1. RocksDBs LSM-Grundprinzipien.
    
2. redbs Transaktions- und Snapshot-Ideen.
    
3. Fjalls Value Separation.
    
4. FASTERS Hybrid-Log-Architektur.
    
5. LanceDBs multimodalen und vektororientierten Ansatz.
    
6. Git-artiges Branching und immutable Manifests.
    

Daraus entsteht eine neue Engine mit einem anderen primären Datenmodell.

**Wichtig:** Dass die Architektur diese Eigenschaften kombiniert, beweist noch keine Performance-Überlegenheit. Das ist eine Forschungs- und Implementierungshypothese. CairnDB muss seine Überlegenheit über realistische Agenten-Workloads nachweisen.

# **14. Was macht CairnDB wirklich besser? Die eigentlichen Innovationen**

Ich würde die Entwicklung auf acht technische Differenzierungsmerkmale konzentrieren.

# **01**

**Atomic Memory Transactions**

Ein Agent verändert Ereignisse, Wissen, Beziehungen, Arbeitszustand und Provenance in einem gemeinsamen Commit.

# **02**

**Locality-aware Retrieval Bundles**

Häufig gemeinsam gelesene Daten werden logisch und physisch auf Segment- und Page-Ebene gruppiert.

# **03**

**Native Memory Version Control**

Branches, Commits, Diffs, Snapshots und Merges sind Teil der Storage-Engine.

# **04**

**Snapshot-consistent Hybrid Search**

Vektor-, Volltext-, Graph- und Metadatenindizes werden auf denselben logischen Datenzustand bezogen.

# **05**

**Token-budget-aware Retrieval**

Nicht die größte Ergebnismenge, sondern der nützlichste Kontext innerhalb eines festen Budgets wird ermittelt.

# **06**

**Native Provenance und Temporal Storage**

Jede Information ist mit Quellen, Zeit, Version und Herkunft verknüpft.

# **07**

**Adaptive Multi-Tier Compaction**

Die Speicherorganisation orientiert sich am tatsächlichen Agenten-Zugriffsmuster.

# **08**

**Agent Lifecycle Storage**

Checkpoints, Events, Tool-Ergebnisse und langfristige Memory-Konsolidierung werden nativ unterstützt.

# **15. Entwicklungsstrategie: Wie ich CairnDB bauen würde**

Ich würde keinesfalls versuchen, alle Funktionen gleichzeitig zu entwickeln.

Das größte Risiko eines solchen Projekts ist, dass am Ende viele interessante Komponenten existieren, aber der Storage-Kern nicht zuverlässig, schnell oder korrekt ist.

Meine Priorität wäre folgende.

## **Phase 1 – Storage Core**

Priorität 1

Zuerst implementieren:

- Append-only WAL.
    
- Record Serialization.
    
- Memtable.
    
- Immutable Sorted Segments.
    
- Primary Key Index.
    
- Blob/Value Separation.
    
- MVCC.
    
- Atomare Transaktionen.
    
- Crash Recovery.
    
- Snapshot Reads.
    

Das Ergebnis muss eine eigenständige, zuverlässige Embedded-Datenbank sein.

Noch keine ANN-Suche und keine komplexe Context Engine.

**Abnahmekriterium:** Ein Crash während beliebiger Schreiboperationen darf keinen inkonsistenten committed Zustand erzeugen.

## **Phase 2 – Physical Storage Optimization**

Priorität 2

Implementieren:

- Locality-aware Placement.
    
- Retrieval Bundles.
    
- Adaptive Compaction.
    
- Block Cache.
    
- Page Layout.
    
- Compression.
    
- SIMD-optimierte Key- und Value-Verarbeitung.
    
- Hintergrund-Garbage-Collection.
    

Hier entscheidet sich, ob CairnDB tatsächlich einen besseren physischen Storage entwickelt.

## **Phase 3 – LLM Data Model**

Priorität 3

Implementieren:

- Agent State.
    
- Events.
    
- Episodic Memory.
    
- Semantic Memory.
    
- Artifacts.
    
- Provenance.
    
- Token-Metadaten.
    
- Temporal Records.
    
- Versionierte Embeddings.
    

Jetzt wird aus einer allgemeinen Storage-Engine eine LLM-native Datenbank.

## **Phase 4 – Retrieval Engine**

Priorität 4

Implementieren:

- Delta Vector Index.
    
- IVF/PQ.
    
- Optional HNSW.
    
- Full-Text Index.
    
- Graph Index.
    
- Metadata Filter.
    
- Hybrid Query Planner.
    
- Snapshot-consistent Index Visibility.
    

Den ANN-Teil würde ich zunächst über klar definierte interne Traits austauschbar gestalten. Die erste Version sollte mit exakter SIMD-Suche und einem kleinen ANN-Index starten, statt sofort eine vollständig eigene DiskANN-Implementierung zu entwickeln.

## **Phase 5 – Memory Version Control**

Priorität 5

Implementieren:

- Content Addressing.
    
- Merkle Manifests.
    
- Commit DAG.
    
- Branches.
    
- Diffs.
    
- Merges.
    
- Historische Abfragen.
    
- Snapshot Retention.
    

Diese Funktion könnte CairnDB von den meisten klassischen Storage-Engines stark differenzieren.

## **Phase 6 – Context Intelligence**

Priorität 6

Implementieren:

- Token Budget Optimizer.
    
- Kontext-Deduplizierung.
    
- Context Cache.
    
- Provenance-aware Ranking.
    
- Temporal Ranking.
    
- Context Reconstruction.
    
- Memory Consolidation APIs.
    

Erst hier ist das vollständige Produkt als Agenten-Storage sinnvoll.

# **16. Die Benchmarks: Wie wir beweisen, dass CairnDB besser ist**

Das halte ich für besonders wichtig.

Man sollte nicht nur YCSB- oder klassische Key-Value-Benchmarks ausführen.

Diese zeigen nicht, ob eine Datenbank für Agenten besser funktioniert.

Ich würde drei Benchmarkgruppen entwickeln.

## **16.1 Storage-Benchmarks**

|Benchmark|Messgröße|
|---|---|
|Sequential Write|Durchsatz|
|Random Write|Durchsatz und Latenz|
|Random Read|p50 / p95 / p99|
|Small Transactions|Commit-Latenz|
|Multi-Object Transactions|Commit-Latenz|
|Concurrent Readers|Read Throughput|
|Concurrent Writers|Write Throughput|
|Recovery|Recovery-Zeit|
|Compaction|Write Amplification|
|Blob Retrieval|I/O-Amplification|

Vergleich gegen RocksDB, redb und Fjall.

## **16.2 Agenten-Benchmarks**

Hier würde ich eigene realitätsnahe Workloads erzeugen.

Beispiel:

```
Agent Task:

1. Receive request
2. Read last 100 events
3. Read working state
4. Retrieve 20 related memories
5. Retrieve 5 semantic facts
6. Update working state
7. Append 10 events
8. Save checkpoint
9. Create 2 provenance links
10. Commit everything
```

Messen:

- Gesamtoperationen pro Sekunde.
    
- Commit-Latenz.
    
- Anzahl physischer Reads.
    
- Anzahl physischer Writes.
    
- CPU-Zeit.
    
- Speicherverbrauch.
    
- Speicherbedarf der Indizes.
    
- Wiederherstellungszeit.
    

Das bildet echte Agentenlast deutlich besser ab.

## **16.3 Retrieval-Benchmarks**

Hier würde ich nicht allein auf Suchlatenz schauen.

|Metrik|Bedeutung|
|---|---|
|Recall@K|Werden relevante Informationen gefunden?|
|nDCG@K|Sind sie sinnvoll sortiert?|
|Context Precision|Wie viel des gelieferten Kontextes ist tatsächlich nützlich?|
|Token Efficiency|Relevanz pro verwendetem Token|
|Retrieval Latency|Gesamte Suchzeit|
|Snapshot Correctness|Werden nur gültige Versionen geliefert?|
|Update Visibility|Wie schnell werden Änderungen suchbar?|

Zusätzlich würde ich realistische Aufgaben aus Coding-Agenten, Research-Agenten und lang laufenden autonomen Workflows verwenden.

### **Die wichtigste Vergleichsmessung**

Ich würde eine Metrik besonders hervorheben:

\[ E_{\text{context}} = \frac{\text{Nützliche Informationen}} {\text{I/O-Kosten}+\text{CPU-Kosten}+\text{Token-Kosten}} \]

Das ist keine standardisierte Benchmark-Metrik, sondern eine vorgeschlagene interne Optimierungsgröße.

Sie zwingt uns, die tatsächliche Aufgabe der Datenbank in den Mittelpunkt zu stellen.

Eine Engine, die 20 % schneller sucht, aber dreimal so viel irrelevanten Kontext liefert, ist für LLMs nicht zwangsläufig besser.

# **17. Meine endgültige Architekturentscheidung**

Wenn ich das Projekt als Principal Storage-Engine-Ingenieur verantworten würde, wäre das meine konkrete technische Auswahl:

|Komponente|Entscheidung|
|---|---|
|Storage-Paradigma|Hybrid Append-only + MVCC LSM|
|Primary Index|ART Memtable + Sorted Immutable Runs|
|Write Path|WAL + Group Commit|
|Transaction Model|Snapshot Isolation + Optimistic Concurrency|
|Large Objects|Content-addressed Blob Storage|
|Small Objects|Inline Record Storage|
|Physisches Layout|Locality-aware Extents und Retrieval Bundles|
|Versionierung|Commit DAG + Copy-on-Write Manifests|
|Vector Search|Delta + IVF/PQ, optional DiskANN|
|Text Search|Inverted Index + BM25|
|Graph Search|Native Adjacency Storage|
|Metadata Search|Typed Indexes + Roaring Bitmaps|
|Retrieval|Hybrid Ranking + Token Budget Optimization|
|Zeitmodell|Bitemporal Records|
|Parallelität|Sharding + Immutable Snapshots + Epoch Reclamation|
|Sprache|Rust|

## **Drei Architekturentscheidungen würde ich besonders verteidigen**

**1. Keine separate Wahrheit für Vector DB und Primary DB.**

Alle Informationen müssen aus einem gemeinsamen versionierten Datenmodell stammen. Sekundärindizes dürfen asynchron aufgebaut werden, aber niemals eine unkontrollierte Inkonsistenz erzeugen.

**2. Versionierung muss unterhalb der API stattfinden.**

Nicht der Agent oder das Framework soll sich um History, Snapshot-Erstellung, Datenkopien und Konflikterkennung kümmern müssen. Das muss die Engine erledigen.

**3. Der Kontext ist eine eigene Storage-Operation.**

Die Datenbank soll nicht nur Objekte speichern und wieder zurückgeben. Sie soll eine optimierte, nachvollziehbare Auswahl von Informationen für einen konkreten Agenten-Task bereitstellen.

# **18. Was ich als nächste technische Herausforderung untersuchen würde**

Es gibt noch eine besonders interessante Weiterentwicklung, die ich nicht in die erste Implementierung pressen würde:

## **Predictive Locality und Learned Retrieval Storage**

Die Engine könnte langfristig aus dem tatsächlichen Zugriffsmuster eines Agenten lernen.

Beispielsweise:

Ein Agent liest häufig:

```
Current Task
    ↓
Related Source File
    ↓
Previous Error
    ↓
Previous Solution
    ↓
Configuration
```

CairnDB könnte solche Zugriffsmuster erkennen und diese Objekte bei der nächsten Compaction bevorzugt gemeinsam platzieren.

Zusätzlich könnte das System erkennen, welche Objekte nach einer bestimmten Query mit hoher Wahrscheinlichkeit gebraucht werden.

Das würde ich als optionalen Optimierungsmechanismus entwickeln:

**Adaptive Access Locality Engine**

Wichtig ist dabei, nicht auf jedem Read einen aufwendigen ML-Prozess auszuführen. Das Lernen würde anhand von Hintergrundstatistiken und gesammelten Access Patterns stattfinden.

Langfristig könnte diese Komponente sogar die physische Datenorganisation dynamisch an unterschiedliche Agententypen anpassen.

# **Fazit**

Ich würde CairnDB nicht als Vector Database, nicht als Dokumentendatenbank und auch nicht als einfaches Key-Value-System positionieren.

Ich würde eine neue Kategorie entwickeln:

# **CairnDB**

## **Agent-native Versioned Storage Engine**

Eine transaktionale, locality-aware, MVCC-basierte Storage-Engine für langfristige, multimodale und strukturierte Agentenintelligenz.

**Core Properties**

**Low-latency**

Optimierter Schreib- und Lesepfad

**Versioned**

Native Branches und Snapshots

**Atomic**

Konsistente Mehrfachänderungen

**LLM-native**

Memory, Provenance und Token-aware Retrieval

**Locality-aware**

Gemeinsam genutzte Daten gemeinsam platzieren

**Hybrid Search**

Vektor, Text, Graph und Metadaten

**Mein wichtigster Rat:** Ich würde zuerst den Transaktionskern, den physischen Datenaufbau und die Versionierung perfektionieren. Anschließend die Retrieval-Engine daraufsetzen.

Denn eine wirklich gute LLM-Datenbank ist nicht einfach eine Datenbank mit zusätzlicher Vektorsuche.

Sie ist ein System, das Informationen zuverlässig aufnimmt, ihre Herkunft und Entwicklung kennt, sie effizient organisiert und für einen bestimmten Denkprozess optimal wieder bereitstellt.

Der nächste sinnvolle Entwicklungsschritt wäre ein vollständiges **CairnDB Technical Design Document (TDD)** mit binären Record-Formaten, WAL-Spezifikation, Commit-Protokoll, MVCC-Algorithmen, Segment-Layout, Rust-Traits und konkreten Modul-Schnittstellen. Daraus ließe sich unmittelbar ein implementierbarer Rust-Prototyp ableiten.