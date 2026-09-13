# MemFuse — Gesamtspezifikation des Endprodukts v5

> **Status:** Verbindlich · Einzige normative Quelle für Produkt, Architektur und Prozess
> **Version:** 5.0 · **Erstellt:** 2026-09-13 · **Basis:** v4 + Strategiebericht (d56289b) + Spezifikationserweiterung §12–§14 + Architektur-Audit §12–§14
> **Geltungsbereich:** Diese Spezifikation beschreibt den **Zielzustand** von MemFuse — wie das System als fertiges Produkt aussieht, sich verhält und verteilt wird. Sie ist keine Fortschritts-, Audit- oder Änderungshistorie und enthält bewusst keine Commit-Referenzen, Revisionsvergleiche oder Statusabzeichen. Alle Aussagen sind normative Festlegungen dessen, was das System leisten MUSS bzw. wie es strukturiert ist.
> **Codeumfang (Größenordnung):** ca. 160.000 Zeilen Rust über 17 Workspace-Crates (16 Kern + `memfuse-sandbox`) plus `memfuse-py` als isoliertes FFI-Workspace, plus `xtask`- und `memfuse-bench`-Werkzeug-Crates.
> **Vorgängerdokument:** `MEMFUSE_ENDPRODUKT_SPEZIFIKATION_FINAL_4.md` (v4). Änderungen gegenüber v4 sind: neue §§ 12–14, neues Crate §4.18 `memfuse-sandbox`, erweiterte §§ 4.14/4.16 (Router-Bandit/Transport-Erweiterung), gehärtete §§ 9.1/10, neue Glossareinträge §11.

---

## §1 — Kernthese

**MemFuse ist die technisch fortschrittlichste, vollständig lokal betriebene Gedächtnisschicht für KI-Agenten.**

Die Kombination aus 4-Signal-Retrieval-Fusion (Vektor, Volltext, Graph, Metadaten) mit PathRAG-Multi-Hop-Traversierung, proaktiver Kalibrierungs-Drift-Erkennung, kryptographisch integritätsgesicherter Storage-Engine, echter WASM-Ausführungs-Isolation und einer echten Air-Gap-Inferenzoption ist im Feld lokal betriebener AI-Memory-Systeme ohne direktes Äquivalent. Der oberste Produktgrundsatz:

> **Release schlägt Feature.** Solange kein erstes stabiles Release existiert, hat jede Aufgabe, die direkt zu einem `uvx`- oder `pip install`-fähigen Artefakt führt, Vorrang vor neuer Retrieval- oder Konsolidierungsarbeit — mit der einzigen Ausnahme von Fehlern mit Silent-Data-Corruption-Risiko.

---

## §2 — Produktvision

### §2.1 Was MemFuse ist

MemFuse ist eine souveräne, lokal betriebene Gedächtnisschicht für KI-Agenten. Sie wird primär als **MCP-Server** (`memfuse-mcp`, Installation via `uvx`) und sekundär als **Python-Library** (`pip install memfuse`) sowie als **Rust-Crate** (`crates.io`) verteilt. MemFuse ist reine Infrastruktur — kein menschlicher Endnutzer interagiert je direkt mit MemFuse ohne einen dazwischenliegenden Agenten, Assistenten oder ein Framework.

### §2.2 Primärer Eingang: MCP-Server zuerst

1. **`memfuse-mcp`** (`uvx`-paketiert) — MCP-Server für Claude Desktop, Cursor, Windsurf, Cline. Installationsfluss: `uvx memfuse-mcp --db-path ~/.memfuse`.
2. **`memfuse` (Python-Library)** — `pip install memfuse` für Python-Entwickler, die LangChain/LlamaIndex/eigene Agentenloops mit lokalem Gedächtnis versorgen.
3. **`memfuse` (Rust-Crate)** — `crates.io`-Distribution für Rust-native Agentenframeworks.
4. **`memfuse` Enterprise** (Fernziel) — DSGVO-konforme, auditierbare Deployments, relevant sobald 1. und 2. echte Nutzer haben.

Eine grafische Desktop-Anwendung ist kein Bestandteil des Produkts (siehe §2.4).

### §2.3 Alleinstellungsmerkmale

1. **4-Signal-Retrieval-Fusion inkl. PathRAG:** HNSW (Vektor) + BM25 (Volltext, deutsche Komposita-Dekomposition) + CSR-Graph (PageRank) + Metadaten-Filter, fusioniert via Reciprocal Rank Fusion (RRF) mit optionalem Resonanz-Kohärenz-Bonus. PathRAG (bidirektionaler Dijkstra) liefert ein viertes, multi-hop-fähiges Signal.
2. **Kalibriertes Retrieval mit proaktiver Drift-Erkennung:** Isotonic-Kalibrierung (PAVA) + Lyapunov-Drift-Watcher erkennen Qualitätsverschlechterung, bevor sie beim Nutzer sichtbar wird. Live-Observability-Daten (Drift-Status, Kalibrierungsfehler, PID-Pool-Größe) sind bis in die Python- und MCP-Grenzschicht durchgehend verdrahtet — keine Platzhalterwerte an der API-Oberfläche.
3. **MCP-native mit technisch erzwungener Zero-Trust-Sandbox:** Prompt Injection Guard, volatile Tool-Output-Verschlüsselung **und** — für die `CodeExecution`-Permission — eine echte WASM-Ausführungsgrenze (`memfuse-sandbox`, §4.18) sind in `memfuse-mcp` first-class. Die Zero-Trust-Eigenschaft ist für Code-Ausführung technisch erzwungen, nicht nur behauptet: Agenten-Tool-Code läuft in einer speicher-isolierten WASM-Instanz mit expliziten Capability-Grenzen, nicht im selben Prozessraum wie MemFuse.
4. **Kryptographische Integrität & Löschung:** WAL-HMAC-Kette und `DeletionProof` (kryptographischer Löschnachweis, DSGVO Art. 17) auf Storage-Ebene.
5. **Typsichere Lock-Infrastruktur:** Session-DAG mit `NodesGuard`-Typ erzwingt Lock-Reihenfolge zur Compile-Zeit. Die Deadlock-Freiheitsgarantie erstreckt sich über den Konsolidierungspfad via `ConsolidationNodesGuard::try_acquire()`.
6. **Echter Air-Gap-Modus mit KV-Cache-Bridge:** Native Candle-GGUF-Inferenz (Pure Rust) ist als vollwertiger Inferenz- und Embedding-Backend-Pfad verdrahtet — kein Ollama-Prozess und kein Netzwerkzugriff nötig. Eine mandantenisolierte, verschlüsselte KV-Cache-Bridge verbindet Retrieval-Treffer direkt mit dem Inferenz-Backend.
7. **Portables, versioniertes Memory-Export-/Import-Format:** Vollständiger Export **und** idempotenter Re-Import einer Collection als JSON, tool- und plattformunabhängig lesbar — Grundlage für Backup, Migration und Interoperabilität. Macht MemFuse zum System ohne Vendor-Lock-in: Nutzer können jederzeit wechseln, was Vertrauen und Adoption erhöht.
8. **Cloud-Egress Privacy Gateway** (optional, `cloud-egress-guard`): Für Nutzer, die Cloud-LLMs bewusst und selbst einsetzen, schützt ein fünfschichtiges Privacy-Gateway (deterministisches Token-Vaulting, lokale Abstraktion, Graph-Generalisierung, Bulk-Exfiltrations-Erkennung via `EgressGuard`, bidirektionaler Zero-Trust) vor ungewollter Datenexfiltration — ohne das Air-Gap-Standardverhalten zu berühren (P5).

### §2.4 Nicht-Ziele (verbindlich)

- Kein Cloud-SaaS, auch nicht optional gehostet.
- Kein Multi-Tenant-Enterprise-Produkt für gleichzeitige Fremdkunden auf einer Instanz.
- Kein eigenes LLM-Training/Fine-Tuning-Feature.
- Keine grafische Desktop-Oberfläche.
- Kein eigenes Agentenframework — reine Gedächtnisschicht für externe Frameworks.
- Keine Cloud-Vektordatenbank-Alternative — embedded/lokal-only.
- Kein Voice-Assistant-Interface.
- Kein verteilter Konsenscluster (`memfuse-cluster`, Raft/`openraft`) — weder als Kern- noch als optionales Feature (§14 begründet dies erschöpfend; das Veto gilt zeit-invariant unabhängig von verfügbarer Implementierungsgeschwindigkeit).

---

## §3 — Architekturprinzipien P1–P20

- **P1 — DAG-Integrität:** `cargo xtask check-dag` ist zwingendes CI-Gate. Kein Code in Layer N darf Abhängigkeiten auf Layer >N besitzen. Wo eine höherschichtige Information (z. B. Router-Drift-Status) in einer niedrigeren Schicht sichtbar sein muss, erfolgt dies ausschließlich über eine optionale, schwache Referenz (`Weak<T>`-Injection zur Konstruktionszeit), niemals über eine harte Abhängigkeitsumkehr. `memfuse-sandbox` (Layer 6.5) ist bewusst zwischen Router (Layer 6) und MCP (Layer 8) positioniert und von Layer 8 (`memfuse-mcp`) stark abhängig — keine DAG-Verletzung, da Layer 8 nur nach Layer 6.5 geordnet ist.
- **P2 — Zero-Panic-Doctrine:** Production-Code ist panic-frei. `unsafe` ist beschränkt auf drei funktionale Kategorien: (a) SIMD-Distanzberechnung, (b) Mmap-Persistenz mit dokumentiertem `// SAFETY:`-Proof, (c) plattformspezifische Systemaufrufe mit dokumentiertem `// SAFETY:`-Proof. `memfuse-py` nutzt `panic = "unwind"` mit `catch_unwind`-Isolation. `memfuse-sandbox` hält `#![forbid(unsafe_code)]` — der WASM-Host-Aufruf läuft über `wasmtime`s sicheres API ohne unsafe.
- **P3 — WAL-First:** Kein Datenschreibvorgang ohne vorherigen WAL-Commit. `fsync` auf Datei- und Directory-Ebene.
- **P4 — Inferenz-Backend-Agnostizismus:** `LlmTextGenerator`/`LlmTextGeneratorStreaming` und `TextEmbeddingEngine` in `memfuse-core` sichern die Abstraktion. Backend-spezifische Fähigkeiten werden ausschließlich additiv über Default-Trait-Methoden angeboten.
- **P5 — Kein Cloud-Zwang:** Inferenz und Retrieval laufen vollständig lokal. Das optionale `cloud-egress-guard`-Feature berührt diesen Grundsatz nicht — es ist standardmäßig deaktiviert und adressiert Nutzer, die Cloud-LLMs *ohnehin* und *selbst* einsetzen, außerhalb von MemFuse.
- **P6 — Eine Quelle für Architekturentscheidungen:** `DECISIONS.md` ist die einzige Quelle für ADRs.
- **P7 — Code-Nachweis-Pflicht für Marketing-Aussagen:** Quantitative Leistungsversprechen benötigen reproduzierbare Benchmark-Nachweise in `memfuse-bench` — null Ausnahmen. Dies gilt explizit für: (a) KV-Cache-Bridge-Prefill-Einsparung (benötigter Benchmark: `bench_kv_bridge_prefill_savings`), (b) Session-DAG-Deadlock-Freiheit (benötigter Stress-Test: `stress_session_dag_deadlock_freedom`), (c) LinUCB-Bandit-Regret vs. Kaskaden-Baseline (`test_bandit_vs_cascade_regret_comparison`, §13.3). Ohne diese Benchmarks bleiben die entsprechenden USP-Aussagen in README/PyPI-Beschreibung als `[BENCHMARK_PENDING]` markiert.
- **P8 — Kalibrierungs-Integrität:** Jede Änderung an `prompt_template_hash`, `temperature_bits` oder `quantization` invalidiert automatisch alle Kalibrierungsstatistiken. Eine analoge `ModelFingerprint`-Bindung (HKDF-gebunden) gilt für alle persistierten KV-Cache-Segmente.
- **P9 — Kein Klartext-Sensitivspeicher:** Sensitiver Tensor-Speicher (Volatile Vault, KV-Cache-Segmente, WASM-Sandbox-Outputs) wird nach Nutzung via `ZeroizeOnDrop` überschrieben.
- **P10 — Reuse vor Neubau:** Prüfung gegen `TYPE_REGISTRY.md` und CI-Gate `check-duplicate-symbols` vor Neuanlage. Für §12: Aho-Corasick-Erkennung nutzt die vorhandene ONNX-Runtime aus `memfuse-embed`; Layer-3-Abstraktion nutzt den vorhandenen `SegmentSynthesizer`-Trait; `EgressGuard` nutzt den vorhandenen HNSW-Index als k-NN-Backend. Für §13: Der Bandit-Router nutzt den bereits für Retrieval berechneten Query-Embedding-Vektor ohne zweite Inferenz; `ndarray 0.15` wird via re-export aus `memfuse-embed/onnx` bezogen, nicht neu eingeführt.
- **P11 — Latenzbudget-Pflicht für Hot-Path:** `RerankPidController` und `PidLatencyController` (beide produktiv) begrenzen P95-Latenzen. Für das Cloud-Egress-Gateway (§12.2.2) gilt: Layer 2 (lokale Abstraktion via `SegmentSynthesizer`) wird durch denselben `PidLatencyController`-Mechanismus begrenzt; bei Latenzbudget-Überschreitung wird Layer 2 übersprungen (Fail-Open zu Layer 3), niemals blockiert.
- **P12 — Physio-Feature-Default-Unsichtbarkeit:** Alle `physio-*`-Feature-Aliase sind per Feature-Flag deaktivierbar, Defaults bleiben transparent.
- **P13 — Modulgrenzen nach Verantwortung:** Klare Trennung Layer 0 (Fundament) bis Layer 8 (Agenten/Interfaces). `memfuse-sandbox` belegt Sublayer 6.5 — abhängig von Layer 2/3 (`memfuse-core`, `memfuse-security`), Abhängigkeit von Layer 8 (`memfuse-mcp`) in umgekehrter Richtung (MCP hängt von sandbox ab, nicht umgekehrt).
- **P14 — Klare Verantwortungstrennung für Hintergrundtasks:** `MaintenanceScheduler` und `ConsolidationEngine` sind zwei sich ergänzende, aber gegenseitig exklusive Mechanismen pro Collection: gesichert durch `consolidation_guard: Arc<tokio::sync::Mutex<()>>` per `try_lock` (ADR-081). Neue Hintergrund-Task-Kategorien dürfen nur mit explizitem ADR eingeführt werden.
- **P15 — Eine Vision pro Release:** MCP-Server zuerst, Python-Library parallel.
- **P16 — Dokumente als Zieldefinitionen:** Dokumente beschreiben Soll-Zustände und maschinenlesbare Invarianten, keine ephemeren Code-Zeilen.
- **P17 — Ambient-Kontext ist keine Garantie:** `AGENTS.md` wird nicht automatisch geladen; jeder Trigger-Prompt erzwingt das Einlesen von `AGENTS.md` und `.jules/SESSION_BOOTSTRAP.md`.
- **P18 — Parallele Sessions brauchen Claims:** `cargo xtask claim --crate X --issue Y` sperrt Ziel-Crates vor paralleler Bearbeitung. Der Check MUSS zusätzlich alle offenen Remote-Branches und Pull-Requests über die GitHub-API auf normalisierte Subject-Ähnlichkeit prüfen, bevor ein PR als merge-fähig gilt; ist die API nicht erreichbar, MUSS dies als expliziter Fehler gemeldet werden, niemals als stiller Fallback auf ein falsches „OK".
- **P19 — Beschlossener, nicht umgesetzter Governance-Beschluss ist gefährlicher als keiner:** Beschlüsse werden per CI-Gate durchgesetzt oder formal widerrufen. Dies gilt symmetrisch: Ein stillschweigend revidierter, aber nicht neu dokumentierter Beschluss (z. B. die Cluster-Veto-Reaffirmation §14) ist ebenso gefährlich wie ein nicht umgesetzter.
- **P20 — Release schlägt Feature:** Höchste Priorität bei Konflikt (siehe §1). `bandit-routing` (§13) und `cloud-egress-guard` (§12) und `memfuse-sandbox` (§4.18) werden per Feature-Flag (Default: off) gegen den Release-Pfad abgeschirmt.

