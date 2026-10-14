# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 2026-10-14 (Update Governance M1a / TP-22) · Gilt für alle Agenten (Jules, Claude Code, Codex, manuelle Sessions)

## 1. Rangfolge

1. Grüne Gates und Code-Realität
2. `AGENTS.md` (diese Datei)
3. `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` & <!-- doc-ref-ignore --> `docs/SPEZIFIKATION_EP*.md`
4. `docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md` & `docs/14-10-26/specs/*`
5. `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (bzw. `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`)
6. Übrige Vorgaben in `docs/`

---

## 2. Werkzeug-Regel (verbindlich)

**`cargo xtask` ist die einzige Quelle der Wahrheit für Analyse, Prüfung und Diagnose.**

- Keine Python-, Shell- oder Ad-hoc-Skripte zur Repo-Analyse schreiben oder ausführen.
- Kein `grep` als Ersatz für einen vorhandenen Befehl. `grep` ist nur für gezielte Lektüre erlaubt.
- Kein `sh -c`. Prozessaufrufe immer als Argument-Array.
- Fehlt ein Werkzeug, wird es als xtask-Befehl implementiert (siehe Abschnitt 11), nicht als Skript.
- Befehl unbekannt? `cargo xtask harness-list` aufrufen — dies ist die verbindliche Befehlswahrheit.

---

## 3. Session-Lebenszyklus

```bash
cargo xtask jules start --card <datei>    # Arbeitskontext laden; Preflight-Validierung
cargo xtask jules check                   # Gates Phase check (guard + symbols + arch)
cargo xtask jules verify                  # Gates Phase verify (integrity + ratchet + diff-budget)
cargo xtask jules submit                  # Gates Phase submit: Abschluss-Prüfung + Session-Report
cargo xtask jules stop                    # Session beenden
```

Preflight ohne Session-Karte: `cargo xtask jules-preflight`.  
Direkte Gate-Befehle (für manuelle Nutzung): `cargo xtask gate-guard`, `cargo xtask gate-arch`, `cargo xtask gate-integrity`, `cargo xtask session-report`.

---

## 4. Befehlsübersicht

**Status-Legende:** ✅ vorhanden · 🟡 vorhanden, aber nicht vollständig · 🔲 geplant (noch nicht implementiert)

Geplante Befehle sind mit `<!-- harness:planned -->` markiert. Ein Agent darf sie nicht aufrufen und nicht als vorhanden behandeln.  
**Verbindliche Wahrheit**: `cargo xtask harness-list` — immer zuerst prüfen.

### 4.1 Tier 0 — schnell, ohne Domain-Build

| Befehl | Status | Zweck |
|---|---|---|
| `cargo xtask harness-list [--json]` | ✅ | Alle registrierten Harness-Befehle auflisten |
| `cargo xtask harness-help <name>` | ✅ | Hilfe zu einem einzelnen Harness-Befehl |
| `cargo xtask fast-diff-lint` | ✅ | Build-freier Diff-Scope-Linter für häufige Doktrin-Verstöße |
| `cargo xtask repo-doctor` | ✅ | Session-Start-Dashboard: Git, Workspace, Unsafe/Panic, Branch |
| `cargo xtask env-validate` | 🔲 <!-- harness:planned --> | Toolchain- und Umgebungsvalidierung |
| `cargo xtask env-attest` | ✅ | Environment Attestation: Toolchain-Pinning und Build-Tools |
| `cargo xtask workspace-verify` | 🔲 <!-- harness:planned --> | Workspace-Vollständigkeit und Konsistenz |
| `cargo xtask registry-check` | ✅ | Vollständigkeit und Konsistenz aller xtask-Befehle gegen registry.toml |
| `cargo xtask explain <gate>` | ✅ | Qualitätsgates und Invarianten erklären |
| `cargo xtask verify [--tier kernel|full]` | ✅ | Einzelbefehl zur Verifikation und Evidenzerzeugung |
| `cargo xtask check-loom-mocks` | ✅ | Scanner für unzulässige Loom-Mocks und Mirrors |

### 4.2 Tier 1 — Workspace-Scan, ohne Domain-Build

| Befehl | Status | Zweck | Exit |
|---|---|---|---|
| `cargo xtask find-unwired-gaps` | ✅ | Workspace-weites Scanning nach undokumentierten & unaufgerufenen `pub`-Symbolen (CSV-Report in `docs/gaps/`) | 0 |
| `cargo xtask orphan-symbols [--json]` | ✅ | Öffentliche Symbole ohne Verwendung (`ORPHAN`, `TEST_ONLY`, `SUSPECT`) | 2 bei `ORPHAN` |
| `cargo xtask unwired-ports [--only-unwired]` | ✅ | Traits ohne Implementierung (`UNWIRED`, `MOCK_ONLY`, `SOLO`) | 2 bei `UNWIRED` |
| `cargo xtask stub-impls [--min-severity hoch]` | ✅ | Stub-Implementierungen (`HOCH`, `MITTEL`, `NIEDRIG`) | 2 bei `HOCH` |
| `cargo xtask doctrine-scan [--crate <name>] [--json]` | ✅ | Zero-Panic-Verstöße, Unsafe-Insel-Verstöße, `#[allow]`-Durchbrechungen | 2 bei Verstoß |
| `cargo xtask dependency-graph-audit [--json]` | ✅ | DAG-Invariante (P5), Ring-0-`tokio` (P26), Zyklen | 2 bei Verstoß |
| `cargo xtask check-ring-layering` | 🔲 <!-- harness:planned --> | Ring-Layering (Kurzfassung) | 2 bei Verstoß |
| `cargo xtask check-ring-layering-full` | 🔲 <!-- harness:planned --> | Ring-Layering (vollständige Analyse) | 2 bei Verstoß |
| `cargo xtask check-ring0-async-purity` | 🔲 <!-- harness:planned --> | Ring-0-Async-Reinheit (P26) | 2 bei Verstoß |
| `cargo xtask check-unsafe-islands` | 🔲 <!-- harness:planned --> | Unsafe nur in deklarierten Inseln | 2 bei Verstoß |
| `cargo xtask check-ring-capabilities-consistency` | 🔲 <!-- harness:planned --> | `capabilities.toml` als einzige Ring-Quelle (P12) | 2 bei Verstoß |
| `cargo xtask determinism-check` | ✅ | Determinismus via Ports (P28) | 2 bei Verstoß |
| `cargo xtask check-duplicate-symbols` | 🔲 <!-- harness:planned --> | Doppelte Funktions-/Struktursymbole über Dateigrenzen | 2 bei Verstoß |
| `cargo xtask check-result-dropped-io` | 🔲 <!-- harness:planned --> | Ignorierte I/O-Fehler-Results (INV-DURABILITY-RING) | 2 bei Verstoß |
| `cargo xtask check-nan-hot-loop` | 🔲 <!-- harness:planned --> | NaN-Validierung in Vektorberechnungs-Hotpaths | 2 bei Verstoß |
| `cargo xtask symbol-exists <pfad>` | ✅ | Einzelnes Symbol-Pfad-Existenz prüfen | 2 bei fehlendem Symbol |
| `cargo xtask claim` | 🔲 <!-- harness:planned --> | Claim-Integrität für Agent-Gates | 2 bei Verstoß |
| `cargo xtask gc-floor-single-source` | ✅ | Verhindert unerlaubte Aufrufe von `min_active_seqno()` außerhalb von `crates/contextra-mvcc/src/floor.rs` | 2 bei Verstoß |

