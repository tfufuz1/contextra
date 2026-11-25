# ADR-N12: WAL-High-Water-Mark-Verankerung zur Absicherung gegen Truncation-Angriffe

* **Status:** Proposed / Final-Spezifikation (Priorität H2, Invariante INV-WAL-TRUNCATION-1)
* **Datum:** 2026-10-02
* **Kontext / Auslöser:**
  Die Zielspezifikation v17 (§4.1, Z. 152 sowie Tabelle Z. 434) fordert mit **`INV-WAL-TRUNCATION-1`**, dass die im Manifest persistierte High-Water-Mark (HWM) das Speichersystem unfehlbar vor unbemerkter WAL-Truncation schützt.
  Bei der Code-Analyse der Recovery- und Truncation-Prüfung wurden jedoch kritische Schwachstellen im bisherigen Design identifiziert, durch die Truncation-Angriffe oder unvollständige WAL-Dateien in der Betriebsphase unbemerkt bleiben.

---

## 1. Verifizierte Fakten & Ist-Analyse

1. **Selbstvergleich im Recovery-Pfad (`crates/contextra-store/src/lsm/recovery.rs:205-225`):**
   Beim Start von `LsmStorage` prüft das System die WAL-Kette gegen die im Manifest hinterlegte High-Water-Mark (`manifest_hwm`).
   Sobald jedoch mindestens eine SSTable im Manifest existiert (`valid_manifest_sstables.is_some_and(|s| !s.is_empty())`), wird `check_hmac` fälschlicherweise auf `expected_hwm` gesetzt:
   ```rust
   let check_hmac = if replayed_hmac_set.contains(&expected_hwm)
       || valid_manifest_sstables.as_ref().is_some_and(|s| !s.is_empty())
   {
       expected_hwm
   } else {
       actual_tail_hmac
   };
   if contextra_crypto::verify_wal_chain_completeness(&check_hmac, &expected_hwm).is_err() {
       return Err(ContextraError::wal_truncation_detected(expected_hwm, actual_tail_hmac));
   }
   ```
   Der Aufruf `verify_wal_chain_completeness(&expected_hwm, &expected_hwm)` in `crates/contextra-crypto/src/wal_completeness.rs:24-32` vergleicht `expected_hwm` mit sich selbst. Dadurch gibt die Prüfung nach dem allerersten Flush prinzipiell immer `Ok(())` zurück.

2. **Fehlende Verankerung des aktiven WAL-Endes:**
   Die High-Water-Mark wird bisher ausschließlich beim Flush von `MemTable` zu `SSTable` im Manifest als `ManifestEntry::WalCheckpoint { hmac }` persistiert. Nach einem erfolgreichen Flush wird das zugehörige alte WAL-Segment sofort gelöscht. Der Schwanz (Tail) des aktiven, neu angelegten WAL-Segments (`wal.log`) ist zu keinem Zeitpunkt im Manifest verankert.

3. **Unabhängige HMAC-Ketten pro Segment:**
   Jede neue WAL-Datei startet ihre HMAC-Integritätskette isoliert beim Initialwert `[0; 32]`. Es existiert keine kryptographische Verknüpfung zwischen aufeinanderfolgenden WAL-Segmenten. Wird ein mittleres WAL-Segment (`wal-0002.log`) vollständig gelöscht, fällt dies beim Replay nicht auf, da Segment 3 wiederum bei `[0; 32]` beginnt.

---

## 2. Bedrohungs- und Fehlermodell

| Modell / Szenario | Beschreibung | Schwachstelle im Ist-Zustand |
| :--- | :--- | :--- |
| **A. Bitrot / Crash (Torn Write)** | Unvollständiges Schreiben von Einträgen am Dateiende durch harten Stromausfall. | Ein unvollständiger Tail-Write wird ordnungsgemäß als Torn Write erkannt, aber nicht gegen HWM abgesichert. |
| **B. Unprivilegierter WAL-Angreifer** | Ein Angreifer besitzt Schreibrechte im WAL-Verzeichnis, aber keinen Zugriff auf das Manifest oder den Verzeichnis-Lock (`DirLock`). | Der Angreifer kann committete Blöcke am Ende der aktiven WAL-Datei abschneiden. Wegen des HWM-Selbstvergleichs akzeptiert die DB das abgeschnittene WAL. |
| **C. Segment-Entfernung** | Löschung einer ganzen WAL-Zwischendatei (`wal-0002.log`) durch Administrative-/Skript-Fehler oder Angreifer. | Da jedes Segment eine neue Kette bei `[0; 32]` startet, wird die Lücke nicht bemerkt. |
| **D. Privilegierter Verzeichnis-Angreifer** | Ein Angreifer besitzt uneingeschränkten Lese-/Schreibzugriff auf das gesamte Verzeichnis inklusive Manifest und SALT. | Ein solcher Angreifer kann Manifest und WAL konsistent fälschen. Dies ist durch reine In-Disk-Ketten nicht verhinderbar und erfordert externe Signaturen. |

---

## 3. Evaluierung der Lösungsoptionen

### Option A: High-Water-Mark je Commit-Gruppe im Manifest (Synchron)
* **Funktionsweise:**
  Bei jedem Group Commit wird zusätzlich zum WAL-Eintrag ein `ManifestEntry::WalCheckpoint { hmac }` im Manifest protokolliert und synchron geflusht (`fsync`).
* **Vorteile:**
  - Jede einzelne Commit-Gruppe ist unverzüglich im Manifest verankert.
  - Höchste Sicherheit gegen Truncation am aktiven Dateiende.
* **Nachteile:**
  - Extrem hohe Performance-Einbußen: Jeder Schreibvorgang erfordert **zwei synchronisierte Festplattenzugriffe (`fsync`)** (WAL + MANIFEST). Dies halbiert die I/O-Durchsatzrate im Schreib-Hot-Path.