---

## §4 — Crate-Spezifikation (mikrofein)

### §4.1 `memfuse-core-ipc-gen` — Layer 0

Auto-generierter FlatBuffers-IPC-Code für die MemFuse-Kernprotokolle. Keine internen Abhängigkeiten (Wurzel des DAG). Enthält die generierten FlatBuffers-Typen für Query-/Response-Strukturen, die über die Python- und MCP-Grenzschicht transportiert werden. Wird nicht manuell editiert; Regeneration via Build-Skript aus `.fbs`-Quellschemata.

### §4.2 `memfuse-core` — Layer 1 (Fundament, Dependency-Root)

Stellt alle domänenweiten Typen, Trait-Abstraktionen und die einheitliche Fehlerbehandlung bereit. Jedes andere Crate hängt transitiv davon ab. Abhängigkeit: `memfuse-core-ipc-gen`.

**Module:**
- `error.rs`/`error_dto.rs` — `MemFuseError`-Enum, DTO-Serialisierung für Grenzschichten.
- `ipc/` — JSON-RPC-Hilfstypen.
- `seq_log.rs` — Sequenz-Logging-Primitive für MVCC-Ordering.
- `snapshot.rs` — `SnapshotRegistry` für MVCC-Lese-Isolation.
- `traits/mod.rs` — Kern-Traits: `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `CheckpointCoordinator`, `Checkpoint`, `Snapshot`, `TextEmbeddingEngine`, `LlmTextGenerator`, `SegmentSynthesizer` (inkl. additiver Modus `SynthesisMode::PrivacyAbstraction` für §12.2.2), `DistanceCalculator`, `MemoryLifecycleManager`, `GroundingValidator`, Stats-Typen, `ConsolidationAction`-Enum, `GroundingAssessment`.
- `traits/embedding.rs` — `EmbeddingProvider`, `TextGenerator`, `EmbeddingError`, `MockEmbedder`, **`LlmTextGeneratorStreaming`**, `generate_with_context()`.
- `model_fingerprint.rs` — kanonischer `ModelFingerprint`-Typ.
- `types.rs` + Untermodule — Domänentypen inkl. `SynthesisMode`-Enum (`SleepCycle` | `PrivacyAbstraction`) als neuer Typ für die duale Nutzung des `SegmentSynthesizer`-Traits.

**Feature-Flags:** `test-utils`.

### §4.3 `memfuse-calibration` — Layer 2

Score- und Wahrscheinlichkeits-Kalibrierung. Abhängigkeit: `memfuse-core`.

- `isotonic.rs` — `IsotonicCalibrator` via PAVA.
- `platt.rs` — `PlattScaler`.
- `pid.rs` — `RerankPidController` und `PidLatencyController` (beide produktiv).

### §4.4 `memfuse-checkpoint` — Layer 2

Öffentlich sichtbarer Checkpoint-Subsystem-Einstiegspunkt. Abhängigkeit: `memfuse-core`.

**Kernschnittstellen:** `CheckpointCoordinator`-Trait-Implementierung; `PersistentCheckpointStore`; `CheckpointGuard`.

### §4.5 `memfuse-graph` — Layer 2

CSR-Graph für Entity-Relation-Traversal sowie Session-DAG. Abhängigkeiten: `memfuse-core`, `memfuse-store`.

- `csr.rs`, `ppr.rs`, `community.rs`, `path_rag.rs`, `session_dag.rs`, `cascade.rs`, `consistency_enforcement.rs`, `edge_reinforcement.rs`/`edge_reinforcement_buffer.rs`, `percolation.rs`, `provenance.rs`.

**`NodesGuard`-Garantie (erweitert):** Die Compile-Zeit-Deadlock-Freiheit gilt jetzt nachweislich über den gesamten Konsolidierungspfad: `consolidation_executor.rs` bezieht Tx-Sperren via `ConsolidationNodesGuard::try_acquire()` — eine eigenständige, vom `NodesGuard`-Typ abgeleitete Hülle, die dieselbe Lock-Reihenfolge erzwingt und durch `test_consolidation_deadlock_freedom` (Regression) permanent abgesichert ist.

**Feature-Flags:** `default = []`, `graph-connectivity-health`, `edge-reinforcement-learning`, `physio-percolation` → `graph-connectivity-health`, `physio-synaptic-edges` → `edge-reinforcement-learning`.

### §4.6 `memfuse-crypto` (Package-Name `memfuse-security`) — Layer 2

Verschlüsselung, Integritätsschutz und mandantenisolierte KV-Cache-Sicherheit. Abhängigkeit: `memfuse-core`. Namenskonvention: Verzeichnisname `memfuse-crypto`, Cargo-Package-Name `memfuse-security`.

- `crypto.rs` — `KeyManager`: AES-256-GCM-SIV, HKDF-SHA256, `derive_file_key()`, `derive_kv_key()`. Alle variablen HKDF-Info-Felder sind längenpräfixiert. Nonces: instanzweites Zufallspräfix plus OsRng-Suffix.
- `wal_crypto.rs` — WAL-HMAC-Kette mit `subtle::ConstantTimeEq`.
- `anti_tamper.rs`, `deletion_proof.rs` — `DeletionProof` (DSGVO Art. 17).
- `kv_cipher.rs` — `KvSegmentCipher`/`EncryptedKvLayer` mit Format-Versionsfeld.
- `kv_segment/` — `TenantIsolatedKvStore`: `DEFAULT_SHARD_COUNT = 32`, `#[repr(align(64))]`-Shards, O(1)-LRU, Deferred Zeroization, `EvictionWorker` auf separatem OS-Thread, `evict_lru_fair()`, `clear_all()` (synchroner Emergency-Wipe).
- `egress_vault.rs` (neu, hinter Feature `cloud-egress-guard`) — Wiederverwendung des `volatile_vault`-Paradigmas für die Surrogat-Mappings des Egress-Gateways (§12.2.1): `EgressVault` hält die `Surrogat → Klartext`-Map AES-256-GCM-SIV-verschlüsselt in einer `TenantIsolatedKvStore`-Instanz mit `ZeroizeOnDrop`; pro Session initialisiert, beim Session-Ende vollständig gewipet.
- `error.rs` — crate-lokale Fehlertypen.

**Feature-Flags:** `test-utils`, `kv-encryption`, `cloud-egress-guard` (neu, aktiviert `egress_vault.rs`).

### §4.7 `memfuse-text` — Layer 2

BM25-Volltextsuche mit deutscher Kompositum-Dekomposition. Abhängigkeit: `memfuse-core`.

- `bm25.rs`, `inverted.rs`, `morphology.rs`, `tokenizer.rs`.

### §4.8 `memfuse-candle` — Layer 3

Native Candle-GGUF-ML-Inferenz-Engine (Pure-Rust). Abhängigkeiten: `memfuse-core`, `memfuse-calibration`, optional `memfuse-crypto` (Feature `kv-bridge`).

- `gguf_loader.rs`, `inference.rs` (`CandleLlmClient`, implementiert `LlmTextGenerator`/`LlmTextGeneratorStreaming`), `embedding.rs`/`embedding_provider.rs`, `model_registry.rs` (`compute_fingerprint()`), `gasp.rs`, `kv_bridge.rs` (Feature `kv-bridge`).

**Feature-Flags:** `default = []`, `candle`, `cuda`, `metal`, `kv-bridge`.

### §4.9 `memfuse-index` — Layer 3

HNSW-Vektorindex mit SIMD-beschleunigter Distanzberechnung. Abhängigkeiten: `memfuse-core`, `memfuse-security`.

- `hnsw.rs`, `distance.rs` (SIMD, unsafe), `diskann.rs` (unsafe), `persistence.rs` (unsafe), `quantize.rs`, `partial_rebuild.rs` (VETO-F02, Feature `partial-index-rebuild`, gesperrt bis `test_partial_rebuild_recall_regression()` 30 Tage stabil, ADR-070).

**Adaptiver DiskANN-Flush-Threshold** (ADR-076): `PENDING_FLUSH_THRESHOLD(N) = max(50, min(1000, ⌊N × 0.05⌋))` — implementiert als dynamisch berechnete Funktion statt statischer Konstante.

**Feature-Flags:** `default = []`, `experimental-diskann`, `partial-index-rebuild`.

### §4.10 `memfuse-ollama` — Layer 3

HTTP-Client für Ollama. Abhängigkeiten: `memfuse-core`, `memfuse-calibration`, `memfuse-embed`.

- `client.rs`, `embedding.rs`, `context_prefixer.rs`, `importance.rs`, `model_info.rs`.

### §4.11 `memfuse-embed` — Layer 4

