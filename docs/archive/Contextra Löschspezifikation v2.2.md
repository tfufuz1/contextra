# Contextra Löschspezifikation v2.2

**Status:** Entwurf zur Ablösung von §5 (Löschkette), §6 (Lesesperre), §4.5 bis §4.8 (Schlüssel), §5.6 (Mehrbetroffene) und §10 (Barrieren) der Spezifikation v2.1. Abschnitte, die nicht genannt sind, bleiben gültig.  
**Kennungen:** `ER-n` für Anforderungen, `ER-I-n` für Invarianten.

---

## 0. Ziele

1. **Sofortige Unlesbarkeit** nach bestätigtem Löschauftrag, ohne Wartezeit und ohne Abhängigkeit von Bereinigung oder Index-Neuaufbau.
2. **Keine hängenden Zustände.** Jeder Vorgang endet in einer gemessenen Klasse oder in einem benannten offenen Restbestand mit Frist.
3. **Löschung ist eine Dateioperation, keine Suche im Inhalt.** Personenbezogene Daten liegen so, dass Entfernen ganzer Dateien genügt.
4. **Ein Schreibprozess.** Keine Nebenläufigkeitsbarrieren, die für einen Knoten mit einem Writer keinen Zweck haben.
5. **Nachweis nur über Messung** mit Positivkontrolle. Nicht gemessene Prüfungen werden als `not_implemented` ausgewiesen, nie als erreicht.

**Nicht-Ziele:** Keine Aussage „vollständig gelöscht“ ohne gemessene Klasse. Keine Garantie für Kopien außerhalb des Systems. Keine automatische Aufteilung von Mehrbetroffenen-Dokumenten (siehe §4).

---

## 1. Datenpartitionen

|Klasse|Inhalt|Schlüssel|Löschbar|
|---|---|---|---|
|**K**|Unternehmenswissen ohne Personenbezug|`ArtKey` je Segment|Über Collection-Drop oder Segmentersatz|
|**P**|Personenbezogene Dokumente|`DocDek` je Dokument-Version|Atomar über Schlüsselvernichtung|
|**D**|Ableitungen (Indizes, Embeddings, Caches, Zusammenfassungen)|`ArtKey` je Segment, mit `source_set`|Kaskadiert über `source_set`|
|**E**|Journal, Gate-Log, Schlüsselregister, Berichte|`ErasureKey` bzw. Journal-Schlüssel|Nach Aufbewahrungsfrist|

**ER-I-1 (Partitionstrennung).** Klassen K, P, D und E werden nie in derselben Datei gemischt. Ein Indexsegment hat genau eine Quellklasse.

**ER-I-2 (Ein Quellbezug je Segment).** Ein Indexsegment der Klasse D trägt genau eine Quellklasse (`KOrd` oder `VerOrd`). Die Sammelmenge `source_set` ist typisiert und kanonisch sortiert.

**Begründung:** Trennung erlaubt Löschen ganzer Segmente. Mischsegmente erzwingen Umschreiben, Umschreiben erzeugt Restkopien und Fehlerstellen.

---

## 2. Schlüsselmodell

**ER-1 (Schlüssel je Version).** Jede Version eines P-Dokuments erhält beim Schreiben einen zufälligen `DocDek` (32 Byte, `OsRng`). Zwei Versionen teilen nie denselben Schlüssel.

**ER-2 (Einheit = Version).** Die Löscheinheit ist die Version, nicht der Betroffene. Ein Dokument mit mehreren Betroffenen hat einen `DocDek` und eine im verschlüsselten Dokumentkopf gespeicherte Betroffenenliste.

**ER-3 (Wrap).** `DocDek` liegt in der KeyTable, gewrappt unter `KEK[collection, epoch]` mit AES-256-GCM-SIV, Nonce 96 Bit zufällig, AD = `key_id ‖ kek_epoch ‖ algo_id`. Die Wurzel liegt im Custody-Anker (§4).

**ER-4 (Keine HKDF für Datenschlüssel).** Datenschlüssel werden nie per HKDF aus anderen Schlüsseln abgeleitet. HKDF bleibt für strukturelle Schlüssel (Segment-MAC, Log-MAC) zulässig.