### 4.3 Tier 2 — Build-Abhängigkeit

| Befehl | Status | Zweck |
|---|---|---|
| `cargo xtask check-compile` | 🔲 <!-- harness:planned --> | Workspace kompiliert fehlerfrei |
| `cargo xtask check-dag` | 🔲 <!-- harness:planned --> | DAG-Prüfung über Cargo-Graph |
| `cargo xtask bench-gate --tolerance 0.05` | 🔲 <!-- harness:planned --> | Benchmark-Gate (`xtask-heavy`) |
| `cargo xtask bench-trend` | 🔲 <!-- harness:planned --> | Benchmark-Trendanalyse |
| `cargo xtask feature-matrix` | 🔲 <!-- harness:planned --> | Feature-Kombinationen prüfen |
| `cargo xtask loom-run` | 🔲 <!-- harness:planned --> | Concurrency-Tests mit loom |
| `cargo xtask panic-inventory` | 🔲 <!-- harness:planned --> | Vollständiges Panic-Inventar |
| `cargo xtask security-scan` | 🔲 <!-- harness:planned --> | Sicherheitsscan (audit, Advisories) |
| `cargo xtask audit-integrity-check` | 🔲 <!-- harness:planned --> | Audit-Ketten-Integrität |
| `cargo xtask check-fbs-drift` | 🔲 <!-- harness:planned --> | FlatBuffers-Schema-Drift-Erkennung |
| `cargo xtask reproducible-build-check` | 🔲 <!-- harness:planned --> | Reproduzierbarkeit des Builds |
| `cargo xtask fuzz-smoke` | ✅ | Fuzz-Smoke über berührte Crates mit Fuzz-Targets |
| `cargo xtask mutants-diff` | ✅ | Diff-scopierter Mutation-Testing-Score |
| `cargo xtask wal-replay-verify` | ✅ | WAL-Replay Crash-Konsistenz-Verifikation (INV-WAL-*) |
| `cargo xtask blast-radius` | ✅ | Auswirkungsbereich geänderter Crates berechnen |
| `cargo xtask risk` | ✅ | Risikobewertung der Änderungen |
| `cargo xtask can-merge` | ✅ | Merge-Entscheidung (aggregiertes Urteil) |

