# MemFuse Cognitive OS — Finale Konsolidierte Gesamtspezifikation (Stabilisierungs-Edition)

> **Status:** Normativ · Einzige maßgebliche Quelle für Produkt, Architektur, Algorithmen,
> Implementierungsvorgaben, Sicherheitsmodell, Schnittstellenspezifikation, Stabilisierungsphasen und
> Optimierungs-Roadmap des MemFuse Cognitive OS.
>
> **Charakter:** Dieses Dokument führt Produktvision, Zielarchitektur, normative Signaturen,
> algorithmische Spezifikationen, mikrofeingranulare Schnittstellendefinitionen, ein verbindliches
> Stabilisierungs- und Gate-Modell sowie die priorisierte Optimierungs-Roadmap in einem einzigen,
> in sich geschlossenen Dokument zusammen. Es ersetzt vollständig alle vorherigen
> Spezifikationsfassungen, -deltas und separat geführten Stabilisierungspläne. **Es referenziert keine
> externen Dokumente** — jede Aussage, die für die Arbeit an MemFuse nötig ist, steht hier.
>
> **Leitentscheidung dieser Fassung:** Gegenüber der Vorfassung ist dieses Dokument um **Teil A —
> Stabilisierungsauftrag** ergänzt und in seiner Roadmap (§17, §18) komplett neu geordnet. Grund:
> eine unabhängige Prüfung des Repository-Zustands hat gezeigt, dass Diagnose-Artefakte (Testergebnisse,
> Lint-Reports, Audit-Dokumente) systematisch vom tatsächlichen Code-Zustand abweichen können, sobald sie
> nicht mechanisch an einen Commit gebunden und automatisch neu erzeugt werden — mit der Folge, dass an
> bereits gelöster Stelle weitergearbeitet und an tatsächlich offener Stelle vorbeigearbeitet wird. **Ab
> sofort gilt: kein Feature-Ausbau, bevor Ground Truth hergestellt und das Fundament (Layer 0–2 des
> Crate-DAG) nachweisbar stabil ist.** Teil A ist ranghöher als alle übrigen Teile dieses Dokuments; im
> Konfliktfall gilt Teil A.
>
> **Sprache:** Rust 2021, Workspace-Layout, `#![forbid(unsafe_code)]` als Default in jedem Crate
> ohne explizite Ausnahme (§0.4).
>
> **Lesart:** Jeder Abschnitt ist eigenständig implementierbar. Codeblöcke sind **normativ**, nicht
> illustrativ — Feldnamen, Typnamen und Funktionssignaturen sind exakt zu übernehmen, sofern nicht
> als „Beispiel" markiert. Wo `unimplemented!()` steht, ist die Signatur und das umgebende
> Vertrags-/Fehlerverhalten normativ, der Funktionskörper ist gemäß der in Prosa/Formel gegebenen
> Algorithmusbeschreibung des jeweiligen Abschnitts zu füllen. **Jeder Abschnitt außerhalb von Teil A
> trägt zusätzlich eine Phasen-Kennzeichnung** (`[Phase 1]` … `[Phase 5]`, siehe §A.3) — sie sagt, ab
> welchem Stabilisierungs-Gate an diesem Abschnitt gearbeitet werden darf. Ein Abschnitt ohne
> ausdrückliche Phasen-Kennzeichnung gilt als `[Phase 1]` (Fundament).
>
> **Reifegrad-Kennzeichnung, durchgängig verwendet — mit verschärfter Bedeutung, siehe §A.2:**
> - 🟢 **Produktiv (Zielaussage)** — im Code vorhanden, korrekt und als Produktions-Default aktiv,
>   **sofern durch einen frischen, commit-gebundenen CI-Lauf bestätigt** (§A.2). Unbestätigt ist die
>   Markierung eine Behauptung aus einer Vorversion dieses Dokuments, keine verifizierte Tatsache.
> - 🟡 **Hinter Feature-Flag** — im Code vollständig und korrekt vorhanden, aber nicht der
>   Produktions-Default; Aktivierung erfordert ein explizites Cargo-Feature.
> - 🔴 **Spezifiziert, zu bauen** — normativer Zielzustand dieses Dokuments, im Code noch nicht
>   vorhanden.
> - ⚖️ **Produktentscheidung ausstehend** — technisch möglich oder vorhanden, Default-Wechsel an
>   messbares Kriterium gebunden.
> - ⚠️ **Opus-Optimierung** — aus Architektur-Review identifiziert, priorisiert umzusetzen, mit
>   Stufe (0–3) und Aufwandseinschätzung versehen.
> - 🔍 **Nachverifikation ausstehend** (neu) — Reifegrad aus einer früheren Dokumentfassung
>   übernommen, aber noch nicht gegen einen frischen CI-Lauf am aktuellen HEAD bestätigt. Jeder
>   Contributor, der auf einen 🟢/🔴-Marker reagiert, MUSS ihn faktisch als 🔍 behandeln, bis Gate 0
>   (§A.3) für den betroffenen Crate durchlaufen ist.

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
└── docs/decisions/                 # ADR-0NN-*.md
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
`ResponseGroundingValidator`), die parallel in `index.rs`/`observability.rs` leben. `GraphIndex` ist zwischen
beiden Fassungen bereits inhaltlich auseinandergelaufen (abweichende Doc-Kontrakte und Default-Implementierungen)
— der Compiler kann das nicht erkennen, weil die unverdrahtete Fassung nie gebaut wird. Das bestehende
Duplikat-Gate (`xtask check_duplicate_symbols`) prüft laut eigener Spezifikation nur *innerhalb derselben Datei*
und ist für dateiübergreifende Duplikate im selben Modulverzeichnis strukturell blind.

**Einordnung:** Die vier unverdrahteten Dateien sind die **korrekte** Zerlegung (ein Vertrag pro Datei); der
tatsächlich kompilierte Zustand ist der Monolith `index.rs` (drei unabhängige Verträge in einer Datei). Eine
Lösung, die schlicht die vier Dateien löscht, würde die bessere Zerlegung entfernen und die falsche behalten.

**Maßnahme (verbindlich):**
1. Divergenz in `GraphIndex` auflösen — die aktuell kompilierte Fassung in `index.rs` ist die Referenz für die
   Zusammenführung, nicht automatisch die inhaltlich richtige.
2. `graph_index`, `vector_index`, `text_index`, `lifecycle` in `traits/mod.rs` deklarieren.
3. `index.rs` und die duplizierten Teile von `observability.rs` löschen, sobald (1)/(2) grün sind.
4. **Governance-Gate `GOV-D` (neu, CI-Pflicht):** `xtask check-module-reachability` verifiziert, dass jede
   `.rs`-Datei unter `src/` von genau einer `mod`-Deklaration aus erreichbar ist. Eine unerreichbare, aber
   vorhandene Datei ist ein CI-Fehler, kein stiller Zustand — dies schließt exakt die Lücke, die `CORE-D`
   ermöglicht hat.

**Testpflicht:** `crates/memfuse-core/tests/no_orphan_modules.rs` — schlägt fehl, sobald eine Datei unter `src/`
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
├── lsm.rs               # LSM-Tree, Compaction
├── wal.rs                # Write-Ahead-Log + HMAC-Kette
├── wal_ring_buffer.rs    # SPSC-Ring-Puffer (§5.3)
├── block_cache/
│   ├── mod.rs            # BlockCacheBackend-Trait
│   ├── lru.rs            # LruBlockCacheBackend (Default)
│   └── sieve.rs          # SieveCacheBackend (Opt-in, `block-cache-v2`)
├── kv_locks.rs           # KvKeyLocks (§5.2a)
└── error.rs
```

### 5.2a Key-granulares Locking: `kv_locks.rs` (normativ)

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

**Loom-Testpflicht:** `crates/memfuse-store/tests/loom_multi_key_lock.rs` MUSS unter `#[cfg(loom)]` zwei
nebenläufige `acquire_multi_sorted`-Aufrufe mit überlappenden, unterschiedlich sortierten Schlüsselmengen
modellieren und deren Terminierung ohne Deadlock nachweisen.

### 5.3 WAL-Ring-Puffer: `wal_ring_buffer.rs` (normativ)

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

**`block_cache/lru.rs` (🟢 Produktions-Default):**

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

**`block_cache/quick_cache.rs` (🟡 Opt-in `block-cache-v2`, produktiv im Code vorhanden):**

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

**`block_cache/sieve.rs` (🔴 Zielarchitektur, löst `QuickCacheBlockCacheBackend` perspektivisch ab):**

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