**Begründung ER-4:** Abgeleitete Datenschlüssel lassen sich nur über ihre Eltern vernichten. Das erzwingt Umwege bei Mehrbetroffenen, die ER-5 vermeidet.

---

## 3. Lesesperre (Gate)

### 3.1 Mengen

|Menge|Element|Bedeutung|
|---|---|---|
|`ErasedVer`|`VerOrd`|Version gelöscht: `Erased`|
|`ErasedSubj`|`SubjOrd`|Betroffener gelöscht: verhindert neue Zuordnung|
|`RestrictedVer`|`VerOrd` + Grund|Eingeschränkt: nur mit Audit-Eintrag lesbar|
|`SupersededVer`|`VerOrd`|Ältere Version: nur mit Audit lesbar|
|`RedactedK`|`KOrd`|Takedown in K: Platzhalter|

Ordinale sind dicht, monoton und nie wiederverwendet. Die Zuordnung Ordinal ↔ ID liegt nur im verschlüsselten Kopf, nie in Klartext.

### 3.2 Prüfreihenfolge

`ErasedVer` > `RestrictedVer` > `SupersededVer`. Jede Prüfung ist O(1), nutzt Roaring-Bitmaps und benötigt keine E/A.

**ER-5 (Gate vor Entschlüsselung).** Jeder Lesepfad (Suche, Abruf, Zeitreise, Cache-Treffer, Export, Vorschau) prüft das Gate vor jedem Schlüsselzugriff und vor jeder Ausgabe. Ein Cache-Eintrag mit `source_set ∩ ErasedVer ≠ ∅` gilt als Miss.

**ER-6 (Zeitreise).** Das Gate wird immer gegen den **aktuellen** Stand geprüft. Eine Zeitreise auf `as_of = T` liefert `Erased`, wenn das Ziel heute gelöscht ist.

### 3.3 Publikation ohne Barriere

Der einzige Schreiber schreibt zuerst den Gate-Eintrag, führt `fsync` aus und veröffentlicht danach einen neuen unveränderlichen Snapshot per Zeigertausch (`ArcSwap`). Leser greifen auf den Snapshot zu.

**ER-7 (Kein Fence, keine Barriere).** Das Protokoll erfordert keine Dekker-Fence und keine CommitBarrier/PurgeBarrier. Die Sichtbarkeit ist durch die Reihenfolge `fsync → Snapshot-Swap` gesichert.

**ER-8 (Fehlerfall).** Schlägt `fsync` fehl, wird der Snapshot nicht getauscht, der Aufruf liefert `Err`, und der Mandant geht in `Quarantine` (keine Lesezugriffe, bis Wiederherstellung).

### 3.4 Gate-Log

Rahmen: `[seq u64][op u8][len u16][payload][mac 32]`, HMAC-SHA256-Kette mit `HKDF(K_gate, "ctx/gate/v2" ‖ tenant)`.

Operationen: `EraseVer`, `EraseSubj`, `Restrict`, `Release` (nur aus `Restricted`), `Supersede`, `RedactK`, `HighWater`, `Snapshot`.

**ER-9 (Verdichtung).** Wöchentlich oder bei Schwellwert: neuer Snapshot der Mengen samt MAC in neuer Datei, atomarer Tausch, alte Datei wird archiviert. `ErasedVer`, `ErasedSubj` und `RedactedK` werden nie verkleinert.

---

## 4. Schlüsselwurzel und Anker

|Stufe|Wurzel|Erreichbare Klasse|
|---|---|---|
|**K1**|Argon2id aus Passphrase (m ≥ 64 MiB, t ≥ 3, ≥ 250 ms), Softwarespeicher|C mit Annahme `A-FLASH`|
|**K2**|Schlüsselmodul (TPM 2.0, HSM, OS-Keyring)|C mit bestandener Konformitätsprüfung|
|**K3**|Externer KMS/HSM mit Vernichtungsnachweis|C mit Anbieter-Attest|

**ER-10 (Rotation mit Generationen).** Die Wurzel besitzt Generationen. Rotation erzeugt eine neue Generation, die alte wird vernichtet, sobald alle Schlüssel umgewrappt sind. Der Anker zählt Generationen, nicht Schlüssel.