### 4.4 Gates (Jules-Lebenszyklus & CI)

| Gate | Befehl | Bündelt | Phase |
|---|---|---|---|
| `guard` | `cargo xtask gate-guard` | Scope- und Protected-Paths-Prüfung, Diff-Budget | check |
| `symbols` | `cargo xtask symbol-exists` | Existenz aller Symbol-Pfade in der Task-Karte | check |
| `arch` | `cargo xtask gate-arch` | Ring-Layering, Unsafe-Inseln, DAG, Capabilities | check |
| `integrity` | `cargo xtask gate-integrity` | Test-Integrität, Ratchet, Diff-Budget, Gate-Weakening | verify |
| `report` | `cargo xtask session-report` | Session-Report für PR | submit |

### 4.5 Dokumentation und Governance

| Befehl | Status | Zweck |
|---|---|---|
| `cargo xtask sync-docs [--check]` | 🔲 <!-- harness:planned --> | Generierte Doku synchron halten |
| `cargo xtask check-type-registry` | 🔲 <!-- harness:planned --> | `docs/TYPE_REGISTRY.md` vollständig |
| `cargo xtask check-vetoes` | 🔲 <!-- harness:planned --> | Veto-Sperren |
| `cargo xtask check-adr-deadlines` | 🔲 <!-- harness:planned --> | ADR-Fristen |
| `cargo xtask check-veto-deadlines` | 🔲 <!-- harness:planned --> | Veto-Fristen |
| `cargo xtask check-agents-integrity` | 🔲 <!-- harness:planned --> | Diese Datei ist konsistent |
| `cargo xtask check-agents-freshness` | 🔲 <!-- harness:planned --> | Diese Datei ist aktuell |
| `cargo xtask check-doc-references` | 🔲 <!-- harness:planned --> | Querverweise in Doku gültig |
| `cargo xtask check-workflow-commands` | 🔲 <!-- harness:planned --> | Workflow-Befehle existieren |
| `cargo xtask check-stale-tags` | 🔲 <!-- harness:planned --> | Veraltete Tags erkennen |
| `cargo xtask check-toc-integrity` | 🔲 <!-- harness:planned --> | Inhaltsverzeichnis-Integrität |
| `cargo xtask consolidate-adrs` | 🔲 <!-- harness:planned --> | ADRs zusammenführen |
| `cargo xtask gen-arch-docs` | 🔲 <!-- harness:planned --> | Architektur-Dokumentation generieren |
| `cargo xtask context-pack` | 🔲 <!-- harness:planned --> | Agent-Kontext-Paket erstellen |
| `cargo xtask crate-context <crate>` | 🔲 <!-- harness:planned --> | Crate-Kontext-Report |
| `cargo xtask gates-catalog-sync` | ✅ | Konsistenz zwischen gates.toml und harness-Phasen |

