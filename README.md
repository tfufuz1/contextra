# <p align="center">CONTEXTRA</p>
<p align="center"><b>Local Knowledge & Memory Engine in Rust</b></p>

<img width="3168" height="1344" alt="banner" src="https://github.com/user-attachments/assets/443fa9d1-7b8f-4ffe-aa61-46e85f5a414a" />

---

## Contextra Komponenten

### 1. Domäne: Hybrid Retrieval & Suche
<img width="2752" height="1536" alt="hybrid-retrieval" src="https://github.com/user-attachments/assets/0c24b7b2-7037-42f2-98a0-b439c33c83c7" />
Diese Domäne kombiniert semantische Vektorsuche, klassische Volltextsuche, Graph-Traversierung und Ergebnisse-Rangfusion zu Such- und RAG-Systemen.

* **Vektorsuche (`contextra-vector`, `contextra-simd`)**:
  * **HNSW (Hierarchical Navigable Small World)**: Graph-basierter Algorithmus zur approximativen Nächste-Nachbarn-Suche (ANN) mit mehrschichtiger Graph-Hierarchie.
  * **SQ8 (Skalare Quantisierung)**: 8-Bit-Quantisierung zur Komprimierung von Vektor-Embeddings im Arbeitsspeicher.
  * **RaBitQ & DiskANN**: Out-of-Core-Graphindex (DiskANN) für große Vektormengen sowie binaure Quantisierungsverfahren (RaBitQ).
  * **Prädikat-augmentierte Vektorsuche (Acorn)**: Filtergestützte Vektorsuchpfade.
  * **SIMD-Distanzberechnungen**: Hardware-beschleunigte Kerne (AVX2, AVX-512, ARM NEON) und skalare Fallbacks zur Berechnung von Kosinus-Distanz (Cosine), Euklidischer Distanz (\\(L_2\\)) und Skalarprodukt (Dot Product).

* **Volltextsuche (`contextra-text`)**:
  * **BM25 Scoring**: Probabilistisches Ranking-Modell zur Relevanzbewertung von Textdokumenten basierend auf Termfrequenz und inverser Dokumentfrequenz.
  * **Block-Max WAND (Weak AND)**: Dynamischer Pruning-Algorithmus zur vorzeitigen Beendigung der Auswertung unkritischer Posting-Listen-Blöcke.
  * **Deutscher Morphologie-Algorithmus**: Regelbasierte Zerlegung von Komposita (Compound Splitting), Umlaut-Normalisierung und Stoppwort-Filterung.
  * **Inverted Index mit CoW-Snapshots**: Invertierter Index mit Copy-on-Write Snapshot-Isolation für lock-freie Lesezugriffe.

* **Graph-Suche & GraphRAG (`contextra-graph`)**:
  * **Personalized PageRank (PPR)**: Forward-Push PPR-Algorithmus (Andersen-Chung-Lang) für zielgerichtete, lokal gewichtete Graph-Traversierung.
  * **CSR (Compressed Sparse Row)**: Speichereffiziente Datenstruktur zur Repräsentation dünnbesetzter Adjazenzmatrizen.
  * **K-Path Diffusion & APPRH**: K-Pfad-Traversierung, Pfad-Diffusion sowie adaptiver PageRank mit Hyperkanten-Expansion.
  * **GraphRAG Community Detection**: Community-Erkennung, Erkennung von Perkolationsschwellen und dynamisches Kanten-Reinforcement-Learning.

* **Rangfusion & Kalibrierung (`contextra-rank`)**:
  * **Weighted RRF (Weighted Reciprocal Rank Fusion)**: Multi-Signal-Rangfusion mit konfigurierbarer Signal-Gewichtung und Standardglättung (\\(k=60\\)).
  * **mRRF (Margin-modulierte RRF)**: Rangfusion mit randabstandsbasierter Gewichtsadaption.
  * **Resonanz-Bonus**: Score-Bonus bei zeitgleicher Koinzidenz von Treffern über mehrere Retrieval-Kanäle hinweg.
  * **Isotonic Regression (PAVA)**: Nicht-parametrische Wahrscheinlichkeitskalibrierung von Konfidenz-Scores via Pool-Adjacent Violators Algorithm.
  * **Platt Scaling**: Parametrische logistische Regression zur Score-Kalibrierung.
  * **Conformal Prediction**: Verfahren für mathematisch garantierte Konfidenzintervalle auf finiten Stichproben.
  * **Drift Detection**: Statistisches Verfahren zur Überwachung von Score-Verteilungen zur Laufzeit.


