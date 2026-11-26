# SPEC-09 — Betrieb: Ingest, Konfiguration, Performance, Simulation und Roadmap

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Architektur](SPEC-01-ARCHITECTURE.md) · [Durability](SPEC-07-DURABILITY.md)

---

## 1. Ingest-Pipeline & inkrementeller Revisions-Diff (`contextra-ingest`)

### 1.1 Ablauf

```
Quelldokument (Bytes + source_uri + ACL-Vorgabe)
  → Format-Erkennung (Magic Bytes / MIME) → Text-Extraktion (PDF, Office, HTML, E-Mail, Code, CSV/JSON)
  → Normalisierung (Unicode NFC) → Injection- & PII-Tag-Scan (TaintFlags!)
  → Strukturbewusstes Chunking (512 tok Ziel, heading_path, AST für Code, L2 extraktiv + tokcount)
  → Revisions-Diff gegen Vorversion derselben source_uri (Match via content_hash!):
      • Identischer content_hash → Behält CellId, Gid, Handle, Embeddings, Summaries & AccessStats!
      • Neuer/geänderter Chunk   → Neue Version + Embedding aus Derivation Cache oder Embedder-Port
      • Weggefallener Chunk      → Atomares retire (+ Supersedes falls Nachfolger erkannt)
  → Atomarer Commit (+ Work-Queue-Jobs für optionale asynchrone L1-Summaries)
```

### 1.2 Embedding-Platzierung

Standardmäßig werden Embeddings **vor** dem Commit über Batch-Aufrufe (Default 32) am `Embedder`-Port bzw. aus dem Derivation Cache aufgelöst, sodass neue Chunks ab Sichtbarkeit sofort vektoriell findbar sind. Im expliziten Bulk-Modus `Deferred` werden sie asynchron nachberechnet und Suchabfragen melden `lane_coverage < 1.0`.

### 1.3 Unterstützte Formate

| Kategorie | Formate | Parser | Chunk-Strategie |
|---|---|---|---|
| **Text** | Markdown, Plain, RST, AsciiDoc | Eigenbau | `markdown-v2` (Überschriften/Absätze/Sätze) |
| **Office & Mail** | PDF, DOCX, XLSX, PPTX, ODT, EML/MIME | `lopdf` / `calamine` / `quick-xml` / `mail-parser` | Layout-/Tabellen-/Thread-bewusst |
| **Code** | Rust, Python, TS, Go, C/C++, Java, ... | `tree-sitter` | `code-treesitter` (AST + Signatur als L2) |
| **Daten & Web** | JSON/JSONL, CSV/TSV, YAML, TOML, HTML | `serde_json` / `csv` / `html5ever` | `json-path` / `table-row` / `html-semantic` |
| **Binär** | Beliebig | — | `binary-fastcdc` (8 KiB Ziel, `boundary_safe = false`) |

---

## 2. Referenz-Konfiguration (`contextra.toml`)

```toml
[repository]
name = "enterprise-memory"
format_version = 1
profile = "standard"                  # lite | standard | scale

[memory_budget]
total = "8GiB"
fixed_pct = 25                        # Manifest, Locators, FST, Quant-Codes, Bitmaps
memtable_pct = 15                     # MemDeltas + Index-Deltas
block_cache_pct = 40                  # Partitionierter Buffer Pool (W-TinyLFU)
query_pct = 10                        # Aktive Abfragen, Fusion, Packer, Rerank
jobs_pct = 10                         # Fold, Compaction, Index-Builds

[storage]
max_pack_size = "256MiB"
max_extent_size = "512KiB"
io_backend = "pread"                  # pread | io_uring (nur Linux mit O_DIRECT)
compression_level_online = 3
compression_level_compact = 19

[wal]
group_commit_max_wait_ms = 1
group_commit_max_batch = 64
segment_max_size = "64MiB"

[versioning]
default_durability = "durable"        # volatile | buffered | durable | strict | replicated
default_isolation = "snapshot"        # snapshot | serializable
snapshot_lease_seconds = 300
idempotency_ttl_days = 7
conflict_auto_resolve_days = 7        # nur außerhalb main; main bleibt konfliktfrei

[merge_gates]
require_provenance = true             # G2
max_blast_radius_pct = 1.0            # G4
block_injection_taint = true          # G6

[gc]
interval_seconds = 3600
grace_period_seconds = 3600
pack_live_threshold = 0.5
reflog_window = 1000

[background]
threads = 4
fold_trigger_size = "32MiB"
fold_trigger_count = 10000
fold_trigger_seconds = 30
relayout_enabled = false              # opt-in nach Benchmark-Nachweis (>= 20% I/O-Gewinn)
relayout_interval_hours = 24
relayout_exploration_pct = 5

[egress_guard]
mode = "policy"                       # local_only | policy
redact_pii_tainted = true
timeout_ms = 100
```

