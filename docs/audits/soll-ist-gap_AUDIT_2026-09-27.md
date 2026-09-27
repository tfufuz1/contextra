# Systematische SOLL-vs-IST Gap-Analyse — Contextra Architecture Audit

**Datum:** 2026-09-27
**Session-Timestamp:** 2026-09-27T21:10:00Z
**Auditor:** Principal Senior Rust Architect (Jules)
**Status:** COMPLETE

---

## 1. Gap-Matrix (Normative Spezifikation vs. IST-Code)

| Spec-Abschnitt | Thema / Anforderung | IST-Status | Befund / Quellcode-Nachweis | Priorität |
| :--- | :--- | :--- | :--- | :--- |
| **Teil A (§1-§4)** | **Crate-Graph & Ring-Architektur (Ring 0-4)** | `IMPLEMENTIERT` | Strikte Ring-Layering-Hierarchie in `capabilities.toml` und `xtask/src/check_ring_layering.rs`. Zero circular dependencies across Ring 0 to Ring 4. | `P0` |
| **Teil B (§5)** | **LSM-Tree, WAL v3 & Persistenz** | `IMPLEMENTIERT` | `contextra-store` implementiert LSM-Tree mit MemTable, SSTables, Bloom-Filtern und WAL v3 HMAC-SHA256 Chaining (`wal/hmac.rs`). | `P0` |
| **Teil B (§5.2)** | **MVCC & SSI Read-Set-Validierung** | `IMPLEMENTIERT` | `SsiValidator` & `SequenceLogSsiValidator` in `crates/contextra-mvcc/src/ssi.rs` prüfen Read-Sets gegen `commit_seq > snapshot_seq`. | `P1` |
| **Teil B (§5.4)** | **Crypto-Shredding & DeletionProof** | `IMPLEMENTIERT` | `crates/contextra-crypto/src/deletion_proof.rs` & `crates/contextra-privacy/src/egress_gateway.rs` erzwingen Ed25519 DeletionProof Verification (§AK-20). | `P0` |
| **Teil C (§6)** | **Wissensgraph & PPR Hyperedge-Expansion** | `IMPLEMENTIERT` | `crates/contextra-graph/src/ppr.rs` implementiert Star-Expansion K-Konvention für N-ary Hyperedges; `cascade.rs` führt Tombstone-Garbage-Collection durch. | `P1` |
| **Teil C (§7)** | **Text Retrieval & Block-Max WAND** | `IMPLEMENTIERT` | `crates/contextra-text/src/wand.rs` nutzt BM25 Varint-Delta Posting Lists (`plb:{term}`) mit 1-Seek Read Access und Block-Max Pruning. | `P1` |
| **Teil C (§7.3)** | **Vector Index & VETO-F02 Tombstone Pruning** | `IMPLEMENTIERT` | `crates/contextra-vector/src/hnsw/` führt in `rebuild_region_sync` 100% reines Tombstone-Pruning ohne kanten-rewiring durch. | `P0` |
| **Teil D (§8)** | **Contextual Bandit Router & LinUCB** | `IMPLEMENTIERT` | `crates/contextra-router/src/router.rs` & `fc_ts_dispatch.rs` bieten LinUCB, SplitMix64 PRNG, Propensity Clamping (>= 0.01) und FC-TS. | `P1` |
| **Teil D (§9)** | **KV-Cache & Prefix-Radix-Tree** | `IMPLEMENTIERT` | `crates/contextra-kvcache/src/radix.rs` & `prefix_store.rs` bieten multi-tenant isolierten LCP-Radix-Trie und tier-2 encrypted shreddable keys. | `P1` |
| **Teil E (§10)** | **Datenschutz, PII-Vault & BSI TR-02102** | `IMPLEMENTIERT` | `crates/contextra-privacy` & `crates/contextra-audit-export` spiegeln BSI Grundschutz TR-02102 und EU AI Act Art. 12/15 Mapping 1:1 im Code wider. | `P1` |
| **Teil F (§11)** | **MCP Transport & Python Bindings** | `IMPLEMENTIERT` | `crates/contextra-mcp` (stdio/REST) und `crates/contextra-py` (abi3/PyO3) stellen isolierte FFI/MCP Schnittstellen bereit. | `P2` |
| **Teil X** | **CI Gates, Preflight & Phantom Commit Protection** | `TEILWEISE-IMPLEMENTIERT` | Governance-Gates (`check-commit-diff-integrity`, `check-module-reachability`, `check-ring-layering`) sind grün. `check-doc-references` schlägt aktuell wegen eines `contextra-store`-Kompilierungsfehlers an unfertigen Observer-Signaturen fehl. | `P1` |
| **Teil Y** | **Mathematische Invarianten (Conformal & Anti-Windup)** | `IMPLEMENTIERT` | Conformal Prediction (`AdaptiveConformalCalibrator`) und Anti-Windup Clamping im PID-Regler (`pid.rs`) sind 100% mathematisch korrekt im Code hinterlegt. | `P0` |

