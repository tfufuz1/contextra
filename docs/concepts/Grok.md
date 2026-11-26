
# Grok

**Hybride Multi-Model-Datenbank-Engine für KI-Agenten und RAG**  
*(Arbeitstitel: **AetherDB** – Unified Agent Memory Engine)*

Ziel: Eine von Grund auf in Rust geschriebene, ACID-fähige Speicher-Engine, die Dense Vectors, Property-Graphs und Volltext nativ in **einer** physischen und logischen Struktur vereint. Keine externen Vektordatenbanken, Graph-Stores oder Such-Engines. Alle hybriden Queries (Graph-gefilterte Vektorsuche + BM25 + Reciprocal Rank Fusion) laufen in einer einzigen Transaktion ohne Daten-Duplikation. Git-ähnliche Versionierung und explizite Schutzmechanismen gegen typische LLM-/Agenten-Schwachstellen sind First-Class-Features.

---

### 1. High-Level Architektur & Schichtenmodell

**Schichten (von unten nach oben):**

1. **Storage Layer (Physisch)**  
   - Unified Page/Block-Manager mit `io_uring` + `tokio`.  
   - Segmentierte LSM-Tree-ähnliche Struktur (Sorted String Tables + MemTables) kombiniert mit Copy-on-Write B+-Tree-Varianten für graphische und vektorielle Nebenstrukturen.  
   - Write-Ahead-Log (WAL) mit Gruppen-Commit und periodischen Snapshots.

2. **Index & Multi-Model Layer**  
   - Ein gemeinsamer **Object Store** (jede Entity hat eine stabile `ObjectId`).  
   - Drei native Index-Familien, die denselben Object-Store referenzieren:  
     - Dense-Vector-Index (HNSW / DiskANN-Variante)  
     - Property-Graph (Adjacency + Property-Maps)  
     - Inverted Index (Token → Postings mit BM25-Statistiken)

3. **Versioning & Concurrency Layer**  
   - MVCC + Merkle-DAG (analog Git).  
   - Jeder Commit erzeugt einen neuen Root-Hash.  
   - Branches sind cheap (Zero-Copy-Pointer auf gemeinsame Blätter).

4. **Query & Planner Layer**  
   - Hybrid Query Engine (Vektor + Graph-Traversal + Lexikalisch).  
   - Token-aware Cost-Based Planner mit Budget- und Positionierungs-Optimierung.

5. **Agent Runtime Layer**  
   - Ephemeral Session-Branches, Provenance-Tracking, Konflikt-Detektor, Time-Decay.

**Write-Path (vereinfacht):**  
Client → Transaction → MemTable(s) + WAL → Hintergrund-Flush → Segment-Merge (mit Merkle-Update) → Commit (neuer Root-Hash).

**Read-Path:**  
Query → Snapshot (Root-Hash oder Branch) → Index-Lookups (gemeinsam) → Hybrid Ranking / Graph-Expansion → Token-Budget-Filter → Ergebnis mit Provenance.

---

### 2. Physisches Storage- & Index-Design

**Kernidee:** Jedes persistente Objekt (Chunk, Entity, Edge, Embedding) trägt eine `ObjectId` (128-bit) und lebt in einem einheitlichen **Object Store**. Indizes speichern nur Referenzen + Metadaten.

**Object Layout (rkyv-serialisiert, Zero-Copy-fähig):**
```rust
struct Object {
    id: ObjectId,
    version: u64,               // MVCC-Version
    kind: ObjectKind,           // Chunk | Entity | Edge | ...
    payload: Payload,           // Text, Properties, Embedding, ...
    provenance: Provenance,     // Quelle, Confidence, Timestamp, Agent-Id
    merkle_hash: Hash,          // Content-Hash
}
```

**Vektor-Index:**
- Primär: DiskANN-ähnliche Graph-Struktur auf SSD (mit RAM-Cache der oberen Ebenen).  
- Alternative/Hybrid: HNSW für heiße Daten im RAM.  
- Jeder Vector-Node speichert `ObjectId` + quantisierte Embedding (PQ/OPQ) + optional Full-Precision-Fallback.  
- Graph-Kanten des Vector-Index und des Property-Graphs können geteilt werden (gemeinsame Nachbarschaftsrelationen).

**Property-Graph:**
- Adjacency-Lists als sortierte Arrays von `(EdgeId, TargetObjectId, EdgeType, Properties)`.  
- Edge-Properties und Node-Properties liegen im Object Store.  
- Multi-Hop-Queries nutzen denselben Snapshot wie die Vektorsuche → echte Hybridität ohne Join über Netzwerk.

