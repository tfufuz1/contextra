# Contextra Baukasten – Normative Gesamtspezifikation v4.1

```
Dokument:  Normative Soll-Zustands-Spezifikation des Baukastens (Repository-Ebene & Crate-Ebene)
Fassung:   4.1 – vollständige Ausarbeitung der Soll-Zustände, Module, Invarianten und Schnittstellen
Stand:     2026-11-08
Status:    Normative Entscheidungsfassung (verbindlich für Architektur, Refactoring und Neuentwicklungen)
Ablage:    docs/SPEC.md
```

## 0. Zweck und Geltung

**Contextra ist ein privates Repository aus isolierten, produktionsreifen, wiederverwendbaren Rust-Bausteinen** für anspruchsvolle Systemanwendungen im Bereich von LLM-Kernen, Retrieval-Augmented Generation (RAG), Durability, Concurrency, Vektormathematik und Privacy/Compliance-Primitiven.

**Kernaussage:**
Contextra ist **kein monolithisches Endnutzer-Produkt, keine Standard-Bibliothek mit globaler Fassade und keine verwobene Ökosystem-Plattform**. Aus den Bausteinen werden in *anderen, dedizierten Repositories* maßgeschneiderte Produkte und Laufzeitdienste komponiert. Jeder Baustein muss so beschaffen sein, dass er durch einfaches Kopieren der Quelldateien oder des Crates (Copy-Paste / Export-Readiness) direkt in Fremdprojekten kompiliert und eingesetzt werden kann.

Diese Spezifikation regelt verbindlich und normativ:
1. Den Baukasten-Standard und die maschinell prüfbaren Kriterien für isolierte Bausteine (§1, §2).
2. Die architektonischen Leitprinzipien für Systemprogrammierung, Concurrency, Durability und Datenflüsse (§3).
3. Die standardisierten Konventionen für Fehlerbehandlung, Feature-Flags, Typtrennung und Trait-Schnittstellen (§4).
4. Die Klassifikation aller Workspace-Crates in Klassen A, B, C und T (§5).
5. Die strikte Entkopplung und Kapselung von Löschketten, DSGVO- und Beweislogik (§6).
6. Den autonomen Export- und Kopiermechanismus (§7).
7. Die Qualitäts-Gates, Ratchets und maschinellen Prüfwerkzeuge (§8, §9).
8. **Die lückenlose, modul- und funktionsgenaue Spezifikation jedes einzelnen Crates im Soll-Zustand (§10–§14).**
9. Das normative Entscheidungsregister (§15) und offene Punkte (§16).

---

## 1. Ausgangslage und Reorganisationsbefund

Die historische Codebasis zeichnete sich durch eine enge zyklische und vertikale Verflechtung über zentrale Typen- und Port-Hubs (`contextra-types`, `contextra-ports`) aus. Bausteine, die rein mathematische, algorithmische oder I/O-spezifische Aufgaben erfüllen (z. B. SIMD-Kernel, RRF-Fusion, HNSW-Graph, BM25, Parser), wiesen unzulässige Abhängigkeiten zu Transaktions-IDs (`TxId`), DSGVO-Löschmarkern (`Purger`, `WitnessLayer`, `DeletionProof`) oder Mandantenstrukturen auf.

Die normative Reorganisation eliminiert diese Verflechtungen vollständig:
- Reines Domänenwissen fremder Schichten wird aus den Kernbausteinen entfernt.
- Typische Hub-Abhängigkeiten werden durch **vertikale Autarkie**, lokale Enums/Structs und Standardtypen (`&[f32]`, `&[u8]`, `&str`, `u64`, `usize`) ersetzt.
- Konvertierungen zwischen Bausteinen erfolgen ausschließlich in **Adaptern** oder in der Kompositionsschicht (Klasse C), niemals in Klasse-A-Bausteinen.

---

## 2. Definition „Baustein“ (Klasse-A-Standard)

Ein Crate gilt nur dann als **Baustein (Klasse A)**, wenn es die folgenden Kriterien lückenlos und maschinell nachweisbar erfüllt:

| ID | Kriterium | Normative Anforderung | Verifikation |
|---|---|---|---|
| **B1** | **Keine internen Abhängigkeiten** | 0 Abhängigkeiten zu anderen `contextra-*`-Crates in `[dependencies]` und `[build-dependencies]`. | Gate G1 (`cargo metadata`) |
| **B2** | **Keine internen Dev-Deps** | 0 Dev-Abhängigkeiten zu `contextra-*`. Testhilfen liegen ausschließlich im Crate (`tests/common/` oder `#[cfg(test)]`). `contextra-testkit` ist für Klasse A strikt verboten. | Gate G1 |
| **B3** | **API in Standardtypen** | Öffentliche Schnittstellen akzeptieren und liefern Standard-Rust-Typen (`&[f32]`, `&[u8]`, `&str`, `u64`, `usize`, `core::time::Duration`) sowie crate-eigene Typen. Keine Fremdtypen in der öffentlichen API. | Gate G2 (`cargo public-api`) |
| **B4** | **Lokales Fehlermodell** | Jedes Crate definiert genau einen primären Fehlertyp (`Error`, `thiserror`, `#[non_exhaustive]`), der `std::error::Error` implementiert. Kein Re-Export oder Import globaler Fehler-Enums. | Gate G2 |
| **B5** | **Isolierter Standalone-Build** | Das Crate baut und testet vollständig isoliert aus einem leeren Verzeichnis heraus mit `--no-default-features` **und** Default-Features. | Gate G3 (`tools/export-crate`) |
| **B6** | **Feature-Entkopplung** | Schwere Abhängigkeiten (`candle`, `ort`, `wasmtime`, `tokio`, `reqwest`, `pyo3`, komplexe Parser) liegen hinter optionalen Features. `default = []` oder `["std"]`. | Gate G4 |
| **B7** | **Domänenreinheit** | Indizes, Parser und Rechenkerne kennen keine DSGVO-Regeln, keine Mandanten, keine Transaktions-IDs, keine Audit-Ketten und keine Löschbeweise (§6). | Gate G5 (`grep`-Filter) |
| **B8** | **Vollständige Dokumentation** | `README.md` mit Copy-Paste-Anleitung, `SPEC.md`, Moduldokumentation (`//!`), Doc-Kommentare (`///`) mit `# Errors`, `# Panics`, `# Safety` für alle `pub`-Symbole sowie Criterion-Benchmarks. | Gate G6 |
| **B9** | **Zero-Panic-Garantie** | Kein `unwrap()`, `expect()`, `panic!()` oder ungeprüftes Slicing im Produktionscode (`#![deny(clippy::indexing_slicing, clippy::unwrap_used)]`). Fehler werden ausnahmslos über `Result` propagiert. | Gate G10 (Clippy-Gate) |
| **B10** | **Unsafe-Kapselung** | `unsafe` ist grundsätzlich verboten (`#![forbid(unsafe_code)]`), außer in explizit ausgewiesenen Unsafe-Inseln (`simd`, `sys`, `wire`). Jeder Block trägt einen nachweisbaren `// SAFETY:`-Kommentar. | `tests/unsafe_islands.rs` |
| **B11** | **Umwelt-Determinismus** | Zeit, Entropie/Zufall und IDs werden als Parameter oder Traits (`Clock`, `Rng`) von außen injiziert. Kein harter Aufruf von `SystemTime::now()` oder globalem Zufall im Kern. | Clippy `disallowed_methods` |
| **B12** | **Lizenzautarkie** | Jedes Crate enthält eigenständige Lizenzdateien (`LICENSE-MIT`, `LICENSE-APACHE`) und eine explizite MSRV (`rust-version = "1.89"`). | Gate G3 |

