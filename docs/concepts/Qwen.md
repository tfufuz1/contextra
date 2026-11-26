# AxiomDB: Systemarchitektur-Spezifikation (v1.0)

## 1. High-Level Architektur & Schichtenmodell

AxiomDB folgt einem strikten Schichtenmodell, das Write- und Read-Pfade entkoppelt, um maximale Parallelität (Concurrency) und ACID-Durability zu gewährleisten.

### Schichtenmodell
1. **API & Protocol Layer:** gRPC / Arrow Flight (für Zero-Copy-Übertragung von Vektor-Batches).
2. **Cognitive Query Planner:** Token-bewusste Planung, RRF-Routing, Hierarchische Abstraktion.
3. **Transaction & Versioning Engine (The "Git" Layer):** MVCC, Merkle-DAG-Manager, Branch-Isolation.
4. **Unified Storage Engine:** Log-Structured Merge-Tree (LSM) mit spezialisierten Index-SSTables.
5. **OS & I/O Layer:** `io_uring` Direct I/O, Lock-free Buffer Pool, Epoch-Based Reclamation.

### Datenfluss (Write-Path)
1. Agent sendet Insert/Update (z. B. neuer Wissens-Chunk mit Vektor, Graph-Kanten, Text).
2. **WAL (Write-Ahead-Log):** Asynchrones, batched Append via `io_uring`. Garantiert Durability.
3. **MemTable (In-Memory):** Daten werden in eine sortierte, nebenläufige Skip-List (für Text/Graph) und einen Roaring-Bitmap-Akkumulator (für Vektoren) geschrieben.
4. **Flush & Compaction:** Im Hintergrund werden MemTables in immutable SSTables auf Disk geschrieben. Dabei werden die Merkle-Hashes der Pages berechnet und der Merkle-DAG aktualisiert.

### Datenfluss (Read-Path)
1. Planner analysiert Query (z. B. "Finde Entitäten im Subgraph von *Agent_X*, deren Vektor-Ähnlichkeit > 0.8 ist und die das Wort *Fehler* enthalten").
2. **Cache-Lookup:** Page-Cache (via `mmap` und `Arc`-Referenzen).
3. **Index-Intersection:** Graph-Bitmap $\cap$ Inverted-Index-Bitmap $\rightarrow$ Resultat als Filter-Maske für die Vektorsuche (HNSW/DiskANN).
4. **Scoring & RRF:** BM25-Score und Cosine-Similarity werden via Reciprocal Rank Fusion vereint.
5. **Token-Aware Projection:** Ergebnisse werden dedupliziert und exakt bis zum Token-Limit des LLM-Contexts assembliert.

---

## 2. Physisches Storage- & Index-Design

Das größte Problem hybrider Systeme ist Daten-Duplikation. AxiomDB löst dies durch ein **Entity-Component-Storage-Modell** auf Block-Ebene (16 KB Pages).

### Physisches Layout
* **Base Entity Store (LSM-Tree):** Speichert die Rohdaten (Text, Vektor-Bytes, Graph-Properties). Jeder Eintrag ist eine `EntityID` (UUID).
* **Unified Indexing (Pointer-basiert):**
  * **Inverted Index:** Roaring Bitmaps. `Token -> [EntityID_1, EntityID_2...]`.
  * **Property Graph:** Adjazenlisten werden als delta-kodierte Arrays in dedizierten Edge-SSTables gespeichert. `SourceID -> [TargetID_1, TargetID_2...]`.
  * **Vector Index:** Wir nutzen ein natives **DiskANN (Vamana)**-Graph-Layout. Die Vektoren selbst liegen im Base Store, der DiskANN-Graph (Nachbarschafts-Pointer) liegt in einer separaten SSTable.

### Hybride Query-Logik (Beispiel: Graph-gefilterte Vektorsuche)
Anstatt alle Vektoren zu scannen, nutzt AxiomDB **Bit-Sliced Indexing**:
1. Der Graph-Traversal liefert eine Roaring Bitmap der erreichbaren `EntityIDs`.
2. Der Inverted Index liefert eine Bitmap der Text-Treffer.
3. Beide Bitmaps werden per `AND` verknüpft.
4. Der Vektor-Index (DiskANN) nutzt diese Bitmap als **Hard-Constraint** während der Graph-Traversal-Suche. Es werden nur Knoten evaluiert, die in der Bitmap gesetzt sind.
5. Das Ergebnis wird via **RRF (Reciprocal Rank Fusion)** mit dem lexikalischen Score kombiniert.

