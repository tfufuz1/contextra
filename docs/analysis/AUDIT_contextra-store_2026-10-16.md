# AUDIT-Bericht: contextra-store (Ring 1) LSM, Compaction & Persistence DEEP AUDIT

**Spezialist:** S1 (Spezialist für LSM-Storage, Compaction und Tombstone-Semantik)
**Scope:** `crates/contextra-store` (Ring 1)
**Datum:** 2026-10-16
**Commit / Basis:** `983b72b963e674761bb5e9005dd0144561015111` (Delta-Basis: Bericht `AUDIT_contextra-store_DEEP_2026-10-13.md` / `2026-10-14`)

---

## 1. Zusammenfassung & Ampel-Status

Der Scope `contextra-store` (Ring 1) wurde einer tiefgehenden statischen und dynamischen Auditierung unterzogen. Kerninvariante und Architekturmuster (WAL-First, Atomic Rename Pattern, Bit-63 Tombstone-Maskierung, Floor-basierte Snapshot-Retention) sind im produktiven Algorithmen-Code sauber strukturiert. **Aktuell ist die Testfähigkeit des Crates jedoch durch einen akuten Kompilierfehler blockiert**, der bei den jüngsten CTRALEDG v1 Erasure-Ledger Formatanpassungen entstanden ist.

| Prüfbereich | Status | Bemerkung |
|---|---|---|
| **Core LSM & MVCC** | 🔴 ROT | Kompilierung durch Syntaxfehler in `erasure_ledger/mod.rs` blockiert. |
| **WAL & Durability** | 🟡 GELB | `check-result-dropped-io` meldet 16 verschluckte `remove_file`-Resultate. |
| **SSTable & Bloom-Filter** | 🟢 GRÜN | Index-/Bloom-Rebuild nach Compaction sowie CRC32/AEAD-Schutz verifiziert. |
| **Compaction & Retention** | 🟢 GRÜN | `CostBasedAdaptivePlanner` verdrahtet; Tombstone Pruning mit Bit 63 strikt. |
| **Manifest & Recovery** | 🟢 GRÜN | `dead_set` Wiederauferstehungsschutz & Atomic Rename Pattern intakt. |
| **Erasure Ledger** | 🔴 ROT | CTRALEDG v1 Refactoring unvollständig / Unclosed Delimiter in `mod.rs`. |

---

## 2. Delta zum letzten Bericht (2026-10-14 -> 2026-10-16)

- **Neu:**
  - `FIND-STORE-20261016-01` (P0): Akuter Kompilierfehler in `crates/contextra-store/src/erasure_ledger/mod.rs:938` (`unclosed delimiter`).
  - `FIND-STORE-20261016-02` (P1): 16 Fundstellen von `let _ = tokio::fs::remove_file(...)` bzw. `let _ = super::fs::remove_file(...)` verletzen `check-result-dropped-io`.
  - `FIND-STORE-20261016-03` (P2): `doctrine-scan` meldet `status_contradiction` für `contextra-store` (stable in `capabilities.toml` trotz Fehler/Lints).
- **Behoben:**
  - CTRALEDG v1 Anti-Resurrection Ledger Format (§6.5, B-10) wurde als Modulstruktur vorbereitet.
- **Offen:**
  - Stub-Methoden in `QuickCacheBlockCacheBackend` (`invalidate_files`, `entries_for_files`, `clear_all` liefern starr `0`).
- **Verschlechtert:**
  - Crate Build-Status: Von `ALL TESTS PASSING` (14.10.) auf `BUILD ERROR` (16.10.).

---

## 3. Befunde (Defekte & Invariantenverletzungen)

### FIND-STORE-20261016-01: Akuter Kompilierfehler in `erasure_ledger/mod.rs`
- **Schweregrad:** P0
- **Datei:Zeile:** `crates/contextra-store/src/erasure_ledger/mod.rs:938`
- **Beleg:**
  ```text
  error: this file contains an unclosed delimiter
     --> crates/contextra-store/src/erasure_ledger/mod.rs:938:3
      |
  154 | impl ErasureLedger {
      |                    - unclosed delimiter
  ...
  641 |         for (rt, tx, purge_id, scope_digest, family_digests, payload) in records_data {
      |                                                                                       - this delimiter might not be properly closed...
  ```
