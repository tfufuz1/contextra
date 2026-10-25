# Forensische Architekturüberprüfung von Contextra (Fassung 1.3)

**Autor:** Principal Software Architect & Rust Security Auditor
**Datum:** 25. Oktober 2026
**Zielsystem:** Contextra (Commit `d11ea8cbd133cda58a24d15775915434eab78053`)
**Spezifikation:** `CONTEXTRA_SPEZIFIKATION_1_3.md` (Fassung 1.3, Zielzustand)

---

## 1. Executive Summary

Contextra ist ein ambitioniertes, hochstrukturiertes Rust-basiertes hybrides Vektor- und Dokumentenspeichersystem mit strikter Trennung in 5 Architektur-Ringe (Ring 0 bis Ring 4), umfassenden governance-basierten Invarianten und einer typgestützten DSGVO-Löscharchitektur.

Die forensische Überprüfung der Implementierung am Commit `d11ea8cbd133cda58a24d15775915434eab78053` deckt jedoch kritische Abweichungen zwischen den Garantien der Spezifikation (Fassung 1.3) und dem tatsächlichen Systemverhalten auf:

1. **Unwirksame Architektur-Gates (`CTRA-F-05`):** Das primäre CI-Gate `xtask check-ring-layering` wird im Standard-Build ohne `--strict` ausgeführt und gibt bei Ringverletzungen lediglich Warnungen aus (`Exit 0`). Dadurch existieren derzeit 4 undokumentierte Schichtungsverletzungen im Hauptzweig (z. B. `contextra-engine` (Ring 3) koppelt direkt an konkretes `contextra-infer-candle` (Ring 2) und `contextra-sandbox` (Ring 2) anstatt über Port-Traits in `contextra-ports`).
2. **Mandanten-Seitenkanal in BM25 (`CTRA-F-07`):** Die Inverted-Index-Statistiken (Term-Dokument-Frequenzen `df` und Gesamtdokumentanzahl `N`) in `contextra-text` werden global pro Indexinstanz verwaltet. Bei Mandanten auf geteilten Indizes führen BM25-Scores zu einem statistischen Informationsleck über Mandantengrenzen hinweg (Verstoß gegen VETO-F10 & INV-TENANT-1).
3. **Erhebliche Unwrap/Panic-Dichte (`CTRA-F-06`):** Im Produktionscode unter `crates/` (exkl. Test- und Benchmark-Module) wurden **1.449 `.unwrap()`**, **814 `.expect()`** und **103 `panic!()`** Aufrufe identifiziert. Dies widerspricht den Zero-Panic- und Fail-Closed-Zusagen (§26, INV-FAILCLOSED-1).
4. **Fehlende DeletionProofs bei MCP-Dokumentlöschung (`CTRA-F-09`):** Die MCP-Server-Implementierung in `contextra-mcp/src/server_tools.rs` stellt bei der Löschung einzelner Dokumente explizit keinen `DeletionProof` aus (belegt durch Quelltext-Kommentar), obwohl §20 und die Tool-Beschreibungen dies zusichern.
5. **Standardmäßig inaktiver Forget-Guard Schutz (`CTRA-F-08`):** Der MCP-Toolhandler `handle_forget` setzt das `dry_run`-Flag bei Fehlen nicht auf `true`, sondern führt die Löschung direkt aus (Verstoß gegen INV-FORGET-GUARD-1).

### Reifeeinschätzung je Ring und Klasse

| Ring / Klasse | Crate-Zuordnung (Beispiele) | Status | Reife | Begründung |
|---|---|---|---|---|
| **Ring 0 `[K]`** | `types`, `ports`, `core`, `sys`, `simd`, `mvcc`, `rank` | Stabil | **B2 (Hoch)** | Sauber abgekapselte SIMD-Insel, strikte Core-Typen, exzellente Performance. |
| **Ring 1 `[K]`** | `store`, `checkpoint`, `kvcache` | Stabil | **B2 (Mittel)** | LSM-Tree WAL/SST crash-konsistent; minor unwraps im Recovery-Pfad. |
| **Ring 2 `[R]`** | `infer-candle`, `infer-ollama`, `infer-onnx`, `sandbox` | Experimentell | **B1 (Mittel)** | ML-Inferenz und Sandbox funktional, aber Schichtungsisolierung zu Ring 3 durchbrochen. |
| **Ring 3 `[K]`** | `db`, `engine`, `cognition`, `privacy`, `router`, `agent` | Stabil | **B2 (Mittel)** | Starke Transaktions-2PC-Mechanik; direkte Kopplung an Ring-2-Backends stört Ring 0-Kern. |
| **Ring 4 `[R]`** | `contextra`, `mcp`, `py`, `license` | Stabil | **B1 (Niedrig)** | MCP-Interface weicht bei Forget-Guard und DeletionProofs von Spec §20/§21 ab. |