In-Process-Text-Embeddings über ONNX Runtime. Abhängigkeiten: `memfuse-core`, `memfuse-calibration`, `memfuse-candle` (optional).

- `lib.rs` — ONNX-Runtime, `max_concurrent_embeddings` Backpressure-Vertrag.
- `reranker.rs` — Cross-Encoder-Reranking.

**`ndarray 0.15` Re-Export:** Das unter Feature `onnx` bereits als transitive Abhängigkeit vorhandene `ndarray 0.15` wird für den LinUCB-Sherman-Morrison-Pfad in `memfuse-router` (§4.14) via `memfuse-embed/onnx` re-exportiert — kein neuer Crate-Import, P10-konform.

**Feature-Flags:** `default = []`, `onnx`, `candle-backend`.

### §4.12 `memfuse-store` — Layer 3

LSM-Tree-basierte Storage-Engine. Abhängigkeiten: `memfuse-core`, `memfuse-security`.

- `wal.rs` — WAL v3 mit HMAC-Kette, WAL-Flusher-Actor (standardmäßig aktiv: `Wal::new()` sowie `open_with_key_manager()` rufen `enable_flusher()` ohne gesonderten Aufrufer-Opt-in auf), Zero-Copy-mmap-Replay (`replay_mmap()`; `replay()` delegiert direkt dorthin. Fallback auf den sequenziellen Stream-Reader bei mmap-Fehlern ist implementiert. Bit-Identität beider Pfade ist über `test_wal_replay_stream_vs_mmap_parity` abgesichert).
  - **WAL-Rotation für passives Shipping** (§14.3): Abgeschlossene WAL-Segmente werden durch `Wal::rotate_and_seal()` atomar versiegelt (via `rename()` + `fsync()` auf Directory) und danach `O_RDONLY`-reflaggt. Nur versiegelte, read-only-geflaggite Segmente werden für das passive WAL-Shipping (Syncthing, iCloud Drive, One-Shot-HTTP-Push) freigegeben. Der Flusher-Actor schreibt ausschließlich in das aktive, nicht-versiegelte Segment; damit sind TOCTOU-Konflikte zwischen Flusher und externem Sync-Daemon strukturell ausgeschlossen.
- `memtable.rs` — 16-Shard-MemTable, Avalanche-64-Bit-Mixer, `scan_prefix_into`/`scan_prefix_into_matching`.
- `sstable.rs` — SSTable mit Block-Binärsuche (`binary_search_in_block()`, `binary_search_in_block_index()`, `binary_search_index_in_block()`), `BlockCacheShard` (`BLOCK_CACHE_SHARDS = 16` Shards, cache-line-ausgerichtet).
- `compaction.rs`, `lsm.rs` (Group-Commit, Zero-Wait-Heuristik: Bei fehlender Commit-Warteschlange verzichtet der Leader über eine `has_followers`-Prüfung auf das künstliche Warteintervall), `manifest.rs`, `mmap.rs`, `checkpoint.rs` (`pub(crate)`), `system_pressure.rs`, `tenant_codec.rs`, `util.rs`.

**Feature-Flags:** `default = []`, `fault-injection`.

### §4.13 `memfuse-db` — Layer 5 (Orchestrierungs-Kern)

Eingebettete Hybrid-Search-Engine für KI-Agenten. Abhängigkeiten: alle Layer-2–4-Crates.

**Lock-Hierarchie:** 1. `MemFuse::collections` → 2. `MemFuse::embedder` → 3. `Collection::insert_lock` / `Collection::embedder`.

**Module:** `collection/`, `fusion.rs` (Late Hydration, NaN-Guards, `AHashMap`), `multistep.rs` (mit `PidLatencyController`), `chunker.rs` (`MarkdownChunker`), `context.rs`/`context_compaction.rs`, `temporal_filter.rs`, `filter.rs`, `decay_controller.rs`, `homeostat.rs`, `memory_consolidation.rs`, `synthesis_phase.rs`, `consolidation_executor.rs` (mit `ConsolidationNodesGuard`), `maintenance_scheduler.rs`, `consolidation_engine.rs`, `maintenance_config.rs`, `background_workers.rs`, `transaction.rs`, `volatile_vault.rs`, `export.rs`, `import.rs`, `pid_latency_controller.rs`.

**`router`-Anbindung:** `MemFuse` hält `router: Option<Weak<RouterEngine>>` (ADR-080). `MemFuseDb::stats()` befüllt `drift_status`, `calibration_ece`, `last_calibration_at`, `pid_pool_size` mit Live-Daten via `Weak::upgrade()`.

**`EmbeddingBackend`-Enum:** `Onnx { model_name, cache_dir }` (Default), `Ollama { base_url, model }`, `Candle { model_dir, quantization }`, `None`.

**Feature-Flags:** `default = []`, `bench`, `sandbox`, `experimental-diskann`, `reranking`, `background-maintenance`, `graph-connectivity-health`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `volatile-vault`, `edge-reinforcement-learning`, `cloud-egress-guard` (neu).

### §4.14 `memfuse-router` — Layer 6

Conformal Router mit outcome-kalibriertem Routing, proaktiver Drift-Überwachung und optionalem Contextual-Bandit-Gate (§13). Abhängigkeiten: `memfuse-core`, `memfuse-store`, `memfuse-db`, `memfuse-ollama`.

**Module:**
- `router.rs` — `RouterEngine`. Kernmethoden: `route()` (async), `profiles()`, `update_profiles()`/`try_update_profiles()`, `calibration_stats()`, `reset_calibration()`, `drift_status()`, `set_lyapunov_baseline()`, `record_outcome()`, `pending_decision_count()`, `reset_all_calibration()`. Zustandstypen: `ConfidenceMetrics`, `RoutingDecision`, `RouterState`.
- `lyapunov.rs` — Lyapunov-Drift-Watcher (ADR-079: event-driven, nicht periodisch).
- `dispatch.rs` — Dispatch-Logik. **Hardening (§12.3, unabhängig von cloud-egress-guard):** `dispatch_to_slm()` MUSS den Subprozess via `Command::new(program).args(&[...])` ohne Shell-Zwischenschicht starten (kein `sh -c`); der `endpoint`-String aus `SlmProfile.mcp_endpoint` MUSS zuvor geparst und in Programmpfad + Argumente aufgeteilt werden. Beim `HttpCloud`-Transport (Feature `cloud-egress-guard`, §12.3) wird der `HttpCloud`-Zweig ausschließlich aufgerufen, wenn `GuardedPayload<Sanitized>` als Typ-State vorliegt.
- `outcome.rs` — `RoutingOutcome`: `Success`, `Escalated { .. }`, `Rejected`.
- `profile.rs` — `SlmProfile` mit neuem Feld `transport: Transport` (Default: `Transport::StdioMcp`, additive Erweiterung, kein Breaking Change). Neu: `bandit_state: Option<BanditProfileState>` (Feature `bandit-routing`).
- `transport.rs` (neu, Feature `cloud-egress-guard`) — `Transport`-Enum:
  ```rust
  pub enum Transport {
      StdioMcp,                    // bestehend, unverändert
      HttpCloud { url: String },   // neu, nur mit cloud-egress-guard + GuardedPayload<Sanitized>
  }
  ```
- `bandit.rs` (neu, Feature `bandit-routing`) — Contextual-Bandit-Implementierung (§13.2):
  - **`BanditProfileState`:** Hält pro Profil $p$ den Gewichtsvektor $\theta_p$ und — je nach `BanditImplementation`-Variante — entweder die diagonale Varianzschätzung (Default) oder die via Sherman-Morrison aktualisierte volle Inverse $A_p^{-1}$.
  - **`BanditImplementation`-Enum (Feature-interne Konfiguration):**
    ```rust
    pub enum BanditImplementation {
        /// Default: O(d) pro Update und Score. Keine BLAS-Abhängigkeit.
        /// A_p^{-1} wird als diag(σ₁², ..., σ_d²) approxmiert.
        DiagonalApproximation,
        /// O(d²) pro Update und Score. Nutzt ndarray re-exportiert aus memfuse-embed/onnx.
        /// A_p^{-1} wird vollständig via Sherman-Morrison-Rang-1-Update geführt.
        ShermanMorrison,
    }
    ```
  - **Diagonale Default-Implementierung (O(d), kein Latenzproblem):**
    - $\hat{r}_p(x) = \theta_p^\top x + \alpha \sqrt{\sum_i \sigma_{p,i}^2 \cdot x_i^2}$
    - Update bei Reward $r$: $\theta_p \leftarrow \theta_p + \eta \cdot (r - \theta_p^\top x) \cdot x$; $\sigma_{p,i}^2 \leftarrow \sigma_{p,i}^2 + x_i^2$ (kumulierte Varianz je Dimension).
    - Speicherbedarf: 2 `Vec<f32>` der Länge $d$ pro Profil. Kein `ndarray`, kein BLAS.
  - **Sherman-Morrison-Implementierung (O(d²), opt-in):**
    - $A_p^{-1}$ wird als $d \times d$ `ndarray::Array2<f32>` (re-exportiert aus `memfuse-embed/onnx`) direkt geführt.
    - Rang-1-Update nach Beobachtung $(x, r)$: $A_p^{-1} \leftarrow A_p^{-1} - \frac{A_p^{-1} x x^\top A_p^{-1}}{1 + x^\top A_p^{-1} x}$ — strikt O(d²) ohne Matrix-Inversion.
    - Score-Berechnung: $\hat{r}_p(x) = \theta_p^\top x + \alpha \sqrt{x^\top A_p^{-1} x}$ — zwei Matrix-Vektor-Produkte O(d²).
    - **Latenz-Garantie:** Beide Implementierungen sind durch den `PidLatencyController`-Mechanismus (P11) budgetiert. Überschreitet der Bandit-Scoring-Schritt das konfigurierte Budget, fällt `RouterEngine` transparent auf `select_profile_cascade()` zurück. Die Diagonal-Approximation ist der sichere Default, der das Latenzbudget strukturell einhält.
  - **Belohnungssignal:** `Success = 1.0`, `Rejected = 0.0`, `Escalated { .. } = 0.3` (konfigurierbar) − $\lambda \cdot \text{resource\_cost\_estimate}$ (Feld in `SlmProfile` bereits vorhanden) − $\mu \cdot \mathbb{1}[\text{profile.transport} = \text{HttpCloud}]$ (Privacy-Malus für Cloud-Profile, konfigurierbar, Default $\mu = 0.2$).
  - **Drift-gekoppelte Exploration:** Bei Drift-Erkennung durch `LyapunovDriftWatcher` für Profil $p$ wird $\alpha_p$ temporär erhöht (konfigurierbar, Default: Faktor 2.0).
  - **Kapazitätsbeschränkung:** Profil-Auslastung > Budget → temporärer Strafterm auf $\hat{r}_p(x)$ via `PidLatencyController`-Rückkopplung.
- `routing_strategy.rs` (neu, Feature `bandit-routing`):
  ```rust
  pub enum RoutingStrategy {
      Cascade,           // Default, bestehend, kalibriert, produktiv
      ContextualBandit,  // Feature bandit-routing, Default: off
  }
  ```
  `RouterEngine::route()` dispatcht je nach `RoutingStrategy`-Feld auf `select_profile_cascade()` (bestehend) oder `select_profile_bandit()` (neu). Der Kaskaden-Pfad wird **nicht** entfernt — er bleibt der Produktions-Default.
- `serde_helpers.rs` — Serialisierungs-Hilfsfunktionen.
- `guarded_payload.rs` (neu, Feature `cloud-egress-guard`) — Typ-State `GuardedPayload<Sanitized>`: nur durch erfolgreichen Durchlauf aller fünf Egress-Guard-Layer konstruierbar. Ein `GuardedPayload<Unsanitized>` an `dispatch_to_slm()` zu übergeben ist ein Compile-Fehler.

**Architektur:** `RouterEngine` hält Profile, Kalibrierung und Drift-Watcher in `ArcSwap<RouterState>` (atomar Hot-Reload), `pending_decisions` in `RwLock<HashMap<...>>` (bewusst getrennt, Contention-Vermeidung).

**Feature-Flags:** `default = []`, `bandit-routing` (Default: off, kein Breaking Change), `cloud-egress-guard` (Default: off), `egress-sherman-morrison` (opt-in für O(d²)-Bandit, Default: diagonal O(d)).

### §4.15 `memfuse-agent` — Layer 7

