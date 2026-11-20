# contextra-ports — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E0 (Fundament und Ports)
**Kern-Closure:** `env`, `ids` (SPEC §2.4). Keine Kante auf `types`.
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-ports`
**Normative Quellen:** `docs/SPEC.md` §2.4, §2.5, §4.2, §4.3, §10.1, §10.2, §11.

---

## 1. Zweck

`contextra-ports` hält die Trait-Verträge, die E1 implementiert und die `contextra-db` verdrahtet.

Kanonisch hier (SPEC §2.5):

- `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`
- `CommitBarrier`, `PurgeBarrier`
- `EffaceableKeySlot`, `KeyAnchor`, `Custody`, `WrappedBlob`, `DestroyAttestation`, `SlotGeneration`

`ErasureLayer`, `ReadGate`, `LayerWitness`, `Scanner<Armed>`, `PurgeCx` und `PurgeOutcome` sind in `contextra-erasure` definiert. Dieses Crate importiert sie nicht und definiert sie nicht (SPEC §7). Eine Abhängigkeit auf `contextra-erasure` ist ausgeschlossen.

Eine Abhängigkeit auf `contextra-types` ist ebenfalls ausgeschlossen. `types` bindet `privacy`, und `privacy` bindet `Purgeable` aus diesem Crate. `ports → types` würde den Zyklus `types → privacy → ports → types` schließen.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf E1, E2 oder E3. Keine Abhängigkeit auf `types`, `privacy` oder `erasure`. Identitätstypen kommen aus `ids`.
3. Traits, die `contextra-db` als `dyn` hält, sind objekt-sicher.
4. Kein konkreter Speicher, kein konkretes AEAD, kein Saga-Koordinator.

## 3. Index-Ports

Die Traits beschreiben die fachlichen Operationen. Die Gate-Prüfung sitzt nicht in diesen Traits. Sie sitzt in `TenantScopedStorage` (`contextra-db`, SPEC §6.1). Ein Index darf die `LiveMask` als vom Aufrufer gesetztes Filterkriterium lesen. Er entscheidet den Saga-Zustand nicht.

```rust
pub trait StorageEngine: Send + Sync {
    fn put(&self, key: &[u8], sealed_record: &[u8]) -> Result<(), PortError>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, PortError>;
    fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PortError>;
}

pub trait VectorIndex: Send + Sync {
    fn search_knn(&self, query: &[f32], k: usize) -> Result<Vec<ScoredId>, PortError>;
}

pub trait TextIndex: Send + Sync {
    fn search(&self, terms: &[String], k: usize) -> Result<Vec<ScoredId>, PortError>;
}

pub trait GraphIndex: Send + Sync {
    fn traverse(&self, start: &[u8], max_hops: u32) -> Result<Vec<ScoredId>, PortError>;
}
```

`ScoredId` ist port-lokal: Schlüsselbytes und `f32`. `SearchHit` bleibt in `contextra-types` (R4). `contextra-db` mappt `ScoredId` auf `SearchHit`.

```rust
pub struct ScoredId {
    pub key: Vec<u8>,
    pub score: f32,
}
```

## 4. Barrieren (SPEC §10)

### 4.1 `CommitBarrier`

`shared` erlaubt parallele Schreiber. `exclusive` wartet auf alle `shared`-Permits und blockiert neue. Der Saga-Koordinator nimmt `exclusive` für den Gate-Commit in Takt 1.

```rust
pub trait CommitBarrier: Send + Sync {
    fn shared(&self) -> BoxFuture<'_, Result<SharedPermit, PortError>>;
    fn exclusive(&self) -> BoxFuture<'_, Result<ExclusivePermit, PortError>>;
}
```

Die Permits sind RAII-Typen dieses Crates. Ihr Drop gibt den Slot frei. Die Methoden liefern `Result`. Ein vergifteter Barrier-Zustand ist ein Fehler, kein Erfolg.

### 4.2 `PurgeBarrier`

`compaction` und `purge` schließen sich aus. `purge_waiting` ist ein nicht blockierendes Signal an laufende Kompaktionen, den Slot freizugeben. Der exklusive Purge gilt für die Purge-Compaction in Schritt 8.

```rust
pub trait PurgeBarrier: Send + Sync {
    fn compaction(&self) -> BoxFuture<'_, Result<CompactionPermit, PortError>>;
    fn purge(&self) -> BoxFuture<'_, Result<PurgePermit, PortError>>;
    fn purge_waiting(&self) -> bool;
}
```

## 5. Schlüsselanker (SPEC §4.3)

```rust
pub enum Custody {
    K1,
    K2,
    K3,
}