---

## 3. Performance-Ziele und Benchmark-Wahrheit [Hypothesen]

Alle Zahlen werden vor M1/M2 auf realer Hardware validiert. **Verbot:** Messungen `< 10⁶` Cells dürfen nicht auf `10⁷–10⁸` extrapoliert werden; veröffentlichte Benchmarks nennen stets Hardware, Datensatz, Profil (`lite`/`standard`/`scale`) und ob Embedding-Inferenz enthalten ist.

| Metrik | Zielwert [Hypothese] | Bedingungen |
|---|---|---|
| **Fork** | < 50 µs, Speicher O(1) | beliebige Repo-Größe |
| **Commit (`durable` Group Commit)** | p99 < 2–5 ms | NVMe, Batch ≤ 64 |
| **Hybrid-Recall (10M Chunks, warm)** | p50 < 10 ms, p99 < 30 ms | Profil `standard`, ohne Embedding-Inferenz |
| **Window (10⁶ Turns)** | < 1 ms | nur Tree-Aggregates |
| **Expand (einzelne Cell L0)** | < 500 µs | Warm Buffer Pool |
| **Vektor Recall@10** | ≥ 0.95 | gegen exakten Scan bei Ziel-`ef`/`L_search` |
| **Recovery** | < 5–10 s je 500 MiB WAL | sofort betriebsbereit via Flat-Fallback |
| **Write-Amplification (Fold)** | < 5× (Bulk-Ingest) | Batch ≥ 1000 Cells |

---

## 4. Test-, Simulations- und Verifikationsstrategie

### 4.1 Deterministische Simulation (`contextra-sim`) — FoundationDB-/TigerBeetle-Stil

Vollständige Ausführung des gesamten Kerns (`L0–L4`) gegen injizierte Ports (`FaultVfs`, `ManualClock`, `SeededRng`):
- Injiziert an jedem I/O-Punkt: **Torn Writes, Short Writes, verlorene/vergiftete `fdatasync`s, `ENOSPC`, Bit-Flips, zufällige Prozess-Abstürze** während Commits, Folds, Seals, Compactions und GC (`≥ 10⁶`–`10⁷` Schritte).
- **Orakel:** Vergleich gegen einfaches In-Memory-Referenzmodell über alle Invarianten **INV-1 bis INV-9**:
  - `RootSet` ist history-unabhängig (INV-1),
  - Crash hinterlässt Präfix bestätigter Commits (INV-3),
  - Löschen aller Index-Segmente liefert über Flat-Fallback identische Kandidatenmengen (INV-4),
  - SATB+Remark-GC verliert niemals erreichbare Objekte (INV-9).

### 4.2 Verifikations-Matrix

| Ebene | Werkzeug / Suite | Prüfung |
|---|---|---|
| **Property-Tests** | `proptest` | History-Unabhängigkeit, Diff/Merge-Roundtrip, Knapsack `tokens_used ≤ budget`, `BeliefBeta` `source_root`-Unabhängigkeit, `Gid → DocId` |
| **Concurrency** | `loom` | `ArcSwap` Manifest-Publikation, `crossbeam-epoch`, Buffer Pool Pin/Evict, Group Commit |
| **Memory Safety** | `Miri` + `cargo-fuzz` | Alle `unsafe`-Inseln (`simd`, `sys`); 24h-Fuzzing für WAL, Pack, Extent, rkyv-Validierung, Parser |
| **Sicherheit & MCP** | Gezielte Suiten | ACL-Leakage (kein Info-Leak in Scores/Zählern/Fehlern), Handle-MAC-Halluzination (1023/1024 Blockade), Egress Fail-Closed, deklarative MCP-Klassifikation |
| **Architektur & Supply Chain** | `cargo metadata` / `xtask` / `cargo-deny` / `cargo-vet` | Layering (L0–L4 `tokio`-frei, keine Zyklen), Unsafe-Baseline, Determinismus-Lint (kein `SystemTime::now()`/`std::fs` im Kern), kein Löschketten-Code |

---

## 5. Monitoring & Health

- **Prometheus / OpenTelemetry** über `MetricsSink`: Commit-Latenz, WAL-Gruppengröße, MemDelta-Druck (`PressureLevel`), `lane_coverage` je Modalität, Segment-Fan-Out, Block-Cache-Trefferquote je Partition, Query-Latenz je Operator, Work-Queue-Backlog, Rerank-Skips, Compaction-Backlog.
- **`HealthStatus`:** `Healthy | Degraded | Unhealthy` (u. a. bei `PartialIndex` oder `Elevated` Backpressure) bzw. Fail-Stop `repo_poisoned`.

---

## 6. Roadmap (mit verbindlichen Exit-Kriterien)

