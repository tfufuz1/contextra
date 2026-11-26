# SPEC-09 — Betrieb: Ingest, Konfiguration, Tests, Performance, Roadmap

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Architektur](SPEC-01-ARCHITECTURE.md) · [Durability](SPEC-07-DURABILITY.md)

---

## 1. Ingest-Pipeline

### 1.1 Überblick

```
Quelldokument ──► Format-Erkennung ──► Parser ──► Chunker ──► Cell-Erzeugung ──► Commit
                   (magic bytes,         │          │              │
                    extension,           │          │         Zoom L2 (extraktiv)
                    MIME sniffing)       │          │         Token-Zählung
                                        │          └── Kanten (follows, part_of)
                                        │
                                   Work Queue:
                                     Embed-Tasks
                                     Summarize-Tasks (L1)
```

### 1.2 Unterstützte Formate

| Kategorie | Formate | Parser | Chunk-Strategie |
|---|---|---|---|
| Text | Markdown, Plain Text, RST, AsciiDoc | Eigenbau | `markdown-v2` |
| Office | PDF, DOCX, XLSX, PPTX, ODT | `lopdf`/`calamine`/`docx-rs` | Struktur-basiert |
| Code | Rust, Python, TypeScript, Go, C/C++, Java, etc. | tree-sitter | `code-treesitter` |
| Daten | JSON, JSONL, CSV, TSV, YAML, TOML, XML | `serde_json`/`csv`/etc. | `json-path` / `table-row` |
| Web | HTML, EPUB | `scraper`/`epub` | `html-semantic` |
| Binär | Beliebige Binärdaten | — | `binary-fastcdc` |

### 1.3 Ingest-API

```rust
pub struct IngestRequest {
    pub source: IngestSource,           // File, URL, Bytes, Stream
    pub namespace: NsId,
    pub branch: BranchName,
    pub chunker_override: Option<String>,
    pub trust: TrustLabel,
    pub author: PrincipalId,
    pub tags: Vec<String>,
    pub durability: Durability,
    pub idempotency_key: Option<String>,
}

pub enum IngestSource {
    File(PathBuf),
    Bytes { data: Bytes, filename: String, mime: Option<MimeType> },
    Directory { path: PathBuf, recursive: bool, glob: Option<String> },
}

pub struct IngestResult {
    pub commit_id: CommitId,
    pub cells_created: u32,
    pub sequences_created: u32,
    pub work_tasks_queued: u32,         // Embed + Summarize Tasks
    pub warnings: Vec<IngestWarning>,
}
```

### 1.4 Automatische Facetten-Erzeugung bei Ingest

| Facette | Erzeugung | Synchron/Asynchron |
|---|---|---|
| Extraktive Zoom L2 (Titel) | Engine-intern, deterministisch | Synchron (beim Commit) |
| Token-Zählungen | Engine-intern | Synchron |
| Kanten (follows, part_of) | Chunker | Synchron |
| Handle-Registrierung | Engine-intern | Synchron |
| Embedding | Extern (Work Queue) | Asynchron |
| Abstraktive Zoom L1 (Summary) | Extern (Work Queue) | Asynchron |

---

## 2. Konfiguration

### 2.1 Repository-Konfiguration (`contextra.toml`)

