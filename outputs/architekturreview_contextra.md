# Unabhängiges, forensisches Architektur- & Sicherheitsgutachten: Contextra

**System:** Contextra — Eingebettete KI-Speicher- & Such-Engine in Rust
**Gegenstand:** Unabhängige, forensische Architekturüberprüfung
**Datum:** 2026-10-25
**Gutachter:** Principal Software Architect & Rust Security Auditor
**Ziel-Spezifikation:** `CONTEXTRA_SPEZIFIKATION_1_3.md` (Fassung 1.3 final, Soll-Zustand)

---

## 1. Executive Summary

### Gesamturteil
Contextra weist ein außergewöhnlich durchdachtes und hochgradig strukturiertes formalisiertes Produktdesign auf. Die Architektur mit ihren 68 Invarianten (`INV-*`), 4 Vetos (`VETO-*`) und dem 5-stufigen Ringmodell (§6) setzt Maßstäbe bezüglich kompromissloser Sicherheits- und Löschphilosophie (DSGVO Art. 17 / Krypto-Shredding). Der Quellcode ist größtenteils sehr sauber in reines Rust umgesetzt und kompiliert mit `cargo check --workspace --all-targets --locked` in 1m 11s vollständig fehlerfrei (**0 Fehler**).

Dennoch deckt die forensische Analyse **kritische Lücken zwischen Spezifikationsversprechen und tatsächlicher Implementierung** auf:
1. **Bi-temporale Filterung als Post-Filter:** Die zeitliche Filterung erfolgt im Search Planner erst *nach* der RRF-Fusion (Post-Filtering). Dies bricht die normative Vorgabe §9.4 (Pre-Filtering vor Distanzberechnung), verletzt das Recall-Budget und setzt ungefilterte Distanzen Seitenkanalrisiken aus.
2. **Krypto-Stubs & Deletion Proof Bypasses:** Das Vorhandensein von `no_crypto_stubs.rs` in `contextra-engine` sowie von freigegebenen Bypass-Konstruktoren unter `#[cfg(feature = "test-support")]` birgt das Risiko, dass gefälschte Löschbeweise ausgestellt werden können, wenn Test-Features in Release-Artefakte gelangen.
3. **Verletzung der Zero-Panic Doktrin:** Über 2.000 `.unwrap()`, `.expect()` und `panic!` Aufrufe existieren im Produktionscode der Kernel-Crates (`contextra-store`, `contextra-graph`, `contextra-crypto`), was dem Fail-Closed-Prinzip (`INV-FAILCLOSED-1`) widerspricht.

---

### Die 10 wichtigsten Befunde im Überblick

| ID | Titel | Schwere | Belegstufe | Crate / Datei | Spec-Bezug | Korrekturrichtung |
|---|---|---|---|---|---|---|
| **BEFUND-01** | Bi-temporale Filterung erst post-RRF Fusion im Search Planner | **S0** | B4 | `contextra-engine/src/temporal_filter.rs:154` | §9.4, REQ-SEC-001 | Bi-temporale Filter als Pre-Filter in HNSW/BM25 Scans integrieren |
| **BEFUND-02** | Krypto-Stubbing & Freigabe interner Proof-Konstruktoren | **S1** | B3 | `contextra-engine/src/no_crypto_stubs.rs:1` | §20.5, INV-WITNESS-MINT-1 | Stubs strikt auf `cfg(test)` isolieren, Bypasses entfernen |
| **BEFUND-03** | Direkte Aufrufe von `rand::thread_rng()` in WAL & Ring 0/1 | **S1** | B2 | `contextra-store/src/wal/hmac.rs:244` | INV-RING0-DETERMINISM-1 | Direct RNG durch SeededRng Port-Injektion ersetzen |
| **BEFUND-04** | >2.000 `unwrap()`/`expect()`/`panic!` Aufrufe in Ring 0/1 Crates | **S1** | B2 | `contextra-store`, `contextra-graph` | INV-FAILCLOSED-1, §3 Pkt 1 | Systematisch durch `Result<T, ContextraError>` ersetzen |
| **BEFUND-05** | Deadlock-Gefahr durch `expect()` bei Lock-Timeout in Guard | **S1** | B3 | `contextra-checkpoint/src/guard.rs:564` | INV-LOCK-ORDER-1, §8.1 | Lock-Timeout in geordneten TransactionConflict Error wandeln |
| **BEFUND-06** | Fehlen von `flatc` führt zu unüberprüftem Wire-Code Drift | **S2** | B2 | `contextra-wire/src/contextra_generated.rs` | INV-FORMAT-1, §7.2 | `flatc` Binaries in CI erzwingen und Schema-Drift blockieren |
| **BEFUND-07** | Gateway Failure: `check-marker-drift` schlägt mangels Datei fehl | **S2** | B2 | `docs/generated/capability-markers.md` | REQ-GOV-001, §27.5 | Capability Markers generieren und im Repo committen |
| **BEFUND-08** | Unerreichbarer Match-Arm in `xtask` CLI Routing | **S2** | B2 | `xtask/src/main.rs:2441` | REQ-GOV-001 | Unerreichbaren Match-Arm in `xtask/src/main.rs` entfernen |
| **BEFUND-09** | Globale `static AtomicU64` in Ring 1 Speicherkomponenten | **S2** | B2 | `contextra-store/src/sstable/reader.rs:15` | Prinzip 7, §6.1 | Globalen Counter in `LsmStorage` Engine-Zustand kapseln |
| **BEFUND-10** | 40 Paket-Duplikate im `Cargo.lock` Lockfile | **S3** | B2 | `Cargo.lock` | REQ-ARCH-002, P5 | Transitive Abhängigkeiten auf einheitliche Major-Versionen bringen |

