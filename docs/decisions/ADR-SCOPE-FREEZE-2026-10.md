# ADR-SCOPE-FREEZE-2026-10: Scope Freeze & Verbindliche Crate-Klassifikation (K, R, E)

* **Datum**: 2026-10-12
* **Status**: 📝 Vorschlag
* **Rolle**: Produkt-Architekt
* **Kontext**: Spezifikation §6.0, Projektbewertung 10.9, ADR-N08, Ring-Modell v2

---

## 1. Kontext & Problemstellung

Im Rahmen der Vorbereitung für das Release v0.1.0/v1.0 von Contextra muss der Funktionsumfang verbindlich eingefroren (Scope Freeze) und die Stabilitätsversprechen für das Gesamtsystem klar gegenüber Anwendern und Auditoren abgegrenzt werden.

Das Contextra-Workspace umfasst derzeit 34 Crates (31 Fach-Crates plus Tooling/Benchmarks/xtask). Gemäß Spezifikation §6.0 und Projektbewertung 10.9 müssen alle Komponenten und Crates in genau eine von drei Verbindlichkeits-Klassen eingestuft werden:

- **Klasse K (Kern / Core)**: Das stabile, deterministische Fundament für lokale RAG- und Vektordatenbank-Operationen. Beinhaltet Ring-0-Primitives, Persistenz, Indexierung sowie Basisausführung. Höchste Qualitäts- und Stabilitätsgarantien (Zero-Panic, deterministische Recovery, volle Abwärtskompatibilität).
- **Klasse R (Rand / Peripheral)**: Äußere Schnittstellen, Adapter, FFI-Bindings, Sandbox-Sicherheitsgrenzen und Endanwender-Fassaden. Garantieren Schnittstellenstabilität und Sicherheit an den Systemgrenzen, hängen jedoch von externen Laufzeiten oder Anwenderumgebungen ab.
- **Klasse E (Experimentell / Experimental)**: Erweiterte kognitive Schleifen, adaptive Heuristiken, experimentelle Algorithmen, dynamische Agenten-Pipelines sowie automatisierte Compliance-Generatoren. Komponenten dieser Klasse fallen ausdrücklich aus den zentralen Stabilitäts-, Determinismus- und SemVer-Garantien des Kerns heraus.

---

## 2. Verbindliche Klassifikation aller Workspace-Crates (§6.0)

Die folgende Tabelle klassifiziert jede im Workspace vorhandene Crate verbindlich in Klasse K, R oder E:

| Crate Name | Ring (capabilities.toml) | Klasse | Beschreibung & Scope-Freeze Status |
|---|---|---|---|
| `contextra-types` | Ring 0 | **K** | Typen, IDs (`DocId`, `TxId`), Invarianten & Fehler-Taxonomie (Kern) |
| `contextra-ports` | Ring 0 | **K** | Trait-Definitionen & Entkopplungs-Schnittstellen (Kern) |
| `contextra-mvcc` | Ring 0 | **K** | MVCC Snapshot-Isolation & Concurrency Primitives (Kern) |
| `contextra-wire` | Ring 0 | **K** | FlatBuffers Zero-Copy Wire-Protokolle (Kern, Unsafe-Insel) |
| `contextra-sys` | Ring 0 | **K** | High-Performance I/O & Memory-Mapping (Kern, Unsafe-Insel) |
| `contextra-simd` | Ring 0 | **K** | SIMD-beschleunigte Distanzberechnungen (Kern, Unsafe-Insel) |
| `contextra-crypto` | Ring 0 | **K** | AES-256-GCM-SIV, Zeroize, HMAC-WAL-Kette & Deletion-Proof (Kern) |
| `contextra-durable-fs` | Ring 0 | **K** | Synchrone Disk-Flushes, `sync_dir`, `atomic_replace` (Kern) |
| `contextra-vector` | Ring 0 | **K** | HNSW, DiskANN Vektor-Indizes & Quantisierung (Kern) |
| `contextra-text` | Ring 0 | **K** | Invertierter BM25/BM25F-Volltext-Index (Kern) |
| `contextra-core` | Ring 0 | **K** | Strangler-Fassade / Re-Export-Layer für Ring-0-Primitives (Kern) |
| `contextra-store` | Ring 1 | **K** | LSM-Tree, WAL Durability & Compaction Engine (Kern) |
| `contextra-kvcache` | Ring 1 | **K** | Stufenmodell KV-Cache Storage & Offloading (Kern) |
| `contextra-checkpoint` | Ring 1 | **K** | Blake3 Checkpoint Engine & Time-Travel Storage (Kern) |
| `contextra-engine` | Ring 3 | **K** | Compute Pool & Asynchrone Storage Execution Engine (Kern) |
| `contextra-db` | Ring 3 | **K** | Hybrid Search Engine & Main Collection Database Abstraction (Kern) |
| `contextra-privacy` | Ring 3 | **K** | Egress Filtering, DLP, PII Vault & Guarded Payload (Kern) |
| `contextra` | Ring 4 | **R** | Haupt-Library-Fassade & Composition Root |
| `contextra-mcp` | Ring 4 | **R** | Stdio JSON-RPC Model Context Protocol Server & Gateway |
| `contextra-sandbox` | Ring 2 | **R** | WASM Execution Boundary & Isolations-Sandbox |
| `contextra-infer-candle` | Ring 2 | **R** | Lokales Candle LLM Inferenz- & KV-Bridge-Modul |
| `contextra-infer-ollama` | Ring 2 | **R** | Ollama Remote Provider Client |
| `contextra-py` | Ring 4 | **R** | PyO3 Python Bindings & FFI-Panicschutz |
| `contextra-testkit` | Tooling | **R** | Fault-VFS, Mock-Engines & Test-Harness |
| `contextra-agent` | Ring 3 | **E** | Agent Execution Pipeline & In-Memory Audit (Experimentell) |
| `contextra-router` | Ring 3 | **E** | Multi-Candidate SLM Kaskaden-Routing (Experimentell) |
| `contextra-cognition` | Ring 3 | **E** | Consolidation & Sleep Cycle Pass (Experimentell) |
| `contextra-graph` | Ring 0 | **K / E** | CSR Graph-Engine (**K**), ausgenommen `graph-PPR` (**E**) |
| `contextra-rank` | Ring 0 | **K / E** | 4-Signal-Fusion & Platt/Isotonic Scaler (**K**), ausgenommen `rank-DRIFT` (**E**) |
| `contextra-adapt` | Ring 0 | **E** | PID Controller, Bandit & Lyapunov Drift Control (Experimentell) |
| `contextra-infer-onnx` | Ring 2 | **E** | ONNX Runtime Native Client (Experimentell) |
| `contextra-license` | Ring 4 | **E** | Lizenz- & Aktivierungsprüfung (Experimentell) |
| `contextra-audit-export` | Ring 0 | **E** | DSGVO Art. 30 Verarbeitungsverzeichnis-Exporter (Experimentell) |
| `contextra-avv-generator` | Ring 0 | **E** | Art. 28 DSGVO AVV-Template Generator (Experimentell) |