```toml
[repository]
name = "my-agent-memory"
format_version = 1
created_at = "2026-11-20T10:00:00Z"

[storage]
page_cache_size = "512MiB"          # Page-Cache für Extent-Heads
memtable_budget = "128MiB"         # RAM für MemDeltas aller Branches
index_cache_size = "128MiB"        # Cache für Locator, FST, Bloom
max_pack_size = "256MiB"
max_extent_size = "512KiB"
compression_level_online = 3       # zstd-Level für Schreib-Pfad
compression_level_compact = 19     # zstd-Level für Compaction

[wal]
group_commit_max_wait_ms = 1
group_commit_max_batch = 64
segment_max_size = "64MiB"
segment_rotation_seconds = 60

[versioning]
default_durability = "durable"
default_isolation = "snapshot"
idempotency_ttl_days = 7
conflict_auto_resolve_days = 7

[gc]
interval_seconds = 3600
grace_period_seconds = 3600
pack_live_threshold = 0.5
reflog_window = 1000
max_duration_seconds = 1800

[tiering]
t1_to_t2_idle_days = 30            # Tage ohne Zugriff → S3
t2_endpoint = ""                   # S3-kompatible URL (leer = kein T2)
t2_bucket = ""
t2_prefix = ""

[background]
threads = 4                         # Hintergrund-Threads
fold_trigger_size = "32MiB"
fold_trigger_count = 10000
fold_trigger_seconds = 30
relayout_interval_hours = 24
relayout_exploration_pct = 5

[encryption]
enabled = false
algorithm = "aes256gcm"             # aes256gcm | xchacha20poly1305

[access_trace]
enabled = true
sample_rate = 0.1                   # 10 % der Recalls
flush_interval_seconds = 10

[mcp]
enabled = false
transport = "stdio"
default_budget = 4000

[grpc]
enabled = false
listen = "127.0.0.1:9100"
max_concurrent = 64

# Namespace-Policies
[namespace."agent/episodic"]
kind = "episodic"
chunker = "markdown-v2"
tokenizers = ["cl100k", "o200k"]
embedders = ["emb-768-v3"]
embed_dimensions = [768]
zoom_ladder = ["L0:raw", "L1:summary<=200tok", "L2:title<=24tok"]
default_trust = "tool-unverified"
max_cell_size = "256KiB"
retention = { hot_days = 14, warm_days = 180, zoom_down_after = "180d" }
merge = { text = "diff3", tags = "union", counters = "sum", default = "manual" }
durability = "durable"
sequence_enabled = true

[namespace."agent/semantic"]
kind = "semantic"
chunker = "passthrough"
tokenizers = ["cl100k"]
embedders = ["emb-768-v3"]
zoom_ladder = ["L0:raw", "L2:title<=24tok"]
default_trust = "tool-unverified"
retention = {}                      # Unbegrenzt
merge = { text = "lww", tags = "union", default = "lww" }
durability = "durable"
```

---

## 3. Performance-Ziele und KPIs

### 3.1 Latenz-Ziele (Hypothesen, zu validieren)

| Metrik | Ziel | Bedingungen |
|---|---|---|
| Fork | < 50 µs, Speicher O(1) | — |
| Commit (1–100 Cells, durable) | p99 < 2 ms | NVMe, Group Commit |
| Recall (1M Cells, warm, 8k-Budget, hybrid) | p50 < 5 ms, p99 < 25 ms | 3 Namespaces |
| Window über 10⁶ Turns | < 1 ms | Ohne Inhaltszugriff |
| Expand (einzelne Cell, L0) | < 500 µs | Warm Cache |
| Extent-Zugriffe pro Recall (nach Re-Layout) | ≤ 4 im Mittel | — |
| Schreibamplifikation (Fold) | < 5× | Batchgröße ≥ 1 000 Cells |
| Recovery nach Crash | < 5 s | 1 GiB WAL |
| Recovery nach Crash (typisch) | < 500 ms | 100 MiB WAL |

### 3.2 Qualitätsmetriken

| Metrik | Beschreibung | Messverfahren |
|---|---|---|
| **Recall@Budget** | Anteil relevanter Cells bei gegebenem Token-Budget | Trace-Replay mit annotierten Datensätzen |
| **Nutzen-pro-Token** | Anteil geladener Token, die der Agent tatsächlich verwendet | Agent-Traces mit Zitierungs-Tracking |
| **Task-Erfolgsrate** | Erfolgsrate bei gleichem Budget vs. Baseline | A/B-Tests mit Agent-Workloads |
| **Expand-Rate** | Wie oft der Agent nachlädt (niedriger = bessere Erstauswahl) | Automatisch aus Access-Trace |
| **Kosten pro Aufgabe** | Token-Verbrauch pro gelöster Aufgabe | Agent-Traces |

### 3.3 Benchmarks

| Typ | Tool | Beschreibung |
|---|---|---|
| Mikro-Benchmarks | `criterion` | Pro Crate: Prolly-Ops, SIMD-Distanz, BM25-Scoring, Knapsack |
| System-Benchmarks | eigenes Harness | Synthetische + aufgezeichnete Workloads |
| Regression-Gates | CI | Latenz UND Qualität dürfen nicht regressieren |
| Vergleichs-Benchmarks | eigenes Harness | Gegen SQLite, Dolt, LanceDB |

---

## 4. Teststrategie

### 4.1 Deterministische Simulation

**FoundationDB/TigerBeetle-Stil:** Gesamte Engine gegen simulierte Umgebung:
- Simulierte Zeit (kein `SystemTime::now()`)
- Simuliertes Dateisystem (`FaultVfs`): torn writes, verlorene fsyncs, Bit-Flips, volle Platte
- Simuliertes Netzwerk (ab Phase 4)
- Seeds reproduzierbar

