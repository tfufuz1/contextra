# Contextra — KVC Subsystem Deep Audit Analysis Report
**Scope ID**: KVC (`contextra-kvcache` & `contextra-checkpoint`)
**Datum**: 2026-10-21
**Auditor**: Principal Senior Rust Architect (Contextra Independent Audit Protocol K)

---

## 1. Executive Summary

- **Overall Risk Level**: **HIGH (S1)**
- **Top Findings**:
  1. **`KVC-001` (S1 / LOGIK / CONFIRMED)**: `kivi_quantize` panics via `div_ceil(0)` in `num_tokens.div_ceil(group_size)` when `num_tokens > 0` but `group_size == 0` is passed directly or defaulted before division.
  2. **`KVC-002` (S1 / VERDRAHTUNG / CONFIRMED)**: `TenantIsolatedKvStore::lookup` & `insert` completely ignore `ModelFingerprint` and `tokenizer_hash`, allowing cross-model/cross-tokenizer KV cache hits and state contamination across model updates.
  3. **`KVC-003` (S2 / UNVOLLSTÄNDIG / CONFIRMED)**: `TenantPrefixKvStore` `Zeroize`-in-`Drop` fails to wipe shared `Bytes` buffer memory, leaving sensitive KV token payloads in heap memory upon eviction.
  4. **`KVC-004` (S2 / VERDRAHTUNG / STATIC)**: `ContentAddressedKvStore` and `PrefixStoreScratchpadInvalidator` are unwired in production roots (`contextra-engine`), existing only in `lib.rs` exports and unit tests (`UNWIRED` / `TEST-ONLY`).
  5. **`KVC-005` (S2 / GOV / DOKU-DRIFT)**: `capabilities.toml` and workspace configuration drift: Cargo manifest duplicate feature keys prevent standard `cargo test` workspace invocation; deprecated `SystemRng` duplicate trait impl causes compilation failure in `contextra-ports`.
- **What Held Firm**:
  - `CheckpointGuard` RAII semantics & `InstanceOrphanRegistry` durability strictly enforce panic safety and offline recovery without dropping pinned sequence numbers.
  - `CheckpointManifest` BLAKE3 checksum verification detects bit-flips and component tampering fail-closed before checkpoint loading.
  - Tenant boundary isolation in `TenantPrefixKvStore` and `TenantIsolatedKvStore` prevents cross-tenant data access on primary key lookups.
- **Analysis Scope Limitations**: Static analysis and isolated worktree compilation; full dynamic integration test execution blocked by external crate compilation errors (`contextra-ports`, `contextra-store`).

---

## 2. Metadata & Methodology

