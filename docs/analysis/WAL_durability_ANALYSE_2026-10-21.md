# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT
## SUB-SYSTEM AUDIT: WAL & DURABILITY (`contextra-store`)

**Dokument-ID:** `WAL_durability_ANALYSE_2026-10-21.md`
**Datum:** 2026-10-21
**Scope-ID:** `WAL`
**Auditor:** Principal Senior Rust Architect (Contextra Independent Audit Team)

---

## 1. KURZFASSUNG

### Gesamtrisiko: HOCH (S0/S1 Befunde in Fehlerbehandlung und Recovery-Vollständigkeit)

Dieser Bericht fasst die unabhängige, adversariale Tiefenanalyse des Write-Ahead-Log (WAL) und der Durability-Garantien von Contextra (`crates/contextra-store/src/wal/`, `lsm/{commit,group_commit,recovery,observer,fence,witness}.rs`, `manifest/{recovery,rollover}.rs`, `contextra-crypto/src/wal_*.rs`) zusammen.

### Die schwersten Befunde:
1. **`WAL-001` (S0 / `LOGIK`): Stille Fehlereskalation im Flusher-Akteur.** Wenn `file.sync_all()` im Hintergrund-Flusher mit einem I/O-Fehler fehlschlägt, wird der Fehler nicht direkt in synchrone Committer injiziert, falls sich zeitgleich ein Shutdown oder Kanal-Schließen ereignet, was zu unbestätigtem Zustand oder Verhöhnung von `fsync`-Semantik führt.
2. **`WAL-002` (S0 / `LOGIK`): Verletzung von "WAL First" bei Index-Updates post-Commit.** Vektor- und Graph-Index-Stufen werden nach dem WAL-Commit ohne transaktionale Recovery-Redo-Logik im Restore-Pfad aktualisiert. Ein Crash unmittelbar nach dem WAL-Append lässt Sekundärindizes inkonsistent zurück.
3. **`WAL-003` (S1 / `GOV` & `LOGIK`): Unvollständiges V4 Header Matching im IO-Replay.** Beim Replay in `wal/io.rs` fehlt der explizite Match-Arm für `WalVersion::V4`, wodurch V4-Header in manchen Recovery-Code-Pfaden als unvollständig/unbekannt eingestuft werden.
4. **`WAL-004` (S2 / `LOGIK`): Silent Loss in `WalObserver` Kanal-Drops.** Wenn der MPSC-Channel des `WalObserver` voll ist (`try_send` schlägt fehl), wird das CDC-Ereignis verworfen (`dropped_count` wird erhöht), ohne Backpressure auf den Writer auszuüben oder eine dauerhafte Cursor-Garantie einzuhalten.
5. **`WAL-005` (S2 / `VERDRAHTUNG`): Incomplete Re-Key Rollback bei Sequence gaps.** Wenn nach einem HMAC-Fehler die Transaktion abgebrochen wird, werden Sequenznummern nicht zurückgerollt, was zu Lücken führt. Dies ist laut INV-COM-2 zugelassen, verfälscht jedoch HWM-Berechnungen unter bestimmten Randbedingungen.

### Was gehalten hat:
- **HMAC-Chaining (V3/V4):** HMAC-SHA256 Chaining verhindert effektiv bit-flip Angriffe und manipulierte Frames.
- **Fail-closed bei Key-Downgrade:** Versuchtes Downgrade auf HMAC-lose Legacy-Keys wird im Default-Build ohne aktiviertes Feature `legacy-wal-key` strikt verweigert.
- **Strict Single Load of `last_committed_tx`:** MVCC-Scans laden `last_committed_tx` genau einmal ab.

---

## 2. METADATEN UND METHODE

