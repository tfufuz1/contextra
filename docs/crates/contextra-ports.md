# contextra-ports — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E2 (Integration & Schnittstellenverträge)  
**Kern-Closure:** `ids`, `env`, `erasure`, `types`  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-ports`  
**Referenzdokumente:** `docs/SPEC.md` §2, §5, §6, §7, §10; `ANALYSIS-E0-CONTRADICTIONS.md` (§7.3, D-08); `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-ports` definiert die systemweiten, dynamisch dispatchbaren Trait-Schnittstellen (Ports) nach dem Prinzip der hexagonalen Architektur (Ports & Adapters):
- **Entkopplung der Schichten:** Ermöglicht dem Orchestrator (`contextra-db`) und dem Löschkern (`contextra-erasure`), Speicherschichten zu steuern, ohne konkrete Implementierungsdetails zu kennen.
- **Synchronisationsverträge:** Definition von Barrieren (`CommitBarrier`, `PurgeBarrier`) und der synchronen Lesesperre (`ReadGate`).
- **Lösch-Protokoll (`ErasureLayer`):** Einheitlicher Vertrag für alle speichernden Schichten (Store, Vector, Text, Graph, Crypto).
- **Messung & Positivkontrolle:** Definition von `LayerWitness`, `MeasureClass` und des Scanner-Protokolls.

### Beseitigung von Typ- und Altlasten-Inkonsistenzen (R4, AP-21)
1. **Fehlertypen typisieren:** `ErasureLayer`-Methoden dürfen nicht `Result<T, String>` zurückgeben, sondern typisierte Fehler (`PortError` bzw. `ContextraError`).
2. **Kanonische `LayerWitness`-Definition:** `LayerWitness` wird kanonisch in `contextra-ports::scanner` definiert (`INV-WITNESS-MINT-1`).
3. **Altlasten-Bereinigung (`AP-21`):** In `src/embedding.rs` Zeile 7 werden alle Verweise auf `infer-candle` und `infer-ollama` bereinigt.

### Architekturgarantien (R1–R6, INV-3, INV-4)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Object Safety (Dyn-Kompatibilität):** Alle Port-Traits sind objekt-sicher (`Box<dyn Port>`).
3. **Keine zirkulären Abhängigkeiten:** Importiert niemals konkrete Speicherschichten (`store`, `vector`, `text`, `graph`, `db`).
4. **Leere Default-Features (R3):** `default = []`.

---

## 2. Kanonische Port-Traits

```
┌─────────────────────────────────────────────────────────────┐
│                       contextra-ports                       │
│                                                             │
│   ┌──────────────────────────┐ ┌──────────────────────────┐ │
│   │       ErasureLayer       │ │         ReadGate         │ │
│   │ resolve, on_gate, purge, │ │ check(key), check_range, │ │
│   │ witness                  │ │ gated_seq                │ │
│   └──────────────────────────┘ └──────────────────────────┘ │
│                                                             │
│   ┌──────────────────────────┐ ┌──────────────────────────┐ │
│   │      CommitBarrier       │ │       PurgeBarrier       │ │
│   │ shared() / exclusive()   │ │ compaction() / purge()   │ │
│   └──────────────────────────┘ └──────────────────────────┘ │
│                                                             │
│   ┌──────────────────────────┐ ┌──────────────────────────┐ │
│   │       LayerWitness       │ │     EmbeddingEngine      │ │
│   │ residue_matches == 0     │ │ embed_query, embed_batch │ │
│   └──────────────────────────┘ └──────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

### 2.1 `ErasureLayer` (`erasure.rs`)

Jedes speichernde Subsystem implementiert dieses Trait:

```rust
use contextra_erasure::{EraseTarget, LayerWitness, Scanner, Armed};
use futures_util::future::BoxFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(pub [u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCoverage {
    Full,
    None,
}

pub struct LayerScope {
    pub layer_id: LayerId,
    pub affected_items: u64,
}

pub struct PurgeCx {
    pub disk_reserve_bytes: u64,
}

pub struct PurgeOutcome {
    pub items_purged: u64,
    pub bytes_reclaimed: u64,
}

pub trait ErasureLayer: Send + Sync {
    /// Eindeutige Schicht-Identifikation (z. B. "CTX_STORE_LSM_01").
    fn id(&self) -> LayerId;

    /// Gibt an, ob diese Schicht direkte Schlüsselabdeckung besitzt.
    fn key_coverage(&self) -> KeyCoverage;

    /// Phase 1: Ermittelt betroffene Segmente/Dateien für das EraseTarget.
    fn resolve(&self, target: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, PortError>>;

    /// Phase 2: Bereitet In-Memory Ausschlussmasken vor (z. B. RoaringBitmap).
    fn on_gate(&self, scope: &LayerScope) -> BoxFuture<'_, Result<(), PortError>>;

    /// Phase 5: Physische Bereinigung unter der PurgeBarrier.
    fn purge(&self, scope: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, PortError>>;

    /// Phase 6: Verifikation mittels Positivkontrolle (Scanner<Armed>).
    fn witness(&self, scope: &LayerScope, sc: &Scanner<Armed>) -> BoxFuture<'_, Result<LayerWitness, PortError>>;
}
```

---

### 2.2 Synchronisations-Barrieren (`barrier.rs`)

```rust
pub trait SharedPermit: Send + Sync {}
pub trait ExclusivePermit: Send + Sync {}

/// Schutz des synchronen Gate-Commits gegen parallele Schreiber.
pub trait CommitBarrier: Send + Sync {
    fn shared(&self) -> BoxFuture<'_, Box<dyn SharedPermit>>;
    fn exclusive(&self) -> BoxFuture<'_, Box<dyn ExclusivePermit>>;
}

pub trait CompactionPermit: Send + Sync {}
pub trait PurgePermit: Send + Sync {}

/// Schutz des physischen Purges gegen parallele LSM-Kompaktionen.
pub trait PurgeBarrier: Send + Sync {
    fn compaction(&self) -> BoxFuture<'_, Box<dyn CompactionPermit>>;
    fn purge(&self) -> BoxFuture<'_, Box<dyn PurgePermit>>;
    fn purge_waiting(&self) -> bool;
}
```

---

## 3. Fehlerbehandlung (`PortError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("Layer resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("OnGate preparation failed: {0}")]
    OnGateFailed(String),

    #[error("Purge execution failed: {0}")]
    PurgeFailed(String),

    #[error("Verification witness scan failed: {0}")]
    WitnessFailed(String),

    #[error("Port timeout exceeded")]
    Timeout,
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-ports"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Canonical dyn-compatible port traits for Contextra subsystems"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-erasure = { workspace = true }
contextra-types = { workspace = true }
futures-util = "0.3"
thiserror = { workspace = true }
serde = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **Signatur-Härtung:**
   - In `src/erasure.rs` alle Rückgabetypen von `Result<_, String>` auf `Result<_, PortError>` migrieren.
2. **Altlasten-Bereinigung (`AP-21`):**
   - In `src/embedding.rs` Zeile 7 Referenzen auf `infer-candle` und `infer-ollama` bereinigen.
3. **Verifikation:**
   - `cargo test -p contextra-ports`
