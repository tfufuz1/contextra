# Red-Team-Review-Bericht

## 1. Urteil
**Urteil: abgelehnt**
Die jüngsten Commits und PR-Berichte behaupten die Wiederherstellung der Kompilierbarkeit und Behebung zentraler Defekte, jedoch bricht `cargo check --workspace --all-targets` mit 49 Fehlern ab, Tests in `contextra-types` und `contextra-arch-tests` schlagen fehl, `contextra-erasure` kollidiert mit dualen Moduldeklarationen (`mod.rs` vs. Dateimodul), und angebliche Regressionstests (z. B. `T-DEL-LOCK-1`) sind tautologisch konstruiert, während der zugrundeliegende Defekt DEF-01 im Produktionscode unberührt bleibt.

---

## 2. Befundtabelle

| Behauptung | Widerlegungsversuch / Prüfung | Ergebnis | Evidenz |
|---|---|---|---|
| **C-01:** `contextra-ports`, `contextra-sys`, `contextra-crypto`, `contextra-mvcc`, `contextra-simd`, `contextra-durable-fs` bauen vollständig (ARCHITECTURE_REVIEW.md §1) | `cargo check -p <crate> --all-targets` je Crate ausgeführt | **Nicht widerlegt**: Alle 6 Crates kompilieren isoliert ohne Fehler. Glob-Re-Export-Warnungen vorhanden. | EV2 |
| **C-02:** `contextra-types` baut wieder; Serde-Fehler `[u8; 65]`/`[u8; 78]` behoben (ARCHITECTURE_REVIEW.md Executive Summary & Commit b92d847c) | `cargo check -p contextra-types --all-targets` und Testsuite ausgeführt | **Widerlegt**: `contextra-types --lib` baut zwar, aber `tests/serde_roundtrips_suite.rs` (3 Fehler: `FilterExpr` ungelöst) und `tests/error_and_witness_suite.rs` (8 Fehler: fehlende Typen `RequestContext`, `SagaRecord`, `ScanMethod`) scheitern beim Kompilieren. | EV2 |
| **C-03:** `T-DEL-LOCK-1` belegt, dass `delete()` das Lock über die gesamte Operation hält (DEF-01 gefixt in Commit bddc4cb7) | Quelltext `delete.rs` geprüft; Testanalyse in `update_delete_tests.rs`; Lock-Ablauf nachverfolgt | **Widerlegt (Tautologischer Test & falsche Behauptung)**: In `crates/contextra-engine/src/collection/crud/delete.rs:28` steht weiterhin unverändert `drop(_guard);` unmittelbar vor `delete_op`! Der Test ist ein Zirkelschluss: Er erwirbt `guard = col.kv_locks.lock_for("race_doc")` und prüft danach `col.kv_locks.try_lock_for("race_doc")`. Da derselbe Shard-Lock noch vom Test gehalten wird, liefert `try_lock_for` trivial immer `None`, unabhängig davon, was der Hintergrund-Task tut. | EV1 / EV2 |
| **C-04:** `T-KDF-1`: Alter fehlerhafter Wert `MIN_M_COST_KIB = 19456` wird abgelehnt, SPEC-Minimum 65536 KiB akzeptiert (Commit bddc4cb7) | `cargo test -p contextra-crypto --test wp32_spec_tests test_t_kdf_1_min_m_kib_rejects_old_buggy_value_19456`; Mutationsprüfung mit `MIN_M_KIB = 19456` in `kdf.rs` | **Nicht widerlegt**: Test läuft aktiv grün. Mutant `MIN_M_KIB = 19456` macht den Test deterministisch rot (`FAILED: assertion left == right failed: Err(Corrupt)`). | EV2 / EV3 |
| **C-05:** `scripts/verify_all.sh` führt vollständige Verifikation durch | Skriptanalyse und Ausführung `bash scripts/verify_all.sh` | **Widerlegt (Prüfskript-Falle)**: Skript prüft fmt nur für `contextra-testkit`, clippy/test nur für `contextra-testkit`. Es nutzt `ls -t target/debug/deps/*.rlib 2>/dev/null | head -n1 || true` (Kompilate-Falle) und generiert in Schritt 7 einen synthetischen Erfolgsreport (`rep.miri_passed = Some(true)`, `rep.coverage_lines_percent = Some(100.0)`), ohne dass Miri/Loom/Coverage real ausgeführt wurden. Bricht bei clippy mit Fehlern in `contextra-types` ab. | EV2 |
| **C-06:** Entfernung von Benchmark-Dateien in Commit 6f31b55a bereinigt Altlasten | Diff-Prüfung `benchmarks/` | **Teilweise widerlegt**: Zwar wurden veraltete Benchmarks entfernt, die auf entfernte Crates wie `contextra-db` zugriffen, jedoch wurden dadurch Benchmark-Messungen (Throughput/Latency/RSS) ersatzlos gestrichen, statt sie an das Ziel-API anzupassen. | EV1 |
| **C-07:** Drei-Säulen-Löschung in `contextra-erasure` einsatzbereit (Commit b92d847c) | `cargo check -p contextra-erasure --all-targets` ausgeführt | **Widerlegt**: Kompiliert nicht (`E0761`). 10 Module existieren doppelt als Datei und als Unterverzeichnis mit `mod.rs` (`vault`, `gate`, `journal`, `scope`, `saga`, `purge`, `measure`, `report`, `ledger`, `restore`). Zudem verstößt die Verwendung von `mod.rs` gegen Invariante D-90 / Test `test_t_arch_7_naming_and_facade_structure`. | EV2 |
| **C-08:** Review-Dokumente (`ARCHITECTURE_REVIEW.md` etc.) enthalten keine verbotenen Begriffe | Volltext-Suche nach verbotenen Begriffen gemäß SPEC 6.4 / T-LINT-1 | **Widerlegt**: `ARCHITECTURE_REVIEW.md` Zeile 146 enthält verbotenen Begriff „rechtskonform“ (*„ist die einzige rechtskonform ehrliche Produktaussage“*). | EV1 |