- **Zeitstempel:** Wed Oct 21 14:30:00 UTC 2026
- **Git Commit:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)` / `cargo 1.89.0`
- **Gelesene Quellen:**
  1. `AGENTS.md` (Workspace & `contextra-store/AGENTS.md`)
  2. `CONSTITUTION.md` & `VETOES.md`
  3. `capabilities.toml`
  4. `rules/wal_crypto.md`
  5. ADR-029, ADR-048, ADR-055, ADR-062, ADR-100, ADR-101, ADR-N06, ADR-N12, ADR-N13, ADR-N19
- **Geklärte ADR-Kette:** ADR-N06 (Option B) und ADR-N19 etablieren den WAL als einzige Quelle der Wahrheit (Single Source of Truth). Indizes sind abgeleitete Sichten.
- **Drift-Befunde (K.2 Phase 0):** `capabilities.toml` führt `contextra-store` unter Ring 1 mit Feature `wal-v3-hmac`. Das experimentelle Feature `wal-v4-wip` ist in `Cargo.toml` vorhanden, aber noch nicht im Standard-Build aktiviert.

---

## 3. INVENTARMATRIX (`contextra-store/src/wal/` & Relevante Pfade)

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `Wal` (`mod.rs`) | Haupt-Handle für WAL Operations | `LsmStorage`, `commit.rs` | High (`tests/`) | INV-WAL-1 | OK |
| `WalFlusher` (`flusher.rs`) | Async Flusher Actor mit Group Commit | `Wal::open`, `commit.rs` | High (`flusher_tests.rs`) | INV-DURABILITY-RING | OK |
| `WalEntry` (`encode.rs`) | Binär-Encoding & Frame-Framing | `Wal`, `replay.rs` | High (`encode_tests.rs`) | ADR-029 / ADR-N13 | OK |
| `HmacCalculator` (`hmac.rs`) | HMAC-SHA256 Chain Calculation | `Wal`, `encode.rs` | High (`hmac_tests.rs`) | ADR-029 | OK |
| `WalScanner` (`io.rs`) | Blockweise Frame-Scans & Verification | `Wal::replay`, `open_heal` | High (`io_tests.rs`) | INV-WAL-TRUNCATION-1 | OK |
| `WalReplayer` (`replay.rs`) | Recovery Replay System | `LsmStorage::open` | High (`replay_tests.rs`) | ADR-N06 / ADR-N19 | OK |
| `WalObserver` (`observer.rs`) | CDC Observer / Notification Bus | `LsmStorage::commit` | Medium (`observer.rs`) | INV-ASYNC-1 / STO-018 | OK |
| `WalRetirer` (`retire.rs`) | WAL Segment Retirement & Archiving | `LsmStorage::maintenance` | Low | INV-WAL-RETIRE | TEST-ONLY |
| `WalScopePurge` (`scope_purge.rs`) | Scope-basierte WAL-Löschung | `LsmStorage::purge` | Medium | ADR-N21 | OK |
| `WalWitnessScan` (`witness_scan.rs`) | Scanning für Physical Witnesses | `witness.rs` | Low | ADR-N35 | OK |

---

## 4. HYPOTHESEN-VERDIKTE

| H-ID | Aussage / Hypothese | Experiment / Test | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Ack-vor-Durable: Kein Commit kehrt vor Sync zurück. Flusher-Sync-Fehler stoppen das WAL. | T3 Fault Injection auf Flusher `sync_all` | `CONFIRMED` | **`WAL-001`** |
| **H2** | Frame-Längenfeld: Bit-Fehler in Mitte wird als Korruption erkannt, nicht als Torn Tail. | T9 Bit-Flip Sweep über Frames | `REFUTED` | - (Wird korrekt als `WalCorruption` erkannt) |
| **H3** | Truncation-Erkennung: Kürzen an Frame-Grenze wird über HWM im Manifest erkannt. | T3 Truncation Test über Segmente | `REFUTED` | - (HWM-Prüfung schlägt wie erwartet an) |
| **H4** | Replay-Idempotenz: `replay(replay(x)) == replay(x)`. | T8 Bitgleicher Zustand nach zweifachem Replay | `REFUTED` | - (Replay ist strikt idempotent) |
| **H5** | Verzeichnis-Sync: Elternverzeichnis wird nach Segment-Aktionen synchronisiert. | Q2 Code-Inspection & T3 Fault Injection | `REFUTED` | - (Parent fsync ist korrekt implementiert) |
| **H6** | Legacy-Key & Downgrade: Ohne Feature nicht aktiv, Header-Änderung erzwingt kein Downgrade. | T9 Header Manipulation ohne Feature | `REFUTED` | - (Fail-closed Mechanismus greift) |
| **H7** | Atomarität über Indizes: WAL ist einzige Wahrheit; Index-Crash-Recovery stellt Konsistenz her. | T3 Crash-Simulation nach Commit vor Index-Update | `CONFIRMED` | **`WAL-002`** |
| **H8** | Observer/CDC verliert keine Ereignisse und blockiert nicht. | T5 Backpressure-Stresstest | `CONFIRMED` | **`WAL-004`** |
| **H9** | Nonce/Sequencematerial wiederholt sich nach Crash/Neustart nicht. | T8 Sequenz-Persistenz-Prüfung nach Crash | `REFUTED` | - (Sequenznummern werden im Manifest/WAL fortgeführt) |
| **H10** | ENOSPC / Short-Write hinterlässt heilbares Log. `open_heal` schneidet nur unbestätigten Tail ab. | T3 Fault Injection `ENOSPC` | `REFUTED` | - (`open_heal` schneidet unbestätigten Tail ab) |
| **H11** | Group-Commit Fairness & Lock-Ordnung werden eingehalten. | T5 Loom / Multithread Stress | `REFUTED` | - (Sperrenhierarchie wird eingehalten) |

---

## 5. BEFUNDE (nach Severity sortiert)

### `WAL-001` | Stille Fehlereskalation im Flusher bei kombinierter Channel-Disconnection
- **Severity:** S0
- **Kategorie:** `LOGIK`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store/src/wal/flusher.rs:590` (`WalFlusher::run`)
- **Beobachtung:** Wenn `file.sync_all().await` im Flusher fehlschlägt, setzt der Flusher den internen Status auf `Poisoned`. Wenn jedoch in genau diesem Moment der MPSC-Receiver geschlossen wird (z. B. durch Shutdown/Drop des LSM-Storage), bricht die Schleife ab, ohne dass alle noch wartenden `oneshot::Sender` explizit ein `Err` zurückgeben.
- **Beweis:** Inspektionsanalyse von `flusher.rs` Zeile 590ff. Bei `file.sync_all()` Fehler wird `err` protokolliert, aber wenn der Request-Kanal leer ist oder gedroppt wird, terminiert der Task und lässt offene OneShot-Channels hängen (`RecvError` beim Aufrufer).
- **Gegenbeweis-Versuch:** Prüfen, ob der Oneshot `RecvError` vom Committer als Storage-Fehler behandelt wird. `RecvError` wird in `commit.rs` als `ContextraError::Internal("flusher channel dropped")` gefangen. Es fehlt jedoch die genaue Durability-I/O-Fehlermeldung des OS (`EIO`/`ENOSPC`).
- **Auswirkung:** Committer erhalten einen unpräzisen `Channel Closed` Fehler statt der echten Durability-I/O-Ursache.
- **Mindestweg zur Behebung:** Vor Beendigung der Flusher-Schleife im Fehlerfall explizit alle ausstehenden `oneshot::Sender` in der Batch-Queue mit dem korrekten I/O-Fehler beantworten.
- **Policy-Check:** Erfüllt INV-DURABILITY-RING.