**ER-11 (Epochen-Rewrap).** Die KEK-Epoche wechselt periodisch. Dabei werden nur KeyTable-Einträge umgewrappt, keine Daten. Aufwand: O(Anzahl Schlüssel).

**ER-12 (KeyTable-Sharding).** Die KeyTable ist in Shards nach `key_id`-Bereich aufgeteilt (Standard 256 Shards). Vernichtung schreibt nur den betroffenen Shard neu: `tmp → fsync → rename → fsync(Verzeichnis)`.

Damit entfällt der Vollneuschreibvorgang pro Vernichtung, der bei großen Beständen teuer wird.

---

## 5. Mehrbetroffene

Dies ist der größte Verbesserungspunkt gegenüber v2.1. Die Vorgabe `BlockAll` lässt Mischsegmente dauerhaft gesperrt. In der Praxis bleibt damit der Großteil realer Löschaufträge im Zustand `Partial` hängen.

**ER-13 (Standardmodus `DeleteAll`).** Löscht ein Betroffener, wird die Version für alle Betroffenen gelöscht. Das ist die sichere Standardwirkung: Unlesbarkeit gegenüber jedem Mitbetroffenen wird nicht riskiert.

**ER-14 (Trockenlauf).** Vor der Löschung läuft innerhalb der Frist `erase_return_timeout` (Standard 10 s) ein Trockenlauf. Er ermittelt Mitbetroffene, Aufbewahrungspflichten, Rechtssperren, Einsprüche und offene Vorgänge. Im Bericht stehen nur Anzahl und Modus, nie Kennungen.

**ER-15 (Konfliktdokumente).** Hat der Trockenlauf einen Konflikt (Aufbewahrungspflicht, Rechtssperre, Einspruch oder offener Vorgang eines Mitbetroffenen), wird die Version sofort `Restricted(MultiSubjectReview)`. Die Löschung wird nicht automatisch ausgeführt.

**ER-16 (Review).** Ein Mensch entscheidet zwischen `DeleteAll`, `RestrictShared` und `RedactSplit`. Ein Review-Fristablauf eskaliert nur mit Erinnerung. Ein automatisches `DeleteAll` nach Fristablauf ist verboten.

**ER-17 (Aufteilung).** `RedactSplit` erzeugt eine neue Version ohne die Anteile des Betroffenen. Die Aufteilung ist ein neuer Schreibvorgang mit eigenem Schlüssel (ER-1). Automatische Aufteilung ist in v2.2 nicht vorgesehen, weil die Erkennung unvollständig ist.

**Konsequenz:** Der Ingest verweigert personenbezogene Dokumente ohne deklarierte Betroffene (§9.1 Stufe 1 der v2.1). Nicht erkannte Mitbetroffene werden erst nach Erkennung durch den Trockenlauf sichtbar.

---

## 6. Ableitungen

### 6.1 Typzwang

**ER-18 (Herkunft).** `Derived<T>` kapselt Nutzdaten zusammen mit einer `NonEmpty<SourceSet>`. Ein Index oder Cache nimmt Daten nur über `put_derived(Derived<T>)` an. Ableitungen ohne Herkunft sind nicht konstruierbar.

### 6.2 Betroffenheit

**ER-19.** Ein Ableitungsobjekt ist betroffen genau dann, wenn `source_set ∩ ErasedVer ≠ ∅` (Klasse D aus P) bzw. `source_set ∩ RedactedK ≠ ∅` (Klasse D aus K).

**ER-20 (Keine Stale-Ableitung).** Betroffene Ableitungen werden in Schritt S2 **entfernt**, nicht als veraltet markiert. Eine Neuerzeugung erfolgt später als neuer Schreibvorgang aus lebenden Quellen.

**Begründung:** Eine veraltete Zusammenfassung oder ein Embedding enthält die gelöschten Inhalte weiterhin. Ein Markierungszustand ist kein Löschen.

### 6.3 Textindex mit inkrementellen Statistiken

BM25 braucht `N`, `df(t)` und `avgdl`. Diese Werte dürfen die gelöschte Version nicht mehr enthalten.

