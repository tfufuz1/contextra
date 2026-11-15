# Systemische Rechts- und Architektur-Analyse zur Evaluation der Schlüsselvernichtung (Crypto-Shredding) als Löschnachweis unter DSGVO, BDSG und EU-KI-Regulierung

Die Durchsetzung des Rechts auf Löschung gemäß Art. 17 DSGVO in zeitgenössischen Datenspeichern steht in einem fundamentalen systemischen Spannungsverhältnis zur Architektur moderner Informationssysteme. Während das Datenschutzrecht ein unverzügliches und vollständiges Unkenntlichmachen personenbezogener Daten verlangt, basieren moderne Vektordatenbanken, Retrieval-Augmented-Generation-Systeme (RAG), Event-Streaming-Plattformen und immutable Log-Strukturen wie Write-Ahead-Logs (WAL) und Sorted String Tables (SSTables) auf Prinzipien der Unveränderlichkeit und des rein anhängenden Schreibens. In der industriellen Praxis wird das gezielte Löschen kryptografischer Entschlüsselungsschlüssel – das sogenannte Crypto-Shredding – häufig als alleinige Universallösung gehandelt, um Unveränderbarkeit und Löschpflichten harmonisch zu vereinen. Die vertiefte Analyse der Behördenpraxis, der Rechtsprechung sowie der informatischen Realität zeigt jedoch, dass die alleinige Schlüsselvernichtung keineswegs automatisch als vollständige Löschung anerkannt werden kann, sondern einer präzisen Verzahnung physischer und kryptografischer Bereinigungsverfahren bedarf.  

## 1. Rechtsdogmatische Einordnung der Datenlöschung nach Art. 17 DSGVO

Das europäische Datenschutzrecht verzichtet in Art. 17 Abs. 1 DSGVO auf eine explizite Definition des Begriffs der Löschung. Aus dem Nebeneinander der Begriffe „Löschen“ und „Vernichten“ in der Verarbeitungsdefinition des Art. 4 Nr. 2 DSGVO leiten Gerichte und Aufsichtsbehörden eine prinzipielle Technikneutralität ab. Löschung bedeutet demnach nicht zwingend die physische Zerstörung des Speichermediums. Das Standard-Datenschutzmodell (SDM, Baustein 60 „Löschen und Vernichten“, V1.0a) definiert Löschen als das Unkenntlichmachen gespeicherter personenbezogener Daten dergestalt, dass die Identifizierbarkeit der betroffenen Person ohne unverhältnismäßigen Aufwand nicht wiederhergestellt werden kann. Das SDM betont jedoch zugleich, dass eine Löschung erst dann vollzogen ist, wenn keine lesbaren Kopien oder Replikate beim Verantwortlichen oder seinen Auftragsverarbeitern verbleiben.  

In der instanzgerichtlichen Rechtsprechung hat sich die Auffassung etabliert, dass eine vollständige Anonymisierung die Erfüllung der Löschpflicht darstellen kann. Das Oberlandesgericht Dresden entschied hierzu im Urteil vom 03.02.2026 (Az. 4 U 292/25), dass die Anonymisierung ein rechtmäßiges „Minus zur Löschung“ bildet, sofern der Betroffene die Entfernung seiner Personenbezüge verlangt. Diese Formel darf nicht mit einer früheren Entscheidung desselben Gerichts verwechselt werden: Das Urteil des OLG Dresden vom 14.12.2021 (Az. 4 U 1278/21) behandelte das Spannungsverhältnis zwischen Aufbewahrungspflichten im Inkassowesen und der Trennung von Archiv- und Nutzungssystemen, entwickelte aber noch nicht die explizite „Minus“-Doktrin. Auch das Landgericht München II ordnete mit Endurteil vom 14.03.2025 (Az. 14 O 1461/24) die Anonymisierung als zulässigen Unterfall der Löschung ein.  

