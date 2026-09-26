# Contextra — Finale Produktspezifikation (Synthese)
## Mikrofeingranulare Schnittstellen- und Implementierungsspezifikation des Zielprodukts

**Dokumentstatus:** Normativ. Konsolidiert und ersetzt: `contextra-roadmap.md`, `CONTEXTRA_SPEC_2 (v4)`, <!-- crate-ref-ignore -->
`CONTEXTRA_SPEC_UPDATED (Fassung 4 inkl. Teile A2–A4)`, `CONTEXTRA_SPEC_v5_Open_Closed`,
`CONTEXTRA_INTERFACE_SPEC_v1`, `CONTEXTRA_PAPER_TRIAGE_2026`, sowie den Forschungsbericht
*„Friktionsfreie Personalisierung und autonome SLM/LLM-Interaktion"*.
**Methodik:** Synthese aller genannten Dokumente, geprüft gegen einen Live-Klon von
`https://github.com/tfufuz1/contextra` (Workspace-Root-`Cargo.toml`, `README.md`). Wo Vorfassungen
widersprüchlich waren, gilt die jeweils zuletzt korrigierte Fassung (Fassung 4 / v5 / Interface-Spec v1)
als Tatsachenquelle. Bereits behobene Bugs und rein historische Fehlerprotokolle sind bewusst nicht
übernommen — dieses Dokument beschreibt ausschließlich die **Zielarchitektur** des Endprodukts.
**Reifegrad-Kennzeichnung:** 🟢 produktiv · 🟡 hinter Feature-Flag, korrekt · 🔴 spezifiziert, zu bauen ·
🔒 closed-source, lizenzpflichtig.

---

## 0. Leitprinzip und Normativität

> **Korrektheit schlägt Performance schlägt Feature schlägt Vision.**

- **MUSS / MUSS NICHT / SOLL / KANN** folgen RFC-2119-Semantik.
- Jede Schnittstelle gehört zu genau einer der vier Schichten aus §5. Die Schicht bestimmt die
  SemVer-Schwelle (§18.1).
- Jede API-Änderung, jede neue Crate-Grenze, jeder neue MCP-Tool-Endpunkt MUSS dieses Dokument zuerst
  aktualisieren, bevor Code gemergt wird (CI-Gate `spec-sync`).
- Ein Feature auf einem unverifizierten Kern zu bauen ist unzulässig — §22 (Härtungsplan) blockiert §7–§17
  hart, unabhängig davon, wie vollständig ein Feature spezifiziert ist.

---

## 1. Produktthese

**Contextra ist eine souveräne, vollständig lokal betriebene Gedächtnis- und Ausführungsschicht für
KI-Agenten**, die vier Eigenschaften in einem einzigen Kern vereint, die ein Cloud-Python-Stack
(Mem0, Zep, Graphiti, Qdrant+LangChain) architektonisch nicht liefern kann:

1. **Beweisbarkeit statt Zusage** — Löschung, Datenzugriff und Agentenhandlung sind kryptographisch
   nachprüfbar, extern verifizierbar ohne Contextra-Zugriff, nicht nur vertraglich zugesichert.
2. **Air-Gap-Fähigkeit** — läuft vollständig ohne Netzwerk, ohne API-Key, ohne Telemetrie.
3. **Deterministische Performance** — Pure Rust, kein GC, Kaltstart < 50 ms, Zero-Panic-Ziel im
   Produktionspfad.
4. **Ein Kern, zwei Märkte** — derselbe verifizierte Kern bedient sowohl den performance-getriebenen
   Personal-/Agent-AI-Nutzer (offen, kostenlos) als auch den nachweispflichtigen regulierten Betrieb
   (Kanzlei, Praxis, Mittelstand — geschlossen, kommerziell), über Feature-Kompilation entkoppelt,
   nicht über Code-Gabelung.

**Verkaufbares Kernversprechen:**

> Beweisbare Datenhoheit für KI-Agenten — jede Speicherung, jede Löschung, jeder Cloud-Zugriff, jede
> Agentenhandlung ist kryptographisch nachweisbar, extern prüfbar ohne Contextra-Zugriff.

Dies ist kein RAG-Qualitätswettbewerb (dort gewinnt Contextra nicht gegen Qdrant/LanceDB), sondern ein
**Nachweisbarkeits- und Souveränitätswettbewerb**, den ein API-Wrapper um einen Cloud-Provider strukturell
nicht führen kann, weil er den Pfad Anfrage→Modell nicht kontrolliert.

**Die vierte Kennzahl** (neben Kaltstart-Latenz, residentem Speicher-Footprint, p99-Retrieval-Latenz):
**Zeit von Löschantrag bis extern verifizierbarem Beweis** (Knopfdruck → signiertes Dokument, prüfbar ohne
Contextra-Zugriff) — die einzige Kennzahl, die kein Cloud-Konkurrent strukturell liefern kann.

---

## 2. Zielgruppen, Vertriebsformen, Nicht-Ziele

### 2.1 Was Contextra ist

Eine eingebettete Bibliothek, kein Server-Produkt. Läuft im Prozess des aufrufenden Agenten (Rust-Crate,
Python-Paket) oder als lokaler MCP-Server über stdio-JSON-RPC. Keine netzwerkexponierte Multi-Tenant-Instanz,
kein Daten-Egress außer über das explizit angeforderte Privacy-Gateway (§15.4).

### 2.2 Zwei Zielgruppen, ein Kern, zwei Vertriebsformen

| | **Personal/Agent AI (offen)** | **Reguliertes Unternehmen (geschlossen)** |
|---|---|---|
| Person | Entwickler, lokales MCP-Memory für Claude/Cursor | Kanzlei/Praxis/Mittelstand mit Mandanten-/Patientendaten |
| Kaufgrund | Geschwindigkeit, Souveränität, kein Vendor-Lock-in | Nachweispflicht (DSGVO, GoBD, Berufsrecht), kein Cloud-Erlaubnis |
| Preis | 0 (MIT/Apache-2.0) | Lizenz pro Instanz/Mandant, Appliance oder SaaS-Aufsatz |
| Vertriebsweg | `cargo add`, `pip install`, `uvx contextra-mcp` | Signiertes Binary/Appliance, Aktivierungsschlüssel |
| Feature-Ring | `fast` | `fast` + `sovereign` + `compliance` |

Beide Gruppen laufen auf **demselben** verifizierten Kern (Ring 0–3). Der Unterschied ist ausschließlich
Kompilations- und Lizenzgrenze (§16), keine Code-Gabelung.

### 2.3 Nicht-Ziele (bindend — jede Abweichung ist ein Scope-Fehler)

- **Kein** Cloud-SaaS mit netzwerkexponierter Multi-Tenant-Instanz.
- **Kein** verteiltes Cluster-/Konsenssystem (`contextra-cluster`-Veto). Passives WAL-Shipping für Backup <!-- crate-ref-ignore -->
  ist erlaubtes Fernziel (Phase 4), kein Kernbestandteil.
- **Kein** LLM-Trainings- oder Quantisierungs-Framework — Contextra *konsumiert* fertig quantisierte
  GGUF-/ONNX-Modelle über Ports, es trainiert und quantisiert keine Gewichte selbst.
- **Kein** primär GUI-getriebenes Desktop-Produkt.
- **Kein** Robotik-/Embedded-Realtime-Vertical (Tokio-Nebenläufigkeit ist mit Echtzeit-Zertifizierung
  architektonisch unvereinbar).
- **Kein** Multi-Agent-LLM-Orchestrator — `contextra-agent` ist ein einzelner deterministischer
  `checkpoint → execute → commit → audit`-Workflow, keine Rollenverteilung über mehrere LLM-Instanzen
  (bewusste Abgrenzung von LangGraph/AutoGen).
- **Kein** Feature-Zuwachs vor grünem Tier-0/1-Gate (§22.2).

---

## 3. Architekturprinzipien und Systeminvarianten

### 3.1 Architekturprinzipien P1–P30 (verbindlich, Auswahl der tragenden Prinzipien)

- **P1–P4:** WAL-First (keine Zustandsänderung sichtbar vor `fsync`), deterministische Recovery allein aus
  dem Log, Zero-Panic-Doktrin, Determinismus über injizierte Zeit-/Zufallsquellen.
- **P5:** Strikte Abhängigkeitsrichtung im Crate-Graph (Ring-Modell, §4), keine Rückwärtskanten.
- **P12:** LLM-Kostenschutz (`max_llm_calls_per_cycle`) für jede generative Pipeline-Stufe.
- **P23:** Wall-Clock-Budget orthogonal zu Fuel-Budget in der WASM-Sandbox.
- **P24:** Lokalität — Kosten einer latenzkritischen Operation skalieren mit der anfragebestimmten
  Teilmenge, nie mit der Gesamtgröße des Bestands.
- **P25:** Cache-Lokalität bei Streaming-Zugriffsmustern (Batch-Größe, z. B. 16 Kandidaten pro Kanalzugriff).
- **P26:** Ring-0-Kern bleibt synchron (kein `tokio`); asynchrone Hintergrundarbeit lebt in Ring 3.
- **P27:** Jeder Port ist ein Trait in Ring 0/1, dyn-kompatibel (`&dyn Trait`-fähig, CI-geprüft).
- **P28:** Injizierter Determinismus — jede Zufalls-/Zeitquelle läuft über `Rng`/`Clock`-Ports, nie über
  globale RNG-Aufrufe im Hot-Path.
- **P29:** Kein globaler veränderlicher Zustand (`static`/`OnceLock` nur für Konstanten); veränderlicher
  Zustand gehört immer einer Instanz.
- **P30:** Jeder Crate-Zuschnitt erfüllt mindestens eines von: Isolation einer volatilen Abhängigkeit (I) ·
  Unsafe-Insel (U) · eigener Bounded Context (C) · Größe > 8.000 LOC (S) · Richtungserzwingung/Composition
  Root (D). Ohne erfülltes Kriterium wird zusammengelegt statt neu geschnitten.

### 3.2 Die sieben Systeminvarianten (gelten für jeden Ring, jeden Crate, unabhängig von Lizenz)

1. **Zero-Panic-Doktrin.** Kein `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`/`unimplemented!` in `src/`
   außerhalb normativer Signatur-Stubs. Keine ungeprüfte Indizierung mit berechneten/externen Indizes
   (`get(..)` + Fehler statt `[]`). Keine überlaufende Größenarithmetik (`checked_*`/`saturating_*`).
   Durchsetzung: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo,
   clippy::unimplemented)]` als Workspace-Lint.
2. **Unsafe-Isolation.** `#![forbid(unsafe_code)]` in jedem Nicht-Insel-Crate. Genau drei benannte
   Unsafe-Inseln: `contextra-sys` (mmap, `LockedBuf`, Owner-only-ACL), `contextra-simd` (Distanzkerne),
   `contextra-wire` (FlatBuffers-Adapter). Jeder Block trägt `// SAFETY:`.
3. **Determinismus.** Gleiches Binary, gleiche SIMD-Dispatch-Stufe, gleiche injizierte Zufalls-/Zeitquelle,
   gleicher logischer Zustand ⇒ gleiches Ergebnis (Replay, Recovery, Tests). Hash-Container-Iterationsreihenfolge
   fließt nie in Ergebnisse/IDs/Serialisierung ein (sortierte Reihenfolge oder totale Ordnung mit Tiebreaker).
   Hasher mit festen Seeds, einmal pro Instanz angelegt, nie pro Aufruf.
4. **Deadlockfreiheit.** Dokumentierte Lock-Hierarchie (§6.1), kanonische Sperrreihenfolge innerhalb einer
   Stufe. Kein `.await` unter einem `std`-Lock. Nachweis über `loom` für jeden nebenläufigen Kernpfad.
5. **Speicherbudget-Transparenz.** Jede Operation, die neben bestehendem Zustand einen zweiten aufbaut
   (Compaction, Rebuild, Konsolidierung), berechnet Residenz **und** Spitzenbedarf vorab (capacity-basiert)
   und bricht kontrolliert mit typisiertem Fehler ab, statt unbegrenzt zu wachsen. Queues/Puffer in
   Produktionspfaden sind begrenzt.
6. **Lokalität und beschränkte Arbeit (P24).** Über die Obergrenze hinausgehende Arbeit wandert in
   persistente, wiederaufnehmbare Hintergrundarbeit statt synchron zu blockieren.
7. **Zero-Copy über Ring-Grenzen.** Payloads werden geteilt (`Bytes`, `Arc<[T]>`, mmap-Slices), nicht
   kopiert. **Snapshot-Regel S1:** ein per `ArcSwap` veröffentlichter Snapshot ist unveränderlich und
   enthält keine Container mit innerer Mutabilität (`scc::HashMap`, `Mutex`, ergebnisrelevante Atomics).

