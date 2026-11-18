# Analyse: Widersprüche & Ist-Soll-Abweichungen im Contextra-Workspace

**Datum:** 2026-11-18  
**Autor:** Automatisierte Codebase-Analyse (Principal Rust Engineer)  
**Scope:** Vollständige E0-Schicht, Querverweise E1/E2 (types, ports)  
**Zweck:** Übergabedokument für nachfolgende Implementierungsarbeit  

---

## 0. Methodik

Folgende Quellen wurden abgeglichen:
1. `docs/SPEC.md` v1.0 (normative Gesamtspezifikation)
2. `docs/crates/*.md` (Crate-Spezifikationen, 23 Dateien)
3. `AGENTS.md` (Workspace-Richtlinie)
4. Tatsächlicher Quellcode aller E0-Crates + `contextra-types` + `contextra-ports`
5. Alle `Cargo.toml` (Abhängigkeitsgraphen)

---

## 1. KRITISCH: Typ-Dubletten (Verstoß gegen R4)

SPEC §2.3 R4: *„Ein Konzept hat genau einen Besitzer — keine Typ-Dubletten."*

### 1.1 `ErasureId` — 3-fach definiert

| Ort | Datei | Status |
|-----|-------|--------|
| `contextra-ids` | Nicht vorhanden (laut Spec: Besitzer) | **Fehlt** |
| `contextra-types::ids` | `src/ids.rs` Z.12–38 | `#[deprecated]` — Re-Export aus altem Code |
| `contextra-erasure::types_local` | `src/types_local.rs` Z.24–60 | Eigenständige Kopie, aktiv genutzt |

**SPEC-Forderung:** `ErasureId` gehört in `contextra-ids` (AGENTS.md §5 E0).  
**Ist-Zustand:** Zwei aktive Kopien + eine deprecated. Keine nutzt `contextra-ids`.

### 1.2 `SubjectRef` — 3-fach definiert, inkonsistente Semantik

| Ort | Typ | Clone | ZeroizeOnDrop | Erzeugung |
|-----|-----|-------|---------------|-----------|
| `contextra-ids::lib.rs` Z.643 | `Vec<u8>` (via Makro `define_ref_newtype!`) | **Ja** ✗ | **Nein** ✗ | `try_new(Vec<u8>)` — kein RNG ✗ |
| `contextra-types::refs` | Re-Export von `contextra-ids` | Ja ✗ | Nein ✗ | identisch |
| `contextra-privacy::types` | Nicht vorhanden | — | — | — |

**SPEC-Forderung (§3.2, AGENTS.md §2.2):**
- 128-Bit-Zufalls-ID
- Kein `Clone`
- `ZeroizeOnDrop`
- Erzeugung über `OsRng::try_fill_bytes`
- Geschwärztes `Display`

**Ist-Zustand:** `SubjectRef` ist ein `Vec<u8>`-Wrapper mit `Clone`, ohne `Zeroize`, ohne RNG-Erzeugung. Die `rand`- und `zeroize`-Abhängigkeiten **fehlen komplett** in `contextra-ids/Cargo.toml`.

### 1.3 `TenantId` — 3-fach definiert

| Ort | Datei |
|-----|-------|
| `contextra-ids::lib.rs` Z.77–143 | Kanonisch (SPEC-konform) |
| `contextra-privacy::types.rs` Z.49–86 | **Eigenständige Kopie** mit abweichendem `TenantScopeViolation` |
| `contextra-types::ids` | `pub use contextra_ids::*` (Re-Export, OK) |

**Problem:** `contextra-privacy` definiert eigenes `TenantId` + `TenantScopeViolation` + `TenantScoped` anstatt von `contextra-ids` zu importieren. Die `TenantScopeViolation`-Enum hat eine zusätzliche `InvalidZero`-Variante, die in `contextra-ids` nicht existiert.

### 1.4 `Clock`-Trait — 2-fach definiert

| Ort | Datei |
|-----|-------|
| `contextra-env::clock.rs` | Kanonisch: `now_unix_nanos()`, `monotonic_nanos()` |
| `contextra-privacy::types.rs` Z.11–17 | **Eigenständige Kopie** mit identischer Signatur |