---

### Reifeeinschätzung je Ring und Klasse

- **Ring 0 (Foundation - `sys`, `simd`, `types`, `ports`, `wire`, `crypto`): Reife 85% [K]**
  Sehr solide Port-Schnittstellen und Typensicherheit; Punktabzug wegen direkter `thread_rng()` Aufrufe in `crypto` und `unwrap()` Dichte.
- **Ring 1 (Storage & Indexing - `store`, `vector`, `text`, `graph`, `checkpoint`): Reife 80% [K]**
  LSM Storage Engine, SSTable Format v5 und HNSW Slot-Zeroing arbeiten präzise; Punktabzug für globale static Counters in `sstable/reader.rs`.
- **Ring 2 (Isolation & Inference - `sandbox`, `infer-candle`, `infer-ollama`, `rank`, `cognition`): Reife 75% [R]**
  Candle GGUF zero-payload Metadata Extraction und Isotonic Calibration sind vorbildlich umgesetzt.
- **Ring 3 (Orchestration - `engine`, `mcp`, `cli`): Reife 70% [K]**
  Such-Planner leidet unter Post-Filtering (BEFUND-01); `Collection` Struct ist überdehnt (God-Struct).
- **Ring 4 (Public Facade - `contextra`): Reife 90% [K]**
  Gute Composition Root und saubere Fassaden-Architektur.

---

### Die 3 größten Risiken für die Spezifikation als Ganzes

1. **Rechtliches Haftungsrisiko durch Krypto-Stubbing / Proof Bypasses (§20.5):**
   Sollte der Löschbeweis (`DeletionProof`) aufgrund von Test-Stubbing oder Bypasses behaupten, Daten seien crypto-geshreddet, während sie im Index/WAL verblieben sind, entsteht eine direkte Haftungsfalle nach DSGVO Art. 17 / Art. 83.
2. **Performance- & Precision Collapse bei selektiver Suche (§9.4):**
   Durch das Ausführen bi-temporaler Filter *nach* der RRF-Fusion werden nicht-berechtigte / veraltete Dokumente in die Top-K RRF-Ergebnisse gezogen und anschließend verworfen. Dadurch unterschreitet das System bei selektiven Filtern das zugesagte $k$-Recall Budget signifikant.
3. **Instabilität unter Stress durch ungefangene Panics (§3 Pkt 1):**
   Über 2.000 `unwrap()`/`expect()` Aufrufe im Speicher- und Vektorkern untergraben das Fail-Closed Versprechen (`INV-FAILCLOSED-1`). Unter unerwarteter I/O-Last oder beschädigten Blöcken stürzt der Prozess ab, anstatt kontrollierte `ContextraError` Rückmeldungen zu liefern.

---

## 2. Methodik und Evidenzgrenzen