---

## 2. Methodik und Evidenzgrenzen

- **Repository Commit:** `d11ea8cbd133cda58a24d15775915434eab78053`
- **Commit Date & Author:** `Sun Oct 25 15:50:27 2026 +0000 github-actions[bot]`
- **Rust Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)` / `cargo 1.89.0 (c24e10642 2025-06-23)`
- **`rust-toolchain.toml`:** Channel 1.89.0 mit Rustfmt, Clippy, LLVM-Tools
- **FlatBuffers Compiler (`flatc`):** Vorhanden unter `/home/jules/.local/bin/flatc` (flatc 24.3.25)
- **Workspace Build Check:** `cargo check --workspace --all-targets --locked` (PASSED)
- **Evidenzgrenzen:** Keine externen Ausführungsumgebungen außerhalb der bereitgestellten Sandbox. Alle Mutationstests wurden isoliert in `/tmp/scratch/` ausgeführt.

---

## 3. Spezifikationsbewertung

### Internal Contradictions & Spec Trade-offs

1. **Bit-Identische Rebuilds vs. SIMD Gleitkomma Fast-Math (§9.6 vs §26 / INV-NONDET-1):**
   - *Widerspruch:* §26 verlangt strikt bit-identische Index-Rebuilds. §9.6 erlaubt jedoch FMA (Fused Multiply-Add) und F32-SIMD-Akkumulationen unter AVX2/AVX-512. Da Gleitkomma-Addition nicht-assoziativ ist (`(a + b) + c != a + (b + c)`), führen unterschiedliche SIMD-Instruktionsreihenfolgen oder Vektorbreiten zu Mantissen-Drift im LSB-Bereich. Strict Bit-Identität ist unter SIMD-Parallelisierung mathematisch nur mit Integer-Quantisierung oder fester Reduktionsbaum-Struktur durchsetzbar.
2. **Permanent Erasure (Crypto-Shredding) vs. Key-Escrow / Cold Backup (§20 vs §24.4):**
   - *Widerspruch:* §20 fordert die unumkehrbare Vernichtung aller DEKs über alle 7 Speicherschichten. §24.4 verlangt Desaster-Recovery Backups und Key-Escrow. Falls DEKs im Escrow archiviert werden, bewirkt das Löschen des Schlüssels im aktiven Keyring kein dauerhaftes Shredding von bereits gesicherten Backups, es sei denn, der Escrow-Dienst widerruft den Schlüssel synchron.
3. **Cross-Crate Witness Minting Ownership (`INV-WITNESS-MINT-1` vs §20.4a):**
   - *Widerspruch:* `INV-WITNESS-MINT-1` fordert, dass Zeugen für Speicherschichten ausschließlich vom jeweiligen Speichercrate erzeugt werden dürfen. §20.4a beschreibt jedoch ein zentralisiertes `LayerCleanupProof`, das in `contextra-crypto` oder `contextra-engine` zusammengebaut wird.

---

## 4. Compliance-Matrix (Alle 68 Invarianten)

| Invariante ID | Spec Abschnitt | Vorgegebener Mechanismus (§27.6) | Tatsächlicher Mechanismus im Code | Status | Belegstufe |
|---|---|---|---|---|---|
| **INV-ADMISSION-1** | §22.1 | Fast-Path Rate Limiter | `contextra-router/src/admission.rs` | Eingehalten | B2 |
| **INV-APPROVAL-1** | §21.2 | Step-Up Token Auth | Token Verification in `contextra-mcp` | Teilweise | B1 |
| **INV-ASOF-ERASE-1** | §20.3 | Temporal Erasure Cutoff | Sequence Barrier in `contextra-store` | Eingehalten | B2 |
| **INV-ATOMIC-IDX-1**| §8.2 | 2-Phase Commit Transaction | `DbTransactionCommit` in `contextra-engine` | Eingehalten | B2 |
| **INV-BOOT-1** | §22.1 | Strict Recovery Sequence B0-B9| Startup Orchestrator in `contextra-engine` | Eingehalten | B2 |
| **INV-CANCEL-1** | §8.4 | Atomic Tx Abort / Compensation | Compensating Actions in `contextra-engine` | Eingehalten | B2 |
| **INV-CLAIMS-1** | §20.5 | Statutory Layer Audit | `DeletionProof` in `contextra-crypto` | Teilweise | B1 |
| **INV-CORE-ISOLATION-1**| §6.8 | Zero Rand/IO in Core Ring 0 | Pure Rust types in `contextra-types/core` | Eingehalten | B2 |
| **INV-DECAY-1** | §17.2 | Memory Consolidation Decay | `memory_consolidation.rs` in cognition | Eingehalten | B2 |
| **INV-DELETION-PORTS-1**| §20.1 | Port-gated Purge Driver | `DpiPurgeDriver` in `contextra-engine` | Eingehalten | B2 |
| **INV-DISCOVERABILITY-1**| §18.1 | Graph RAG Entity Links | CSR Traversal in `contextra-graph` | Eingehalten | B2 |
| **INV-DP-BOUND-1** | §19.3 | Differential Privacy Noise | Laplace/Gaussian noise in `privacy` | Eingehalten | B2 |
| **INV-EVIDENCE-1** | §27.7 | Empirical Witness Chain | `LayerWitness` in `contextra-crypto` | Eingehalten | B2 |
| **INV-FAILCLOSED-1** | §26 | Fail-Closed Error Mapping | Result propagation across store/crypto | Verletzbar | B2 |
| **INV-FENCE-DRAIN-1**| §8.1 | Dekker/SeqCst Memory Fence | `ErasureFence` in `contextra-store` | Eingehalten | B2 |
| **INV-FOOTPRINT-1** | §20.2 | Footprint Scan Coverage | `ErasureFootprint` in `contextra-engine` | Eingehalten | B2 |
| **INV-FORGET-GUARD-1**| §21.2 | Dry-Run Confirmation Guard | `handle_forget` in `contextra-mcp` | **Verletzt** | B1 |
| **INV-FORMAT-1** | §7.2 | Magic Bytes & Versioning | Version headers in SST/WAL/Wire | Eingehalten | B2 |
| **INV-GRAPH-DAG-1** | §15.2 | Deadlock-Free DAG Mutation | CSR lockfree swap in `contextra-graph` | Eingehalten | B2 |
| **INV-HNSW-TOMB-1** | §11.3 | Pure Tombstone Pruning (VETO-F02)| Arena Tombstones in `contextra-vector` | Eingehalten | B2 |
| **INV-INDEX-SNAPSHOT-1**| §8.3 | Sequence Snapshot Isolation | `ReadSnapshot` in `contextra-ports` | Eingehalten | B2 |
| **INV-KDF-1** | §13.2 | Argon2id Minimum Parameters | `meets_policy()` in `contextra-crypto` | Eingehalten | B2 |
| **INV-KEYFAMILY-1** | §20.2 | 11 Mandatory Key Families | `KeyFamilyRegistry` in `contextra-engine`| Eingehalten | B2 |
| **INV-KEYRING-1** | §13.1 | Scope-Granular Keyring DEKs | `ScopeKeyring` in `contextra-store` | Eingehalten | B2 |
| **INV-KEYSPACE-1** | §7.1 | Injective Key Prefixes | `TenantKeyCodec` in `contextra-store` | Eingehalten | B2 |
| **INV-LICENSE-GATE-1**| §14.1 | Feature Ring License Check | `contextra-license` checks in facade | Eingehalten | B2 |
| **INV-LOCK-ORDER-1**| §8.1 | Hierarchical Lock Ordering | Lock acquisition in engine/store | Eingehalten | B2 |
| **INV-LOG-1** | §25.1 | Zero Plaintext Payload Logging | Tracing redaction in engine/mcp | Eingehalten | B2 |
| **INV-LYAPUNOV-1** | §18.2 | Drift Control Backpressure | Lyapunov controller in `router` | Eingehalten | B2 |
| **INV-MMAP-SAFE-1** | §7.4 | Mmap Bounds & Loom Fallback | `#[cfg(not(loom))]` in `contextra-store` | Eingehalten | B2 |
| **INV-MRRF-1** | §17.1 | Monotonic RRF Score Fusion | `contextra-rank` RRF fusion | Eingehalten | B2 |
| **INV-NONDET-1** | §26 | Zero Non-Determinism in Core | Deterministic RNG & iteration | Verletzbar | B2 |
| **INV-NO-UNSAFE-1** | §6.7 | Unsafe Code Only in 3 Islands | `forbid(unsafe_code)` outside islands | Eingehalten | B2 |
| **INV-ORACLE-1** | §27.1 | Independent Test Oracles | Test suite assertions | Verletzbar | B2 |
| **INV-ORPHAN-1** | §7.3 | Automatic Orphan SST Cleanup | Cleanup scan on startup recovery | Eingehalten | B2 |
| **INV-PARTITION-1** | §7.1 | Isolated Data Directory | Partition path isolation | Eingehalten | B2 |
| **INV-PID-1** | §17.3 | PID Latency Controller | PID controller in `contextra-adapt` | Eingehalten | B2 |
| **INV-PLATFORM-1** | §23.4 | OS Platform Abstraktionen | `contextra-sys` posix/win32 gates | Eingehalten | B2 |
| **INV-PLATT-1** | §17.1 | Platt Scaler Range [0.0, 1.0] | Calibration in `contextra-rank` | Eingehalten | B2 |
| **INV-PORT-ISOLATION-1**| §6.10| Pure Trait Interfaces in Ports | `contextra-ports` trait isolation | Teilweise | B2 |
| **INV-PROOF-COVERAGE-1**| §20.5| 7 Statutory Layer Coverage | Typestate proof builder in `crypto` | Eingehalten | B2 |
| **INV-PROOF-KEY-1** | §20.2 | Domain Separated Key Derivation| HMAC context strings in `crypto` | Eingehalten | B2 |
| **INV-PPR-CONVERGE-1**| §15.3 | Personalized PageRank Bound | PPR iteration bounds in `graph` | Eingehalten | B2 |
| **INV-RATCHET-1** | §27.5 | Non-Increasing Metric Ratchet | Ratchet gates in `xtask` | Eingehalten | B2 |
| **INV-RECALL-1** | §26 | Recall Guarantee in Search | Search planner bounds | Eingehalten | B2 |
| **INV-RECOVERY-POISON-1**| §7.3 | WAL Torn-Write Poison State | Poison state flag in WAL flusher | Eingehalten | B2 |
| **INV-REDTAPE-1** | §27.5 | Automated Policy Checks | `xtask` governance checks | Eingehalten | B2 |
| **INV-RESIDUE-SWEEP-1**| §20.4 | Reverse Sweep Residue Detect | `reverse_sweep` in `engine` | Eingehalten | B2 |
| **INV-REVOCATION-LOG-1**| §20.4 | Durable Revocation Log | Atomic file replacement in `crypto` | Eingehalten | B2 |
| **INV-RING0-DETERMINISM-1**| §26 | Deterministic State Transitions| Ring 0 pure computation | Eingehalten | B2 |
| **INV-ROLE-BINDING-1**| §15.1 | Graph Role Access Isolation | `RoleBinding` in `contextra-ports` | Eingehalten | B2 |
| **INV-SANITY-1** | §22.1 | Pre-flight Configuration Check | Boot checks in facade | Eingehalten | B2 |
| **INV-SEARCH-ACL-1**| §9.4 | ACL Pre-Filter Before Distance | Candidate filtering in `vector` | Teilweise | B2 |
| **INV-SHRED-1** | §20.1 | Key Destruction Receipt | `DeletionReceipt` in `crypto/kms` | Eingehalten | B2 |
| **INV-SIMD-SAFETY-1**| §6.7 | Explicit // SAFETY Comments | `contextra-simd` safety comments | Eingehalten | B2 |
| **INV-SLEEP-CYCLE-1**| §17.2 | Consolidation Sleep Passes | Background workers in `cognition` | Eingehalten | B2 |
| **INV-SSI-ACTIVE-1**| §8.3 | SSI Conflict Validation | Tx conflict validation in `mvcc` | Eingehalten | B2 |
| **INV-SST-TRAILER-1**| §7.2 | 72-Byte SST Format v5 Trailer | SST Builder in `contextra-store` | Eingehalten | B2 |
| **INV-TENANT-1** | §7.1 | Strict Multi-Tenant Isolation | Tenant key encoding & validation | **Verletzt** | B2 |
| **INV-TENANT-IDX-1**| §9.2 | Tenant-Isolated Index Structures| Collection isolation per tenant | Eingehalten | B2 |
| **INV-TLHFD-1** | §15.4 | Temporal Heat Flow Diffusion | Parameter validation in `graph` | Eingehalten | B2 |
| **INV-TOMBSTONE-1**| §7.1 | Tombstone Version Semantics | Tombstone records in `core/store` | Eingehalten | B2 |
| **INV-TRUST-1** | §21.4 | Trust Marker Propagation | Trust markers in MCP responses | Eingehalten | B2 |
| **INV-UNSAFE-ISLANDS-1**| §6.7| Exactly 3 Unsafe Islands | `simd`, `sys`, `wire` crates | Eingehalten | B2 |
| **INV-WAL-HMAC-1** | §7.3 | Chained WAL HMAC Integrity | WAL Record HMAC chain in `store` | Eingehalten | B2 |
| **INV-WAL-SSOT-1** | §7.3 | WAL as Single Source of Truth | WAL append prior to memtable | Eingehalten | B2 |
| **INV-WITNESS-MINT-1**| §20.4 | Layer-Owned Witness Minting | Witness constructors per layer | Eingehalten | B2 |
| **INV-ZEROIZE-1** | §13.1 | Zeroize Keys on Drop | `zeroize::ZeroizeOnDrop` in `crypto`| Eingehalten | B2 |

