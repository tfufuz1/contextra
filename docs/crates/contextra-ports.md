> **Löschpfad v3.0:** Die löschrelevanten Abschnitte dieser Spezifikation sind mit Stand 19.11.2026 durch `docs/SPEC.md` v3.0 ersetzt bzw. präzisiert (§5, §6, §7, §9, §12, §13, §14, §16 der SPEC). Widerspricht ein Abschnitt unten der SPEC, gilt die SPEC. Abschnitte, die so markiert sind, sind überholt. Weitere Verweise: §23 SPEC (Ersetzungstabelle).

# contextra-ports — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Fundament & Schnittstellenverträge / Ring 0)  
**Kern-Closure:** `ids`, `env`, `types`  
**Status:** Normativ (Soll-Zustand)  
**Version:** 2.2  
**Geltungsbereich:** `crates/contextra-ports`  
**Referenzdokumente:** `docs/SPEC.md` §2.1, §2.4, §5, §6, §7, §10

---

## 1. Zweck & Architekturgarantien

`contextra-ports` definiert systemweite, trait-basierte Schnittstellenverträge (Ports) nach dem Prinzip der hexagonalen Architektur (Ports & Adapters):
- **Schichtentrennung (E0):** Als Fundamentbaustein auf Ebene E0 hängt `contextra-ports` ausschließlich von `contextra-env`, `contextra-types` und `contextra-ids` ab.
- **Entkopplung der Schichten:** Ermöglicht den E1-Domänenbausteinen (`store`, `vector`, `text`, `graph`), standardisierte Schnittstellen zu implementieren, und dem E2-Orchestrator (`contextra-db`), diese homogen zu steuern.
- **Synchronisationsverträge:** Definition von Barrieren (`CommitBarrier`, `PurgeBarrier`).
- **EmbeddingEngine-Port:** Abstrahiert Text- und Vektoreinbettungen.

### Architekturgarantien (R1–R6)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Object Safety (Dyn-Kompatibilität):** Alle Port-Traits sind objekt-sicher (`Box<dyn Port>`).
3. **Keine konkreten Speicherschichten:** Importiert niemals `store`, `vector`, `text`, `graph`, `db`.
4. **Leere Default-Features (R3):** `default = []`.
5. **R1-Konformität:** Keine Aufwärtsabhängigkeiten (E0 hängt von keinem E1/E2/E3 ab).

---

## 2. Kanonische Port-Traits

```
┌─────────────────────────────────────────────────────────────┐
│                       contextra-ports                       │
│                                                             │
│   ┌──────────────────────────┐ ┌──────────────────────────┐ │
│   │      CommitBarrier       │ │       PurgeBarrier       │ │
│   │ shared() / exclusive()   │ │ compaction() / purge()   │ │
│   │                          │ │ purge_waiting() -> bool  │ │
│   └──────────────────────────┘ └──────────────────────────┘ │
│                                                             │
│   ┌──────────────────────────┐                             │
│   │     EmbeddingEngine      │                             │
│   │ embed_query, embed_batch │                             │
│   └──────────────────────────┘                             │
│                                                             │
│  ⚠ ErasureLayer, ReadGate, LayerWitness, Scanner<Armed>    │
│    sind in contextra-erasure (E0) definiert, nicht hier.   │
└─────────────────────────────────────────────────────────────┘
```

---

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

### 2.1 Synchronisations-Barrieren (`barrier.rs`)

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
    /// Signalisiert wartenden Kompaktionen, dass ein Purge ansteht.
    fn purge_waiting(&self) -> bool;
}
```

---

## 3. EmbeddingEngine (`embedding.rs`)

```rust
pub trait EmbeddingEngine: Send + Sync {
    fn embed_query(&self, text: &str) -> BoxFuture<'_, Result<Vec<f32>, PortError>>;
    fn embed_batch(&self, texts: &[&str]) -> BoxFuture<'_, Result<Vec<Vec<f32>>, PortError>>;
    fn dimension(&self) -> usize;
}
```

---

## 4. Fehlerbehandlung (`PortError`)

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

## 5. `Cargo.toml` Spezifikation

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

## 6. Implementierungs-Arbeitspakete für Entwickler

1. **ErasureLayer entfernen:** `src/erasure.rs` löschen. `ErasureLayer`, `ReadGate`, `Scanner<Armed>`, `LayerWitness` werden aus `contextra-erasure` importiert, nicht in `contextra-ports` definiert.
2. **Signatur-Härtung:** Alle Rückgabetypen von `Result<_, String>` auf `Result<_, PortError>` migrieren.
3. **Altlasten-Bereinigung:** In `src/embedding.rs` Referenzen auf `infer-candle` und `infer-ollama` entfernen.
4. **Verifikation:**
   - `cargo clippy -p contextra-ports -- -D warnings`
   - `cargo test -p contextra-ports`
