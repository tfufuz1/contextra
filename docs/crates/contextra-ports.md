# contextra-ports — Crate-Spezifikation (Port-Schnittstellen / Traits)

**Ebene:** E2 (Integration & Schnittstellenverträge)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-ports`  
**Referenz:** `docs/SPEC.md` §2, §5, §6, §7, §10

---

## 1. Zweck & Systemrolle

`contextra-ports` definiert die systemweiten Trait-Schnittstellen (Ports) nach dem Prinzip der hexagonalen Architektur (Inversion of Control):
- **Entkopplung der Schichten:** Ermöglicht dem Löschkern (`contextra-erasure`) und der Orchestrierungsebene (`contextra-db`), Speicherschichten zu steuern, ohne direkt von ihnen abzuhängen.
- **Synchronisationsverträge:** Definition von Barrieren (`CommitBarrier`, `PurgeBarrier`) und Lesesperren (`ReadGate`).
- **Standardisierte Messung:** Definition des kanonischen `LayerWitness`- und `Scanner`-Protokolls.

---

## 2. Die kanonischen Port-Traits

### 2.1 `ErasureLayer` (`erasure.rs`)
Jedes speichernde Subsystem implementiert dieses Trait:
```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn key_coverage(&self) -> KeyCoverage;

    fn resolve(&self, target: &EraseTarget)
        -> BoxFuture<'_, Result<LayerScope, String>>;

    fn on_gate(&self, scope: &LayerScope)
        -> BoxFuture<'_, Result<(), String>>;

    fn purge(&self, scope: &LayerScope, cx: &PurgeCx)
        -> BoxFuture<'_, Result<PurgeOutcome, String>>;

    fn witness(&self, scope: &LayerScope, sc: &Scanner<Armed>)
        -> BoxFuture<'_, Result<LayerWitness, String>>;
}
```

### 2.2 `ReadGate` (`gate.rs`)
Synchroner Hot-Path-Leseschutz:
```rust
pub trait ReadGate: Send + Sync {
    fn check(&self, key: &[u8]) -> GateDecision;
    fn check_range(&self, range: &KeyRange) -> RangeGate;
    fn gated_seq(&self, key: &[u8]) -> Option<Seq>;
}

pub enum GateDecision {
    Open,
    Erased,
    Restricted,
}
```

### 2.3 Barrieren (`barrier.rs`)
```rust
pub trait CommitBarrier: Send + Sync {
    async fn shared(&self) -> SharedPermit<'_>;
    async fn exclusive(&self) -> ExclusivePermit<'_>;
}

pub trait PurgeBarrier: Send + Sync {
    async fn compaction(&self) -> CompactionPermit<'_>;
    async fn purge(&self) -> PurgePermit<'_>;
    fn purge_waiting(&self) -> bool;
}
```

### 2.4 `LayerWitness` & Positivkontrolle (`scanner.rs`)
**INV-WITNESS-MINT-1:** `LayerWitness` MUSS kanonisch in `contextra-ports::scanner` definiert sein:
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerWitness {
    pub layer_id: LayerId,
    pub purged_count: u64,
    pub bytes_reclaimed: u64,
    pub residue_matches: u64, // MUSS 0 sein für Completed
    pub scan_class: MeasureClass,
}
```

---

## 3. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
2. **Keine konkreten Speicher-Imports:** `contextra-ports` importiert weder `store`, `vector`, `text`, `graph` noch `db`.
3. **Alle Traits sind `Send + Sync`:** Garantierte Thread-Sicherheit über asynchrone Tokio-Tasks.
