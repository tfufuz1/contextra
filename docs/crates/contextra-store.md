# contextra-store — Crate-Spezifikation (LSM-Storage Engine)

**Ebene:** E1b (Kern-Closure: `ids`, `env`, `sys`, `durable-fs`, `crypto`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-store`  
**Referenz:** `docs/SPEC.md` §2, §3, §4, §5, §6, §7

---

## 1. Zweck & Systemrolle

`contextra-store` ist die primäre Persistenz-Engine für Contextra auf Basis einer Log-Structured Merge-Tree (LSM-Tree) Architektur. Es garantiert:
- **ACID-konforme Persistenz** über Write-Ahead Logging (WAL) mit konfigurierbaren Fsync-Garantien.
- **Mehrschichtige Speicherung:** In-Memory Active MemTable, Immutable MemTables, Disk-SSTables (Level-0 bis Level-N).
- **Verschlüsselte Nutzlasten:** Payloads werden vor dem Einreihen in den MemTable/WAL durch den `RecordSealer` versiegelt (AEAD via `contextra-crypto`).
- **Deterministische physische Löschung:** Vollständige Bereinigung von gelöschten Schlüsseln und Datenbereichen durch gezielte SSTable-Rewrites und WAL-Drains im Rahmen der DSGVO-Löschkette.

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
│   │   │ TableManifest     │    │ ErasureLayer Impl  │   │   │
│   │   └───────────────────┘    └────────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Datenformat & Verschlüsselung auf Festplatte

### 3.1 SSTable-Dateiformat (`.sst`)
Jede SSTable ist in 4KB/64KB Blöcke unterteilt und folgt strikter binärer Serialisierung:
1. **Magic & Header:** `b"CXTR-SST\x01"`, Blockgröße, Flags (inkl. Kompressions- und Krypto-Kennung).
2. **Data Blocks:** Schlüssel-Wert-Paare, sortiert nach Key aufsteigend. Werte sind AEAD-versiegelt.
3. **Filter Block:** Roaring- oder Split-Block-Bloom-Filter zur Vermeidung unnötiger Plattenzugriffe.
4. **Index Block:** Block-Offsets und Schlüssel-Grenzen (Sparse Index).
5. **Footer:** Feste Länge (48 Bytes), Prüfsumme (CRC32C / BLAKE3), Index-Offset, Filter-Offset, Magic Trailer.

### 3.2 Verschlüsselung & AAD-Invariante
- **Niemals Klartext auf Datenträger (T-DEL-1):** Payloads werden mit `AES-256-GCM-SIV` verschlüsselt.
- **Zugeordnete Daten (AAD):** `tenant_id (16B) || collection_id (16B) || doc_id (16B) || version (8B)`.
- **Schlüssel-Präfixe:** Präfixe für interne Index-Schlüssel (`__doc:`, `__meta:`) enthalten ausschließlich 128-Bit-Hashes oder Zufalls-IDs (`PersonalRef`/`SubjectRef`), niemals ungeschützte Klartext-PII.

---

## 4. DSGVO-Löschkette: ErasureLayer-Implementierung

`contextra-store` implementiert das Trait `contextra_ports::erasure::ErasureLayer` mit statischer `LayerId`:

```rust
impl ErasureLayer for LsmStorage {
    fn id(&self) -> LayerId {
        LayerId(*b"CTX_STORE_LSM_01")
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }

    fn resolve(&self, target: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;
    fn on_gate(&self, scope: &LayerScope) -> BoxFuture<'_, Result<(), String>>;
    fn purge(&self, scope: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>;
    fn witness(&self, scope: &LayerScope, sc: &Scanner<Armed>) -> BoxFuture<'_, Result<LayerWitness, String>>;
}
```

### 4.1 Die 4 Phasen in `contextra-store`

#### 1. `resolve(target)`
- Durchsucht das `TableManifest` und die Bloom-Filter aller SSTables nach Überschneidungen mit `target` (Range `[start, end)` oder Einzelschlüssel).
- Ermittelt betroffene aktive WAL-Segmente.
- Liefert `LayerScope` mit Dateipfaden und SSTable-IDs zurück (read-only, keine I/O-Mutationen).

#### 2. `on_gate(scope)`
- Registriert die Schlüssel im In-Memory `WriteFence`.
- Verhindert, dass neue Schreib- oder Leseoperationen für diese Schlüssel akzeptiert werden.

#### 3. `purge(scope, cx)` — Physische Auslöschung
Wird unter exklusivem Halt der `PurgeBarrier` ausgeführt:
- **Sub-Schritt 4c (WAL-Drain):** Die aktuelle aktive MemTable wird gefriergetrocknet (Immutable) und sofort via `flush_to_level0()` auf Platte geschrieben. Der alte WAL wird geschlossen, die genullten Header geschrieben und die Datei gelöscht.
- **Sub-Schritt 4d (SSTable-Rewrite):** 
  - Für jede im `LayerScope` identifizierte SSTable wird eine Kompaktion gestartet.
  - Der Reader iteriert über die SSTable; alle Einträge, die unter das `EraseTarget` fallen, werden **komplett verworfen (skipped)**.
  - Eine neue, bereinigte SSTable (`.sst.tmp`) wird sequentiell geschrieben und mit `fsync` synchronisiert (`contextra-durable-fs`).
  - Das `TableManifest` wird transaktional aktualisiert (Austausch alter SST-IDs gegen neue).
  - Die alten SSTable-Dateien werden auf Dateisystemebene mit Nullen überschrieben (`shred_file`) und anschließend via `unlink` gelöscht.
- Gibt `PurgeOutcome { purged_count, bytes_reclaimed }` zurück.

#### 4. `witness(scope, sc)` — Verifikation & Messung
- Scannt sämtliche im Datenverzeichnis verbliebenen SSTables und WAL-Dateien.
- Führt eine binäre Mustersuche nach dem Canary-Byte-Muster bzw. den Ziel-Schlüsseln durch.
- Liefert `LayerWitness { residue_matches: 0, scan_class: MeasureClass::Mk2, ... }`. Ergibt der Scan `residue_matches > 0`, schlägt die gesamte Löschsaga fehl.

---

## 5. Synchroner Lesepfad: ReadGate-Integration

Vor jedem Lesezugriff in `get()`, `scan()` oder `scan_prefix()` MUSS der `ReadGate` synchron befragt werden:

```rust
pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ContextraError> {
    // 1. O(1) Gate-Check
    match self.read_gate.check(key) {
        GateDecision::Open => {},
        GateDecision::Erased => return Ok(None), // Fail-Closed: Keine Daten, kein Leak
        GateDecision::Restricted => return Err(ContextraError::RestrictedReadAuditRequired),
    }

    // 2. MemTable Lookup
    // 3. SSTable Lookup
    // 4. Decrypt via DocumentSealer
    // ...
}
```

---

## 6. Fehlerbehandlung & Invarianten

1. **`#![forbid(unsafe_code)]`:** Keine unsafe-Blöcke. Dateisystem-Primitive und mmap-Operationen werden über `contextra-durable-fs` und `contextra-sys` bezogen.
2. **Zero-Panic-Policy:** Keine `unwrap()`, `expect()` oder `panic!()` in Produktivpfaden.
3. **Crash-Invarianz (Z-E4):** Bricht das System während des SSTable-Rewrites ab, bleibt die Original-SSTable gültig, bis das Manifest atomar aktualisiert ist. Nach Wiederanlauf setzt `run_purge_window` den Purge-Schritt idempotent fort.
4. **Keine verwaisten Klartexte:** Bei Entschlüsselung im Speicher werden Puffer mit `Zeroizing<Vec<u8>>` allokiert.