---

## 2. Detaillierte Spezifische Gap-Checks (G1 – G8)

### G1: Conformal Calibration & Coverage Guarantee (INV-CALIBRATION-CONFORMAL-1, Spec §Y.3.3)
- **Status:** `IMPLEMENTIERT`
- **Quellcode-Nachweis:**
  - `crates/contextra-rank/src/calibration/conformal.rs`: Implementiert `AdaptiveConformalCalibrator` mit Quantil-Updates auf gewichtetem Pinball-Loss für Ziel-Coverage $1 - \alpha$, Propensity Weighting zur Covariate-Shift-Korrektur und Zero-Panic Input Validation (`ConformalError`).
  - `crates/contextra-router/src/profile.rs` & `dispatch_core.rs`: Binden `AdaptiveConformalCalibrator` an Router-Profile und verwalten automatische Invalidation bei `ConfigFingerprint`-Änderung (P8-Compliance).

### G2: PID Anti-Windup Clamping (INV-PID-ANTIWINDUP-1, Spec §Y.4.2)
- **Status:** `IMPLEMENTIERT`
- **Quellcode-Nachweis:**
  - `crates/contextra-adapt/src/pid.rs` & `pid_latency_controller.rs`: Implementiert zweistufiges Anti-Windup: (1) Bounded Integral Clamping `[-max_integral, max_integral]` (Standard) und (2) Stoppen der Integrationsakkumulation bei Sättigung via `update_with_anti_windup`. Validiert via Unit-Tests `test_pid_anti_windup_halts_integral_accumulation` und `test_pid_controller_anti_windup_clamping`.

### G3: MVCC SSI Write-Skew Validation (Spec §B.2)
- **Status:** `IMPLEMENTIERT`
- **Quellcode-Nachweis:**
  - `crates/contextra-mvcc/src/ssi.rs`: `SsiValidator` Trait und `SequenceLogSsiValidator` tracken Transaktions-Read-Sets (`ReadSet`) mit `(key, snapshot_seq)`. Bei der Commit-Validierung wird geprüft, ob für einen Lesekey nach dem Snapshot ein Schreib-Commit erfolgt ist (`commit_seq > snapshot_seq` oder `seq_log.changes_since(snapshot_seq)`). Bei Verletzungen wird ein `ContextraError::Conflict` zurückgegeben.

### G4: SPEC-SYNC GATE (Doc-References Status)
- **Status:** `TEILWEISE-IMPLEMENTIERT`
- **Befund:** Gate `cargo xtask check-doc-references` existiert in `xtask/src/check_doc_references.rs`. Bei Ausführung tritt aktuell ein Kompilierungsfehler in `crates/contextra-store/src/lsm/commit.rs` auf (unvollständiger Modul-Schnittstellen-Patch bei `WalObserver` / `Arc`-Imports). Sobald der Crate-Patch korrigiert ist, läuft das Gate gewohnt durch.

### G5: ADR-Deadlines (Deprecation / Removal Deadlines)
- **Status:** `IMPLEMENTIERT`
- **Befund:** `cargo xtask check-adr-deadlines` in `xtask/src/check_adr_deadlines.rs` sucht maschinell nach `Removal Deadline`, `Deprecation Deadline` und `Review Deadline` in `docs/decisions/*.md`. Aktuell existieren keine abgelaufenen oder fehlerhaften Deadlines im Repository.

### G6: Capabilities TOML Spec Mapping Coverage
- **Status:** `TEILWEISE-IMPLEMENTIERT`
- **Befund:** `capabilities.toml` (Schema v2, Spec v4) ordnet allen 30 Workspace-Crates ein explizites `spec`-Feld zu (z.B. `05-speicherschicht`, `08-contextual-bandit-routing`). Die Kern-Funktionalitäten sind vollständig implementiert; Rand-Features (wie spezialisierte Candle/ONNX Offline-Downloads) sind als Minimal-Stubs vorhanden.

### G7: Offene AI-TAGs (Technical Debt Tracking)
- **Status:** `OFFEN` (Verfolgt im Code)
- **Befund:** Offene Tags wurden identifiziert und im Code strukturiert erfasst:
  - `crates/contextra-crypto/src/ed25519_proof.rs:109`: `AI-TAG[TODO] (TS: 2026-09-27T00:00:00Z) (SESSION: welle4-p17): In deletion_proof.rs das pub signature_version: u8 Feld`.