---

## 4. Ring-Architektur: der finale Crate-Graph

Das Ring-Modell löst den historischen Layer-0–5-Crate-DAG ab (dessen Abwärts-Abhängigkeitsregel real
verletzt war). Migration erfolgt strangler-artig — bis ein Schritt abgeschlossen ist, existiert der alte
Crate als `#[deprecated]`-Re-Export. Live-Repo-Stand bestätigt: mehrere Ring-0/1-Crates
(`contextra-mvcc`, `contextra-adapt`, `contextra-kvcache`, `contextra-engine`, `contextra-cognition`,
`contextra-rank`) existieren bereits als eigenständige Workspace-Member neben den noch nicht abgelösten
Alt-Crates (`contextra-core`, `contextra-db`) — die Migration ist im Gange, nicht am Anfang.

| Ring | Crate | Inhalt | Kriterium (P30) |
|---|---|---|---|
| **0** (sync, kein `tokio`) | `contextra-types` | IDs, `ModelFingerprint`, Filter-AST, Budgets, Importance, `ErrorClass`, Schema-Versionen, Tombstone-Semantik | D |
| | `contextra-ports` | Alle Trait-Verträge (§5.2), sync + async (`BoxFuture`) | D |
| | `contextra-mvcc` | `SeqLog`, `SnapshotRegistry`, `TxBuffer`; loom-getestet | C, D |
| | `contextra-wire` | FlatBuffers-Generat und Adapter — **Unsafe-Insel** | U |
| | `contextra-sys` | `ReadOnlyMap` (mmap), `LockedBuf` (mlock/VirtualLock), Owner-only-ACL — **Unsafe-Insel** | U |
| | `contextra-simd` | Distanzkernel, Laufzeit-Dispatch (AVX-512/NEON) — **Unsafe-Insel** | U |
| | `contextra-crypto` | Schlüsselhierarchie, AEAD, WAL-HMAC-Kette, `DeletionProof`, Zeroize, Anti-Tamper | C |
| | `contextra-vector` | HNSW, DiskANN, SQ8-/RaBitQ-Quantisierung | S, C |
| | `contextra-text` | BM25/BM25F, deutsche Morphologie/Kompositazerlegung | C |
| | `contextra-graph` | CSR, Forward-Push-PPR, Leiden-Community-Detection, Hyperkanten | S, C |
| | `contextra-rank` | 4-Signal-Fusion, Isotonic-/Platt-Kalibrierung, Drift | C |
| | `contextra-adapt` | Bandit (LinUCB/Sherman-Morrison/FC-TS), Lyapunov, PID, Homeostat, Decay — `Clock`/`Rng` injiziert | C |
| **1** (Persistenz, async an I/O-Grenzen) | `contextra-store` | WAL (Group-Commit, HMAC-Kette), LSM, MVCC-Pin | S, C |
| | `contextra-kvcache` | Prefix-Radix-Baum, KV-Blöcke, Tiering, AEAD, Segmentdateien | C |
| | `contextra-checkpoint` | Time-Travel-Registry gegen `StorageEngine`-Port, kein globaler Zustand | C, D |
| **2** (Blätter, volatile Abhängigkeiten) | `contextra-infer-candle` | GGUF, eigenes Llama-Modell mit `KvState` | I |
| | `contextra-infer-ollama` | HTTP-Backend, Contextual-Chunk-Prefixing | I |
| | `contextra-infer-onnx` | `ort`, Cross-Encoder-Reranker; **aus `default-members` ausgeschlossen** | I |
| | `contextra-sandbox` | WASM-Isolation, Fuel-/Wall-Clock-Budget, implementiert `ToolSandbox` | I |
| **3** (Anwendungskern) | `contextra-engine` | `Collection`, Transaktionen, `RetrievalPlanner`, Ingestion, Export/Import, `ComputePool` | S, C |
| | `contextra-cognition` | Konsolidierung, Synthese, semantische Aggregation, Kompaktierung, Scheduler | C |
| | `contextra-privacy` | Egress-Gateway, PII-Vault, DLP, `GuardedPayload`, Prompt-Injection-Filter | C |
| | `contextra-router` | SLM-Profil-Routing, MCP-Dispatch (schlank, keine Numerik — die liegt in `adapt`) | C |
| | `contextra-agent` | Workflow-Engine (Idle → Running → Checkpointing → Auditing), DLQ | C |
| **4** (Ränder) | `contextra` | Fassade, Builder — **einzige Composition Root** | D |
| | `contextra-mcp` | stdio-JSON-RPC-Server, Protokoll, Tool-Wiring | C |
| | `contextra-py` | PyO3-Bindings, regulärer Workspace-Member, `panic = "unwind"` an der Wurzel | I |
| **Tooling** | `contextra-testkit`, `contextra-bench`, `xtask` | Fault-VFS, `ManualClock`, In-Memory-`StorageEngine`, Benchmarks, CI-Architekturlinting | — |
| **🔒 Compliance-Schicht** (separates Repo/Binary) | `contextra-license`, `contextra-compliance-export`, `contextra-tenant` | Lizenzdurchsetzung, Aufbereitung, Mandanten-Scoping | C | <!-- crate-ref-ignore -->

**Abhängigkeitsmatrix (verbindlich):**

```
Ring 0  →  types → {ports, mvcc, wire, sys, simd, crypto} → {vector, text, graph, rank, adapt}
           Kerne (vector, text, graph, rank, adapt) kennen einander NICHT; Kommunikation nur über types/ports/mvcc.
Ring 1  →  nur Ring 0. Kein Ring-1-Crate hängt von einem anderen Ring-1-Crate ab.
Ring 2  →  nur types, ports (+ crypto für Fingerprints). Niemals Ring 1 oder 3.
Ring 3  →  Ring 0, Ring 1, Ports von Ring 2 (nie deren konkrete Crates). Intern: privacy < engine < {cognition, router, agent}.
Ring 4  →  alles.
dev-Kanten → nur contextra-testkit und Crates desselben oder tieferen Rings.
```

**Erzwingung:** `tests/layering.rs` prüft jede Kante (normal/build/dev/target-spezifisch) gegen die Matrix;
`deny.toml` verbietet `tokio`/`candle-core`/`ort`/`wasmtime`/`pyo3`/`reqwest` außerhalb des jeweils einen
erlaubten Blatt-Crates (`[[bans.deny]]` mit `wrappers`); `tests/unsafe_islands.rs` verifiziert die
Drei-Inseln-Regel. **Governance-Gate `GOV-D`:** `xtask check-module-reachability` verifiziert, dass jede
`.rs`-Datei unter `src/` von genau einer `mod`-Deklaration aus erreichbar ist — eine unerreichbare Datei mit
divergierender Zweitdefinition eines Ports ist ein CI-Fehler.

Die Compliance-Schicht kompiliert **gegen die offenen Ports aus Ring 0**, niemals gegen konkrete
Ring-3-Implementierungen — dieselbe Disziplin, die zwischen den Rängen gilt, wird als Lizenzgrenze
wiederverwendet, nicht neu erfunden.

---

## 5. Vier-Schichten-Interface-Modell

```
┌─────────────────────────────────────────────────────────────────┐
│ Schicht 4 — PRODUKTGRENZE (extern, versioniert, Wire-stabil)      │
│   MCP-Tools (contextra-mcp) · Python-FFI (contextra-py) ·         │
│   Binäres Wire-Format (contextra-wire / FlatBuffers)              │
├─────────────────────────────────────────────────────────────────┤
│ Schicht 3 — FASSADE (Rust-öffentlich, workspace-stabil)           │
│   `Contextra` / `Collection<S,V>` / Query-Builder                │
├─────────────────────────────────────────────────────────────────┤
│ Schicht 2 — PORTS (Traits, hexagonale Grenze, austauschbar)       │
│   StorageEngine, VectorIndex, TextIndex, GraphIndex,              │
│   EmbeddingProvider, Checkpoint, MetricsSink, Clock, Rng, IdGen,  │
│   KvPrefixStore, BanditPolicy, ToolSandbox, …                     │
├─────────────────────────────────────────────────────────────────┤
│ Schicht 1 — DOMÄNENTYPEN (Datenverträge, kein Verhalten)          │
│   TenantId, DocId, EntityId, TxId, CollectionId, ContextraError,  │
│   DistanceMetric, MemoryType, ScoredDocument …                    │
└─────────────────────────────────────────────────────────────────┘
```

**Regel:** Eine Adapter-Implementierung (z. B. `LsmStorage`) implementiert Schicht 2, hängt aber NIE von
Schicht 3 ab. Automatisiert geprüft über `tests/layering.rs` und `xtask`-Architekturlinting.

### 5.1 Schicht 1 — Domänentypen (`contextra-types`)

Alle IDs sind `#[repr(transparent)]`-Newtypes über Ganzzahlen (bewusst gegen stringly-typed APIs).

| Typ | Repräsentation | Zweck |
|---|---|---|
| `TenantId` | `struct TenantId(u64)`, `SYSTEM = 0`, `try_new(0) → Err` | Mandanten-Isolation (INV-TENANT-1) |
| `CollectionId` | `struct CollectionId(u64)`, `try_new(0) → Err` | Logische Sammlung innerhalb eines Tenants |
| `DocId` | `u64` (Default) / `u128` (Feature `wide-ids`, BLAKE3-Hash) | Dokument-/Chunk-Identität |
| `EntityId` | `struct EntityId(u64)` | Knoten im Wissensgraphen |
| `TxId` | `struct TxId(u64)`, dreigeteilter Wertebereich (Collection/Wall-Clock-Gap/System) | MVCC-Transaktionsnummer, monoton |
| `RoleId` / `HyperEdgeId` | `u32` / `u64` Newtypes | n-äre Hyperkanten-Rollen |
| `DistanceMetric` | Enum (Cosine/L2/Dot) | Implementiert `DistanceCalculator` |
| `MemoryType` | Enum | Lifecycle-Klassifikation |
| `ContextraError` | `#[non_exhaustive] enum` (`thiserror`) | **Einzige** Fehler-Enum-Wurzel des Workspace |

**Fehlerkontrakt (normativ):** `ContextraError` ist `#[non_exhaustive]`; neue Varianten werden ausschließlich
angehängt. Jeder Downstream-Match MUSS einen `_ =>`-Arm haben. Es DARF KEINE zweite Fehler-Enum-Wurzel geben —
`contextra-mcp`/`contextra-py` übersetzen ausschließlich in Protokollfehler, definieren nie neu.
`Result<T> = std::result::Result<T, ContextraError>` ist der kanonische, workspace-weit re-exportierte Alias.

### 5.2 Schicht 2 — Ports (`contextra-ports`, vollständiges Trait-Inventar)

Jeder Port ist `Send + Sync + 'static` und dyn-kompatibel (CI-geprüft in `#[cfg(test)] mod dyn_safety`).

| Trait | Datei | Zweck |
|---|---|---|
| `StorageRead`, `StorageWrite`, `StorageEngine` | `storage.rs` | LSM-Storage-Abstraktion inkl. `get_at_seq`/`scan_prefix_at` (Snapshot-Isolation) |
| `VectorIndex`, `HybridSearchProvider` | `vector_index.rs` | Vektorsuche-Abstraktion |
| `TextIndex`, `TextEmbeddingEngine`, `SegmentSynthesizer` | `text_index.rs` | Volltext-Indizierung |
| `GraphIndex` (inkl. `add_entity`, `commit`), `CommunityResolver` | `graph_index.rs` | Graph-Index-Abstraktion |
| `GraphCollectionMutation` | `graph.rs` | Graph-Mutation (`relate`, `relate_n_ary`) |
| `EmbeddingProvider`, `TextGenerator`, `LlmTextGenerator`, `LlmTextGeneratorStreaming` | `embedding.rs` | LLM-/Embedding-Abstraktion; `ContextSegment::with_fingerprint` bindet `ModelFingerprint` für Provenienz |
| `DistanceCalculator`, `MemoryLifecycleManager`, `GroundingValidator`, `ResponseGroundingValidator`, `ContextPreparer` | `lifecycle.rs` | Kognitions-Lifecycle; `score_grounding` ist die technische Grundlage jeder Explainability-Oberfläche (§17) |
| `Checkpoint`, `CheckpointCoordinator`, `Snapshot` | `checkpoint.rs` | Agent-Recovery, Time-Travel |
| `MetricsSink` | `metrics.rs` | Zeitreihen-Export |
| `Clock` | `clock.rs` | Injizierte Zeit (P28) |
| `Rng` | `rng.rs` | Injizierter Zufallszahlengenerator (P28) |
| `IdGen` | `id_gen.rs` | Injizierte ID-Generierung (P28) |
| `KvPrefixStore` | `kv.rs` | KV-Cache-Präfix-Abstraktion, **`tenant: TenantId` erstes Argument jeder Methode** |
| `DriftStatusProvider` | `observability.rs` | Drift-Beobachtbarkeit |
| `ToolSandbox` | `sandbox.rs` | Sichere Werkzeugausführung (implementiert von `contextra-sandbox`) |
| `BanditPolicy` | `bandit.rs`-Vertrag (Adapter in `contextra-adapt`) | `select_arm`/`update`, `Result`-basierte Dimensionsprüfung |

