# SPEC-01 — Architektur, Invarianten und Systemgrenzen

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md)

> **Normsprache** (RFC 2119): MUSS / MUSS NICHT = zwingend. SOLL = Regelfall, Abweichung nur per Entscheidungsregister. KANN = optional. Alle Latenz-, Durchsatz- und Kapazitätswerte sind **Hypothesen**; die einzige verbindliche Zielwerttabelle steht in [SPEC-09](SPEC-09-OPERATIONS.md) §3.

---

## 1. Vision und Scope

Contextra ist eine **eingebettete, versionierte Datenbank und hybride Context Engine in Rust** für LLM- und Agenten-Workloads. Token-Budgets, Zoom-Stufen, spekulative Branches mit Merge-Gates, bitemporale Fakten, Provenienz und Trust/Taint sind Primitiven der Speicherschicht.

Unterhalb der Versionierung ist Contextra ein content-addressed Objektspeicher mit PAX-Extents (Hot-Head/Cold-Tail). Oberhalb ist es eine Wissensdatenbank mit deterministischem Truth-Layer (`RootSet`), verwerfbaren Index-Segmenten (`IndexManifest`) und budgetoptimierendem Packer.

### 1.1 Deployment-Modell

| Aspekt | Festlegung |
|---|---|
| Primärmodus | Eingebettete Library (`contextra`) |
| Server-Modus | Dünner Daemon (`contextra-server`, `contextra-mcp`) |
| Instanzmodell | Ein Store = ein schreibender Prozess (exklusiver `LOCK`); Mehrprozesszugriff nur über den Server |
| Isolationsgrenze | Der Store ist die **harte Mandantengrenze** (eigener `repo_seed`, Schlüssel, WAL, Derivation Cache) |
| Plattformen | Linux x86-64/aarch64 primär; macOS/Windows mit `pread`/`pwrite`-Threadpool und plattformeigenem `DurableFs` |
| I/O | `pread`/`pwrite` Baseline, `io_uring` + `O_DIRECT` opt-in; **kein `mmap` im Produktpfad** |

### 1.2 Ressourcenprofile

Profile sind reine Parametersätze und schalten keine Funktion ab. Die Vektorstrategie folgt aus der RAM-Rechnung in [SPEC-09](SPEC-09-OPERATIONS.md) §4, nicht aus dem Profilnamen: Jedes Segment wählt die genaueste Quantisierung, die in das `fixed`-Budget passt.

| Profil | Referenzhardware | Zielbestand [Hypothese] | Default-`memory_budget` | Vektorstrategie |
|---|---|---|---|---|
| `lite` | 4 Kerne, 8 GiB | ≤ 2·10⁶ Cells | 4 GiB | Flat + HNSW (SQ8) |
| `standard` | 16 Kerne, 64 GiB, NVMe | ≤ 5·10⁷ Cells | 48 GiB | HNSW (SQ8) für kleine, Vamana (PQ/RaBitQ) für große Bestände |
| `scale` | ≥ 32 Kerne, 256 GiB, NVMe-RAID | ≤ 10⁸ Cells, 10⁹ Kanten | 192 GiB | Vamana (PQ/RaBitQ), Graph und Vollvektoren auf SSD |

### 1.3 Nicht-Ziele

| Nicht-Ziel | Konsequenz |
|---|---|
| Verteilter Multi-Writer-Konsens | Single-Node; Replikation, Objekt-Tier und Read-Replicas erst v2 |
| Eigene SQL-Engine | Rust-API, JSON-Plan, MCP; optional SQLite-vtab |
| Git-Wire-Kompatibilität | Nur Export/Import |
| Einzeldateiformat | Verzeichnislayout |
| DSGVO-Löschkette, Löschnachweise, Crypto-Shredding pro Subjekt | `retire` + Retention + Best-Effort-`purge`; **keine Zusicherung** physischer Unwiederbringlichkeit |
| LLM-Inferenz im Kern | Ports und externe Worker |
| Trigger / Stored Procedures | Commit-Subscriptions und Work Queue |
| Forschungs-Heuristiken im Kern | Je Modalität genau ein Standardalgorithmus |
| Seitenkanalfreiheit (Timing) | INV-5 deckt Ergebnisse, Zähler, Scores im definierten Umfang und Fehlermeldungen ab, nicht Laufzeitmessung |

### 1.4 Zugriffsmuster (Auslegungsannahme)