---

## 3. Versionierungs- & Concurrency-Engine (Git-like MVCC)

Um Agenten in Sandboxes arbeiten zu lassen, implementieren wir **Content-Addressed Storage (CAS)** kombiniert mit MVCC.

### Merkle-DAG & Zero-Copy Branching
* Jede 16 KB Page auf der Disk ist immutable und wird über ihren SHA-256 / BLAKE3 Hash adressiert.
* Der Zustand der Datenbank zu jedem Zeitpunkt $T$ ist ein **Root-Hash**, der auf einen B-Tree zeigt, der wiederum auf die Hashes der aktiven Pages zeigt.
* **Branching:** Ein Branch ist *nur* ein Pointer auf einen Root-Hash. Zero-Copy. Das Anlegen eines Sandbox-Branches für einen Agenten kostet $O(1)$ Zeit und 0 MB Speicher.

### MVCC & Concurrency in Rust
* Jeder Record hat Metadaten: `(EntityID, Version, TxnID, IsDeleted)`.
* **Lock-free Reads:** Lese-Transaktionen nutzen `crossbeam-epoch`. Sie "pinnen" den aktuellen Merkle-Root. Solange die Transaktion lebt, können alte Pages vom Garbage Collector (Compaction) nicht gelöscht werden.
* **Wait-free Writes:** Der Buffer Pool nutzt `parking_lot::RwLock` nur für den Meta-Status, Daten-Manipulationen im RAM erfolgen über atomare Operationen (`AtomicPtr`).

### Merge-Logik für Agenten-Sandboxes
Wenn ein Agent seinen Branch in `main` mergen will:
1. **Text/Properties:** Standard 3-Wege-Merge. Bei Konflikten wird ein `ConflictState::Pending` Flag gesetzt.
2. **Vektoren/Graphen:** Da Vektoren und Kanten schwer zu "mergen" sind, nutzen wir **CRDTs (Conflict-free Replicated Data Types)**. Der Graph ist append-only mit Tombstones. Vektoren werden bei Konflikten als *neue Version* (Historie) angelegt, das Time-Travel-System entscheidet über die Gültigkeit.

---

## 4. LLM- & Agenten-Spezialfunktionen (Anti-Halluzination & Effizienz)

Hier hebt sich AxiomDB von klassischen DBs ab. Wir bauen Schutzmechanismen direkt in die Engine.

### A. Kryptografisches Provenance-Tracking
Jeder Chunk und jede Graph-Kante besitzt eine `ProvenanceComponent`:
```rust
struct Provenance {
    source_hash: Blake3Hash,      // Hash des Original-Dokuments/URL
    ingestion_txn: TxnID,         // Welche Transaktion hat dies eingefügt?
    confidence: f32,              // Initiales Embedding-Confidence-Score
    temporal_decay: f32,          // Faktor für Time-Travel-Gewichtung
}
```
LLMs erhalten im RAG-Prompt nicht nur den Text, sondern auch den `source_hash`. Das LLM kann so gezwungen werden, Zitate exakt dieser Hashes zu verwenden.

### B. Token-Aware Query Planner & "Lost-in-the-Middle" Prävention
Der Planner akzeptiert ein `max_context_tokens: u32` Argument.
1. **Hierarchisches Fetching:** Die Engine speichert Chunks in 3 Ebenen (Abstract, Summary, Raw). Der Planner fetcht zuerst nur Summaries, berechnet die Relevanz, und lädt nur die Top-K Raw-Chunks nach.
2. **Positionierung:** Ergebnisse werden nicht einfach appendiert. Der Planner sortiert sie nach *Informationsdichte* (BM25 * Vektor-Score). Die absolut besten Treffer werden an den **Anfang und das Ende** des Result-Sets platziert (bekannte LLM-Schwäche: "Lost-in-the-Middle" wird durch U-Shape-Verteilung kompensiert).
3. **MMR Deduplizierung:** Maximal Marginal Relevance wird nativ im Storage-Engine-Iterator berechnet, um redundante Fakten aus verschiedenen Quellen vor dem Senden an das LLM zu streichen.

### C. Temporales Indexing & Time-Decay
Fakten veralten. AxiomDB nutzt eine native Decay-Funktion im Scoring:
`Final_Score = Base_Score * e^(-lambda * TimeDelta)`
Agenten können Queries mit `as_of: Timestamp` ausführen (Time-Travel), um den exakigen Wissensstand der Datenbank zu einem vergangenen Datum abzufragen.