Kombiniert semantische Vektorsuche (HNSW, SQ8, SIMD) mit klassischer Volltextsuche (BM25) und Rangfusion (RRF).

---
### 2. Domäne: Autonome Agenten & Kognition (`agentic`)
<img width="2752" height="1536" alt="agentic" src="https://github.com/user-attachments/assets/87780e2a-54bb-492f-817b-df9f0d67c1c3" />
Diese Domäne stellt die Execution-Engine für mehrstufige Agenten-Prozesse und die langfristige Verwaltung und Verdichtung des Gedächtnisses bereit.

* **Agenten-Engine (`contextra-agent`)**:
  * **StateGraph Exec-Schleife**: Deterministischer Zustandsautomat entlang des Zyklus `checkpoint` → `execute` → `commit` → `audit`.
  * **RAII Auto-Checkpointing & Rollback**: Automatisches Erzeugen von Zustands-Snapshots vor Handlungsschritten mit deterministischem Rollback im Fehlerfall.
  * **Token-Budget-Steuerung (`TokenBudget`)**: Strikte Begrenzung von Token-Verbrauch und Ausführungsschritten zur Schleifen- und Kostenkontrolle.

* **Langzeitgedächtnis & Kognition (`contextra-cognition`)**:
  * **Memory Consolidation (Structural Pass)**: Erkennung von Beinahe-Duplikaten (*Detect Near Duplicates*), Schiebefenster-Clustering (*Sliding Window*) und Strukturanalyse.
  * **Context Compaction**: Transaktions- und Dialogverdichtung unter Verwendung von *Truncate*-, *Summary*- und *Selective*-Strategien.
  * **LeanRAG Stage 3 (Semantische Aggregation)**: Diagonal-GMM (Gaussian Mixture Model) Clustering, Typ-Denoising, Alpha-Knoten-Synthese und Generierung von Super-Hyperkanten.
  * **Transitivity Veto**: Schutzalgorithmus zur Verhinderung fehlerhafter transitiver Verschmelzungen von Erinnerungsnetzwerken.

---
### 3. Domäne: Adaptives Routing & Modellsteuerung (`llm`)
<img width="2752" height="1536" alt="routing" src="https://github.com/user-attachments/assets/d792ecc5-2744-4010-b258-f9011030ad81" />
Steuert die Auswahl, Auslastung und Parametrierung verschiedener KI-Modelle (Candidate Pools, SLMs, Gateways).

* **Bandit-Routing & Regler (`contextra-adapt`, `contextra-router`)**:
  * **LinUCB Contextual Bandits**: Kontextuelle Bandit-Auswahl mit Sherman-Morrison-Matrixaktualisierung und Sketched Projection.
  * **Flow-Corrected Thompson Sampling (FC-TS)**: Bayessches Routing mit automatischer Kompensation von Performance-Drift.
  * **RIE-Greedy**: Regularisierungsinduzierte Exploration für personalisiertes Modell-Routing.
  * **PID-Latenzregler (`PidController`)**: Proportional-Integral-Derivative-Regelkreis zur dynamischen Anpassung der Candidate-Pool-Größen basierend auf Ziel-Latenzen.
  * **Homeostat & Decay-Controller**: Exponentielles Abklingen (Half-Life) und Regelung des Gleichgewichts von Modell-Pools.

---
### 4. Domäne: Datenschutz, Sicherheit & Compliance (`safety` & `compliance`)
<img width="2752" height="1536" alt="security" src="https://github.com/user-attachments/assets/2c02271c-171c-4256-92fc-d43a4b98623d" />
Gewährleistet den Schutz vor Datenabfluss, Injection-Angriffen und die Einhaltung von Datenschutzvorgaben.