**Entscheidungsregel für neuen Code:** Jede neue Integration eines externen Backends (Inferenz, Storage,
Embedding, Sandbox-Runtime) MUSS als neuer oder erweiterter Port in dieser Tabelle landen, bevor Ring-0/1-Code
ihn direkt aufrufen darf. Eine Port-Erweiterung ist Pflichtbestandteil jeder solchen PR.

**Härtungsvertrag (Multi-Tenant):** Jeder neue Port, der tenant-sensible Daten berührt, MUSS `tenant: TenantId`
als erstes Argument jeder Methode führen — ein tenant-loser Zugriffspfad, der sich erst zur Laufzeit über
einen Kontext-Parameter „irgendwo im Aufrufstapel" auflöst, ist unzulässig, weil nicht statisch prüfbar.
Durchsetzung über `clippy`-Lints und `tests/layering.rs`.

### 5.3 Schicht 3 — Fassade (`contextra-engine`, `contextra`)

**`Contextra`** (Top-Level-Handle) — konfiguriert über `ContextraConfig` (Builder-Pattern,
`with_consolidation_launcher`). `EmbeddingBackend`-Enum wählt Candle/ONNX/Ollama. Escape-Hatch
`inner_storage(&self) -> Arc<LsmStorage>` ist ausschließlich für Tests/Migrationswerkzeuge zulässig; jede
Produktivnutzung ist ein Architekturbruch.

**`Collection<S: StorageEngine, V: VectorIndex>`** — generisch über Storage-/Vektor-Index-Adapter (Standard:
`LsmStorage`, `HnswIndex`); Austausch eines Adapters ist ein Typparameter-Wechsel, kein Rewrite.

| Methode (Auszug) | Zweck |
|---|---|
| `new` / `with_hnsw` | Konstruktion mit konkretem Vektor-Index |
| `with_embedder` / `set_embedder` | Laufzeit-Austausch des Embedders |
| `with_kv_store` / `set_kv_store` / `kv_store` | Bindung an mandantenisolierten KV-Cache |
| `graph_index() -> Arc<CsrGraph>` | Zugriff auf CSR-Graphrepräsentation |
| `run_community_detection[_with_config]` | Graph-Community-Erkennung |
| `evict_decayed_chunks` / `reap_by_thermostat` / `reap_expired_documents` / `trigger_reaper` | Lifecycle-Sweeps |
| `run_percolation_check` | Ausbreitungs-/Schwellenwertprüfung im Graph |
| `relate` / `relate_bidirectional` / `relate_n_ary` | Kantenerzeugung |
| `consolidate_semantic_hyperedges` (🔴 §13.4) | Dritte Konsolidierungsstufe (LeanRAG) |
| `begin_transaction` / `allocate_tx` / `next_tx` | Transaktionssteuerung |
| `drop_collection` | Vollständige, unwiderrufliche Löschung |

**Query-Builder (Fluent API):**
```rust
collection.query()
    .text(query: &str)     // BM25F-Signal
    .vector(&embedding)    // HNSW-/DiskANN-Signal
    .graph_seeds(&[EntityId])  // PPR-Traversierung
    .filter(MetadataFilter)    // Metadaten-Filter, FilterOp-Enum
    .scope(ScopeConstraint)    // ACORN-Hard-Boundary (§10.4)
    .k(usize)                  // MUSS ≤ MAX_SEARCH_K sein
    .execute()                 // -> Result<Vec<ScoredDocument>>
```
Text-, Vektor- und Graphsignal werden über `fusion::fuse_signals`/`fuse_search_results_with_strategy`
zusammengeführt (§8.1); Metadaten-Filterung über `MetadataFilter::matches(&self, metadata: &Value) -> bool`.

### 5.4 Schicht 4 — Produktgrenze

#### 5.4.1 MCP-Server (`contextra-mcp`, Binary `contextra-mcp`)

Tools, dispatcht über `McpServer::call_tool(name: &str, args: &Value)`. **Jeder** Aufruf durchläuft zuerst
`self.sandbox.validate_tool_call(name, args)` — kein Pfad umgeht die Sandbox-Validierung.

| Tool | Pflichtparameter | Optionale Parameter | Rückgabe |
|---|---|---|---|
| `contextra_search` | `query: string` (≤ `MAX_SEARCH_QUERY_BYTES`, nicht leer) | `collection` (Default `"default"`), `k`/`limit` (gedeckelt auf `MAX_SEARCH_K`), `scope` (ACORN-Hard-Boundary) | `ScoredDocument[]`, jedes Element mit `content_provenance: "retrieved_untrusted_data"` |
| `contextra_insert` | Dokumenttext | `collection` | Insert-Quittung; löst optional automatische Tripel-Extraktion aus (§14) |
| `contextra_get` | Dokument-ID | `collection` | Dokument |
| `contextra_forget` | Dokument-ID/Filter | `collection` | Löschbestätigung (Vorstufe zu `DeletionProof`, §15.1) |
| `contextra_collections` | – | – | Liste vorhandener Collections |
| `contextra_consolidate` | `collection` | Konsolidierungsparameter | `ConsolidationPipelineResult` (§12) |
| `contextra_cloud_query` | `CloudQueryRequest` | – | `CloudQueryResponse`, läuft zwingend durch das Egress-Gateway (§15.4) |
| `contextra_relate` | `from`, `to`, `label` | – | binäre Kante |
| `contextra_relate_n_ary` | Teilnehmerliste (≤ `MAX_RELATE_PARTICIPANTS = 64`) | – | `HyperEdgeId` |
| `contextra_cascade_status` | `doc_id` | – | `CascadeStatus` (Fortschritt asynchroner Löschkaskaden, §7.6) |

**Normativer Kontrakt für `contextra_search`:**
1. `query` fehlt/kein String → `McpError::invalid_params`.
2. `query` leer/nur Whitespace → `invalid_params`.
3. `query` überschreitet `MAX_SEARCH_QUERY_BYTES` → `invalid_params` mit Byte-Zahlen in der Meldung.
4. `collection` fehlt/leer → Fallback `"default"`; ungültiger Name → Fehler, kein stiller Fallback.
5. `k`/`limit`: `k` hat Vorrang; Werte über `MAX_SEARCH_K` werden **gekappt, nicht abgelehnt**, mit
   `tracing::warn!` protokolliert.
6. Ausführung ist immer hybrid (Text+Vektor, optional Graph) — nie reines Vektor-only oder Text-only bei
   diesem Tool.
7. Jedes Ergebnis durchläuft `injection_guard.process_result(...)` vor Rückgabe — nicht optional, nicht über
   Tool-Parameter abschaltbar.

**Fehlerabbildung:** `McpError` ist die einzige Fehlerform, die die Produktgrenze verlässt. Jede
`ContextraError`-Variante wird über `From<ContextraError> for McpError` abgebildet — keine
`ContextraError`-Variante verlässt MCP unkonvertiert (strukturell ausgeschlossene Informationslecks).

#### 5.4.2 Python-FFI (`contextra-py`)

PyO3-Bindings, binden die Schicht-3-Fassade direkt, nicht die Ports. Root-Profil `panic = "unwind"`
(FFI-Voraussetzung), `contextra-py` ist regulärer Workspace-Member. Testpflicht: ein Panic muss im
`maturin build --release`-Wheel als `PyErr` ankommen, nicht nur im Debug-Build.

#### 5.4.3 Wire-Format (`contextra-wire`, `schemas/contextra.fbs`)

FlatBuffers-Schema für Persistenz und IPC (Zero-Copy-KV-Cache-Sharing). Additive Feldnummerierung, kein
Feld-Reuse — On-Disk-Format und IPC-Format teilen dasselbe Schema. **CI-Drift-Gate**
(`xtask check-flatbuffers-drift`) vergleicht den Hash des generierten Codes gegen einen committeten
Referenz-Hash; jede Schema-Änderung ohne begleitende Regenerierung schlägt den Merge fehl.

---

## 6. Speicherschicht

### 6.1 Grundprinzip und Lock-Hierarchie

Keine Zustandsänderung wird sichtbar, bevor sie physisch ins WAL geschrieben und mit dem Datenträger
synchronisiert wurde. Schlüsselgranulares statt Collection-weites Locking:

```
collections (RwLock) → kv_locks (schlüssel-granular, KvKeyLocks) → embedder (RwLock)
```

`KvKeyLocks::key_hash` ist der **einzige** zulässige Weg, aus einem Schlüssel einen Lock-Hash zu erzeugen
(eine `ahash::RandomState`-Instanz mit festen Seeds pro `KvKeyLocks`-Instanz, nie pro Aufruf).
`acquire_multi_sorted(&[u64])` erwirbt N Shards strikt in aufsteigender Shard-Index-Reihenfolge (kanonisches
Multi-Key-Locking, Voraussetzung für `relate_n_ary` mit N Teilnehmern, §7.5 H2).

### 6.2 Modulstruktur `contextra-store`

```
contextra-store/src/
├── lsm.rs                # LSM-Tree, Compaction
├── wal.rs / wal_flusher.rs  # WAL + HMAC-Kette; begrenzter MPSC-Group-Commit-Actor
├── block_cache/{mod,lru,sieve}.rs   # BlockCacheBackend-Trait, LRU (Default) + SIEVE (Opt-in)
├── kv_locks.rs            # KvKeyLocks
└── error.rs
```

### 6.3 WAL-Pipe: begrenzter MPSC-Group-Commit-Actor

**Zielarchitektur:** genau ein Eigentümer des File-Handles, von `fsync` und der HMAC-Kettenfortschreibung
(der Flusher-Task); viele Produzenten, eine **begrenzte** Queue (`DEFAULT_WAL_QUEUE_CAPACITY = 1024`).

```rust
pub struct WalHandle { /* mpsc::Sender<WalCommand> */ }
impl WalHandle {
    /// Wartet bei voller Queue (Backpressure); kehrt erst NACH fsync zurück.
    pub async fn append(&self, payload: Bytes) -> Result<WalSeq, WalError>;
    /// Nicht wartend: volle Queue ⇒ Err(Backpressure).
    pub fn try_append(&self, payload: Bytes) -> Result<oneshot::Receiver<Result<WalSeq, WalError>>, WalError>;
}
```

**Vertrag (Durability):** `append` kehrt erst nach dem `fsync` zurück, der den Eintrag enthält. Der Flusher
ist der **einzige** Aufrufer von `fsync`; die HMAC-Kette wird ausschließlich dort fortgeschrieben (kein
Kettenfork durch geteilte Eigentümerschaft am File-Handle).

### 6.4 Block-Cache

`BlockCacheBackend<K, V>`-Trait, zwei austauschbare Backends: `LruBlockCacheBackend` (Default) und
`SieveCacheBackend` (Opt-in, `block-cache-v2`, geringere Lock-Kontention als LRU).

### 6.5 Öffentliche Storage-API

`StorageRead::get_at_seq`/`scan_prefix_at` sind der Snapshot-Isolationsvertrag — **jede** Storage-
Implementierung MUSS Point-in-Time-Reads gegen eine explizite `seq_no` unterstützen (Grundlage für MVCC
und für den Löschbeweis, da „danach war der Schlüssel nicht mehr lesbar" nur mit versionierten Reads
exakt führbar ist).

---

## 7. Wissensgraph: binäre Kanten und n-äre Hyperkanten

### 7.1 Designentscheidung