| Muster | Anteil [Annahme] | Optimierter Pfad |
|---|---|---|
| Recall unter Budget | ~65 % | Gid-Bitmaps → Segmente → Packer |
| `expand` | ~15 % | `CellLocator` → ein Cold-Read |
| `window` | ~10 % | Token-Summen-Baum, O(log n) |
| Fork → Gates → Merge | ~5 % der Transaktionen | Persistenter MemDelta, Prolly-Diff |
| Ingest | dominiert Schreiblast | Revisions-Diff, Derivation Cache |
| Hintergrundjobs | kontinuierlich | Work Queue, Prioritätsbudgets |

---

## 2. Leitprinzipien

1. **Truth ≠ Index.** Nur das `RootSet` ist Wahrheit; Indizes sind verwerfbar.
2. **Immutable first.** Veränderlich sind ausschließlich: WAL-Tail, Manifest-Tail, `CURRENT`-Slots, Kernel-Sidecars und atomar publizierte Zeiger im RAM.
3. **Ein Commit-Punkt.** Ein Commit ist dauerhaft, sobald sein WAL-Frame die Durability-Stufe erfüllt. Kein 2PC.
4. **Identität ≠ Ort.** Hashes bestimmen Identität, Locators den Ort.
5. **Agenten schreiben nie direkt nach geschützten Branches.**
6. **Budget und Belegbarkeit statt Top-k**, einschließlich expliziter Abstention.
7. **Abgeleitet ist ersetzbar, Herkunft ist Pflicht.**
8. **Fail-closed bei Sicherheit, fail-open bei Index-Verfügbarkeit**, wobei jede Degradierung im Ergebnis sichtbar ist.
9. **Kein stilles Verwerfen.**
10. **Abhängigkeiten zeigen nur nach unten**, in CI erzwungen.

---

## 3. Systeminvarianten (normativ)

Jede Invariante besitzt ein Prüfprädikat in `contextra-sim` oder eine benannte Testsuite.

| ID | Invariante | Prüfung |
|---|---|---|
| **INV-1** | **History-unabhängiger Truth-Hash:** Identische kanonische Schlüssel-Wert-Mengen aller sieben `RootSet`-Bäume ergeben denselben `truth_hash`, unabhängig von Schreibreihenfolge, Batching, Fold-Zeitpunkten, Re-Layout und Index-Compaction. *Nicht* zugesichert: Gleichheit zweier unabhängig erzeugter, nur inhaltlich äquivalenter Bestände (deren `Gid`, `CellKey`, `born_seq` und Beobachtungszeiten unterscheiden sich). | Property-Test: gleiche KV-Menge in Permutationen |
| **INV-2** | **Snapshot-Einheit:** Alle Operatoren einer Query lesen denselben Snapshot `(branch, head, base, delta, index_view, catalog, now)`. | Typestate `Snapshot<'db>` |
| **INV-3** | **Präfix-Konsistenz:** Nach Recovery ist der sichtbare Zustand ein Präfix der WAL-Commit-Folge, das mindestens alle mit `durable`/`strict` bestätigten Commits enthält. | Crash-Matrix |
| **INV-4** | **Index ⊆ Truth:** Jeder Treffer wird gegen den Snapshot validiert. Nicht indizierte Zeilen werden exakt ergänzt, solange sie `flat_fallback_max_rows` nicht überschreiten; andernfalls MUSS `lane_coverage < 1` und `PartialIndex` gemeldet werden. | Differenzialtest gegen exakten Scan |
| **INV-5** | **ACL vor Ranking:** Unsichtbare Cells erscheinen nie in Treffern, Zählern, Pfaden, `explain`, `did_you_mean` oder Fehlertexten; BM25-Statistiken folgen der Isolationsstufe aus [SPEC-05](SPEC-05-INDEXES.md) §3.3. | ACL-Leakage-Suite |
| **INV-6** | **Idempotenz:** Gleicher Scope, Key und Request-Digest ⇒ dasselbe Ergebnis ohne zweiten Commit, solange der Commit auf dem Branch erreichbar und die TTL nicht abgelaufen ist; abweichender Digest ⇒ `idempotency_mismatch`. | Crash- und Race-Tests |
| **INV-7** | **Integrität geschützter Branches:** Sie enthalten nie strukturelle Merge-Konflikte (`ConflictCell` mit Status `Open`). Inhaltliche Widersprüche (`Contradicts`-Kanten) sind zulässig und werden beim Lesen sichtbar gemacht. | Gate- und Direct-Write-Tests |
| **INV-8** | **Budget-Garantie:** Die mit dem angeforderten, registrierten Tokenizer gezählte Länge des gerenderten Packs ist ≤ `max_tokens − reserve_tokens`. | Property-Test über Renderformate |
| **INV-9** | **GC-Sicherheit:** Kein innerhalb des Retention-Horizonts von einer Wurzel erreichbares Objekt wird entfernt; einzige Ausnahme ist der protokollierte `Expunge` durch `purge`. | GC-Simulation |
| **INV-10** | **Speichergrenzen:** Kein Teilsystem überschreitet sein Budget; Folge ist Backpressure, nie OOM. | Allokationsbuchhaltung, Soak |
| **INV-11** | **Formatversionierung:** Jede Datei trägt Magic, Version und Feature-Flags; unbekannte inkompatible Flags verhindern das Öffnen. | Golden-Files, Fuzzing |
| **INV-12** | **Branch-Isolation:** Ein Branch ändert die Sicht eines anderen nur durch Merge oder Rebase. | Referenzmodell |
| **INV-13** | **Monotone Ableitungssicherheit:** Für jedes Derivat gilt: Sichtbarkeit ⊆ Schnitt der Quellsichtbarkeiten, Trust ≤ min(Quellen, Producer-Cap), Taint ⊇ Vereinigung der Quellen. | Property-Test über Lineage-DAG |
| **INV-14** | **Nonce-Eindeutigkeit:** Kein Paar (AEAD-Schlüssel, Nonce) verschlüsselt je zwei verschiedene Klartexte, auch nicht nach Absturz oder Wiederholung. | Konstruktionsnachweis [SPEC-03](SPEC-03-STORAGE-ENGINE.md) §8, Fault-Tests |