Persistenter Agenten-Workflow-Loop nach dem Muster `checkpoint → execute → commit → audit`. Abhängigkeiten: `memfuse-core`, `memfuse-db`, `memfuse-graph`, `memfuse-checkpoint`, `memfuse-store`, `memfuse-router`, `memfuse-index`, `memfuse-text`.

- `engine.rs`, `graph.rs`, `step.rs`, `context.rs`, `dlq.rs`, `audit.rs`, `event_source.rs`.

### §4.16 `memfuse-mcp` — Layer 8 (primäre Grenzschicht)

Model-Context-Protocol-Server — stdio-basiertes JSON-RPC-2.0-Interface. Layer-8-Rand-Crate ohne `unsafe`-Toleranz. Abhängigkeiten: `memfuse-db`, `memfuse-core`, `memfuse-security`, `memfuse-ollama`, `memfuse-embed` (optional), `memfuse-agent` (optional), `memfuse-candle` (optional), `memfuse-sandbox` (Feature `wasm-sandbox`).

- `bin/memfuse-mcp-server.rs` — Binary-Entry-Point. CLI-Flags: `--db-path`, `--provider`, `--ollama-url`, `--embed-model`, `--onnx-model-path`, `--read-only`, `--allow-write`. Stderr-only-Logging.
- `lib.rs` — `McpServer`. stdio-Leseschleife gehärtet: (a) Idle-Timeout (`MEMFUSE_MCP_IDLE_TIMEOUT_SECS`, Default 300s), (b) `read_line_bounded()` begrenzt Zeilenlänge (`MAX_RPC_BYTES`) mit JSON-RPC-Parse-Error `-32700` bei Überschreitung.
  - **Sechs MCP-Tools** (fünf bestehend + ein neues):
    1. `memfuse_search` — Hybrid Semantic Search (Vektor + BM25 + Graph).
    2. `memfuse_insert` — Dokument einspeichern.
    3. `memfuse_get` — Dokument per ID abrufen.
    4. `memfuse_collections` — Collections auflisten.
    5. `memfuse_consolidate` — manueller Konsolidierungstrigger.
    6. `memfuse_cloud_query` (neu, Feature `cloud-egress-guard`) — Sendet eine Query über das fünfschichtige Egress-Gateway (§12) an einen konfigurierten Cloud-Endpunkt. Nur aufrufbar wenn `cloud-egress-guard`-Feature aktiviert und `SandboxPolicy::allow_cloud_egress = true` (Default: false). Rückgabe enthält re-hydrierten Klartext nach Layer-5-Rücksubstitution und Prompt-Injection-Guard-Prüfung.
- `config.rs` — `EmbeddingConfig`, `is_write_allowed_by_env()`.
- `prompt_injection.rs` — Prompt Injection Guard: NFKC-Unicode-Normalisierung, Zero-Width-Character-Entfernung, rekursive Base64-Payload-Dekodierung (Tiefe 2), Homoglyphen-Normalisierung via `skeletonize_char()`-Mapping (kyrillische/griechische Homoglyphen). Quarantäne-Modi: Strict (redact), Escalate (log + redact), Passthrough.
- `sandbox.rs` — MCP-Sandbox: `execute_with_timeout()`, volatile Tool-Outputs AES-256-GCM-SIV-verschlüsselt im RAM, `SandboxPolicy` (Read/Write/CodeExecution/CloudEgress als separate Permissions, Default: nur Read). Für `CodeExecution`: Wenn Feature `wasm-sandbox` aktiv, delegiert `execute_with_timeout()` an `memfuse-sandbox::WasmExecutor`; ohne Feature ist `CodeExecution` auf Policy-Ebene blockiert (keine stille Prozess-Ausführung). Neu: `allow_cloud_egress: bool` (Default: false) in `SandboxPolicy`.
- `protocol.rs` — `McpError`-Enum.
- `egress_gateway.rs` (neu, Feature `cloud-egress-guard`) — Orchestriert die fünf Layer des Cloud-Egress-Privacy-Gateways (§12.2) für den `memfuse_cloud_query`-Tool-Aufruf. Gibt `GuardedPayload<Sanitized>` zurück, das von `dispatch.rs::dispatch_to_cloud()` konsumiert wird.

**Feature-Flags:** `default = []`, `agent-workflows`, `onnx`, `candle`, `kv-bridge`, `wasm-sandbox` (neu, aktiviert `memfuse-sandbox`-Abhängigkeit), `cloud-egress-guard` (neu), `test-utils`.

### §4.17 `memfuse-py` — Grenzschicht (isoliertes FFI-Workspace)

Python-Bindings via PyO3. Eigenes Cargo-Workspace mit `panic = "unwind"`. Abhängigkeiten: `memfuse-core`, `memfuse-db`.

**Öffentliche Python-API:** `open()` → `PyMemFuse`; `PyMemFuse` (Haupt-Facade); `PyCollection`; `memfuse_crud_methods!`-Makro; Statistik-Typen `PyDbStats` (inkl. Live-Observability-Felder `drift_status`, `calibration_ece`, `last_calibration_at`, `pid_pool_size`), `PyVectorIndexStats`, `PyStorageStats`.

**Zero-Copy-Strategie:** Eingabe-Vektordaten zero-copy aus NumPy-Arrays; FlatBuffer-Antworten als `PyBytes`.

### §4.18 `memfuse-sandbox` — Layer 6.5 (NEU)

WASM-Ausführungsgrenze für die `CodeExecution`-Permission. Reaktivierung der ursprünglichen `memfuse-sandbox`-Crate (archiviert in Commit `55a34647`) unter formalisiertem ADR (§9.2: dediziertes ADR vor Merge). Abhängigkeiten: `memfuse-core`, `memfuse-security`.

**Designgrundsatz:** `#![forbid(unsafe_code)]`. Alle Interaktionen mit `wasmtime` erfolgen über dessen safe Rust-API. Kein direkter `mmap`/`mlock`-Aufruf.

**Module:**
- `executor.rs` — `WasmExecutor`: zentrale WASM-Ausführungseinheit.
  - `fn execute(wasm_bytes: &[u8], input: &[u8], capabilities: &WasmCapabilities, timeout: Duration) -> Result<WasmOutput, SandboxError>`
  - Jeder `execute()`-Aufruf startet eine frische `wasmtime::Store` und `wasmtime::Instance` — kein Zustandsüberlauf zwischen Aufrufen.
  - `wasmtime::Config` mit aktiviertem `fuel`-Mechanismus (CPU-Limit) und `max_wasm_stack`-Konfiguration (Speicher-Limit).
  - Timeout: `wasmtime`s asynchrones `call_async()` kombiniert mit `tokio::time::timeout`.
- `capabilities.rs` — `WasmCapabilities`: strikte Whitelist für den WASM-Gast.
  ```rust
  pub struct WasmCapabilities {
      pub allow_stdout: bool,            // WASM darf auf stdout schreiben
      pub allow_stderr: bool,            // WASM darf auf stderr schreiben
      pub max_memory_pages: u32,         // Default: 16 (= 1 MB)
      pub max_fuel: u64,                 // CPU-Ticks, Default: 10_000_000
      pub allow_filesystem: bool,        // Default: false — kein Dateisystemzugriff
      pub allow_network: bool,           // Default: false — kein Netzwerkzugriff
      pub allow_clock: bool,             // Default: true — monotone Uhr erlaubt
  }
  impl Default for WasmCapabilities {
      fn default() -> Self { /* alle allow_*: false, max_memory_pages: 16 */ }
  }
  ```
  Das WASM-Guest-Modul hat keinen Zugriff auf MemFuse-interne Datenstrukturen, den Storage-Layer oder die Krypto-Primitiven — der WASM-Adressraum ist vollständig vom Host-Prozessraum isoliert.
- `output.rs` — `WasmOutput { stdout: zeroize::Zeroizing<Vec<u8>>, stderr: Vec<u8>, fuel_consumed: u64 }`. `stdout` ist `ZeroizeOnDrop`, damit keine Tool-Outputs im RAM verbleiben.
- `error.rs` — `SandboxError`-Enum: `Timeout`, `MemoryExceeded`, `FuelExhausted`, `CapabilityViolation`, `WasmTrap(String)`, `InvalidModule`.

**Integration mit `memfuse-mcp::sandbox.rs`:** Wenn Feature `wasm-sandbox` aktiv ist, erhält `McpSandbox` ein `Arc<WasmExecutor>`-Feld. `execute_with_timeout()` für `ToolCategory::CodeExecution` delegiert an `WasmExecutor::execute()`. Das WASM-Modul wird vom Aufrufer als `&[u8]` übergeben (vorab kompilierte `.wasm`-Datei); MemFuse selbst übersetzt keinen Quellcode.

**Wichtiger Scope-Hinweis:** `memfuse-sandbox` ist KEIN WASM-Compiler und kein WASM-Laufzeit-Ökosystem für Endnutzer. Es ist eine eng begrenzte, sicherheitsfokussierte Ausführungsgrenze für Agenten-Tools, die ein MCP-Client als vorab kompilierte WASM-Binaries bereitstellt. Der `CodeExecution`-Use-Case betrifft ausschließlich Tool-Implementierungen, die der MCP-Client-Entwickler selbst als WASM kompiliert und vertrauenswürdig bereitstellt — nicht Endnutzer-Code.

**Feature-Flags:** `default = []`, `wasm-sandbox` (schaltet `wasmtime`-Abhängigkeit ein; ohne dieses Feature existiert das Crate zwar im Workspace, aber `memfuse-mcp` bindet es nicht ein und `CodeExecution` bleibt auf Policy-Ebene blockiert).

---

## §5 — Systemweite Datenflüsse

### §5.1 Schreibpfad (`insert`/`upsert`)

`memfuse-mcp::memfuse_insert` oder `memfuse-py::insert()` → `memfuse-db::collection::crud` → Chunking → Embedding-Erzeugung → `memfuse-core::TxBuffer`-Staging → `memfuse-store::wal.rs` (WAL-First) → `memfuse-store::memtable.rs` → asynchron `memfuse-index::hnsw.rs` + `memfuse-text::bm25.rs`/`inverted.rs` + `memfuse-graph::csr.rs`.

### §5.2 Lesepfad (`search`/`hybrid_search`)

Anfrage → parallele Ausführung der vier Signale (HNSW, BM25, CSR/PathRAG, Metadaten-Filter) → `memfuse-db::fusion.rs` (RRF, Late Hydration, optionaler Kohärenz-Bonus) → optionales Cross-Encoder-Reranking → optionale Isotonic-Kalibrierung → Rückgabe. Bei Routing zusätzlich: Konfidenzbewertung, Drift-Check, `RoutingStrategy`-Dispatch (`Cascade` oder `ContextualBandit`).

### §5.3 Generierungspfad mit KV-Cache-Bridge

Retrieval-Ergebnisse → `LlmTextGenerator::generate_with_context()` → bei Candle + `kv-bridge`: `KvBridgeAdapter` konsultiert `TenantIsolatedKvStore` (Cache-Hit: entschlüsseltes KV-Segment injiziert, Prefill entfällt; Cache-Miss: voller Prefill, danach gecacht). Für Backends ohne Bridge: textuelle Segmentkoncatenation.

### §5.4 Konsolidierungspfad ("Sleep Cycle")

Zwei gegenseitig exklusive Mechanismen (P14, ADR-081): (a) `MaintenanceScheduler` (schwellenwertbasiert) und (b) `ConsolidationEngine` (periodisch, vollständiger Sleep-Cycle inkl. Generative Synthesis). Beide Pfade wenden anschließend via `consolidation_executor.rs` Tombstones und Graph-Cascade-Invalidierung an.

### §5.5 Export-/Importpfad

Export: `memfuse-db::export.rs` → `memfuse-export-v1.json` (Dokumente, Embeddings, `embedding_model`, `importance_score`, `relations`).
Import: `Collection::import_memories()` → schemaversionsgeprüfte, idempotente Upsert-Wiedereinspielung → `ImportSummary`.

### §5.6 Cloud-Egress-Pfad (optional, Feature `cloud-egress-guard`)

`memfuse_cloud_query` (MCP-Tool) → `McpServer` prüft `allow_cloud_egress`-Permission → `egress_gateway.rs` orchestriert Layer 1–4 → `GuardedPayload<Sanitized>` (Typ-State) → `dispatch.rs::dispatch_to_cloud()` → Cloud-API via `Transport::HttpCloud { url }` → Antwort → Layer 5 Re-Hydration + Prompt-Injection-Guard → re-hydrierter Klartext → Aufrufer.