Dieser instanzgerichtlichen Praxis steht eine deutlich restriktivere Haltung der Aufsichtsbehörden gegenüber. Der Europäische Datenschutzausschuss (EDPB) kritisiert in seinem CEF-Bericht 2025 (verabschiedet am 10.02.2026), dass Unternehmen in der Praxis fälschlicherweise bloße Pseudonymisierungen, Verfremdungen oder Maskierungen als vollwertige Löschung deklarieren. Die Aufsichtsbehörden verlangen für das Erreichen des Anonymitätsstatus den Nachweis, dass eine Re-Identifizierung durch Dritte und den Verantwortlichen nach allgemeinem Ermessen unwahrscheinlich ist.  

|Rechtsquelle / Institution|Dokument / Urteil|Kernaussage & Relevanz für Crypto-Shredding|Belegstärke|
|---|---|---|---|
|EuGH|_Breyer_ (C-582/14) & _EDPS/SRB_ (C-413/23 P)|Relativer Personenbezug: Daten sind personenbezogen, wenn eine Stelle über vernünftigerweise einsetzbare Mittel zur Re-Identifizierung verfügt.|Verbindliches Recht **[R]**|
|OLG Dresden|Urt. v. 03.02.2026 – 4 U 292/25|Anonymisierung ist ein „Minus zur Löschung“ und erfüllt den Löschanspruch aus Art. 17 DSGVO.|Rechtsprechung Instanz **[R-Instanz]**|
|LG München II|Urt. v. 14.03.2025 – 14 O 1461/24|Anonymisierung gilt als rechtmäßiger Unterfall der Löschung.|Rechtsprechung Instanz **[R-Instanz]**|
|EDPB / EDSA|Leitlinien 02/2025 v2 (07.07.2026)|Verschlüsselte Daten bleiben personenbezogen; Schlüsselvernichtung macht Daten unlesbar, bis Algorithmen gebrochen werden.|Behördenauslegung **[B]**|
|EDPB / EDSA|CEF-Bericht 2025 (10.02.2026)|Reines Pseudonymisieren oder Maskieren genügt nicht als Löschung; vollkommene Anonymisierung ist erforderlich.|Behördenauslegung **[B]**|
|DSK|OH RAG (10/2025)|Löschung in RAG-Vektordatenbanken erfordert die physische Entfernung von Datenbankeinträgen.|Behördenauslegung **[B]**|
|NIST|SP 800-88 Rev. 2 (26.09.2025)|_Cryptographic Erase_ (CE) wird als Purge-Sanitisierungsmethode anerkannt, setzt aber strikte Key-Governance voraus.|Technischer Standard **[P]**|

 

## 2. Behördliche und internationale Bewertung der Schlüsselvernichtung

Die rechtliche Bewertung des Crypto-Shredding basiert auf der kryptografischen Natur der Verschlüsselung. Verschlüsselung stellt datenschutzrechtlich eine Form der Pseudonymisierung gemäß Art. 4 Nr. 5 DSGVO dar. Solange der Entschlüsselungsschlüssel existiert, verbleiben die Chiffretexte im personenbeziehbaren Raum. Wird der Schlüssel vernichtet, stellt sich die Frage, ob der verbleibende Chiffretext als anonymisiert oder gelöscht eingestuft werden kann.  

Die maßgebliche europäische Leitlinie findet sich in den vom EDPB verabschiedeten _Guidelines 02/2025 on processing of personal data through blockchain_ (Endfassung v2 vom 07.07.2026). Der EDPB stellt in den Randnummern 51–53 sowie 96 klar, dass verschlüsselte personenbezogene Daten auch nach der Vernichtung des Schlüssels datenschutzrechtlich personenbezogene Daten bleiben. Die Schlüsselvernichtung führt zwar dazu, dass der Chiffretext für Dritte unlesbar wird, dies gilt jedoch nur so lange, bis der kryptografische Algorithmus gebrochen wird, Entschlüsselungstechniken (etwa durch Quantencomputing) fortschreiten oder der Schlüssel vor seiner Löschung kompromittiert oder unbefugt kopiert wurde. Der EDPB fordert von Verantwortlichen, die auf kryptografische Löschkonzepte setzen, regelmäßige Risikobewertungen hinsichtlich der mathematischen Obsoleszenz von Algorithmen sowie Notfallpläne für den Algorithmenwechsel. Eine pauschale Fiktion, dass die Schlüsselvernichtung auf unveränderlichen Speichern wie einer Blockchain automatisch eine vollständige Löschung darstelle, wird vom EDPB ausdrücklich abgelehnt.  