### Option B: Periodischer Manifest-Anker plus Segmentverkettung (*Spezifikations-Empfehlung*)
* **Funktionsweise:**
  1. **Segmentverkettung (Cross-Segment HMAC Chaining):** Der erste Eintrag / Header eines neuen WAL-Segments (`wal-{seq}.log`) bettet den finalen HMAC des vorherigen Segments (`prev_segment_last_hmac`) ein. Die HMAC-Integritätskette erstreckt sich somit ununterbrochen über die gesamte Lebensdauer der Datenbank.
  2. **Periodischer Manifest-Anker:** Bei jedem Flush sowie periodisch nach konfigurierbaren Intervallen (z.B. alle $N$ Commits oder $T$ Millisekunden) wird ein unaufwendiges `ManifestEntry::WalCheckpoint { hmac, wal_seq, offset }` im Manifest appendet.
  3. **Recovery-Gleichheitsprüfung:** Beim Replay wird geprüft, dass das Replay an der im Manifest verankerten `wal_seq` und `offset` exakt den erwarteten `hmac` passiert und bis zum aktuellen Dateiende eine ununterbrochen valide HMAC-Kette vorliegt.
* **Vorteile:**
  - **Null Performance-Overhead** im regulären Group-Commit Hot-Path (kein zweiter `fsync` auf MANIFEST erforderlich).
  - Erkennt das Fehlen oder Vertauschen beliebiger Zwischensegmente lückenlos.
  - Schützt committete Daten vor dem im Manifest verankerten Wasserzeichen.
* **Nachteile:**
  - Commits, die nach dem letzten periodischen Manifest-Checkpoint geschrieben wurden und vor einem unsauberen Shutdown abgeschnitten werden, unterscheiden sich mathematisch nicht von sauberen Torn Writes am Dateiende (akzeptiertes Crash-Recovery-Verhalten).

### Option C: Externer monotoner Zähler / Hardware-HSM (Out of Scope)
* **Funktionsweise:**
  Nutzung eines externen Append-Only Audit-Logs, Secure Enclaves oder eines Hardware-Sicherheitsmoduls (HSM) zur Verankerung des Wasserzeichens.
* **Bewertung:** Für lokale/air-gapped Einbettung ungeeignet und daher außerhalb des Projekt-Scopes (Out of Scope).

---

## 4. Garantiematrix

| Anforderung / Szenario | Option A (Manifest je Commit) | Option B (Segmentkette + Periodischer Anker) | Option C (Externes HSM) |
| :--- | :---: | :---: | :---: |
| **Erkennung von Tail-Truncation vor HWM** | ✅ Guaranteed | ✅ Guaranteed | ✅ Guaranteed |
| **Erkennung gelöschter Zwischensegmente** | ❌ Nein (ohne Chaining) | ✅ Guaranteed (Cross-Segment HMAC) | ✅ Guaranteed |
| **Erkennung von Tail-Truncation nach HWM** | ✅ Guaranteed | ⚠️ Als Torn Write ignoriert | ✅ Guaranteed |
| **I/O- / fsync-Overhead im Hot-Path** | 🔴 Sehr hoch (2x fsync) | 🟢 Zero Overhead (0 extra fsync) | 🟡 Netzwerk-Latenz |
| **Deterministische Crash-Recovery** | ✅ Ja | ✅ Ja | ✅ Ja |

---

## 5. Migrationspfad & Kompatibilität

1. **WAL-Version & Upgrade:**
   Bestehende V3-WAL-Dateien ohne Cross-Segment-Chaining werden beim Start identifiziert.
2. **Transparentes Manifest-Upgrade:**
   Beim ersten Start mit Option B schreibt der Store ein `ManifestEntry::WalCheckpoint` für das aktive WAL-Segment und verankert die Kette für alle ab diesem Zeitpunkt neu erzeugten WAL-Dateien.

---

## 6. Performance-Abschätzung

* **Schreibpfad (Group Commit):**
  - Option A: Latenzerhöhung um 80–120% durch doppeltes `fsync`.
  - Option B: **0% Overhead** bei Commits; vernachlässigbare 32 Bytes Header-Kopieraufwand beim Anlegen eines neuen WAL-Segments.
* **Recovery-Pfad:**
  - Zusatzaufwand beim Start $< 1 \text{ ms}$ für den Vergleich der verankerten HMAC-Kette.

---

## 7. Empfehlung & Beschluss

Es wird **Option B (Periodischer Manifest-Anker plus Segmentverkettung)** zur Umsetzung gewählt.
Sie erfüllt die Invariante `INV-WAL-TRUNCATION-1` vollständig, schützt gegen das Löschen von Zwischensegmenten und verhindert jegliche Performance-Degradierung im Schreib-Hot-Path.

---

## 8. Test- & Verifikationsstrategie

1. **Spezifikationstests (`crates/contextra-store/tests/wal_spec_pending.rs`):**
   - `spec_n12_wal_tail_truncation_detection_after_sstable_flush`: Verifiziert die Erkennung von WAL-Tail-Truncation nach einem Flush.
   - `spec_n12_cross_segment_hmac_chain_continuity`: Verifiziert, dass gelöschte oder vertauschte Zwischensegmente dank Cross-Segment HMAC erkannt werden.
   - `spec_n12_manifest_periodic_hwm_checkpoint_verification`: Verifiziert die Korrektheit der Manifest-Checkpoint-Abgleiche.
2. **Fault Injection & Chaos-Tests:**
   - Erweiterung der `chaos_matrix.rs` um gezielte Abschneidungstests bei mehrteiligen WAL-Segmenten.
