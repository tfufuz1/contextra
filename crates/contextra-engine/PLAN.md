# Implementierungsplan: contextra-engine

**Ebene:** L3 · **Ziel-Zustand:** einziger Composition Root. Verdrahtet Kern, Krypto, Dateischicht, SIMD und Retrieval über `engine::projection`. Modulplan nach SPEC §2.1: `platform`, `kernel`, `projection`, `collection`, `query`, `ingest`, `compliance`, `workers`.
**Basis:** SPEC R14 §2.1, §2.4, §3.3, §8.5, §11, §13.1, DEC-01, DEC-02, DEC-19, DEC-21, DEC-52; R15 §2.3, DEC-41…DEC-52. Quellcode gemessen, Module nur in Struktur gelesen.

## 1. Ist-Befund (gemessen)

- 86 Quelldateien, 21 675 Zeilen, Hauptmodule: `adapters`, `background_workers`, `backends.rs`, `chunker`, `collection`, `contextra_impl`, `decay_controller.rs`, `export.rs`, `extraction`, `filter.rs`, `fusion.rs`, `import.rs`, `lib.rs`, `metrics_names.rs`, `orchestration`, `ollama.rs`, `privacy.rs`, `temporal_filter.rs`, `text.rs`, `transaction`.
- **169 Treffer verbotener Muster im Produktionscode** (`std::fs`, `tokio`, `block_on`, `blake3`, `bincode`, `parking_lot`, `serde`).
- **`contextra_store`-Importe in 26 Dateien**, keine Abhängigkeit in `Cargo.toml` (R15 §2.3). `engine` baut in diesem Zustand nicht.
- `Cargo.toml` hat `blake3`, `bincode`, `hmac`, `sha2`, `tokio`, `tokio-util`, `loom` (cfg). Laut §2.7 erlaubt: `tokio` + `tokio-util` (nur Rand), `serde`/`serde_json` (API). `blake3`, `bincode`, `hmac`, `sha2` sind in `engine` nicht vorgesehen.
- Altmodule laut §13.3/§2.7:
  - `orchestration/` (NZ-21)
  - `decay_controller.rs` (NZ-22)
  - `adapters/ollama.rs` (NZ-23)
  - `collection/kv_lock.rs`
  - `transaction/` (DbTransaction, CommitLedger, CompensatingAction — 4-Wege-2PC, DEC-19)
  - `contextra_impl/` (teilweise Legacy-Fassade, Inhalt vorher prüfen)
- `collection/maintenance.rs` (972 Zeilen) vermischt Wartung und Erase.
- `collection/tests/hybrid_search_tests.rs` (1287 Zeilen) ist die größte Testdatei im Projekt, testet vermutlich Altpfade.
- `background_workers/tests.rs` (766 Zeilen).
- Kein `tests/`-Verzeichnis für Architektur-Tests (DEC-52).

## 2. Entfernen

| Ziel | Stelle | Begründung |
|---|---|---|
| `orchestration/` | Verzeichnis | NZ-21, §13.2 Nr. 7 |
| `decay_controller.rs` | Datei | NZ-22 |
| `adapters/ollama.rs` | Datei | NZ-23; Modellhosting nicht im Scope |
| `collection/kv_lock.rs` | Datei | §13.2 Nr. 7 |
| `transaction/` komplett (DbTransaction, CommitLedger, CompensatingAction) | Verzeichnis (7 Dateien) | DEC-19: Ein-Wahrheits-Commit ersetzt 2PC; §13.3 |
| Feature `legacy-store` und alle `contextra_store`-Importe | 26 Dateien | G-CUTOVER; bis dahin hinter Feature, danach löschen |
| `blake3`, `bincode`, `hmac`, `sha2` in `Cargo.toml` | — | §2.7 nur Kern/Krypto |
| `tokio` außer in `kernel` und Rand | verschiedene | §2.7: Async nur am Rand |
| `block_on` außerhalb `kernel` | verschiedene | §13.3, DEC-01 |
| `contextra-store`-Verweise in `metrics_names.rs` | — | Metrikvertrag §15 |
| Tests, die nur Altpfade prüfen | `transaction/tests.rs`, Teile von `hybrid_search_tests.rs` | nach Einzelprüfung |

**Vor jeder Löschung:** `grep -rn '<Modul>' crates/` und Prüfung auf Verwendung in `collection/` und `contextra_impl/`.

## 3. Ändern

### 3.1 Modulplan (§2.1)

Bestehende Module auf die Zielstruktur abbilden:

| Ziel | Quelle heute | Aktion |
|---|---|---|
| `platform` | (neu) | `Fs`-Adapter über `durable-fs`/`sys`, `Clock`, `Entropy`, `HostProbe`, `FileAnchor` |
| `kernel` | `backends.rs`, `lib.rs` (Teile) | Kern öffnen, Blocking-Pool (`spawn_blocking`), Fehlerabbildung |
| `projection` | (neu) | `SegmentOwner`/`ConsumerFeed`/Watermark/Attest je Index |
| `collection` | `collection/` | CRUD, Relationen, `TenantPolicy`; `kv_lock` raus |
| `query` | `fusion.rs`, `filter.rs`, `temporal_filter.rs`, `text.rs` | QP-1…QP-9, `SnapshotSet`, Fusion |
| `ingest` | `chunker/`, `extraction/`, `import.rs` | Chunker, Pseudonymisierung, Publish-Record, Quarantäne |
| `compliance` | `privacy.rs`, `export.rs` | Profile, Receipt → `DeletionProof`, Export |
| `workers` | `background_workers/` | Wartung, Projektion, Rebuild |

### 3.2 `lib.rs` und `contextra_impl/`

- `contextra_impl/` ist eine parallele Fassade. Inhalt in die Zielmodule verteilen, dann Verzeichnis löschen.
- `lib.rs`: nur Modulliste und öffentliche API.

### 3.3 `collection/maintenance.rs` (972 Zeilen)

Aufteilen in `workers/maint_schedule.rs` (Lanes, Priorität) und `compliance/erase_flow.rs` (Löschauftrag, Ticket). Eine Datei entscheidet nicht über beides.

### 3.4 `background_workers/`

Umbenennen in `workers`, Lanes (`urgent > flush > compact > blobgc > idle`) aus §2.4 exakt übernehmen. Governor-Anteil `maint_io_fraction` 0,5 aus Config.

### 3.5 `export.rs` und `import.rs`

Forward-Export und Forward-Ingest aus `contextra-store` (NZ-7, G-R5). Kein SSTable-Format. Export-Anteil `export_io_fraction` 0,2.

### 3.6 `privacy.rs` und `chunker/`

- `privacy.rs`: Pseudonym-Tresor, `pseudonym_scope` (PerPerson), `placeholder_retention` (Keep/Scrub). Standard laut §15.
- `chunker/`: Chunks werden erst nach Publish sichtbar (Ingest-Invariante, §11.4). Staging-Run mit `staged_orphan_ttl` 24 h.

### 3.7 Tests

- `collection/tests/hybrid_search_tests.rs` (1287 Zeilen): auf neue Schnittstellen umstellen oder aufteilen in `query/tests/`.
- `background_workers/tests.rs` (766): auf `workers` umbenennen.
- Neue Architektur-Tests in `tests/architecture.rs` (DEC-52): `layering` und `unsafe_islands` über `cargo metadata`.

## 4. Implementieren

### 4.1 `platform`

- `Fs`-Adapter: ≤ 400 LOC über `durable-fs` (DEC-39).
- `Clock`: monotone Zeit, UTC, `clock_floor` (AN-09, `max_clock_skew` 5 min ⇒ `ClockUncertain`, fail-closed).
- `Entropy`: über `contextra-crypto::OsEntropy` (einzige Stelle mit `getrandom`).
- `FileAnchor`: `Weak`; `acknowledged_weak` in Config (§8.9).
- `HostProbe`: Swap, Core-Dumps, memlock, FS-Typ; `Unknown` ⇒ `Declared`.

### 4.2 `kernel`

- Öffnen: Ports verdrahten, `ComplianceProfile` prüfen.
- Blocking-Pool: einzige Stelle mit `block_on` (§13.3). Async-Fassade über `spawn_blocking`.
- Fehlerabbildung: Kernfehler → `hint()`-tragende Engine-Fehler.

### 4.3 `projection` (R2)

- `SegmentOwner`, `ExtensionBlock`, `BlockPruner`, `ConsumerFeed`, `View::filter_visible` (FIX-28).
- Je Index (Vektor, Text, Graph, IdentityDirectory): Sink-Implementierung über `SegmentIo`.
- `rewrite_without`, `verify_absent`.
- `Stamp↔Epoch`-Zuordnung.
- Watermark und Attest je Index; Projektion idempotent aus Feed (DEC-19).
- Modus-Wahl A/B/C (DEC-24) nach Größe und Personenbezug.

### 4.4 `collection`

- Collection → `SpaceId` je (Collection, Schutzklasse).
- `TenantPolicy`: Mandanten-Logik liegt hier, nicht im Kern (NZ-16).
- `tenant_tag` als zufälliger, geheimer Bezeichner, nie aus Mandanten-ID ableitbar (AT-10).

### 4.5 `query` (R3)

