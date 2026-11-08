# Contextra – Architektur (Baukasten-System)

| Feld | Wert |
|---|---|
| Stand | 2026-11-08 |
| Status | Nicht normativ. Normativ: `docs/SPEC.md` Fassung 4.1 |
| Messmethode | Statische Auswertung von `Cargo.toml`, `capabilities.toml`, `git grep` auf `src/` |
| Architekturmodell | Baukasten-Architektur: Klassen A, B, C, T & Schichtenringe 0–4 |

---

## 1. Überblick

Contextra ist ein Cargo-Workspace aus isolierten, wiederverwendbaren Bausteinen („Building Blocks“) unter `crates/`. Der Workspace umfasst 35 Crates sowie das Integrations-Benchmark `benchmarks/contextra-bench`. Die historische Strangler-Fassade `contextra-core` wurde im Rahmen der Modularisierung vollständig dekommissioniert und entfernt.

Das Gesamtsystem folgt einer zweidimensionalen Ordnung:
1. **Baukasten-Klassifikation (Klassen A, B, C, T)** gemäß `docs/SPEC.md` §5.
2. **Architektur-Ringe (Ringe 0 bis 4)** zur Sicherung der hierarchischen Schichtung und DAG-Topologie.

---

## 2. Baukasten-Klassifikation (Klassen A, B, C, T)

| Klasse | Bezeichnung | Kriterien & Rolle | Enthaltene Crates |
|---|---|---|---|
| **Klasse A** | **Autarke Bausteine** | Erfüllen die Kriterien B1–B12 lückenlos: 0 interne Abhängigkeiten zu `contextra-*`, API ausschließlich in Standard-Rust-Typen oder lokalen Typen, lokales Fehlermodell (`thiserror`, `#[non_exhaustive]`), isolierter Standalone-Build (`tools/export-crate`), vollständige Entkopplung von Lösch- und Domänenlogik fremder Schichten. | `contextra-sys`, `contextra-simd`, `contextra-wire`, `contextra-rank`, `contextra-text`, `contextra-vector`, `contextra-graph`, `contextra-kvcache`, `contextra-durable-fs`, `contextra-crypto`, `contextra-erasure`, `contextra-privacy`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-ingest`, `contextra-sandbox`, `contextra-adapt`, `contextra-infer-onnx`, `contextra-infer-ollama`, `contextra-infer-candle` |
| **Klasse B** | **Storage-Bundle** | Gemeinsam kopierbare Speichereinheit mit kohärenter Transaktions- und LSM-Semantik (`TxId`, `DocId`, Snapshot-Isolation). Klasse-A-Bausteine dürfen keine Typen aus Klasse B importieren. | `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-store`, `contextra-checkpoint` |
| **Klasse C** | **Komposition & Integration** | Anwendungsschichten, Adapter und Endnutzer-Bindings. Dienen ausschließlich der Orchestrierung autarker Bausteine und enthalten keine originäre Index- oder Rechenlogik. | `contextra`, `contextra-db`, `contextra-engine`, `contextra-cognition`, `contextra-agent`, `contextra-mcp`, `contextra-py`, `contextra-router` |
| **Klasse T** | **Tooling & Testkit** | Test-Doubles, Fault-Injektion und Benchmarks; nur für Tests innerhalb von Klasse B/C zugelassen. | `contextra-testkit`, `benchmarks/contextra-bench` |

---

## 3. Ringmodell

| Ring | Rolle | Enthaltene Crates |
|---|---|---|
| **0** | Fundament: Typen, Ports, System-Primitive, Unsafe-Inseln, elementare Indizes, Volltext, Rang, Krypto, Graph | `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-simd`, `contextra-wire`, `contextra-mvcc`, `contextra-crypto`, `contextra-rank`, `contextra-text`, `contextra-vector`, `contextra-graph`, `contextra-adapt`, `contextra-durable-fs`, `contextra-audit-export`, `contextra-avv-generator` |
| **1** | Speicher, Transaktions-Persistenz & KV-Cache | `contextra-store`, `contextra-checkpoint`, `contextra-kvcache` |
| **2** | Inferenz-Adapter & WASM-Sandbox | `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-sandbox` |
| **3** | Engine, Routing, Privacy, Agent, DB & Löschkern | `contextra-engine`, `contextra-router`, `contextra-privacy`, `contextra-agent`, `contextra-db`, `contextra-cognition`, `contextra-erasure` |
| **4** | Oberfläche & Bindings: Facade, MCP, Python, Ingest, Lizenz | `contextra`, `contextra-mcp`, `contextra-py`, `contextra-ingest`, `contextra-license` |
| **Tooling** | Testkit & Benchmarking | `contextra-testkit`, `benchmarks/contextra-bench` |

---

## 4. Crate-Inventar

Stand: 2026-11-08. Quellen: `Cargo.toml`, `capabilities.toml` und Quellcode.

| Crate | Klasse | Ring | Reife | Unsafe-Insel | Interne Abhängigkeiten (`may_depend_on`) |
|---|---|---|---|---|---|
| `contextra` | C | Ring 4 | stable | nein | audit-export, avv-generator, crypto, db, infer-candle, license, ports, privacy, rank, store, types |
| `contextra-adapt` | A | Ring 0 | beta | nein | ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-agent` | C | Ring 3 | beta | nein | checkpoint, db, graph, ports, router, store, types |
| `contextra-audit-export` | A | Ring 0 (Ext.) | experimental | nein | ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-avv-generator` | A | Ring 0 (Ext.) | experimental | nein | types *(in Entkopplung zu 0 Deps)* |
| `contextra-checkpoint` | B | Ring 1 | stable | nein | mvcc, ports, types |
| `contextra-cognition` | C | Ring 3 | experimental | nein | engine, graph, ports, store, types, vector |
| `contextra-crypto` | A | Ring 0 | beta | nein | ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-db` | C | Ring 3 | beta | nein | adapt, checkpoint, cognition, crypto, engine, graph, ports, rank, store, sys, text, types, vector |
| `contextra-durable-fs` | A | Ring 0 | beta | nein | sys *(in Entkopplung zu 0 Deps)* |
| `contextra-engine` | C | Ring 3 | beta | nein | checkpoint, crypto, graph, mvcc, ports, rank, sandbox, store, sys, text, types, vector |
| `contextra-erasure` | A | Ring 3 | beta | nein | crypto, ports, types |
| `contextra-graph` | A | Ring 0 | stable | nein | adapt, ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-infer-candle` | A | Ring 2 | stable | nein | crypto, ports, rank, types *(in Entkopplung zu 0 Deps)* |
| `contextra-infer-ollama` | A | Ring 2 | beta | nein | ports, rank, types *(in Entkopplung zu 0 Deps)* |
| `contextra-infer-onnx` | A | Ring 2 | experimental | nein | ports, rank, types *(in Entkopplung zu 0 Deps)* |
| `contextra-ingest` | A | Ring 4 | experimental | nein | engine, types *(in Entkopplung zu 0 Deps)* |
| `contextra-kvcache` | A | Ring 1 | beta | nein | crypto, ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-license` | A | Ring 4 | experimental | nein | ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-mcp` | C | Ring 4 | beta | nein | contextra, crypto, infer-candle, infer-ollama, infer-onnx, license, ports, privacy, rank, router, types, wire |
| `contextra-mvcc` | B | Ring 0 | stable | nein | ports, types |
| `contextra-ports` | B | Ring 0 | stable | nein | types |
| `contextra-privacy` | A | Ring 3 | stable | nein | avv-generator, ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-py` | C | Ring 4 | stable | nein | adapt, db, rank, store, types, wire |
| `contextra-rank` | A | Ring 0 | stable | nein | ports, types *(vollständig autark als Klasse A)* |
| `contextra-router` | C | Ring 3 | stable | nein | adapt, ports, privacy, types, wire |
| `contextra-sandbox` | A | Ring 2 | stable | nein | *(0 interne Abhängigkeiten)* |
| `contextra-simd` | A | Ring 0 | beta | **ja** | types *(in Entkopplung zu 0 Deps)* |
| `contextra-store` | B | Ring 1 | beta | nein | crypto, mvcc, ports, sys, types |
| `contextra-sys` | A | Ring 0 | stable | **ja** | *(0 interne Abhängigkeiten)* |
| `contextra-testkit` | T | Tooling | stable | nein | ports, types |
| `contextra-text` | A | Ring 0 | beta | nein | ports, types *(in Entkopplung zu 0 Deps)* |
| `contextra-types` | B | Ring 0 | stable | nein | *(0 interne Abhängigkeiten)* |
| `contextra-vector` | A | Ring 0 | beta | nein | crypto, mvcc, ports, simd, sys, types *(in Entkopplung zu 0 Deps)* |
| `contextra-wire` | A | Ring 0 | stable | **ja** | *(0 interne Abhängigkeiten)* |