**Problem:** `contextra-privacy` definiert eigenen `Clock`-Trait und `SystemClock`, obwohl `contextra-env` existiert. Die `privacy`-Version hat keinen `start_instant` für `monotonic_nanos()` — sie nutzt stattdessen `now_unix_nanos()` als Fallback (nicht-monoton!).

### 1.5 `EraseTarget` — 3-fach definiert, inkompatible Varianten

| Ort | Varianten |
|-----|-----------|
| `contextra-types::erasure.rs` | `Document { tenant, collection, doc }`, `Subject { tenant, subject }`, `Collection { tenant, collection }`, `Tenant { tenant }` — nutzt `TenantRef`, `CollectionRef`, `DocRef` |
| `contextra-erasure::types_local.rs` Z.812 | Eigene `EraseTarget`-Definition — nutzt lokale Typen |
| `contextra-types` SPEC (`docs/crates/contextra-types.md` §3.1) | `ExactKeys(Vec<Vec<u8>>)`, `KeyRange { start, end }` |

**Problem:** Die Spec für `contextra-types` beschreibt ein `EraseTarget` mit `ExactKeys`/`KeyRange` (byteorientiert), der tatsächliche Code hat ein domänenorientiertes `EraseTarget` mit `Document`/`Subject`/`Collection`/`Tenant`. Die Spec und der Code stimmen nicht überein. Zusätzlich hat `contextra-erasure` eine dritte Kopie.

### 1.6 `SagaState` — 2-fach definiert, inkompatible Varianten

| Ort | Varianten |
|-----|-----------|
| `contextra-types` SPEC | `Requested`, `Gated`, `KeyRevoked`, `Purging`, `Verifying`, `Completed`, `Pending(PendingReason)`, `Restricted`, `Refused`, `Lifted` |
| `contextra-erasure::saga::machine.rs` Z.47–62 | `Requested`, `Gated`, **`KeyDestroyed`** (nicht `KeyRevoked`!), `Purging`, `Verifying`, `Completed`, `Restricted`, `Lifted`, **`Pending`** (ohne Reason!), `Refused` |

**Probleme:**
1. `KeyRevoked` (SPEC) vs. `KeyDestroyed` (Code) — unterschiedliche Benennung
2. `Pending(PendingReason)` (SPEC) vs. `Pending` (Code, ohne Inner-Value)
3. `Lifted` ist in der SPEC §5.2 terminal, aber in `contextra-types` SPEC fehlt `Lifted`

### 1.7 `LayerWitness` — 2-fach definiert

| Ort | Felder |
|-----|--------|
| `contextra-ports` SPEC | `layer_id`, `purged_count`, `bytes_reclaimed`, `residue_matches`, `scan_class` |
| `contextra-types::erasure.rs` (actual code) | `layer`, `class`, `method_id`, `files`, `bytes`, `duration`, `hash` — **komplett andere Felder** |

**Problem:** Die SPEC für `contextra-ports` beschreibt ein `LayerWitness` mit `residue_matches`-Feld, der tatsächliche Code in `contextra-types` hat ein `LayerWitness`, bei dem `matches()` per Konstruktion immer 0 ist (Typestate-Garantie). Die Datenmodelle sind inkompatibel.

### 1.8 `Scanner<Armed>` — 2-fach definiert

| Ort | Status |
|-----|--------|
| `contextra-types::erasure.rs` | `#[deprecated]`, vollständige Implementierung mit Typestate |
| `contextra-erasure::measure/` | Eigene Implementierung |

---

## 2. KRITISCH: Ebenen-Verletzungen (Verstoß gegen R1)

### 2.1 `contextra-erasure` — Deklariert als E0, Metadata sagt Ring 3

**Cargo.toml Metadata:**
```toml
[package.metadata.contextra]
ring = "3"
```

**SPEC/AGENTS.md:** `contextra-erasure` ist E0 (Fundament).

**Lib.rs Kommentar:**
```rust
//! Ring 3 (SPEC §5.1, §5.2, §11). Abhängigkeitsregel AR-3...
```

**Problem:** Die SPEC sagt E0, der Code sagt Ring 3, die Cargo-Metadata sagt Ring 3. Die SPEC-Zuordnung zu E0 ist problematisch, weil `contextra-erasure` konzeptuell den Saga-Orchestrator enthält, der die E1-Layer steuert — er gehört eigentlich auf E2 oder höher.