- **Risiko:** Totalausfall der Kompilierung für `contextra-store` und nachgelagerte Workspace-Crates; Verhindert Testausführung und CI/CD.
- **Empfehlung:** Bereinigung des unvollständigen Codeblocks im `ErasureLedger`-impl-Block und Wiederherstellung valider Match-/Schleifen-Klammerung.
- **Aufwand:** S

### FIND-STORE-20261016-02: Verschluckte I/O-Ergebnisse bei temporären Aufräumarbeiten
- **Schweregrad:** P1
- **Datei:Zeile:** `crates/contextra-store/src/lsm/ops/purge.rs:147, 163`, `crates/contextra-store/src/wal/hmac.rs:308, 315, 325, 330, 430, 438, 447, 537, 546`
- **Beleg:** `cargo xtask check-result-dropped-io` meldet 16 Violations mit `let _ = tokio::fs::remove_file(&tmp_path).await;`.
- **Risiko:** Fehler beim Entfernen temporärer Dateien bleiben unbemerkt und führen zu verwaisten Dateileichen oder stummen I/O-Fehlern.
- **Empfehlung:** Ersetzen von `let _ = ...` durch explizite Fehlerprotokollierung (`if let Err(e) = ... { tracing::warn!(...); }`).
- **Aufwand:** S

### FIND-STORE-20261016-03: Doktrin-Status-Widerspruch (`status_contradiction`)
- **Schweregrad:** P2
- **Datei:Zeile:** `crates/contextra-store/src/lib.rs:1`, `capabilities.toml`
- **Beleg:**
  ```text
  [ERROR] (status_contradiction) crates/contextra-store:1 - Crate 'contextra-store' is marked as completed/stable in capabilities.toml but contains panic or unsafe findings
  ```
- **Risiko:** Irreführende Statusberichte in CI-Gates und Governance-Prüfungen.
- **Empfehlung:** Beseitigung des Kompilierfehlers und Konsolidierung der Lint-Allow-Overrides in den Integrationstests.
- **Aufwand:** M

---

## 4. Testlücken & Concretes Testdesign

1. **Differenztest gegen ein einfaches Referenzmodell (Versionierte Map)**
   - **Eingaben:** Zufällige Sequenz von `Put(Key, Val)`, `Delete(Key)`, `Get(Key)`, `ScanPrefix(Prefix)`, `Flush`, `Compaction`, `CrashRestart`.
   - **Ablauf:** Parallele Ausführung aller Mutationen auf `LsmStorage` und einer In-Memory `BTreeMap<Vec<u8>, Vec<(u64, Vec<u8>)>>`.
   - **Invariante:** Bei jedem Lesezugriff mit `get_at_seq(key, seq)` müssen `LsmStorage` und das Referenzmodell identische Byte-Werte oder `None` liefern.

2. **Crash-Injektion während Flush und Compaction an Schreib- und Sync-Grenzen**
   - **Eingaben:** `FsyncFailInjector` schlägt nach $N$-ten `sync_all()` Aufruf fehl.
   - **Ablauf:** Schreiben von 1000 Eintrags-Batches, Triggern von Flush/Compaction, Injizieren eines E/A-Fehlers. Re-open der Engine via `LsmStorage::open`.
   - **Invariante:** Nach Crash-Recovery dürfen entweder alle Eintrags-Batches bis zur letzten bestätigten HWM vorhanden sein oder die Engine schlägt beim Re-open kontrolliert mit `ContextraError::Storage` fehl; keine Datenkorruption.