---

## 4. Schichtenarchitektur

```
L6  API & Server        contextra · contextra-mcp · contextra-server · contextra-cli
                        (opt.) contextra-sqlite · contextra-py
L5  Engine & Dienste    contextra-engine · -rank · -ingest · -agent · -cognition · -security
L4  Abgeleitete Indizes contextra-vector · -text · -graph
L3  Wahrheit & MVCC     contextra-mvcc
L2  Object Store        contextra-store
L1  Durability          contextra-wal · contextra-durable-fs
L0  Fundament           contextra-types · -ports · -wire · -simd · -sys · -crypto
```

| Schicht | Verantwortung | Grenze |
|---|---|---|
| L0 | Newtypes, Ports, kanonische Kodierung, SIMD, OS-Primitiven, Krypto-Primitiven | Keine Geschäftslogik, keine Policy |
| L1 | WAL, Group Commit, Manifest, Superblock, Recovery | Kennt kein Datenmodell; **einziger Aufrufer von Sync-Syscalls** |
| L2 | Packs, Extents, Kompression, Locators, Buffer Pool, Re-Layout | Kennt keine Commits oder Branches |
| L3 | Prolly-Trees, `RootSet`, MemDelta, Fold, Transaktionen, Merge, GC, Sidecars | Kennt keine ANN-/BM25-Algorithmen |
| L4 | Index-Segmente, Watermarks, Delta-Overlays | Kennt keine Fusion und keine Prompts |
| L5 | Planer, Fusion, Packer, Verifier, Ingest, Sessions, Gates, Egress | Dateizugriff nur über Ports |
| L6 | Protokolle, CLI, FFI | Keine Storage-/Index-Interna |

L3 steuert L4 nicht direkt an: L4 implementiert `DerivedIndex`, die Registrierung erfolgt in L5 (Dependency Inversion). So bleibt die Richtung L4 → L3 azyklisch.

### 4.1 Port-Traits (`contextra-ports`)

