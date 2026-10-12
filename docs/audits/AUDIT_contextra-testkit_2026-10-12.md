# Audit-Bericht: `contextra-testkit` (Testinfrastruktur)

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect für Contextra
**Crate:** `contextra-testkit` v0.1.0 (`crates/contextra-testkit/src/`)
**Git HEAD:** `cf65dbcba86807c380a9dcbfd66523cb41528d18`
**Session Hash:** `547fe5db`
**Claim Ticket:** `testkit-audit`
**Klassifizierung:** Tooling-Ring Testinfrastruktur
**Sicherheitseinstellung:** `#![forbid(unsafe_code)]`

---

## Executive Summary

Der Crate `contextra-testkit` stellt zentrale, deterministische Test-Mocks und Utilities (`InMemoryStorageEngine`, `ManualClock`, `FaultVfs`, `ReferenceModel`) für das gesamte Contextra-Workspace zur Verfügung. Da Modul- und Integrationstests im gesamten System auf diesen Komponenten aufbauen, muss die Testinfrastruktur frei von Panics (`unimplemented!()`), korrekt entkoppelt und bezüglich Fehler-Injektion verlässlich sein.

Das Audit untersuchte die Prüfpunkte **P1** (ManualClock Monotonität & API-Klarheit), **P2** (InMemoryStore Vollständigkeit & Trait-Abdeckung), **P3** (FaultVfs Fehler-Simulationsarten & Lücken), **P4** (Zero-Unsafe Garantie) und **P5** (Dependency-DAG-Reinheit im Tooling-Ring).

---

## Prüfpunkt P1: ManualClock Monotonität & API-Unterscheidung

### 1. Monotonie & Zeitänderung (`set_nanos` vs. `advance`)
- `ManualClock` verwendet intern eine atomare Variable `nanos: AtomicU64`.
- **`advance(&self, duration: Duration)`:** Erhöht den Nanosekunden-Zähler streng monoton mittels `AtomicU64::fetch_add`.
- **`set_nanos(&self, nanos: u64)`:** Setzt den Zähler direkt via `AtomicU64::store`.
  - *Befund:* `set_nanos` erlaubt das Rückwärtssetzen der Uhr auf einen kleineren Wert als den aktuellen Zeitstempel.
  - *Bewertung:* Für Testfälle (z. B. Simulation von Abklingzeiten, TTL-Ablauf, NTP-Zeitsprüngen oder Timeouts) ist ein manuelles Zurücksetzen legitim und notwendig. Es muss jedoch dokumentiert sein, um fehlerhafte Erwartungen in Tests mit Monotonie-Anforderung zu vermeiden.
- **Floating-Point Cast Clippy-Warnung:**
  - In `crates/contextra-testkit/src/manual_clock.rs:58` berechnet `now_system_time()`:
    `UNIX_EPOCH + Duration::from_nanos((self.now_secs_f64() * 1_000_000_000.0) as u64)`
  - Dies führt unter `cargo clippy -- -D warnings` zu zwei Clippy-Linter-Fehlern:
    - `clippy::cast_possible_truncation`
    - `clippy::cast_sign_loss`
  - *Empfehlung:* Anstelle der Konvertierung über `f64` sollte direkt `self.now_nanos()` verwendet werden: `UNIX_EPOCH + Duration::from_nanos(self.now_nanos())`.

### 2. API-Klarheit gegenüber `SystemClock`
- `ManualClock` implementiert den Port-Trait `contextra_ports::Clock` (`now_unix_nanos(&self) -> u64` und `monotonic_nanos(&self) -> u64`).
- Zusätzlich bietet `ManualClock` spezifische Steuerungsmethoden (`new`, `advance`, `set_nanos`, `now_nanos`, `now_secs_f64`, `now_system_time`).
- *Ergebnis:* Die API unterscheidet sich klar von `SystemClock`. Eine Verwechslung im Produktivcode ist ausgeschlossen, da Steuerungsmethoden wie `advance` und `set_nanos` nicht Bestandteil des `Clock`-Traits sind.

---

## (1) InMemoryStore-Vollständigkeit (P2)

`InMemoryStorageEngine` wurde mit der Schnittstelle `StorageEngine` aus `contextra-ports` abgeglichen.

### Methoden-Vergleichstabelle (`StorageEngine` Trait vs. `InMemoryStorageEngine`)

