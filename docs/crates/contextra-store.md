# contextra-store — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env`, `sys`, `durable-fs`, `crypto` (max. 5 Mitglieder gemäß SPEC §2.1)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-store`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3, §4, §5.3 (Schritte 4c & 4d), §6, §13 (INV-1, INV-6); `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-store` ist die primäre Persistenz-Engine von Contextra auf Basis einer Log-Structured Merge-Tree (LSM-Tree) Architektur:
- **Crash-sicheres Write-Ahead Logging (WAL):** Durables Append-Only Log mit Gruppen-Commits und Fsync-Disziplin (`contextra-durable-fs`).
- **Mehrschichtige LSM-Speicherung:** Aktiver MemTable (SkipList), Immutable MemTables, Disk-SSTables (Level-0 bis Level-N) mit Block-Cache.
- **Kryptographische Versiegelung im Ruhezustand (Data at Rest):** Sämtliche Payloads der Klassen K und P werden vor dem Schreiben in den MemTable via AES-256-GCM-SIV versiegelt (`contextra-crypto`).
- **Physisches SSTable-Rewrite & WAL-Drain (DSGVO Schritte 4c & 4d):** Bereinigung gelöschter Schlüsselbereiche bei Kompaktionen und Nullung physischer SSTable-Dateien (Scrubbing).