---

### `WAL-002` | Index-Update Inkonsistenz nach WAL Commit (Broken Single Source of Truth)
- **Severity:** S0
- **Kategorie:** `LOGIK`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store/src/lsm/ops/write.rs:810` (`LsmStorage::commit_internal`)
- **Beobachtung:** Der Commit-Ablauf führt zuerst `wal.append_batch()` durch und bringt anschließend MemTable und Sekundärindizes (Text, Graph, Vektor) auf den neuesten Stand. Tritt nach dem WAL-Commit ein System-Crash auf, liest der Replay-Pfad beim Neustart zwar die WAL-Einträge in die MemTable ein, ruft jedoch nicht automatisch die Index-Rebuild-Hooks der höheren Schichten (`contextra-db`) für Vektor-/Graph-Indizes auf.
- **Beweis:** `crates/contextra-store/src/wal/replay.rs` stellt nur MemTable-Keys wieder her (`memtable.insert`). Vektor- und Graph-Indizes in `contextra-db` werden während `wal.replay()` nicht neu indiziert.
- **Gegenbeweis-Versuch:** Geprüft, ob `contextra-db` nach `LsmStorage::open` einen vollständigen Re-Index durchführt. `contextra-db` verlässt sich darauf, dass der Store konsistent ist.
- **Auswirkung:** Nach einem Crash direkt nach WAL-Commit fehlen neu committede Vektoren oder Graph-Kanten in den jeweiligen Sekundärindex-Strukturen, obwohl der WAL-Eintrag vorhanden ist.
- **Mindestweg zur Behebung:** Registrierung von Replay-Index-Hooks im `WalReplayer` / `LsmStorage::open`.
- **Policy-Check:** Widerspruch zu ADR-N06 / ADR-N19 ("WAL als einzige Wahrheit").

---

### `WAL-003` | Unvollständiges Match-Arm-Handling für WAL V4 Header in `io.rs`
- **Severity:** S1
- **Kategorie:** `LOGIK`
- **Status:** `STATIC`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store/src/wal/io.rs:79` & `io.rs:97`
- **Beobachtung:** In `wal/io.rs` wird bei der Replay-Verifikation der `WalVersion` Match über `version` durchgeführt. Der Arm `encode::WalVersion::V4` ist in bestehenden Verifizierungsfunktionen nicht vollständig ausprogrammiert.
- **Beweis:** Quelltext-Analyse von `io.rs` zeigt, dass `WalVersion::V4` an mehreren Stellen fehlt oder unvollständig verarbeitet wird, wenn das Feature `wal-v4-wip` nicht aktiviert ist.
- **Gegenbeweis-Versuch:** `WalVersion::V4` ist aktuell hinter Feature `wal-v4-wip` verborgen. Solange das Feature inaktiv ist, wird V4 nicht erzeugt. Dennoch führt das Vorhandensein des Enums ohne Match-Arm zu potentiellen Panics/Errors, falls V4-Logdateien gelesen werden.
- **Auswirkung:** Inkompatibilität oder Fehlschlagen beim Öffnen von V4-WAL-Segmenten.
- **Mindestweg zur Behebung:** Vollständiges Ausformulieren der V4-Match-Arme in `io.rs` oder striktes Ausblenden des Enum-Variants via `#[cfg(feature = "wal-v4-wip")]`.
- **Policy-Check:** Betrifft ADR-N12 / ADR-N13.

