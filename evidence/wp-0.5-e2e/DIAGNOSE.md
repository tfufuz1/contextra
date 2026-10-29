# Diagnosebericht: Analyse der vier roten Löschketten-E2E-Tests (WP 0.5 / G2)

**Datum:** 2026-10-29
**Crate:** `contextra` (`crates/contextra/tests/deletion_chain_e2e.rs`)
**Ziel:** Gezielte Diagnose und Ursachenklassifikation der 4 fehlschlagenden E2E-Löschtests zur Wiederherstellung der Garantie G2.

---

## 1. Übersicht der Testergebnisse und Panic-Texte

| Szenario | Roh-Panic-Text | Ursachenklasse | Verweis (Crate, Datei:Zeile) |
|---|---|---|---|
| `test_scenario_2_voller_pfad` | `Löschbericht muss verified == true sein: Some(ErasureFailure { layer: "LsmMemtable", reason: "Layer has 11 residual live entries" })` | Produktfehler | `contextra-store`: `crates/contextra-store/src/scanner/memtable.rs:69`, `crates/contextra-store/src/scanner/wal.rs:65` |
| `test_scenario_3_rohdaten_needle` | `Löschung muss verifiziert werden: Some(ErasureFailure { layer: "LogicalStorage", reason: "Logical delete failed: Transaction error: Storage commit failed: Conflict: Serializable isolation violation for TxId(11): snapshot too old (snapshot_seq 15 < pruned_through 9223372036854775807) for key '\"t:1:__col:testcol_needle:\0\0doc_needle_id_333\"'" })` | Produktfehler | `contextra-engine`/`contextra-db`: `crates/contextra-engine/src/transaction.rs:180`, `crates/contextra-db/src/collection.rs:120` |
| `test_scenario_4_mehrere_level_und_neustart` | `Löschen nach Reopen muss verified sein: Some(ErasureFailure { layer: "HnswIndex", reason: "Layer has 11 residual live entries" })` | Produktfehler | `contextra-vector`: `crates/contextra-vector/src/hnsw/scanner.rs:104`, `crates/contextra-vector/src/hnsw/purge.rs:55` |
| `test_scenario_5_collection_loeschung` | `erase_collection muss verified == true sein: Some(ErasureFailure { layer: "LsmMemtable", reason: "Layer has 11 residual live entries" })` | Produktfehler | `contextra-store`: `crates/contextra-store/src/scanner/memtable.rs:69`, `crates/contextra-store/src/scanner/wal.rs:65` |

---

## 2. Detaillierte Befunde je Szenario

### 2.1 Szenario 2: `test_scenario_2_voller_pfad`
- **Schichten-Zusammenfassung:**
  - `TextIndex`: scanned = 1, remaining = 0, control_matches = 1
  - `HnswIndex`: scanned = 1, remaining = 0, control_matches = 1
  - `LsmMemtable`: scanned = 12, remaining = 11, control_matches = 1
  - `SsTableAllLevels`: scanned = 1, remaining = 0, control_matches = 1
  - `WalAllSegments`: scanned = 12, remaining = 11, control_matches = 1
- **Fehleranalyse (Produktfehler):**
  Beim Einfügen des Dokuments schreibt die Collection/Storage-Schicht nicht nur den Dokument-Eintrag, sondern auch System- und Metadaten-Schlüssel (z. B. `__col_idx:\x00testcol_full`). In Stage D rufen `MemtableScanner` und `WalScanner` `codec.scan_prefix()` auf, welches das Tenant-Präfix `b"t:1:"` liefert.
  Statt zu prüfen, ob der Schlüssel zum spezifischen Dokument-Target (`ScanTarget`) passt, markieren `MemtableScanner` und `WalScanner` JEDEN Schlüssel für den Tenant als `is_residual`. Dadurch werden alle verbliebenen 11 Tenant-Systemschlüssel in Memtable und WAL als verbliebene Eintragsreste des zu löschenden Dokuments gezählt.

### 2.2 Szenario 3: `test_scenario_3_rohdaten_needle`
- **Schichten-Zusammenfassung:**
  - `layers`: `[]` (Abbruch in Stage B: Logisches Löschen)
- **Fehleranalyse (Produktfehler):**
  Vor dem Aufruf von `erase_document` führt der Test `db.flush().await` aus.
  Der Flush-Vorgang flusht die Memtable und setzt im MVCC/SSI-Transaktionsmanager die globale `pruned_through`-Sequenznummer hoch (auf `u64::MAX`).
  Wenn `erase_document` anschließend `col.delete(doc_id)` aufruft, versucht die Transaktionslogik, den Tombstone mit einer Snapshots-Sequenznummer vor dem Flush zu committen. `validate_ssi_gate` erkennt einen Konflikt (`snapshot_seq 15 < pruned_through 9223372036854775807`) und bricht die Löschung in Stage B mit einem `Serializable isolation violation`-Fehler ab.

