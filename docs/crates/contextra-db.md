# contextra-db — Crate-Spezifikation (Integrationsmotor)

**Ebene:** E2 (Integration & Orchestrierung)  
**Kern-Closure:** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-checkpoint`  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-db`  
**Bindung:** `docs/SPEC.md` §2, §3 (Datenklassifikation), §4 (Schlüsselhierarchie), §5 (Löschkette), §6 (ReadGate), §7 (ErasureLayer), §8 (Lineage), §9 (Zeitreise), §10 (Barrieren), §11 (Fehlerverhalten), §13 (Invarianten), §14 (Konfiguration)

---

## 1. Zweck & Systemrolle

`contextra-db` ist der zentrale Integrationsmotor und das Herzstück des Contextra-Systems. Als Ebene-E2-Komponente bündelt es alle autonomen Domänenbausteine (LSM-Store, HNSW-Vektorindex, BM25-Volltextindex, CSR-Graph, RRF-Ranking, Checkpointing, MVCC) zu einer einheitlichen, transaktionalen und bitemporalen Datenbank-Engine.

Hauptaufgaben:
1. **Multi-Index-Orchestrierung:** Parallele Koordination von LSM-Storage (`contextra-store`), Vektorindex (`contextra-vector`), Volltextsuche (`contextra-text`) und Wissensgraph (`contextra-graph`).
2. **Hybrid-Search-Pipeline:** 4-Signal-Suche mit automatischer Relevanz-Fusion (`contextra-rank`) und Auffüllen auf $k$.
3. **Transaktionskoordination:** Atomare Multi-Index-Schreibvorgänge mit Two-Phase-Commit (2PC), Staging-Puffern (`TxBuffer`) und Intent-Logging.
4. **DSGVO-Löschketten-Integration:** Initiiert Lösch-Sagas über `ErasureCore` und erzwingt das `ReadGate` synchron an ausnahmslos jedem Lesepfad.
5. **Bitemporale Zeitreise:** Unabhängige Filterung nach Transaktionszeit (`as_of`) und Gültigkeitszeit (`valid_at`).
6. **Mandantenisolation:** Strikte Durchsetzung von `TenantPolicy` auf Collection- und Storage-Ebene.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                contextra-db                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                              Contextra                              │   │
│   │                                                                     │   │
│   │   ┌──────────────────────┐             ┌────────────────────────┐   │   │
│   │   │    CollectionMap     │             │  TenantCollectionMap   │   │   │
│   │   │  (Default Namespace) │             │  (Isolated Per Tenant) │   │   │
│   │   └──────────┬───────────┘             └───────────┬────────────┘   │   │
│   │              │                                     │                │   │
│   │              ▼                                     ▼                │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │                         Collection                          │   │   │
│   │   │                                                             │   │   │
│   │   │   ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐   │   │
│   │   │   │  LsmStorage  │ │  HnswIndex   │ │   InvertedIndex   │   │   │
│   │   │   │  (Store-L1)  │ │ (Vector-L1)  │ │     (Text-L1)     │   │   │
│   │   │   └──────────────┘ └──────────────┘ └───────────────────┘   │   │
│   │   │   ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐   │   │
│   │   │   │   CsrGraph   │ │  Checkpoint  │ │    KvLockTable    │   │   │
│   │   │   │  (Graph-L1)  │ │  (Backup-L1) │ │ (Per-Key Mutexes) │   │   │
│   │   │   └──────────────┘ └──────────────┘ └───────────────────┘   │   │
│   │   └─────────────────────────────────────────────────────────────┘   │   │
│   │                                                                     │   │
│   │   ┌──────────────────────┐             ┌────────────────────────┐   │   │
│   │   │ MultiStepEngine      │             │ PidLatencyController   │   │   │
│   │   │ (Query Rewriting)    │             │ (Dynamic Target Lat.)  │   │   │
│   │   └──────────────────────┘             └────────────────────────┘   │   │
│   │   ┌──────────────────────┐             ┌────────────────────────┐   │   │
│   │   │ BackgroundWorkers    │             │ AdaptiveDecayController│   │   │
│   │   │ (Orphan/Expiry/Decay)│             │ (Relevance Decay)      │   │   │
│   │   └──────────────────────┘             └────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Datenklassifikation & Schreibpfad (SPEC §3)