**ER-21 (Inkrementelle Statistik).** Beim Löschen einer Version werden `N`, `df(t)` und die Längensumme um deren Beiträge verringert. Das geschieht im selben Schreibschritt wie der Gate-Eintrag. Eine volle Neuberechnung ist nur bei Verdichtung nötig.

**ER-22 (Keine Zahlen aus gelöschten Quellen).** Ranking, `df` und `avgdl` lesen nur lebende Versionen. Damit beeinflusst eine gelöschte Version keine Rangfolge mehr, auch nicht über Häufigkeiten.

### 6.4 Vektorindex

**ER-23 (Flache exakte Suche, Slot-Sperre).** Vektoren werden in fp32 gespeichert und exakt durchsucht. Beim Löschen werden die Slots der Version im Arenaspeicher mit Nullen überschrieben (`zeroize`). Ein Slot wird erst nach vollständigem Segmentneuaufbau wiederverwendet.

**ER-24 (Kein Umverdrahten).** Ein Graph-Index wird beim Löschen nicht umverdrahtet. Das verhindert Topologie-Restinformation. Ist ein Graph-Index vorhanden, gilt ER-25.

**ER-25 (Neuaufbau bei Tombstone-Anteil).** Überschreitet der Anteil gelöschter Slots 20 %, baut ein Hintergrundprozess eine neue Generation aus lebenden Slots, schaltet atomar um und vernichtet den `ArtKey` der alten Generation.

---

## 7. Speicherformate

|Modus|Schlüssel|Einsatz|Löschwirkung|
|---|---|---|---|
|`RecordSealed`|`DocDek` je Version|P-Datensätze|Schlüsselvernichtung macht Datensätze unlesbar|
|`SegmentSealed`|`ArtKey` je Segment|K, D, Journal|Ersatzsegment schreiben, alten `ArtKey` vernichten, Altdatei entfernen|

**ER-26 (Dateiwahl).** Personenbezogene Dokumente werden je Betroffenencluster in eigene Segmente geschrieben. Cluster mit mehr als `cluster_seg_min` (4 MiB) erhalten eigene Segmente. Kleinere werden nach Schlüssel gebündelt.

**ER-27 (Clusterkennung ohne Klartext).** Der Clusterschlüssel wird aus dem Betroffenenschlüssel abgeleitet und nicht gespeichert. Nach Vernichtung des Betroffenenschlüssels bleibt eine verbleibende Cluster-ID ein bedeutungsloser Zufallswert. Das Restrisiko „Gruppierung gleicher Cluster-IDs“ wird ausgewiesen.

**ER-28 (Handkodierte Formate).** Journal, Segmentkopf, Gate-Log und KeyTable werden handkodiert mit geprüften Feldzugriffen. Konfiguration, API und Berichte nutzen serde. Plattenformate dürfen nicht aus Strukturlayouts entstehen.

**ER-29 (Segmentgröße).** `segment_max_bytes` = 256 MiB begrenzt die Kosten eines Neuschreibens.

---

## 8. Löschsaga v2.2

Die Schritte sind idempotent. Der Journalzustand ist die Wahrheit.

|Schritt|Takt|Inhalt|Durabilitätspunkt|Klasse|
|---|---|---|---|---|
|**S0**|1|Validierung, Berechtigung, Aufbewahrungsprüfung, Trockenlauf (ER-14)|—|—|
|**S1**|1|Gate-Eintrag, `fsync`, Snapshot-Swap, Rückgabe des Tickets|Gate-Log `fsync`|**L**|
|**S2**|2|Ableitungen mit betroffenem `source_set` entfernen (ER-20), Statistik verringern (ER-21)|Journal je Ableitung|L|
|**S3**|2|Physische Bereinigung: Caches zeroizen, Schreibpuffer leeren, Segmente ersetzen, Blobs überschreiben und entfernen, Slots nullen|Journal je Teilschritt|P (vorbereitet)|
|**S4**|2|Messung A mit Positivkontrolle (§10)|Messbericht|**P**|
|**S5**|2|Schlüsselvernichtung: Intent-Journal, Zeroize, KeyTable-Shard ersetzen, Registereintrag|`fsync` Intent, `fsync` Shard|(K)|
|**S6**|2|Messung B: Nadelscan, Entschlüsselungsprobe, Canary|Messbericht|**C**|
|**S7**|3|Epochen-Rewrap (ER-11), wenn ausstehend oder Schwellwert erreicht|Journal|C|
|**S8**|Hintergrund|Verdichtung, WAL-Entfernung mit Prüfung, Checkpoint-Sweep, Index-Neuaufbau bei ER-25|Manifest|—|
|**S9**|3|Hygiene-Messung, signierter Bericht (§9), Abschluss|Bericht `fsync`|**M**|