```rust
pub trait Clock: Send + Sync {
    fn wall_ms(&self) -> i64;
    fn monotonic(&self) -> MonoInstant;           // Deadlines, Leases, Group-Commit-Timer
}
pub trait Entropy: Send + Sync { fn fill(&self, dest: &mut [u8]) -> Result<(), EntropyError>; }

pub trait Vfs: Send + Sync {
    fn open(&self, p: &VPath, o: OpenOptions) -> Result<Box<dyn VFile>>;
    fn rename(&self, from: &VPath, to: &VPath) -> Result<()>;
    fn remove(&self, p: &VPath) -> Result<()>;
    fn list(&self, dir: &VPath) -> Result<Vec<VPath>>;
    fn sync_dir(&self, dir: &VPath) -> Result<()>;
}
pub trait VFile: Send + Sync {
    fn read_at(&self, buf: &mut [u8], off: u64) -> Result<usize>;
    fn write_all_at(&self, buf: &[u8], off: u64) -> Result<()>;
    fn preallocate(&self, len: u64) -> Result<()>;
    fn sync_data(&self) -> Result<()>;
}

pub trait ObjectStore: Send + Sync {
    fn put(&self, kind: ChunkKind, canonical: &[u8]) -> Result<ChunkId>;
    fn get(&self, id: &ChunkId) -> Result<Bytes>;
    fn extent_row(&self, addr: ExtentRowAddr, lanes: LaneMask) -> Result<ExtentRowSlice>;
    fn flush_durable(&self) -> Result<DurableMark>;
}

pub trait DerivedIndex: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn build(&self, snap: &dyn TruthReader, delta: &dyn ChangeStream, budget: &JobBudget)
        -> Result<SegmentDraft>;
    fn search(&self, seg: &SegmentHandle, q: &LaneQuery, allow: &DocSet, k: usize,
        deadline: MonoInstant) -> Result<LaneHits>;
}

pub trait MergeGate: Send + Sync {
    fn id(&self) -> GateId;
    fn check(&self, ctx: &MergeGateContext<'_>) -> GateOutcome;
}

// Inferenz: synchron definiert, ab L5 in Blocking-/Async-Adapter gekapselt.
pub trait Embedder: Send + Sync { fn embed(&self, s: &SpaceId, i: &[&str]) -> Result<Vec<Vec<f32>>>; }
pub trait Reranker: Send + Sync { fn rerank(&self, q: &str, d: &[&str], dl: MonoInstant) -> Result<Vec<f32>>; }
pub trait Llm: Send + Sync   { fn transport(&self) -> Transport; /* Local | Cloud */ }
pub trait Judge: Send + Sync { fn transport(&self) -> Transport; }
pub trait Tokenizer: Send + Sync { fn id(&self) -> TokenizerId; fn count(&self, text: &str) -> u32; }
```

`TokenizerId` enthält den Digest von Vokabular und Regeln; eine geänderte Implementierung ist ein neuer Tokenizer.

### 4.2 Determinismusregeln

- L0–L4 MÜSSEN `SystemTime::now`, `Instant::now`, `std::fs`, `rand::thread_rng` und `HashMap`-Iterationsreihenfolge in beobachtbaren Pfaden meiden (CI-Lint mit Allowlist für `contextra-sys`).
- Hintergrundarbeit läuft über einen injizierten `Scheduler`; die Simulation ersetzt Thread-Pools durch kooperative, seed-gesteuerte Ausführung.
- Float-Ergebnisse (Distanzen, Scores) gehen **nie** in gehashte Objekte ein.

---

## 5. Crate-Workspace

| Schicht | Crate | Inhalt | `unsafe` |
|---|---|---|---|
| L0 | `contextra-types` | IDs, `Hlc`, `Seq`, `Lsn`, `TrustTier`, `TaintFlags`, Fehlerkatalog | verboten |
| L0 | `contextra-ports` | Traits aus §4.1 | verboten |
| L0 | `contextra-wire` | CBE-Kodierung ([SPEC-02](SPEC-02-DATA-MODEL.md) §2), Formatheader | verboten |
| L0 | `contextra-simd` | Distanzen, Bitpacking (AVX2/AVX-512/NEON/Skalar) | isoliert |
| L0 | `contextra-sys` | `pread`/`pwrite`, `io_uring`, Locks, ausgerichtete Puffer, Sync | isoliert |
| L0 | `contextra-crypto` | BLAKE3, HKDF, AEAD, Ed25519, Handle-MAC, Capability-MAC | verboten |
| L1 | `contextra-durable-fs` | Atomare Dateiprotokolle, `FaultVfs`-Referenz | verboten |
| L1 | `contextra-wal` | WAL, Group Commit, Manifest-Log, Superblock | verboten |
| L2 | `contextra-store` | Packs, Extents, Locators, Buffer Pool, Re-Layout | verboten |
| L3 | `contextra-mvcc` | Prolly, `RootSet`, Txn, Merge, GC, Sidecars | verboten |
| L4 | `contextra-vector`, `-text`, `-graph` | Indizes | verboten |
| L5 | `contextra-rank` | Fusion, Kalibration, Redundanz | verboten |
| L5 | `contextra-engine` | Katalog, Planer, Packer, Verifier, Derivations, Retention | verboten |
| L5 | `contextra-ingest` | Parser, Chunker, Revisions-Diff, Injection-Scan | verboten |
| L5 | `contextra-agent` | Sessions, Gates, Quoten, Action-Log | verboten |
| L5 | `contextra-cognition` | Konsolidierungsjobs über Ports | verboten |
| L5 | `contextra-security` | ACL-Auswertung, Capabilities, Egress-Guard | verboten |
| L6 | `contextra`, `-mcp`, `-server`, `-cli` | Fassade, Protokolle, CLI | verboten |
| opt. | `contextra-infer-{onnx,candle,ollama}` | Adapter | isoliert (FFI) |
| opt. | `contextra-sandbox`, `-kvcache` | WASM-Merge/Gates; Prefix-Cache | verboten |
| opt. | `contextra-sqlite`, `contextra-py` | FFI-Frontends | isoliert |
| dev | `contextra-testkit`, `-sim`, `-bench` | Referenzmodell, Simulation, Benchmarks | nur Tests |