**Volltext (Inverted Index):**
- Token → Posting-List (ObjectId, TF, Field, Positionen).  
- BM25-Statistiken (doc-length, avgdl, IDF) pro Segment.  
- Positionen ermöglichen Phrase- und Proximity-Scoring.

**Hybride Query-Ausführung (Beispiel):**
```
MATCH (e:Entity)-[:RELATED*1..3]->(c:Chunk)
WHERE vector_similarity(c.embedding, $query_vec) > 0.72
  AND bm25(c.text, $query) > 4.0
ORDER BY rrf(vector_score, bm25_score, graph_path_score)
LIMIT 20 WITH TOKEN_BUDGET 3500
```
Der Planner materialisiert zuerst die Graph-Filter (sehr selektiv), dann Vektor + BM25 nur auf dem Kandidaten-Set und fusioniert mit Reciprocal Rank Fusion. Alles innerhalb eines Snapshot.

**Layout auf Disk:**
- Segments (immutable) enthalten Object-Store-Blöcke + Index-Blöcke (Vector-Graph, Adjacency, Inverted).  
- MemTables (pro Branch/Hot-Path) in RAM.  
- Buffer-Pool (ARC oder Clock-Pro) für heiße Pages.

---

### 3. Versionierungs- & Concurrency-Engine

**Modell:** MVCC + Merkle-DAG (Git-inspiriert).

- Jeder Commit erzeugt einen neuen **Commit-Object** mit:
  - Parent-Hash(es)
  - Root-Hash des Object-Stores + aller Indizes
  - Timestamp, Agent-Id, Message, Provenance-Summary
- Branches sind Named-Pointer auf Commit-Hashes.  
- `main` ist nur ein spezieller Branch-Name.

**Zero-Copy-Branching:**  
Ein neuer Branch kopiert nur den Root-Pointer. Alle unveränderten Blätter und Index-Segmente werden geteilt (Copy-on-Write bei Schreibzugriff).

**Time-Travel / Point-in-Time:**  
Jede Query kann einen Commit-Hash, Branch-Name oder Timestamp angeben. Der Snapshot-Resolver lädt den entsprechenden Root und alle darunterliegenden immutable Segments.

**Merge:**  
- Content-Addressable → strukturelle Gleichheit wird erkannt.  
- Graph- und Vector-Merge: Konflikt, wenn dieselbe `ObjectId` unterschiedliche Embeddings oder widersprüchliche Properties hat.  
- Automatische Konflikt-Erkennung + manuelle/regelbasierte Resolution (siehe Abschnitt 4).  
- Fast-Forward und 3-Way-Merge analog Git.

**Concurrency in Rust:**
- Lock-free Read-Path über immutable Snapshots (Arc + generational indices).  
- Schreibende Transaktionen halten kurze Write-Locks nur auf MemTables und den aktuellen Branch-Pointer.  
- `crossbeam` / `parking_lot` für feingranulare Locks, `tokio` + `io_uring` für asynchrones I/O.  
- Ownership-Modell verhindert Data-Races; `rkyv` oder `arrow` für Zero-Copy-Deserialisierung aus dem Buffer-Pool.

**Durability:**
- WAL mit `fsync`-Gruppen-Commit.  
- Periodische Snapshots (Checkpoint) des Root-Hashes.  
- Crash-Recovery: WAL-Replay bis zum letzten gültigen Root.

---

### 4. LLM- & Agenten-Spezialfunktionen

**4.1 Kryptografisches Provenance-Tracking**
- Jeder Chunk und jede Kante trägt eine `Provenance`-Struktur:
  - Source-URI / Document-Hash
  - Extraction-Agent + Model-Version
  - Confidence-Score (kalibriert)
  - Merkle-Pfad bis zum Commit
- Bei Retrieval wird die Provenance immer mitgeliefert.  
- Halluzinations-Schutz: Agenten-Antworten können nur Fakten zitieren, die eine gültige Provenance-Kette besitzen. Fehlt sie → automatische Ablehnung oder „Unknown“.

**4.2 Token-aware Query Planner**
- Der Planner erhält ein Token-Budget und eine Präferenz (Zusammenfassung vs. Rohdaten).  
- Strategien:
  - Hierarchische Abstraktion: zuerst Summary-Nodes, bei Bedarf Drill-down.  
  - Redundanz-Eliminierung (Semantische Deduplizierung über Embedding-Ähnlichkeit).  
  - Positionierungs-Optimierung: wichtige Fakten an den Anfang/Ende des Contexts (gegen Lost-in-the-Middle).  
  - Dynamische Chunking-Grenzen je nach verbleibendem Budget.