### 8.1 Reihenfolge

**ER-30 (Purge vor Vernichtung, Ausnahme nach Klasse).** Für Klassen ohne besonderen Schutzbedarf gilt: S3 und S4 vor S5. Für Klassen mit erhöhtem Schutzbedarf (ER-31) darf S5 vor S3 liegen.

**ER-31 (Frühe Vernichtung).** Für als besonders schutzbedürftig markierte Klassen oder bei Einhaltung einer Fristzusage (SLA ≥ 70 % der Zeit) wird der Schlüssel direkt nach S1 vernichtet. Die Bereinigung wird dann zur Verteidigung in der Tiefe. Der Bericht weist aus, welche Reihenfolge galt.

**Abwägung:** Frühe Vernichtung schützt vor Fehlern in der Bereinigung, verhindert aber das Nachholen von Schritten, die auf den Schlüssel angewiesen sind. Der Ablauf muss deshalb ohne Schlüssel auskommen: Die Liste der betroffenen Blöcke wird vorher als Inventar festgehalten (§10).

### 8.2 Fristen und Eskalation

**ER-32 (Bereinigungsfrist).** S3 hat eine Frist von 1 Stunde (`clean_deadline`). Bei Überschreitung: Zustand `Pending(CleanupTimeout)`, Alarm, und der Schlüssel wird trotzdem vernichtet (fail-forward). Der Bericht führt den Restbestand als `R-CLEAN-TIMEOUT`.

**Begründung:** Eine Löschung mit lebendem Schlüssel nach Fristablauf ist schlechter als eine teilweise bereinigte Löschung mit vernichtetem Schlüssel. Die Lesbarkeit ist dann ohnehin aufgehoben, und der Restbestand ist benannt.

**ER-33 (Crash-Invarianz).** Ein Absturz an jedem Durabilitätspunkt führt nach dem Neustart zur Fortsetzung. Der Start ruft `recover()` vor dem Öffnen von Lesepfaden auf. Bekannte Zustände:

|Journalzustand beim Start|Aktion|
|---|---|
|`Requested`|Erneut S0|
|`Gated` (S1 vollständig)|Ab S2 fortsetzen|
|`KeyDestroyBegun` ohne Abschluss|Vernichtung wiederholen (idempotent)|
|`Cleaning`|S3 fortsetzen|
|`KeyDestroyed`|Ab S6 fortsetzen|
|`Pending(...)`|Beim nächsten Takt aufnehmen|

Eine durable Sperre geht nie verloren (ER-I-3).

**ER-I-3 (Sperre bleibt).** Eine durable Sperre wird nie aufgehoben, außer durch den Pfad `Restricted → Released`, der nur für Aufbewahrungsfälle gilt. Ein Löschauftrag kehrt nie in `Restricted` oder `Released` zurück.

### 8.3 Nebenläufigkeit

**ER-34 (Ein Koordinator, Einzelausführung).** Pro Scope läuft höchstens eine Saga. Ein Duplikat liefert das bestehende Ticket. Der Key-Lock der Einheit wird von S1 bis S5 gehalten. Konkurrierende Schreibvorgänge auf derselben Einheit werden abgewiesen.

---

## 9. Bericht

### 9.1 Inhalt

Pflichtfelder:

- erreichte Klassen (`L`, `P`, `C`, `M`) und jeweils `reached` oder `not_reached`
- Custody-Stufe (K1 bis K3) als Feld, nicht im Klassennamen
- Messverfahren, Positivkontrollergebnisse, Schichtdigests
- Annahmen: `A-FLASH`, `A-RAM`, `A-EXTBACKUP`, `A-KEYLEAK`, `A-HOSTHIST`
- Residuen: `R-CLEAN-TIMEOUT`, `R-GROUP-LINK` (ER-27), `R-SAMPLE` (bei Stichproben)
- Löschreihenfolge (Standard oder früh nach ER-31)
- Anzahl Mitbetroffener und gewählter Modus, nie Kennungen
- `subject_ref_hash`: keyed BLAKE3 unter `K_ref` der aktuellen Epoche
- Verifikation und Validierung getrennt (siehe §10.4)