---

## 3. Architektonische Leitprinzipien

### 3.1 Vertikale Autarkie vor DRY
Wiederholungen trivialer Datenstrukturen (z. B. `Metric`, `ScoredPoint`, `Token`, `SliceRange`) zwischen voneinander unabhängigen Crates sind erwünscht und bewusst vorgeschrieben. DRY (Don't Repeat Yourself) über Crate-Grenzen hinweg erzeugt unzulässige Kopplung. Konvertierungen erfolgen an den Systemgrenzen in Adaptern.

### 3.2 Abhängigkeitsrichtung: Adapter & Komposition → Baustein
Die Abhängigkeitskante zeigt immer von der integrierenden Schicht auf den autarken Baustein. Ein Baustein darf niemals sein Adapter-Crate oder eine höhere Orchestrierungsschicht kennen.

### 3.3 Statischer Dispatch auf Leistungspfaden
Hot-Paths (SIMD-Distanzen, HNSW-Graph-Traversal, WAND-Posting-Scans, RRF-Rank-Fusion, Radix-Lookups) arbeiten ausschließlich mit monomorphisierten Generics und Inlining (`#[inline]`). Dynamischer Dispatch (`dyn Trait`) ist auf lose gekoppelte Plugin-Grenzen und I/O-Ränder beschränkt.

### 3.4 Sync-Kern, Async-Schale
Rechnerische Algorithmen, Datenstrukturen, Indizes und Enkapsulierungen sind vollständig synchron und frei von asynchronen Runtimes (`tokio`, `async-std`). Asynchrone Schnittstellen sind ausschließlich Bausteinen vorbehalten, deren ureigener Zweck Netzwerk- oder Prozess-I/O ist (`contextra-infer-ollama`), und müssen dort hinter Feature-Flags isoliert werden.

### 3.5 Durability- und Concurrency-Disziplin
- **Lock-Hierarchie:** Es existiert eine strikte, globale Akquisitionsreihenfolge für Locks, um Deadlocks mathematisch auszuschließen:
  `Dokument-Lock -> Commit-Barriere -> Purge-Barriere -> Journal/WAL -> Speicherebene`.
- **Kein Lock über Await:** Niemals darf ein synchroner Mutex- oder RwLock-Guard über einen `.await`-Punkt gehalten werden.
- **Fail-Closed & Poisoning:** Fsync- und I/O-Fehler auf Persistenzebene führen zur sofortigen Poisoning-Markierung der betroffenen Handles (`INV-FSYNC-1`). Es gibt keine stillen Wiederholungsversuche bei beschädigtem Speicherzustand.

---

## 4. Konventionen für Schnittstellen und Fehler

### 4.1 Einheitliches Fehlermodell
Jeder Baustein deklariert ein eigenes Fehler-Enum:
```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("ungültige Eingabedaten: {0}")]
    InvalidInput(&'static str),
    #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")]
    CapacityExceeded { requested: usize, maximum: usize },
    #[error("E/A-Fehler aufgetreten: {0}")]
    Io(#[from] std::io::Error),
    #[error("interner Zustandskonflikt: {0}")]
    StateConflict(&'static str),
}
pub type Result<T> = core::result::Result<T, Error>;
```

### 4.2 Standardisierte Feature-Flags
| Feature-Name | Normativer Zweck |
|---|---|
| `std` | Standardbibliothek. Aktiviert Betriebssystem-Typen. Ermöglicht künftiges `no_std`/`alloc` durch Deaktivierung. |
| `serde` | Serialisierung und Deserialisierung für alle öffentlichen Datenstrukturen. |
| `tracing` | Strukturiertes Logging via `tracing`-Crate. Bei Nichtaktivierung entfallen alle Log-Makros spurlos. |
| `inspect` | Gibt Lesezugriff auf interne Index- und Speicherstrukturen frei (nur für Diagnosetools und Auditoren). |
| `experimental` | Isoliert in Entwicklung befindliche oder noch nicht verdrahtete Algorithmen (E-07). |

### 4.3 Normative Index- und Storage-Traits

#### Kern-Index-Schnittstelle (Klasse A)
```rust
pub trait Index<Id, Query> {
    type Hit;
    fn insert(&mut self, id: Id, payload: &[u8]) -> Result<()>;
    fn remove(&mut self, id: &Id) -> Result<bool>;
    fn search(&self, query: &Query, limit: usize) -> Result<Vec<Self::Hit>>;
}

pub trait Maintain {
    type Stats;
    fn compact(&mut self) -> Result<Self::Stats>;
    fn vacuum(&mut self) -> Result<()>;
}
```

#### Transaktions-Schnittstelle (Klasse B)
```rust
pub trait TransactionalIndex<Id, Query, TxId>: Index<Id, Query> {
    fn insert_versioned(&mut self, tx_id: TxId, id: Id, payload: &[u8]) -> Result<()>;
    fn remove_versioned(&mut self, tx_id: TxId, id: &Id) -> Result<bool>;
    fn commit_tx(&mut self, tx_id: TxId) -> Result<()>;
    fn rollback_tx(&mut self, tx_id: TxId) -> Result<()>;
}
```

---

## 5. Crate-Klassifikation und Architektur-Ringe

### 5.1 Klasse A – Autarke Bausteine (Vollständige Entkopplung B1–B12)
Autarke Crates ohne interne Abhängigkeiten zu anderen `contextra-*`-Paketen.
- **System & Low-Level:** `contextra-sys`, `contextra-simd`, `contextra-wire`
- **Retrieval & Suche:** `contextra-rank`, `contextra-text`, `contextra-vector`, `contextra-graph`
- **Cache & Persistenz-Bausteine:** `contextra-kvcache`, `contextra-durable-fs`
- **Kryptographie & Compliance:** `contextra-crypto`, `contextra-erasure`, `contextra-privacy`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`
- **Ingestion & Sandboxing:** `contextra-ingest`, `contextra-sandbox`, `contextra-adapt`
- **Inferenz-Treiber:** `contextra-infer-onnx`, `contextra-infer-ollama`, `contextra-infer-candle`

### 5.2 Klasse B – Storage-Bundle (Gemeinsam kopierbare Speicher-Einheit)
Enthält den gemeinsamen Transaktions- und LSM-Kern:
`contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-store`, `contextra-checkpoint`.
- Diese Crates teilen gemeinsame Transaktions- (`TxId`) und Snapshot-Semantiken.
- Klasse-A-Bausteine dürfen keine Typen aus Klasse B importieren.

### 5.3 Klasse C – Kompositions- und Integrationsschicht
Anwendungsschichten, Adapter und Endnutzer-Bindings:
`contextra`, `contextra-db`, `contextra-engine`, `contextra-cognition`, `contextra-agent`, `contextra-mcp`, `contextra-py`, `contextra-router`.
- Keine originäre Geschäfts- oder Indexlogik; dient ausschließlich der Verdrahtung autarker Bausteine.

### 5.4 Klasse T – Test- und Benchmark-Werkzeuge
`contextra-testkit` (Mock-Storage, Fault-Injektion, Clock-Fälschung) und `contextra-bench`. Nur für Tests innerhalb Klasse B/C zugelassen.

---

## 6. Vollständige Kapselung der Löschlogik und DSGVO

### 6.1 Zuständigkeit und Kapselungsgebot
Sämtliche Kaskadenlöschungen, Löschsagas, kryptographischen Vernichtungsnachweise (Crypto-Shredding), Restore-Guards, Witness-Minting und DSGVO-Prüfungen sind ausschließlich in `contextra-erasure` konzentriert. Datenhaltende Indizes (`vector`, `text`, `graph`, `store`) implementieren lediglich neutrale, technische Operationen: `insert`, `remove` und `compact` (physikalische Slot-Freigabe).

### 6.2 `contextra-erasure` als autarker Löschkern
- Verwaltet die Saga-Zustandsmaschine (Pending, KeysRevoked, StoragePurged, Verified, Committed).
- Verwaltet den kryptographischen Tresor (`KeyVault`) für Scope- und Dokumentenschlüssel.
- Erstellt und verifiziert kryptographisch signierte Löschzertifikate (Ed25519-Signatur über Deletion-Merkle-Tree).

### 6.3 Adapter-Trennung via `contextra-erasure-adapters`
Die Anbindung realer Indizes an die Lösch-Engine erfolgt über ein separates Adapter-Crate `contextra-erasure-adapters`:
- Übersetzt zwischen den neutralen Index-Schnittstellen (`Index::remove`, `Maintain::compact`) und den Port-Anforderungen (`Purger`, `LayerScanner`, `WitnessLayer`).
- Indizes bleiben frei von Kenntnis über Löschsagas, Zeitstempel, Prüfsummen oder Nachweise.

---

## 7. Export- und Kopiermechanismus (Copy-Paste-Readiness)

Jeder Klasse-A-Baustein muss über das automatisierte Werkzeug `tools/export-crate` in ein beliebiges externes Repository kopierbar sein:
- Auflösung aller `workspace = true`-Vererbungen zu expliziten Versionen.
- Beilegen der kanonischen Lizenzdateien.
- Kompilierung und Durchlauf aller Tests im Zielverzeichnis ohne Netzwerkzugriff (`--locked --offline`).

---

## 8. Verifikations-Gates und Gütekriterien

| Gate | Name | Normatives Kriterium | Prüfbefehl |
|---|---|---|---|
| **G1** | Crate-Klassen-Gate | Klasse A besitzt 0 interne Deps; Klasse B nur Kanten innerhalb Klasse B. | `python3 tools/gate_dependencies.py` |
| **G2** | Öffentliche API | Keine internen Crate-Typen in der öffentlichen Signatur von Klasse A. | `cargo public-api` |
| **G3** | Export-Test | Exportiertes Crate baut und besteht Tests in isolierter Umgebung. | `tools/export-crate <crate> /tmp/test-export && cargo test` |
| **G4** | Feature-Matrix | Baut mit `--no-default-features` und `--all-features`. | `cargo check -p <crate> --no-default-features` |
| **G5** | Domänenreinheit | Kein Vorkommen verbotener Begriffe (DSGVO, Saga, Witness) in Indizes. | `grep -rnE "(Purger|WitnessLayer|DeletionProof)" crates/<crate>/src` |
| **G6** | Dokumentation | `README.md`, `SPEC.md`, Moduldokumentation und Benchmarks vorhanden. | `cargo test --doc -p <crate>` |
| **G7** | Minimal-Komposition | Minimale Zusammensetzungen von Bausteinen kompilieren erfolgreich. | `cargo check -p contextra-engine --no-default-features` |
| **G8** | Meta-Test | Gate-Prüfskripte besitzen funktionierende Negativ-Tests. | `cargo test -p contextra-arch-tests` |
| **G9** | Zero-Warning | Kompilation unter `RUSTFLAGS="-D warnings"` ohne Warnungen. | `cargo check --workspace --all-targets` |
| **G10** | Unsafe & Panic | Keine unzulässigen Unsafe-Blöcke; Zero-Panic im Produktionscode. | `tests/unsafe_islands.rs` & Clippy-Gate |

---

## 9. Qualität pro Baustein

- **Unit-Tests:** 100 % Abdeckung der öffentlichen Schnittstellenfunktionen und Fehlerpfade.
- **Property-Based Testing:** Einsatz von `proptest` zur Verifikation mathematischer Invarianten (Dreiecksungleichung, Kommutativität, Invertierbarkeit).
- **Referenz-Orakel:** Gegenüberstellung optimierter Pfade mit unoptimierten, mathematisch trivialen Referenzmodellen (z. B. SIMD vs. Scalar-F32, HNSW vs. Flat-Brute-Force).
- **Fault-Injection:** Gezielte Simulation von E/A- und Disk-Ausfällen auf Dateisystemebene ohne externe Testframeworks.

---

## 10. Normative Spezifikation: Foundation & Low-Level (Klasse A)

### 10.1 `contextra-sys` (Systemabstraktionen & FFI-Unsafe-Insel)
- **Zweck:** Kapselt alle plattformspezifischen OS-Aufrufe (`libc`, `mman`, `sys/resource`, `io_uring`) als deklarierte und isolierte Unsafe-Insel unter strengster Auditierung. Kein anderes Crate im Baukasten darf direkte FFI- oder unsafe System-Calls durchführen.
- **Modulstruktur:**
  - `sync`: Dauerhafte Dateisynchronisation (`fsync`, `fdatasync`, Verzeichnissynchronisation).
  - `memlock`: Speicherblockierung im RAM (`mlock`, `munlock`, `MADV_DONTDUMP`, `MADV_DODUMP`) zur Verhinderung des Auslagerns sensibler Schlüssel ins Swap.
  - `hardening`: Prozess-Isolation und Schutz vor Core-Dumps (`RLIMIT_CORE = 0`, `PR_SET_DUMPABLE = 0`).
  - `perm`: Dateisystemberechtigungen (POSIX ACLs, strikte umask-Restriktionen auf `0700` bzw. `0600`).
  - `rename`: Atomares Umbenennen und Ersetzen von Dateien via `renameat2(RENAME_NOREPLACE)` oder Standard-POSIX-Fallback.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn durable_flush_file(fd: RawFd) -> Result<(), SysError>;
  pub fn durable_flush_dir(path: &Path) -> Result<(), SysError>;
  pub fn lock_memory_region(ptr: *const u8, len: usize) -> Result<(), SysError>;
  pub fn unlock_memory_region(ptr: *const u8, len: usize) -> Result<(), SysError>;
  pub fn apply_process_hardening(mode: HardeningMode) -> Result<(), SysError>;
  pub fn atomic_replace_file(src: &Path, dst: &Path) -> Result<(), SysError>;
  ```
- **Fehlermodell (`SysError`):** `Io(io::Error)`, `PermissionDenied`, `PoisonedHandle`, `UnsupportedPlatform`.
- **Invarianten:**
  - `INV-SYS-UNSAFE-1`: Jedes `unsafe`-Statement muss mit einem validen `// SAFETY:`-Kommentar begründet und im Unsafe-Register registriert sein.
  - `INV-SYS-NO-PANIC-1`: Keine Systemfunktion darf paniken; OS-Fehler werden immer über `Result<T, SysError>` propagiert.

### 10.2 `contextra-simd` (Vektor-Distanzkerne)
- **Zweck:** Autarke, hardwarebeschleunigte SIMD-Berechnung von Vektordistanzen (Kosinus-Ähnlichkeit, euklidische Distanz L2, Skalarprodukt / Dot Product, Manhattan L1, Hamming) mit dynamischer CPU-Feature-Erkennung (AVX-512, AVX2, NEON, Scalar-Fallback).
- **Modulstruktur:**
  - `dispatch`: Dynamische Laufzeitauswahl des schnellsten Instruktionssatzes via CPUID/AT_HWCAP.
  - `kernels::avx512`: AVX-512F/BW/VNNI-Kerne mit 512-Bit-Registern und 16/64-Element-Unrolling.
  - `kernels::avx2`: AVX2/FMA-Kerne für Standard-x86_64-Server.
  - `kernels::neon`: ARM64-NEON-Kerne mit optimierten Fused-Multiply-Add-Pipelines.
  - `kernels::scalar`: Portabler, numerisch stabiler Referenz-Fallback.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn dot_product_f32(a: &[f32], b: &[f32]) -> f32;
  pub fn euclidean_distance_squared_f32(a: &[f32], b: &[f32]) -> f32;
  pub fn cosine_similarity_f32(a: &[f32], b: &[f32]) -> f32;
  pub fn dot_product_u8(a: &[u8], b: &[u8]) -> u32;
  pub fn hamming_distance(a: &[u64], b: &[u64]) -> u32;
  ```
- **Invarianten:**
  - `INV-SIMD-SLICING-1`: Die Vektorlängen `a.len()` und `b.len()` müssen exakt übereinstimmen; Dimensionsmismatch führt zu `f32::NAN` oder definierten Fehler-Rückgaben, niemals zu Speicherfehlern.
  - `INV-SIMD-CROSS-1`: Für alle Dimensionen $D \in [1, 4096]$ darf die Differenz zwischen SIMD- und Scalar-Ergebnis maximal $10^{-5}$ betragen.

### 10.3 `contextra-wire` (Zero-Copy IPC & Serialisierung)
- **Zweck:** Zero-Copy-Serialisierung und Framing für Speicher-, Netzwerk- und Prozessgrenzen auf Basis robuster Byte-Frames und optionaler FlatBuffers-Schemata.
- **Modulstruktur:**
  - `frame`: 64-Byte-Norm-Header (Magic `CTXW`, Version, Payload-Länge, BLAKE3-Prüfsumme, Verschlüsselungs-Flags).
  - `reader`: Zero-Copy Slice-Parser mit strikter Validierung von Offset- und Längengrenzen.
  - `writer`: Allokationsfreier Frame-Builder mit vorallokierten Speicherpuffern.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn encode_frame<W: io::Write>(writer: &mut W, msg_type: u16, payload: &[u8]) -> Result<usize, WireError>;
  pub fn decode_frame_header(buf: &[u8; 64]) -> Result<FrameHeader, WireError>;
  pub fn verify_payload_integrity(header: &FrameHeader, payload: &[u8]) -> bool;
  ```
- **Fehlermodell (`WireError`):** `BufferTooSmall`, `InvalidMagic`, `ChecksumMismatch`, `LengthOverflow`, `CorruptFrame`.
- **Invarianten:**
  - `INV-WIRE-ALIGN-1`: Alle Payload-Offsets sind an 8-Byte-Grenzen ausgerichtet.
  - `INV-WIRE-BOUNDS-1`: Payload-Längen dürfen `MAX_PAYLOAD_BYTES` (128 MiB) nicht überschreiten.

---

## 11. Normative Spezifikation: Indizes, Ranking & Graph (Klasse A)

### 11.1 `contextra-rank` (Ergebnis-Fusion & Kalibrierung)
- **Zweck:** Mathematische Kernbibliothek für Multi-Signal-Fusion (Vektor, BM25, Graph, Metadaten), Score-Normalisierung und Konformitäts-Kalibrierung.
- **Modulstruktur:**
  - `fusion`: Reciprocal Rank Fusion (RRF), margin-moduliertes RRF (mRRF) und gewichtete lineare Signal-Kombination.
  - `calibration`: Isotonische Regression (PAVA-Algorithmus), Platt-Scaling und Adaptive Conformal Calibration.
  - `diversity`: Maximal Marginal Relevance (MMR) zur Reduktion von Redundanz in Trefferlisten.
  - `explain`: Deterministische, sprachlich präzise Begründungsgenerierung für Treffer-Rangfolgen.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn rrf_fuse(rankings: &[Vec<u64>], k: f32) -> Vec<(u64, f32)>;
  pub fn mrrf_fuse(rankings: &[Vec<(u64, f32)>], k: f32, margin: f32) -> Vec<(u64, f32)>;
  pub fn mmr_select(candidates: &[u64], embeddings: &[&[f32]], query: &[f32], lambda: f32, top_k: usize) -> Vec<u64>;
  ```
- **Invarianten:**
  - `INV-RANK-MONOTONE-1`: Die Score-Fusion muss monoton bezüglich der Einzelränge sein.
  - `INV-RANK-BOUND-1`: Alle normalisierten Scores liegen im Intervall $[0.0, 1.0]$.

### 11.2 `contextra-text` (Volltextsuche & BM25)
- **Zweck:** Autarker invertierter Index für lexikalische Volltextsuche mit Okapi BM25, Block-Max-WAND-Beschleunigung und Tokenisierung.
- **Modulstruktur:**
  - `tokenizer`: Panic-freie Zerlegung in Unicode-Wörter, Normalisierung (Kleinschreibung, Akzententfernung) und Stoppwortfilterung.
  - `posting`: Gepackte Posting-Listen (Delta-VByte-Kompression für Doc-IDs und Term-Frequenzen).
  - `wand`: Block-Max Weak AND (WAND) Algorithmus für beschleunigte Top-K-Abfragen ohne Durchsuchen aller Treffer.
  - `segment`: Segmentierung und Verschmelzung (Merge) unveränderlicher Text-Segmente.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn index_document(&mut self, doc_id: u64, text: &str) -> Result<(), TextError>;
  pub fn search_bm25(&self, query: &str, top_k: usize, k1: f32, b: f32) -> Result<Vec<(u64, f32)>, TextError>;
  pub fn compact_segments(&mut self) -> Result<CompactionStats, TextError>;
  ```
- **Invarianten:**
  - `INV-TEXT-DETERMINISM-1`: Die Tokenisierung identischer Texte liefert auf allen Plattformen identische Term-IDs.
  - `INV-TEXT-WAND-1`: Der WAND-Algorithmus liefert exakt dieselben Top-K Dokumente wie die erschöpfende BM25-Evaluation.

### 11.3 `contextra-vector` (HNSW-Vektorindex)
- **Zweck:** Hierarchischer Navigable Small World (HNSW) Graphindex für hochdimensionale Vektoren mit echten Löschungen und Graphreparatur.
- **Modulstruktur:**
  - `hnsw`: Kern-Datenstruktur mit hierarchischen Layern, Einstiegsknoten und Nachbarschaftslisten.
  - `deletion`: Synchroner Knoten-Entfernungskern (`remove_with_graph_repair`) mit Nachbarschafts-Re-Wiring 2. Ordnung.
  - `quantize`: Skalare Quantisierung (SQ8) und RaBitQ zur Reduktion des Speicherbedarfs um bis zu 75 %.
  - `storage`: Lokales Vektorfeld mit Zeroizing von gelöschten Slots.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn insert(&mut self, doc_id: u64, vector: &[f32]) -> Result<(), VectorError>;
  pub fn search(&self, vector: &[f32], top_k: usize, ef_search: usize) -> Result<Vec<(u64, f32)>, VectorError>;
  pub fn remove_with_graph_repair(&mut self, doc_id: u64) -> Result<DeletionStats, VectorError>;
  ```
- **Invarianten:**
  - `INV-VEC-CONNECTIVITY-1`: Nach der Entfernung eines Knotens darf kein Nachbarknoten isoliert werden, wenn alternative Pfade 2. Ordnung existieren.
  - `INV-VEC-ZEROIZE-1`: Bei Löschung eines Vektors werden die Rohdaten im Speicher sofort genullt.

### 11.4 `contextra-graph` (CSR-Wissensgraph)
- **Zweck:** Cache-effizienter Wissensgraph auf Basis von Compressed Sparse Row (CSR) für Beziehungsrecherche, Pfadtraversierung und Personalized PageRank (PPR).
- **Modulstruktur:**
  - `csr`: Speicherstruktur mit `row_offsets` und `col_indices` für schnellen Kanten-Scan.
  - `ppr`: Forward-Push-Algorithmus für näherungsweisen Personalized PageRank.
  - `community`: Knotengruppierung über Label-Propagation.
  - `hyperedge`: Abbildung von n-ären Relationen über Hyperkanten-Projektionen.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn add_edge(&mut self, src: u64, dst: u64, weight: f32) -> Result<(), GraphError>;
  pub fn compute_ppr(&self, seeds: &[u64], alpha: f32, epsilon: f32) -> Result<Vec<(u64, f32)>, GraphError>;
  pub fn neighbors(&self, node: u64) -> Result<&[u64], GraphError>;
  ```
- **Invarianten:**
  - `INV-GRAPH-SORTED-1`: Die Kanten jedes Knotens im CSR-Array sind strikt aufsteigend nach Ziel-ID sortiert.

---

## 12. Normative Spezifikation: Persistenz, Cache, Krypto & Privacy (Klasse A)

### 12.1 `contextra-durable-fs` (Durable I/O & Fsync)
- **Zweck:** Robuste Dateisystem-Abstraktion mit garantierter Persistenz, Fsync-Fehlerbehandlung und Verzeichnis-Sync.
- **Modulstruktur:**
  - `vfs`: Synchroner `Vfs`-Trait (`read`, `write_atomic`, `sync_dir`, `remove`).
  - `poison`: Handle-Poisoning bei EIO/Fsync-Fehlern (`INV-FSYNC-1`).
  - `atomic`: Sicheres Atomic-File-Replacement via Scratchpad und atomares Umbenennen.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), FsError>;
  pub fn sync_directory(dir: &Path) -> Result<(), FsError>;
  pub fn verify_file_persistence(path: &Path) -> Result<bool, FsError>;
  ```
- **Invarianten:**
  - `INV-FSYNC-1`: Tritt bei `fsync` ein OS-Fehler auf, wird der Handle sofort vergiftet (Poisoned). Alle weiteren Operationen schlagen fehl.

### 12.2 `contextra-kvcache` (KV-Cache für LLM-Tokens)
- **Zweck:** Verwaltung und hierarchisches Prefix-Caching von Key-Value-Tensoren für LLM-Inferenz ohne Bindung an spezifische Runtimes.
- **Modulstruktur:**
  - `radix`: Radix-Baum zur Identifikation gemeinsamer Prompt-Präfixe über Session-Grenzen hinweg.
  - `allocator`: Paged Attention Block-Allokator mit Fixed-Size-Memory-Pools.
  - `eviction`: Least-Recently-Used (LRU) und Attention-basierte Eviction-Strategien.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn match_prefix(&self, tokens: &[u32]) -> (Option<CacheHandle>, usize);
  pub fn allocate_blocks(&mut self, count: usize) -> Result<Vec<BlockId>, CacheError>;
  pub fn release_blocks(&mut self, blocks: &[BlockId]) -> Result<(), CacheError>;
  ```