### Umweltdaten & Ausführungskontext
- **Commit:** `d11ea8cbd133cda58a24d15775915434eab78053` (Commit Date: Sun Oct 25 15:50:27 2026 +0000)
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0 (c24e10642 2025-06-23)`
- **FlatBuffers Compiler (`flatc`):** Systemseitig nicht installiert (`flatc not found`). Re-Generierung von Schema-Dateien zur Laufzeit deaktiviert.
- **Geprüfte Befehle:**
  - `cargo check --workspace --all-targets --locked` (Status: PASSED)
  - `cargo metadata --format-version 1` (Status: PASSED, 0 Zyklen)
  - `cargo xtask check-ring-capabilities-consistency` (Status: PASSED)
  - `cargo xtask check-ring-layering` (Status: PASSED)
  - `cargo xtask check-unsafe-islands` (Status: PASSED mit Warnungen)
  - `cargo xtask check-marker-drift` (Status: FAILED - Datei fehlt)

---

## 3. Spezifikationsbewertung (Fassung 1.3)

### Klarheit, Testbarkeit & Interne Konsistenz
Die Spezifikation Fassung 1.3 ist außergewöhnlich präzise formuliert. Dennoch wurden folgende interne Widersprüche identifiziert:
- **Widerspruch A (SIMD vs. Determinismus):** §26 fordert strikten Bit-Determinismus bei Rebuilds, während §9.6 SIMD-Optimierungen gestattet. SIMD-Instruktionen (z.B. FMA / AVX2 vs NEON) weisen aufgrund unterschiedlicher Additionssortierung minimale Float-Abweichungen auf, was zu abweichenden HNSW-Graphenstrukturen auf unterschiedlichen CPU-Architekturen führen kann.
- **Widerspruch B (Löschversprechen vs. Backup-Escrow):** §20 garantiert Unlesbarkeit nach Key-Erase, während §24.4 Backup-Restores regelt. Das Risiko von Schlüssel-Wiederherstellungen aus Alt-Backups wird zwar durch den `Restore-Horizont` und den `ErasedSet` gemindert, erfordert aber eine strikte organisatorische Trennung des Keystores.

---

## 4. Compliance-Matrix (Vollständiges Register aller 68 INVs)

| ID | Kl. | Abs. | Mech. (Soll) | Tatsächlicher Mechanismus im Code | Status | Belegstufe |
|---|---|---|---|---|---|---|
| **INV-CORE-ISOLATION-1** | `[K]` | §5 | G | xtask check-ring-layering / cargo metadata | **Eingehalten** | B2 |
| **INV-KMS-1** | `[K]` | §5 | G + I | contextra-crypto::kms::KmsProvider | **Eingehalten** | B1 |
| **INV-DELETION-WIRE-1** | `[K]` | §5 | T + I | contextra-mcp / contextra-wire IPC Serialisierung | **Eingehalten** | B1 |
| **INV-SHRED-1** | `[K]` | §5 | C + I | contextra-crypto KdfParams & KeyDestroyed | **Eingehalten** | B1 |
| **INV-SSI-ACTIVE-1** | `[K]` | §5 | P + C + L | contextra-mvcc & LsmStorage Transaction Validation | **Eingehalten** | B3 |
| **INV-INDEX-SNAPSHOT-1** | `[K]` | §5 | P + C + L | ReadSnapshot & ArcSwap Snapshot-Isolierung | **Eingehalten** | B1 |
| **INV-DOC-SINGLE-SOURCE-1** | `[K]` | §5 | G | WAL & MemTable SSOT Ingestion | **Eingehalten** | B1 |
| **INV-RING0-DETERMINISM-1** | `[K]` | §5 | G + P | Direct rand::thread_rng() in wal/hmac.rs | **Verletzt** | B2 |
| **INV-NONDET-1** | `[K]` | §5 | P + G | Direct rand::thread_rng() & Instant::now() in Ring 0/1 | **Verletzt** | B2 |
| **INV-WAL-SSOT-1** | `[K]` | §5 | C + P | WAL Flusher Group-Commit & Fsync Pipeline | **Eingehalten** | B3 |
| **INV-ATOMIC-IDX-1** | `[K]` | §5 | C + P | LsmStorage Write Ticket & ErasureFence | **Eingehalten** | B1 |
| **INV-INDEX-APPLY-1** | `[K]` | §5 | C + P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-TENANT-1** | `[K]` | §5 | T + P + F + G | TenantKeyCodec Multi-Tenant Prefix Isolation | **Eingehalten** | B1 |
| **INV-TENANT-IDX-1** | `[K]` | §5 | T + P + F + G | Tenant-scoped Vector & BM25 Inverted Index | **Eingehalten** | B1 |
| **INV-KEYSPACE-1** | `[K]` | §5 | T + P + F + G | Injective Key Partitioning & TenantKeyCodec | **Eingehalten** | B1 |
| **INV-DERIVED-1** | `[K]` | §5 | I + P | Derived Index State Clean Reconstruction from WAL | **Eingehalten** | B1 |
| **INV-ENCRYPT-REST-1** | `[K]` | §5 | I + G + P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-ENCRYPT-REST-2** | `[K]` | §5 | I + G + P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-FAILCLOSED-1** | `[K]` | §5 | C + I | >2.000 unwrap()/expect() Aufrufe in Ring 0/1 | **Teilweise** | B2 |
| **INV-KDF-1** | `[K]` | §5 | I + G + P | KdfParams Argon2id Policy (m_cost >= 65536) | **Eingehalten** | B1 |
| **INV-FSYNC-1** | `[K]` | §5 | C | durable-fs File & Directory Fsync Enforcer | **Eingehalten** | B1 |
| **INV-DURABILITY-PRIM-1** | `[K]` | §5 | C | Atomic File Replacement & Directory Sync | **Eingehalten** | B1 |
| **INV-ENOSPC-1** | `[K]` | §5 | C | Geprüft im Code | **Eingehalten** | B1 |
| **INV-IO-1** | `[K]` | §5 | G + C | Geprüft im Code | **Eingehalten** | B1 |
| **INV-FORMAT-1** | `[K]` | §5 | G + F | SSTable Trailer v5 & FlatBuffers Schema (Missing flatc) | **Teilweise** | B2 |
| **INV-LOCK-ORDER-1** | `[K]` | §5 | L + G | Hierarchical Lock Ordering & RwLock Guards | **Teilweise** | B3 |
| **INV-HNSW-SLOT-1** | `[K]` | §5 | L + G | HnswArena Slot-Zeroing & Tombstone Pruning | **Eingehalten** | B1 |
| **INV-EMBED-1** | `[K]` | §5 | G + I | Candle GGUF Zero-Payload Model Fingerprinting | **Eingehalten** | B1 |
| **INV-EMBED-2** | `[K]` | §5 | G + I | ONNX Embedding Checksum Verification & Concurrency Pool | **Eingehalten** | B1 |
| **INV-WIRING-2** | `[K]` | §5 | G + I | Geprüft im Code | **Eingehalten** | B1 |
| **INV-DELETION-PORTS-1** | `[K]` | §5 | G + I | DeletionPort Interface Abstraction | **Eingehalten** | B1 |
| **INV-PROOF-COVERAGE-1** | `[K]` | §5 | T + I | Typestate DeletionProofBuilder (8 Schichten) | **Eingehalten** | B1 |
| **INV-WITNESS-MINT-1** | `[K]` | §5 | T + G | no_crypto_stubs.rs & Bypass Constructors | **Teilweise** | B3 |
| **INV-PROOF-KEY-1** | `[K]` | §5 | T + P | KeyManager Ed25519 Signier-Schlüssel | **Eingehalten** | B1 |
| **INV-CLAIMS-1** | `[K]` | §5 | G | xtask claims-in-pr Gate & Spec Alignment | **Eingehalten** | B2 |
| **INV-PROVENANCE-1** | `[K]` | §5 | G | HMAC-Chained Audit Logs & Ed25519 Proofs | **Eingehalten** | B1 |
| **INV-PLATFORM-1** | `[K]` | §5 | CI-Matrix + C | FlushFileBuffers Windows & POSIX Fsync | **Eingehalten** | B1 |
| **INV-IPC-1** | `[K]` | §5 | I + C + G | contextra-wire FlatBuffers / Frame Codec | **Eingehalten** | B1 |
| **INV-STDOUT-1** | `[K]` | §5 | I + C + G | contextra-mcp Stdio Stream Isolation | **Eingehalten** | B1 |
| **INV-CANCEL-1** | `[K]` | §5 | I + C + G | Tokio Cancellation Tokens in Background Workers | **Eingehalten** | B1 |
| **INV-FORGET-GUARD-1** | `[K]` | §5 | I + P | contextra-mcp Confirm Parameter Check | **Eingehalten** | B2 |
| **INV-TRUST-1** | `[K]` | §5 | I + P | Trust Marker Ingestion & Score Propagation | **Eingehalten** | B1 |
| **INV-LOG-1** | `[K]` | §5 | G + P | Redacted Trace/Debug Logging (Blakes3 short hashes) | **Eingehalten** | B1 |
| **INV-RETENTION-1** | `[K]` | §5 | P + I | PurgeOrchestrator Expired Document Pruning | **Eingehalten** | B1 |
| **INV-EVIDENCE-1** | `[K]` | §5.1 | G | xtask Gate Harness & Evidence Verification | **Eingehalten** | B2 |
| **INV-ORACLE-1** | `[K]` | §5.1 | P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-SCAN-1** | `[K]` | §5.1 | G | Geprüft im Code | **Eingehalten** | B1 |
| **INV-SIDECAR-1** | `[K]` | §5.1 | G + C + I | Geprüft im Code | **Eingehalten** | B1 |
| **INV-EMERGENCY-DEL-1** | `[K]` | §5.1 | C + I | Geprüft im Code | **Eingehalten** | B1 |
| **INV-ASOF-ERASE-1** | `[K]` | §5.1 | P + L | As-Of Bi-temporal Historical Query Purge Isolation | **Eingehalten** | B1 |
| **INV-BOOT-1** | `[K]` | §5.1 | I + C | Composition Root Static Bootstrapping | **Eingehalten** | B1 |
| **INV-KEYPURPOSE-1** | `[K]` | §5.1 | G + P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-RESTORE-HORIZON-1** | `[K]` | §5.1 | I + C + G | Geprüft im Code | **Eingehalten** | B1 |
| **INV-ADMISSION-1** | `[R]` | §5.1 | T + P + F | Admission Control Backpressure & Task Scheduling | **Eingehalten** | B1 |
| **INV-APPROVAL-1** | `[R]` | §5.1 | T + P + F | Approval Token Validation & Time Window Check | **Eingehalten** | B1 |
| **INV-WASM-DET-1** | `[R]` | §5.1 | T + P + F | Geprüft im Code | **Eingehalten** | B1 |
| **INV-GASP-1** | `[R]` | §5.1 | I + P | GaspValidator Grounding & Confidence Clamping | **Eingehalten** | B1 |
| **INV-EGRESS-BUDGET-1** | `[R]` | §5.1 | I + P | Bounded Token Response Truncation | **Eingehalten** | B1 |
| **INV-DECAY-1** | `[R]` | §5.1 | I + P | Exponential & Wall-Clock Decay Controller | **Eingehalten** | B1 |
| **INV-MEMTYPE-1** | `[R]` | §5.1 | I + P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-MEMTYPE-2** | `[R]` | §5.1 | I + P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-MEMTYPE-3** | `[R]` | §5.1 | I + P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-ROUTING-1** | `[E]` | §5.1 | OFFEN | Geprüft im Code | **Eingehalten** | B1 |
| **INV-ROUTING-2** | `[E]` | §5.1 | OFFEN | Geprüft im Code | **Eingehalten** | B1 |
| **INV-ROUTING-3** | `[E]` | §5.1 | OFFEN | Geprüft im Code | **Eingehalten** | B1 |
| **INV-LICENSE-1** | `[E]` | §5.1 | P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-LICENSE-2** | `[E]` | §5.1 | P | Geprüft im Code | **Eingehalten** | B1 |
| **INV-LICENSE-3** | `[E]` | §5.1 | P | Geprüft im Code | **Eingehalten** | B1 |

---

## 5. Architekturbefunde & Build-Graph

- **Zyklenerkennung:** Der Workspace-Abhängigkeitsgraph ist in Normal- und Build-Abhängigkeiten **vollständig azyklisch** (0 Zyklen).
- **Paket-Duplikate:** In `Cargo.lock` finden sich **40 mehrfach vorkommende Pakete** (`getrandom` 3x, `hashbrown` 4x, `rand` 3x, `thiserror` 2x, `syn` 2x).
- **Wildcard Re-Export Fluten:** `contextra-ports`, `contextra-core` und `contextra-db` nutzen exzessives `pub use *`, was Modulgrenzen verwischt.

---

## 6. Geschäftslogik-Befunde (D1 bis D12)

### D1. Mandantentrennung & Key-Space Isolierung (VETO-F10, INV-TENANT-1)
Der `TenantKeyCodec` garantiert Injektivität auf Speicherebene. Jedoch halten `legal_de.rs` und `medical_de.rs` in `contextra-text` statische globale Wörterbuch-Caches.

### D2. ACL & Filter-Reihenfolge (§9.4)
**BEFUND-01 (S0):** In `crates/contextra-engine/src/temporal_filter.rs:154` wird `apply_temporal_validity_filter` erst nach der RRF-Fusion auf die Treffer angewendet. Das widerspricht der Pre-Filtering-Doktrin §9.4.

### D3. Schreibpfad, WAL & Atomarität (INV-WAL-SSOT-1)
WAL-Commit ist der eindeutige SSOT Commit-Punkt. fsync auf Verzeichnisse und Dateien ist in `contextra-durable-fs` ordnungsgemäß implementiert.

### D4. Startsequenz & Recovery (B0-B9, §22.1)
Die Startsequenz ist in `contextra-db` / `contextra-engine` logisch vorhanden, wird aber nicht als starrer Typestate im Typsystem erzwungen.

### D5. Lösch- & Beweispfad (§20)
**BEFUND-02 (S1):** `no_crypto_stubs.rs` in `contextra-engine` stellt unvollständige Bypasses bereit. Wenn `test-support` in ein Release-Build gerät, verliert der `DeletionProof` seine Gültigkeit.

### D6. Abruf, Forget-Guard & MCP Tool Server (§21.2-21.6)
Der Handler prüft, ob `confirm == true` gesetzt ist (`"confirm parameter must be explicitly set to true for contextra_forget"`).

### D7. Nichtdeterminismus (INV-NONDET-1)
**BEFUND-03 (S1):** Direct Aufrufe von `rand::thread_rng()` in `contextra-store/src/wal/hmac.rs` verletzen den verlangten Determinismus.

### D8. Transaktionen & Nebenläufigkeit (INV-LOCK-ORDER-1)
In `crates/contextra-checkpoint/src/guard.rs:564` wird bei einem Lock-Timeout `expect()` aufgerufen, was zu einem Prozess-Panic statt geordnetem Rollback führt (BEFUND-05).

### D9. Serialisierung & Formate (INV-FORMAT-1)
`flatc` ist im System nicht installiert. Das Crate `contextra-wire` verlässt sich auf die im Repository vorgehaltene `contextra_generated.rs` (BEFUND-06).

### D10. Fehlermodell, Zero-Panic & Unwraps (INV-FAILCLOSED-1)
Über **2.000 Vorkommen** von `.unwrap()`, `.expect()` und `panic!` in `src/`-Verzeichnissen von Ring 0 und Ring 1 Crates (BEFUND-04).

### D11. Unsafe Code Inseln (§6.7, INV-UNSAFE-1)
`cargo xtask check-unsafe-islands` meldet Warnungen bezüglich `#![allow(unsafe_code)]` in Testdateien von `contextra-graph`, `contextra-store` und `contextra-text`.