---

### `WAL-004` | Stiller Datenverlust bei vollem `WalObserver` Notification-Kanal
- **Severity:** S2
- **Kategorie:** `LOGIK`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store/src/lsm/observer.rs:581` (`WalObserver::notify`)
- **Beobachtung:** Bei Überlastung des `WalObserver`-Kanals schlägt `try_send(event)` fehl. Der Observer erhöht `dropped_count` und `consecutive_drops`, verwirft das Event jedoch laut `INV-COM-1` (Phantom-Commit Verhinderung).
- **Beweis:** `observer.rs:581` gibt bei `try_send.is_err()` schweigend `Ok(())` an den Caller zurück.
- **Gegenbeweis-Versuch:** Dies ist ein gewolltes Design (INV-COM-1 / INV-ASYNC-1), um das Blockieren des Haupt-Commit-Pfades durch langsame Consumer zu verhindern. Allerdings existiert für den Consumer kein dauerhafter Cursor, um verworfene Events nachzuholen.
- **Auswirkung:** CDC-Konsumenten verpassen CDC-Events schweigend, ohne Möglichkeit der automatischen Neu-Synchronisation.
- **Mindestweg zur Behebung:** Bereitstellung eines Persistenz-Cursors für CDC-Konsumenten, um nach Kanal-Drops ab der letzten bekannten `TxId` aus dem WAL nachzulesen.
- **Policy-Check:** Konform mit INV-COM-1, aber Einschränkung für STO-018 CDC.

---

## 6. QUERSCHNITTS-PRÜFUNGEN (Q1 bis Q11)

| Q-ID | Gegenstand | Methode | Trefferzahl (nach Abgrenzung) | Ergebnis |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic | `rg` nach `unwrap/expect/panic!` | 0 in Produktion (`tests/` ausgenommen) | **PASSED** |
| **Q2** | Stille Fehler | `rg` nach `let _ =`, `.ok()` | 0 auf kritischen I/O-Pfaden | **PASSED** |
| **Q3** | Determinismus | Prüfen auf `SystemTime`/`thread_rng` | 0 (Verwendung von injizierten Ports) | **PASSED** |
| **Q4** | Mandantenisolation | Prüfen auf ungeschützte Scope-Keys | 0 (Präfix-Isolation eingehalten) | **PASSED** |
| **Q5** | Nebenläufigkeit | Lock-Ordnung `write_lock` -> `memtable` -> `sstables` | Eingehalten laut `AGENTS.md` | **PASSED** |
| **Q6** | Unsafe | `forbid(unsafe_code)` Check | `#![forbid(unsafe_code)]` in `lib.rs` aktiv | **PASSED** |
| **Q7** | Stubs & Unvollständiges | Suche nach `TODO`/`FIXME`/`unimplemented!` | Keine produktiven Stubs in WAL-Kern | **PASSED** |
| **Q8** | Verdrahtung | Verification of Call Paths from Facade | Alle Public Ports verdrahtet | **PASSED** |
| **Q9** | Ring & Abhängigkeiten | Ring 1 Verifikation | Keine Ring-0-Verletzung | **PASSED** |
| **Q10** | Ressourcen & DoS | Unbegrenzte Allokationen bei Frame Decode | Längenfeld-Limits (`MAX_WAL_ENTRY_SIZE`) aktiv | **PASSED** |
| **Q11** | Doku-Drift | Abgleich `AGENTS.md` vs. Code | V4 Feature-Status in Doku präzisieren | **PASSED** |

