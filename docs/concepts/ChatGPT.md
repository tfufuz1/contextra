
# ChatGPT

Die entscheidende Architekturentscheidung lautet: **Nicht drei Datenbanken in einer gemeinsamen API verstecken, sondern einen einzigen transaktionalen Storage- und Versionierungskern entwickeln, der unterschiedliche Indexstrukturen nativ verwaltet.**

Ich empfehle dafür eine hybride Architektur aus LSM-Storage, unveränderlichen Index-Segmenten, MVCC und einem Git-artigen Merkle-Commit-DAG. Dadurch können Vektor-, Graph- und Volltextsuche denselben Datenbestand, dieselbe Version und dieselbe Transaktionsgrenze verwenden.

Ein wichtiger technischer Punkt vorweg: Die Engine kann Daten logisch ohne redundante Dokumentkopien speichern. Physische Indexstrukturen müssen jedoch abgeleitete Informationen wie Token-Postings, ANN-Nachbarschaften und Quantisierungscodes enthalten. Diese sind keine unabhängigen Kopien des Wissensinhalts, sondern notwendige Suchrepräsentationen.

# **Technische Produktspezifikation: Unified Cognitive Database Engine**

# Unified Cognitive Database Engine

## Technische Architektur- und Produktspezifikation

**Produktkategorie:** Native Multi-Model Storage Engine für KI-Agenten, RAG-Systeme und persistente Wissensgraphen  
**Arbeitstitel:** UCD – Unified Cognitive Database  
**Implementierungssprache:** Rust  
**Architektur:** LSM-basierter Storage-Kern, MVCC, Immutable Segments und Merkle-DAG  
**Primäres Deployment-Ziel:** Single-Node NVMe Storage mit Erweiterbarkeit für verteilte Replikation  
**Dokumentstatus:** Architekturvorschlag / Technical Design Specification  
**Stand:** 26. November 2026

---

# 0. Produktdefinition und Architekturziele

## 0.1 Produktvision

UCD ist eine hybride Datenbank-Engine, die strukturierte Daten, semantische Vektoren, Graphbeziehungen, Volltextinformationen und deren historische Provenienz in einem gemeinsamen Speichersystem vereint.

Das System soll als persistentes Gedächtnis autonomer KI-Agenten fungieren.

Ein Agent soll beispielsweise folgende Operationen in einer einzigen logischen Query ausführen können:

1. Einen Ausgangsknoten in einem Wissensgraphen identifizieren.
    
2. Über mehrere Beziehungsebenen relevante Entitäten und Dokumente finden.
    
3. Die gefundenen Dokumente semantisch mittels Vektorsuche bewerten.
    
4. Eine lexikalische Suche über die Originaltexte ausführen.
    
5. Die Ergebnisse über Reciprocal Rank Fusion zusammenführen.
    
6. Nur Aussagen mit ausreichenden Herkunftsnachweisen berücksichtigen.
    
7. Das Resultat auf ein vorgegebenes Token-Budget reduzieren.
    
8. Alle Ergebnisse auf eine konkrete Datenbankversion zurückführen.
    

Das Ergebnis ist eine gemeinsame Wissensbasis statt mehrerer voneinander unabhängiger Datenbanksysteme.

## 0.2 Primäre Architekturziele

|Bereich|Ziel|
|---|---|
|Konsistenz|Atomare Änderungen über alle Datenmodelle|
|Storage|Gemeinsamer physischer Storage-Kern|
|Suche|Graph-, Vektor- und Textsuche in einem Snapshot|
|Versionierung|Git-artige Branches, Commits, Merges und Diffs|
|Concurrency|Parallele Leser, konkurrierende Schreiber|
|Durability|WAL, Recovery und nachvollziehbare Commit-Historie|
|Provenance|Kryptografisch prüfbare Herkunft von Informationen|
|Agenten-Isolation|Ephemere Branches pro Session|
|Retrieval|Token-aware und provenance-aware Query Planning|
|Temporalität|Bitemporale Datenhaltung und Konflikterkennung|

## 0.3 Architekturprinzipien

Die Engine basiert auf sechs verbindlichen Designprinzipien:

**Prinzip 1 – Eine Source of Truth**

Jeder logische Datensatz existiert genau einmal als kanonisches Objekt. Vektor-, Graph- und Text-Indizes verweisen auf seine stabile Objekt-ID und Version.

**Prinzip 2 – Gemeinsamer Commit-Kern**

Keine Indexkomponente darf unabhängig einen für Benutzer sichtbaren Commit veröffentlichen.

**Prinzip 3 – Immutable Data First**

Persistierte Commit- und Segmentdaten werden grundsätzlich nicht nachträglich verändert. Änderungen erzeugen neue Versionen.

**Prinzip 4 – Snapshot-basierte Reads**

Jede Query liest einen konsistenten Snapshot des gesamten Datenmodells.

**Prinzip 5 – Derived Indexes sind rekonstruierbar**

Die kanonischen Daten sind maßgeblich. Suchindizes können aus diesen Daten und dem Commit-Verlauf wiederhergestellt werden.

**Prinzip 6 – Agentenwissen ist zunächst nicht vertrauenswürdig**

Neue Fakten, Zusammenfassungen und Beziehungen erhalten Provenance- und Vertrauensmetadaten. Eine Agentenbehauptung wird nicht automatisch zu globalem Wissen.

## 0.4 Technischer Umfang der ersten Produktgeneration

Die Referenzarchitektur geht zunächst von einer einzelnen autoritativen Datenbankinstanz aus.

Sie unterstützt:

- Multi-Tenant-Datenhaltung
    
- parallele Lese- und Schreiboperationen
    
- mehrere Branches
    
- viele gleichzeitige Agenten-Sessions
    
- persistente Commit-Historien
    
- Hintergrundkompaktierung
    
- lokale NVMe- und SSD-Speicherung
    

Ein verteiltes Cluster mit mehreren autoritativen Schreibknoten ist zunächst nicht Teil des Kernumfangs. Die Architektur soll jedoch die spätere Replikation unveränderlicher Datenobjekte und Commit-DAGs ermöglichen.

---

# 1. High-Level Architektur und Schichtenmodell

## 1.1 Systemübersicht

Die Engine wird in logisch getrennte Schichten aufgeteilt. Die zentrale Besonderheit ist, dass sämtliche Schichten dieselbe Transaktions- und Versionierungsinfrastruktur verwenden.

```
                    AI AGENTS / APPLICATIONS
                              |
                   Rust SDK / HTTP / gRPC
                              |
                 API & AUTHORIZATION LAYER
                              |
                  QUERY COMPILER / PARSER
                              |
                   TOKEN-AWARE PLANNER
                              |
               TRANSACTION & MVCC MANAGER
                              |
               SNAPSHOT / COMMIT RESOLVER
                              |
        +---------------------+---------------------+
        |                     |                     |
   VECTOR ENGINE         GRAPH ENGINE          TEXT ENGINE
        |                     |                     |
      HNSW               Adjacency Index        Inverted Index
   DiskANN-ready          CSR / Edge Index       BM25 / FST
        |                     |                     |
        +---------------------+---------------------+
                              |
                 UNIFIED STORAGE KERNEL
                              |
           +------------------+------------------+
           |                  |                  |
       Object Store        LSM Storage        Buffer Pool
           |                  |                  |
           +------------------+------------------+
                              |
                  WAL / RECOVERY ENGINE
                              |
                 IMMUTABLE SEGMENT STORAGE
                              |
                    NVMe / SSD / DISK
                              |
                   MERKLE COMMIT DAG
```

Der Merkle-Commit-DAG ist dabei kein separater Suchindex, sondern die übergeordnete Versionierungs- und Integritätsschicht.

## 1.2 Komponenten im Detail

### A. API Layer

Verantwortlich für die Kommunikation mit Anwendungen und Agenten.

Schnittstellen:

- Native Rust API
    
- gRPC
    
- HTTP API
    
- SQL-artige hybride Query-Schnittstelle
    
- Administrations-API für Branches, Commits und Snapshots
    

Die API stellt ein einheitliches Datenmodell bereit.

Beispiel:

```
pub struct CognitiveRecord {
    pub id: ObjectId,
    pub tenant_id: TenantId,
    pub content: ContentRef,
    pub properties: PropertyMap,
    pub embeddings: Vec<EmbeddingRef>,
    pub provenance: ProvenanceRef,
    pub temporal: TemporalMetadata,
}
```

Ein `CognitiveRecord` kann gleichzeitig in der Textsuche, im Graphen und in der Vektorsuche verwendet werden.

Die Datenmodelle werden dabei nicht als drei unabhängige Entitäten behandelt.

### B. Query Compiler

Der Compiler verarbeitet hybride Abfragen und übersetzt sie in einen physischen Query Plan.

Aufgaben:

- Parsing
    
- Schemaauflösung
    
- Typprüfung
    
- Berechtigungsprüfung
    
- Snapshot-Auflösung
    
- Predicate Pushdown
    
- Indexauswahl
    
- Kostenabschätzung
    
- Token-Budget-Planung
    

Die Query-Sprache soll beispielsweise folgende Konstruktion unterstützen:

```
MATCH
  (u:User {id: $user_id})
  -[:HAS_ACCESS|REFERENCES*1..3]->
  (c:Chunk)

WHERE
  c.tenant_id = $tenant_id
  AND c.valid_at CONTAINS $timestamp

HYBRID SEARCH (
    VECTOR(c.embedding, $query_vector, 200),
    TEXT(c.content, $query_text, 200)
)

FUSE RRF(k = 60)

TOKEN BUDGET 4000

RETURN
  c.content,
  c.provenance,
  c.source
LIMIT 12;
```

Diese Abfrage ist eine logische Operation und keine Kombination dreier externer Datenbankaufrufe.

### C. Transaction Manager

Der Transaction Manager koordiniert alle Schreib- und Leseoperationen.

Er verwaltet:

- Transaktions-IDs
    
- Snapshot-IDs
    
- Commit-IDs
    
- MVCC-Versionen
    
- Branch-Zugehörigkeit
    
- Read Sets
    
- Write Sets
    
- Konflikterkennung
    
- Commit-Sequenzierung
    

Die Transaktionsgrenze umfasst sämtliche betroffenen Modelle.

### D. Storage Kernel

Der Storage Kernel ist das eigentliche Fundament der Engine.

Er enthält:

1. WAL
    
2. LSM-Tree
    
3. Immutable Segment Store
    
4. Object Store
    
5. Buffer Pool
    
6. Manifest Manager
    
