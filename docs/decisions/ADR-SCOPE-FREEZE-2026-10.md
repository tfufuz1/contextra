# ADR-SCOPE-FREEZE-2026-10: Scope Freeze & Crate-Klassifizierung (Kern, Rand, Experimentell)

* **Datum:** 2026-10-12
* **Status:** Vorschlag / Proposed
* **Rolle:** Produkt-Architekt
* **Kontext / Referenzen:** Spezifikation §6.0, Projektbewertung 10.9, ADR-N08 (`capabilities.toml`)

---

## 1. Kontext & Zielsetzung

Das Contextra-Projekt umfasst 34 Rust-Workspace-Crates mit unterschiedlichen Aufgabenbereichen. Um unbeabsichtigten Scope Creep zu verhindern, Ressourcen auf die Kern-Engine zu fokussieren und verlässliche Stabilitätsversprechen (SemVer, Invarianten-Garantien) abzugeben, wird hiermit ein verbindlicher **Scope Freeze** definiert.

Gemäß Spezifikation §6.0 werden alle Crates sowie spezifische Submodule in drei eindeutige Klassen unterteilt:
- **Klasse K (Kern / Core Engine):** Das kritische Fundament (Storage, MVCC, Indexierung, Vektor/Text/Graph-Kerne, Kryptographie, Compute-Pool). Höchste Anforderungen an Invarianten, Testabdeckung und Stabilität.
- **Klasse R (Rand / Peripherie & FFI):** Schnittstellen, Endnutzer-Facade, Protokoll-Server, Inferenz-Adapter, Python-Bindings und Isolations-Sandboxes.
- **Klasse E (Experimentell):** Forschungspfade, Zusatzgeneratoren, adaptive Routing-/Bandit-Module und fortgeschrittene Graph-Verfahren. Diese Komponenten sind **aus allen Stabilitäts- und Abwärtskompatibilitätsgarantien ausgenommen**.

---

## 2. Zuordnung aller 34 Workspace-Crates (K, R, E)

| Crate | Ring | Klasse | Kurzbeschreibung & Scope-Einordnung |
| :--- | :--- | :--- | :--- |
| `contextra` | Ring 4 | **R** | Haupt-Library Facade für Endanwender |
| `contextra-adapt` | Ring 0 | **E** | PID Latency Controller / Modell-Adaptierung |
| `contextra-agent` | Ring 3 | **E** | Agent Execution Pipeline & InMemoryStorageEngine (*fällt aus Stabilitätsaussagen*) |
| `contextra-audit-export` | Ring 0 | **E** | Audit Log & DSGVO Art. 30 Verarbeitungsverzeichnis-Export (*AVV-/Audit-Zusatz*) |
| `contextra-avv-generator` | Ring 0 | **E** | AVV (Auftragsverarbeitungsvertrag) Template Generator gemäß Art. 28 DSGVO |
| `contextra-checkpoint` | Ring 1 | **K** | Persistent Checkpoint Engine & Blake3 Manifest-Verifikation |
| `contextra-cognition` | Ring 3 | **E** | Konsolidierungs- & Background Sleep Passes (*fällt aus Stabilitätsaussagen*) |
| `contextra-core` | Ring 0 | **K** | Kern-Datenstrukturen, Tombstones, Invarianten & Memory-Buffer |
| `contextra-crypto` | Ring 0 | **K** | Kryptographische Vaults, Zeroize, Egress-Verschlüsselung & HMAC |
| `contextra-db` | Ring 3 | **K** | Database Abstraction, Hybrid Search & Collection Management |
| `contextra-durable-fs` | Ring 0 | **K** | Durable Filesystem Operations & synchrone Disk-Flushes |
| `contextra-engine` | Ring 3 | **K** | Bounded Compute Pool & Asynchrone Storage-Execution Layer |
| `contextra-graph` | Ring 0 | **K*** | CSR Graph-Engine & Traversal (*Kern K; Submodul PPR -> E*) |
| `contextra-infer-candle` | Ring 2 | **R** | Candle In-Process LLM Inference Client & KV-Bridge Adapter |
| `contextra-infer-ollama` | Ring 2 | **R** | Ollama External Provider Client & Embeddings Integration |
| `contextra-infer-onnx` | Ring 2 | **E** | ONNX Embeddings & Cross-Encoder Execution Provider |
| `contextra-kvcache` | Ring 1 | **K** | Stufenmodell KV-Cache Storage & Offloading |
| `contextra-license` | Ring 4 | **E** | Lizenz- und Aktivierungsprüfung (*Lizenz-Crate*) |
| `contextra-mcp` | Ring 4 | **R** | Model Context Protocol Server & Cloud Egress Gateway |
| `contextra-mvcc` | Ring 0 | **K** | Multi-Version Concurrency Control & Isolation-Mechanismen |
| `contextra-ports` | Ring 0 | **K** | Abstrakte Port-Schnittstellen & Trait-Definitionen |
| `contextra-privacy` | Ring 3 | **K** | Egress Filtering, Anonymisierung & Privacy Rules |
| `contextra-py` | Ring 4 | **R** | PyO3 Python Bindings & FFI-Grenzschicht |
| `contextra-rank` | Ring 0 | **K*** | Ranking-, Fusion- & Kalibrierungs-Algorithmen (*Kern K; Submodul DRIFT -> E*) |
| `contextra-router` | Ring 3 | **E** | Lyapunov Drift Control & Multi-Candidate Search Routing (*fällt aus Stabilitätsaussagen*) |
| `contextra-sandbox` | Ring 2 | **R** | WASM Execution Boundary & Isolations-Sandbox |
| `contextra-simd` | Ring 0 | **K** | SIMD-beschleunigte Distanzberechnungen & Vektor-Operationen |
| `contextra-store` | Ring 1 | **K** | LSM-Tree Storage Engine, WAL Durability & Compaction |
| `contextra-sys` | Ring 0 | **K** | Systemnahe Bindings, Low-Level I/O & Memory-Mapping |
| `contextra-testkit` | Tooling | **R** | Test Fixtures, Mock Engines & Chaos Matrix Harness |
| `contextra-text` | Ring 0 | **K** | Volltextsuche & Inverted-Index (BM25 Engine) |
| `contextra-types` | Ring 0 | **K** | Grundlegende Typdefinitionen (DocId, TxId, TenantId, Errors) |
| `contextra-vector` | Ring 0 | **K*** | Vektorsuche & Indexierung (*Kern K; experimental-diskann -> E*) |
| `contextra-wire` | Ring 0 | **K** | FlatBuffers Wire-Protokolle & Serialisierungs-Formate |