### 9.2 Kodierung und Signatur

**ER-35.** Der Bericht wird im kanonischen JSON nach RFC 8785 (JCS) kodiert und mit Ed25519 signiert. Der Signaturschlüssel ist ein Anker-Schlüssel, kein Datenschlüssel. Felder ohne kanonische Kodierung (z. B. Binärstrukturen ohne Versionsangabe) sind für signierte Inhalte verboten.

**ER-36 (Bescheinigung).** Aus dem Bericht wird eine einseitige Bescheinigung erzeugt mit Datum, Scope-Art, erreichten Klassen, Annahmen, Ausschlüssen, Digest und Signatur. Sie enthält nie IDs, Dateinamen oder Inhalte.

**ER-37 (Außentext).** Außentexte werden nur aus einer geschlossenen Satzbibliothek erzeugt. Jeder Satz ist an einen gemessenen Wert gebunden. Verbotene Begriffe und Formulierungen sind durch ein Build-Gate gesperrt (§10.5).

---

## 10. Messung

### 10.1 Grundsätze

1. **Messen vor Behaupten.** Ein Nachweis entsteht nur aus einem Scanner der besitzenden Schicht.
2. **Positivkontrolle.** Vor jedem Scanlauf wird ein synthetischer Canary eingeschleust. Der Scanner muss ihn finden, sonst ist der Lauf ungültig.
3. **Typzustand.** Ein `Scanner<Armed>` kann nur nach bestandener Positivkontrolle entstehen. Es gibt keinen Konstruktor, der ihn ohne Prüfung erzeugt.
4. **Zwei Quellen je Pflichtschicht.** Scanner der Schicht und unabhängige Black-Box-Probe des Orchestrators. Reine Selbstauskunft einer Schicht ist unzulässig.
5. **Ehrliche Zustände.** `reached`, `not_reached`, `not_implemented`, `sampled`. Eine nicht implementierte Prüfung wird nie als `reached` gemeldet.

### 10.2 Messungen

|Messung|Verfahren|Ergebnis|
|---|---|---|
|**A: Klartext** (S4)|Rohbytes aller Dateien nach kanonischen Termen, Roh-IDs, Embedding-Bytes, Chunk-Prüfsummen; dekodierte Sicht mit allen aktiven Schlüsseln|`P` bei 0 Treffern|
|**Inventar** (vor S5)|Je `key_id`: Liste aller Chiffratblöcke (Datei, Offset, Länge) als Digest und Zähler. Nur Metadaten|Grundlage für B|
|**B1: Nadelscan** (S6)|Suche der 96-Bit-Wrap-Nonce des vernichteten Schlüssels in allen Dateien einschließlich Temp und `.bak`|0 Treffer|
|**B2: Entschlüsselungsprobe** (S6)|Echte `open`-Operation auf je einem Block aus dem Inventar|Muss mit `KeyAbsent` fehlschlagen|
|**B3: Canary** (S6)|Eigener Datensatz mit eigenem Schlüssel über denselben Vernichtungspfad|Belegt Scanner und Pfad, nicht den konkreten Schlüssel|

**ER-38 (Inventarpflicht).** Blöcke ohne Zuordnung zu einem vernichteten oder lebenden Schlüssel sind ein Fehler (fail-closed).

### 10.3 Grenzen

Im Bericht stehen immer:

- Flash-Remanenz unterhalb des Dateisystems (`A-FLASH`)
- Schlüssel in RAM, Swap, Core-Dumps (`A-RAM`)
- externe Sicherungen (`A-EXTBACKUP`)
- Copy-on-Write-Kopien unterhalb der Dateisystemebene
- Daten, die vor der Löschung an externe Dienste ausgeleitet wurden

Der Canary-Lauf beweist die Korrektheit der Bereinigung, nicht die physische Abwesenheit im Speichermedium. Die kryptografische Garantie liegt in der Schlüsselvernichtung.

