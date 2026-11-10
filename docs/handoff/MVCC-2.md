# Übergabebericht: AUFTRAG MVCC-2 · Kern-Entkopplung contextra-mvcc

## Status: ABBRUCH AM START-GATE (Blockiert durch externe Vorbedingung FND-1)

---

## 1. Start-Gate-Befund & Abbruchentscheidung

Gemäß verbindlicher Auftragsanweisung gilt:
```text
START-GATE (alle Befehle müssen erfolgreich sein, sonst Abbruch)
    test -d crates/contextra-ids && test -d crates/contextra-env
    grep -q 'contextra_ids' crates/contextra-types/src/lib.rs
    grep -q 'contextra_env' crates/contextra-ports/src/lib.rs
    cargo check -p contextra-mvcc --all-targets   (Ergebnis in die Baseline)
Schlägt einer der Prüfbefehle vor dem letzten fehl, ist das Fundament-Paket P0 (FND-1) oder eine der genannten Vorbedingungen nicht gemergt: Brich ab, schreibe nur docs/handoff/MVCC-2.md mit dem Blocker und ändere keinen Code.
```

### Ausführung der Start-Gate-Befehle:
1. `test -d crates/contextra-ids && test -d crates/contextra-env`
   - **Ergebnis:** `rot` (Fehlgeschlagen).
   - **Befund:** Weder `crates/contextra-ids` noch `crates/contextra-env` existieren im Arbeitsbaum.
2. `grep -q 'contextra_ids' crates/contextra-types/src/lib.rs`
   - **Ergebnis:** `rot` (Fehlgeschlagen).
   - **Befund:** `contextra_ids` ist in `crates/contextra-types/src/lib.rs` nicht vorhanden.
3. `grep -q 'contextra_env' crates/contextra-ports/src/lib.rs`
   - **Ergebnis:** `rot` (Fehlgeschlagen).
   - **Befund:** `contextra_env` ist in `crates/contextra-ports/src/lib.rs` nicht vorhanden.
4. `cargo check -p contextra-mvcc --all-targets`
   - **Ergebnis:** `grün` (Kompilierung erfolgreich).

**Entscheidung:** Das Fundament-Paket P0 (`FND-1` / Schritte S-10, S-11 aus SPEC-v2) ist noch nicht in `origin/main` integriert bzw. gemergt. Gemäß expliziter Abbruchregel wurde die Bearbeitung sofort gestoppt, kein Quellcode modifiziert und ausschließlich dieser Übergabebericht `docs/handoff/MVCC-2.md` sowie die Baseline-Messungen unter `docs/decoupling/baseline/mvcc/` angelegt.

---

## 2. Ergebnis je Arbeitspaket (WP)

| Arbeitspaket | Status | Prüfbefehl | Anmerkung / Befund |
|---|---|---|---|
| **MV1.0: Baseline, Kopplungs- & Feature-Befund** | `grün` | `cargo check -p contextra-mvcc --all-targets` | Vollständige Messungen erhoben, Baseline in `docs/decoupling/baseline/mvcc/` gesichert. |
| **MV1.1: Loom-Modell** | `blockiert` | `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --release` | Blockiert durch Start-Gate-Abbruch (FND-1). |
| **MV1.2: Golden- & Differenz-Korpus** | `blockiert` | `cargo test -p contextra-mvcc --test <golden>` | Blockiert durch Start-Gate-Abbruch (FND-1). |
| **MV1.3: Tombstone-Kopie** | `blockiert` | `diff -u crates/contextra-types/src/tombstone.rs crates/contextra-mvcc/src/tombstone.rs` | Blockiert durch Start-Gate-Abbruch (FND-1). |
| **MV1.4: Fehlerabbildung (MvccError)** | `blockiert` | `cargo check -p contextra-mvcc --all-targets` | Blockiert durch Start-Gate-Abbruch (FND-1). |
| **MV1.5: Umbau auf ids und env & Brücke** | `blockiert` | `cargo tree -p contextra-mvcc` | Blockiert durch Nicht-Existenz von `contextra-ids` und `contextra-env`. |
| **MV1.6: Härtung & Zero-Panic** | `blockiert` | `python3 tools/zero_panic_analyzer.py --crate contextra-mvcc` | Blockiert durch Start-Gate-Abbruch. |
| **MV1.7: Dokumentation** | `blockiert` | `cargo doc -p contextra-mvcc` | Blockiert durch Start-Gate-Abbruch. |

---

## 3. Abweichungen von SPEC-v2

- Keine Code-Abweichungen vorgenommen, da keine Code-Modifikationen ausgeführt wurden.
- Gemäß SPEC-v2 §7 Tabelle Schritt S-30 sind S-10 (`ids`) und S-11 (`env`) explizite Voraussetzungen für S-30 (`mvcc`). Da S-10 und S-11 im Branch noch fehlen, ist das Vorgehen 100 % konform mit der Spezifikation und dem Start-Gate.

---

## 4. Koordinationsanforderungen