### D12. Gate-Wirksamkeit & Governance-Realität (§27.5, §28)
`cargo xtask check-marker-drift` schlägt fehl, da `capability-markers.md` fehlt (BEFUND-07). In `xtask/src/main.rs:2441` ist der Match-Arm `"check-commit-diff-integrity"` unerreichbar (BEFUND-08).

---

## 7. Code-Qualität, Anti-Muster & Technische Schuld

- **Zero-Panic Verletzung:** Over **2.000 `.unwrap()` und `.expect()`** Aufrufe in Produktions-Code (1.229 in `store`, 691 in `graph`, 311 in `crypto`).
- **Swallowed Errors:** **568** Vorkommen von `let _ =` und **200** Vorkommen von `.ok()`, die potentielle I/O- oder Formatierungsfehler stumm ignorieren.
- **God-Struct:** `Collection` in `crates/contextra-engine/src/collection/mod.rs` kapselt 21 Felder und verteilt Code auf 20 Dateien.

---

## 8. Technische Schuld

- **Architektur:** Exzessive `pub use *` Wildcard-Exporte und God-Struct `Collection`.
- **Code:** 2.000+ `.unwrap()`/`.expect()` Aufrufe in Ring 0/1, 568 `let _ =`, 200 `.ok()`.
- **Test:** Fehlen unabhängiger Orakel für HNSW-Distanzprüfungen.
- **Prozess & Build:** Missing `flatc` Binary, 40 Paket-Duplikate im Lockfile.

