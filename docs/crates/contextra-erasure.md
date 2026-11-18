# contextra-erasure — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Autarkes Fundament / Ring 0 — Löschkern)  
**Kern-Closure:** `ids`, `env`, `durable-fs`, `privacy` (E0-Fundamente, keine E1/E2/E3-Abhängigkeiten)  
**Status:** Normativ (Soll-Zustand)  
**Version:** 2.1  
**Geltungsbereich:** `crates/contextra-erasure`  
**Referenzdokumente:** `docs/SPEC.md` §2.4, §3, §4, §5, §6, §7, §10, §11, §12, §13  

---

## 1. Zweck & Architekturgarantien

`contextra-erasure` ist der zentrale, formale **Lösch- und Quarantänekern** von Contextra (Art. 17 & Art. 18 DSGVO). Es garantiert die rechtskonforme, kryptographisch unwiderrufliche und messbare Löschung personenbezogener Daten:
- **Zustandsmaschine der 6-Schritte-Löschsaga:** Idempotente, wiederholbare Orchestrierung mit 4 Pflicht-Fsync-Ankern.
- **In-Memory $O(1)$ ReadGate (`SagaGate`):** Synchrone, latenzfreie Lesesperre auf dem Hot-Path vor jeder Entschlüsselung (Fail-Closed).
- **Key-Revocation & Crypto-Shredding:** Atomare Vernichtung von Schlüsselslots im Vault; Verifikation der Unlesbarkeit.
- **Messung mit Positivkontrolle:** Typestate-gestützter Nachweis (`Scanner<Armed>`), dass keine Datenrückstände verbleiben (`residue_matches == 0`).
- **Beweisführung & Audit:** Ed25519-signierter Löschbericht und kryptographisch hash-verkettetes Ledger außerhalb des Datenverzeichnisses.

### E0-Architekturregeln für dieses Crate

1. **E0-Fundament-Status:** `contextra-erasure` liegt auf Ebene E0. Es darf **keine** Abhängigkeiten auf E1 (`crypto`, `store`, `vector` usw.) oder E2 (`types`, `ports`) haben.
2. **Kanonische Typbesitzer:** `ErasureId`, `GateId`, `DocId`, `SubjectRef`, `PersonalRef`, `TenantId`, `CollectionId`, `Seq`, `Timestamp` stammen aus `contextra-ids`. Keine lokalen Kopien.
3. **Kanonischer `SagaState`:** `KeyRevoked` ist der normative Zustandsname. `KeyDestroyed` existiert nicht. Alle Code-Vorkommen von `KeyDestroyed` sind zu ersetzen.
4. **Keine Krypto-Duplikate:** `aes-gcm-siv`, `sha2`, `hmac` dürfen **nicht** in den `[dependencies]` von `contextra-erasure` stehen. Kryptographische Primitives gehören ausschließlich in `contextra-crypto` (E1). In `contextra-erasure` verbleibt nur: Vault-Slot-Management (Zeroize), optional Ed25519-Signierung (feature `signing`).
5. **ErasureLayer ist E0:** Das Trait `ErasureLayer` wird in diesem Crate definiert und von E1-Crates **implementiert**. `contextra-erasure` kennt keine konkreten E1-Implementierungen.

### Architekturgarantien (R1–R6, INV-9, INV-10, INV-11, INV-12)
1. **`#![forbid(unsafe_code)]`:** Vollständig speichersicher.
2. **Zero-Panic-Policy:** Keine Panics; alle Fehlerpfade nutzen `ErasureError`.
3. **Leere Default-Features (R3):** `default = []`.
4. **INV-9 (Terminale Zustände):** `Completed`, `Refused` und `Lifted` sind endgültig.
5. **INV-10 (Gate-Irreversibilität):** Ein einmal gesetztes `ReadGate` wird im regulären Betrieb niemals zurückgenommen.
6. **INV-11 (Zeroize):** Schlüsselspeicher werden vor dem Drop mit `0x00` überschrieben.
7. **INV-12 (PII-Freiheit):** Logs, Traces, Receipts und signierte Berichte enthalten niemals Klartext-PII oder Inhalte des Löschziels.