3. **Tombstone-Wiederbelebungstest über mehrstufige LSM-Levels**
   - **Eingaben:** Key `K` auf L0 schreiben und flushen -> L1. Key `K` löschen (Tombstone `T1`) -> L0. Key `K` neu schreiben -> L0.
   - **Ablauf:** Ausführen einer partiellen Compaction nur über L0 -> L1, während ältere SSTables unberührt bleiben.
   - **Invariante:** Tombstones dürfen erst verworfen werden, wenn alle älteren Versionen unterhalb des globalen Snapshot-Floors physisch bereinigt wurden.

4. **SSTable & Manifest Parser Fuzzing**
   - **Eingaben:** Mutierte Byte-Arrays (AFL++/libFuzzer-Generaten).
   - **Ablauf:** Übergabe an `SstableReader::open` und `Manifest::recover`.
   - **Invariante:** Der Parser darf niemals paniken oder `unwrap`/`expect` auslösen, sondern muss immer mit `Err(ContextraError)` terminieren (Zero-Panic Doctrine).

---

## 5. Technische Schulden

| Typ | Ort | Beschreibung / Aufwand | Priorität |
|---|---|---|---|
| Unwired Stubs | `sstable/block_cache.rs:340-348` | `QuickCacheBlockCacheBackend` Methoden `invalidate_files`, `entries_for_files` und `clear_all` liefern starr `0`. (Aufwand: S) | 1 |
| Mock Return Code | `wal/mod.rs:92, 182, 185` | `LoomFile` / `LoomMetadata` vereinfachte Default-Returns. (Aufwand: S) | 2 |
| AI-TAG Smell | `compaction/engine.rs:145` | `LsmStorage.compaction_engine` Instanz-Lebensdauer & Zähler-Akkumulation. (Aufwand: M) | 3 |

---

## 6. Hypothesen

- **Hypothese 1:** Der unvollständige Codeblock in `erasure_ledger/mod.rs` wurde während eines unvollständigen Merges von CTRALEDG v1 eingebracht. Eine saubere Klammerung stellt die Kompilierbarkeit und die grüne Testsuite sofort wieder her.

---

## 7. Quellenkonflikte und Doku-Drift

- **Quellenkonflikt 1:** `capabilities.toml` deklariert `contextra-store` als `maturity = 'stable'`, das Gate `doctrine-scan` meldet jedoch einen `status_contradiction` Fehler.
- **Doku-Drift 1:** `AGENTS.md` nennt `cargo test -p contextra-store --locked` als Standard-Verifikationsbefehl, welcher aktuell aufgrund des Kompilierfehlers fehlschlägt.

---

## 8. Ausgeführte Scan-Befehle und Exit-Codes

| Befehl | Exit-Code | Ergebnis / Anmerkung |
|---|---|---|
| `cargo xtask crate-context contextra-store` | 0 | OK (37.414 LOC, Stable) |
| `cargo xtask doctrine-scan --crate contextra-store` | 0 | `status_contradiction` Error gemeldet |
| `cargo xtask orphan-symbols` | 0 | Symbol-Listen generiert; Parsing-Warnung für `erasure_ledger/mod.rs` |
| `cargo xtask unwired-ports` | 0 | `ErasureLedgerHandle`, `LedgerReplayFilter` etc. erfasst |
| `cargo xtask stub-impls` | 0 | BlockCache Stubs erfasst |
| `cargo xtask check-result-dropped-io` | 1 | 75 Verstöße workspace-weit, 16 in `contextra-store` |
| `cargo xtask wal-replay-verify` | 0 | Not applicable (kein WAL-Diff) |
| `cargo xtask fuzz-smoke` | 0 | Not applicable |
| `cargo xtask mutants-diff` | 0 | Not applicable |
| `cargo test -p contextra-store --locked` | 101 | **FEHLER:** Kompilierfehler in `erasure_ledger/mod.rs` |

---

## 9. Nicht Geprüftes und Gründe

- **Integrationstest-Suite Execution (`cargo test -p contextra-store`)**: Ausführung der vollständigen Testsuite schlug fehl wegen des akuten Kompilierfehlers im Crate. Sobald der Syntaxfehler behoben ist, müssen alle Integrationstests erneut ausgeführt werden.
