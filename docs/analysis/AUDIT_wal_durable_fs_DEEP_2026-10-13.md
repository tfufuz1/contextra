# Tiefenanalyse AUDIT-Bericht: Write-Ahead Log (WAL) & Dauerhafter I/O (Ring 0 & Ring 1)

**Spezialist:** Spezialist für WAL, Recovery, dauerhaften I/O und Change Data Capture
**Commit Hash:** `59f853cb`
**Datum:** 2026-10-13
**Vorgänger-Berichte:**
- `docs/audits/AUDIT_wal-subsystem_DEEP_2026-10-12.md`
- `docs/audits/AUDIT_contextra-store_2026-10-12.md`
**Session Status:** `VERIFIED-BY-SESSION: OK`

---

## Kopf & Zusammenfassung

Dieser Bericht dokumentiert die vollständige Tiefenanalyse des WAL-Subsystems in `crates/contextra-store/src/wal/` (Ring 1) sowie der dauerhaften I/O- und Dateisystem-Primitive in `crates/contextra-durable-fs/src/` (Ring 0). Es wird die Einhaltung aller Sicherheits- und Durabilitätsinvarianten (P1, INV-DURABILITY-RING, INV-WAL-LEGACY-KEY-1, INV-WAL-TRUNCATION-1, INV-FSYNC-1, INV-ENOSPC-1, ADR-029) geprüft.

---

## 1 Scope und Abdeckung

| Datei | Pfad | Zeilen (ca.) | Gelesen | Begründung / Anmerkung |
|---|---|---|---|---|
| `lib.rs` | `crates/contextra-durable-fs/src/lib.rs` | 420 | Ja | Kern-I/O-Primitives (`atomic_replace`, `durable_remove`, `sync_dir`, `DurableHandle`, `DirLock`) |
| `fault_injector.rs` | `crates/contextra-durable-fs/src/fault_injector.rs` | 130 | Ja | Trait `FaultInjector` & `NoFault` Fehlerinjektor |
| `vfs.rs` | `crates/contextra-durable-fs/src/vfs.rs` | 180 | Ja | VFS Abstraktion und `RealFs` |
| `journal.rs` | `crates/contextra-durable-fs/src/journal.rs` | 110 | Ja | `PendingDeleteJournal` zur Crash-Recovery von Löschvorgängen |
| `scrub.rs` | `crates/contextra-durable-fs/src/scrub.rs` | 190 | Ja | Zero-Overwriting & Sicherheitsprüfungen (`ScrubPolicy`, `ScrubRefusal`) |
| `health.rs` | `crates/contextra-durable-fs/src/health.rs` | 90 | Ja | Fail-closed Gesundheitstracker (`HealthState`, `AtomicHealthState`) |
| `mod.rs` | `crates/contextra-store/src/wal/mod.rs` | 890 | Ja | WAL lifecycle, Versioning (`V1`/`V2`/`V3`), Open/Heal und Modul-Orchestrierung |
| `flusher.rs` | `crates/contextra-store/src/wal/flusher.rs` | 490 | Ja | Background Flusher Actor, Batch-Coalescing, Poisoning & Set-Len-Truncation |
| `io.rs` | `crates/contextra-store/src/wal/io.rs` | 380 | Ja | Synchrones/Asynchrones Entry-Scanning, Headers, Length-Header Guards |
| `hmac.rs` | `crates/contextra-store/src/wal/hmac.rs` | 560 | Ja | HMAC-SHA256 Chaining, Retirement Marker & Rekeying, UUID Sidecars |
| `encode.rs` | `crates/contextra-store/src/wal/encode.rs` | 280 | Ja | WAL Entry Frame Serialisierung, CRC32, Typisierte Payload-Header |
| `replay.rs` | `crates/contextra-store/src/wal/replay.rs` | 270 | Ja | Replay Stream/Mmap Execution, Filtering & Tail-Truncation-Abläufe |
| `retire.rs` | `crates/contextra-store/src/wal/retire.rs` | 140 | Ja | WAL Segment Retirement & Cleanups |
| `replay_filter.rs` | `crates/contextra-store/src/wal/replay_filter.rs` | 60 | Ja | `WalReplayFilter` Trait |
| `witness_scan.rs` | `crates/contextra-store/src/wal/witness_scan.rs` | 110 | Ja | Document Erasure Witness Scans über WAL-Segmente |