```rust
pub trait Vfs: Send + Sync {
    fn open(&self, path: &Path, opts: OpenOptions) -> Result<Box<dyn VfsFile>>;
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn remove(&self, path: &Path) -> Result<()>;
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn sync_dir(&self, path: &Path) -> Result<()>;
}

pub trait VfsFile: Send + Sync {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;
    fn sync(&self) -> Result<()>;
    fn len(&self) -> Result<u64>;
    fn truncate(&self, len: u64) -> Result<()>;
}
```

### 4.2 Property-Tests

`proptest` gegen triviales In-Memory-Referenzmodell:

| Eigenschaft | Beschreibung |
|---|---|
| History-Unabhängigkeit | `gleicher Inhalt ⇒ gleicher Hash` |
| Diff-Roundtrip | `apply(diff(a, b), a) = b` |
| Diff-Identität | `diff(a, a) = ∅` |
| Merge-Identität | `merge3(b, a, a) = a` |
| Merge-Trivial | `merge3(b, x, b) = x` |
| Count-Konsistenz | `Rang/COUNT stimmt mit Iteration überein` |
| Token-Summen | `sum(children) = parent.aggregate` |
| Idempotenz | `commit(key, data); commit(key, data) = 1 Commit` |
| Trust-Propagierung | `derived_trust = min(source_trusts)` |

### 4.3 Crash-Matrix

`FaultVfs` bricht an **jedem** I/O-Schritt ab:
- Erwartung: Zustand = Präfix der bestätigten Commits
- `ctx verify` grün nach Recovery
- Mindestens 10⁶ Schritte pro Testlauf

### 4.4 GC-Simulation

Deterministische Interleavings von Schreiber, Dedup, GC und Abstürzen:
- Property: „kein erreichbares Objekt geht verloren"
- Mindestens 10⁶ Schritte

### 4.5 Vollständige Testmatrix

| Netz | Inhalt | Phase |
|---|---|---|
| Invarianten-Checker (`ctx verify`) | Hashbaum, Locator, Index↔Primärdaten, Lineage, GC | 0 |
| `loom` | Lock-freie Strukturen (ArcSwap, Epoch) | 0 |
| Miri | `unsafe`-Crates (simd, sys) | 0 |
| Fuzzing | Alle Decoder (Objekt, Knoten, Commit, WAL, Lane-Codecs) | 0 |
| Differential-Tests | Gleiche SQL gegen SQLite-Nativtabellen und `contextra-sqlite` | 1 |
| Format-Goldens | Referenzdateien je Schicht; ID-Stabilität über Versionen | 1 |
| Soak/Chaos | Langläufer mit Agenten-Workload, Absturz-Injektion, Speicherdruck | 2 |
| Agenten-Tests | Golden-Transkripte typischer Tool-Aufrufe | 2 |
| Retrieval-Qualität | CI-Gate: Utility@Budget darf nicht regressieren | 2 |
| Supply Chain | `cargo deny` (Lizenzen, Advisories) | 0 |
| Benchmarks | Regression-Gates in CI | 1 |

---

## 5. Monitoring und Metriken

### 5.1 Metriken-Katalog (Prometheus-kompatibel)

| Metrik | Typ | Beschreibung |
|---|---|---|
| `contextra_commits_total` | Counter | Gesamtzahl der Commits |
| `contextra_commit_latency_seconds` | Histogram | Commit-Latenz |
| `contextra_recall_latency_seconds` | Histogram | Recall-Latenz |
| `contextra_recall_budget_used_tokens` | Histogram | Token-Verbrauch pro Recall |
| `contextra_recall_candidates_total` | Histogram | Kandidaten pro Recall |
| `contextra_recall_winners_total` | Histogram | Gewinner pro Recall |
| `contextra_expand_total` | Counter | Expand-Aufrufe |
| `contextra_fork_total` | Counter | Fork-Operationen |
| `contextra_merge_total` | Counter | Merge-Operationen |
| `contextra_merge_conflicts_total` | Counter | Konflikte bei Merge |
| `contextra_wal_bytes_written` | Counter | WAL-Bytes geschrieben |
| `contextra_wal_fsync_latency_seconds` | Histogram | WAL-fsync-Latenz |
| `contextra_fold_latency_seconds` | Histogram | Fold-Latenz |
| `contextra_fold_entries_total` | Histogram | Einträge pro Fold |
| `contextra_gc_runs_total` | Counter | GC-Läufe |
| `contextra_gc_bytes_reclaimed` | Counter | Freigegeben durch GC |
| `contextra_pack_count` | Gauge | Aktuelle Pack-Anzahl |
| `contextra_pack_bytes_total` | Gauge | Gesamtgröße aller Packs |
| `contextra_extent_reads_per_recall` | Histogram | I/O pro Recall |
| `contextra_page_cache_hit_ratio` | Gauge | Cache-Trefferquote |
| `contextra_memdelta_bytes` | Gauge | RAM für MemDeltas |
| `contextra_work_queue_pending` | Gauge | Ausstehende Work-Tasks |
| `contextra_work_queue_leased` | Gauge | Aktuell bearbeitete Tasks |
| `contextra_active_branches` | Gauge | Aktive Branches |
| `contextra_cells_total` | Gauge | Gesamtzahl Cells |
| `contextra_evidence_quality` | Histogram | Evidence Sufficiency Score |