- **Invarianten:**
  - `INV-KV-NO-OVERCOMMIT-1`: Die Blockallokation schlägt sauber fehl, bevor das physische Speicherkontingent überschritten wird.

### 12.3 `contextra-crypto` (Kryptographische Primitive)
- **Zweck:** Bereitstellung ausschließlich standardisierter Krypto-Primitive ohne Abhängigkeit von Framework-Typen.
- **Modulstruktur:**
  - `aead`: AES-256-GCM-SIV (RFC 8452) für deterministische, manipulationssichere Verschlüsselung.
  - `kdf`: Argon2id (Speicher $64\,	ext{MiB}$ bis $4\,	ext{GiB}$) und HKDF-SHA256 für Schlüssenableitung.
  - `integrity`: BLAKE3 und HMAC-SHA256 für fälschungssichere Append-Only-Logs.
  - `keys`: `ZeroizeOnDrop`-Wrapper für Master- und Session-Keys.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn encrypt_gcm_siv(key: &SecretKey, nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError>;
  pub fn decrypt_gcm_siv(key: &SecretKey, nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError>;
  pub fn derive_subkey(master: &SecretKey, context: &[u8], out: &mut [u8]) -> Result<(), CryptoError>;
  ```
- **Invarianten:**
  - `INV-CRYPTO-ZEROIZE-1`: Alle Speicherstrukturen, die Schlüsselmaterial halten, überschreiben dieses beim Droppen mit Nullen.

### 12.4 `contextra-privacy` (PII-Erkennung & Egress-Guard)
- **Zweck:** Autarke Erkennung und Schwärzung personenbezogener Daten (PII) und Egress-Filtering.
- **Modulstruktur:**
  - `scanner`: Regex- und Heuristik-Scanner für E-Mail, IBAN, Telefon, IP, Kreditkarten und Namen.
  - `redactor`: Maskierungs-Routinen (Ersetzen durch Pseudonyme oder statische Token wie `[REDACTED_EMAIL]`).
  - `egress_guard`: Fail-closed Filter vor externen Cloud-Aufrufen.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn scan_pii(text: &str) -> Vec<PiiMatch>;
  pub fn redact_pii(text: &str, matches: &[PiiMatch]) -> String;
  pub fn validate_clean_egress(text: &str) -> Result<(), PrivacyViolation>;
  ```
- **Invarianten:**
  - `INV-PRIVACY-FAILCLOSED-1`: Erkennt der Egress-Guard unmaskierte PII-Muster, bricht der Vorgang mit Fehler ab; Daten fließen niemals unkontrolliert ab.

---

## 13. Normative Spezifikation: Ingestion, Sandboxing & Inferenz (Klasse A)

### 13.1 `contextra-ingest` (Dokumenten-Parsing & Chunking)
- **Zweck:** Ressourcengedeckeltes, zero-panic Dokumentenparsing (PDF, DOCX, ODF, HTML, TXT) und Chunking.
- **Modulstruktur:**
  - `detect`: MIME-Typ-Erkennung über Magic Bytes, strikt begrenzt auf max. $64\,	ext{KiB}$ Lesepuffer (`MAX_DETECT_BYTES`).
  - `parse`: Format-Parser ohne Dekomprimierungsbomben (ZIP-Schutz via Kompressions-Ratio-Limits).
  - `chunk`: Semantisches Chunking nach Absätzen, Sätzen und Tokengrenzen mit Overlap.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn detect_content_type(stream: &mut impl io::Read) -> Result<ContentType, IngestError>;
  pub fn parse_document(stream: &mut impl io::Read, content_type: ContentType) -> Result<ExtractedDoc, IngestError>;
  pub fn chunk_text(text: &str, chunk_size: usize, chunk_overlap: usize) -> Vec<TextChunk>;
  ```
- **Invarianten:**
  - `INV-INGEST-BOUNDED-1`: Dateierkennung liest niemals mehr als $64\,	ext{KiB}$ ein.
  - `INV-INGEST-NO-BOMB-1`: Dekomprimierung bricht ab, wenn entpackte Daten das 100-Fache des Archivs überschreiten.

### 13.2 `contextra-sandbox` (WASM-Ausführungskapselung)
- **Zweck:** Isolierte Ausführung von User-Code und Erweiterungen via WebAssembly (Wasmtime).
- **Modulstruktur:**
  - `runtime`: Engine- und Store-Verwaltung mit deterministischem Fuel-Meter.
  - `limits`: Speicher- und Rechenzeitgrenzen (`MemoryLimit`, `FuelBudget`).
  - `bindings`: Minimales Host-Guest-Interface für Input/Output.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn execute_wasm(wasm_bytes: &[u8], input: &[u8], limits: ResourceLimits) -> Result<Vec<u8>, SandboxError>;
  ```
- **Invarianten:**
  - `INV-SANDBOX-FUEL-1`: Sobald das zugewiesene Fuel-Budget aufgebraucht ist, bricht die Ausführung deterministisch ab.

### 13.3 `contextra-infer-candle` (Lokale GGUF- & Transformer-Inferenz)
- **Zweck:** Standalone Inferenz-Treiber auf Basis von Hugging Face Candle für lokale GGUF-Modelle.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn load_gguf_model(vfs: &impl Vfs, path: &Path) -> Result<CandleModel, InferError>;
  pub fn generate_embeddings(model: &CandleModel, texts: &[&str]) -> Result<Vec<Vec<f32>>, InferError>;
  ```

### 13.4 `contextra-infer-ollama` (Ollama Client & Streaming)
- **Zweck:** Autarker HTTP-Client für lokale Ollama-Server mit Retry- und Mock-Transport-Fähigkeit.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub async fn embed(client: &OllamaClient, prompt: &str) -> Result<Vec<f32>, InferError>;
  pub async fn complete(client: &OllamaClient, prompt: &str) -> Result<String, InferError>;
  ```

### 13.5 `contextra-infer-onnx` (ONNX-Runtime Inferenz)
- **Zweck:** Autarke Ausführung von ONNX-Modellen für Embeddings und Cross-Encoder-Reranking.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn load_onnx_model(model_bytes: &[u8]) -> Result<OnnxSession, InferError>;
  pub fn run_rerank(session: &OnnxSession, query: &str, docs: &[&str]) -> Result<Vec<f32>, InferError>;
  ```

---

## 14. Normative Spezifikation: Spezial- & Compliance-Bausteine (Klasse A)

### 14.1 `contextra-license` (Lizenz- & Token-Validierung)
- **Zweck:** Offline-Validierung kryptographisch signierter Lizenz-Token ohne externe Netzwerkverbindung.
- **Modulstruktur:**
  - `token`: Ed25519-Signaturprüfung und Entschlüsselung von Lizenz-Payloads.
  - `claims`: Validierung von Gültigkeitsdauer, Instanzgrenzen und Feature-Freigaben.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn verify_license(token: &str, public_key: &[u8; 32]) -> Result<LicenseClaims, LicenseError>;
  pub fn is_feature_enabled(claims: &LicenseClaims, feature_name: &str) -> bool;
  ```
- **Invarianten:**
  - `INV-LIC-OFFLINE-1`: Die Lizenzprüfung ist vollständig autark und führt keinerlei Netzwerk-Calls durch.

### 14.2 `contextra-audit-export` (Beweis- & Audit-Log-Export)
- **Zweck:** Formatierung und sicherer Export von Revisions- und Audit-Nachweisen in unveränderbare Formate (PDF/A, JSON-LD, CSV).
- **Modulstruktur:**
  - `formatter`: Deterministische Serialisierung strukturierter Protokolldaten.
  - `signer`: Einbindung digitaler Signaturen zur Beglaubigung von Audit-Berichten.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn generate_audit_report(entries: &[AuditEntry], signer: &impl Signer) -> Result<Vec<u8>, AuditError>;
  ```

### 14.3 `contextra-avv-generator` (Auftragsverarbeitungs-Dokumentation)
- **Zweck:** Schablonenbasierte, deklarative Generierung von technischen Anlagen zu Auftragsverarbeitungsverträgen (AVV / TOMs nach Art. 28 DSGVO).
- **Modulstruktur:**
  - `template`: Sichere Template-Engine ohne dynamische Code-Ausführung.
  - `specs`: Vordefinierte technische Profile über Verschlüsselung, Speicherung und Mandantentrennung.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn render_tom_document(profile: &TechnicalProfile) -> Result<String, TomGenError>;
  ```

### 14.4 `contextra-adapt` (Protokoll- & Adapter-Schicht)
- **Zweck:** Isolierte Konvertierungsadapter zwischen Bausteinen der Klasse A und Fremdformaten (OpenAI, Anthropic, HuggingFace, Arrow).
- **Modulstruktur:**
  - `openai`: Mappings für Embedding- und Completion-Schnittstellen.
  - `arrow`: Zero-Copy-Konvertierung von Vektorfeldern nach Apache Arrow RecordBatches.

### 14.5 `contextra-erasure` (Löschkern)
- **Zweck:** Zentraler Kern für die Orchestrierung der "Drei Säulen der Löschung" (ReadGate, physische Bereinigung, kryptographische Schlüsselnullung).
- **Modulstruktur:**
  - `backend`: Abstrakte `StorageBackend`- und `ErasureTarget`-Traits.
  - `saga`: Zustandsautomat für mehrstufige Löschungen mit idempotentem Rollback.
  - `proof`: BLAKE3-Ketten-Hashing für fälschungssichere Löschquittungen.
- **Kernfunktionen & Signaturen:**
  ```rust
  pub fn initiate_erasure(target: &impl ErasureTarget, scope: ErasureScope) -> Result<ErasureReceipt, ErasureError>;
  pub fn verify_receipt(receipt: &ErasureReceipt, public_key: &[u8; 32]) -> Result<bool, ErasureError>;
  ```
- **Invarianten:**
  - `INV-ERASURE-ISOLATION-1`: Löschketten und DSGVO-spezifische Begriffe existieren ausschließlich in diesem Crate.

---

## 15. Normative Spezifikation: Storage-Bundle (Klasse B)

Das Storage-Bundle bildet ein kohärentes, gemeinsam entnehmbares Subsystem für persistente, transaktionale Datenspeicherung:

### 15.1 `contextra-types` (Zentrale Daten- und Transaktionstypen)
- **Zweck:** Pure Datentypen ohne I/O, Threads oder Async (`DocId`, `TxId`, `Timestamp`, 78-Byte `KeyBuf` mit invertiertem Sequence-Byte `!commit_seq`).

### 15.2 `contextra-ports` (Trait-Verträge des Storage-Kerns)
- **Zweck:** Dyn-fähige Schnittstellenverträge (`StorageEngine`, `CommitBarrier`, `PurgeBarrier`, `Vfs`).

### 15.3 `contextra-mvcc` (Multi-Version Concurrency Control)
- **Zweck:** Transaktionsverwaltung mit Snapshot-Isolation und Serializable Snapshot Isolation (SSI). Strikte Lock-Hierarchie: Dokument-Lock $	o$ Commit-Barriere $	o$ Purge-Barriere $	o$ Journal.

### 15.4 `contextra-checkpoint` (Atomare Snapshots & Backups)
- **Zweck:** Erstellung konsistenter Point-in-Time-Snapshots via Dateisystem-Hardlinks und Metadaten-Manifeste.

### 15.5 `contextra-store` (LSM-Tree Storage Engine)
- **Zweck:** Robuste LSM-Speicher-Engine mit synchronem Write-Ahead-Log (WAL), In-Memory Memtable, SSTables und Leveled Compaction.
- **Invarianten:**
  - Direct I/O und mmap sind verboten (0 mmap, 0 std::fs in Production); Lesezugriffe laufen rein über `VfsFile::read_at`.

---

## 16. Normative Spezifikation: Komposition & Facade (Klasse C)

### 16.1 `contextra-engine` (Collection-Engine)
- **Zweck:** Orchestriert LSM-Store, Vektor-, Text- und Graph-Indizes zu einer einheitlichen Collection mit Multi-Signal-Suche.

### 16.2 `contextra-cognition` (Hintergrund-Konsolidierung)
- **Zweck:** Asynchrone Kontextkonsolidierung, Re-Ranking, Memory-Pruning und automatische Indexpflege.

### 16.3 `contextra-router` (Semantisches Query-Routing)
- **Zweck:** Analyse und Routing von Anfragen an spezialisierte Indizes oder LLM-Pipelines basierend auf Intent-Klassifikation.

### 16.4 `contextra-agent` (Agenten-Laufzeitumgebung)
- **Zweck:** Werkzeugausführung, Memory-Management und iterative Denk-Schleifen für autonome LLM-Agenten.

### 16.5 `contextra-mcp` (Model Context Protocol Server)
- **Zweck:** Exponiert Contextra-Funktionalitäten über das Model Context Protocol (MCP) als JSON-RPC-Dienst für Agenten und IDEs.

### 16.6 `contextra-py` (Python Bindings)
- **Zweck:** PyO3- und C-ABI-Wrapper zur Einbindung von Contextra-Bausteinen in Python-Datenpipelines.

### 16.7 `contextra` (Facade & Composition Root)
- **Zweck:** Façade unter `#![forbid(unsafe_code)]`. Enthält keinerlei Geschäftslogik, sondern verbindet die Subsysteme über den `ContextraBuilder`.

---

## 17. Normatives Entscheidungsregister (ADR-Status)

| ID | Entscheidung | Normative Begründung |
|---|---|---|
| **E-01** | **Dreiklassige Architektur (A/B/C)** | Rechnerische und indexierende Bausteine (Klasse A) müssen ohne Transaktionsballast entnehmbar sein; der Storage-Kern (Klasse B) verbleibt als kohärentes Bundle. |
| **E-02** | **Kapselung von `types` und `ports`** | Klasse A importiert weder `types` noch `ports`. Gemeinsame Typen werden lokal deklariert oder als Standardtypen übergeben. |
| **E-03** | **Adapter-Muster für Schnittstellen** | Konvertierungen zwischen Bausteinen und Framework-Ports finden ausschließlich in Adaptern statt. |
| **E-04** | **Zentralisierung der Löschlogik** | Löschketten und DSGVO-Beweise existieren nur in `contextra-erasure`; Indizes stellen nur elementare Lösch- und Kompaktierungsbefehle bereit. |
| **E-05** | **Substanz-Erhalt vor Dead-Code-Pruning** | Vorhandene Algorithmen und mathematische Kerne dürfen nicht gelöscht werden, sondern werden in Bausteine überführt oder hinter `experimental` gestellt. |
| **E-06** | **Funktionsinventar-Pflicht** | Vor jeder Migration großer Module muss ein vollständiges maschinelles Funktionsinventar erstellt und per Diff gegen Funktionsverlust gesichert werden. |
| **E-07** | **Zero-Panic-Doktrin** | Kein `unwrap()` oder `expect()` in Bibliotheksfunktionen. Fehler werden deterministisch über `Result` abgewickelt. |
| **E-08** | **Erhalt von Spezial-Bausteinen** | Crates wie `license`, `avv-generator` und `audit-export` bleiben als autarke Klasse-A-Bausteine erhalten. |
| **E-09** | **Getrennte Index-Traits** | Trennung in elementaren Index (`Index`), Wartung (`Maintain`) und Transaktionssteuerung (`TransactionalIndex`). |
| **E-10** | **Automatisierter Export** | Der Baustein-Export muss über automatisierte Skripte erfolgen, um `Cargo.toml`-Vererbungen aufzulösen. |
| **E-11** | **Einzige Quelle der Wahrheit** | Dieses Dokument (`docs/SPEC.md`) ist die höchste normative Referenz für Soll-Zustände im Repository. |
| **E-12** | **Lokale Test-Utilities** | Bausteine dürfen nicht von `contextra-testkit` abhängen, sondern pflegen eigene, schlanke Mocks. |

---

## 18. Offene Punkte und Entwicklungsentscheidungen

| ID | Gegenstand | Verbindlicher Richtwert / Standard |
|---|---|---|
| **O-1** | `no_std`/`alloc`-Unterstützung | Vorbereitung durch `std`-Feature-Flags; vollständige `no_std`-Validierung in Folgeversion. |
| **O-2** | Minimale Rust-Version (MSRV) | Einheitlich Rust 1.89 für den gesamten Workspace. |
| **O-3** | Modulare Aufteilung von `contextra-store` | Beibehaltung klarer Modulgrenzen (`wal`, `memtable`, `sstable`, `compaction`) vor eventueller Crate-Teilung. |
| **O-4** | Rechtliche Verbindlichkeit | Generierte Berichte und Vorlagen sind technische Artefakte und stellen keine Rechtsberatung dar. |
| **O-5** | Router-Entkopplung | `contextra-router` verbleibt in Klasse C, bis alle Abhängigkeiten zu `ports` und `wire` aufgelöst sind. |

---

## 19. Änderungsverlauf

| Fassung | Datum | Art der Änderung |
|---|---|---|
| **4.0** | 2026-11-08 | Vollständige Neufassung: Abkehr vom Plattform-Monolithen hin zum modularen Baukasten. Einführung der Klassen A/B/C/T und der Gates G1–G10. |
| **4.1** | 2026-11-08 | Massive normative Vertiefung: Vollständige Ausarbeitung der Soll-Zustände, Modulstrukturen, Invarianten, Kernel-Funktionen und Fehlermodelle aller 36 Crates. Präzisierung der Concurrency- und Durability-Garantien. |
