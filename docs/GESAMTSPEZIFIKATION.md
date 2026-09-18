# MemFuse Cognitive OS — Finale Konsolidierte Gesamtspezifikation

> **Status:** Normativ · Einzige maßgebliche Quelle für Produkt, Architektur, Algorithmen,
> Implementierungsvorgaben, Sicherheitsmodell, Schnittstellenspezifikation und Optimierungs-Roadmap
> des MemFuse Cognitive OS.
>
> **Synthese aus:**
> - `MEMFUSE_GESAMTSPEZIFIKATION.md` — Produktvision, Architekturprinzipien P1–P25, Reifegrad-Tracking
> - `MEMFUSE_IMPLEMENTIERUNGSSPEZIFIKATION.md` — Normative Signaturen, Modulstrukturen, Fehler-Enums, Testkriterien
> - `Mikrofeingranulare Schnittstellenspezifikation.md` — Lock-freie Algorithmen, mathematische Grundlagen, SIMD-Spezifikation
> - `Implementierungsplan_Opus_Optimierungen.md` — Priorisierte Optimierungsmaßnahmen Stufe 0–3
>
> **Charakter:** Dieses Dokument führt Produktvision, Zielarchitektur, normative Signaturen,
> algorithmische Spezifikationen, mikrofeingranulare Schnittstellendefinitionen und die priorisierte
> Optimierungs-Roadmap in einem einzigen, in sich geschlossenen Dokument zusammen. Es ersetzt
> vollständig alle vorherigen Spezifikationsfassungen und -deltas.
>
> **Sprache:** Rust 2021, Workspace-Layout, `#![forbid(unsafe_code)]` als Default in jedem Crate
> ohne explizite Ausnahme (§0.4).
>
> **Lesart:** Jeder Abschnitt ist eigenständig implementierbar. Codeblöcke sind **normativ**, nicht
> illustrativ — Feldnamen, Typnamen und Funktionssignaturen sind exakt zu übernehmen, sofern nicht
> als „Beispiel" markiert. Wo `unimplemented!()` steht, ist die Signatur und das umgebende
> Vertrags-/Fehlerverhalten normativ, der Funktionskörper ist gemäß der in Prosa/Formel gegebenen
> Algorithmusbeschreibung des jeweiligen Abschnitts zu füllen.
>
> **Reifegrad-Kennzeichnung, durchgängig verwendet:**
> - 🟢 **Produktiv** — im Code vorhanden, korrekt und als Produktions-Default aktiv.
> - 🟡 **Hinter Feature-Flag** — im Code vollständig und korrekt vorhanden, aber nicht der
>   Produktions-Default; Aktivierung erfordert ein explizites Cargo-Feature.
> - 🔴 **Spezifiziert, zu bauen** — normativer Zielzustand dieses Dokuments, im Code noch nicht
>   vorhanden.
> - ⚖️ **Produktentscheidung ausstehend** — technisch möglich oder vorhanden, Default-Wechsel an
>   messbares Kriterium gebunden.
> - ⚠️ **Opus-Optimierung** — aus Architektur-Review identifiziert, priorisiert umzusetzen, mit
>   Stufe (0–3) und Aufwandseinschätzung versehen.

---

## Inhaltsverzeichnis