---

## 2 Inventar der pub-Items

### `crates/contextra-durable-fs`
- `pub fn sync_dir(dir: &Path) -> std::io::Result<()>` (lib.rs:260) — Produktiv genutzt
- `pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> std::io::Result<()>` (lib.rs:265) — Produktiv / Tests
- `pub fn atomic_replace(path: &Path, bytes: &[u8]) -> std::io::Result<()>` (lib.rs:280) — Produktiv genutzt
- `pub fn atomic_replace_with_injector<F: FaultInjector>(...) -> std::io::Result<()>` (lib.rs:286) — Produktiv / Tests
- `pub fn durable_remove(path: &Path) -> std::io::Result<()>` (lib.rs:335) — Produktiv genutzt
- `pub fn durable_remove_with_injector<F: FaultInjector>(...) -> std::io::Result<()>` (lib.rs:341) — Produktiv / Tests
- `pub fn scrub_and_remove(path: &Path) -> std::io::Result<ScrubReceipt>` (lib.rs:363) — Produktiv / API
- `pub fn scrub_and_remove_with_injector<F: FaultInjector>(...) -> std::io::Result<ScrubReceipt>` (lib.rs:369) — Nur Test / Optional
- `pub struct DurableHandle<F: FaultInjector>` (lib.rs:75) — Produktiv genutzt
- `pub struct DirLock` (lib.rs:185) — Produktiv genutzt
- `pub trait FaultInjector` (fault_injector.rs:20) — Produktiv / Tests
- `pub struct NoFault` (fault_injector.rs:40) — Produktiv / Standard
- `pub struct PendingDeleteJournal` (journal.rs:15) — Produktiv / Recovery
- `pub fn scrub_if_safe(...)` (scrub.rs:50) — Produktiv genutzt
- `pub struct AtomicHealthState` (health.rs:25) — Produktiv / Monitoring

### `crates/contextra-store/src/wal`
- `pub struct Wal` (mod.rs:125) — Produktiv genutzt
- `pub enum WalVersion` (encode.rs:15) — Produktiv / Versionierung
- `pub struct WalEntry` (encode.rs:45) — Produktiv / Encoded Frames
- `pub enum WalOp` (encode.rs:85) — Produktiv / Operations
- `pub struct WalFlusherConfig` (flusher.rs:115) — Produktiv / Configuration
- `pub trait WalReplayFilter` (replay_filter.rs:10) — Produktiv / Filtering
- `pub struct WalReplayReport` (replay.rs:20) — Produktiv / Metrics
- `pub trait ReplayProgressSink` (replay.rs:35) — Produktiv / Progress Tracking

---

## 3 Fachliche Antworten & Prüfkatalog A–M

### Beantwortung der Fachlichen Prüffragen

1. **Fehlerpropagation bei write und fsync:**
   - In `contextra-durable-fs`: `DurableHandle::flush()` und `write()` propagieren I/O-Fehler direkt als `std::io::Error` und vergiften das Handle (`is_poisoned = true`, INV-FSYNC-1). `atomic_replace` und `durable_remove` propagieren alle Systemfehler strikt mit `?`.
   - In `contextra-store/src/wal/`: `WalFlusher` fängt Write- und Sync-Fehler ab, führt `file.set_len(size_before)` und `file.seek(...)` aus, setzt `poisoned = true` und sendet `Err(ContextraError::Storage)` an alle wartenden Acks. Nachfolgende Appends scheitern sofort fail-closed.
   - **Ausnahme/Auffälligkeit:** In `PendingDeleteJournal::process` (`journal.rs:71`) wird bei unerwarteten Fehlern `self.flush()` mit `let _ = self.flush();` aufgerufen und das Ergebnis verworfen.

2. **Daten-Sichtbarkeit vor WAL-Eintrag (P1-Verletzung):**
   - **Geprüft, kein Befund.** In allen Schreibpfaden (`single-commit` in `lsm/ops/write.rs` und `group-commit` in `lsm/group_commit.rs`) wird `wal.append()` bzw. `wal.append_batch()` vollständig abgewartet und geflusht, bevor Einträge in die `MemTable` eingefügt und `last_committed_tx` aktualisiert werden.

