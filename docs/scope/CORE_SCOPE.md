# Contextra — Produktkern & Scope-Abgrenzung (CORE_SCOPE.md)

**Stand:** 2026-10-12
**Autor:** Governance Architecture Board / Workspace Architect
**Status:** Aktiv

---

## 1. Übersicht & Zielsetzung

Dieses Dokument definiert den verbindlichen Produktkern der **Contextra Memory Engine** (lokaler Speicher für KI-Agenten mit nachweisbarem Vergessen) sowie die Trennung zwischen Kernkomponenten und experimentellen Erweiterungen.

---

## 2. Kern-Crates (Produktkern / Core Scope)

Die folgenden Crates bilden das normative Produktfundament von Contextra. Sie zeichnen sich durch Stabilität, vollständige Invariantenprüfung (Zero-Panic, Deadlockfreiheit, Crash-Dauerhaftigkeit) und strikte Ring-Einhaltung aus.

* **Ring 0 (Foundation & Pure Numerics/Types):**
  - `contextra-types`: Kanonische Domain-Typen, IDs, Budgets, Filter-AST, Error-Klassen.
  - `contextra-ports`: Port-Schnittstellen (Traits) für Inversion of Control.
  - `contextra-core`: Strangler-Fassade für abwärtskompatible Ring-0-Re-Exports.
  - `contextra-sys`: Low-Level OS-System-Bindings (mmap, mlock, VirtualLock, ACLs) — Unsafe-Insel.
  - `contextra-durable-fs`: Synchronisierende Dateisystem-Garantien und Fsync-Operationen.
  - `contextra-wire`: FlatBuffers-Wire-Protokoll und Zero-Copy IPC-Adapter — Unsafe-Insel.
  - `contextra-mvcc`: Multi-Version Concurrency Control, Sequence-Log, Isolation.
  - `contextra-crypto`: Encryption at Rest, Deletion-Proofs, WAL-HMAC, KeyRegistry, RevocationLog.
  - `contextra-simd`: SIMD-beschleunigte Vektordistanzberechnungen — Unsafe-Insel.
  - `contextra-vector`: HNSW- und DiskANN-Vektorindizierung mit Tombstone-Pruning.
  - `contextra-text`: BM25-Volltextsuche und Inverted Indexing.
  - `contextra-graph`: CSR-Graph-Engine für Entitäts- und Pfad-RAG (Basisfunktionen).
* **Ring 1 (Persistence & Cache):**
  - `contextra-store`: LSM-Tree Storage Engine, WAL v3 Group Commit, Compaction, Compaction-Merging.
  - `contextra-checkpoint`: Persistent Checkpoint Engine, Blake3 Manifest-Verifikation.
* **Ring 3 (Orchestration & Execution):**
  - `contextra-engine`: Compute Pool & Speicher-Orchestrierungs-Engine.
  - `contextra-db`: Haupt-Datenbank-Abstraktion (Embedded Hybrid-Search).
* **Ring 4 (Boundary & Composition Roots):**
  - `contextra`: Offizielle Haupt-Fassade & Composition Root.
  - `contextra-mcp`: Model Context Protocol Server & IPC Gateway.
  - `contextra-py`: Python-Bindings (PyO3 FFI, FFI-Isolationsgrenze, per Spezifikation §0.5/A.3 außerhalb von `default-members`).
* **Tooling:**
  - `contextra-testkit`: Test-Fixtures, ManualClock, FaultVfs, Mock-Engines (nur als Dev-Hilfe / Testkit).

---

## 3. Experimentelle Crates (Experimental / Aus default-members entfernt)

Die folgenden Crates befinden sich in aktiver Entwicklung oder stellen optionale/schwergewichtige Erweiterungen dar. Sie sind aus `default-members` im Root-Cargo.toml entfernt oder ausgeschlossen:

* **Aus default-members entfernt (Maturity = 'experimental'):**
  - `contextra-agent`: Agent Workflow Engine, Audit & Execution Pipeline.
  - `contextra-router`: Multi-Candidate Search Routing & Cascade Engine.
  - `contextra-adapt`: Adaptations-Layer (PID Latency Controller, Homeostat, Banditen).
  - `contextra-cognition`: Konsolidierungs- und Background Sleep Cycle Passes.
  - `contextra-kvcache`: Stufenmodell KV-Cache & Radix-Tree Storage.
  - `contextra-infer-candle`: Native Candle LLM Inference Client & Adapter.
* **Bereits zuvor ausgeschlossen (per Spezifikation §0.5 / A.3):**
  - `contextra-sandbox`: WASM Execution Boundary (`wasmtime` Isolation).
  - `contextra-infer-onnx`: ONNX Runtime Embeddings Provider.
  - `contextra-infer-ollama`: Ollama External Process Provider.
  - `contextra-py`: PyO3 Python FFI Bindings.

---

## 4. Unentschieden (Braucht Produktentscheidung)

Die folgenden Crates sind im Workspace vorhanden, stehen in keiner der beiden obigen Listen und verbleiben vorerst unverändert:

- `contextra-rank`: 4-Signal Fusion, Platt/Isotonische Kalibrierung, Drift-Detektor.
- `contextra-privacy`: Egress Filtering, Anonymisierung & DLP Compliance.
- `contextra-license`: Lizenz- und Aktivierungsprüfung für Feature-Ringe.
- `contextra-avv-generator`: AVV Template Generator (Art. 28 DSGVO).
- `contextra-audit-export`: DSGVO Art. 30 Verarbeitungsverzeichnis-Export.
- `contextra-bench`: Reproduzierbarer Benchmark-Harness.

---

## 5. Transitive Experimental-Abhängigkeiten (Befund & Folgeaufgabe)

**Befund:**
Obwohl `contextra-agent`, `contextra-router`, `contextra-adapt`, `contextra-cognition`, `contextra-kvcache` und `contextra-infer-candle` aus `default-members` entfernt wurden, zieht `contextra-engine` über seine Cargo `default`-Feature-Konfiguration weiterhin transitive Abhängigkeiten zu `graph`, `rank`, `adapt`, `kvcache` und `candle` ein.

**Entscheidung:**
Diese Cargo-Feature-Entkopplung wird in einer separaten Sitzung / Folgeaufgabe behandelt (Keine Cargo.toml-Dependency- oder Feature-Änderungen in dieser Sitzung gemäß Arbeitsbereich-Regeln).
