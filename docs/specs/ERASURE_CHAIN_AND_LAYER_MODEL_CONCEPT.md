# Konzept- und Testentwurf: Technische Löschprozesse, Schichtmodell und Rechenschaft

Normative Referenz: `docs/SPEC.md` (§6.4, §8.2, §11, §12, §13, Anhang D, Anhang F).
Rolle: Leitender Ingenieur für technische Löschprozesse mit Datenschutzrecht-Verständnis (Art. 5 Abs. 2, 17, 25, 32 DSGVO; keine Rechtsberatung).

---

## 1. Einleitung und normative Leitlinien

Contextra ist eine eingebettete Rust-Bibliothek (Single-Node, kein Server) für lokales Wissen und Gedächtnis von Agenten- und RAG-Systemen. Das Löschen personenbezogener Daten ist ein gemessener, unabhängig prüfbarer technischer Prozess; die gesamte Architektur ist um Löschbarkeit gebaut.

Rechtsanforderungen aus der DSGVO werden in **prüfbare technische Zustände** übersetzt, niemals in bloße Versprechen oder rechtliche Wertungen:
- **Art. 17 DSGVO (Recht auf Löschung / Vergessenwerden):** Sofortige Lesesperre (ReadGate), kryptografische Schlüsselvernichtung ($O(1)$ bezüglich Zielgröße) und deterministischer physischer Purge in allen betroffenen Speicherschichten.
- **Art. 5 Abs. 2 DSGVO (Rechenschaftspflicht / Accountability):** Signierter, kryptografisch fälschungssicherer Löschbericht mit getrennter Feldmessung (`field_measurement`) und Lab-Evidenz (`lab_evidence`), lückenloser Hashkette im Ledger.
- **Art. 25 DSGVO (Datenschutz durch Technikgestaltung / Privacy by Design):** Keine Klartext-Identifikatoren in Schlüsseln, Journalen oder Ledgern; fail-closed Design.
- **Art. 32 DSGVO (Sicherheit der Verarbeitung):** Block-AEAD-Verschlüsselung, HMAC-Integritätsketten, physische Zerstörung von Schlüsselmaterial (Zeroizing).

### Sprachliche Schutzvorschrift (SPEC §6.4)
Der Wortlaut im Code, in Berichten und in Dokumentationen unterliegt strengen Restriktionen:
- **Verboten:** Unzulässige Außenaussagen, Heilsversprechen und nicht belegbare Aussagen bezüglich Backups, Flash-Remanenz, FS-Journalen, Swap oder externen Verarbeitern gemäß der Negativliste in SPEC §6.4.
- **Zulässiger normativer Wortlaut (SPEC §1.5):**
  > „Contextra macht Löschen zu einem messbaren, unabhängig prüfbaren Vorgang: Wirkung sofort, Entfernung physisch, Nachweis signiert, Restrisiken benannt.“

---

## 2. Die Löschprozesskette je Löschziel

Jeder Löschauftrag durchläuft eine deterministische Kette:
$$\text{Auftrag} \longrightarrow \text{Gate} \longrightarrow \text{Schlüsselvernichtung } (O(1)) \longrightarrow \text{Purge je Schicht} \longrightarrow \text{Messung (MK1--3)} \longrightarrow \text{signierter Bericht}$$

### 2.1 Schritt 1: Auftrag (`EraseRequest`)
Ein Löschauftrag spezifiziert:
- `target`: `EraseTarget` (`Document`, `Subject`, `Collection`, `Tenant`, `DocumentReplace`, `Superseded`).
- `timing`: `Timing::Now` (sofort) oder `Timing::Batched` (im nächsten Batch-Fenster).
- `retention`: `RetentionAttestation` mit Pflicht-Code (`RetentionCode`), opaker Operator-Referenz und Zeitstempel. Fehlt die Attestierung oder steht eine Aufbewahrungspflicht entgegen (`ObligationActive`), wird der Auftrag vor dem Gate mit `Refused` terminal abgelehnt (SPEC §11.2, I-8).
- `mode`: `EraseMode::Erase` (vollständige Löschung) oder `EraseMode::Restrict` (Einschränkung nach Art. 18 DSGVO).