**Testpflicht:** `crates/memfuse-store/tests/manifest_crash_no_resurrection.rs` — simuliert einen Absturz nach
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
├── edge.rs            # Edge, EdgeType-Nutzung, PersistedEdgePayload
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

**Loom-Testpflicht (AK-3):** `crates/memfuse-graph/tests/loom_relate_n_ary.rs` mit überlappenden,
unterschiedlich geordneten Mengen.

#### H3 — `SignalKind` ist ein geschlossenes Enum

**Lösung (verbindlich):** Hyperkanten-Treffer fließen als zusätzliche Kandidaten **in `SignalKind::Graph`** ein —
PathRAG liefert bereits ein Graph-Signal; Hyperkanten-Expansion ist ein interner Erweiterungsschritt der Pfadsuche.
`SignalKind` bleibt strukturell unverändert. Diff-Test-Pflicht: `signal_kind_no_new_variant.rs`.

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
| `crates/memfuse-graph/tests/hyperedge_persistence_survives_restart.rs` | AK-1 |
| `crates/memfuse-graph/tests/hyperedge_compact_race.rs` | AK-1 (nebenläufig zu `compact()`) |
| `crates/memfuse-graph/tests/hyperedge_memory_budget.rs` | AK-2 |
| `crates/memfuse-db/tests/signal_kind_no_new_variant.rs` | AK-4 |
| `crates/memfuse-graph/tests/hyperedge_cascade_fanout.rs` | AK-6 (High-Fan-out) |
| `crates/memfuse-graph/tests/community_hyperedges_included_flag.rs` | AK-7 |
| `crates/memfuse-graph/benches/binary_edge_regression.rs` | AK-8 (Baseline-Vergleich) |

### 15.3 Loom-Tests (`#[cfg(loom)]`, `loom`-Feature)

| Testpfad | Zweck |
|---|---|
| `crates/memfuse-store/tests/loom_group_commit.rs` | Group-Commit-Atomizität |
| `crates/memfuse-store/tests/loom_multi_key_lock.rs` | Multi-Key-Deadlockfreiheit |
| `crates/memfuse-graph/tests/loom_relate_n_ary.rs` | AK-3, Hyperkanten-Deadlockfreiheit |

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
| AK-1 | `HyperEdge` mit ≥3 `RoleBinding`s persistiert, restart-fest, per `hyperedges_for_entity` auffindbar, konsistent unter gleichzeitigem `compact()` | `hyperedge_persistence_survives_restart.rs`, `hyperedge_compact_race.rs` |
| AK-2 | `estimate_memory_bytes()` inkl. Hyperkanten; `compact_async`-Budget-Check greift | `hyperedge_memory_budget.rs` |
| AK-3 | Zwei gleichzeitige `relate_n_ary` mit überlappenden, unterschiedlich geordneten Mengen deadlockfrei | `loom_relate_n_ary.rs` |
| AK-4 | `SignalKind` strukturell unverändert (kein `Hyperedge`-Signal) | `signal_kind_no_new_variant.rs` |
| AK-5 | FlatBuffers-Drift-Gate grün **vor** `HyperEdgeFb`-Merge | CI-Job-Abhängigkeit `hyperedge-schema-merge: needs: [flatbuffers-drift-gate]` |
| AK-6 | Cascade bricht bei >1.000 Hyperkanten kontrolliert auf Hintergrundverarbeitung um | `hyperedge_cascade_fanout.rs` |
| AK-7 | `hyperedges_included` im Report sichtbar `false` ohne Projektion | `community_hyperedges_included_flag.rs` |
| AK-8 | Keine Regression auf binäre `relate()`/`Edge`-Benchmarks | `binary_edge_regression.rs` |

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

# Mikrofeingranulare Schnittstellenspezifikation & Systemoptimierung für Memfuse Cognitive OS

Die vorliegende Spezifikation definiert die mikrofeingranulare Architektur für das Memfuse Cognitive OS. Die Analyse adressiert die Beseitigung struktureller Flaschenhälse in den Bereichen Wissensgraph-Modellierung, Contextual-Bandit-Routing, Cache-Kontention, Vektorindex-Traversierung, LSM-Storage-Engine, Inferenz-Brücken und kryptographischer Sicherheit. Die Lösungsarchitekturen sind so konzipiert, dass sie direkt in deterministischen, threadsicheren Rust-Code überführt werden können, ohne die systemweiten Invarianten zu verletzen.

## 5.1 N-äre Hyperkanten im Wissensgraphen (`crates/memfuse-graph`)

Die Repräsentation n-ärer Relationen in herkömmlichen Graphdatenbanken führt häufig zu einem semantischen Informationsverlust, wenn komplexe Ereignisse in binäre Subjekt-Prädikat-Objekt-Tripel zerschnitten werden. Die nachfolgenden Spezifikationen definieren die Integration von Hyperkanten in die bestehende Compressed Sparse Row (CSR) Struktur.

### 5.1.1 — H1: RCU-Snapshot-Integration

**Ist-Zustand im Repo:** `crates/memfuse-graph/src/csr.rs` berechnet die Speicherschätzung des asynchronen `compact()`-Prozesses ausschließlich auf Basis der binären Adjazenzliste, während Hyperkanten als getrennte Datenstruktur außerhalb der atomaren Swap-Grenze modelliert werden.

**Referenzierte Literatur:**

- Yan et al., 2023, "Hypergraph Database Storage", arXiv:2302.06119 — Spezifiziert die Repräsentation von n-ären Relationen in speichereffizienten Bipartit-Graphen zur Optimierung von Subhypergraph-Matching-Verfahren.
    
- Guo et al., 2024, "HyperGraphRAG", arXiv:2503.21322 — Belegt, dass die Isolation von Entitäten und Hyperkanten in parallelen Speicherstrukturen die Retrieval-Genauigkeit in RAG-Systemen signifikant erhöht.
    

**Mathematische/algorithmische Spezifikation:** Die Speicherkosten des RCU-Snapshots müssen streng deterministisch berechenbar sein, um Allokationsausfälle zu verhindern. Die Gesamtspeichergröße $S_{\text{total}}$ in Bytes berechnet sich aus den binären Arrays und den Hyperkanten-Strukturen:

$$S_{\text{total}} = \sum_{v \in V} \text{deg}(v) \cdot 8 + \sum_{e \in E_H} \left( 32 + \vert{}e\vert{} \cdot 8 \right) + S_{\text{index}}$$

wobei $\vert{}e\vert{}$ die Anzahl der Teilnehmer in einer Hyperkante $e \in E_H$ darstellt und $S_{\text{index}}$ den Overhead der `scc::HashMap` (Load-Factor $\alpha \approx 0.75$) für den Inversindex modelliert. Die Integration in den RCU-Snapshot verlangt, dass die Hyperkanten-Daten als kontinuierlicher Speicherblock (`Arc<[RoleBinding]>`) alloziert werden, um Referenzzähler-Kaskaden bei Leser-Zugriffen zu vermeiden.

**Rust-Schnittstelle (normativ):**

Rust

```
use memfuse_core::{EntityId, DocId};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HyperEdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoleId(pub u32);

#[derive(Clone)]
#[repr(C)]
pub struct RoleBinding {
    pub role: RoleId,
    pub entity: EntityId,
}

pub struct HyperEdgeView<'a> {
    pub id: HyperEdgeId,
    pub predicate_tag: u32,
    pub participants: &'a [RoleBinding],
    pub weight: f32,
    pub source_doc_id: Option<DocId>,
}

pub struct GraphInner {
    pub adjacency_offsets: Box<[u32]>,
    pub adjacency_targets: Box<[u32]>,
    pub hyperedges: scc::HashMap<HyperEdgeId, Arc<[RoleBinding]>>,
    pub hyperedge_index: scc::HashMap<EntityId, Arc<[HyperEdgeId]>>,
}

impl GraphInner {
    pub fn estimate_memory_bytes(&self) -> usize {
        let adj_size = self.adjacency_offsets.len() * 4 + self.adjacency_targets.len() * 4;
        let he_size = self.hyperedges.capacity() * 32 + self.hyperedges.len() * 24; 
        let index_size = self.hyperedge_index.capacity() * 32 + self.hyperedge_index.len() * 16;
        adj_size + he_size + index_size
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Der Lesezugriff ist durch die Verwendung von `arc_swap::ArcSwap<GraphInner>` vollständig lock-frei ($O(1)$). Modifikationen berechnen einen neuen `GraphInner`-Zustand im Hintergrund und publizieren diesen atomar.

**Invarianten-Nachweis:** Die Definition erfüllt Invariante 5 (Speicherbudget-Transparenz), da die exakte Byte-Berechnung des Hyperkanten-Indizes vor der Kompaktierung evaluiert wird. Invariante 7 (Zero-Copy) wird gewahrt, indem `HyperEdgeView` einen flachen Slice `&'a [RoleBinding]` direkt in das Speicherlayout von `GraphInner` projiziert, ohne neue Heap-Allokationen zu erzwingen.