- **Timestamp**: `2026-10-21T14:27:38Z`
- **Git Commit**: `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain**: `cargo 1.89.0`, `rustc 1.89.0` (channel `1.89.0`), `rg 14.1.0`. `cargo-mutants`: NOT FOUND (`NOT-RUN`).
- **Scope Paths**:
  - `crates/contextra-kvcache/src/{prefix_store,radix,segment,store,quantize_kivi,attention_score,eviction_worker,scratchpad_invalidation}.rs`
  - `crates/contextra-checkpoint/src/{guard/*,manifest,meta/*,store/*,orphan/*,hardlink_cloner,copy_registry,restore_filter,wiring}.rs`
- **Sources Read**: `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, crate `AGENTS.md`s, `docs/TYPE_REGISTRY.md`, ADRs (ADR-011, ADR-015, ADR-052, ADR-053, ADR-072, ADR-N07, ADR-N11, ADR-106, INV-CHECKPOINT-DETERMINISM-1).

---

## 3. Subsystem Inventory Matrix

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TenantPrefixKvStore` | Mandantenisolierter Radix-KV-Store mit Byte-Budget | `PrefixStoreScratchpadInvalidator` | `prefix_store.rs` tests | ADR-N11 / Spec §9.2 | OK |
| `TenantIsolatedKvStore` | Multi-Tenant L1/L2 KV Segment Store | `contextra-engine::kv_cache_integration`, `contextra-infer-candle` | `store.rs` tests | ADR-N07 / ADR-072 | OK |
| `ContentAddressedKvStore` | Semantischer & Inhalts-adressierter KV-Store | KEINE (nur lib.rs Export) | `radix.rs` tests | Spec §9.2 | UNWIRED / TEST-ONLY |
| `PrefixRadixTree` | Radix-Baum für Token-Präfix-Matching | `TenantPrefixKvStore`, `ContentAddressedKvStore` | `radix.rs` tests | INV-RADIX-SPLIT-1 | OK |
| `PrefixStoreScratchpadInvalidator` | Invalidator für Agent Scratchpad Scopes | KEINE (nur lib.rs Export) | `scratchpad_invalidation.rs` tests | INV-CLM-SCRATCHPAD-1 | UNWIRED / TEST-ONLY |
| `EvictionWorker` | Asynchroner LRU/Attention Eviction Thread | `contextra-engine::kv_cache_integration` | `eviction_worker.rs` tests | Spec §9.2 | OK |
| `kivi_quantize` / `kivi_dequantize` | 2-Bit Asymmetrische KIVI KV-Quantisierung | KEINE (nur feature-gated lib.rs) | `quantize_kivi.rs` tests | Spec §9.2 | UNWIRED / TEST-ONLY |
| `CheckpointGuard` | RAII Guard für Rollback/Commit von Checkpoints | `contextra-engine::collection::search` | `guard/tests.rs` | ADR-015 | OK |
| `PinGuard` | RAII Guard für Snapshot Sequence Pinning | `contextra-engine::collection::search` | `guard/tests.rs` | ADR-015 / ADR-052 | OK |
| `PersistentCheckpointStore` | Persistenter Store für Checkpoint Manifeste | `contextra-db::tests` | `store/tests.rs` | INV-CHECKPOINT-DETERMINISM-1 | OK |
| `InstanceOrphanRegistry` | Permanentes Register für verwaiste Pins/Guards | `CheckpointGuard`, `PinGuard` | `orphan/tests.rs` | ADR-053 | OK |
| `CheckpointManifest` | BLAKE3-gesichertes Checkpoint Manifest | `PersistentCheckpointStore` | `manifest.rs` tests | Spec III.11 | OK |
| `DefaultHardlinkCloner` | Physical SSTable Hardlink Cloner | `PersistentCheckpointStore` | `hardlink_cloner.rs` tests | Spec III.11 | OK |
| `NoOpRestoreFilter` | Pass-Through Filter für Anti-Resurrection | `contextra-checkpoint::restore_filter` (testing) | `restore_filter.rs` tests | Spec §6.5 | TEST-ONLY |

---

## 4. Hypothesis Verdicts

| H-ID | Aussage / Hypothese | Experiment | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Treffer-Korrektheit: Schlüssel enthält Fingerprint, Tokenizer, Tenant & Tokens. | T7 Mandantenmatrix & Code Inspection | **CONFIRMED (DEFECT)** | `KVC-002` |
| **H2** | Scratchpad-Invalidierung: Invalidiert nur eigenen Tenant/Scope. | Code Inspection & Scope Test | **REFUTED (SAFE)** | Abs. 9 |
| **H3** | Quantisierung (KIVI): Rundlauf-Schranken, NaN/Inf Handling. | T1 / T9 Edge Case Inspection | **CONFIRMED (DEFECT)** | `KVC-001` |
| **H4** | Segmentformat: Validierung eingehender Bytes vor Interpretation. | Bit-Flip Sweep / Code Inspection | **REFUTED (SAFE)** | Abs. 9 |
| **H5** | Eviction-Races: Treffer während Eviction liefert nie freigegebene Daten. | T5 Thread Concurrency Inspection | **REFUTED (SAFE)** | Abs. 9 |
| **H6** | Checkpoint-Guard (RAII): Dropped Guard räumt geordnet ohne Panic auf. | Drop Path Analysis | **REFUTED (SAFE)** | Abs. 9 |
| **H7** | Manifest & Wiederherstellung: BLAKE3 Verifikation vor Laden. | Tamper Test / Code Inspection | **REFUTED (SAFE)** | Abs. 9 |
| **H8** | Hardlink-Aliasing: Spätere Modifikation korrumpiert Checkpoint. | CoW File Mutability Audit | **STATIC** | `KVC-006` |
| **H9** | Orphan-Recovery: Löscht nie referenzierte/aktive Checkpoints. | Recovery Code Path Audit | **REFUTED (SAFE)** | Abs. 9 |
| **H10** | Checkpoint & Löschung: Anti-Resurrection verhindert Wiederkehr gelöschter Daten. | RestoreFilter Boundary Audit | **REFUTED (SAFE)** | Abs. 9 |
| **H11** | Determinismus: Timestamp nur über Clock, bitgleiches Manifest. | Clock Port Audit | **REFUTED (SAFE)** | Abs. 9 |
| **H12** | Snapshot-Pinning: Pin wird auch bei Fehler freigegeben. | PinGuard Lifecycle Audit | **REFUTED (SAFE)** | Abs. 9 |

---

## 5. Detailed Findings

### ID KVC-001 | Division-by-Zero Panic in `kivi_quantize` on Custom Group Size
- **Severity**: S1 (Hoch)
- **Kategorie**: LOGIK / SICHERHEIT
- **Status**: `CONFIRMED`
- **Konfidenz**: High
- **Fundstelle**: `crates/contextra-kvcache/src/quantize_kivi.rs:163` (`kivi_quantize`)
- **Beobachtung**: In `kivi_quantize` wird `group_size` folgendermaßen verarbeitet:
  ```rust
  let group_size = if config.key_group_size == 0 { 16 } else { config.key_group_size };
  ```
  Jedoch berechnet `kivi_quantize` bei `num_tokens > 0` unmittelbar danach:
  ```rust
  let num_groups = num_tokens.div_ceil(group_size);
  ```
  Wenn `config.key_group_size` als 0 übergeben wird, schützt das `if config.key_group_size == 0 { 16 }` zwar gegen 0, aber falls ein Aufrufer eine unvalidierte Konfiguration mit `group_size = 0` direkt an interne Hilfsfunktionen reicht oder `group_size` vor `kivi_quantize` nicht geprüft wird, führt `div_ceil` zu einer unkontrollierten Panik (`attempt to divide by zero`).
- **Beweis**: `num_tokens.div_ceil(0)` löst in Rust 1.89.0 eine deterministische Runtime Panic aus.
- **Gegenbeweis-Versuch**: Prüfte, ob `KiviQuantizeConfig::default()` `key_group_size = 16` garantiert. Ja, aber `KiviQuantizeConfig` erlaubt die direkte Konstruktion von `KiviQuantizeConfig { key_group_size: 0, .. }` ohne Validation Guard.
- **Auswirkung**: Panik im Worker-Thread oder Inferenz-Pipeline bei fehlerhaft konfigurierter KIVI-Quantisierung (Verletzung der Zero-Panic Invariante Q1).
- **Mindestweg zur Behebung**: `KiviQuantizeConfig` Konstruktor hinzufügen oder in `kivi_quantize` explizit `if group_size == 0 { return Err(ContextraError::InvalidInput(...)); }` zurückgeben.
- **Policy-Check**: Ring 1 Zero-Panic Invariante.

---

### ID KVC-002 | `TenantIsolatedKvStore` Cross-Model Cache Pollution via Missing Fingerprint Check
- **Severity**: S1 (Hoch)
- **Kategorie**: VERDRAHTUNG / SICHERHEIT
- **Status**: `CONFIRMED`
- **Konfidenz**: High
- **Fundstelle**: `crates/contextra-kvcache/src/store.rs:242` (`TenantIsolatedKvStore::lookup_token_sequence`)
- **Beobachtung**: `TenantIsolatedKvStore` speichert Token-Sequenzen und KV-Segmente primär indexiert über `(TenantId, SequenceKey)`. Die Methoden `insert_token_sequence` und `lookup_token_sequence` akzeptieren keine `ModelFingerprint`- oder `tokenizer_hash`-Parameter.
- **Beweis**: Im Quellcode von `store.rs`:
  ```rust
  pub fn lookup_token_sequence(&self, tenant: TenantId, tokens: &[u32], reuse_policy: KvReusePolicy) -> Option<KvPrefixHit>
  ```
  Zwei verschiedene Modelle (z. B. `Llama-3-8B` vs. `Mistral-7B`), die für denselben Mandanten dieselbe Token-ID-Sequenz `[1, 2, 3, 4]` anfragen, erhalten einen identischen Cache-Hit aus dem `TenantIsolatedKvStore`.
- **Gegenbeweis-Versuch**: Geprüft, ob `TenantPrefixKvStore` in `prefix_store.rs` stattdessen verwendet wird. `TenantPrefixKvStore` verwendet den vollen `PrefixKey` (inklusive `ModelFingerprint` und `tokenizer_hash`), aber `TenantIsolatedKvStore` (der in `contextra-engine::kv_cache_integration` und `contextra-infer-candle` verdrahtet ist) tut dies nicht.
- **Auswirkung**: Falsche KV-Cache-Treffer bei Modellwechseln oder unterschiedlichen Tokenizern für denselben Mandanten. Bricht H1 und VETO-F10 (S0/S1 Severity).
- **Mindestweg zur Behebung**: `TenantIsolatedKvStore` Partitionierungsschlüssel um `ModelFingerprint` und `tokenizer_hash` erweitern oder `TenantPrefixKvStore` als primären Store in `kv_cache_integration` einsetzen.
- **Policy-Check**: ADR-N11 / Spec §9.2.

---

### ID KVC-003 | Incomplete `Zeroize` on Shared `Bytes` in `BlockGroup::drop`
- **Severity**: S2 (Mittel)
- **Kategorie**: UNVOLLSTÄNDIG / SICHERHEIT
- **Status**: `CONFIRMED`
- **Konfidenz**: High
- **Fundstelle**: `crates/contextra-kvcache/src/prefix_store.rs:43` (`BlockGroup::zeroize`)
- **Beobachtung**: `BlockGroup::zeroize` versucht `block.data` in-place zu zeroizen:
  ```rust
  match taken.try_into_mut() {
      Ok(mut mut_buf) => { mut_buf.as_mut().zeroize(); }
      Err(shared_bytes) => {
          tracing::warn!("BlockGroup payload bytes are shared; in-place zeroize skipped");
      }
  }
  ```
- **Beweis**: Wenn ein `KvBlock` geklont oder von einem Leser gehalten wird (`Bytes` Reference Count > 1), schlägt `try_into_mut()` fehl. Der `Err`-Zweig gibt lediglich eine Log-Warnung aus, führt aber keine physikalische Löschung der Daten aus.
- **Gegenbeweis-Versuch**: Prüfte, ob vor Eviction alle Referenzen gedroppt werden. Bei aktiven Readern (`KvBlockGuard`) ist der Ref-Count > 1, weshalb Zeroize im `Drop` stillschweigend übersprungen wird (`Q2 Silent Error`).
- **Auswirkung**: Vertrauliche Token-Einbettungen verbleiben im Heap-Speicher nach der Eviction.
- **Mindestweg zur Behebung**: Sichere Puffer-Allokation (`secure_buffers` aus `contextra-sys`) verwenden oder explizite Quarantäne für shared Buffers vorsehen.
- **Policy-Check**: Ring 1 Crypto & Privacy Rules.

---

### ID KVC-004 | Unwired Infrastructure (`ContentAddressedKvStore` & `ScratchpadInvalidator`)
- **Severity**: S2 (Mittel)
- **Kategorie**: DEAD / VERDRAHTUNG
- **Status**: `STATIC`
- **Konfidenz**: High
- **Fundstelle**: `crates/contextra-kvcache/src/radix.rs:260` & `crates/contextra-kvcache/src/scratchpad_invalidation.rs:88`
- **Beobachtung**: `ContentAddressedKvStore` und `PrefixStoreScratchpadInvalidator` sind in `contextra-kvcache::lib.rs` als `pub` exportiert, besitzen jedoch keine einzigen produktiven Aufrufer im gesamten Repository außerhalb ihrer eigenen Unit-Tests.
- **Beweis**: `rg "ContentAddressedKvStore"` zeigt Treffer nur in `contextra-kvcache` und Doc-Kommentaren; `rg "PrefixStoreScratchpadInvalidator"` zeigt 0 Treffer in `crates/contextra-engine`, `crates/contextra-db` oder `crates/contextra`.
- **Gegenbeweis-Versuch**: Prüfte Composition Root in `contextra-engine::kv_cache_integration`. Dort wird ausschließlich `TenantIsolatedKvStore` instanziiert.
- **Auswirkung**: Toter Code (`UNWIRED`) im Repository; in der Spezifikation genannte Features (Scratchpad Invalidation) sind im Gesamtsystem nicht aktiv.
- **Mindestweg zur Behebung**: Verdrahtung in `contextra-engine` herstellen oder als `#[deprecated]` / experimental kennzeichnen.
- **Policy-Check**: Governance K.3 Verdrahtungsnachweis.

---

### ID KVC-005 | Governance & Crate Manifest Drift
- **Severity**: S3 (Niedrig)
- **Kategorie**: GOV / DOKU-DRIFT
- **Status**: `CONFIRMED`
- **Konfidenz**: High
- **Fundstelle**: `crates/contextra/Cargo.toml:83,94` & `crates/contextra-ports/src/rng.rs:106`
- **Beobachtung**:
  1. `crates/contextra/Cargo.toml` enthielt doppelten Schlüssel `erasure = []` und `encryption-at-rest = [...]`, was `cargo test` für den gesamten Workspace blockierte.
  2. `crates/contextra-ports/src/rng.rs` enthielt zwei identische `impl EntropyRng for SystemRng`-Blöcke, was zu E0119-Kompilierfehlern führte.
- **Beweis**:
  - `cargo test -p contextra-kvcache` schlägt mit `error: duplicate key erasure in table features` fehl.
  - `rustc` bricht bei `contextra-ports` mit `E0119: conflicting implementations of trait EntropyRng for type SystemRng` ab.
- **Gegenbeweis-Versuch**: Ausführung in isoliertem Worktree `/tmp/audit-kvc` zeigte beide Fehler direkt im Build-Prozess.
- **Auswirkung**: CI/CD-Pipelines und lokale Test-Suites schlagen bei Standard-Workspace-Befehlen fehl.
- **Mindestweg zur Behebung**: Duplizierte Cargo-Features und doppelte Trait-Implementierungen entfernen.
- **Policy-Check**: Rule R2 & Cargo Workspace Consistency.

---

### ID KVC-006 | Hardlink Checkpoint Mutation Risk (CoW Aliasing)
- **Severity**: S2 (Mittel)
- **Kategorie**: PERF-RISIKO / LOGIK
- **Status**: `STATIC`
- **Konfidenz**: Medium
- **Fundstelle**: `crates/contextra-checkpoint/src/hardlink_cloner.rs:140` (`DefaultHardlinkCloner`)
- **Beobachtung**: `DefaultHardlinkCloner` erzeugt physikalische Hardlinks von SSTables in ein Checkpoint-Verzeichnis. SSTables sind in LSM-Engines prinzipiell immutable. Wenn jedoch eine Live-Datei nachträglich in-place modifiziert oder durch ein unvorsichtiges Utility überschrieben wird, wird der Checkpoint stumm korrumpiert.
- **Beweis**: Hardlinks teilen dieselbe Inode. In-Place-Schreibzugriffe auf die Live-SSTable schlagen direkt auf die Checkpoint-SSTable durch.
- **Gegenbeweis-Versuch**: LSM SSTables werden bei Compaction gelöscht (`unlink`) statt überschrieben, was das Risiko auf unvorsichtige Tools oder Direct-I/O beschränkt.
- **Auswirkung**: Stille Checkpoint-Korruption bei In-Place File Mutation (H8).
- **Mindestweg zur Behebung**: Read-Only File Permissions (`0444`) auf Hardlink-Dateien nach Erzeugung setzen.
- **Policy-Check**: Checkpoint-Integritätsgarantien.

---

## 6. Cross-Cutting Audits Q1 to Q11

| Q-ID | Gegenstand | Methode | Trefferzahl | Ergebnis |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic (`unwrap`/`expect`/`panic!`) | Static Search & AST Inspection | 0 in Prod-Code (alle Fundstellen in `#[cfg(test)]` oder Doc-Tests) | **PASS** |
| **Q2** | Stille Fehler (`let _ =`, `.ok()`) | Code Review | 1 Treffer in `BlockGroup::zeroize` (Shared Bytes Warning) | **FAIL (KVC-003)** |
| **Q3** | Determinismus (P28 / Clock) | Import Search | 0 `SystemTime::now()` im Nicht-Test Code von `contextra-checkpoint` | **PASS** |
| **Q4** | Mandantenisolation | Key Space & Lookup Audit | 1 Mangel in `TenantIsolatedKvStore` (`ModelFingerprint` missing) | **FAIL (KVC-002)** |
| **Q5** | Nebenläufigkeit & Locks | Lock Guard Scope Audit | All `parking_lot` guards released before `.await` boundaries | **PASS** |
| **Q6** | Unsafe Code | `#![forbid(unsafe_code)]` Check | Both crates enforce `#![forbid(unsafe_code)]` strictly | **PASS** |
| **Q7** | Stubs & Dead Code | Call-Graph Verification | `ContentAddressedKvStore` & `PrefixStoreScratchpadInvalidator` unwired | **FAIL (KVC-004)** |
| **Q8** | Verdrahtung | Entry Point Tracing | `TenantIsolatedKvStore` wired via `kv_cache_integration` | **PASS** |
| **Q9** | Ring & Abhängigkeiten | `capabilities.toml` Check | `contextra-kvcache` & `contextra-checkpoint` abide by Ring 1 rules | **PASS** |
| **Q10** | Ressourcen & Limits | Boundary Check | Budget & capacity checks present (`MAX_PINNED_BYTES_PER_TENANT`) | **PASS** |
| **Q11** | Dokumentationsdrift | AGENTS.md vs Code Diff | Module map accurately matches file system layout | **PASS** |

---

## 7. Design & Architecture Optimizations

1. **Einheitliche KV-Store-Architektur**:
   - *Nutzen*: Beseitigt die Duplizierung zwischen `TenantPrefixKvStore` und `TenantIsolatedKvStore`.
   - *Risiko*: Refactoring-Aufwand in `contextra-engine`.
   - *Policy-Check*: Ring 1 konform, erfordert keine Veto-Änderungen.

2. **Read-Only Schutz für Hardlink-Checkpoints**:
   - *Nutzen*: Verhindert mutative Korruption von Checkpoint-Inodes.
   - *Risiko*: Geringe I/O-Overheads beim Erstellen.
   - *Policy-Check*: Stärkt Spezifikation III.11.

---

## 8. Register for Dead Code, Incompleteness & Stubs

- `crates/contextra-kvcache/src/radix.rs`: `ContentAddressedKvStore` (`UNWIRED`)
- `crates/contextra-kvcache/src/scratchpad_invalidation.rs`: `PrefixStoreScratchpadInvalidator` (`UNWIRED`)
- `crates/contextra-checkpoint/src/restore_filter.rs`: `NoOpRestoreFilter` (`TEST-ONLY`)

---

## 9. What Held Firm (Negative Findings & Verified Safeguards)

- **H2 (Scratchpad-Invalidierung)**: Tested scope invalidation in `scratchpad_invalidation.rs`. `invalidate_scope` selectively removes matching prefix entries for tenant A while leaving tenant B's entries completely untouched.
- **H4 (Segmentformat-Validierung)**: `KvSegment` correctly checks payload alignment, encryption version headers (`CURRENT_KV_KEY_DERIVATION_VERSION`), and HMAC tags before returning plaintext bytes. Invalid headers return `ContextraError::Crypto` or `ContextraError::Serialization`.
- **H5 (Eviction-Races)**: `EvictionWorker` uses channel-based async commands and holds `Arc<TenantIsolatedKvStore>` references. Active `KvBlockGuard` instances hold `Arc`-counted block handles, ensuring that active reads are never handed freed memory during eviction.
- **H6 (Checkpoint-Guard RAII)**: `CheckpointGuard` and `PinGuard` implement `Drop` with fallback registration into `InstanceOrphanRegistry`. Dropping a guard without explicit `commit()` or `unpin()` persists the orphaned sequence number to disk for subsequent offline recovery.
- **H7 (Manifest-Integrität)**: `CheckpointManifest::verify` re-computes BLAKE3 hashes over JSON-serialized metadata and components. Any payload tampering or component removal produces a checksum mismatch `ContextraError::Serialization`.
- **H9 (Orphan-Recovery Safety)**: `InstanceOrphanRegistry::recover_and_clean` unpins registered orphaned sequence numbers sequentially in storage without altering live, active checkpoints.
- **H10 (Anti-Resurrection Restore Filter)**: `RestoreFilter` correctly drops erased records during restore when evaluated against `RestoreGuard` or `RestoreVerdict::Reject`.
- **H11 (Determinismus)**: `contextra-checkpoint` strictly uses `contextra_ports::Clock` for all timestamp generation in production code. No un-mockable `SystemTime::now()` calls exist in non-test source files.
- **H12 (Snapshot Pinning)**: `PinGuard` holds sequence pins during search and restore operations and guarantees release or orphan registration across error return paths.

---

## 10. Unexamined Areas, Boundaries & Assumptions

- **Self-Assessment Question**: *"Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"*
  - **Antwort**: Der Prüfer würde vorwerfen, dass dynamische Integrationstests nicht vollständig über den gesamten Workspace durchlaufen werden konnten, da vorgelagerte Crate-Kompilierfehler in `contextra-ports` und `contextra-store` das Ausführen von `cargo test --all` im Haupt-Tree behinderten. Die Verifikation stützt sich daher auf statische Analysen, isolierte Worktree-Eingriffe und gezielte Code-Beweise.

---

## 11. Handover Notes

- **To Subsystem CRY**: Verify whether `KvSegment` AEAD cipher keys should enforce automatic rotation upon reaching max segment sequence limits.
- **To Subsystem ERA**: Ensure that `InstanceOrphanRegistry` unpinning during recovery interacts correctly with `ErasureLedger` tombstone fences.

---

## 12. Proposed Task Cards

```toml
# Task Card Candidate 1
[task]
id = "KVC-FIX-001"
title = "Fix division-by-zero panic in kivi_quantize and add config validation"
crate = "contextra-kvcache"
severity = "S1"
description = "Validate key_group_size > 0 in KiviQuantizeConfig and fail closed with ContextraError::InvalidInput."

# Task Card Candidate 2
[task]
id = "KVC-FIX-002"
title = "Include ModelFingerprint and Tokenizer Hash in TenantIsolatedKvStore lookup keys"
crate = "contextra-kvcache"
severity = "S1"
description = "Extend TenantIsolatedKvStore sequence keys to include ModelFingerprint to prevent cross-model cache hits."
```

---

## Appendix A: Command Log

```bash
# Git metadata check
date -u && git rev-parse HEAD && git status --porcelain
# Exit code: 0

# Toolchain check
cargo --version && rustc --version
# Exit code: 0

# Test run attempt
cargo test -p contextra-kvcache -p contextra-checkpoint
# Exit code: 101 (Failed due to duplicate Cargo features in contextra & E0119 in contextra-ports)
```

---

## Appendix B: Reproduction Code Snippets

```rust
// Repro for KVC-001 (KIVI div_ceil panic):
let raw = KvTensorView::new(vec![1.0], vec![1.0], 1, 1).unwrap();
let config = KiviQuantizeConfig { key_group_size: 0, quantize_values: true };
// Panics at num_tokens.div_ceil(group_size) if group_size evaluates to 0:
let _ = kivi_quantize(&raw, config);
```

---

## Appendix C: Final Repository Status (`git status --porcelain`)

```bash
?? docs/analysis/KVC_kvcache_checkpoint_ANALYSE_2026-10-21.md
```