---

## 3. Ausweisung der Experimentell-Crates & Subkomponenten (Projektbewertung 10.9)

Ausdrücklich aus den Kern-Stabilitätsaussagen, Garantie-Zusagen und der API-Schnittstellenkonstanz ausgenommen sind die folgenden **7 Hauptbereiche (Crates und isolierte Subkomponenten)** gemäß Projektbewertung 10.9:

1. **`contextra-agent`**: Agenten-Workflow-Pipeline, Prompt-Synthese und dynamische Aufruf-Orchestrierung.
2. **`contextra-router`**: Dynamisches SLM-Kaskadenrouting und Multi-Candidate-Profilauswahl.
3. **`contextra-cognition`**: Hintergrund-Konsolidierung, Schlafzyklus-Kompaktierung und Session-Grouping.
4. **`graph-PPR`** (Subkomponente in `contextra-graph`): Personalized PageRank Diffusions-Algorithmen und approximatives PPR-Routing.
5. **`rank-DRIFT`** (Subkomponente in `contextra-rank`): Adaptives Drift-Monitoring, Conformal-Prediction-Rescaling und dynamisches Score-Refactoring.
6. **`contextra-license`**: Lizenzierungs-Gates und digitale Schlüssel-Aktivierungsprüfungen.
7. **`contextra-avv-generator`**: Automatische Vorlagen-Generierung von Auftragsverarbeitungsverträgen (Art. 28 DSGVO).

*Zusätzlich* bleiben als experimentell ausgewiesen: `contextra-adapt` (PID Controller / Bandit), `contextra-audit-export` (Art. 30 Exporter) sowie `contextra-infer-onnx` (ONNX C-Library Backend).

---

## 4. Offene Fragen & Abweichungen zur Spezifikation (§6.0 vs. `capabilities.toml`)

Bei der Gegenüberstellung von `capabilities.toml` und Spezifikation §6.0 / Projektbewertung 10.9 wurden folgende systematische Diskrepanzen identifiziert. Diese werden hiermit als **offene Fragen** zur späteren Bereinigung festgehalten:

1. **Diskrepanz im Reifegrad-Marker (`maturity`)**:
   - In `capabilities.toml` sind `contextra-agent`, `contextra-router`, `contextra-cognition`, `contextra-graph` und `contextra-rank` mit `maturity = 'stable'` eingetragen.
   - *Offene Frage*: Soll im nachfolgenden W0-Prompt zur Bereinigung von `capabilities.toml` für `agent`, `router` und `cognition` das Attribut `maturity` auf `'experimental'` umgestellt werden, sowie für `graph` und `rank` eine Kennzeichnung der experimentellen Submodule (`graph-PPR`, `rank-DRIFT`) in den Capabilities-Flags erfolgen?

2. **Diskrepanz bei der Ring-Zuordnung (`ring`)**:
   - `capabilities.toml` führt `contextra-audit-export` und `contextra-avv-generator` in `Ring 0`.
   - Spezifikation III.21 und III.22 klassifizieren beide Komponenten als reine `Ring 4` Utility-Module.
   - *Offene Frage*: Sollen `contextra-audit-export` und `contextra-avv-generator` im W0-Prompt von `Ring 0` nach `Ring 4` verschoben werden, um der Schichtenarchitektur (keine Business-Utility-Logik in Ring 0) zu entsprechen?

3. **Keine direkte Änderung an `capabilities.toml` in diesem ADR**:
   - Gemäß Arbeitsanweisung bleibt `capabilities.toml` in diesem ADR unberührt. Die Anpassungen an `capabilities.toml` erfolgen im separaten Umsetzungs-Prompt (W0).

---

## 5. Konsequenzen

- **Klarheit für Anwender**: Nutzer des Core-SDKs verlassen sich auf Stabilisierungszusagen in Klasse K.
- **Entwicklungs-Freiheit**: Module in Klasse E können ohne Breaking Changes am Kern weiterentwickelt oder überarbeitet werden.
- **Unveränderlichkeit der Konfiguration**: `capabilities.toml` bleibt in diesem Schritt unberührt; die Spezifikation §6.0 ist mit dieser ADR verbindlich formuliert.