### §5.7 WASM-Sandbox-Ausführungspfad (optional, Feature `wasm-sandbox`)

MCP-Client sendet `ToolCategory::CodeExecution` + `wasm_bytes` → `McpSandbox::execute_with_timeout()` → `WasmExecutor::execute()` (frische `wasmtime::Store`, `WasmCapabilities`-Check, `fuel`-Budget) → `WasmOutput { stdout: ZeroizeOnDrop, ... }` → Tool-Ergebnis AES-256-GCM-SIV-verschlüsselt in Volatile Vault → Antwort an MCP-Client.

### §5.8 Passives WAL-Shipping-Pfad (optional, Betriebskonzept, kein Code-Feature)

`memfuse-store::wal.rs::rotate_and_seal()` versiegelt abgeschlossene WAL-Segmente (atomar via `rename()`, danach `O_RDONLY`-Flag). Externer Sync-Daemon (Syncthing, iCloud Drive, oder optionaler `memfuse wal-push`-CLI-Befehl als One-Shot-HTTP-Push ohne dauerhaften Listener) repliziert nur versiegelte, read-only-Segmente auf eine passive Kopie. Die passive Kopie bleibt rein lesend, bis der Nutzer explizit ein manuelles Failover auslöst (`memfuse wal-restore`). Kein Konsens, kein Leader-Election, kein Split-Brain-Szenario möglich (exakt ein Schreiber zu jedem Zeitpunkt).

---

## §6 — Feature-Klassifikation (verbindlich)

**Kernfeatures (dauerhaft, produktdefinierend):** 4-Signal-RRF-Fusion inkl. PathRAG, Structural Consolidation Pass, Generative Synthesis Pass, DecayController, Immunologische Widerspruchsabwehr, Session-DAG mit `NodesGuard`/`ConsolidationNodesGuard`, WAL-HMAC-Kette, `DeletionProof`, MCP-Sandbox mit Prompt Injection Guard, Memory-Export-/Import-Format v1, KV-Cache-Bridge.

**Optionale Features (hinter Feature-Flag, dauerhaft unterstützt):**
- `graph-connectivity-health` — Graph-Connectivity/Perkolation.
- `edge-reinforcement-learning` — Hebbianisches Kanten-Reinforcement.
- `coherence-bonus-fusion` — Kohärenz-Bonus in RRF.
- `adaptive-candidate-pool-sizing` — Adaptive Kandidatenpool-Größe.
- `reranking` — Cross-Encoder-Reranking.
- `kv-bridge` — KV-Cache-Bridge für Candle-Inferenzpfad.
- `bandit-routing` (**neu**, Default: **off**) — Contextual-Bandit-Gate (`RoutingStrategy::ContextualBandit`, §13). Darf erst zum Default werden, wenn `test_bandit_vs_cascade_regret_comparison` offline auf historischen `RoutingOutcome`-Daten einen nachweisbaren Regret-Vorteil zeigt (P7).
- `cloud-egress-guard` (**neu**, Default: **off**) — Cloud-Egress Privacy Gateway (§12). Verstößt nicht gegen P5; adressiert Nutzer, die Cloud-LLMs ohnehin extern einsetzen.
- `wasm-sandbox` (**neu**, Default: **off**) — WASM-Ausführungsgrenze für `CodeExecution` via `memfuse-sandbox` (§4.18). Bewirbt erst nach erfolgreicher Integration `CodeExecution` als unterstützt.
- `egress-sherman-morrison` (opt-in, nur wenn `bandit-routing`) — Volles Sherman-Morrison-O(d²) statt Diagonal-O(d) für LinUCB.
- `experimental-diskann` — DiskANN-Basisimplementierung.

**Permanent verworfen (kein Zukunftsvorhaben):** Desktop-App, Replicator-Dynamics-Gewichtung, Voice-Assistant-Interface, Cross-Tenant-Wissensaustausch, dateisystembasiertes Claim-Locking, verteilte ADR-Dateien, `memfuse-cluster` (Raft/`openraft`).

**Dauerhaft gesperrt bis Architektur-Revision (VETO):** `partial-index-rebuild` (VETO-F02, conditionally_accepted bis 2026-10-07, täglich via `nucleation-recall-history.yml` überwacht); Cross-Tenant-Wissensaustausch (VETO-F10, permanent_rejected); Voice-Assistant (VETO-OP3, conditionally_accepted bis 2027-03-08).

---

## §7 — Sicherheits- und Datenschutzmodell

1. **Zero-Trust gegenüber Tool-Output:** Jeder zurückgegebene Inhalt gilt als untrusted.
2. **Prompt Injection Guard:** NFKC-Normalisierung, Zero-Width-Character-Entfernung, rekursive Base64-Dekodierung (Tiefe 2), Homoglyphen-Normalisierung (kyrillisch/griechisch). Modi: Strict/Escalate/Passthrough.
3. **Volatile-Vault-Verschlüsselung:** Tool-Outputs AES-256-GCM-SIV im RAM, `ZeroizeOnDrop`.
4. **KV-Cache-Bridge-Isolation:** Kryptographische Mandantenisolation via `(TenantId, ModelFingerprint)`-gebundenem HKDF-Sub-Schlüssel; faire LRU-Eviction; Emergency-Wipe.
5. **Integritätskette:** WAL-HMAC-Kette.
6. **Kryptographischer Löschnachweis:** `DeletionProof` (DSGVO Art. 17) mit `ExcludedScope`-Deklaration.
7. **Whitelist-Berechtigungsmodell:** Read/Write/CodeExecution/CloudEgress als getrennte Permissions; Default: ausschließlich Read.
8. **WASM-Ausführungs-Isolation** (Feature `wasm-sandbox`): `CodeExecution`-Permission-Tool-Code läuft in einer speicher-isolierten WASM-Instanz mit `WasmCapabilities`-Whitelist. Kein Dateisystem- und Netzwerkzugriff per Default. `WasmOutput.stdout` ist `ZeroizeOnDrop`. Ein WASM-Trap oder Timeout führt zu `SandboxError`; niemals zu stiller Prozess-Ausführung im Host-Kontext.
9. **Cloud-Egress Privacy Gateway** (Feature `cloud-egress-guard`, §12):
   - **Layer 1 — Deterministisches Token-Vaulting:** Aho-Corasick (O(n)) + Regex-Nachvalidierung für strukturierte PII; ONNX-NER (Reuse `memfuse-embed`) für unstrukturierte Entitäten. Surrogate: `[USER_ENTITY_<blake3(entity_text ‖ session_salt)[..4]>]`; `session_salt` = frischer `OsRng`-Wert. Speicher: `EgressVault` (§4.6, `ZeroizeOnDrop`).
   - **Layer 2 — Lokale Vorabstraktion:** `SegmentSynthesizer`-Trait im Modus `SynthesisMode::PrivacyAbstraction` (P10: Reuse). Bei Latenzbudget-Überschreitung: Fail-Open (übersprungen), nie Fail-Closed.
   - **Layer 3 — Graph-Generalisierung:** Kanten-Labels auf Community-Zugehörigkeit abstrahiert (`memfuse-graph::community.rs`). Fernziel: Echter $(\varepsilon,\delta)$-DP-Mechanismus mit Privacy-Budget-Ledger — explizit **kein Kernfeature**, eigenes ADR erforderlich, um keine Falsch-Compliance-Aussage zu erzeugen (P7).
   - **Layer 4 — `EgressGuard`:** Bulk-Exfiltrations-Erkennung via HNSW-k-NN (Reuse bestehender Index). Bei Ähnlichkeit ≥ Schwellenwert **und** Payload-Länge ≥ Mindestwert: Egress geblockt. **Fail-Closed** bei Guard-Fehler (Umkehrung der normalen Fail-Open-Philosophie — Datenexfiltrations-Risiko dominiert über Verfügbarkeit; bewusste, dokumentierte Asymmetrie).
   - **Layer 5 — Inbound Re-Hydration mit bidirektionalem Zero-Trust:** Rücksubstitution nur bei exaktem Format-Match (`[USER_ENTITY_[0-9a-f]{8}]`). Cloud-Antwort MUSS denselben Prompt-Injection-Guard durchlaufen wie `memfuse_search`/`memfuse_get`-Ergebnisse — Cloud ist eine untrusted Quelle.
10. **`dispatch.rs`-Hardening:** Subprozess-Aufruf via `Command::new(program).args(&[...])` ohne Shell-Zwischenschicht (kein `sh -c`), unabhängig von §12.
11. **Kein Multi-Tenant-Fremdkundenbetrieb:** `TenantId` dient ausschließlich Prozess-/Test-Isolation sowie kryptographischer KV-Cache-Trennung.

---

## §8 — Betriebsmodi

| Modus | Air-Gap-fähig | Ext. Prozess | KV-Cache-Bridge | WASM-Sandbox | Cloud-Egress |
|---|:---:|:---:|:---:|:---:|:---:|
| ONNX-Embedding (Default) | ✅ | ❌ | — | optional | — |
| Candle-Embedding/-Inferenz | ✅ | ❌ | ✅ (opt-in) | optional | — |
| Ollama-Embedding/-Inferenz | ❌ | ✅ (Ollama) | ❌ | optional | — |
| MCP stdio + Zero-Trust-Sandbox | ✅ | ❌ | — | optional (`wasm-sandbox`) | optional (`cloud-egress-guard`) |
| Cloud-Egress-Gateway | ❌ (bewusst) | ❌ (kein Listener) | — | optional | ✅ (opt-in) |

**Benchmark-Strategie:** Vergleichsrahmen gegen Mem0, Zep/Graphiti, VelesDB über LongMemEval- und LoCoMo-Datensätze. Benötigte, noch nicht erstellte Benchmarks (P7): `bench_kv_bridge_prefill_savings`, `stress_session_dag_deadlock_freedom`, `test_bandit_vs_cascade_regret_comparison`.

---

## §9 — Governance & Entwicklungsprozess

### §9.1 Säulen des Entwicklungssystems

1. **Preflight Gate:** zentraler Aggregator aller lokalen und CI-Gates vor jedem Commit/PR.
2. **Anti-Collision Claim System:** `cargo xtask claim --crate X --issue Y` mit TTL-Expiry und Cross-Branch-/Open-PR-Abgleich via GitHub-API (P18).
3. **Single Source of Truth:** `DECISIONS.md` (ADRs); `WORKING_STATE.md` autogeneriert. Code-Kommentare verweisen auf `DECISIONS.md`-ADR-Nummern.
4. **Automatisierte CI-Guardrails — vollständige, mergeblockierende Gate-Katalog:**
   - **Architektur-Integrität:** `check-dag`, `check-duplicate-symbols`, `check-type-registry`.
   - **Prozess-/Kollisions-Prävention:** `check-duplicate-intent` (inkl. Cross-Branch-/Open-PR-Abgleich, P18).
   - **Qualitäts-Trend-Tracking:** `check-unwrap-baseline-trend`, `check-recall-stability`, Mutation-Testing, Benchmark-Regressions-Gates.
   - **Audit-Integrität:** `check-audit-verdict-independence` — **MUSS** in `merge-gate.yml` als mergeblockierender Required-Check verankert sein (aktuell nur in `scheduled-audit.yml`; dies ist eine offene Lücke die vor dem nächsten Release zu schließen ist). `check-audit-duplication`.
   - **Governance-Frische:** `check-vetoes` (gegen `VETOES.md`), `check-adr-deadlines`, `check-stale-tags`, `check-jules-context-freshness`, `check-agents-integrity`.
   - **Grenzschicht-Sicherheit:** `check-ffi-panic-boundary` (aktuell `continue-on-error: true`, MUSS auf `continue-on-error: false` hochgestuft werden per TODO in `merge-gate.yml`).
   - **Formale Konsistenz:** `check-commit-messages` (Shell-Commit-Unterbindung für `lsm.rs`, `wal.rs`, `fusion.rs`, `prompt_injection.rs`, `dispatch.rs`), `check-workflow-commands`, `check-doc-references`, `check-placeholder-refs`, `check-phantom-files`.
   - **NEU — Dispatch-Shell-Hardening:** `check-dispatch-no-sh-c` — prüft via `grep -rn 'Command::new("sh")' crates/memfuse-router/src/dispatch.rs`, dass kein `sh -c`-Muster in `dispatch.rs` vorkommt. Mergeblockierend.
   - **NEU — Bandit-Latenz-Safety:** `check-bandit-latency-budget` — stellt sicher, dass `BanditImplementation::ShermanMorrison` ausschließlich als opt-in via `egress-sherman-morrison`-Feature-Flag aktivierbar ist und nicht als Default-Pfad. Prüft dass `routing_strategy.rs` das Diagonal-Approximation-Default korrekt wählt.
   - Jedes neu eingeführte Gate MUSS zwingend auch in der CI-Workflow-Konfiguration als mergeblockierender Schritt verankert werden — ein implementiertes, aber nicht eingebundenes Gate erfüllt seinen Zweck nicht.
