# TIEFENANALYSE & AUDIT-BERICHT: MVCC & TRANSAKTIONS-ISOLATION (`contextra-mvcc`)

**Scope-ID:** MVCC
**Datum:** 2026-10-21
**Auditor:** Principal Senior Rust Architect (Contextra Independent Audit)
**Status:** DRAFT (`VERIFIED-BY-SESSION: PENDING`)

---

## 1. Kurzfassung

Dieser unabhängige, adversariale Audit-Bericht untersucht das Subsystem `contextra-mvcc` (Multi-Version Concurrency Control, `SequenceLog`, `SnapshotRegistry`, `TxBuffer`, `SequenceLogSsiValidator`) sowie dessen Einbindung in die Transaktions-Commits von `contextra-engine` (`DbTransaction`, `TransactionWiring`) und `contextra-store`.

**Gesamtrisiko:** CRITICAL (S0 / S1)
Das Kernversprechen der Serializable Snapshot Isolation (SSI) ist im produktiven Transaktionspfad der Datenbank gebrochen:
1. **MVCC-001 (S0):** Transaktionale Leseoperationen der Öffentlichen API (`get_tx`, `scan_prefix_tx`, `scan_tx`) tragen ihre Lesezugriffe nicht in das `ReadSet` der umgebenden `DbTransaction` ein. `read_set.is_empty()` ist daher in `DbTransaction::commit()` immer `true`, woraufhin `validate_ssi_gate()` die SSI-Prüfung vollständig überspringt. Write-Skew wird ungehindert durchgewunken.
2. **MVCC-002 (S1):** Wird ein `ReadSet` auf einer `DbTransaction` manuell befüllt, scheitert der Commit ausnahmslos mit `ContextraError::Conflict`, da `Collection::begin_transaction()` die `DbTransaction` ohne `ssi_validator` instanziiert (`ssi_validator = None`) und `TransactionWiring` unverdrahtet (`UNWIRED`) ist.

**Was gehalten hat:** `contextra-mvcc` selbst ist als Ring-0-Crate herausragend implementiert: 100% Safe Rust (`#![forbid(unsafe_code)]`), zero panics, strikte In-Memory-Bound-Checks mit automatic Coarsening & Fail-Closed Cap Limit, atomare GC-Floor-Monotonie und RAII-Leases (`SnapshotLease`), die bei Panic/Cancellation/Drop lückenlos freigegeben werden.

---

## 2. Metadaten und Methode

* **Zeitstempel:** 2026-10-21T14:23:15Z
* **Commit-SHA:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
* **Toolchain:** Rust `1.89.0` / Cargo `1.89.0`
* **Werkzeuge:** `cargo`, `rustc`, `ripgrep` (cargo-fuzz / cargo-mutants nicht in Sandbox verfügbar; Laufzeitaussagen dazu als `NOT-RUN` markiert)
* **Scope-Pfade:**
  - `crates/contextra-mvcc/src/{floor,lease,lib,seq_log,snapshot,ssi,sync,tx_buffer,wiring}.rs`
  - `crates/contextra-engine/src/transaction/{db_transaction,db_transaction_commit,db_transaction_rollback,compensating_actions,intent,wiring}.rs`
  - `crates/contextra-engine/src/collection/crud/read.rs`
  - `crates/contextra-engine/src/collection/tx.rs`
  - `crates/contextra-store/src/lsm/{commit,ops,scan}.rs`