Im internationalen Vergleich ordnet der US-amerikanische Standard NIST SP 800-88 Rev. 2 (veröffentlicht am 26.09.2025) _Cryptographic Erase_ (CE) als Sanitisierungsmethode der Kategorie _Purge_ ein. NIST betont jedoch, dass CE die physische Unzugänglichkeit des Mediums nicht ersetzt, sondern eine logische Bereinigung darstellt, die strikten Audit- und Verifikationsprozeduren unterliegt. Aus der Rechtsprechung des Europäischen Gerichtshofs zum relativen Personenbezug (EuGH C-582/14 _Breyer_ und EuGH C-413/23 P _EDPS/SRB_) folgt: Verfügt eine Stelle nach der Schlüsselvernichtung über keinerlei vernünftigerweise einsetzbare Mittel zur Re-Identifizierung mehr, entfällt für diese Stelle der Personenbezug. Es existiert jedoch bislang kein Gerichtsurteil, das Crypto-Shredding isoliert und ohne begleitende physische Bereinigungsmaßnahmen als vollstreckte Löschung anerkennt.  

## 3. Das technische Dilemma in RAG-, Vektor- und Event-Systemen

In Retrieval-Augmented-Generation-Systemen (RAG) werden Dokumente in Text-Chunks zerlegt, über Embedding-Modelle in hochdimensionale Vektoren transformiert und in Vektordatenbanken mittels Algorithmen wie _Hierarchical Navigable Small World_ (HNSW) indiziert. Das Löschen von Datensätzen in solchen Architekturen offenbart gravierende technische Sicherheitslücken, die den Einsatz von reinem Crypto-Shredding erschweren:  

1. **Ghost Vectors in HNSW-Indizes:** Vektordatenbanken nutzen aus Latenzgründen häufig logische Löschungen (_Soft Delete_ via Bitsets oder Tombstones). Der Vektor und seine Kantenbeziehungen verbleiben im HNSW-Graphen, um die Graph-Topologie nicht zu beschädigen. _Ghost Vectors_ bleiben über die Struktur der Nachbarschaftsknoten rekonstruierbar, was den Betroffenenanspruch untergräbt.  
    
2. **Inversionsangriffe (Vec2Text):** Hochdimensionale Vektor-Embeddings stellen keine anonymen Einweg-Hashes dar. Mithilfe spezialisierter Inversionsmodelle lässt sich der ursprüngliche Klartext aus dem Vektor mit hoher Genauigkeit rekonstruieren. Vektoren müssen daher rechtlich als pseudonyme, inhaltsreiche personenbezogene Daten behandelt werden.  
    
3. **Seitenkanäle und Metadaten-Leakage:** Verbleiben Chiffretexte fixer Länge, unveränderte Dokument-IDs oder Suchstatistiken im System, können Angreifer über Längenanalysen oder Membership-Inference-Angriffe Rückschlüsse auf das ursprüngliche Dokument ziehen.  
    

Theoretische Lösungsansätze wie die Epochen-Rotation, bei der Daten periodisch mit neuen Schlüsselgenerationen re-verschlüsselt werden, erweisen sich in der Praxis als unzureichend, wenn alte Schlüsselkopien in Backups oder Hauptspeicher-Dumps überdauern. Ein belastbares System muss daher flüchtige Indizes physisch bereinigen und unzugängliche persistente Reste kryptografisch entwerten.  