**4.3 Ephemeral Agent-Branches & Fehlertoleranz**
- Jede Agenten-Session startet in einem privaten Branch (`agent/<session-id>`).  
- Tool-Calls und Zwischen-Ergebnisse schreiben nur in diesen Branch.  
- Erst nach erfolgreicher Validierung (Confidence-Threshold + menschliche/LLM-Review) wird ein Merge nach `main` durchgeführt.  
- Fehlgeschlagene oder halluzinierte Aktionen werden einfach verworfen (Branch löschen).  
- Nicht-Determinismus wird isoliert.

**4.4 Temporales Indexing & Konflikt-Erkennung**
- Jede Tatsache hat eine Gültigkeitsperiode und einen Time-Decay-Faktor.  
- Beim Insert widersprüchlicher Fakten (gleiche Entity + Property, aber unterschiedlicher Wert + überlappender Zeitraum) wird ein Konflikt-Objekt erzeugt.  
- Der Merge-Prozess und der Query-Planner können Konflikte explizit surfacing (z. B. „zwei widersprüchliche Aussagen mit Confidence 0.81 und 0.73“).

---

### 5. Konkreter Rust-Technologie-Stack & Architektur-Trade-offs

**Empfohlener Stack:**
- Runtime: `tokio` + `io_uring` (via `tokio-uring` oder `glommio`)
- Serialisierung: `rkyv` (Zero-Copy) + optional Apache Arrow für analytische Pfade
- Concurrency: `crossbeam`, `parking_lot`, `arc-swap`
- Hashing / Merkle: `blake3` oder `sha2` + custom Merkle-Tree
- Vektor-Index: eigene DiskANN/HNSW-Implementierung (oder `usearch` / `hnsw` als Ausgangspunkt, stark angepasst)
- WAL & Storage: eigene LSM-ähnliche Engine oder stark erweiterte `sled`/`redb`-Ideen
- Query-Language: eigene DSL oder Erweiterung von GraphQL / Cypher-ähnlich + Vektor-Operatoren
- Monitoring: `tracing` + `metrics`

**Wesentliche Trade-offs:**

| Zielkonflikt                        | Entscheidung / Begründung                                                                 | Preis |
|-------------------------------------|-------------------------------------------------------------------------------------------|-------|
| Write-Amplification (LSM)           | Akzeptiert für Write-Heavy Agenten-Workloads; kompensiert durch Segment-Merge-Strategien und CoW | Höherer Disk-I/O bei Merges |
| RAM vs. Latenz                      | Heiße Vektor-Graphen und Postings im Buffer-Pool; kalte Segmente auf SSD                  | Höherer RAM-Bedarf bei großen Knowledge-Bases |
| Merge-Komplexität (Vektor + Graph)  | Content-Addressable + explizite Konflikterkennung statt „last-write-wins“                 | Merge kann manuelle Resolution brauchen |
| Zero-Copy vs. Flexibilität          | `rkyv` für den heißen Pfad; Fallback auf owned-Strukturen bei komplexen Transformationen | Etwas mehr Code-Komplexität |
| Starke Isolation vs. Throughput     | Ephemeral Branches + Snapshot-Isolation                                                   | Branch-Management-Overhead |
| Quantisierung vs. Recall            | Adaptive Quantisierung (PQ für Suche, Full-Precision bei Bedarf)                          | Leicht höherer Speicher für Full-Precision-Cache |

**Nicht-Ziele (bewusst):**
- Kein verteiltes Multi-Node in v1 (Single-Node mit späterem Sharding über ObjectId-Hash).  
- Keine vollständige SQL-Kompatibilität.  
- Kein generisches Key-Value-Store-Interface als primäre API (Agenten-API und hybride Query-API stehen im Vordergrund).

---

**Zusammenfassung der Kerninnovation**

AetherDB löst das fundamentale Fragmentierungsproblem heutiger RAG-/Agenten-Stacks, indem Vektor, Graph und Text in einer physischen Speicher-Engine mit gemeinsamer Versionierung und gemeinsamen Snapshots leben. Die Git-ähnliche Branching-Semantik plus die eingebauten Provenance-, Token-Budget- und Konflikt-Mechanismen machen die Engine von vornherein robust gegen die typischen Fehlerquellen von LLM-basierten Agenten.

Die Architektur ist speicher- und concurrency-sicher durch Rusts Ownership-Modell, durable durch WAL + Snapshots und query-fähig für echte hybride, token-bewusste Retrieval-Pfade innerhalb einer Transaktion.

Dies ist die vollständige konzeptionelle Spezifikation. Bei Bedarf kann daraus als Nächstes ein detaillierteres Component-Interface, ein PoC-Crate-Layout oder konkrete Algorithmus-Skizzen (z. B. der Token-aware Planner oder der Hybrid-Merger) abgeleitet werden.

---