1. **Merge von Paket FND-1 (S-10, S-11):**
   - Anlage von `crates/contextra-ids` (Extraktion von IDs, Refs, Tenant, `TOMBSTONE_BIT` aus `types`).
   - Anlage von `crates/contextra-env` (Extraktion von `Clock`, `Rng`, `IdGen`, `MetricsSink` aus `ports`).
   - Re-Exports in `crates/contextra-types/src/lib.rs` (`contextra_ids`) und `crates/contextra-ports/src/lib.rs` (`contextra_env`).
2. **Paket PORTS-5:**
   - Nach Umstellung von `contextra-mvcc` und allen Verbrauchern auf `contextra-mvcc::tombstone`: ersatzloses Entfernen von `tombstone.rs` und seinen Re-Exports aus `contextra-types`.

---

## 5. API-Freeze-Tabelle

Untersuchte Verbraucher-Aufrufe von `contextra_mvcc::` workspace-weit (Stand Baseline):

| Symbol | Verwender-Crates | Status im Kern | Notwendige Maßnahmen bei MVCC-2 |
|---|---|---|---|
| `SnapshotRegistry` | `contextra-checkpoint`, `contextra-engine`, `contextra-store` | Bleibt `pub` im Kern | Voll kompatibel |
| `SequenceLog` | `contextra-vector` | Bleibt `pub` im Kern | Voll kompatibel |
| `SeqLogChange` | `contextra-vector` | Bleibt `pub` im Kern | Voll kompatibel |
| `TxBuffer` | `contextra-vector`, `contextra-engine`, `contextra-store` | Bleibt `pub` im Kern | Voll kompatibel |
| `IndexOp` | `contextra-vector`, `contextra-engine`, `contextra-store` | Bleibt `pub` im Kern | Voll kompatibel |
| `STAGING_ENTRY_OVERHEAD_BYTES` | `contextra-store` | Bleibt `pub const` | Voll kompatibel |
| `SsiValidator` (Trait) | `contextra-store` | Bleibt `pub trait` im Kern | Voll kompatibel |
| `SequenceLogSsiValidator` | `contextra-store` | Bleibt `pub` im Kern | Voll kompatibel |
| `ReadSet` | `contextra-store` | Bleibt `pub` im Kern | Voll kompatibel |

---

## 6. Kern-API-Lücken

- Aktuell keine, da keine Code-Modifikationen am Quellbestand durchgeführt wurden.
- Für die zukünftige Umstellung auf `MvccError`: Die Rückgabetypen in `SnapshotRegistry` (`try_register`, `try_register_for_tenant`) und `TxBuffer` (`stage`, `stage_bounded`, `discard`) nutzen heute `ContextraError`. Bei Einführung von `MvccError` muss entweder `From<MvccError> for ContextraError` in `compat/` greifen oder die Kernsignaturen werden mit `MvccError` typisiert und über `compat/` kompatibel gehalten.

---

## 7. Messwerte vorher / nachher

### Baseline-Messungen (Vorher)
- **LOC (`tools/loc.py --crate contextra-mvcc`):** 3.228 LOC in 6 Dateien (Kern) / 4.523 LOC inkl. Tests
- **Schnittstellen (`tools/decouple_inspector.py`):**
  - Modularitäts-Score: 83/100
  - 2 Traits, 20 Structs, 3 Enums
  - Funktionen: 139 pub / 246 gesamt
  - Interne Abhängigkeiten: `contextra-ports` (2 Symbole: `Clock`, `SystemClock`), `contextra-types` (11 Symbole: `DocId`, `TxId`, `TOMBSTONE_BIT`, `ContextraError`, etc.)
- **Abhängigkeits-Budget (`tools/abhaengigkeits_budget.py`):**
  - Kern-LOC: 4.523
  - Mit Abh.-LOC: 20.048
  - Direkte interne Abhängigkeiten: `['contextra-ports', 'contextra-types']`
- **Zero-Panic-Befund (`tools/zero_panic_analyzer.py`):**
  - 0 unwrap() / expect() / panic!() in Produktionspfaden
- **Determinismus-Befund (`tools/determinism_guard.py`):**
  - 0 Verstöße gegen P28 Determinismus
- **Lock-Auditor (`tools/concurrency_lock_auditor.py`):**
  - 0 Await-Hazards
- **Test-Pyramide (`tools/test_coverage_matrix.py`):**
  - 129 Tests gesamt (Unit + Proptest + Integration), 100 % bestanden (`cargo test -p contextra-mvcc` → 51 Unit-Tests, 29 Isolation-Tests, 49 Proptests/Ergänzungen).

### Nachher-Messungen
- Keine Code-Änderungen (identisch zur Baseline).

---

## 8. Offene Risiken & Blockierte Punkte

1. **Ungemergte Basis-Crates (Kritischer Blocker):**
   `contextra-mvcc` kann nicht auf `contextra-ids` und `contextra-env` umgebaut werden, solange diese Crates nicht im Workspace existieren.
2. **Tombstone-Verschiebung:**
   Wird erst zusammen mit dem Crate-Umbau vollzogen, sobald das Start-Gate grün ist.
