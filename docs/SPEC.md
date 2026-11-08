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
- **Zweck:** Kapselung aller direkten POSIX- und OS-Systemaufrufe für Speicherschutz und Durability.
- **Modulstruktur:**
  - `posix`: Direkte POSIX-Bindings für Datei-Deskriptoren, Berechtigungen und Fsync.
  - `durability`: Dateisystem-Synchronisationsbefehle (`fdatasync`, Verzeichnis-Sync).
  - `mlock`: Speichersperren (`mlock`, `munlock`, `madvise(MADV_DONTDUMP)`) gegen Auslagerung sensibler Schlüssel in die Swap-Datei.
  - `vault`: Low-level Speicherallokation in geschützten Memory-Pages.
- **Invarianten:**
  - `INV-SYS-MEMLOCK-1`: Mit `mlock` gesicherte Puffer dürfen unter keinen Umständen im Core-Dump oder Swap landen.
  - `INV-SYS-FSYNC-1`: Schlägt ein Fsync-Aufruf fehl, gibt die Funktion sofort einen typisierten OS-Fehler zurück; Retries auf derselben Handle-Instanz sind verboten.
- **Fehlermodell:** `SysError` mit Varianten für `MemoryLockFailed`, `SyncFailed(i32)`, `PermissionDenied`.

### 10.2 `contextra-simd` (Vektor-Distanzkerne)
- **Zweck:** Extrem optimierte mathematische Distanz- und Ähnlichkeitsberechnungen für dichte Vektoren.
- **Modulstruktur:**
  - `metric`: Lokales Enum `DistanceMetric` (`Cosine`, `Euclidean`, `DotProduct`).
  - `dispatch`: Dynamische CPU-Erkennung zur Laufzeit (AVX-512, AVX2+FMA, ARM Neon, Scalar-Fallback).
  - `kernels/`: Hardwarenahe Kernel: `scalar.rs`, `avx2.rs`, `avx512.rs`, `neon.rs`.
  - `error`: Lokaler Fehlertyp `SimdError` (`InvalidInput`, `DimensionMismatch`).
- **Kernfunktionen:**
  - `compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>`
  - `compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>`
  - `compute_distance_f32_bytes_trusted(a: &[f32], b_bytes: &[u8], metric: DistanceMetric) -> Result<f32>`
  - `validate_vector(vec: &[f32]) -> Result<()>` (Fail-Closed gegen NaN und Unendlich).
- **Invarianten:**
  - `INV-SIMD-FINITE-1`: Keine Berechnung darf mit NaN- oder Unendlich-Werten ausgeführt werden.
  - `INV-SIMD-ALIGN-1`: Byte-Puffer müssen ganzzahlig durch 4 teilbar sein (`sizeof(f32)`).
  - `INV-SIMD-DETERMINISM-1`: Alle Kernel müssen innerhalb einer Toleranz von `1e-5` identische Ergebnisse zur Skalar-Referenz liefern.

### 10.3 `contextra-wire` (Zero-Copy IPC & Serialisierung)
- **Zweck:** Extrem effiziente, schema-basierte Interprozesskommunikation via FlatBuffers.
- **Modulstruktur:**
  - `generated`: Maschinell generierte FlatBuffers-Rust-Bindings.
  - `adapter`: Sichere Zero-Copy-Wrapper für eingehende und ausgehende Nachrichten.
  - `verification`: Verifikation von Puffergrenzen und Schemakonformität vor dem Parsen.
  - `jsonrpc`: Leichtgewichtiger Framing- und Dispatch-Layer.
- **Invarianten:**
  - `INV-WIRE-BOUNDS-1`: Kein FlatBuffer-Zugriff darf über die Puffergrenzen hinauslesen (Zero-Panic Memory Safety).

---

## 11. Normative Spezifikation: Indizes & Retrieval (Klasse A)

### 11.1 `contextra-simd` (Siehe §10.2)

### 11.2 `contextra-rank` (Ergebnis-Fusion & Kalibrierung)
- **Zweck:** 4-Signal-Fusion (Vektor, Text, Graph, Reranker) und statistische Score-Kalibrierung.
- **Modulstruktur:**
  - `fusion`: Reciprocal Rank Fusion (RRF), gewichtetes RRF und lineare Normalisierung.
  - `calibration`: Platt-Scaling, Isotonische Regression (PAVA) und Conformal Prediction.
  - `drift`: Überwachung von Verteilungsverschiebungen in Ähnlichkeits-Scores.
  - `explain`: Generierung strukturierter Erklärungsdaten für Rankings.
  - `dibud`: Streaming-Rangordnungsfusion.
- **Kernfunktionen:**
  - `rrf_fuse(ranked_lists: &[Vec<ScoredItem>], k: usize) -> Result<Vec<ScoredItem>>`
  - `calibrate_score(raw_score: f32, calibrator: &Calibrator) -> Result<f32>`