3. **Verzeichnis-fsync nach Dateierzeugung/Rename & Plattformkapselung:**
   - **Geprüft, kein Befund.** `atomic_replace` führt nach `rename_durable` ein `flush_dir(parent)` aus. `durable_remove` führt nach `remove_file` ein `flush_dir(parent)` aus. Die Plattformunterschiede (Unix vs Windows) sind über `contextra_sys` vollständig gekapselt.

4. **Batch-Atomarität gegen Abbruch:**
   - Bricht ein Batch-Schreibvorgang halb geschrieben ab (I/O-Fehler oder Crash/Torn Write), setzt der Flusher `file.set_len(size_before)` ab, um ungefluste Page-Cache-Byte-Reste auf Disk abzuschneiden.
   - Bei einem echten Systemabsturz/Stromausfall sieht die Recovery beim nächsten Start (`Wal::open_with_config`) am Ende der Datei einen unvollständigen Frame. Die HMAC/CRC32-Prüfung schlägt an `pos >= file_size` fehl, woraufhin `wal.truncate(verified_end, last_hmac)` ausgeführt wird und die Datei sauber auf den letzten validen HMAC-Frame gekürzt wird.

5. **HMAC-Kette über Batch-Grenzen & Truncation-Erkennung:**
   - Der `WalFlusher` führt `flusher_last_hmac` fortlaufend über alle Batches hinweg.
   - Truncation-Erkennung: Das Manifest verwaltet eine High-Water-Mark HMAC (`expected_hwm`). Beim Recovery-Check (`recovery.rs`) wird `verify_wal_chain_completeness(&check_hmac, &expected_hwm)` erzwungen. Fehlt ein Teil des Logs vor der HWM, wird die Recovery mit `WalTruncationDetected` gestoppt.

6. **Observer Cursor & Commit-Pfad Latenz:**
   - `WalObserver` (`contextra_store::lsm::observer`) besitzt **keinen** automatisch persolidierten Disk-Cursor. CDC-Konsumenten müssen ihren eigenen Sequenz-Cursor verwalten.
   - Ein langsamer Konsument wird durch einen `BoundedObserverWorker` mit beschränkter Queue (256) und Circuit Breaker isoliert. Geht die Queue über oder überschreitet die Ausführung das Limit (`max_observer_latency`), greift ein Fallback/Circuit-Breaker, damit der globale Datenbank-Commit-Pfad nicht blockiert wird.

7. **Aktives WAL-Format (V3/V4), Migration & Legacy-Schlüssel:**
   - **Aktives Format:** WAL V3 mit HMAC-SHA256 Chaining und Header `b"WAL3"`. WAL V4 ist ein experimentelles/geplantes Format.
   - **Legacy-Schlüssel-Isolation (INV-WAL-LEGACY-KEY-1):** Ein Legacy-Schlüssel oder ein legacy V1/V2 Header kann **niemals** implizit aktiviert werden. `Wal::open` verweigert das Öffnen legacy-formtierter Logs strikt. Der Zugriff ist ausschließlich über `Wal::open_for_legacy_migration` möglich, andernfalls schlägt der Start mit `invalid_input` fehl.

8. **Statusmarker (Saga-Stages):**
   - Statusmarker (wie `PurgeOrchestrator` Intent-Dateien oder `PendingDeleteJournal`) sind **kein** Teil des atomaren LSM WAL-Commits. Sie werden als separate Dateien via `atomic_replace` geschrieben und besitzen eigene synchrone Fehlerpfade.

---

### Prüfkatalog A–M Ergebnisse

- **A. Unvollständige / angefangene Implementierungen:**
  - **Befund F-WAL-01:** `crates/contextra-store/src/wal/mod.rs:814` enthält einen offenen `TODO(Implementer)` bzgl. unvollständiger Truncation partieller Frames bei V3-AEAD-Verschlüsselung (`JULES-P01-02`).
- **B. Fehlerhafte Logik:**
  - **Befund F-DFS-01:** `PendingDeleteJournal::process` (`journal.rs:71`) ignoriert den Rückgabewert von `self.flush()` (`let _ = self.flush();`), wenn ein Löschfehler auftritt. Dadurch kann der Disk-Zustand des Journals von den verbleibenden In-Memory-Einträgen abweichen.