Kein generisches RDF-Reifikations-Pattern. Stattdessen eine kohärente, erstklassige Rust-Struktur mit
Zero-Copy-Deserialisierung via FlatBuffers/Mmap. Fakten mit mehr als zwei Beteiligten (z. B. „Partei A
verklagt Partei B wegen Vertrag C vor Gericht D") werden als **eine** `HyperEdgeId` mit N `RoleBinding`s
modelliert, nicht als N binäre Kanten — struktureller Unterschied zu Property-Graph-Konkurrenten.

### 7.2 Datenstruktur

```rust
pub struct HyperEdgeId(pub u64);
pub struct RoleId(pub u32);
pub struct RoleBinding { pub role: RoleId, pub entity: EntityId }

/// Besitzende Struktur — Schreibpfad (relate_n_ary).
pub struct HyperEdge {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: Arc<[RoleBinding]>,      // min. 2, validiert
    pub weight: f32,
    pub tx_valid_from: Option<TxId>,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}

/// Zero-Copy-Lesesicht — Traversal-Hotpath (hyperedges_for_entity, PPR-Expansion).
/// `participants` referenziert den RCU-Snapshot-Speicher direkt (kein Vec-Allocation-Overhead).
pub struct HyperEdgeView {
    pub id: HyperEdgeId, pub predicate: EdgeType,
    pub participants: ArcSlice<RoleBinding>, pub weight: f32, pub source_doc_id: Option<DocId>,
}
```

**API-Oberfläche:**
```rust
impl Collection {
    /// Binärer Pfad — bleibt Hotpath, KEINE interne Umleitung auf relate_n_ary.
    pub fn relate(&self, from: EntityId, to: EntityId, predicate: EdgeType, doc_id: DocId) -> Result<(), DbError>;
    /// Hyperkanten-Schreibpfad.
    pub fn relate_n_ary(&self, predicate: EdgeType, participants: &[(RoleId, EntityId)], doc_id: DocId)
        -> Result<HyperEdgeId, DbError>;
}
```

**child_edge_ids** (zwingende Erweiterung, Voraussetzung für LeanRAG §13.4): jede Hyperkante trägt eine
Liste subsumierter Original-/Sub-Hyperkanten-IDs. Ohne dieses Feld kann ein Löschauftrag einen bereits
konsolidierten Super-Knoten nicht bis zur Quelle durchqueren — das würde die Löschbeweis-Zusage silently
brechen. FlatBuffers-Feld `child_edge_ids: [uint64]` (leer = Blatt-Hyperkante).

### 7.3 Traversal-Semantik

PPR-Traversierung (§8.2) wird um einen Hyperkanten-Expansionsschritt ergänzt: beim Erreichen eines Knotens
werden zusätzlich alle `RoleBinding`-Partner als „virtuelle" Nachbarn mit rollenspezifischem
Gewichtsabschlag (`hyperedge_decay`, Default `0.85`) eingespeist.

### 7.4 Persistenz

LSM-Präfix `__graph:hyperedge:` (FlatBuffers-serialisiert), Sekundärindex
`__graph:hyperedge_by_entity:{EntityId} -> Vec<HyperEdgeId>`.

### 7.5 Sechs Integrationslösungen (verbindlich)

| # | Problem | Lösung |
|---|---|---|
| **H1** | RCU-Snapshot-Inkonsistenz zwischen CSR und Hyperkanten-Sekundärindex | Hyperkanten-Index wird **Teil von `GraphInner`** — automatisch vom `ArcSwap`-Swap miterfasst |
| **H2** | Multi-Key-Locking-Deadlock bei `relate_n_ary` mit N Teilnehmern | `acquire_multi_sorted` — Shards strikt in aufsteigender Index-Reihenfolge |
| **H3** | `SignalKind` ist ein geschlossenes Enum | Erweiterung NIEMALS durch neue Varianten; Hyperkanten-Signal läuft über bestehendes `Graph`-Signal |
| **H4** | FlatBuffers-Schemaerweiterung erfordert aktives CI-Drift-Gate | Gate MUSS grün sein, **bevor** `HyperEdge`-Schema gemergt wird (harte CI-Job-Abhängigkeit) |
| **H5** | Cascade-Invalidierung: unbegrenzter Fan-out, keine Crash-Sicherheit | Hartes Fan-out-Limit (`DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT = 1000`), **ein** atomarer WAL-Commit für synchrone Tombstones + Queue-Einträge, **persistente** idempotente Queue (`__graph:cascade_queue:{doc_id}:{hyperedge_id}`), Worker in Batches von 128, `DeletionProof` erst bei vollständig leerer Queue |
| **H6** | Community-Detection/Leiden sieht Hyperkanten nicht | Stern-Expansion: jede Hyperkante wird zu einem künstlichen Knoten $v_e$; binäre Kanten $O(|e|)$ statt Cliquen-Expansion $O(|e|^2)$; Gewicht $a(e) = 2w(e)/(|e|-1)$ (Konvention K, Schur-Komplement-Äquivalenz zur Referenz-Clique bewiesen) |

**Cascade-Report-Kontrakt:**
```rust
pub struct CascadeReport {
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
    pub ticket: Option<CascadeTicket>,
    pub deletion_proof: Option<contextra_crypto::DeletionProof>, // nur bei vollständiger Löschung
}
pub struct CommunityDetectionConfig { pub resolution_gamma: f32, pub hyperedges_included: bool }
```

### 7.6 Fehler-Enum

```rust
pub enum GraphMutationError {
    LockAcquisitionTimeout,
    RoleBindingInvalid(String),
    EpochReclamationPending,
    InsufficientParticipants(usize),
}
```

---

## 8. Retrieval-Pipeline: 4-Signal-Fusion

### 8.1 Signal-Modell

```rust
/// Geschlossen — Erweiterung NIEMALS durch neue Varianten (H3).
pub enum SignalKind { Vector, Text, Graph, EdgeReinforcement /* feature-gated */ }
```

Fusion standardmäßig über **Reciprocal Rank Fusion (RRF)** — score-blind, robust bei Signalausfall.
Score-normalisierte Fusion als Opt-in mit hartem RRF-Fallback.

### 8.2 Graph-Signal: Forward-Push-PPR (Andersen-Chung-Lang)

Exploriert nur Knoten, die signifikant zur PageRank-Masse beitragen: $O(1/\alpha\epsilon)$, **unabhängig
von der Gesamtgröße des Graphen** (P24). Erweitert um Hyperkanten-Partner als virtuelle Nachbarn (§7.3).

```rust
pub struct PprParams { pub alpha: f32, pub epsilon: f32, pub hyperedge_decay: f32 }
pub fn forward_push_ppr(graph: &CsrGraph, seeds: &[EntityId], params: &PprParams) -> AHashMap<EntityId, f32>;
```

### 8.3 Volltextsuche: BM25/BM25F mit Block-Max WAND

```rust
pub struct Bm25fParams { pub k1: f32, pub b: f32, pub field_weights: AHashMap<FieldId, f32> }
impl ResidentPostingIndex {
    pub fn search_topk(&self, query_terms: &[TermId], k: usize, params: &Bm25fParams) -> Vec<(DocId, f32)>;
}
pub fn decompose_german_compound(word: &str, dictionary: &CompoundDictionary) -> Vec<String>;
```
Deutsche Fachterminologie (juristisch/medizinisch) wird als Domänenpaket auf dieser Basis ergänzt (§17.6),
nicht als neue Engine.

### 8.4 Vektorindex: HNSW + DiskANN

```rust
pub struct HnswIndex<const D: usize> { /* layers, entry_point, sq8_codebook */ }
impl<const D: usize> HnswIndex<D> {
    pub fn search_knn(&self, query: &[f32; D], k: usize, ef_search: usize) -> Vec<(DocId, f32)>;
    pub fn insert(&mut self, id: DocId, vector: [f32; D]) -> Result<(), IndexError>;
    pub fn delete(&mut self, id: DocId) -> Result<(), IndexError>; // native Tombstone
}
pub struct DiskAnnIndex<const D: usize> { /* mmap, native Tombstones, kein HNSW-Fallback nötig */ }
```
Zielarchitektur „HNSW v2": Arena-Allocator (`HnswArena`, Offsets statt Pointer, CAS-basiertes Relinking
statt Mutex). SQ8-Quantisierung mit gemessener Bias-Kalibrierung (`Sq8Bias`, kalibrierte Score-Schwellen
subtrahieren den gemessenen Offset, statt ihn analytisch abzuleiten). RaBitQ als produktive
Binärquantisierungs-Alternative zu SQ8.

### 8.5 ACORN — Prädikatsagnostische Vektorsuche für Hard-Boundary-Retrieval (🔴, §11.4)