## 4. Die Zwei-Ebenen-Löscharchitektur

Um die Diskrepanz zwischen den rechtlichen Vorgaben und den Speichereigenschaften unveränderbarer Medien aufzulösen, erfordert die rechtssichere Löscharchitektur eine strikte Aufteilung in zwei systemische Ebenen. Eine reine Schlüsselvernichtung greift zu kurz, da sie unverschlüsselte Hauptspeicher-Indizes ignoriert; eine rein physische Überschreibung scheitert wiederum an SSD-Controller-Logiken (Wear-Leveling), Immutable-WAL-Strukturen und Backup-Snapshots.  

### Systemische Abgrenzung der Löschebenen

|Parameter|Ebene A: Klartext-Strukturen (Purge)|Ebene B: Persistente Reste (Crypto-Shredding)|
|---|---|---|
|**Systemische Verortung**|Arbeitsspeicher (RAM), flüchtige Caches, aktive HNSW-Vektorindizes (fp32 / SQ8), BM25-Postings, Term-Wörterbücher.|Immuta-SSTables auf Disk, Write-Ahead-Logs (WAL), Blob-Segmente, interner Checkpoint-Speicher, SSD-Flash-Zellen.|
|**Erforderliche Löschmethode**|Physische Bereinigung, Zeroizing von Arena-Slots, Segment-Rewrite der Posting-Listen, Cache-Invalidierung.|Kryptografische Entwertung durch Vernichtung des dokumentspezifischen Schlüssels (DocDek).|
|**Begründung der Methode**|Suchindizes benötigen unverschlüsselte Daten zur Traversierung. Schlüsselvernichtung wirkt hier nicht.|Physisches Überschreiben einzelner Blöcke ist auf SSDs wegen Wear-Leveling unzuverlässig. Crypto-Shredding greift sofort.|
|**Messung der Löschwirkung**|Struktureller Scan der Hauptspeicher-Slots und Dateisegmente; Positivkontrolle via Canary-Record.|Nachweis der Schlüsselabwesenheit in der KeyTable; Fehlschlagen der Entschlüsselungsprobe.|

 

### Hierarchische Schlüsselarchitektur auf Dokumentenebene

Eine wesentliche Fehlerquelle beim Crypto-Shredding ist eine zu grobe Schlüsselgranularität. Wird ein Schlüssel pro Mandant oder pro Datenbank-Collection verwendet, führt das Löschen eines einzelnen Dokuments zum unzulässigen Kollateralschaden (alle Daten des Mandanten werden unlesbar) oder die Löschung wird unterlassen. Die Architektur erzwingt daher eine Schlüsselhierarchie bis auf Dokumentenebene:  

KEKroot​⟶KEKcoll​⟶DocDekdoc​

Der Wurzelschlüssel (KEKroot​) wird über den Schlüsselableitungsalgorithmus Argon2id (m≥64 MiB,t≥3) aus einer Passphrase generiert. Daraus wird je Collection ein Schlüssel (KEKcoll​) abgeleitet. Jedes einzelne Dokument beziehungsweise jede Betroffenen-Einheit erhält einen eigenen 256-Bit-AEAD-Data-Encryption-Key (DocDekdoc​). Alle zugehörigen Text-Chunks, Vektoren und Blob-Einträge werden unter diesem konkreten DocDek verschlüsselt.  

Der gewrappte DocDek liegt in einer veränderbaren Schlüsseltabelle (`KeyTable`) mit Ersetzen-Semantik. Bei einem Löschbefehl wird der Schlüsseleintrag mit Nullen überschrieben, geflusht (`fsync`) und die Segmentdatei der `KeyTable` atomar ohne den gelöschten Schlüssel neu geschrieben. Ein append-only Logbuch für Schlüssel ist unzulässig, da es eine verbleibende Schlüsselkopie darstellen würde.  

### HNSW-Slot-Zeroing ohne Graph-Rewiring