### 3.1 Schreibpfad-Invarianten

Jeder Schreibvorgang unterliegt der konfigurierten `DataClassPolicy`:
- **`DataClassPolicy::Enforced`:** Jeder Schreibaufruf deklariert zwingend seine Datenklasse (`K`, `P`, `D`, `E`).
- **Klasse K (Unternehmenswissen):** Schreiboperationen (`insert`, `update`) akzeptieren für K-Daten nur `Validated<T>` (`contextra-privacy`). Bei Fund unerkannter PII bricht die Operation vor dem Schreiben ab (`Err(ContextraError::PersonalDataFound)`).
- **Klasse P (Personenbezogene Daten):** Adressierung erfolgt strikt über `SubjectRef` oder `PersonalRef`. Verschlüsselung erfolgt mit individuellem `SubjectDocDEK`, abgeleitet aus `SubjectKey(subject_ref)`.
- **Klasse D (Abgeleitete Daten):** Vektoren, Postings und Graph-Knoten tragen zwingend `LineageRef` auf ihr Quelldatum.
- **Klartext-Verbot:** Nutzlasten werden vor der Übergabe an das LSM-Write-Ahead-Log (WAL) durch den `RecordSealer` versiegelt (`AES-256-GCM-SIV`). Kein unverschlüsselter Payload erreicht persistente Schichten.

### 3.2 Transaktionsablauf (2-Phase-Commit)

1. **Staging:** Mutationen werden im `TxBuffer` (`contextra-mvcc`) gepuffert. Per-Key-Locks im `KvLockTable` verhindern TOCTOU-Kollisionen.
2. **SSI-Validierung:** Vor dem Commit prüft der `SsiValidator` das `ReadSet` gegen parallele Schreibvorgänge (Anti-Dependencies, Write-Skew-Schutz).
3. **CommitBarrier:** Erwerb des `CommitBarrier::shared()` Permits.
4. **Multi-Index-Commit:**
   - WAL-Eintrag synchron schreiben (`fsync` via `contextra-durable-fs`)
   - MemTable-Aktualisierung
   - HNSW-Graph-Knoten einfügen
   - BM25-Posting-Listen aktualisieren
   - Graph-Kanten und Entitäten eintragen
5. **Freigabe:** Shard-Locks und Per-Key-Locks werden deterministisch freigegeben.

---

## 4. Hybrid-Suche & Relevanz-Orchestrierung

### 4.1 Die 4 Signale

`HybridQueryBuilder` führt Suchen über bis zu 4 Kanäle aus:
1. **Vektorsignal (Dense):** HNSW ANN-Suche via `contextra-vector` mit SIMD-Distanzberechnung.
2. **Volltextsignal (Sparse):** BM25 / BM25F Abfrage via `contextra-text` mit Block-Max-WAND Pruning.
3. **Graphsignal (Strukturell):** Personalized PageRank (PPR) oder BFS-Traversierung via `contextra-graph`.
4. **Rerank-Signal (Präzision):** Cross-Encoder Nachbewertung oder Homeostat-gesteuerter Rerank-Pass.

### 4.2 Fusion & Relevanzgarantien

- **Fusion:** Zusammenführung der Signal-Trefferlisten über `contextra-rank` mittels `rrf_fuse` (Reciprocal Rank Fusion) oder gewichteter Normalisierung.
- **Tie-Breaking:** Bei identischem Score erfolgt der Tie-Break zwingend nach `DocId` aufsteigend (INV-5 Determinismus).
- **Auffüllen auf $k$ (SPEC §6.2):** Werden Kandidaten durch das `ReadGate` oder temporale Gültigkeitsfilter verworfen, zieht die Engine aus den vorgeschalteten Kanälen weitere Treffer nach, bis die gewünschte Anzahl $k$ erreicht ist oder alle Kanäle erschöpft sind.