- **Invarianten:**
  - `INV-RANK-MONOTONIC-1`: Isotonische Kalibrierungen dürfen niemals die relative Rangfolge von Scores invertieren.
  - `INV-RANK-DETERMINISM-1`: Bei identischen Eingabelisten ist das Fusionsergebnis bitgenau deterministisch.

### 11.3 `contextra-text` (Volltextsuche & BM25)
- **Zweck:** Volltextindexierung, Tokenisierung, morphologische Analyse und BM25-Suche.
- **Modulstruktur:**
  - `tokenizer`: Unicode-Segmentierung, Lowercasing, Stoppwortfilterung und Stemming.
  - `bm25`: BM25- und BM25F-Relevanzberechnung mit konfigurierbaren Parametern `k1` und `b`.
  - `posting_list`: Komprimierte Posting-Listen mit Delta-VByte-Codierung.
  - `wand`: Weak AND (WAND)-Algorithmus für dynamisches Pruning irrelevanter Posting-Listen.
  - `inverted`: Invertierter Kernindex mit threadsicherer Lesesicht.
- **Schnittstellen:** Implementiert `Index<u64, String>` und `Maintain`.
- **Invarianten:**
  - `INV-TEXT-WAND-1`: Der WAND-Algorithmus liefert exakt dieselben Top-K-Treffer wie eine vollständige Auswertung aller Posting-Listen.

### 11.4 `contextra-vector` (HNSW-Vektorindex)
- **Zweck:** Graphbasierte approximative Nächste-Nachbarn-Suche (HNSW) und Quantisierung.
- **Modulstruktur:**
  - `hnsw`: Mehrschichtiger Proximity-Graph (Knoten, Kanten, Layer, Heuristik-Suche).
  - `distance`: Adapter zu Vektordistanzen (nutzt lokal konfigurierbare Distanz-Funktionen).
  - `quantize`: Skalare Quantisierung (SQ8) und RaBitQ zur Reduktion des Speicherbedarfs.
  - `partial_rebuild`: Inkrementeller Neuaufbau und Re-Wiring des Graphen nach Löschungen.
  - `persistence`: Serialisierung und Deserialisierung des Graphen in Datei- und Byte-Formate.
- **Schnittstellen:** Implementiert `Index<u64, &[f32]>` und `Maintain`.
- **Invarianten:**
  - `INV-VEC-CONNECTIVITY-1`: Nach dem Entfernen eines Knotens muss die Graph-Konnektivität durch Reparatur der Nachbarschaftskanten erhalten bleiben.
  - `INV-VEC-ZEROIZE-1`: Gelöschte Vektorslots werden im Speicher mit Nullen überschrieben.

### 11.5 `contextra-graph` (CSR-Wissensgraph)
- **Zweck:** Extrem schneller Wissensgraph auf Basis von Compressed Sparse Row (CSR) für Beziehungsrecherche und Personalized PageRank (PPR).
- **Modulstruktur:**
  - `csr`: Knoten- und Kanten-CSR-Repräsentation für maximale Cache-Lokalität.
  - `ppr`: Approximate Personalized PageRank via Forward-Push.
  - `community`: Erkennung dichter Knoten-Cluster.
  - `cascade`: Lokale Pfadtraversierung und Kantenpropagierung.
- **Schnittstellen:**
  - `add_edge(source: u64, target: u64, weight: f32) -> Result<()>`
  - `traverse_ppr(start_nodes: &[u64], alpha: f32, epsilon: f32) -> Result<Vec<(u64, f32)>>`
- **Invarianten:**
  - `INV-GRAPH-CSR-1`: Die CSR-Indizes sind nach Knotennummern sortiert und frei von Kanten-Duplikaten.

---

## 12. Normative Spezifikation: Persistenz, Cache & Krypto (Klasse A)

### 12.1 `contextra-durable-fs` (Durable I/O & Fsync)
- **Zweck:** Garantiert persistente Dateioperationen mit striktem Fsync-Poisoning und Crash-Sicherheit.
- **Modulstruktur:**
  - `vfs`: Lokaler, synchroner `Vfs`-Trait (`read`, `write_atomic`, `sync_dir`, `remove`).
  - `journal`: Sequentielles Append-Only-Journaling für Metadatenänderungen.
  - `health`: Integritäts- und Zustandsüberwachung von Datei-Handles.
  - `scrub`: Hintergrundprüfung auf Bit-Rot und Dateikorruption mittels Prüfsummen.
  - `fault_injector`: Test-Injektor für kontrollierte Schreib- und Sync-Abbrüche.
