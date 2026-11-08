# Implementierungsspezifikation & Arbeitsplan: `contextra-store`

> **Ring:** 1 | **Klasse:** B (Storage Bundle) | **Code-Budget:** 13400 LOC | **Unsafe:** Streng verboten (`#![forbid(unsafe_code)]`)
> **Architekturrolle:** Autarkes LSM-Storage Triebwerk, WAL Durability, Block-Cache, SSTable-Compaction und Scanner/Purger
> **Spezifikationsbezug:** `docs/SPEC.md` §4, §5, `crates/contextra-store/SPEC.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-store` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

1. **Modulare Einordnung in Klasse B:** Der Crate funktioniert als Kernbaustein des kohärenten Storage-Bundles (`types`, `ports`, `mvcc`, `crypto`, `sys`, `store`). Externe Interaktion erfolgt primär über die Ports aus `contextra-ports`.
2. **Entkopplung der Löschkette:** Sämtliche juristischen Begrifflichkeiten (DSGVO Art. 17), Nachweisketten, Deletion-Proofs und Saga-Orchestrierungen verbleiben exklusiv in `contextra-erasure`. Datenhaltende Module implementieren ausschließlich neutrale, funktionale Bereinigungs-Ports (`Purger`, `LayerScanner`).
3. **Design by Contract (DbC) & Invarianten:** Alle Vorbedingungen, Nachbedingungen und Systeminvarianten (INV-STORE-1 bis INV-STORE-5) werden strikt geprüft (Fail-Closed).
4. **Zero Shared Mutable State:** Nebenläufigkeit wird über deterministisches Message-Passing (Flusher-Actor), atomare Sequenznummern und Lock-Handoff im Group-Commit abgesichert.
5. **Code- & Unsafe-Disziplin:** `#![forbid(unsafe_code)]` ist im gesamten Crate verbindlich gesetzt. Keine unwrap(), expect() oder Panics im Produktivcode.

---

## 2. Modul- & Komponentenarchitektur

### 2.1 Interne Struktur & Verantwortung (Tatsächliche Pfade)

| Komponente / Modul | Tatsächlicher Dateipfad | Verantwortung & Schnittstellen |
|---|---|---|
| `lsm` | `src/lsm/` (`engine.rs`, `commit.rs`, `flush.rs`, `group_commit.rs`, `recovery.rs`, `scan.rs`, `ops/`) | LsmStorage Orchestrator, Commit, Recovery, Group-Commit |
| `wal` | `src/wal/` (`encode.rs`, `flusher.rs`, `hmac.rs`, `io.rs`, `replay.rs`) | Write-Ahead Log mit Flusher-Actor, HMAC-Kette, Replay und Open-Heal |
| `memtable` | `src/memtable.rs` | Nebenläufige geschardete BTreeMap mit Sequenznummern und Tombstones |
| `sstable` | `src/sstable/` (`builder.rs`, `reader.rs`, `block_cache.rs`, `bloom.rs`, `stream.rs`, `block_search.rs`) | Block-basierte SSTables (V4 Standard), Bloom-Filter, Block-Cache |
| `manifest` | `src/manifest/` (`core.rs`, `entry.rs`, `recovery.rs`, `rollover.rs`) | Transaktionales Manifest für SSTable-Generationen |
| `compaction` | `src/compaction/` (`engine.rs`, `adaptive.rs`, `config.rs`, `merge_operator.rs`, `retention.rs`) | Tiered & Leveled Compaction, `MergeOperator`, Tombstone-Bereinigung |
| `scanner` | `src/scanner/` (`mod.rs`, `memtable.rs`, `sstable.rs`, `wal.rs`, `purge.rs`) | Purger & LayerScanner Implementierung für externe Audit-Adapter |
| `fsync` | `src/fsync.rs` | Zentralisierte, Fehler-propagierende Sync-Operationen (INV-FSYNC-1) |
| `tenant_codec`| `src/tenant_codec.rs` | Mandanten-Präfixierung und Schlüssel-Isolation |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types, contextra-ports, contextra-mvcc, contextra-crypto, contextra-sys`
- **Verbotene Abhängigkeiten:** Höhere Ringe (`contextra-engine`, `contextra-db`, `contextra-vector`, `contextra-graph`, `contextra-sandbox`).
- **Konsumierte / Implementierte Ports:** Spezifiziert in `SPEC.md` und `AGENTS.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-STO-1: Bereinigung der Löschketten-Relikte
- **Fokus / Betroffene Dateien:** `src/kv/mod.rs`, `src/kv/segment.rs`, `src/kv/delete_mode.rs`
- **Problemstellung & Baukasten-Ziel:** Vollständiges Entfernen von Shredding-, DSGVO- und Deletion-Proof-Zitaten; Entflechtung der CompactionEngine.
- **Status:** In Arbeit (DEBT-01).