### 2.2 Schritt 2: Gate (`ReadGate`, `ErasedSet`, `gates.log`)
Sofortwirkung in $O(1)$ bezüglich der Dokumentanzahl des Ziels (SPEC §1.5 M1, P16, D-57):
1. **Commit-Barriere:** Exklusive Commit-Barriere anfordern (`barrier.exclusive_commit()`).
2. **Umfangsauflösung:** Vor dem Gate ruft jede Schicht `resolve(&target)` auf; das Ergebnis wird als `LayerScope` fixiert und persistiert (`.scp` oder im Journal-Intent; I-8).
3. **Gate-Eintrag:** Eintragung des Zielbereichs in das synchron arbeitende Speicher-Intervall (`ErasedSet`) mit aktueller Sequenznummer `gated_seq`.
4. **Persistenz:** Append im `gates.log` mit Block-AEAD und Hashkette + `fsync` (fsync #2; D-58).
5. **Wirkung:** Ab `gated_at` liefert jeder Lesepfad (`get`, `scan`, Text-Suche, Vektor-Suche) für Ziel-Schlüssel sofort `GateDecision::Erased` bzw. `ContextraError::Erased` (I-1, I-2, I-21).

### 2.3 Schritt 3: Schlüsselvernichtung ($O(1)$ in der Datenmenge)
Kryptografisches Löschen (Cryptographic Erase nach NIST SP 800-88 Rev. 2, D-01, D-79):
1. Ermittlung des Slots der höchsten Ebene im `FileVault`:
   - `Document`: $D$-Slot
   - `Subject`: $S$-Slot
   - `Collection`: $C$-Slot
   - `Tenant`: $T$-Slot
2. Überschreiben des 64-Byte-Slots mit Nullen (`Slot::tombstone()`, I-23) an ausgerichteter 64-Byte-Grenze.
3. Erzwingen der Dauerhaftigkeit via `sync_data` auf dem Shard (fsync #3; D-58).
4. **Verifikations-Leseprobe:** Lesendes Wiederöffnen des Slots; schlägt die Ableitung nicht fehl, tritt sofort ein Abbruch/Fehler ein (`T-VAULT-1`).
5. **DEK-Cache-Eviction:** Flüchtigen DEK-Cache für den gelöschten Bereich leeren und nullen (`RecordSealer::evict`).
6. Journalisierung des Zustands `KeyDestroyed` mit `fsync` (fsync #4).
7. Rückmeldung an den Aufrufer: `EraseReceipt` mit Status `Gated` (Rückgabe erfolgt garantiert nach maximal 4 fsyncs, D-30, D-58).

### 2.4 Schritt 4: Purge je Schicht
Physische Entfernung im Purge-Fenster (zielgrößenabhängig, abbrechbar und crash-resistent; SPEC §11.2 Schritt 5):
1. **Widerruf aktiver Snapshots:** Alle aktiven MVCC-Snapshots des betroffenen Mandanten mit `read_seq <= gated_seq` werden invalidiert (`SnapshotRegistry::revoke_covering`, D-61, D-127).
2. **Textsegment-Rewrite (erstes Fenster, D-69, D-121):**
   - Segmente ermitteln, deren Mitgliedschaftsblock (`m(doc_tag)` oder `m(cluster_tag)`) Treffer meldet.
   - Segment-Rewrite: Alle Chunks übernehmen, deren Bindung noch entschlüsselbar ist (Löschprädikat I-22).
   - Manifest-Eintrag `TextCommit { add, remove, floor }` atomar austauschen.
   - $F$-Slots der Altsegmente nullen, Altdateien per `unlink` entfernen.
3. **LSM-Store & WAL-Floor (D-73, D-127):**
   - Kurze Commit-Barriere: Aktive Memtable einfrieren, WAL rotieren.
   - Ziel-Einträge aus eingefrorener Memtable verwerfen, flushen.
   - WAL-Segmente bis zum Flush-Punkt entfernen (`drop_wal_until(WalFloor)`).
   - SSTables umschreiben: Betroffene SSTables filtern (Nicht-Ziel-Records bytegleich übernehmen, neuer Dateischlüssel); nach dauerhaftem Manifest-Swap $F$-Slots der Altdateien nullen und `unlink` ausführen (I-3, I-17, D-85).
4. **Vektorindex (D-77, D-122):**
   - Surrogat-Ersetzung für kleine Mengen vor Gate ($\le 4096$ Slots); größere Mengen zu Beginn des Purge-Fensters.
   - Kantenreparatur im HNSW-Graph, Freiliste aktualisieren, Rebuild nach Rebuild-Quote.
5. **Vault-Cleanup:** Restliche $D$-, $C$- und $S$-Slots der Zielmenge entfernen, verwaiste $F$-Slots bereinigen (D-83).
6. **Cache-Invalidierung:** Block-Cache, DEK-Cache und OS-Puffer verwerfen.

### 2.5 Schritt 5: Messung (MK1–3) mit Positivkontrolle
Vor Ausstellung des Berichts führt die Lösch-Engine über `Scanner<Armed>` die Feldmessung durch (SPEC §12.1, §12.2, D-72, I-19):
- **Positivkontrolle (HR-10):** Ein Scanner darf erst dann den Typzustand `Armed` annehmen (`Scanner<Unarmed>::arm(ControlEvidence)`), wenn er in einem synthetischen Kontrollpuffer ein injiziertes Testmuster nachweislich detektiert und bei Mutation ablehnt.
- **MK1 (Strukturprüfung):** Vollständiger Scan der entschlüsselten Indexstrukturen, Manifeste und Wörterbücher auf verbliebene Verweise des Ziels.
- **MK2 (Rohscan):** Rohscan über alle Dateien des Store-Verzeichnisses (inkl. `tmp/`, WAL, Snapshots) auf Klartext-Fragmente und Tag-Muster außerhalb verschlüsselter Blöcke.
- **MK3 (Schlüsselprobe):** Verifikation im Feld, dass der Vault-Slot genullt ist und jeder Entschlüsselungsversuch fehlschlägt (`Erased`).
- **Residuen-Verbot:** Findet die Messung auch nur ein einziges Residuum (`matches != 0`), darf kein `LayerWitness` ausgestellt werden (`Residue`-Fehler). Der Saga-Zustand wechselt nach `Pending(WitnessMissing)` und darf niemals `Completed` erreichen (I-4, I-19).

### 2.6 Schritt 6: Signierter Bericht (`ErasureReport`)
Abschluss der Saga:
1. Zusammenstellung des Berichts mit kanonischem JSON (RFC 8785).
2. Signierung mit instanzgebundenem Ed25519-Schlüssel ($K_{sign}$, abgeleitet aus $K_{master}$).
3. Append-Eintrag in das externe Ledger (`LedgerSink::append`) mit Commitment-Hash und Range-Set (I-7, D-74).
4. Journal-Eintrag `Completed` + Gate-Kompaktion.

---

## 3. Schichtmodell: Status, Entscheidungsregel und Restrisiken

### 3.1 Das Schichtmodell im Löschbericht
Jede Schicht wird im Bericht explizit ausgewiesen:
1. `entfernt` (`Purged`): Die Daten wurden physisch durch Umschreiben/Truncation entfernt.
2. `gemessen` (`Witnessed`): Ein gültiger `LayerWitness` (aus `Scanner<Armed>` mit `matches == 0`) liegt für die Schicht vor.
3. `nicht erfasst` (`Uncovered` / `NotCovered`): Die Schicht ist im System nicht vorhanden, wurde nicht angesprochen oder lieferte keinen Messnachweis.

### 3.2 Die normative ENTSCHEIDUNGSREGEL
> **Fehlt eine Schicht im Code oder liefert sie keinen gültigen Zeugen, wird sie als „nicht erfasst“ ausgewiesen, niemals implizit mitgezählt.**
> **Vektor- und Textindex gelten erst nach gemessenem Nachweis als erfasst.**

Ist ein Index-Subsystem deaktiviert oder noch nicht integriert, führt der Bericht dieses Subsystem zwingend als `not_covered` mit `KeyCoverage::None` auf.

### 3.3 Restrisiko-Katalog (SPEC Anhang D)
Da Contextra auf Standard-Betriebssystemen und Standard-Datenträgern läuft, können physische Effekte unterhalb der Dateisystemgrenze nicht durch Software behoben werden. Der Bericht benennt diese Lücken zwingend als offene Restrisiken:

| Risiko-ID | Bezeichnung | Ursache / Technischer Hintergrund |
|---|---|---|
| `RISK-FLASH-REMANENCE` | Flash-Remanenz | Wear-Leveling und Out-of-Place-Writes im Flash-Controller (FTL) entziehen sich OS-Schreibbefehlen. |
| `RISK-FS-JOURNAL-OR-COW` | FS-Journale / CoW | Copy-on-Write (Btrfs, ZFS, APFS) und Dateisystemjournale (ext4) halten alte Datenblöcke vor. |
| `RISK-WRAPPED-KEY-REMANENCE` | Reste verpackter Schlüsselkopien | Mögliche Altdaten verpackter Schlüssel in temporären Bereichen. |
| `RISK-FREED-BLOCK-REMANENCE` | Reste in freigegebenen Blöcken | Freigegebene Dateisystemblöcke vor deren Überschreibung durch neue Allokationen. |
| `RISK-OLD-KEYSLOT-REMANENCE` | Reste alter Keyslots | Überschriebene Passphrase-/Master-Keyslots im Dateisystem. |
| `RISK-PROCESS-MEMORY` | Flüchtiger Prozessspeicher | Datenreste im RAM bis zur Wiederallokation oder bei fehlendem `mlock`/`madvise`. |
| `RISK-INDEX-TOPOLOGY-INFERENCE` | Indextopologie-Rückschlüsse | Kantenstrukturen im HNSW-Graphen bis zum vollständigen Neuaufbau (`rebuild`). |
| `RISK-TEXT-INDEX-UNTIL-PURGE` | Textindex bis Segment-Rewrite | Unverlinkte Postings im Segment bis zum Segment-Rewrite im ersten Purge-Fenster. |
| `RISK-REPLICA-WINDOW` | Replika-Synchronisationsfenster | Zeitfenster bei Replikationen vor Übernahme des Löschauftrags. |
| `RISK-EXTERNAL-COPIES` | Externe Backups / Kopien | Kalte Backups, Archivbänder oder externe Snapshots außerhalb der Contextra-Instanz. |
| `RISK-EXTERNAL-PROCESSORS` | Externe Verarbeiter | An Subunternehmer oder externe APIs gesendete Datenbestände. |
| `RISK-PSEUDONYM-COMMITMENT` | Pseudonym-Commitment | Verbleibende Hash-Commitments im externen Ledger zur Restore-Erkennung. |
| `RISK-PURGE-SCALES-WITH-DB` | Purge-Skalierung | Physisches Umschreiben großer SSTables skaliert mit der Datenbankgröße. |
| `RISK-PURGE-DEFERRED` | Aufgeschobener Purge | Purge erfolgt verzögert im Batch-Fenster (`Timing::Batched`). |
| `RISK-KEY-COMPROMISE` | Schlüsselkompromittierung | Risiko bei Kompromittierung des Master-Schlüssels oder der Passphrase. |

---

## 4. Rechenschaft, Datenschutz und Datenhygiene

Zur Erfüllung von Art. 5 Abs. 2 DSGVO ohne irreführende Aussagen gilt:
1. **Keine Rechtsberatung:** Der Bericht dokumentiert verifizierte technische Fakten (Zustände, Scans, Hashes), keine rechtliche Wertung.
2. **Datenhygiene in Journal und Ledger (P13, HR-4, I-11):**
   - Weder das Intent-Journal (`erasure.jrn`) noch das Ledger (`ledger.log`) noch der Bericht enthalten Klartext-Namen, Dokumentinhalte oder Betroffenenkennungen.
   - Identifikatoren werden ausschließlich als BLAKE3-Keyed-Hashes (`doc_tag`, `cluster_tag`, `tenant_tag`) oder opake `ErasureId` geführt.
   - Schlüsselmaterial verlässt niemals unverschlüsselt den Tresor und erscheint in keinem Log.
3. **Signaturumfang (SPEC E-L11):**
   - Der Bericht trägt den Pflichttext: *„Die Signatur beweist Integrität und Herkunft aus dieser Installation, nicht die Wahrheit des Inhalts.“*

---

## 5. Zustandsmaschine, Idempotenz und Wiederanlauf

```
  [Requested] ──────► [Gated] ──────► [KeyDestroyed] ──────► [Purging] ──────► [Verifying] ──────► [Completed]
       │                 │                   │                    │                   │
       └─────────────────┴───────────────────┴───────────► [Pending(reason)] ◄───────┘
  [Refused] (terminal)
```

### 5.1 Absturz- und Wiederanlauf-Matrix (Crash-Recovery)

| Absturzpunkt | Zustand auf Platte bei Neustart | Wiederanlauf-Aktion (`Startup`, 14.1) | Endzustand |
|---|---|---|---|
| Vor Gate-Log-fsync | Nur Journal `Requested` oder gar nichts | Verwaisten Journal-Eintrag verwerfen oder ablehnen | Keine Wirkung |
| Nach Gate-Log, vor Vault-Slot-Nullung | Gate-Log persistiert, Slot noch aktiv | Recovery liest Gate-Log vor Lesepfaden (I-2), nullt Vault-Slot nach (FM-1) | Fortsetzung bis `Completed` |
| Nach Vault-Nullung, vor Journal `KeyDestroyed` | Slot genullt, Gate aktiv | Recovery erkennt fehlenden Schlüssel, schreibt Journal `KeyDestroyed` | Fortsetzung bis `Completed` |
| Während SSTable/Segment-Rewrite | Alte Dateien intakt, `.tmp`-Dateien vorhanden | Verwaiste Dateien in `tmp/` entfernen (FM-2), Rewrite wiederholen | Fortsetzung bis `Completed` |
| Während Messung | Purge abgeschlossen, Zeuge fehlt | Positivkontrolle und MK1–3 Messung erneut ausführen | `Completed` (oder `Pending`) |
| Bei fsync-Fehler | Dateihandle vergiftet (`Poisoned`) | Handle verwerfen, Store verweigert Start / geht in Recovery | `Pending(Io)` |

---

## 6. End-to-End Testentwurf (Lab-Messung, ctx-verify, Mutantenprüfung)

### 6.1 Testfall 1: Vollständige Löschkette (`T-ERASE-E2E-1`)
- **Ziel:** Nachweis der Kette Auftrag → Gate → KeyDestroyed ($O(1)$) → Purge → MK1–3 → Report.
- **Ablauf:**
  1. Mandanten anlegen, 100 Dokumente mit bekannten Needles und Embeddings einfügen.
  2. Zeitmessung: `erase(doc_1)` aufrufen.
  3. Verifizieren: Zeit bis `Gated` liegt im Millisekundenbereich ($O(1)$); Lesepfad liefert sofort `ContextraError::Erased`.
  4. Schlüsselprüfung: $D$-Slot in Vault ist mit Nullen überschrieben; Entschlüsselung schlägt fehl.
  5. Purge abwarten: SSTable und Text-/Vektorsegmente werden umgeschrieben; WAL wird getruncated (`WalFloor`).
  6. Messung: `Scanner<Armed>` liefert Zeugen für alle Schichten mit 0 Matches.
  7. Bericht: Ed25519-Signatur ist gültig, Restrisiken vollständig vorhanden, keine Klartext-PII im Bericht.

### 6.2 Testfall 2: Unabhängige Verifikation mit `ctx-verify scan` (`T-CTX-VERIFY-1`)
- **Ziel:** Unabhängige externe Prüfung ohne Produktionsdecoder.
- **Ablauf:**
  1. Nach Abschluss von Test 1 wird das Datenverzeichnis mit dem CLI-Werkzeug `ctx-verify` analysiert:
     ```bash
     ctx-verify report verify --report erasure_report.json --key pubkey.pem
     ctx-verify scan --data-dir ./data --target-needle "GEHEIMNIS_NEEDLE"
     ```
  2. `ctx-verify` bestätigt die Korrektheit der kanonischen JSON-Formatierung, der Signatur und der Abwesenheit von Zielfragmenten (MK2).

### 6.3 Testfall 3: Mutantenprüfung „Schicht vergessen“ (`T-MUT-LAYER-FORGOTTEN-1`)
- **Ziel:** Nachweis, dass eine unterdrückte Schichtlöschung fail-closed erkannt wird.
- **Ablauf:**
  1. Mutation injizieren: Im Purge-Orchestrator wird der Aufruf `text_layer.purge(...)` oder `vector_layer.purge(...)` absichtlich übersprungen.
  2. Messung ausführen: Der Scanner MK1/MK2 findet verbliebene Residuen im Text- oder Vektorindex.
  3. Erwartetes Verhalten:
     - `Scanner::finish` schlägt mit `Err(Residue { matches: >0 })` fehl.
     - Es wird kein `LayerWitness` erzeugt.
     - Der Saga-Zustand wechselt zwingend nach `ErasureStatus::Pending { reason: PendingReason::WitnessMissing }`.
     - Ein Abschlussbericht mit Status `Completed` wird **verhindert** (Invarianten I-4, I-19 verletzt).