5. **Prompter & Bootstrap Protocol:** `AGENTS.md`, `.jules/SESSION_BOOTSTRAP.md` unüberspringbar. `CONSTITUTION.md` on-demand.
6. **Governance-Dokumentenkatalog:** `AGENTS.md`, `DECISIONS.md`, `WORKING_STATE.md`, `VETOES.md`, `CONSTITUTION.md`, `SECURITY.md`, `TESTING.md`.

### §9.2 Zwei-Stufen-Entwicklungsprozess

- **Stufe 1 (Orchestrator):** liest Repository-Zustand, prüft Architektur/ADRs, trifft Entscheidungen, verfasst Task-Spezifikationen — schreibt keinen Produktionscode.
- **Stufe 2 (Ausführender Agent):** Mandatory Bootstrap, Crate-Claim, Preflight-Gate, exakte Umsetzung. Keine eigenständigen ADRs; bei Bedarf `ADR-VORSCHLAG:` im PR-Body.

**ADR-Pflicht für neue Komponenten aus §4–§5 dieser Spezifikation:**
- `memfuse-sandbox`-Reaktivierung: eigenes ADR (ADR-082 oder folgendes) mit Scope-Abgrenzung und `wasmtime`-Begründung, vor Merge.
- `bandit-routing`-Feature: ADR nach Verifikation via `test_bandit_vs_cascade_regret_comparison`.
- `cloud-egress-guard`-Feature: ADR mit Privacy-Threat-Model-Verweis auf `SECURITY.md`.
- WAL-Rotation-und-Shipping-API: ADR mit expliziter TOCTOU-Risikobewertung für externe Sync-Daemons.

### §9.3 Sprache

Deutsch für interne Governance-Dokumentation; Englisch für Code-Kommentare und öffentliche API-Dokumentation.

### §9.4 Lizenz & Contributions

MIT OR Apache-2.0. Vollständig Open Source.

### §9.5 Crate-Zielstruktur

```
KERN (6 Module, Fusionsziel):
  memfuse-core         [Fundament, Traits, Domain-Typen]
  memfuse-security     [Fusion: memfuse-crypto]
  memfuse-persistence  [Fusion: memfuse-store + memfuse-checkpoint]
  memfuse-retrieval    [Fusion: memfuse-index + memfuse-graph + memfuse-text]
  memfuse-orchestrator [memfuse-db, Scheduler-Konsolidierung]
  memfuse-inference    [Fusion: memfuse-calibration + memfuse-ollama + memfuse-candle + memfuse-router + memfuse-embed]

GRENZSCHICHT:
  memfuse-mcp          [unverändert]
  memfuse-py           [unverändert]
  memfuse-agentic      [memfuse-agent]
  memfuse-sandbox      [WASM-Ausführungsgrenze, separater Layer 6.5]

WERKZEUG:
  xtask, memfuse-bench [unverändert]
```

Migrationsreihenfolge: Security → Persistence → Inference → Scheduler → Retrieval → Bereinigung. Diese Konsolidierung ist nachgelagert und darf Release-Kriterien (§10) nicht verzögern.

---

## §10 — Abnahmekriterien des Endprodukts

Das System gilt als abnahmefertig (Release-reif), wenn **sämtliche** der folgenden Kriterien erfüllt sind:

1. `cargo test --workspace` besteht fehlerfrei mit 0 Regressionen.
2. `uvx memfuse-mcp` startet fehlerfrei gegen einen frischen `~/.memfuse`-Pfad und beantwortet eine `memfuse_search`-Anfrage ohne laufende Ollama-Instanz (ONNX-Default).
3. `pip install memfuse` installiert fehlerfrei auf Linux x86_64, macOS arm64 und Windows x86_64; `memfuse.open()` → `insert()` → `search()` funktioniert ohne externe Abhängigkeiten.
4. Dimension- und Versionsnummer sind zwischen `memfuse-py`, `memfuse-db` und `Cargo.toml` durchgängig konsistent (`dimension=768`).
5. `DeletionProof` (`memfuse-crypto`) erbringt den negativen Rekonstruktionstest.
6. `cargo xtask check-dag` bestätigt 0 Layer-Verletzungen (inkl. `memfuse-sandbox` auf Layer 6.5).
7. Ein Git-Tag markiert das Release; alle für `memfuse-tauri` spezifischen CI-Workflows sind entfernt oder deaktiviert.
8. `MemFuseDb::stats()` liefert durchgängig Live-Daten aus dem Router-/Kalibrierungszustand (`drift_status`, `calibration_ece`, `last_calibration_at`, `pid_pool_size`) via `Weak<RouterEngine>::upgrade()` (ADR-080) statt Platzhalterwerten.
9. `MaintenanceScheduler` und `ConsolidationEngine` sind für jede Collection nachweislich gegenseitig exklusiv aktiv: `consolidation_guard: Arc<tokio::sync::Mutex<()>>` mit `try_lock`-Semantik produktiv verdrahtet (ADR-081).
10. Die KV-Cache-Bridge ist über Feature `kv-bridge` durchgängig von `memfuse-crypto` bis `memfuse-mcp` verdrahtet, mit textidentischem Verhalten bei deaktiviertem Feature und nachgewiesener Tenant-Isolation unter Nebenläufigkeit.
11. `AGENTS.md`/`WORKING_STATE.md` sind tagesaktuell zum letzten Code-Stand.
12. **NEU:** `dispatch_to_slm()` in `memfuse-router::dispatch.rs` verwendet ausschließlich `Command::new(program).args(&[...])` ohne `sh -c` (verifiziert durch `check-dispatch-no-sh-c`-Gate).
13. **NEU:** `check-audit-verdict-independence` ist als mergeblockierender Required-Check in `merge-gate.yml` verankert (nicht nur in `scheduled-audit.yml`); `check-ffi-panic-boundary` läuft mit `continue-on-error: false`.
14. **NEU (Feature-Gates):** Wenn Feature `wasm-sandbox` aktiviert, besteht `memfuse-sandbox::executor::tests::test_wasm_memory_isolation` und `test_wasm_fuel_exhaustion_returns_error` fehlerfrei. Wenn Feature `cloud-egress-guard` aktiviert, besteht `test_egress_guard_fail_closed_on_index_unavailable` fehlerfrei (Fail-Closed-Verifikation). Wenn Feature `bandit-routing` aktiviert, besteht `test_bandit_diagonal_vs_linucb_latency_budget` fehlerfrei (Latenz-Budget-Verifikation).
15. **NEU (P7-Benchmarks):** `bench_kv_bridge_prefill_savings` und `stress_session_dag_deadlock_freedom` sind in `memfuse-bench` vorhanden und zeigen messbare Ergebnisse; entsprechende USP-Aussagen in README/PyPI sind nicht mehr `[BENCHMARK_PENDING]`.
16. WAL-Rotation-API (`Wal::rotate_and_seal()`) ist implementiert, read-only-Flagging ist durch `test_wal_rotate_and_seal_readonly_guarantee` verifiziert.

---

## §11 — Glossar

- **RRF (Reciprocal Rank Fusion):** Fusion mehrerer Ranglisten unterschiedlicher Retrieval-Signale (`memfuse-db::fusion.rs`).
- **PathRAG:** Graph-basierte Retrieval-Methode mit bidirektionalem Dijkstra für Multi-Hop-Fragen (`memfuse-graph::path_rag.rs`).
- **SAOS (Synthesized Agent Operating System):** Sammelbegriff für die typsichere Multi-Signal-Query- und Kontextfenster-Repräsentationsschicht in `memfuse-core`.
- **ConfigFingerprint:** Hash-Fingerabdruck über Modell-/Kalibrierungsparameter zur automatischen Invalidierung veralteter Statistiken.
- **ModelFingerprint:** SHA-256-basierter Fingerabdruck über Modellgewichte und Quantisierungsstufe; Grundlage der KV-Cache-Schlüsselableitung.
- **KV-Cache-Bridge:** Mandantenisolierte, verschlüsselte Wiederverwendung berechneter LLM-KV-Tensoren über Retrieval-Treffer hinweg (`memfuse-crypto::kv_segment`, `memfuse-candle::kv_bridge`).
- **DeletionProof:** Kryptographischer Nachweis, dass gelöschte Daten auf Storage-Ebene nicht mehr rekonstruierbar sind (`memfuse-crypto::deletion_proof.rs`, DSGVO Art. 17).
- **Lyapunov-Drift-Watcher:** Statistisches Verfahren zur proaktiven Erkennung von Verteilungsverschiebungen in Kalibrierungs-Scores (`memfuse-router::lyapunov.rs`). Wird event-driven direkt nach jeder Routing-Entscheidung aufgerufen (ADR-079).
- **Session-DAG/`NodesGuard`:** Typsichere Datenstruktur zur Abbildung verzweigter Konversationen mit Compile-Time-Deadlock-Prävention (`memfuse-graph::session_dag.rs`).
- **`ConsolidationNodesGuard`:** Vom `NodesGuard`-Typ abgeleitete Hülle für den Konsolidierungspfad; schließt die zuvor offene Deadlock-Freiheitslücke zwischen Session-DAG und `consolidation_executor.rs`.
- **Structural Consolidation Pass:** Deterministischer, LLM-freier Teil des Sleep-Cycle (`memfuse-db::memory_consolidation.rs`).
- **Generative Synthesis Pass:** LLM-basierter Teil des Sleep-Cycle, erzeugt `SynthesizedChunk`s (`memfuse-db::synthesis_phase.rs`).
- **Memory-Export-/Import-Format v1:** Versioniertes, portables JSON-Format je Collection mit idempotentem Re-Import (`memfuse-db::export.rs`/`import.rs`). Macht MemFuse zum System ohne Vendor-Lock-in.
- **EmbeddingBackend:** Konfigurations-Enum mit Varianten Onnx (Default), Ollama, Candle, None.
- **Late Hydration:** Architekturmuster in `memfuse-db::fusion.rs`, das Metadaten-Deserialisierung auf Top-K-Treffer beschränkt.
- **WAL-Flusher-Actor:** Dedizierte Hintergrund-Task, die WAL-Batches koaleziert.
- **Lock-Sharding:** Partitionierung in cache-line-ausgerichtete Shards zur Contention-Vermeidung.
- **PidLatencyController:** Latenzbudget-Regler für Multi-Step-Retrieval (implementiert in `memfuse-db::pid_latency_controller.rs` und in `multistep.rs` via `MultiStepEngine` verdrahtet) und Cloud-Egress-Layer-2 (§12.2.2).
- **`RoutingStrategy`:** Enum (`Cascade` [Default] | `ContextualBandit`) in `memfuse-router`, Feature `bandit-routing`. Wählt zwischen deterministischem Schwellenwert-Kaskaden-Klassifikator und lernfähigem LinUCB-Bandit.
- **LinUCB:** Linear Upper Confidence Bound — kontextueller Bandit-Algorithmus (Li et al., 2010) für adaptives Profil-Routing. Implementiert in zwei Varianten: diagonal O(d) (Default, kein BLAS) und Sherman-Morrison O(d²) (opt-in via `egress-sherman-morrison`-Feature).
- **Sherman-Morrison-Formel:** Rang-1-Update für Matrix-Inverse: $A^{-1} \leftarrow A^{-1} - \frac{A^{-1}xx^\top A^{-1}}{1 + x^\top A^{-1}x}$, O(d²) — vermeidet O(d³)-Inversion im Hot-Path.
- **`GuardedPayload<Sanitized>`:** Typ-State in `memfuse-router::guarded_payload`, der nur durch erfolgreichen Durchlauf aller fünf Egress-Guard-Layer konstruierbar ist. Macht einen vergessenen Guard-Durchlauf zum Compile-Fehler.
- **`EgressGuard`:** Bulk-Exfiltrations-Erkennungskomponente (Layer 4 des Cloud-Egress-Gateways, §12.2.4). Bewusst anders benannt als `NodesGuard` (der Lock-Ordnungs-Typ), da beide komplett verschiedene Semantik haben (Symbolkollision wäre P10-Verstoß).
- **Graph-Generalisierung:** Deterministisches $k$-Anonymitäts-Verfahren auf Graphkanten (Community-Abstrahierung), eingesetzt in Layer 3 des Cloud-Egress-Gateways. Kein Differential-Privacy-Mechanismus.
- **`Transport`-Enum:** `StdioMcp` (Default, bestehend) | `HttpCloud { url }` (neu, nur mit `cloud-egress-guard` + `GuardedPayload<Sanitized>`). Feld in `SlmProfile`.
- **Passives WAL-Shipping:** Replikation versiegelter, read-only-geflaggter WAL-Segmente auf eine passive Kopie ohne Konsensprotokoll, Leader-Election oder Split-Brain-Risiko. Exakt ein Schreiber; passive Kopie rein lesend bis zu explizitem manuellem Failover (`memfuse wal-restore`).
- **`memfuse-sandbox`:** WASM-Ausführungsgrenze für `CodeExecution`-Permission. Layer-6.5-Crate. Nutzt `wasmtime` mit `fuel`-Limit und `WasmCapabilities`-Whitelist. `#![forbid(unsafe_code)]`. Kein Dateisystem-/Netzwerkzugriff per Default.
- **`WasmCapabilities`:** Capability-Whitelist für WASM-Guest-Module in `memfuse-sandbox`. Definiert erlaubte I/O-Kanäle, Speicherlimit, CPU-Ticks und Netzwerk-/Dateisystemzugriff.
- **`SynthesisMode`:** Enum (`SleepCycle` | `PrivacyAbstraction`) für den `SegmentSynthesizer`-Trait, das die Wiederverwendung desselben Traits für Sleep-Cycle-Konsolidierung und Cloud-Egress-Layer-2 ermöglicht.