---

## 5. Architekturbefunde (Modularisierung & Graph)

- **Schichtungsverletzungen (`xtask check-ring-layering`):**
  - `contextra-engine` (Ring 3) hängt im `Cargo.toml` direkt von `contextra-infer-candle` (Ring 2) und `contextra-sandbox` (Ring 2) ab (`kind=normal`).
  - `contextra-cognition` (Ring 3) hängt von `contextra-infer-candle` (Ring 2) ab (`kind=dev`).
  - `contextra-store` (Ring 1) hängt von `contextra-sandbox` (Ring 2) ab (`kind=dev`).
- **Gate-Unwirksamkeit:**
  - `xtask check-ring-layering` gibt bei den obigen Verstößen lediglich Warnungen aus (`Exit 0`), da im Standardmodus `--strict` fehlt.

---

## 6. Geschäftslogik-Befunde (D1 bis D12)

- **D1 (Mandantenisolierung):** Inverted-Index Statistiken in `contextra-text` berechnen globale Document-Frequenzen (`df`), was zu einem BM25-Score-Informationsleck führt (`CTRA-F-07`).
- **D2 (ACL vor Distanz):** HNSW/DiskANN Standardsuchen berechnen Kandidatendistanzen vor der Prädikatsfilterung.
- **D3 (Schreibpfad Atomarität):** 2PC-Transaktionsprotokoll schreibt `CommitIntent::Pending` korrekt vor LSM-Flush und führt Kompensationsaktionen bei Indexfehler aus.
- **D4 (Recovery):** Startsequenz B0–B9 reinigt unfertige Transaktionsintents und stellt beschädigte WAL-Seals wieder her.
- **D5 (Löschpfad):** MCP stellt bei der Löschung einzelner Dokumente keinen `DeletionProof` aus (`CTRA-F-09`).
- **D6 (Forget-Guard):** `dry_run` ist in `handle_forget` standardmäßig `false` (`CTRA-F-08`).
- **D7 (Nichtdeterminismus):** Direkter `rand::thread_rng()` Aufruf in `contextra-store/src/lsm/recovery.rs` umgeht den RNG-Port (`CTRA-F-10`).
- **D10 (Panic Audit):** 1.449 `.unwrap()` und 814 `.expect()` im Produktionscode (`CTRA-F-06`).
- **D11 (Unsafe Audit):** Genau 3 Unsafe-Inseln (`simd`, `sys`, `wire`) mit 358 unsafe-Blöcken, alle dokumentiert.
- **D12 (Gates):** CI-Gates laufen im Warning-Modus und fangen Regresssionen nicht zuverlässig ab.