### Beseitigung von Ebenen- und Dev-Dependency-Verstößen (R1, R2, R3)
1. **Keine E2-Abhängigkeiten:** Die optionalen Abhängigkeiten zu `contextra-types` und `contextra-ports` sowie die Features `workspace`, `types-compat` und `ports-compat` werden vollständig entfernt.
2. **Keine E3-Dev-Abhängigkeiten (R2):** `contextra-testkit` (E3) in `[dev-dependencies]` ist ein Verstoß gegen R2 (*„Dev-Abhängigkeiten auf höhere Ebenen sind verboten"*) und wird ersatzlos gestrichen.
3. **Leere Default-Features (R3):** `default = []` (Entfernung von `default = ["cache"]`).
4. **Altlasten-Bereinigung (`AP-21`):** In `src/lib.rs` Zeile 26 (`contextra-agent`) und `src/compaction/merge_operator.rs` Zeile 11 (`contextra-sandbox`) werden alle Bezüge auf entfernte Crates bereinigt.
5. **Kein `Uuid::new_v4()`:** Generierung von Dateinamen und Identifikatoren erfolgt über den fehlbaren CSPRNG (`OsRng`) oder atomare Sequenznummern, niemals über ungeprüftes unverschlüsseltes UUIDv4.

### Architekturgarantien (R1–R6, INV-1, INV-4, INV-6, INV-11)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Keine Panics; alle Fehlerpfade liefern `StoreError`.
3. **INV-6 Unveränderlichkeit:** WAL-, Manifest-, Flush- und Recovery-Formate bleiben abwärtskompatibel.
4. **Niemals Klartext auf Datenträger (T-DEL-1):** Alle persistierten Payloads sind verschlüsselt.

---

## 2. Architektur & Komponenten

```
               ┌──────────────────────────────┐
               │    contextra-db / Engine     │
               └──────────────┬───────────────┘
                              │ Writes / Reads
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       contextra-store                       │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                     LsmStorage                      │   │
│   │                                                     │   │
│   │   ┌──────────────┐             ┌────────────────┐   │   │
│   │   │    WAL       │ (append-    │ Active MemTable│   │   │
│   │   │  (encrypted) │  only, fsync│   (SkipList)   │   │   │
│   │   └──────────────┘             └───────┬────────┘   │   │
│   │                                        │ flush      │   │
│   │                                        ▼            │   │
│   │                                ┌────────────────┐   │   │
│   │                                │Immutable Tables│   │   │
│   │                                └───────┬────────┘   │   │
│   │                                        │ compactor  │   │
│   │                                        ▼            │   │
│   │   ┌─────────────────────────────────────────────┐   │   │
│   │   │ Level-0 SSTables   [ sst-001 ] [ sst-002 ]  │   │   │
│   │   ├─────────────────────────────────────────────┤   │   │
│   │   │ Level-1 SSTables   [ sst-101 ] [ sst-102 ]  │   │   │
│   │   └─────────────────────────────────────────────┘   │   │
│   │                                                     │   │
│   │   ┌───────────────────┐    ┌────────────────────┐   │   │
│   │   │ TableManifest     │    │ ErasurePurgeEngine │   │   │
│   │   └───────────────────┘    └────────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. DSGVO-Löschkette: WAL-Drain & SSTable-Rewrite (Schritte 4c & 4d)

Unter exklusivem Halt der `PurgeBarrier`:

### 3.1 Schritt 4c: WAL-Drain & MemTable-Flush
1. `flush_active_memtable()`: Schiebt den aktuellen MemTable in den Immutable-Zustand.
2. Schreibt den Immutable-MemTable als neue L0-SSTable auf Disk (`write_atomic`).
3. Draint das aktuelle WAL-Segment und schneidet es ab (`truncate`).

### 3.2 Schritt 4d: SSTable-Rewrite (Kompaktion)
1. **Target-Prüfung:** Ermittelt alle SSTables, die Schlüssel des gelöschten Targets enthalten.
2. **Kompaktions-Filter:** Beim Zusammenführen der SSTables filtert der `CompactionFilter` alle Einträge aus, deren Schlüssel im `ReadGate` als `Erased` gemeldet sind.
3. **Schreiben neuer SSTables:** Neue SSTables enthalten weder Daten noch Tombstones der gelöschten Entitäten.
4. **Manifest-Update:** Atomare Aktualisierung des Manifests via `write_atomic` und Directory-Sync.
5. **Scrubbing (`contextra-durable-fs`):** Alte, ersetzte SSTable-Dateien werden vor dem Unlink physisch mit Nullen überschrieben (`scrub_file`).

---

## 4. SSTable-Format (`.sst`)

Jede SSTable ist strikt strukturiert:
```
┌─────────────────────────────────────────────────────────────┐
│ Magic Header (b"CXTR-SST\x01")                             │
├─────────────────────────────────────────────────────────────┤
│ Data Blocks (4 KB / 64 KB)                                  │
│   - Block 0: Encrypted Key-Value Pairs                      │
│   - Block 1: Encrypted Key-Value Pairs                      │
│   - ...                                                     │
├─────────────────────────────────────────────────────────────┤
│ Filter Block (Roaring Bitmap / Bloom Filter)                │
├─────────────────────────────────────────────────────────────┤
│ Index Block (Sparse Index: Key Boundaries & Offsets)        │
├─────────────────────────────────────────────────────────────┤
│ Footer (48 Bytes: CRC32C, Offsets, Magic Trailer)           │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. Fehlerbehandlung (`StoreError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("WAL corruption detected: {0}")]
    WalCorrupted(String),

    #[error("SSTable verification failure: {0}")]
    SstableCorrupted(String),

    #[error("Manifest recovery failed: {0}")]
    ManifestRecovery(String),

    #[error("Compaction aborted: {0}")]
    CompactionFailed(String),

    #[error("Durable filesystem error: {0}")]
    DurableFs(String),

    #[error("Cryptographic error in record sealing: {0}")]
    Crypto(String),

    #[error("Underlying I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## 6. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-store"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Log-Structured Merge-Tree (LSM) storage engine, encrypted WAL, and SSTables for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-sys = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-crypto = { workspace = true }
contextra-mvcc = { workspace = true }
bytes = { workspace = true }
blake3 = { workspace = true }
crc32fast = { workspace = true }
parking_lot = { workspace = true }
ahash = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"
tokio = { workspace = true, features = ["full", "test-util"] }
criterion = { workspace = true }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
cache = []
```

---

## 7. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports`.
   - Entfernen von `contextra-testkit` aus `[dev-dependencies]` (R2).
   - Features `workspace`, `types-compat`, `ports-compat` entfernen.
   - `uuid` Dependency entfernen.
   - `default = []` setzen (R3).
2. **Altlasten-Bereinigung (`AP-21`):**
   - In `src/lib.rs` (Z. 26) und `src/compaction/merge_operator.rs` (Z. 11) alle Referenzen auf `contextra-agent` und `contextra-sandbox` entfernen.
3. **Schritte 4c & 4d Härtung:**
   - WAL-Drain und SSTable-Rewrite fest an `contextra-durable-fs::scrub_file` koppeln.
4. **Verifikation:**
   - `cargo test -p contextra-store`