Um bei Löschungen im HNSW-Vektorindex das aufwändige und nicht-deterministische Neu-Verknüpfen der Graph-Nachbarn (_Rewiring_) zu vermeiden, nutzt die Architektur ein Slot-Zeroing-Verfahren:  

1. Der betroffene Graph-Knoten bleibt als reiner Routing-Knoten im HNSW-Graphen erhalten, um die Routing-Topologie für Suchen aufrechtzuerhalten.  
    
2. Die Vektor-Nutzdaten des Slots (sowohl die fp32-Vollpräzision als auch die SQ8-Quantisierungscodes) werden im Hauptspeicher unverzüglich **mit Nullen überschrieben**.  
    
3. Der Knoten enthält keinerlei personenbezogene Inhaltsdaten oder Vektorrepräsentationen mehr.  
    
4. Erst wenn der Anteil inaktiver Tombstone-Knoten eine definierte Schwelle überschreitet (z. B. `tombstone_rebuild_ratio` =0,2), wird der HNSW-Index im Hintergrund vollkommen neu aufgebaut.  
    
5. Arena-Slots geleertes Vektoren dürfen erst wiederverwendet werden, wenn keine veralteten Kantenreferenzen mehr auf sie verweisen.  
    

## 5. Stufenmodell der Löschwirkung, Löschsaga und Nachweisführung

Um den rechtlichen Anforderungen der Rechenschaftspflicht nach Art. 5 Abs. 2 DSGVO zu genügen, unterteilt das System die Löschwirkung in vier nachweisbare Klassen. Statische Garantien wie „100 % gelöscht“ sind unzulässig; stattdessen gibt das System systemisch nur diejenige Klasse aus, die messtechnisch belegt ist.  

### Die vier Löschwirkungsklassen

|Klasse|Name|Aussage & Systemzustand|Voraussetzung für Zertifizierung|
|---|---|---|---|
|**Klasse L**|Logisch gesperrt|Der Scope ist für alle Lese- und Suchpfade gesperrt.|`ErasedSet`-Eintrag durabel via `fsync` geschrieben; Lesesperre aktiv.|
|**Klasse P**|Klartext bereinigt|In allen aktiven Indizes und Caches der Ebene A existiert keine Klartextkopie mehr.|Ebene-A-Purge vollzogen; Positivkontrolle (Canary-Scan) erfolgreich.|
|**Klasse C**|Persistente Reste unlesbar|Alle verbleibenden Kopien auf Ebene B liegen nur als Chiffrat unter vernichtetem DocDek vor.|Ebene-B-Key-Destruction vollzogen; Schlüsselabwesenheit gemessen; Canary-Entschlüsselungsprobe fehlgeschlagen.|
|**Klasse E**|Technisch vollständig|Klasse P und Klasse C sind lückenlos nachgewiesen; alle Systemschichten gemessen.|Nachweis aller Schichten (`LayerWitness`); Vollständige Positivkontrolle; eIDAS-Zeitstempel verankert.|

 

### Die 7-stufige Löschsaga

Der Löschvorgang wird als dauerhafte, zustandsbasierte Saga ausgeführt, die auch bei Systemabstürzen (_Crash/Power-Loss_) deterministisch fortgesetzt werden kann. Die Ablaufsequenz lautet:  

Requested⟶Frozen⟶LogicallyDeleted⟶PlaintextPurged⟶PlaintextMeasured⟶KeyDestroyed⟶Measured⟶Reported/Completed

In Schritt 2 (`Frozen`) wird der Eintrag im `ErasedSet` durabel via `fsync` fixiert; ab diesem Moment greift die Lesesperre für alle Anfragen. In Schritt 4 (`PlaintextPurged`) werden die Indizes der Ebene A gereinigt und Dateiblöcke neu geschrieben. Erst nach der messtechnischen Bestätigung der Ebene A in Schritt 5 (`PlaintextMeasured`) darf Schritt 6 (`KeyDestroyed`) ausgeführt werden. Würde der Schlüssel vor der Bereinigung der Ebene A vernichtet, wären Blöcke, die Daten mehrerer Dokumente enthalten, nicht mehr geordnet rekonstruierbar.  