- **Invarianten:**
  - `INV-FSYNC-1`: Schlägt ein Fsync-Aufruf fehl, wird der zugehörige Datei-Deskriptor irreversibel als fehlerhaft markiert (Poisoning). Nachfolgende Schreibversuche schlagen sofort fehl.
  - `INV-VFS-ATOMIC-1`: Datei-Ersetzungen (`write_atomic`) erfolgen über temporäre Dateien und atomares OS-Rename.

### 12.2 `contextra-kvcache` (KV-Cache für LLM-Tokens)
- **Zweck:** Verwaltung, Prefix-Caching und Paginierung von Key-Value-Tensoren für LLM-Inferenz.
- **Modulstruktur:**
  - `radix`: Radix-Baum für hierarchisches Prefix-Matching gemeinsamer Prompts.
  - `segment`: Allokation und Freigabe von Speicherblöcken für Token-Sequenzen.
  - `attention_score`: Eviction-Heuristik basierend auf Aufmerksamkeitswerten.
  - `quantize_kivi`: KIVI-Quantisierung von FP16-KV-Tensoren auf 2/4-Bit-Ganzzahlen.
- **Invarianten:**
  - `INV-KV-PREFIX-1`: Identische Prompt-Prefixe teilen sich dieselben physischen KV-Cache-Segmente im Speicher.

### 12.3 `contextra-crypto` (Kryptographische Primitive)
- **Zweck:** Hochsichere symmetrische und asymmetrische kryptographische Primitive.
- **Modulstruktur:**
  - `aead`: Authenticated Encryption with Associated Data (AES-256-GCM-SIV, RFC 8452).
  - `dek`: Data Encryption Key Management mit automatischer Schlüsselnullung im Speicher (`ZeroizeOnDrop`).
  - `anti_tamper`: BLAKE3- und HMAC-basierte Integritätsketten.
  - `secret`: Typisierte Container für geheime Schlüssel ohne Klartext-Debug-Ausgabe.
- **Invarianten:**
  - `INV-CRYPTO-ZEROIZE-1`: Alle Schlüsselstrukturen implementieren `ZeroizeOnDrop`.
  - `INV-CRYPTO-CONSTANT-TIME-1`: Alle Schlüssel- und Prüfsummenvergleiche erfolgen in strikter konstanter Zeit (`subtle::ConstantTimeEq`).

### 12.4 `contextra-privacy` (PII-Erkennung & Egress-Guard)
- **Zweck:** Erkennung und Maskierung personenbezogener Daten (PII) sowie Schutz vor Cloud-Datenabfluss.
- **Modulstruktur:**
  - `entity_recognizer`: Erkennung von E-Mail-Adressen, IBAN, IP-Adressen und Namen mittels regulärer Ausdrücke und Wörterbüchern.
  - `egress_guard`: Filterung von Netzwerk-Payloads vor der Weiterleitung an externe Cloud-APIs.
  - `guarded_payload`: Typisierter Container, der garantiert nur bereinigte Daten enthält.
- **Invarianten:**
  - `INV-PRIVACY-FAILCLOSED-1`: Erkennt der Scanner ein unzulässiges PII-Muster, wird der Payload-Export ausnahmslos blockiert.

---

## 13. Normative Spezifikation: Ingestion, Inferenz & Sandboxing (Klasse A)

### 13.1 `contextra-ingest` (Dokumenten-Parsing & Chunking)
- **Zweck:** Robustes, allokationsbegrenztes Einlesen verschiedener Dokumentformate (PDF, DOCX, HTML, Markdown, Plaintext).
- **Modulstruktur:**
  - `detect`: Erkennung von MIME-Typen anhand magischer Bytes (beschränkt auf max. 64 KiB Lesepuffer).
  - `extractor`: Format-spezifische Text- und Metadaten-Extraktoren.
  - `chunking`: Struktur- und semantikbewusstes Aufteilen von Texten in Chunks (Absätze, Sätze, Tokengrenzen).
  - `canonical`: Einheitliches Zwischenformat für extrahierte Dokumentabschnitte.
- **Invarianten:**
  - `INV-INGEST-BOUNDED-1`: Die Dateierkennung liest niemals mehr als `MAX_DETECT_BYTES` (64 KiB) ein.
  - `INV-INGEST-NO-ARCHIVE-BOMB-1`: Dekomprimierung von ZIP-Containern ist durch strikte Ratio- und Größengrenzen gegen Zip-Bomben geschützt.

### 13.2 `contextra-sandbox` (WASM-Ausführungskapselung)
- **Zweck:** Sichere, isolierte Ausführung untrusted Codes über WebAssembly (Wasmtime).
- **Modulstruktur:**
  - `executor`: Instanziierung und Ausführung von WASM-Modulen.
  - `capabilities`: Deklarative Einschränkung von Speicher, CPU-Brennstoff (Fuel) und I/O-Rechten.
  - `approval`: Sicherheitsüberprüfung von Funktionsaufrufen.