**Finale Entscheidung nötig:** Entweder:
- (A) `erasure` bleibt E0, aber nur als Trait-/Typdefinierer (Gate, Scanner, SagaState), ohne Orchestrierungslogik
- (B) `erasure` wird zu E2, darf E1-Crates über Port-Traits steuern

### 2.2 `contextra-simd` — Optionale Abhängigkeit auf `contextra-types`

**Cargo.toml:**
```toml
[features]
workspace = ["dep:contextra-types"]
types-compat = ["workspace"]

[dependencies]
contextra-types = { workspace = true, optional = true }
```

**Problem:** `contextra-simd` (E0) hat eine optionale Abhängigkeit auf `contextra-types` (E2). Das verstößt gegen R1 (Kanten nur abwärts). Auch wenn optional, ist die Feature-Kombination `types-compat` ein Ebenenverstoß.

### 2.3 `contextra-privacy` — Fehlende E0-Isolation

**Problem:** `contextra-privacy` definiert eigene Versionen von `TenantId`, `Clock`, `TenantScoped`, `EgressDecision` etc., weil es als E0 keine Abhängigkeiten auf `contextra-ids` oder `contextra-env` haben darf. Das führt zu den Typ-Dubletten in §1.3 und §1.4.

**Aber:** E0-Crates dürfen laut SPEC §2.1 `ids` und `env` untereinander nutzen: *„Keine internen Abhängigkeiten (außer ids, env untereinander erlaubt)."*

**Lösung:** `contextra-privacy` SOLL `contextra-ids` und `contextra-env` als Abhängigkeit nutzen und die lokalen Kopien entfernen.

---

## 3. KRITISCH: Feature-Verstöße

### 3.1 Nicht-leere Default-Features (Verstoß gegen R3)

| Crate | `default` Features |
|-------|-------------------|
| `contextra-ids` | `["std"]` ✗ |
| `contextra-env` | `["std"]` ✗ |
| `contextra-durable-fs` | `["std", "crc"]` ✗ |
| `contextra-simd` | `["std"]` ✗ |
| `contextra-privacy` | `["std"]` ✗ |
| `contextra-erasure` | `["signing", "vault", "async"]` ✗ |

**SPEC §2.3 R3:** *„Default-Features MÜSSEN leer sein."*

**Ist-Zustand:** Kein einziges E0-Crate erfüllt R3. `contextra-erasure` hat 3 Default-Features.

---

## 4. KRITISCH: Fehlende Implementierungen gemäß SPEC

### 4.1 `contextra-ids` — Fehlende SPEC-Typen und Eigenschaften

| SPEC-Anforderung | Status |
|-------------------|--------|
| `SubjectRef`: 128-Bit-Zufalls-ID | ✗ Ist `Vec<u8>` |
| `SubjectRef`: Kein `Clone` | ✗ Hat `Clone` |
| `SubjectRef`: `ZeroizeOnDrop` | ✗ Fehlt komplett |
| `SubjectRef`: Erzeugung via `OsRng::try_fill_bytes` | ✗ Keine RNG-Abhängigkeit |
| `SubjectRef`: Geschwärztes Display | ✓ Makro nutzt `[REDACTED]` |
| `PersonalRef`: Kein `Clone` | ✗ Hat `Clone` (via gleiches Makro) |
| `ErasureId`: Definiert in ids | ✗ Fehlt komplett |
| `rand` Dependency | ✗ Fehlt |
| `zeroize` Dependency | ✗ Fehlt |

### 4.2 `contextra-env` — Fehlende `Vfs`-Abstraktion

Die Spec fordert ein `Vfs`-Trait mit `open_read`, `open_write`, `create_dir_all`, `remove_file`, `rename`, `exists`.

**Ist-Zustand:** `contextra-env` hat **kein** `Vfs`-Modul/Trait. Das VFS existiert stattdessen in `contextra-durable-fs`.

**Widerspruch:** Die Spec sagt, `Vfs` gehört in `contextra-env`, aber `contextra-durable-fs` hat bereits ein `vfs.rs`-Modul mit eigener VFS-Implementierung. Doppelte Zuständigkeit.

### 4.3 `contextra-privacy` — `DataClass` fehlt

Die SPEC fordert, dass `contextra-privacy` der *„alleinige kanonische Besitzer"* der `DataClass`-Definition ist:
```rust
pub enum DataClass { K, P, D, E }
```