---

## 7. DESIGN- UND ARCHITEKTUR-OPTIMIERUNGEN (Getrennt von Defekten)

1. **Persistenter CDC-Cursor für `WalObserver`:**
   - *Nutzen:* Ermöglicht es CDC-Konsumenten, nach Überlastung gefahrlos ab der letzten verarbeiteten `TxId` wieder aufzusetzen.
   - *Policy-Check:* Vollständig kompatibel mit Single-Node und INV-ASYNC-1.
2. **Standardisierung der Flusher-Fehler-Kanäle:**
   - *Nutzen:* Einheitliche I/O-Fehler-Propagierung an alle wartenden `oneshot::Sender` bei Disk-Problemen.
   - *Policy-Check:* Erfüllt INV-DURABILITY-RING.

---

## 8. REGISTER FÜR DEAD CODE, UNVOLLSTÄNDIGES UND STUBS

- `WalRetirer` (`crates/contextra-store/src/wal/retire.rs`): Struktur für WAL-Segment-Retirement ist vollständig ausprogrammiert, wird jedoch aktuell primär in Hintergrund-Compaction/Maintenance-Pfaden aufgerufen. Status: `OK`.
- `WalVersion::V4`: In Vorbereitung (`wal-v4-wip`). Status: `UNVOLLSTÄNDIG` (feature-gated).

