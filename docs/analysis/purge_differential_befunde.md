# Purge Differential Testing Befunde & Analyse (T-2026-0302)

**Datum**: 2026-10-15
**Rolle**: S14 Spezialist für Teststrategie und deterministisches Testen
**Spezifikation**: v4.1, Abschnitt 9.3, Kriterium S5
**Test-Suite**: `crates/contextra-store/tests/differential/purge_correctness.rs`

---

## 1. Übersicht der Invarianten-Testergebnisse

| Invariante | Status | Beschreibung |
|------------|--------|--------------|
| **I1** (`Invariante_I1_outside_scope_snap`) | ✅ GRÜN | Schlüssel außerhalb des Purge-Scopes liefern für Schnappschüsse `seq >= as_of_tx` exakt die Modellwerte. Zwingender Mandanten-Isolierungstest `t:4:` vs. `t:42:` erfolgreich. |
| **I2** (`Invariante_I2_version_ordering_and_l0_overlap`) | ❌ ROT (BEFUND-D01) | Versionsreihenfolge und neuere Versionen (`seq > as_of_tx`) wurden bei `purge_keys` fälschlicherweise ebenfalls physikalisch entfernt. |
| **I3** (`Invariante_I3_raw_sweep_purged_in_scope`) | ✅ GRÜN | Nach dem Purge sind im Scope alle Canary-Bytes in SSTables und `*.purge_tmp` im Rohsweep vollständig bereinigt. Positivkontrolle vor Purge erfolgreich nachgewiesen. |
| **I4** (`Invariante_I4_fault_injection_error_and_byte_identical_files`) | ✅ GRÜN | Injizierter Lesefehler führt zu `Err(PurgeError::ReadError)` und belässt alle nicht-korrupten SSTable-Dateien byteidentisch. |
| **I5** (`Invariante_I5_idempotent_double_purge`) | ✅ GRÜN | Zweifacher Purge desselben Scopes führt zu identischem Zustand (`records_purged == 0` beim 2. Lauf). |
| **Proptest** (`purge_correctness_proptest`) | ❌ ROT (BEFUND-D01) | Property-based Differential Test zeigt Abweichung bei Operationen mit neueren Versionen nach `as_of_tx`. |

---

## 2. Befunde (Out-of-Scope Findings & Invarianten-Verletzungen)

### BEFUND-D01: Missing `as_of_tx` Sequence Check in `LsmStorage::purge_keys`
- **Invariante**: I2 (`Invariante_I2_version_ordering_and_l0_overlap`) und `purge_correctness_proptest`.
- **Lokalisierung im Produktionscode**: `crates/contextra-store/src/lsm/ops/purge.rs`, Zeile 105–120.
- **Root Cause**:
  In `LsmStorage::purge_keys` filtert die Schleife über SSTable-Einträge ausschließlich mit `if scope.matches_key(&key)`. Es fehlt der Vergleich `seq <= as_of_tx`. Wenn eine SSTable neu geschrieben wird, werden **alle** Eintragsversionen des Scopes verworfen — auch neuere Versionen mit `seq > as_of_tx`, die laut Spezifikation §9.3 erhalten bleiben müssen.
- **Deterministischer Seed**: `RngSeed::from([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32])`
- **Minimale Reproduktionsfolge**:
  1. `storage.put(TxId(10), b"key_a", b"val_a_v1")` -> `flush()` (SSTable 1, seq 10)
  2. `storage.put(TxId(30), b"key_a", b"val_a_v2")` -> `flush()` (SSTable 2, seq 30)
  3. `storage.purge_keys(PurgeScope::Keys(vec![b"key_a".to_vec()]), as_of_tx = 15, None)`
- **Erwartetes Ergebnis**: Version bei `seq 30` (`> 15`) MUSS erhalten bleiben. `get_at_seq("key_a", 35)` liefert `Some("val_a_v2")`.
- **Tatsächliches Ergebnis**: `get_at_seq("key_a", 35)` liefert `None`, da `purge_keys` SSTable 2 neu schreibt und Version `seq 30` ebenfalls löscht.
- **Folgekarte / Zuordnung**: S1 (P9 und Folgekarten).

---

### BEFUND-D02: Broken existing test `purge_keys_selection.rs` in `contextra-store`
- **Lokalisierung**: `crates/contextra-store/tests/purge_keys_selection.rs`
- **Root Cause**:
  Aufgrund der Refactoring-Änderung in Commit `d45ea7c` (`fix(store): map purge errors to typed PurgeError variants instead of CompactionBusy`) nutzt `purge_keys_selection.rs` noch den alten Typ `PurgeBlocked` statt `PurgeError`, weshalb `purge_keys_selection.rs` nicht kompiliert (`expected PurgeError, found PurgeBlocked`).
- **Hinweis**: Fällt außerhalb des Scopes der Karte T-2026-0302 und wird als Out-of-scope Finding gemeldet.

---

## 3. Erwarteter Befund: WAL-Nutzdaten bis zur WAL-Barriere

Wie in der Aufgabenstellung vorgegeben, sind WAL-Segmente in dieser Karte vom Rohsweep ausgeschlossen. Der WAL trägt Nutzdaten bis zur WAL-Barriere (Spezifikation 5.2 Schritt 8, CRASH-15). Die Bereinigung der WAL-Segmente erfolgt in der dedizierten Folgekarte.