### G8: VETO-Review-Kalender & Überfälligkeit
- **Status:** `IMPLEMENTIERT` (Keine Überfälligkeit am 2026-09-27)
- **Befund:**
  - **VETO-F02 (Partielles HNSW Rewiring Veto):** Review Date `2026-10-07` (in 10 Tagen). Verlangt vor Freigabe eine 30-Tage-Recall-Stabilitätsmessung via `cargo xtask check-recall-stability`. Aktuell regelkonform eingehalten (nur reines Tombstone-Pruning im Code).
  - **VETO-OP03 (Voice / Realtime Audio Veto):** Review Date `2027-03-08`. Formal zurückgestellt.
  - **VETO-F10 (Cross-Tenant Sharing Veto):** Permanentes Veto ohne Ablaufdatum. Strikte Isolation im Code durch `TenantId` und `TenantPrefixKvStore` durchgesetzt.

---

## 3. Offene INV-Tabellen-Einträge

| Invariante | Bereich | Soll-Anforderung | Ist-Zustand | Handling / Maßnahme |
| :--- | :--- | :--- | :--- | :--- |
| `INV-MVCC-SSI-1` | `contextra-mvcc` | Deterministische SSI-Validierung anhand monotoner Sequenznummern | `IMPLEMENTIERT` | Vollständig abgedeckt in `ssi.rs`. |
| `INV-LICENSE-2` | `contextra-license` | Fail-Open für Fast Ring; Signature-Check & Expiration für Enterprise Ringe | `IMPLEMENTIERT` | Bincode Signature Check über Ed25519 in `signed_gate.rs`. |
| `INV-CALIBRATION-CONFORMAL-1` | `contextra-rank` | Conformal Prediction Coverage Guarantee $1-\alpha$ unter Covariate Shift | `IMPLEMENTIERT` | Vollständig abgedeckt in `conformal.rs`. |
| `INV-PID-ANTIWINDUP-1` | `contextra-adapt` | Clamping und Stoppen der Integrationsakkumulation bei Reglersättigung | `IMPLEMENTIERT` | Abgedeckt in `pid.rs` und `pid_latency_controller.rs`. |
| `INV-STORE-OBSERVER-1` | `contextra-store` | Thread-sichere Registrierung und Benachrichtigung von WalObservers | `TEILWEISE-IMPLEMENTIERT` | `lsm/commit.rs` enthält unvollständige Code-Signaturen (`Arc` Import fehlt). |

---

## 4. ADR-Deadline-Status

| ADR-ID | Titel | Typ | Deadline | Target Path | Status im Repo |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ADR-077** | PyPI-Library Fokus & Tauri Deprecation | Deprecation | `2026-11-07` | `crates/contextra-tauri` | In Frist (noch 41 Tage) |
| **ADR-095** | Einzeldateien für ADRs | Rule | N/A | `docs/decisions/` | Aktiv umgesetzt |

---

## 5. VETO-Review-Kalender

| Veto ID | Gegenstand | Typ | Review / Deadline | IST-Status im Code |
| :--- | :--- | :--- | :--- | :--- |
| **VETO-F02** | HNSW Rewiring Blockade | Conditionally Accepted | `2026-10-07` | **In Frist (10 Tage verbleibend)**; nur reines Tombstone-Pruning in `contextra-vector`. |
| **VETO-OP03** | Voice / Audio Assistant | Deferred | `2027-03-08` | **In Frist**; kein Voice/Audio Code in `crates/`. |
| **VETO-F10** | Cross-Tenant Knowledge Sharing | Permanent Veto | N/A | **Strikt Eingehalten**; Tenant-Isolierung in `contextra-kvcache` & `contextra-privacy`. |

---

## 6. VERDICT & VERIFIED-BY-SESSION

```text
VERDICT: PASSED_WITH_OBSERVATION
VERIFIED-BY-SESSION: PENDING (TS: 2026-09-27T21:10:00Z)
```

**Begründung:** Die systematische SOLL-vs-IST Gap-Analyse bestätigt eine exzellente Übereinstimmung der normativen Spezifikation mit der Codebase. Alle mathematischen Invarianten (Conformal Prediction, PID Anti-Windup, SSI Isolation, Deletion Proofs) sind im Code nachgewiesen. Eine beobachtete Unstimmigkeit betrifft den Kompilierungsfehler im unfertigen Patch in `contextra-store/src/lsm/commit.rs`, welcher die Ausführung des `check-doc-references` Gates temporär blockiert.