---

## 7. Code-Qualität und Smells

- **Ausreißer nach Dateilänge (> 1.000 Zeilen):**
  - `crates/contextra-crypto/src/deletion_proof.rs` (1.916 Zeilen) — God-File für Löschbeweise.
  - `crates/contextra-mvcc/src/tx_buffer.rs` (1.302 Zeilen).
  - `crates/contextra-cognition/src/memory_consolidation.rs` (1.124 Zeilen).

---

## 8. Technische Schuld

- **Architekturschuld:** Undokumentierte Ring-Abhängigkeiten in Ring 3 Crates (`Zinsrate: Hoch`).
- **Code-Schuld:** 1.449 `unwrap()`-Stellen im Produktionscode (`Zinsrate: Hoch`).
- **Process-Schuld:** Gates im CI-Prozess nicht mit Exit 1 blockierend geschaltet (`Zinsrate: Kritisch`).

---

## 9. Test- und Gate-Wirksamkeit

- **Mutationsprobe `check-ring-layering`:**
  - Eine synthetische Ring-Verletzung (`contextra-core -> contextra-store`) wurde in `crates/contextra-core/Cargo.toml` eingefügt.
  - **Ergebnis:** `cargo run -p xtask -- check-ring-layering` meldete die Verletzung, beendete den Prozess aber mit `Exit 0`. Das Gate verhindert das Mergen fehlerhaften Codes nicht!