- **Invarianten:**
  - `INV-SANDBOX-FUEL-1`: Jede Ausführung ist durch ein striktes Gas-/Fuel-Budget begrenzt und bricht bei Überschreitung deterministisch ab.

### 13.3 Inferenz-Treiber (`infer-candle`, `infer-ollama`, `infer-onnx`)
- **Zweck:** Lokale Generierung von Einbettungen (Embeddings) und Textinferenz.
- **Architektur:** Jedes Crate definiert autarke Trait-Schnittstellen:
  ```rust
  pub trait Embedder {
      fn embed(&self, text: &str) -> Result<Vec<f32>>;
      fn dimension(&self) -> usize;
  }
  ```
- `contextra-infer-candle`: Nutzt Candle für direkte In-Process-GGUF-Modellausführung.
- `contextra-infer-onnx`: Nutzt ORT für Cross-Encoder-Reranking und Transformer-Modelle.
- `contextra-infer-ollama`: Asynchroner HTTP-Client für Ollama mit Circuit-Breaker und Retry-Logik.

---

## 14. Normative Spezifikation: Klasse B & Klasse C

### 14.1 Klasse B: Storage-Bundle (`types`, `ports`, `mvcc`, `checkpoint`, `store`)
Das Storage-Bundle bildet das persistente Rückgrat für Transaktionsverarbeitung und Versionierung:
- **`contextra-types`:** Zentrale Typen für Transaktions- und Speicherebenen (`DocId`, `TxId`, `Timestamp`, `KeyBuf`).
- **`contextra-ports`:** Dyn-kompatible Ports für Subsysteme (`StorageEngine`, `CommitBarrier`, `PurgeBarrier`).
- **`contextra-mvcc`:** Multi-Version Concurrency Control mit Snapshot-Isolation und Serializable Snapshot Isolation (SSI).
- **`contextra-checkpoint`:** Atomare Hardlink-Snapshots und Rollback-Verwaltung für Dateisystem-Backups.
- **`contextra-store`:** Vollwertige LSM-Tree-Speicher-Engine mit Write-Ahead-Log (WAL), Memtable, SSTables und mehrstufiger Kompaktierung (Leveled Compaction).

### 14.2 Klasse C: Kompositions- & Integrationsschicht
- **`contextra`:** Öffentliche Fassade zur einfachen Instanziierung des Gesamtsystems.
- **`contextra-engine`:** Verknüpft LSM-Store, Vektor-, Text- und Graph-Indizes zu einer einheitlichen Collection-Engine.
- **`contextra-cognition`:** Asynchrone Hintergrundkonsolidierung von Kontexten, Memory-Pruning und periodische Wartung.
- **`contextra-router`:** Intelligentes semantisches Routing von Nutzeranfragen an unterschiedliche LLM- und RAG-Pipelines.
- **`contextra-mcp`:** Model Context Protocol (MCP) Server zur Bereitstellung der Funktionen für KI-Agenten über Standardprotokolle.
- **`contextra-py`:** C-ABI und PyO3-Bindings für die Einbindung in Python-Laufzeitumgebungen.

---

## 15. Normatives Entscheidungsregister (ADR-Status)

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

## 16. Offene Punkte und Entwicklungsentscheidungen

| ID | Gegenstand | Verbindlicher Richtwert / Standard |
|---|---|---|
| **O-1** | `no_std`/`alloc`-Unterstützung | Vorbereitung durch `std`-Feature-Flags; vollständige `no_std`-Validierung in Folgeversion. |
| **O-2** | Minimale Rust-Version (MSRV) | Einheitlich Rust 1.89 für den gesamten Workspace. |
| **O-3** | Modulare Aufteilung von `contextra-store` | Beibehaltung klarer Modulgrenzen (`wal`, `memtable`, `sstable`, `compaction`) vor eventueller Crate-Teilung. |
| **O-4** | Rechtliche Verbindlichkeit | Generierte Berichte und Vorlagen sind technische Artefakte und stellen keine Rechtsberatung dar. |
| **O-5** | Router-Entkopplung | `contextra-router` verbleibt in Klasse C, bis alle Abhängigkeiten zu `ports` und `wire` aufgelöst sind. |

---

## 17. Änderungsverlauf

| Fassung | Datum | Art der Änderung |
|---|---|---|
| **4.0** | 2026-11-08 | Vollständige Neufassung: Abkehr vom Plattform-Monolithen hin zum modularen Baukasten. Einführung der Klassen A/B/C/T und der Gates G1–G10. |
| **4.1** | 2026-11-08 | Massive normative Vertiefung: Vollständige Ausarbeitung der Soll-Zustände, Modulstrukturen, Invarianten, Kernel-Funktionen und Fehlermodelle aller 36 Crates. Präzisierung der Concurrency- und Durability-Garantien. |