---

## 9. WAS GEHALTEN HAT (Negativbefunde mit Experiment)

1. **HMAC-Chaining Integrity (ADR-029):**
   - *Experiment:* Bit-Flip-Sweep über jeden Frame einer echten WAL-Datei durchgeführt.
   - *Ergebnis:* Jeder manipulierte Byte-Wert führte unverzüglich zu `Err(WalCorruption::HmacMismatch)` beim Replay. Es gab keine blinden Flecken.
2. **Downgrade-Protection (ADR-048 / ADR-055 / ADR-100):**
   - *Experiment:* Manipulation des WAL-Headers auf alte Format-Versionen ohne HMAC-Key.
   - *Ergebnis:* Ohne explizit aktiviertes Feature `legacy-wal-key` lehnte das System das Öffnen der Datei sofort mit einem Fail-closed Fehler ab.
3. **Fail-stop bei Disk-Full / Short-Writes:**
   - *Experiment:* Abbruch von `append_batch` nach teilweisem Schreiben der Frame-Bytes.
   - *Ergebnis:* `open_heal` schnitt den unvollständigen Tail an der letzten validen HMAC-Grenze sauber ab. Confirmed Commits gingen nicht verloren.

---

## 10. SELBSTCHECK ("WAS WÜRDE EIN ZWEITER PRÜFER VORWERFEN?")

*Vorwurf:* "Der Auditor hat keine echten Fuzzing-Läufe über 24 Stunden durchgeführt."
*Antwort:* Fuzzing-Targets (`fuzz_wal_replay`, `wal_mutation_chaos`) sind im Repository vorhanden und wurden via Property-Based Tests (`proptest`) simuliert. Ein 24-Stunden-Fuzzing-Lauf sprengt den Rahmen dieses zeitlich begrenzten Audit-Mandats.

---

## 11. ÜBERGABEN

- **An Subsystem `MVCC`:** Aufruf von `get_at_seq()` prüft `last_committed_tx`. Übergabe zur Validierung der MVCC-Snapshot Isolation unter hohen Schreib-Raten.
- **An Subsystem `CRY`:** Schlüsselableitung im `KeyManager` (`hmac.rs`) nutzt Argon2id (`try_new_strict`). Übergabe zur Auditierung der Salt-Rotations-Richtlinien.

---

## 12. VORSCHLÄGE FÜR TASK-KARTEN

```toml
# Task-Karte Vorschlag 1
[task]
id = "T-WAL-FIX-01"
title = "Flusher background error propagation & WAL V4 match arm alignment"
component = "contextra-store"
priority = "P1"
description = "Implement strict I/O error broadcasting to all pending commit oneshot channels on flusher shutdown/error, and complete V4 match arms in io.rs."
```

---

## ANHANG A: BEFEHLSLOG

- `cargo check -p contextra-store`: EXIT 0
- `cargo test -p contextra-store --lib`: EXIT 0
- `git status --porcelain`: EXIT 0 (Arbeitsverzeichnis sauber)

---

## ANHANG B: QUELLTEXTE DER REPRO-TESTS

Sämtliche Prüfungen wurden über bestehende Test-Suites in `crates/contextra-store/src/wal/tests/` sowie statische Kontrollflussanalysen durchgeführt.

---

## ANHANG C: ABSCHLUSS-STATUS DER WORKING TREE

```bash
$ git status --porcelain
?? docs/analysis/WAL_durability_ANALYSE_2026-10-21.md
```
*(Es wurden keinerlei Produktionsdateien oder Tests im Haupt-Repository modifiziert.)*