### 10.4 Verifikation und Validierung

**ER-39.** Verifikation ist der technische Lauf (Messung, Positivkontrolle, Signatur). Validierung ist die Abnahme durch den Betreiber. Beide Status stehen getrennt im Bericht. Im regulierten Betrieb ist eine Gegenzeichnung durch eine zweite Person Pflicht.

**ER-40 (Unabhängiger Prüfer).** Ein separates Werkzeug prüft Berichte, ohne Code aus dem Löschkern zu teilen. Es prüft Signatur, Kanonisierung, Digests und die Zustandsübergänge im Log.

### 10.5 Sprach- und Build-Gate

**ER-41.** Folgende Formulierungen sind in Berichten, Oberflächen, Dokumentation, Quelltext und Werbung verboten: „100 % gelöscht“, „garantiert gelöscht“, „garantiert vernichtet“, „unwiederbringlich“, „vergisst Daten“, „DSGVO-konform“, „DSGVO-zertifiziert“, „erfüllt Art. 17 automatisch“, „löscht aus allen Backups“, „revisionssicher“, „fehlerfrei“, jede Behauptung einer Zertifizierung oder behördlichen Billigung. Das Wort „gelöscht“ steht nur zusammen mit der erreichten Klasse. Ein Prüfskript sperrt den Build.

---

## 11. Backup und Wiederherstellung

**ER-42 (Zwei Spuren).** Datensicherungen enthalten nur Chiffrat und keine KeyTable und keinen Anker. Schlüsselsicherungen tragen ein signiertes `valid_until` mit `key_backup_max_age` (Standard 30 Tage). Abgelaufene Schlüsselsicherungen werden verweigert.

**ER-43 (Erasure-Register).** Gate-Log und Schlüsselregister werden in eine unabhängige Senke gespiegelt (separater Datenträger oder Ankerwert). Die Senke ist nicht Teil der Datensicherung.

**ER-44 (Restore-Guard).** Vor dem ersten Lesezugriff nach Restore:

1. Gate-Log der Sicherung mit der Senke abgleichen.
2. Wiederhergestellte Versionen mit `ErasedVer` vernichten (Schlüssel erneut vernichten, Segmente erneut bereinigen).
3. Ereignis `restore_reapplied` ins Log schreiben.

Fehlt die Senke oder ist sie veraltet, startet der Knoten in `Quarantine`. Im Standardbetrieb ist ein Restore ohne Senke nur mit ausdrücklicher lokaler Option zulässig und wird im Bericht ausgewiesen.

**ER-45 (Aufbewahrungshorizont).** `commit_key_retention` bestimmt, wie lange Schlüsselsicherungen gelten. Die Senke muss mindestens diesen Horizont abdecken. Andernfalls ist der Schutz nach Restore nicht belegbar, und das Verhalten `Quarantine` greift.

---

## 12. Aufbewahrung, Einschränkung, Legal Hold

**ER-46 (Trennung).** Die Einschränkung nach Art. 18 DSGVO (`Restricted`) und die Löschung nach Art. 17 (`Gated` → Vernichtung) sind getrennte Pfade. `Restricted` wird vor `Gated` entschieden und ist keine Rücknahme eines Löschauftrags.

**ER-47 (Zustandsautomat).**

```
Requested → Gated → Cleaning → Cleaned → KeyDestroyed → Measured → Reported → Completed
Requested → Restricted(Grund, bis)        (reversibel)
Restricted → Gated                        (Eskalation zur Löschung)
Restricted → Released                     (terminal)
Requested → Refused                       (Aufbewahrungspflicht, terminal)
jeder nicht-terminale Zustand → Pending(Grund) → Resume
```

Terminal: `Completed`, `Released`, `Refused`.

**ER-48 (Sichtbarkeit).** Ab `Gated` liefert kein Standardpfad Klartext. Ab `KeyDestroyed` ist der Inhalt kryptografisch nicht mehr zugänglich.

---

## 13. Parameter