pub struct WrappedBlob {
    pub bytes: Vec<u8>,
}

pub struct SlotGeneration(pub u64);

pub struct DestroyAttestation {
    pub custody: Custody,
    pub generation: SlotGeneration,
    pub at_unix_nanos: u64,
}

pub trait EffaceableKeySlot: Send + Sync {
    fn custody(&self) -> Custody;
    fn wrap(&self, material: &Zeroizing<[u8; 32]>, ad: &[u8]) -> Result<WrappedBlob, KeyError>;
    fn unwrap(&self, blob: &WrappedBlob, ad: &[u8]) -> Result<Zeroizing<[u8; 32]>, KeyError>;
    fn rotate(&self) -> Result<SlotGeneration, KeyError>;
    fn destroy(&self, generation: SlotGeneration) -> Result<DestroyAttestation, KeyError>;
}

pub trait KeyAnchor: EffaceableKeySlot {}
```

`Custody::K0` ist kein Slot. K0 bedeutet, dass kein `EffaceableKeySlot` gesetzt ist und Klasse C unerreichbar bleibt. K2 und K3 geben `KeyError::CapabilityUnsupported` zurück, solange die Konformitätssuite fehlt. Der Zeitstempel in `DestroyAttestation` kommt vom Aufrufer aus `Clock`. Dieses Crate liest keine Systemuhr.

`KeyError` ist `#[non_exhaustive]`: `WrapFailed`, `UnwrapFailed`, `CapabilityUnsupported`, `GenerationUnknown`.

## 6. `Purgeable` (Schicht S4)

Caches, die Nutzdaten oder dokumentbezogene Zwischenwerte halten, implementieren diesen Vertrag. Die Saga ruft ihn in Schritt 3 auf. Die Implementierung invalidiert den Scope und zeroized die Werte. Ein leeres `Ok(())` ohne Invalidierung ist ein Stub und verboten (INV-WIRING-2).

```rust
pub trait Purgeable: Send + Sync {
    fn invalidate(&self, doc_key: &[u8]) -> Result<(), PortError>;
}
```

Bekannte Implementierungen: Block-Cache in `contextra-store`, `er_cache` in `contextra-rank`, Surrogat-Vault in `contextra-privacy`. `contextra-db` registriert sie beim Koordinator.

## 7. `EmbeddingEngine`

Der Port bleibt für einen vom Betreiber gelieferten Embedder. Die entfernten Inferenz-Crates werden hier nicht erwähnt und nicht importiert (SPEC §2.2, Gate `G-REACTIVATE`).

```rust
pub trait EmbeddingEngine: Send + Sync {
    fn embed_query(&self, text: &str) -> BoxFuture<'_, Result<Vec<f32>, PortError>>;
    fn embed_batch(&self, texts: &[&str]) -> BoxFuture<'_, Result<Vec<Vec<f32>>, PortError>>;
    fn dimension(&self) -> usize;
}
```

Embeddings personenbezogener Texte sind Klasse D und brauchen eine `LineageRef`. Die Pflicht prüft `contextra-db`, nicht dieser Port.

## 8. Fehler (`PortError`)

`#[non_exhaustive]`: `Storage`, `Index`, `Barrier`, `Timeout`, `Key(KeyError)`. Es gibt keine Varianten `OnGateFailed` oder `WitnessFailed`. Diese Fehler leben in `ErasureError`.

## 9. `Cargo.toml`

```toml
[package]
name = "contextra-ports"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Port traits: storage, indexes, barriers, and the effaceable key slot"

[lints]
workspace = true

[dependencies]
contextra-env = { workspace = true }
contextra-ids = { workspace = true }
futures-util = { version = "0.3", default-features = false }
thiserror = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }

[features]
default = []
```

## 10. Abnahme

1. Kein Typ aus SPEC §2.5, dessen Besitzer `contextra-erasure` oder `contextra-types` ist, wird hier definiert.
2. `cargo metadata` zeigt keine Kante auf `contextra-types`.
3. `Purgeable::invalidate` hat keine Implementierung, die ohne Seiteneffekt `Ok(())` liefert.