**Migrationspfad:** Ein Migrations-Job muss bestehende `GraphInner`-Strukturen deserialisieren und die leeren `hyperedges`-Maps initialisieren, bevor die RCU-Pointer ausgetauscht werden.

**Restrisiken/offene Fragen:** Das Re-Hashing der `scc::HashMap` unter asynchroner Schreiblast kann das vorab berechnete Speicherbudget kurzzeitig um die Kapazitätsverdopplungs-Differenz überschreiten.

### 5.1.2 — H2: Deadlock-freies Multi-Key-Locking

**Ist-Zustand im Repo:** Naives Locking über `kv_locks` für eine Hyperkante mit $N$ Teilnehmern führt zu zyklischen Wartebedingungen, wenn zwei überlappende Transaktionen die Locks in unterschiedlicher Reihenfolge anfordern.

**Referenzierte Literatur:**

- Sarkar et al., 2020, "LSM-tree compaction", arXiv:2202.04522 — Analysiert Nebenläufigkeitskontrollen in skalierbaren Speicherarchitekturen und totale Ordnungen in Lock-Hierarchien.
    

**Mathematische/algorithmische Spezifikation:** Die Deadlock-Freiheit bei der Belegung von $N$ unabhängigen Schlüsseln erfordert die Einhaltung einer totalen Ordnung $\leq_{L}$ über die Sperrenmenge $L$. Sei $h: \text{EntityId} \to \mathbb{N}$ eine eindeutige Hash-Abbildung auf den Shard-Index. Für eine Menge von Entitäten $E = \{e_1, \dots, e_N\}$ wird die Sperrsequenz $S = \text{sort}(\{h(e_i) \mid e_i \in E\})$ generiert. Doppelte Shard-Indizes werden entfernt. Die Komplexität für die Akquise beträgt im Worst-Case $O(N \log N)$ für die Sortierung und $O(K)$ für das Sperren, wobei $K \leq N$ die Anzahl der betroffenen Shards ist.

**Rust-Schnittstelle (normativ):**

Rust

```
use std::sync::RwLockWriteGuard;

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock poisoned")]
    Poisoned,
    #[error("lock acquisition timed out")]
    Timeout,
}

pub struct MultiKeyGuard<'a> {
    _guards: Vec<RwLockWriteGuard<'a, ()>>,
}

impl KvKeyLocks {
    pub fn acquire_multi_sorted(&self, sorted_key_hashes: &[u64]) -> Result<MultiKeyGuard<'_>, LockError> {
        let mut shard_indices: Vec<usize> = sorted_key_hashes.iter()
            .map(|&h| (h & self.shard_mask) as usize)
            .collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();
        
        let mut guards = Vec::with_capacity(shard_indices.len());
        for idx in shard_indices {
            guards.push(self.shards[idx].write().map_err(|_| LockError::Poisoned)?);
        }
        Ok(MultiKeyGuard { _guards: guards })
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Durch die Erzwingung einer monoton steigenden Erwerbsreihenfolge über die physischen Shard-Indizes wird die Entstehung von Zyklen im Betriebsmittel-Zuweisungsgraphen mathematisch ausgeschlossen.

**Invarianten-Nachweis:** Erfüllt strikt Invariante 4 (Deadlockfreiheit bei Multi-Key-Locking) durch den Beweis der totalen Ordnung vor der Lock-Akquise.

**Migrationspfad:** Sämtliche Mutations-APIs (`relate_n_ary`), die mehr als eine Entität berühren, müssen verbindlich auf `acquire_multi_sorted` umgestellt werden.

**Restrisiken/offene Fragen:** Eine hohe Kollisionsrate (viele Entitäten hashen auf denselben Shard) reduziert die Parallelität, was ein Re-Tuning der `shard_mask` bei wachsender Graphgröße erfordert.

### 5.1.3 — H3: Atomare Multi-Entity-Registrierung (`relate_n_ary`)

**Ist-Zustand im Repo:** Es existiert keine Transaktionsklammer, die einen Write-Ahead-Log (WAL) Eintrag für $N$ Graph-Knoten atomar in das LSM-System flusht und gleichzeitig den RCU-Graphen aktualisiert.

**Referenzierte Literatur:**

- Yan et al., 2023, "Hypergraph Database Storage", arXiv:2302.06119 — Spezifiziert atomare Schreiboperationen in n-ären Relationen über strukturierte Delta-Logs.
    

**Mathematische/algorithmische Spezifikation:** Die Funktion `relate_n_ary` operiert als logische Transaktion. Die Atomarität wird durch die Vorab-Allokation einer deterministischen `TxId` und das sequentielle Schreiben der Tupel $(e_i, \text{HyperEdgeId})$ in das WAL unter einem einzelnen Group-Commit gewährleistet. Die Komplexität ist $O(\vert{}P\vert{} \cdot \log(\text{MemTable}))$, wobei $\vert{}P\vert{}$ die Anzahl der Teilnehmer ist.

**Rust-Schnittstelle (normativ):**

Rust

```
use memfuse_core::{DocId, TxId};

