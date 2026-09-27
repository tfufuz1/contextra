# Audit-Bericht: `contextra-testkit` (Testinfrastruktur)

**Datum:** 2026-09-27
**Crate:** `contextra-testkit` v0.1.0 (`crates/contextra-testkit/src/`)
**Auditor:** Principal Senior Rust Architect (Jules)
**Klassifizierung:** Tooling-Ring Testinfrastruktur
**Sicherheitseinstellung:** `#![forbid(unsafe_code)]`

---

## Executive Summary

Der Crate `contextra-testkit` stellt zentrale, deterministische Test-Mocks und Utilities (`InMemoryStorageEngine`, `ManualClock`, `FaultVfs`) für das gesamte Contextra-Workspace zur Verfügung. Da alle Modul- und Integrationstests im System auf diesen Komponenten aufbauen, muss die Testinfrastruktur frei von Panics (`unimplemented!()`), korrekt entkoppelt und bezüglich Fehler-Injection verlässlich sein.

Das Audit untersuchte die Prüfpunkte **P1** (ManualClock Monotonität & API-Klarheit), **P2** (InMemoryStore Vollständigkeit & Trait-Abdeckung), **P3** (FaultVfs Fehler-Simulationsarten & Lücken), **P4** (Zero-Unsafe Garantie) und **P5** (Dependency-DAG-Reinheit im Tooling-Ring).

---

## Prüfpunkt P1: ManualClock Monotonität & API-Unterscheidung

### 1. Monotonie & Zeitänderung (`set_nanos` vs. `advance`)
- `ManualClock` verwendet intern eine atomare Variable `nanos: AtomicU64`.
- **`advance(&self, duration: Duration)`:** Erhoht den Nanosekunden-Zähler streng monoton mittels `AtomicU64::fetch_add`.
- **`set_nanos(&self, nanos: u64)`:** Setzt den Zähler direkt via `AtomicU64::store`.
  - *Befund:* `set_nanos` erlaubt das Rückwärtssetzen der Uhr auf einen kleineren Wert als den aktuellen Zeitstempel.
  - *Bewertung:* Für Testfälle (z. B. Simulation von Abklingzeiten, TTL-Ablauf oder Zeitsprüngen) ist ein manuelles Zurücksetzen legitim. Es birgt jedoch das Risiko, dass Tests, die strenge Monotonie erwarten, bei unbedachtem Einsatz fehlschlagen. In der Dokumentation von `set_nanos` fehlt derzeit ein expliziter Hinweis darauf, dass Rückwärtssprünge die Verträge von `monotonic_nanos()` verletzen.

### 2. API-Klarheit gegenüber `SystemClock`
- `ManualClock` implementiert den Port-Trait `contextra_ports::Clock` (`now_unix_nanos(&self) -> u64` und `monotonic_nanos(&self) -> u64`).
- Zusaetzlich bietet `ManualClock` spezifische Steuerungsmethoden (`new`, `advance`, `set_nanos`, `now_nanos`, `now_secs_f64`, `now_system_time`).
- *Ergebnis:* Die API unterscheidet sich klar von `SystemClock`. Eine Verwechslung im Produktivcode ist ausgeschlossen, da Steuerungsmethoden wie `advance` und `set_nanos` nicht Bestandteil des `Clock`-Traits sind.

---

## (1) InMemoryStore-Vollständigkeit (P2)

`InMemoryStorageEngine` wurde mit der Schnittstelle `StorageEngine` aus `contextra-ports` abgeglichen.

### Methoden-Vergleichstabelle (`StorageEngine` Trait vs. `InMemoryStorageEngine`)

| Trait-Methode (`StorageEngine`) | Implementierungs-Status in `InMemoryStorageEngine` | Verhalten / Anmerkung |
| :--- | :---: | :--- |
| `get` | **Implementiert** | Holt Wert direkt aus Staged-Map / Main-Map |
| `get_at_seq` | **Implementiert** | Prüft Versionierung (`val_seq <= seq`) |
| `put` | **Implementiert** | Transaktionales Staging in `StagedTxMap` |
| `put_if_absent` | *Trait-Default* | Gibt `ContextraError::capability_unsupported("put_if_absent")` zurück |
| `put_batch` | *Trait-Default* | Sequentieller Loop über `put` |
| `delete` | **Implementiert** | Setzt `StagedOp::Delete` im Transaktions-Puffer |
| `delete_many` | *Trait-Default* | Sequentieller Loop über `delete` |
| `delete_prefix` | *Trait-Default* | Führt `scan_prefix` aus und löscht Treffer |
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
- **Zero Panics:** In `in_memory_store.rs` existiert **kein** `unimplemented!()` oder `todo!()` Aufruf.
- **Trait-Defaults:** Nicht überschriebene optionale Methoden (`put_if_absent`, `scan_prefix_at`, `scan_bounded`) nutzen das Standardverhalten des Traits und liefern bei Aufruf ein strukturierte `ContextraError::CapabilityUnsupported` zurück. Tests schlagen somit bei fehlender Unterstützung sauber mit einem Fehler fehl, anstatt unvermittelt zu paniken.