| Trait-Methode (`StorageEngine`) | Implementierungs-Status in `InMemoryStorageEngine` | Verhalten / Anmerkung |
| :--- | :---: | :--- |
| `get` | **Implementiert** | Holt Wert direkt aus Staged-Map / Main-Map |
| `get_at_seq` | **Implementiert** | Prüft Versionierung (`val_seq <= seq`) |
| `get_tracked` | *Trait-Default* | Delegiert an `self.get(key)` ohne SSI ReadSet Tracking |
| `get_at_seq_tracked` | *Trait-Default* | Delegiert an `self.get_at_seq(key, seq)` ohne SSI ReadSet Tracking |
| `scan_prefix_tracked` | *Trait-Default* | Delegiert an `self.scan_prefix(prefix)` ohne SSI ReadSet Tracking |
| `supports_ssi_tracking` | *Trait-Default* | Gibt `false` zurück |
| `put` | **Implementiert** | Transaktionales Staging in `StagedTxMap` |
| `put_if_absent` | *Trait-Default* | Gibt `ContextraError::capability_unsupported("put_if_absent")` zurück |
| `put_batch` | *Trait-Default* | Sequentieller Loop über `self.put` |
| `delete` | **Implementiert** | Setzt `StagedOp::Delete` im Transaktions-Puffer |
| `delete_many` | *Trait-Default* | Sequentieller Loop über `self.delete` |
| `delete_prefix` | *Trait-Default* | Führt `scan_prefix` aus und löscht Treffer über `delete_many` |
| `commit` | **Implementiert** | Atomares Anwenden aller Staged Operations mit Inkrement von `seq_no` |
| `rollback` | **Implementiert** | Verwirft Staged-Operations der Transaktion |
| `rollback_to_tx` | **Implementiert** | Verwirft gesamten Staging-Puffer (`clear()`) |
| `flush` | **Implementiert** | No-Op (`Ok(())`) |
| `stats` | **Implementiert** | Errechnet Byte-Größen aus In-Memory-Map |
| `last_seq_no` | **Implementiert** | Atomares Auslesen von `seq: AtomicU64` |
| `last_tx_id` | **Implementiert** | Auslesen der zuletzt committeten Transaktions-ID |
| `pin_checkpoint` | **Implementiert** | No-Op (`Ok(())`) |
| `unpin_checkpoint` | **Implementiert** | No-Op (`Ok(())`) |
| `scan_prefix` | **Implementiert** | Präfix-Scan auf geordneter `BTreeMap` |
| `scan_prefix_bounded` | *Trait-Default* | Paginierter Wrapper um `scan_prefix` |
| `scan_prefix_at` | *Trait-Default* | Gibt `ContextraError::capability_unsupported("snapshot_read_at")` zurück |
| `scan` | **Implementiert** | Bounded Scan über `BTreeMap::range` |
| `scan_bounded` | *Trait-Default* | Gibt `ContextraError::capability_unsupported("scan_bounded")` zurück |

### Befund zu `unimplemented!()` / Panics:
- **Zero Panics:** In `in_memory_store.rs` existiert **kein** `unimplemented!()`, `todo!()` oder `panic!()` Aufruf.
- **Trait-Defaults:** Nicht explizit überschriebene optionale Methoden (`put_if_absent`, `scan_prefix_at`, `scan_bounded`) nutzen das Standardverhalten des Traits und liefern bei Aufruf ein strukturiertes `ContextraError::CapabilityUnsupported` zurück. Tests schlagen somit bei fehlender Unterstützung sauber mit einem Fehler fehl, anstatt unvermittelt zu paniken.

---

## (2) FaultVfs-Fault-Typen & Abdeckung (P3)

`FaultVfs` ist eine In-Memory-Dateisystem-Simulation zur gezielten Fehler-Injektion in Tests.

### 1. Unterstützte Fault-Typen
- **`fail_writes_after: Option<usize>`:** Provoziert nach N erfolgreichen Schreibvorgängen den Fehler `io::ErrorKind::WriteZero`.
- **`fail_syncs_after: Option<usize>`:** Provoziert nach N erfolgreichen `sync()`-Aufrufen den Fehler `io::ErrorKind::Other` ("Simulated fsync I/O failure").
- **`fail_reads_after: Option<usize>`:** Provoziert nach N erfolgreichen Lesevorgängen den Fehler `io::ErrorKind::UnexpectedEof`.
- **`fail_all: bool`:** Globale Sperre: Jeder I/O-Aufruf schlägt sofort mit `io::ErrorKind::Other` fehl.
- **`trigger_crash()` / `crashed: AtomicBool`:** Simuliert einen harten Systemabsturz. Alle nachfolgenden I/O-Aufrufe schlagen mit `io::ErrorKind::BrokenPipe` fehl.

### 2. Kritische Fehlende Fault-Typen für Store- / WAL-Tests
1. **Torn Writes / Partial Writes:** `write_file(&self, path, data)` ersetzt den Inhalt in der internen Map atomar im Speicher. Es existiert keine Möglichkeit, nur einen Teilpuffer (z. B. die ersten N Bytes eines WAL-Eintrags) zu schreiben, um das Absturzverhalten bei unvollständigen Schreibvorgängen (Torn Writes) zu testen.
2. **Bit-Rot / WAL-Korruption:** Es fehlen Injektionsmethoden zur gezielten Manipulation (Bit-Flips, Truncation) von bestehenden Dateien im VFS, um die Prüfsummenvalidierung (CRC32/BLAKE3) und Recovery-Logik von `contextra-store` zu testen.
3. **Granularität per Pfad:** Fehler-Limits (`fail_writes_after` etc.) wirken global über alle Dateien hinweg, nicht spezifisch pro Dateipfad.
4. **Dateisystem-Verzeichnisoperationen:** `FaultVfs` bietet `write_file`, `read_file` und `sync`, unterstützt jedoch keine Verzeichnisstruktur-Faults (z. B. fehlgeschlagene Rename-, Unlink- oder Directory-Sync-Operationen).