### 4.3 Multi-Step Retrieval

Über `MultiStepEngine` können komplexe Anfragen mittels injiziertem `QueryRewriter` in mehrere Teilabfragen zerlegt, sequentiell/parallel ausgeführt und aggregiert werden.

---

## 5. Durchsetzung der Lesesperre (ReadGate — SPEC §6)

`contextra-db` ist die primäre Durchsetzungsstelle für die synchrone Lesesperre. **Kein Lesepfad darf Daten ohne vorherige Prüfung gegen `ReadGate` ausliefern:**

| Lesepfad | Prüfstelle | Verhalten bei `Erased` |
|----------|------------|------------------------|
| `get(id)` | Vor LSM-Lookup und vor Entschlüsselung | Liefert `Ok(None)` — nicht unterscheidbar von nie existent |
| `get_at_snapshot(id, seq)` | Vor Entschlüsselung | Liefert `Ok(None)` (Z-T5: ReadGate gilt immer im aktuellen Stand) |
| `scan_prefix(prefix)` | Pro Eintrag vor Aufnahme in Ergebnis | Eintrag wird lautlos übersprungen |
| `search(&query, k)` | Auf Kandidatenebene vor Fusion/Rerank | Kandidat wird verworfen; Auffüllen auf $k$ aus Folgekandidaten |
| `hybrid_search(params)` | Auf Kandidatenebene vor Fusion | Kandidat wird verworfen; Auffüllen auf $k$ |
| Graph-Traversierung | Je Kante und Zielentität | Verwirft gesperrte Kanten/Knoten lautlos |
| `export_collection` | Pro exportiertem Dokument | Gesperrte Dokumente werden ausgelassen, im Report nur gezählt |

Verhalten bei `GateDecision::Restricted` (Art. 18 DSGVO):
- Zugriff wird mit `Err(ContextraError::RestrictedReadAuditRequired)` verweigert, es sei denn, ein valider Audit-Kontext liegt vor.

---

## 6. Teilnahme an der Löschkette (SPEC §5)

### 6.1 Lösch-Operationen

- **`erase_document(doc_id, attestation)`:** Initiiert Dokumenten-Lösch-Saga (Klasse K oder Einzeldokument P).
- **`erase_subject(subject_ref, attestation)`:** Initiiert atomare Betroffenen-Löschung (Klasse P, $O(1)$ Schlüsselvernichtung über alle Collections).
- **`drop_collection(collection_id)`:** Vernichtet den `CollectionKEK` atomar in $O(1)$. Nachfolgende Neuanlage derselben Collection startet mit inkrementierter Epoche (`epoch + 1`).

### 6.2 Koordination des Purge-Hintergrundläufers

- Stellt den periodischen Aufruf von `run_purge_window` über Hintergrund-Worker bereit.
- Hält während des physischen Purge die `PurgeBarrier` exklusiv gegen Hintergrund-Kompaktionen.

---

## 7. Zeitreise & Temporale Filterung (SPEC §9)

Zwei orthogonale Zeitachsen werden streng getrennt:
1. **Transaktionszeit (`as_of: u64`):**
   - MVCC-Sequenznummer oder logischer Timestamp.
   - Bestimmt, welcher historische Zustand der Indizes abgefragt wird.
   - Spätere Schreibvorgänge verändern historische Antworten nicht (Z-T2).
2. **Gültigkeitszeit (`valid_at: i32`):**
   - Kalendertage seit 1970-01-01 (Julian Day / Epoch Days).
   - Fachlicher Gültigkeitszeitraum des Dokuments (`valid_from` bis `valid_to`).
   - Wirkt als Vorfilter auf Kandidatenebene.

**Invariante (Z-T5):** Auch bei historischem `as_of`-Stand wird der `ReadGate` immer im **aktuellen** Zustand geprüft. Eine Löschung macht Daten für alle Zeitreise-Zeitpunkte unlesbar.

---

## 8. Hintergrund-Worker & Autonome Wartung