* **Egress-Schutz & DLP (`contextra-privacy`, `contextra-prompt-guard`)**:
  * **Rule-Based Entity Recognition & DLP**: Regel- und gazetteerbasierte Erkennung personenbezogener Daten (PII wie IBAN, E-Mail).
  * **Surrogate Vaulting**: Ersetzung sensibler Daten durch deterministische Surrogat-Token vor der Modellweiterleitung und spätere Antwort-Rehydrierung.
  * **k-NN Egress Guard**: Vektorielle Ähnlichkeitsprüfung von Egress-Payloads gegen geschützte Sperrinhalte.
  * **Bulk Exfiltration Detector**: Sliding-Window-Rate-Limiting zur Erkennung ungewöhnlicher Datenabflussvolumina.
  * **Prompt-Injection Detection & Audit**: Sicherheitsschleife zur Erkennung bösartiger Systemanweisungen.
  * **Context Edit Audit**: BLAKE3-kryptografisch verkettetes Audit-Protokoll.

* **Compliance & Löschung (`contextra-erasure`, `contextra-crypto`, `contextra-vault`)**:
  * **Cryptographic Shredding**: Unumkehrbares Vernichten kryptografischer Schlüssel (AES-256-GCM-SIV, KDF via Argon2id / HKDF) für punktuelles Löschen einzelner Dokumente ohne Re-Indexing.
  * **Ed25519 Deletion Proofs**: Erzeugung digital signierter Löschnachweise.
  * **RAM-Locking & Vaulting (`contextra-sys`, `contextra-vault`)**: `mlock` / `VirtualLock` zur Verhinderung von Speicher-Swapping auf die Festplatte sowie `ZeroizeOnDrop` zum sicheren Löschen flüchtiger Daten im Arbeitsspeicher.


---

### 5. Domäne: Speichersystem, Transaktionen & Isolation (`storage` & `foundation`)
<img width="2752" height="1536" alt="storage" src="https://github.com/user-attachments/assets/4ac5cdb6-b731-4653-af1c-4435c670a7fc" />
Stellt die fundamentale Persistenz, ACID-Garantien und isolierte Laufzeitumgebungen bereit.

* **LSM-Tree Speicherkern (`contextra-store`)**:
  * **LSM-Tree (Log-Structured Merge-Tree)**: MemTable, unveränderliche SSTables (V4) mit hierarchischer Hintergrund-Kompaktierung.
  * **Zweistufige Bloom-Filter**: Probabilistische Datenstruktur zur Abfrage von Schlüssel-Existenzen ohne Festplattenzugriff.
  * **Write-Ahead-Log (WAL)**: Crash-konsistentes Append-Only Log mit kryptografischen HMAC-Ketten.

* **Transaktions-Isolation (`contextra-mvcc`)**:
  * **SSI (Serializable Snapshot Isolation)**: Transaktionsvalidierung zur Vermeidung von Write-Skew und Anti-Dependencies.
  * **SequenceLog & ReadSet-Prüfung**: Erfassung aller Lese- und Schreibmengen zur Konflikterkennung vor dem Commit.

* **Ausführungs-Isolation (`contextra-sandbox`)**:
  * **WASM Fuel-Budgeting**: Rechenschritt-Zählung (*Fuel*) zur zeitlichen und ressourcenmäßigen Begrenzung isolierter WASM-Tools.

---
### 6. Domäne: Ingest, Inferenz & Anwendungen (`data`, `llm` & `apps`)
<img width="2752" height="1536" alt="ingest" src="https://github.com/user-attachments/assets/18e529ac-5458-4d62-8dcb-48c962df873c" />
* **Dokumenten-Ingest (`contextra-ingest`)**:
  * **Prose & Structure Chunking**: Formatbewusste Extraktion (PDF, DOCX, Markdown) und deterministisches semantisches Text-Chunking.
* **Inferenz-Anbindung (`contextra-infer-candle`, `contextra-infer-onnx`, `contextra-infer-ollama`)**:
  * **GGUF-Metadaten-Parsing & In-Process Inferenz**: On-Device Inferenz mit Candle/ONNX Runtime sowie REST-Circuit-Breaker für Ollama.
* **Model Context Protocol (`contextra-mcp`)**:
  * **JSON-RPC 2.0 / STDIO Protocol Server**: Zustandsloser Protocol-Handler für IDEs und Agenten-Umgebungen.