---

## 2. Architektur & Subsysteme

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              contextra-erasure                              │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                             SagaMachine                             │   │
│   │              (Zustandsautomat & Idempotenzsteuerung)                │   │
│   └───────────────┬─────────────────┬─────────────────┬─────────────────┘   │
│                   │                 │                 │                     │
│         ┌─────────┴───────┐ ┌───────┴───────┐ ┌───────┴─────────┐           │
│         ▼                 ▼ ▼               ▼ ▼                 ▼           │
│   ┌───────────┐     ┌───────────┐     ┌───────────┐     ┌───────────┐       │
│   │ SagaGate  │     │SagaJournal│     │ SagaVault │     │SagaMeasure│       │
│   │   (O(1)   │     │ (WAL der  │     │(Shredding,│     │ (Scanner, │       │
│   │ ReadGate) │     │  Saga)    │     │ Zeroize)  │     │   Armed)  │       │
│   └───────────┘     └───────────┘     └───────────┘     └───────────┘       │
│                           │                                 │               │
│                           ▼                                 ▼               │
│                     ┌───────────┐                     ┌───────────┐         │
│                     │SagaLedger │                     │SagaReport │         │
│                     │(Chained   │                     │ (Ed25519  │         │
│                     │ Log ext.) │                     │  Signer)  │         │
│                     └───────────┘                     └───────────┘         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Zustandsautomat & Kanonische Typen

### 3.1 `SagaState` (SPEC §5.2, D-07)

```rust
use serde::{Deserialize, Serialize};

/// Die normativen Zustände eines Löschvorgangs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SagaState {
    /// 1. Löschauftrag erfasst und im Journal fixiert (fsync #1).
    Requested,

    /// 2. ReadGate aktiv: Alle Lesepfade liefern Fail-Closed `Erased` (fsync #2).
    Gated,

    /// 3. Schlüsselslot genullt und kryptographisch verifiziert (fsync #3).
    ///    Ab hier ist das Datum unlesbar. Aufrufer erhält EraseReceipt.
    KeyRevoked,

    /// 4. Asynchrone physische Bereinigung der Schichten (Purge) läuft.
    Purging,

    /// 5. Residuen-Scan mit Positivkontrolle (Scanner<Armed>) läuft.
    Verifying,

    /// 6. Erfolgreich abgeschlossen, Bericht signiert, Ledger geschrieben (terminal).
    Completed,

    /// Zugriffsbeschränkung aktiv (Art. 18 DSGVO, D-116).
    Restricted,

    /// Zugriffsbeschränkung aufgehoben (terminal).
    Lifted,

    /// Wiederholbare Verzögerung mit Grund (z. B. I/O-Stau, Plattenreserve).
    Pending(PendingReason),

    /// Verweigert wegen aktiver gesetzlicher Aufbewahrungspflicht (terminal).
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PendingReason {
    DiskReserveLow,
    CompactionRunning,
    IoBackoff,
}

impl SagaState {
    /// Gibt an, ob der Zustand endgültig ist (INV-9).
    #[inline]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Refused | Self::Lifted)
    }

    /// Prüft, ob ab diesem Zustand Lesezugriffe blockiert sind.
    #[inline]
    pub fn blocks_reads(&self) -> bool {
        matches!(
            self,
            Self::Gated
                | Self::KeyRevoked
                | Self::Purging
                | Self::Verifying
                | Self::Completed
                | Self::Pending(_)
        )
    }
}
```

---

### 3.2 Löschziel (`EraseTarget`)

Kanonische Repräsentation der fachlichen und physischen Löschziele:

