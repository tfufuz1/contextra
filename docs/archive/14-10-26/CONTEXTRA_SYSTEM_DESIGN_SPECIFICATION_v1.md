superseded by CONTEXTRA_ENDPRODUKT_SPEZIFIKATION.md

# Contextra — Technische Systemdesign- und Implementierungsspezifikation (v1)

**Datum:** 2026-10-14 · **Stand:** HEAD `500e9246` (Merge PR #4785)  
**Rolle:** Principal Senior Rust Architect für Contextra  
**Status:** Normative technische Spezifikation für das gesamte Multi-Crate-Ökosystem  
**Zweck:** Vollständige Spezifikation aller Architektur- und Modul-Schnittstellen, Datenstrukturen, Invarianten und Algorithmen zur Behebung aller identifizierten Schwachstellen und zur maximalen Entkopplung des Systems.

---

## Inhaltsverzeichnis

1. [Grundsatz-Architektur & Zero-Cost-Plugin-Modell](#1-grundsatz-architektur--zero-cost-plugin-modell)
2. [Port-Spezifikation: `contextra-ports::erasure`](#2-port-spezifikation-contextra-portserasure)
3. [Löschnachweis-Plugin-Architektur (Off, Assured, Attested)](#3-löschnachweis-plugin-architektur-off-assured-attested)
4. [Purge v2 & Lockless Fence-Protokoll (Option B)](#4-purge-v2--lockless-fence-protokoll-option-b)
5. [Crate-Entkopplung, Feature-Matrix & Produkt-Manifeste](#5-crate-entkopplung-feature-matrix--produkt-manifeste)
6. [Beseitigung der Struktur- und Typ-Schulden (Downcasts, `core`, Shims)](#6-beseitigung-der-struktur--und-typ-schulden-downcasts-core-shims)
7. [Geschäftslogik-Konsolidierung (Delete, MCP, Egress, Builder)](#7-geschäftslogik-konsolidierung-delete-mcp-egress-builder)
8. [MVCC, SSI & Concurrency-Spezifikation](#8-mvcc-ssi--concurrency-spezifikation)
9. [Wire-Format, Serialisierung & TLV-Layout](#9-wire-format-serialisierung--tlv-layout)
10. [Phasengenaue Ausführungs- und Test-Matrix (K0–K5)](#10-phasengenaue-ausführungs--und-test-matrix-k0k5)

---

## 1. Grundsatz-Architektur & Zero-Cost-Plugin-Modell

### 1.1 Das Paradoxon von Performance und Löschnachweis
Contextra verspricht **einerseits** ultra-performanten, eingebetteten Vektor- und Hybridspeicher (Latenz im Sub-Millisekundenbereich, maximale Schreib-/Lese-Throughputs auf Single-Node) und **andererseits** kryptografisch beweisbare, physische Datenvernichtung (DSGVO Art. 17, ZPO § 371a, Ed25519-Zeugen).

Im bisherigen Entwurf führte die Kopplung des Löschnachweises zu:
- Unnötigen Synchronisations- und Allokationskosten im Hot-Path.
- Hart verdrahteten Abhängigkeiten von Speicher-Engines auf Krypto- und Filesystem-Module.
- Ausfall des Standard-Löschpfads (`CapabilityUnsupported`), wenn kein kryptografischer Beweiser konfiguriert war.

### 1.2 Die Entkopplungsdoktrin: Drei-Schichten-Entkopplung
1. **Compile-Zeit (Feature-Gating):**  
   Ohne das Feature `erasure` existieren im Crate `contextra-store` und `contextra-engine` keinerlei Datenstrukturen oder Abhängigkeiten zu Beweis-Algorithmen. `ErasureGate` ist ein Zero-Sized Type (`ZST`), der vom Rust-Compiler vollständig wegoptimiert wird.
2. **Laufzeit-Entkopplung (Mode-Dispatch):**  
   Ist das Feature einkompiliert, entscheidet der persistente `ErasureMode` im Manifest:
   - `ErasureMode::Off`: Null Kosten im Schreibpfad, Standard-Tombstones im Speicher.
   - `ErasureMode::Assured`: Physisches Purging + Prefix-Fence, Anti-Resurrection-Ledger, Crash-Resume; jedoch **keine** kryptografische Signatur und keine Layer-Scans.
   - `ErasureMode::Attested`: Echte Layer-Scans, Zeugenerzeugung, Ed25519-Signierung, v4-TLV-Export.
3. **Typ-Entkopplung (Port-Pattern):**  
   Weder `contextra-store` noch `contextra-engine` kennen konkrete Krypto-Crates. Sie kommunizieren ausschließlich über `contextra-ports::erasure`.

```
               ┌────────────────────────────────────────────────────────┐
               │                  contextra-ports                        │
               │  Traits: ErasureBackend, ErasureHooks, LayerScan, ...   │
               └───────────▲───────────────────────────────▲────────────┘
                           │                               │
        ┌──────────────────┴─────────────┐    ┌────────────┴─────────────────┐
        │        Kern-System             │    │       Erasure-Plugins        │
        │ contextra-store (Ring 1)       │    │ contextra-erasure (Assured)  │
        │ contextra-engine (Ring 3)      │    │ contextra-evidence (Attested)│
        │ contextra-vector (Ring 0)      │    │ contextra-verify (Standalone)│
        └────────────────────────────────┘    └──────────────────────────────┘
```

---

## 2. Port-Spezifikation: `contextra-ports::erasure`

Die Schnittstelle zwischen Kern und Plugins wird in [`crates/contextra-ports/src/erasure.rs`](../../../crates/contextra-ports/src/erasure.rs) als normative Quelle definiert.

### 2.1 Modus und Typen

```rust
use std::sync::Arc;
use std::time::Duration;
use crate::{ContextraError, TxId};

/// Persistente Schutzklasse für Löschgarantien.
/// Bildet eine monotone Hochwassermarke im LSM-Store-Manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ErasureMode {
    /// Kein Löschnachweis, kein Purge-Zwang. Tombstones + normale LSM-Compaction.
    Off = 0,
    /// Garantierte physische Löschung via Rewrite-Purge, Fence und Anti-Resurrection-Ledger.
    Assured = 1,
    /// Assured + kryptografische Zeugen aus echten Scans, signierter v4-TLV-Beweis.
    Attested = 2,
}

impl Default for ErasureMode {
    #[inline(always)]
    fn default() -> Self {
        Self::Off
    }
}

/// Opaque physische Adressierungsmenge für Lösch- und Fence-Vorgänge.
///
/// SICHERHEITSINVARIANTE (INV-ERASURE-3):
/// Konstruktion ist ausschließlich durch `ErasureBackend::physical_scope` möglich.
/// Dadurch wird der Fehler N-K (Nichtübereinstimmung logischer und mandantenkodierter
/// physischer Schlüssel) strukturell zur Compile-Zeit unmöglich gemacht.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalScope {
    prefixes: Vec<Vec<u8>>,
    exact_keys: Vec<Vec<u8>>,
}

impl PhysicalScope {
    /// Interner Konstruktor für Storage-Backends.
    #[inline]
    pub fn from_backend_unchecked(prefixes: Vec<Vec<u8>>, exact_keys: Vec<Vec<u8>>) -> Self {
        Self { prefixes, exact_keys }
    }

    #[inline]
    pub fn prefixes(&self) -> &[Vec<u8>] {
        &self.prefixes
    }

    #[inline]
    pub fn exact_keys(&self) -> &[Vec<u8>] {
        &self.exact_keys
    }

    #[inline]
    pub fn matches_key(&self, key: &[u8]) -> bool {
        self.exact_keys.iter().any(|k| k.as_slice() == key)
            || self.prefixes.iter().any(|p| key.starts_with(p.as_slice()))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.prefixes.is_empty() && self.exact_keys.is_empty()
    }
}

/// Logische Spezifikation des Lösch-Ziels durch den Aufrufer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalScope {
    Document { collection: String, doc_id: String },
    Collection { collection: String },
    Tenant { tenant_id: u64 },
}
```

### 2.2 Storage-Backend-Port (Eliminierung von `downcast_ref::<LsmStorage>`)

Bisher nutzten 5 Stellen im Workspace `downcast_ref::<LsmStorage>()`. Dies verletzte P5 und koppelte Ring 3 an Ring 1. Die Abstraktion erfolgt über `ErasureBackend`:

```rust
/// Abstraktion des Speicher-Backends für Purge- und Fence-Operationen.
pub trait ErasureBackend: Send + Sync {
    /// Übersetzt einen logischen Anwendungs-Scope in den tenant-isolierten physischen Scope.
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope, ContextraError>;

    /// Führt den physischen Rewrite-Purge synchron aus.
    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of_tx: TxId,
        probe: &dyn FreeSpaceProbe,
    ) -> Result<PurgeExecutionOutput, PurgeError>;

    /// Erwirbt einen Write-Fence auf den physischen Scope.
    fn acquire_fence(&self, scope: &PhysicalScope, purge_id: [u8; 16]) -> Result<FenceLease, ContextraError>;
}

/// Erweiterung für StorageEngine zur Entdeckung des ErasureBackends ohne Downcasts.
pub trait StorageErasureExt {
    fn erasure_backend(&self) -> Option<&dyn ErasureBackend>;
}
```

### 2.3 Plugin-Lifecycle-Hooks (`ErasureHooks`)

```rust
/// Eindeutiges Token einer begonnenen Löschtransaktion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErasureToken(pub u64);

/// Kontextinformationen für das Plugin.
pub struct ErasureContext<'a> {
    pub logical_scope: &'a LogicalScope,
    pub physical_scope: &'a PhysicalScope,
    pub mode: ErasureMode,
    pub tx_id: TxId,
}

/// Ergebnis der Attestierung nach erfolgreichem Purge.
#[derive(Debug)]
pub enum DeletionAttestation {
    None,
    Attested(Arc<[u8]>),
}

/// Lifecycle-Hooks für Plugins (Assured / Attested).
pub trait ErasureHooks: Send + Sync + 'static {
    /// Aufruf vor Beginn der Löschung (Schritt 1: Ledger-Append + Intent).
    fn on_erasure_begin(&self, ctx: &ErasureContext<'_>) -> Result<ErasureToken, ContextraError>;

    /// Aufruf nach erfolgreichem Tombstone-Commit (Schritt 2).
    fn on_tombstone_committed(&self, token: ErasureToken, tx_id: TxId) -> Result<(), ContextraError>;

    /// Aufruf nach physischem Purge zur Zeugenerhebung (Schritt 3).
    fn on_physical_purged(
        &self,
        token: ErasureToken,
        scanners: &[&dyn LayerScan],
    ) -> Result<DeletionAttestation, ContextraError>;

    /// Aufruf bei Fehler zur Bereinigung oder Intent-Markierung.
    fn on_erasure_failed(&self, token: ErasureToken, error: &ContextraError);
}
```

---

## 3. Löschnachweis-Plugin-Architektur (Off, Assured, Attested)

### 3.1 Das Zero-Cost-Monomorphisierungs-Pattern

Um bei `ErasureMode::Off` oder in Builds ohne `erasure`-Feature zero Runtime-Overhead zu garantieren, wird in `contextra-engine` folgender Dispatch-Mechanismus spezifiziert:

```rust
pub struct CollectionErasureDriver {
    mode: ErasureMode,
    hooks: Option<Arc<dyn ErasureHooks>>,
}

impl CollectionErasureDriver {
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.mode != ErasureMode::Off && self.hooks.is_some()
    }

    #[inline]
    pub async fn execute_delete<S: ErasureBackend>(
        &self,
        backend: &S,
        scope: LogicalScope,
        tx_id: TxId,
    ) -> Result<DeleteOutcome, ContextraError> {
        // Fast-Path: Zero-Cost bei Off
        if !self.is_active() {
            return Ok(DeleteOutcome::Logical { tx: tx_id });
        }

        let hooks = self.hooks.as_ref().expect("checked above");
        let physical = backend.physical_scope(&scope)?;
        
        let ctx = ErasureContext {
            logical_scope: &scope,
            physical_scope: &physical,
            mode: self.mode,
            tx_id,
        };

        // 1. Phasen-Koordination
        let token = hooks.on_erasure_begin(&ctx)?;
        
        // 2. Tombstone wurde bereits in Collection committet -> Bestätigen
        hooks.on_tombstone_committed(token, tx_id)?;

        // 3. Physischer Purge (im Hintergrund oder synchron je nach Policy)
        // ...
        Ok(DeleteOutcome::Assured { tx: tx_id, purge_handle: PurgeHandle::new(token.0) })
    }
}
```

---

## 4. Purge v2 & Lockless Fence-Protokoll (Option B)

### 4.1 Ursachen der Defekte im Ist-Zustand
Die Deep-Audits identifizierten fundamentale Fehler im LSM-Purge-Pfad:
1. **L5:** `while let Ok(Some(..))` brach bei I/O-Fehlern still ab. Unvollständige Sweeps galten als erfolgreich.
2. **L4:** `dispose()` übergab `Vec::new()`, wodurch veraltete SSTables mit intakten Payload-Daten auf der Festplatte verblieben.
3. **L6:** Der neugeschriebene Reader verlor seinen relativen Sequenz-Index, wodurch ältere Versionen gelöschter Keys wieder sichtbar wurden.
4. **N-A:** Nach einem Absturz lud `resume_pending()` ein leeres `OrxHooks::default()`, verwarf den Scope und löschte das Intent-File ohne Datenbereinigung (fail-open).
5. **N-K:** Der Purge filterte nach logischen Keys (`doc_123`), auf der Platte lagen jedoch tenant-kodierte Keys (`t:1:doc_123`). Trefferquote = 0%.

### 4.2 Die Purge v2 Spezifikation

In `crates/contextra-store/src/lsm/ops/purge.rs` gilt ab sofort folgende strikte Implementierungsformel:

```rust
pub fn purge_keys_v2<P: FreeSpaceProbe>(
    manifest: &ManifestContext,
    scope: &PhysicalScope,
    as_of_tx: TxId,
    probe: &P,
) -> Result<PurgeExecutionOutput, PurgeError> {
    // 1. Kapazitätsprüfung (L8-Fix: None = Unbekannt, konservativ; Some(0) = Abbruch)
    match probe.available_bytes() {
        Some(bytes) if bytes < MIN_PURGE_HEADROOM_BYTES => {
            return Err(PurgeError::InsufficientDiskSpace {
                needed: MIN_PURGE_HEADROOM_BYTES,
                available: Some(bytes),
            });
        }
        _ => {}
    }

    let mut obsolete_sstables = Vec::new();
    let mut rewritten_sstables = Vec::new();

    // 2. SSTable-Sweeper mit striktem 3-Arm-Matching (L5-Fix)
    for sstable in manifest.active_sstables() {
        if !sstable.may_contain_scope(scope) {
            continue;
        }

        let mut reader = sstable.open_reader()?;
        let mut writer = sstable.create_replacement_writer()?;
        let mut entries_purged = 0u64;

        loop {
            match reader.read_next_entry() {
                Ok(Some(entry)) => {
                    if scope.matches_key(entry.key()) && entry.sequence() <= as_of_tx {
                        // Eintrag gezielt überspringen (Purge)
                        entries_purged += 1;
                        continue;
                    }
                    // Erhaltener Eintrag wird gestreamt (N-F Fix)
                    writer.append_entry(&entry)?;
                }
                Ok(None) => break, // Reguläres EOF
                Err(io_err) => {
                    // L5-Fix: Fail-Closed! Keine stillen Abbrüche!
                    writer.abort()?;
                    return Err(PurgeError::ReadError {
                        file: sstable.path().to_string_lossy().into_owned(),
                        offset: reader.current_offset(),
                        source: io_err,
                    });
                }
            }
        }

        if entries_purged > 0 {
            let new_sst = writer.finalize_with_inherited_rank(sstable.rank(), sstable.seq_range())?; // L6-Fix
            rewritten_sstables.push(new_sst);
            obsolete_sstables.push(sstable.path().to_path_buf()); // L4-Fix
        } else {
            writer.abort()?;
        }
    }

    // 3. Atomares Manifest-Commit
    manifest.commit_sstable_replacement(&obsolete_sstables, &rewritten_sstables)?;

    // 4. Physisches Überschreiben und Löschen (L4-Fix & Dispose)
    dispose_obsolete_files(&obsolete_sstables, true)?;

    Ok(PurgeExecutionOutput {
        purged_files_count: obsolete_sstables.len(),
        obsolete_files: obsolete_sstables,
    })
}
```

### 4.3 Lockless Fence-Protokoll (Option B)

Um den Schreibpfad bei laufendem oder inaktivem Purge maximal schnell zu halten, wird der Write-Fence in `contextra-store/src/lsm/fence.rs` als RCU-artiges Gate realisiert.

#### Speicherstruktur
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use arc_swap::ArcSwap;

#[derive(Clone)]
struct ActiveFence {
    scope: PhysicalScope,
    purge_id: [u8; 16],
    ref_count: usize,
}

pub struct ErasureGate {
    /// Schneller Check im Hot-Path. Ist false, wenn keine Fences aktiv sind.
    has_active_fences: AtomicBool,
    /// Atomar austauschbare Tabelle aktiver Fences.
    fences: ArcSwap<Vec<ActiveFence>>,
}
```

#### Integration in `commit_internal()` (`ops/write.rs`)
Der Gruppen-Commit-Leader führt die Prüfung **nach** der Wiederaufnahme des `commit_mutex` durch:

```rust
// FAST PATH: Ein einziger Atomic-Load (P-E3 Invariante)
if self.erasure_gate.has_active_fences.load(Ordering::Acquire) {
    let active_fences = self.erasure_gate.fences.load();
    for op in batch.operations() {
        for fence in active_fences.iter() {
            if fence.scope.matches_key(op.key()) {
                return Err(StoreError::WriteFenced {
                    key: op.key().to_vec(),
                    purge_id: fence.purge_id,
                });
            }
        }
    }
}
```

---

## 5. Crate-Entkopplung, Feature-Matrix & Produkt-Manifeste

### 5.1 Crate-Split für `contextra-crypto`
Das heutige `contextra-crypto` (~9.300 LOC) verletzt die Ring-0-Invariante durch Abhängigkeiten zu `contextra-durable-fs` und Dateisystem-I/O.

Es wird verbindlich in zwei Einheiten aufgespalten:
1. **`contextra-crypto-core` (Ring 0, pure numerics, I/O-frei):**
   - Enthält: KeyManager, KDF, Nonces, ChaCha20/AES-GCM-SIV, Ed25519-Signer/Verifier, BLAKE3-Präfixe, Zeroize-Puffer.
   - Abhängigkeiten: `contextra-types`, `contextra-ports`. **Kein `durable-fs`, kein `std::fs`**.
2. **`contextra-crypto-kv` (Ring 0 / Crypto-Erase):**
   - Enthält: Segment-Verschlüsselung, `RevocationLog`, KV-Shredding.
   - Abhängigkeiten: `contextra-crypto-core`, `contextra-durable-fs`.
   - Feature: `kv-encryption`.

### 5.2 Produkt-Manifeste (`products/*.toml`)

Ein Produkt wird formal durch ein Manifest in `products/` definiert. Das neue Gate `cargo xtask check-product-hull --product <ID>` verifiziert, dass keine Rand-Crates in den Dependency-Baum gelangen.

#### Manifest für EP1 (`products/ep1.toml`)
```toml
[product]
id = "EP1"
name = "Contextra Memory"
status = "release-1.0"
target_binary = "contextra-mcp"

[crates]
allowed = [
    "contextra-types",
    "contextra-ports",
    "contextra-sys",
    "contextra-wire",
    "contextra-simd",
    "contextra-durable-fs",
    "contextra-crypto-core",
    "contextra-mvcc",
    "contextra-text",
    "contextra-rank",
    "contextra-store",
    "contextra-vector",
    "contextra-checkpoint",
    "contextra-engine",
    "contextra-db",
    "contextra-license",
    "contextra",
    "contextra-mcp",
]

forbidden = [
    "contextra-graph",
    "contextra-cognition",
    "contextra-adapt",
    "contextra-router",
    "contextra-agent",
    "contextra-privacy",
    "contextra-sandbox",
    "contextra-infer-ollama",
    "contextra-infer-onnx",
    "contextra-audit-export",
    "contextra-avv-generator",
    "contextra-core",
]

[features]
required_default_empty = [
    "graph",
    "cognition",
    "adapt",
    "router",
    "privacy",
    "sandbox",
]
```

---

## 6. Beseitigung der Struktur- und Typ-Schulden

### 6.1 Beseitigung von `contextra-core` (Strangler-Muster)
`contextra-core` (123 LOC) ist eine deprecated Re-Export-Hülle.
- **Ist-Zustand:** 110 Quellcode-Dateien nutzen `use contextra_core::...`.
- **Umsetzungsplan:**
  1. Identifikation der 4 Ursprungs-Crates:
     - Domain-Typen & IDs → `contextra-types`
     - Trait-Ports → `contextra-ports`
     - MVCC & Sequenzen → `contextra-mvcc`
     - FlatBuffers/Wire → `contextra-wire`
  2. Batch-Ersetzung der Importe modulweise:
     - PR 1: `contextra-store` & `contextra-vector` (32 Dateien)
     - PR 2: `contextra-engine` & `contextra-db` (45 Dateien)
     - PR 3: Restliche Crates & Benchmarks (33 Dateien)
  3. Löschung von `crates/contextra-core` aus `Cargo.toml`.

### 6.2 Eliminierung aller Downcasts (`downcast_ref`)
Die 5 unzulässigen Stellen werden vollständig auf `StorageErasureExt` migriert:

| Datei | Bisheriger Code | Neuer Standard |
|---|---|---|
| `collection/mod.rs:405` | `storage.downcast_ref::<LsmStorage>()` | `storage.erasure_backend().ok_or(Unsupported)` |
| `collection/mod.rs:528` | `storage.downcast_ref::<LsmStorage>()` | `storage.erasure_backend().ok_or(Unsupported)` |
| `collection/mod.rs:532` | `storage.downcast_ref::<LsmStorage>()` | `storage.erasure_backend().ok_or(Unsupported)` |
| `crud/delete.rs:79` | `storage.downcast_ref::<LsmStorage>()` | `storage.erasure_backend().ok_or(Unsupported)` |
| `db_transaction_commit.rs:35` | `storage.downcast_ref::<LsmStorage>()` | `storage.erasure_backend().ok_or(Unsupported)` |

---

## 7. Geschäftslogik-Konsolidierung

### 7.1 Vereinheitlichung von `remember` / `recall` (Entscheidung D6)
- **Problem:** MCP definierte in `server_dispatch.rs` eigene Speicherpfade, abweichend von der Facade `contextra::ContextraDb`.
- **Spezifikation:** MCP ruft **ausschließlich** die öffentlichen Methoden der Fassade auf (`db.insert()`, `db.search()`, `db.delete()`). Jegliche doppelte Logik zur Vektorisierung oder Metadaten-Formatierung im MCP-Crate wird gelöscht.

### 7.2 Fail-Closed Egress-Gateway
- `contextra_cloud_query` wird im MCP-Server hinter `#[cfg(feature = "guard")]` gelegt.
- Ist das Feature nicht aktiv, wird das Tool im MCP-Tool-Katalog gar nicht registriert (`server_dispatch.rs`). Ein Aufruf scheitert mit `UnknownTool` (Fail-Closed).

### 7.3 Bereinigung der Builder-Methoden
In [`crates/contextra/src/builder.rs`](../../../src/builder.rs):
- `with_adapt(enabled)`: Verwirft nicht mehr den Parameter (`let _ = enabled`), sondern konfiguriert den Adapt-Controller oder liefert einen Fehler, falls das Feature fehlt.
- Jedes Builder-Feld muss ein korrespondierendes Feld in `EngineWiringConfig` erhalten.
- Beim Aufruf von `.build()` wird zwingend `verify_wiring(&config)` ausgeführt. Ist ein benötigter Port nicht vorhanden, bricht `.build()` mit `ConfigurationError` ab.

---

## 8. MVCC, SSI & Concurrency-Spezifikation

### 8.1 Behebung der Audit-Befunde
- **MVCC-S04:** Die veralteten Methoden `register()` und `register_at()` in `contextra-mvcc` werden endgültig entfernt. Alle Aufrufer nutzen `acquire()`.
- **MVCC-S05:** `recover_pending_intents` in `cleanup.rs` wird im Startup-Pfad `repair_on_open()` verdrahtet.

### 8.2 SSI vs. SSN (Serial Safety Net) Entscheidungskriterium
Der bestehende `SequenceLogSsiValidator` erfüllt die Cahill-Fekete-Bedingung. Er bleibt für Release 1.0 Primär-Standard.
SSN (Mechanismus 4.2 aus der Forschungsanalyse) wird **nicht** vor K4 implementiert.
Voraussetzung für einen Wechsel zu SSN:
1. Ein Property-Test weist nach, dass der aktuelle SSI-Validator bei parallelen Read-Heavy-Workloads mehr als 5% Fehl-Abbrüche (False Aborts) erzeugt.
2. Das Dyadik-Modell von SSN wird deterministisch im Single-Threaded-Replay nachgewiesen.

---

## 9. Wire-Format, Serialisierung & TLV-Layout

### 9.1 DeletionProof v4 TLV-Spezifikation
Alle Löschnachweise verwenden verbindlich das versionierte TLV-Format mit Domain-Tag. `bincode` ist im Signatur- und Nachweispfad verboten.

```
+-----------------------------------------------------------------------+
| Magic Domain Tag: "contextra/deletion-proof/v4\0" (32 Bytes)          |
+-----------------------------------------------------------------------+
| TLV Record 0x01: DeletionScope (Hash BLAKE3)                          |
+-----------------------------------------------------------------------+
| TLV Record 0x02: TxId (u64 Little-Endian)                             |
+-----------------------------------------------------------------------+
| TLV Record 0x03: Timestamp (RFC 3339 UTC Nano)                        |
+-----------------------------------------------------------------------+
| TLV Record 0x10: Array of LayerWitnesses                              |
|   - Sub-Tag 0x11: LayerId (u32)                                       |
|   - Sub-Tag 0x12: StateDigest (32 Bytes BLAKE3)                       |
|   - Sub-Tag 0x13: CleanAssertion (u8: 0x01 = clean)                   |
+-----------------------------------------------------------------------+
| TLV Record 0xFE: PublicKey (32 Bytes Ed25519)                         |
+-----------------------------------------------------------------------+
| TLV Record 0xFF: Signature (64 Bytes Ed25519)                         |
+-----------------------------------------------------------------------+
```

---

## 10. Phasengenaue Ausführungs- und Test-Matrix (K0–K5)

```
[Phase 0a: Build-Fixes (N-N, N-O, Lockfile)] 
       │
[Phase 0b: N-B Hotfix (Delete ohne Capability-Fehler)]
       │
[Phase 0d/0e: Purge v2 & Ehrliche Zeugenerzeugung (L1, L4, L5, L6, L8, N-A, N-K)]
       │
[Phase K0: Gate-Wahrheit, capabilities.toml, Unwrap-Baseline]
       │
[Phase K1: Fail-Closed Enforcing & Builder-Wiring]
       │
[Phase K2: Kanten schneiden, core-Entfernung, Hülle ≤ 18]
       │
[Phase K3: Rohträger-Test & Vollständiger Löschnachweis]
       │
[Phase K4: Crash-Konsistenz, Loom & Release v0.1.0]
```

### 10.1 Verbindliche Abnahmekriterien für Phase 0a / 0b / 0d
1. `cargo check --locked -p contextra-engine` kompiliert ohne Warnungen.
2. `cargo check --locked -p contextra --features router` kompiliert.
3. Test `test_purge_correctness` sweepet Marker-Bytes physisch und besteht den BTreeMap-Differenztest.
4. `Collection::delete()` auf einem `TenantScopedStorage` läuft fehlerfrei durch und gibt `DeleteOutcome` zurück.

---

> **Ergebnis:** Mit dieser Systemdesign-Spezifikation sind alle architektonischen Freiheitsgrade geschlossen. Entwickler und Agenten implementieren exakt nach den hier festgeschriebenen Signaturen, Traits und Invarianten.