---

## §12 — Cloud-Egress Privacy Gateway (optional, Feature `cloud-egress-guard`)

### §12.1 Designprinzipien und Vorbedingungen

Dieses Feature ist **standardmäßig deaktiviert** und verletzt P5 (Kein Cloud-Zwang) **nicht**: Es adressiert Nutzer, die Cloud-LLMs (z. B. Gemini, Claude via API, GPT) *ohnehin* einsetzen, und bietet technische Schadensbegrenzung für eine Realität, die außerhalb von MemFuse bereits existiert. Air-Gap bleibt der Standardzustand.

**Namenskorrektur (zwingend vor Implementierung):** Die Bulk-Exfiltrations-Erkennungskomponente heißt `EgressGuard`, **nicht** `NodesGuard` (dieser Name ist für den Session-DAG-Lock-Ordnungs-Typ kanonisch reserviert, §4.5). Eine Verwechslung würde `cargo xtask check-duplicate-symbols` sofort scheitern lassen.

**DAG-Position:** `memfuse-router::egress_guard` (Layer 6) — benötigt `memfuse-db` (Layer 5), `memfuse-security` (Layer 2) und optional `memfuse-candle` (Layer 3). Rein additiv: bei deaktiviertem Feature identisches Verhalten zum bestehenden System.

### §12.2 Fünfschichtige Spezifikation

#### §12.2.1 Layer 1 — Deterministisches Token-Vaulting

**Zwei parallele Erkennungspfade (kein ReDoS-Risiko durch klare Aufgabentrennung):**

1. **Strukturierte PII** (E-Mail, IP, API-Key-Muster, Kreditkarten-Luhn-Check):
   - Aho-Corasick-Multi-Pattern-Automat, worst-case O(n) über den Payload unabhängig von Musterzahl.
   - Reguläre-Ausdruck-Nachvalidierung **nur** auf den Aho-Corasick-Treffern (nicht auf dem Gesamttext — verhindert ReDoS).
2. **Unstrukturierte Entitäten** (Personen, Organisationen):
   - Wiederverwendung der ONNX-Runtime aus `memfuse-embed` (P10) für ein quantisiertes Token-Classification-NER-Modell.
   - Kein zweiter Inferenzpfad, keine neue Abhängigkeit.

**Sitzungsstabiles Surrogat:** Jede erkannte Entität → `[USER_ENTITY_<blake3(entity_text ‖ session_salt)[..4 Byte, hex]>]`.
- `session_salt` = frischer `OsRng`-Wert pro Session. Gleiche Entität → gleiches Surrogat innerhalb einer Session (Cloud-LLM kann kohärent referenzieren); verschiedene Sessions nicht korrelierbar.
- **Speicherung:** `EgressVault` in `memfuse-security::egress_vault` (AES-256-GCM-SIV, `ZeroizeOnDrop`).

#### §12.2.2 Layer 2 — Lokale Vorabstraktion

- **P10-konform:** `SegmentSynthesizer`-Trait (produktiv für Sleep-Cycle) im Modus `SynthesisMode::PrivacyAbstraction` — keine Parallelstruktur.
- **Latenzbudget-Kopplung:** Derselbe `PidLatencyController` wie für Multi-Step-Retrieval (§4.13/P11). Bei Überschreitung: **Fail-Open** (Layer 2 übersprungen, Payload fließt zu Layer 3) — niemals Fail-Closed, da Verfügbarkeit hier nicht hinter Sicherheit zurücksteht; PII wurde bereits in Layer 1 deterministisch tokenisiert.

#### §12.2.3 Layer 3 — Graph-Generalisierung

**Standard-Mechanismus (Default):** Deterministische $k$-Anonymitäts-Generalisierung:
- Kantenlabels → Community-Zugehörigkeit (`memfuse-graph::community.rs`, produktiv).
- Spezifische Entity-Namen → Layer-1-Surrogate.
- Kein Rauschmechanismus, keine Budget-Buchführung.

**Fernziel — Echter $(\varepsilon,\delta)$-DP (explizit kein Kernfeature dieser Version):**
Ein echter DP-Mechanismus (Laplace-Mechanismus auf PageRank-/Zentralitäts-Scores mit kalibrierter Sensitivität) benötigt einen **Privacy-Budget-Ledger** (stateful, pro Tenant/Session, mit Kompositions-Tracking über wiederholte Anfragen). Diese Komponente ist **nicht** in v5 enthalten — eigenes ADR und eigenes Release erforderlich, um keine falsche Compliance-Aussage zu erzeugen (P7: DSGVO-Auditoren, die "Differential Privacy" hören, erwarten beweisbares $\varepsilon$).

#### §12.2.4 Layer 4 — `EgressGuard`

**Aufgabe:** Bulk-Chunk-Exfiltrations-Erkennung (nahezu wörtliche Wiedergabe eines gespeicherten Memory-Chunks im ausgehenden Payload). **Nicht** PII-Erkennung — das ist deterministisch Layer 1 überlassen.

**Algorithmus:**
- Ausgehender Payload → Embedding via bestehender `memfuse-embed`/HNSW-Infrastruktur (P10: kein Zweitmodell).
- k-NN-Anfrage gegen lokalen HNSW-Index (kein Vollscan — das ist genau der Anwendungsfall des Index).
- Blockierung wenn: `max(cosine_similarity) ≥ threshold` **und** `payload_length ≥ min_payload_bytes` (verhindert False Positives bei kurzen, generischen Fragmenten).
- **Fail-Closed** bei Guard-Fehler (Index nicht verfügbar, Timeout): Egress wird blockiert, nicht durchgelassen. Bewusste Asymmetrie zum KV-Bridge-Fail-Open-Muster (§4.8) — hier dominiert Exfiltrationsrisiko über Verfügbarkeit.

#### §12.2.5 Layer 5 — Inbound Re-Hydration mit bidirektionalem Zero-Trust

- **Reverse-Mapping:** Rücksubstitution aus `EgressVault` **nur** bei exaktem Format-Match (`[USER_ENTITY_[0-9a-f]{8}]`) — kein unscharfes Pattern-Matching, um Prompt-Injection durch manipulierte Cloud-Antworten zu verhindern.
- **Bidirektionaler Zero-Trust:** Cloud-Antwort MUSS denselben Prompt-Injection-Guard-Pfad durchlaufen wie `memfuse_search`/`memfuse_get`-Ergebnisse (§7.2). Cloud ist eine externe, untrusted Quelle — dies gilt für beide Richtungen der Kommunikation.

### §12.3 Transport-Erweiterung (`Transport`-Enum)

`SlmProfile` erhält additives `transport: Transport`-Feld (Default: `StdioMcp`):
```rust
pub enum Transport {
    StdioMcp,                   // bestehend, unverändert
    HttpCloud { url: String },  // neu, nur mit cloud-egress-guard-Feature
}
```
`dispatch_to_cloud()` darf den `HttpCloud`-Zweig **ausschließlich** aufrufen, wenn `GuardedPayload<Sanitized>` als Typ-State vorliegt. Dies ist ein **Compile-Fehler**, wenn der Guard-Durchlauf fehlt — keine Laufzeitprüfung, kein Flag-Check.

**Unabhängiges Hardening (§7.10, kein cloud-egress-guard nötig):** `dispatch_to_slm()` MUSS `Command::new(program)` ohne Shell-Zwischenschicht verwenden. Dies ist ein separates, sofort umzusetzendes Hardening, unabhängig vom Cloud-Egress-Feature-Gate.

### §12.4 Nicht-Ziel-Klarstellung

Dieses Feature macht aus einer privatsphäreschützenden Sicht das Senden von Daten an Cloud-LLMs sicherer — es macht es nicht *empfohlen*. Die MemFuse-Default-Empfehlung bleibt Air-Gap (P5). Nutzer, die dieses Feature aktivieren, tun dies in vollem Bewusstsein, dass sie MemFuse-Daten an externe Dienste senden.

---

## §13 — Router-Neuausrichtung: Contextual-Bandit-Gate (Feature `bandit-routing`)

### §13.1 Ist-Zustand (normativ dokumentiert)

`RouterEngine::select_profile_cascade()` ist ein deterministischer **Kaskaden-Klassifikator** (Profile nach `min_relevance_score` sortiert, Schwellenwert-Check pro Profil, erster Treffer gewinnt). Er ist gut kalibriert (Isotonic/Platt, konformale Quantile), produktiv und getestet. Er bleibt der **unveränderliche Default** — `RoutingStrategy::Cascade` ist und bleibt die einzig aktivierte Strategie bis `bandit-routing` explizit per ADR und Benchmark freigegeben wird (P7, P20).

**Bekannte Einschränkungen des Kaskaden-Klassifikators:** keine gelernte, kontinuierliche Gating-Funktion über Query-Kontext; kein Mechanismus gegen Profil-Verhungern (ein Profil mit niedrigem Schwellenwert kann systematisch unterbeschäftigt sein); kein explizites Exploration/Exploitation-Gleichgewicht; `RoutingOutcome` wird ausschließlich für Kalibrierungs-Statistik, nicht als Belohnungssignal genutzt.

### §13.2 Zielarchitektur: LinUCB Contextual Bandit

**Designprinzipien:**
1. **Kein zweiter Embedding-Aufruf (P10):** Gating-Kontext = bereits berechneter Query-Embedding-Vektor aus `memfuse-embed`/`memfuse-candle`. Kein Performance-Overhead für die Embedding-Berechnung selbst.
2. **Latenz-sicherer Algorithmus:** Die Spezifikation mandatiert **zwingend** eine O(d²)-oder-besser-Implementierung. Die naive O(d³)-Inversion von $A_p$ im Hot-Path ist **verboten** — dies würde das `PidLatencyController`-Budget bei d=768 unweigerlich sprengen.
3. **Bestehendes Belohnungssignal:** `RoutingOutcome::Success = 1.0`, `Rejected = 0.0`, `Escalated { .. } = 0.3` (konfigurierbar).
4. **Bestehende Kapazitätskontrolle:** `PidLatencyController`-Muster (§4.13).
5. **Bestehende Drift-Kopplung:** `LyapunovDriftWatcher` (ADR-079).

**Score-Formel:**
$$\hat{r}_p(x) = \theta_p^\top x + \alpha_p \sqrt{\Sigma_p(x)} - \lambda \cdot c_p - \mu \cdot \mathbb{1}[\text{transport} = \text{HttpCloud}]$$

Dabei ist $\Sigma_p(x)$ je nach `BanditImplementation`:
- **Diagonal (Default):** $\Sigma_p(x) = \sum_i \sigma_{p,i}^2 \cdot x_i^2$ — O(d), kein BLAS.
- **Sherman-Morrison (opt-in):** $\Sigma_p(x) = x^\top A_p^{-1} x$ via Matrixvektor-Produkt — O(d²), `ndarray`.