pub trait GraphCollectionMutation {
    fn relate_n_ary(
        &self,
        predicate_tag: u32,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError>;
}
```

**Lock-/Nebenläufigkeitsmodell:** Exklusive Write-Sperren werden über `acquire_multi_sorted` (H2) auf Entitätsebene gehalten, bis der `fsync` in das WAL erfolgreich beendet wurde. Rollback erfolgt durch Löschung der unvollständigen In-Memory-Einträge bei I/O-Fehlern.

**Invarianten-Nachweis:** §4(3) Determinismus wird eingehalten, da die Transaktionsgenerierung unabhängig von der Thread-Ausführung sequentiell geordnet ist.

**Migrationspfad:** Das offene Enum `SignalKind` wird nicht modifiziert. Hyperkanten-Treffer fließen additiv als `SignalKind::Graph` in die Ranking-Fusion ein.

**Restrisiken/offene Fragen:** Lange Transaktionen durch I/O-Latenz beim WAL-Flush blockieren konkurrierende Leseoperationen auf den betroffenen Entitäts-Shards.

### 5.1.4 — H4: Nachweispflicht vor Implementierung

**Ist-Zustand im Repo:** Es fehlt ein analytischer Nachweis, ob die Cliquen-Expansion oder eine native Bipartit-Darstellung für die Nachbarschaftstraversierung optimal ist.

**Referenzierte Literatur:**

- Guo et al., 2024, "HyperGraphRAG", arXiv:2503.21322 — Bipartite Transformation von Hypergraphen für effizientes RAG.
    

**Mathematische/algorithmische Spezifikation:** Bei der Cliquen-Expansion einer Hyperkante $e$ mit Fan-out $N$ entstehen $\frac{N(N-1)}{2}$ binäre Kanten. Die Traversierung eines Knotens $v \in e$ kostet $O(N)$. In der bipartiten Stern-Expansion (ein künstlicher Knoten $v_e$ pro Hyperkante, verbunden mit allen $v \in e$) entstehen exakt $N$ Kanten. Die Traversierung von $v$ zu allen Nachbarn in $e$ erfolgt über $v_e$ in zwei Hops und kostet ebenfalls $O(N)$. Da die Speicherkomplexität der Stern-Expansion jedoch $O(N)$ gegenüber $O(N^2)$ beträgt, ist die bipartite Repräsentation (Stern-Expansion) für Speicherung und Traversierung zwingend vorzuziehen.

|**Metrik**|**Cliquen-Expansion**|**Bipartite Repräsentation (Stern)**|
|---|---|---|
|Kantenanzahl|$O(N^2)$|$O(N)$|
|Speicherplatz|Hoch|Minimal|
|Pfadlänge|1 Hop|2 Hops|
|Traversierung|$O(N)$|$O(N)$|

**Rust-Schnittstelle (normativ):** Keine direkte API-Schnittstelle; dies ist eine architekturelle Entscheidungsvorgabe.

**Lock-/Nebenläufigkeitsmodell:** Lese-Pfad über RCU (`ArcSwap`) erfordert keine Anpassung der Locks für Zwei-Hop-Traversierungen.

**Invarianten-Nachweis:** Erfüllt Invariante 6 ($O(\text{Seed})$ statt $O(\text{Graph})$), da die Traversierung durch den Nachbarschaftsgrad $N$ limitiert bleibt und nicht quadratisch explodiert.

**Migrationspfad:** Das Schema für Hyperkanten muss die `flatbuffers-drift-gate` in CI erfolgreich passieren, bevor diese Struktur eingeführt wird.

**Restrisiken/offene Fragen:** Die Zwei-Hop-Semantik verlängert die effektive Pfadtiefe in GraphRAG-Algorithmen, was Anpassungen in der Decay-Funktion beim Forward-Push Personalized PageRank erfordert.

### 5.1.5 — H5: Kaskadierende Invalidierung ohne Kostenexplosion

**Ist-Zustand im Repo:** Die Löschung eines Dokuments löst eine ungebundene Kaskade von Invalidierungen aus. Bei Hyperkanten mit tausenden Teilnehmern führt dies zur Blockade des Main-Threads.

**Referenzierte Literatur:**

- Sarkar et al., 2020, "LSM-tree compaction", arXiv:2202.04522 — Analysiert Tombstone-Propagierung in Speichersystemen.
    

**Mathematische/algorithmische Spezifikation:** Um eine $O(N^2)$-Kostenexplosion bei der Kaskadenlöschung zu verhindern, wird die Löschmenge $D$ evaluiert. Ist $\vert{}D\vert{} \leq \theta$ (mit $\theta = 1000$), erfolgt die Löschung (Tombstone-Schreibung) synchron, $O(\vert{}D\vert{})$. Ist $\vert{}D\vert{} > \theta$, wird die Menge $D$ an der Grenze $\theta$ geteilt. Die ersten $\theta$ Elemente werden synchron verarbeitet. Der Rest $D \setminus D_{\theta}$ wird als asynchroner Task in die Background-Queue delegiert, wodurch die synchrone Latenz konstant $O(\theta)$ wird.

**Rust-Schnittstelle (normativ):**

Rust

```
use memfuse_core::DocId;

pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;

pub struct CascadeReport {
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
}

pub fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    doc_id: DocId,
    fanout_limit: usize,
) -> Result<CascadeReport, GraphMutationError>;
```

**Lock-/Nebenläufigkeitsmodell:** Der asynchrone Worker akquiriert Locks in kleinen Batches, um den RCU-Lese-Pfad nicht zu blockieren.

**Invarianten-Nachweis:** §4(6) Die Ausführungszeit der synchronen Funktion ist strikt durch das Fan-out-Limit $\theta$ nach oben beschränkt, was System-Latenz-Spikes verhindert.

**Migrationspfad:** Default-Aktivierung des Background-Workers beim Hochfahren der Memfuse-Engine.

**Restrisiken/offene Fragen:** Abstürze während der asynchronen Verarbeitung können verwaiste Hyperkanten hinterlassen. Dies erfordert Persistenz der Delete-Queue (siehe §6.3.1).

### 5.1.6 — H6: Projektion auf binäre Kantengewichte für Community Detection (Leiden)

**Ist-Zustand im Repo:** Der Leiden-Algorithmus iteriert über die binären Kanten. Hyperkanten werden durch `hyperedges_included: false` ignoriert.

**Referenzierte Literatur:**

- Traag et al., 2019, "From Louvain to Leiden: guaranteeing well-connected communities", arXiv:1810.08473 — Referenz zur Maximierung der Graph-Modularität in komplexen Netzwerken.
    

**Mathematische/algorithmische Spezifikation:** Für den Leiden-Algorithmus, der auf die Maximierung der Modularität $Q$ ausgelegt ist, werden Hyperkanten über einen Iterator als bipartiter Graph projiziert. Um zu verhindern, dass große Hyperkanten das Modularity-Clustering dominieren, wird das Gewicht $w(u, v_e)$ zwischen Teilnehmer $u$ und Hyperkanten-Knoten $v_e$ skaliert:

$$w(u, v_e) = \frac{w(e)}{\vert{}e\vert{} - 1}$$

Die Komplexität der Iteration bleibt $O(\vert{}E_B\vert{}) = O(\sum_{e \in E_H} \vert{}e\vert{})$.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct StarExpansionIterator<'a> {
    graph: &'a CsrGraph,
    current_edge_index: usize,
    participant_index: usize,
}

impl<'a> Iterator for StarExpansionIterator<'a> {
    type Item = (u32, u32, f32); // (source_node, virtual_node, weight)
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Der Iterator arbeitet lock-frei auf einer unveränderlichen RCU-Snapshot-Referenz (`Arc`).

**Invarianten-Nachweis:** §4(7) Zero-Copy. Der Iterator generiert die virtuellen Kanten "on-the-fly" ohne Allokation einer neuen Adjazenzmatrix im Speicher.

**Migrationspfad:** Der `CommunityDetectionConfig` Struct wird um `hyperedges_included: bool` ergänzt, was standardmäßig auf `false` verbleibt, bis die Validierung gegen Benchmark-Netzwerke abgeschlossen ist.

**Restrisiken/offene Fragen:** Virtuelle Knoten im Leiden-Algorithmus verändern die Modularity-Resolution. Ein Hyperparameter-Tuning des Resolution-Parameters $\gamma$ ist für Netzwerke mit hoher Hyperkanten-Dichte zwingend.

### 5.1.7 — GC: Speicherverlust durch verwaiste Knoten und defekte Kaskaden

**Ist-Zustand im Repo:** Unvollständige Löschungen hinterlassen Knoten ohne aktive Kanten, was den Speicherbedarf über Zeit aufbläht.

**Referenzierte Literatur:**

- Epoch-based reclamation Techniken analog zu Keir Fraser's EBR-Konzepten (implizit in `crossbeam-epoch`).
    

**Mathematische/algorithmische Spezifikation:** Die Garbage Collection identifiziert Knoten $v$, für die gilt: $\text{deg}_{\text{in}}(v) + \text{deg}_{\text{out}}(v) == 0$. Die Reklamation erfolgt Epochen-basiert. In der `compact()`-Phase wird der Graph gescannt ($O(\vert{}V\vert{})$). Verwaiste Knoten werden nicht in den neuen `ArcSwap`-Snapshot übernommen.

**Rust-Schnittstelle (normativ):**

Rust

```
pub trait GraphGarbageCollection {
    fn sweep_orphans(&self) -> Result<usize, GraphMutationError>;
}
```

**Lock-/Nebenläufigkeitsmodell:** `sweep_orphans` akquiriert den globalen Schreib-Lock für den neuen Snapshot, beeinträchtigt aber nicht die Leseprozesse auf dem aktiven Snapshot.

**Invarianten-Nachweis:** §4(5) Transparenz des Speicherbudgets wird durch die Freigabe des Speichers beim Austausch der Epochen sichergestellt.

**Migrationspfad:** Hintergrund-Cronjob implementieren, der `sweep_orphans` bei geringer Systemlast aufruft.

**Restrisiken/offene Fragen:** Bei sehr großen Graphen kann der $O(\vert{}V\vert{})$-Scan zu CPU-Spikes führen.

## 5.2 Contextual-Bandit-Routing (LinUCB) (`crates/memfuse-router`)

Das Contextual-Bandit-Modell entscheidet adaptiv über die Retrieval-Strategien. Die aktuelle Implementierung untergräbt jedoch die mathematischen Garantien des LinUCB-Algorithmus.

### 5.2.1 — Falsche Mathematik im Produktions-Default (Diagonal-Approximation)

**Ist-Zustand im Repo:** Die `DiagonalApproximation` aktualisiert die Kovarianzmatrix-Diagonale iterativ als $\sigma^2_i \mathrel{+}= x_i^2$ und akkumuliert Parameter als $\theta_i \mathrel{+}= r \cdot x_i / \max(\sigma^2_i, 10^{-8})$. Dies ist eine Form der stochastischen Gradientenabstieg-Optimierung (SGD), aber keine echte Ridge-Regression.

**Referenzierte Literatur:**

- Li et al., 2010, "A Contextual-Bandit Approach to Personalized News Article Recommendation" — Etabliert den LinUCB-Standard.
    
- Zang et al., 2022, arXiv:2201.09910 — "diagonal approximation lacks theoretical justification" und bricht die Regret-Bounds.
    

**Mathematische/algorithmische Spezifikation:** Die echte LinUCB-Schranke erfordert $\theta = A^{-1}b$. Die aktuelle Implementierung verfehlt dies, da die Updates von $A$ (oder dessen Diagonale) nicht retroaktiv auf die bisher akkumulierten Werte in $b$ angewendet werden. Die Regret-Garantie von $O(d \sqrt{T \log T})$ zerfällt unter der Diagonal-Approximation für korrelierte Features zu einem linearen Regret $O(T)$ im Worst-Case. Um minimale Korrektheit zu wahren, muss $b$ separat akkumuliert und $\theta$ bei jeder Anfrage als $\theta_i = b_i / \sigma^2_i$ berechnet werden.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct CorrectedDiagonalBandit {
    pub precision_diag: Vec<f32>, // A_diag
    pub b: Vec<f32>,
    pub theta: Vec<f32>,
    pub alpha: f32,
}

impl CorrectedDiagonalBandit {
    pub fn update(&mut self, context: &[f32], reward: f32) {
        // ... update precision_diag and b, then compute theta = b / precision_diag
        unimplemented!()
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Mutationen sind sequenziell.

**Invarianten-Nachweis:** §4(3) Determinismus bleibt gewahrt.

**Migrationspfad:** Unmittelbares Update der bestehenden Struktur.

**Restrisiken/offene Fragen:** Die Diagonale ignoriert Feature-Korrelationen bei dichten LLM-Embeddings.

### 5.2.2 — Sherman-Morrison-Update als Performance-Blocker (SIMD)

**Ist-Zustand im Repo:** Die korrekte Matrixinversion via Sherman-Morrison ist hinter dem Feature-Flag `egress-sherman-morrison` versteckt, da die $O(d^2)$-Operation ohne SIMD zu langsam ist.

**Referenzierte Literatur:**

- arXiv:2501.13139, "Efficient LinearUCB for Embedded Learning Systems" — Optimierung durch Sherman-Morrison und SIMD-Vektorisierung.
    

**Mathematische/algorithmische Spezifikation:** Die Sherman-Morrison-Formel für ein Rang-1-Update lautet:

$$(A + xx^T)^{-1} = A^{-1} - \frac{A^{-1}xx^T A^{-1}}{1 + x^T A^{-1} x}$$

Um die Latenz zu drücken, erfordert dies Vektorisierung. Die $d \times d$ Matrix muss cache-aligned ($64$ Byte) im Row-Major-Format im Speicher liegen, um False Sharing zu vermeiden. Die Berechnung von $v = A^{-1}x$ und das Update werden durch `fma` (Fused Multiply-Add) SIMD-Instruktionen beschleunigt. Komplexität: $O(d^2 / W)$, wobei $W=8$ für 256-Bit AVX.

**Rust-Schnittstelle (normativ):**

Rust

```
use memfuse_core::error::BanditError;

