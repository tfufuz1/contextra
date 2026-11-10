# Contextra als Baukasten: Entkopplungs- und Entnahmeplan

Stand: 9. November 2026 · Grundlage: `tfufuz1/contextra`, HEAD `80f1c33`, 36 Workspace-Mitglieder, rund 424.000 Zeilen Rust (inkl. Tests) · Methode: ausschließlich statische Analyse (Cargo.toml, `use`-Pfade, Modulbäume, Datei-Diffs). Es wurde nichts gebaut, installiert oder getestet. Alle Zahlen stammen aus Skript-Zählungen über den Quelltext; wo es Heuristiken sind, steht das dabei.

## 1. Kurzfassung

Das Repo ist um zwei Hub-Crates (`contextra-types`, `contextra-ports`) herum gebaut. Die vorhandenen „Klasse-A“- und „compat“-Mechanismen sollten Bausteine entkoppeln, tun das aber nur auf dem Papier. Für das Ziel „Code entnehmen und in einem anderen Projekt einsetzen“ stehen zehn belegbare Hindernisse im Weg (Abschnitt 2). Die wichtigsten:

- **Hub-Kopplung:** 24 von 35 Crates deklarieren `contextra-types`, 20 deklarieren `contextra-ports`. Der gemeinsame Fehlertyp `ContextraError` hat 43 Varianten, darunter Sandbox, Cluster, KV-Quantisierung und HNSW-Reparatur.
- **Scheinbare Entkopplung:** `vector`, `graph`, `kvcache`, `checkpoint` und `mvcc` importieren `contextra_types`/`contextra_ports`/`contextra_mvcc` überwiegend ohne `cfg`-Gate, obwohl die Abhängigkeiten in `Cargo.toml` teils optional sind. Ein Standalone-Build ohne Default-Features ist damit statisch nicht plausibel.
- **Mehrfachdefinitionen:** `DocId` existiert in 8 Crates, `Clock` in mindestens 10 Dateien, `Rng` in 5 Crates, `DistanceMetric` in 3.
- **Löschwesen und Lizenz im Kern:** `Purger`/`LayerScanner` werden von store, vector, text, graph, kvcache, crypto und testkit importiert. `FeatureRing`/`LicenseGate` stecken in `store/lsm/config.rs`, `engine`, `kvcache` und der Fassade.
- **Crates ohne Verwender:** `durable-fs`, `simd`, `ingest`, `kvcache`, `erasure`, `license`, `audit-export`, `avv-generator`, `agent`, `cognition` werden von keinem anderen Crate eingebunden. `durable-fs` und `sys::durability` überlappen, `simd` wird von `vector` nicht benutzt (`vector/src/distance.rs` hat eine eigene Skalar-Implementierung), `ingest` überlappt `engine/chunker`.

Ich empfehle fünf Entscheidungen (Details in Abschnitt 4):

1. **Zwei neue Mikro-Kerne** ohne interne Abhängigkeiten: `contextra-ids` (DocId, TxId, TenantId …) und `contextra-env` (Clock, Rng, IdGen, MetricsSink). Sie ersetzen die Mehrfachdefinitionen.
2. **Jeder Baustein bekommt genau eine Brücke** `src/compat/`, hinter dem Feature `workspace` (nicht in `default`). Nur dort stehen `contextra_types`/`contextra_ports`. Der Kern kennt sie nicht.
3. **Jeder Baustein hat seinen eigenen Fehlertyp.** `ContextraError` bleibt nur Integrationsvokabular.
4. **Compliance-Crates bleiben kompilierbar** (Gruppe `compliance/`, Tier frozen) statt in `archive/` zu wandern. Das weicht von Entscheidung D1 deines Plans ab (Abschnitt 3, K6).
5. **`engine`, `db`, `contextra` sind Referenz-Verdrahtung**, kein Baustein. Entnehmbare Teile (chunker, filter, temporal\_filter, Experimente) werden herausgelöst.

## 2. Befunde mit Beleg

### F1 – Hub-Kopplung

- Deklarierte Abhängigkeit auf `contextra-types`: contextra, agent, checkpoint, cognition, crypto, db, engine, infer-candle, mcp, mvcc, ports, py, router, store, testkit (nicht optional) sowie audit-export, graph, infer-ollama, kvcache, license, rank, simd, text, vector (optional). Das sind 24, nicht die 30, die `types/src/lib.rs` Zeile 20–28 behauptet.
- `contextra-ports` wird von 20 Crates deklariert (6 davon optional).
- `types/src/error.rs` (1.162 Zeilen) enthält `ContextraError` mit 43 Varianten, u. a. `Sandbox`, `MemoryLimitExceeded`, `SandboxTimeout`, `Cluster`, `KvDeleteModeConfig`, `KvQuantization`, `HnswConnectivityDegraded`, `GraphRepairFailed`, `PinBudgetExceeded`, `WalTruncationDetected`, `CrossDeviceLink`, `Plugin`, `ModelLoad`. Jeder Baustein, der den Typ nutzt, erbt das Vokabular aller anderen.
- `types` enthält außerdem Domänenfremdes: `erasure.rs` (616 Z.), `witness.rs` (97 Z.), `tombstone.rs` (158 Z., gehört zu MVCC), `model_fingerprint.rs` (79 Z., gehört zum KV-Cache), `retrieval_strategy.rs` (101 Z., Router), `TlHfdParams`/`PprConfig`/`PprAlgorithm` in `types/domain/misc.rs` ab Zeile 375 (Graph-Experimente in einem Ring-0-Crate).

### F2 – `ports` mischt fünf Anliegen

`ports` hat 6.763 Zeilen in Top-Level-Dateien plus `scanner/`. `lib.rs` Zeilen 16–22 re-exportiert zusätzlich `ErasureId`, `GateId`, `SagaRecord`, `ScanMethod` aus `types`. Inhalt nach Anliegen:

- **Index- und Speicher-Ports:** `storage.rs`, `vector_index.rs`, `text_index.rs`, `graph_index.rs`, `graph.rs`, `embedding.rs`, `reranker.rs`, `query_rewriter.rs`, `checkpoint.rs`, `kv.rs`, `kv_bridge_port.rs`, `attention.rs`, `lifecycle.rs`, `observability.rs`, `plugin.rs`.
- **Umwelt:** `clock.rs`, `rng.rs`, `id_gen.rs`, `metrics.rs`, `vfs.rs` (+ Fassaden `env.rs`, `fs.rs`).
- **Löschwesen:** `erasure.rs`, `erasure_legacy.rs`, `gate.rs`, `ledger.rs`, `observer.rs`, `report_signer.rs`, `keys.rs`, `barrier.rs`, `scanner/*`.
- **Lizenz:** `license.rs` (enthält auch `VectorDeleteMode`, Zeilen 1–17, eine reine Vektorindex-Einstellung).
- **Testinfrastruktur:** `conformance.rs` (278 Z.), `contract_tests.rs` (632 Z.).

Verifiziert: Die Symbole von `erasure`, `gate`, `ledger`, `observer`, `report_signer`, `keys`, `barrier` werden außerhalb von `ports`, `types` und `contextra-erasure` nirgends benutzt. Diese Dateien sind also gefahrlos verschiebbar. Das gilt nicht für `scanner/*` und `license.rs` (F5).

### F3 – Das „compat“-Muster ist nicht durchgesetzt

Neun Crates deklarieren `contextra-types`/`-ports` als optionale Abhängigkeit hinter `types-compat` oder `ports-compat` (`graph`, `vector`, `text`, `rank`, `kvcache`, `simd`, `infer-ollama`, `license`, `audit-export`). Gezählt wurden Referenzen auf `contextra_types|ports|mvcc|sys|crypto` in `src/`-Dateien (ohne Tests), die in einer Datei ohne irgendein compat-`cfg` stehen. Das ist eine Untergrenze, denn eine Datei mit einem einzigen `cfg` gilt schon als gegatet:

- `vector`: 114 von 188 Referenzen, 32 Dateien. Beispiele: `hnsw/types.rs` Zeile 5 (`use contextra_types::{ContextraError, DocId, Result}`), 128–143, 203 (`contextra_mvcc::SequenceLog`), 208–210 (`dyn contextra_ports::Rng/Clock/MetricsSink`); `quantize/mod.rs` Zeile 19 und 72–110 (liefert `contextra_types::Result`).
- `graph`: 134 von 144 Referenzen, 50 Dateien (u. a. `purger.rs`, `provider.rs`, `hyperedge_suggest.rs`).
- `kvcache`: 55 von 69, 9 Dateien (`radix.rs`, `prefix_store.rs`, `eviction_worker.rs`, `scratchpad_invalidation.rs`).
- `checkpoint`: 72 von 72, 7 Dateien (kein Feature, nur harte Abhängigkeit). `mvcc`: 11 von 11, 5 Dateien (ebenfalls hart).
- Ausnahmen, die wirklich nahe am Ziel sind: `rank` (0 ungegatete Referenzen, `local_types.rs` mit 19 cfg-Gates), `simd` (0), `text` (8 Referenzen in 4 Dateien: `stream.rs`, `inverted.rs`, `inverted/purge.rs`, `inverted/morph_index.rs`).

Der Docstring von `vector/src/types.rs` Zeile 3–6 behauptet einen autarken Baustein ohne `contextra-ports`, `contextra-types` oder `contextra-mvcc`. Das stimmt für die Typen in dieser Datei, nicht für den Rest des Crates.

Außerdem prüft das Export-Werkzeug nichts Entscheidendes: `tools/export_crate.py` (Docstring, Abschnitt „Bekannte Grenzen“) klammert interne `contextra-*`-Abhängigkeiten im Export aus. Ein grünes Gate G3 beweist damit keine Entnehmbarkeit.

### F4 – Dieselben Konzepte mehrfach definiert

- `DocId`: `types/types/domain/ids.rs:173/180`, `vector/types.rs:18/25`, `text/types.rs:9/15`, `graph/types.rs:58`, `rank/local_types.rs:10`, `ingest/chunking/types.rs:17`, `erasure/types_local.rs:1628`.
- `TxId`: `types`, `vector/types.rs:97`, `graph/types.rs:104`, `erasure/types_local.rs:1665`.
- `Clock`: `ports/clock.rs:8`, `vector/types.rs:169` und `vector/contract.rs:39`, `text/contract.rs:16`, `graph/contract.rs:15`, `adapt/clock.rs:9` und `adapt/contract.rs:9`, `rank/calibration/isotonic.rs:14`, `privacy/types.rs:11`, `license/types.rs:122`, `erasure/types_local.rs:2059`.
- `Rng`: `ports/rng.rs:12`, `vector/types.rs:203` und `contract.rs:45`, `text/contract.rs:19`, `graph/contract.rs:17`, `adapt/rng.rs:6` und `contract.rs:17`.
- `DistanceMetric`: `types/.../document.rs:20`, `vector/types.rs:130`, `simd/metric.rs:5`. `MetricsSink`: `ports/metrics.rs:19`, `vector/types.rs:245`.

Die Mehrfachdefinitionen sind die Folge des Versuchs, Bausteine durch Kopieren zu entkoppeln. Sie sparen die Abhängigkeit, erzwingen aber `From`-Konvertierungen (z. B. `kvcache/src/compat.rs`) und bergen Drift-Risiko. Ob die Trait-Signaturen der `Clock`-Kopien identisch sind, habe ich nicht verglichen; das ist ein Pflicht-Schritt in WP-1 (Abschnitt 6).

### F5 – Löschwesen und Lizenz im Kern

- `ports::scanner` (`Purger`, `LayerScanner`, `ScanTarget`, `ScanSession`, `WitnessLayer`, `ScanTechnique`, `PurgeStats`) wird importiert von `store` (u. a. `scanner/*`), `vector` (`hnsw/purge.rs`, `hnsw/scanner.rs`), `text` (`inverted/purge.rs`), `graph` (`purger.rs`), `kvcache`, `crypto` und `testkit`. Das ist mechanische Löschung (Tombstone-Purge). Sie bleibt als Fähigkeit erhalten, gehört aber in die Brücke, nicht in den Kern.
- `FeatureRing` und `LicenseGate` stecken in `store/src/lsm/config.rs:76–92` (`validate_against_features`, bereits `#[deprecated]`), `engine/src/lib.rs:17,22,373,410`, `engine/src/contextra_impl/lifecycle.rs:9–55`, `kvcache/src/adapters.rs:15,135–221`, `contextra/src/builder.rs:33–35,243–350`, `contextra/src/performance_profile.rs` (32 Treffer) und `contextra/src/collection_profile.rs:33,128`.
- `VectorDeleteMode` liegt in `ports/license.rs` und wird von `engine/lib.rs:22,246,341` re-exportiert.

### F6 – `crypto` ist verflochten und teilweise Dublette von `erasure`