| Phase | Inhalt | Exit-Kriterium |
|---|---|---|
| **0 — Fundament & Simulation** | Workspace-Bereinigung (Entfernung alter Löschketten-/Schalen-Crates), Format-Spezifikation (`contextra-wire`), **`contextra-sim`** (`FaultVfs`/`ManualClock`), Benchmark-Korpus & Baselines | CI-Layering grün; Simulation führt WAL-Crash-Zyklen aus; Ist-Baselines erfasst |
| **1 — Truth & Storage Kernel** | WAL (`contextra-wal`), PAX-Extents & Locators (`contextra-store`), entkoppelte Cells, Prolly-Trees + `RootSet` + `MemDelta` (`contextra-mvcc`), `IdempotencyTable`-Sidecar, CLI `ctx verify` | ≥ 10⁶ Simulationsläufe ohne INV-1/3/6/9-Verletzung; Commit p99 < 5 ms; Write-Amp gemessen |
| **2 — Indizes & Hybride Query** | `Gid↔DocId`, Flat/HNSW/Vamana-Segmente, FTS (FST + BlockMaxWAND + globale BM25F-Stats), CSR + Forward-Push-PPR, `IndexManifest` + Watermarks | Recall@10 ≥ 0.95; Index-Verlust-Test (Löschen aller Index-Segmente ⇒ Flat-Fallback + vollständiger Rebuild) bestanden |
| **3 — Context Engine, Agenten & MCP** | Zoom-Knapsack + Small-to-Big, MAC-Handles (`c:k7qd-3f`), `VERIFY`, Abstention, Session-Branches + Merge-Gates `G1`–`G7`, Ingest-Revisions-Diff, `WorkQueue`-Sidecar, MCP & Server | `tokens_used ≤ budget` in 100 % der Fälle; Utility@Budget signifikant > Top-k-Baseline; ACL- & Handle-Tests grün |
| **4 — Skalierung & Verteilungs-Sync** | S3-Tier (`T2`), Async Push/Pull-Objekt-Sync, Read-Replicas, optional Affinity Re-Layout (nur nach ≥ 20 % Benchmark-Nachweis) | Multi-Device-Sync & Fault-Tests grün; Re-Layout-Effekt gemessen |
| **5 — Härtung & Optionales** | KEK/DEK-AEAD-Audits, `io_uring`-Evaluierung, optional `contextra-sqlite` / `contextra-kvcache`, 72h-Soak-Test, Formatfreeze 1.0 | Alle Soak-, Fuzz- (24h) und Last-Gates innerhalb Zielkorridor |

---

## 7. Konsolidiertes Risiko- & Blindfleckregister

| # | Risiko | Schwere | Gegenmaßnahme / Frühindikator |
|---|---|---|---|
| 1 | **Prolly-Tree Write-Amplification** | mittel | `MemDelta` + Batch-Fold; Bulk-Load-Pfad; Phase-0-Go/No-Go-Messung |
| 2 | **`Gid → DocId` Übersetzungskosten bei sehr großen ACLs** | mittel | Sortiertes Range-Slicing + pro-Query-Cache + vorberechnete Gruppen-Bitmaps; Alarm bei > 20 % Query-Anteil |
| 3 | **Segment-Fan-Out durch viele Session-Branches** | mittel | Sessions puffern in Memtable, versiegeln spät und kompaktieren beim Merge; Fan-out-Limit ≤ 24 |
| 4 | **Flat-Fallback-Latenz bei starkem Index-Rückstand** | mittel | Hohe Work-Queue-Priorität für Index-Builds, `lane_coverage`-Metrik + Ingest-Backpressure |
| 5 | **Prompt-Injection über externe Dokumente** | hoch | `TrustTier` + `TaintFlags` + isolierter Datenrahmen im Prompt + Gate `G6` + Egress-Guard |
| 6 | **Hub-Knoten im Graph (z. B. globale Entitäten)** | mittel | Degree-Skip-Index (> 10⁴), harte TraversalLimits & Hub-Dämpfung im Forward-Push-PPR |
| 7 | **Vergessene Snapshot-Pins / offene Sessions blockieren GC** | mittel | Harte 300 s Snapshot-Leases + Branch-TTLs (`branches_days`) |
| 8 | **Re-Layout-Feedbackschleife ("Rich-get-richer")** | niedrig | Opt-in erst ab Phase 3/4 + 5 % Exploration + Locator-Rollback |
| 9 | **Embedding-Modellwechsel im Großbestand** | mittel | Dual-Space-Betrieb, `lane_coverage`, Priorisierung nach Salience |
| 10 | **Performance-Überversprechen vor Messung** | hoch | Alle Latenzzahlen strikt als Hypothesen markiert; verbindliche Phase-0/2-Gates |

---

*Ende SPEC-09. Zurück zum [Spezifikationsindex](SPEC-00-INDEX.md).*