- **C. Fehlende Verdrahtung:**
  - `scrub_and_remove_with_injector` und `sync_dir_with_injector` in `contextra-durable-fs` sind als orphaned Symbole gemeldet, stellen aber beabsichtigte Injektions-APIs für Tests/Harnesses dar.
- **D. Dead Code:**
  - **Geprüft, kein relevanter Produktions-Befund.**
- **E. Fehlerbehandlung:**
  - `check-result-dropped-io` identifiziert 66 Stellen im Workspace, darunter `journal.rs:71` (`let _ = self.flush()`) und Temp-Datei-Löschungen in `hmac.rs` (`let _ = super::fs::remove_file(...)`).
- **F. Panics und Unwraps:**
  - **Geprüft, kein Befund.** In `contextra-durable-fs/src/lib.rs` ist `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` aktiviert.
- **G. Nebenläufigkeit:**
  - `WalFlusher` verwendet bounded Channels (`queue_capacity = 1024`) und Mutexes ohne async-Sperrkonflikte.
- **H. Determinismus:**
  - Systemzeit und Zufallswerte (z. B. UUIDs) nutzen definierte System- und Crypto-RNG-Schnittstellen.
- **I. Mandanten- und Berechtigungsgrenzen:**
  - Tenant Isolation ist in `TenantKeyCodec` verankert; WAL-Frames enthalten Transaktions- und Key-Payloads.
- **J. Architektur und Design:**
  - Striktes Ring-Layering: `contextra-durable-fs` (Ring 0) hängt von keiner internen Crate ab (`may_depend_on = []`), `contextra-store` (Ring 1) nutzt Ring 0 Primitive.
- **K. Code-Smells:**
  - Duplizierte Temp-Datei-Erstellungs-Patterns in `hmac.rs` für UUID-Sidecars und Rekeying-Marker.
- **L. Tests:**
  - test_journal_process_crash_resume_idempotency schlägt fehl.
- **M. Dokumentationsdrift:**
  - `wal-replay-verify` schlägt fehl aufgrund von verschobenen/umbenannten Test-Targets (`chaos_matrix`, `crash_recovery`, `crash_prefix_enumeration`).

---

## 4 Befunde

### Befund F-WAL-01: Offenes TODO zur Truncation partieller AEAD-Frames
- **Schwere:** Mittel
- **Kategorie:** A (Unvollständige Implementierung)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-store/src/wal/mod.rs:814`
- **Befund:** Der Code enthält den Hinweis `// TODO(Implementer): [P01 / F-02 / HIGH / JULES-P01-02] Unvollständige Truncation partieller Frames beim Replay auf Open (Invariante I-3)`.
- **Evidenz:**
  ```rust
  // crates/contextra-store/src/wal/mod.rs:814
  // TODO(Implementer): [P01 / F-02 / HIGH / JULES-P01-02]
  // Unvollständige Truncation partieller Frames beim Replay auf Open (Invariante I-3)...
  ```
- **Betroffene Invariante:** I-3 / Crash-Konsistenz
- **Empfehlung:** Sicherstellen, dass `verified_end` auch bei verschlüsselten WAL-Frames die exakte Byte-Grenze des letzten validen HMAC/AEAD-Tags trifft.
- **Red-Test-Vorschlag:** `cargo test -p contextra-store --test p01b_wal_replay_partial_tail_crc`