```rust
use contextra_ids::{CollectionId, DocId, SubjectRef, TenantId};

/// Fachliches Ziel eines Löschauftrags (SPEC §5.1).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EraseTarget {
    /// Einzelnes Dokument (Klasse K).
    Document {
        tenant: TenantId,
        collection: CollectionId,
        doc_id: DocId,
    },
    /// Betroffene Person über alle Dokumente und Collections (Klasse P, atomar O(1)).
    Subject {
        tenant: TenantId,
        subject: SubjectRef,
    },
    /// Vollständige Collection (Klasse K).
    Collection {
        tenant: TenantId,
        collection: CollectionId,
    },
    /// Vollständiger Mandant (Offboarding).
    Tenant {
        tenant: TenantId,
    },
}
```

---

### 3.3 Aufbewahrungs-Attestierung (`RetentionAttestation`)

```rust
use contextra_ids::{OperatorRef, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RetentionAttestation {
    pub code: RetentionCode,
    pub attested_by: OperatorRef,
    pub attested_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RetentionCode {
    /// Keine Pflicht bekannt.
    NoObligation,
    /// Pflicht bestand und ist abgelaufen.
    ObligationExpired,
    /// Betreiber-Direktive / behördliche Anordnung.
    OperatorOverride,
    /// Gesetzliche Aufbewahrungspflicht aktiv -> Löschung MUSS mit Refused enden.
    ObligationActive,
    /// Rechtsstreit anhängig.
    LegalClaimsPending,
    /// Betroffene Person verlangt Einschränkung (Art. 18 DSGVO).
    DataSubjectRequestedRestriction,
}

impl RetentionCode {
    #[inline]
    pub fn blocks_erasure(&self) -> bool {
        matches!(self, Self::ObligationActive | Self::LegalClaimsPending)
    }
}
```

---

## 4. Die 4 Fsync-Ankerpunkte & Phasenablauf

Jede Phase ist idempotent und crash-resistent. Der Zustand wird im `SagaJournal` gesichert.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Die 6 Phasen der Saga                             │
│                                                                             │
│  Phase 1: Validierung ──► fsync #1 (Intent im Journal)                      │
│                               │                                             │
│  Phase 2: Barrier & Gate ─► fsync #2 (Gated im Journal & ReadGate aktiv)    │
│                               │                                             │
│  Phase 3: Key Shredding ──► fsync #3 (KeySlot im Vault genullt)             │
│                               │                                             │
│  Phase 4: Receipt ────────► Return EraseReceipt (Aufrufer entlassen)        │
│                               │                                             │
│  Phase 5: Asynchroner Purge► Schichtenweiser Purge (PurgeBarrier)           │
│                               │                                             │
│  Phase 6: Verifikation ───► fsync #4 (Abschluss, Ledger ext., Completed)    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: Intent & Aufbewahrungsprüfung (fsync #1)
1. `retention.code.blocks_erasure()` prüfen. Bei aktiver Pflicht -> Übergang zu `SagaState::Refused` (terminal) und sofortiger Abbruch.
2. Duplikaterkennung via HMAC über das Target. Bei laufender Saga -> bestehendes Receipt idempotent zurückgeben.
3. Journal-Eintrag schreiben und mit `sync_data()` durabel auf Disk sichern (**fsync #1**).

### Phase 2: Exklusive Lesesperre (fsync #2)
1. Exklusiver Lock auf die `CommitBarrier` -> Schreiber werden blockiert.
2. In-Memory `SagaGate` aktivieren: Alle Leseabfragen auf den Schlüsselraum liefern ab sofort `GateDecision::Erased`.
3. Zustandsübergang `SagaState::Gated` im Journal schreiben und synchronisieren (**fsync #2**).
4. `CommitBarrier` freigeben.

### Phase 3: Schlüsselvernichtung (fsync #3)
1. Schlüssel im Tresor lokalisieren (DEK, SubjectKey, CollectionKEK oder TenantKEK).
2. Slot im Tresor mit kryptographischen Zufallsbytes überschreiben.
3. Slot mit `0x00` nullen (`zeroize`).
4. `fsync` auf die Keyring-Datei (**fsync #3**).
5. `verify_key_absent`: Lesende Gegenprüfung via Constant-Time-Vergleich. Slot muss vollständig `0x00` sein.

### Phase 4: Quittungsausstellung
1. Journal-Eintrag `SagaState::KeyRevoked` schreiben.
2. Aufrufer erhält `EraseReceipt { id, status: KeyRevoked, gated_at }`.
3. Der synchrone Request-Pfad ist beendet. Die Daten sind kryptographisch unlesbar.

### Phase 5: Asynchrones Purge-Fenster
Wird durch den Hintergrund-Runner unter der `PurgeBarrier` ausgeführt:
1. `check_disk_reserve`: Sicherstellen von mindestens 1 GB Plattenreserve.
2. Purge-Aufruf über alle Schichten in normativer Reihenfolge:
   - a) Snapshot-Widerruf (`mvcc`)
   - b) Inverted Index Rewrite (`text`)
   - c) WAL-Drain & Memtable Flush (`store`)
   - d) SSTable Compaction Rewrite (`store`)
   - e) HNSW-Graph Reparatur (`vector`)
   - f) GraphRAG-Bereinigung (`graph`)
   - g) Key-Slot physisch freigeben (`crypto`)
   - h) In-Memory Cache Invalidation (alle Schichten)