---

## 5. Crate- und Komponenten-spezifische Schnell-Diagnose (xtask Cheat-Sheet)

Agenten müssen **vor und nach jeder Modifikation** zielgerichtet die relevanten xtask-Befehle für die jeweilige Crate- und Produktkomponente ausführen:

### 5.1 Übersicht nach Endprodukten (EP1 bis EP6)

| Komponente | Zuständige Crates | Relevante Schnell-Diagnose-Befehle |
|---|---|---|
| **EP1: Memory (Core)** | `contextra-store`<br>`contextra-engine`<br>`contextra-vector`<br>`contextra-text`<br>`contextra-mvcc`<br>`contextra-types` | `cargo xtask crate-context contextra-store`<br>`cargo xtask doctrine-scan --crate contextra-store`<br>`cargo xtask wal-replay-verify`<br>`cargo xtask check-unsafe-islands`<br>`cargo xtask check-nan-hot-loop`<br>`cargo test -p contextra-store --test purge_v2_resilience` |
| **EP2: Proof & Krypto**| `contextra-crypto`<br>`contextra-erasure-evidence`<br>`contextra-proof-verify`<br>`contextra-audit-export`<br>`contextra-avv-generator` | `cargo xtask crate-context contextra-crypto`<br>`cargo xtask doctrine-scan --crate contextra-crypto`<br>`cargo xtask determinism-check`<br>`cargo test -p contextra-crypto --test deletion_proof_v4_tamper_matrix`<br>`cargo test -p contextra-proof-verify` |
| **EP3: Retrieval+** | `contextra-graph`<br>`contextra-rank` | `cargo xtask crate-context contextra-graph`<br>`cargo xtask crate-context contextra-rank`<br>`cargo xtask unwired-ports`<br>`cargo xtask check-nan-hot-loop`<br>`cargo test -p contextra-graph --test path_rag_k_path_budget` |
| **EP4: Inference** | `contextra-kvcache`<br>`contextra-embedder`<br>`contextra-checkpoint` | `cargo xtask crate-context contextra-kvcache`<br>`cargo xtask check-unsafe-islands`<br>`cargo xtask check-loom-mocks`<br>`cargo test -p contextra-kvcache --test kivi_quantization_accuracy` |
| **EP5: Cognition** | `contextra-cognition`<br>`contextra-adapt`<br>`contextra-agent` | `cargo xtask crate-context contextra-adapt`<br>`cargo xtask determinism-check`<br>`cargo test -p contextra-adapt --test sketched_linucb_regret` |
| **EP6: Guard & Safe** | `contextra-privacy`<br>`contextra-sandbox`<br>`contextra-mcp`<br>`contextra-py` | `cargo xtask crate-context contextra-privacy`<br>`cargo xtask crate-context contextra-mcp`<br>`cargo test -p contextra-mcp --test egress_guard_fail_closed` |

### 5.2 Schnell-Befehle für jedes einzelne Crate