**Entfernt:** `contextra-core`, `-db`, `-checkpoint`, `-audit-export`, `-avv-generator`, `-license`, `-adapt`, `-router`.

### 5.1 Bauregeln (CI-erzwungen)

1. Abhängigkeiten nur abwärts; `cargo metadata`-Prüfung in `xtask`.
2. `tokio` ist in L0–L4 verboten (auch transitiv, per `cargo-deny`-Ban mit Wrapper-Allowlist).
3. `#![forbid(unsafe_code)]` in allen als „verboten" markierten Crates. Zero-Copy-Zugriffe nutzen ausschließlich geprüfte APIs (`bytemuck`, `rkyv` mit Validierung). Jeder `unsafe`-Block trägt `// SAFETY:` und steht in der Unsafe-Baseline.
4. Clippy `deny`: `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` in Decodern.
5. Edition 2021, stable ≥ 1.89, `--locked`, `cargo-deny`, `cargo-vet`.
6. `panic = "unwind"`; ein Panic in einem Kern-Thread vergiftet den Store (`repo_poisoned`), er wird nie verschluckt.

---

## 6. Nebenläufigkeitsarchitektur

| Rolle | Anzahl | Aufgabe |
|---|---|---|
| WAL-Writer | 1 | LSN/Seq/HLC-Vergabe, Append, Sync, Publikation in Reihenfolge |
| Branch-Sequencer | logisch 1 je Branch (Mutex, **kein** Thread) | OCC/SSI-Validierung, Einreihung |
| I/O-Pool | konfigurierbar | Blockierende Reads, Pack-Writes |
| Compute-Pool (`rayon`) | ≈ Kerne | Suche, Fold, Index-Builds |
| Job-Scheduler | 1 | Prioritäten, Token-Bucket für I/O und CPU |
| `tokio` | ab L5 | Netzwerk, Timer, Adapter |

**Lock-Hierarchie** (nur in dieser Reihenfolge): `StoreAdmin → ManifestWriter → BranchSequencer → BranchDelta → SidecarShard → BufferPoolShard`. Über Sync-Syscalls und Port-Aufrufe wird kein Lock gehalten (Ausnahme: `ManifestWriter` über den eigenen Manifest-Sync).

Leser holen `Arc<BranchState>` wait-free über `ArcSwap` und registrieren eine Epoche; physische Freigabe erfolgt erst nach Ablauf aller Epochen und Leases.

---

## 7. Abgrenzung

| Aspekt | LSM-KV | Prolly-SQL | Vektor-DB | Contextra |
|---|---|---|---|---|
| Primitive | Key→Bytes | Zeilen | Vektor + Payload | Cell (Content/Meta/Edges) + Zoom |
| Versionierung | Snapshots | Branch/Merge | Dataset-Versionen | Session-Branches, Gates, Bitemporalität |
| Konsistenz Index/Daten | — | B-Baum | variiert | `RootSet` + `IndexManifest` + Validierung |
| LLM-Bezug | — | — | Top-k | Budget, Handles, Abstention, Taint |

Die Tabelle beschreibt typische Kategorien, keine Messung einzelner Produkte.

---

## 8. Architektur-Abnahme

- CI-Prüfung der Schicht-, `tokio`- und `unsafe`-Regeln ist grün und blockierend.
- Jede INV-ID ist in `contextra-sim` oder einer Suite referenziert; `xtask invariants` meldet unreferenzierte IDs.
- Jede Abweichung von einem SOLL besitzt einen Registereintrag in [SPEC-00](SPEC-00-INDEX.md).

---

*Weiter: [SPEC-02 Datenmodell](SPEC-02-DATA-MODEL.md)*
