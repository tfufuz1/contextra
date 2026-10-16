---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "04"
---
## 4. Systemarchitektur: der Crate-Graph (Ring-Modell, vormals Crate-DAG)

> **Status dieses Abschnitts:** Der Layer-0–5-Crate-DAG (unten unter „4.0 Vorher" archiviert) ist **nicht mehr
> normativ**. Ab dieser Fassung gilt das in §4.2 beschriebene Ring-0–4-Modell aus Teil A2 als verbindlich. Der
> Umbau erfolgt strangler-artig gemäß §20, nicht per Big-Bang — bis ein Migrationsschritt abgeschlossen ist,
> existiert der jeweilige alte Crate als `#[deprecated]`-Re-Export.

### 4.i Systeminvarianten §4(1)–§4(7) (normativ, Fassung 2.1)

Die sieben Invarianten gelten für jeden Crate und jeden Abschnitt dieses Dokuments. Verweise der Form „§4(n)"
und „Invariante n" (auch in Anhang B) bezeichnen die hier nummerierte Invariante. Ein Entwurf, der eine
Invariante nur „im Prinzip" wahrt, gilt als verletzt.

1. **Zero-Panic-Doktrin.** In Produktionspfaden (`src/`, ohne Tests und Benchmarks) gibt es kein `unwrap`,
   `expect`, `panic!`, `unreachable!`, `unimplemented!` (außer in normativen Signatur-Stubs dieses Dokuments),
   `todo!`, keine ungeprüfte Indizierung oder Slice-Bildung mit berechneten oder externen Indizes (stattdessen
   `get(..)` plus Fehler) und keine überlaufende Größenarithmetik auf Eingabewerten (`checked_*`/`saturating_*`).
   Erzwingung: Lint-Konfiguration aus §0.4. `catch_unwind` nur an FFI-Grenzen und nur unter `panic = "unwind"`.
2. **Unsafe-Isolation.** `#![forbid(unsafe_code)]` in jedem Nicht-Insel-Crate; `unsafe` nur in den drei
   Unsafe-Inseln (§0.4), jeder Block mit `// SAFETY:`. Folge: Entwürfe, die `unsafe` brauchen (intrusive
   lock-freie Listen, `MaybeUninit`-Ringe, `crossbeam_epoch::Shared::deref`), sind außerhalb der Inseln
   unzulässig.
3. **Determinismus.** Gleiches Binary, gleiche SIMD-Dispatch-Stufe, gleiche injizierte Zufalls- und Zeitquelle
   (P28) und gleicher logischer Zustand liefern gleiche Ergebnisse (Replay, Recovery, Tests). Die
   Iterationsreihenfolge von Hash-Containern geht nie in Ergebnisse, IDs, Serialisierung oder Solver-Eingaben
   ein (sortierte Reihenfolge oder totale Ordnung mit Tiebreaker). Ein Hasher, der Lock-Zuordnung oder Ergebnisse
   bestimmt, wird **einmal** pro Instanz mit festen Seeds angelegt, nie pro Aufruf (§6.6 H2). Über SIMD-Stufen
   hinweg gilt die Toleranz des Proptest-Orakels (§15.1), keine Bitgleichheit. Es besteht kein Cluster- oder
   Replikationsanspruch (§2.4).
4. **Deadlockfreiheit.** Sperrenhierarchie (§5.1) plus kanonische Reihenfolge innerhalb einer Stufe
   (aufsteigender Shard-Index, §5.2a, H2). Kein `.await` unter einem `std`-Lock. Wer auf einer begrenzten Queue
   blockieren kann, hält keinen Lock, den die Gegenseite braucht. Nachweis: Loom (§15.3).
5. **Speicherbudget-Transparenz.** Jede Operation, die neben einem bestehenden Snapshot, Index oder Puffer einen
   zweiten aufbaut (Compaction, Rebuild, Merge), berechnet vorab Residenz **und** Spitze (capacity-basiert,
   inklusive Tabellen-Overhead; §6.3) und bricht bei Überschreitung ohne Seiteneffekt mit typisiertem Fehler
   ab. Queues und Puffer in Produktionspfaden sind begrenzt.
6. **Lokalität und beschränkte Arbeit (P24).** Die Kosten einer latenzkritischen Operation hängen von der
   Größe der anfragebestimmten Teilmenge oder einer konfigurierten Obergrenze ab, nie von der Gesamtgröße des
   Bestands. Darüber hinausgehende Arbeit wird in persistente, wiederaufnehmbare Hintergrundarbeit überführt
   (§6.6 H5).
7. **Zero-Copy.** Payloads (Bytes, Vektoren, Teilnehmerlisten) werden zwischen Ringen geteilt (`Bytes`,
   `Arc<[T]>`, mmap-Slices), nicht kopiert; eine „Sicht" darf keine Kopie erzeugen (Test per `Arc::ptr_eq` oder
   Allokationszähler). **Snapshot-Regel S1:** Ein per `ArcSwap` veröffentlichter Snapshot ist unveränderlich;
   seine Felder enthalten keine Container mit innerer Mutabilität (`scc::HashMap`, `Mutex`, Atomics mit Einfluss
   auf die Lesesemantik), sonst ist die Atomaritätsargumentation des RCU-Swaps hinfällig.