---

## 5. Unsafe-Inseln und Speichersicherheit

Gemäß `docs/SPEC.md` §2 B10 und `capabilities.toml` ist `unsafe` im gesamten Workspace strikt verboten (`#![forbid(unsafe_code)]`), mit Ausnahme von genau drei deklarierten Low-Level-Inseln:
1. **`contextra-sys`**: Kapselung direkter POSIX- und OS-Systemaufrufe für Speicherschutz (`mlock`, `munlock`, `madvise`), Direct-I/O und Fsync.
2. **`contextra-simd`**: Hardwarenahe SIMD-Vektorbeschleunigung (AVX2, AVX-512, NEON) mit formalem `// SAFETY:`-Beweis an jedem Block und deterministischem Skalar-Fallback.
3. **`contextra-wire`**: Zero-Copy-Serialisierung und FlatBuffers IPC-Framing.

Alle anderen Crates (inklusive `contextra-vector`, `contextra-store`, `contextra-crypto` etc.) sind zu 100 % Safe Rust.

---

## 6. Kapselung der Löschlogik und DSGVO

Gemäß `docs/SPEC.md` §6 ist sämtliche DSGVO-, Deletion-Proof-, Saga- und Audit-Logik vollständig in `contextra-erasure` gekapselt:
- Datenhaltende Indizes (`vector`, `text`, `graph`, `store`) implementieren neutrale, technische Operationen (`insert`, `remove`, `compact`).
- Die Anbindung realer Speicherkerne an die Lösch-Engine erfolgt über das Adapter-Muster (`contextra-erasure-adapters`).
- Keine Vermischung von Rechtskonzepten, Sagas oder Deletion-Proof-Zertifikaten mit mathematischen Kernen oder Speichertreibern.