| Crate | Ring | Primärer Diagnose-Befehl | Wichtige Prüf-Invarianten |
|---|---|---|---|
| `contextra-types` | 0 | `cargo test -p contextra-types` | `INV-TENANT-1` (`TenantId::try_new(0) -> Err`), Newtype-Validierung |
| `contextra-errors` | 0 | `cargo check -p contextra-errors` | Zero-Panic, einheitliche Fehler-Codes |
| `contextra-ports` | 1 | `cargo xtask check-ring0-async-purity` | Keine Tokio-Abhängigkeit, Trait-Object-Safety (`dyn`) |
| `contextra-store` | 2 | `cargo xtask wal-replay-verify`<br>`cargo xtask check-unsafe-islands` | ADR-012 mmap-Insel, `INV-WAL-TRUNCATION-1`, L1–L6 Purge v2 |
| `contextra-vector` | 2 | `cargo xtask check-unsafe-islands`<br>`cargo xtask check-nan-hot-loop` | ADR-007 SIMD-Insel, `VETO-F02` (kein HNSW-Rewire bei Purge) |
| `contextra-text` | 2 | `cargo test -p contextra-text` | Snapshot-Isolation, BM25 WAND, keine unbegrenzten Abfragen |
| `contextra-mvcc` | 2 | `cargo xtask gc-floor-single-source`<br>`cargo test -p contextra-mvcc` | `INV-MVCC-SSI-1` (Read-Set-Tracking), monotone TxId |
| `contextra-crypto` | 2 | `cargo xtask determinism-check`<br>`cargo xtask doctrine-scan --crate contextra-crypto` | BSI TR-02102, ZeroizeOnDrop, CSPRNG-Disziplin (P28) |
| `contextra-graph` | 2 | `cargo test -p contextra-graph`<br>`cargo xtask unwired-ports` | `AGT-GRAPH-001`, CSR-Thread-Safety, PathRAG-Kapselung |
| `contextra-rank` | 2 | `cargo xtask check-nan-hot-loop`<br>`cargo test -p contextra-rank` | Conformal Calibration, `f64::total_cmp` Sortier-Disziplin |
| `contextra-kvcache` | 2 | `cargo xtask check-unsafe-islands`<br>`cargo xtask check-loom-mocks` | ADR-019 Aligned-Alloc, KIVI 2-Bit Quantisierung, SnapKV |
| `contextra-checkpoint` | 2 | `cargo test -p contextra-checkpoint` | `INV-CHECKPOINT-DETERMINISM-1`, SSTable Hardlink-Cloner |
| `contextra-adapt` | 2 | `cargo xtask determinism-check`<br>`cargo test -p contextra-adapt` | P28-Determinismus via `Clock`-Port, Sketched LinUCB |
| `contextra-cognition`| 2 | `cargo test -p contextra-cognition` | Brand's Rang-1 SVD, Hyperedge Spectral Deduplication |
| `contextra-privacy` | 2 | `cargo test -p contextra-privacy` | `INV-EGRESS-AUDIT-1` (Hash-Kette), Luhn/IBAN DLP Fail-Closed |
| `contextra-sandbox` | 3 | `cargo test -p contextra-sandbox` | WASM-Fuel-Limit (10 Mio. Ticks), Timeout (5s), `ZeroizeOnDrop` |
| `contextra-embedder`| 3 | `cargo test -p contextra-embedder` | Isolation von `MockEmbedder`, Ollama/Candle Port-Implementierung |
| `contextra-mcp` | 3 | `cargo test -p contextra-mcp --test egress_guard_fail_closed` | `INV-MCP-CLASSIFY-1`, Behebung B1 (Cloud Egress) & B9 (Delegation) |
| `contextra-engine` | 4 | `cargo xtask check-compile`<br>`cargo test -p contextra-engine` | Behebung N-B (`ErasureBackend`-Trait), N-K (`PhysicalScope`) |
| `contextra` | 4 | `cargo xtask check-ring-capabilities-consistency`<br>`cargo test -p contextra` | Composition Root, `verify_wiring()`-Pflichtprüfung (S5) |

---

## 6. Roadmap der geplanten Architektur-Änderungen (`docs/14-10-26/`)

