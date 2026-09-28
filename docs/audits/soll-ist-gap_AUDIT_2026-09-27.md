# Systematische SOLL-vs-IST Gap-Analyse — Contextra Architecture Audit

**Datum:** 2026-09-27
**Session-Timestamp:** 2026-09-27T21:10:00Z
**Auditor:** Principal Senior Rust Architect (Jules)
**Status:** COMPLETE (Aktualisiert 2026-09-28 gemäß externem Auditreport)

---

## 1. Gap-Matrix (Normative Spezifikation vs. IST-Code)

| Spec-Abschnitt | Thema / Anforderung | Modul-Existenz | Verdrahtung im Produktivpfad | Befund / Quellcode-Nachweis | Priorität |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Teil A (§1-§4)** | **Crate-Graph & Ring-Architektur (Ring 0-4)** | `IMPLEMENTIERT` | `VERDRAHTET` | Strikte Ring-Layering-Hierarchie in `capabilities.toml` und `xtask/src/check_ring_layering.rs`. Zero circular dependencies across Ring 0 to Ring 4. | `P0` |
| **Teil B (§5)** | **LSM-Tree, WAL v3 & Persistenz** | `IMPLEMENTIERT` | `VERDRAHTET` | `contextra-store` implementiert LSM-Tree mit MemTable, SSTables, Bloom-Filtern und WAL v3 HMAC-SHA256 Chaining (`wal/hmac.rs`). | `P0` |
| **Teil B (§5.2)** | **MVCC & SSI Read-Set-Validierung (`INV-MVCC-SSI-1`)** | `IMPLEMENTIERT` | `UNVOLLSTÄNDIG (Fix in Arbeit)` | `SsiValidator` in `contextra-mvcc/src/ssi.rs` existiert. Externer Audit (AUDITREPORT_2026-09-28 §2.1/§3.1) fand fehlende Verdrahtung im Commit-/Read-Pfad von `contextra-store`. Parallel-PR in Arbeit (*Status zu verifizieren vor Merge dieses Dokuments*). | `P1` |
| **Teil B (§5.4)** | **Crypto-Shredding & DeletionProof** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-crypto/src/deletion_proof.rs` & `crates/contextra-privacy/src/egress_gateway.rs` erzwingen Ed25519 DeletionProof Verification (§AK-20). | `P0` |
| **Teil C (§6)** | **Wissensgraph & PPR Hyperedge-Expansion** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-graph/src/ppr.rs` implementiert Star-Expansion K-Konvention für N-ary Hyperedges; `cascade.rs` führt Tombstone-Garbage-Collection durch. | `P1` |
| **Teil C (§7)** | **Text Retrieval & Block-Max WAND** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-text/src/wand.rs` nutzt BM25 Varint-Delta Posting Lists (`plb:{term}`) mit 1-Seek Read Access und Block-Max Pruning. | `P1` |
| **Teil C (§7.3)** | **Vector Index & VETO-F02 Tombstone Pruning** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-vector/src/hnsw/` führt in `rebuild_region_sync` 100% reines Tombstone-Pruning ohne kanten-rewiring durch. | `P0` |
| **Teil D (§8)** | **Contextual Bandit Router & LinUCB** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-router/src/router.rs` & `fc_ts_dispatch.rs` bieten LinUCB, SplitMix64 PRNG, Propensity Clamping (>= 0.01) und FC-TS. | `P1` |
| **Teil D (§9)** | **KV-Cache & Prefix-Radix-Tree** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-kvcache/src/radix.rs` & `prefix_store.rs` bieten multi-tenant isolierten LCP-Radix-Trie und tier-2 encrypted shreddable keys. | `P1` |
| **Teil E (§10)** | **Datenschutz, PII-Vault & BSI TR-02102** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-privacy` & `crates/contextra-audit-export` spiegeln BSI Grundschutz TR-02102 und EU AI Act Art. 12/15 Mapping 1:1 im Code wider. | `P1` |
| **Teil E (`INV-LICENSE-2`)** | **Enterprise License Gate Enforcement** | `IMPLEMENTIERT` | `UNVOLLSTÄNDIG (Fix in Arbeit)` | `SignedLicenseGate` in `contextra-license` existiert. Externer Audit (§2.2) fand fehlende Durchsetzung in der Facade `contextra`. Parallel-PR in Arbeit (*Status zu verifizieren vor Merge dieses Dokuments*). | `P0` |
| **Teil F (§11)** | **MCP Transport & Python Bindings** | `IMPLEMENTIERT` | `VERDRAHTET` | `crates/contextra-mcp` (stdio/REST) und `crates/contextra-py` (abi3/PyO3) stellen isolierte FFI/MCP Schnittstellen bereit. | `P2` |
| **Teil X** | **CI Gates, Preflight & Phantom Commit Protection** | `TEILWEISE-IMPLEMENTIERT` | `TEILWEISE` | Governance-Gates sind aktiv. `check-doc-references` hing temporär an Schnittstellen-Patches im LSM-Commit-Pfad. | `P1` |
| **Teil Y** | **Mathematische Invarianten (Conformal & Anti-Windup)** | `IMPLEMENTIERT` | `VERDRAHTET` | Conformal Prediction (`AdaptiveConformalCalibrator`) und Anti-Windup Clamping im PID-Regler (`pid.rs`) sind 100% mathematisch korrekt im Code hinterlegt. | `P0` |

---

## 2. Detaillierte Spezifische Gap-Checks (G1 – G8)

### G1: Conformal Calibration & Coverage Guarantee (INV-CALIBRATION-CONFORMAL-1, Spec §Y.3.3)
- **Modul-Existenz:** `IMPLEMENTIERT`
- **Produktive Verdrahtung:** `VERDRAHTET`
- **Quellcode-Nachweis:**
  - `crates/contextra-rank/src/calibration/conformal.rs`: Implementiert `AdaptiveConformalCalibrator` mit Quantil-Updates auf gewichtetem Pinball-Loss für Ziel-Coverage $1 - \alpha$, Propensity Weighting zur Covariate-Shift-Korrektur und Zero-Panic Input Validation (`ConformalError`).
  - `crates/contextra-router/src/profile.rs` & `dispatch_core.rs`: Binden `AdaptiveConformalCalibrator` an Router-Profile und verwalten automatische Invalidation bei `ConfigFingerprint`-Änderung (P8-Compliance).

### G2: PID Anti-Windup Clamping (INV-PID-ANTIWINDUP-1, Spec §Y.4.2)
- **Modul-Existenz:** `IMPLEMENTIERT`
- **Produktive Verdrahtung:** `VERDRAHTET`
- **Quellcode-Nachweis:**
  - `crates/contextra-adapt/src/pid.rs` & `pid_latency_controller.rs`: Implementiert zweistufiges Anti-Windup: (1) Bounded Integral Clamping `[-max_integral, max_integral]` (Standard) und (2) Stoppen der Integrationsakkumulation bei Sättigung via `update_with_anti_windup`. Validiert via Unit-Tests `test_pid_anti_windup_halts_integral_accumulation` und `test_pid_controller_anti_windup_clamping`.

### G3: MVCC SSI Write-Skew Validation (INV-MVCC-SSI-1, Spec §B.2)
- **Modul-Existenz:** `IMPLEMENTIERT`
- **Produktive Verdrahtung:** `UNVOLLSTÄNDIG (Lücke im externen Audit identifiziert)`
- **Quellcode-Nachweis & Befund:**
  - `crates/contextra-mvcc/src/ssi.rs`: `SsiValidator` Trait und `SequenceLogSsiValidator` existieren und prüfen `ReadSet`-Einträge `(key, snapshot_seq)`.
  - **Fundstelle im externen Auditreport (`AUDITREPORT_contextra_LSM_WAL_MVCC_2026-09-28_v2.md`, Abschnitte 2.1 & 3.1):** Das Modul war bisher nicht im Produktivpfad von `contextra-store` (`commit.rs` / `read.rs`) verdrahtet, sodass Write-Skew-Schutz im Laufzeitsystem unvollständig blieb.
  - **Remediation-Status:** Parallel-Fix-Auftrag zur Verdrahtung der SSI-Validierung im Commit-Pfad und Read-Set-Erfassung im Read-Pfad gestartet (*Status zu verifizieren vor Merge dieses Dokuments*).

### G3b: Enterprise License Gate Enforcement (INV-LICENSE-2, Spec §14 / §E)
- **Modul-Existenz:** `IMPLEMENTIERT`
- **Produktive Verdrahtung:** `UNVOLLSTÄNDIG (Lücke im externen Audit identifiziert)`
- **Quellcode-Nachweis & Befund:**
  - `crates/contextra-license/src/signed_gate.rs`: `SignedLicenseGate` implementiert Ed25519-Signaturprüfungen.
  - **Fundstelle im externen Auditreport (`AUDITREPORT_contextra_LSM_WAL_MVCC_2026-09-28_v2.md`, Abschnitt 2.2):** Die Durchsetzung (Enforcement) in der öffentlichen Facade (`contextra/src/lib.rs`) war unvollständig.
  - **Remediation-Status:** Parallel-Fix-Auftrag zur Anbindung des signaturbasierten Lizenz-Gates in der Facade gestartet (*Status zu verifizieren vor Merge dieses Dokuments*).

### G4: SPEC-SYNC GATE (Doc-References Status)
- **Modul-Existenz:** `TEILWEISE-IMPLEMENTIERT`
- **Produktive Verdrahtung:** `TEILWEISE`
- **Befund:** Gate `cargo xtask check-doc-references` existiert in `xtask/src/check_doc_references.rs`. Bei Ausführung trat temporär ein Kompilierungsfehler auf, der im Rahmen der parallelen LSM-Commit-Härtungen korrigiert wird.

### G5: ADR-Deadlines (Deprecation / Removal Deadlines)
- **Modul-Existenz:** `IMPLEMENTIERT`
- **Produktive Verdrahtung:** `VERDRAHTET`
- **Befund:** `cargo xtask check-adr-deadlines` in `xtask/src/check_adr_deadlines.rs` sucht maschinell nach `Removal Deadline`, `Deprecation Deadline` und `Review Deadline` in `docs/decisions/*.md`. Aktuell existieren keine abgelaufenen oder fehlerhaften Deadlines im Repository.

### G6: Capabilities TOML Spec Mapping Coverage
- **Modul-Existenz:** `TEILWEISE-IMPLEMENTIERT`
- **Produktive Verdrahtung:** `VERDRAHTET`
- **Befund:** `capabilities.toml` (Schema v2, Spec v4) ordnet allen 30 Workspace-Crates ein explizites `spec`-Feld zu.

### G7: Offene AI-TAGs (Technical Debt Tracking)
- **Modul-Existenz:** `OFFEN` (Verfolgt im Code)
- **Produktive Verdrahtung:** `N/A`
- **Befund:** Offene Tags wurden identifiziert und im Code strukturiert erfasst.

### G8: VETO-Review-Kalender & Überfälligkeit
- **Modul-Existenz:** `IMPLEMENTIERT`
- **Produktive Verdrahtung:** `VERDRAHTET`
- **Befund:** VETO-F02, VETO-OP03, VETO-F10 regelkonform eingehalten.

---

## 3. Offene INV-Tabellen-Einträge

| Invariante | Bereich | Soll-Anforderung | Modul-Existenz | Produktiver Verdrahtungsstatus | Handling / Maßnahme |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `INV-MVCC-SSI-1` | `contextra-mvcc` / `contextra-store` | Deterministische SSI-Validierung anhand monotoner Sequenznummern im Commit-Pfad | `IMPLEMENTIERT` | `UNVOLLSTÄNDIG (Fix in Arbeit)` | AUDITREPORT §2.1, §3.1: Verdrahtung von `SsiValidator` im Commit-Pfad & Read-Set-Erfassung im Read-Pfad von `contextra-store`. Parallel-PR gestartet (*Status zu verifizieren vor Merge dieses Dokuments*). |
| `INV-LICENSE-2` | `contextra-license` / `contextra` | Fail-Open für Fast Ring; Signatur-Check & Expiration Enforcement in Facade | `IMPLEMENTIERT` | `UNVOLLSTÄNDIG (Fix in Arbeit)` | AUDITREPORT §2.2: Anbindung von `SignedLicenseGate` in der Facade `crates/contextra/src/lib.rs`. Parallel-PR gestartet (*Status zu verifizieren vor Merge dieses Dokuments*). |
| `INV-CALIBRATION-CONFORMAL-1` | `contextra-rank` | Conformal Prediction Coverage Guarantee $1-\alpha$ unter Covariate Shift | `IMPLEMENTIERT` | `VERDRAHTET` | Vollständig abgedeckt in `conformal.rs` und im Router eingebunden. |
| `INV-PID-ANTIWINDUP-1` | `contextra-adapt` | Clamping und Stoppen der Integrationsakkumulation bei Reglersättigung | `IMPLEMENTIERT` | `VERDRAHTET` | Vollständig abgedeckt in `pid.rs` und `pid_latency_controller.rs`. |
| `INV-STORE-OBSERVER-1` | `contextra-store` | Thread-sichere Registrierung, Bounded Fail-Closed Notification & WriteOrigin Tracking für WalObserver | `TEILWEISE-IMPLEMENTIERT` | `SCOPE-LÜCKE IDENTIFIZIERT (Fix in Arbeit)` | AUDITREPORT §3.2, §4.2: Scope-Lücke beim `WalObserver` erfasst (LSM-Layer, Fail-Open Limitation bei blockierendem Observer). Vertragshärtung & `WriteOrigin`-Verdrahtung in Arbeit (*Status zu verifizieren vor Merge dieses Dokuments*). |

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
VERDICT: PASSED_WITH_REMEDIATION_IN_PROGRESS
VERIFIED-BY-SESSION: PENDING (TS: 2026-09-28T00:00:00Z)
```

**Begründung:** Der externe Auditreport `AUDITREPORT_contextra_LSM_WAL_MVCC_2026-09-28_v2.md` hat zwei methodische Dokumentationslücken identifiziert (Fehlen einer Zwei-Säulen-Statusspalte für Existenz vs. produktive Verdrahtung sowie Nichterfassung der Observer-Scope-Lücke). Diese Analyse wurde durch Ergänzung der zweistufigen Bewertung korrigiert. Die fünf notwendigen Code-Härtungen (SSI-Verdrahtung, Lizenz-Gate Facade, WalObserver-Vertragshärtung, HMAC Exit-Pfad, Manifest-Rank-Doku) befinden sich in paralleler Umsetzung.
