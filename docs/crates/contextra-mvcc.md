# contextra-mvcc — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env` (E0-Fundamente, max. 5)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-mvcc`  
**Referenzdokumente:** `docs/SPEC.md` §2, §5.3 (Schritt 4a), §9; `ANALYSIS-E0-CONTRADICTIONS.md`  

---

## 1. Zweck & Architekturgarantien

`contextra-mvcc` implementiert das Multi-Version Concurrency Control (MVCC) Subsystem für Contextra:
- **Snapshot-Isolation:** Konsistente point-in-time Lesezugriffe auf Basis von Sequenznummern (`Seq`), ohne nebenläufige Schreiber zu blockieren.
- **Pin-Verwaltung (`SnapshotRegistry`):** Fadenfeste Nachverfolgung aktiver Leser zur Ermittlung der minimalen aktiven Sequenznummer (`min_active_seqno`) für die SSTable-Kompaktion.
- **Snapshot-Widerruf (DSGVO Phase 4a):** Zwanghafte Entwertung aktiver Snapshots bei Löschungen (`revoke_before`), damit alte Pins physische Purges nicht blockieren.
- **Transaktions-Puffer (`TxBuffer`):** Vorhaltung lokaler Transaktions-Writes vor dem Atomic-Commit ins WAL/Memtable.

### Beseitigung von Ebenenverstößen (R1)
Die optionalen Abhängigkeiten auf `contextra-types` und `contextra-ports` sowie die Features `workspace`, `types-compat` und `ports-compat` werden vollständig entfernt. `contextra-mvcc` operiert als reiner E1-Baustein ausschließlich auf `contextra-ids` und `contextra-env`.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-9)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Alle Fehlerpfade liefern `MvccError`.
3. **Leere Default-Features (R3):** `default = []`.
4. **Monotonie:** Sequenznummern steigen streng monoton; kein Zurückspringen.
5. **RAII Pin-Freigabe:** Beim Droppen von `SnapshotPin` wird der Pin verlässlich aus der Registry ausgetragen.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────┐
│                        contextra-mvcc                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                  SnapshotRegistry                   │   │
│   │                                                     │   │
│   │   Sequenz-Achse:                                    │   │
│   │   [seq=100] ──► [seq=105 (Pin A)] ──► [seq=110]     │   │
│   │                   │                                 │   │
│   │                   ▼                                 │   │
│   │         min_active_seqno = 105                      │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ SnapshotPin (RAII) │    │ Revocation Table  │   │   │
│   │   │                    │    │ (DSGVO Phase 4a)  │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Kanonische Schnittstellen & Typen

### 3.1 `SnapshotRegistry` & `SnapshotPin`

```rust
use std::sync::Arc;
use parking_lot::RwLock;
use contextra_ids::Seq;

/// RAII-Guard für einen aktiven Snapshot-Pin.
pub struct SnapshotPin {
    seq: Seq,
    registry: Arc<SnapshotRegistry>,
}

impl SnapshotPin {
    #[inline]
    pub fn seq(&self) -> Seq {
        self.seq
    }

    /// Prüft, ob dieser Snapshot durch einen DSGVO-Löschauftrag entwertet wurde.
    pub fn validate(&self) -> Result<(), MvccError> {
        if self.registry.is_revoked(self.seq) {
            Err(MvccError::SnapshotRevoked { seq: self.seq })
        } else {
            Ok(())
        }
    }
}

impl Drop for SnapshotPin {
    fn drop(&mut self) {
        self.registry.release(self.seq);
    }
}

pub struct SnapshotRegistry {
    state: RwLock<RegistryState>,
}

struct RegistryState {
    active_pins: Vec<Seq>,
    revoked_up_to: Seq,
}

impl SnapshotRegistry {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(RegistryState {
                active_pins: Vec::new(),
                revoked_up_to: Seq::ZERO,
            }),
        }
    }

    /// Registriert einen neuen Snapshot-Leser für die angegebene Sequenz.
    pub fn acquire(self: &Arc<Self>, seq: Seq) -> Result<SnapshotPin, MvccError> {
        let mut state = self.state.write();
        if seq <= state.revoked_up_to {
            return Err(MvccError::SnapshotRevoked { seq });
        }
        state.active_pins.push(seq);
        Ok(SnapshotPin {
            seq,
            registry: Arc::clone(self),
        })
    }

    /// Ermittelt die älteste noch aktive Sequenznummer aller Leser.
    /// Wird von contextra-store genutzt, um alte SSTable-Versionen freizugeben.
    pub fn min_active_seqno(&self, current_head: Seq) -> Seq {
        let state = self.state.read();
        state.active_pins.iter().copied().min().unwrap_or(current_head)
    }

    /// DSGVO Schritt 4a: Widerruft alle Snapshots bis einschließlich `gated_seq`.
    pub fn revoke_before(&self, gated_seq: Seq) -> usize {
        let mut state = self.state.write();
        if gated_seq > state.revoked_up_to {
            state.revoked_up_to = gated_seq;
        }
        let initial_count = state.active_pins.len();
        state.active_pins.retain(|&s| s > gated_seq);
        initial_count - state.active_pins.len()
    }

    #[inline]
    pub fn is_revoked(&self, seq: Seq) -> bool {
        let state = self.state.read();
        seq <= state.revoked_up_to
    }

    fn release(&self, seq: Seq) {
        let mut state = self.state.write();
        if let Some(pos) = state.active_pins.iter().position(|&s| s == seq) {
            state.active_pins.swap_remove(pos);
        }
    }
}
```

---

## 4. Fehlerbehandlung (`MvccError`)

```rust
use contextra_ids::Seq;

#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum MvccError {
    #[error("Snapshot at sequence {seq} was revoked by GDPR erasure operation (Phase 4a)")]
    SnapshotRevoked { seq: Seq },

    #[error("Write-write conflict: key already modified by concurrent transaction at seq {seq}")]
    WriteConflict { seq: Seq },

    #[error("Transaction serialization failure in SSI validation")]
    SerializationFailure,
}
```

---

## 5. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-mvcc"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Multi-Version Concurrency Control (MVCC) and snapshot isolation for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
parking_lot = { workspace = true }
ahash = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"
loom = "0.7"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
```

---

## 6. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports`.
   - Entfernen der Features `workspace`, `types-compat`, `ports-compat`.
   - `default = []` setzen (R3).
2. **Kanonisierung `SnapshotRevoker`:**
   - Bereitstellung von `revoke_before(gated_seq: Seq) -> usize`.
   - Valide Fehlerbehandlung mit `MvccError::SnapshotRevoked`.
3. **Verifikation:**
   - `cargo test -p contextra-mvcc`