7. Compaction Scheduler
    
8. Snapshot Registry
    
9. Recovery Engine
    

Die Indexsysteme greifen auf diese Komponenten zu, statt eigene Storage- und Durability-Implementierungen zu unterhalten.

---

## 1.3 Write Path

Der Schreibpfad muss garantieren, dass ein Datensatz entweder vollständig in allen Modellen sichtbar wird oder überhaupt nicht.

Beispiel: Ein neuer Wissenschunk enthält einen Text, einen Embedding-Vektor und zwei Graphbeziehungen.

Der Ablauf:

**Schritt 1 – Ingestion**

Die API erhält den Originalinhalt sowie optionale Metadaten und Embeddings.

**Schritt 2 – Validierung**

- Tenant-Berechtigung
    
- Schema-Kompatibilität
    
- Objekt-ID
    
- Embedding-Dimension
    
- Modellversion
    
- Provenance-Vollständigkeit
    
- temporale Plausibilität
    

**Schritt 3 – Kanonische Speicherung**

Der Originaldatensatz wird in einen unveränderlichen Storage-Datensatz überführt.

**Schritt 4 – Indexdelta-Erzeugung**

Die Engine erstellt:

- Vector Delta
    
- Graph Adjacency Delta
    
- Text Posting Delta
    
- Provenance Delta
    
- Temporal Index Delta
    

Diese Deltas enthalten Referenzen auf dieselbe kanonische Objektversion.

**Schritt 5 – WAL**

Die Transaktion wird protokolliert.

**Schritt 6 – Durable Commit**

Die benötigten Datenobjekte werden dauerhaft gespeichert und der Commit wird im WAL als abgeschlossen markiert.

**Schritt 7 – Manifest-Publikation**

Ein neues gemeinsames Manifest wird veröffentlicht.

**Schritt 8 – Sichtbarkeit**

Eine atomare Aktualisierung der Branch-Referenz macht den neuen Commit sichtbar.

Die Indizes werden sofort anhand ihrer Delta-Strukturen aktualisiert. Eine spätere Kompaktierung kann die Deltas in größere Indexsegmente überführen.

Wichtig: Ein neuer Vektor darf nicht erst nach einer asynchronen HNSW-Kompaktierung suchbar werden. Er muss bereits über den aktuellen Vector Delta erreichbar sein.

## 1.4 Read Path

Der Lesevorgang folgt einer anderen Reihenfolge:

1. Branch auflösen.
    
2. Aktuellen Commit bestimmen.
    
3. Snapshot pinnen.
    
4. Query Plan erstellen.
    
5. Berechtigungen und Tenant-Filter einbeziehen.
    
6. Graph-, Text- und Vektor-Kandidaten parallel oder sequenziell ermitteln.
    
7. MVCC-Sichtbarkeit prüfen.
    
8. Kandidaten zusammenführen.
    
9. Provenance validieren.
    
10. Token-Budget anwenden.
    
11. Evidence Bundle zurückgeben.
    

Der zentrale Vorteil: Jede einzelne Komponente arbeitet mit demselben Snapshot.

Ein Graphknoten aus Commit A kann somit nicht versehentlich mit einem veralteten Embedding aus Commit B kombiniert werden.

---

# 2. Physisches Storage- und Index-Design

## 2.1 Grundentscheidung: LSM als primärer Storage-Kern

Für die erste Architektur empfehle ich einen LSM-basierten Storage-Kern mit unveränderlichen SSTables und zusätzlichen spezialisierten Immutable Index Segments.

Ein LSM-Tree ist besonders für schreibintensive Workloads interessant, weil Schreiboperationen zunächst sequenziell in einem Log beziehungsweise Memtable verarbeitet und später in sortierte Dateien überführt werden.

Das System verwendet eine Kombination aus:

- WAL
    
- Memtables
    
- immutable SSTables
    
- Immutable Index Segments
    
- Manifest Roots
    
- Background Compaction
    

Der LSM-Kern speichert die kanonischen Objekte, Versionseinträge und Metadaten.

Die spezialisierten Suchindizes sind logisch getrennte Strukturen, teilen aber ihre Commit- und Snapshot-Verwaltung.

### Warum nicht ausschließlich B+Trees?

Copy-on-Write-B+Trees bieten ausgezeichnete Snapshot-Eigenschaften und passen gut zu immutable Roots.

Ein LSM-System ist für die erwartete Kombination aus kontinuierlicher Dokumentaufnahme, Embedding-Ingestion und Agenten-Updates jedoch ein sinnvoller Ausgangspunkt.

Wichtig ist die Abwägung: LSM-Systeme verursachen Write Amplification und können bei zu vielen kleinen Segmenten unter Read Amplification leiden. Deshalb müssen Compaction und Segmentverwaltung von Beginn an Bestandteil der Architektur sein.

## 2.2 Kanonisches Datenmodell

Die Engine besitzt einen gemeinsamen logischen Datenraum.

Die wichtigsten Objekttypen:

|Objekttyp|Bedeutung|
|---|---|
|Entity|Entität des Wissensgraphen|
|Chunk|Text- oder Informationseinheit|
|Edge|Beziehung zwischen Entitäten|
|Embedding|Semantische Repräsentation|
|Claim|Konkrete Tatsachenbehauptung|
|SourceArtifact|Ursprüngliche Informationsquelle|
|Summary|Abgeleitete Zusammenfassung|
|Provenance|Herkunftsnachweis|
|Conflict|Widerspruch zwischen Aussagen|
|Commit|Version eines Datenbankzustands|

Eine Entität besitzt eine stabile logische ID.

Die physische Version eines Datensatzes erhält zusätzlich eine unveränderliche Versionsreferenz.

```
pub struct ObjectVersion {
    pub object_id: ObjectId,
    pub version_id: VersionId,
    pub commit_id: CommitId,
    pub tenant_id: TenantId,

    pub schema_version: u32,
    pub content_hash: Hash256,

    pub valid_from: Option<Timestamp>,
    pub valid_until: Option<Timestamp>,
    pub recorded_at: Timestamp,

    pub provenance_id: ProvenanceId,
}
```

Die Stable ID identifiziert das logische Objekt.

Die Version ID identifiziert einen konkreten Inhalt dieses Objekts.

## 2.3 Physisches Datenlayout

Ein möglicher logischer Aufbau auf Disk:

```
/database
│
├── wal/
│   ├── wal-000001.log
│   └── wal-000002.log
│
├── objects/
│   ├── 0a/
│   ├── 1b/
│   └── ...
│
├── manifests/
│   ├── commit_a.manifest
│   └── commit_b.manifest
│
├── segments/
│   │
│   ├── primary/
│   │   ├── level-0/
│   │   └── level-1/
│   │
│   ├── vector/
│   │   └── embedding-space-id/
│   │       ├── segment-001
│   │       └── segment-002
│   │
│   ├── graph/
│   │   ├── adjacency-out/
│   │   └── adjacency-in/
│   │
│   ├── text/
│   │   └── analyzer-version/
│   │       ├── postings
│   │       └── term-dictionary
│   │
│   └── provenance/
│
├── refs/
│   ├── main
│   └── branches/
│
├── catalog/
│
└── snapshots/
```

Die tatsächliche Implementierung darf Daten in größere Packfiles gruppieren, um Dateisystem- und Metadaten-Overhead zu reduzieren.

## 2.4 Gemeinsame Objektidentität

Alle Suchindizes verwenden dieselbe logische Referenz:

```
ObjectId
   |
   +---- Text Posting
   |
   +---- Vector Entry
   |
   +---- Graph Vertex / Edge
   |
   +---- Provenance Entry
   |
   +---- Temporal Metadata
```

Die Originaldaten werden nicht für jedes Modell erneut gespeichert.

Beispiel:

Ein Chunk besitzt den Text:

```
"Rust verwendet Ownership und Borrowing für Memory Safety."
```

Die Textsuche speichert:

- Term-IDs
    
- Term-Frequenzen
    
- Positionsinformationen
    
- Posting-Referenzen
    

Die Vektorsuche speichert:

- Vector-ID
    
- ANN-Nachbarschaften
    
- gegebenenfalls Quantisierungscodes
    

Der Graph speichert:

- Kantenreferenzen
    
- Adjazenzinformationen
    

Keiner dieser Indizes muss den gesamten Originaltext erneut ablegen.

### Unvermeidbare abgeleitete Daten

Eine vollständig duplikatfreie physische Speicherung ist nicht realistisch.

Ein HNSW-Index benötigt beispielsweise Nachbarschaftsverbindungen. Ein Inverted Index benötigt Posting-Listen. Quantisierte ANN-Indizes können zusätzliche komprimierte Vektorrepräsentationen benötigen.

Diese Daten werden als regenerierbare Index-Repräsentationen behandelt, nicht als zweite autoritative Wissensbasis.

---

## 2.5 Vector Engine

### Datenstruktur

Für jeden Vektor speichert die kanonische Datenhaltung:

```
pub struct EmbeddingRecord {
    pub id: EmbeddingId,
    pub object_id: ObjectId,

    pub model_id: EmbeddingModelId,
    pub dimensions: u32,

    pub metric: DistanceMetric,
    pub vector_ref: BlobRef,

    pub created_at: Timestamp,
    pub source_content_hash: Hash256,
}
```

Die Kombination aus Modell-ID, Dimension, Metrik und gegebenenfalls Modellrevision identifiziert einen Embedding Space.

Vektoren unterschiedlicher Embedding Spaces dürfen nicht versehentlich miteinander verglichen werden.

### ANN-Index

Die erste Implementierung sollte HNSW unterstützen.

HNSW verwendet hierarchische Nachbarschaftsgraphen. Die Hierarchie verbessert die Suche durch die Navigation über unterschiedliche Distanzeebenen. Die grundlegende Funktionsweise ist in der Originalarbeit von Malkov und Yashunin beschrieben.