**Updates (beide Varianten) bei Beobachtung $(x, r_{\text{adjusted}})$:**
- $\theta_p \leftarrow \theta_p + \eta \cdot (r_{\text{adjusted}} - \theta_p^\top x) \cdot x$ — O(d).
- Diagonal: $\sigma_{p,i}^2 \leftarrow \sigma_{p,i}^2 + x_i^2$ — O(d).
- Sherman-Morrison: $A_p^{-1} \leftarrow A_p^{-1} - \frac{A_p^{-1} x x^\top A_p^{-1}}{1 + x^\top A_p^{-1} x}$ — O(d²), keine Inversion.

**Belohnungsanpassung:**
$$r_{\text{adjusted}} = r_{\text{outcome}} - \lambda \cdot c_p - \mu \cdot \mathbb{1}[\text{transport} = \text{HttpCloud}]$$
- $c_p$ = `resource_cost_estimate` (Feld in `SlmProfile`, bereits vorhanden).
- $\mu$ = Privacy-Malus für Cloud-Profile (Default: 0.2, konfigurierbar). Cloud-Profile gewinnen nur, wenn ihr Qualitätsvorteil den Privacy-Malus überkompensiert — lokal-bevorzugendes Verhalten ist direkt im Reward kodiert.
- $\lambda$ = Kostensensitivitäts-Parameter (Default: 0.1, konfigurierbar).

**Drift-gekoppelte Exploration:** Bei `LyapunovDriftWatcher` → Drift für Profil $p$: $\alpha_p \leftarrow \alpha_p \cdot k_{\text{drift}}$ temporär erhöht (Default: $k_{\text{drift}} = 2.0$, konfigurierbar; Rückfall auf Basis-$\alpha$ nach Drift-Auflösung).

**Kapazitätsbeschränkung via PID:** Profil $p$ überschreitet Auslastungsbudget → temporärer Strafterm auf $\hat{r}_p(x)$ bis Fensterverschiebt (Wiederverwendung des `PidLatencyController`-Feedbacks auf die Zustandsgröße Profil-Auslastung).

**`ndarray`-Abhängigkeit:** Nur für `ShermanMorrison`-Variante und nur wenn Feature `egress-sherman-morrison` aktiv. Bezogen via re-export aus `memfuse-embed/onnx` (`ndarray 0.15`, bereits im Workspace vorhanden). Die Diagonal-Variante benötigt ausschließlich `Vec<f32>` — keine zusätzliche Abhängigkeit, kein BLAS.

### §13.3 Migrationsstrategie (P20-konform)

- Feature-Flag `bandit-routing` (Default: **off**). Kaskaden-Pfad bleibt **unverändert bestehend**.
- `RoutingStrategy`-Enum: `Cascade` (Default) | `ContextualBandit`.
- **Verifikationspflicht vor Default-Aktivierung:** `test_bandit_vs_cascade_regret_comparison` — Offline-Replay-Benchmark auf historischen `RoutingOutcome`-Daten in `memfuse-bench`. Dieser Benchmark MUSS einen messbaren Regret-Vorteil des Bandits gegenüber der Kaskade zeigen, bevor `ContextualBandit` je zum Default wird (P7). Kein Wechsel ohne Benchmark-Nachweis.
- **Latenz-Sicherheits-Test:** `test_bandit_diagonal_vs_linucb_latency_budget` verifiziert, dass der Diagonal-Default das `PidLatencyController`-Budget unter d=768 nicht überschreitet.

---

## §14 — Reaffirmation: `memfuse-cluster`-Veto & Passives WAL-Shipping

### §14.1 Das Veto bleibt bestehen (zeit-invariant)

Das Veto gegen `memfuse-cluster` (Raft/`openraft`) ist **permanent und gilt unverändert**. Kein Umfang an verfügbarer Implementierungsgeschwindigkeit (Commit-Tempo, Agenten-Schwarm) ändert die zugrunde liegende Risikoklasse:

1. **Verifikationsschulden sind qualitativ anders:** Split-Brain-Szenarien, Leader-Election-Liveness und verteilte Lock-Semantik sind in der Verifikations-Literatur (Jepsen-Framework) die Fehlerklasse, die spezialisierte Teams (etcd, Consul, CockroachDB) noch Jahre nach Produktivsetzung als Incident-Ursache finden. Mehr Code-Durchsatz bei gleicher oder überlasteter Audit-Kapazität **erhöht** dieses Risiko.
2. **P2 (Zero-Panic-Doctrine):** Ein fehlerhaftes verteiltes Konsensprotokoll ist der Prototyp eines Silent-Data-Corruption-Risikos — im Unterschied zu einem lokalen LSM-Bug betrifft ein Split-Brain-Fehler per Definition mehrere Knoten gleichzeitig und ist oft nicht mehr rekonstruierbar.
3. **Markenkern-Begründung ist zeit-invariant:** Das Air-Gap-/Zero-Trust-Versprechen, die Nicht-Ziel-Liste (§2.4) und die Behauptung "kein Netzwerk-Angriffsvektor, weil kein Netzwerk-Listener existiert" sind keine Funktion der Entwicklungsgeschwindigkeit.
4. **P19:** Revision eines dokumentierten Beschlusses erfordert explizites ADR — kein Ad-hoc-Umkehr.

### §14.2 Tatsächlicher Nutzungsbedarf: Passives WAL-Shipping

**Kurzfristig (bereits vorhanden):** Memory-Export-/Import-Format v1 (§2.3 Punkt 7) deckt Batch-Snapshots für Migration/Restore vollständig ab.

**Mittelfristig — Passives WAL-Shipping (neues, risikoarmes Feature):**

Das durch HMAC-Kette integritätsgesicherte WAL (§4.12) wird periodisch auf eine passive Kopie repliziert — **kein Konsens, kein Leader-Election, kein Split-Brain** möglich, da zu jedem Zeitpunkt exakt ein Schreiber existiert und die Kopie rein lesend bleibt bis zu explizitem manuellem Failover.

**Implementierung** (`memfuse-store::wal.rs`):
- `Wal::rotate_and_seal()`: Atomares Versiegeln eines WAL-Segments (rename + fsync auf Directory, danach `O_RDONLY`-Flag). Nur versiegelte Segmente sind für externen Sync freigegeben.
- **TOCTOU-Schutz gegen externe Sync-Daemons (Syncthing, iCloud Drive):** Der Flusher-Actor schreibt ausschließlich in das aktive, nicht-versiegelte Segment. Externe Sync-Daemons lesen ausschließlich versiegelte, read-only-geflaggite Segmente. Datei-Locking-Konflikte zwischen Flusher und Sync-Daemon sind strukturell ausgeschlossen — nicht durch Locking-Protokolle, sondern durch die Invariante "Flusher und Sync-Daemon schreiben/lesen nie dasselbe Segment gleichzeitig". `test_wal_rotate_and_seal_readonly_guarantee` verifiziert diese Invariante.
- Optionaler `memfuse wal-push`-CLI-Befehl: One-Shot-HTTP-Push eines versiegelten Segment-Batches ohne dauerhaften Listener — der Server läuft nur während des Push-Vorgangs.
- `memfuse wal-restore`: Manuelles, explizites Failover einer passiven Kopie zum Primär-Gerät.

**Langfristig (Fernziel, eigenes ADR):** CRDT-artige, versionsvektor-basierte Delta-Merges für asynchrone Zusammenführung zweier unabhängig geschriebener Instanzen. Geeigneter für das tatsächliche Nutzungsmuster (Handy + Laptop, beide offline geschrieben, Merge wenn online). Wird **nicht** in v5 aufgenommen (P20).

### §14.3 Invarianten des Passiven WAL-Shippings

- Exactly-one-writer: Zu jedem Zeitpunkt existiert genau eine schreibende Instanz.
- Passive Kopie: Rein lesend bis zu explizitem `memfuse wal-restore`.
- HMAC-Integrität: Jedes geshippte Segment trägt die vollständige HMAC-Kette; die passive Instanz verifiziert die Kette beim Replay.
- Kein Automatismus: Failover ist ein manueller, bewusster Entschluss des Nutzers — kein automatischer Failover-Mechanismus, der Split-Brain provozieren könnte.
- Keine neue Netzwerkfläche im Steady-State: MemFuse öffnet keinen dauerhaften Netzwerk-Listener für Shipping-Zwecke.

---

*Diese Spezifikation ist die einzige normative Produktquelle für MemFuse v5. Sie beschreibt den vollständigen Zielzustand des Systems und wird bei jeder architektonisch relevanten Änderung aktualisiert. Sie ersetzt `MEMFUSE_ENDPRODUKT_SPEZIFIKATION_FINAL_4.md` als maßgebliches Referenzdokument.*

---

## Anhang A — Delta zu v4 (Kurzübersicht)

Folgende normative Änderungen wurden gegenüber v4 vorgenommen:

| Abschnitt | Änderungstyp | Inhalt |
|---|---|---|
| §2.3 | Erweiterung | WASM-Sandbox als USP #3 (ersetzt Verschlüsselungs-Sandbox); Cloud-Egress-Gateway als neues USP #8 |
| §2.4 | Ergänzung | Explizites Cluster-Veto in Nicht-Ziele aufgenommen |
| §3 P2 | Präzisierung | `memfuse-sandbox`-unsafe-Freiheitsgarantie |
| §3 P7 | Ergänzung | Explizite Benchmark-Pflicht für KV-Bridge, Session-DAG, LinUCB (§13) |
| §3 P10 | Ergänzung | Konkrete Reuse-Mandate für §12 (Aho-Corasick, ONNX, HNSW) und §13 (ndarray) |
| §4.6 | Erweiterung | `egress_vault.rs` (Feature `cloud-egress-guard`); Feature `cloud-egress-guard` |
| §4.9 | Ergänzung | Adaptiver DiskANN-Flush-Threshold (ADR-076) |
| §4.11 | Ergänzung | `ndarray 0.15` Re-Export für §13 |
| §4.12 | Ergänzung | `Wal::rotate_and_seal()` für passives WAL-Shipping (§14) |
| §4.13 | Ergänzung | `cloud-egress-guard`-Feature-Flag |
| §4.14 | Erweiterung (neu) | `RoutingStrategy`-Enum, `bandit.rs`, `transport.rs`, `guarded_payload.rs`, `routing_strategy.rs`; `dispatch.rs`-Hardening (kein `sh -c`) |
| §4.16 | Erweiterung | 6. MCP-Tool `memfuse_cloud_query`; `allow_cloud_egress`-Permission; `egress_gateway.rs`; `wasm-sandbox`-Feature |
| §4.18 | **NEU** | `memfuse-sandbox`-Crate (WASM-Ausführungsgrenze, Layer 6.5) |
| §5.6 | **NEU** | Cloud-Egress-Pfad |
| §5.7 | **NEU** | WASM-Sandbox-Ausführungspfad |
| §5.8 | **NEU** | Passives WAL-Shipping-Pfad |
| §6 | Erweiterung | Features `bandit-routing`, `cloud-egress-guard`, `wasm-sandbox`, `egress-sherman-morrison` |
| §7 | Erweiterung | Punkte 8 (WASM), 9 (Cloud-Egress-Layers 1–5), 10 (dispatch-Hardening), 11 (Cloud-Egress-Nicht-Ziel) |
| §8 | Erweiterung | Neue Tabellenspalten für WASM-Sandbox und Cloud-Egress |
| §9.1 | **Korrektur/Ergänzung** | `check-audit-verdict-independence` als Merge-Gate (war nur scheduled); `check-ffi-panic-boundary` auf `continue-on-error: false`; neue Gates `check-dispatch-no-sh-c`, `check-bandit-latency-budget` |
| §10 | Ergänzung | Abnahmekriterien 12–16 (neu) |
| §11 | Ergänzung | Glossar-Einträge: `ConsolidationNodesGuard`, `EgressGuard`, `GuardedPayload`, `RoutingStrategy`, `LinUCB`, `Sherman-Morrison`, `Transport`, passives WAL-Shipping, `memfuse-sandbox`, `WasmCapabilities`, `SynthesisMode` |
| §12 | **NEU** | Cloud-Egress Privacy Gateway (vollständige normative Spezifikation) |
| §13 | **NEU** | Contextual Bandit Router mit latenz-sicherem LinUCB (Diagonal-Default O(d), Sherman-Morrison opt-in O(d²)) |
| §14 | **NEU** | Cluster-Veto-Reaffirmation & Passives WAL-Shipping mit TOCTOU-Schutz |