---

## 3. Änderungen + Tests mit Befehlsausgabe

### Test 1: Mutationsprüfung C-04 (`wp32_spec_tests.rs`)
- **Befehl:**
  ```bash
  cargo test -p contextra-crypto --test wp32_spec_tests test_t_kdf_1_min_m_kib_rejects_old_buggy_value_19456
  ```
- **Ausgabe (Original - Grün):**
  ```text
  running 1 test
  test test_t_kdf_1_min_m_kib_rejects_old_buggy_value_19456 ... ok
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
  ```
- **Mutant (`MIN_M_KIB = 19456` in `kdf.rs`) - Rot:**
  ```text
  test test_t_kdf_1_min_m_kib_rejects_old_buggy_value_19456 ... FAILED
  assertion `left == right` failed: T-KDF-1: from_header(19456) muss Corrupt zurückgeben (< 65536 KiB)
    left: Ok(KdfParams { m_kib: 19456, t: 3, p: 4 })
   right: Err(Corrupt)
  ```

### Test 2: Architektur- und Governance-Gates (`contextra-arch-tests`)
- **Befehl:**
  ```bash
  cargo test -p contextra-arch-tests
  ```
- **Ausgabe (Fehlschlag):**
  ```text
  failures:
      test_t_arch_5_crate_headers_and_descriptions
      test_t_arch_6_exports_and_dev_dependencies
      test_t_arch_7_naming_and_facade_structure
  test result: FAILED. 13 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
  ```
- **Details der Verletzungen:**
  - `T-ARCH-7`: 10x `mod.rs` in `contextra-erasure` (Verstoß gegen D-90).
  - `T-ARCH-6`: Unzulässige Glob-Re-Exporte `pub use <mod>::*;` in `contextra-crypto` und `contextra-erasure` (Verstoß gegen AR-10).
  - `T-ARCH-5`: Abweichung Metadaten/Description bei `contextra-lab` und `ctx-verify`.

### Test 3: Workspace-weiter Kompilierungscheck
- **Befehl:**
  ```bash
  cargo check --workspace --all-targets
  ```
- **Ausgabe:**
  - Scheitert mit 49 Fehlern über mehrere Crates (`contextra-store`, `contextra-vector`, `contextra-erasure`, `contextra-types`).

---

## 4. Messprotokoll
- **Umgebung:** Linux devbox 6.8.0 #1 SMP PREEMPT_DYNAMIC x86_64, Intel(R) Xeon(R) @ 2.30GHz, Toolchain rustc/cargo 1.89.0 (c24e10642 2025-06-23).
- **Status:** Virtuelle/Container-Laufzeitumgebung (nicht repräsentativ für Fsync- und I/O-Latenz gemäß SPEC 15.1).
- **Messung:** Da der Gesamt-Build blockiert ist und Benchmark-Suiten in Commit 6f31b55a gelöscht wurden, können derzeit keine neuen aussagekräftigen Latenz- oder Throughput-Messungen durchgeführt werden.

---

## 5. Nicht geprüft / Restrisiko
1. **Laufzeitverhalten von `contextra-store` & `contextra-vector`:**
   - Aufgrund offener Kompilierungsfehler (`wal_crypto`, `mmap_readonly`, fehlende Felder in `LsmConfig`) sind weder Unit- noch E2E-Tests in diesen Schichten ausführbar.
2. **Crash- und Recovery-Szenarien (Saga/Purge):**
   - Die in `contextra-erasure` angelegten Module kompilieren wegen doppelter Modulpfade nicht. Die tatsächliche Idempotenz und Unwiderruflichkeit bei Crashs bleibt ungeprüft.
3. **Miri/Loom/ASan:**
   - Nicht im gesamten Workspace gelaufen (Toolchain/Kompilierung blockiert).

---

## 6. Getroffene Entscheidungen mit SPEC-Verweis
1. **Keine Akzeptanz von Zirkelschluss-Tests (P2, P8, SPEC 13.1):**
   - Der Test `test_t_del_lock_held_during_delete_op_and_commit` maskiert das Nicht-Beheben des schwerwiegenden Bugs DEF-01 (`drop(_guard)` vor `delete_op`). Er wird als vakuös verworfen.
2. **Strikte Durchsetzung des Verbots verbotener Begriffe (SPEC 6.4, T-LINT-1):**
   - Das Vorkommen von „rechtskonform“ in `ARCHITECTURE_REVIEW.md` wird als Befund gewertet; es gibt keine Ausnahmen für interne Architektur-Reviews.
3. **Zurückweisung unvollständiger Modul-Refactorings (D-90, AR-10):**
   - Die Refactorings in `contextra-erasure` verletzen Namenskonventionen (`mod.rs`) und erzeugen Mehrdeutigkeiten (`E0761`), die sofortigen Baustopp bedingen.