Die Evolution des Gesamtsystems erfolgt streng phasenbasiert gemäß [`docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md`](<!-- doc-ref-ignore -->) und den disjunktiven Spezifikationen in [`docs/14-10-26/`](file:///home/freddy/Projekte/contextra/docs/14-10-26/):

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        ARCHITEKTUR-ROADMAP (MASTERPLAN v2)                             │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ PHASE 0: Blocker-Beseitigung & Stabilisierung (Sofort / Vorbereitung EP1)             │
│ ├── 0a: Build-Rettung: Engine Match-Arm (N-N), Fassade Clock (N-O), Lockfile (N-P)    │
│ ├── 0b: Standard-Delete Rettung: Downcast-Eliminierung via ErasureBackend (N-B)        │
│ ├── 0c: Lockless Schreib-Fence: Option B mit atomarem FenceEpoch-Check                 │
│ ├── 0d: Purge v2 Streaming-Rewrite: PhysicalScope (N-K), Intent-Resume (N-A), L1–L6   │
│ └── 0e: Zeugen-Bereinigung & v4-Konsolidierung: Echte Scans (N-D), v3-Entfernung (N-I) │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ PHASE 1: Zero-Cost-Entkopplung & Core-Bereinigung (EP1 Release 1.0)                   │
│ ├── 1a: Builder-Validierung: verify_wiring() Pflicht (S5), KmsProvider-Verdrahtung    │
│ ├── 1b: contextra-core Strangulation & rückstandsfreie Eliminierung (110 Altdateien)  │
│ └── 1c: Durchsetzung der DurabilityMode × ErasureMode Kompatibilitätsmatrix            │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ PHASE 2: Ring-Architektur & Port-Abstraktion                                           │
│ ├── Beseitigung der 6 verbotenen Ring-Kanten (Sandbox & Candle hinter Ports)          │
│ └── contextra-durable-fs formell in Ring-0-Ordnung eingliedern                         │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ PHASE 3: SOTA Algorithmen, Graph & KV-Cache (EP3 & EP4)                               │
│ ├── Irminsul KV-Cache Management (Prefix-Reuse + SnapKV + KIVI 2-Bit)                  │
│ ├── PathRAG Dijkstra-Exposition & KPathDiffusion pub(crate)-Kapselung                  │
│ └── Conformal Fusion Calibration in contextra-rank (Ersatz von PAVA/Platt)             │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ PHASE 4: Adaptive Kognition & Hypergraph-Reasoning (EP5)                               │
│ ├── Sketched LinUCB Bandits mit Achlioptas-Projektion (O(k²) Sherman-Morrison)         │
│ ├── Brand's Rang-1 SVD für Hyperkanten-Deduplizierung mit Kahan-Summation             │
│ └── Durchgängiger P28-Determinismus im PID Latency Controller via Clock-Port          │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ PHASE 5: Löschnachweis-Plugin & BSI TR-02102 (EP2)                                     │
│ ├── Crate-Split in contextra-crypto-core, contextra-erasure, evidence & verify         │
│ ├── Behebung B4: DeletionProofV4 an MCP-Client durchreichen statt proof: null          │
│ └── Autonomer Standalone-Verifier (< 500 µs, 0 DB-State, 0 I/O)                       │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ PHASE 6: Egress-Firewall & Sicherheitsgrenzen (EP6)                                    │
│ ├── Behebung B1: Fail-Closed EgressGuard Pflichtprüfung vor contextra_cloud_query      │
│ ├── Behebung B9: Eliminierung des Dual-Paths im MCP (strikte ContextraDb-Delegation)   │
│ └── BLAKE3 Hash-Chained Audit-Trail (INV-EGRESS-AUDIT-1)                               │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Vor jeder Änderung

1. Task-Karte und zugehörige Komponentenspezifikation ([<!-- doc-ref-ignore --> `docs/SPEZIFIKATION_EP*.md`](file:///home/freddy/Projekte/contextra/docs/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md)) lesen.
2. Existenz prüfen: `capabilities.toml`, `docs/TYPE_REGISTRY.md`, `cargo xtask symbol-exists <pfad>`.
3. Verfügbare Befehle prüfen: `cargo xtask harness-list` — **niemals** Ad-hoc-Befehle nutzen.
4. Plan mit Dateiliste und Akzeptanzbefehlen erstellen. Akzeptanzbefehle **nur** aus Abschnitt 4 und 5 mit Status ✅ oder 🟡.
5. Kleinste mögliche Änderung. Vor und nach der Änderung mit xtask verifizieren.

---

## 8. Scope

- Nur Dateien im `scope` der Task-Karte ändern.
- Befunde außerhalb des Scopes nicht direkt beheben. Im PR unter „Out-of-scope Findings" dokumentieren.

---

## 9. Invarianten (verbindlich)

| ID | Regel | Prüfbefehl |
|---|---|---|
| P2 | `unwrap_used`, `expect_used`, `panic` sind `deny` in Produktionscode | `cargo clippy --all-targets` · `cargo xtask doctrine-scan` |
| P5 | DAG: Abhängigkeiten nur zu niedrigeren Ringen | `cargo xtask dependency-graph-audit` |
| P12 | `capabilities.toml` ist einzige Ring-Quelle | `cargo xtask check-ring-capabilities-consistency` |
| P26 | Ring 0 bindet `tokio` nicht als Produktions-Abhängigkeit ein | `cargo xtask check-ring0-async-purity` |
| P28 | Determinismus via Ports (`Clock`, `Rng`, `IdGenerator`); CSPRNG für Key- und Salt-Material | `cargo xtask determinism-check` |
| INV-TENANT-1 | `TenantId::try_new(0)` → Err; `0` ist SYSTEM | `cargo test -p contextra-types` |
| INV-DELETION-1/2 | `DeletionProof::create()` nur nach physischer Bereinigung; kein Zeiger auf gelöschte IDs | `cargo test -p contextra-crypto` |
| INV-DURABILITY-RING | `MemoryOnly` inkompatibel mit DeletionProof und Sovereign | `cargo test -p contextra` |
| INV-PERF-PROFILE-1 | `BareMetal` und `Balanced` inkompatibel mit Löschbeweis | `cargo test -p contextra` |
| INV-COLLECTION-PROFILE-1/2 | Aktiver Proof erfordert `CryptoShred`; Presets konsistent | `cargo test -p contextra` |
| INV-WAL-LEGACY-KEY-1 | Legacy-HMAC-Fallback nie implizit aktiv | `cargo test -p contextra-store` |
| INV-WAL-TRUNCATION-1 | WAL-Truncation-Schutz über Manifest-High-Water-Mark | `cargo test -p contextra-store` |
| INV-MVCC-SSI-1 | SSI Read-Set-Tracking und Write-Skew-Validierung aktiv | `cargo test -p contextra-mvcc` |
| INV-EGRESS-AUDIT-1 | Egress-Audit-Trace hash-verkettet, ohne Plaintext | `cargo test -p contextra-privacy` |
| INV-MCP-CLASSIFY-1 | Jedes gelistete MCP-Tool explizit klassifiziert | `cargo test -p contextra-mcp` |
| INV-CHECKPOINT-DETERMINISM-1 | Checkpointing ohne direkte Wall-Clock- oder Random-Aufrufe | `cargo test -p contextra-checkpoint` |
| AGT-GRAPH-001 | CSR-Graph-Aktionen deterministisch und thread-safe | `cargo test -p contextra-graph` |
| INV-VAULT-1/2/3 | Zeroize, Surrogate-Kollisionsschutz, Key-Isolation | `cargo test -p contextra-crypto` |
| VETO-F02/F10/OP03 | HNSW-Pruning ohne Rewire; KV-Scratchpad-Invalidierung nur unter Prefix; Memory-Limits | `cargo test -p contextra-vector` |
| Snapshot-Isolation | Alle Signale einer Anfrage lesen denselben Sequenzstand (`max_seq`) | `cargo test -p contextra-text` |
| Fail-Closed | Fail-closed für SSI, Sandbox, Egress-Timeout und Egress-Indexausfall | `cargo test -p contextra-mcp` |

### 9.1 Vorgeschlagene Invarianten (Entwurf / Vorschläge — KEIN aktives Gate)

> **Hinweis**: Die folgenden Invarianten stellen Entwürfe/Vorschläge dar (ADR-T01 bis ADR-T06) und sind **noch keine** verbindlich geschalteten Qualitätsgates in der CI.

| ID | Vorschlag / Regel | Zugehöriger ADR-T Entwurf | Status |
|---|---|---|---|
| P29 | Kein globaler veränderlicher Zustand (`static mut`, globale mutable Einzellokationen verboten) | [ADR-T01](docs/decisions/ADR-T01-P29-kein-globaler-veraenderlicher-zustand.md) | Entwurf (Kein Gate) |
| INV-KMS-1 | Strikte KMS-Provider-Verdrahtung in Crypto- und Schlüssel-Modulen | [ADR-T02](docs/decisions/ADR-T02-INV-KMS-1-kms-provider-verdrahtung.md) | Entwurf (Kein Gate) |
| INV-DELETION-WIRE-1 | Durchgängige Löschpfad-Verdrahtung über alle Speicher-Layer | [ADR-T03](docs/decisions/ADR-T03-INV-DELETION-WIRE-1-loeschpfad-verdrahtung.md) | Entwurf (Kein Gate) |
| INV-SSI-ACTIVE-1 | Strikte Durchsetzung von Serializable Snapshot Isolation (SSI) | [ADR-T04](docs/decisions/ADR-T04-INV-SSI-ACTIVE-1-ssi-durchsetzung.md) | Entwurf (Kein Gate) |
| INV-DOC-SINGLE-SOURCE-1 | Dokumentationsreferenzen strictly gegen Primärquellen | [ADR-T05](docs/decisions/ADR-T05-INV-DOC-SINGLE-SOURCE-1-doc-single-source.md) | Entwurf (Kein Gate) |
| INV-RING0-DETERMINISM-1 | Ring-0 Determinismus ohne ungekapselte Zeit/RNG-Aufrufe | [ADR-T06](docs/decisions/ADR-T06-INV-RING0-DETERMINISM-1-ring0-determinismus.md) | Entwurf (Kein Gate) |

---

## 10. Architektur-Karte

- `capabilities.toml`: verbindliches Crate- und Ring-Verzeichnis, einzige Quelle für Ringe, Composition-Root-Menge und Unsafe-Inseln.
- `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`: Gesamtsynthese der disjunktiven Produktkomponenten (EP1–EP6).
- <!-- doc-ref-ignore --> `docs/SPEZIFIKATION_EP*.md`: Einzel-Feinspezifikationen pro Komponente.
- `docs/TYPE_REGISTRY.md`: alle Domänen-Typen.
- `docs/decisions/README.md`: ADRs und Veto-Sperren.
- `xtask/registry.toml`: alle Harness-Befehle mit `owner_tier`, `phase` und `summary` — **verbindliche Befehlswahrheit**.
- `crates/<name>/AGENTS.md`: crate-spezifische Betriebsanleitungen und Invarianten. <!-- doc-ref-ignore -->

---

## 11. Werkzeuge erweitern

Ein neues Werkzeug entsteht als xtask-Befehl. Kein Skript.

1. Modul in `xtask/src/harness/<name>.rs` <!-- doc-ref-ignore --> mit `//!`-Docstring (Zweck, Tier, Exit-Codes).
2. Eintrag in `xtask/registry.toml` mit `owner_tier`, `summary`, `phase`.
3. Tests in `xtask/tests/<name>.rs` <!-- doc-ref-ignore --> mit Fixtures für Befund und Nicht-Befund.
4. Befund-Kategorien: Muss-Fehler (Exit 2) und Hinweis (Exit 0).
5. `--json` unterstützen.
6. Befehl in Abschnitt 4 dieser Datei eintragen — zuerst als `🔲 <!-- harness:planned -->`, nach Merge und Eintrag in `registry.toml` als `✅`.

Verbindliche Regeln für Werkzeuge:
- Tier 0 und Tier 1 dürfen keine Domain-Crate als Cargo-Dependency haben.
- Rust-Analyse nutzt `syn`, nicht Regex, sofern nicht explizit begründet.
- Test-Code wird strukturell ausgeschlossen (über den WorkspaceIndex), nicht über Textschnitt.
- Kein `sh -c`. Prozessaufrufe als Argument-Array (via `shlex`).

---

## 12. Nicht tun

- Kein Python-, Shell- oder Ad-hoc-Analyseskript im Repository.
- Keine Gate-, Lint-, Schwellen-, Baseline- oder Toolchain-Pin-Abschwächung.
- Keine neuen `#[allow(...)]` für Sicherheits-Lints ohne expliziten ADR.
- Keine Verwendung von `sh -c`.
- Keine Single-Node-Verletzung: keine Multi-Node-, Sharding-, P2P-Sync- oder Cluster-Logik.
- Sperrenhierarchie einhalten: `collections` → `kv_locks` → `embedder`. Keine Locks über `.await`.
- Jede API-Änderung in `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` spiegeln.
- Scope-Ausschlüsse: kein globales HNSW-Rebuilding, keine Cross-Tenant-Flüsse, keine Realtime-Audio- oder Voice-Features, keine Veto-Bypasses ohne ADR.
- Keine Analyse-Befehle erfinden. Nur Befehle aus Abschnitt 4 und 5 ausführen — und vorher mit `cargo xtask harness-list` bestätigen.