Für harte Scope-Begrenzung („Antworte NUR basierend auf Dokument X"): klassisches Post-Filtering kollabiert
bei hoher Selektivität (z. B. 1 %), da der Beam vorzeitig verhungert; Pre-Filtering degradiert zum
Brute-Force-Scan. **ACORN-$\gamma$** erhöht die Kantenzahl pro HNSW-Knoten künstlich auf $\gamma \cdot M$
(z. B. $\gamma=2$) und erlaubt die Traversierung von Knoten, die das Prädikat nicht erfüllen, um deren
Nachbarn als Brücke zu erreichen — garantiertes Retrieval in $O(\log N)$ bei striktem Ausschluss aus dem
Ergebnis (Zero-Cross-Contamination).

```rust
pub trait FilteredIndex {
    fn search_knn_acorn(&self, query: &[f32], k: usize, ef_search: usize,
        predicate: &dyn Fn(DocId) -> bool) -> Result<Vec<(DocId, f32)>, IndexError>;
}
```
Kandidaten-Speicher als `Vec` statt `HashSet` (keine Hash-/Allokator-Latenz im Hot-Path, begünstigt
SIMD-Vektorisierung über `contextra-simd`).

### 8.6 Community-Detection: Leiden

Binärer Pfad produktiv; Stern-Expansion für Hyperkanten-Projektion (§7.5 H6) über `StarExpansionIterator`,
der einen `Arc<GraphInner>`-Snapshot hält statt Hyperkanten zu klonen (deterministische Reihenfolge,
allokationsfrei pro `next()`).

### 8.7 Provenance-Tracking

`ProvenanceBuilder` (benannte Struct statt positioneller Parameter): `source_doc_id`,
`signal_contributions: Vec<(SignalKind, f32)>`, `fusion_mode`, `calibrated_threshold` — Grundlage der
Explainability-Oberfläche (§17.7).

---

## 9. Contextual-Bandit-Routing

### 9.1 Gemeinsame Schnittstelle

```rust
pub trait BanditPolicy: Send + Sync {
    fn select_arm(&self, context: &[f32]) -> Result<RetrievalStrategy, BanditError>;
    fn update(&mut self, context: &[f32], arm: RetrievalStrategy, reward: f32) -> Result<(), BanditError>;
}
pub enum RetrievalStrategy { Vector, Text, Graph, Hybrid }
```

### 9.2 Drei Implementierungsvarianten

| Variante | Reifegrad | Eigenschaft |
|---|---|---|
| `DiagonalApproximationBandit` | 🟢 Low-Memory-Opt-out | SGD-artig, **keine** exakte Ridge-Regression |
| `ShermanMorrisonBandit` | 🟡 Produktions-Ziel | Mathematisch korrekte inkrementelle Matrixinversion, $O(d^2)$, `Result`-basierte Dimensionsprüfung; einziger Pfad mit LinUCB-Regret-Garantie |
| `FlowCorrectedThompsonBandit` (FC-TS, §13.3) | 🔴 | Richtungsbewusste Drift-Korrektur über Transport-Operator |

**Discounting (Vergessen):** $A^{-1} \leftarrow \gamma^{-1}A^{-1}$ **vor** dem Rang-1-Update (Unsicherheit
wächst korrekt nach Vergessen — die inverse Umsetzung würde die Politik überzuversichtlich machen).
**Default-Umstellung** auf `ShermanMorrison` ist an ein CI-Latenzbudget-Gate gebunden: < 5 % der medianen
LLM/SLM-Inferenzlatenz.

### 9.3 Gedeckelter Lyapunov-Drift-Regelkreis mit PID-Kopplung

$D_t = D_{KL}(P_t \| P_{base})$ misst Distributional Drift der Reward-Verteilung; $L(t) = \tfrac12 D_t^2$,
Drift-Exponent $\lambda_t = L(t)-L(t-1)$. Bei $\lambda_t > 0$ (Instabilität) wird der Confidence-Schwellenwert
des GASP-Validators autonom erhöht (z. B. 0,70 → 0,90) — erreicht das fusionierte Signal diesen Wert nicht,
verweigert das System die Generierung (**Abstention**) statt einer störenden Rückfrage. Anti-Windup:
Integrator stoppt sofort bei PID-Sättigung.

```rust
pub struct LyapunovDriftWatcher { pub drift_decay_window: u32, pub drift_gamma: f32, /* … */ }
impl LyapunovDriftWatcher {
    pub fn update(&self, error: f32, dt_seconds: f32, saturated: bool) -> f32;
}
```
**Drift-Bandit-Kopplung (verbindlich):** Die Drift-Reaktionsmethode wird bei `DriftDetected` tatsächlich
aufgerufen (konfigurierbares `k_drift`), nicht nur geloggt.

### 9.4 Off-Policy-Evaluation (IPS) — Voraussetzung Randomisierung

LinUCB wählt deterministisch (Argmax) — Propensity des gewählten Arms ist 1, aller anderen 0; IPS ist ohne
Randomisierung nicht definiert. **$\varepsilon$-greedy-Logging-Policy** über $K$ Arme
($\mu(a|x) = (1-\varepsilon)\mathbb{1}[a=a^*] + \varepsilon/K$) mit $\varepsilon \ge 0{,}04$; Zufallszahl aus
dem `Rng`-Port (P28); **Propensity zum Entscheidungszeitpunkt** wird im WAL-/Provenance-Datensatz gespeichert
(`propensity: f32`, nie nachträglich rekonstruiert). Self-Normalized-IPS (SNIPS) bei Bedarf gegen
Varianzprobleme.

---

## 10. Inferenz, KV-Cache und friktionsfreie SLM/LLM-Personalisierung

### 10.1 Inferenz-Backends (Ring 2)

- `contextra-infer-candle`: lokales GGUF-Backend, air-gap-fähig, eigenes Llama-Modell mit `KvState`.
- `contextra-infer-ollama`: HTTP-Backend, Contextual-Chunk-Prefixing
  (`OllamaClient::generate(prompt, contextual_prefix)`).
- `contextra-infer-onnx`: `ort`-Backend, Cross-Encoder-Reranking; aus `default-members` ausgeschlossen
  (kein Netzwerk-Download im Standard-Build).

### 10.2 KV-Cache v2 — Zero-Copy-IPC und Prefix-Sharing

`contextra-kvcache` (Ring 1): Prefix-Radix-Baum, `KvPrefixStore`-Port-Implementierung, AEAD-verschlüsselte
Segmentdateien, kontrollierter LSM-Spill bei Speicherdruck statt verlustbehaftetem Verwerfen. **Geteilter
Kontext über mehrere Agenten hinweg ohne doppelte Inferenzkosten:** Ein Agent schreibt einen Prefix-Block
über `KvPrefixStore::insert`, ein zweiter liest denselben Block über `lookup` — beide Aufrufe sind zwingend
mit derselben `TenantId` parametrisiert; Cross-Tenant-Zugriff ist durch die Signatur (nicht durch
Konvention) ausgeschlossen.

### 10.3 GASP-Validator (Grounding-Aware Sensitivity by Perturbation)

Isotonic-kalibrierte Post-hoc-Halluzinationsprüfung, gekoppelt an das Confidence-Gate des Lyapunov-Drift-
Regelkreises (§9.3): erhöhter Schwellenwert bei erkannter Instabilität führt zu Abstention statt Rückfrage.

### 10.4 Friktionsfreie Personalisierung — vier HCI-Archetypen, vier Mechanismen (🔴, Zielarchitektur)

Die Lösungsebene verlagert sich von der Applikationsschicht (Ring 4) in die mathematischen und
speichertechnischen Fundamente (Ring 0/1) — kognitive Mühelosigkeit auf Nutzerseite wird durch
algorithmische Strenge und Autonomie auf Systemebene erkauft.

| Archetyp | Antipattern | Mechanismus | Ziel-Crate |
|---|---|---|---|
| **Der faule Nutzer** (Zero-Effort) | Verweigert explizites Onboarding, formuliert nur beiläufige Präferenzen | **RIE-Greedy** (Regularization-Induced Exploration): implizites Profiling ohne Rückfragen | `contextra-adapt` (Ring 0) |
| **Der unbedachte Nutzer** (High Input Volatility) | Widersprüchliche/sprunghafte Eingaben, „Lost in Conversation" | **Contrastive Hidden-State Steering** + State-Machine-Rollback | `contextra-infer-candle` (Ring 2) ↔ `contextra-agent` (Ring 3) |
| **Der schnell genervte Nutzer** (Zero-Disturbance) | Over-Contextualization, störende Bestätigungsdialoge | **SnapKV/H2O/GemFilter**-Eviction + **KIVI**-2-Bit-Quantisierung + Lyapunov/PID-Abstention | `contextra-kvcache` (Ring 1), `contextra-adapt` (Ring 0) |
| **Extrem granulares Scoping** | Harte Dokumentbegrenzung ineffizient in klassischem HNSW | **ACORN $\gamma$-Augmentation** (§8.5) | `contextra-vector` (Ring 0) |

#### 10.4.1 RIE-Greedy: implizites Profiling ohne Stochastik im Hot-Path

Präzisionsmatrix und Informationsvektor mit **Temporal Decay Factor** $\gamma \in (0,1]$:

$$\Lambda_t = \lambda I_d + \sum_{s=1}^{t} \gamma^{t-s} x_s x_s^\top, \qquad
\eta_t = \sum_{s=1}^{t} \gamma^{t-s} r_s x_s$$

Vor dem Rang-1-Update: $\Lambda_t^{-1} \leftarrow \gamma^{-1}\Lambda_{t-1}^{-1}$ (Sherman-Morrison, $O(d^2)$).
Deterministisch-gierige Auswahl $\hat\mu_t = \Lambda_t^{-1}\eta_t$ — die für Exploration nötige Varianz
entsteht durch Regularisierung und zeitliches Abklingen, **ohne** aktiven `Rng`-Portaufruf im Ring-0-Hot-Path
(erfüllt P28 strenger als UCB/Thompson-Sampling, die explizites Stochastik-Sampling benötigen).

#### 10.4.2 SnapKV / GemFilter: Attention-Guided Eviction

Kumulierte Aufmerksamkeit eines Beobachtungsfensters $W$ am Prompt-Ende identifiziert „Heavy Hitters":

$$S_j^{(l)} = \sum_{i=N-W}^{N} A_{i,j}^{(l)} \quad \text{für } j \in \{1,\dots,N-W\}$$

Token mit niedrigstem $S_j$ werden aus dem KV-Cache entfernt — Speicherkomplexität $O(N) \to O(K)$.
GemFilter/PromptDistill beschleunigen die Evaluierung zusätzlich (bis zu 30 % VRAM-Reduktion gemessen).
Betrifft `EvictionWorker`/`KvReusePolicy` in `contextra-kvcache::prefix_store.rs`.

#### 10.4.3 KIVI: asymmetrische 2-Bit-KV-Quantisierung

Key-Tensor (kanalweise Ausreißer) wird **per-channel** skaliert, Value-Tensor (token-basierte Ausreißer)
**per-token**:

$$s_c^{(K)} = \frac{\max_i K_{i,c} - \min_i K_{i,c}}{2^b-1}, \qquad
s_i^{(V)} = \frac{\max_c V_{i,c} - \min_c V_{i,c}}{2^b-1}$$

Neue Struktur `KiviQuantizedBlock` in `segment.rs`: Key-Faktoren im Block-Header, Value-Faktoren inline,
AEAD-Verschlüsselung **nach** Quantisierung.

#### 10.4.4 Mental State Graphs — Belief Updates gegen Präferenz-Veralterung

HorizonBench-Befund: LLMs greifen bei Präferenzänderungen über lange Zeiträume zu > 33 % auf veraltete
Distraktoren zurück. Gegenmaßnahme: Präferenzen als **zeitlich abklingende n-äre Hyperkanten** (§7) statt
als flacher Vektor-Kontext — Konsolidierung (§12) hält den Graphen aktuell statt unbegrenzt wachsend.

#### 10.4.5 Contrastive Hidden-State Steering + Deterministisches Fail-Recovery

Bei sprunghaften Eingaben wird die Distanz zwischen „benignem" und „adversariellem/abdriftendem"
Hidden-State-Vektor in mittleren Layern überwacht. Fällt sie unter einen kritischen Wert, triggert
`contextra-agent`s State-Machine (Idle → Running → Checkpointing → Auditing, WAL-First) ein
deterministisches Rollback auf den letzten gesicherten Zustand; `contextra-router` leitet die Anfrage im
Hintergrund an ein alternatives SLM-Profil oder abstrahiert den Kontext über den RRF-Fusionsplaner. Der
Abbruch wird in der DLQ protokolliert, für den Nutzer bleibt der Dialog stabil und unsichtbar unterbrochen.

#### 10.4.6 TenantPrefixKvStore — kryptographische Scope-Isolation

`PrefixKey`-Strukturen führen `TenantId` und `DocId` als BLAKE3-Hashes; ein Scope-Wechsel erzwingt einen
Pfadwechsel im Radix-Baum ($O(1)$-Umschaltung), Vektor-Leakage ist damit systemisch ausgeschlossen.

### 10.5 AES-Schlüsselverwaltung — kein globaler Zustand (P29)

```rust
pub struct KeyManager { ciphers: scc::HashMap<KeyId, OnceCell<Aes256GcmSiv>> /* Instanzzustand, nicht static */ }
impl KeyManager { pub fn cipher_for(&self, key_id: KeyId) -> Result<&Aes256GcmSiv, CryptoError>; }
```
Nonce-Strategie: 4-Byte-Präfix je Schlüssel + 8 Byte `OsRng`, bewusst ohne persistierten Zähler
(Kollisionswahrscheinlichkeit $\approx 2^{-32}$ bei $\approx 9\cdot10^4$ Nachrichten/Schlüssel);
AES-256-GCM-SIV ist nonce-misuse-resistent. Schlüsselrotation über gezielten Austausch des
`OnceCell`-Eintrags je `KeyId`.

---

## 11. Konsolidierung (`contextra-cognition`) — dreistufige Kognitionspipeline

1. **Struktureller Pass** (deterministisch, LLM-frei) — `memory_consolidation.rs`.
2. **Generative Synthese** (LLM-Kostenschutz `max_llm_calls_per_cycle`, `CommunityStabilityTracker`) —
   `synthesis_phase.rs`.
3. **Semantische Aggregation** (🔴, §13.4 LeanRAG) — Gaussian-Mixture-Clustering mit deterministischem Seed.

Baut aus Interaktionen einen wartbaren, alternden Wissensgraphen statt eines unbegrenzt wachsenden
Vektor-Haufens — struktureller Unterschied zu einfachem RAG.

```rust
pub struct ConsolidationPipelineResult {
    pub structural: ConsolidationPhaseResult,
    pub synthesis: SynthesisPhaseResult,          // vereinheitlichter Typ (keine Doppeldefinition)
    pub aggregation: Option<AggregationPhaseResult>,  // None, falls Stufe 3 deaktiviert/nicht erreicht
}
```

---

## 12. SOTA-Algorithmen-Erweiterung (normativ, Phase 2 — Priorisierung siehe §22.5)

Alle vier Verfahren sind `[Phase 2]` (Hot-Path/Struktur-Stufe) und dürfen erst nach grünem Gate 0/1 für den
jeweiligen Crate begonnen werden.

### 12.1 TL-HFD — Thresholded Local Hyper-Flow Diffusion (`contextra-graph`)

Ersetzt den heuristischen ε-schwellenwertbasierten `forward_push_ppr` für n-äre Hyperkanten durch einen
formal exakten, projizierten Subgradientenabstieg auf der Lovász-Erweiterung der Hyperkanten-Schnittkosten,
mit **hart** (nicht nur statistisch) begrenzter Lokalität:

$$\min_x F(x) := \tfrac12\sum_{e\in E}\theta_e f_e(x)^2 + \tfrac{\sigma}{2}x^\top D x - \langle \Delta-d, x\rangle$$

Aktive Region $A(t)$ plus Ein-Hop-Grenze $\partial A(t)$; Subgradient nur dort ausgewertet. Integration als
neue Variante `PprAlgorithm::TlHfd(TlHfdParams)` (nicht als isolierte Methode), zunächst ausschließlich über
`ShadowMode { compare_against: TlHfd(..) }`, Default-Flip erst nach Diskrepanz-Log-Auswertung.
**Deterministischer Cutoff** `max_hyperedge_sort_size` statt stochastischer Stichprobe bei sehr dichten
Kanten ($|e| > 1000$) — vermeidet einen neuen `Rng`-Port in einem bislang RNG-freien Algorithmus.

### 12.2 DiBud — Direct Budgeting für deterministische RRF-Präfixe (`contextra-rank`)

Ersetzt den blockierenden Vollabruf fester Top-$k$-Kandidatenmengen durch inkrementelle, budget-gesteuerte
Fusion mit beweisbar exaktem RRF-Präfix. **Zweistufig gegatet:** Schritt 1 (Vorbedingung) verlangt, dass
`contextra-vector` und `contextra-text` je einen `impl Iterator<Item = DocId>` exponieren, der Kandidaten
batchweise (16 pro Kanalzugriff) nachliefert, statt intern vollständige Ergebnismengen zu berechnen — ohne
diese Vorarbeit liefert DiBud **keinen** P24-Gewinn.

```rust
pub fn fuse_exact_prefix<I1, I2, I3>(vector_stream: &mut I1, text_stream: &mut I2, graph_stream: &mut I3,
    edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget) -> Result<Vec<DocId>, ErrorClass>
where I1: Iterator<Item = DocId>, I2: Iterator<Item = DocId>, I3: Iterator<Item = DocId>;
```
Harte Abbruchbedingung `current_accesses >= max_total_accesses`; Tie-Breaker-Priorität: Graph → Text →
Vector → totale `DocId`-Ordnung.

### 12.3 FC-TS — Flow-Corrected Thompson Sampling (`contextra-adapt`)

Erweitert die Drift-Bandit-Kopplung (§9.3) um eine **richtungsbewusste** Korrektur statt pauschaler
Diskontierung der gesamten Präzisionsmatrix: historische Beobachtungen werden über einen expliziten
Transport-Operator in die Gegenwart korrigiert. Bayesianisches Modell je Arm:
$w_t^{(a)} \sim \mathcal N(\mu_t^{(a)}, (\Lambda_t^{(a)})^{-1})$, transportierte Belohnung
$\hat r_{s\to t} = r_s + (t-s)\langle\hat\delta_t^{(a)}, x_s\rangle$. Integration als dritte Variante von
`BanditImplementation` (neben `ShermanMorrison`/`DiagonalApproximation`), nicht als isolierter Typ, damit
die bestehende Off-Policy-Evaluation (§9.4) unverändert weiterverwendet wird. `drift_rate` wird im
Ring-0-Hot-Path nur **lesend** verwendet; die Rekalkulation über `drift_window` (feste Kapazität, kein
dynamisches Wachstum) läuft ausschließlich im asynchronen Ring-3-Hintergrund-Task.

### 12.4 LeanRAG Semantic Aggregation — dritte Pipeline-Stufe (`contextra-cognition`)

**Höchste Priorität der vier Verfahren** (größter Teil der Infrastruktur bereits vorhanden).
**Vorbedingung 0 (hart blockierend):** `child_edge_ids` (§7.2) und die rekursive H5-Cascade-Erweiterung
MÜSSEN gemergt und CI-grün sein, bevor irgendein Teil dieser Stufe produktiv aktiviert wird — ohne diese
Erweiterung kann ein DSGVO-Löschauftrag einen konsolidierten Super-Knoten nicht erreichen.
**Vorbedingung 1:** keine dritte, kollidierende `ConsolidationConfig` — neuer Typ `AggregationConfig` im
selben Namensmuster wie `SynthesisConfig`.

Pipeline: (1) Self-Supervised Type-Denoising, (2) deterministisches Gaussian-Mixture-Clustering
(`gmm_deterministic_seed`), (3) LLM-Synthese eines Meta-Knotens je Cluster (wiederverwendet
`max_llm_calls_per_cycle` und `CommunityStabilityTracker`), (4) Super-Hyperkante mit `child_edge_ids` bei
Konnektivität $\lambda_{j,k} > \tau$, (5) atomarer Transaktions-Commit über das bestehende
Manifest-Batch-Fsync-Muster.

```rust
pub struct AggregationConfig {
    pub max_compaction_peak_memory_mb: usize, pub clustering_tau_threshold: f32,
    pub gmm_deterministic_seed: u64, pub max_llm_calls_per_cycle: usize,
}
pub struct AggregationPhaseResult {
    pub raw_edges_tombstoned: usize, pub abstract_hyperedges_created: usize,
    pub peak_memory_used_mb: usize, pub child_edge_ids_written: usize,
}
impl Collection {
    pub async fn consolidate_semantic_hyperedges(&self, embedder: &dyn Embedder, llm: &dyn TextGenerator,
        config: &AggregationConfig) -> Result<AggregationPhaseResult, ErrorClass>;
}
```
Budget-Vorprüfung ruft die bereits vorhandene `estimate_compaction_peak_bytes()`-Primitive auf (keine neue
Speicherbudget-Logik). Läuft als `async`-Task in Ring 3, während aufgerufene `contextra-graph`-Operationen
(Ring 0) synchron bleiben (P26).

---

## 13. Automatische Wissensgraph-Konstruktion (🔴, neue Kernlücke)

Contextra hat aktuell **keine** automatische Entity-/Relation-Extraktion aus Text — jede Kante muss manuell
über `relate()`/`relate_n_ary()` gesetzt werden (im Produkt selbst als einzige klar 🔴-markierte Kernlücke
geführt). Zielarchitektur:

| Feature | Ziel-Crate | Ansatz |
|---|---|---|
| LLM-gestützte Open-Information-Extraction beim `insert()` | `contextra-engine` (Ingestion-Pfad) | Nutzt bestehenden `TextGenerator`-Port, kein neues externes Modell nötig |
| Automatische Hyperkanten-Vorschläge aus Co-Occurrence + LLM-Validierung | `contextra-graph` (Hyperedge-Builder) + `contextra-cognition` | Wiederverwendung von `max_llm_calls_per_cycle` als Kostenschutz (§12.4), kein neuer Mechanismus |

Diese Erweiterung ist eine reine Ports-Kompositionsaufgabe (kein neues externes Modell, keine neue
Trainingsinfrastruktur) und schließt die einzige im Produkt selbst dokumentierte Kernfunktionslücke.

---

## 14. Sicherheits- und Datenschutzmodell

### 14.1 Löschbeweis (`DeletionProof`, offen im `sovereign`-Ring)

Ein rein symmetrisch (HMAC) konstruierter Löschbeweis ist Selbstattestierung — wer prüfen kann (den
Schlüssel besitzt), kann auch fälschen. Zielarchitektur:

```rust
pub struct DeletionProof {
    pub deleted_keys_hash: [u8; 32],   // Längenpräfix-Hash über sortierte Schlüsselmenge (kollisionssicher)
    pub hmac_chain_entry: [u8; 32],
    pub prev_hmac: [u8; 32],
    pub timestamp: i64,
    pub signature: [u8; 64],           // Ed25519 über die vorstehenden Felder
    pub signer_key_id: [u8; 16],
    pub signature_version: u8,         // 3 = Ed25519; 1/2 = legacy HMAC, weiterhin verifizierbar
}
impl DeletionProof {
    /// Reine Funktion des öffentlichen Schlüssels — ohne Zugriff auf interne WAL-HMAC-/AEAD-Schlüssel.
    /// Schlüsseltrennung: Kompromittierung des Signing-Keys legt nie den Daten-Schlüssel offen.
    pub fn verify_external(&self, signer_public_key: &ed25519_dalek::VerifyingKey) -> Result<(), CryptoError>;
    pub fn create_with_wal_receipt(/* … */) -> Result<Self>;
    pub fn export_for_audit(&self) -> Result<String>;
}
```

**Schnittstellenkette, Knopfdruck bis Beweis:**
```
MCP contextra_forget → Collection::drop_collection()|dokumentspezifischer Löschpfad
   → StorageEngine::Write-Pfad (Tombstone, WAL) → DeletionProof::create_with_wal_receipt(...)
   → DeletionProof::verify(proof_key) → DeletionProof::export_for_audit()
```
**Bedingung:** Dies ist nur ein Beweis, keine Behauptung, wenn `verify()` ohne laufenden Contextra-Prozess
ausführbar ist (reine Funktion über `proof_key` + exportiertes Dokument).

**Crypto-Shredding (Zielarchitektur):** eigener Data-Encryption-Key (DEK) pro Mandant/Betroffenem;
Löschung = physische Tombstone-Löschung **und** DEK-Vernichtung. Der Löschbeweis belegt nur die lokale
Schicht (LSM/WAL/Graph/KV-Cache) — nicht Backup-/Wear-Leveling-Kopien und nichts über Daten, die ein
externer LLM-Provider empfangen hat (dafür ausschließlich das Privacy-Gateway, §14.4).

### 14.2 WASM-Sandbox (offen, `fast`-Ring-kompatibel)

```rust
pub struct WasmCapabilities { pub max_fuel: Option<u64>, pub max_wall_clock_ms: u64, pub allow_cloud_egress: bool }
impl SandboxExecutor { pub fn execute(&self, module: &[u8], caps: &WasmCapabilities) -> Result<Vec<u8>, SandboxError>; }
```
Fuel-Budget und Wall-Clock-Budget (Default 5 s) sind **orthogonal**. `contextra-sandbox` ist der **einzige**
Pfad, über den `McpServer::call_tool` Aufrufe zulässt. Jede neue Werkzeugintegration MUSS über diesen Pfad
laufen — ein direkter Aufruf von Speicher-/Netzwerkoperationen unter Umgehung der Sandbox ist ein
Sicherheitsdefekt, kein Style-Verstoß. Dies ist die konkrete Antwort auf „sichere Mensch-zu-Agent-
Interaktion": Freigabe-Oberflächen für Agentenaktionen statt Blind-Trust.

### 14.3 Prompt-Injection-Abwehr (`contextra-mcp::prompt_injection`)

Jedes Suchergebnis durchläuft `injection_guard.process_result` vor Rückgabe. Pflichtfeld
`content_provenance: "retrieved_untrusted_data"` in jeder Suchantwort — Downstream-Agenten-Prompts MÜSSEN
dieses Feld auswerten und abgerufene Inhalte nie als Systemanweisung interpretieren. Lesepfad durchgehend
Zero-Copy, um Pufferkopien sensibler Daten zu vermeiden.

### 14.4 Privacy-Gateway (`contextra-privacy`, `sovereign`-Ring, 5 Schichten)

1. **Token-Vaulting/Pattern-Matching** (`EgressVault`, `RegexSet`-Klassifikation vor jedem Cloud-Egress).
2. **Vorabstraktion.**
3. **Graph-Generalisierung** vor Versand.
4. **Bulk-Exfiltration-Detektor** (`BulkExfiltrationDetector`, großvolumige Anfragemuster).
5. **Re-Hydration** (`CloudResponseRehydrator::rehydrate`, round-trip-sicher, Multibyte-UTF-8-panic-sicher).

```rust
pub struct EgressVault { /* pattern_matcher: RegexSet, surrogate_map: scc::HashMap<SurrogateToken, OriginalEntity> */ }
impl EgressVault {
    pub fn generate_surrogate(&self, entity: &OriginalEntity, session: SessionId) -> SurrogateToken;
    pub fn get_entity(&self, token: &SurrogateToken) -> Option<OriginalEntity>;
}
```
Strukturell das isolierteste, am schnellsten marktreife Verkaufsargument: kein LLM-Provider-Wrapper kann
eine Kontrolle vor sich selbst einbauen.

### 14.5 Mandanten-Scoping innerhalb einer Instanz (`compliance`-Ring)

„Kein Multi-Tenant-Enterprise-System" bleibt Architektur-Nicht-Ziel — keine verteilte,
netzwerkexponierte Mehrmandanten-Instanz. Aber: logische Trennung **innerhalb eines
Single-Tenant-Prozesses** für eine Kanzlei mit mehreren Mitarbeitern. Erweiterung von `contextra-privacy`
und `contextra-crypto`: Mandanten-Scoping auf Collection-/Key-Präfix-Ebene, eigener DEK pro Mandant
(Voraussetzung für Crypto-Shredding), Rollen-/Rechte-Modell. `TenantId` existiert bereits — diese
Erweiterung härtet eine vorhandene Primitive, erfindet keine neue.

### 14.6 Lizenz- und Aktivierungsschicht (`contextra-license`, 🔒 `compliance`-Ring) <!-- crate-ref-ignore -->

- Signierte Lizenz-Tokens (Ed25519 — derselbe Mechanismus wie der Löschbeweis, kein zweiter kryptographischer
  Grundbaustein).
- Offline-Aktivierung (Kanzlei-/Praxis-Rechner mit Mandantendaten haben oft kein Internet).
- Hardware-Fingerprint-Bind ohne Telemetriepflicht.
- Signierte, notarisierte Binaries mit signaturgeprüftem Update-Kanal.

---

## 15. Lizenz-/Feature-Ring-Modell und rechtlicher Schutz

### 15.1 Die drei Ringe

```toml
[features]
default    = ["fast"]
# Ring "fast" — MIT/Apache-2.0, für jeden Nutzer offen. Vektor+Text+Graph-Retrieval, Bandit-Routing,
# lokale Inferenz. Kein Krypto-Overhead im Hot-Path.
fast       = []
# Ring "sovereign" — Code bleibt offen (Vertrauensargument: ein Löschbeweis, den niemand außer dem
# Hersteller lesen kann, ist Selbstattestierung). Aktivierung ist opt-in wegen Latenzkosten, nicht Geheimhaltung.
sovereign  = ["contextra-crypto", "contextra-privacy", "deletion-proof"]
# Ring "compliance" — Quellcode verlässt das Haus nie (Betriebsgeheimnis). Vertrieb nur als signiertes
# Binary/Appliance. Reine Reporting-/Lizenz-/Aufbereitungsschicht über bereits offenen Belegen aus "sovereign".
compliance = ["sovereign", "audit-export", "avv-generator", "license-gate", "multi-tenant-scoping"]
```

**Warum dieser Schnitt:** Der Löschbeweis selbst (die kryptographische Grundwahrheit) MUSS offen bleiben,
sonst ist er wieder Selbstattestierung. Geschlossen bleibt ausschließlich die **Aufbereitung**: Compliance-
Reports, AVV-Generierung, Lizenzdurchsetzung, Mandanten-Rollenmodell — der einzige Schnitt, der trotz
schwacher Copyright-Position an KI-generiertem Code wirtschaftlich verteidigungsfähig ist.

### 15.2 Entkopplung im Code (kleinstmöglicher Diff)

```toml
# contextra-store/Cargo.toml, contextra-engine/Cargo.toml — je gleiches Muster
[dependencies]
contextra-crypto = { workspace = true, optional = true }
[features]
default = []
encryption-at-rest = ["contextra-crypto"]
```
Speicherpfade (WAL, SSTable, KV-Cache-Segmente) erhalten einen `#[cfg(feature = "encryption-at-rest")]`-
Zweig mit Klartextpfad als Default (Compile-Time-Entscheidung, keine Laufzeitverzweigung). Ein deaktiviertes
`deletion-proof`-Feature gibt `Ok(())` **ohne** Proof zurück — nie einen leeren oder vorgetäuschten Proof.

### 15.3 Rechtlicher Schutzmechanismus für den `compliance`-Ring

KI-generierter Code ist nach § 2 UrhG schwer urheberrechtlich durchsetzbar. Drei unabhängige,
belastbarere Mechanismen:

1. **Betriebsgeheimnis (GeschGehG).** Quellcode verlässt das Haus nie; Vertrieb nur als kompiliertes,
   signiertes Binary oder betriebene Appliance/SaaS.
2. **Vertragsrecht, technisch durchgesetzt.** EULA pro Installation über `contextra-license`. <!-- crate-ref-ignore -->
3. **Marke.** „Contextra" bei DPMA/EUIPO anmelden **vor** kommerziellem Vertrieb, Namenskollision vorab
   prüfen.

---

## 16. Compliance-Produktschicht (🔒 `compliance`-Ring, reine Aufbereitung vorhandener Belege)

Kein Punkt dieser Schicht erfordert eine neue kryptographische oder algorithmische Grundwahrheit:

1. **Automatisiertes Verarbeitungsverzeichnis (Art. 30 DSGVO)** — generiert aus tatsächlichen
   Privacy-Gateway- und Löschbeweis-Logs; Kompositionsschicht über `DeletionProof::export_for_audit`,
   `SegmentSynthesizer::model_id`, `CloudQueryRequest`/`CloudQueryResponse`, `MetricsSink` — Knopfdruck
   statt manueller Pflege.
2. **BSI-Grundschutz-/TR-02102-Mapping** als Produktbestandteil — jede Krypto-Primitive im Code
   referenziert die passende Technische Richtlinie im Audit-Export.
3. **AVV-Vorlage** als generiertes Dokument, das die technischen Garantien des Produkts referenziert statt
   generischer Boilerplate.
4. **Agent Decision Log** — pro Tool-Aufruf ein exportierbares, signiertes Protokoll aus der Sandbox-/
   Audit-Engine, lesbar durch Anwalt/Auditor ohne Rust-Kenntnisse. Adressiert EU-AI-Act-
   Protokollierungspflicht (Art. 12) und menschliche Aufsicht (Art. 14) als Verkaufsargument.
5. **Signierte, reproduzierbare Builds** (`cargo-vet`, SBOM, Reproducible-Build-Nachweis) — für Behörden/
   Kanzleien oft härteres Kriterium als Feature-Umfang.
6. **Deutsche Fachterminologie** — juristische/medizinische Vokabulare als Domänenpakete auf der
   bestehenden BM25F-Basis (§8.3), nicht als neue Engine.
7. **Explainability-Oberfläche** („warum wurde dieses Dokument abgerufen") aus bereits vorhandenen
   Provenance-/Kalibrierungsdaten der 4-Signal-Fusion (§8.7) und `ResponseGroundingValidator::score_grounding`.
8. **Backup/Restore mit Aufbewahrungsnachweis** — für GoBD (10 Jahre) und berufsrechtliche Fristen.
9. **Mandanten-Scoping** (§14.5) und **Lizenz-/Aktivierungsschicht** (§14.6) als Bestandteil dieser Schicht.

---

## 17. Fehlertaxonomie (crateübergreifend, verbindlich)

Jeder Crate exportiert genau einen (oder wenige klar abgegrenzte) `thiserror`-Fehlertyp. Kein öffentlicher
Rückgabetyp ist `Box<dyn std::error::Error>`. Die oberste Konsumentenschicht (`contextra`/`contextra-engine`)
bündelt alle Unterfehler verlustfrei per `#[from]`/`#[error(transparent)]`.

| Crate | Fehler-Enum |
|---|---|
| `contextra-types`/`contextra-ports` | `ContextraError` (Wurzel, `#[non_exhaustive]`) |
| `contextra-store` | `StoreError` (bettet `WalError`, `LockError`) |
| `contextra-crypto` | `CryptoError` |
| `contextra-vector` | `IndexError` |
| `contextra-graph` | `GraphMutationError`, `GraphError` |
| `contextra-adapt` | `BanditError` |
| `contextra-sandbox` | `SandboxError` |
| `contextra-privacy` | `EgressError` |
| `contextra-mcp` | `McpError` (einzige Fehlerform an der Produktgrenze) |
| `contextra-engine`/`contextra` | `DbError`/`ErrorClass` (bündelt alle Unterfehler) |

Fehler, die eine Sicherheits- oder Compliance-Zusage betreffen (`CryptoError::InvalidProofSignature`,
`SandboxError::FuelExhausted`), sind eigene Varianten, nie generische `Other(String)`-Fälle — jede
Compliance-Aussage muss auf einen typisierten Fehler zurückführbar sein, sonst ist sie im Audit-Export
nicht maschinenlesbar.

---

## 18. Stabilität, Versionierung, Feature-Flags

### 18.1 SemVer-Geltung pro Schicht

| Schicht | Breaking-Change-Schwelle |
|---|---|
| 4 (Produktgrenze) | MAJOR, nur mit Migrationspfad für bestehende MCP-Clients |
| 3 (Fassade) | MINOR bis MAJOR je nach Downstream-Nutzung |
| 2 (Ports) | MAJOR, IMMER — jede Trait-Änderung propagiert in alle Adapter |
| 1 (Domänentypen) | MAJOR bei Feldänderung, MINOR bei additiver `#[non_exhaustive]`-Enum-Variante |

### 18.2 Serialisierung

Produktgrenze (MCP): JSON über `serde_json::Value`. Persistenz/IPC: FlatBuffers, additive
Feldnummerierung. Bewusst getrennt — JSON priorisiert Lesbarkeit, FlatBuffers Zero-Copy-Performance; keine
Vereinheitlichung anzustreben.

### 18.3 Feature-Flag-Katalog (Auszug, verbindlich)

| Feature-Flag | Reifegrad | Beschreibung |
|---|---|---|
| `cloud-egress-guard` | 🟢 | DLP/Egress-Kontrolle, Surrogat-Tokenisierung |
| `bandit-routing` | 🟢 | LinUCB-Grundfunktion, Lyapunov-Kopplung |
| `egress-sherman-morrison` | 🟡 | Mathematisch korrekte Ridge-Regression |
| `kv-bridge` | 🟢 | KV-Cache-Bridge inkl. LSM-Fallback, AES-256-GCM-SIV |
| `wasm-sandbox` | 🟢 | Fuel-/Wall-Clock-Budget |
| `experimental-diskann` | 🟢 (Tier) | Native Tombstones, SQ8-Perzentil-Clipping |
| `docid-128` | 🟡 | 128-Bit-BLAKE3-DocId, Default bleibt `u64` |
| `block-cache-v2` | 🟡 | SIEVE-Backend, Default bleibt LRU |
| `bm25f` | 🟢 | Feldgewichtete BM25-Bewertung |
| `flatbuffers-drift-gate` (xtask) | 🟢 | CI-Gate gegen Schema-Drift |
| `edge-reinforcement-learning` | 🟢 (Gate) | Kantenverstärkung als optionales Fusionsverhalten |
| `encryption-at-rest` | 🔴 | Krypto-Entkopplung (§15.2) |
| **Hyperkanten** (`relate_n_ary`, `HyperEdge`, `child_edge_ids`) | 🔴 | Kernerweiterung, kein Flag — §7 |
| **ACORN** (`search_knn_acorn`) | 🔴 | §8.5 |
| **KIVI/SnapKV/RIE-Greedy** | 🔴 | §10.4 |
| **TL-HFD/DiBud/FC-TS/LeanRAG** | 🔴 `[Phase 2]` | §12 |
| **Automatische KG-Konstruktion** | 🔴 | §13 |
| **`contextra-infer-onnx`** | 🟢, aber aus `default-members` ausgeschlossen | Kein Netzwerk-Download im Standard-Build |

---

## 19. Betriebsmodi und Deployment-Formen

| Form | Ring | Zielgruppe |
|---|---|---|
| MCP-Server (`uvx contextra-mcp`) | `fast` | Claude Desktop, Cursor, beliebige MCP-Clients |
| Python-Bibliothek (`pip install contextra`) | `fast`, optional `sovereign` | In-Process-Einbettung in Python-Agenten |
| Rust-Crate (`cargo add contextra`) | `fast`, optional `sovereign` | Native Rust-Anwendungen |
| On-Prem-Appliance (signiertes Binary + Aktivierungsschlüssel) | `fast`+`sovereign`+`compliance` | Kanzlei/Praxis/Mittelstand |
| SaaS-Aufsatz (Explainability-UI, Audit-Export) | `compliance` | Bestehender Appliance-Kunde, Phase 3–4 |

Es gibt in keiner Form einen Server-Modus mit Netzwerk-Listener für Multi-Tenant-Zugriff über
Instanzgrenzen hinweg. Der Privacy-Gateway-Pfad ist der einzige Punkt, an dem Daten das lokale System
verlassen — ausschließlich auf explizite Anforderung, nie als Hintergrundtelemetrie.

**Deployment-Checkliste:** Datenspeicher-Pfad mit ausreichend Speicher (≥ 10 GB empfohlen) · Ollama oder
Candle/ONNX-Backend konfiguriert · Feature `sovereign` aktiviert, wenn Verschlüsselung/Löschbeweis benötigt
wird · Embedding-Provider-Umgebungsvariablen gesetzt · Max-Memory-Budget gesetzt · für
Compliance-Nutzung: `deletion_proof`-Schlüsselpaar sicher aufbewahren.

**Performance-Zielwerte:** KV-Cache-Tier-1-Treffer < 100 µs · HNSW-Suche (10k Dokumente) < 5 ms ·
BM25F-Suche (10k Dokumente) < 10 ms · 4-Signal-Fusion < 50 ms · WAL-Commit (Group) < 1 ms ·
Löschantrag → verifizierbarer Beweis (vierte Kennzahl, §1).

---

## 20. Normative Typen, IDs, Invarianten, Grenzwerte

### 20.1 ID-Typen

```rust
DocId(u64)          // BLAKE3(key)[0..8], Kollisionscheck bei Insert
EntityId(u64)        // BLAKE3(key)[0..8] oder parse-as-u64
TxId(u64)            // dreigeteilter Wertebereich: Collection | Wall-Clock-Gap | System
CollectionId(u64)    // try_new(0) → Err
TenantId(u64)        // SYSTEM = 0, try_new(0) → Err (INV-TENANT-1)
HyperEdgeId(u64)
RoleId(u32)
```

### 20.2 Kritische Invarianten (Verletzung = Bug)

| Invariante | Beschreibung |
|---|---|
| INV-TENANT-1 | `TenantId(0)` = SYSTEM, `try_new(0) → Err` |
| INV-DELETION-1 | `DeletionProof::create()` NUR nach physischer Bereinigung |
| INV-P8-1 | Bei `SlmProfile`-Fingerprint-Wechsel: `invalidate()` aufrufen |
| INV-WAL-FIRST | Persistenz erst nach `fsync` sichtbar |
| INV-DAG-DEPS | Ring-N importiert nie Ring N+k rückwärts (§4-Matrix) |
| INV-SINGLE-ERROR | Nur eine Fehler-Enum-Wurzel im Workspace |
| INV-LOCK-ORDER | `collections → kv_locks → embedder` |
| INV-ZERO-PANIC | Keine `unwrap`/`expect`/`panic` in `src/` |
| INV-TXID-RANGE | `TxId` muss gültigen Ursprung haben (`is_valid_origin()`) |
| INV-HASHER-SEEDS | `KvKeyLocks`-Hasher mit festen Seeds, nie `RandomState::new()` |

### 20.3 Maximale Grenzwerte

```rust
pub const MAX_SEARCH_K: usize = 1000;
pub const MAX_SEARCH_QUERY_BYTES: usize = 64 * 1024;
pub const MAX_RELATE_PARTICIPANTS: usize = 64;
pub const MAX_SCAN_MERGE_ACCUMULATOR: usize = 100_000;
pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;
pub const CASCADE_BATCH: usize = 128;
pub const DEFAULT_WAL_QUEUE_CAPACITY: usize = 1_024;
```

---

## 21. Systeminvarianten-Nachweis, Test- und CI-Pflichten

- **Unit-Tests:** jede öffentliche Funktion mit nicht-trivialer Logik erhält Normalfall-, Grenzfall- und
  Fehlerfall-Test (konkretes `Result::Err`-Enum-Mitglied, kein pauschales `is_err()`).
- **Loom-Tests** (`#[cfg(loom)]`): Group-Commit-Atomizität, Multi-Key-Deadlockfreiheit,
  Hyperkanten-Deadlockfreiheit — MÜSSEN als eigener, sichtbar grüner CI-Job laufen.
- **Differential-Testing** gegen battle-tested Referenzimplementierungen (Store gegen `redb`, Vektor/Text
  gegen `tantivy`/`usearch`) für Tier-0/1/2-Komponenten.
- **Property-Testing:** In-Memory-Modell + `proptest`-State-Machine über
  `put/delete/get/compact/checkpoint/crash`-Sequenzen.
- **Crash-/Fault-Injection:** Failpoint-Injektion an jedem `write`/`fsync` im WAL-Pfad, Neustart,
  Invariante prüfen.
- **Mutation-Testing** über alle Tier-0/1/2-Crates.
- **CI-Pflicht-Jobs:** `flatbuffers-drift-gate` · `hyperedge-schema-merge` (`needs: [flatbuffers-drift-gate]`)
  · `check-bandit-latency-budget` · `loom-tests` · `clippy-panic-lints` (`-D warnings` gegen die
  Zero-Panic-Lint-Konfiguration).
- **GitHub Branch Protection** auf `main` für alle Tier-0/1-Kern-Crates
  (`contextra-store`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-engine`,
  `contextra-mcp`); CODEOWNERS-Pflicht-Review für `contextra-crypto/**`/`contextra-store/**`; CI-Actions auf
  Commit-SHA gepinnt.
- **Coverage-Ziel differenziert nach Tier:** Tier 0/1 mindestens 85 % Line-Coverage, mindestens 70 %
  Mutation Score; Tier 2/3 bewusst niedriger.
- **Zweites-Modell-Regel:** Property-Tests und Fuzz-Targets werden von einer zweiten Session/einem zweiten
  Modell geschrieben, nie von derselben Session, die den zu prüfenden Code gebaut hat.

---

## 22. Verifikations- und Härtungsplan (blockierend für §7–§16)

### 22.1 Priorisierungsprinzip

Verifiziert wird nach Schadenspotenzial eines unentdeckten Fehlers, nicht nach Zeilenzahl:

| Tier | Fehlerklasse | Komponente | Konsequenz |
|---|---|---|---|
| 0 | Tatsachenbehauptung nach außen, falsch belegt | Löschbeweis (`contextra-crypto`) | Rechtliches/Vertrauensrisiko |
| 1 | Datenverlust | WAL/LSM (`contextra-store`), Checkpoint | Irreversibel, betrifft jeden Nutzer |
| 2 | Stille Falschergebnisse | Vektorsuche, BM25, Fusion/Ranking | Vertrauen erodiert langsam |
| 3 | Qualität/Performance | Graph, Router, Adapt/Bandit | Unangenehm, nicht katastrophal |

### 22.2 Tier-0/1-Gate (harte Blockade)

**Tier 0 — Löschbeweis:** Ed25519-Migration (additiv zu Legacy-Versionen), Längenpräfix-Fix (Property-Test:
`hash(A) ≠ hash(B)` für beliebige `A ≠ B`), externes Kryptographie-Review (höchste Hebelwirkung — der
einzige Punkt, an dem Selbstprüfung durch dasselbe Modell, das den Fehler gebaut hat, nicht ausreicht),
Fuzzing des Signatur-Verifikationspfads inkl. Negativtest.

**Tier 1 — Store/WAL/Checkpoint:** Differential-Testing gegen `redb`, `proptest`-State-Machine,
Failpoint-Injektion, `loom`-Nachweis für Group-Commit-Lock-Handoff, Benchmark gegen dieselbe Referenz.

**Erst danach:** Tier 2 (Vektor/Text — Brute-Force-Recall-Ground-Truth, Differential gegen
`tantivy`/`usearch`) und Tier 3 (Graph/Router/Rank/Adapt — Konvergenz-/Stabilitätsnachweise).

### 22.3 Governance

Eine Absichtserklärung ist kein Gate — alle Punkte in §21 sind technisch erzwungen, nicht formuliert.

### 22.4 Abnahmekriterien (Auszug, verbindlich für jeden Merge)

- Kein Merge in einen Tier-0/1-Crate ohne grünes Differential-, Property- und Fault-Injection-Gate.
- Kein `sovereign`- oder `compliance`-Feature aktivierbar ohne explizit gesetztes
  `encryption-at-rest`/`deletion-proof` — kein impliziter Krypto-Overhead im `fast`-Default.
- Kein Löschauftrag gilt als abgeschlossen, ohne dass `child_edge_ids`-Traversierung bestätigt, dass keine
  konsolidierte Hyperkante das Quelldokument noch referenziert.
- Keine öffentliche Kommunikation verwendet „DSGVO-konform" ohne aktives `signature_version: 3` und
  abgeschlossenes externes Kryptographie-Review.
- Kein `compliance`-Binary wird ausgeliefert ohne gültige Signatur und funktionierenden
  Aktivierungsmechanismus.

### 22.5 Priorisierung der SOTA-Erweiterungen (§12) und KV-Cache-Backlog

Basierend auf der Code-fundierten Paper-Triage (141 → 34 implementierungswürdige Titel):

| Priorität | Bereich | Ziel-Crate | Hebel |
|---|---|---|---|
| **A** (sofort, größter Hebel) | H₂O, SnapKV, KIVI, Irminsul, C²KV, CacheGen, Scissorhands, Leyline | `contextra-kvcache` | Einzige Kategorie mit klar bestätigtem, unimplementiertem algorithmischem Fortschritt; isoliertes Crate ohne Querabhängigkeiten zu offenen Governance-Punkten |
| **B** | RIE-Greedy | `contextra-adapt` | Implizites Nutzer-Profiling ohne UI-Störung |
| **C** | LLM-Open-Information-Extraction, automatische Hyperkanten-Vorschläge | `contextra-engine`, `contextra-graph`/`contextra-cognition` | Schließt die einzige im Produkt selbst dokumentierte Kernlücke (§13) |
| **D** (beobachten, nachrangig, 22 Titel) | u. a. Query-Expansion, HPO-Methoden, EE-Net, ACON-Kontextkompression, Xetrieval-Explainability | diverse | Thematisch passend, kein akuter Blocker |

**Explizit ausgeschlossen** (kein neuer `pub`-Vertrag, unabhängig von Einzelanfragen):
Multi-Agent-Routing/-Orchestrierung (18 Titel, widerspricht `contextra-agent`-Designprinzip),
Modell-Training/-Quantisierung (22 Titel, außerhalb der Produktgrenze — Contextra konsumiert nur GGUF/ONNX),
In-Context-Learning/Generator-Verhalten (Contextra generiert nicht selbst), Cloud/Enterprise/Cluster
(8 Titel, expliziter Nicht-Ziel-Verstoß), gelernte-Embedding-KG-Reasoning (Contextra nutzt algorithmische
PPR, keine trainierten Graph-Embeddings).

---

## 23. Roadmap (kein Big-Bang, jede Phase hat ein Exit-Kriterium)

| Phase | Inhalt | Gate |
|---|---|---|
| **0** (läuft) | Tier-0/1-Härtung (§22.2), Feature-Freeze außerhalb der Kern-Crate-Liste | Löschbeweis extern verifizierbar; Crash-Konsistenz-Suite grün bei ≥ 95 % der Failpoint-Kombinationen |
| **1** | `fast`-Ring als OSS-MCP-Server veröffentlichen; Krypto-Entkopplung (§15.2); Privacy-Gateway isoliert vermarkten; KV-Cache-Priorität-A (§22.5) parallel vorbereiten | 10 echte Nutzergespräche geführt und dokumentiert |
| **1½** | SOTA-Algorithmen (§12: TL-HFD, DiBud, FC-TS, LeanRAG) im `ShadowMode`, jeweils gegatet auf Vorbedingungen | Diskrepanz-Logs ausgewertet, Default-Flip als eigene Entscheidung |
| **2** | `contextra-license` (§14.6); Compliance-Export-Schicht (§16, Punkte 1/4) als reine Reporting-Schicht; Hyperkanten/`relate_n_ary` produktiv (§7) | Ein zahlender Pilotkunde | <!-- crate-ref-ignore -->
| **3** | Ein Vertical vollständig (Vorschlag: Steuerkanzleien — GoBD passt strukturell auf WAL-/Checkpoint-Determinismus); Mandanten-Scoping (§14.5); Backup/Restore produktionsreif; automatische KG-Konstruktion (§13) | Pilotkunde referenzierbar |
| **4** | Weitere Verticals, Explainability-UI (§16 Punkt 7), LeanRAG-Aktivierung produktiv, WAL-Shipping-Backup, ACORN produktiv (§8.5), friktionsfreie Personalisierung (§10.4) vollständig | Erst nach belegtem Kernprodukt in Phase 3 |

Graph-Hyperkanten-Feinschliff über den hier spezifizierten Stand hinaus, Bandit-Router-Feinschliff über
§9/§12.3 hinaus, sowie jede Robotik-Integration bleiben bis Phase 4 eingefroren: kompiliert, unverändert —
Optionalität ohne laufende Pflegekosten, kein Verlust.

---

## 24. Explizit eingefrorene bzw. verworfene Bereiche

Kein neuer `pub`-Vertrag wird in diesen Bereichen vor Abschluss von Phase 2 angenommen, unabhängig von
Einzelanfragen: Graph-Analytik-Erweiterungen über den hier spezifizierten Umfang hinaus,
Bandit-Router-Feinschliff über §9/§12.3 hinaus, jede Robotik-Integration, jedes verteilte Cluster-/
Konsenssystem (`contextra-cluster`), jedes Multi-Agent-LLM-Orchestrierungsfeature, jede Retriever- <!-- crate-ref-ignore -->
Trainings- oder Modell-Quantisierungsfunktion.

---

## 25. Quellen dieser Synthese

- `contextra-roadmap.md` — Marktpositionierung, Feature-Ring-Ursprungskonzept, Roadmap-Phasen. <!-- crate-ref-ignore -->
- `CONTEXTRA_SPEC_2 (v4, Kanonisches Master-Dokument)` — Workspace-Layout, Port-Trait-Inventar (§22),
  normative Typen/Invarianten (§24), Betriebsmodi (§23).
- `CONTEXTRA_SPEC_UPDATED (Fassung 4, inkl. Teile A2–A4)` — Ring-Modell (§4.2), Speicher-/Graph-/Retrieval-/
  Bandit-/KV-Cache-/Sicherheitsdetails, normative SOTA-Algorithmen-Erweiterung (§21 dort → §12 hier).
- `CONTEXTRA_SPEC_v5_Open_Closed` — Lizenz-/Feature-Ring-Architektur, Verifikations-/Härtungsplan,
  Compliance-Produktschicht, Roadmap-Gates.
- `CONTEXTRA_INTERFACE_SPEC_v1` — Vier-Schichten-Interface-Modell, MCP-Tool-Kontrakte, Adapter-Tabelle.
- `CONTEXTRA_PAPER_TRIAGE_2026` — Code-verifizierte Priorisierung von SOTA-Literatur, automatische
  KG-Konstruktion als neu identifizierte Lücke.
- *„Friktionsfreie Personalisierung und autonome SLM/LLM-Interaktion in der Contextra-Architektur"* —
  RIE-Greedy, SnapKV/KIVI, ACORN, Mental State Graphs, Contrastive Steering (§10.4).
- Live-Klon `https://github.com/tfufuz1/contextra` (Root-`Cargo.toml`, `README.md`) — Verifikation des
  Workspace-Ist-Zustands gegen die Zielarchitektur.

**Ende des Dokuments. §0–§25 bilden zusammen die vollständige, in sich geschlossene
Zielspezifikation des Contextra-Endprodukts.**