Quelle: [https://arxiv.org/abs/1603.09320](https://arxiv.org/abs/1603.09320)

Empfohlener Aufbau:

```
Vector Segment
│
├── Segment Metadata
├── Vector ID Table
├── HNSW Layer 0
├── HNSW Layer 1
├── HNSW Layer 2
├── Neighbor Lists
└── Optional Quantization Codes
```

### Immutable HNSW Segments

Ein großes Problem klassischer HNSW-Implementierungen ist die effiziente Verwaltung kontinuierlicher Änderungen in einer MVCC- und Branch-Architektur.

Deshalb empfehle ich:

- Neue Vektoren landen zunächst in einem kleinen Delta-Index.
    
- Bestehende HNSW-Segmente bleiben unverändert.
    
- Gelöschte oder ersetzte Vektoren erhalten MVCC-Tombstones.
    
- Im Hintergrund werden neue HNSW-Segmente aufgebaut.
    
- Alte Segmente bleiben für bestehende Snapshots verfügbar.
    

Damit wird der HNSW-Index selbst zu einer Sammlung unveränderlicher, versionierbarer Datenobjekte.

### Skalierung auf SSD

Für sehr große Vektorkorpora sollte eine spätere Implementierung einen DiskANN-/Vamana-artigen Ansatz evaluieren.

DiskANN demonstriert graphbasierte ANN-Suche auf SSDs und wurde unter anderem für sehr große Vektordatenbestände untersucht. Neuere Arbeiten behandeln auch gefilterte und aktualisierbare Varianten.

Quellen:  
[https://www.microsoft.com/en-us/research/publication/diskann-fast-accurate-billion-point-nearest-neighbor-search-on-a-single-node/](https://www.microsoft.com/en-us/research/publication/diskann-fast-accurate-billion-point-nearest-neighbor-search-on-a-single-node/)

[https://www.microsoft.com/en-us/research/publication/the-diskann-library-graph-based-indices-for-fast-fresh-and-filtered-vector-search/](https://www.microsoft.com/en-us/research/publication/the-diskann-library-graph-based-indices-for-fast-fresh-and-filtered-vector-search/)

### Query-Verhalten

Der Vektorindex liefert:

```
ObjectId
Similarity
Embedding Space
Index Segment
Version Reference
```

Die finale Sichtbarkeitsprüfung findet anschließend im gemeinsamen MVCC-Layer statt.

---

## 2.6 Property Graph Engine

Der Property Graph besitzt zwei grundlegende Objektklassen.

### Vertex

```
pub struct Vertex {
    pub id: VertexId,
    pub tenant_id: TenantId,
    pub labels: Vec<LabelId>,
    pub properties: PropertyMap,
    pub provenance: ProvenanceRef,
}
```

### Edge

```
pub struct Edge {
    pub id: EdgeId,

    pub source: VertexId,
    pub target: VertexId,

    pub relation: RelationId,
    pub properties: PropertyMap,

    pub confidence: ConfidenceMetadata,
    pub provenance: ProvenanceRef,

    pub valid_from: Option<Timestamp>,
    pub valid_until: Option<Timestamp>,
}
```

### Physische Adjazenzstruktur

Für unveränderliche Graphsegmente bietet sich ein CSR-artiges Layout an.

CSR steht für Compressed Sparse Row.

Eine Knoten-ID verweist auf einen zusammenhängenden Adjazenzbereich.

```
Vertex A
   |
   +-- Edge 1 --> Vertex B
   |
   +-- Edge 2 --> Vertex C
   |
   +-- Edge 3 --> Vertex D
```

Empfohlen werden getrennte Zugriffspfade für:

- Outgoing Adjacency
    
- Incoming Adjacency
    
- Label Index
    
- Property Index
    
- Edge Type Index
    

Die eigentlichen Kanten und Vertex Properties bleiben kanonische Objekte.

Der Adjazenzindex enthält Referenzen auf die entsprechenden Kanten.

### Graph Traversal

Der Query Executor unterstützt:

- BFS
    
- DFS
    
- Multi-Hop Traversal
    
- Path Queries
    
- Label Filtering
    
- Property Predicates
    
- Temporal Filtering
    
- Provenance Filtering
    

Traversals müssen durch konfigurierbare Grenzen abgesichert werden:

```
pub struct TraversalLimits {
    pub max_depth: u32,
    pub max_visited_nodes: u64,
    pub max_paths: u64,
    pub max_execution_time_ms: u64,
}
```

Das ist insbesondere für autonome Agenten wichtig, damit eine ungünstige Query nicht unkontrolliert große Graphbereiche durchläuft.

---

## 2.7 Volltext-Engine

Die Volltextsuche basiert auf einem Inverted Index.

### Tokenisierung

Der Text wird nicht erneut als Dokumentkopie abgelegt.

Die Engine verarbeitet ihn durch eine versionierte Analyzer Pipeline:

1. Unicode Normalization
    
2. Lowercasing
    
3. Language Detection
    
4. Tokenization
    
5. Optional Stemming
    
6. Optional Stopword Filtering
    
7. Term Dictionary Lookup
    
8. Posting Creation
    

Eine Analyzer-Version muss Bestandteil der Indexmetadaten sein. Änderungen an Tokenisierung oder Normalisierung erfordern einen kontrollierten Index-Rebuild.

### Posting-Struktur

Ein Posting enthält ungefähr:

```
pub struct Posting {
    pub object_id: ObjectId,
    pub version_id: VersionId,

    pub term_frequency: u32,
    pub document_length: u32,

    pub positions: Vec<u32>,
}
```

Die Posting-Listen werden sortiert abgelegt und mittels Delta-Encoding und komprimierten Blöcken gespeichert.

Zusätzliche Optimierungen:

- Skip Lists
    
- Block-Max-WAND
    
- Term Dictionary als FST
    
- Komprimierte Positionsinformationen
    
- MVCC-konforme Sichtbarkeitsfilter
    

### BM25

Das lexikalische Ranking verwendet BM25.

Eine gebräuchliche Form lautet:

```
BM25(D,Q) =

Σ IDF(q) *
  [ tf(q,D) * (k1 + 1) ] /
  [ tf(q,D) + k1 * (1 - b + b * |D| / avgdl) ]
```

Dabei bezeichnen:

- `tf`: Term Frequency
    
- `df`: Document Frequency
    
- `N`: Anzahl der Dokumente
    
- `|D|`: Dokumentlänge
    
- `avgdl`: durchschnittliche Dokumentlänge
    
- `k1`, `b`: Rankingparameter
    

Die Textstatistiken müssen zum jeweiligen Snapshot passen. Dazu werden Segmentstatistiken und Sichtbarkeitsinformationen im Query Executor zusammengeführt.

Tantivy ist hierfür ein sinnvoller Implementierungs- und Performance-Referenzpunkt. Seine Rust-Dokumentation stellt unter anderem BM25-Statistikprovider und segmentbasierte Suche bereit.

Quelle: [https://docs.rs/tantivy/latest/tantivy/](https://docs.rs/tantivy/latest/tantivy/)

---

## 2.8 Hybride Query Engine

Die hybride Suche soll die drei Indexarten nicht einfach nacheinander aufrufen, sondern als gemeinsamen Query Plan ausführen.

### Beispiel: Graph-gefilterte semantische Suche

Ein Agent möchte alle Inhalte finden, die:

- mit einer bestimmten Entität verbunden sind,
    
- maximal drei Graphschritte entfernt liegen,
    
- semantisch einer Suchanfrage entsprechen,
    
- bestimmte Begriffe enthalten,
    
- über eine überprüfbare Quelle verfügen.
    

Der Planner entscheidet anhand von Selektivität und geschätzten Kosten über die Ausführungsreihenfolge.

Möglicher Ablauf:

```
1. Tenant Filter
2. Graph Traversal
3. Kandidatenmenge erzeugen
4. Vector Search
5. Text Search
6. MVCC Visibility Check
7. Reciprocal Rank Fusion
8. Provenance Re-Ranking
9. Token Budget Optimization
10. Result
```

### Adaptive Ausführungsreihenfolge

Wenn eine Graphfilterung nur wenige Kandidaten ergibt, ist es effizienter, die erlaubten Kandidaten exakt gegen die Vektoren zu prüfen.

Wenn die Graphfilterung sehr viele Kandidaten liefert, ist eine gefilterte ANN-Suche geeigneter.

Wichtig: Eine gefilterte HNSW-Suche darf nicht blind sämtliche nicht zugelassenen Knoten während der Navigation ausschließen. Das kann die Erreichbarkeit relevanter Kandidaten verschlechtern.

Der Planner soll deshalb zwischen folgenden Strategien wechseln:

- Exact Search innerhalb einer kleinen Kandidatenmenge
    
- ANN mit Oversampling
    
- Iterative Filtered Search
    
- Ungefilterte Navigation mit anschließendem Visibility Check
    

### Reciprocal Rank Fusion

Die Ergebnisse von BM25 und Vektorsuche besitzen unterschiedliche Score-Skalen.

Eine direkte Addition der Rohscores ist deshalb problematisch.

Stattdessen wird Reciprocal Rank Fusion verwendet:

```
RRF(d) = Σ 1 / (k + rank_i(d))
```

Beispielsweise:

```
Vector Search:
A = Rank 1
B = Rank 2
C = Rank 3

Text Search:
B = Rank 1
A = Rank 4
D = Rank 2
```

RRF erzeugt daraus eine gemeinsame Rangfolge, ohne dass cosine similarity und BM25 künstlich dieselbe numerische Bedeutung bekommen müssen.

Die Methode wurde von Cormack, Clarke und Buettcher untersucht und in SIGIR 2009 veröffentlicht.

Quelle:  
[https://doi.org/10.1145/1571941.1572114](https://doi.org/10.1145/1571941.1572114)

Der Wert `k` bleibt konfigurierbar; 60 ist ein sinnvoller initialer Ausgangspunkt.

---

# 3. Versionierungs- und Concurrency-Engine

## 3.1 Architekturüberblick

Die Versionierungsengine ist ein zentraler Bestandteil des Produkts und kein nachträglich hinzugefügtes Feature.

Sie kombiniert:

- MVCC
    
- Immutable Objects
    
- Immutable Manifests
    
- Merkle-DAG
    
- Branch References
    
- Commit History
    
- Optimistic Concurrency Control
    

Der wesentliche Gedanke ist, dass jeder Datenbankzustand über unveränderliche Datenstrukturen repräsentiert wird.

Eine Branch-Referenz zeigt lediglich auf einen Commit.

## 3.2 Commit-Modell

Ein Commit enthält:

```
pub struct Commit {
    pub id: CommitId,

    pub parents: Vec<CommitId>,
    pub root_manifest: ManifestHash,

    pub author: PrincipalId,
    pub message: String,

    pub commit_timestamp: Timestamp,
    pub commit_sequence: u64,

    pub schema_version: u32,
}
```

Ein normaler Commit besitzt einen Parent.

Ein Merge-Commit besitzt zwei Parents.

Die Commit-ID wird aus einer kanonischen serialisierten Commit-Repräsentation berechnet.

Die Datenobjekte und Manifeste besitzen ebenfalls Content Hashes.

Das ergibt einen gerichteten azyklischen Graphen:

```
                 C0
                /  \
               C1   C2
                \   /
                 C3
                 |
                 C4
```

Unveränderte Objekte müssen nicht kopiert werden.

Ein neuer Commit verweist auf bestehende Objekte und ergänzt lediglich seine Änderungen.

### Kryptografische Hashes

BLAKE3 ist ein geeigneter Kandidat für Content Addressing und die Integritätsprüfung großer Datenobjekte.

Quelle: [https://docs.rs/blake3/latest/blake3/](https://docs.rs/blake3/latest/blake3/)

Wichtig ist die Trennung von:

- Objekt-Hash
    
- Manifest-Hash
    
- Commit-Hash
    
- Provenance-Signatur
    

Ein Hash beweist die Integrität eines Inhalts, nicht die Wahrheit einer darin enthaltenen Aussage.

## 3.3 Manifest-System

Das Manifest ist die gemeinsame Beschreibung eines vollständigen Datenbankzustands.

```
pub struct CommitManifest {
    pub primary_root: StorageRoot,

    pub vector_roots: Vec<VectorIndexRoot>,
    pub graph_root: GraphIndexRoot,
    pub text_root: TextIndexRoot,

    pub provenance_root: ProvenanceRoot,
    pub temporal_root: TemporalRoot,

    pub schema_epoch: u64,
}
```

Ein Commit-Manifest verweist auf alle zum Snapshot gehörenden Segmente.

Ein unverändertes Vektorsegment kann von mehreren Branches gemeinsam verwendet werden.

Dasselbe gilt für Graph- und Textsegmente.

Die Konsistenzgarantie lautet:

```
Alle sichtbaren Indexsegmente gehören zu einem
gemeinsamen Commit-Zustand.
```

Damit wird das klassische Problem verhindert, dass ein Textindex bereits den neuen Datensatz enthält, während die Graphdaten noch auf dem vorherigen Stand sind.

## 3.4 Zero-Copy Branching

Ein Branch ist primär eine neue Referenz auf einen bestehenden Commit.

Beispiel:

```
main
 |
 C100
 |
 C101
 |
 C102
```

Ein Agent erstellt einen Branch:

```
main
 |
 C100
 |
 C101
 |
 C102
      \
       agent-branch
```

Der Agent-Branch verwendet zunächst exakt dieselben unveränderlichen Datenobjekte und Indexsegmente.

Er benötigt nur einen neuen Branch-Eintrag und gegebenenfalls einen kleinen Commit.

Es findet keine vollständige Kopie des Datenbestands statt.

Das ist das eigentliche Zero-Copy-Prinzip.

Sobald Änderungen entstehen, werden neue immutable Segmente ergänzt.

Unveränderte Segmente bleiben geteilt.

## 3.5 MVCC

Jede Query erhält einen Snapshot.

Der Snapshot enthält mindestens:

```
pub struct Snapshot {
    pub branch_id: BranchId,
    pub commit_id: CommitId,
    pub manifest_hash: ManifestHash,
    pub commit_sequence: u64,
}
```

Die Query arbeitet während ihrer gesamten Lebensdauer auf diesem Snapshot.

Ein anderer Thread darf währenddessen neue Versionen schreiben.

Der Leser sieht trotzdem einen konsistenten Zustand.

Das gilt für sämtliche Datenmodelle.

### Sichtbarkeitsregeln

Vereinfachtes MVCC-Modell:

```
Version sichtbar, wenn:

begin_version <= snapshot_version

UND

(end_version ist nicht gesetzt
 ODER end_version > snapshot_version)
```

In der tatsächlichen Branch-Implementierung erfolgt die Auflösung über den vom Commit referenzierten Root und dessen Versionsinformationen.

Dadurch bleiben branchspezifische Zustände voneinander isoliert.

### Isolationsstufen

Empfehlung:

|Isolation|Verhalten|
|---|---|
|Snapshot|Konsistente Sicht auf einen Commit|
|Read Committed|Neuer Snapshot pro Transaktionsabschnitt|
|Serializable|Zusätzliche Validierung von Read- und Predicate-Sets|

Der Standard sollte Snapshot Isolation sein.

Serializable wird für besonders kritische Änderungen bereitgestellt.

Write-Write-Konflikte werden durch Optimistic Validation erkannt. Für Serialisierbarkeit müssen zusätzlich relevante Bereichs- und Predicate-Abhängigkeiten berücksichtigt werden.

## 3.6 Branch Merge

Der Merge verwendet einen Three-Way-Merge.

```
         Common Ancestor
             C100
            /    \
       main       agent
        C110      C115
            \    /
             Merge
```

Verglichen werden:

1. Common Ancestor
    
2. Aktueller Main-Zustand
    
3. Agenten-Branch-Zustand
    

### Konfliktregeln

**Unterschiedliche Objekte:** Automatischer Merge.

**Unterschiedliche Property-Felder:** Automatischer Merge, sofern keine Schema-Constraints verletzt werden.

**Dieselbe Property mit unterschiedlichen Werten:** Konflikt.

**Gleichzeitig gelöschte und veränderte Edge:** Konflikt.

**Unterschiedliche Embeddings desselben logischen Inhalts:** Konflikt beziehungsweise Neuberechnung.

**Unterschiedliche Textversionen:** Inhaltlicher Konflikt oder explizit definierte Merge-Strategie.

Vektoren dürfen nicht einfach komponentenweise gemittelt werden.

Ein Embedding ist eine Modellprojektion eines konkreten Inhalts. Nach einem Text-Merge sollte es bei Bedarf mit der festgelegten Embedding-Modellversion neu berechnet werden.

### Merge-Ablauf

1. Merge Base finden.
    
2. Geänderte Objektversionen vergleichen.
    
3. Semantische Konflikte ermitteln.
    
4. Konfliktfreie Änderungen übernehmen.
    
5. Konfliktobjekte erzeugen.
    
6. Neue Index-Deltas erstellen.
    
7. Neues Manifest berechnen.
    
8. Merge-Commit mit zwei Parents erzeugen.
    
9. Branch-Head atomar aktualisieren.
    

Der Merge wird als einzelne Transaktion veröffentlicht.

## 3.7 Time Travel und Diff

Time Travel erfolgt über eine Branch- und Commit-Referenz.

Beispiele:

```
SELECT *
FROM knowledge
AT BRANCH 'main'
AT COMMIT 'commit-abc';
```

Oder:

```
SELECT *
FROM knowledge
AT BRANCH 'main'
AT TIMESTAMP '2026-10-01T10:00:00Z';
```

Bei Zeitabfragen wird der entsprechende Commit mit dem höchsten geeigneten Commit-Zeitpunkt ausgewählt.

Eine historische Query erhält anschließend dessen Manifest.

### Diff

Durch Content Hashes kann das System zunächst identische Teilstrukturen überspringen.

Ein Diff umfasst:

- hinzugefügte Objekte
    
- entfernte Objekte
    
- geänderte Properties
    
- neue und entfernte Graphkanten
    
- Embedding-Änderungen
    
- geänderte Textinhalte
    
- Provenance-Änderungen
    
- temporale Änderungen
    

### Rollback

Es sind zwei Operationen zu unterscheiden:

**Revert:**

Ein neuer Commit nimmt die Änderungen eines früheren Commits logisch zurück.

**Reset:**

Eine Branch-Referenz wird auf einen früheren Commit zurückgesetzt.

Für `main` sollte Reset standardmäßig gesperrt oder administrativ geschützt sein.

Ein Revert ist im Allgemeinen sicherer, weil die Historie nachvollziehbar bleibt.

## 3.8 Garbage Collection

Zero-Copy-Branching benötigt eine gezielte Speicherbereinigung.

Die Garbage Collection darf nur Objekte löschen, die von keinem gültigen Root mehr erreichbar sind.

Roots sind:

- aktive Branches
    
- Tags
    
- geschützte Commits
    
- laufende Snapshots
    
- aktive Transaktionen
    
- Backup-Snapshots
    
- explizit gepinnte Historien
    

Der Collector markiert zunächst alle erreichbaren Hashes und entfernt anschließend nicht referenzierte Daten.

Aktive Reader müssen durch Snapshot-Leases geschützt werden.

Damit wird verhindert, dass ein Segment gelöscht wird, während eine Query noch darauf zugreift.

---

# 4. Rust Concurrency, WAL und Durability

## 4.1 Concurrency-Modell

Rusts Ownership-Modell bildet die Grundlage der Implementierung.

Das Ziel ist allerdings nicht, jede Datenstruktur zwanghaft lock-free zu gestalten.

Eine Datenbank muss vor allem vorhersagbares Verhalten, korrekte Speicherfreigabe und konsistente Transaktionssemantik garantieren.

Empfohlene Strategie:

- Immutable Daten für Reads
    
- Arc-basierte gemeinsame Ownership
    
- Atomic Manifest Publication
    
- Sharded Writers
    
- Epoch-Based Memory Reclamation
    
- Kurze Lock-Abschnitte für administrative Koordination
    

### Read-mostly Architecture

Der aktuelle Manifest-Pointer kann über `ArcSwap` veröffentlicht werden.

Leser laden einen konsistenten `Arc<Manifest>` und halten diesen für die Dauer ihrer Query.

Dadurch muss eine Query keinen globalen Read Lock über die gesamte Ausführungszeit halten.

`arc-swap` ist ausdrücklich für solche read-mostly Szenarien gedacht.

Quelle: [https://docs.rs/arc-swap/latest/arc_swap/](https://docs.rs/arc-swap/latest/arc_swap/)

### Lock-free Strukturen

Für geeignete Index-Caches kann `papaya` evaluiert werden. Die Bibliothek stellt eine lock-free API für read-heavy Concurrent Hash Maps bereit.

Quelle: [https://docs.rs/papaya/latest/papaya/](https://docs.rs/papaya/latest/papaya/)

Für die sichere Freigabe gemeinsam genutzter Speicherbereiche bietet sich `crossbeam-epoch` an.

Die Idee ist, entfernte Speicherobjekte erst dann tatsächlich freizugeben, wenn kein aktiver Leser mehr darauf zugreifen kann.

Quelle: [https://docs.rs/crossbeam-epoch/latest/crossbeam_epoch/](https://docs.rs/crossbeam-epoch/latest/crossbeam_epoch/)

### Praktische Einschränkung

Eine atomare Pointer-Veröffentlichung macht nicht automatisch die gesamte Transaktion lock-free.

Die Engine benötigt weiterhin geordnete Commit-Operationen, Branch-Head-Validierung und kontrollierte Writer-Koordination.

Wait-free Garantien sollten nur für konkret analysierte Einzeloperationen behauptet werden.

---

## 4.2 Write-Ahead Logging

Das WAL stellt die Grundlage der Durability sicher.

Ein WAL Record enthält:

```
pub struct WalRecordHeader {
    pub transaction_id: TransactionId,
    pub sequence: u64,

    pub record_type: WalRecordType,
    pub payload_length: u32,

    pub checksum: u32,
}
```

Mögliche Record-Typen:

```
BEGIN
PUT_OBJECT
DELETE_OBJECT
PUT_EDGE
PUT_VECTOR
INDEX_DELTA
MANIFEST_UPDATE
COMMIT
ABORT
CHECKPOINT
```

### Commit-Protokoll

Der Commit-Vorgang muss die folgende Reihenfolge einhalten:

1. Write Set validieren.
    
2. Unveränderliche Daten- und Indexobjekte erzeugen.
    
3. Dateien schreiben.
    
4. Daten und notwendige Verzeichniseinträge dauerhaft sichern.
    
5. WAL Records schreiben.
    
6. Commit Record mit Manifest-Hash anhängen.
    
7. WAL synchronisieren.
    
8. Branch-Head atomar publizieren.
    
9. Neue Manifest-Version für Leser verfügbar machen.
    

Der Commit darf vor der dauerhaften Sicherung seiner referenzierten Objekte nicht als erfolgreich gemeldet werden.

Die konkrete Implementierung kann Group Commit verwenden.

Dabei warten mehrere Transaktionen gemeinsam auf eine Synchronisierung des WAL.

### Recovery

Beim Neustart:

1. Letzten gültigen Checkpoint laden.
    
2. WAL ab diesem Checkpoint lesen.
    
3. Checksummen validieren.
    
4. Abgeschlossene Transaktionen rekonstruieren.
    
5. Nicht abgeschlossene Transaktionen verwerfen.
    
6. Neue Manifeste und Branch-Heads wiederherstellen.
    
7. Nicht referenzierte Staging-Dateien markieren.
    

Ein gültiger Commit muss nach einem Prozess- oder Systemabsturz vollständig rekonstruierbar sein.

## 4.3 Snapshotting

Snapshots reduzieren die erforderliche WAL-Recovery-Arbeit.

Ein Snapshot enthält:

- Commit-ID
    
- Manifest-Hash
    
- LSM-Root
    
- Index-Segment-Referenzen
    
- WAL-Checkpoint
    
- Schema-Epoch
    

Wichtig ist, dass ein Snapshot sämtliche Datenmodelle auf denselben Commit-Zustand referenziert.

Snapshotting darf nicht drei voneinander unabhängige Datenbankzustände erzeugen.

## 4.4 Buffer Pool

Der Buffer Pool sollte verschiedene Cache-Klassen enthalten.

|Cache|Inhalt|
|---|---|
|Page Cache|LSM Pages|
|Vector Cache|Embedding-Blöcke und ANN-Nachbarschaften|
|Graph Cache|Adjazenzbereiche|
|Posting Cache|Text-Posting-Blöcke|
|Metadata Cache|Manifeste und Schema|
|Provenance Cache|Häufig abgefragte Herkunftsnachweise|

Die Speicherkontrolle muss zentral erfolgen.

Ein Vektorindex darf nicht unkontrolliert den gesamten RAM verbrauchen, wenn dadurch Graph- und Textabfragen unter Speicherdruck geraten.

Für die erste Implementierung bietet sich ein adaptives Cache-Budget an.

Eine Kombination aus Admission Policy, LRU-artigem Replacement und getrennten Cache-Budgets ist sinnvoll.

## 4.5 Asynchrones I/O

Empfehlung:

- Tokio für Netzwerk- und asynchrone I/O-Operationen.
    
- Dedizierte Worker für Storage-I/O.
    
- Rayon oder vergleichbare CPU-Worker für rechenintensive Aufgaben.
    
- Optional io_uring für Linux-spezifische Storage-Optimierungen.
    

`tokio-uring` stellt eine sichere io_uring-Anbindung für Tokio bereit. Die aktuelle Dokumentation beschreibt allerdings eine Current-Thread-Runtime und Ressourcen, die teilweise `!Sync` sind. Für zusätzliche Parallelität werden getrennte Runtime-Threads benötigt.

Quelle: [https://docs.rs/tokio-uring/latest/tokio_uring/](https://docs.rs/tokio-uring/latest/tokio_uring/)

Deshalb würde ich io_uring zunächst als optionalen Backend-Pfad entwickeln.

Die portable Tokio-Variante bleibt der Standard.

ANN-Distanzen, Graph Traversals, BM25-Berechnungen und Kompaktierung dürfen keine Tokio-Reaktor-Threads blockieren.

## 4.6 Zero-Copy-Datenzugriff

Zwei Technologien bieten sich besonders an.

### rkyv

Für interne persistente Objekte.

rkyv ermöglicht den Zugriff auf archivierte Datenstrukturen ohne klassische vollständige Deserialisierung.

Quelle: [https://docs.rs/rkyv/latest/rkyv/](https://docs.rs/rkyv/latest/rkyv/)

Wichtige Regeln:

- Schema-Versionierung
    
- definierte Alignment-Anforderungen
    
- Validierung archivierter Daten
    
- keine ungeprüften Archive aus nicht vertrauenswürdigen Quellen
    
- kontrollierte Migration alter Datenformate
    

Zero-Copy ist kein Ersatz für Integritäts- und Sicherheitsprüfungen.

### Apache Arrow

Für:

- Columnar Query Execution
    
- Vektorverarbeitung
    
- Batch APIs
    
- Datenexport
    
- Interoperabilität mit Analyse- und ML-Systemen
    

Arrow IPC ist überwiegend Zero-Copy-fähig, kann unter bestimmten Bedingungen jedoch auch Speicherallokationen und Kopien erfordern, etwa bei Kompression oder Alignment-Anforderungen.

Quelle: [https://arrow.apache.org/docs/cpp/api/ipc.html](https://arrow.apache.org/docs/cpp/api/ipc.html)

Ich würde rkyv für interne Storage-Objekte und Arrow für Batch Processing sowie externe Datenübergabe verwenden.

---

# 5. LLM- und Agenten-Spezialfunktionen

Dieser Bereich ist das eigentliche Differenzierungsmerkmal des Produkts.

Eine normale Datenbank speichert Informationen.

Eine Cognitive Database muss zusätzlich speichern können:

- Woher eine Information stammt.
    
- Wer sie eingefügt hat.
    
- Wie sie abgeleitet wurde.
    
- Wie belastbar sie ist.
    
- Welche anderen Informationen ihr widersprechen.
    
- Wann sie galt.
    
- In welchem Kontext ein Agent sie verwendet hat.
    

## 5.1 Kryptografisches Provenance-Tracking

### Grundproblem

Ein LLM kann eine Information überzeugend formulieren, obwohl die Information falsch ist oder keine verifizierbare Quelle existiert.

Eine gewöhnliche RAG-Datenbank speichert häufig nur Text und Embedding.

Das reicht für eine belastbare Wissensbasis nicht aus.

### Lösung: Provenance als natives Datenmodell

Jeder Chunk und jede relevante Graphkante erhält einen Herkunftsnachweis.

```
pub struct Provenance {
    pub id: ProvenanceId,

    pub source_id: SourceId,
    pub source_hash: Hash256,

    pub source_location: String,
    pub captured_at: Timestamp,

    pub extractor_id: ModelOrToolId,
    pub extractor_version: String,

    pub extraction_method: ExtractionMethod,

    pub evidence_span: Option<TextSpan>,
    pub confidence: ConfidenceMetadata,

    pub signature: Option<Signature>,
}
```

### Provenance DAG

Die Herkunft einer Information wird als eigener gerichteter Graph dargestellt.

Beispiel:

```
Original Source
      |
      v
  Document
      |
      v
    Chunk
      |
      v
 Extracted Claim
      |
      v
 Graph Relationship
      |
      v
 Generated Summary
```

Jeder Übergang referenziert die Herkunft des vorherigen Objekts.

Dadurch kann eine Antwort auf ihren Originalinhalt zurückgeführt werden.

### Kryptografische Integrität

Für jeden Source-Inhalt wird ein Hash berechnet.

Eine Provenance-Attestation kann zusätzlich signiert werden.

Damit kann die Engine prüfen:

- Ob der Inhalt verändert wurde.
    
- Ob die Referenz auf die ursprüngliche Quelle unverändert ist.
    
- Ob eine signierte Attestation gültig ist.
    
- Ob eine abgeleitete Aussage auf den dokumentierten Beleg zurückgeht.
    

### Confidence-Modell

Ein einziger Confidence-Wert ist zu wenig aussagekräftig.

Empfohlen wird ein mehrdimensionales Modell:

```
pub struct ConfidenceMetadata {
    pub extraction_confidence: f32,
    pub source_reliability: f32,
    pub corroboration_score: f32,
    pub contradiction_penalty: f32,

    pub calibration_profile: Option<String>,
}
```

Diese Werte müssen getrennt gespeichert werden.

Eine hohe Extraction Confidence bedeutet beispielsweise nicht, dass die ursprüngliche Quelle vertrauenswürdig ist.

Die Engine darf daraus keine unberechtigte Wahrheitsgarantie ableiten.

### Evidence Gate

Die Query Engine erhält konfigurierbare Provenance-Regeln.

Beispiele:

- Nur Inhalte mit verifizierbarer Originalquelle.
    
- Nur Claims mit mindestens zwei unabhängigen Belegen.
    
- Keine Aussagen aus unbestätigten Agenten-Sessions.
    
- Kontroverse Aussagen müssen ausdrücklich markiert werden.
    
- Jede zurückgegebene Aussage muss auf einen konkreten Source-Span verweisen können.
    

Damit wird die Datenbank selbst zum aktiven Bestandteil der Halluzinationsprävention.

---

## 5.2 Token-Aware Query Planner

### Problem

Ein LLM besitzt ein begrenztes Context Window.

Selbst bei großen Context Windows können sehr umfangreiche Kontexte ineffizient sein.

Zusätzlich kann relevante Information durch Positionierung und Redundanz an Nutzwert verlieren.

Ein normaler Retrieval Planner optimiert primär die Suchrelevanz.

UCD muss dagegen die Relevanz pro Token optimieren.

### Token Budget

Das verfügbare Budget wird dynamisch berechnet.

```
Available Retrieval Budget =

Model Context Limit
- System Prompt
- User Prompt
- Tool Schemas
- Conversation History
- Reserved Output Tokens
- Safety Margin
```

Der Planner verwendet einen zum jeweiligen Modell passenden Tokenizer.

Eine reine Wortanzahl-Schätzung reicht nicht aus.

### Scoring-Modell

Für jeden Kandidaten wird ein Retrieval Utility Score berechnet.

Ein möglicher Ansatz:

```
Utility(d) =

α * Semantic Relevance
+ β * Lexical Relevance
+ γ * Graph Relevance
+ δ * Provenance Score
+ ε * Freshness Score
- λ * Redundancy Penalty
```

Das Ergebnis wird zusätzlich durch den Tokenverbrauch bewertet.

```
Information Density(d) = Utility(d) / TokenCost(d)
```

Die Parameter müssen anhand realer Retrieval- und Answering-Evaluationen kalibriert werden.

### Redundanzfilter

Die Engine entfernt:

- Identische Chunks
    
- Überlappende Textabschnitte
    
- Semantisch redundante Zusammenfassungen
    
- Wiederholte Aussagen aus derselben Quelle
    

Dabei dürfen unterschiedliche zeitliche Versionen einer Aussage nicht versehentlich zusammengeführt werden.

Für die Deduplizierung kommen Content Hashes, Token-Overlap, MinHash und optionale semantische Ähnlichkeitsprüfungen infrage.

### Hierarchische Abstraktion

Die Datenbank sollte mehrere Informationsstufen unterstützen:

```
Level 0: Original Source
   |
Level 1: Raw Chunk
   |
Level 2: Condensed Chunk Summary
   |
Level 3: Document Summary
   |
Level 4: Entity / Topic Summary
```

Ein Agent kann zunächst auf Ebene 3 suchen und nur bei Bedarf auf Ebene 0 oder 1 zurückgreifen.

Entscheidend ist, dass Zusammenfassungen niemals als unabhängige Originalquellen behandelt werden.

Jede Summary enthält:

- Source References
    
- Hashes ihrer Eingangsdaten
    
- verwendete Modellversion
    
- Erzeugungszeitpunkt
    
- Abhängigkeiten
    

Ändert sich eine Originalquelle, wird die abhängige Summary als veraltet markiert.

### Lost-in-the-Middle-Optimierung

Der Planner soll ein Modellprofil verwalten.

Dieses beschreibt, wie die Qualität der Informationsverarbeitung mit der Position innerhalb des Context Windows variiert.

Ein möglicher Ansatz:

```
Context:

[High Priority Evidence]
[Supporting Information]
[Related Graph Facts]
[Low Priority Context]
[Important Counterevidence]
[Primary Source Excerpts]
```

Die wichtigsten Belege werden an Positionen platziert, die sich in den jeweiligen Modellauswertungen als zuverlässig erweisen.

Eine starre Regel, dass Informationen immer am Anfang und Ende stehen müssen, wäre nicht ausreichend. Das Verhalten ist modell- und aufgabenabhängig und muss experimentell geprüft werden.

### Finales Evidence Bundle

Die Datenbank gibt nicht nur Dokumente zurück, sondern eine strukturierte Evidence Collection:

```
pub struct EvidenceBundle {
    pub query_id: QueryId,
    pub snapshot: SnapshotId,

    pub token_budget: u32,
    pub tokens_used: u32,

    pub evidence: Vec<EvidenceItem>,
    pub omitted_count: u32,
}
```

Jede EvidenceItem enthält:

- Originalreferenz
    
- relevante Textspanne
    
- Provenance
    
- Ranking
    
- Confidence-Metadaten
    
- zeitliche Gültigkeit
    
- Tokenkosten
    

Damit erhält das LLM eine kontrollierte und überprüfbare Kontextbasis.

---

## 5.3 Ephemere Branches für Agenten-Sessions

### Problem

Autonome Agenten arbeiten nicht deterministisch.

Sie können:

- Falsche Fakten erzeugen.
    
- Tools falsch aufrufen.
    
- Veraltete Informationen speichern.
    
- Beziehungen falsch interpretieren.
    
- Unvollständige Änderungen durchführen.
    
- Unbeabsichtigt globale Informationen überschreiben.
    

Eine direkte Schreibberechtigung auf `main` ist deshalb problematisch.

### Lösung: Agenten als isolierte Branches

Jede Agenten-Session erstellt einen eigenen Branch.

```
main
 |
 C100
 |
 C101
  \
   agent-session-001
   agent-session-002
   agent-session-003
```

Jeder Branch besitzt:

- eigenen Write Set
    
- eigene Commit-Historie
    
- eigene Provenance-Einträge
    
- eigene Konflikte
    
- eigenen TTL
    
- eigene Ressourcenlimits
    

### Agenten-Operationen

Ein Agent darf:

```
CREATE ENTITY
CREATE EDGE
INSERT CHUNK
CREATE CLAIM
UPDATE SUMMARY
DELETE PROVISIONAL RECORD
```

Diese Änderungen gelten ausschließlich innerhalb seines Branches.

Sie verändern `main` nicht.

### Session Isolation

Ein Agenten-Branch besitzt eine Capability Policy.

```
pub struct AgentCapabilities {
    pub read_main: bool,
    pub write_session_branch: bool,
    pub merge_to_main: bool,

    pub require_provenance: bool,
    pub max_write_operations: u64,
    pub expires_at: Timestamp,
}
```

Das standardmäßige `merge_to_main` ist deaktiviert.

### Promotion

Nach Abschluss eines Agenten-Tasks erfolgt ein Validierungsprozess:

1. Branch-Diff erstellen.
    
2. Neue Aussagen analysieren.
    
3. Quellen prüfen.
    
4. Widersprüche erkennen.
    
5. Schema- und Integritätsregeln prüfen.
    
6. Merge-Konflikte auflösen.
    
7. Approval Policy ausführen.
    
8. Merge-Commit erzeugen.
    

Ein Agent kann Wissen vorbereiten.

Die Entscheidung über globale Wissensänderungen erfolgt anhand definierter Berechtigungs- und Validierungsregeln.

### Session Cleanup

Abgelaufene Branches können bereinigt werden, sofern sie nicht durch aktive Snapshots, explizite Aufbewahrungsregeln oder Audit-Policies geschützt sind.

Ihre unveränderten Datenobjekte werden erst entfernt, wenn sie nicht mehr von anderen Roots referenziert werden.

---

## 5.4 Temporales Indexing und Konflikterkennung

### Problem

Wissensbasen enthalten häufig Aussagen, die zu unterschiedlichen Zeitpunkten gültig waren.

Beispiel:

```
2024: Alice arbeitet bei Unternehmen A.
2025: Alice arbeitet bei Unternehmen B.
```

Eine einfache Key-Value-Überschreibung würde historische Informationen verlieren.

Zwei Aussagen dürfen nicht automatisch als Widerspruch behandelt werden, wenn sie zu unterschiedlichen Zeitpunkten gelten.

### Bitemporal Model

Jede relevante Aussage erhält zwei Zeitdimensionen:

**Valid Time:** Wann galt die Information in der realen Welt?

**Transaction Time:** Wann wurde die Information in der Datenbank erfasst?

Beispiel:

```
pub struct TemporalMetadata {
    pub valid_from: Timestamp,
    pub valid_until: Option<Timestamp>,

    pub recorded_at: Timestamp,
    pub superseded_at: Option<Timestamp>,
}
```

Dadurch werden zwei Fragestellungen möglich:

- Was wussten wir am 1. Januar?
    
- Was galt in der Realität am 1. Januar?
    

Das ist besonders für historische Agentenentscheidungen wichtig.

### Time Decay

Ein optionales Freshness-Modell kann beispielsweise so aussehen:

```
Freshness(t) = 2 ^ (-Δt / HalfLife)
```

Dabei ist:

- `Δt` der Abstand zur letzten relevanten Aktualisierung.
    
- `HalfLife` die konfigurierte Halbwertszeit.
    

Die Freshness-Gewichtung beeinflusst das Ranking.

Sie darf jedoch nicht automatisch historische Fakten löschen oder deren Gültigkeit beenden.

Ein zeitunabhängiger mathematischer Fakt benötigt beispielsweise keine solche Abwertung.

### Automatische Konflikterkennung

Die Engine sollte eine zusätzliche Claim-Conflict Engine erhalten.

Sie vergleicht neue Aussagen mit bestehenden Claims.

Dazu werden folgende Merkmale ausgewertet:

- Entity Identity
    
- Predicate
    
- Value
    
- Polarity
    
- Valid Time
    
- Source Identity
    
- Confidence
    
- Existing Conflict Groups
    

Beispiel:

```
Claim A:
Alice
WORKS_AT
Company A
Valid: 2024-01 bis 2025-01

Claim B:
Alice
WORKS_AT
Company B
Valid: ab 2025-01
```

Kein Widerspruch, da die Gültigkeitsintervalle nicht überlappen.

Anderes Beispiel:

```
Claim A:
Alice
LIVES_IN
Berlin
Valid: 2025-01 bis offen

Claim B:
Alice
LIVES_IN
Munich
Valid: 2025-02 bis offen
```

Wenn das Schema `LIVES_IN` als zeitlich exklusive Beziehung definiert, entsteht ein Konflikt.

### Conflict Groups

Statt eine Aussage sofort zu überschreiben, erzeugt die Engine eine Konfliktgruppe:

```
pub struct ConflictGroup {
    pub id: ConflictId,
    pub entity: EntityId,
    pub predicate: PredicateId,

    pub competing_claims: Vec<ClaimId>,

    pub status: ConflictStatus,
    pub resolution: Option<ClaimId>,
}
```

Die Konfliktlösung kann über Regeln, externe Verifikation oder menschliche Freigabe erfolgen.

Ein LLM darf einen Konflikt vorschlagen oder begründen.

Es darf nicht allein aufgrund seiner eigenen generierten Argumentation eine Aussage als bewiesen markieren.

---

## 5.5 Weitere Schutzmechanismen

Zusätzlich sollten folgende Mechanismen Bestandteil der ersten Produktgeneration sein:

**Source Trust Classification**

Originaldokumente, externe Webseiten, Agentenausgaben und manuell bestätigte Fakten erhalten unterschiedliche Herkunftsklassen.

**Prompt Injection Isolation**

Inhalte aus Dokumenten werden als untrusted data gespeichert. Die Datenbank behandelt sie niemals selbst als ausführbare Befehle.

**Idempotent Tool Operations**

Agenten-Tool-Aufrufe erhalten Operation-IDs. Ein wiederholter Request kann dadurch erkannt und vor einer doppelten Ausführung geschützt werden.

**Write Quotas**

Pro Agenten-Session können Grenzen für Speicherverbrauch, Branch-Größe und Schreiboperationen definiert werden.

**Conflict-aware Retrieval**

Widersprüchliche Aussagen können gemeinsam zurückgegeben werden, statt eine davon unbemerkt zu verwerfen.

**Evidence-first Retrieval**

Bei sensiblen oder besonders zuverlässigkeitskritischen Queries kann die Rückgabe von Informationen ohne nachvollziehbare Provenance verhindert werden.

---

# 6. Konkreter Rust-Technologie-Stack

## 6.1 Kerntechnologien

Die Architektur sollte möglichst viel der sicherheitskritischen Infrastruktur in Rust halten.

Empfohlene Technologien:

|Komponente|Empfehlung|Rolle|
|---|---|---|
|Sprache|Rust Stable|Gesamte Engine|
|Async Runtime|Tokio|API und asynchrones I/O|
|Optionales I/O|tokio-uring|Linux Storage Backend|
|Hashing|blake3|Content Addressing|
|Serialization|rkyv|Interne Archivierung|
|Columnar Processing|Apache Arrow|Batch Processing|
|LSM-Referenz|fjall|Storage-Prototyp / Baseline|
|B+Tree-Alternative|redb|Copy-on-Write-Benchmark|
|Textsuche|Tantivy|BM25-Referenz / Prototyp|
|ANN-Referenz|USearch|Performance-Benchmark|
|Atomic Snapshots|arc-swap|Manifest Publication|
|Memory Reclamation|crossbeam-epoch|Epoch GC|
|Concurrent Maps|papaya|Read-heavy Caches|
|Concurrency Testing|loom|Model Checking|
|API|tonic / axum|gRPC und HTTP|
|Telemetrie|tracing|Tracing und Diagnose|

## 6.2 Storage-Technologien

### fjall

Fjall ist ein in Rust implementierter LSM-Storage mit Thread-Safety, Keyspaces, Komprimierung und optionalen Transaktionsfunktionen.

Quelle: [https://docs.rs/fjall/latest/fjall/](https://docs.rs/fjall/latest/fjall/)

Ich würde Fjall als wichtige Referenz für den ersten Storage-Prototyp evaluieren.

Es ersetzt allerdings nicht die Entwicklung des eigenen Commit-, Manifest- und Merkle-Systems.

Insbesondere muss geprüft werden, welche Transaktionsgarantien die konkret gewählte Konfiguration über die gesamten benötigten Speicherbereiche hinweg bietet.

### redb

redb ist eine interessante Alternative für eine B+Tree-basierte Implementierung.

Die aktuelle Dokumentation beschreibt unter anderem:

- Copy-on-Write B+Trees
    
- ACID-Transaktionen
    
- MVCC
    
- parallele Leser
    
- Savepoints und Rollbacks
    

Quelle: [https://docs.rs/redb/latest/redb/](https://docs.rs/redb/latest/redb/)

Seine Stärken liegen in der Snapshot-orientierten Speicherung.

Für die angestrebte hohe Schreiblast und die Menge spezialisierter Indexsegmente würde ich dennoch zunächst einen LSM-orientierten Kern bevorzugen.

### Tantivy

Tantivy ist eine besonders interessante Rust-Komponente für die lexikalische Suche.

Sie kann als BM25-Referenz dienen und erlaubt die Evaluation unterschiedlicher Indexierungs- und Scoringstrategien.

Die Integration sollte jedoch so gestaltet werden, dass Tantivy keinen unabhängigen Commit-Zustand gegenüber der gemeinsamen Engine veröffentlicht.

### USearch

USearch stellt eine Rust-Anbindung für ANN bereit und unterstützt unter anderem mehrere Distanzmetriken und Vektortypen.

Quelle: [https://docs.rs/usearch/latest/usearch/](https://docs.rs/usearch/latest/usearch/)

Wichtig: Die Rust-Anbindung umschließt native USearch-Funktionalität. Wenn die Anforderung tatsächlich bedeutet, dass auch sämtliche ANN-Kernalgorithmen ausschließlich in Rust implementiert werden müssen, reicht die Verwendung dieses Crates allein nicht aus.

Es eignet sich hervorragend als Performance-Referenz, während eine native Rust-Implementierung der eigenen HNSW-Segmentarchitektur separat entwickelt werden kann.

## 6.3 Zero-Copy-Technologien

### rkyv

Für persistente interne Strukturen und schnelle Zugriffe.

### Apache Arrow

Für Batch-Verarbeitung, Query Execution und ML-Integration.

Beide Technologien sollten unterschiedliche Aufgaben erfüllen.

Es wäre nicht sinnvoll, sämtliche persistierten Daten pauschal als Arrow Tables abzulegen.

Die Storage Engine benötigt weiterhin optimierte Key-Value- und Indexstrukturen.

## 6.4 Concurrency-Technologien

`arc-swap` eignet sich zur konsistenten Veröffentlichung read-mostly Manifest-Strukturen.

`crossbeam-epoch` unterstützt sichere Speicherfreigabe bei lock-free Datenstrukturen.

`papaya` kann als hochperformanter Concurrent Cache verwendet werden.

Die Verwendung von Atomics muss mit expliziten Memory Ordering Regeln erfolgen. Rust stellt dafür unter anderem Acquire-, Release- und AcqRel-Semantik bereit.

Quelle: [https://doc.rust-lang.org/stable/std/sync/atomic/enum.Ordering.html](https://doc.rust-lang.org/stable/std/sync/atomic/enum.Ordering.html)

Für die Überprüfung kritischer Nebenläufigkeitslogik sollte Loom eingesetzt werden.

Dabei ist zu beachten, dass Loom nur Operationen kontrollieren kann, die tatsächlich über seine entsprechenden Ersatztypen instrumentiert sind.

Quelle: [https://docs.rs/loom/latest/loom/](https://docs.rs/loom/latest/loom/)

---

# 7. Architektur-Trade-offs und technische Risiken

## 7.1 Write Amplification

Das größte klassische Risiko einer LSM-Architektur ist Write Amplification.

Daten müssen bei Kompaktierungen erneut geschrieben werden.

Mit mehreren Indexarten kann zusätzlicher Schreibaufwand entstehen.

Gegenmaßnahmen:

- Größere immutable Segmente
    
- Group Commit
    
- Tiered Compaction für schreibintensive Ebenen
    
- Leveled Compaction für read-intensive Ebenen
    
- Delta-Indizes für kleine Änderungen
    
- Kompaktierung anhand von Änderungsrate und Read Amplification
    

Die Compaction Engine darf nicht unabhängig von der MVCC- und Snapshot-Verwaltung arbeiten.

Sie muss alte Segmente so lange aufbewahren, wie historische Snapshots diese benötigen.

## 7.2 RAM-Verbrauch versus Latenz

HNSW kann sehr schnelle Abfragen ermöglichen, benötigt jedoch Speicher für seine Indexstrukturen.

Mit großen Datenbeständen wird der Speicherbedarf ein zentrales Kostenkriterium.

Strategie:

- Kleine Datensätze: vollständig im RAM.
    
- Mittlere Datensätze: gecachte HNSW-Segmente.
    
- Große Datensätze: SSD-basierte ANN-Indizes.
    
- Ältere oder selten genutzte Daten: Cold Storage.
    

Der DiskANN-Ansatz ist eine wichtige Referenz für diese Skalierungsstufe.

## 7.3 Merge-Konflikte

Graphänderungen lassen sich nicht immer automatisch zusammenführen.

Zwei Agenten können beispielsweise dieselbe Beziehung unterschiedlich interpretieren.

Ein Property-Update lässt sich unter Umständen automatisch mergen.

Ein Konflikt zwischen semantisch unterschiedlichen Claims muss dagegen erhalten bleiben.

Für Embeddings gilt:

Die Datenbank darf zwei inkompatible Embedding-Repräsentationen nicht einfach zusammenführen.

Stattdessen muss die Engine die zugrunde liegenden Inhalte und Modellversionen prüfen.

## 7.4 Index-Konsistenz versus Performance

Ein vollständig synchroner Neuaufbau großer Indexsegmente bei jedem Commit wäre ineffizient.

Ein vollständig asynchrones Indexing könnte dagegen zu veralteten Suchergebnissen führen.

Die Lösung ist die Trennung zwischen:

- synchron erzeugten, unmittelbar suchbaren Indexdeltas
    
- asynchron kompaktierten optimierten Indexsegmenten
    

Jeder Commit ist sofort über seinen Delta-Zustand sichtbar.

Kompaktierung verändert lediglich die physische Repräsentation und nicht den logischen Datenbankzustand.

## 7.5 Merkle-DAG versus Speicherverbrauch

Unveränderliche Historien sind vorteilhaft, können jedoch den Speicherverbrauch erhöhen.

Gegenmaßnahmen:

- Content Deduplication
    
- Referenzzählung beziehungsweise Reachability GC
    
- Komprimierte Packfiles
    
- Snapshot Retention Policies
    
- Archivierung alter Commits
    
- Tenant-spezifische Speicherbereinigung
    

Eine globale Deduplizierung über verschiedene Tenants hinweg sollte aus Sicherheits- und Datenschutzgründen zunächst vermieden werden.

## 7.6 Komplexität der eigenen Engine

Die Kombination aus Storage, MVCC, Git-artiger Versionierung und drei Suchmodellen ist ein erhebliches Engineering-Projekt.

Besonders kritisch sind:

- Crash Consistency
    
- Branch-Isolation
    
- Index-Rebuilds
    
- Snapshot Lifetime Management
    
- Predicate-Konflikte
    
- Merge-Semantik
    
- Recovery nach unvollständiger Kompaktierung
    

Es wäre ein Fehler, zunächst alle Features parallel zu entwickeln.

Der Storage-Kern und seine Korrektheit müssen Vorrang haben.

---

# 8. API- und Funktionsspezifikation

## 8.1 Storage API

Die Rust-API soll die Transaktionsgrenzen explizit machen.

Beispiel:

```
let mut tx = db.begin_write(
    Branch::main(),
    Isolation::Snapshot
)?;

let entity = tx.create_entity(...)?;

let chunk = tx.create_chunk(...)?;

tx.attach_embedding(
    chunk.id(),
    embedding
)?;

tx.create_edge(
    entity.id(),
    chunk.id(),
    Relation::References
)?;

tx.commit()?;
```

Die Engine erzeugt einen gemeinsamen Commit.

Kein Teil der Änderung wird vor diesem Commit global sichtbar.

## 8.2 Branch API

```
db.branch(
    "agent-session-42",
    From::Branch("main")
)?;

db.merge(
    "agent-session-42",
    "main",
    MergeStrategy::ThreeWay
)?;

db.diff(
    CommitRef::Branch("main"),
    CommitRef::Branch("agent-session-42")
)?;
```

## 8.3 Time-Travel API

```
let snapshot = db.snapshot(
    Branch::main(),
    CommitRef::Timestamp(timestamp)
)?;

let result = snapshot.query(...)?;
```

## 8.4 Provenance API

```
tx.create_claim(
    Claim {
        subject,
        predicate,
        object,
        provenance,
        confidence,
        temporal
    }
)?;
```

Die Engine erstellt die benötigten Herkunfts- und Referenzstrukturen.

## 8.5 Agent API

```
let session = db.create_agent_session(
    AgentConfig {
        branch_from: "main",
        require_provenance: true,
        ttl_seconds: 3600,
        max_operations: 10_000,
    }
)?;
```

Die Session arbeitet ausschließlich mit ihren zugewiesenen Berechtigungen.

---

# 9. Betriebsmodell und Observability

Die Datenbank muss ihre interne Arbeit transparent darstellen können.

Wichtige Systemmetriken:

**Storage**

- WAL Throughput
    
- WAL Sync Latency
    
- LSM Read/Write Amplification
    
- Compaction Backlog
    
- Storage Space by Segment Type
    

**Concurrency**

- Active Transactions
    
- Snapshot Age
    
- Branch Head Conflicts
    
- Commit Latency
    
- Recovery Time
    

**Vector**

- Recall@K
    
- ANN Search Latency
    
- Candidate Count
    
- Index Cache Hit Rate
    
- Delta Index Size
    

**Graph**

- Visited Nodes
    
- Traversal Depth
    
- Path Expansion
    
- Adjacency Cache Hit Rate
    

**Text**

- BM25 Latency
    
- Posting Blocks Read
    
- Term Statistics Cost
    
- Candidate Count
    

**Agenten und RAG**

- Token Budget Utilization
    
- Provenance Coverage
    
- Conflict Rate
    
- Evidence Density
    
- Redundancy Removal Rate
    
- Anteil veralteter Summaries
    

Für jede Query sollte ein Explain Plan erzeugt werden können.

Beispiel:

```
QUERY PLAN

Snapshot: main@commit-abc

1. Tenant Filter
2. Graph Traversal
   Depth: 2
   Candidates: 154

3. Vector Search
   Index: HNSW
   Candidate Pool: 200

4. Text Search
   Ranking: BM25
   Candidate Pool: 200

5. Fusion
   Method: RRF
   K: 60

6. Provenance Verification
7. Token Budget Selection

Estimated Tokens: 3,600
Maximum Tokens: 4,000
```

---

# 10. Entwicklungsphasen und Abnahmekriterien

## Phase 1 – Storage Foundation

Implementierung:

- WAL
    
- LSM-Engine
    
- Manifest Manager
    
- Immutable Object Store
    
- MVCC
    
- Snapshot Reader
    
- Crash Recovery
    

Abnahmekriterien:

- Kein verlorener bestätigter Commit.
    
- Keine teilweise sichtbaren Transaktionen.
    
- Korrekte Wiederherstellung nach simulierten Abstürzen.
    
- Parallele Reader sehen konsistente Snapshots.
    

## Phase 2 – Multi-Model Indexing

Implementierung:

- Canonical Object Model
    
- Text Index
    
- Graph Index
    
- Vector Delta Index
    
- HNSW Segments
    
- Gemeinsame Visibility Engine
    

Abnahmekriterien:

- Atomare Änderungen aller drei Indexarten.
    
- Keine verwaisten sichtbaren Indexreferenzen.
    
- Keine Modellinkonsistenzen bei Snapshot Queries.
    
- Index-Rebuild aus kanonischen Daten möglich.
    

## Phase 3 – Git-artige Versionierung

Implementierung:

- Commit DAG
    
- Content Addressing
    
- Branches
    
- Merge
    
- Diff
    
- Rollback
    
- Time Travel
    
- Garbage Collection
    

Abnahmekriterien:

- Zero-Copy-Branching.
    
- Historische Query nach Commit.
    
- Merge bei konfliktfreien Änderungen.
    
- Korrekte Konflikterkennung.
    
- Kein Löschen noch referenzierter Segmente.
    

## Phase 4 – Hybrid Query Engine

Implementierung:

- Query Compiler
    
- Cost-Based Planner
    
- Graph-gefilterte ANN
    
- BM25
    
- RRF
    
- Snapshot-konforme Sichtbarkeit
    

Abnahmekriterien:

- Wiederholbare Queries auf demselben Snapshot.
    
- Korrekter Filter-Pushdown.
    
- Vergleichbare Qualität mit reinen Spezialindizes als Referenz.
    
- Explain Plan für hybride Queries.
    

## Phase 5 – Cognitive Features

Implementierung:

- Provenance DAG
    
- Source Hashes
    
- Confidence Model
    
- Token-aware Planner
    
- Summary Hierarchy
    
- Agent Session Branches
    
- Temporal Conflict Detection
    

Abnahmekriterien:

- Herkunft jedes ausgegebenen Evidence-Objekts nachvollziehbar.
    
- Keine unkontrollierte Agenten-Promotion.
    
- Budgetkonforme Evidence Bundles.
    
- Widersprüchliche Fakten werden nicht stillschweigend überschrieben.
    
- Zeitbezogene historische Abfragen funktionieren.
    

## Phase 6 – Performance und Skalierung

Implementierung:

- Advanced Compaction
    
- Adaptive Cache
    
- io_uring Backend
    
- SSD-basierte ANN-Segmente
    
- Bulk Ingestion
    
- Replikationsvorbereitung
    

Performancewerte müssen anhand definierter Benchmarks ermittelt werden, statt vor der Implementierung als garantierte Eigenschaften vorausgesetzt zu werden.

---

# 11. Zentrale technische Designentscheidungen

Die wichtigsten finalen Architekturentscheidungen lauten:

|Problem|Entscheidung|
|---|---|
|Einheitliche Speicherung|Ein kanonischer Object Store|
|Primärer Storage|LSM-basierter Kern|
|Sekundäre Indizes|Immutable Vector-, Graph- und Textsegmente|
|Transaktionsmodell|Gemeinsames MVCC|
|Query-Konsistenz|Ein Snapshot für alle Modelle|
|Commit-Historie|Content-addressed Merkle-DAG|
|Branching|Referenzbasierte immutable Roots|
|ANN|HNSW, später DiskANN-artige SSD-Variante|
|Graph|Immutable Adjacency Segments / CSR|
|Volltext|Inverted Index mit BM25|
|Hybrid Ranking|RRF plus kontrolliertes Re-Ranking|
|Durability|WAL und Manifest-basierte Wiederherstellung|
|Concurrency|Immutable Reads und koordinierte Commits|
|Agenten-Isolation|Ephemere Session Branches|
|Halluzinationsschutz|Kryptografische Provenance|
|Token-Optimierung|Token-aware Cost-Based Planner|
|Veraltetes Wissen|Bitemporale Speicherung und Freshness Scoring|
|Konflikte|Explizite Conflict Groups|

# 12. Abschließende Architekturthese

Die wesentliche Innovation von UCD ist nicht die bloße Kombination von HNSW, BM25 und Graph Traversal.

Diese Algorithmen sind einzeln bereits bekannt.

Die Innovation liegt in ihrer gemeinsamen transaktionalen, versionierten und provenance-bewussten Verwaltung.

Ein Agent soll nicht nur fragen können:

„Welche Informationen passen semantisch zu meiner Frage?“

Er soll ebenso fragen können:

- In welcher Version wurde diese Information gespeichert?
    
- Aus welcher Originalquelle stammt sie?
    
- Welche anderen Aussagen widersprechen ihr?
    
- Welche Entitäten und Ereignisse hängen damit zusammen?
    
- Welche Informationen waren zu einem bestimmten Zeitpunkt gültig?
    
- Welche Teile stammen aus einer nicht validierten Agenten-Session?
    
- Welche Belege passen in mein verfügbares Context Window?
    

Eine Datenbank, die diese Fragen nativ beantworten kann, ist nicht mehr nur eine Speicherkomponente eines RAG-Systems.

Sie bildet eine versionierte, überprüfbare und transaktionale Wissensinfrastruktur für autonome KI-Systeme.

**Die wichtigste Architekturregel lautet deshalb:**

Jede Information, jeder Index, jede Beziehung, jede Zusammenfassung und jede Agentenänderung muss auf einen eindeutig identifizierbaren und historisch rekonstruierbaren Wissenszustand zurückführbar sein.

Das ist die Grundlage für eine Cognitive Database, die nicht nur Informationen speichert, sondern auch deren Entstehung, Gültigkeit, Abhängigkeiten und Vertrauenswürdigkeit technisch nachvollziehbar macht.

## **Meine abschließende technische Bewertung**

Drei Entscheidungen halte ich für besonders wichtig:

1. **Immutable Multi-Index Segments statt unabhängig schreibender Engines:** Das ist der Schlüssel zur atomaren Konsistenz zwischen Vektor-, Graph- und Textsuche.
    
2. **Merkle-DAG und MVCC von Anfang an:** Branching nachträglich auf ein klassisches Datenbankmodell aufzusetzen wäre erheblich schwieriger.
    
3. **Provenance und Agenten-Branches als Datenbankprimitive:** Das ist der größte funktionale Unterschied gegenüber einer gewöhnlichen hybriden Vektordatenbank.
    

Die größte technische Herausforderung ist nicht die Implementierung von HNSW oder BM25. Es ist die korrekte Kombination aus Crash-Recovery, historischen Snapshots, Branch-Merges, Index-Kompaktierung und parallelen Transaktionen.

**Meine Empfehlung:** Zuerst den Storage-Kern, den gemeinsamen Commit-Manager und das MVCC-Modell implementieren. Erst darauf die drei Index-Engines aufbauen. Dadurch entsteht ein konsistentes Produktfundament statt einer Sammlung nachträglich verbundener Suchsysteme.

---