### Positivkontrolle und Auflösung des Lösch-Paradoxons

Um fälschlich positive Messergebnisse durch defekte Scanner auszuschließen, nutzt das System zwei Positivkontrollen:  

1. **Canary-Klartext-Scan (Ebene A):** Ein vordefinierter Kontroll-Datensatz (_Canary_) wird vor dem Scan eingeschleust. Der Scanner muss diesen Kontroll-Datensatz finden; schlägt dies fehl, ist der gesamte Scan-Lauf ungültig.  
    
2. **Canary-Entschlüsselungsprobe (Ebene B):** Vor der Vernichtung des DocDek wird ein Test-Chiffrat erzeugt. Nach der Schlüsselvernichtung führt der Scanner eine Entschlüsselungsprobe durch. Die Löschung auf Ebene B gilt erst dann als bewiesen, wenn die Entschlüsselung nachweislich fehlschlägt.  
    
3. **Datensparsame Commitments:** Um das _Lösch-Paradoxon_ – bei dem die Speicherung von Namen oder IDs im Löschnachweis einen neuen Löschanspruch auslöst – zu vermeiden, speichert der Löschnachweis und das `ErasedSet` ausschließlich HMAC-Commitments:  
    

SubjectCommitment=HMAC-SHA256(Kcommit​[t,e],Kanonische Betroffenen-ID)

Der Schlüssel Kcommit​ wird je Mandant t und Epoche e geführt. Nach Ablauf der Aufbewahrungsfrist der Löschnachweise wird Kcommit​ vernichtet. Damit werden die Löschnachweise anonymisiert, während die kryptografische Verettung der Audit-Protokolle erhalten bleibt.  

### Backup-Handling und Restore-Guards

Das SDM (Baustein 60) und der EDPB-CEF-Bericht 2025 akzeptieren, dass Backups primär der Systemintegrität dienen und nicht bei jedem Löschantrag punktuell verändert werden müssen. Maßgeblich ist das Verhalten beim Restore:  

- **Restore-Guard:** Vor dem ersten Lesezugriff nach dem Einspielen eines Backups liest das System zwingend das `ErasedSet` des Zielsystems. Alle gelöschten Betroffenen werden sofort nachgesperrt, wiederhergestellte DocDek erneut vernichtet und die Löschsaga wiederholt.  
    
- **Restore-Horizont:** Checkpoints oder Backups, die älter sind als die Haltedauer des `ErasedSet` (`erased_set_retention`), werden bei der Wiederherstellung verweigert (`RestoreBeyondHorizon`), da der Restore-Guard ihre Bereinigung nicht mehr garantieren kann. Es gilt die Regelschranke: `backup_max_age` ≤ `erased_set_retention`.  
    
- **Unanwendbarkeit von § 35 BDSG:** Die Versuche, das Löschen in automatisierten Vektordatenbanken oder Backups unter Berufung auf § 35 Abs. 1 BDSG zu verweigern, sind rechtswidrig. § 35 BDSG ist auf _nicht-automatisierte_ Datenverarbeitungen beschränkt und findet bei unrechtmäßigen Verarbeitungen keine Anwendung.  
    

### Aufbewahrungspflichten und der Systemzustand `Restricted`

Stehen gesetzliche Aufbewahrungspflichten (z. B. § 147 AO oder § 257 HGB) einer Löschung entgegen, greift die Ausnahme nach Art. 17 Abs. 3 lit. b DSGVO. Das Bürokratieentlastungsgesetz IV (BEG IV, BGBl. 2024 I Nr. 323) hat die Aufbewahrungsfrist für Buchungsbelege zum 01.01.2025 von 10 auf **8 Jahre** verkürzt (Handelsbriefe verbleiben bei 6 Jahren).  