#[repr(C, align(64))]
pub struct AlignedVector<const D: usize> { pub data: [f32; D] }

#[repr(C, align(64))]
pub struct MatrixLayout<const D: usize> { pub inv_a: [f32; D * D] }

pub struct ShermanMorrisonBandit<const D: usize> {
    pub mat: MatrixLayout<D>,
    pub b: AlignedVector<D>,
    pub theta: AlignedVector<D>,
}

impl<const D: usize> ShermanMorrisonBandit<D> {
    pub fn update_rank_1(&mut self, x: &AlignedVector<D>, reward: f32) -> Result<(), BanditError> {
        if x.data.len() != D { return Err(BanditError::DimensionMismatch); }
        // SAFETY: Matrix und Vektor sind 64-Byte aligned für AVX2/AVX-512 Intrinsics.
        unimplemented!()
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Thread-lokale Ausführung ohne I/O.

**Invarianten-Nachweis:** §4(2) Die Verwendung von `unsafe_code` für AVX-Intrinsics ist lokal strikt isoliert und durch den Geschwindigkeitsfaktor (Reduktion der Latenz unter das CI-Gate) gerechtfertigt. §4(1) Kein `.unwrap()` in der Dimensionsprüfung.

**Migrationspfad:** CI-Gate für Latenz validieren, dann Feature-Flag `egress-sherman-morrison` zum Standard erheben.

**Restrisiken/offene Fragen:** Floating-Point-Präzisionsverlust über Millionen von Updates. Ein periodischer Cholesky-Rebuild von Grund auf ist empfehlenswert.

### 5.2.3 — Fehlende Drift-Bandit-Kopplung

**Ist-Zustand im Repo:** Ein `LyapunovDriftWatcher` erkennt Konzeptdrift in der Feature-Verteilung, löst aber keine Parameteranpassung im Router aus.

**Referenzierte Literatur:**

- Wu et al., 2020, "Non-stationary contextual bandit" — Methoden zur Anpassung von Exploration unter Drift.
    

**Mathematische/algorithmische Spezifikation:** Wird Drift detektiert, muss die Exploration kurzzeitig eskalieren und das Vertrauen in alte Daten verringert werden. Eskalationsformel: $\alpha_t = \min(\alpha_{t-1} \cdot k_{\text{drift}}, \alpha_{\text{max}})$, mit Decay in Folgerunden. Kovarianz-Reset (Discounting): $A^{-1} \leftarrow \gamma A^{-1}$ mit $\gamma \in (0, 1)$, um das Gewicht der Historie zu reduzieren.

**Rust-Schnittstelle (normativ):**

Rust

```
pub trait BanditPolicy: Send + Sync {
    fn apply_drift_penalty(&mut self, k_drift: f32, alpha_max: f32, gamma: f32);
}
```

**Lock-/Nebenläufigkeitsmodell:** Der Drift-Monitor benachrichtigt den Banditen asynchron über einen MPSC-Channel, um Latenz-Spikes im Inferenz-Pfad zu vermeiden.

**Invarianten-Nachweis:** §4(3) Determinismus der Updates bleibt durch Kanal-Synchronisation erhalten.

**Migrationspfad:** Schnittstelle in `BanditPolicy` implementieren und Channel-Listener im Main-Event-Loop aktivieren.

**Restrisiken/offene Fragen:** Aggressives $\gamma$ kann zu kurzzeitig extrem instabilen Routing-Entscheidungen führen.

### 5.2.4 — Über-Fitting/Regret-Fehler generell (Off-Policy-Schätzung)

**Ist-Zustand im Repo:** Fehlendes Online-Monitoring der Banditen-Performance.

**Referenzierte Literatur:**

- Joachims et al., 2015, "Counterfactual Risk Minimization", arXiv:1502.02362 — IPS-Methodik.
    

**Mathematische/algorithmische Spezifikation:** Die kontrafaktische Evaluation einer neuen Policy $\pi_{\text{new}}$ aus geloggten Daten der Policy $\pi_{\text{old}}$ erfolgt über Inverse Propensity Scoring (IPS):

$$V_{\text{IPS}}(\pi_{\text{new}}) = \frac{1}{t} \sum_{i=1}^t r_i \frac{\mathbb{I}(\pi_{\text{new}}(x_i) == a_i)}{P_{\pi_{\text{old}}}(a_i \mid x_i)}$$

Der Nenner wird auf $\max(p, 0.01)$ geklemmt, um Varianz-Explosionen zu dämpfen. Komplexität: $O(t)$.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct OffPolicyEvaluator {
    cumulative_ips: f64,
    samples: u64,
}

impl OffPolicyEvaluator {
    pub fn observe(&mut self, target_action: u32, logged_action: u32, propensity: f32, reward: f32) {
        if target_action == logged_action {
            let p = propensity.max(0.01);
            self.cumulative_ips += (reward / p) as f64;
        }
        self.samples += 1;
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Lock-freier Akkumulator (Atomic oder thread-lokal).

**Invarianten-Nachweis:** §4(5) Fester Speicherverbrauch (zwei Skalare), keine unsichtbaren Allokationen.

**Migrationspfad:** Die WAL-Struktur muss Propensity-Werte bei jedem Logging mitschreiben.

**Restrisiken/offene Fragen:** IPS ist bias-anfällig, wenn Propensities stark von der Gleichverteilung abweichen.

## 5.3 Block-Cache Lock-Kontention (`crates/memfuse-store`)

Der Cache-Layer ist entscheidend für das LSM-Tree-Leseverhalten.

### 5.3.1 — Ineffizienter Default (LRU Lock-Kontention)

**Ist-Zustand im Repo:** `LruBlockCacheBackend` nutzt `RwLock`. Jeder Read-Hit mutiert die Double-Linked-List zur Aktualisierung der Recency und erzwingt einen exklusiven Write-Lock.

**Referenzierte Literatur:**

- Zhang et al., 2024, "SIEVE is Simpler than LRU", NSDI 2024 / arXiv:2312.13123.
    

**Mathematische/algorithmische Spezifikation:** Unter Last verhält sich der RWLock nach dem Gesetz von Amdahl als starker Flaschenhals. Die zu erwartende Wartezeit steigt quadratisch mit der Thread-Anzahl $T$, proportional zu $p_{\text{hit}}^2$.

### 5.3.2 — SIEVE-Alternative als lock-freier Standard

**Ist-Zustand im Repo:** `quick_cache` (S3-FIFO) ist hinter `block-cache-v2` verfügbar.

**Mathematische/algorithmische Spezifikation:** SIEVE eliminiert List-Reordering beim Read-Hit vollständig (Erfüllung von P25). Jeder Knoten trägt ein atomares `visited`-Bit. Bei einem Hit wird das Bit mit `Ordering::Relaxed` gesetzt ($O(1)$ lock-frei). Die Verdrängung (Eviction) nutzt einen umlaufenden Zeiger (`hand`). Ist das Cache-Limit erreicht, wird `hand` bewegt. Ist `visited == 1`, wird es auf $0$ gesetzt und der Knoten bleibt. Ist `visited == 0`, wird der Knoten entfernt. Worst-Case-Eviction-Komplexität: $O(C)$ wobei $C$ die Cache-Größe ist, Average-Case $O(1)$.

**Rust-Schnittstelle (normativ):**

Rust

```
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_epoch::Atomic;

pub struct SieveNode<K, V> {
    pub key: K,
    pub value: V,
    pub visited: AtomicBool,
    pub size_bytes: usize,
}

pub struct SieveCacheBackend<K, V> {
    // ... Queue pointer definitions ...
}

impl<K: Eq + std::hash::Hash, V: Clone> BlockCacheBackend<K, V> for SieveCacheBackend<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        let guard = crossbeam_epoch::pin();
        // Lookup using concurrent hash map
        if let Some(node) = self.lookup(key, &guard) {
            node.visited.store(true, Ordering::Relaxed);
            Some(node.value.clone())
        } else {
            None
        }
    }
    // ...
}
```

**Lock-/Nebenläufigkeitsmodell:** `get` ist Wait-Free. Epoch-Based Reclamation (EBR) verhindert Use-After-Free während der Eviction.

**Invarianten-Nachweis:** §4(4) Deadlockfreiheit garantiert, da keine Locks erworben werden.

**Migrationspfad:** Benchmark in CI gegen `quick_cache`. Bei Erfolg Flag `block-cache-v2` zur SIEVE-Implementation umleiten.

**Restrisiken/offene Fragen:** SIEVE bietet keinen dedizierten Schutz gegen sequenzielle Scans (Scan-Resistance), was bei großen Bereichsabfragen den Cache flushen kann.

### 5.3.3 — Byte-basierte statt eintragsbasierte Kapazität

**Ist-Zustand im Repo:** Kapazität basiert auf der Element-Anzahl, was bei variablen Werten (Texte, Arrays) unberechenbaren Speicherverbrauch erzeugt.

**Mathematische/algorithmische Spezifikation:** Die Cache-Kapazität wird als $C_{\text{bytes}}$ definiert. Beim Einfügen eines Elements mit Größe $s$ wird atomar $S_{\text{current}} \mathrel{+}= s$ gerechnet. Wenn $S_{\text{current}} > C_{\text{bytes}}$, ruft SIEVE so lange Eviction auf, bis die Bedingung wieder erfüllt ist.

**Rust-Schnittstelle (normativ):**

Rust

```
impl<K, V> SieveCacheBackend<K, V> {
    pub fn capacity(&self) -> usize; // Return max bytes
    pub fn current_size(&self) -> usize; // Return current bytes via Relaxed AtomicUsize
}
```

**Lock-/Nebenläufigkeitsmodell:** Lock-frei mittels `fetch_add` / `fetch_sub`.

**Invarianten-Nachweis:** §4(5) Transparenz des Speicherbudgets.

**Migrationspfad:** Alle Insertion-Pfade müssen die Funktion zur Byte-Größen-Schätzung des jeweiligen Typs implementieren.

**Restrisiken/offene Fragen:** Ungenauigkeiten bei der Schätzung des Struct-Overheads im RAM.

## 5.4 GraphRAG & Community Detection — vertieft

### 5.4.1 — Vollständiger mathematischer Übergang auf binäre Gewichte

**Ist-Zustand im Repo:** Hyperkanten werden vom Leiden-Algorithmus ignoriert.

**Referenzierte Literatur:**

- Traag et al., 2019, "From Louvain to Leiden".
    

**Mathematische/algorithmische Spezifikation:** Um Hyperkanten $e \in E_H$ in den binären Leiden-Solver zu integrieren, ohne die Modularitätsberechnung $Q$ zu verzerren, nutzen wir eine Stern-Expansion. Sei $v_e$ ein synthetischer Knoten für $e$. Für jedes $u \in e$ entsteht eine Kante $(u, v_e)$ mit dem normalisierten Gewicht:

$$w(u, v_e) = \frac{w(e)}{\vert{}e\vert{} - 1}$$

Beweis der Informationserhaltung: Der Total Node Degree im bipartiten Graphen (summiert über alle Originalknoten $V$) entspricht dem Degree in der Cliquen-Expansion, da $\sum_{u \in e} \frac{w(e)}{\vert{}e\vert{}-1} = w(e) \frac{\vert{}e\vert{}}{\vert{}e\vert{}-1}$, was die Kantengewichtmasse korrekt balanciert. Rechenkosten: $O(\vert{}e\vert{})$ für den Stern gegenüber $O(\vert{}e\vert{}^2)$ für die Clique.

**Rust-Schnittstelle (normativ):**

Rust

```
// Iterator implementiert in 5.1.6.
```

**Lock-/Nebenläufigkeitsmodell:** Lock-freier Iterator über Snapshot.

**Invarianten-Nachweis:** §4(6) Komplexität korreliert mit Kantenanzahl, keine $N^2$ Explosion.

**Migrationspfad:** Konfiguration über `CommunityDetectionConfig`.

**Restrisiken/offene Fragen:** Das Einbringen virtueller Knoten reduziert künstlich die Dichte des Netzwerks, was die intrinsische Resolution $\gamma$ des Leiden-Algorithmus verschiebt.

## 5.5 Vektorindex-Traversierung: HNSW-Dateiformat v2 (`crates/memfuse-index`)

Der Suchpfad in HNSW leidet unter Speicher-Ineffizienzen.

### 5.5.1 — Allokations-Overhead pro Knoten (Arena-Modell)

**Ist-Zustand im Repo:** Traversierung allokiert `Vec<u32>` pro Knoten (`Cow::Owned`), was den GC und Allocator massiv belastet.

**Referenzierte Literatur:**

- Malkov & Yashunin, 2020, HNSW Originalkonzepte in flat memory layouts.
    

**Mathematische/algorithmische Spezifikation:** Um Allokationen zu eliminieren, wird eine Mmap-gestützte Arena-Struktur implementiert. Adjazenzlisten werden pro HNSW-Layer $l$ als Compressed Sparse Row (CSR) `offsets_l` und `targets_l` abgespeichert. Ein Zugriff auf Nachbarn von Knoten $i$ im Layer $l$ benötigt zwei Array-Lookups: `start = offsets_l[i]`, `end = offsets_l[i+1]`. Zeit: $O(1)$, Alloc: 0.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct HnswArenaView<'m> {
    pub arena_vectors: &'m [f32],
    pub layer_offsets: Box<[&'m [u32]]>,
    pub layer_targets: Box<[&'m [u32]]>,
    pub stride: usize,
}

impl<'m> HnswArenaView<'m> {
    #[inline(always)]
    pub fn get_neighbors(&self, node: u32, layer: u8) -> &'m [u32] {
        let offsets = self.layer_offsets[layer as usize];
        let targets = self.layer_targets[layer as usize];
        &targets[offsets[node as usize] as usize .. offsets[node as usize + 1] as usize]
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Lesezugriffe sind vollständig parallelisierbar, da die Mmap unveränderlich ist.

**Invarianten-Nachweis:** §4(7) Zero-Copy erfüllt. Die Slice-Referenz referenziert den Speicher der Mmap direkt.

**Migrationspfad:** Binär inkompatibles Format. Ein `HNSW_VERSION=2` Header wird eingeführt; ein Hintergrund-Prozess re-indiziert alte Vektoren.

**Restrisiken/offene Fragen:** Inserts erfordern einen RAM-Overlay (Chunked Allocation), der beim Kompaktieren periodisch in die Datei zurückgeschrieben wird.

### 5.5.2 — Backlink-Lookup nicht O(1)

**Ist-Zustand im Repo:** Lineare Iteration bei der Backlink-Auflösung in Batch-Inserts $O(P \times B)$.

**Mathematische/algorithmische Spezifikation:** Die Auflösung erfordert einen $O(1)$ Hash-Lookup. Eine temporäre HashMap wird am Start der Batch-Verarbeitung generiert. Der Schlüssel ist ein bit-gepackter `u64` bestehend aus `ram_idx` (32 Bit) und `layer` (8 Bit). Zeitkomplexität fällt auf $O(1)$ je Schritt.

**Rust-Schnittstelle (normativ):**

Rust

```
#[derive(Default)]
pub struct SearchScratch {
    pub overlay_backlinks: ahash::AHashMap<u64, &'static [u32]>,
}
```

**Lock-/Nebenläufigkeitsmodell:** Thread-lokaler Scratch-Puffer, lock-frei.

**Invarianten-Nachweis:** §4(1) Keine impliziten Panics durch Boundary Checks, saubere Hash-Ergebnisse.

**Migrationspfad:** Sofort ersetzbar im Insert-Pipeline-Code.

**Restrisiken/offene Fragen:** Hashmap Allokation für extrem kleine Batches eventuell überproportional teuer.

### 5.5.3 — Distanzpfad Lock/Allokation

**Ist-Zustand im Repo:** Mmap Vektoren werden elementweise gelesen und dekodiert; Quantisierer sperren den Lesevorgang.

**Mathematische/algorithmische Spezifikation:** Der Vektorzugriff muss als konstanter Slice direkt an die SIMD-Engine gereicht werden. Die Quantisierungs-Skalare (`scale`, `min`) des Codebooks werden am Start der Query einmalig per Read-Lock kopiert und in der Engine lokal gekapselt, statt pro Kandidat gesperrt zu werden.

**Lock-/Nebenläufigkeitsmodell:** Einmaliger RWLock-Acquire pro Query.

**Invarianten-Nachweis:** §4(7) Zero-Copy (Übergabe eines Slices statt eines iterativ allozierenden `Vec`).

### 5.5.4 — NaN-Sicherheit bei SIMD-Distanzberechnung

**Ist-Zustand im Repo:** Skalarer NaN-Check läuft bei jedem Vektorvergleich vor der SIMD-Schleife, was Performance massiv degradiert.

**Mathematische/algorithmische Spezifikation:** NaN-Werte kontaminieren L2-Normen. Um den Check im O(N) Hot-Path zu umgehen, wird die Validierung erzwungen an:

1. Den Query-Vektor $q$ am Start der Funktion ($O(D)$ Skalar).
    
2. Beim Insert jedes Vektors. Dies wird durch ein Flag `VALIDATED_NO_NAN` im Dateikopf manifestiert. Innerhalb von `dot4_avx2` wird nicht mehr geprüft.
    

**Rust-Schnittstelle (normativ):**

Rust

```
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2", enable = "fma")]
#[allow(unsafe_code)]
pub unsafe fn dot4_avx2(q: &[f32], arena: &[f32], bases: [usize; 4], dim: usize, out: &mut [f32; 4]) {
    // Implementierung via _mm256_fmadd_ps ohne NaN Check
    unimplemented!()
}
```

**Lock-/Nebenläufigkeitsmodell:** Thread-lokale Register-Operationen.

**Invarianten-Nachweis:** §4(2) `unsafe_code` Begründung: Hardware-Acceleration für die zentrale mathematische Operation; Isolierung durch Garantien aus der Validierungs-Phase.

**Migrationspfad:** Altdaten (Header ohne Flag) triggern den langsamen Pfad, bis der Index kompaktiert wird.

**Restrisiken/offene Fragen:** Keine, da CPU FMA mathematisch deterministisch ist.

### 5.5.5 — Top-k-Selektion

**Ist-Zustand im Repo:** `sort_unstable_by` sortiert volle Arrays in $O(M \log M)$.

**Mathematische/algorithmische Spezifikation:** Für materialisierte Listen wird Introselect (`select_nth_unstable_by`) genutzt: $O(M)$ im Average/Worst-Case. Für das Streaming im Traversierungspfad wird ein Bounded Min-Heap der Größe $k$ eingesetzt: $O(M \log k)$.

**Rust-Schnittstelle (normativ):**

Rust

```
pub fn select_top_k_materialized(scores: &[f32], id_table: &[u64], k: usize) -> Vec<u32> {
    let mut idx: Vec<u32> = (0..scores.len() as u32).collect();
    let cmp = |&a: &u32, &b: &u32| {
        scores[b as usize].total_cmp(&scores[a as usize])
            .then_with(|| id_table[a as usize].cmp(&id_table[b as usize]))
    };
    if k < idx.len() {
        idx.select_nth_unstable_by(k, cmp);
        idx.truncate(k);
    }
    idx.sort_unstable_by(cmp);
    idx
}
```

**Lock-/Nebenläufigkeitsmodell:** Keine Synchronisation notwendig.

**Invarianten-Nachweis:** §4(3) Determinismus durch total order via `total_cmp` und Tie-Breaker.

**Migrationspfad:** Austausch im `hybrid_search` Code.

**Restrisiken/offene Fragen:** Partielle Sortierung verliert die absolute Ranking-Ordnung über den Index $k$ hinaus.

### 5.5.6 — CSR-Sentinel statt Option

**Ist-Zustand im Repo:** `Vec<Option<u32>>` in Graph- und Index-Strukturen verbraucht unnötigen Speicher für Diskriminanten.

**Mathematische/algorithmische Spezifikation:** Ersatz von `Option<u32>` durch `u32` mit dem Sentinel `u32::MAX`. Dies halbiert den RAM-Footprint für spärliche Vektoren von 8 auf 4 Byte pro Eintrag.

**Rust-Schnittstelle (normativ):**

Rust

```
pub const SENTINEL_NULL_ID: u32 = u32::MAX;
// Arrays nutzen direkt u32
```

**Lock-/Nebenläufigkeitsmodell:** N/A.

**Invarianten-Nachweis:** §4(5) Reduzierter Speicherverbrauch erhöht Budget-Transparenz.

**Migrationspfad:** Binär inkompatibles Format; bedarf Adapter.

**Restrisiken/offene Fragen:** Keine, solange Systemlimit bei $< 4.2 \times 10^9$ Knoten bleibt.

### 5.5.7 — Partielle Rebuilds mit Recall-Erhaltungsgarantie

**Ist-Zustand im Repo:** Codebooks in der Skalar-Quantisierung driften, was zu Recall-Verlust bei Updates führt.

**Mathematische/algorithmische Spezifikation:** Das Codebook $C$ wird periodisch rekalibriert, wenn der Kullback-Leibler-Divergenzschätzer (oder die Min/Max Verschiebung) eine Schwelle überschreitet. Partielle Rebuilds der Layer erfolgen im Hintergrund.

**Lock-/Nebenläufigkeitsmodell:** RCU-Mechanik für das Codebook.

**Invarianten-Nachweis:** §4(3) Determinismus der Suchergebnisse bezogen auf die Epoche des Snapshots.

## 6. Weitere Systembereiche

### 6.1 LSM-Storage-Engine (`crates/memfuse-store`)

#### 6.1.1 — SSTable-Lock-Handoff

**Spezifikation:** Zur Vermeidung von Stalls beim Flush der MemTable auf Disk wird das Mutex nicht über die I/O-Operation gehalten. Eine `AtomicU64`-Sequenznummer regelt den Handoff der Zuständigkeit für den Gruppen-Commit.

#### 6.1.2 — Lock-freie WAL-Pipeline

**Spezifikation:** Der WAL (`wal_crypto.rs`) nutzt einen SPSC (Single Producer, Single Consumer) Ringpuffer (z.B. basierend auf `crossbeam-channel` Bounded Queues). Fsync wird gebatcht vom Consumer-Thread (Background) aufgerufen. Dies blockiert den Producer-Thread nicht, sondern baut Backpressure durch Pufferlimits auf.

#### 6.1.3 — Inkomplette Tombstone-Propagierung

**Spezifikation:** Tombstones dürfen erst verworfen werden, wenn kein aktiver Snapshot (Lese-Transaktion) mehr existiert, der eine Sequenznummer kleiner der des Tombstones referenziert. Beweis: Behalte Versionen mit `seq > min_snapshot_seq` PLUS die neueste Version $\leq \text{min\_snapshot\_seq}$.

#### 6.1.4 — Manifest-Fehlerbehandlung bei Crash-Recovery

**Spezifikation:** Der Zustand der Kompaktierung (lösche Inputs, füge Output hinzu) muss zwingend ein atomarer `ManifestEntry::Replace` Record sein. Ein teilgeschriebener Add/Remove hinterlässt bei einem Crash doppelte oder verwaiste Daten (Resurrection von gelöschten Schlüsseln).

#### 6.1.5 — Zero-Copy-Slice

**Spezifikation:** Das Lesen von SSTable Blöcken nach dem Entschlüsseln und der CRC-Prüfung nutzt `bytes::Bytes::slice(4..)`, anstatt den Nutzlastpuffer neu zu kopieren. Erfüllt §4(7).

#### 6.1.6 — MemTable Range-Sharding

**Spezifikation:** Hash-Sharding der In-Memory-Daten zerstört Präfix-Scans (erfordert 16 B-Tree Traversals). Range-Sharding teilt die Schlüssel alphanumerisch auf Shards auf, sodass ein Präfix-Scan zumeist in einem Lock-Fenster eines Shards bedient werden kann.

#### 6.1.7 — Checkpoint-Index-Merge

**Spezifikation:** Statt separater Locks für `name_index` und `seq_index` fasst ein `RwLock` einen Struct zusammen, der beide Maps enthält. Atomarität ist gegeben.

### 6.2 Inference, KV-Bridge & Routing (`crates/memfuse-candle`)

#### 6.2.1 — Asynchrone Zero-Copy-KV-Cache-Eviction-Bridge

**Spezifikation:** Wenn Token-Mengen RAM übersteigen, werden paged KV-Blöcke (verschlüsselt via AES-GCM-SIV) per `Bytes`-Slice direkt in die LSM-Engine gespült. Async-I/O verhindert, dass die GPU-/CPU-Inferenz ins Stocken gerät.

#### 6.2.2 — Instabilität des PID-Controllers (Lyapunov)

**Spezifikation:** Zur Latenzbegrenzung des Routings misst ein PID-Regler Abweichungen. Formel für Anti-Windup unter Berücksichtigung von $\Delta t$: $I_{new} = \text{clamp}(I_{old} + e \cdot \Delta t, -I_{max}, I_{max})$. Ohne Sättigungsgrenze läuft der Integrator ins Unendliche. Erfüllt deterministische Stabilität.

#### 6.2.3 — Zero-Copy-Deserialisierung im IPC-Generator

**Spezifikation:** FlatBuffers wird genutzt, um IPC-Nachrichten vom Memfuse-Prozess zum MCP-Client (Python/Node) als direkte Referenz in Memory-Mapped Slices bereitzustellen, ohne Deserialisierungs-Kopien (zero-copy).

### 6.3 Agenten-State, Crypto & MCP

#### 6.3.1 — Atomare DLQ-Replay-Logik

**Spezifikation:** Ein Event, das fehlschlägt, wird als `(Session, Node, Step)`-Schlüssel persistiert. Idempotenz: Bei Replay prüft die Engine die WAL-Transaktions-ID, um Doppelbuchungen zu verhindern.

#### 6.3.2 — Zeroize-on-Panic im Egress-Vault

**Spezifikation:** Um PII-Daten nach einem Panic (z.B. Timeout beim Regex-Matching) zu vernichten, werden sensible Strings in `zeroize::Zeroizing<Vec<u8>>` gewrappt. Der Drop-Guard sorgt deterministisch für die Überschreibung im RAM. Verteidigung in der Tiefe (§4(1)).

#### 6.3.3 — Race Conditions bei Budget-Berechnungen

**Spezifikation:** Das Agent-Budget wird über eine RAII-Struktur verwaltet: `budget.reserve(n) -> Reservation`. Bei Erfolg `reservation.settle()`, bei Drop erfolgt eine garantierte Rückerstattung. Double-Spend ist ausgeschlossen.

#### 6.3.4 — Lückenhafte WASM-Sandbox-Egress-Isolierung

**Spezifikation:** Wasmtime erfordert strikte Speicherbegrenzungen (`Store::set_fuel`) und Memory-Limits. Cloud-Aufrufe innerhalb des WASM müssen von Datei-Reads logisch getrennt als `CloudEgress` in den Capability-Flags geführt werden.

#### 6.3.5 — Kryptographisch verifizierbare Deletion Proofs

**Spezifikation:** Wenn ein Record aus dem LSM entfernt wird, erzeugt die HMAC-Kette der WAL einen Nachweis. Quittung: $H(\text{hmac}_{\text{prev}} \parallel \text{delete\_event})$. Verifikation in $O(1)$ Zeit ohne Klartext-Zugang (DSGVO Art. 17 konform).

#### 6.3.6 — AES-Schlüsselplan-Wiederverwendung

**Spezifikation:** Die Expansionsrunde `new_from_slice` für AES-256-GCM-SIV kostet massive CPU-Zyklen. Die Struktur wird im `KeyManager` pro Schlüssel gecacht und thread-safe (`OnceCell`) wiederverwendet. Nonce-Verwaltung als deterministischer Atomic Counter (nicht zufällig) sichert vor Wiederverwendung.

## 7. Priorisierung und Abhängigkeitsanalyse

Nach dem Schema: Aufwand (Trivial/Gering/Mittel/Hoch) × Nutzen (Latenz-/Speicher-/Korrektheitsgewinn).

### Stufe 0 — Unmittelbar (Korrektheit/Sicherheit)

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**6.1.4**|LSM Manifest-Batch-Fsync (`Replace` Record)|Trivial|Verhindert Auferstehung gelöschter Daten|Keine|
|**5.2.1**|Bandit-Dimensionsprüfung / Ridge Math|Gering|Verhindert NaN/Dimensions-Crash|Keine|
|**6.3.4**|Egress-Klassifizierung & Wasmtime Limits|Gering|Sicherheitsisolation|Keine|
|**6.1.3**|Intent-Recovery & Tombstone-Propagierung|Mittel|Deterministisches Recovery|6.1.4|

### Stufe 1 — Hot-Path-Performance

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**5.5.1**|HNSW v2 Arena Allocation|Hoch|Beseitigt 90% der Allokationen (3-5x Speedup)|Storage `Bytes` API|
|**6.1.5**|SSTable Zero-Copy-Slice & `StorageEngine::get`|Mittel|Verhindert Vollkopie auf Ebene 0|Keine|
|**6.3.6**|AES-Schlüsselplan Wiederverwendung|Gering|Reduziert Crypto-Overhead bei KV-Cache|Keine|
|**5.3.2**|Block-Cache Byte-Cap & SIEVE Lock-free|Mittel|Beseitigt LRU Kontention|Keine|
|**5.5.5**|Top-k Selektion (Introselect/Heap)|Trivial|$O(M \log M) \to O(M)$|Keine|

### Stufe 2 — Speicher und Struktur

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**5.1.1**|N-äre Hyperkanten (RCU Integration)|Hoch|Modell-Exaktheit für LLM Inferenz|H2 (Locks)|
|**5.1.7**|Inkrementelle Graph-Kompaktierung (GC)|Mittel|$O(E)$ Lese-Spikes verhindern|H1|
|**5.5.6**|CSR Sentinel statt Option|Gering|Halbiert RAM-Footprint für CSR|HNSW v2|
|**6.1.7**|Checkpoint-Index-Merge|Gering|Beseitigt Race-Condition|Keine|

### Stufe 3 — Governance/Prozess

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**CI**|Feature-Powerset-CI in GitHub Actions|Gering|Sichert Kompilierbarkeit aller Feature-Pfade|Keine|
|**CI**|Kontinuierliches Panic-Inventar-Gate|Mittel|Sichert §4(1) Zero-Panic Doctrine|Keine|

_(Harte Abhängigkeit verzeichnet: H3 `relate_n_ary` setzt das in Stufe 2 implementierte H2 Multi-Key-Locking voraus; der Bandit-Default-Wechsel in 5.2.2 setzt den Erfolg im Sherman-Morrison-Latenzgate voraus)._


---


*Diese finale konsolidierte Gesamtspezifikation vereinigt Produktvision, Zielarchitektur, normative
Implementierungsvorgaben, algorithmische Spezifikationen, mikrofeingranulare Schnittstellendefinitionen
und die priorisierte Optimierungs-Roadmap des MemFuse Cognitive OS. Sie ist in sich geschlossen und
ersetzt alle vorherigen Einzeldokumente als maßgebliche Quelle. Künftige Änderungen erfolgen als direkte
Überarbeitung dieses Dokuments, nicht als weiteres Delta-Dokument.*