### Befund F-DFS-01: Verworfener Fehler bei Journal-Flush in PendingDeleteJournal
- **Schwere:** Hoch
- **Kategorie:** B (Fehlerhafte Logik) / E (Fehlerbehandlung)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-durable-fs/src/journal.rs:71`
- **Befund:** In `PendingDeleteJournal::process` wird bei einem Fehler während der Dateientfernung `self.flush()` mit `let _ = self.flush();` aufgerufen. Fällt `flush()` ebenfalls aus (z.B. wegen ENOSPC), wird dieser Fehler verschluckt und der verbleibende Zustand nicht auf Disk gesichert.
- **Evidenz:**
  ```rust
  // crates/contextra-durable-fs/src/journal.rs:71
  remaining.push(entry.clone());
  self.entries = remaining;
  let _ = self.flush(); // <-- Ignorierter I/O-Fehler!
  return Err(err);
  ```
- **Betroffene Invariante:** INV-ENOSPC-1 / INV-DURABLE-FS-SAFE
- **Empfehlung:** `self.flush()` mit `?` oder expliziter Fehlerbehandlung verknüpfen, sodass Journal-Schreibfehler ordnungsgemäß an den Aufrufer propagiert werden.
- **Red-Test-Vorschlag:** `cargo test -p contextra-durable-fs --test journal_crash_resume`

---

## 5 Vorgänger-Abgleich

| Vorgänger-ID | Titel / Thema | Status | Aktuelle Bewertung / Ort |
|---|---|---|---|
| W1 | HMAC-Chain-Verifikation | **BEHOBEN** | In `hmac.rs` und `encode.rs` voll wirksam. |
| W2 | Length Header Guard / Bounds | **BEHOBEN** | In `io.rs:114-135` (`MAX_WAL_ENTRY_SIZE = 64MB`) wirksam. |
| W3 | CRC32-Validierung | **BEHOBEN** | In `encode.rs:262` und `replay.rs:31` wirksam. |
| W4 | Flusher Fsync-Garantie | **BEHOBEN** | In `flusher.rs:271-294` und `421-490` wirksam. |
| W5 | Partial-Write Recovery & Truncation | **OFFEN** | Grundfunktion wirksam, aber TODO F-WAL-01 in `mod.rs:814` offen. |
| W6 | WAL Rotation Atomizität | **BEHOBEN** | In `flusher.rs:364` wirksam. |
| W7 | Multi-Segment Recovery | **BEHOBEN** | In `lsm/recovery.rs` wirksam. |

---

## 6 Testlage im Scope

- `cargo test -p contextra-durable-fs`: Unit-Tests bestehen; 1 Integrationstest (`journal_crash_resume`) schlägt fehl.
- `cargo test -p contextra-store --test wal_truncate_ordering`: Fällt aktuell wegen globaler Breaking Changes auf `main` in `lsm/engine.rs` / `purge/orchestrator.rs` durch den Compiler. Unit-Tests in `wal/tests` sind vorhanden.

---

## 7 Nicht prüfbar und warum

- **Fuzzing & Long-Running Mutation-Tests:** `fuzz-smoke` und `mutants-diff` konnten im Zeitbudget des Analyseauftrags nicht vollständig für stundenlange Läufe ausgeführt werden.

---

## 8 Out-of-scope-Befunde

- Behebung des unvollständigen Pattern Matchings in `crates/contextra-store/src/lsm/purge/dispose.rs` für `DisposePolicy::Disabled` durch Ergänzung von `DisposePolicy::Disabled => ScrubPolicy::Disabled`.
- Vorkommen von `let _ = std::io::stdout().flush()` in `crates/contextra-db/examples/`.

---

## 9 Priorisierte Maßnahmen

1. **Behebung F-DFS-01 (`PendingDeleteJournal` Flush Propagation):**
   - *Fehler:* In `journal.rs:71` den ungeschützten Aufruf `let _ = self.flush();` durch saubere Fehlerpropagation ersetzen.
   - *Akzeptanzbefehl:* `cargo test -p contextra-durable-fs`
2. **Behebung F-WAL-01 (`mod.rs:814` TODO Resolution):**
   - *Fehler:* Truncation-Byteberechnung für verschlüsselte WAL-Frames vervollständigen und TODO auflösen.
   - *Akzeptanzbefehl:* `cargo test -p contextra-store --test wal_truncate_ordering`
3. **Korrektur der Harness Target Registry (`wal-replay-verify`):**
   - *Fehler:* Aktualisieren der Testziel-Namen in `xtask` für `wal-replay-verify`.
   - *Akzeptanzbefehl:* `cargo xtask wal-replay-verify`

<!-- EVIDENCE: logs/audits/wal_durable_fs_2026-10-13/cargo_test_contextra_durable_fs.log (exit=101) -->
<!-- EVIDENCE: logs/audits/wal_durable_fs_2026-10-13/check_result_dropped_io.log (exit=1) -->
<!-- EVIDENCE: logs/audits/wal_durable_fs_2026-10-13/wal_replay_verify.log (exit=2) -->
<!-- EVIDENCE: logs/audits/wal_durable_fs_2026-10-13/cargo_test_wal_truncate_ordering.log (exit=101) -->