1. **`ExpiryCleanupWorker`:** Entfernt Dokumente mit abgelaufener TTL aus dem LSM-Store und den Indizes.
2. **`OrphanCleanupWorker`:** Bereinigt verwaiste Checkpoint- und Index-Fragmente nach Crash-Recovery.
3. **`DecayCleanupWorker` / `AdaptiveDecayController`:** Berechnet Relevanz-Zerfall für dynamische Gedächtnisprofile.
4. **`PidLatencyController`:** PID-Regler zur dynamischen Begrenzung von Suchtiefen und Rerank-Budgets bei Latenzdruck.
5. **`Percolation` (Graph Health):** Überwacht Graph-Konnektivität und stößt Rebonding-Kandidaten an.

---

## 9. Konfiguration & Schnittstellen

```rust
pub struct ContextraConfig {
    pub dimension: usize,
    pub max_elements: usize,
    pub distance_metric: DistanceMetric,
    pub encryption_passphrase: Option<String>,
    pub max_ram_mb: u64,
    pub group_commit_window_micros: u64,
    pub durability_mode: DurabilityMode,
    pub deletion_proof_active: bool,
    pub vector_delete_mode: VectorDeleteMode,
    pub memtable_size_limit: usize,
    pub expiry_reaper_interval: Duration,
    pub orphan_registry_path: Option<PathBuf>,
    pub community_detection: CommunityDetectionConfig,
    pub embedding_backend: EmbeddingBackend,
    pub consolidation_enabled: bool,
    pub consolidation_interval: Duration,
    pub max_llm_calls_per_cycle: usize,
    pub consolidation_launcher: Option<ConsolidationLauncher>,
    pub tenant_policy: TenantPolicy,
    pub metrics_sink: Option<Arc<dyn MetricsSink>>,
}
```

---

## 10. Invarianten

1. **`#![forbid(unsafe_code)]`** im gesamten Crate.
2. **Zero-Panic-Policy:** Keine `unwrap()`, `expect()`, `panic!()` in Produktivpfaden.
3. **Fail-Closed bei Lesezugriffen:** Unklarer Gate-Zustand, fehlender Schlüssel oder Berechtigungsfehler führt zum sofortigen Abbruch ohne Datenlecks.
4. **Kein Klartext im WAL oder auf Platte (T-DEL-1):** Alle Payloads werden verschlüsselt serialisiert.
5. **Determinismus (INV-5):** Identische Eingaben führen zu identischen Such- und Fusion-Ergebnissen (Tie-Break: `DocId` aufsteigend).
6. **Mandantenisolation:** Ist `TenantPolicy::Required` konfiguriert, scheitert jeder Aufruf ohne expliziten `TenantScoped`-Kontext mit `TenantScopeViolation`.

---

## 11. Test-Pflichten

| Test-ID | Beschreibung |
|---------|-------------|
| T-DB1 | Insert → Get Roundtrip liefert identisches Dokument mit Metadaten |
| T-DB2 | Hybrid-Search: 4-Signal-Kombination liefert deterministisches Ranking mit RRF |
| T-DB3 | ReadGate: Gelöschtes Dokument liefert `Ok(None)` auf `get`, `scan` und `search` |
| T-DB4 | Auffüllen auf $k$: Gate-gefilterte Kandidaten werden durch Folgetreffer ersetzt |
| T-DB5 | Zeitreise: `as_of` liefert historischen Zustand; gelöschte Dokumente bleiben gesperrt (Z-T5) |
| T-DB6 | Temporale Validität: Dokumente außerhalb `valid_at` werden herausgefiltert |
| T-DB7 | 2PC Atomarität: Fehler in Vektorindex-Einfügung rollt LSM- und Text-Änderungen zurück |
| T-DB8 | Mandantenisolation: Tenant A kann keine Collections oder Dokumente von Tenant B lesen |
| T-DB9 | Drop Collection: Widerruf des CollectionKEK macht alle Dokumente sofort unlesbar |
| T-DB10 | Crash-Recovery: Nach unvollständigem Schreibvorgang stellt WAL konsistenten Zustand her |