---

## 3. Experimentelle Komponenten ohne Stabilitätsaussage (Projektbewertung 10.9)

Gemäß Projektbewertung 10.9 und Spezifikation §6.0 fallen die folgenden Komponenten explizit aus den Stabilitäts- und Abwärtskompatibilitätsaussagen des Gesamtsystems:

1. **`contextra-agent`**: Agent Execution Pipeline & InMemoryStorageEngine.
2. **`contextra-router`**: Conformal Calibration & Multi-Candidate Routing Engine.
3. **`contextra-cognition`**: Sleep-Cycle Consolidation & Background Maintenance.
4. **`graph-PPR`** (in `contextra-graph`): Personalized PageRank (PPR) Graph-Expansion & Random-Walk-Schätzungen.
5. **`rank-DRIFT`** (in `contextra-rank` / `contextra-adapt`): Adaptives Drift-Scoring, PID Latency Control & Off-Policy Feedback.
6. **Lizenz-Crates (`contextra-license`)**: Lizenzierungs- & Aktivierungs-Gates.
7. **AVV- & Audit-Crates (`contextra-avv-generator`, `contextra-audit-export`)**: Template-Generatoren für DSGVO Art. 28 / Art. 30 Verarbeitungsverzeichnisse.

---

## 4. Offene Fragen & Identifizierte Abweichungen zu `capabilities.toml`

Bei der Gegenüberstellung der Spezifikation §6.0 mit dem aktuellen Stand von `capabilities.toml` (ADR-N08) wurden folgende Abweichungen identifiziert:

1. **Maturity-Flag-Diskrepanz:**
   - In `capabilities.toml` sind `contextra-agent`, `contextra-router` und `contextra-cognition` aktuell fälschlicherweise als `maturity = 'stable'` eingeordnet.
   - *Offene Frage:* Entsprechend §6.0 müssen diese auf `maturity = 'experimental'` gesetzt werden.
2. **Feingranulare Subkomponenten-Klassifizierung:**
   - `capabilities.toml` führt derzeit Reifegrade nur auf Crate-Ebene. Submodule wie `graph-PPR` (`contextra-graph`) und `rank-DRIFT` (`contextra-rank`) benötigen zukünftig ggf. Feature-Maturity-Marker im TOML-Manifest.

---

## 5. Beschluss & Nächste Schritte

- **Status:** Dieser ADR stellt den abgestimmten Klassifizierungsstand als **Vorschlag** bereit.
- **Keine Änderung an `capabilities.toml` in diesem Schritt:** Die physische Aktualisierung der Manifest-Einträge in `capabilities.toml` erfolgt in einem separaten W0-Prompt.