### WP-STO-2: Purger & LayerScanner Implementierung
- **Fokus / Betroffene Dateien:** `src/scanner/purge.rs`, `src/scanner/mod.rs`
- **Problemstellung & Baukasten-Ziel:** Implementierung der neutralen Ports für bereichsweise Purge-Kompaktion (`compact_range`) und Residual-Scans.
- **Status:** Implementiert in `src/scanner/` (Commit `4043bb903`). Ziel für Welle 4 ist die Auslagerung in `contextra-erasure-adapters`.

### WP-STO-3: WAL Durability & Commit-Abbruch-Sicherheit
- **Fokus / Betroffene Dateien:** `src/wal/flusher.rs`, `src/wal/hmac.rs`, `src/wal/io.rs`
- **Problemstellung & Baukasten-Ziel:** Synchrones HMAC-Rollback bei Timeout/Abbruch; Flusher prüft geschlossene Acks vor Disk-Sync; dauerhafte Vergiftung bei `fsync`-Fehlern.
- **Status:** Abgeschlossen (Commits `08128447d`, `4abbae56a`).

### WP-STO-4: Vfs-Port Umstellung
- **Fokus / Betroffene Dateien:** `src/sstable/builder.rs`, `src/sstable/reader.rs`, `src/wal/io.rs`, `src/manifest/core.rs`
- **Problemstellung & Baukasten-Ziel:** Vollständige Umstellung aller Dateioperationen auf `contextra_ports::Vfs`; In-Memory-Testbarkeit ohne Dateisystemabhängigkeit.
- **Status:** Geplant für WP 4.4 / D4.

### WP-STO-5: Write-Fencing & Lock-Handoff Invariante
- **Fokus / Betroffene Dateien:** `src/lsm/group_commit.rs`, `src/lsm/engine.rs`
- **Problemstellung & Baukasten-Ziel:** Lock-Handoff Invariante zur totalen Ordnung physischer WAL-Appends und Vermeidung von HMAC-Ketten-Races.
- **Status:** Abgeschlossen (Commit `b635ff958`, dokumentiert in ADR-0XX-P0A).

---

## 4. Test-, Benchmark- & Qualitätssicherungsplan

### 4.1 Unit-, Integration- & Conformance-Tests
- **Unit-Tests:** Umfassende Testabdeckung für Kernfunktionen und Randfall-Behandlungen in allen Modulen.
- **Port-Konformität:** Validierung gegen die Prüfsuiten in `contextra-testkit`.
- **Traceability:** Tests annotieren Spezifikationsanforderungen.

### 4.2 Property-Based Testing & Fuzzing
- Testen aller Invarianten unter zufälligen Eingaben (`tests/proptest_model_based.rs`, `tests/proptest_sstable_zero_copy_block_decode.rs`).
- Fuzzing aller Parser-, Deserialisierungs- und Eingabepfade (`fuzz/fuzz_targets/`).

### 4.3 Mutations- & Chaos-Testing
- Chaos-Tests für Bitflips, Memory-Druck, Prozessabbrüche und Stromausfallsimulationen (`tests/chaos_*`).

---

## 5. Definition of Done (DoD) für diesen Baustein

1. Alle deklarierten Arbeitspakete sind vollständig implementiert oder als geplante Schritte sauber inventarisiert.
2. `cargo check -p contextra-store`, `cargo test -p contextra-store` und `cargo clippy -p contextra-store` laufen fehlerfrei durch.
3. Code-Budget von maximal 13400 LOC wird eingehalten.
4. Keine unzulässigen Querabhängigkeiten zu höheren Ringen.
5. Die Crate-README dokumentiert den Einsatz innerhalb des Storage-Bundles mit einem lauffähigen Minimalbeispiel.