### 2.3 Szenario 4: `test_scenario_4_mehrere_level_und_neustart`
- **Schichten-Zusammenfassung:**
  - `TextIndex`: scanned = 1, remaining = 0, control_matches = 1
  - `HnswIndex`: scanned = 22, remaining = 11, control_matches = 1
  - `LsmMemtable`: scanned = 12, remaining = 11, control_matches = 1
  - `SsTableAllLevels`: scanned = 1, remaining = 0, control_matches = 1
  - `WalAllSegments`: scanned = 12, remaining = 11, control_matches = 1
- **Fehleranalyse (Produktfehler):**
  In diesem Szenario werden 1 Target-Dokument und 10 weitere Dokumente (insgesamt 11 Dokumente) eingefügt.
  Beim Aufruf von `erase_document` nach dem Reopen markiert `HnswPurger` das Ziel-Dokument in der HNSW-Struktur lediglich mit einem Tombstone. Es erfolgt jedoch weder eine Nullung der Vektor-Bytes im RAM noch ein partieller/vollständiger Rebuild.
  `HnswScanner` prüft in `scan_index_content` alle Knoten mit `is_tombstoned && has_non_zero_vector`. Da die Vektorbytes der als gelöscht markierten Knoten im Speicher verbleiben, meldet `HnswScanner` 11 verbleibende Vektoreinträge. Parallel dazu treten die gleichen Über-Scans in `MemtableScanner` und `WalScanner` auf.

### 2.4 Szenario 5: `test_scenario_5_collection_loeschung`
- **Schichten-Zusammenfassung:**
  - `LsmMemtable`: scanned = 12, remaining = 11, control_matches = 1
  - `WalAllSegments`: scanned = 12, remaining = 11, control_matches = 1
- **Fehleranalyse (Produktfehler):**
  `erase_collection` entfernt die Präfixe `__col:drop_collection_test:` und `__txt:drop_collection_test:`.
  Anschließend scannen `MemtableScanner` und `WalScanner` pauschal das Tenant-Präfix `t:1:`. Da dort 11 systemweite Tenant-Schlüssel verbleiben, melden beide Scanner fälschlicherweise Restdaten für die gelöschte Collection.

---

## 3. Bewertung der Hypothesen H1 bis H5

- **[BELEGT] H1: Ein Scanner oder Purger fehlt oder meldet Rest (PLAN U-03, VectorScanner/HnswScanner teilweise):**
  Belegt durch `MemtableScanner`, `WalScanner` (über-selektiver Scan aller Tenant-Schlüssel) und `HnswScanner`/`HnswPurger` (fehlende Vektor-Nullung / Rebuild bei HNSW-Tombstones).
- **[BELEGT] H2: Reihenfolge Purger (Text, Vector, LSM) gegen Flush und Scans:**
  Belegt in Szenario 3: `db.flush()` setzt `pruned_through` im SSI-Transaktionsmanager hoch, wodurch spätere `col.delete`-Transaktionen mit `snapshot_seq < pruned_through` schlagen.
- **[WIDERLEGT] H3: ProofKeyProvider oder Schlüsselauflösung:**
  Widerlegt. `TestProofKeyProvider` löst den Schlüssel in allen Tests fehlerfrei auf.
- **[WIDERLEGT] H4: Annahme im Test falsch (Tenant, Collection-Name):**
  Widerlegt. Die Mandanten-ID (`TenantId(1)`) und Collection-Namen im Test sind korrekt konfiguriert.
- **[WIDERLEGT] H5: Neustart-Szenario (WAL, Lock, Tempdir):**
  Widerlegt. Der Reopen-Prozess verläuft technisch sauber; das Scheitern beruht auf den genannten Scanner-/Purger-Produktfehlern.

---

## 4. Empfohlene Folgeaufgaben (Out-of-Scope für diesen PR)

Die folgenden Produktfehler müssen in separaten WPs behoben werden:
1. **`contextra-store` Scanner-Targeting:** `MemtableScanner` und `WalScanner` müssen `ScanTarget` exakt auswerten und dürfen nicht pauschal alle Keys des Tenants (`codec.scan_prefix()`) als `is_residual` flaggen, wenn nach einem spezifischen Dokument oder einer Collection gesucht wird.
2. **`contextra-vector` HNSW Purge & Rebuild:** `HnswPurger` muss gemaß VETO-F02 Vektor-Slots Nullen und nach Tombstone-Pruning einen Rebuild durchführen, damit `HnswScanner` keine Vektorbytes im Speichermedium mehr vorfindet.
3. **`contextra-engine` / `contextra-db` SSI-Snapshot nach Flush:** `db.flush()` darf die Snapshot-Historie für laufende/neue Transaktionen in `col.delete` nicht so abschneiden, dass `Serializable` Transaktionen verwerfen.