|Parameter|Standard|Bedeutung|
|---|---|---|
|`erase_return_timeout`|10 s|Trockenlauf innerhalb der Antwortzeit|
|`clean_deadline`|1 h|ER-32|
|`epoch_len`|30 Tage|KEK-Epoche|
|`seal_after_deletion_max`|7 Tage|Spätester Rewrap nach Löschung|
|`segment_max_bytes`|256 MiB|ER-29|
|`cluster_seg_min`|4 MiB|ER-26|
|`keytable_shards`|256|ER-12|
|`tombstone_rebuild_ratio`|0,20|ER-25|
|`key_backup_max_age`|30 Tage|ER-42|
|`commit_key_retention`|≥ Restore-Horizont|ER-45|
|`review_remind_after`|14 Tage|ER-16, nur Erinnerung|
|`kdf_argon2id`|m ≥ 64 MiB, t ≥ 3, ≥ 250 ms|ER §4|
|`min_record_bytes`|256|Mindestgröße je Record|

Alle Werte sind Startwerte. Sie werden mit Messungen angepasst und im Bericht ausgewiesen.

---

## 14. Umfang der Änderung gegenüber v2.1

|Bereich|v2.1|v2.2|Wirkung|
|---|---|---|---|
|Mehrbetroffene|`BlockAll`, `Partial`|`DeleteAll` mit Trockenlauf und Review|Keine dauerhaften Sperren mehr|
|Schlüsselgranularität|DocDek je Einheit|DocDek je Version (ER-1, ER-2)|Klarere Löschwirkung bei Korrekturen|
|Ableitungen|Stale möglich (§5.3 Schritt 2)|Sofort entfernen (ER-20)|Keine gelöschten Inhalte in Ableitungen|
|Statistik|Neuberechnung bei Segmentneuschreiben|Inkrementell (ER-21)|Keine Zahlen aus gelöschten Quellen|
|Gate-Publikation|Dekker-Fence, Barrieren|`fsync` → Snapshot-Swap (ER-7)|Entfall von `CommitBarrier`, `PurgeBarrier`, Fence|
|Schlüsselvernichtung|Immer nach Bereinigung|Nach Klasse (ER-30, ER-31)|Kürzere Dauer bei besonderen Daten|
|KeyTable|Vollständiges Neuschreiben|Sharding (ER-12)|Kleinere Schreibmengen|
|Klassen|L, P, C*, C1–C3, M mit Siegeln 1–4|L, P, C, M, Custody als Feld|Einfacher zu erklären und zu prüfen|
|Restore|Schutz über Sicherungsfenster|Senke plus Restore-Guard (ER-43, ER-44)|Ein Mechanismus, klare Quarantäne|
|Berichtskodierung|Ed25519, kanonisch|JCS nach RFC 8785 (ER-35)|Verifizierbar mit Standardwerkzeugen|

**Neue Invarianten:** ER-I-1 (Partitionstrennung), ER-I-2 (ein Quellbezug je Segment), ER-I-3 (Sperre bleibt).

**Entfallende Komponenten:** Dekker-Protokoll (§10.3), CommitBarrier und PurgeBarrier (§10.1, §10.2), Siegel 1–4 (§5.3 Schritt 7–8), Erfüllungsklassen C1–C3 als Klassennamen, Graph-Residual R-GRAPH (nur mit Graph-Index relevant).

---

## 15. Offene Entscheidungen

1. **Früher Schlüssel (ER-31):** Welche Klassen gelten als besonders schutzbedürftig. Das ist eine Fachentscheidung des Betreibers, keine technische.
2. **Trockenlauf-Tiefe (ER-14):** Ob der Trockenlauf Mitbetroffene nur über Metadaten oder auch über Inhalte erkennt. Inhaltserkennung erhöht die Trefferquote, erzeugt aber Verarbeitung, die selbst zu prüfen ist.
3. **Senke (ER-43):** Welcher unabhängige Speicher oder Anker in der Zielumgebung verfügbar ist.
4. **Rechtliche Einordnung:** Ob Schlüsselvernichtung als Löschung gilt, ist nach dem Stand der Diskussion offen. Die Spezifikation trifft keine Aussage dazu und verlangt die Kombination mit Lesesperre und Bereinigung. Die Einordnung liegt beim Verantwortlichen und ist vor Außenkommunikation juristisch zu prüfen. Dies ist keine Rechtsberatung.