### 5.2 Health-Checks

```rust
pub struct HealthStatus {
    pub status: Health,                 // Healthy | Degraded | Unhealthy
    pub wal_lag_bytes: u64,            // Unverarbeitete WAL-Bytes
    pub memdelta_pressure: f32,        // 0.0–1.0
    pub page_cache_pressure: f32,
    pub gc_lag: Duration,              // Zeit seit letztem erfolgreichen GC
    pub pack_count: u32,
    pub work_queue_backlog: u32,
    pub last_error: Option<String>,
}
```

---

## 6. Schema-Evolution und Kompatibilität

### 6.1 Format-Versionierung

Jedes On-Disk-Format hat:
- **Magic Bytes** (4 Bytes): Identifikation des Formats
- **Format-Version** (u16): Inkrementiert bei inkompatiblen Änderungen
- **Feature-Flags** (u16): Bitfeld für optionale Features

### 6.2 Kompatibilitätsregeln

| Änderung | Strategie |
|---|---|
| Neues optionales Feature | Feature-Flag setzen; alte Leser ignorieren unbekannte Flags |
| Neuer CellKind-Wert | `Unknown(u8)` Fallback; keine Schema-Migration nötig |
| Neuer EdgeKind-Wert | `Unknown(u8)` Fallback |
| Neuer AttrValue-Variant | `Unknown(Bytes)` Fallback |
| Neues Lane-Format | Lane-Codec-ID im Extent-Header; unbekannte Lanes werden übersprungen |
| Inkompatible Formatänderung | Format-Version inkrementieren; Migrationstool `ctx migrate` |

### 6.3 Namespace-Policy-Änderungen

| Änderung | Verhalten |
|---|---|
| Neuer Tokenizer hinzufügen | Bestehende Cells ohne tokcount für neuen Tokenizer; Neuberechnung bei Zugriff |
| Neuer Embedder hinzufügen | Re-Embedding über Work Queue; alte Segmente bleiben bis Migration |
| Zoom-Level hinzufügen | Neue Stufen werden bei nächstem Zugriff oder Background-Job erzeugt |
| Merge-Strategie ändern | Gilt für zukünftige Merges; bestehende Konflikte unberührt |
| Retention ändern | Gilt für zukünftige Retention-Durchläufe |

---

## 7. Roadmap

| Phase | Inhalt | Dauer | Exit-Kriterium |
|---|---|---|---|
| **0 — Spike** | Format-Spezifikation, Prototyp: CAS + WAL + Prolly-Tree + Fork/Commit, Simulationstest-Gerüst, Trace-Recorder | 6–8 Wochen | Fork/Commit/Recovery korrekt unter Simulation; Baselines erhoben |
| **1 — MVP Kern** | Cells/Lanes, Hot-Head/Cold-Tail-Extents, Branches/Merge (typisiert), Idempotenz, FTS, Flat-Vektor-Delta, Token-Summen, Rust-API + CLI + `ctx verify` | 3–4 Monate | Alle Invarianten grün; Replay-Evaluation vs. Baseline |
| **2 — Context Engine + MCP** | Segment-Vektorindex (Disk), Hybrid-Fusion, Zoom-Knapsack, Handles/Expand/Explain, MCP-Server, LLM-Fehler, SQL-vtab | 2–3 Monate | Utility@Budget signifikant über Top-k-Baseline |
| **3 — Lernen & Lebenszyklus** | Access-Trace, Affinity Re-Layout, Derivation Cache, Work Queue, Retention/Zoom-Down, Trust-Labels, Capabilities | 3 Monate | Extent-Zugriffe/Recall messbar reduziert |
| **4 — Verteilung** | Fetch/Push-Sync, Read-Replicas, Quorum-Durability, Tiering zu Objektspeicher | 3–4 Monate | Multi-Device-Szenario; Fault-Tests in Simulation |
| **5 — Prefix/KV-Store** | Prefix-Digest, KV-Block-Store, Konnektoren, kostenbasierte Eviction | 2–3 Monate | Gemessene Prefix-Cache-Trefferquote + TTFT-Verbesserung |