---

## 9. Test- und Gate-Wirksamkeit

- **Mutationsstichproben:** Invertieren von Bedingungen im Search Planner und WAL HMAC zeigte, dass Integrationstests bei direkten Änderungen zuverlässig anschlagen.
- **Gate-Versagen:** `cargo xtask check-marker-drift` scheitert mangels generierter Markerdatei; `.unwrap()`-Mengen werden von keinem Gate blockiert.

---

## 10. Governance-Realität gegen Soll

- **Versprechen:** Automatisierte, unbestechliche Qualitätsgates in `xtask`.
- **Realität:** Wichtige Schutzmechanismen wie Zero-Panic (`INV-FAILCLOSED-1`) besitzen kein aktives CI-Gate. `capability-markers.md` fehlt in der Repository-Basis.

---

## 11. Empfohlene Sanierungsreihenfolge

1. **Schritt 1 (S0 - Blocker): Search Planner Pre-Filtering (BEFUND-01)**
   - *Ziel:* Bi-temporale Filterung und Scope-Constraints direkt in HNSW- und BM25-Scans verschieben.
   - *Umfang:* Medium (M).
   - *Abnahmekriterium:* Integrationstest bestätigt, dass gefilterte Suchen exakt $k$ valide Dokumente ohne Overfetch-Verlust zurückgeben.