---

## (2) FaultVfs-Fault-Typen & Abdeckung (P3)

`FaultVfs` ist eine In-Memory-Dateisystem-Simulation zur gezielten Fehler-Injektion in Tests.

### 1. Unterstüzte Fault-Typen
- **`fail_writes_after: Option<usize>`:** Provoziert nach N erfolgreichen Schreibvorgängen den Fehler `io::ErrorKind::WriteZero`.
- **`fail_syncs_after: Option<usize>`:** Provoziert nach N erfolgreichen `sync()`-Aufrufen den Fehler `io::ErrorKind::Other` ("Simulated fsync I/O failure").
- **`fail_reads_after: Option<usize>`:** Provoziert nach N erfolgreichen Lesevorgängen den Fehler `io::ErrorKind::UnexpectedEof`.
- **`fail_all: bool`:** Globale Sperre: Jeder I/O-Aufruf schlägt sofort mit `io::ErrorKind::Other` fehl.
- **`trigger_crash()` / `crashed: AtomicBool`:** Simuliert einen harten Systemabsturz. Alle nachfolgenden I/O-Aufrufe schlagen mit `io::ErrorKind::BrokenPipe` fehl.

### 2. Kritische Fehlende Fault-Typen für Store- / WAL-Tests
1. **Torn Writes / Partial Writes:** `write_file(&self, path, data)` ersetzt den Inhalt in der internen Map atomar im Speicher. Es existiert keine Möglichkeit, nur einen Teilpuffer (z. B. die ersten N Bytes eines WAL-Eintrags) zu schreiben, um das Absturzverhalten bei unvollständigen Schreibvorgängen (Torn Writes) zu testen.
2. **Bit-Rot / WAL-Korruption:** Es fehlen Injektionsmethoden zur gezielten Manipulation (Bit-Flips, Abschneiden/Truncation) von bestehenden Dateien im VFS, um die Prüfsummenvalidierung und Recovery-Logik von `contextra-store` zu testen.
3. **Granularität per Pfad:** Fehler-Limits (`fail_writes_after` etc.) wirken global über alle Dateien hinweg, nicht spezifisch pro Dateipfad oder Handle.
4. **I/O-Abstraktion:** `FaultVfs` ist eine eigenständige Hilfsstruktur mit benutzerdefinierten Synchronmethoden (`write_file`, `read_file`, `sync`) und implementiert weder standardmäßige Rust I/O-Traits (`Read`, `Write`) noch asynchrone VFS-Schnittstellen aus `contextra-store`.

---

## Prüfpunkt P4: Zero-Unsafe-Prüfung

- **Compiler-Direktive:** `#![forbid(unsafe_code)]` ist in `crates/contextra-testkit/src/lib.rs` verankert.
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

### Unit-Tests
```bash
cargo test -p contextra-testkit --locked -- --nocapture
```
- **Ergebnis:** `7 passed; 0 failed; 0 ignored`
- **Log:** `/tmp/audit-testkit-test.log`

### Clippy
```bash
cargo clippy -p contextra-testkit --all-targets -- -D warnings
```
- **Ergebnis:** `0 errors, 0 warnings`
- **Log:** `/tmp/audit-testkit-clippy.log`

---

## (3) VERDICT & VERIFIED-BY-SESSION

**VERDICT: PASSED_WITH_OBSERVATIONS**

### Beobachtungen & Empfehlungen:
1. **`ManualClock::set_nanos`:** Sollte in der Dokudokumentation mit einem Hinweis versehen werden, dass Rückwärtssprünge die Monotonie-Anforderung von `monotonic_nanos()` verletzen.
2. **`FaultVfs` Erweiterung:** Für tiefgehende Recovery-Tests in `contextra-store` empfiehlt sich in Zukunft die Ergänzung von Partial-Write/Torn-Write-Injektion und gezielter WAL-Dateikorruption (Bit-Rot).
3. **`InMemoryStorageEngine`:** Sämtliche Trait-Methoden sind panic-frei und verhalten sich bei Aufruf nicht-unterstützter Funktionen durch `CapabilityUnsupported`-Fehler deterministisch.

VERIFIED-BY-SESSION: PENDING (TS: 2026-09-27T20:47:09Z)