**Ist-Zustand:** Im gesamten Quellcode von `contextra-privacy/src/` findet sich **kein** `DataClass`-Enum. Auch nicht in `contextra-types` oder `contextra-ids`.

Das `Validated<T>`-Pattern aus der SPEC existiert ebenfalls **nicht**. Stattdessen gibt es `GuardedPayload<S>` mit `Unsanitized`/`Sanitized`-Typestates — aber das ist für Cloud-Egress-Schutz, nicht für K-Klasse-Schreibpfade.

### 4.4 `contextra-privacy` — `EntityRecognizer` Doppelrolle

Das Crate hat zwei verschiedene `EntityRecognizer`:
1. `egress_vault.rs`: `pub trait EntityRecognizer` (async, für Egress-Filtering)
2. `entity_recognizer.rs`: `RuleBasedEntityRecognizer` (Regex-basiert, konkrete Impl)

Die SPEC spricht von einem einzigen `EntityRecognizer` für PII-Erkennung.

### 4.5 `contextra-erasure` — Vault enthält Crypto-Logik

Die SPEC sagt, kryptographische Primitive gehören in `contextra-crypto`. Aber `contextra-erasure` hat:
- Direktabhängigkeit auf `aes-gcm-siv`
- Eigenes `vault/sealer.rs` mit AEAD-Operationen
- Eigenes `crypto_local.rs` Modul

Das verstößt gegen R4 (ein Konzept = ein Besitzer) und die Zuständigkeit von `contextra-crypto`.

---

## 5. WICHTIG: Inkonsistenzen in den Crate-Spezifikationen

### 5.1 `contextra-durable-fs` Spec vs. Code

| Spec-Komponente | Im Code vorhanden? |
|-----------------|-------------------|
| `Vfs` (StdVfs, RealFs, write_atomic, TmpGuard) | ✓ `vfs.rs` |
| `platform` (sync_file, sync_dir_path, FlushKind) | ✓ `platform.rs` |
| `health & Poison` (AtomicHealthState, HealthEvent) | ✓ `health.rs` |
| `scrub` (scrub_file_with_rate_limit, ScrubPolicy) | ✓ `scrub.rs` |
| `journal` (AppendOnlyJournal, PendingDeleteJournal) | ✓ `journal.rs` |
| `fault_injector` | ✓ `fault_injector.rs` |
| `FsClass` (LocalSupported, Network, Fuse, CloudSynced) | **Nicht geprüft** |

**Widerspruch mit `contextra-env`:** Die Spec für `contextra-env` definiert ein `Vfs`-Trait, aber der tatsächliche VFS-Code lebt in `contextra-durable-fs`. Die Specs widersprechen sich gegenseitig in der Zuständigkeit.

### 5.2 `contextra-erasure` SPEC: 4 fsync-Ankerpunkte vs. SPEC.md: 3