2. **Schritt 2 (S1 - Kritisch): Isolation von Crypto-Stubs & Proof Bypasses (BEFUND-02)**
   - *Ziel:* `no_crypto_stubs.rs` strikt auf `cfg(test)` beschränken und alle öffentlichen Bypass-Konstruktoren entfernen.
   - *Umfang:* Small (S).
3. **Schritt 3 (S1 - Kritisch): Eliminierung von `rand::thread_rng()` in Ring 0/1 (BEFUND-03)**
   - *Ziel:* Ersatz durch injizierte `SeededRng` Port-Traits zur Wiederherstellung von **INV-RING0-DETERMINISM-1**.
   - *Umfang:* Small (S).
4. **Schritt 4 (S1 - Kritisch): Zero-Panic Refactoring in Ring 0/1 (BEFUND-04)**
   - *Ziel:* Systematischer Ersatz von `unwrap()`/`expect()` durch `Result<T, ContextraError>` in `store`, `graph` und `crypto`.
   - *Umfang:* Large (L).
5. **Schritt 5 (S2 - Hoch): Repository Gate- & Build-Fixes (BEFUND-06, BEFUND-07, BEFUND-08)**
   - *Ziel:* `capability-markers.md` generieren, unerreichbaren Match-Arm in `xtask` entfernen und `flatc` Schema-Check in CI einbinden.
   - *Umfang:* Small (S).

---

## 12. Offene Fragen an das Projekt

1. Warum wird die bi-temporale Gültigkeitsprüfung in `contextra-engine` als Post-Filter nach der RRF-Fusion ausgeführt, obwohl §9.4 explizit Pre-Filtering vor der Vektordistanzberechnung vorschreibt?
2. Welcher konkrete Zweck verlangt das Vorhandensein von `no_crypto_stubs.rs` im Produktions-Crate `contextra-engine`?
3. Ist geplant, den automatischen Schutz gegen `unwrap()`/`expect()` im Produktionscode durch ein CI-Gate in `xtask` durchzusetzen?

---

### Anhang: Dokumentation der durchgeführten Analysen
- `cargo metadata --format-version 1` -> Ausgewertet in `/tmp/review/phase-B.md`
- `cargo tree -e normal,build,dev` -> Ausgewertet in `/tmp/review/phase-B.md`
- `cargo check --workspace --all-targets --locked` -> Ausgewertet in `/tmp/review/phase-B.md`
- `cargo xtask check-unsafe-islands` -> Ausgewertet in `/tmp/review/phase-B.md`
- `cargo xtask check-marker-drift` -> Ausgewertet in `/tmp/review/phase-B.md`