- Interne Kette (per `use crate::…` belegt): `crypto.rs` (`KeyManager`) → `anti_tamper` (Zeile 45) und `kv_cipher::ModelFingerprint` (241, 295, 306); `kv_cipher.rs` → `kv_shredding::KeyRegistry` (Zeilen 107–220, `cfg(legacy-erasure)`); `kv_shredding` → `revocation_log`; `kv_segment/manager.rs:17` → `kv_shredding::KeyRegistry`; `audit_chain` → `kv_shredding`; `deletion_proof/*` → `audit_chain`, `ed25519_proof`; `wal_crypto` → `crypto`.
- Externe Nutzer: `store` nur `wal_crypto::{WalHmac, IntegrityVerifier, WalEntrySnapshot}` (`wal/encode.rs:5`, `wal/io.rs:5`, `wal/replay.rs:5`), `crypto::KeyManager` (`wal/mod.rs:343`) und `verify_wal_chain_completeness` (`lsm/recovery.rs:373`). `kvcache` nutzt `CryptoKey`, `EncryptedKvLayer`, `KvSegmentCipher`, `ModelFingerprint`, `CryptoError` (`segment.rs:25,41`, `store.rs:561–598`, `service.rs:39`). `infer-candle/kv_bridge.rs:13–16,304` nutzt `KvSegment`, `TenantIsolatedKvStore`, `EvictionWorker`. `mcp/sandbox.rs` nutzt `CryptoKey`.
- `audit_chain`, `deletion_proof`, `ed25519_proof`, `kv_shredding`, `revocation_log` haben keinen externen Nutzer, und `contextra-erasure` enthält eigene Dateien gleichen Namens (`deletion_proof.rs`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`). Die `#[deprecated]`-Hinweise in `crypto/src/lib.rs` sagen „migriert nach contextra-erasure“. Die Defaults `legacy-erasure` und `kv-encryption` (setzt `legacy-erasure`) schalten die Altlast trotzdem ein.
- Folge: `kv_segment` und `KvSegmentCipher` hängen für Verschlüsselung und Crypto-Shredding am Altmodul `kv_shredding`. Das ist nicht einfach zu archivieren.

### F7 – Drei Dateisystem-Schichten, eine ohne Verwender

- `contextra-durable-fs` (1.910 Zeilen src) hat keinen Verwender. Es enthält `flush_file`, `flush_dir`, `rename_durable`, `DurableHandle`, `DirLock`, `journal`, `scrub`, `fault_injector` und einen eigenen `Vfs`-Trait (`vfs.rs`: `read`, `write_atomic`, `append`, `sync_file`).
- `contextra-sys/src/durability.rs` (471 Z.) implementiert `durable_flush_file`, `durable_flush_dir`, `DurableFile` mit `libc::fcntl(F_FULLFSYNC)` (Zeile 102) und wird von `store` genutzt. `sys/lib.rs` Zeile 19 behauptet, `durable-fs` baue darauf auf; die Cargo.toml von `durable-fs` zeigt keine solche Abhängigkeit.
- `ports/vfs.rs` (404 Z.) definiert einen dritten `Vfs`-Trait (`open` → `VfsFile`), `store/src/io_seam.rs` die Implementierung `RealFs`, `testkit/fault_vfs.rs` die Fehlerinjektion. `store/src/fsync.rs` hat nochmals einen eigenen `sync_file`.

### F8 – Verwaiste und doppelte Bausteine

- `ingest` (29.792 Z.) hat keinen Verwender und eine eigene `DocId`; `engine/src/chunker/` (1.533 Z., nur 1 Referenz auf `types`) erfüllt dieselbe Rolle wie `ingest/src/chunking/`.
- `adapt` (7.938 Z., null interne Abhängigkeiten, bereits Standalone) hat Parallelen: `decay_controller.rs` (255 Z.) vs. `engine/src/decay_controller.rs` (192), `homeostat.rs` (232) vs. `db/src/homeostat.rs` (58), `pid_latency_controller.rs` (309) vs. `db/src/pid_latency_controller.rs` (134), `drift.rs` (322) vs. `rank/src/drift.rs` (137). Die Diffs (242–395 Zeilen) zeigen: keine reinen Kopien. Eine Konsolidierung braucht einen Semantikvergleich.
- `mcp/src/bulk_exfiltration_detector.rs`, `egress_gateway.rs`, `egress_guard.rs` sind dünne Adapter (4/28/54 Zeilen) über `privacy`. Das ist sauber und bleibt.

### F9 – Experimentelles: Ist-Zustand

- `graph`: `default = ['types-compat', 'ports-compat', 'experimental-graph']`, also ist `session_dag` (1.008 Z.) standardmäßig an. `hyperedge` (1.183 Z., 19 Inbound-Dateien, von `cognition` und `engine` genutzt), `community` (1.463 Z.), `path_rag` (1.676 Z., 14 Inbound-Dateien, `apprh` hängt daran), `tl_hfd` (862 Z.), `entity_extraction` (318 Z.) und `consistency_enforcement` (481 Z., 6 Inbound-Dateien) sind ohne Feature kompiliert.
- `vector`: `diskann` (12 Dateien, 4.168 Z.) ist gegatet, hat nur `lib.rs` als Inbound und hängt ausgehend an `hnsw` (5), `quantize` (3), `distance` (3), `contextra_sys::mmap_readonly` und dem `VectorIndex`-Port. `quantize_rabitq` (523 Z.) ist ein Blatt. `acorn` (213 Z.) wird von `hnsw/acorn_filtered.rs` und `engine` genutzt.
- `rank`: `dibud` (517 Z., Feature, Blatt), `drift` (137 Z., ungegatet, Inbound `fusion/signal.rs`), `resonance` ist `fusion/resonance.rs` (90 Z., 3 Inbound in `fusion/`), kein Top-Level-Modul.
- `router`: `fc_ts_dispatch` (92 Z.), `routing_strategy` (72 Z.), `transport` (92 Z.) sind gegatete Blätter.
- `engine`: `extraction/` (642 Z.), `background_workers/` (1.298 Z., ausgehend 21× vector, 16× store, 10× graph, 8× text), `decay_controller.rs` (5 Inbound, von `db` genutzt).
- `types` trägt Experiment-Spuren: `TlHfdParams`, `PprConfig`, `PprAlgorithm`, `AutoExtractionMode`, `MemoryImportance`/`DecayFunction`.

### F10 – Metadaten und Dokumentation sind nicht belastbar

- `[package.metadata.contextra]` mit `ring = 0` bis `4` und `class`/`klasse` ist uneinheitlich (z. B. `privacy`: `ring = 3`, Beschreibung „Klasse A“).
- Doc-Behauptungen widersprechen dem Code (30 Verwender von `types`, `ingest` als Verwender, `sys` ↔ `durable-fs`).
- `PLAN.md`, `AGENTS.md` und `docs/` enthalten Agenten-Prompts (Autonomie- und Drohtext). Das deckt sich mit WP-10 deines Plans.

## 3. Abgleich mit `RESTRUCTURE_PLAN.md`

**Übernehmen:** Leitplanken 1–7 (Verschieben statt Löschen, ein WP ein Commit, kein Umbau im Verschiebe-Commit); das Tier-Modell; den Isolationsvertrag E1–E7; WP-10, WP-13, WP-14, WP-15 (Aufräumen von Prompt-Resten, Doku, Tools, CI); WP-44 bis WP-47 (Katalog, Standalone-Check, Entnahme-Test, neue AGENTS.md).

**Korrigieren (jeweils mit Beleg):**

- **K1 – WP-11, Annahme „kein Stable-Code nutzt die Lizenz-/Erasure-Ports“:** Für die Erasure-Familie stimmt das (F2). Für `license.rs` stimmt es nicht (F5), und `VectorDeleteMode` muss vor dem Verschieben nach `vector`.
- **K2 – WP-12, Kandidatenliste für `crypto`:** `kv_segment` wird von `infer-candle` genutzt, `anti_tamper` von `KeyManager`, und `kv_cipher::ModelFingerprint` von `KeyManager` und `kvcache`. Die echten Kandidaten sind `audit_chain`, `deletion_proof`, `ed25519_proof`, `revocation_log`, `kv_shredding` (Dubletten in `erasure`), aber `kv_segment/manager.rs` und `kv_cipher.rs` binden `kv_shredding::KeyRegistry` ein (F6). WP-12 und WP-40 widersprechen sich außerdem selbst (WP-12 archiviert `kv_segment`, WP-40 will KV-Teile bündeln).
- **K3 – WP-11/12 „types/ports verschlanken“ greift zu kurz:** Die Kopplung kommt aus Hub-Semantik, Mehrfachdefinition und ungegateten Imports (F1, F3, F4), nicht allein aus Compliance-Resten.
- **K4 – Tier-Tabelle:** `graph` als beta ist mit dem Default-Feature `experimental-graph` nicht haltbar; `vector` als stable passt nicht zu F3; `durable-fs` und `simd` als stable passen nicht zu „kein Verwender“. Ich stufe sie auf beta bzw. experimental, bis ein Verwender existiert.
- **K5 – WP-30-Kandidaten:** `rank::resonance` ist ein Untermodul von `fusion` (Klasse B/C), `graph::hyperedge` ist Klasse C (19 Inbound-Dateien), `vector::acorn` ist Klasse B, `vector::quantize` ist geteilt und nicht experimentell, die `router`-Kandidaten sind bereits gegatete Blätter (Klasse A).
- **K6 – D1 (Compliance ins Archiv):** `erasure` ist bereits Standalone (keine `contextra-*`-Abhängigkeit, eigene `types_local.rs` mit 2.264 Zeilen und `crypto_local.rs`). `license` und `audit-export` haben nur optionale `ports`/`types`-Abhängigkeiten, `avv-generator` hat keine. Ein Verschieben nach `archive/` kostet die Kompilierbarkeit und gewinnt nichts. Empfehlung: Gruppe `compliance/`, Tier frozen, `cargo check` wöchentlich. Die Entscheidung bleibt bei dir.
- **K7 – Fehlend:** ids/env-Mikro-Kerne, Fehlerstrategie, `compat`-Isolation je Crate (F3), Dateisystem-Konsolidierung (F7), `ingest`/`chunker`-Dedup, `adapt`-Parallelen und ein ehrlicher Standalone-Nachweis statt `export_crate.py` mit ausgeklammerten Abhängigkeiten.

## 4. Zielarchitektur und Regeln

### 4.1 Gruppen und Verzeichnislayout

Nur Pfade ändern sich, Paketnamen bleiben (`contextra-*`); die beiden neuen Kerne heißen `contextra-ids` und `contextra-env`.

- `crates/foundation/`: **ids** (neu), **env** (neu), `sys`, `durable-fs`, `simd`, `wire`, `crypto`, `mvcc`
- `crates/contracts/`: `types`, `ports` (Integrationsvokabular, kein Baustein)
- `crates/storage/`: `store`
- `crates/retrieval/`: `vector`, `text`, `graph`, `rank`, `checkpoint`
- `crates/llm/`: `kvcache`, `router`, `adapt`, `infer-ollama`, `infer-candle`, `infer-onnx`
- `crates/data/`: `ingest`
- `crates/safety/`: `privacy`, `sandbox`
- `crates/agentic/`: `agent`, `cognition`
- `crates/compliance/`: `erasure`, `license`, `audit-export`, `avv-generator`
- `crates/integration/`: `engine`, `db`, `contextra`
- `crates/apps/`: `mcp`, `py`
- `crates/testing/`: `testkit`; `benchmarks/contextra-bench` bleibt

### 4.2 Ebenen und Abhängigkeitsbudget

- **Ebene 0 (Blätter, keine internen Abhängigkeiten):** ids, env, sys, durable-fs, simd, wire, adapt, sandbox, privacy, ingest, erasure, avv-generator, infer-onnx. Ziel: `cargo tree -p <crate> -e normal` zeigt kein `contextra-*`. Heute stimmt das schon für `wire`, `sys`, `durable-fs`, `adapt`, `sandbox`, `privacy`, `ingest`, `erasure`, `avv-generator`, `infer-onnx`.
- **Ebene 1 (Bausteine):** dürfen nur von Ebene 0 abhängen, höchstens vier interne Closure-Mitglieder: mvcc, crypto, vector, text, graph, rank, checkpoint, kvcache, store, infer-ollama, infer-candle.
- **Ebene 2 (Integration):** `types`, `ports`, `router` (→ adapt), `engine`, `db`, `contextra`, `agent`, `cognition`, `license`, `audit-export`.
- **Ebene 3 (Anwendungen):** `mcp`, `py`, `testkit`, `bench`.

### 4.3 Regeln

- **R1 – Richtung:** Abhängigkeiten zeigen nur nach unten. Experimentell → Stabil, nie umgekehrt (entspricht E1).
- **R2 – Eine Brücke pro Crate:** `contextra_types` und `contextra_ports` dürfen nur in `src/compat/` vorkommen, hinter dem Feature `workspace` (nie in `default`). Dort liegen sämtliche Trait-Implementierungen auf `ports`-Traits (z. B. `hnsw/vector_index_impl.rs`, `diskann/vector_index_impl.rs`, `text/inverted/purge.rs`, `graph/purger.rs`, `store/scanner/*` und die `StorageEngine`-Implementierung von `store`) und alle `From`-Konvertierungen. Vorlagen im Repo: `kvcache/src/compat.rs`, `vector/src/types.rs:279ff`, `vector/src/error.rs:61–91`.
- **R3 – Eigene Fehler:** Jeder Baustein hat `<Name>Error` mit `thiserror` (`VectorError` existiert in `vector/error.rs`). Konvertierung nach `ContextraError` nur in `compat/` (die Orphan-Regel ist erfüllt, weil der Baustein-Fehler lokal ist).
- **R4 – Ein Besitzer pro Konzept:** IDs → `ids`; Clock/Rng/IdGen/MetricsSink → `env`; Distanzmetrik → `simd`; Tombstone → `mvcc`; ModelFingerprint → `kvcache`; Ppr-/TlHfd-Parameter → `graph`.
- **R5 – Experimente:** E1–E7 deines Plans plus **E8**: Die Naht (`ext.rs`) liegt im Host-Kern ohne `types`/`ports`; die Experimente implementieren sie. Ein Experiment mit mehr als ca. 2.000 Zeilen, eigenen externen Abhängigkeiten oder Nutzung in mehreren Hosts wird eigenes Crate (Kriterium K).
- **R6 – Nachweis statt Behauptung:** Ein Baustein gilt als entnehmbar, wenn (a) `cargo tree -e normal` nur die erlaubte Closure zeigt, (b) `rg 'contextra_(types|ports)' src --glob '!compat/**'` leer ist, (c) `cargo check --no-default-features` gelingt und (d) ein Export ohne Ausklammern interner Abhängigkeiten in einem leeren Verzeichnis baut. Diese Prüfungen führst du später aus; ich habe sie nicht ausgeführt.

## 5. Spezifikation pro Crate

Jeder Eintrag nennt Ist-Zustand (mit Beleg), Schnitt (Dateien und Komponenten), Ziel-Closure und die statische Prüfung, die du später ausführst. Pfade sind relativ zu `crates/<crate>/src/`, sofern nicht anders angegeben. Kein Schnitt ändert Verhalten; jeder Schritt ist ein Verschieben, Hinter-ein-Feature-Stellen oder Ersetzen einer Dublette durch ihren Besitzer.

### 5.1 foundation

#### contextra-ids (neu)

- **Quelle:** `types/src/types/domain/ids.rs` (514 Z.: `DocId` mit 64 oder 128 Bit über `docid-128`, `EntityId`, `TxId` mit `INTERNAL_BASE`, `TenantId` mit `SYSTEM`, `CollectionId`; `from_key` per BLAKE3-8-Byte-Präfix), `types/src/ids.rs` (213 Z.: `FileId`, `StoreId`, `Seq`, `Timestamp`), `types/src/refs.rs` (`CollectionRef`, `DocKey`, `OperatorRef`, `SubjectRef`, `TenantRef`, `MAX_REF_LEN`), `types/src/tenant_scope.rs` (`TenantScoped`, `TenantScopeViolation`), Tests `types/.../tests/ids_tests.rs` (473 Z.). Dazu die Basiskonstanten `TOMBSTONE_BIT` und `MAX_SEARCH_K` aus `types/.../misc.rs`, weil `text`, `store`, `vector`, `graph` sie unabhängig von `types` brauchen. `ErasureId` und `GateId` (heute in `types/src/ids.rs`) gehen nach `compliance/erasure`.
- **Schnitt:** Die Validierung nutzt heute `ContextraError::InvalidInput` (`ids.rs` Zeilen 14, 64, 98). Ersatz: eigener `IdError`. `types` re-exportiert per `pub use contextra_ids::*`, damit alle Importpfade `contextra_types::DocId` weiter gelten.
- **Ersetzt:** die Kopien in `vector/types.rs:18/25/97`, `text/types.rs:9/15`, `graph/types.rs:58/104`, `rank/local_types.rs:10`, `ingest/chunking/types.rs:17`, `erasure/types_local.rs:1628/1665` (F4). Das Feature `docid-128` führen heute `types`, `store`, `text`, `vector`, `graph`, `py` getrennt; es wird in `ids` besessen und von dort durchgereicht.
- **Ziel-Closure:** keine interne. Externe: `blake3`, `serde`, `thiserror`.
- **Risiko:** Die Byte-Ausgabe von `DocId::from_key` und die Serde-Form müssen identisch bleiben, weil `store`, `vector`, `text` sie persistieren. Dafür die 473 Zeilen `ids_tests.rs` mitnehmen und gegen die Kopien in `vector`/`text`/`graph` als Differenztest laufen lassen.

#### contextra-env (neu)

- **Quelle:** `ports/clock.rs` (109 Z.: `Clock`, `SystemClock`), `ports/rng.rs` (208 Z.: `Rng`, `SeededRng`, SplitMix64), `ports/id_gen.rs` (101 Z.: `IdGen`, `SequentialIdGen`), `ports/metrics.rs` (335 Z.: `MetricsSink`, `NoopMetricsSink`, `InMemoryMetricsSink`, `MetricKey`, Snapshots), `ports/env.rs` (16 Z.). Die `ManualClock` aus `testkit/manual_clock.rs` (172 Z.) kommt unter Feature `test-utils` hinzu.
- **Ersetzt:** alle `Clock`/`Rng`/`MetricsSink`-Kopien aus F4.
- **Pflichtschritt vor dem Schnitt:** Signaturen der zehn `Clock`- und fünf `Rng`-Varianten per Diff vergleichen. `vector/contract.rs:39–45` beschreibt Unix-Millisekunden. Weichen Signaturen ab, definiert `env` die Obermenge und die lokalen Traits werden `pub use` oder dünne Adapter in `compat/`.
- **Ziel-Closure:** keine interne. `ports` re-exportiert `env`, damit `contextra_ports::Clock` gültig bleibt.

#### contextra-sys

- **Ist:** 2.204 Zeilen (`acl_win32` 517, `durability` 471, `mlock` 306, `vault` 297, `mmap` 274, `posix` 192, `error` 78). Verwender: `store` (5 Dateien), `vector` (2), `db` (1), `bench`. Keine interne Abhängigkeit; einziger `unsafe`-Hafen des Workspace.
- **Aktion:** bleibt unverändert, Tier beta. `durability.rs` wird die einzige Implementierung des dauerhaften Flush (siehe `durable-fs`). Die Doc-Zeile `lib.rs:19` korrigieren. Offen und nicht verglichen: Rollenüberschneidung `sys/vault.rs` (297 Z.) und `db/volatile_vault.rs` (592 Z.).

#### contextra-durable-fs

- **Ist:** 1.910 Z. (`lib.rs` 455, `journal` 436, `scrub` 318, `fault_injector` 300, `health` 139, `vfs` 141, `platform` 48). Kein Verwender. Dependencies: `thiserror`, optional `crc32fast`, `serde`, `tracing`.
- **Entscheidung:** Empfohlen wird, `durable-fs` zum Besitzer der Dateisystem-Schicht zu machen, statt es zu löschen: `journal`, `scrub`, `health`, `fault_injector` (zusammen rund 1.190 Z.) haben in `sys` keine Entsprechung.
- **Schnitt:**
  1. `platform.rs` (Zeilen 25–41, `sync_data`/`sync_all`) delegiert hinter Feature `sys-flush` an `contextra_sys::durable_flush_file`/`durable_flush_dir`. Vorher die Semantik von `platform.rs` und `sys/durability.rs:101ff` vergleichen.
  2. `ports/vfs.rs` (404 Z.: `Vfs`, `VfsFile`, `VfsOpenOptions`), `store/io_seam.rs` (357 Z., `RealFs`) und `testkit/fault_vfs.rs` (457 Z.) wandern als `vfs::handle` hierher; der vorhandene `Vfs` (`read`, `write_atomic`, `append`, `sync_file`) wird `vfs::whole`. `ports` re-exportiert unter dem alten Pfad.
  3. `store/fsync.rs` (125 Z.) entfällt zugunsten der `flush_*`-Funktionen.
- **Abweichung von 4.2:** Mit Schritt 1 wird `durable-fs` Ebene 1 (Closure: `sys`). Bis `store` den neuen Weg tatsächlich nutzt, Tier experimental. Schritt 2 und 3 sind optional, wenn du `store` nicht anfassen willst; dann bleibt `durable-fs` ein eigenständiger Baustein ohne Verwender.

#### contextra-simd

- **Ist:** 5.868 Z. (`dispatch.rs`, `kernels/`, `metric.rs`, `error.rs`), Dependencies `thiserror`, `bytemuck`, `tracing`, `serde`. 13 Referenzen auf `types`, alle hinter `types-compat`. Kein Verwender; `vector/distance.rs:2` sagt ausdrücklich „ohne contextra-simd“, und `adapt/bandit.rs:224–225` wartet auf SIMD-Kernel.
- **Schnitt:** `simd` besitzt `DistanceMetric` (`metric.rs:5`); die Kopien in `types/.../document.rs:20` und `vector/types.rs:130` werden Re-Exports. `vector` bekommt ein optionales Feature `simd`, das die Skalarpfade in `distance.rs` (`compute_distance` ab Zeile 538, `euclidean_distance_sq_u8`, `dot_product_u8` …) durch `contextra_simd::dispatch` ersetzt. Die Skalarversionen bleiben als Orakel für Differenztests.
- **Ziel-Closure:** keine. `types-compat` wird zu `workspace` und in `compat/` verschoben.

#### contextra-wire

- **Ist:** 6.047 Z., FlatBuffers-Schemas (`schemas/`, `build.rs`) mit generiertem Code (`contextra_generated.rs`, `generated.rs`) sowie `jsonrpc.rs`, `adapter.rs`, `verification.rs`. Verwender: `mcp`, `py`, `router`. Keine interne Abhängigkeit.
- **Aktion:** keine Strukturänderung, Ebene 0. Generierten Code im LOC-Report ausweisen. Prüfen, ob `router` die zwei `contextra_wire`-Referenzen real braucht (sonst Abhängigkeit streichen).

#### contextra-mvcc

- **Ist:** 7.542 Z. (`tx_buffer` 1.435, `ssi` 1.206, `snapshot` 952, `seq_log` 836). Verwender: `store` (22 Referenzen), `vector` (`hnsw/types.rs:203,279`, `SequenceLog`). Kopplung zu `types`: `DocId`, `TenantId`, `TxId`, `TOMBSTONE_BIT`, `ContextraError`, `Result`, `SeqBitTombstone`, `TombstoneSemanticsCheck`; zu `ports`: `Clock`, `MetricsSink`. Alle 11 Referenzen sind ungegatet, es gibt kein Feature.
- **Schnitt:** `types/src/tombstone.rs` (158 Z.) zieht nach `mvcc`; `TOMBSTONE_BIT` bleibt in `ids`. IDs aus `ids`, Clock und MetricsSink aus `env`. Eigener `MvccError`; `From<MvccError> for ContextraError` nur in `compat/`.
- **Ziel-Closure:** `ids`, `env`.

#### contextra-crypto

Drei Teile, abgeleitet aus F6:

- **Kern (bleibt):** `aead.rs` (190), `dek.rs` (153), `kdf.rs` (319), `secret.rs` (146), `error.rs` (146), `anti_tamper.rs` (191), `crypto.rs` (948, `KeyManager`), `wal_crypto.rs` (1.163), `wal_completeness.rs` (75). Das ist alles, was `store` braucht (`WalHmac`, `IntegrityVerifier`, `WalEntrySnapshot`, `KeyManager`, `verify_wal_chain_completeness`). `crypto.rs` nutzt `TenantId`/`TenantScoped` aus `types`; sie kommen aus `ids`.
- **KV-Teil (wandert nach `kvcache`):** `kv_cipher.rs` (405), `kv_segment/` (`store` 859, `segment` 366, `eviction_worker` 220, `manager` 160, `delete_mode` 55), dazu `kv_shredding.rs` (613) und `revocation_log.rs` (531), weil `kv_segment/manager.rs:17` und `kv_cipher.rs:107–220` `KeyRegistry` brauchen. Dort als Feature `kv-shredding` (Experiment). `KeyManager::derive_kv_key`, `derive_kv_key_scoped`, `cipher_for`, `cipher_for_scoped` (`crypto.rs:238–316`) nehmen heute `kv_cipher::ModelFingerprint`; die Signaturen werden auf rohe Fingerprint-Bytes umgestellt, bevor `kv_cipher` das Crate verlässt. `infer-candle/kv_bridge.rs` bezieht dann `KvSegment`, `TenantIsolatedKvStore`, `EvictionWorker` aus `kvcache`.
- **Altlast (raus):** `audit_chain.rs` (430), `deletion_proof/` (rund 1.450 Z.), `ed25519_proof.rs` (244). Kein externer Nutzer; Dubletten in `erasure`. Vor dem Entfernen die Vollständigkeit per Diff gegen `erasure/src/{deletion_proof,ed25519_proof}.rs` prüfen und die Dateien nach `archive/` verschieben, nicht löschen.
- **Features und Abhängigkeiten:** `default` wird leer (`legacy-erasure`, `kv-encryption`, `test-utils` entfallen aus den Defaults). `ed25519-dalek` fällt aus `crypto` heraus, sobald `ed25519_proof` weg ist (per `rg` bestätigen).
- **Ziel-Closure:** `ids`, `env`.

### 5.2 contracts

#### contextra-types

Rolle danach: Integrationsvokabular für `engine`, `db`, `contextra`, `mcp`, `py`, `router`, `testkit`, kein Baustein.

- **Bleibt:** `error.rs` (1.162, `ContextraError`), `error_dto.rs` (599), `types/budget.rs` (709), `types/filter.rs` (427), `types/importance.rs` (329), `types/saos.rs` (940), `types/domain/document.rs` (247), `types/domain/misc.rs` (602 abzüglich der Auszüge), `schema.rs` (129), und von `witness.rs` der neutrale Teil `RequestContext` (wird von `store`, `text`, `vector`, `graph`, `db`, `kvcache` genutzt).
- **Geht nach `ids`:** `types/domain/ids.rs`, `ids.rs`, `refs.rs`, `tenant_scope.rs`, `TOMBSTONE_BIT`, `MAX_SEARCH_K`.
- **Geht nach `mvcc`:** `tombstone.rs`.
- **Geht nach `kvcache`:** `model_fingerprint.rs` (79 Z.). `crypto/kv_cipher.rs` und `kvcache/types.rs` führen eigene Varianten; ein Besitzer, nämlich `kvcache`.
- **Geht nach `router`:** `retrieval_strategy.rs` (101 Z.).
- **Geht nach `graph`:** `TlHfdParams`, `PprConfig`, `PprAlgorithm` (`misc.rs` ab Zeile 375). `engine` und `cognition` importieren sie danach aus `graph`.
- **Geht nach `compliance/erasure`:** `erasure.rs` (616), von `witness.rs` die Typen `SagaRecord` und `ScanMethod`, dazu `ErasureId`, `GateId`. `types/src/lib.rs` Zeilen 40–42, 51 und 75–76 anpassen. Vor dem Verschieben `erasure/types_local.rs` (2.264 Z.) gegen diese Typen auf Dubletten prüfen.
- **Experiment-Spuren:** `AutoExtractionMode` und das Feature `auto-extraction-opt-out` gehören zu `engine::extraction` (E5). `MemoryImportance`/`DecayFunction` (`types/importance.rs`) bleiben, bis `decay_controller` (engine/adapt) konsolidiert ist.
- **Fehlertyp:** `ContextraError` bleibt unverändert nutzbar; die Varianten `Sandbox*`, `Cluster`, `KvQuantization`, `KvDeleteModeConfig`, `HnswConnectivityDegraded`, `GraphRepairFailed` werden erst entfernt, wenn die Baustein-Fehler (R3) existieren und `compat/` konvertiert.
- **Ziel-Closure:** `ids`.

#### contextra-ports

Rolle danach: reine Trait-Schicht (SPI) für Integration, keine Implementierungen außer Test-Doubles.

- **Raus nach `env`:** `clock.rs`, `rng.rs`, `id_gen.rs`, `metrics.rs`, `env.rs`.
- **Raus nach `durable-fs`:** `vfs.rs`, `fs.rs`.
- **Raus nach `compliance/erasure`:** `erasure.rs` (272), `erasure_legacy.rs` (53), `gate.rs` (224), `ledger.rs` (179), `observer.rs` (157), `report_signer.rs` (130), `keys.rs` (204), `barrier.rs` (165). Bestätigt: kein Verwender außerhalb von `ports`, `types`, `erasure` (F2). Die Re-Exports in `lib.rs` Zeilen 16–22 und 156–206 anpassen.
- **Raus nach `compliance/license`:** `license.rs` (121) ohne `VectorDeleteMode` (das geht nach `vector`), `policy.rs` (34, `LicenseGate`). `EgressPolicy` aus derselben Datei nach `privacy` prüfen.
- **Raus nach `testkit`:** `conformance.rs` (278), `contract_tests.rs` (632) samt Features `conformance`, `contract-tests`, `testkit`. Dabei gegen `testkit/conformance/` (1.612 Z.) auf Überschneidungen prüfen.
- **Bleibt:** `storage.rs`, `vector_index.rs`, `text_index.rs`, `graph_index.rs`, `graph.rs`, `embedding.rs`, `reranker.rs`, `query_rewriter.rs`, `checkpoint.rs`, `kv.rs`, `kv_bridge_port.rs`, `attention.rs`, `lifecycle.rs`, `observability.rs`, `plugin.rs`, `scanner/*` (die Brücken der Bausteine brauchen `Purger`/`LayerScanner`). `kv`, `kv_bridge_port`, `attention` bleiben in `ports`, weil `kvcache` und `infer-candle` sie gemeinsam nutzen und keine direkte Abhängigkeit voneinander haben sollen.
- **Experiment-Leck:** `graph.rs` definiert `HyperEdgeId`, `RoleBinding`, `RoleId`, also Hyperedge-Typen (Experiment) im Port. Sie wandern hinter ein Feature `exp-hyperedge` oder in `graph`.
- **Ziel-Closure:** `types`, `ids`, `env`.

### 5.3 storage

#### contextra-store

- **Ist:** 55.313 Z. (inkl. Inline-Tests). `wal/` (8.261), `lsm/` (7.521), `sstable/` (3.987), `compaction/` (3.798), `manifest/` (1.568), `scanner/` (1.400), `memtable.rs` (1.085), `tenant_codec.rs` (900), `system_pressure.rs` (394), `engine/` (370), `io_seam.rs` (357), `fsync.rs` (125), `key_source.rs` (71), `util.rs` (117). `kv/` (56 Z.) ist in `lib.rs` nicht eingebunden (die Modulliste enthält kein `kv`), also toter Code. Der Blocker `pub mod kv;` aus `PLAN.md` ist im HEAD bereits behoben.
- **Kopplung:** `ContextraError` 120 Referenzen, `StorageEngine` 103, dazu `Vfs` (16), `Clock` (18), `MetricsSink`, `IdGen`, `scanner/*`, `FeatureRing` (`lsm/config.rs`), `crypto` (WAL), `mvcc` (22), `sys` (7).
- **Schnitt:**
  1. Kern bleibt: `wal/`, `lsm/`, `memtable`, `sstable/`, `compaction/`, `manifest/`, `system_pressure`, `key_source`, `tenant_codec` (nur `TenantId`, aus `ids`), `io_seam`, `fsync`, `util`. Eigener `StoreError`; die heute genutzten `ContextraError`-Varianten (`Storage`, `Io`, `WalCorruption`, `ChecksumMismatch`, `Transaction*`, `Conflict`, `WalTruncationDetected`, `CrossDeviceLink`) werden dort abgebildet.
  2. Nach `compat/`: `engine/` (370 Z., `StorageEngine`-Implementierung), `scanner/` (1.400 Z.: `memtable.rs`, `purge.rs`, `runtime.rs`, `sstable.rs`, `wal.rs`, Implementierung von `Purger`/`LayerScanner`), `From<StoreError> for ContextraError`.
  3. `lsm/config.rs:68–92` (`validate_against_features` mit `FeatureRing`, bereits `#[deprecated]`) entfernt. `validate_durability` bleibt.
  4. `kv/` nach `archive/`.
- **Ziel-Closure:** `ids`, `env`, `sys`, `mvcc`, `crypto` (Kern). Das ist die einzige Ausnahme vom Budget von vier (fünf Mitglieder), weil WAL-HMAC und MVCC-Sequenzen konstitutiv sind. Ersetzt `io_seam`/`fsync` später `durable-fs`, wird die Closure um `durable-fs` größer; deshalb dieser Schritt optional.
- **Experimentell in `store`:** keiner belegt. Der adaptive Compaction-Planer (`compaction/adaptive`) ist im Export von `lib.rs` sichtbar, ich habe ihn nicht als Experiment klassifiziert.

### 5.4 retrieval

#### contextra-vector

- **Ist:** 29.037 Z. in `hnsw/` (u. a. `arena`, `batch`, `core_insert`, `core_search`, `core_rebuild`, `deletion`, `staging`, `adaptive_ef`, `sq8_bias`), `distance.rs` (712), `quantize/` (1.167), `persistence/`, `diskann/` (4.168), `quantize_rabitq.rs` (523), `acorn/` (213), `partial_rebuild.rs` (366), `compute_pool.rs` (227), `candidate_stream.rs` (170), `contract.rs`, `types.rs`, `error.rs` (`VectorError`). Verwender: `engine` (dep), `db`/`bench`.
- **Schnitt Kern/Brücke:**
  - In `compat/`: `hnsw/vector_index_impl.rs`, `diskann/vector_index_impl.rs` (Implementierungen von `ports::VectorIndex`), `hnsw/purge.rs` und `hnsw/scanner.rs` (`HnswPurger`, `HnswScanner`), `error.rs:61–91` (`From<VectorError> for ContextraError`), `types.rs:279ff`.
  - Im Kern ersetzen: `hnsw/types.rs` Zeilen 5, 128–143, 203–210, 241–242, 279 (`ContextraError`/`DocId`/`Result` → `VectorError`/`ids`; `Rng`/`Clock`/`MetricsSink` → `env`), `quantize/mod.rs` (28 Referenzen, `contextra_types::Result` → `VectorError`), `hnsw/core_rebuild.rs` (12), `core_search.rs` (8), `core_insert.rs` (5), `persistence/*`, `distance.rs`, `candidate_stream.rs`.
  - `VectorDeleteMode` aus `ports/license.rs` kommt nach `vector` (`hnsw/config.rs`); `engine/lib.rs:22,246,341` importiert von dort.
- **Experimente:**
  - `diskann/` (12 Dateien, 4.168 Z., Kriterium K erfüllt: über 2.000 Z., `memmap2`, `sys::mmap_readonly`): eigenes Crate `contextra-vector-diskann` (retrieval/, experimental), Abhängigkeit auf `vector` (Experimentell → Stabil). Inbound heute nur `lib.rs:45–46,68–72`; Outbound `hnsw` (5), `quantize` (3), `distance` (3). `vector` muss dafür `quantize`, `distance`, `types` als `pub` anbieten (sind es schon). Das Feature `experimental-diskann` im Manifest von `engine` zeigt dann auf das neue Crate. `diskann/tests.rs`, `integrity.rs`, `format.rs` ziehen mit; das Crate ist so isoliert testbar. `predicate_augmented.rs` bleibt hinter `exp-predicate-augmented-search`.
  - `quantize_rabitq.rs` (523 Z., Blatt): Modul `experimental/rabitq.rs`, Feature `exp-rabitq`.
  - `acorn/` plus `hnsw/acorn_filtered.rs`: Klasse B. Trait `FilteredSearchStrategy` in `hnsw/ext.rs`. Vorher die Verwendung in `engine` per `rg 'contextra_vector::acorn' crates/contextra-engine` auflisten.
  - Feature-Schema vereinheitlichen: `experimental`, `diskann`, `quantize-rabitq`, `experimental-*` werden `exp-diskann`, `exp-rabitq`, `exp-acorn`, Sammelfeature `experimental-all`.
- **Ziel-Closure:** `ids`, `env`, `mvcc`, `sys` (nur Feature `mmap`), `simd` (optional) = höchstens vier.

#### contextra-text

- **Ist:** 17.306 Z. (`inverted/` 3.235, `morphology/` 1.271, `tokenizer.rs` 1.031, `wand.rs` 890, `bm25.rs` 686, `posting_list.rs` 585, `memory_index.rs` 507). Nahe am Ziel: 8 ungegatete Referenzen in `stream.rs:17–18`, `inverted.rs:24`, `inverted/purge.rs:22–24`, `inverted/morph_index.rs:5–6`.
- **Schnitt:** Der Index persistiert über `StorageEngine` (47 Referenzen). Statt des breiten Ports ein kleiner lokaler Trait `PostingStore` (genutzte Methoden per `rg` ermitteln); die Brücke implementiert ihn für `StorageEngine`. `inverted/purge.rs` und `morph_index.rs` (`TextIndex`-Implementierung) nach `compat/`. `types.rs` (130 Z.) und `contract.rs` (39 Z.) werden `pub use` auf `ids`/`env`. Das leere Feature `experimental` und `bm25f` prüfen: leer entfernen.
- **Ziel-Closure:** `ids`, `env`.

#### contextra-graph

- **Ist:** 32.728 Z.: `csr/` (6.857), `ppr/` (4.027), `path_rag/` (1.603), `apprh/` (1.107), `cascade.rs` (1.092), `session_dag.rs` (1.008), `tl_hfd/` (862), `community/` (827) plus `community.rs` (636), `hyperedge.rs` (796) plus Unterverzeichnis, `consistency_enforcement.rs` (481), `types.rs` (388), `percolation.rs` (330), `entity_extraction.rs` (318). 134 von 144 Referenzen auf `types`/`ports` ungegatet (50 Dateien). Das Default enthält `experimental-graph`.
- **Schnitt Kern/Brücke:** `provider.rs`, `purger.rs`, die `GraphIndex`-Implementierung unter `csr/graph_index/*` und alle Importe von `ports::graph_index`/`ports::graph` nach `compat/`. Lokale Typen (`types.rs:58,104`) werden `ids`. `default = []`.
- **Experimente nach Klasse (Beleg: Inbound-Dateien):**
  - Klasse A (Blatt, nur `lib.rs`): `hyperedge_suggest` (184), `entity_extraction` (318), `percolation` (330, Feature `graph-connectivity-health`, von `engine` genutzt), `edge_reinforcement_buffer` (218).
  - Klasse B (Naht nötig): `apprh` (1.107) und `tl_hfd` (862), beide nur von `lib.rs` und `ppr/mod.rs` referenziert (Naht: Trait `DiffusionAlgorithm` in `ppr/ext.rs`), `session_dag` (1.008, Inbound `consistency_enforcement.rs`), `edge_reinforcement` (170, Inbound `csr/inner/reinforcement.rs`), `community` (Inbound `provider.rs`).
  - Klasse C (verflochten): `hyperedge` (19 Inbound-Dateien, u. a. `csr/graph_write/*`, `cascade.rs`, `community/*`; extern `cognition`, `engine`), `path_rag` (14 Inbound-Dateien, darunter alle `apprh/*`, `cascade.rs`; extern `db`), `consistency_enforcement` (6). Sie bleiben im Kern; Zielzustand optional ein Feature `exp-hyperedge`, das die 19 Aufrufstellen gattet. Eintrag in `OPEN_ISSUES.md`, kein erzwungener Split.
  - Kriterium K: A+B ohne `community` ergeben rund 4.200 Z. (`apprh`, `tl_hfd`, `session_dag`, `percolation`, `hyperedge_suggest`, `entity_extraction`, `edge_reinforcement`, `edge_reinforcement_buffer`), also eigenes Crate `contextra-graph-research` (retrieval/, experimental) mit Abhängigkeit auf `graph`.
- **Ziel-Closure:** `ids`, `env`. Externe: `arc-swap`, `scc`, `bincode`.

#### contextra-rank

- **Ist:** 8.659 Z. (`fusion/` 2.368, `calibration/` 960 mit Platt, Isotonic, Conformal, `explain.rs` 465, `metrics.rs` 321, `drift.rs` 137, `dibud/` 517, `local_types.rs` 270). Mit `simd` das am weitesten entkoppelte Crate: 0 ungegatete Referenzen, `local_types.rs` trägt 19 `types-compat`-Gates.
- **Schnitt:** `local_types.rs:10` (`DocId`) und `calibration/isotonic.rs:14` (`Clock`) werden `ids`/`env`. `types-compat` wird `workspace` in `compat/`. `engine/src/temporal_filter.rs` (383 Z., 10 Referenzen auf `rank`) zieht als `rank/temporal.rs` ein.
- **Experimente:** `dibud` (517 Z., Blatt, Inbound nur `lib.rs`): `experimental/dibud/`, Feature `exp-dibud`. `drift` (Klasse B, Inbound `fusion/signal.rs`): Naht in `fusion/ext.rs`; Vergleich mit `adapt/drift.rs` (322 Z.) bleibt ein Semantikvergleich. `fusion/resonance.rs` (90 Z., Inbound `fusion/{mod,normalized,rrf}.rs`): Klasse C, Feature `exp-resonance` an drei Stellen vertretbar.
- **Ziel-Closure:** `ids`, `env`.

#### contextra-checkpoint

- **Ist:** 9.623 Z. (`store.rs` 1.002 plus `store/` 913, `guard.rs` 889 plus `guard/` 300, `orphan.rs` 815 plus `orphan/` 134, `hardlink_cloner.rs` 291, `meta.rs` 227, `manifest.rs` 221). 72 von 72 Referenzen ungegatet, kein Feature.
- **Kopplung:** `ports::{Checkpoint, CheckpointCoordinator, StorageEngine, StorageStats, Vfs, Clock, MetricsSink, IdGen}` und `types::{ContextraError, Result, TxId, WorkflowState}`.
- **Schnitt:** Statt der vollen `StorageEngine` (29 Referenzen) ein kleiner lokaler Trait `SnapshotSource` mit den tatsächlich genutzten Methoden (per `rg 'storage\.' src/` ermitteln). `hardlink_cloner.rs` nutzt eine schmale Dateisystem-Naht (std oder `durable-fs`). Eigener `CheckpointError`. `WorkflowState` (drei Verwendungen) lokal abbilden. Die Implementierung von `ports::Checkpoint`/`CheckpointCoordinator` und die Fehlerkonvertierung gehen nach `compat/`.
- **Abweichung vom Plan:** Die optionale Zusammenführung `checkpoint` in `mvcc` rate ich ab. `checkpoint` ist Datei-Snapshot-Mechanik (Guard, Orphan-Scan, Hardlink), `mvcc` Transaktionsisolation im Speicher; es gibt dafür keinen Beleg im Code.
- **Ziel-Closure:** `ids`, `env`.

---

# Contextra als Baukasten: Implementierungsspezifikation der übrigen Crates

Stand: 10. November 2026 · Grundlage: `tfufuz1/contextra`, HEAD `80f1c33` (flacher Klon, deckt sich mit dem Stand deines Plans) · Methode: ausschließlich statische Analyse (Cargo.toml, `use`-Pfade, Modulbäume, `wc`, `grep`). Es wurde nichts gebaut, installiert oder getestet. Zeilenzahlen zählen nur `src/` (inkl. Inline-Tests und `src/tests/`); sie weichen deshalb teils von den Zahlen im Plan ab (z. B. `ingest` 21.039 statt 29.792). Wo ich Heuristiken benutze, steht das dabei. „Nicht verifiziert“ heißt: nicht geprüft, nicht „wahrscheinlich richtig“.

**Abgrenzung.** Vorausgesetzt ist dein Entkopplungsplan (Abschnitte 1–5.4). Dieses Dokument spezifiziert die Crates, die dort noch keinen Eintrag haben: `kvcache`, `router`, `adapt`, `infer-ollama`, `infer-candle`, `infer-onnx`, `ingest`, `privacy`, `sandbox`, `agent`, `cognition`, `erasure`, `license`, `audit-export`, `avv-generator`, `engine`, `db`, `contextra` (Fassade), `mcp`, `py`, `testkit`. `vector`, `graph`, `store` sind ausgenommen, wie gewünscht. `text`, `rank`, `checkpoint`, `mvcc`, `crypto`, `sys`, `simd`, `wire`, `durable-fs`, `types`, `ports` sind im Plan schon spezifiziert und kommen hier nur vor, wo sie die Spezifikation berühren.

## 1. Kurzfassung

Von den 21 Crates sind neun bereits Blätter ohne interne Abhängigkeit (`adapt`, `privacy`, `sandbox`, `ingest`, `erasure`, `avv-generator`, `infer-onnx`, dazu `license` und `audit-export` mit optionalen Brücken). Bei ihnen geht es um Aufräumen, nicht um Umbau. Echte Entkopplungsarbeit steckt in `kvcache`, `infer-candle`, `infer-ollama`, `cognition`, `agent` und `mcp`. `engine`, `db` und `contextra` sind Verdrahtung und werden nur entlastet.

Ergebnis in vier Punkten:

1. **Vier Korrekturen am Plan**, jeweils belegt (Abschnitt 3): Ebene-0-Crates dürfen `ids`/`env` nicht einbinden, `ModelFingerprint` kann nicht nach `kvcache`, `RetrievalStrategy` kann nicht nach `router`, und `engine/chunker` ist nicht ungenutzt.
2. **Zwei neue Bausteine** werden aus bestehenden Crates herausgelöst: `contextra-prompt-guard` (aus `mcp/prompt_injection/`) und `contextra-vault` (aus `db/volatile_vault.rs`). Beide haben praktisch keine internen Abhängigkeiten.
3. **`cognition` und `agent` drehen die Abhängigkeitsrichtung.** Sie definieren schmale lokale Ports; die Bindung an `engine`/`db` wandert in eine Brücke. Heute hat keines der beiden einen Verwender (Cargo.toml-Scan), die Umkehr kostet also nichts.
4. **Feature-Hygiene** ist ein eigenes Thema: fünf Crates ziehen über `default` Workspace-Abhängigkeiten oder Experimente hinein, drei Features sind leer deklariert (Abschnitt 2, G5/G6).

## 2. Befunde mit Beleg

### G1 – Ebene 0 und Mehrfachdefinitionen widersprechen sich im Plan

Plan 4.2 definiert Ebene 0 als „keine internen Abhängigkeiten“. F4 will aber alle `Clock`/`Rng`/`DocId`-Kopien durch `ids`/`env` ersetzen, darunter die in `adapt`, `privacy`, `license`, `erasure`, `ingest`. Dann hätten diese Blätter eine interne Abhängigkeit.

Beleg, dass die Kopien aktuell kompatibel sind:

- `Clock`: identische Methoden `now_unix_nanos`, `monotonic_nanos` in `ports/clock.rs:8–25`, `vector/types.rs:169ff`, `adapt/clock.rs:9–13`, `adapt/contract.rs:9–13`, `privacy/types.rs:11–15`. Unterschied: `ports::Clock` verlangt zusätzlich `'static` (`clock.rs:8`).
- `Rng`: **nicht** identisch. `ports/rng.rs:12–36` hat `next_u64(&self)` und `next_unit_f64(&self)`; `adapt/contract.rs:17ff` hat `next_u64(&mut self)` und `next_f64`. Das ist der erste belegte Fall für die „Pflichtschritt“-Warnung in deinem WP-1: `env` kann den `Rng` nicht einfach als Obermenge übernehmen, weil `&self` und `&mut self` verschiedene Implementierer zulassen.
- Das Muster, das schon funktioniert: `license/types.rs:282` hat `impl<T: contextra_ports::clock::Clock + ?Sized> Clock for T`. Ein Blanket-Impl in der Brücke macht jeden Port-`Clock` zum lokalen `Clock`, ohne Abhängigkeit im Kern.

### G2 – `ModelFingerprint` hat drei Nutzergruppen, `kvcache` ist keine davon als Besitzer

Plan 5.2 schickt `types/model_fingerprint.rs` nach `kvcache`. Dagegen spricht:

- `ports/src/kv.rs:3,46` benutzt `ModelFingerprint` in `PrefixKey.model`. Wandert der Typ nach `kvcache`, braucht `ports` eine Abhängigkeit auf `kvcache`, also umgekehrt zur Schichtung.
- `crypto/kv_cipher.rs:42` und `infer-candle/model_registry.rs:50` sind reine Re-Exports (`pub use contextra_types::ModelFingerprint`), keine eigenen Varianten. Dateien mit dem Namen: 16 in `infer-candle`, 12 in `kvcache`, 8 in `crypto`, 3 in `ports`, 3 in `types`.
- `kvcache/types.rs:81` hat tatsächlich eine eigene Variante (`ModelFingerprint(pub [u8; 32])`), und `kvcache/compat.rs:20–25` konvertiert sie. Das ist die einzige echte Kopie.

### G3 – `RetrievalStrategy` hängt an `ports`

`ports/lib.rs:27` re-exportiert `RetrievalStrategy` aus `types`; `router/arm_registry.rs:16` importiert es von dort. `adapt/strategy.rs` hat eine zweite Variante mit abweichender Gestalt (`Global` ohne Felder, nicht `#[non_exhaustive]`), die `types`-Variante hat `Global { max_community_nodes, min_community_size }`. Verwender des Namens: `types` (4 Dateien), `adapt` (4), `router` (2), `rank` (1, nur Fehlertext), `ports` (1).

### G4 – `engine/chunker` ist produktiv genutzt

`engine/lib.rs:39` hat `pub mod chunker`; `mcp/server_tools.rs:231–241` ruft `contextra::chunker::MarkdownChunker::new(ChunkerConfig::default())` im Speicherpfad auf. Der Plan (F8) nennt den Chunker nur wegen der einen Referenz auf `types` isoliert. Der Paritätstest `ingest/tests/chunk_prose_parity.rs` nennt im Kopf den Engine-Chunker, aber `ingest/Cargo.toml` führt keine `contextra-*`-Abhängigkeit, auch nicht als Dev-Dependency. Der Test vergleicht also mit Fixtures, nicht mit dem lebenden Engine-Code.

### G5 – Defaults ziehen Workspace-Abhängigkeiten oder Experimente ein

| Crate | `default` | Folge |
|---|---|---|
| `kvcache` | `std`, `ports-compat`, `crypto-compat`, `kv-encryption` | `types`, `ports`, `crypto` plus Altlast `legacy-erasure` im Standard |
| `license` | `std`, `ports-compat` | `ports`, `types` im Standard |
| `audit-export` | `std`, `types-compat` | `types` im Standard |
| `adapt` | `std`, `bandit-routing`, `flow-corrected-thompson`, `rie-greedy-personalization` | drei Experimente im Standard |
| `testkit` | `testkit`, `erasure-conformance` | Erasure-Suiten im Standard |

### G6 – Leere oder wirkungslose Features

- `adapt`: `sketched-bandit = ["bandit-routing"]` ist deklariert (`Cargo.toml:26`), aber `grep` findet in `src/` kein `cfg(feature = "sketched-bandit")`. Der Sketch-Code selbst existiert ungegatet in `bandit.rs` (z. B. `ensure_sketched_state`, Z. 426). Das Feature schaltet also nichts, und der Code ist immer an, sobald `bandit-routing` an ist.
- `kvcache`: `kvcache-attention-eviction = []`, keine Verwendung in `src/`. Außerdem doppelt gemeint: `kivi-quantization` und `kvcache-kivi-quant` gaten dasselbe Modul (`lib.rs`, `cfg(any(...))`).
- `infer-candle`: `candle = []` schaltet nichts, `candle-core`, `candle-nn`, `candle-transformers`, `tokenizers`, `half` sind nicht optional. `lib.rs` bindet `inference`, `gguf_loader`, `embedding` ohne Gate ein. Wer den Crate nur wegen der Traits will, baut trotzdem die ML-Laufzeit.
- `db`: `coherence-bonus-fusion = []`, `edge-reinforcement-learning = []` (kein Forwarding, anders als `graph-connectivity-health`). Verwendung nicht geprüft.

### G7 – Die `*_local`-Dateien in `cognition` und `router` entkoppeln nichts

`cognition/graph_local.rs`, `ports_local.rs`, `engine_local.rs`, `types_local.rs` bestehen aus `pub use contextra_graph as graph; pub use ...`. Sie benennen nur um. `router/ports_local.rs` ist ein `#[deprecated]`-Re-Export von `contextra_ports`; `router/lib.rs:32–45` re-exportiert `bandit`, `lyapunov`, `offpolicy` deprecated aus `adapt`. `cognition/lib.rs` behauptet, `db` binde den Crate ein; kein `Cargo.toml` im Workspace listet `contextra-cognition`.

### G8 – `erasure` trägt eigene Kopien der `ports`-Traits

`erasure/types_local.rs`: `DocId` Z. 1628, `TxId` Z. 1665, `Purger` Z. 1853, `LayerScanner` Z. 2033, `Clock` Z. 2059. Dateien gleichen Namens wie in `crypto`: `deletion_proof.rs` (2.744 Z. in `erasure`, rund 1.450 laut Plan in `crypto`), `kv_shredding.rs` (631 gegenüber 613), `revocation_log.rs` (461 gegenüber 531), `ed25519_proof.rs` (168 gegenüber 244). Das sind keine Kopien; ein Diff ist Pflicht, bevor die `crypto`-Seite archiviert wird (der Plan sagt das für `deletion_proof` und `ed25519_proof`, nicht für `kv_shredding`/`revocation_log`).

### G9 – `db` ist dünn, aber nicht leer

`lib.rs` (908 Z.) enthält ab Zeile 128 nur Tests; produktiv sind rund 120 Zeilen Re-Exports (`prelude`, `advanced`) und Feature-Forwarding. Substanz hat `db` nur in `volatile_vault.rs` (592 Z., importiert `contextra_types` 2×, `contextra_sys`, `zeroize`), `multistep.rs` (442 Z., `MultiStepEngine`, nutzt `ports::QueryRewriter`, `EntityId`, `MAX_SEARCH_K`), `pid_latency_controller.rs` (134) und `homeostat.rs` (58). Die Struktur `Contextra` selbst liegt in `engine/lib.rs:368`.

### G10 – `mcp` enthält einen eigenständigen Baustein und ein Duplikat

- `mcp/prompt_injection/` (`audit.rs`, `guard.rs`, `policy.rs`, `tests.rs`, zusammen 1.162 Z. plus 57 Z. Wrapper) importiert laut `grep` nur `std` und `serde`, keinen `contextra_*`.
- `mcp/protocol.rs` (149 Z.) definiert JSON-RPC-Strukturen, obwohl `mcp` von `wire` abhängt und `wire/jsonrpc.rs` `JsonRpcRequest` (Z. 82), `JsonRpcResponse` (Z. 127), `JsonRpcError` (Z. 149) enthält. Semantik nicht verglichen.
- Namensfalle: `mcp/sandbox.rs` (757 Z.) hat nichts mit dem Crate `contextra-sandbox` zu tun (`mcp` bindet es nicht ein; nur `engine` tut das).

### G11 – Drei verschiedene `Embedder`-Traits

`infer-ollama/traits.rs:11` (`provider_name`, sync `embed`), `infer-candle/traits.rs:12` (`embed`, Batch-Variante), `infer-onnx/traits.rs` (`embed`, `embed_batch`, `dimension`). Dazu `ports::EmbeddingProvider`. Alle vier sind unabhängig voneinander entstanden.

### G12 – `agent` hängt produktiv nur an `db`, `graph`, `checkpoint`, `router`

`agent/Cargo.toml`: `store`, `testkit`, `vector`, `text` stehen unter `[dev-dependencies]`. Referenzen auf `db|graph|vector|text|store|checkpoint|router` nach Datei: `context.rs` 12, `audit.rs` 7, `engine.rs` 5, `event_source.rs` 1, `clm_scratchpad.rs` 1, Rest in Tests. Aus `db` genutzt: `Contextra::open_with_config`, `ContextraConfig`, `Collection`, `DistanceMetric`, `VolatileContextVault`, `VaultConfig`, `VaultChunk`, `SignalModality`.

### G13 – `cognition` hängt hart an `engine`

`engine_local.rs` re-exportiert `Collection`, `StoredDocument`, `StoredDocumentMeta`, `AdaptiveDecayController`, `DecayControllerConfig`, `CommitIntent`, `ProvenanceRecord`, `SearchResult`. Verwendung nach Datei: `context.rs` 4, `semantic_aggregation_facade.rs` 4, `maintenance_scheduler.rs` 3, `consolidation_executor.rs` 3, `context_compaction/{session,compactor}.rs` je 2. Aus `graph`: `cascade`, `csr::EdgeType`, `hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId}`, `detect_communities`, `CsrGraph`, optional `PercolationConfig`. Das ist dieselbe `hyperedge`-Verflechtung, die der Plan bei `graph` als Klasse C führt.

## 3. Korrekturen und Ergänzungen zum Plan

- **K8 – Ebene-0-Crates behalten ihre lokalen Mini-Traits.** `ids` und `env` ersetzen Kopien nur in Ebene 1 (`vector`, `text`, `graph`, `rank`, `mvcc`, `checkpoint`, `kvcache`, `store`). In Ebene 0 (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) bleibt die Kopie, die Brücke bindet per Blanket-Impl an (Vorlage `license/types.rs:282`). Neue Regel **R7**: *Ein Ebene-0-Crate definiert seine Umwelt-Traits lokal und minimal; die Verbindung zu `env` oder `ports` liegt ausschließlich in `compat/`.* Das streicht aus F4/5.1 `ids`: „Ersetzt“ die Einträge `ingest/chunking/types.rs:17` und `erasure/types_local.rs:1628/1665`, und aus `env` die Einträge `adapt`, `privacy`, `license`, `erasure`.
- **K9 – `ModelFingerprint` geht nach `ports/kv`, nicht nach `kvcache`** (G2). `crypto` nimmt wie im Plan rohe `[u8; 32]`, `infer-candle` bezieht den Typ aus `ports`, `kvcache` behält seinen Newtype und konvertiert in `compat/` (wie heute in `compat.rs`).
- **K10 – `RetrievalStrategy` bleibt in `types`** (G3). `adapt` behält seine lokale Variante (Ebene 0); `router` bildet beide in `arm_registry` ab, weil `router` ohnehin beide kennt. Der Eintrag „Geht nach `router`: `retrieval_strategy.rs`“ in 5.2 entfällt.
- **K11 – `engine/chunker` wird nicht entfernt, sondern ersetzt** (G4). Reihenfolge: (1) Paritätstest in `engine` mit `ingest` als Dev-Dependency, (2) `engine/chunker` zu einem dünnen Adapter über `ingest::chunking`, (3) erst danach die Doppelung streichen. `mcp` bleibt unberührt, weil es über `contextra::chunker` geht.

## 4. Spezifikation pro Crate

Aufbau wie im Plan: Ist (mit Beleg), Schnitt, Experimente, Ziel-Closure, Prüfung (führst du später aus), Risiko. Pfade relativ zu `crates/contextra-<name>/src/`.

### 4.1 llm

#### contextra-adapt (Ebene 0, bleibt Blatt)

- **Ist:** 5.334 Z. (`bandit.rs` 1.218, `flow_thompson.rs` 918, `lyapunov.rs` 554, `pid.rs` 465, `drift.rs` 322, `pid_latency_controller.rs` 309, `offpolicy.rs` 307, `decay_controller.rs` 255, `homeostat.rs` 232, `rie_greedy.rs` 221, `shadow_mode.rs` 126). Keine interne Abhängigkeit, keine Referenz auf `contextra_*`. Verwender: `router`, `engine` (optional), `db`. Zwei `Clock`-Traits im selben Crate (`clock.rs:9`, `contract.rs:9`), zwei `Rng` (`rng.rs:6`, `contract.rs:17`); `off_policy.rs` (6 Z.) neben `offpolicy.rs` (307 Z.).
- **Schnitt:** Keine Verschiebung. (1) `clock.rs`/`rng.rs` und `contract.rs` zusammenführen: eine `Clock`, eine `Rng`; `contract.rs` re-exportiert. (2) `off_policy.rs` entfernen, wenn `rg 'off_policy::'` leer ist. (3) `default = ["std"]`; `bandit-routing`, `flow-corrected-thompson`, `rie-greedy-personalization` bleiben Features, der `router` setzt sie gezielt. (4) `sketched-bandit`: entweder den Sketch-Code in `bandit.rs` hinter das Feature stellen oder das Feature streichen (G6); welche Variante gewollt ist, entscheidest du.
- **Rolle für die Konsolidierung (Plan F8):** `adapt` wird Besitzer der Regler. `decay_controller` (255), `homeostat` (232), `pid_latency_controller` (309), `drift` (322) sind die Zielfassungen; die Gegenstücke in `engine/decay_controller.rs` (192), `db/homeostat.rs` (58), `db/pid_latency_controller.rs` (134), `rank/drift.rs` (137) werden erst nach dem Semantikvergleich zu `pub use`. Der Vergleich ist offen.
- **Experimente:** `flow_thompson` (918 Z.) und `rie_greedy` (221 Z.) sind Blätter hinter Features und liegen unter dem Kriterium K (2.000 Z.): bleiben im Crate, Features umbenennen zu `exp-flow-thompson`, `exp-rie-greedy`.
- **Ziel-Closure:** keine.
- **Prüfung:** `cargo tree -p contextra-adapt -e normal` ohne `contextra-*`; `rg 'trait (Clock|Rng)' src` liefert je einen Treffer.
- **Risiko:** Zwei `Rng`-Formen (G1). Beim Zusammenführen die Signatur von `contract.rs:17` (`&mut self`) übernehmen, nicht die von `ports`.

#### contextra-router (Ebene 2)

- **Ist:** 7.784 Z., davon rund 4.700 in `src/tests/`, `tests.rs` und `bandit_regret_tests.rs`. Produktiv: `router/dispatch_core.rs` 655, `profile.rs` 540, `dispatch.rs` 291, `outcome.rs` 202, `arm_registry.rs` 184, `router/{mod,lifecycle,outcomes}.rs`, `routing_strategy.rs`, `fc_ts_dispatch.rs`, `transport.rs`. Abhängigkeiten, alle hart: `types` (47 Referenzen, 13 Dateien: `ContextChunk`, `DocId`, `ConfigFingerprint`, `TokenBudget`, `ContextWindow`, `EntityId`, `TxId`, `RetrievalStrategy`, `ContextraError`), `ports` (11, 5 Dateien: `CommunityResolver`, `ContextPreparer`, `HybridSearchProvider`, `Clock`, `IdGen`, `Vfs`, `BoxFuture`), `adapt` (16), `wire` (nur `dispatch.rs:23`, `JsonRpcRequest/Response`). Verwender: `agent`, `mcp`.
- **Entscheidung:** Der Router bleibt Ebene 2. Er konsumiert `ContextChunk`/`TokenBudget` als Eingabe; diese Typen lokal zu duplizieren bringt dem Baukasten nichts, weil jede Verwendung ohnehin `types` hat. Die Entnehmbarkeit entsteht durch Ausdünnen, nicht durch Spiegelung.
- **Schnitt:**
  1. `ports_local.rs` und die deprecated Re-Exports in `lib.rs:32–45` entfernen (G7), sobald `rg 'contextra_router::(ports_local|bandit|lyapunov|offpolicy)'` außerhalb des Crates leer ist. `mcp/` importiert noch `contextra_router::ports_local::{HybridSearchProvider, ContextPreparer, CommunityResolver}`; diese drei Stellen zuerst auf `contextra_ports` umstellen.
  2. `wire` wird optional hinter `subprocess-dispatch`; `dispatch.rs` und `transport.rs` liegen dahinter. Der Router-Kern ist danach frei von FlatBuffers (`wire` hat `build.rs`).
  3. `Vfs`-Nutzung (`router/…`, eine Stelle) per `rg 'ports::vfs'` lokalisieren; Ziel: Pfad als `&Path`-Funktion oder schmaler Trait.
  4. `RetrievalStrategy`-Abbildung `types` ↔ `adapt` an einer Stelle in `arm_registry.rs` bündeln (K10).
- **Experimente:** `fc_ts_dispatch` (92), `routing_strategy` (72), `transport` (92) sind gegatete Blätter (Klasse A): Features zu `exp-fc-ts`, `exp-routing-strategy`, `cloud-egress-guard` beibehalten.
- **Ziel-Closure:** `types`, `ports`, `adapt`, optional `wire`.
- **Prüfung:** `rg 'contextra_wire' src --glob '!dispatch.rs' --glob '!transport.rs'` leer; `cargo check --no-default-features`.

#### contextra-kvcache (Ebene 1)

- **Ist:** 6.876 Z. (`store.rs` 1.331, `radix.rs` 1.206, `quantize_kivi.rs` 842, `prefix_store.rs` 687, `segment.rs` 668, `service.rs` 588, `eviction_worker.rs` 327, `attention_score.rs` 324, `scratchpad_invalidation.rs` 307, `adapters.rs` 237). Kein Verwender. Referenzen insgesamt: `types` 29 (11 Dateien), `ports` 29 (7), `crypto` 10 (3: `segment.rs:25,41,328,356`, `store.rs:561–598`, `service.rs:39`). Defaults siehe G5. `compat.rs` (70 Z.) existiert, enthält aber nur `From`-Konvertierungen und den `CryptoSegmentCipher`; die eigentlichen Importe stehen im Kerncode (`radix.rs:24`, `quantize_kivi.rs:27`, `store.rs:27–28`).
- **Schnitt Kern/Brücke:**
  - `compat/`: `prefix_store.rs` (687, implementiert `ports::kv::KvPrefixStore`), `adapters.rs` (237, `EmbeddingProvider`, `KvBridgeStorage`, `StorageEngine`, `FeatureRing`, `PluginManifest`), `store.rs:790–862` (`KvCachePruner`/`KvCacheScanner`, `Purger`, `LayerScanner`), `scratchpad_invalidation.rs` (nutzt `ports::kv::PrefixKey`), der Port-Teil von `service.rs` (`AttentionExporter`, `KvBridgeStorage`), `compat.rs`.
  - Kern: `radix.rs`, `quantize_kivi.rs`, `attention_score.rs`, `eviction_worker.rs`, `segment.rs`, Rest von `store.rs`/`service.rs`. `ContextraError`/`Result` → `KvCacheError` (existiert in `error.rs`, 57 Z.), `TenantId` → lokaler Typ in `types.rs` (existiert), `Clock` → `env`, `CacheDirective`/`StepId` lokal (heute `store.rs:27` aus `types::error` und `types.rs` doppelt).
  - `FeatureRing` (`adapters.rs:15,135–221`) entfällt aus dem Kern ersatzlos; die Lizenzprüfung ist Sache der Brücke.
  - Aus `crypto` kommen laut Plan `kv_cipher`, `kv_segment`, `kv_shredding`, `revocation_log` herein (Feature `kv-shredding`, experimentell). `store.rs:561–598` und `service.rs:39` importieren dann lokal statt aus `contextra_crypto`. Übrig bleibt die Abhängigkeit auf `crypto` (Kern: `aead`, `dek`, `kdf`).
  - `ModelFingerprint`: Kern nutzt den eigenen Newtype (K9).
- **Features:** `default = ["std"]`. `workspace` ersetzt `ports-compat`/`crypto-compat`. `kv-encryption` bleibt, setzt kein `legacy-erasure` mehr. `kivi-quantization` und `kvcache-kivi-quant` → ein Feature `exp-kivi` (842 Z., Blatt unter dem Kriterium K). `content-addressed-kv-cache` → `exp-content-addressed`. `kvcache-attention-eviction` streichen (G6).
- **Ziel-Closure:** `env`, `crypto` (Kern, nur mit `kv-encryption`). `ids` entfällt, solange `TenantId` lokal bleibt.
- **Prüfung:** `rg 'contextra_(types|ports)' src --glob '!compat/**'` leer; `rg 'contextra_crypto' src` nur hinter `kv-encryption`; `cargo check --no-default-features`.
- **Risiko:** Das Verschieben von `kv_segment` aus `crypto` ist von `kv_shredding::KeyRegistry` abhängig (Plan F6). Beide Schritte (WP für `crypto`, WP für `kvcache`) in einem Commit-Paar.

#### contextra-infer-ollama (Ebene 1)

- **Ist:** 6.657 Z., davon `client/tests.rs` 1.739. Produktiv: `client/core.rs` 1.265, `importance.rs` 682, `embedding.rs` 485, `context_prefixer.rs` 461, `client/policy.rs` 409, `query_rewriter.rs` 362, `model_info.rs` 343, `error.rs` 288, `client/circuit_breaker.rs` 193. `types` und `ports` sind optional (`ports-compat`), aber Referenzen stehen in Dateien mit wenigen Gates (Heuristik, Referenzen/`cfg`-Zeilen): `error.rs` 34/9, `trait_impls.rs` 8/1, `query_rewriter.rs` 5/12, `core.rs` 3/1. Genutzt aus `types`: `ContextraError`-Varianten (`Internal`, `Io`, `Storage`, `PolicyViolation`, `NotFound`, `InvalidInput`), `Result`, `QueryRewriteOutput`, `ScoredEntry`, `ImportanceScore`, `ConfigFingerprint`. Aus `ports`: `LlmTextGenerator(+Streaming)`, `EmbeddingProvider`, `EmbeddingError`, `QueryRewriter`, `SegmentSynthesizer`, `Rng`, `SeededRng`, `SystemClock`, `BoxFuture`. Verwender: `mcp` (optional).
- **Schnitt:**
  - `compat/`: `client/trait_impls.rs` (67, implementiert die `ports`-Traits), `query_rewriter.rs` (362, schon `cfg(client, ports-compat)`), die `ContextraError`-Konvertierung aus `error.rs`, die eine Referenz in `embedding.rs`.
  - Kern: `OllamaError` (in `error.rs`) rein lokal; `ImportanceScore` (Wertetyp 0..1) lokal mit `From` in `compat/`; `traits.rs` (`Embedder`, `AsyncEmbedder`, `TextGenerator`, `BoxFuture`) ist bereits die lokale Abstraktion und bleibt.
  - `Rng`/`Clock` aus `ports` (`SeededRng`, `SystemClock`) → `env`.
- **Features:** `types-compat` und `ports-compat` → ein Feature `workspace`. `client` bleibt (HTTP-Stack). `lib.rs` hat `error` mit `cfg(client)` neben ungegateten `pub use error::…` (Zeilen aus der Grep-Ausgabe, Gate-Zuordnung nicht zeilenweise verifiziert): vor dem Umbau prüfen, ob `--no-default-features` überhaupt kompiliert.
- **Ziel-Closure:** `env`.
- **Prüfung:** `rg 'contextra_(types|ports)' src --glob '!compat/**'` leer.
- **Risiko:** Gering. `importance.rs` ist die einzige Datei mit Domänenwert (`ImportanceScore`); sie muss ihre Skala dokumentieren, bevor sie lokal wird.

#### contextra-infer-candle (Ebene 1, schwer)

- **Ist:** 6.364 Z. (`inference/core.rs` 985, `inference/tests.rs` 816, `kv_bridge.rs` 694, `gasp.rs` 672, `model/quantized_llama.rs` 590, `embedding.rs` 458, `gguf_loader.rs` 394, `inference/quantized_llama.rs` 380, `kv_state.rs` 318, `embedding_provider.rs` 289). Harte Abhängigkeiten auf `types`, `ports`, `rank` (nur `gasp.rs:24`, `IsotonicCalibrator`), optional `crypto`. Aus `types`: `ContextraError`, `Result`, `ConfigFingerprint`, `ContextChunk`, `DocId`, `TenantId`, `TxId`, `ModelFingerprint`. Aus `ports`: `kv::{RopeConfig, KvLayout, KvBlock, KvPrefixHit, KvPrefixStore, PrefixKey}`, `LlmTextGenerator(+Streaming)`, `EmbeddingProvider`, `TextEmbeddingEngine`, `ContextSegment`, `KvBridgeStorage`, `AttentionExporter`, `GroundingValidator`. Verwender: `mcp`, `contextra` (beide optional).
- **Schnitt:**
  - `compat/`: `embedding_provider.rs` (289), `attention_exporter.rs` (119), `kv_bridge.rs` (694, bleibt hinter `kv-bridge` und braucht danach `kvcache` statt `crypto`-KV-Teil, siehe Plan 5.1), `gasp.rs` (672; `GaspValidator` implementiert `GroundingValidator`).
  - Kern: `inference/*`, `model/quantized_llama.rs`, `gguf_loader.rs`, `kv_state.rs`, `embedding.rs`, `error.rs` (50 Z.), `traits.rs` (lokale `Embedder`, `TextGenerator`, `KvStateSink/Source`, `GenerationConfig`).
  - `rank`: `IsotonicCalibrator` ist eine kleine Funktion. Entweder `gasp` bekommt einen schmalen lokalen Trait `ConfidenceCalibrator`, den die Brücke mit `rank` füllt, oder `rank` wird optionales Feature `calibration`. Empfehlung: lokaler Trait (hält Ebene 1 → Ebene 0 ein).
  - `candle-core`, `candle-nn`, `candle-transformers`, `tokenizers`, `half` hinter das Feature `candle` stellen (G6). Module `inference`, `gguf_loader`, `embedding`, `model` bekommen `cfg(feature = "candle")`. `fake.rs` (99) → `test-utils`.
- **Experimente:** `kv-stage-b` (`kv_state`, `model/`) und `gasp` bleiben Features; `kv-bridge` bündelt die `kvcache`-Kopplung. Namen: `exp-kv-stage-b`, `exp-gasp`, `kv-bridge`.
- **Ziel-Closure:** `env` (falls Uhr gebraucht), optional `kvcache` (nur `kv-bridge`).
- **Prüfung:** `cargo tree -e normal --no-default-features` ohne `candle-*`; `rg 'contextra_(types|ports|rank)' src --glob '!compat/**'` leer.
- **Risiko:** Das Gaten der ML-Module berührt `lib.rs`-Re-Exports (`pub use inference::*`). Vorher `rg 'contextra_infer_candle::'` über `mcp`/`contextra` auflisten, damit kein Verwender stillschweigend ein Symbol verliert.

#### contextra-infer-onnx (Ebene 0, bleibt Blatt)

- **Ist:** 948 Z. (`embedder.rs` 251, `config.rs` 216, `reranker/mod.rs` 197, `fakes.rs` 148, `error.rs` 62, `traits.rs` 44). Keine interne Abhängigkeit. `ort`, `tokenizers`, `ndarray`, `sha2` hinter `onnx`, `reqwest` hinter `download`. `fakes.rs` ist `pub mod` ohne Gate; `experimental` und `inspect` sind leer deklariert. Verwender: `engine`, `mcp` (beide optional).
- **Schnitt:** `fakes` hinter `test-utils`; leere Features streichen. Die `Embedder`-/`Reranker`-Traits bleiben lokal (R7). Die Anbindung an `ports::EmbeddingProvider`/`Reranker` liegt bisher in `engine`/`mcp`; dorthin gehört sie auch weiter, oder in ein `compat/`-Modul hinter `workspace`.
- **Zu klären (G11):** Ob ein gemeinsamer Embedder-Trait gewollt ist. Empfehlung: nein. Drei lokale Traits mit je einer `compat`-Brücke zu `ports::EmbeddingProvider` sind für Entnahme besser als ein gemeinsames Mikro-Crate, das alle drei an eine Signatur bindet.
- **Ziel-Closure:** keine.
- **Prüfung:** `cargo check --no-default-features` und mit `--features onnx` getrennt; `rg 'pub mod fakes' src` hat ein Gate.

### 4.2 data

#### contextra-ingest (Ebene 0, bleibt Blatt)

- **Ist:** 21.039 Z. (`formats/` 11.741, `chunking/` 5.839, `safe/` 1.029, `detect/` 768, `canonical.rs` 323, `ir.rs` 253, `format.rs` 250, `extractor.rs` 223, `pipeline.rs` 177, `registry.rs` 164). `formats/`: `text` 1.703, `web` 1.584, `ole` 1.506, `docx` 1.502, `structured` 1.465, `odf` 995, `email` 878, `sheets` 780, `pptx` 694, `pdf` 589. Abhängigkeiten: `serde`, `serde_json`, `thiserror`, `blake3`, `unicode-segmentation`, dazu pro Format optionale Parser (`zip`, `quick-xml`, `html5ever`, `calamine`, `lopdf`, `cfb`, `mail-parser`, `yaml-rust2`, `toml`). `default = ["fmt-text"]`. Keine interne Abhängigkeit, kein Verwender. Eigene `DocId(pub u64)` (`chunking/types.rs:17`), `ContextChunk` (Z. 56) und `ContextChunkLink` (Z. 47); `types::ContextChunk` (`saos.rs:230`) ist ein zweiter Typ gleichen Namens.
- **Entscheidung (Abweichung vom Plan, K8):** `ingest` bleibt ohne `ids`. Eine 20-Zeilen-Newtype gegen den Status als Null-Abhängigkeits-Parser-Bibliothek einzutauschen, ist für das Ziel „entnehmen“ ein schlechter Tausch. Die Verbindung zu `ids::DocId` und `types::ContextChunk` entsteht in `compat/` hinter `workspace`.
- **Schnitt:** `fuzz_support.rs` (37 Z., `pub mod` ohne Gate) hinter `fuzz`. Keine Aufteilung nach Format: kein Format erreicht das Kriterium K (größtes ist `text` mit 1.703 Z.), die Features sind bereits die Isolationsgrenze. `all-formats` bleibt.
- **Chunker-Konsolidierung (K11):** Der Besitzer des Chunkings ist `ingest` (`chunking/prose` 2.109 Z. gegenüber `engine/chunker` 1.533 Z.). Schritte: Paritätstest nach `engine/` verschieben (oder dort neu schreiben, mit `ingest` als Dev-Dependency), dann `engine/chunker` auf `ingest` abbilden. Das Ergebnis `contextra::chunker::MarkdownChunker` muss API-stabil bleiben (`mcp/server_tools.rs:240`).
- **Ziel-Closure:** keine.
- **Prüfung:** `cargo tree -p contextra-ingest -e normal` ohne `contextra-*`; `cargo check --no-default-features`; für jedes `fmt-*`-Feature `cargo check --no-default-features --features fmt-x` (nicht ausgeführt).
- **Risiko:** Die Chunk-Reihenfolge und Breadcrumbs sind persistenzrelevant für nachgelagerte IDs. Der Paritätstest ist deshalb Voraussetzung, nicht Nacharbeit.

### 4.3 safety

#### contextra-privacy (Ebene 0, bleibt Blatt)

- **Ist:** 5.617 Z. (`entity_recognizer.rs` 1.398, `egress_vault.rs` 992, `egress_gateway.rs` 852, `egress_guard.rs` 539, `bulk_exfiltration_detector.rs` 437, `processing_registry.rs` 422, `context_edit_audit.rs` 376, `types.rs` 151, `audit_trace.rs` 123, `guarded_payload.rs` 107). Keine interne Abhängigkeit; eine einzige Treffer-Zeile für `contextra_types` (`processing_registry.rs:14`, ein Doc-Kommentar). `lib.rs` behauptet im Docstring „Ausgehend: `contextra-types`, `contextra-ports`“; das stimmt nicht. Eigene `Clock` (`types.rs:11`), eigene Mandanten- und Egress-Policy-Typen. Verwender: `mcp`.
- **Schnitt:** Keine. Docstring korrigieren (F10). Leere Features `experimental`, `inspect`, `serde`, `tracing` prüfen und streichen oder verdrahten (`serde = []` ist leer deklariert; ob `#[cfg(feature = "serde")]` im Quelltext vorkommt, nicht geprüft).
- **Überschneidung (nicht vergleichen, nur notieren):** `processing_registry.rs` (VVT nach Art. 30) und `audit-export` (Register-Export) behandeln denselben Gegenstand. Ein Adapter `ProcessingRegistry → AuditRecord` wäre sinnvoll, gehört aber nicht in `privacy`, sondern in eine Brücke von `audit-export` (siehe dort).
- **Ziel-Closure:** keine.
- **Prüfung:** `rg 'contextra_' src` hat nur den Doc-Treffer; danach `cargo check --no-default-features`.

#### contextra-sandbox (Ebene 0, bleibt Blatt)

- **Ist:** 2.377 Z. (`approval.rs` 532, `wasi/sys.rs` 337, `executor/tests.rs` 327, `wasi/fd.rs` 325, `capabilities.rs` 183, `executor/mod.rs` 160, `executor/runner.rs` 151, `merge.rs` 118). Keine interne Abhängigkeit. `wasmtime`, `tokio`, `anyhow` hinter `wasm`. Ohne `wasm` bleiben `approval`, `capabilities`, `error`, `output`. `zeroize` ist hart. Verwender: `engine` (optional, `merge_operator_adapter.rs` mit `store`).
- **Schnitt:** Keine. Die Brücke zu `store`-Merge-Operatoren bleibt in `engine/merge_operator_adapter.rs` (201 Z., Ebene 2). Ein Hinweis in der Crate-Doku, dass `mcp/sandbox.rs` ein anderes Ding ist (G10).
- **Experimente:** WASI-Eigenbau (`wasi/`, 714 Z.) ist sicherheitsrelevanter Eigencode neben `wasmtime`; als Feature `wasm` ohnehin gegatet. Kein Split.
- **Ziel-Closure:** keine.
- **Prüfung:** `cargo check` mit und ohne `wasm`; `[[test]]`-Einträge haben `required-features = ["wasm"]` (in `Cargo.toml` Zeile 35 ff. sichtbar).

#### contextra-prompt-guard (neu, Ebene 0)

- **Quelle:** `mcp/src/prompt_injection/{audit,guard,policy,tests}.rs` (1.162 Z.) und der Wrapper `mcp/src/prompt_injection.rs` (57 Z.). Imports laut `grep`: nur `serde`, `std::{fs, io, path, sync}`.
- **Schnitt:** Verschieben nach `crates/safety/contextra-prompt-guard`. `mcp/lib.rs:26,53` re-exportiert danach aus dem neuen Crate. `SecurityAuditLogger` schreibt per `OpenOptions` direkt in eine Datei: hinter ein Feature `file-audit` stellen, damit der Guard ohne Dateisystem nutzbar ist.
- **Ziel-Closure:** keine.
- **Prüfung:** `cargo tree -p contextra-prompt-guard -e normal` zeigt nur externe Crates; `rg 'prompt_injection' crates/contextra-mcp/src` zeigt nur den Re-Export.

#### contextra-vault (neu, Ebene 1)

- **Quelle:** `db/src/volatile_vault.rs` (592 Z.): `VolatileContextVault`, `VaultChunk`, `VaultConfig`, `SignalModality`. Importiert `contextra_types` 2× (Fehlerseite), `contextra_sys` (mlock), `zeroize`.
- **Schnitt:** Eigener `VaultError`, `contextra_types`-Bezug nur in `compat/`. `db` re-exportiert (`db/lib.rs:108`), `agent` bezieht `VolatileContextVault` weiterhin über `db` oder direkt.
- **Offen, nicht verglichen:** Überschneidung mit `sys/vault.rs` (297 Z.). Erst nach dem Vergleich entscheiden, ob `contextra-vault` `sys::vault` ersetzt oder darauf aufsetzt.
- **Ziel-Closure:** `sys`.
- **Prüfung:** `rg 'contextra_(types|ports)' src --glob '!compat/**'` leer.

### 4.4 agentic

#### contextra-agent (Ebene 2)

- **Ist:** 5.555 Z. (`engine.rs` 850, `clm_scratchpad.rs` 776, `tools/` 588, `context.rs` 572, `audit.rs` 512, `goal_condition.rs` 496, `engine/` 426, `dlq.rs` 409, `graph.rs` 324, `event_source.rs` 296, `step.rs` 115, `budget.rs` 90). Kein Verwender. Produktive Abhängigkeiten: `types` (56 Referenzen, 14 Dateien), `ports` (22, 9: `BoxFuture` 10×, `MetricsSink`, `VectorIndex`, `StorageEngine`, `Clock`, `Checkpoint`), `db` (18, 7), `graph`, `checkpoint`, `router`. Dev: `store`, `testkit`, `vector`, `text` (G12).
- **Schnitt:**
  - Laufzeitkern (bleibt, ohne `db`): `engine.rs`, `engine/condition.rs`, `goal_condition.rs`, `budget.rs`, `step.rs`, `dlq.rs`, `event_source.rs`, `tools/{manifest,registry}.rs`, `audit.rs` (Referenzen dort per `rg` ermitteln: 7 auf `db|graph|…`).
  - Speicherbindung (wandert in `compat/`): `context.rs` (12 Referenzen), `clm_scratchpad.rs`, `tools/retrieve.rs`, `graph.rs` (324, `CsrGraph`), Teile von `engine.rs` (5).
  - Dafür drei lokale Ports im Crate: `ContextStore` (Retrieval und Schreiben über `Collection`), `ScratchpadStore` (Checkpoint-Schnitt), `AgentClock`/`AgentMetrics` (statt `ports`-Direktimport, nur falls Ebene 2 darauf verzichten soll; als Ebene-2-Crate darf `agent` `ports` weiter nutzen).
  - `router`-Kopplung (`DefaultRouterEngine`, `DecisionId`, `RoutingOutcome`, 4 Referenzen) bleibt, weil `RoutingOutcome` Teil des Agent-Audits ist; optional hinter Feature `routing`.
- **Ziel-Closure (Kern):** `types`, `ports`. (Brücke: `db`, `graph`, `checkpoint`, `router`.)
- **Prüfung:** `rg 'contextra_db' src --glob '!compat/**'` leer.
- **Risiko:** `context.rs` und `audit.rs` sind die dichtesten Stellen; vor dem Schnitt `rg 'db::' src/context.rs` auswerten.

#### contextra-cognition (Ebene 2, Richtung umkehren)

- **Ist:** 7.334 Z. (`context_compaction/` 1.502, `memory_consolidation.rs` 1.298, `consolidation_executor.rs` 1.143, `aggregation_phase.rs` 906, `context.rs` 640, `maintenance_scheduler.rs` 564, `synthesis_phase.rs` 237, `transitivity_veto.rs` 168, `leanrag_input.rs` 136, `channel.rs` 130, `consolidation_locks.rs` 127, `semantic_aggregation_facade.rs` 126, `graph_sink.rs` 105). Harte Abhängigkeit auf `engine` (G13), `graph`, `types`, `ports`. Kein Verwender (G7). `#![deny(dead_code)]` aktiv.
- **Schnitt:**
  1. Lokale Ports statt Re-Exports: `DocumentSource` (lesen: `Collection`/`StoredDocument`), `ConsolidationSink` (schreiben, `CommitIntent`), `HyperedgeSink` (existiert als `graph_sink.rs`, 105 Z., und `channel.rs`, 130 Z., „Baukasten-Integration“ laut Modulkommentar: Ausgangspunkt), `DecayPolicy` (statt `AdaptiveDecayController` aus `engine`).
  2. Reine Algorithmen bleiben im Kern: `context_compaction`, `memory_consolidation` (Planung), `aggregation_phase`, `synthesis_phase`, `transitivity_veto`, `leanrag_input`, `maintenance_scheduler`, `consolidation_locks`. Engine-gebunden: `consolidation_executor.rs` (3 Referenzen), `context.rs` (4), `semantic_aggregation_facade.rs` (4). Diese gehen in `compat/`.
  3. Fehler: `CognitionError` statt `ContextraError::{Internal, PolicyViolation, InvalidInput}` (rund 30 Stellen, `consolidation_executor.rs`, `memory_consolidation.rs`, `channel.rs`, `synthesis_phase.rs`).
  4. `graph`-Kopplung (Hyperedge-Typen) bleibt vorerst: Experiment-Feature `exp-hyperedge` (Plan 5.4, Klasse C) deckt die 19 Aufrufstellen in `graph` ab; `cognition` bekommt sie dort mit.
  5. Die Brücke implementiert die lokalen Ports für `engine::Collection` und liegt in `db` (Feature `cognition`), nicht in `cognition`. So bleibt die Richtung `db → cognition`, wie der Docstring es schon behauptet.
- **Experimente:** `graph-connectivity-health` und `edge-reinforcement-learning` leiten heute an `graph` **und** `engine` weiter (Cargo.toml `[features]`); nach dem Schnitt nur noch an `graph`.
- **Ziel-Closure:** `types`, `ports`, `graph` (nur über `exp-hyperedge`).
- **Prüfung:** `rg 'contextra_engine' src --glob '!compat/**'` leer; `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine`.
- **Risiko:** Mittel. `CommitIntent` kommt aus `engine::transaction` (1.652 Z.); die Transaktionsschicht kann nicht ohne weiteres als Port abgebildet werden. Wenn der Port-Zuschnitt dort scheitert, `CommitIntent` als eigener Typ in `cognition` führen und in der Brücke abbilden.

### 4.5 compliance

#### contextra-erasure (Ebene 0, bleibt Blatt)

- **Ist:** 16.709 Z. (`deletion_proof.rs` 2.744, `vault/` 2.360, `types_local.rs` 2.264, `saga/` 1.395, `report/` 1.257, `gate/` 925, `journal/` 866, `measure/` 724, `kv_shredding.rs` 631, `revocation_log.rs` 461, `scope/` 353, `vault.rs` 377, `restricted.rs` 211, `purge/` 206). Keine `contextra-*`-Abhängigkeit, aber **keine Features**: `ed25519-dalek`, `aes-gcm-siv`, `tokio`, `futures-util`, `bincode`, `rand 0.8` sind immer an. Eigene Kopien von `DocId`, `TxId`, `Purger`, `LayerScanner`, `Clock` (G8).
- **Schnitt:**
  1. Features einführen: `signing` (`report/signing.rs`, `deletion_proof.rs`, `ed25519_proof.rs`), `vault`, `async` (`tokio`, `futures-util`). Welche Datei welche externe Abhängigkeit braucht, per `rg 'use (ed25519_dalek|aes_gcm_siv|tokio|futures_util|bincode)'` ermitteln; nicht geschehen.
  2. Typen aus `types/erasure.rs` (616 Z.), `ports/erasure*.rs`, `gate.rs`, `ledger.rs`, `observer.rs`, `report_signer.rs`, `keys.rs`, `barrier.rs` (Plan 5.2) kommen hierher, **gegen `types_local.rs` abgeglichen**: Vorgehen laut Plan, Dubletten zuerst.
  3. Die Scanner-Traits `Purger`/`LayerScanner` (`types_local.rs:1853,2033`) bleiben lokal (R7). `ports::scanner` bleibt unverändert. `compat/` bildet beide aufeinander ab; `store/scanner/*`, `vector`, `text`, `graph`, `kvcache` implementieren weiter `ports::scanner`.
  4. `crypto`-Altlast (`audit_chain`, `deletion_proof/`, `ed25519_proof`) erst archivieren, wenn der Diff gegen `erasure` (G8) die Vollständigkeit belegt.
- **Gruppe:** `compliance/`, Tier frozen, `cargo check` wöchentlich (Plan K6).
- **Ziel-Closure:** keine.
- **Prüfung:** `cargo tree -e normal` ohne `contextra-*`; `cargo check --no-default-features`; `rg 'trait (Purger|LayerScanner)'` zeigt je eine Definition in `erasure` und `ports` (bewusst).

#### contextra-license (Ebene 0 → Ebene 2 nach dem Plan-Schnitt)

- **Ist:** 1.137 Z. (`signed_gate.rs` 809, `types.rs` 287). Eigene `Clock`, `TenantId`, `FeatureRing`, `LicenseGate`, `FixedClock`. `default = ["std", "ports-compat"]` (G5). Die Brücke steht inline: `types.rs:~210–290` (`From`-Impls, `impl LicenseGate for OpenFastGate`, Blanket-`Clock`) und `signed_gate.rs:802` (`impl ports::LicenseGate for SignedLicenseGate`). Kein Verwender: `engine`/`kvcache`/`contextra` benutzen `ports::license::{FeatureRing, LicenseGate}`, nicht dieses Crate.
- **Schnitt:** (1) `default = ["std"]`. (2) Inline-Brücken nach `compat/` hinter `workspace`. (3) Plan 5.2 verschiebt `ports/license.rs` (121 Z.) und `policy.rs` (34 Z.) hierher: Danach liegen Trait **und** Implementierung im Crate und `ports` verliert die Lizenz. `contextra` (Fassade) und `engine` hängen dann an `license` (Ebene 2 → Ebene 2, zulässig). `store` und `kvcache` dürfen keine `FeatureRing`-Referenz behalten (`store/lsm/config.rs:68–92`, `kvcache/adapters.rs:135–221` entfallen).
- **Ziel-Closure:** keine (Kern), `workspace` nur für die Brücke.
- **Prüfung:** `rg 'FeatureRing|LicenseGate' crates/contextra-{store,vector,text,graph,kvcache,rank,checkpoint,mvcc}` leer; `rg 'contextra_(types|ports)' src --glob '!compat/**'` leer.

#### contextra-audit-export (Ebene 0)

- **Ist:** 984 Z. (`lib.rs` 386, `bsi_mapping.rs` 152, `markdown_template.rs` 147, `csv_template.rs` 142, `testkit.rs` 126, `error.rs` 31). Eigene `TenantId`; eine Brücke `lib.rs:106–118` hinter `types-compat`, das aber im `default` steht (G5). `testkit.rs` ist `pub mod` ohne Gate.
- **Schnitt:** `default = ["std"]`; Brücke nach `compat/`; `testkit` hinter `test-utils`. Optional später: `compat/privacy.rs` mit `From<ProcessingRegistryEntry>` (hinter Feature `privacy`), um die Überschneidung mit `privacy::processing_registry` zu schließen. Das ist die einzige Stelle, an der ein Compliance-Crate das andere kennen dürfte; entscheide, ob das gewollt ist.
- **Ziel-Closure:** keine.
- **Prüfung:** `rg 'contextra_' src --glob '!compat/**'` leer.

#### contextra-avv-generator (Ebene 0)

- **Ist:** 1.078 Z. (`lib.rs` 358, `template.rs` 308, `error.rs` 97, sechs kleine Abschnittsdateien). Nur `thiserror` und optional `serde`. Doku nennt „Ring 4“ (F10). `error.rs` exportiert `AvvError` und `AvvGeneratorError` (beide in `lib.rs`): doppelte Fehlertypen, Verhältnis nicht geprüft.
- **Schnitt:** Keiner. Nach der Doku-Korrektur und der Klärung der Fehlertypen fertig. Die Vorlagen verweisen im Text auf Funktionen des Kerns (Ed25519-Löschbeweis, Egress, KV-Cache-Verschlüsselung); diese Zusagen müssen mit den Features übereinstimmen, die nach dem Umbau noch existieren. Das ist ein Textabgleich, kein Code.
- **Ziel-Closure:** keine.

### 4.6 integration

#### contextra-engine (Ebene 2, Referenz-Verdrahtung)

- **Ist:** 21.896 Z. (`collection/` 12.522, `transaction/` 1.652, `chunker/` 1.533, `contextra_impl/` 1.449, `background_workers/` 1.298, `extraction/` 642, `lib.rs` 496, `filter.rs` 425, `export.rs` 395, `temporal_filter.rs` 383, `orchestration/` 330, `merge_operator_adapter.rs` 201, `decay_controller.rs` 192, `import.rs` 144). Referenzen: `types` 417 (63 Dateien), `ports` 83 (52), `store` 129, `vector` 129, `graph` 88, `text` 68, `rank` 16, `mvcc` 4, `checkpoint` 4, `adapt` 3, `sandbox` 2. Verwender: `db`, `cognition`. `engine/filter.rs` und `types/filter.rs` sind **keine** Dubletten (Diff 807 Zeilen bei je rund 425).
- **Was herausgelöst wird** (Rest bleibt Verdrahtung):
  - `chunker/` → ersetzt durch `ingest` (K11).
  - `temporal_filter.rs` (383, 10 Referenzen auf `rank`) → `rank/temporal.rs` (Plan 5.4).
  - `decay_controller.rs` (192) → `adapt` (nach Semantikvergleich), `db` und `cognition` importieren von dort.
  - `extraction/` (642) → Feature `exp-extraction`; Überschneidung mit `graph/entity_extraction.rs` (318): Vergleich offen. Features `entity-extraction` (genutzt, `lib.rs:45`) und `auto-extraction-opt-out` ziehen mit.
  - `merge_operator_adapter.rs` (201) bleibt hier: es ist die Brücke `sandbox` ↔ `store`.
- **Bleibt:** `collection/` (CRUD, Suche, Wartung), `transaction/` (inkl. `CommitIntent`, `compensating_actions`), `contextra_impl/`, `background_workers/` (21× `vector`, 16× `store`, 10× `graph`, 8× `text` Referenzen laut Plan), `orchestration/`, `export`, `import`, `filter`.
- **Lizenz:** `contextra_impl/lifecycle.rs:9–55` und `lib.rs:17,22,373,410` binden `FeatureRing`/`LicenseGate` ein; nach dem Verschieben der Lizenz-Ports (siehe `license`) auf `contextra-license` umstellen. `VectorDeleteMode` (`lib.rs:22,246,341`) kommt aus `vector` (Plan 5.4).
- **Features:** `edge-reinforcement-learning`, `reranking`, `adaptive-candidate-pool-sizing` prüfen: `reranking = []` ist leer deklariert (Verwendung nicht geprüft). Experiment-Features einheitlich `exp-*`.
- **Ziel-Closure:** Referenz-Verdrahtung, keine Budgetgrenze.
- **Prüfung:** `rg 'crate::chunker' src` nach der Ersetzung leer; `cargo check --no-default-features` ohne `sandbox`, `onnx`, `adapt`.

#### contextra-db (Ebene 2, Fassade)

- **Ist:** 2.134 Z. nach G9, produktiv rund 1.250: `volatile_vault.rs` 592, `multistep.rs` 442, `pid_latency_controller.rs` 134, `homeostat.rs` 58, `lib.rs` rund 120 produktiv. Verwender: `agent`, `contextra`, `bench`. Abhängigkeiten: produktiv `engine`, `types`, `ports`; `vector` (mit `experimental-diskann`), `store`, `graph`, `text`, `adapt`, `checkpoint` stehen nur unter `[dev-dependencies]`. Das Feature `experimental-diskann` verweist trotzdem auf `contextra-vector/experimental-diskann`; ob Cargo das für eine reine Dev-Abhängigkeit akzeptiert, habe ich nicht geprüft.
- **Schnitt:**
  - `volatile_vault.rs` → `contextra-vault` (4.3).
  - `pid_latency_controller.rs`, `homeostat.rs` → `adapt` (nach Vergleich, siehe `adapt`).
  - `multistep.rs` (`MultiStepEngine`, `QueryRewriter`-Port): bleibt vorerst; ist ein Retrieval-Orchestrator ohne Speicherbezug, aber nur 442 Zeilen. Entnahmekandidat nur bei Bedarf, dann nach `retrieval/` als eigenes Crate mit lokalem `QueryRewriter`.
  - Das Feature `experimental-diskann` zeigt nach Plan 5.4 auf `contextra-vector-diskann`; dann muss das Crate eine echte (optionale) Abhängigkeit werden, nicht nur Dev-Abhängigkeit.
- **Ergebnis:** `db` ist nach dem Schnitt ein Feature-Forwarding-Crate mit `prelude`/`advanced`. Als Entnahmequelle uninteressant, als Einstieg sinnvoll.
- **Prüfung:** `rg 'volatile_vault' crates --glob '!crates/contextra-vault/**'` zeigt nur Re-Exports.

#### contextra (Fassade, Ebene 2)

- **Ist:** 2.023 Z. (`builder.rs` 504, `agent_memory.rs` 414, `performance_profile.rs` 326, `query_rewriter.rs` 272, `lib.rs` 204, `collection_profile.rs` 199, `boot.rs` 104). Abhängigkeiten: `db`, `types`, `ports`, optional `infer-candle`. Verwender: `mcp`, `py`. Aus `ports`/`types` genutzt: `license::{LicenseGate, FeatureRing, LicenseError, GateToken}` (Lizenz-Gating in `builder.rs:33–35,243–350`, `performance_profile.rs` 32 Treffer laut Plan, `collection_profile.rs:33,128`), `ContextraError`, `DistanceMetric`, `AutoExtractionMode`. `lib.rs` re-exportiert `chunker`.
- **Schnitt:** (1) Lizenz-Gating in ein Feature `license` legen, das `contextra-license` einbindet; ohne Feature baut die Fassade ohne Lizenzprüfung. (2) `agent_memory.rs` (414, `remember`/`recall`/`forget`/`relate`) ist die brauchbarste Einstiegs-API und bleibt. (3) `query_rewriter.rs` (272, `LlmQueryRewriter`) gegen `infer-ollama/query_rewriter.rs` (362) auf Doppelung prüfen, nicht verglichen. (4) `performance_profile`/`collection_profile` sind Konfigurationspresets; gehören mit `license`-Gating zusammen.
- **Ziel-Closure:** `db`, `types`, `ports`, optional `license`, optional `infer-candle`.

### 4.7 apps und testing

#### contextra-mcp (Ebene 3)

- **Ist:** 5.839 Z. (`prompt_injection/` 1.162, `tests.rs` 974, `config.rs` 907, `sandbox.rs` 757, `server_tools.rs` 376, `server_dispatch.rs` 358, `explain.rs` 268, `tools_crud.rs` 191, `server.rs` 168, `protocol.rs` 149, `routing.rs` 119). Abhängigkeiten: `contextra`, `types`, `ports`, `wire`, `rank`, `crypto`, `privacy`, `router`, optional `infer-candle/-ollama/-onnx`. Aus `crypto`: nur `CryptoKey` (`sandbox.rs`, 6 Referenzen); aus `rank`: `IsotonicCalibrator`, `ProvenanceRecord`.
- **Schnitt:** (1) `prompt_injection/` → `contextra-prompt-guard` (4.3). (2) `protocol.rs` (149) gegen `wire/jsonrpc.rs` vergleichen; bei Gleichheit auf `wire` umstellen, sonst Unterschied dokumentieren. (3) `egress_*.rs` (4/28/54 Z.) bleiben als dünne Adapter über `privacy`. (4) `sandbox.rs` und `crypto::CryptoKey`: nach dem `crypto`-Umbau bleibt `CryptoKey` im Kern; keine Änderung.
- **Ziel-Closure:** Anwendung, keine Budgetgrenze.
- **Prüfung:** `rg 'prompt_injection' src` nur noch Re-Export.

#### contextra-py (Ebene 3)

- **Ist:** 3.529 Z. (`bindings/` 3.092 mit `collection`, `db`, `document`, `functions`, `hyperedge` und weiteren, `kv_links.rs` 296, `lib.rs` 141). Abhängigkeiten: `types`, `contextra`, `wire`, `flatbuffers`, `tokio`; `pyo3`, `numpy`, `pythonize` optional. Feature `docid-128 = ["contextra-types/docid-128"]`.
- **Schnitt:** Keiner außer dem Feature-Forwarding: nach Plan 5.1 besitzt `ids` das Feature `docid-128`; `py` leitet an `ids` (oder an `types`, das `ids` weiterleitet) weiter. `wire` bleibt (FlatBuffers-Pfad). Reine Bindungsschicht, keine Entnahmekandidaten.
- **Prüfung:** `cargo check` ohne und mit `--features pyo3` getrennt.

#### contextra-testkit (Ebene 3)

- **Ist:** 4.063 Z. (`conformance/` 1.612, `fault_vfs.rs` 457, `in_memory_store.rs` 408, `mocks.rs` 302, `reference_model.rs` 283, `in_memory_graph.rs` 251, `in_memory_vector.rs` 231, `in_memory_text.rs` 206, `manual_clock.rs` 172). Abhängigkeiten: `ports` (Feature `testkit`), `types`, `bytes`, `parking_lot`. In `Cargo.toml` aufgeführt bei `agent` (dev), `checkpoint`, `cognition` (dev), `crypto`, `db`, `engine`, `mvcc`, `store`, `testkit` (Sektion bei den anderen nicht einzeln geprüft). `default` enthält `erasure-conformance` (G5).
- **Schnitt:** Damit Ebene-0/1-Bausteine nicht von `testkit` abhängen müssen:
  - `conformance/{clock,rng,metrics}.rs` und `manual_clock.rs` → `env` (Feature `test-utils`, Plan 5.1).
  - `conformance/vfs.rs` und `fault_vfs.rs` → `durable-fs` (zusammen mit `vfs`, Plan 5.1).
  - `ports/conformance.rs` (278), `ports/contract_tests.rs` (632) kommen laut Plan hierher; vorher gegen `conformance/` (1.612 Z.) auf Überschneidung prüfen.
  - `erasure-conformance` raus aus `default`; Suite wandert nach `erasure` (Feature `conformance`).
  - Rest (`in_memory_*`, `mocks`, `reference_model`, `conformance/{storage,text,vector,graph}`) bleibt hier, weil er `ports`-Traits implementiert.
- **Ziel-Closure:** `ports`, `types`, `env`, `durable-fs` (nur für `fault_vfs`).
- **Prüfung:** `cargo tree -p contextra-vector -e normal --edges dev` zeigt nach dem Schnitt keinen Pfad zu `testkit` für Ebene-0-Crates (nicht ausgeführt).

## 5. Reihenfolge

Die Pakete folgen dem Prinzip deines Plans: ein Paket, ein Commit, kein Umbau im Verschiebe-Commit. Jedes Paket ist von seinem Vorgänger abhängig, wo „nach“ steht.

1. **P1 – Ebene-0-Hygiene** (unabhängig, ohne Risiko): `default` bereinigen bei `license`, `audit-export`, `adapt`, `testkit` (G5); leere Features streichen (G6); Docstrings korrigieren (`privacy`, `cognition`, `kvcache`, `avv-generator`); `ports_local.rs` und deprecated Re-Exports in `router` entfernen (nach Umstellung der drei `mcp`-Importe).
2. **P2 – Brücken ausziehen bei den Ebene-0-Crates**: `license/compat/`, `audit-export/compat/`. Rein mechanisch.
3. **P3 – Neue Bausteine**: `contextra-prompt-guard` (aus `mcp`), `contextra-vault` (aus `db`). Beide ohne Abhängigkeit zu anderen Paketen.
4. **P4 – nach `ids`/`env` aus dem Plan**: `kvcache` (nach `crypto`-KV-Umzug, siehe Plan 5.1), `infer-ollama`, `infer-candle`: Brücken, lokale Fehlertypen, Feature-Schnitt.
5. **P5 – `ModelFingerprint` nach `ports/kv`** (K9). Muss **vor** P4 für `kvcache` und `infer-candle` liegen, sonst entsteht ein Zwischenzustand mit `ports → kvcache`.
6. **P6 – Chunker-Ersatz** (K11): Paritätstest, Adapter, Entfernung.
7. **P7 – Richtung umkehren**: `cognition` (lokale Ports, Brücke in `db`), danach `agent` (lokale Ports, Brücke in `compat/`).
8. **P8 – Konsolidierungen mit Semantikvergleich** (offen, nicht automatisierbar): Regler (`adapt` ↔ `engine`/`db`/`rank`), `protocol.rs` ↔ `wire/jsonrpc.rs`, `extraction` ↔ `entity_extraction`, `sys/vault` ↔ `contextra-vault`, `query_rewriter` (`contextra` ↔ `infer-ollama`).

## 6. Offene Punkte und Grenzen dieser Analyse

- **Nicht geprüft:** ob `cargo check --no-default-features` heute für irgendeins dieser Crates gelingt. Alle Aussagen zu Gates sind statisch (Dateiebene, nicht zeilenweise `cfg`-Zuordnung, wo nicht ausdrücklich genannt).
- **Zeilenzahlen** sind `src/`-Zählungen per `wc`; Inline-Tests sind enthalten. Zahlen aus deinem Plan habe ich nicht neu gezählt, außer wo ich sie hier nenne.
- **Verwender-Aussagen** („kein Verwender“) beruhen auf einem Scan aller `Cargo.toml` im Workspace und `benchmarks/`; Beispiele, `tests/` im Wurzelverzeichnis und `examples/` habe ich nicht geprüft.
- **Rng-Signaturen** von `vector`, `text`, `graph` habe ich nicht mit `ports` verglichen, nur `adapt`. Für `env` bleibt der Pflichtschritt aus deinem WP-1 bestehen.
- **Zwei Semantikvergleiche** liegen außerhalb dessen, was statische Zählung leistet: die Regler-Dubletten und die JSON-RPC-Dublette. Die Spezifikation behandelt beide als offen, nicht als erledigt.
- **Ebene-0-Regel (K8)** ist meine Auslegung deines Ebenenmodells. Gibst du Ebene 0 stattdessen „höchstens `ids`/`env`“ als Budget, entfallen K8 und die Ausnahmen bei `ingest`, `adapt`, `privacy`, `license`, `erasure`; der Preis ist, dass diese Crates ohne `ids` und `env` nicht mehr entnehmbar sind.