0. [Meta: Workspace-Layout und Build-Konfiguration](#0-meta)
1. [Kernthese und Leitprinzip](#1-kernthese)
2. [Produktvision, Alleinstellungsmerkmale und Nicht-Ziele](#2-vision)
3. [Architekturprinzipien P1–P25](#3-prinzipien)
4. [Systemarchitektur: der Crate-DAG](#4-architektur)
5. [Speicherschicht: LSM-Tree, WAL und Block-Cache](#5-speicher)
6. [Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten](#6-graph)
7. [Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen](#7-retrieval)
8. [Contextual-Bandit-Routing](#8-bandit)
9. [Inferenz, KV-Cache-Bridge und Zero-Copy-IPC](#9-inferenz)
10. [Sicherheits- und Datenschutzmodell](#10-sicherheit)
11. [Betriebsmodi](#11-betrieb)
12. [FlatBuffers-Schema (vollständig)](#12-schema)
13. [Fehlertaxonomie (crateübergreifend)](#13-fehler)
14. [Feature-Flag-Politik: Produktions-Default vs. Opt-in](#14-features)
15. [Test- und CI-Spezifikation](#15-tests)
16. [Vollständige Abnahmekriterien](#16-abnahme)
17. [Priorisierte Optimierungs-Roadmap (Opus-Analyse)](#17-optimierungen)
18. [Gesamtroadmap](#18-roadmap)
19. [Rückverfolgbarkeitsmatrix](#19-matrix)

---

<a id="0-meta"></a>
## 0. Meta: Workspace-Layout und Build-Konfiguration

### 0.1 Verzeichnisstruktur

```
memfuse/
├── Cargo.toml                      # [workspace], resolver = "2"
├── xtask/                          # CI-Tooling (Drift-Gates, Benchmarks)
│   └── src/
│       ├── main.rs
│       ├── check_flatbuffers_drift.rs
│       └── check_bandit_latency_budget.rs
├── schemas/
│   └── memfuse.fbs                 # §12
├── crates/
│   ├── memfuse-core-ipc-gen/       # Layer 0 — FlatBuffers-generierter Code
│   ├── memfuse-core/               # Layer 0 — Kerntypen, Traits, Fehlerbehandlung
│   ├── memfuse-store/              # Layer 1 — LSM-Tree, WAL, Block-Cache
│   ├── memfuse-crypto/             # Layer 1 — AES-256-GCM-SIV, DeletionProof, KV-Segment-Security
│   ├── memfuse-text/               # Layer 1 — BM25/BM25F-Volltextindex, deutsche Morphologie
│   ├── memfuse-index/              # Layer 1 — HNSW/DiskANN-Vektorindex, SIMD-Distanz
│   ├── memfuse-graph/              # Layer 1 — CSR-Graph, PPR, Leiden, Hyperkanten
│   ├── memfuse-checkpoint/         # Layer 1 — Snapshotting
│   ├── memfuse-calibration/        # Layer 1 — Score-Kalibrierung, Drift-Erkennung
│   ├── memfuse-db/                 # Layer 2 — Collection-API, 4-Signal-Fusion, Provenance
│   ├── memfuse-router/             # Layer 3 — Contextual-Bandit-Routing
│   ├── memfuse-candle/             # Layer 3 — Natives GGUF-Inferenz-Backend, KV-Cache-Bridge
│   ├── memfuse-ollama/             # Layer 3 — Ollama-Client, Contextual-Chunk-Prefixing
│   ├── memfuse-embed/              # Layer 3 — ONNX-Embeddings, Cross-Encoder (optional)
│   ├── memfuse-agent/              # Layer 3 — Persistente Agent-Workflow-Engine
│   ├── memfuse-py/                 # Layer 3 — Python-FFI via PyO3 (eigener Workspace)
│   ├── memfuse-sandbox/            # Layer 6.5 — WASM Execution Boundary
│   ├── memfuse-mcp/                # Layer 4 — MCP-Server, Egress-Gateway
│   └── memfuse-bench/              # Layer 5 — Benchmark-Harness
├── benchmarks/
│   └── memfuse-bench/
├── .github/workflows/
│   └── merge-gate.yml              # §15.4
└── docs/decisions/                 # ADR-0NN-*.md <!-- doc-ref-ignore -->
```

### 0.2 Root-`Cargo.toml` (normativ)

```toml
[workspace]
resolver = "2"
members = ["crates/*", "xtask"]

[workspace.package]
edition = "2021"
rust-version = "1.79"
license = "Apache-2.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
flatbuffers = "23"
crossbeam-epoch = "0.9"
arc-swap = "1"
ahash = "0.8"
scc = "2"
quick_cache = "0.5"
zerocopy = "0.7"
thiserror = "1"
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
aes-gcm-siv = "0.11"
blake3 = "1"
wasmtime = "23"
```

### 0.3 Cargo-Feature-Katalog (crateübergreifend normativ)

| Feature | Definierender Crate | Default | Wirkung |
|---|---|---|---|
| `docid-128` | `memfuse-core` | aus | `DocId` wird `u128` statt `u64` (§6.1) |
| `block-cache-v2` | `memfuse-store` | aus | `SieveCacheBackend` statt `LruBlockCacheBackend` als aktives Backend (§5.4) |
| `egress-sherman-morrison` | `memfuse-router` | aus | `ShermanMorrisonBandit` statt `DiagonalApproximation` (§8.2) |
| `experimental-diskann` | `memfuse-index` | aus | `DiskAnnIndex` kompiliert und ist über `VectorIndexTier::DiskAnn` wählbar (§7.5) |
| `bandit-routing` | `memfuse-router` | an | Aktiviert den Bandit-Router überhaupt |
| `cloud-egress-guard` | `memfuse-mcp` | an | Aktiviert `egress_gateway`-Modul |
| `wasm-sandbox` | `memfuse-mcp` | an | Aktiviert `sandbox`-Modul |
| `kv-bridge` | `memfuse-candle` | an | Aktiviert `kv_cache_bridge`-Modul |
| `edge-reinforcement-learning` | `memfuse-graph` | aus | Aktiviert `SignalKind::EdgeReinforcement`-Pfad |
| `fault-injection` | `memfuse-store` | nur `dev-dependencies` | Deterministische I/O-Fehlerinjektion für Tests |
| `loom` | `memfuse-store`, `memfuse-graph` | nur `dev-dependencies` | Aktiviert `loom::sync::*` statt `std::sync::*` hinter `#[cfg(loom)]` |
| `bm25f` | `memfuse-text` | an | Feldgewichtete BM25-Bewertung |
| `adaptive-decay` / `-control` | `memfuse-db` | an | Kalibrierungs-Feintuning |
| `partial-index-rebuild` | `memfuse-index` | an | Inkrementeller Indexaufbau |

### 0.4 Globale Compile-Time-Regeln

Jeder Crate erhält in `lib.rs` genau eine der beiden folgenden Kopfzeilen:

```rust
#![forbid(unsafe_code)]
```

oder, **ausschließlich** in den folgenden Crates, mit dokumentierter Begründung:

| Crate | Begründung für `#![deny(unsafe_code)]` |
|---|---|
| `memfuse-index` | SIMD-Hardware-Optimierungen (AVX2, AVX-512, NEON) und Read-only Memory-Mapped Index I/O (ADR-017/ADR-034) |
| `memfuse-store` | Win32-DACL/ACL-File-Permission-Enforcement (`#[cfg(windows)]`) |
| `memfuse-db` | RAM-Buffer Memory-Locking gegen OS-Swapping (`mlock`/`munlock`, feature-gated `volatile-vault`) |
| `memfuse-embed` | C-FFI zum ONNX-Runtime-Backend (feature-gated) |
| `memfuse-core-ipc-gen` | Automatisch generierter FlatBuffers-IPC-Code |
| `memfuse-router` | SIMD-Intrinsics für Sherman-Morrison-Matrixarithmetik (§8.2) |

Jeder Crate erhält zusätzlich:

```rust
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

mit einer versionierten Ausnahmeliste `.unwrap-baseline.json` im Crate-Root, die in CI gegen Neuvorkommen
geprüft wird (Ratchet: die Datei darf nur schrumpfen, nie wachsen — CI-Job `check-unwrap-baseline`).

### 0.5 Non-Obvious Decisions (systemweit bindend)

- **TxId-Generation:** IMMER `collection.allocate_tx()` — NIEMALS `SystemTime::as_nanos()`
- **fsync-Fehler:** IMMER mit `?` propagieren — NIEMALS `let _ = dir.sync_all()`
- **Dokument-Chunking:** IMMER `MarkdownChunker` — NIEMALS gesamten Text als 1 Vektor embedden
- **MCP-Transport:** stdio JSON-RPC 2.0 ONLY — axum wurde entfernt (ADR-010)
- **WAL-HMAC-Key:** IMMER via `load_or_create_integrity_key()` — NIEMALS hardcoded
- **TOMBSTONE_BIT-Disziplin (ADR-041):** Bit 63 strikt maskieren (`seq & !TOMBSTONE_BIT`) vor `max_seq`-Vergleichen
- **SSTable-Flush-Sichtbarkeit (ADR-043):** `last_committed_tx` vor `sstables.push()` aktualisieren

---

<a id="1-kernthese"></a>
## 1. Kernthese und Leitprinzip

**MemFuse ist eine souveräne, vollständig lokal betriebene Gedächtnisschicht für KI-Agenten** — eine
eingebettete, kryptographisch isolierte AI-Memory-Bibliothek in Rust mit Python- und MCP-Bindings, die
ohne Cloud-Abhängigkeit, ohne Telemetrie und ohne API-Key betrieben werden kann.

Ihr Alleinstellungsmerkmal ist die Kombination aus:

- einer **4-Signal-Retrieval-Fusion** (Vektor, Volltext, Graph, Metadaten) statt reiner Vektorsuche,
- einer **kryptographisch integritätsgesicherten Storage-Engine** (LSM-Tree, WAL mit HMAC-Kette, AES-256-GCM-SIV at rest),
- **WASM-/Sandbox-Ausführungsisolation** für Agent-Tool-Aufrufe,
- echter **Air-Gap-Inferenz** (lokales GGUF-Backend, kein Netzwerkzwang) mit verschlüsseltem, LSM-rückfallfähigem KV-Cache,
- einem **Contextual-Bandit-Router**, der Anfragen adaptiv auf Retrieval-Strategien verteilt,
- und — als jüngste, noch zu bauende Erweiterung des Datenmodells — **n-ären Hyperkanten** für Fakten, die sich
  nicht auf ein Subjekt-Prädikat-Objekt-Tripel reduzieren lassen (§6).

### Leitprinzip

**Korrektheit schlägt Performance schlägt Feature.**

Jede Optimierung, die eine Korrektheitsgarantie (Datenintegrität, Nebenläufigkeitssicherheit,
Wiederherstellbarkeit, Deadlockfreiheit) aufweicht, ist unzulässig — unabhängig vom Performancegewinn.
Jede Performance-Optimierung, die eine noch nicht spezifizierte Fähigkeit vorwegnimmt, ist nachrangig
gegenüber der Fertigstellung bereits spezifizierter Fähigkeiten. Jede Erweiterung eines bestehenden
Subsystems muss geprüft werden gegen die Invarianten, die dieses Subsystem bereits trägt — nicht nur
dagegen, *dass* eine Erweiterung grundsätzlich möglich ist, sondern *welches bestehende Invariant dadurch
unter Druck gerät* und wie es gewahrt bleibt.

---

<a id="2-vision"></a>
## 2. Produktvision, Alleinstellungsmerkmale und Nicht-Ziele

### 2.1 Was MemFuse ist

Eine eingebettete (embedded) Gedächtnisschicht, kein Cloud-Service. MemFuse läuft im Prozess des
aufrufenden Agenten oder als lokaler MCP-Server — es gibt keine serverseitige Multi-Tenant-Instanz
und keine Datenübertragung an Dritte, sofern nicht explizit über das Cloud-Egress-Gateway (§10.4)
angefordert.

### 2.2 Distributionswege

| Kanal | Paket | Zielgruppe |
|---|---|---|
| MCP-Server (primär) | `uvx memfuse-mcp --db-path ... --allow-write` | Claude Desktop, Cursor, beliebige MCP-Clients |
| Python-Bibliothek | `pip install memfuse` | In-Process-Einbettung in Python-Agenten |
| Rust-Crate | `cargo add memfuse-db` | Native Rust-Anwendungen |

Eine Desktop-Shell (`memfuse-tauri`) existierte als Prototyp, ist aber zugunsten der PyPI-Bibliothek
und des MCP-Servers als primäre Vertriebswege eingestellt (deprecated, ADR-077).

### 2.3 Alleinstellungsmerkmale und ihr Reifegrad

1. **4-Signal-Hybridsuche** (🟢) — Vektorsuche (HNSW), Volltextsuche (BM25/BM25F), Wissensgraph-Traversierung
   (CSR + Forward-Push-Personalized-PageRank), Metadaten-Filter; fusioniert über Reciprocal Rank Fusion
   (RRF, Default 🟢) oder score-normalisierte Fusion (Opt-in 🟡, mit hartem RRF-Fallback bei Signaldegradation).

2. **Kalibriertes Retrieval mit Lyapunov-Drift-Erkennung** (🟢) — Score-Schwellenwerte werden nicht statisch,
   sondern über ein laufend kalibriertes Modell mit gedeckelter Drift-Eskalation bestimmt.

3. **MCP-native Zero-Trust-Sandbox** (🟢) — Tool-Ausführung mit getrennt konfigurierbarem Fuel- (Rechenschritt-)
   und Wall-Clock-Budget (Default 5 s), orthogonal zueinander konfigurierbar.

4. **Kryptographische DSGVO-Art.-17-Löschung** (🟢) — Löschvorgänge erzeugen einen verifizierbaren `DeletionProof`
   über eine race-freie HMAC-Kette.

5. **Session-DAG** (🟢) — Konversationsverzweigung als persistenter, azyklischer Graph.

6. **Air-Gap-KV-Cache-Bridge mit LSM-Fallback-Spill** (🟢) — der KV-Cache liegt primär verschlüsselt im RAM; bei
   Speicherdruck greift kontrolliertes Auslagern auf die SSD statt verlustbehafteten Verwerfens.

7. **Cloud-Egress Privacy Gateway** (🟢, weitgehend auditiert) — mehrschichtiger DLP-Pfad mit Surrogat-Tokenisierung,
   Bulk-Exfiltration-Erkennung und Rehydration der Cloud-Antwort (§10.4).

8. **Contextual-Bandit-Routing** (🟢 Grundfunktion / 🟡 mathematisch korrekte Variante) — LinUCB-basiertes Routing;
   siehe §8 für die Unterscheidung zwischen Produktions-Default und Ridge-korrekter Opt-in-Variante.

9. **Gestufte Vektorindex-Architektur** (🟢 HNSW / 🟢 DiskANN als Tier) — HNSW als Standard, DiskANN für
   RAM-sprengende Korpora.

10. **Deutsche Morphologie inkl. BM25F** (🟢) — Kompositazerlegung im Volltextindex plus feldgewichtete Bewertung.

11. **Zero-Copy-Storage-Pfad** (🟢) — seit der Grundarchitektur produktiv.

12. **Key-granulare Schreibnebenläufigkeit** (🟢) — `kv_locks` statt collection-weitem Mutex.

13. **N-äre Hyperkanten** (🔴, vollständig spezifiziert, siehe §6) — Fakten mit mehr als zwei Beteiligten als
    erstklassige, atomar invalidierbare Struktur statt Zerlegung in mehrere, im Zusammenhang verlorene Binärkanten.

### 2.4 Nicht-Ziele

MemFuse ist explizit **kein** Cloud-SaaS-Produkt, **kein** Multi-Tenant-Enterprise-System, **kein** Framework für
LLM-Training, **keine** primär GUI-getriebene Desktop-Anwendung und **kein** Cluster-/Replikations-System. Ein
`memfuse-cluster`-Veto besteht bewusst: verteilter Konsensbetrieb ist kein Ziel der aktuellen Produktphase.
Passives WAL-Shipping für Backup-Zwecke ist als Fernziel vorgesehen (Roadmap-Stufe 4, §18), aber nicht Bestandteil
des Kernprodukts.

---

<a id="3-prinzipien"></a>
## 3. Architekturprinzipien P1–P25

Diese Prinzipien sind normativ für jede gegenwärtige und künftige Erweiterung des Systems.

**P1 — Korrektheit schlägt Performance schlägt Feature.** Siehe §1.

**P2 — WAL-First-Persistenz.** Keine Zustandsänderung wird im Speicher sichtbar gemacht, bevor sie physisch
in das Write-Ahead-Log geschrieben und mit dem Datenträger synchronisiert wurde.

**P3 — Deterministische Recovery.** Der Systemzustand muss sich allein aus dem WAL rekonstruieren lassen.

**P4 — Keine stillschweigende I/O-Fehlerunterdrückung.** `fsync`-Fehler IMMER mit `?` propagieren.

**P5 — Strikte DAG-Modularität.** Abhängigkeiten im Crate-Graphen verlaufen strikt abwärts; ein Verstoß
gilt als Architekturdefekt.

**P6 — Feingranulare Fehlerbehandlung.** Domänenspezifische `Result<T, E>`-Enums statt generischer `panic!`-Pfade.

**P7 — Verbot von `unwrap()`/`expect()` auf toxischen Daten.** CI-gated über `.unwrap-baseline.json`.

**P8–P22 — Sovereign-Core-Grundsätze.** Umfassen u. a.: Verschlüsselung at rest als Default, HMAC-Kettenintegrität,
Zero-Trust-Sandbox, Air-Gap-fähige Inferenz, deterministische Transaktions-ID-Vergabe, Tombstone-Bit-Disziplin,
SSTable-Flush-Sichtbarkeit, Kaskaden-Invalidierung, kryptographische Löschnachweise, feingranulare Feature-Gates.

**P23 — Zeitbudgets sind orthogonal konfigurierbar.** Rechenschritt-Budget (Fuel) und Wall-Clock-Budget für
Sandbox-Ausführungen sind zwei unabhängige Achsen. Ein Tool kann rechnerisch günstig, aber durch blockierendes I/O
langsam sein, oder umgekehrt — beide Fälle müssen unabhängig begrenzbar sein. 🟢 Produktiv erfüllt (§10.2).

**P24 — Lokalität vor globaler Neuberechnung.** Jeder Algorithmus, dessen Eingabe eine anfragebestimmte Teilmenge
des Gesamtzustands ist (PPR mit wenigen Seed-Knoten, Cascade-Invalidierung ausgehend von einem Dokument), MUSS
eine zur Anfragegröße proportionale Laufzeit haben — niemals zur Größe des Gesamtzustands ($O(V+E)$ ist für
solche Anfragen unzulässig). Dieses Prinzip ist der normative Grund für den Forward-Push-PPR-Algorithmus (§7.2)
und für das harte Fan-out-Limit der Hyperkanten-Cascade-Invalidierung (§6.5, H5).

**P25 — Cache-Treffer sind lock-frei bzw. lock-günstig zu gestalten.** Ein Lesetreffer im Block-Cache soll nach
Möglichkeit keinen exklusiv sperrenden, mutierenden Zugriff erfordern, da Cache-Treffer der mit Abstand häufigste
Zugriffspfad sind und jede darin verborgene Schreibsperre unter Last zur Kontention wird (§5.4).

### Ergänzende Grundsätze

- **Nebenläufigkeitssicherheit vor Nebenläufigkeitsperformance:** Sperrenhierarchien werden explizit dokumentiert
  und dürfen nicht durch bloßen Analogieschluss auf neue Mutationspfade übertragen werden, ohne die
  Deadlockfreiheit für den neuen Fall erneut zu beweisen (konkretes Beispiel: §6.5, H2).

- **Geschlossene Enums bleiben geschlossen:** Wo ein Enum bewusst **nicht** `#[non_exhaustive]` deklariert ist
  (z. B. `SignalKind`), ist das eine architektonische Entscheidung. Eine neue Kategorie von Information wird
  in ein bestehendes offenes Signal integriert, statt das Enum breaking zu erweitern (§6.5, H3).

- **Kein Sicherungsnetz, keine Schema-Änderung:** Persistenzformat-Änderungen werden nur vorgenommen, wenn ein
  automatisiertes CI-Drift-Gate zwischen Schema und generiertem Code aktiv läuft (§6.5, H4).

- **Explizite Unvollständigkeit statt stiller Lücken:** Wo ein Subsystem eine neue Datenklasse strukturell nicht
  berücksichtigt, wird dies über ein sichtbares Konfigurations-/Report-Flag markiert (§6.5, H6).

---

<a id="4-architektur"></a>
## 4. Systemarchitektur: der Crate-DAG

MemFuse gliedert sich in einen mehrschichtigen Rust-Workspace. Abhängigkeiten verlaufen strikt abwärts; eine
Abhängigkeit, die gegen die Schichtrichtung verstößt, gilt als Architekturdefekt, nicht als Stilfrage.

| Layer | Crates | Verantwortung |
|---|---|---|
| **0** | `memfuse-core-ipc-gen`, `memfuse-core` | FlatBuffers-generierter IPC-Code; Kerntypen (`DocId`, `EntityId`, `TxId`, `ConfigFingerprint`), Traits, Fehlerbehandlung. |
| **1** | `memfuse-store` (LSM-Tree-Storage, WAL, Block-Cache), `memfuse-index` (HNSW/DiskANN-Vektorindex, SIMD-Distanz), `memfuse-text` (BM25/BM25F-Volltextindex, deutsche Morphologie), `memfuse-crypto` (AES-256-GCM-SIV, KV-Segment-Security, Deletion-Proof-Kette), `memfuse-graph` (CSR-Graph, PathRAG, Forward-Push-PPR, Leiden-Community-Detection, Hyperkanten), `memfuse-checkpoint` (Snapshotting), `memfuse-calibration` (Score-Kalibrierung, Drift-Erkennung) | Persistenz- und Indexierungs-Primitive. Keine Kenntnis voneinander außerhalb dieser Schicht. |
| **2** | `memfuse-db` | Öffentliche `Collection`-API, 4-Signal-Fusion, Multi-Step-Query-Engine, Kontext-Kompaktierung, Provenance-Tracking. Konsumiert alle Layer-1-Crates. |
| **3** | `memfuse-ollama` (Ollama-Client, Contextual-Chunk-Prefixing), `memfuse-candle` (natives GGUF-Inferenz-Backend, KV-Cache-Bridge), `memfuse-embed` (ONNX-Embeddings, Cross-Encoder-Reranking, feature-gated), `memfuse-agent` (persistente Agent-Workflow-Engine), `memfuse-router` (Contextual-Bandit-Routing), `memfuse-py` (Python-FFI via PyO3, eigener Cargo-Workspace) | Inferenz-Backends und Anwendungslogik oberhalb der Datenschicht. |
| **4** | `memfuse-mcp` (MCP-Server, Sandbox, Cloud-Egress-Gateway), `memfuse-sandbox` (WASM Execution Boundary) | Externe Schnittstelle für Agenten (stdio-JSON-RPC). |
| **5** | `memfuse-bench` | Reproduzierbarer Benchmark-Harness für Retrieval-Genauigkeit und Latenz. |

### 4.1 Safety-First-Doktrin

Safe Rust ist der Standard; `#![forbid(unsafe_code)]` gilt per Default und wird nur in einer geschlossenen,
dokumentierten Ausnahmeliste durchbrochen — jeweils mit einem `// SAFETY:`-Beweiskommentar direkt am Code, der die
Invarianten (insbesondere Pointer-Alignment) beweist (siehe §0.4 für die vollständige Ausnahmeliste).

### 4.2 Trait-Eindeutigkeit und Modul-Governance (⚠️ Opus-Optimierung 2.6, Stufe 2, mittel)

**Problem:** `memfuse-core/src/traits/` enthält zehn Dateien; `mod.rs` deklariert nur fünf. Vier Dateien
(`graph_index.rs`, `lifecycle.rs`, `text_index.rs`, `vector_index.rs`) sind dadurch **nie kompiliert** und
enthalten Zweitdefinitionen von neun Verträgen (`VectorIndex`, `TextEmbeddingEngine`, `SegmentSynthesizer`,
`TextIndex`, `GraphIndex`, `DistanceCalculator`, `MemoryLifecycleManager`, `GroundingValidator`,
`ResponseGroundingValidator`), die parallel in `index.rs`/`observability.rs` leben. `GraphIndex` ist zwischen <!-- doc-ref-ignore -->
beiden Fassungen bereits inhaltlich auseinandergelaufen (abweichende Doc-Kontrakte und Default-Implementierungen)
— der Compiler kann das nicht erkennen, weil die unverdrahtete Fassung nie gebaut wird. Das bestehende
Duplikat-Gate (`xtask check_duplicate_symbols`) prüft laut eigener Spezifikation nur *innerhalb derselben Datei*
und ist für dateiübergreifende Duplikate im selben Modulverzeichnis strukturell blind.

**Einordnung:** Die vier unverdrahteten Dateien sind die **korrekte** Zerlegung (ein Vertrag pro Datei); der
tatsächlich kompilierte Zustand ist der Monolith `index.rs` (drei unabhängige Verträge in einer Datei). Eine <!-- doc-ref-ignore -->
Lösung, die schlicht die vier Dateien löscht, würde die bessere Zerlegung entfernen und die falsche behalten.

**Maßnahme (verbindlich):**
1. Divergenz in `GraphIndex` auflösen — die aktuell kompilierte Fassung in `index.rs` ist die Referenz für die <!-- doc-ref-ignore -->
   Zusammenführung, nicht automatisch die inhaltlich richtige.
2. `graph_index`, `vector_index`, `text_index`, `lifecycle` in `traits/mod.rs` deklarieren. <!-- doc-ref-ignore -->
3. `index.rs` und die duplizierten Teile von `observability.rs` löschen, sobald (1)/(2) grün sind. <!-- doc-ref-ignore -->
4. **Governance-Gate `GOV-D` (neu, CI-Pflicht):** `xtask check-module-reachability` verifiziert, dass jede
   `.rs`-Datei unter `src/` von genau einer `mod`-Deklaration aus erreichbar ist. Eine unerreichbare, aber
   vorhandene Datei ist ein CI-Fehler, kein stiller Zustand — dies schließt exakt die Lücke, die `CORE-D`
   ermöglicht hat.

**Testpflicht:** `crates/memfuse-core/tests/no_orphan_modules.rs` — schlägt fehl, sobald eine Datei unter `src/` <!-- doc-ref-ignore -->
existiert, die von keinem `mod`-Pfad aus erreichbar ist.

---

<a id="5-speicher"></a>
## 5. Speicherschicht: LSM-Tree, WAL und Block-Cache

### 5.1 Grundprinzip und Lock-Hierarchie

Keine Zustandsänderung wird im Speicher sichtbar gemacht, bevor sie physisch in das Write-Ahead-Log geschrieben
und mit dem Datenträger synchronisiert wurde (WAL-First, P2). Der Systemzustand muss sich allein aus dem Log
rekonstruieren lassen (deterministische Recovery, P3). Schreibzugriffe sperren nicht die gesamte Collection, sondern
nur die betroffenen Schlüssel über eine key-granulare Lock-Hierarchie:

```
collections (RwLock) → kv_locks (schlüssel-granular, KvKeyLocks) → embedder (RwLock)
```

Diese Hierarchie ist für **Einzelschlüssel**-Mutationen ausgelegt und deadlockfrei bewiesen. Jede künftige
Mutation, die mehrere Schlüssel gleichzeitig unter `kv_locks` hält, muss diesen Beweis für den Mehrschlüsselfall
gesondert führen — sie darf sich nicht per Analogieschluss auf den Einzelschlüsselfall berufen (konkret
angewendet in §6.5, H2).

### 5.2 Modulstruktur `memfuse-store`

```
crates/memfuse-store/src/
├── lib.rs
├── lsm.rs               # LSM-Tree, Compaction <!-- doc-ref-ignore -->
├── wal.rs                # Write-Ahead-Log + HMAC-Kette <!-- doc-ref-ignore -->
├── wal_ring_buffer.rs    # SPSC-Ring-Puffer (§5.3) <!-- doc-ref-ignore -->
├── block_cache/
│   ├── mod.rs            # BlockCacheBackend-Trait
│   ├── lru.rs            # LruBlockCacheBackend (Default) <!-- doc-ref-ignore -->
│   └── sieve.rs          # SieveCacheBackend (Opt-in, `block-cache-v2`) <!-- doc-ref-ignore -->
├── kv_locks.rs           # KvKeyLocks (§5.2a) <!-- doc-ref-ignore -->
└── error.rs
```

### 5.2a Key-granulares Locking: `kv_locks.rs` (normativ) <!-- doc-ref-ignore -->

```rust
use ahash::AHashMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

/// Sperrenhierarchie (verbindlich, systemweit einzuhalten):
///   collections (RwLock) → kv_locks (schlüssel-granular) → embedder (RwLock)
pub struct KvKeyLocks {
    shards: Vec<RwLock<()>>,
    shard_mask: u64,
}

pub struct KeyGuard<'a> {
    _guard: RwLockWriteGuard<'a, ()>,
}

pub struct MultiKeyGuard<'a> {
    _guards: Vec<RwLockWriteGuard<'a, ()>>,
}

impl KvKeyLocks {
    pub fn new(shard_count_pow2: u32) -> Self {
        let n = 1u64 << shard_count_pow2;
        Self {
            shards: (0..n).map(|_| RwLock::new(())).collect(),
            shard_mask: n - 1,
        }
    }

    fn shard_for(&self, key_hash: u64) -> usize {
        (key_hash & self.shard_mask) as usize
    }

    pub fn acquire(&self, key_hash: u64) -> KeyGuard<'_> {
        let idx = self.shard_for(key_hash);
        KeyGuard { _guard: self.shards[idx].write().unwrap() }
    }

    /// H2-Pflichtmethode: Erwirbt N Shards STRIKT in aufsteigender Shard-Index-Reihenfolge.
    pub fn acquire_multi_sorted(&self, sorted_key_hashes: &[u64]) -> Result<MultiKeyGuard<'_>, LockError> {
        let mut shard_indices: Vec<usize> = sorted_key_hashes.iter()
            .map(|h| self.shard_for(*h)).collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();
        let guards = shard_indices.iter()
            .map(|&idx| self.shards[idx].write().map_err(|_| LockError::Poisoned))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MultiKeyGuard { _guards: guards })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock poisoned")]
    Poisoned,
    #[error("lock acquisition timed out")]
    Timeout,
}
```

**Loom-Testpflicht:** `crates/memfuse-store/tests/loom_multi_key_lock.rs` MUSS unter `#[cfg(loom)]` zwei <!-- doc-ref-ignore -->
nebenläufige `acquire_multi_sorted`-Aufrufe mit überlappenden, unterschiedlich sortierten Schlüsselmengen
modellieren und deren Terminierung ohne Deadlock nachweisen.

### 5.3 WAL-Ring-Puffer: `wal_ring_buffer.rs` (normativ) <!-- doc-ref-ignore -->

Die klassische Implementierung leidet unter geteilter Eigentümerschaft am File-Handle, was zu HMAC-Ketten-Forks
und stillen Datenverlusten führen kann. Die Zielarchitektur sieht eine lock-freie WAL-Pipe auf Basis eines
Single-Producer-Single-Consumer-(SPSC-)Ring-Puffers vor.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

/// SPSC-Ring-Puffer. `capacity` MUSS eine Zweierpotenz sein (Invariante wird in `new` erzwungen),
/// um Modulo durch `& (capacity - 1)` zu ersetzen.
pub struct WalRingBuffer {
    buf: Box<[std::mem::MaybeUninit<WalEntry>]>,
    capacity_mask: usize,
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}

pub struct WalEntry {
    pub payload: Vec<u8>,
    pub hmac_prev: [u8; 32],
}

impl WalRingBuffer {
    pub fn new(capacity_pow2: usize) -> Result<Self, WalError> {
        if !capacity_pow2.is_power_of_two() {
            return Err(WalError::CapacityNotPowerOfTwo);
        }
        Ok(Self {
            buf: (0..capacity_pow2).map(|_| std::mem::MaybeUninit::uninit()).collect(),
            capacity_mask: capacity_pow2 - 1,
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        })
    }

    /// Producer-Seite: `Ordering::Release` beim Veröffentlichen des neuen write_idx.
    pub fn try_push(&self, entry: WalEntry) -> Result<(), WalEntry> { unimplemented!() }

    /// Consumer-Seite (Flusher-Task, exklusiv): `Ordering::Acquire` beim Lesen von write_idx.
    pub fn try_pop(&self) -> Option<WalEntry> { unimplemented!() }
}

#[derive(Debug, thiserror::Error)]
pub enum WalError {
    #[error("ring buffer capacity must be a power of two")]
    CapacityNotPowerOfTwo,
    #[error("hmac chain fork detected at sequence {0}")]
    HmacChainFork(u64),
}
```

Der Flusher-Task ist der **einzige** Aufrufer von `fsync`; er läuft als dedizierter `tokio::task`, der per `mpsc`
über neue `try_pop`-Ergebnisse benachrichtigt wird, statt zu pollen.

**⚠️ Opus-Optimierung 0.1 — WAL-Replay-Panic entschärfen (Stufe 0, gering):**
Die Replay-Routine liest die Dateigröße einmalig vor dem `mmap`, prüft Zugriffsgrenzen aber gegen diesen separat
gehaltenen Wert statt gegen die tatsächliche Länge der gemappten Region. Maßnahme: Dateigröße ausschließlich aus
`mmap.len()` ableiten, alle Slice-Zugriffe auf `mmap.get(a..b)` mit `.ok_or(WalCorruption)` umstellen. Die zweite
parallele Scan-Implementierung auf denselben Hilfsfunktions-Pfad reduzieren.

**⚠️ Opus-Optimierung 0.5 — Recovery-Pfad differenzieren (Stufe 0, mittel):**
Den Intent-Datensatz um einen expliziten Ergebnisstatus (committed/aborted) erweitern und bei Repair-on-Open
auswerten, statt pauschal vorwärts zu committen.

### 5.4 Block-Cache: `BlockCacheBackend`-Trait (normativ)

```rust
pub trait BlockCacheBackend<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V);
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
}
```

**`block_cache/lru.rs` (🟢 Produktions-Default):** <!-- doc-ref-ignore -->

```rust
pub struct LruBlockCacheBackend<K, V> {
    inner: RwLock<lru::LruCache<K, V>>,
}

impl<K: Hash + Eq + Clone, V: Clone + Send + Sync> BlockCacheBackend<K, V>
    for LruBlockCacheBackend<K, V>
{
    fn get(&self, key: &K) -> Option<V> {
        // Cache-Hit erfordert Write-Lock, da LRU-Reordering mutiert (P25-Verstoß, dokumentiert als
        // bewusster Trade-off des Default-Pfads — siehe SieveCacheBackend für den lock-freien Pfad).
        self.inner.write().unwrap().get(key).cloned()
    }
    fn insert(&self, key: K, value: V) { self.inner.write().unwrap().put(key, value); }
    fn capacity(&self) -> usize { self.inner.read().unwrap().cap().get() }
    fn len(&self) -> usize { self.inner.read().unwrap().len() }
}
```

**`block_cache/quick_cache.rs` (🟡 Opt-in `block-cache-v2`, produktiv im Code vorhanden):** <!-- doc-ref-ignore -->

Der tatsächlich implementierte lock-günstige Pfad wrappt den `quick_cache`-Crate (S3-FIFO-artige Eviction über
drei FIFO-Warteschlangen: Small ≈ 10 % Kapazität mit „Quick Demotion" für One-Hit-Wonders, Main, Ghost) als
`QuickCacheBlockCacheBackend`. Dies ist der Stand, der durch `FINAL_12` §0.3 am Code verifiziert ist — `LRU`
bleibt der Produktions-Default, `block-cache-v2` schaltet auf `QuickCacheBlockCacheBackend` um.

```rust
use quick_cache::sync::Cache as QuickCache;

pub struct QuickCacheBlockCacheBackend<K, V> {
    inner: QuickCache<K, V>,
}

impl<K, V> BlockCacheBackend<K, V> for QuickCacheBlockCacheBackend<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> { self.inner.get(key) }
    fn insert(&self, key: K, value: V) { self.inner.insert(key, value); }
    fn capacity(&self) -> usize { self.inner.capacity() as usize }
    fn len(&self) -> usize { self.inner.len() }
}
```

**`block_cache/sieve.rs` (🔴 Zielarchitektur, löst `QuickCacheBlockCacheBackend` perspektivisch ab):** <!-- doc-ref-ignore -->

SIEVE ist die noch radikalere Reduktion gegenüber S3-FIFO: Es verzichtet vollständig auf Listen-Neuordnung bei
Lesetreffern. Ein Cache-Hit reduziert sich auf das Setzen eines einzigen atomaren `visited`-Bits
(`Ordering::Relaxed`), ohne jede Mutation der Listenstruktur — kein Aufruf in eine fremde Crate-Implementierung,
volle Kontrolle über das Speicherlayout. Eviction erfolgt über einen umlaufenden Zeiger („Hand"): gesetztes
`visited`-Bit → begnadigt (Bit gelöscht, verbleibt im Cache), gelöschtes Bit → verdrängt.

**Verbindliche Einordnung (löst den Widerspruch zwischen `Cargo.toml`-Abhängigkeit und Zielarchitektur auf):**
`quick_cache` bleibt so lange die deklarierte Abhängigkeit und `QuickCacheBlockCacheBackend` der Inhalt von
`block-cache-v2`, bis `SieveCacheBackend` denselben Trait implementiert, denselben Loom-/Benchmark-Nachweis wie
`QuickCacheBlockCacheBackend` erbringt und per ADR als Ablösung beschlossen wird — erst danach wird die
`quick_cache`-Abhängigkeit aus `Cargo.toml` entfernt. Bis dahin ist `SieveCacheBackend` ein zusätzlicher,
nicht kompilierter Zielentwurf unter `#[cfg(feature = "block-cache-sieve-experimental")]`, kein Ersatz für den
bestehenden Opt-in-Pfad.

```rust
use crossbeam_epoch::{self as epoch, Atomic, Owned, Shared};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct SieveNode<K, V> {
    pub key: K,
    pub value: V,
    pub visited: AtomicBool,
    pub next: Atomic<SieveNode<K, V>>,
}

pub struct SieveCacheBackend<K, V> {
    head: Atomic<SieveNode<K, V>>,
    tail: Atomic<SieveNode<K, V>>,
    hand: Atomic<SieveNode<K, V>>,
    capacity: usize,
    size: AtomicUsize,
    index: scc::HashMap<K, ()>,
}

impl<K, V> BlockCacheBackend<K, V> for SieveCacheBackend<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> {
        let guard = epoch::pin();
        self.lookup_node(key, &guard).map(|node| {
            node.visited.store(true, Ordering::Relaxed);
            node.value.clone()
        })
    }

    fn insert(&self, key: K, value: V) {
        if self.size.load(Ordering::Relaxed) >= self.capacity {
            self.evict_one();
        }
        self.push_front(key, value);
    }

    fn capacity(&self) -> usize { self.capacity }
    fn len(&self) -> usize { self.size.load(Ordering::Relaxed) }
}
```

Epochenbasierte Speicherfreigabe (Epoch-Based Reclamation, EBR) verhindert ABA-Probleme und Use-After-Free.

**Sharding:** Alle Backends werden über ein `ShardedBlockCache<K, V, B: BlockCacheBackend<K,V>>` mit
konfigurierbarer Shard-Zahl (Default 16, `ahash`-basiertes Routing) gekapselt.

**⚠️ Opus-Optimierung 1.7 — Byte-basierte Cache-Kapazität (Stufe 1, mittel):**
Kapazität byte-basiert statt eintragsbasiert führen (Eviction anhand der tatsächlichen Bytegröße) — gilt für
`QuickCacheBlockCacheBackend` und die perspektivische `SieveCacheBackend` gleichermaßen.

### 5.5 Öffentliche Storage-API

```rust
pub struct LsmStore {
    wal: WalRingBuffer,
    block_cache: Box<dyn BlockCacheBackend<BlockId, Bytes>>,
    kv_locks: KvKeyLocks,
}

impl LsmStore {
    pub fn get(&self, prefix: &str, key: &[u8]) -> Result<Option<Bytes>, StoreError>;
    pub fn put(&self, prefix: &str, key: &[u8], value: Bytes) -> Result<(), StoreError>;
    pub fn delete(&self, prefix: &str, key: &[u8]) -> Result<DeletionProof, StoreError>;
    pub fn scan_prefix(&self, prefix: &str) -> Result<impl Iterator<Item = (Vec<u8>, Bytes)>, StoreError>;
    pub fn compact(&self) -> Result<(), StoreError>;
    pub fn compact_async(&self, max_compaction_peak_memory_mb: usize) -> tokio::task::JoinHandle<Result<(), StoreError>>;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)] Wal(#[from] WalError),
    #[error(transparent)] Lock(#[from] LockError),
    #[error("compaction memory budget {budget_mb}MB exceeded (estimated {estimated_mb}MB)")]
    CompactionBudgetExceeded { budget_mb: usize, estimated_mb: usize },
    #[error(transparent)] Io(#[from] std::io::Error),
}
```

**⚠️ Opus-Optimierung 1.4 — SSTable Zero-Copy-Slice (Stufe 1, trivial):**
Beim Lesen eines Datenblocks nach CRC-Prüfung: referenzzählendes Slicing statt vollständiger Kopie.

**⚠️ Opus-Optimierung 2.4 — Manifest-Batch-Fsync (Stufe 2, mittel):**
Zusammengehörige Manifest-Änderungen in einem Batch, ein `fsync` pro Zustandsübergang statt pro Einzeleintrag.

**⚠️ Opus-Optimierung 1.6 — MemTable Range-Sharding (Stufe 1, hoch):**
Sharding-Grenzen aus dem Namensraum-Präfix ableiten, sodass Flush sortierfrei und Präfix-Scan auf eine Partition
beschränkt wird.

### 5.6 Compaction/MANIFEST-Atomarität (⚠️ Opus-Optimierung 0.6, Stufe 0, gering — STO-A)

**Einordnung:** Dies ist einer der fünf systemweit schwerwiegendsten Befunde des Architektur-Reviews, auf
derselben Prioritätsstufe wie der WAL-Replay-Panic-Fix (§5.3, Opus 0.1), und gehört ebenso in Stufe 0.

**Problem:** `maybe_compact()` schreibt Manifest-Änderungen nicht als einen atomaren Übergang, sondern als
Sequenz: (1) `manifest.append(Add { output_path })`, (2) In-Memory-Swap, (3) `manifest.append(Remove { old })` +
Löschen der alten Dateien — Fehler in (3) werden nur geloggt, nicht propagiert. Ein Absturz oder ein regulärer
Abbruch zwischen (1) und (3) hinterlässt ein MANIFEST, das die gemergte **und** alle Input-SSTables als gültig
führt. Beim Recovery werden beide geladen; bei einer vollständigen Kompaktierung verwirft der Merge Tombstones —
die alten SSTables enthalten die gelöschten Versionen jedoch noch. **Gelöschte Daten können nach einem Absturz
zurückkehren.** Für ein System mit kryptographischer `DeletionProof`-Zusicherung (Art. 17 DSGVO / Machine
Unlearning) ist das keine reine Storage-Performance-Frage, sondern ein Bruch des Sicherheitsversprechens aus §10.

Zusätzlich: Ein beschädigtes oder nicht ladbares MANIFEST degradiert aktuell still auf „lade jede `.sst`-Datei im
Verzeichnis" — ein Verstoß gegen die projektweite „No Silent Failures"-Doktrin (P4/P6) — und die Shadowing-
Reihenfolge zwischen Merge-Ausgabe und ihren Inputs wird nicht persistiert, sondern nach Recovery lexikografisch
neu geraten.

**Lösung (verbindlich):**

```rust
/// Ein Zustandsübergang = ein Record, ein fsync. Ersetzt die bisherige Add/Remove-Paar-Sequenz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub kind: ManifestEntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManifestEntryKind {
    /// Atomarer Kompaktierungs-Übergang: entfernte Dateien, neue Datei, Position (löst die
    /// Shadowing-Reihenfolge-Frage — die Position gehört in den Record, nicht in die Recovery-Heuristik).
    Replace { removed: Vec<PathBuf>, added: PathBuf, position: u32 },
    Add { path: PathBuf },
}

impl Manifest {
    /// Geschrieben und ge-fsynct NACH erfolgreichem Merge, VOR dem In-Memory-Swap.
    /// Ein halb geschriebener Record fällt über die CRC-Prüfung heraus — der Vorzustand gilt dann als aktuell.
    pub fn commit_replace(&self, removed: Vec<PathBuf>, added: PathBuf, position: u32)
        -> Result<(), ManifestError>;

    /// MUSS `Err` propagieren — kein Fallback auf Verzeichnis-Scan bei Ladefehler.
    pub fn load(path: &Path) -> Result<Vec<ManifestEntry>, ManifestError>;

    /// Rollover per `MANIFEST.new` + atomarem `rename` statt unbegrenztem Wachstum der Historie —
    /// Startzeit wird proportional zu den aktiven SSTables statt zur vollständigen Historie.
    pub fn rollover(&self) -> Result<(), ManifestError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest record failed CRC check at offset {0}, previous state retained")]
    CorruptRecord(u64),
    #[error(transparent)] Io(#[from] std::io::Error),
}
```

Kandidatenauswahl für Compaction wird zusätzlich in **einem** Lock-Fenster gelesen und verwendet (nicht über
drei getrennte Fenster hinweg), um den zugehörigen Out-of-Bounds-Panic-Pfad bei nebenläufiger Compaction
auszuschließen.

**Testpflicht:** `crates/memfuse-store/tests/manifest_crash_no_resurrection.rs` — simuliert einen Absturz nach <!-- doc-ref-ignore -->
Schritt (1) im alten Modell (bzw. nach dem `commit_replace`-`fsync` im neuen Modell) und belegt, dass nach
Recovery keine Tombstone-Version aus den alten SSTables sichtbar wird.

---

<a id="6-graph"></a>
## 6. Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten

### 6.1 Binäre Kanten als Grundmodell (🟢)

Der Wissensgraph wird primär als gerichteter, gewichteter Graph in einer CSR-Struktur gehalten:

```rust
#[derive(Debug, Clone)]
pub struct Edge {
    pub target: EntityId,
    pub weight: f32,
    pub edge_type: EdgeType,
    pub tx_valid_from: TxId,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}
```

`EdgeType` ist als `#[non_exhaustive] enum { Default }` deklariert. Kanten tragen sowohl transaktionale (MVCC)
als auch fachliche (Business-Zeit) Gültigkeit — bi-temporal.

`DocId` (Default `u64`, feature-gated `u128` via BLAKE3-Truncation) und `EntityId` (`u64`) bilden die gemeinsame
Identitätsgrundlage:

```rust
#[cfg(not(feature = "docid-128"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

impl DocId {
    pub fn derive(collection_key: &[u8], seq: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(collection_key);
        hasher.update(&seq.to_le_bytes());
        let digest = hasher.finalize();
        #[cfg(not(feature = "docid-128"))]
        { Self(u64::from_le_bytes(digest.as_bytes()[0..8].try_into().unwrap())) }
        #[cfg(feature = "docid-128")]
        { Self(u128::from_le_bytes(digest.as_bytes()[0..16].try_into().unwrap())) }
    }
}
```

### 6.2 Modulstruktur `memfuse-graph`

```
crates/memfuse-graph/src/
├── lib.rs
├── csr.rs             # CsrGraph, GraphInner, ArcSwap-RCU
├── edge.rs            # Edge, EdgeType-Nutzung, PersistedEdgePayload <!-- doc-ref-ignore -->
├── hyperedge.rs       # HyperEdge, RoleBinding, RoleId, HyperEdgeId — 🔴
├── ppr.rs             # Forward-Push (Andersen-Chung-Lang)
├── path_rag.rs        # PathGraph, bidirektionale Suche, Hyperkanten-Expansion
├── community.rs       # Leiden, Stern-Expansion-Projektion
├── cascade.rs         # Cascade-Invalidierung binär + Hyperkanten
└── error.rs
```

### 6.3 RCU-Snapshot-Architektur: `csr.rs` (normativ)

```rust
use arc_swap::ArcSwap;
use ahash::AHashMap;
use std::sync::Arc;

pub struct GraphInner {
    pub adjacency: Vec<Vec<Edge>>,
    pub node_index: AHashMap<EntityId, usize>,
    // 🔴 NEU (H1): Teil von GraphInner, NICHT separat — automatisch vom ArcSwap miterfasst.
    pub hyperedges: AHashMap<HyperEdgeId, HyperEdge>,
    pub hyperedge_index: AHashMap<EntityId, Vec<HyperEdgeId>>,
}

impl GraphInner {
    /// MUSS um Hyperkanten-Anteile erweitert sein (H1).
    pub fn estimate_memory_bytes(&self) -> usize {
        let edge_bytes: usize = self.adjacency.iter()
            .map(|v| v.len() * std::mem::size_of::<Edge>()).sum();
        let hyperedge_bytes: usize = self.hyperedges.values()
            .map(|h| std::mem::size_of::<HyperEdge>() + h.participants.len() * std::mem::size_of::<RoleBinding>())
            .sum();
        let hyperedge_index_bytes: usize = self.hyperedge_index.values()
            .map(|v| v.len() * std::mem::size_of::<HyperEdgeId>())
            .sum();
        edge_bytes + hyperedge_bytes + hyperedge_index_bytes
    }
}

pub struct CsrGraph {
    inner: ArcSwap<GraphInner>,
    kv_locks: Arc<memfuse_store::KvKeyLocks>,
}

impl CsrGraph {
    /// Atomarer Snapshot-Austausch. Leser sehen NIE einen gemischten Alt-/Neu-Zustand.
    pub fn compact(&self) -> Result<(), GraphError> {
        let old = self.inner.load();
        let new_inner = Self::rebuild(&old)?;
        self.inner.store(Arc::new(new_inner));
        Ok(())
    }

    pub fn compact_async(&self, max_compaction_peak_memory_mb: usize)
        -> tokio::task::JoinHandle<Result<(), GraphError>>
    {
        let estimated = self.inner.load().estimate_memory_bytes() / (1024 * 1024);
        if estimated > max_compaction_peak_memory_mb {
            return tokio::spawn(async move {
                Err(GraphError::CompactionBudgetExceeded {
                    budget_mb: max_compaction_peak_memory_mb,
                    estimated_mb: estimated,
                })
            });
        }
        unimplemented!()
    }

    pub fn neighbors_with_weights(&self, id: EntityId) -> Vec<(EntityId, f32)> { unimplemented!() }

    /// Sekundärindex-Zugriff, additiv — kein Eingriff in CSR-Adjazenzstruktur.
    pub fn hyperedges_for_entity(&self, id: EntityId) -> Vec<HyperEdgeId> {
        self.inner.load().hyperedge_index.get(&id).cloned().unwrap_or_default()
    }
}
```

**⚠️ Opus-Optimierung 2.1 — Inkrementelle Graph-Kompaktierung (Stufe 2, hoch):**
PPR-Pfad löst bei jeder Anfrage vollständigen CSR-Rebuild aus. Ziel: append-only Delta-Segmente plus
periodischer Merge im Hintergrund.

**⚠️ Opus-Optimierung 2.2 — CSR-Sentinel statt `Option` (Stufe 2, mittel):**
Mehrere Kantenspalten als `Vec<Option<T>>` — ca. Halbierung des Speicherbedarfs pro Kante durch Sentinel-Werte.

### 6.4 Hyperkanten: Datenstruktur (🔴 vollständig spezifiziert)

**Designentscheidung (verbindlich):** Es wird **kein** generisches RDF-Reifikations-Pattern verwendet.
Stattdessen wird eine kohärente, erstklassige Rust-Struktur mit Zero-Copy-Deserialisierung via FlatBuffers/Mmap
spezifiziert. **Zwei Repräsentationen, ein Persistenzformat:** Der Schreibpfad (`relate_n_ary`) konstruiert eine
neue Hyperkante ohnehin aus frisch übergebenen Daten — dort ist eine besitzende Struktur korrekt und einfach.
Der Lesepfad (`hyperedges_for_entity`/Traversal) dereferenziert dagegen bei **jeder** Anfrage potenziell
tausende bereits persistierter Hyperkanten aus dem RCU-Snapshot; hier erzwingt eine besitzende `Vec<RoleBinding>`
pro gelesener Hyperkante eine Heap-Kopie, obwohl die Daten bereits deserialisiert im Snapshot-Speicher liegen.
Die Lesesicht referenziert diesen Speicher stattdessen zero-copy:

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HyperEdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role: RoleId,
    pub entity: EntityId,
}

/// Referenzzählender Slice auf einen zusammenhängenden `RoleBinding`-Bereich innerhalb des
/// Mmap-/RCU-Snapshot-Speichers. Hält nur Pointer + Länge + geteilten Referenzzähler auf den
/// zugrundeliegenden `Arc`-Puffer — kein `Vec`-Allocation-Overhead beim Lesen.
#[derive(Clone)]
pub struct ArcSlice<T> {
    backing: Arc<[T]>,
    start: u32,
    len: u32,
}

impl<T> std::ops::Deref for ArcSlice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] { &self.backing[self.start as usize..(self.start + self.len) as usize] }
}

/// Besitzende Variante — Konstruktionspfad (`relate_n_ary`, Deserialisierung beim Schreiben).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: Vec<RoleBinding>,         // min. 2, validiert in `relate_n_ary`
    pub weight: f32,
    pub tx_valid_from: Option<TxId>,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}

/// Zero-Copy-Lesesicht — Traversal-Hotpath (`hyperedges_for_entity`, PPR-Expansion, §7.2).
/// `participants` referenziert den RCU-Snapshot-Speicher direkt statt ihn zu kopieren.
#[derive(Clone)]
pub struct HyperEdgeView {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: ArcSlice<RoleBinding>,
    pub weight: f32,
    pub source_doc_id: Option<DocId>,
}

impl HyperEdge {
    /// Erzeugt die Zero-Copy-Sicht ohne die `participants` zu kopieren — teilt sich den `Arc`
    /// mit dem im `GraphInner` gehaltenen Original (siehe §6.3, H1).
    pub fn as_view(self: &Arc<Self>) -> HyperEdgeView {
        HyperEdgeView {
            id: self.id,
            predicate: self.predicate,
            participants: ArcSlice {
                backing: Arc::from(self.participants.as_slice()),
                start: 0,
                len: self.participants.len() as u32,
            },
            weight: self.weight,
            source_doc_id: self.source_doc_id,
        }
    }
}

/// Rollen-Interner — dieselbe Interning-Strategie wie EdgeType/Prädikate.
pub struct RoleInterner {
    forward: scc::HashMap<String, RoleId>,
    backward: scc::HashMap<RoleId, String>,
    next_id: std::sync::atomic::AtomicU32,
}

impl RoleInterner {
    pub fn intern(&self, name: &str) -> RoleId;
    pub fn resolve(&self, id: RoleId) -> Option<String>;
}
```

**Konsequenz für `GraphInner` (§6.3, H1):** `hyperedges: AHashMap<HyperEdgeId, Arc<HyperEdge>>` (nicht `HyperEdge`
direkt) — erst das `Arc` macht `as_view()` zero-copy-fähig, da mehrere `HyperEdgeView`s denselben Teilnehmer-
Speicher teilen können, ohne dass ihre Lebensdauer an eine geliehene Referenz auf `GraphInner` gebunden ist
(wichtig, weil `GraphInner` selbst per `ArcSwap` ausgetauscht wird, siehe H1-Lösung unten).

**Persistenz:** Neues LSM-Präfix `__graph:hyperedge:`, Value = FlatBuffers-serialisiertes `HyperEdge`.
Sekundärindex `__graph:hyperedge_by_entity:{EntityId} -> Vec<HyperEdgeId>`. Additiv — kein Breaking Change.

**API-Oberfläche:**
```rust
impl Collection {
    /// Binärer Pfad — bleibt Hotpath, KEINE interne Umleitung auf `relate_n_ary`.
    pub fn relate(&self, from: EntityId, to: EntityId, predicate: EdgeType, doc_id: DocId) -> Result<(), DbError>;

    /// NEU — Hyperkanten-Schreibpfad.
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[(RoleId, EntityId)],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, DbError>;
}
```

### 6.5 Traversal-Semantik

PathRAG wird um einen optionalen Hyperkanten-Expansionsschritt ergänzt: Beim Erreichen eines Knotens während der
Forward-Push-Traversierung werden zusätzlich alle `RoleBinding`-Partner als „virtuelle" Nachbarn mit
rollenspezifischem Gewichtsabschlag (Startwert `0.85`) eingespeist.

### 6.6 Integrationshindernisse H1–H6 und ihre verbindliche Lösung

Diese sechs Hindernisse benennen nicht nur, *dass* eine Integration möglich ist, sondern *welches bestehende
Invariant unter Druck gerät* und wie es gewahrt bleibt.

#### H1 — RCU-Snapshot-Inkonsistenz zwischen CSR und Hyperkanten-Sekundärindex

**Problem:** Separater Hyperkanten-Index außerhalb von `GraphInner` → Zeitfenster für inkonsistenten Zustand.

**Lösung (verbindlich):** Der Hyperkanten-Index wird **Teil von `GraphInner`** selbst — automatisch vom
`ArcSwap`-Swap miterfasst. `GraphInner::estimate_memory_bytes()` MUSS Hyperkanten einschließen.

#### H2 — Kanonisches Multi-Key-Locking zur Deadlock-Prävention

**Problem:** `relate_n_ary()` mit N Teilnehmern muss N Entitäten gleichzeitig unter `kv_locks` halten.
Naives Lock-Ordering → Deadlock bei überlappenden, unterschiedlich sortierten Mengen.

**Lösung (verbindlich):**

```rust
impl CsrGraph {
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError> {
        if participants.len() < 2 {
            return Err(GraphMutationError::InsufficientParticipants(participants.len()));
        }

        // 1. Kanonische Sortierung (H2) — Grundlage der Deadlockfreiheit.
        let mut entities: Vec<EntityId> = participants.iter().map(|p| p.entity).collect();
        entities.sort_unstable_by_key(|e| e.0);
        entities.dedup();
        let key_hashes: Vec<u64> = entities.iter()
            .map(|e| ahash::RandomState::new().hash_one(e)).collect();

        // 2. Multi-Key-Lock in sortierter Shard-Reihenfolge.
        let _guards = self.kv_locks.acquire_multi_sorted(&key_hashes)
            .map_err(|_| GraphMutationError::LockAcquisitionTimeout)?;

        // 3. Atomare LSM-Schreibung: Primär + Sekundärindex für JEDEN Teilnehmer.
        let id = HyperEdgeId(self.next_hyperedge_id());
        let hyperedge = HyperEdge {
            id, predicate, participants: participants.to_vec(), weight: 1.0,
            tx_valid_from: None, tx_valid_to: None,
            business_valid_from: None, business_valid_to: None,
            source_doc_id: Some(doc_id),
        };
        self.persist_hyperedge_atomic(&hyperedge)?;

        // 4. RCU-Registrierung (H1).
        self.register_in_rcu_snapshot(&hyperedge)?;

        Ok(id)
    }
}
```

**Loom-Testpflicht (AK-3):** `crates/memfuse-graph/tests/loom_relate_n_ary.rs` mit überlappenden, <!-- doc-ref-ignore -->
unterschiedlich geordneten Mengen.

#### H3 — `SignalKind` ist ein geschlossenes Enum

**Lösung (verbindlich):** Hyperkanten-Treffer fließen als zusätzliche Kandidaten **in `SignalKind::Graph`** ein —
PathRAG liefert bereits ein Graph-Signal; Hyperkanten-Expansion ist ein interner Erweiterungsschritt der Pfadsuche.
`SignalKind` bleibt strukturell unverändert. Diff-Test-Pflicht: `signal_kind_no_new_variant.rs`. <!-- doc-ref-ignore -->

#### H4 — FlatBuffers-Schemaerweiterung erfordert aktives CI-Drift-Gate

**Lösung (verbindlich, harte Vorbedingung):** Das FlatBuffers-CI-Drift-Gate MUSS produktiv und grün sein,
**bevor** das `HyperEdge`-FlatBuffers-Schema gemerged wird — per CI-Job-Abhängigkeit erzwungen (`needs: [flatbuffers-drift-gate]`).

#### H5 — Cascade-Invalidierung: hartes Fan-out-Limit gegen Kostenexplosion

**Lösung (verbindlich, Pflichtbestandteil):**

```rust
pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;

pub struct CascadeReport {
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
    pub deletion_proof: Option<memfuse_crypto::DeletionProof>,
}

pub fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    doc_id: DocId,
    fanout_limit: usize,
) -> Result<CascadeReport, GraphMutationError> {
    let affected = graph.hyperedges_for_doc(doc_id);
    if affected.len() <= fanout_limit {
        // Synchron: alle atomar tombstonieren.
        for hedge_id in &affected {
            graph.tombstone_hyperedge_atomic(*hedge_id)?;
        }
        Ok(CascadeReport { tombstoned_synchronously: affected.len(), queued_for_background: 0, deletion_proof: None })
    } else {
        let (sync_part, async_part) = affected.split_at(fanout_limit);
        for hedge_id in sync_part {
            graph.tombstone_hyperedge_atomic(*hedge_id)?;
        }
        let proof = enqueue_background_cascade(async_part.to_vec());
        Err(GraphMutationError::PartialCascadeQueued(proof))
    }
}
```

Idempotenz-Pflicht: Wiederholter Lauf auf denselben `doc_id` darf keine doppelten Tombstones erzeugen.

#### H6 — Community-Detection/Leiden sieht Hyperkanten nicht

**Lösung (verbindlich für Sichtbarkeit):**

```rust
pub struct CommunityDetectionConfig {
    pub resolution_gamma: f32,
    /// H6: sichtbares Unvollständigkeits-Flag, Default false.
    pub hyperedges_included: bool,
}

pub struct CommunityAssignment {
    pub node_to_community: AHashMap<EntityId, u32>,
    pub hyperedges_included: bool, // 1:1 aus Config, im Report sichtbar
}
```

**Stern-Expansion als Zielarchitektur:** Der Hypergraph wird in einen bipartiten Graphen überführt. Jede
Hyperkante wird als künstlicher Knoten repräsentiert; es entstehen nur binäre Kanten mit $O(|e|)$
Skalierung (statt $O(|e|^2)$ bei Cliquen-Expansion). Zero-Allocation: `StarExpansionIterator` generiert
virtuelle Kanten on-the-fly.

### 6.7 Hyperkanten-Fehler-Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum GraphMutationError {
    #[error("lock acquisition timed out")]
    LockAcquisitionTimeout,
    #[error("cascade fan-out limit exceeded, {0} hyperedges queued for background processing")]
    PartialCascadeQueued(memfuse_crypto::DeletionProof),
    #[error("role binding invalid: {0}")]
    RoleBindingInvalid(String),
    #[error("rcu snapshot reclamation pending, retry")]
    EpochReclamationPending,
    #[error("hyperedge requires >= 2 participants, got {0}")]
    InsufficientParticipants(usize),
}
```

---

<a id="7-retrieval"></a>
## 7. Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen

### 7.1 4-Signal-Fusion

Jede Hybridsuche kombiniert bis zu vier unabhängige Signale — Vektor (HNSW-k-NN), Text (BM25/BM25F), Graph
(PPR-Traversierung inkl. Hyperkanten-Erweiterung), optional Kanten-Reinforcement (feature-gated) — über das
geschlossene `SignalKind`-Enum:

```rust
/// BEWUSST NICHT `#[non_exhaustive]` — Erweiterung erfolgt NIEMALS durch neue Varianten (H3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    Vector,
    Text,
    Graph,
    EdgeReinforcement, // feature-gated
}

impl SignalKind {
    /// Allokationsfrei, `eq_ignore_ascii_case` statt `to_lowercase()`-Allokation.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("vector") { Some(Self::Vector) }
        else if name.eq_ignore_ascii_case("text") { Some(Self::Text) }
        else if name.eq_ignore_ascii_case("graph") { Some(Self::Graph) }
        else if name.eq_ignore_ascii_case("edgereinforcement") { Some(Self::EdgeReinforcement) }
        else { None }
    }
}
```

Die Fusion erfolgt standardmäßig über **Reciprocal Rank Fusion (RRF)**, score-blind und robust bei
Signalausfall. Score-normalisierte Fusion als Opt-in mit hartem RRF-Fallback.

**⚠️ Opus-Optimierung 1.10 — Top-k-Selektion (Stufe 1, gering):**
Volle Sortierung durch begrenzte Selektion in linearer Zeit ersetzen.

### 7.2 Graph-Signal: Forward-Push-PPR (🟢)

**Andersen-Chung-Lang Forward-Push-Algorithmus.** Exploriert nur Knoten, die signifikant zur PageRank-Masse beitragen.

Initialisierung für Seed-Knoten $s$: $r(s) = 1$, $p(s) = 0$. Für jeden Knoten $u$ mit
$\frac{r(u)}{d(u)} > \epsilon$:

1. $p(u) \leftarrow p(u) + \alpha \cdot r(u)$
2. $r(u) \leftarrow (1 - \alpha) \frac{r(u)}{2}$
3. $r(v) \leftarrow r(v) + (1 - \alpha) \frac{r(u)}{2\, d(u)}$ für alle Nachbarn $v$

Laufzeit: $O\!\left(\frac{1}{\alpha \epsilon}\right)$ — **unabhängig von der Gesamtgröße des Graphen** (P24).

```rust
pub struct PprParams {
    pub alpha: f32,           // Teleport-Wahrscheinlichkeit
    pub epsilon: f32,         // Fehlertoleranz-Schwellenwert
    pub hyperedge_decay: f32, // Default 0.85
}

pub fn forward_push_ppr(
    graph: &CsrGraph,
    seeds: &[EntityId],
    params: &PprParams,
) -> AHashMap<EntityId, f32> {
    let mut p: AHashMap<EntityId, f32> = AHashMap::new();
    let mut r: AHashMap<EntityId, f32> = seeds.iter()
        .map(|&s| (s, 1.0 / seeds.len() as f32)).collect();
    let mut queue: std::collections::VecDeque<EntityId> = seeds.iter().copied().collect();

    while let Some(u) = queue.pop_front() {
        let degree = graph.degree(u).max(1) as f32;
        let r_u = *r.get(&u).unwrap_or(&0.0);
        if r_u / degree <= params.epsilon { continue; }

        *p.entry(u).or_insert(0.0) += params.alpha * r_u;
        let residual_kept = (1.0 - params.alpha) * r_u / 2.0;
        r.insert(u, residual_kept);

        let push_share = (1.0 - params.alpha) * r_u / (2.0 * degree);

        // Binäre Nachbarn (unveränderter Hotpath).
        for (v, _w) in graph.neighbors_with_weights(u) {
            *r.entry(v).or_insert(0.0) += push_share;
            queue.push_back(v);
        }

        // 🔴 NEU (H3): Hyperkanten-Partner als virtuelle Nachbarn, Gewichtsabschlag.
        for hedge_id in graph.hyperedges_for_entity(u) {
            for role_binding in graph.hyperedge_participants(hedge_id) {
                if role_binding.entity == u { continue; }
                *r.entry(role_binding.entity).or_insert(0.0) += push_share * params.hyperedge_decay;
                queue.push_back(role_binding.entity);
            }
        }
    }
    p
}
```

### 7.3 Volltextsuche: BM25 mit Block-Max WAND und BM25F (🟢)

```rust
pub struct ResidentPostingIndex {
    postings: AHashMap<TermId, PostingList>,
    doc_lengths: Vec<u32>,
    field_lengths: AHashMap<(DocId, FieldId), u32>, // BM25F-Voraussetzung
}

pub struct Bm25fParams {
    pub k1: f32,
    pub b: f32,
    pub field_weights: AHashMap<FieldId, f32>,
}

impl ResidentPostingIndex {
    /// Block-Max WAND: Top-k ohne vollständige Postinglisten-Traversierung.
    pub fn search_topk(&self, query_terms: &[TermId], k: usize, params: &Bm25fParams) -> Vec<(DocId, f32)>;

    /// BM25F-Score für ein einzelnes Dokument, feldgewichtet.
    fn bm25f_score(&self, doc: DocId, terms: &[TermId], params: &Bm25fParams) -> f32;
}

/// Deutsche Kompositazerlegung.
pub fn decompose_german_compound(word: &str, dictionary: &CompoundDictionary) -> Vec<String>;
```

Persistenz: Residenter Index wird beim Start aus LSM-Präfix `__text:posting:` materialisiert.

**⚠️ Opus-Optimierung 1.9 — Text-Posting-Format (Stufe 1, hoch):**
Umstellung von Einzelschlüssel- auf Listenspeicherung. Delta-kodierte Dokument-IDs. Ein Lesezugriff
pro Suchbegriff statt unbegrenztem Präfix-Scan.

### 7.4 Vektorindex: HNSW + DiskANN (🟢)

```rust
pub struct HnswIndex<const D: usize> {
    layers: Vec<HnswLayer<D>>,
    entry_point: AtomicUsize,
    sq8_codebook: Sq8Codebook,
}

pub struct Sq8Codebook {
    pub min: [f32; D_MAX],
    pub max: [f32; D_MAX],
    pub clip_percentile: f32, // Default 0.999
}

impl<const D: usize> HnswIndex<D> {
    pub fn search_knn(&self, query: &[f32; D], k: usize, ef_search: usize) -> Vec<(DocId, f32)>;
    pub fn insert(&mut self, id: DocId, vector: [f32; D]) -> Result<(), IndexError>;
    pub fn delete(&mut self, id: DocId) -> Result<(), IndexError>; // native Tombstone
}
```

**NaN-sichere Distanz-Pipeline:**
```rust
/// Bitweise SIMD-Maskierung statt Branch: NaN → f32::INFINITY.
#[inline]
fn masked_l2_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    // SAFETY: `a`/`b` sind 64-Byte-aligned und exakt D Elemente lang.
    unsafe { unimplemented!() }
}
```

**DiskANN (🟢 offizieller Tier):**
```rust
pub struct DiskAnnIndex<const D: usize> {
    mmap: memmap2::Mmap,
    tombstones: scc::HashSet<DocId>, // native, kein HNSW-Fallback nötig
    tombstone_wal: TombstoneWal,
}

pub enum VectorIndexTier {
    Hnsw,
    #[cfg(feature = "experimental-diskann")]
    DiskAnn,
}
```

**Zielarchitektur „HNSW v2" (🔴 Arena-Allocator):**
```rust
pub struct HnswArena<const D: usize> {
    storage: std::sync::Arc<MmapArena>,
    head: crossbeam_epoch::Atomic<NodeRecord<D>>,
    capacity: usize,
}

pub struct NodeRecord<const D: usize> {
    pub vector: [f32; D],
    pub neighbor_offsets: [u32; MAX_M],   // Offsets statt Pointer
    pub neighbor_count: u16,
}

impl<const D: usize> HnswArena<D> {
    /// Relinking über CAS statt Mutex.
    pub fn relink(&self, node_offset: u32, new_neighbors: &[u32]) -> Result<(), IndexError>;
}
```

**⚠️ Opus-Optimierungen für den HNSW-Hot-Path:**

| ID | Maßnahme | Aufwand |
|---|---|---|
| 1.1 | Nachbarlisten-Auflösung ohne Allokation — Referenz statt Kopie; mittelfristig fester Stride | Gering → Hoch |
| 1.2 | Backlink-Auflösung von O(P×B) auf O(1) — HashMap pro Suche | Gering |
| 1.3 | Lock auf Quantisierer einmalig pro Suchaufruf, Distanz direkt auf Mmap-Slice | Mittel |

### 7.5 Community-Detection: Leiden (🟢 binärer Pfad)

Stern-Expansion für Hyperkanten-Projektion (🔴). `StarExpansionIterator` gaukelt Leiden-Solver bipartite
Inzidenzmatrix vor (Zero-Allocation):

```rust
pub struct StarExpansionIterator<'a> {
    graph: &'a CsrGraph,
    current_hyperedge_idx: usize,
}

impl<'a> Iterator for StarExpansionIterator<'a> {
    type Item = (EntityId, VirtualHyperedgeNode);
    fn next(&mut self) -> Option<Self::Item> { unimplemented!() }
}
```

### 7.6 Provenance-Tracking

**⚠️ Opus-Optimierung 1.8 — `build_provenance` Struct (Stufe 1, gering-mittel):**
Von 14 positionellen Parametern auf benannte Struct:

```rust
#[derive(Default)]
pub struct ProvenanceBuilder {
    source_doc_id: Option<DocId>,
    signal_contributions: Vec<(SignalKind, f32)>,
    fusion_mode: Option<FusionMode>,
    calibrated_threshold: Option<f32>,
}

impl ProvenanceBuilder {
    pub fn source_doc_id(mut self, id: DocId) -> Self { self.source_doc_id = Some(id); self }
    pub fn add_signal(mut self, kind: SignalKind, score: f32) -> Self {
        self.signal_contributions.push((kind, score)); self
    }
    pub fn build(self) -> Result<ProvenanceRecord, DbError> { unimplemented!() }
}
```

---

<a id="8-bandit"></a>
## 8. Contextual-Bandit-Routing

MemFuse integriert einen Multi-Armed-Bandit-Router (LinUCB, Li et al. 2010) zur adaptiven Aussteuerung der
Retrieval-Strategien.

### 8.1 Gemeinsame Schnittstelle

```rust
pub trait BanditPolicy: Send + Sync {
    fn select_arm(&self, context: &[f32]) -> RetrievalStrategy;
    fn update(&mut self, context: &[f32], arm: RetrievalStrategy, reward: f32);
}

pub enum RetrievalStrategy { Vector, Text, Graph, Hybrid }
```

### 8.2 Zwei Implementierungsvarianten

**`DiagonalApproximation` (🟢 Produktions-Default):**

```rust
pub struct DiagonalApproximationBandit {
    theta: Vec<f32>,
    sigma_sq: Vec<f32>,
    drift: LyapunovDriftWatcher,
}

impl BanditPolicy for DiagonalApproximationBandit {
    fn update(&mut self, context: &[f32], _arm: RetrievalStrategy, reward: f32) {
        for (i, &xi) in context.iter().enumerate() {
            self.theta[i] += reward * xi / self.sigma_sq[i].max(1e-8);
            self.sigma_sq[i] += xi * xi;
        }
    }
    fn select_arm(&self, context: &[f32]) -> RetrievalStrategy { unimplemented!() }
}
```

Dies ist strukturell ein SGD-artiges Verfahren — **keine** exakte Ridge-Regression im Sinne von $\theta = A^{-1}b$.

**`ShermanMorrisonBandit` (🟡 Opt-in, `egress-sherman-morrison`):**

Mathematisch korrekte inkrementelle Matrixinversion:

$$(A + xx^\top)^{-1} = A^{-1} - \frac{A^{-1}xx^\top A^{-1}}{1 + x^\top A^{-1} x}$$

wobei $A = \sum x_t x_t^\top + \lambda I$ und $b = \sum r_t x_t$, $\theta = A^{-1}b$.

```rust
#[repr(C, align(64))]
pub struct AlignedVector<const D: usize> { pub data: [f32; D] }

pub struct ShermanMorrisonBandit<const D: usize> {
    pub inv_a: AlignedVector<{ D * D }>, // A^{-1}, flach, row-major
    pub b: AlignedVector<D>,
    pub theta: AlignedVector<D>,
    pub lambda: f32,                     // Ridge-Regularisierung
}

impl<const D: usize> ShermanMorrisonBandit<D> {
    /// O(d²) Lock-free Update via AVX-512/NEON SIMD.
    pub fn update_rank_1(&mut self, x: &AlignedVector<D>, reward: f32) -> Result<(), BanditError> {
        let v = self.matvec_inv_a(x);
        let s = 1.0 + Self::dot(x, &v);
        if s.abs() < 1e-8 { return Err(BanditError::SingularUpdate); }
        self.rank1_update_inv_a(&v, s);
        self.b.data.iter_mut().zip(x.data.iter()).for_each(|(bi, xi)| *bi += reward * xi);
        self.theta = self.matvec_inv_a(&self.b);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BanditError {
    #[error("rank-1 update denominator near zero")]
    SingularUpdate,
}
```

64-Byte-Cache-Line-Alignment verhindert False Sharing. Jeder `unsafe`-Block für SIMD-Intrinsics erfordert
`// SAFETY:`-Kommentar.

**⚠️ Opus-Optimierung 0.2 — Dimensionsprüfung (Stufe 0, gering):**
`score()`/`update()` von `debug_assert` auf harte `Result`-Fehlerbehandlung mit `DimensionMismatch { expected, actual }`.
Dimensions-Versionierung in `BanditProfileState`.

**⚠️ Opus-Optimierung 0.3 — Drift-Bandit-Kopplung (Stufe 0, gering):**
Drift-Reaktionsmethode bei `DriftDetected` tatsächlich aufrufen statt nur loggen. Mit konfigurierbarem `k_drift`.

### 8.3 Gedeckelter Lyapunov-Drift-Regelkreis (🟢)

```rust
pub struct LyapunovDriftWatcher {
    pub drift_decay_window: u32,   // Default 50
    pub drift_gamma: f32,          // Default 0.95
    steps_remaining: std::sync::atomic::AtomicU32,
    integrator_state: std::sync::atomic::AtomicU32, // f32-Bits
}

impl LyapunovDriftWatcher {
    /// Anti-Windup: bei PID-Sättigung stoppt der Integrator sofort.
    pub fn update(&self, error: f32, dt_seconds: f32, saturated: bool) -> f32 {
        if saturated { return self.current_alpha(); }
        // Zeitfensterbasierte, gedeckelte Eskalation.
        unimplemented!()
    }
}
```

### 8.4 Default-Umstellung

Der Wechsel des Produktions-Defaults zu `ShermanMorrison` ist an ein CI-Latenzbudget-Gate gebunden:
Kriterium < 5 % der medianen LLM/SLM-Inferenzlatenz. `DiagonalApproximation` bleibt als Low-Memory-Opt-out.

---

<a id="9-inferenz"></a>
## 9. Inferenz, KV-Cache-Bridge und Zero-Copy-IPC

### 9.1 Zero-Copy-Eviction-Bridge (🟢)

Durchgehende Zero-Copy-Datenpipeline auf Basis von `Bytes` und Mmap. FlatBuffers erlaubt direktes Auslesen
aus `&[u8]` ohne Heap-Allokation.

### 9.2 KV-Cache-Bridge mit LSM-Fallback-Spill (🟢)

```rust
pub struct KvCacheBridge {
    ram_cache: scc::HashMap<SessionId, EncryptedKvSegment>,
    lsm_fallback: memfuse_store::LsmStore,
    cipher_worker: CipherWorkerHandle,
}

pub struct EncryptedKvSegment {
    pub ciphertext: Vec<u8>,
    pub rope_offset: u32,
    pub model_fingerprint: [u8; 32],
}

impl KvCacheBridge {
    pub fn get(&self, session: SessionId, model_fingerprint: [u8; 32])
        -> Result<Option<KvCache>, KvBridgeError>
    {
        if let Some(seg) = self.ram_cache.get(&session) {
            if seg.model_fingerprint != model_fingerprint {
                return Err(KvBridgeError::FingerprintMismatch);
            }
            return Ok(Some(self.decrypt(seg)));
        }
        self.load_from_lsm_fallback(session)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KvBridgeError {
    #[error("rope offset mismatch")]
    RopeOffsetMismatch,
    #[error("model fingerprint mismatch")]
    FingerprintMismatch,
}
```

**Testpflicht:** Alle vier Kombinationen: RAM-Hit, RAM-Miss/LSM-Hit, beide-Miss, Fingerprint-Mismatch.

### 9.3 AES-Schlüsselplan-Wiederverwendung

```rust
use aes_gcm_siv::Aes256GcmSiv;
use std::sync::OnceLock;

static CIPHER_INSTANCE: OnceLock<Aes256GcmSiv> = OnceLock::new();

pub fn cipher() -> &'static Aes256GcmSiv { unimplemented!() }
```

**⚠️ Opus-Optimierung 1.5 — AES-Schlüsselplan (Stufe 1, mittel):**
Cipher-Instanz einmalig pro Schlüssel aufbauen und wiederverwenden. Schlüsselrotation über gezielten Austausch.

### 9.4 Layer-3-Crates

- **`memfuse-ollama`:** `OllamaClient::generate(prompt, contextual_prefix) -> Result<String, OllamaError>`.
  Contextual-Chunk-Prefixing fügt Retrieval-Kontext als System-Präfix ein.
- **`memfuse-embed`:** `EmbeddingModel::embed(texts) -> Result<Vec<Vec<f32>>, EmbedError>` (ONNX, feature-gated),
  `CrossEncoderReranker::rerank(query, candidates) -> Vec<SearchResult>`.
- **`memfuse-agent`:** `AgentWorkflow`-Engine mit persistentem Zustand über `Checkpointable`.
- **`memfuse-py`:** PyO3-Bindings in eigenem Cargo-Workspace (Panic-Strategie-Isolation: `panic = "unwind"` nur
  hier, restlicher Workspace `panic = "abort"` für Release-Profile).

**⚠️ Opus-Optimierung 2.5 — `memfuse-py` in Root-Workspace (Stufe 2, gering):**
Crate in die `members`-Liste des Root-`Cargo.toml` aufnehmen, damit die FFI-Grenze denselben CI-Prüfungen
unterliegt. Abweichende Profileinstellungen per paketspezifischem Profil erhalten.

---

<a id="10-sicherheit"></a>
## 10. Sicherheits- und Datenschutzmodell

### 10.1 Kryptographische Grundlagen

AES-256-GCM-SIV für Daten at rest, WAL mit race-freier HMAC-Kette, `DeletionProof` für DSGVO-Art.-17-Nachweise.

```rust
pub struct DeletionProof {
    pub key_hash: [u8; 32],
    pub hmac_chain_entry: [u8; 32],
    pub prev_hmac: [u8; 32],
    pub timestamp: i64,
}

pub struct HmacChain {
    last: std::sync::Mutex<[u8; 32]>,
}

impl HmacChain {
    /// MUSS atomar gegenüber gleichzeitigen `append`-Aufrufen sein.
    pub fn append(&self, payload: &[u8]) -> Result<[u8; 32], CryptoError>;
    pub fn verify_chain(entries: &[[u8; 32]]) -> Result<(), CryptoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("hmac chain fork at index {0}")]
    ChainFork(usize),
    #[error("aead operation failed")]
    AeadFailure,
}
```

### 10.2 WASM-Sandbox (🟢)

```rust
pub struct WasmCapabilities {
    pub max_fuel: Option<u64>,
    pub max_wall_clock_ms: u64, // Default 5000; 0 = unbegrenzt
    pub allow_cloud_egress: bool, // Default false
}

pub struct SandboxExecutor {
    engine: wasmtime::Engine,
}

impl SandboxExecutor {
    pub fn execute(&self, module: &[u8], caps: &WasmCapabilities) -> Result<Vec<u8>, SandboxError> {
        let mut store = wasmtime::Store::new(&self.engine, ());
        if let Some(fuel) = caps.max_fuel { store.set_fuel(fuel).map_err(SandboxError::from)?; }
        // Wall-Clock unabhängig von Fuel via separatem Timeout-Task (P23).
        unimplemented!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("fuel budget exhausted")]
    FuelExhausted,
    #[error("wall clock budget of {0}ms exceeded")]
    WallClockExceeded(u64),
}
```

`fd_write`-WASI-Stub MUSS `iovs` korrekt parsen.

### 10.3 Prompt-Injection-Schutz

Eingaben aus dem Agenten-Kontext werden nicht ungeprüft als Steuerbefehle interpretiert; der Lesepfad ist
durchgehend Zero-Copy, um unnötige Pufferkopien sensibler Daten zu vermeiden.

### 10.4 Cloud-Egress Privacy Gateway (🟢, 5-Schichten-Architektur)

1. **Token-Vaulting/Pattern-Matching (`EgressVault`):** `RegexSet`-Klassifikation, Payload-Deckel.
2. **Vorabstraktion.**
3. **Graph-Generalisierung.**
4. **Bulk-Exfiltration-Detektor:** Großvolumige Anfragemuster erkennen.
5. **Re-Hydration:** `CloudResponseRehydrator::rehydrate` — Round-Trip-sicher, Multibyte-UTF-8-panic-sicher.

```rust
pub struct EgressVault {
    pattern_matcher: regex::RegexSet,
    surrogate_map: scc::HashMap<SurrogateToken, OriginalEntity>,
}

impl EgressVault {
    pub fn generate_surrogate(&self, entity: &OriginalEntity, session: SessionId) -> SurrogateToken;
    pub fn get_entity(&self, token: &SurrogateToken) -> Option<OriginalEntity>;
}

pub struct BulkExfiltrationDetector {
    pub max_bytes_per_window: usize,
    pub window: std::time::Duration,
}

pub struct CloudResponseRehydrator;
impl CloudResponseRehydrator {
    pub fn rehydrate(&self, response: &str, vault: &EgressVault) -> String { unimplemented!() }
}
```

**⚠️ Opus-Optimierung 0.4 — Egress-Klassifizierung (Stufe 0, gering):**
Eigene, restriktivere Policy-Kategorie für Cloud-Egress-Methoden statt gleiche wie lokale Lesezugriffe.

---

<a id="11-betrieb"></a>
## 11. Betriebsmodi

MemFuse wird ausschließlich eingebettet betrieben: im Prozess des aufrufenden Agenten (Rust- oder Python-Bindung)
oder als lokaler MCP-Server über stdio-JSON-RPC. Es gibt keinen Server-Modus mit Netzwerk-Listener für
Multi-Tenant-Zugriff. Der Cloud-Egress-Pfad (§10.4) ist der einzige Punkt, an dem Daten das lokale System
verlassen — ausschließlich auf explizite Anforderung, nie als Hintergrundtelemetrie.

---

<a id="12-schema"></a>
## 12. FlatBuffers-Schema (vollständig, `schemas/memfuse.fbs`)

```fbs
namespace memfuse.ipc;

table RoleBindingFb {
  role: uint32;
  entity: uint64;
}

table HyperEdgeFb {
  id: uint64;
  predicate_tag: uint32;          // EdgeType-Diskriminante
  participants: [RoleBindingFb];  // min. 2, validiert applikationsseitig
  weight: float32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;            // 0 = None (Sentinel, dokumentiert)
  business_valid_from: int64;
  business_valid_to: int64;       // i64::MIN = None (Sentinel)
  source_doc_id: uint64;          // oder uint128-Encoding bei docid-128
}

table EdgeFb {
  target: uint64;
  weight: float32;
  edge_type_tag: uint32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;
  business_valid_from: int64;
  business_valid_to: int64;
  source_doc_id: uint64;
}

root_type HyperEdgeFb;
```

**CI-Drift-Gate (`xtask check-flatbuffers-drift`):** Vergleicht Hash des generierten Codes gegen committeten
Referenz-Hash. Jede Schema-Änderung ohne begleitende Regenerierung schlägt den Merge-Gate-Job fehl. **Dieses
Gate MUSS grün sein, bevor `HyperEdgeFb` gemerged wird (H4).**

---

<a id="13-fehler"></a>
## 13. Fehlertaxonomie (crateübergreifend)

| Crate | Fehler-Enum | Einbettet |
|---|---|---|
| `memfuse-core` | `CoreError` | — |
| `memfuse-store` | `StoreError` | `WalError`, `LockError`, `CoreError` |
| `memfuse-crypto` | `CryptoError` | — |
| `memfuse-index` | `IndexError` | `CoreError` |
| `memfuse-graph` | `GraphMutationError`, `GraphError` | `LockError` |
| `memfuse-router` | `BanditError` | — |
| `memfuse-candle` | `KvBridgeError` | `CryptoError` |
| `memfuse-mcp` | `SandboxError`, `EgressError` | `wasmtime::Error` |
| `memfuse-db` | `DbError` | alle Layer-1-Fehler per `#[from]` |

**Regel (verbindlich):** Kein öffentlicher Funktionsrückgabetyp ist `Box<dyn std::error::Error>`. Jeder Crate
exportiert genau einen (oder wenige, klar abgegrenzte) `thiserror`-Fehlertyp(en); `memfuse-db` als oberste
Konsumentenschicht bündelt alle Unterfehler verlustfrei per `#[from]`/`#[error(transparent)]`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)] Store(#[from] memfuse_store::StoreError),
    #[error(transparent)] Graph(#[from] memfuse_graph::GraphMutationError),
    #[error(transparent)] Index(#[from] memfuse_index::IndexError),
    #[error("provenance builder missing required field: {0}")]
    ProvenanceIncomplete(&'static str),
}
```

---

<a id="14-features"></a>
## 14. Feature-Flag-Politik: Produktions-Default vs. Opt-in

Ein Breaking-Change- oder Performance-Trade-off-Feature wird hinter einem Cargo-Feature isoliert, bis eine
explizite Produktentscheidung den Wechsel des Defaults auslöst. Dies ist **kein Mangel**, sondern verbindliche
Politik.

| Feature-Flag | Reifegrad | Beschreibung |
|---|---|---|
| `cloud-egress-guard` | 🟢 | DLP/Egress-Kontrolle, Surrogat-Tokenisierung, Bulk-Exfiltration-Detektor |
| `bandit-routing` | 🟢 | LinUCB-Grundfunktion, Lyapunov-Kopplung, gedeckelte Drift-Eskalation |
| `egress-sherman-morrison` | 🟡 | Mathematisch korrekte Ridge-Regression; einziger Pfad mit LinUCB-Regret-Garantie |
| `kv-bridge` | 🟢 | KV-Cache-Bridge inkl. LSM-Fallback, AES-256-GCM-SIV |
| `wasm-sandbox` | 🟢 | Fuel- und Wall-Clock-Budget orthogonal |
| `experimental-diskann` | 🟢 (Tier) | Native Tombstones, SQ8-Perzentil-Clipping |
| `docid-128` | 🟡 | 128-Bit-BLAKE3-DocId, Rollout vollzogen, Default bleibt `u64` |
| `block-cache-v2` | 🟡 | SIEVE-Backend, Default bleibt LRU |
| `bm25f` | 🟢 | Feldgewichtete BM25-Bewertung |
| `flatbuffers-drift-gate` (xtask) | 🟢 | CI-Gate gegen Schema-Drift |
| `fault-injection` | 🟢 | Test-only |
| `loom` | Dev | Nebenläufigkeits-Modelltests |
| `adaptive-decay` / `-control` | 🟢 | Kalibrierungs-Feintuning |
| `partial-index-rebuild` | 🟢 | Inkrementeller Indexaufbau |
| `edge-reinforcement-learning` | 🟢 (Gate) | Kantenverstärkung als optionales Fusionsverhalten |
| **Hyperkanten (`relate_n_ary`, `HyperEdge`)** | **🔴** | Nicht implementiert — kein Flag, siehe §6 |

---

<a id="15-tests"></a>
## 15. Test- und CI-Spezifikation

### 15.1 Unit-Tests

Jede öffentliche Funktion mit nicht-trivialer Logik erhält mindestens:
- einen Normalfall-Test,
- einen Grenzfall-Test (leere Eingabe, einzelnes Element),
- einen Fehlerfall-Test, der das korrekte `Result::Err`-Enum-Mitglied prüft (kein pauschales `is_err()`).

### 15.2 Integrationstests

| Testpfad | Zweck / AK |
|---|---|
| `crates/memfuse-graph/tests/hyperedge_persistence_survives_restart.rs` | AK-1 | <!-- doc-ref-ignore -->
| `crates/memfuse-graph/tests/hyperedge_compact_race.rs` | AK-1 (nebenläufig zu `compact()`) | <!-- doc-ref-ignore -->
| `crates/memfuse-graph/tests/hyperedge_memory_budget.rs` | AK-2 | <!-- doc-ref-ignore -->
| `crates/memfuse-db/tests/signal_kind_no_new_variant.rs` | AK-4 | <!-- doc-ref-ignore -->
| `crates/memfuse-graph/tests/hyperedge_cascade_fanout.rs` | AK-6 (High-Fan-out) | <!-- doc-ref-ignore -->
| `crates/memfuse-graph/tests/community_hyperedges_included_flag.rs` | AK-7 | <!-- doc-ref-ignore -->
| `crates/memfuse-graph/benches/binary_edge_regression.rs` | AK-8 (Baseline-Vergleich) | <!-- doc-ref-ignore -->

### 15.3 Loom-Tests (`#[cfg(loom)]`, `loom`-Feature)

| Testpfad | Zweck |
|---|---|
| `crates/memfuse-store/tests/loom_group_commit.rs` | Group-Commit-Atomizität |
| `crates/memfuse-store/tests/loom_multi_key_lock.rs` | Multi-Key-Deadlockfreiheit | <!-- doc-ref-ignore -->
| `crates/memfuse-graph/tests/loom_relate_n_ary.rs` | AK-3, Hyperkanten-Deadlockfreiheit | <!-- doc-ref-ignore -->

Alle drei MÜSSEN als eigener CI-Job sichtbar grün laufen.

### 15.4 `.github/workflows/merge-gate.yml` — Pflicht-Jobs

```yaml
jobs:
  unit-tests:
    run: cargo test --workspace
  flatbuffers-drift-gate:
    run: cargo run -p xtask -- check-flatbuffers-drift
  hyperedge-schema-merge:
    needs: [flatbuffers-drift-gate]
    run: cargo test -p memfuse-graph --features hyperedges -- hyperedge
  check-bandit-latency-budget:
    run: cargo run -p xtask --features memfuse-router/egress-sherman-morrison -- check-bandit-latency-budget
  loom-tests:
    run: RUSTFLAGS="--cfg loom" cargo test --workspace --features loom -- --test-threads=1
  check-unwrap-baseline:
    run: cargo run -p xtask -- check-unwrap-baseline
```

**⚠️ Opus-Optimierung 3.1 — Feature-Kombinationen in CI (Stufe 3, gering):**
Powerset-Build der relevanten Feature-Flags ergänzen.

**⚠️ Opus-Optimierung 3.2 — Panic-Inventar (Stufe 3, gering):**
Gate nur für `src/` (ohne Tests/Benchmarks), harte sinkende Obergrenze.

---

<a id="16-abnahme"></a>
## 16. Vollständige Abnahmekriterien

### 16.1 Kernsystem-Abnahmekriterien

| Nr. | Kriterium | Reifegrad |
|---|---|---|
| K-1 | `insert_lock` vollständig durch key-granulare `kv_locks` ersetzt | 🟢 |
| K-2 | HNSW-Hotpath nutzt unaligned-SIMD-Distanzkernel statt Heap-Allokation | 🟢 |
| K-3 | BM25 nutzt residenten Postinglisten-Index mit Block-Max WAND | 🟢 |
| K-4 | Block-Cache erzwingt bei Lesetreffer im Opt-in-Backend keinen Write-Lock | 🟡 |
| K-5 | DiskANN löscht ohne HNSW-Fallback (native Tombstones) | 🟢 |
| K-6 | Bandit-Latenzbudget CI-gated | 🟢 |
| K-7 | `compact()` blockiert keine nebenläufigen Leser (RCU-Swap) | 🟢 |
| K-8 | PPR proportional zur Seed-Menge, nicht zur Graphgröße (P24) | 🟢 |
| K-9 | `build_provenance` nutzt `ProvenanceBuilder`-Struct | 🟢 |
| K-10 | FlatBuffers-Drift-Gate aktiv | 🟢 |
| K-11 | `SignalKind::from_name` allokationsfrei (`eq_ignore_ascii_case`) | 🟢 |
| K-12 | BM25F produktiv nutzbar | 🟢 |
| K-13 | KV-Cache-Bridge LSM-Fallback mit Testabdeckung für alle Kombinationen | 🟢 |
| K-14 | Drift-Alpha gedeckelt | 🟢 |
| K-15 | Cloud-Egress 5-Schichten produktiv, Rehydration inkl. UTF-8-Sicherheit | 🟢 |
| K-16 | ⚖️ Bandit-Default → ShermanMorrison (sobald Gate besteht) | ⚖️ |
| K-17 | ⚖️ Block-Cache-Default → SIEVE (sobald entschieden) | ⚖️ |
| K-18 | ⚖️ DocId-128 als Produktions-Default (Major-Release) | ⚖️ |
| K-19 | Group-Commit-Loom-Test sichtbar grün in CI | 🔴 |

### 16.2 Hyperkanten-Abnahmekriterien AK-1 bis AK-8 (normativ und abschließend)

| AK | Kriterium | Nachweis (Testpfad) |
|---|---|---|
| AK-1 | `HyperEdge` mit ≥3 `RoleBinding`s persistiert, restart-fest, per `hyperedges_for_entity` auffindbar, konsistent unter gleichzeitigem `compact()` | `hyperedge_persistence_survives_restart.rs`, `hyperedge_compact_race.rs` | <!-- doc-ref-ignore -->
| AK-2 | `estimate_memory_bytes()` inkl. Hyperkanten; `compact_async`-Budget-Check greift | `hyperedge_memory_budget.rs` | <!-- doc-ref-ignore -->
| AK-3 | Zwei gleichzeitige `relate_n_ary` mit überlappenden, unterschiedlich geordneten Mengen deadlockfrei | `loom_relate_n_ary.rs` |
| AK-4 | `SignalKind` strukturell unverändert (kein `Hyperedge`-Signal) | `signal_kind_no_new_variant.rs` | <!-- doc-ref-ignore -->
| AK-5 | FlatBuffers-Drift-Gate grün **vor** `HyperEdgeFb`-Merge | CI-Job-Abhängigkeit `hyperedge-schema-merge: needs: [flatbuffers-drift-gate]` |
| AK-6 | Cascade bricht bei >1.000 Hyperkanten kontrolliert auf Hintergrundverarbeitung um | `hyperedge_cascade_fanout.rs` | <!-- doc-ref-ignore -->
| AK-7 | `hyperedges_included` im Report sichtbar `false` ohne Projektion | `community_hyperedges_included_flag.rs` | <!-- doc-ref-ignore -->
| AK-8 | Keine Regression auf binäre `relate()`/`Edge`-Benchmarks | `binary_edge_regression.rs` | <!-- doc-ref-ignore -->

---

<a id="17-optimierungen"></a>
## 17. Priorisierte Optimierungs-Roadmap (Opus-Analyse)

Die Reihenfolge der Stufen ist **verbindlich**: Stufe 0 blockiert bzw. gefährdet den Betrieb, Stufe 1 ist der
größte Hebel für Latenz/Durchsatz, Stufe 2 betrifft Speicherverbrauch und Struktur, Stufe 3 ist Governance.

### Stufe 0 — Korrektheit und Betriebssicherheit (ZUERST)

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **0.1** | **WAL-Replay-Panic entschärfen** | Dateigröße separat von `mmap.len()`, Direktindizierung → Panic bei veränderter Datei, Crash-Loop möglich | Gering |
| **0.2** | **Bandit-Dimensionsprüfung** | `debug_assert` statt `Result` bei Dimensionsmismatch → stilles Teil-Skalarprodukt nach Modellwechsel | Gering |
| **0.3** | **Drift-Bandit-Kopplung verdrahten** | Drift-Wächter erkennt Verteilungsverschiebung, ruft aber Bandit-Reaktionsmethode nie auf | Gering |
| **0.4** | **Cloud-Egress-Klassifizierung** | Egress-Methode in gleicher Policy-Kategorie wie lokale Lesezugriffe | Gering |
| **0.5** | **Recovery-Pfad differenzieren** | Offene Intents werden pauschal vorwärts committet, egal ob Erfolg oder Abbruch | Mittel |

### Stufe 1 — Hot-Path-Performance (größter Hebel)

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **1.1** | **HNSW: Nachbarlisten ohne Allokation** | Pro Knoten eine Heap-Allokation für Adjazenzliste, größter Einzelfaktor im Suchpfad | Gering → Hoch |
| **1.2** | **HNSW: Backlink O(1)** | Lineare Suche über Einfüge-Operationen bei Batch-Insert, quadratisch bei großen Batches | Gering |
| **1.3** | **HNSW: Distanzpfad Lock/Allokation** | Lock auf Quantisierer pro Kandidat, Mmap-Vektor Element-für-Element dekodiert | Mittel |
| **1.4** | **SSTable: Zero-Copy-Slice** | Vollständige Blockkopie nach CRC-Prüfung, obwohl Puffertyp Slicing unterstützt | Trivial |
| **1.5** | **AES-Schlüsselplan wiederverwenden** | Key-Schedule-Neuaufbau pro Verschlüsselung/Entschlüsselung | Mittel |
| **1.6** | **MemTable: Range-Sharding** | Hash-Sharding zerstört Flush-Sortierung und Präfix-Scans | Hoch |
| **1.7** | **Block-Cache: Byte-basierte Kapazität** | Eintragsbasierte Kapazität bei variabler Blockgröße → unvorhersehbarer Speicher | Mittel |
| **1.8** | **RRF: `build_provenance`-Struct** | 14 positionelle Parameter desselben Typs → stille Vertauschung möglich | Gering-Mittel |
| **1.9** | **Text: Posting-Format umstellen** | Jedes Posting als Einzelschlüssel → massive Schreib-/Leseverstärkung | Hoch |
| **1.10** | **RRF/Text: Top-k-Selektion** | Volle Sortierung statt linearer k-Selektion | Gering |

### Stufe 2 — Speicher und Struktur

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **2.1** | **Graph: Inkrementelle Kompaktierung** | PPR löst vollständigen CSR-Rebuild pro Anfrage aus | Hoch |
| **2.2** | **CSR: Sentinel statt `Option`** | `Vec<Option<T>>` ohne Nischenoptimierung → doppelter Speicher pro Kante | Mittel |
| **2.3** | **Checkpoint: Indizes zusammenführen** | Zwei separate Locks für Sequenz- und Namens-Index → Inkonsistenzfenster | Gering-Mittel |
| **2.4** | **Manifest: Batch-Fsync** | Ein `fsync` pro Einzeleintrag statt pro Zustandsübergang | Mittel |
| **2.5** | **`memfuse-py` in Workspace** | FFI-Grenze nicht von `cargo test --workspace` erfasst | Gering |

### Stufe 3 — Governance und Prozess

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **3.1** | **Feature-Kombinationen in CI** | Opt-in-Features werden nie in Kombination gebaut | Gering (Einrichtung) |
| **3.2** | **Panic-Inventar kontinuierlich** | Gate unterscheidet nicht zwischen Test- und Produktivcode | Gering |

### Kurzübersicht nach Aufwand/Nutzen

| Sofort umsetzbar (gering, hoher Nutzen) | Mittelfristig (mittel) | Struktureller Umbau (hoch) |
|---|---|---|
| 0.1 WAL-Replay-Bounds | 1.3 Distanzpfad-Lock/Allokation | 1.1 HNSW-Nachbarformat |
| 0.2 Bandit-Dimensionsprüfung | 1.5 AES-Schlüsselplan | 1.6 MemTable Range-Sharding |
| 0.3 Drift-Bandit-Kopplung | 1.7 Byte-basierte Cache-Kapazität | 1.9 Text-Posting-Format |
| 0.4 Egress-Kategorisierung | 1.8 build_provenance-Struct | 2.1 Inkrementelle Graph-Kompaktierung |
| 1.2 Backlink-Lookup | 2.2 CSR-Sentinel statt Option | |
| 1.4 SSTable-Zero-Copy-Slice | 2.3 Checkpoint-Index-Merge | |
| 2.5 memfuse-py in Workspace | 2.4 Manifest-Batch-Fsync | |
| 3.1 / 3.2 Governance-Gates | 0.5 Transaktions-Intent-Status | |

---

<a id="18-roadmap"></a>
## 18. Gesamtroadmap

### Stufe 0 — Unmittelbar

1. **Opus-Optimierungen Stufe 0** (§17): WAL-Replay-Panic, Bandit-Dimensionsprüfung, Drift-Kopplung, Egress-Klassifizierung, Intent-Recovery.
2. Loom-Test für Group-Commit sichtbar grün in CI (reine Verifikationslücke).
3. Benchmark-Ausführung des Bandit-Latency-Gates mit produktivem $d$.

### Stufe 1 — Strukturell

4. **Opus-Optimierungen Stufe 1** (§17): HNSW-Hot-Path, SSTable, AES, MemTable, Block-Cache, Provenance, Text-Index, Fusion.
5. **N-äre Hyperkanten** (§6), vollständig spezifiziert, Reihenfolge: H4-Nachweis → Datenmodell (§6.4) → H2 → H1 → H3 → H5 → H6 → Stern-Expansion.
6. HNSW-Dateiformat v2 (Arena + CSR + allokationsfreie Traversierung).

### Stufe 2 — Speicher, Struktur und Produktions-Default-Entscheidungen

7. **Opus-Optimierungen Stufe 2** (§17): Graph-Kompaktierung, CSR-Sentinel, Checkpoint, Manifest, memfuse-py.
8. Bandit-Default `DiagonalApproximation` → `ShermanMorrison` (⚖️ sobald Gate besteht).
9. Block-Cache-Default LRU → SIEVE (⚖️ sobald entschieden).
10. RaBitQ-/PQ-Evaluierung (nach HNSW v2).

### Stufe 3 — Governance und Produktentscheidungen

11. **Opus-Optimierungen Stufe 3** (§17): Feature-Powerset CI, Panic-Inventar.
12. Formale ADR-Revision der Leiden-Umstellung.
13. Major-Release-Planung: DocId-128-Cutover mit DiskANN-Tier-Vollfreigabe bündeln.

### Stufe 4 — Fernziele

14. Memory Consolidation (`consolidate_via_llm()`)
15. CausalEdge
16. Passives WAL-Shipping
17. Vollständige `ProvenanceRecord`-API-Exposition
18. `edge-reinforcement-learning`-Vollspezifikation
19. Automatische NLP-Extraktion n-ärer Fakten aus Freitext

---

<a id="19-matrix"></a>
## 19. Rückverfolgbarkeitsmatrix

| Bereich | Reifegrad | Verweis |
|---|---|---|
| Key-granulare `kv_locks` statt collection-weitem Mutex | 🟢 | §5.1, §5.2a |
| HNSW-SIMD-Hotpath (unaligned Distanzkernel, `AHashSet`-Vorallokation, `try_write()`-Pruning) | 🟢 | §7.4 |
| SQ8-Perzentil-Clipping | 🟢 | §7.4 |
| BM25 residenter Index + Block-Max WAND | 🟢 | §7.3 |
| BM25F feldgewichtete Bewertung | 🟢 | §7.3 |
| Block-Cache: klassisches LRU | 🟢 (Default) | §5.4 |
| Block-Cache: SIEVE/S3-FIFO | 🟡 | §5.4 |
| Sherman-Morrison-Bandit + CI-Latency-Gate | 🟡 (Opt-in) / Gate 🟢 | §8 |
| Leiden statt Label-Propagation (binärer Pfad) | 🟢 | §7.5 |
| RCU-Snapshot-Swap für `CsrGraph::compact()` | 🟢 | §6.3 |
| Native DiskANN-Tombstones | 🟢 | §7.4 |
| DocId-128-Bit-Migration (Rollout) | 🟡 | §6.1 |
| Score-normalisierte Fusion mit RRF-Fallback | 🟡 (Opt-in) | §7.1 |
| Forward-Push-PPR | 🟢 | §7.2 |
| `ProvenanceBuilder`-Struktur | 🟢 | §7.6 |
| FlatBuffers-CI-Drift-Gate | 🟢 | §12 |
| Cloud-Egress Fünf-Schichten (Surrogat, Bulk, Rehydration) | 🟢 | §10.4 |
| KV-Cache-Bridge LSM-Fallback-Spill | 🟢 | §9.2 |
| Bandit-Drift-Alpha-Eskalation gedeckelt | 🟢 | §8.3 |
| Loom-Test sichtbar grün in CI | 🔴 | §15.3 |
| HNSW-Dateiformat v2 (Arena) | 🔴 | §7.4 |
| RaBitQ/PQ-Quantisierung jenseits SQ8 | 🔴 | §7.4 |
| ADR-Formalrevision (Leiden statt LPA) | 🔴 (Dokumentation) | §18 |
| **N-äre Hyperkanten (gesamt: H1–H6, `relate_n_ary`)** | **🔴** | §6 |
| WAL-Replay-Panic-Fix | ⚠️ Opus 0.1 | §5.3, §17 |
| Bandit-Dimensionsprüfung | ⚠️ Opus 0.2 | §8.2, §17 |
| Drift-Bandit-Kopplung verdrahten | ⚠️ Opus 0.3 | §8.2, §17 |
| Egress-Klassifizierung korrigieren | ⚠️ Opus 0.4 | §10.4, §17 |
| Intent-Recovery differenzieren | ⚠️ Opus 0.5 | §5.3, §17 |
| HNSW-Nachbarlisten-Allokation | ⚠️ Opus 1.1 | §7.4, §17 |
| HNSW-Backlink O(1) | ⚠️ Opus 1.2 | §7.4, §17 |
| Distanzpfad Lock/Allokation | ⚠️ Opus 1.3 | §7.4, §17 |
| SSTable Zero-Copy-Slice | ⚠️ Opus 1.4 | §5.5, §17 |
| AES-Schlüsselplan wiederverwenden | ⚠️ Opus 1.5 | §9.3, §17 |
| MemTable Range-Sharding | ⚠️ Opus 1.6 | §5, §17 |
| Block-Cache byte-basiert | ⚠️ Opus 1.7 | §5.4, §17 |
| `build_provenance` Struct | ⚠️ Opus 1.8 | §7.6, §17 |
| Text-Posting-Format | ⚠️ Opus 1.9 | §7.3, §17 |
| Top-k-Selektion | ⚠️ Opus 1.10 | §7.1, §17 |
| Graph inkrementelle Kompaktierung | ⚠️ Opus 2.1 | §6.3, §17 |
| CSR-Sentinel statt Option | ⚠️ Opus 2.2 | §6.3, §17 |
| Checkpoint-Index-Merge | ⚠️ Opus 2.3 | §17 |
| Manifest-Batch-Fsync | ⚠️ Opus 2.4 | §5, §17 |
| `memfuse-py` in Root-Workspace | ⚠️ Opus 2.5 | §9.4, §17 |
| Feature-Powerset CI | ⚠️ Opus 3.1 | §15.4, §17 |
| Panic-Inventar-Gate | ⚠️ Opus 3.2 | §15.4, §17 |

---

*Diese finale konsolidierte Gesamtspezifikation vereinigt Produktvision, Zielarchitektur, normative
Implementierungsvorgaben, algorithmische Spezifikationen, mikrofeingranulare Schnittstellendefinitionen
und die priorisierte Optimierungs-Roadmap des MemFuse Cognitive OS. Sie ist in sich geschlossen und
ersetzt alle vorherigen Einzeldokumente als maßgebliche Quelle. Künftige Änderungen erfolgen als direkte
Überarbeitung dieses Dokuments, nicht als weiteres Delta-Dokument.*