---

## Prüfpunkt P4: Zero-Unsafe-Prüfung

- **Compiler-Direktive:** `#-[#forbid(unsafe_code)]` ist in `crates/contextra-testkit/src/lib.rs` verankert.
- **Codebase-Grep:** `grep -rn "unsafe" crates/contextra-testkit/src/`
  - Ergebnis: Ausschliesslich Vorkommen in Kommentaren und Modul-Invarianten.
  - **0 `unsafe` Blöcke, 0 `unsafe` Funktionen.**

---

## Prüfpunkt P5: Dependency-DAG & Ring-Reinheit

`Cargo.toml` Deklaration:
```toml
[dependencies]
contextra-ports = { workspace = true }
contextra-types = { workspace = true }
bytes = { workspace = true }
parking_lot = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt"] }
```

- **Ring-Einordnung:** `contextra-testkit` befindet sich im **Tooling-Ring**.
- **DAG-Analyse:** Der Crate importiert ausschließlich Ports (`contextra-ports`), Types (`contextra-types`) sowie allgemeine Dienstprogramm-Crates. Er importiert **keine** Produktiv-Crates wie `contextra-store`, `contextra-engine` oder `contextra-core`.
- *Ergebnis:* Es existieren keine zirkulären Abhängigkeiten.

---

## Test- & Lint-Ausführung

### Unit- & Integrationstests
```bash
cargo test -p contextra-testkit --locked -- --nocapture
```
- **Ergebnis:** `14 passed; 0 failed; 0 ignored` (8 Unit-Tests in `src/`, 6 Integrationstests in `tests/`).
- **Log:** `logs/audits/testkit-test.log`

### Clippy
```bash
cargo clippy -p contextra-testkit --all-targets -- -D warnings
```
- **Ergebnis:** `2 clippy errors` in `manual_clock.rs:58` (`clippy::cast_possible_truncation`, `clippy::cast_sign_loss`).
- **Log:** `logs/audits/testkit-clippy.log`

---

## (3) VERDICT & EVIDENCE

```
================================================================================
VERDICT: PASSED_WITH_OBSERVATIONS
================================================================================
Die Testinfrastruktur `crates/contextra-testkit/src/` ist vollständig panic-frei
(P2, 0 unimplemented!/todo!), verwendet kein unsafe code (P4, #![forbid(unsafe_code)]),
und hält die Ring-Layering-Vorgaben strikt ein (P5, nur contextra-ports/types).
In ManualClock wurden Rückwärtssprünge dokumentiert und Clippy-Guss-Befunde (P1)
identifiziert; FaultVfs erfordert künftige Erweiterungen für Torn-Write/WAL-Tests (P3).

[EVIDENCE: P1_MANUAL_CLOCK_BACKWARD_TIME_SETTING_POSSIBLE]
[EVIDENCE: P1_MANUAL_CLOCK_CLIPPY_FLOAT_CAST_FINDING]
[EVIDENCE: P2_IN_MEMORY_STORE_NO_PANICS_ZERO_UNIMPLEMENTED]
[EVIDENCE: P2_IN_MEMORY_STORE_14_EXPLICIT_12_TRAIT_DEFAULTS]
[EVIDENCE: P3_FAULT_VFS_WRITE_READ_SYNC_CRASH_SIMULATED]
[EVIDENCE: P3_FAULT_VFS_MISSING_TORN_WRITE_BITROT_FAULTS]
[EVIDENCE: P4_ZERO_UNSAFE_FORBID_UNSAFE_CODE_VERIFIED]
[EVIDENCE: P5_NO_PRODUCTION_DEPENDENCIES_TOOLING_RING_OK]
[EVIDENCE: TEST_PASSED_14_TESTS_OK]
[EVIDENCE: CLIPPY_FAILED_2_WARNINGS_FLOAT_CAST_MANUAL_CLOCK]
================================================================================
```

### Empfehlungen für Nachfolge-Tasks:
1. **Clippy-Fix in `ManualClock`**: `now_system_time()` umstellen auf `UNIX_EPOCH + Duration::from_nanos(self.now_nanos())` zur Vermeidung von Floating-Point Cast-Warnungen.
2. **`FaultVfs` Erweiterung**: Einbauen von Partial-Write/Torn-Write-Injektion und gezielter Dateikorruption für erweiterte Store/WAL-Recovery-Tests.