Das System reagiert auf gesetzliche Aufbewahrungspflichten mit dem Übergang in den Zustand **`Restricted`** (Einschränkung der Verarbeitung nach Art. 18 DSGVO): Es findet keine Schlüsselvernichtung und kein physischer Purge statt. Das Dokument wird aus dem Retrieval-Index isoliert und im `ErasedSet` als gesperrt markiert. Suchanfragen im aktiven Wissensspeicher liefern das Dokument nicht mehr zurück, während das Archivdokument unverändert im gesperrten Speicher verbleibt, bis die Frist abläuft.  

## 6. Haftung, AI Act, NIS2 und Lauterkeitsrecht

Nach der Systematik der DSGVO (Art. 4 Nr. 7, Art. 28) ist die rechtliche Rollenverteilung zwischen Softwareanbieter und Anwender strikt zu trennen: Ein Anbieter, der On-Premise-Software lizenziert und keinen Zugriff auf Klartextdaten, Indizes oder Schlüssel des Kunden hat, ist weder Verantwortlicher noch Auftragsverarbeiter. Der Anwender verbleibt die alleinige verantwortliche Stelle und trägt die Verantwortung für die Wahl der Löschparameter und die rechtliche Bewertung des Crypto-Shredding. Art. 25 DSGVO bindet primär den Verantwortlichen; eine direkte Herstellerhaftung existiert im geltenden Recht noch nicht.  

Im Rahmen des EU AI Act (VO (EU) 2024/1689) qualifiziert ein RAG-System als KI-System (Art. 3 Nr. 1). Durch den _Digital Omnibus on AI_ (VO (EU) 2026/1744 vom 08.07.2026) wurden die Fristen für Anhang-III-Hochrisiko-Systeme auf den **02.12.2027** verschoben. Art. 12 AI Act verlangt die automatische Aufzeichnung von System-Ereignissen, **nicht** die dauerhafte Klartextspeicherung aller Inhaltsdaten. Der Konflikt mit Art. 17 DSGVO wird aufgelöst, indem Protokolle auf inhaltsfreie Metadaten beschränkt werden.  

Unter der NIS2-Richtlinie (Art. 21 Abs. 2 lit. h) ergibt sich für Betreiber kritischer Infrastrukturen die Pflicht zur Krypto-Agilität: Das System muss veraltete Chiffren geordnet austauschen können, ohne dass Datenverluste entstehen.  

Unzulässig und abmahnfähig nach § 5 UWG (Irreführung) sowie sachmangelbegründend nach § 434 BGB sind lauterkeitsrechtliche Werbeversprechen. Aussagen wie „garantiert DSGVO-konform gelöscht“ oder „erfüllt Art. 17 automatisch“ sind rechtswidrig. Produktbeschreibungen müssen präzise offenlegen, was technisch geschieht:  

```
UNZULÄSSIG (Irreführend nach § 5 UWG):
❌ "Unsere Vektordatenbank löscht personenbezogene Daten garantiert DSGVO-konform per Krypto-Shredding."
❌ "Mit unserer Software erfüllen Sie Art. 17 DSGVO vollautomatisch auf Knopfdruck."

RECHTSSICHER (Technisch präzise & datenschutzrechtlich korrekt):
✔ "Das System führt bei Löschaufrufen einen physischen Purge der Arbeitsspeicher-Indizes (HNSW/BM25) durch. Unveränderliche Speichersegmente werden durch die Vernichtung des dokumentspezifischen Schlüssels (DocDek) unlesbar gemacht. Das System stellt einen signierten, datensparsamen Löschbericht (DeletionProof) über die gemessenen Systemzustände aus. Die rechtliche Bewertung der Löschwirkung sowie die Einhaltung von Aufbewahrungsfristen verbleiben in der Verantwortung des Betreibers."
```

## 7. Systematischer Vergleich der Löschansätze