### Phase 6: Messung, Ledger & Abschluss (fsync #4)
1. Für jede Schicht `Scanner<Armed>` ausführen. Jeder Layer muss `residue_matches == 0` melden.
2. Ed25519-signierten JSON-Löschbericht generieren.
3. Hash-verketteten Eintrag im Ledger **außerhalb des Datenverzeichnisses** schreiben und synchronisieren (**fsync #4**).
4. Journal-Zustand auf `SagaState::Completed` setzen (terminal).
5. Gate-Eintrag kompaktieren.

---

## 5. Lesesperre (`SagaGate` & `ReadGate`)

Der Hot-Path-Check ist synchron, nicht-blockierend und $O(1)$:

```rust
/// Entscheidung des synchronen Gate-Checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    /// Datum ist freigegeben.
    Open,
    /// Datum ist gelöscht / gesperrt (Fail-Closed).
    Erased,
    /// Datum ist nach Art. 18 DSGVO eingeschränkt (Audit-Eintrag zwingend erforderlich).
    Restricted,
}

/// Bereichsentscheidung für SSTable-Kompaktionen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeGate {
    AllOpen,
    ContainsErased,
    RestrictedNeverDiscard,
}

pub trait ReadGate: Send + Sync {
    /// Synchroner O(1) Check auf Dokument- oder Key-Ebene vor jeder Entschlüsselung.
    fn check(&self, key: &[u8]) -> GateDecision;

    /// Bereichsprüfung für LSM-Kompaktionen.
    fn check_range(&self, start: &[u8], end: &[u8]) -> RangeGate;
}
```

---

## 6. Messung & Positivkontrolle (`Scanner<Armed>`)

Um sicherzustellen, dass Scanner nicht fälschlich 0 Rückstände melden, erzwingt das Typsystem eine vorherige erfolgreiche Positivkontrolle:

```rust
use std::time::Duration;

pub struct Unarmed;
pub struct Armed(());

/// Nachweis über erfolgreich gefundene Positivkontrolle (Canary).
pub struct ControlEvidence(());

impl ControlEvidence {
    #[inline]
    pub const fn observed() -> Self {
        Self(())
    }
}

pub struct Scanner<State> {
    layer_id: &'static str,
    files_scanned: u64,
    bytes_scanned: u64,
    _state: State,
}

impl Scanner<Unarmed> {
    pub fn new(layer_id: &'static str) -> Self {
        Self {
            layer_id,
            files_scanned: 0,
            bytes_scanned: 0,
            _state: Unarmed,
        }
    }

    /// Schärft den Scanner nach belegter Positivkontrolle.
    pub fn arm(self, _evidence: ControlEvidence) -> Scanner<Armed> {
        Scanner {
            layer_id: self.layer_id,
            files_scanned: self.files_scanned,
            bytes_scanned: self.bytes_scanned,
            _state: Armed(()),
        }
    }
}

impl Scanner<Armed> {
    pub fn record_file(&mut self, bytes: u64) {
        self.files_scanned += 1;
        self.bytes_scanned += bytes;
    }

    /// Schließt die Schicht-Messung ab.
    /// matches != 0 führt zwingend zu Err(ResidueFound).
    pub fn finish(
        self,
        residue_matches: u64,
        scan_duration: Duration,
    ) -> Result<LayerWitness, ResidueFound> {
        if residue_matches != 0 {
            return Err(ResidueFound { matches: residue_matches });
        }
        Ok(LayerWitness {
            layer_id: self.layer_id,
            files_scanned: self.files_scanned,
            bytes_scanned: self.bytes_scanned,
            residue_matches: 0,
            scan_duration,
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Residue detected during verification: {matches} matches found")]
pub struct ResidueFound {
    pub matches: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayerWitness {
    pub layer_id: &'static str,
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub residue_matches: u64,
    pub scan_duration: Duration,
}
```

---

## 7. Fehlerbehandlung (`ErasureError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ErasureError {
    #[error("Retention obligation active: erasure refused")]
    RetentionRefused(RetentionCode),

    #[error("Key slot missing in vault for target")]
    SlotMissing,

    #[error("Cryptographic key shredding failed: verification not zeroed")]
    ShredVerificationFailed,

    #[error("Disk reserve below threshold ({available} bytes < {required} bytes)")]
    DiskReserveExhausted { available: u64, required: u64 },

    #[error("Residue matches detected in layer {layer}: {matches}")]
    ResidueDetected { layer: &'static str, matches: u64 },

    #[error("Saga journal I/O failure: {0}")]
    JournalIo(#[from] std::io::Error),

    #[error("Durable filesystem failure: {0}")]
    DurableFs(String),
}
```

---

## 8. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-erasure"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Canonical GDPR deletion core, Saga state machine, ReadGate, and verifiable crypto-shredding for Contextra"

[package.metadata.contextra]
ring = "0"
group = "foundation"
level = 0

[lints]
workspace = true

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
signing = ["dep:ed25519-dalek"]

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-privacy = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }
ed25519-dalek = { version = "2", optional = true }
parking_lot = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"
```

---

## 9. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - `ring = "0"` in `[package.metadata.contextra]` eintragen.
   - `aes-gcm-siv`, `sha2`, `hmac` **vollständig entfernen** — diese gehören in `contextra-crypto`.
   - `default = []` setzen (R3).
   - `[lints] workspace = true` eintragen.
2. **Typ-Bereinigung:**
   - `types_local.rs` löschen.
   - Alle Vorkommen von `KeyDestroyed` → `KeyRevoked` umbenennen.
   - Sicherstellen: keine E1/E2-Crate in `[dependencies]`.
3. **SagaGate In-Memory Lookup:**
   - `ReadGate::check()` MUSS O(1) ohne Lock-Contention und ohne I/O ausführen.
   - Beim Öffnen der Datenbank: alle nicht-terminalen Sagas aus Journal laden, Gate sofort aktivieren.
4. **Scanner-Härtung:**
   - `ControlEvidence::observed()` ist der einzige Weg zu `Scanner<Armed>`.
   - Privater Konstruktor sicherstellen.
5. **Vault-Härtung:**
   - `verify_key_absent` mit Constant-Time-Vergleich.
   - Zeroize-Sequenz: Zufallsbytes → Null → fsync → Verifikation.
6. **Verifikation:**
   - `cargo clippy -p contextra-erasure -- -D warnings`
   - `cargo test -p contextra-erasure`
