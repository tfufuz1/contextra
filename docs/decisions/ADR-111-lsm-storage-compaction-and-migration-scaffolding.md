# ADR-111: Status und Architekturanalyse des LSM Storage Engine Compaction, Observer & Migration Scaffolding-Codes

* **Datum**: 2026-10-04
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-store` (`lsm/engine.rs`, `compaction/engine.rs`, `lsm/observer.rs`, `kv/segment.rs`, `memtable.rs`, `tenant_codec.rs`, `wal/encode.rs`, `wal/hmac.rs`, `wal/io.rs`, `wal/replay.rs`, `sstable/block_cache.rs`, `sstable/builder.rs`, `lsm/flush.rs`, `lsm/scan.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_VOLLSTAENDIGE_FEATURE_SPEZIFIKATION.md` (§4, C.4.2, INV-WAL-LEGACY-KEY-1), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen des Auditings der öffentlichen API-Fläche wurden 31 Funktionen in `contextra-store` identifiziert, die aktuell keine direkten Konsumenten außerhalb des Crates besitzen.

### Kernbefunde:
1. **WAL-Migration & Legacy Protection (`lsm/engine.rs`, `wal/hmac.rs`)**:
   `has_pending_legacy_wal_migration`, `migrate_legacy_wal_keys` und `is_standard` bilden den Migrationspfad für ältere WAL-Segmente.
   *Architektur-Bezug*: Invariante `INV-WAL-LEGACY-KEY-1` erzwingt, dass der Legacy-Integritätsschlüssel-Fallback niemals implizit aktiv sein darf, sondern zwingend explizite Migrationsaufrufe erfordert.
2. **WAL Observer / Change Data Capture (CDC) (`lsm/observer.rs`, `wal/io.rs`, `wal/replay.rs`)**:
   `dropped_count_for`, `is_any_circuit_breaker_open`, `clear_circuit_breaker`, `try_append_batch_locked`, `try_append_batch` und `replay_stream` bieten synchrone WAL-Tailer- und Circuit-Breaker-Schnittstellen für reaktive Agenten (CDC, C.4.2).
3. **Adaptive Compaction & Block Cache (`compaction/engine.rs`, `sstable/block_cache.rs`, `sstable/builder.rs`)**:
   `with_adaptive_planner`, `create_block_cache`, `current_size` und `set_format_version` dienen der dynamischen Compaction-Planung und SSTable-Formatversionierung.
4. **KV Segment Shredding & Multi-Tenant Encoding (`kv/segment.rs`, `tenant_codec.rs`, `memtable.rs`, `lsm/scan.rs`)**:
   `write_segment`, `read_segment`, `delete_segment`, `generate_deletion_proof`, `encode_chunk_key`, `encode_graph_key`, `collection_prefix`, `decode_tenant_id`, `tx_range`, `scan_range_into`, `shard_entry_counts` und `point_lookup_metrics`.

---

## 2. Architekturanalyse & Kontext

`contextra-store` ist der zentrale Persistenz-Crate in Ring 1. Die unaufgerufenen Methoden betreffen sicherheitskritische Sicherheitsnetze (WAL Migration nach V3/V4 Header-Update), reactive CDC Observer sowie Multi-Tenant Encoding Hilfsmittel. Die Funktionen garantieren die Einhaltung der Invarianten INV-WAL-LEGACY-KEY-1 und INV-TENANT-2.

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige CDC Observer & WAL Migration Auto-Execution Integration
* **Beschreibung**:
  Verdrahtung des `WalObserver` an das Event-System in `contextra-engine` für Echtzeit-Ereignisströme und automatisches Ausführen von `migrate_legacy_wal_keys` beim Start von `LsmStorage`.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. CDC Event Stream Pipeline, Observer Latency Benchmarks <1ms, und Migration Validation Suite)*.
* **Pro**:
  - Reaktive Agenten ohne Polling direkt am WAL-Append-Punkt.
  - Nahtlose automatische Altlasten-Migration bei Upgrades.
* **Contra**:
  - Geringes Risiko von Latenzspitzen im Append-Pfad bei langsamen Observern.

### Option B: Rückbau von CDC Observer & Legacy Migration Scaffolding
* **Beschreibung**:
  Entfernen des `WalObserver`-Subsystems und der `migrate_legacy_wal_keys`-Methoden.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Bereinigung von WAL IO)*.
* **Pro**:
  - Schlankerer WAL-Append-Hot-Path.
  - Einsparung von ca. 900 LOC in Ring 1.
* **Contra**:
  - Kein Migrationspfad für Legacy-WAL-Formate.
  - Verlust von reactive WAL-Tailer Fähigkeiten.

### Option C: Beibehaltung des Ist-Zustands als Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Belassen der Methoden in `contextra-store`. Die WAL-Migration bleibt explizit aufrufbar (INV-WAL-LEGACY-KEY-1 konform).
* **Aufwandsschätzung**: **0 Stunden**
* **Pro**:
  - Garantierte Fail-Closed Sicherheit gegen implizite Key-Fallbacks.
  - Sofort einsatzbereite CDC Observer Infrastruktur.
* **Contra**:
  - Unintegrierte öffentliche Persistenz-Methoden.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Beibehaltung des Zustands. Die explizite Migration schützt vor unbeabsichtigtem Fallback.

2. **Langfristig**:
   Anbindung der WAL-Observer an das Agent Event Sourcing System (Option A).
