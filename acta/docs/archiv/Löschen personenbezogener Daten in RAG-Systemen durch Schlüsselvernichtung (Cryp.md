# Löschen personenbezogener Daten in RAG-Systemen durch Schlüsselvernichtung (Crypto-Shredding)

*Konsolidierte Rechts- und Architekturanalyse zu DSGVO, BDSG, EU-KI-Regulierung und technischen Standards. Arbeitsstand: 27. Oktober 2026. Dies ist keine Rechtsberatung.*

## 0. Gegenstand, Ergebnis und Lesehinweise

### 0.1 Gegenstand

Untersucht wird, ob und unter welchen Bedingungen ein lokal betriebenes, verschlüsseltes Retrieval-Augmented-Generation-System (RAG) das „Vergessen“ personenbezogener Daten durch die Vernichtung kryptografischer Schlüssel (Crypto-Shredding) rechtlich belastbar umsetzen kann. Das Dokument führt die rechtliche Bewertung (Löschpflicht nach Art. 17 DSGVO, Anonymisierung und Unkenntlichmachung, Nachweispflichten, Rollenverteilung, Aufbewahrungspflichten, KI-Regulierung) mit der technischen Analyse (Speicherorte, Vektorindizes, Schlüsselarchitektur, Löschnachweis) zusammen. Es trennt dabei durchgehend zwischen verbindlichem Recht, Behördenauslegung, Literatur- und Praxisauffassungen sowie ungeklärten Fragen.

### 0.2 Ergebnis in Kürze

1. **Kein Methodenzwang, aber ein klarer Maßstab.** Die DSGVO definiert „Löschung“ nicht und schreibt keine Methode vor. Löschen ist selbst eine Verarbeitung. Der behördliche Maßstab lautet: Die Daten müssen so unkenntlich gemacht sein, dass keine Person mehr identifizierbar ist und der Informationsgehalt nach allgemeinem Ermessen nicht oder nur unwahrscheinlich reproduzierbar ist.
2. **Schlüsselvernichtung als Löschung ist rechtlich offen.** Weder eine Aufsichtsbehörde noch ein Gericht hat Crypto-Shredding bislang ausdrücklich als Löschung im Sinne des Art. 17 DSGVO anerkannt oder abgelehnt. Die Argumentation ist möglich, hängt aber an strengen, dokumentierbaren Bedingungen.
3. **Verschlüsselte Daten bleiben personenbezogen, solange irgendein Schlüssel existiert.** Nach Vernichtung aller Schlüsselkopien ist der Chiffretext nur unlesbar, bis der Algorithmus gebrochen wird oder der Schlüssel zuvor kompromittiert wurde. Dieses Restrisiko ist zu dokumentieren und periodisch neu zu bewerten.
4. **Die Stärke des Verfahrens liegt im Wirkungsbereich des Schlüssels.** Ein RAG-System hält Daten in mehreren Repräsentationen, darunter unverschlüsselte Such- und Vektorstrukturen im Arbeitsspeicher. Diese erfasst die Schlüsselvernichtung nicht. Eine belastbare Architektur kombiniert deshalb physische Bereinigung der Klartext-Strukturen mit kryptografischer Entwertung persistenter Reste.
5. **Embeddings sind vorsorglich wie personenbezogene Daten zu behandeln.** Eine verbindliche Entscheidung dazu fehlt. Die gemeinsame Speicherung mit Chunk-Text, die Verknüpfung über Kennungen und die nachgewiesene Rekonstruierbarkeit sprechen für die vorsichtige Praxis.
6. **Backups müssen nicht sofort überall verändert werden, aber nach einer Rücksicherung muss unmittelbar erneut gelöscht werden.** Eine feste Höchstfrist nennt keine Behörde. Konkrete Rotationsfristen sind Praxisempfehlungen.
7. **Nachweis über Metadaten, nicht über Inhalte.** Die Rechenschaftspflicht (Art. 5 Abs. 2 DSGVO) verlangt, die Löschung belegen zu können. Der Nachweis darf keine Inhalte oder Klartext-Kennungen der betroffenen Person dauerhaft vorhalten.
8. **Rollen.** Ein Anbieter lokal installierter Software ohne Zugriff auf Klartext, Chunks, Vektoren und Schlüssel ist in der Regel weder Verantwortlicher noch Auftragsverarbeiter. Der Anwender trägt die datenschutzrechtliche Verantwortung. Pauschale Werbeaussagen zur Löschwirkung bergen erhebliches Wettbewerbs- und Vertragsrisiko.
9. **Aufbewahrungspflichten gehen vor, werden aber je Datum geprüft.** Dann tritt Einschränkung an die Stelle der Löschung. § 35 BDSG trägt für automatisierte Vektordatenbanken und Backups nicht.
10. **KI-Regulierung ersetzt die DSGVO nicht.** Der AI Act verlangt Ereignisprotokolle, keine dauerhafte Klartextspeicherung von Inhalten. Fristen für Hochrisiko-Systeme wurden nach den vorliegenden Angaben verschoben.

### 0.3 Einordnung der Aussagen

Das Dokument unterscheidet drei Stufen. **Verbindliches Recht** sind Gesetze, Verordnungen und Urteile des EuGH und des BGH. Entscheidungen von Instanzgerichten binden über den Einzelfall hinaus nicht und werden als solche benannt. **Behördenauslegung** umfasst Stellungnahmen, Leitlinien und Orientierungshilfen des Europäischen Datenschutzausschusses (EDPB), der Datenschutzkonferenz (DSK), der Landesbehörden und das Standard-Datenschutzmodell (SDM). Sie sind nicht bindend, bilden aber den Maßstab der Aufsichtspraxis. **Literatur, Praxis und technische Standards** sind Empfehlungen und Fachmeinungen ohne Rechtsverbindlichkeit. Fragen ohne belastbare Antwort sind ausdrücklich als „umstritten“ oder „offen“ gekennzeichnet.

### 0.4 Vorbehalt zum Prüfstand

Gesetzestexte, Behördenpapiere und Entscheidungen sind teils nur über Sekundärberichte oder Leitsätze belegt. Wo das für die Bewertung wesentlich ist, steht es im Text („nach den vorliegenden Berichten“, „erstinstanzlich“, „Volltext vor Verwendung prüfen“). Vor jeder Außenverwendung, insbesondere in Produkttexten, Verträgen oder Gutachten, sind alle Fundstellen am Primärtext zu verifizieren.

## 1. Ausgangslage: RAG-Architektur, Speicherorte und das Löschproblem

### 1.1 Aufbau eines RAG-Systems

Ein RAG-System trennt das statische Sprachmodell (LLM) von einer dynamischen, durchsuchbaren Wissensbasis. Quelldokumente werden in Textfragmente (Chunks) zerlegt. Ein Embedding-Modell wandelt die Chunks in hochdimensionale Vektoren um, die in einer Vektordatenbank mit Näherungssuche indiziert werden, häufig über Graphverfahren wie Hierarchical Navigable Small World (HNSW). Ergänzend arbeiten viele Systeme mit einem Volltextindex (etwa BM25 mit Posting-Listen und Term-Wörterbüchern). Bei einer Anfrage werden die relevantesten Chunks gesucht und dem LLM als Kontext übergeben.

### 1.2 Speicherorte mit eigenem Personenbezug

Jeder der folgenden Orte kann personenbezogene Daten enthalten und muss im Löschkonzept eigenständig behandelt werden:

- Quelldokument (Originaldatei, Archiv)
- Chunk-Text und Chunk-Metadaten (Dokument-ID, Seitenzahl, Zeitstempel, Autor)
- Vektor-Embeddings, im Arbeitsspeicher oft in voller Präzision (fp32) und quantisiert (etwa SQ8)
- Vektorindex mit Graphstruktur (Kanten, Nachbarschaftsbeziehungen, Tombstones)
- Volltextindex (Posting-Listen, Term-Wörterbücher, Segmente)
- Caches: Embedding-Caches, Abfrage-Caches, Kontext-Caches des LLM
- Protokolle: Abfrage- und Prompt-Historien, Fehlerprotokolle, Telemetrie
- Persistente Strukturen: Write-Ahead-Logs (WAL), Sorted String Tables (SSTables), Blob-Segmente, Checkpoints
- Temporäre Daten: Swap-Dateien, Crash- und Speicherabbilder, Debug-Dumps, Exporte
- Backups und Snapshots der Wissensbasis und des Schlüsselspeichers
- Schlüsselmaterial: Schlüsselspeicher, Hardware-Sicherheitsmodule (HSM), Cloud-KMS, Recovery- und Escrow-Kopien
- Kontexte bei externen LLM-Diensten, sofern Chunks dorthin übermittelt werden

### 1.3 Warum Löschen technisch schwierig ist

Das Datenschutzrecht verlangt unverzügliches, vollständiges Unkenntlichmachen. Moderne Speicherarchitekturen arbeiten dagegen mit Unveränderlichkeit und rein anhängendem Schreiben:

- **Immutable Strukturen.** WAL, SSTables und Event-Logs werden nicht an Ort und Stelle verändert. Löschungen entstehen erst bei späterer Kompaktierung.
- **Tombstones und Soft Delete.** Viele Vektorindizes markieren gelöschte Einträge nur logisch (Bitset oder Tombstone), um die Graph-Topologie nicht zu beschädigen. Vektor und Kantenbeziehungen können bis zur Kompaktierung oder zum Neuaufbau im Index verbleiben und darüber rekonstruierbar sein („Ghost Vectors“).
- **SSD-Controller.** Wear-Leveling und Over-Provisioning führen dazu, dass das Überschreiben einzelner Blöcke nicht verlässlich die physische Zelle erreicht.
- **Segment-Merges** im Volltextindex lassen alte Segmente zeitweise neben neuen bestehen.
- **Replikation und Backups** vervielfachen Kopien außerhalb der Kontrolle des einzelnen Löschbefehls.

### 1.4 Folgerung für die rechtliche Bewertung

Eine Löschbehauptung muss für jeden Speicherort einzeln begründet werden. Zwei Fragen sind zu trennen: Erstens, ob das gewählte Verfahren rechtlich als Löschung (oder als gleichwertige Unkenntlichmachung) gelten kann. Zweitens, ob es technisch tatsächlich wirkt und sich nachweisen lässt. Die Rechtsfrage wird durch die Technikfrage bestimmt, weil Löschung nach dem behördlichen Maßstab erst vollzogen ist, wenn keine lesbare Kopie beim Verantwortlichen oder seinen Auftragsverarbeitern mehr besteht.

## 2. Löschpflicht und Löschbegriff (Art. 17 DSGVO)

### 2.1 Pflicht und Auslöser

Art. 17 Abs. 1 DSGVO verpflichtet den Verantwortlichen, personenbezogene Daten unverzüglich zu löschen, wenn einer von sechs Gründen vorliegt: Wegfall des Zwecks (lit. a), Widerruf der Einwilligung ohne andere Rechtsgrundlage (lit. b), Widerspruch (lit. c), unrechtmäßige Verarbeitung (lit. d), Löschung zur Erfüllung einer rechtlichen Verpflichtung (lit. e) und Dienste der Informationsgesellschaft gegenüber Kindern nach Art. 8 (lit. f). Die Pflicht besteht auch ohne Antrag der betroffenen Person. Auf Anträge ist nach Art. 12 Abs. 3 DSGVO grundsätzlich binnen eines Monats zu reagieren.

Hat der Verantwortliche Daten öffentlich gemacht oder an Empfänger weitergegeben, kommen die Informationspflichten nach Art. 17 Abs. 2 und Art. 19 DSGVO hinzu. Der EuGH verlangt dazu angemessene Maßnahmen, um andere Verantwortliche über das Löschverlangen zu informieren (Proximus, C-129/21). Für RAG-Systeme ist das relevant, wenn Quelldokumente öffentlich zugänglich waren oder Chunks an externe Dienste übermittelt wurden.

### 2.2 Kein gesetzlicher Löschbegriff, keine vorgeschriebene Methode

Die DSGVO definiert „Löschung“ nicht. Art. 4 Nr. 2 DSGVO führt „Löschen“ und „Vernichtung“ nebeneinander als Formen der Verarbeitung auf. Daraus folgt zweierlei. Erstens ist die Löschung selbst eine Verarbeitung und unterliegt den Grundsätzen des Art. 5 sowie der Sicherheit nach Art. 32 DSGVO. Zweitens wird das Nebeneinander beider Begriffe so gelesen, dass Löschen nicht zwingend physische Zerstörung des Datenträgers bedeutet; so hat es etwa die österreichische Datenschutzbehörde in einem Bescheid vom 05.12.2018 vertreten (nach Sekundärbericht). Die herrschende Auffassung geht von Technikneutralität aus: Eine bestimmte Methode ist nicht vorgeschrieben.

Das Oberlandesgericht München hat mit Urteil vom 22.03.2022 (18 U 1697/21 Pre) angenommen, dass der Löschbegriff autonom auszulegen ist und der Löschanspruch einschließt, dass gelöschte Daten nicht erneut verarbeitet werden (nach Sekundärbericht).

**Das Standard-Datenschutzmodell, Baustein 60 „Löschen und Vernichten“ (Version 1.0a, gültig seit 02.09.2020), konkretisiert den Begriff als Behördenauslegung:**

- Löschen bedeutet Unkenntlichmachen gespeicherter personenbezogener Daten. Danach darf keine Person mehr identifizierbar sein, und der Informationsgehalt darf nicht oder nach allgemeinem Ermessen nur unwahrscheinlich reproduzierbar sein (Erwägungsgrund 26 DSGVO).
- Die Löschung ist erst vollzogen, wenn auch keine Kopie oder Replikation mehr beim Verantwortlichen oder Auftragsverarbeiter besteht.
- „Vernichten“ meint die Zerstörung des Datenträgers und ist eine unwiderrufliche Form des Löschens.
- Als abgestufte Methoden nennt das Modell Datenbank-Löschbefehle mit anschließender Reorganisation und Überschreiben, Wipe-Tools, das Überschreiben ganzer Datenträger unter Beachtung von Wear-Leveling sowie die physische Zerstörung.
- Nicht ausreichend sind danach einfache Betriebssystem-Löschbefehle, Schnellformatierung und bloße Nutzungsverbote.
- Kryptografisches Löschen wird als Methode nicht ausdrücklich genannt. Das Modell verlangt aber Regeln für den Umgang mit Schlüsseln verschlüsselter Daten und für das Löschen dieser Schlüssel.

Das Modell ist keine Rechtsquelle, sondern eine Auslegungshilfe der Aufsichtsbehörden. Ob für Baustein 60 inzwischen eine neuere Fassung im aktuellen Modell (Versionen 3.0 und 3.1 bestehen) vorliegt und ob sie den Inhalt ändert, ist vor Verwendung zu prüfen.

### 2.3 Umstritten: Kann Anonymisierung die Löschpflicht erfüllen?

- **Dafür.** Das Oberlandesgericht Dresden hat im Urteil vom 03.02.2026 (4 U 292/25, Verfahren gegen Meta) die Anonymisierung als „Minus zur Löschung“ eingeordnet, das die Löschpflicht erfüllen kann, wenn die betroffene Person die Entfernung ihres Personenbezugs verlangt (Leitsatz nach Sekundärbericht). Das Landgericht München II hat mit Urteil vom 14.03.2025 (14 O 1461/24) die Anonymisierung als zulässigen Unterfall der Löschung behandelt (Leitsatz). Ebenso vertreten es das Positionspapier des Bundesbeauftragten zur Anonymisierung (2020) und die österreichische Datenschutzbehörde. Ein Urteil des BGH oder des EuGH dazu ist nicht bekannt.
- **Vorbehalt der Aufsicht.** Der EDPB kritisiert im Bericht zur koordinierten Prüfung 2025 zum Recht auf Löschung (angenommen am 10.02.2026), dass Verantwortliche in der Praxis häufig bloße Pseudonymisierung, Verfremdung oder Maskierung als Löschung ausgeben. Verlangt wird der Nachweis, dass eine Re-Identifizierung nach allgemeinem Ermessen unwahrscheinlich ist. Leitlinien des EDPB zur Anonymisierung sind in Vorbereitung, auch im Anschluss an die EuGH-Entscheidung zum relativen Personenbezug (EDPS/SRB, C-413/23 P).
- **Wichtige Abgrenzung.** Das Urteil des OLG Dresden vom 14.12.2021 (4 U 1278/21) betrifft eine Inkasso-Streitigkeit mit einer namensgleichen Person und das Zusammenwirken von Aufbewahrungspflichten und Löschanspruch. Es enthält die „Minus“-Formel nicht und sagt nichts zu Crypto-Shredding.

### 2.4 Speicherbegrenzung, Datenminimierung, Privacy by Design

Art. 5 Abs. 1 lit. e DSGVO erlaubt die identifizierbare Speicherung nur so lange, wie es der Zweck erfordert. Art. 5 Abs. 1 lit. c verlangt die Beschränkung auf das notwendige Maß. Art. 25 Abs. 2 Satz 2 DSGVO bezieht die Speicherfrist ausdrücklich in die datenschutzfreundlichen Voreinstellungen ein. Für RAG-Systeme folgt daraus:

- Datenminimierung beginnt vor der Indexierung. Nicht erforderliche personenbezogene Daten sind bei der Datenaufbereitung zu entfernen oder zu anonymisieren. Die Auswahl der in die Vektordatenbank übernommenen Dokumente ist nach der DSK-Orientierungshilfe zu RAG der zentrale Hebel. Das reduziert die spätere Löschlast.
- RAG verringert den Umfang personenbezogener Daten im Sprachmodell selbst nicht.
- Technische Unmöglichkeit rechtfertigt nach den EDPB-Leitlinien 02/2025 keine Verstöße (mit Verweis auf Art. 25 Abs. 1 und den Bericht der ChatGPT-Taskforce vom Mai 2024). Die Aussage steht im Blockchain-Kontext, ist aber allgemein formuliert. Eine Architektur, die granulare Löschung konstruktiv verhindert, ist deshalb kaum zu verteidigen.

### 2.5 Einschränkung statt Löschung

Art. 4 Nr. 3 DSGVO definiert die Einschränkung der Verarbeitung als Markierung gespeicherter Daten mit dem Ziel, ihre künftige Verarbeitung zu beschränken (Art. 18). Ein Schlüssel, der nur gesperrt, aber nicht vernichtet wird, ist deshalb Einschränkung und keine Löschung. Dieselbe Unterscheidung gilt für Dokumente, die aus dem Suchindex genommen, aber im Archiv gehalten werden. Das ist bei Aufbewahrungspflichten das richtige Mittel, darf aber nicht als Löschung bezeichnet werden.

## 3. Kryptografisches Löschen (Crypto-Shredding)

### 3.1 Verfahren und Wirkprinzip

Beim Crypto-Shredding werden Daten von Anfang an verschlüsselt abgelegt. Die Beseitigung erfolgt nicht durch Überschreiben des Chiffretexts, sondern durch gezielte, unwiederbringliche Vernichtung des zugehörigen Entschlüsselungsschlüssels (Data Encryption Key, DEK). Der Chiffretext bleibt physisch vorhanden, ist aber ohne Schlüssel nicht mehr lesbar. Das Verfahren löst ein Kernproblem unveränderlicher und flashbasierter Speicher: Es genügt, ein kleines, kontrollierbares Objekt (den Schlüssel) zu vernichten, statt jede verstreute Kopie der Daten zu finden. Voraussetzung ist, dass jede Datenkopie unter genau diesem Schlüssel steht.

### 3.2 Rechtlicher Ausgangspunkt (verbindliches Recht)

- Verschlüsselung ist eine Sicherheitsmaßnahme (Art. 32 Abs. 1 lit. a DSGVO) und kann zugleich Pseudonymisierung im Sinne des Art. 4 Nr. 5 DSGVO sein. Pseudonymisierte Daten bleiben personenbezogen, solange Zusatzinformationen (hier der Schlüssel) existieren.
- Erwägungsgrund 26 stellt auf Mittel ab, die „nach allgemeinem Ermessen wahrscheinlich“ zur Identifizierung genutzt werden, unter Berücksichtigung von Kosten, Zeitaufwand, verfügbarer Technologie und technologischer Entwicklung.
- Der EuGH folgt einem relativen Ansatz: Ob Daten personenbezogen sind, beurteilt sich aus Sicht der Stelle, die über vernünftigerweise einsetzbare Mittel zur Re-Identifizierung verfügt (Breyer, C-582/14; EDPS/SRB, C-413/23 P vom 04.09.2025; Volltext vor Verwendung prüfen). Für den Verantwortlichen, der den Schlüssel hält, bleiben die Daten personenbezogen.
- **Argumentationslinie (Literatur, keine gesicherte Rechtslage):** Sind sämtliche Schlüsselkopien unwiederbringlich vernichtet, kann der verbleibende Chiffretext für jede Stelle ohne vernünftigerweise einsetzbare Mittel nicht mehr identifizierbar sein. Ob das rechtlich als „Löschung“ oder als „Anonymisierung“ zu führen ist, ist offen. Beide Wege laufen auf dieselbe Wirksamkeitsprüfung hinaus.

### 3.3 Aussagen der Aufsichtsbehörden

| Quelle | Aussage | Reichweite |
| --- | --- | --- |
| EDPB, Leitlinien 02/2025 zur Verarbeitung personenbezogener Daten in Blockchains (Entwurf April 2025, Endfassung 07.07.2026) | Nach Vernichtung des Schlüssels sind verschlüsselte Daten nur unverständlich, bis der Algorithmus gebrochen wird, Entschlüsselungstechniken fortschreiten oder der Schlüssel zuvor kompromittiert oder kopiert wurde. Verschlüsselte personenbezogene Daten bleiben personenbezogen. Selbst perfekt implementierte Verschlüsselung wird bei unbegrenzter Speicherung von der Zeit überholt. Dieselbe Logik gilt für Hashes mit geheimem Schlüssel oder Salt. Löschung muss wirksam sein und kann je nach Methode und Fakten künftige Identifizierung verhindern. Verlangt werden ein Notfallplan für den Algorithmenwechsel, eine Risikobewertung unter Einbeziehung von Obsoleszenz und Quantencomputern mit periodischer Neubewertung sowie Erasure by Design. Verschlüsselte oder gehashte Daten sollen nicht auf die Blockchain. | Einzige gefundene EDPB-Aussage zur Schlüsselvernichtung. Für Blockchains formuliert; die Übertragung auf RAG ist eine Analogie. |
| EDPB, Leitlinien 01/2025 zur Pseudonymisierung | Pseudonymisierte Daten bleiben personenbezogen, solange sie der betroffenen Person zugeordnet werden können. Ein klarer Test für wirksame Pseudonymisierung fehlt, was kritisiert wird. Endfassung nicht verifiziert. | Keine ausdrückliche Aussage, dass Schlüsselvernichtung eine Löschung ist. |
| EDPB, Bericht zur koordinierten Prüfung 2025 (Recht auf Löschung, 10.02.2026) | Pseudonymisierung und Maskierung genügen nicht als Löschung; vollständige Anonymisierung kann genügen. Löschung ist zu verifizieren und nachzuweisen. | Gilt für Anonymisierungsbehauptungen allgemein. |
| DSK und Landesbehörden | Keine ausdrückliche Aussage zu Crypto-Shredding gefunden. Das SDM verlangt Regeln zu Schlüsseln und deren Löschung. Die DSK-Orientierungshilfe zu RAG erwähnt Crypto-Shredding nicht. | Offene Lücke. |
| ICO (Vereinigtes Königreich) | Für Backups reicht „beyond use“, bis das Überschreiben möglich ist. Der Leitfaden steht nach dem Data (Use and Access) Act vom 19.06.2025 unter Überprüfung. Keine verifizierte Aussage zu Crypto-Shredding. | UK-Recht, nur vergleichend. |
| CNIL | Keine belegte Aussage gefunden. | Offen. |

### 3.4 Technische Normen und Standards (Literatur und Praxis)

| Standard | Befund |
| --- | --- |
| NIST SP 800-88 Rev. 2 (26.09.2025, löst Rev. 1 ab) | Cryptographic Erase gilt als Purge-Methode unter strengen Voraussetzungen. Der Schwerpunkt verlagert sich auf ein Sanitisierungsprogramm mit Verifikation und Validierung; gerätespezifische Verfahren regelt IEEE 2883:2022. Die Voraussetzungen im Einzelnen sind vor Verwendung zu prüfen. |
| ISO/IEC 27040 | Behandelt Speichersicherheit einschließlich Sanitisierung. Dass die Norm Schlüsselvernichtung ausdrücklich „akzeptiert“, ist nicht belegt. |
| ISO/IEC 27001:2022, Maßnahmen A.8.10 und A.8.24 | Allgemeine Kontrollen zur Löschung von Informationen und zur Kryptografie, ohne konkrete Aussage zu Crypto-Shredding. |
| BSI IT-Grundschutz, Baustein CON.6 | Fordert die Auswahl geeigneter Lösch- und Vernichtungsverfahren. Fassung prüfen. |
| BSI TR-02102 | Regelt kryptografische Verfahren und Schlüssellängen, nicht das Löschen. Maßgeblich für die Algorithmenwahl, nicht als Löschempfehlung. |
| DIN 66399 | Vernichtung von Datenträgern. Das SDM hält für personenbezogene Daten Sicherheitsstufe 4 oder höher für geeignet. Keine Aussage zu Crypto-Shredding. |
| DIN 66398, ISO/IEC 27555 | Entwicklung von Löschkonzepten mit Löschregeln, Löschfristen und Startzeitpunkten. |

Technische Normen sind Stand-der-Technik-Maßstäbe im Rahmen des Art. 32 DSGVO, aber keine DSGVO-Rechtsquellen. Sie erkennen Cryptographic Erase technisch an. Die rechtliche Wirkung als Löschung im Sinne des Art. 17 DSGVO folgt daraus nicht automatisch.

### 3.5 Bedingungen, unter denen die Argumentation trägt

Schlüsselvernichtung kann nur dann als Unkenntlichmachung vertreten werden, wenn Folgendes geprüft und dokumentiert ist:

1. **Schlüsselgranularität.** Der Schlüssel muss genau die zu löschende Einheit abdecken (Betroffener, Dokument oder Wissensbasis). Ein Schlüssel pro Mandant oder pro Sammlung führt entweder zu Kollateralverlust fremder Daten oder dazu, dass die Löschung unterbleibt.
2. **Vernichtung aller Schlüsselkopien.** Zu erfassen sind Schlüsselspeicher, KMS, HSM, Sicherungen des Schlüsselspeichers, Escrow- und Recovery-Kopien, Swap-Dateien, Crash- und Speicherabbilder, Protokolle und Exporte. Ein append-only Schlüsselprotokoll wäre eine verbleibende Schlüsselkopie und ist unzulässig.
3. **Kein unverschlüsselter Parallelbestand.** Embeddings, Volltextindex, Caches, Protokolle, temporäre Dateien und Kontexte bei externen LLM-Anbietern dürfen nicht im Klartext fortbestehen, es sei denn, sie werden gesondert und nachweisbar entfernt.
4. **Metadaten ohne Re-Identifizierung.** Verbleibende Dokument-IDs, Größen, Zeitstempel, Schlüssel-IDs und Suchstatistiken dürfen den Personenbezug nicht wiederherstellen. Chiffretexte fester Länge und unveränderte IDs ermöglichen Längenanalysen und Membership-Inference-Angriffe.
5. **Stand der Technik und Algorithmenwechsel.** Der Algorithmus muss dem Stand der Technik entsprechen (Art. 32 Abs. 1 lit. a DSGVO; Algorithmenwahl nach BSI-Empfehlungen). Erforderlich sind Risikobewertung zur Obsoleszenz, periodische Neubewertung und ein Verfahren zum Algorithmenwechsel.
6. **Passphrase-basierte Schlüsselableitung.** Wird der Wurzelschlüssel aus einer Nutzer-Passphrase abgeleitet, hängt die Sicherheit an Ableitungsfunktion und Passphrasenstärke. Ein Brute-Force-Angriff auf das Chiffrat muss praktisch ausgeschlossen sein.
7. **Echte Vernichtung.** Das Schlüsselmaterial muss überschrieben und ein Persistieren auf Datenträgern ausgeschlossen werden. Das bloße Entfernen eines Verzeichniseintrags genügt nicht.
8. **Wartefristen bei Cloud-Schlüsselverwaltung.** Cloud-Dienste verzögern die endgültige Schlüsselvernichtung häufig. Fachliche Angaben nennen für AWS KMS 7 bis 30 Tage, für Azure Key Vault (Soft Delete) 7 bis 90 Tage und für Google Cloud KMS standardmäßig 30 Tage. Das ist nur ein Indiz und anhand der Herstellerdokumentation zu prüfen. In dieser Zeit ist die Löschwirkung nicht gegeben und muss als solche dokumentiert werden.
9. **Keine Kompromittierung vor der Vernichtung.** Wurde der Schlüssel exportiert, kopiert oder in einem unkontrollierten System genutzt, trägt die Argumentation nicht.
10. **Nachweisbarkeit.** Schlüssellebenszyklus und Vernichtung müssen belegbar sein.

### 3.6 Rechtliche Würdigung

- Es gibt kein Gerichtsurteil und keine Behördenaussage, die Crypto-Shredding als ausreichende Löschung anerkennt. Behauptungen, Gerichte sähen den Chiffretext als hinreichend anonymisiert an, sind nicht belegt und beruhen auf einer Verwechslung der unter 2.3 genannten Entscheidungen.
- Der EDPB-Vorbehalt („bis“ der Algorithmus gebrochen wird) zeigt das Restrisiko: Gelöscht im Sinne der Unlesbarkeit ist nur, was nach dem Stand von Kryptografie und Rechenleistung unlesbar bleibt. Daraus folgen Dokumentation der Restrisiken und eine periodische Neubewertung, nicht aber ein Verbot des Verfahrens.
- Crypto-Shredding ist für persistente, unveränderliche Speicher (WAL, SSTables, Flash, Backups) das sachgerechte Mittel, weil physisches Überschreiben dort unzuverlässig ist. Für unverschlüsselte Klartext-Strukturen im Arbeitsspeicher und in aktiven Indizes wirkt es nicht. Diese sind physisch zu bereinigen.
- Ein gesperrter, aber nicht vernichteter Schlüssel ist Einschränkung der Verarbeitung, keine Löschung.
- Produktkommunikation muss Verfahren, Voraussetzungen und Grenzen benennen und darf keine absolute Löschwirkung versprechen.
