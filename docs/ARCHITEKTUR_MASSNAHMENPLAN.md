# Architektonischer Maßnahmenplan: Transformation in einen modularen Rust-Baukasten

**Dokument-ID:** ARCH-PLAN-2026-11
**Rolle:** Principal Senior Rust Engineer & Systemarchitekt
**Status:** In Umsetzung / Leitfaden
**Geltungsbereich:** Workspace (36 Crates), insb. `contextra-checkpoint`, `contextra-cognition`, `contextra-erasure`, `contextra-core`, `contextra-engine`

---

## 1. Executive Summary & Zielbild

Das Repository wird von einem monolithisch integrierten Gesamtsystem in eine hochgradig modulare **Rust-Baukasten-Bibliothek** transformiert. Entwickler sollen einzelne Crates (z. B. HNSW-Vektorindex, SIMD-Kernel, RRF-Rankfusion, Prefix-KV-Cache, Checkpoint-Koordination) ohne Zwang zu globalen Workspace-Abhängigkeiten in beliebige Drittsysteme (wie Cloud-Microservices, Enterprise-RAG oder Robotik-Steuerungen) portieren oder kopieren können.

Die Leitprinzipien sind:
1. **Zero-Monolith & Zero Code-Verlust:** Algorithmen bleiben vollständig erhalten, werden jedoch sauber gekapselt.
2. **Auflösung monolithischer Fassaden:** Zerschlagung künstlicher Monolithen (`contextra-core`, `contextra-engine`) zugunsten klarer Ports & Adapter.
3. **Strikte Domänentrennung (DSGVO/Compliance):** Reine Index- und Persistence-Schnittstellen (`insert`, `remove`, `search`) ohne Vermischung mit Löschketten; die gesamte DSGVO- und Art.-17-Saga residiert isoliert in `contextra-erasure`.
4. **Concurrency-Hygiene in Tokio:** Strikte Vermeidung von `lock_across_await` (Deadlock-Prävention) und Eliminierung blockierender `block_on`-Aufrufe in asynchronen Pfaden.
5. **Ergonomie & Portabilität:** Eigenständige, typisierte Fehler via `thiserror`, saubere Feature-Gates und Vorrang für Standardtypen (`&[f32]`, `&str`, `u64`).

---

## 2. Detaillierter Maßnahmenplan

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            ARCHITEKTUR-ROADMAP                              │
├────────────────────────┬────────────────────────────────────────────────────┤
│ Phase 1: Concurrency   │ • Beseitigung von lock_across_await in Checkpoint  │
│          & Async       │   und Cognition (Guard Scopes minimieren)          │
│                        │ • Beseitigung blockierender block_on-Aufrufe       │
├────────────────────────┼────────────────────────────────────────────────────┤
│ Phase 2: DSGVO-        │ • Indizes (Vector/Text/Store) rein auf CRUD        │
│          Entflechtung  │ • Alle Löschketten/Sagas strikt in contextra-      │
│                        │   erasure kapseln                                  │
├────────────────────────┼────────────────────────────────────────────────────┤
│ Phase 3: Fassaden-     │ • contextra-core deprecaten/entfernen              │
│          Auflösung     │ • contextra-engine in schlanke Services zerlegen   │
├────────────────────────┼────────────────────────────────────────────────────┤
│ Phase 4: Baukasten-DX  │ • Autarkieprüfung pro Crate (Feature-Gates)        │
│          & Portierung  │ • Standardisierte Fehler via thiserror             │
└────────────────────────┴────────────────────────────────────────────────────┘
```

---

## 3. Konkrete Code-Muster & Refactoring-Strategien

### 3.1 Concurrency-Hygiene: Beseitigung von `lock_across_await` und `block_on`

#### Problemstellung
Das Halten synchroner oder asynchroner Lock-Guards über `.await`-Aufhängepunkte hinweg blockiert Tokio-Worker-Threads oder führt bei verschachtelten Aufrufen unweigerlich zu Inversions-Deadlocks. Ebenso führt `block_on` innerhalb laufender Tokio-Runtimes zu Panics oder Thread-Starvation.

#### Code-Muster A: Guard-Scope Verkürzen vor `.await` (Checkpoint/Store)
Statt einen Lock über asynchrone I/O-Aufrufe hinweg offen zu halten, wird der kritische Abschnitt auf die reine Zustandstransformation im Speicher beschränkt:

```rust
// ❌ ANTI-PATTERN: Lock wird über .await gehalten (Deadlock-Gefahr)
pub async fn save_checkpoint_bad(&self, meta: CheckpointMeta) -> Result<()> {
    let mut guard = self.index.write(); // parking_lot::RwLock
    self.storage.put(meta.tx_id, b"key", b"val").await?; // ⚠️ COMPILE/RUNTIME HAZARD
    guard.insert(meta);
    Ok(())
}

// ✅ REFACTORED PATTERN: Scope-Verkürzung & Sequentialisierung
pub async fn save_checkpoint_clean(&self, meta: CheckpointMeta) -> Result<()> {
    // 1. Asynchrone I/O ohne synchronen Lock
    let manifest_bytes = serde_json::to_vec(&meta)?;
    self.storage.put(meta.tx_id, b"key", &manifest_bytes).await?;
    self.storage.commit(meta.tx_id).await?;

    // 2. Zustand atomar und kurz im In-Memory-Index aktualisieren
    {
        let mut index_guard = self.index.write();
        index_guard.insert(meta);
    } // Guard wird hier VOR jedem weiteren async Call freigegeben

    Ok(())
}
```

#### Code-Muster B: Vermeidung von `block_on` in synchronen Konstruktoren
Synchronen Code nicht mit erzwungenem `block_on` aufblähen, sondern asynchrone Initialisierung primär anbieten und synchrone Adapter sauber kapseln:

```rust
// ❌ ANTI-PATTERN: Verschachteltes block_on
pub fn new_blocking(storage: Arc<S>) -> Result<Self> {
    let rt = tokio::runtime::Handle::current();
    rt.block_on(Self::open(storage)) // ⚠️ Panik in async Runtime
}