- QP-1…QP-9 nach §11.10.
- `SnapshotSet = (instance_id, V) + (partition, generation, applied_stamp)`.
- `Index ∪ Delta` Abfrage.
- Fusion über `contextra-rank`, Tokenbudget über `DiBud`.
- `min_stamp` (ryw_wait 50 ms).
- Kein Lesen unter Rang 1 (Z-17): Abfrage ohne E/A, Daten aus Snapshot.

### 4.6 `ingest` (R4)

- Publish-Record (`head`) statt Mega-Change (DEC-20).
- `PiiClassifier`-Routing, Quarantäne bei Unsicherheit.
- Pseudonymisierung vor Korpus-Indexierung (DEC-25).
- Idempotenz je Teil-Change.
- Orphan-Worker für Staging-Runs.
- `EgressPolicy` vor jedem LLM-Aufruf (DEC-37).

### 4.7 `compliance`

- Profile: `Standard` (Default), `Strict` (ohne Acknowledgments, DEC-30).
- Receipt → `DeletionProof` (Signatur optional, §3.2 dieses Dokuments).
- Export: `export_cluster`.
- Retention über `history_retention`.

### 4.8 `workers`

- Lanes wie §2.4.
- Rebuild geteilter Indizes mit `index_rebuild_bytes` 10 %.
- Aufräumen: Blob-GC, Unlink nach `unlink_ack`.

### 4.9 Erase-Koordination

- Ticket-Zustände aus Anhang A nur über `storage` ausgelöst.
- `engine` entscheidet *ob* gelöscht wird (Restriction, TenantPolicy); `storage` setzt um (§2.3, Mechanismus nie Politik).
- `on_gate` von Ownern ist Beschleuniger, Kern-Gate ist Barriere (DEC-22).

## 5. Tests

| Test | Prüft | Gate |
|---|---|---|
| `tests/architecture.rs::layering` | §2.1, INV-LAYER-1 | G-0 |
| `tests/architecture.rs::unsafe_islands` | §2.6, FIX-26 | G-0 |
| `query::differential` | `Index ∪ Delta ≡ Vollscan` | G-R3 |
| `query::race_gate_x_query` | INV-RAG-4 | G-R3 |
| `projection::owner_conformance` | R2 je Owner | G-R2 |
| `ingest::crash_sweep_IG1_6` | IG-1…IG-6, Partial-Ingest unsichtbar | G-R4 |
| `ingest::pii_residual_rate` | H-34 | G-R4 |
| `compliance::receipt_verify` | `verify_receipt` offline | G-E |
| `compliance::time_travel_degraded` | FIX-15 | G-E |
| `workers::governor` | Reserve, Lanes | G-M |
| Bestehende Tests | Regression nach Umbau | laufend |

## 6. Abhängigkeiten nach Umbau

Erlaubt: `contextra-mvcc`, `contextra-ports`, `contextra-types`, `contextra-storage`, `contextra-crypto`, `contextra-durable-fs`, `contextra-simd`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-checkpoint`, `tokio` und `tokio-util` (nur Rand), `serde`/`serde_json` (nur API), `ahash`, `bytes`, `parking_lot` (nur Rand, falls nötig), `thiserror`, `tracing`, `zeroize`.
Verboten im Kern, erlaubt hier nur mit Begründung: `blake3`, `bincode`, `hmac`, `sha2` (über `crypto`-Ports).
`loom`: nur `cfg(loom)`.

## 7. Gate und Reihenfolge

1. **P0-A:** `contextra_store`-Importe hinter Feature `legacy-store` (26 Dateien). Kein Build ohne diesen Schritt.
2. Altmodule löschen: `orchestration`, `decay_controller`, `ollama`, `kv_lock`, `transaction/`, `contextra_impl/` (nach Verteilung).
3. Modulplan §2.1 herstellen (Verzeichnisse umbenennen, `platform`, `kernel` anlegen).
4. **P0-B:** `platform` (Fs, Clock, Entropy, FileAnchor, HostProbe).
5. **R2:** `projection` mit Ownern.
6. **R3:** `query` mit Differential-Test.
7. **R4:** `ingest` mit Publish-Record.
8. **P5/P6/P7:** `workers`, `compliance`, Branches.
9. **Architektur-Tests** (DEC-52) bis spätestens G-0.

**Risiken:**
- Größe: 21 675 Zeilen. Aufspaltung nur nach Trigger (> 40 kLOC oder zyklische Abhängigkeit, §17.3). Heute kein Trigger; Aufspaltung erst nach Löschen der Altmodule prüfen.
- `hybrid_search_tests.rs` (1287 Zeilen) ist vermutlich nicht lauffähig nach Umbau. Vorher Testabdeckung der Kernfunktion sichern, dann Datei neu schreiben statt anpassen.
- Engine ist der einzige Ort mit allen Abhängigkeiten. Jede Änderung in einem L2-Crate wirkt hier; Integrationstests sind Pflicht.