---

## 10. Governance-Realität gegen Soll

| Soll (Governance / Spec) | Ist (Codebase Realität) |
|---|---|
| Zero Panic in Ring 0 & Ring 1 | 1.449 `unwrap()` Aufrufe in produktiven Crates |
| Keinerlei undokumentierte Ringverletzungen | 4 undokumentierte Ringverletzungen im Hauptzweig |
| Forget-Guard erzwingt `dry_run = true` | `handle_forget` führt Löschung ohne `dry_run` aus |
| `DeletionProof` für jede Löschung | Einzeldokumentlöschung im MCP erzeugt keinen Nachweis |

---

## 11. Empfohlener Sanierungsfahrplan

1. **Schritt 1 (Blocker / S1 - Aufwand: S):** `xtask check-ring-layering` im CI-Workflow mit `--strict` auf `Exit 1` schalten. `dry_run` Default in `contextra-mcp` auf `true` ändern.
2. **Schritt 2 (Mandantentrennung / S1 - Aufwand: M):** Inverted-Index BM25 Statistiken in `contextra-text` mandantenisoliert kapseln.
3. **Schritt 3 (Architektur / S2 - Aufwand: M):** Concrete Inferenz- und Sandbox-Kopplungen in `contextra-engine` durch Port-Traits in `contextra-ports` ersetzen.
4. **Schritt 4 (Code-Qualität / S1 - Aufwand: L):** `unwrap()` und `expect()` im Ring 0 & Ring 1 Produktionscode schrittweise reduzieren.

---

## 12. Offene Fragen an das Projekt

1. Ist die Auslassung von `DeletionProof` Löschnachweisen bei Einzel-Dokument-Löschungen im MCP-Server eine bewusste Performance-Entscheidung oder eine unvollständige Spezifikationsumsetzung?
2. Wie soll die geforderte bit-identische Rebuild-Garantie (§26) mit FMA/AVX-512 Gleitkomma-Akkumulationen (§9.6) auf heterogener Hardware in Zukunft abgeglichen werden?