// ✅ REFACTORED PATTERN: Saubere async/sync Trennung via spawn_blocking
pub async fn open(storage: Arc<S>) -> Result<Self> {
    // Reiner asynchroner Einstiegspunkt für Tokio-Umgebungen
    let initial_state = storage.last_tx_id().await?;
    Ok(Self { storage, state: initial_state })
}

pub fn open_sync_dedicated(storage: Arc<S>) -> Result<Self> {
    // Wenn synchron zwingend erforderlich: Separater OS-Thread ohne Runtime-Konflikt
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(ContextraError::Configuration(
            "Synchroner Konstruktor darf nicht innerhalb einer laufenden Tokio-Runtime aufgerufen werden. Bitte 'open().await' nutzen.".into(),
        ));
    }
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    rt.block_on(Self::open(storage))
}
```

---

### 3.2 Domänentrennung: Reines CRUD für Indizes vs. `contextra-erasure`

#### Problemstellung
Indizes (Vektor, Text, KV-Store) enthielten historisch Löschketten-Prüfungen (`DeletionProof`, `WitnessLayer`, DSGVO-Zertifikate). Dadurch waren Indizes nicht portabel.

#### Code-Muster: Bereinigte Index-Schnittstelle
Indizes beschränken sich rein auf elementare Schnittstellen. Sämtliche Nachweise und Lösch-Sagas werden von außen orchestriert:

```rust
// ✅ REFACTORED PATTERN: Minimaler, portabler Vektorindex
pub trait VectorIndex: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn insert(&self, id: u64, vector: &[f32]) -> impl std::future::Future<Output = std::result::Result<(), Self::Error>> + Send;
    fn search(&self, query: &[f32], k: usize) -> impl std::future::Future<Output = std::result::Result<Vec<(u64, f32)>, Self::Error>> + Send;
    fn remove(&self, id: u64) -> impl std::future::Future<Output = std::result::Result<bool, Self::Error>> + Send;
}

// ✅ DSGVO-Orchestrierung ausschließlich in contextra-erasure:
pub struct ErasureSaga<V: VectorIndex, S: StorageEngine> {
    vector_index: Arc<V>,
    storage: Arc<S>,
}

impl<V: VectorIndex, S: StorageEngine> ErasureSaga<V, S> {
    pub async fn execute_erasure(&self, target_doc: u64) -> Result<ErasureReceipt> {
        // 1. Physisches Entfernen über neutrale Index-API
        self.vector_index.remove(target_doc).await
            .map_err(|e| ErasureError::Storage(e.to_string()))?;

        // 2. Kryptographisches Shredding & Receipt-Generierung rein in Erasure-Schicht
        let receipt = ErasureReceipt::generate(target_doc);
        Ok(receipt)
    }
}
```

---

### 3.3 Auflösung monolithischer Fassaden (`contextra-core`, `contextra-engine`)

#### Problemstellung
Monolithische Fassaden führten zu zyklischen Kopplungen und machten ein Auskoppeln einzelner Bausteine unmöglich.

#### Code-Muster: Dependency Inversion über generische Bausteine
Statt harter Verdrahtung werden Bausteine rein über Generics gekoppelt:

```rust
// ✅ REFACTORED PATTERN: Autarker Memory-Consolidator ohne starren Engine-Monolithen
pub struct MemoryConsolidator<S, V> {
    storage: Arc<S>,
    vector_index: Arc<V>,
}

impl<S, V> MemoryConsolidator<S, V>
where
    S: StorageEngine + 'static,
    V: VectorIndex + 'static,
{
    pub fn new(storage: Arc<S>, vector_index: Arc<V>) -> Self {
        Self { storage, vector_index }
    }

    pub async fn consolidate(&self) -> Result<(), ConsolidatorError> {
        // Verwendet ausschließlich generische Traits, keine Kenntnis des Gesamtsystems
        Ok(())
    }
}
```

---

### 3.4 Crate-Design für maximale Autarkie & Kopierbarkeit

1. **Typisierte Fehler mit `thiserror` pro Crate:**
```rust
#[derive(thiserror::Error, Debug)]
pub enum CheckpointError {
    #[error("I/O error during checkpointing: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization failure: {0}")]
    Serialization(String),
    #[error("Checkpoint '{0}' was not found")]
    NotFound(String),
    #[error("Serialization barrier violation: {0}")]
    BarrierViolation(String),
}
```

2. **Feature-Gates in `Cargo.toml`:**
```toml
[package]
name = "contextra-rank"
version = "0.1.0"
edition = "2021"

[features]
default = ["std"]
std = []
conformal = ["dep:statrs"]
types-compat = ["dep:contextra-types"]

[dependencies]
thiserror = "2"
statrs = { version = "0.17", optional = true }
contextra-types = { path = "../contextra-types", optional = true }
```

3. **Verwendung nativer Standardtypen:**
Schnittstellen akzeptieren primär `&[f32]`, `&[u8]`, `u64`, `&str` statt komplexer, proprietärer Wrapper-Typen. Dadurch können Fremdsysteme die Bausteine sofort ohne Adapter-Code aufrufen.