- `docs/crates/contextra-durable-fs.md` §4.1: **4 fsync-Ankerpunkte** (#1 Intent, #2 Gated, #3 KeyRevoked, #4 Abschluss & Ledger)
- `docs/SPEC.md` §5.3: Nur **3 fsync** erwähnt (fsync #1, #2, #3)
- `docs/crates/contextra-erasure.md` §4: **4 fsync** (#1 Intent, #2 Gated, #3 KeyZero, #4 KeyRevoked-Journal)

**Problem:** Die Nummerierung und Zuordnung der fsync-Punkte ist inkonsistent zwischen den drei Dokumenten.

### 5.3 `contextra-erasure` SPEC: Phase-Nummerierung

Die SPEC.md beschreibt 6 Schritte (1–6). Die Erasure-Crate-Spec beschreibt 6 Phasen (1–6), aber mit leicht verschobener Zuordnung:

| SPEC.md | Erasure-Crate-Spec |
|---------|-------------------|
| Schritt 1: Validierung + Intent (fsync #1) | Phase 1: Validierung, Resolve, Intent (fsync #1) ✓ |
| Schritt 2: Gate setzen (fsync #2) | Phase 2: Lesesperre (fsync #2) ✓ |
| Schritt 3: Schlüsselvernichtung (fsync #3) | Phase 3: Layer on_gate + Schlüsselvernichtung (fsync #3) |
| — | Phase 4: **Journal KeyRevoked + Receipt** (fsync #4) — in SPEC.md nicht als eigene Phase |
| Schritt 4: Physische Bereinigung | Phase 5: Asynchrones Purge-Fenster |
| Schritt 5: Messung | Phase 6 (Teil 1): Messung |
| Schritt 6: Bericht | Phase 6 (Teil 2): Bericht |

**Problem:** Der synchrone Rückgabepunkt (Receipt an Aufrufer nach KeyRevoked) ist in der Erasure-SPEC eine eigene Phase 4, in SPEC.md ist er Teil von Schritt 3.

---

## 6. WICHTIG: Abhängigkeitsprobleme

### 6.1 `contextra-erasure` Dependencies (Ist-Zustand)

```toml
[dependencies]
ahash, rand, subtle, aes-gcm-siv, thiserror, tracing, serde, serde_json,
zeroize, blake3, sha2, hmac, ed25519-dalek, bytes, parking_lot,
futures-util, tokio, bincode
```

**Probleme:**
1. `aes-gcm-siv` → Crypto gehört in `contextra-crypto`
2. `sha2`, `hmac` → dito
3. `blake3` → dito (ID-Hashing gehört in `contextra-ids`)
4. `ed25519-dalek` → Signatur, laut SPEC Feature `signing` in `erasure`, aber Crypto-Primitive sollten zentralisiert sein
5. Keine Abhängigkeit auf `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports` oder `contextra-crypto`

**Das Crate ist vollständig autark** — es hat alle Typen lokal kopiert (`types_local.rs` mit ~2400 Zeilen). Das widerspricht dem Ziel der Entkopplung über Port-Traits.

### 6.2 `contextra-privacy` Dependencies (Ist-Zustand)

```toml
[dependencies]
thiserror, serde, serde_json, rand, tracing, zeroize, parking_lot, tokio, scc, blake3, regex
```

**Probleme:**
1. `tokio` als Direktabhängigkeit in einem E0-Crate → fragwürdig für ein Fundament
2. `scc` (concurrent HashMap) → unklar warum nötig
3. Keine Abhängigkeit auf `contextra-ids` oder `contextra-env`

### 6.3 Fehlende Workspace-Lints-Einbindung

| Crate | `[lints] workspace = true` |
|-------|---------------------------|
| `contextra-ids` | **Fehlt** ✗ |
| `contextra-env` | **Fehlt** ✗ |
| `contextra-durable-fs` | ✓ |
| `contextra-sys` | ✓ |
| `contextra-simd` | ✓ |
| `contextra-privacy` | ✓ |
| `contextra-erasure` | ✓ |

---

## 7. `contextra-types` und `contextra-ports` — Die Migrations-Altlast

### 7.1 `contextra-types::erasure.rs` — Vollständig deprecated

Das gesamte Modul `erasure.rs` (~500 Zeilen) ist `#[deprecated]` mit dem Hinweis:
```
Löschtypen gehören in die Erasure-Familie (contextra-erasure-spi per D-E1)
```

Aber es wird noch aktiv aus `contextra-types` re-exportiert:
```rust
#[allow(deprecated)]
pub use erasure::*;
```

**Betroffene Typen (alle deprecated aber noch exportiert):**
- `EraseTarget`, `EraseMode`, `Timing`
- `RetentionCode`, `RetentionAttestation`
- `EraseRequest`, `EraseReceipt`
- `ErasureStatus`, `ErasurePhase`, `SagaState`
- `Scanner<Unarmed>`, `Scanner<Armed>`, `ControlEvidence`
- `LayerWitness`, `Residue`
- `MeasureClass`, `LayerId`
- `Unarmed`, `Armed`

### 7.2 `contextra-types::ids.rs` — Wildcard Re-Export + Deprecated Extras

```rust
pub use contextra_ids::*;  // Re-Export alles aus ids

// Dann eigene deprecated Typen:
pub struct ErasureId(...);  // deprecated
pub struct GateId(...);     // deprecated
```

### 7.3 `contextra-ports` — Abhängigkeit auf types

`contextra-ports` (als E2 deklariert) hängt von `contextra-types` und `contextra-env` ab:
```toml
[dependencies]
contextra-env = { workspace = true }
contextra-types = { workspace = true }
```

**Problem:** Die SPEC deklariert `ports` als E2, was korrekt ist. Aber die Ports-Spec beschreibt Traits (`ErasureLayer`, `ReadGate`, `CommitBarrier`, `PurgeBarrier`), die konzeptuell auf E0/E1-Ebene benötigt werden. E1-Crates wie `store`, `vector`, `text` müssen diese Traits implementieren, dürfen aber E2 nicht importieren (R1).

**Mögliche Auflösung:**
- (A) `ports` wird zu E0 herabgestuft, verliert Abhängigkeit auf `types`
- (B) Die Traits werden in die E0-Crates verteilt (Gate in `erasure`, Barrier in `erasure`)
- (C) `ports` wird aufgeteilt: Port-Traits (E0) + Port-Typen (E2)

---

## 8. `contextra-env` — Quellcode-Details

### 8.1 Module und Zustand

| Modul | Inhalt | SPEC-konform? |
|-------|--------|--------------|
| `clock.rs` | `Clock`-Trait, `SystemClock` mit `start_instant` | ✓ |
| `env.rs` | Aggregierter `Env`-Struct | ✓ (nicht in SPEC, aber sinnvoll) |
| `id_gen.rs` | `IdGen`-Trait, `SequentialIdGen` | ✓ (nicht in SPEC, aber sinnvoll) |
| `manual_clock.rs` | `ManualClock` (feature-gated `test-utils`) | ✓ |
| `metrics.rs` | `MetricsSink`, `InMemoryMetricsSink`, `NoopMetricsSink` | ✓ |
| `rng.rs` | `Rng`-Trait, `SeededRng` (SplitMix64) | Teilweise ✓ |
| `lib.rs` | Re-Exports + `unix_ms()` Helper | ✓ |

### 8.2 Fehlende `Vfs`-Abstraktion

Die Spec (`docs/crates/contextra-env.md` §2.2) fordert ein `Vfs`-Trait in `contextra-env`. Im Code existiert kein solches Modul — VFS lebt in `contextra-durable-fs`.

**Finale Entscheidung nötig:** VFS gehört in genau eines der beiden Crates.

---

## 9. `contextra-sys` — Quellcode-Details

### 9.1 Module

| Modul | Inhalt |
|-------|--------|
| `lib.rs` | Root-Modul |
| `mlock.rs` | `LockedMemory<T>` — mlock/munlock Wrapper |
| `mmap.rs` | Memory-mapped I/O Abstraktionen |
| `posix.rs` | POSIX-spezifische Primitives (fsync, fadvise) |
| `durability.rs` | Durability-Level und fsync-Strategien |
| `vault.rs` | Secure memory vault (mlock + zeroize) |
| `contract.rs` | Trait-Contracts |
| `error.rs` | `SysError` |
| `acl_win32.rs` | Windows ACL-Abstraktionen |

### 9.2 Externe Abhängigkeiten

```toml
[dependencies]
memmap2, thiserror, tracing
[target.'cfg(unix)'.dependencies]
libc = "0.2"
[target.'cfg(windows)'.dependencies]
windows-sys = "0.61"
```

**Problem:** `memmap2` macht `mmap`-Aufrufe, die `unsafe` sind, aber das Crate nutzt `memmap2`'s safe API. Die Spec erwähnt `memmap2` nicht explizit als Abhängigkeit.

---

## 10. `contextra-simd` — Quellcode-Details

### 10.1 Module

| Modul | Inhalt |
|-------|--------|
| `lib.rs` | Dispatcher + API |
| `dispatch.rs` | CPU-Feature-Erkennung |
| `kernels/scalar.rs` | IEEE 754 Referenzimplementierung |
| `kernels/avx2.rs` | AVX2-Implementierung |
| `kernels/avx512.rs` | AVX-512-Implementierung |
| `kernels/neon.rs` | ARM NEON |
| `metric.rs` | Metrik-Enum (Cosine, L2, Dot) |
| `error.rs` | `SimdError` |
| `contract.rs` | Trait-Contracts |
| `compat/mod.rs` | Kompatibilitäts-Adapter |

### 10.2 Problematisches Feature

```toml
[features]
workspace = ["dep:contextra-types"]
types-compat = ["workspace"]
```

Die optionale Abhängigkeit auf `contextra-types` (E2) in einem E0-Crate ist ein Ebenenverstoß. Muss entfernt werden.

---

## 11. `contextra-durable-fs` — Quellcode-Details

### 11.1 Module

| Modul | Datei | SPEC-konform? |
|-------|-------|--------------|
| VFS | `vfs.rs` | ✓ |
| Platform | `platform.rs` | ✓ |
| Health/Poison | `health.rs` | ✓ |
| Scrub | `scrub.rs` | ✓ |
| Journal | `journal.rs` | ✓ |
| Fault Injector | `fault_injector.rs` | ✓ |
| Error | `error.rs` | ✓ |

### 11.2 Abhängigkeiten

Alle Abhängigkeiten sind optional und extern — keine internen Workspace-Abhängigkeiten. ✓

---

## 12. Entscheidungsmatrix (offene Punkte)

| # | Entscheidung | Option A | Option B | Empfehlung |
|---|-------------|----------|----------|------------|
| D-01 | Wo lebt `ErasureId`? | `contextra-ids` (SPEC) | `contextra-erasure` (Kohäsion) | **A** — Folge SPEC, `ids` ist der ID-Kernel |
| D-02 | Wo lebt `DataClass`? | `contextra-privacy` (SPEC) | `contextra-types` (da es ein Datenmodell ist) | **A** — SPEC ist klar |
| D-03 | Wo lebt `Validated<T>`? | `contextra-privacy` (SPEC) | Neu als `GuardedPayload` Erweiterung | **A** — Separater Typ für K-Klasse-Guard |
| D-04 | Wo lebt `Vfs`? | `contextra-env` (Spec) | `contextra-durable-fs` (Code) | **B** — VFS braucht fsync/health, gehört zu durable-fs |
| D-05 | `contextra-erasure` Ebene? | E0 (SPEC) | E2 (Code-Metadata Ring 3) | **Kompromiss:** Split in `erasure` (E0, Typen+Gate+Scanner) + Orchestrator in `db` (E2) |
| D-06 | `SubjectRef` Repräsentation? | `[u8; 16]` (SPEC: 128-Bit) | `Vec<u8>` (Ist) | **A** — SPEC-konform, 128-Bit fixed |
| D-07 | `SagaState` Name `KeyRevoked` vs `KeyDestroyed`? | `KeyRevoked` (SPEC) | `KeyDestroyed` (Code) | **A** — SPEC ist normativ |
| D-08 | `contextra-ports` Ebene? | E2 (SPEC) | Split E0+E2 | **Split:** Traits für E1 in eigenem E0-Crate oder inline in `erasure` |
| D-09 | `contextra-privacy` darf `ids`/`env` nutzen? | Ja (SPEC §2.1 erlaubt E0-intern) | Nein (autark) | **Ja** — Eliminiert Typ-Dubletten |
| D-10 | Default-Features? | Leer (SPEC R3) | Status quo | **Leer** — SPEC ist klar |

---

## 13. Reihenfolge der Bereinigung (Bottom-Up)

### Phase 1: `contextra-ids` (Fundament aller IDs)
1. `SubjectRef` → `[u8; 16]`, kein `Clone`, `ZeroizeOnDrop`, `OsRng::try_fill_bytes`
2. `PersonalRef` → `[u8; 16]`, kein `Clone`
3. `ErasureId` → hierher verschieben aus `types`/`erasure`
4. `GateId` → hierher verschieben
5. `rand` und `zeroize` als Dependencies hinzufügen
6. `default = []` setzen
7. Workspace-Lints einbinden

### Phase 2: `contextra-env` (Zeitquellen + Metriken)
1. `Vfs` aus Spec **entfernen** (lebt korrekt in `durable-fs`)
2. `default = []` setzen
3. Workspace-Lints einbinden

### Phase 3: `contextra-sys` (OS-Primitive)
1. Keine kritischen Änderungen — Code und Spec stimmen überein
2. `memmap2`-Abhängigkeit in Spec dokumentieren

### Phase 4: `contextra-durable-fs` (Crash-Safety)
1. `default = []` setzen
2. Ist weitgehend SPEC-konform

### Phase 5: `contextra-simd` (SIMD-Kerne)
1. Feature `workspace` / `types-compat` **entfernen** (Ebenenverstoß)
2. `default = []` setzen

### Phase 6: `contextra-privacy` (Datenschutz)
1. `contextra-ids` und `contextra-env` als Dependency hinzufügen
2. Lokale `TenantId`, `Clock`, `TenantScoped` **entfernen** → Re-Export aus `ids`/`env`
3. `DataClass { K, P, D, E }` implementieren
4. `Validated<T>` implementieren (separater Typ neben `GuardedPayload`)
5. `default = []` setzen
6. `tokio`-Abhängigkeit evaluieren (E0 sollte runtime-agnostisch sein)

### Phase 7: `contextra-erasure` (Löschkern)
1. Lokale `ErasureId` → Import aus `contextra-ids`
2. Lokale `EraseTarget` und alle `types_local.rs`-Typen konsolidieren
3. `aes-gcm-siv`, `sha2`, `hmac` → Delegation an `contextra-crypto`
4. `SagaState::KeyDestroyed` → `KeyRevoked` umbenennen
5. Ring-Metadata korrigieren (E0 oder E2 je nach D-05)
6. `default = []` setzen

### Phase 8: `contextra-types` (E2, Datenmodell)
1. Deprecated `erasure.rs` **komplett entfernen**
2. Deprecated `ErasureId`, `GateId` aus `ids.rs` entfernen
3. `EraseTarget` auf EINE kanonische Definition reduzieren
4. Re-Exports bereinigen

### Phase 9: `contextra-ports` (E2, Port-Traits)
1. `ErasureLayer`-Trait Fehlersignatur: `String` → typisierter Error
2. `LayerWitness` auf eine kanonische Definition normieren
3. Ebenen-Problematik für E1-Implementierer lösen

---

## 14. Quellcode-Referenzen (Dateipfade)

### E0 Crates
- `crates/contextra-ids/src/lib.rs` — 720 Zeilen, 1 Datei
- `crates/contextra-env/src/` — 6 Module (clock, env, id_gen, manual_clock, metrics, rng)
- `crates/contextra-sys/src/` — 9 Module (acl_win32, contract, durability, error, mlock, mmap, posix, vault)
- `crates/contextra-durable-fs/src/` — 7 Module (error, fault_injector, health, journal, platform, scrub, vfs)
- `crates/contextra-simd/src/` — 10 Module (compat, contract, dispatch, error, kernels/*, metric)
- `crates/contextra-privacy/src/` — 12 Module (audit_trace, bulk_exfiltration_detector, context_edit_audit, contract, egress_*, entity_recognizer, error, guarded_payload, processing_registry, types)
- `crates/contextra-erasure/src/` — ~50 Dateien in 12+ Modulgruppen

### E2 Referenz-Crates
- `crates/contextra-types/src/` — 11 Module (error, error_dto, model_fingerprint, retrieval_strategy, schema, tenant_scope, tombstone, types, erasure, ids, refs, witness)
- `crates/contextra-ports/` — Cargo.toml zeigt deps auf `env` + `types`

---

## 15. Zusammenfassung der Schweregrade

| Schwere | Anzahl | Kategorie |
|---------|--------|-----------|
| 🔴 KRITISCH | 7 | Typ-Dubletten, Ebenen-Verletzungen, fehlende SPEC-Implementierungen |
| 🟡 WICHTIG | 5 | Inkonsistente Specs, Feature-Verstöße, Abhängigkeitsprobleme |
| 🟢 GERING | 3 | Benennungsinkonsistenzen, fehlende Lint-Einbindung |

**Gesamtbewertung:** Der Workspace befindet sich in einem halb-entkoppelten Zustand. Die Migration der Löschdomäne von `contextra-types` nach `contextra-erasure` wurde begonnen (alles `#[deprecated]`), aber nicht abgeschlossen. Gleichzeitig hat `contextra-erasure` alle Typen lokal kopiert statt die kanonischen Quellen zu nutzen. `contextra-privacy` hat eigene Kopien fundamentaler Typen angelegt. Das Ergebnis sind 3+ Kopien derselben Konzepte mit subtilen Inkompatibilitäten.

Die Bereinigung MUSS strikt bottom-up erfolgen: Zuerst `contextra-ids` als Single Source of Truth für alle Identitätstypen stabilisieren, dann die Abhängigkeitsketten aufwärts bereinigen.