### D. Automatische Widerspruchs-Erkennung (Contradiction Detection)
Beim `INSERT` oder `UPDATE` prüft ein asynchroner Hintergrund-Worker (ein "Critic-Agent" oder ein leichtgewichtiges lokales Modell):
* Gibt es im Graphen oder Text-Index semantisch ähnliche, aber logisch invertierte Aussagen? (z. B. "CEO ist Müller" vs. "CEO ist Meier").
* Wenn ja, wird der neue Eintrag als `Status::Contested` markiert. Das LLM/RAG-System erhält bei der Abfrage beide Versionen mit ihren Provenance-Hashes und muss den Konflikt auflösen (Active Learning Loop).

---

## 5. Konkreter Rust-Technologie-Stack & Trade-offs

### Empfohlener Rust-Stack (Stand 2026)
* **Async Runtime & I/O:** `tokio` kombiniert mit `tokio-uring` (für Linux `io_uring` Direct I/O, um Kernel-Bypass und Zero-Copy vom NVMe-Drive in den User-Space zu ermöglichen).
* **Zero-Copy Serialization:** `rkyv` für interne Page-Strukturen (extrem schnell, keine Deserialisierung nötig, direkt `mmap`-fähig). `apache-arrow` für den Transport von Vektor-Batches über gRPC.
* **Concurrency & Memory:** `crossbeam` (Epoch-Based Reclamation für MVCC), `parking_lot` (schnelle Locks), `mimalloc` (Allocator).
* **Indizes:** Native Rust-Implementierung von Roaring Bitmaps (`roaring` crate). Für DiskANN/HNSW nutzen wir FFI-Binding zu `usearch` oder implementieren einen quantisierten HNSW-Graphen nativ in Rust mit SIMD-Intrinsics (`std::simd`).
* **Hashing:** `blake3` (schneller als SHA-256, essentiell für Merkle-DAG Performance).

### Architektur-Trade-offs & Mitigation

| Zielkonflikt | Problem | AxiomDB Mitigation |
| :--- | :--- | :--- |
| **Write-Amplification vs. Merkle-DAG** | Das ständige Neuberechnen von Merkle-Hashes bei LSM-Compaction erzeugt CPU-Overhead. | Hashes werden *inkrementell* während des Flushes der MemTable in SSTables berechnet. Der Merkle-Tree wird als B-Tree *über* den SSTables gespeichert, nicht über einzelnen Rows. |
| **RAM-Verbrauch vs. Vektor-Latenz** | HNSW-Graphen im RAM sind schnell, aber bei Milliarden von Vektoren unbezahlbar. | Wir nutzen **DiskANN** mit `mmap`. Nur der oberste Layer des Vektor-Graphen (Routing-Layer) wird im RAM gehalten (`Arc<PageCache>`), der Rest liegt auf NVMe und wird via `io_uring` mit Prio-Queues geladen. |
| **Graph-Merge-Konflikte** | Wenn zwei Agenten unabhängig Kanten im Sandbox-Branch hinzufügen, ist ein 3-Wege-Merge auf Graphen extrem komplex. | Wir erzwingen **CRDT-Adjazenlisten**. Kanten haben UUIDs und Timestamps. Beim Merge werden Kanten unioniert. Löschungen propagieren via Tombstones. Der Graph "wächst" kontrolliert, alte Strukturen werden via Time-Travel ausgeblendet. |
| **Token-Planner Overhead** | Das Zählen von Tokens und MMR-Berechnungen in der DB verlangsamt die Query. | Der Tokenizer (z.B. BPE) ist als hochoptimierte Rust-Lookup-Table (Trie) direkt in der Engine implementiert. MMR wird als Stream-Operator ausgeführt, der early-terminated, sobald das Token-Budget erreicht ist. |

---

### Fazit für das Engineering-Team
AxiomDB ist nicht nur eine Datenbank; es ist ein **Operating System für KI-Wissen**. 
Indem wir die physikalische Speicherung (LSM), die logische Versionierung (Merkle-DAG) und die semantische Abfrage (Hybrid-Planner) in Rust mit maximaler Speichersicherheit und `io_uring`-Performance vereinen, eliminieren wir die Fragmentierung des KI-Stacks. 

Agenten können nun endlich wie Menschen arbeiten: Sie lesen, sie schreiben, sie machen Fehler in einem Notizblock (Sandbox-Branch), und sie korrigieren sich, ohne das Langzeitgedächtnis (Main-Branch) zu korrumpieren – vollständig nachvollziehbar durch kryptografische Provenance.