### 4.0 Vorher: der Layer-0–5-Crate-DAG (archiviert, nicht mehr normativ)

Contextra gliederte sich bis zur Vorfassung in einen mehrschichtigen Rust-Workspace mit zehn benannten Crates.
Abhängigkeiten sollten strikt abwärts verlaufen; **verifiziert wurde jedoch, dass diese Regel selbst verletzt
war** (§A2.1, P5-Δ) — der wichtigste Grund für die Ablösung durch das Ring-Modell.

| Layer | Crates | Verantwortung |
|---|---|---|
| **0** | `contextra-core-ipc-gen`, `contextra-core` | FlatBuffers-generierter IPC-Code; Kerntypen (`DocId`, `EntityId`, `TxId`, `ConfigFingerprint`), Traits, Fehlerbehandlung. | <!-- crate-ref-ignore -->
| **1** | `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-crypto`, `contextra-graph`, `contextra-checkpoint`, `contextra-rank` | Persistenz- und Indexierungs-Primitive. Sollten einander laut Spec nicht kennen — real hingen `store` und `index` beide von `crypto` ab, `candle` von `store` (§A2.1, „§4 Tabelle" widerlegt). |
| **2** | `contextra-db` | Öffentliche `Collection`-API, 4-Signal-Fusion, Multi-Step-Query-Engine, Kontext-Kompaktierung, Provenance-Tracking. Real: 26,7k LOC, > 40 öffentliche Methoden, bündelte mehrere Bounded Contexts (Datenebene, Retrieval, Kognition, Ingestion, `volatile_vault`) und hing **aufwärts** von `candle`/`ollama`/`embed` ab. |
| **3** | `contextra-infer-ollama`, `contextra-infer-candle`, `contextra-infer-onnx`, `contextra-agent`, `contextra-router`, `contextra-py` | Inferenz-Backends und Anwendungslogik. `contextra-router` war laut Spec reine Bandit-Mathematik; real enthielt er zusätzlich SLM-Profil-Routing, MCP-Dispatch und Type-State-Egress-Schutz. |
| **4** | `contextra-mcp`, `contextra-sandbox` | Externe Schnittstelle für Agenten. `contextra-sandbox` war real ein Waisen-Crate ohne Konsumenten. |
| **5** | `contextra-bench` | Benchmark-Harness. Erzwang real ein hartes `onnx`-Feature und zog dadurch Netzwerk-Downloads in jeden `--workspace`-Build. |

Diese Tabelle wird ausschließlich zur Einordnung bestehenden Codes und bestehender Diskussionen (Issues, PRs,
ADRs vor dieser Fassung) aufbewahrt. Für neue Arbeit gilt ab sofort §4.2.

### 4.1 Safety-First-Doktrin

Safe Rust ist der Standard; `#![forbid(unsafe_code)]` gilt per Default in jedem Nicht-Insel-Crate. **Jetzt: drei**
namentlich benannte Unsafe-Inseln (`contextra-sys`, `contextra-simd`, `contextra-wire`) mit `#![allow(unsafe_code)]`
auf Crate-Ebene und `// SAFETY:`-Beweiskommentar an jeder Stelle (§0.4). **Vorher** waren es sechs pauschal
benannte Ausnahme-Crates mit einer Mechanik (`forbid` + lokales `allow`), die wegen E0453 nicht kompilierbar
gewesen wäre (§A2.1, D2) — dieser Fehler wird mit dieser Fassung korrigiert, nicht fortgeschrieben.

### 4.2 Ring-Modell (Zielarchitektur v2, verbindlich)

**Leitprinzipien:** siehe §3, P26–P30, sowie P1–P25 unverändert. **Warum diese Schnitte:** (a) reine Trennung
gegen I/O, (b) Unsafe-Isolation, (c) flüchtige Abhängigkeiten (candle, ort, wasmtime, pyo3, reqwest) als
Blätter, (d) Bounded Contexts, geprüft gegen das Kriterium I/U/C/S/D (P30). Die Anzahl der Crates ist kein
Ziel für sich — sie ist eine Konsequenz der Kriterien, mit einer offenen Konsolidierungsoption (§A2.4 Nr. 4).

| Ring | Crate | Inhalt | Herkunft (vorher) | Kriterium (P30) |
|---|---|---|---|---|
| **0** (sync, kein `tokio`) | `contextra-types` | IDs (`DocId`, `DocIdx`, `TxId`, `TenantId`), `ModelFingerprint`, Filter-AST, Budgets, Importance, `ErrorClass`, Schema-Versionen, Tombstone-Semantik | `contextra-core` (Teilmenge `types`) | D |
| | `contextra-ports` | Traits (P27): `VectorIndex`/`TextIndex`/`GraphIndex`/`StorageRead` (sync), `StorageWrite`/`Embedder`/`TextGenerator`/`KvPrefixStore`/`ToolSandbox` (async, `BoxFuture`), `Clock`/`Rng`/`IdGen`/`MetricsSink`/`DriftStatusProvider` | `contextra-core` (Teilmenge `traits`), `contextra-db::DriftStatusProvider` | D |
| | `contextra-mvcc` **(neu)** | `SeqLog`, `SnapshotRegistry`, `TxBuffer`; loom-getestet | `contextra-core` (`seq_log`, `snapshot`, `tx_buffer` — in der Vorfassung des Migrationsplans nicht zugeordnet, §A2.1 D4) | C, D |
| | `contextra-wire` | FlatBuffers-Generat und Adapter, Unsafe-Insel | `contextra-core-ipc-gen` | U | <!-- crate-ref-ignore -->
| | `contextra-sys` **(neu)** | `ReadOnlyMap` (mmap), `LockedBuf` (mlock/VirtualLock), Owner-only-ACL (Win32), Unsafe-Insel | verteilt über `contextra-store`, `contextra-vector`, `contextra-db` (§0.4) | U |
| | `contextra-simd` | Distanzkernel, Laufzeit-Dispatch, Unsafe-Insel | `contextra-vector` (`distance.rs`) | U |
| | `contextra-crypto` | Schlüsselhierarchie, AEAD, WAL-HMAC-Kette, Deletion-Proof, Zeroize, Anti-Tamper | `contextra-crypto` (verschlankt: Egress-Vault und KV-Segment wandern nach `privacy`/`kvcache`) | C |
| | `contextra-vector` | HNSW, DiskANN, Quantisierung | `contextra-vector` | S, C |
| | `contextra-text` | BM25/BM25F, Morphologie | `contextra-text` | C |
| | `contextra-graph` | CSR, PPR, Leiden, Hyperkanten | `contextra-graph` | S, C |
| | `contextra-rank` | 4-Signal-Fusion, Isotonic/Platt-Kalibrierung, Drift | `contextra-db::fusion`, `contextra-rank` | C |
| | `contextra-adapt` | Bandit, Lyapunov, PID, Homeostat, Decay; `Clock`/`Rng` injiziert (P28) | `contextra-router` (Bandit/Lyapunov-Teil), `contextra-rank::pid`, `contextra-db` (Decay/Homeostat/PID) | C |
| **1** (Persistenz, async an I/O-Grenzen erlaubt) | `contextra-store` | WAL (Group-Commit, HMAC-Kette), LSM, MVCC-Pin | `contextra-store` | S, C |
| | `contextra-kvcache` **(neu)** | Prefix-Radix-Baum, KV-Blöcke, Tiering, AEAD, Segmentdateien (§9) | `contextra-crypto::kv_segment`, `contextra-infer-candle::kv_bridge` | C |
| | `contextra-checkpoint` | Time-Travel-Registry gegen Port `StorageEngine`, ohne globalen Zustand | `contextra-checkpoint` (Global-State `ORPHAN_REGISTRY` entfernt, P29) | C, D |
| **2** (Blätter, flüchtige Abhängigkeiten) | `contextra-infer-candle` | GGUF, eigenes Llama-Modell mit `KvState` (§9, Stufe B) | `contextra-infer-candle` | I |
| | `contextra-infer-ollama` | HTTP-Backend, Contextual-Chunk-Prefixing | `contextra-infer-ollama` | I |
| | `contextra-infer-onnx` | `ort`, Cross-Encoder; aus `default-members` ausgeschlossen | `contextra-infer-onnx` | I |
| | `contextra-sandbox` | WASM-Isolation, Fuel- und Wall-Clock-Budget; implementiert `ToolSandbox` | `contextra-sandbox` (bislang Waisen-Crate, jetzt an `contextra-agent`/`contextra-mcp` angebunden) | I |
| **3** (Anwendungskern) | `contextra-engine` | Collection, Transaktionen, `RetrievalPlanner`, Ingestion, Export/Import, `ComputePool` | `contextra-db` (Datenebene) | S, C |
| | `contextra-cognition` | Consolidation, Synthese, Kompaktierung, Scheduler | `contextra-db` (Kontrollebene) | C |
| | `contextra-privacy` | Egress-Gateway, PII-Vault, DLP, `GuardedPayload`, Prompt-Injection-Filter | `contextra-crypto::egress_vault`, `contextra-mcp::egress_gateway`, `contextra-router::guarded_payload` | C |
| | `contextra-router` | SLM-Profil-Routing, MCP-Dispatch (schlank; **keine** Numerik mehr — die wandert nach `adapt`) | `contextra-router` (Rest nach Abzug von `adapt`) | C |
| | `contextra-agent` | Workflow-Engine, Audit, DLQ | `contextra-agent` | C |
| **4** (Ränder) | `contextra` **(neu)** | Fassade, Builder, **einzige Composition Root** | `contextra-db` (Fassaden-Anteil) | D |
| | `contextra-mcp` | stdio-JSON-RPC, Protokoll, Tool-Wiring | `contextra-mcp` | C |
| | `contextra-py` | PyO3, eigene Runtime, `catch_unwind` | `contextra-py` | I |
| **Tooling** | `contextra-testkit` **(neu)**, `contextra-bench`, `xtask` | Fault-VFS, `ManualClock`, In-Memory-`StorageEngine` (P28); Benchmarks; CI-Tooling (Ziel < 3.000 LOC) | `contextra-bench`, `xtask` | — |

**Entfallen als eigenständige Crates:** `contextra-core`, `contextra-core-ipc-gen`, `contextra-rank`, <!-- crate-ref-ignore -->
`contextra-infer-onnx`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-db` (nach Abschluss der Strangler-Phase, §20).
**Löschkandidat, vor Löschung zu prüfen:** `contextra-core/types/saos.rs` (707 LOC, nur intern referenziert).

### 4.3 Abhängigkeitsmatrix (löst die alte Layer-Reihenfolge ab)

```
Ring 0  → Ring 0 in Reihenfolge  types → {ports, mvcc, wire, sys, simd, crypto} → {vector, text, graph, rank, adapt}
          Kerne (vector, text, graph, rank, adapt) kennen einander nicht; Kommunikation nur über types/ports/mvcc.
Ring 1  → Ring 0. Kein Ring-1-Crate hängt von einem anderen Ring-1-Crate ab.
Ring 2  → types, ports (+ crypto für Fingerprints). Niemals Ring 1 oder 3.
Ring 3  → Ring 0, Ring 1, Ports von Ring 2 (nie deren konkrete Crates).
          Interne Ordnung: privacy < engine < {cognition, router, agent}.
Ring 4  → alles.
dev-Kanten → nur contextra-testkit und Crates desselben oder tieferen Rings.
```

**Erzwingung (verbindlich ab Migrationsphase 1a, Warnmodus in Phase 0R, §20):**
1. `tests/layering.rs` (Workspace-Root, `cargo_metadata`) prüft alle Kantentypen (normal, build, dev,
   target-spezifisch) gegen die Matrix.
2. `deny.toml` (bestehende Datei **erweitern**, nicht neu anlegen — §A2.1 D7) erhält `[[bans.deny]]`-Einträge
   mit `wrappers`-Listen für `tokio` (verboten außerhalb Ring 1+), `candle-core`, `ort`, `wasmtime`, `pyo3`,
   `reqwest` (je genau ein erlaubtes Blatt-Crate). `wrappers` begrenzt nur direkte Abhängige; transitives
   Einschleppen prüft eine `cargo tree`-Assertion in CI zusätzlich.
3. `tests/unsafe_islands.rs` (§0.4).

### 4.4 Trait-Eindeutigkeit und Modul-Governance (⚠️ Opus-Optimierung 2.6, Stufe 2, mittel) — historischer Befund, Crate-Zuordnung aktualisiert

**Hinweis:** Der folgende Befund wurde am vormaligen `contextra-core/src/traits/` erhoben. Nach der Migration
(§20, Phase 1b) liegt dieses Verzeichnis in `contextra-ports`; die Maßnahmen gelten unverändert für den neuen
Ort. Der Befund selbst bleibt hier unverändert dokumentiert, damit er nicht verloren geht.

**Problem:** `contextra-core/src/traits/` enthält zehn Dateien; `mod.rs` deklariert nur fünf. Vier Dateien
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

**Testpflicht:** `crates/contextra-core/tests/no_orphan_modules.rs` — schlägt fehl, sobald eine Datei unter `src/`
existiert, die von keinem `mod`-Pfad aus erreichbar ist.

---

<a id="5-speicher"></a>