|Bewertungsdimension|Rein logisches Löschen (_Soft Delete_)|Pauschales Crypto-Shredding (Scope-Key)|Physisches Überschreiben (_Scrubbing_)|Zwei-Ebenen-Löscharchitektur (Referenzmodell)|
|---|---|---|---|---|
|**Technische Durchführung**|Inaktiv-Flag / Tombstone im Index gesetzt.|Vernichtung eines zentralen Mandanten- oder Collection-Keys.|Sektorweisendes Überschreiben aller Blöcke auf Medium.|Physischer Purge der Ebene A + DocDek-Vernichtung für Ebene B.|
|**Erreichte Löschklasse**|Klasse L (Logisch gesperrt).|Klasse C (auf Blockebene), aber Kollateralschaden.|Klasse P (Klartext bereinigt), scheitert an Flash-Speichern.|Klasse E (Technisch vollständig nachgewiesen).|
|**Rechtliche Anerkennung (Art. 17)**|Unzulässig nach DSK und EDPB.|Umstritten; EDPB 02/2025 verweist auf Restrisiken.|Anerkannt, sofern lückenlos vollzogen.|Höchste Praxisakzeptanz durch Krypto- und Purge-Kombination.|
|**Anwendbarkeit auf SSDs / RAG**|Problemlos, aber rechtlich wirkungslos.|Zerstört fremde Daten bei grober Granularität.|Unzuverlässig wegen SSD-Wear-Leveling & HNSW-Graphbrüchen.|Speziell für HNSW-Vektorindizes, LSM-Trees und SSDs entwickelt.|
|**Restrisiken**|Inversionsangriffe auf Vektoren (_Ghost Vectors_).|Krypto-Obsoleszenz, Key-Kompromittierung vor Vernichtung.|Hohe I/O-Latenz, Zerstörung von Nachbarknoten.|Remanenzspuren im nicht-adressierbaren RAM/Swap (deklariert).|
|**Nachweisform**|Internes Datenbank-Log.|Schlüssel-Löscheintrag im KMS.|Vernichtungsprotokoll des Wipe-Tools.|Signierter `DeletionProof` mit Positivkontrolle & eIDAS-Zeitstempel.|

 

## 8. Synthese und Handlungsempfehlungen

Aus der rechtsdogmatischen und technischen Evaluation ergeben sich folgende zentrale Handlungsempfehlungen für die Gestaltung datenintensiver Wissensspeicher:

1. **Kombinationspflicht von Purge und Crypto-Shredding:** Die reine Schlüsselvernichtung stellt ohne Begleitmaßnahmen keinen vollständigen Löschnachweis dar. Systeme müssen Hauptspeicher-Indizes (HNSW, BM25, Caches) physisch bereinigen (Ebene A) und unzugängliche, immutable Speicherblöcke kryptografisch entwerten (Ebene B).  
    
2. **Schlüsselgranularität auf Dokumentenebene:** Crypto-Shredding darf nicht auf Mandanten- oder Collection-Ebene ansetzen. Jeder Betroffene beziehungsweise jedes Quelldokument benötigt einen eigenen Data Encryption Key (DocDek), um Einzellöschungen ohne Kollateralverlust zu ermöglichen.  
    
3. **Automatisierte Restore-Guards:** Backups müssen bei der Wiederherstellung zwingend durch automatisierte Guards abgefangen werden, die den aktuellen Sperr-Index (`ErasedSet`) einlesen, wiederhergestellte Dokumente sofort nachsperren und deren Schlüssel erneut vernichten.  
    
4. **Datensparsame Nachweisführung:** Löschnachweise dürfen keine Klartext-IDs enthalten. Die Verknüpfung muss über zeitlich befristete, epochenbasierte HMAC-Commitments erfolgen, deren Schlüssel nach Ablauf der Aufbewahrungsfrist vernichtet wird.  
    
5. **Ehrliche Außenkommunikation:** In Handbüchern und Verträgen ist auf irreführende Begrifflichkeiten zu verzichten. Nachzuweisen sind ausschließlich die konkret gemessenen Systemzustände und deren deklarierte Schranken.