### 7.1 Team-Skizze

- 3–4 Storage-Ingenieure (Rust)
- 1 Retrieval/ML-Ingenieur
- 1 Test-/Simulations-Ingenieur
- Frühe Nutzung durch internes Agenten-Team als Design-Partner

### 7.2 Go/No-Go-Punkte

| Meilenstein | Kriterium |
|---|---|
| Nach Phase 0 | Prolly-Tree-Schreibamplifikation < 10× bei Einzelwrites |
| Nach Phase 1 | Alle Crash-Tests bestanden; Baselines für Utility@Budget erhoben |
| Nach Phase 2 | Utility@Budget signifikant über Top-k-Baseline; Agent-Integration demonstriert |
| Nach Phase 3 | Re-Layout reduziert Extent-Zugriffe um > 30 % |

---

## 8. Risiken (finale Bewertung mit Mitigationen)

| # | Risiko | Schwere | Wahrscheinlichkeit | Mitigation |
|---|---|---|---|---|
| 1 | **Schreibamplifikation Prolly-Tree** | mittel | mittel | Delta-Log + Fold-Batching; B-Parameter kalibrieren; Go/No-Go nach Phase 0 |
| 2 | **ANN + Versionierung** (Segmente, Deletion Vectors, Filter) | hoch | mittel | Stufenweise: Phase 1 Flat, Phase 2 Vamana; Delta-Index eliminiert Lag |
| 3 | **Learned-Layout-Feedbackschleife** | mittel | niedrig | 5% Exploration; Gewichtszerfall; Locator-Rollback |
| 4 | **Tokenizer-Kopplung** | mittel | mittel | Zählungen pro Tokenizer-Familie; lazy Neuberechnung; 2% Toleranzmarge |
| 5 | **Semantische Merge-Konflikte** | mittel | mittel | ConflictCell + externe Auflösung; Auto-Resolve nach 7d (lww) |
| 6 | **GC bei vielen Branches** | mittel | niedrig | TTL für Speculative Branches; generationelles GC; Reflog-Limits |
| 7 | **Format-Lock-in** | hoch | mittel | Feature-Flags; Format-Version; Migrationstool; Spezifikation vor Phase 1 einfrieren |
| 8 | **Scope-Creep** | hoch | hoch | Phasen strikt; Kern (Phase 0–2) liefert bereits Hauptnutzen |
| 9 | **Kein Nutzensnachweis ohne Daten** | hoch | mittel | Trace-Recorder in Phase 0; Go/No-Go nach Phase 2 |
| 10 | **Embedding-Lanes bei Modellwechsel** | mittel | mittel | Zeitlich begrenzter Dual-Betrieb; Work Queue priorisiert Re-Embedding |

---

## 9. KV-Cache-Store (optional, Phase 5)

Bewusst vom Kern getrennt, da modellabhängig und hardwarenah.

### 9.1 Konzept

- **Schlüssel:** `(model_id, tokenizer_id, Merkle-Hash-Kette über Token-Blöcke)` — Radix-Struktur, Präfixe teilen gemeinsame Blöcke.
- **Blockgröße:** 16–256 Token pro Block (an Inferenz-Server angepasst).
- **Speicherung:** Eigene Pack-Art auf NVMe, Tiering RAM ↔ NVMe ↔ S3.
- **Eviction:** Kostenbasiert: `Wiederberechnungskosten = Token × Modellkosten × Nutzungsfrequenz`.
- **Anbindung:** Konnektoren zu Inferenz-Stacks; Contextra liefert Prefix-Digest (SPEC-06, 3.5).
- **Invalidierung:** Automatisch über Hash-Kette.

---

*Ende SPEC-09. Zurück zum [Index](SPEC-00-INDEX.md).*