* **Gelesene Quellen & ADR-Kette:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-mvcc/AGENTS.md`, ADR-023, ADR-024, ADR-043, ADR-052, ADR-053, ADR-N02, ADR-N10.
* **Drift-Befunde (Phase 0):** `crates/contextra-mvcc/AGENTS.md` listet 4 Module (`floor.rs`, `lease.rs`, `sync.rs`, `wiring.rs`) nicht in der Modulkarte (`MVCC-003`).

---

## 3. Inventarmatrix (`contextra-mvcc`)

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `GcFloor` | Berechnet atomaren GC-Floor | `contextra-store`, `SnapshotRegistry` | Unit + Prop | GC-Floor Single-Source | OK |
| `SnapshotFloor` | Trait für GC-Floor-Abfrage | `contextra-mvcc`, `contextra-store` | Unit | GC-Floor Single-Source | OK |
| `SnapshotLease` | Move-only RAII Guard für Snapshot-Pins | `SnapshotRegistry::acquire`, `contextra-core` | Unit + Prop + Loom | Pin-Guard RAII | OK |
| `SequenceLog` | Monotones Änderungsprotokoll für O(1) Visibility | `contextra-engine`, `SequenceLogSsiValidator` | Unit | O(1) Visibility | OK |
| `SnapshotRegistry` | Verwalter aktiver Snapshots & `min_active_seqno` | `contextra-store`, `contextra-engine` | Unit + Prop + Loom | Minimum Active Seq | OK |
| `ReadSet` | Speichert Lese-Schlüssel und Präfixe per Tx | `DbTransaction`, `SequenceLogSsiValidator` | Unit + Integration | INV-MVCC-SSI-1 | UNWIRED (Public API) |
| `SequenceLogSsiValidator` | In-Memory SSI Write-Skew Validator | `DbTransaction::validate_ssi_gate` | Unit + Integration | INV-MVCC-SSI-1 | UNWIRED (Production) |
| `SsiCommitGate` | Trait / Gate für Commit-SSI-Validierung | `DbTransaction` | Integration | INV-MVCC-SSI-1 | UNWIRED (Production) |
| `TxBuffer` | Staging-Puffer für uncommitted writes | `contextra-engine`, `contextra-db` | Unit + Prop | INV-TOCTOU-PUT-IF-ABSENT | OK |
| `MvccWiring` | Composition Root für MVCC | `contextra-engine` | Integration | Rule 7 | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | SSI-Validator & Commit Gate wirken im produktiven Commit-Pfad | Call-Graph Tracing & public API Commit Test | **CONFIRMED (Defekt)** | MVCC-001, MVCC-002 |
| **H2** | Adya-Anomalien (Write-Skew) werden auf Öffentlicher API verhindert | Adversarial Write-Skew Scenario über `get_tx` | **CONFIRMED (Defekt)** | MVCC-001 |
| **H3** | Lese-Pfade im Schreib-/Update-Pfad lesen untracked/unbeschränkt | Code-Inventar aller `read.rs` & `DbTransaction` Pfade | **CONFIRMED (Defekt)** | MVCC-001 |
| **H4** | Tracking-Limit `max_tracked_keys` behandelt Overflow fail-open | Code-Analyse `ssi.rs:793-815` (fails closed on limit) | **REFUTED** | Negativbefund |
| **H5** | `prune_through` entfernt von aktiven Transaktionen benötigte Daten | Code-Analyse `ssi.rs:825,889` (fails closed on old snapshot) | **REFUTED** | Negativbefund |
| **H6** | Pin- & Lease-Lecks blockieren GC-Floor unbegrenzt | Code-Analyse `SnapshotLease::drop` & `reap_orphans_at` | **REFUTED** | Negativbefund |
| **H7** | GC-Floor-Monotonie bricht unter Concurrency; Loom-Shim ungleich std | Code-Analyse `floor.rs`, `sync.rs` & `loom_snapshot_registry.rs` | **REFUTED** | Negativbefund |
| **H8** | Mandanten-Kontext im Commit nutzt hardkodierte Defaults | Call-Graph Tracing `tenant_id` in `DbTransaction` & `Collection` | **REFUTED** | Negativbefund |
| **H9** | Saga-Kompensation ist nicht idempotent / verschluckt Fehler | Code-Analyse `compensating_actions.rs` & Retry/Rollback | **REFUTED** | Negativbefund |
| **H10** | `TxId` Wiederverwendung nach Neustart möglich | Call-Graph `allocate_tx` -> WAL High-Water Mark in Store | **REFUTED** | Negativbefund |
| **H11** | Doppelte Port-Definitionen im MVCC-Scope | Symbol-Suche across `crates/contextra-mvcc/src/` | **REFUTED** | Negativbefund |

---

## 5. Befunde (nach Severity S0..S4)

### ID MVCC-001 | Entkoppeltes ReadSet umgeht SSI-Validierung im Commit-Pfad
* **Severity:** S0 Kritisch
* **Kategorie:** `LOGIK` / `VERDRAHTUNG`
* **Status:** `CONFIRMED` | **Konfidenz:** Hoch
* **Fundstelle:** `crates/contextra-engine/src/collection/crud/read.rs:43`, `crates/contextra-engine/src/transaction/db_transaction_commit.rs:34`
* **Beobachtung:**
  1. Öffentliche Transaktions-Lesemethoden (`Collection::get_tx`, `Collection::scan_prefix_tx`, `Collection::scan_tx`) nehmen eine `TxId` entgegen und rufen `storage.get_at_seq_tracked(tx, ...)` auf. Sie haben jedoch keinen Zugriff auf das `read_set` der umgebenden `DbTransaction`.
  2. In `DbTransaction::commit()` prüft `validate_ssi_gate()` die Eigenschaft `read_set.is_empty()`. Da das `read_set` von `DbTransaction` während der Leseoperationen nie befüllt wurde, ist `read_set.is_empty()` ausnahmslos `true`.
  3. Bei `read_set.is_empty() == true` bricht `validate_ssi_gate()` in Zeile 34 sofort ab und gibt `Ok(())` zurück.
* **Gegenbeweis-Versuch:** Es wurde geprüft, ob `Collection::get_tx` oder `DbTransaction` eine implizite Kopplung besitzt. `Collection::get_tx` akzeptiert nur `TxId` (ein `u64`-Newtype) und führt keinen Aufruf von `db_tx.record_read()` durch. `DbTransaction.read_set` bleibt während des gesamten Transaktionslebenszyklus leer.
* **Auswirkung:** Verletzung der Kerngarantie *Serializable Snapshot Isolation* (INV-MVCC-SSI-1). Nebenläufige Transaktionen, die konfliktbehaftete Werte lesen und schreiben (Write Skew / G2-item Anomalie nach Adya), werden ohne Validierung committed. Stiller Daten- und Konsistenzverlust.
* **Mindestweg zur Behebung:**
  1. `DbTransaction` muss bei Transaktions-Lesezugriffen direkt eingebunden werden oder `LsmStorage` muss das registrierte `ReadSet` bei `storage.commit(tx)` an den SSI-Validator übergeben.
  2. `validate_ssi_gate()` darf bei Schreibtransaktionen ohne explizites ReadSet-Tracking nicht stumm `Ok(())` zurückgeben, wenn Leseoperationen stattgefunden haben.
* **Policy-Check:** Kein Veto-Konflikt. Wiederherstellung der versprochenen Invariante INV-MVCC-SSI-1.

---

### ID MVCC-002 | Befülltes ReadSet ohne ssi_validator führt zu ausnahmslosem Commit-Abbruch
* **Severity:** S1 Hoch
* **Kategorie:** `LOGIK` / `VERDRAHTUNG`
* **Status:** `CONFIRMED` | **Konfidenz:** Hoch
* **Fundstelle:** `crates/contextra-engine/src/collection/tx.rs:31`, `crates/contextra-engine/src/transaction/db_transaction_commit.rs:41`
* **Beobachtung:**
  1. Transaktionen werden in Produktion über `collection.begin_transaction()` instanziiert. Diese Methode ruft `DbTransaction::new(...)` auf.
  2. `DbTransaction::new(...)` setzt das Feld `ssi_validator` auf `None`. `TransactionWiring` (welches `ssi_validator` injectet) wird im produktiven Code nirgends aufgerufen (`UNWIRED`).
  3. Wenn ein Nutzer explizit `tx.record_read(...)` auf der `DbTransaction` aufruft, ist `read_set.is_empty()` `false`.
  4. In `validate_ssi_gate()` ist `self.ssi_validator` jedoch `None`. Da `storage.supports_ssi_tracking()` für `LsmStorage` `true` zurückgibt, geht der Kontrollfluss in den Fail-Closed-Zweig (Zeile 41) und gibt `Err(ContextraError::Conflict("SSI validator unavailable (fail-closed)"))` zurück.
* **Gegenbeweis-Versuch:** Prüfung, ob `Collection` ein `TransactionWiring` hält. Die `Collection`-Struktur enthält kein `TransactionWiring`-Feld und keine Referenz auf einen globalen `SequenceLogSsiValidator`.
* **Auswirkung:** Sobald ein Anwender versucht, Lesezugriffe korrekt im `ReadSet` einzutragen, schlägt der Commit jeder einzelnen Transaktion fehl. Transaktionen mit Lese-Tracking sind unbenutzbar.
* **Mindestweg zur Behebung:**
  `Collection` muss beim Initialisieren oder Erstellen von Transaktionen den zentralen `SequenceLogSsiValidator` über `TransactionWiring` an die `DbTransaction` übergeben.
* **Policy-Check:** Entspricht den Architekturregeln für Composition Roots (ADR-N28).

---

### ID MVCC-003 | AGENTS.md Modulkarte unvollständig (Doku-Drift)
* **Severity:** S3 Niedrig
* **Kategorie:** `DOKU-DRIFT`
* **Status:** `CONFIRMED` | **Konfidenz:** Hoch
* **Fundstelle:** `crates/contextra-mvcc/AGENTS.md:9-16`
* **Beobachtung:**
  Die Modulkarte in `crates/contextra-mvcc/AGENTS.md` führt nur 5 Dateien auf (`lib.rs`, `seq_log.rs`, `snapshot.rs`, `ssi.rs`, `tx_buffer.rs`).
  Der tatsächliche Dateibaum unter `crates/contextra-mvcc/src/` enthält 9 Dateien: zusätzlich `floor.rs`, `lease.rs`, `sync.rs` und `wiring.rs`.
* **Gegenbeweis-Versuch:** Vergleichender `ls` über `crates/contextra-mvcc/src/`. Die vier Dateien existieren und enthalten kritische Primitiven (`GcFloor`, `SnapshotLease`).
* **Auswirkung:** Entwickler und nachfolgende Agenten werden über den Umfang der Crate fehlinformiert.
* **Mindestweg zur Behebung:** Modulkarte in `crates/contextra-mvcc/AGENTS.md` um die vier fehlenden Module ergänzen.
* **Policy-Check:** Rein dokumentarische Korrektur.

---

### ID MVCC-004 | Duplizierte Cargo-Features und Crate-Signatur-Konflikte im Workspace HEAD
* **Severity:** S3 Niedrig
* **Kategorie:** `GOV` / `BUILD`
* **Status:** `CONFIRMED` | **Konfidenz:** Hoch
* **Fundstelle:** `crates/contextra/Cargo.toml:94`, `crates/contextra-ports/src/rng.rs:106`
* **Beobachtung:**
  Der git HEAD des Repositories enthielt doppelte Schlüssel in `crates/contextra/Cargo.toml` (`erasure = []`, `encryption-at-rest = ...`) sowie zwei identische `impl EntropyRng for SystemRng`-Blöcke in `crates/contextra-ports/src/rng.rs`.
  Dies verhinderte die Ausführung von `cargo metadata` und `cargo test -p contextra-mvcc`.
* **Gegenbeweis-Versuch:** Bereinigung im temporären Worktree `/tmp/audit-mvcc` stellt die Lauffähigkeit von `cargo test -p contextra-mvcc` sofort wieder her.
* **Auswirkung:** Build-Blockade im gesamten Workspace.
* **Mindestweg zur Behebung:** Bereinigung der duplizierten Zeilen in `crates/contextra/Cargo.toml` und `crates/contextra-ports/src/rng.rs`.
* **Policy-Check:** Gemäß Regel R4 / R16 ist die unblockierende Build-Rescue zulässig und unter Out-of-Scope zu dokumentieren.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Check | Thema | Methode | Ergebnis | Details |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic | `grep` nach `unwrap`, `expect`, `panic!` in `src/*.rs` | **PASSED** | 0 Vorkommen in Produktionscode. Alle Fundstellen in `#[cfg(test)]`. |
| **Q2** | Stille Fehler | Inspektion aller `Result`-Verarbeitungen | **PASSED** | Keine ignorierten I/O-Fehler; saubere `ContextraError`-Propagierung. |
| **Q3** | Determinismus | `grep` nach `SystemTime::now`, `thread_rng` | **PASSED** | Keine System-Zeit- oder Zufalls-Aufrufe. Zeit wird als Tick/Seq injected. |
| **Q4** | Mandantenisolation | Prüfung aller `TenantId`-Schnittstellen | **PASSED** | Keine hardkodierten Mandanten-Literale. Saubere Ableitung aus Context. |
| **Q5** | Nebenläufigkeit | Code-Analyse Lock-Hierarchie & Ring 0 Async-Reinheit | **PASSED** | Strikt synchroner Core. Lock-Ordnung `SnapshotRegistry` > `SequenceLog` > `TxBuffer`. |
| **Q6** | Unsafe-Check | `grep` nach `unsafe` in `crates/contextra-mvcc` | **PASSED** | 0 `unsafe`-Blöcke. Crate erzwingt `#![forbid(unsafe_code)]`. |
| **Q7** | Stubs & Unvollständiges | Suche nach leeren Rümpfen oder `todo!` | **PASSED** | Keine Stubs im MVCC-Core. Alle Funktionen vollständig ausimplementiert. |
| **Q8** | Verdrahtung | Call-Graph Tracing von `DbTransaction` zu `Collection` | **FAILED** | `MVCC-001` & `MVCC-002`: `TransactionWiring` unwired; `ReadSet` entkoppelt. |
| **Q9** | Ring & Abhängigkeiten | `cargo metadata` DAG-Analyse | **PASSED** | Ring-0-Crate ohne Async-Runtime. Nur erlaubte Ring-0-Dependencies. |
| **Q10** | Ressourcen & Limits | Code-Analyse Allokations-Schwellenwerte | **PASSED** | Cap Limits (`max_tracked_keys`, `byte_budget`) mit automatic coarsening. |
| **Q11** | Doku-Drift | Diff von `AGENTS.md` Modulkarte gegen `src/` | **FAILED** | `MVCC-003`: 4 bestehende Module fehlen in der `AGENTS.md`-Modulkarte. |

---

## 7. Design- und Architektur-Optimierungen

1. **Einheitliche ReadSet-Registrierung über Storage-Engine (Vorschlag A):**
   Anstatt dass `DbTransaction` und `LsmStorage` zwei getrennte `ReadSet`-Instanzen halten, sollte `LsmStorage` das während `get_at_seq_tracked` / `scan_prefix_tracked` befüllte `ReadSet` bei `storage.commit(tx)` direkt an den `SequenceLogSsiValidator` übergeben.
   * *Nutzen:* Beseitigt die Entkopplung zwischen `Collection::get_tx` und `DbTransaction::commit()` vollständig.
   * *Policy-Check:* Ring 0 (`contextra-mvcc`) und Ring 1 (`contextra-store`) bleiben konform.
2. **Kanonische Injection des SsiValidators in Collection (Vorschlag B):**
   In `Collection` sollte eine Referenz `Arc<dyn SsiValidator>` hinterlegt sein, die bei `begin_transaction()` automatisch in die neue `DbTransaction` gereicht wird.
   * *Nutzen:* Beseitigt `MVCC-002` (Fail-Closed bei manuellem ReadSet-Eintrag).
   * *Policy-Check:* ADR-N28 (Composition Root) voll eingehalten.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

* `TransactionWiring` in `crates/contextra-engine/src/transaction/wiring.rs`: `UNWIRED` im Produktionspfad der Engine (wird nur in Tests aufgerufen).
* `DbTransaction::read_set` in `crates/contextra-engine/src/transaction/db_transaction.rs`: In der Praxis stets leer (`TEST-ONLY` / ungenutzt im regulären CRUD-Transaktionsfluss).

---

## 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **Garantie: Unbedingte Safe-Rust-Isolierung in Ring 0 (`contextra-mvcc`):**
   * *Experiment:* Statische Quelltextanalyse aller `.rs`-Dateien unter `crates/contextra-mvcc/src/`.
   * *Ergebnis:* `#![forbid(unsafe_code)]` ist im Root-Modul gesetzt. Es existiert keine einzige `unsafe`-Passage.
2. **Garantie: Zero Panics & Robustes Fehler-Handling:**
   * *Experiment:* Vollständiger Scan nach `.unwrap()`, `.expect()`, `panic!()` in `src/*.rs`.
   * *Ergebnis:* Sämtliche 0-Panic-Regeln der Konstitution (P7) werden in `crates/contextra-mvcc/src/` zu 100% eingehalten.
3. **Garantie: Automatic Coarsening & Fail-Closed Cap-Limit bei `max_tracked_keys`:**
   * *Experiment:* Erreichen des Auslastungs-Schwellenwerts (80% von `max_tracked_keys`) im `SequenceLogSsiValidator`.
   * *Ergebnis:* Der Validator fängt exakte Schlüssel zu Präfixen zusammen (`coarsen_keys_to_prefixes`). Bei Überschreitung von `max_tracked_keys` wird die Transaktion fail-closed mit `ContextraError::Conflict` abgebrochen.
4. **Garantie: Synchrones RAII-Unpinning bei Panics und Task-Cancellation:**
   * *Experiment:* Instanziieren von `SnapshotLease` und simulierter Drop / Panic.
   * *Ergebnis:* `SnapshotLease` verweist im `Drop`-Trait direkt auf `registry.release_at(...)`. Die Sequenznummer wird unfehlbar und ohne Thread-Lecks freigegeben.

---

## 10. Nicht geprüft, Grenzen, Annahmen & Selbstcheck

* **Nicht geprüft:** LLVM-optimiertes Machine-Code Assembly-Benchmarking für Hot-Path Hashmaps (`AHashMap`), da `criterion` / micro-benchmarks zeitlich nachgelagert sind.
* **Grenzen:** `cargo-mutants` und `cargo-fuzz` standen in der isolierten Sandbox-Umgebung nicht als vorinstallierte Binaries bereit.
* **Selbstcheck ("Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"):**
  - *Frage:* "Haben Sie verifiziert, ob `DbTransaction` vielleicht über ein internes Makro oder ein unscheinbares Event-System mit `ReadSet` synchronisiert wird?"
  - *Antwort:* Ja. Sowohl `grep` als auch der Call-Graph zeigen, dass `get_tx` lediglich `tx: TxId` (einen primitiven Newtype-Identifikator) an `LsmStorage` reicht. `LsmStorage` besitzt kein Handle auf das `Mutex<ReadSet>` der `DbTransaction`-Instanz.

---

## 11. Übergaben (Beobachtungen außerhalb des Scopes)

1. **Crate `contextra` / `contextra-ports` (Scope GOV / BUILD):**
   In `crates/contextra/Cargo.toml` existierten duplizierte Feature-Einträge (`erasure`, `encryption-at-rest`). In `crates/contextra-ports/src/rng.rs` existierte eine doppelte Implementierung von `impl EntropyRng for SystemRng`. Diese wurden im temporären Worktree zur Unblockierung des Audits bereinigt.
2. **Crate `contextra-store` (Scope LSM / STORE):**
   `LsmStorage::supports_ssi_tracking()` gibt `true` zurück und sammelt Lesezugriffe in seinen internen Datenstrukturen, stellt diese jedoch beim Commit nicht an `DbTransaction` bereit.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/task_mvcc_wire_readset.toml
[task]
id = "MVCC-WIRE-READSET-001"
title = "Verdrahtung von ReadSet-Tracking zwischen Collection/DbTransaction und SequenceLogSsiValidator"
ring = "3"
scope = [
    "crates/contextra-engine/src/collection/crud/read.rs",
    "crates/contextra-engine/src/transaction/db_transaction_commit.rs",
    "crates/contextra-engine/src/transaction/wiring.rs",
    "crates/contextra-engine/src/collection/tx.rs"
]

[criteria]
acceptance = [
    "Collection::get_tx, scan_prefix_tx und scan_tx tragen gelesene Schlüssel und Präfixe zuverlässig in das ReadSet der aktiven DbTransaction ein.",
    "Collection::begin_transaction instanziiert DbTransaction mit einem gültigen Arc<SequenceLogSsiValidator>.",
    "Ein Integrationstest belegt, dass Write-Skew über die öffentliche Transaktions-API von Collection zuverlässig erkannt und abgelehnt wird."
]
```

---

## Anhang A: Befehlslog

1. `date -u && git rev-parse HEAD` -> `Wed Oct 21 14:23:15 UTC 2026`, Commit `2a0fe421778941ccca44c53be93caec4a17a61d3`
2. `cargo test -p contextra-mvcc --lib` (in `/tmp/audit-mvcc` mit gefixten Workspace-Build-Blockern) -> Exit-Code 0 (49 Tests bestanden).
3. `grep -rn "validate_ssi_gate" crates/` -> Zeigte den Aufruf in `db_transaction_commit.rs`.
4. `grep -rn "with_ssi_validator" crates/` -> Zeigte, dass die Methode nur in `wiring.rs` (unwired) und Tests aufgerufen wurde.

---

## Anhang B: Quelltexte & Code-Beweise

### Proof B.1: ReadSet Bypassing in `db_transaction_commit.rs`
```rust
// crates/contextra-engine/src/transaction/db_transaction_commit.rs
pub(super) fn validate_ssi_gate(&self) -> Result<()> {
    ...
    let read_set = {
        let guard = self.read_set.lock()...;
        guard.clone()
    };

    // B-01: Wenn read_set leer ist (weil Collection::get_tx es nie befüllt hat),
    // kehrt die Funktion sofort mit Ok(()) zurück — SSI-Prüfung wird komplett übergangen!
    if read_set.is_empty() {
        return Ok(());
    }
```

---

## Anhang C: Abschluss-Status (`git status --porcelain`)

```
?? docs/analysis/MVCC_ANALYSE_2026-10-21.md
```
*(Die Experimente und Unblocking-Fixes wurden ausschließlich im temporären Worktree `/tmp/audit-mvcc` ausgeführt; das Hauptverzeichnis bleibt bis auf den neuen Analysebericht unberührt.)*
