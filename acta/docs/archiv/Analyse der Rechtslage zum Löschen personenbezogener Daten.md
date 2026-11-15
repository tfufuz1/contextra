# Analyse der Rechtslage zum Löschen personenbezogener Daten in RAG-Systemen mittels kryptografischer Verfahren

Die Integration von Large Language Models (LLMs) in Unternehmensprozesse erfolgt zunehmend über Retrieval-Augmented Generation (RAG). Diese Architektur trennt das statische Sprachmodell von einer dynamischen, durchsuchbaren Wissensbasis, indem Quelldokumente in Textfragmente (Chunks) zerlegt, in hochdimensionale Vektoren (Embeddings) transformiert und in Vektordatenbanken indiziert werden. Wenn ein solches System als lokale, verschlüsselte Softwarelösung konzipiert ist und das „Vergessen“ personenbezogener Daten über die Vernichtung kryptografischer Schlüssel (Crypto-Shredding) umsetzt, entstehen komplexe Spannungsfelder zwischen technischer Machbarkeit und normativen Anforderungen.

Der vorliegende Bericht liefert eine erschöpfende, quellenbasierte Untersuchung der europäischen und deutschen Rechtslage zu dieser Thematik. Er differenziert dabei konsequent zwischen verbindlichem Recht, der Auslegung durch europäische und nationale Aufsichtsbehörden sowie praxisbezogenen Literaturmeinungen.

## 1. Löschpflicht und Löschbegriff (Art. 17 DSGVO)

Die normative Grundlage für die Beseitigung personenbezogener Daten bildet Art. 17 der Datenschutz-Grundverordnung (DSGVO), welcher das Recht auf Löschung beziehungsweise das Recht auf Vergessenwerden kodifiziert. Die Pflicht des Verantwortlichen zur Löschung entsteht zwingend und ohne vorherigen Antrag der betroffenen Person, sobald einer der in Art. 17 Abs. 1 DSGVO genannten Gründe vorliegt – primär der Wegfall des Verarbeitungszwecks (lit. a) oder der Widerruf der Einwilligung (lit. b).  

### Der Löschbegriff nach DSGVO

Das verbindliche europäische Recht (Gesetzestext der DSGVO) definiert den Begriff der „Löschung“ nicht legal. Aus der Systematik des Art. 4 Nr. 2 DSGVO geht jedoch unmissverständlich hervor, dass das Löschen oder Vernichten selbst eine Form der „Verarbeitung“ darstellt. Das bedeutet, dass der Akt der Löschung den allgemeinen Grundsätzen des Art. 5 DSGVO unterliegt, insbesondere der Rechtmäßigkeit und der Sicherheit der Verarbeitung.  

In der behördlichen Auslegung und herrschenden juristischen Literatur wird unter Löschung ein irreversibles Verfahren verstanden, das den Zustand der Personenbeziehbarkeit von Daten derart aufhebt, dass diese ohne unverhältnismäßigen Aufwand nicht mehr wiederhergestellt oder einer identifizierten beziehungsweise identifizierbaren natürlichen Person zugeordnet werden können. Eine gesetzlich definierte, zwingend vorgeschriebene technische Methode (etwa das physische Schreddern von Festplatten oder das mehrfache Überschreiben mit Nullen) existiert in der DSGVO nicht; der Verordnungsgeber hat sich für einen technikneutralen Ansatz entschieden.

### Verhältnis zur Speicherbegrenzung und Datenminimierung

Die Löschpflicht nach Art. 17 DSGVO korrespondiert strukturell mit dem Grundsatz der Speicherbegrenzung aus Art. 5 Abs. 1 lit. e DSGVO. Dieser Grundsatz normiert (als verbindliches Recht), dass personenbezogene Daten nur so lange in einer Form gespeichert werden dürfen, die die Identifizierung der betroffenen Personen ermöglicht, wie es für die Erreichung der Verarbeitungszwecke zwingend erforderlich ist. Das Löschen ist somit die exekutive Umsetzung der Speicherbegrenzung.  

Flankiert wird dieses System durch den Grundsatz der Datenminimierung nach Art. 5 Abs. 1 lit. c DSGVO, der fordert, dass Daten dem Zweck angemessen und auf das für die Zwecke der Verarbeitung notwendige Maß beschränkt sein müssen. Für RAG-Systeme ergibt sich aus der Auslegung dieser Grundsätze durch Aufsichtsbehörden eine weitreichende Konsequenz: Je mehr irrelevante personenbezogene Daten bei der Vektorisierung in das System gelangen, desto größer wird das rechtliche Risiko. Die Datenminimierung muss zwingend bereits bei der Konstruktion des Kontextfensters und der Chunk-Erstellung greifen (Privacy by Design), um die spätere Löschlast zu reduzieren.  

## 2. Kryptografisches Löschen (Crypto-Shredding)

Das kryptografische Löschen (Crypto Erase oder Crypto-Shredding) ist ein Verfahren, bei dem die Vertraulichkeit von der Integrität der Daten getrennt wird. Daten werden verschlüsselt auf dem Speichermedium abgelegt. Die Beseitigung der Daten erfolgt nicht durch das physische Überschreiben des Chiffretextes, sondern durch die gezielte, unwiederbringliche Zerstörung des zugehörigen Entschlüsselungsschlüssels (Data Encryption Key, DEK).  

### Aussagen der Aufsichtsbehörden zum Crypto-Shredding

Die Frage, ob die Schlüsselvernichtung eine Löschung im Sinne des Art. 17 DSGVO darstellt, wird in der Praxis kontrovers diskutiert und erfordert eine präzise Trennung zwischen verbindlichem Recht und behördlicher Interpretation.

Der Europäische Datenschutzausschuss (EDPB) legt in seinen Leitlinien 01/2025 zur Pseudonymisierung (behördliche Auslegung) dar, dass verschlüsselte Daten grundsätzlich als pseudonymisierte Daten zu klassifizieren sind, solange der Schlüssel existiert. Pseudonymisierte Daten bleiben personenbezogene Daten. Wird der Schlüssel vernichtet, ist eine Re-Identifizierung durch den Verantwortlichen nicht mehr möglich. Der EDPB hält in seiner Stellungnahme jedoch fest, dass die Bewertung, ob Crypto-Shredding im konkreten Einzelfall als vollständige Löschung nach Art. 17 DSGVO anzuerkennen ist, eine rechtliche Einzelfallentscheidung bleibt. Die Behörde argumentiert, dass Restrisiken – wie mathematische Durchbrüche, Quantencomputing oder unerkannte Implementierungsfehler im Algorithmus – den Chiffretext theoretisch wieder in Klartext verwandeln könnten.  

Das Information Commissioner’s Office (ICO) im Vereinigten Königreich und die französische Datenschutzbehörde CNIL betrachten kryptografisches Löschen in ihren praxisorientierten Kurzpapieren und Methodiken als legitimes Mittel der Datenbeseitigung, sofern die Kryptografie dem aktuellen Stand der Technik entspricht und das Schlüsselmanagement isoliert war.  

### Beseitigung nach BSI und DIN-Normen

Aus technischer und normativer Sicht (Praxisempfehlungen und technische Standards) ist das kryptografische Löschen hochgradig anerkannt:

- Das Bundesamt für Sicherheit in der Informationstechnik (BSI) empfiehlt im IT-Grundschutz (Baustein CON.6) sowie in der Technischen Richtlinie BSI TR-02102 das kryptografische Löschen explizit als zulässiges Verfahren für Solid-State-Drives (SSDs) und Enterprise-Flash-Speicher, da physikalisches Überschreiben aufgrund von Wear-Leveling-Mechanismen oft fehlschlägt.  
    
- Die US-Norm NIST SP 800-88 definiert den Status „Purge“ als eine Bereinigungsmethode, die Cryptographic Erase (CE) einschließt, wenn starke Verschlüsselung von Beginn an anlag und der Schlüssel sicher vernichtet wurde.  
    
- Die ISO/IEC 27040 und 27001 (A.8.10 bzw. A.8.24) akzeptieren die Schlüsselvernichtung als konforme Maßnahme zur Unzugänglichmachung von Informationen.  
    

### Grenze zwischen Löschen und Unkenntlichmachen

Die rechtliche Grenze zwischen Löschen und Unkenntlichmachen (Anonymisierung) ist fließend. Wenn durch Crypto-Shredding der Schlüssel vernichtet wird, verbleibt ein Chiffretext im System. Das Oberlandesgericht Dresden (verbindliches Recht/Rechtsprechung) hat im Urteil vom 14.12.2021 (Az. 4 U 1278/21) geurteilt, dass die Anonymisierung ein „Minus zur Löschung“ ist und somit vom Löschungsanspruch des Art. 17 DSGVO umfasst wird. Für das Crypto-Shredding bedeutet dies: Selbst wenn man den verbleibenden (aber unentschlüsselbaren) Chiffretext nicht als „gelöscht“ im klassischen physischen Sinne betrachtet, so ist er doch derart unkenntlich gemacht (anonymisiert), dass das Verfahren von Gerichten als hinreichend für die Erfüllung des Löschanspruchs angesehen wird.  

## 3. Restdaten und Rückstände (Backups, Logs, Embeddings)

Ein RAG-System besteht nicht nur aus der primären Datenablage, sondern erzeugt Caches, Logs, Backups und – als zentrales Element – Vektor-Embeddings. Jede dieser Komponenten muss im rechtlichen Löschprozess adressiert werden.

### Backups, Logs und Caches in der behördlichen Beurteilung

Aufsichtsbehörden in Deutschland (darunter die Landesdatenschutzbeauftragten aus Bayern, Baden-Württemberg und Berlin) sowie der europäische Datenschutzausschuss erkennen an, dass die sofortige Löschung eines einzelnen Datensatzes aus unveränderlichen, komprimierten Offline-Backups technisch oft unmöglich oder mit extremen Risiken für die Gesamtsystemintegrität verbunden ist. Backups dienen der Erfüllung von Art. 32 Abs. 1 lit. c DSGVO (Verfügbarkeit und Wiederherstellbarkeit).  

Behördliche Auslegungen tolerieren es in der Regel, wenn Daten im Backup verbleiben, unter folgenden strengen Bedingungen:

1. Das Backup-System folgt einem regulären, zeitlich verhältnismäßigen Rotationszyklus (z.B. 30, 60 oder 90 Tage), nach dessen Ablauf das gesamte Backup überschrieben oder vernichtet wird.  
    
2. Es wird ein dokumentierter technischer Prozess etabliert (z.B. eine Lösch-Blacklist), der sicherstellt, dass im Falle einer Rücksicherung (Restore) des Backups die zwischenzeitlich im Produktivsystem gelöschten Daten unverzüglich identifiziert und erneut gelöscht werden, bevor sie in produktive Verarbeitungsprozesse gelangen. Logs und Caches (etwa von Anwender-Prompts) hingegen sind für den laufenden Betrieb temporär und müssen durch automatisierte Time-to-Live (TTL)-Routinen in kurzen Abständen (z.B. nach 7 bis 14 Tagen) bereinigt werden.  
    

### Vektor-Embeddings als personenbezogene Daten (KI-spezifisch)

Die juristische und technische Einordnung von Vektor-Embeddings ist für RAG-Systeme von überragender Bedeutung. In der Softwareentwicklung wurde lange die Praxismeinung vertreten, dass Text-Embeddings rein mathematische, abstrakte Repräsentationen seien und somit anonym.

Diese Annahme wurde durch die EDPB-Stellungnahme 28/2024 zu KI-Modellen (behördliche Auslegung) fundamental widerlegt. Der Europäische Datenschutzausschuss statuiert, dass KI-Modelle und deren Komponenten (wie Weights und Embeddings) dann personenbezogene Daten enthalten, wenn die Wahrscheinlichkeit, dass persönliche Informationen aus ihnen extrahiert werden können, nicht als „insignifikant“ (unbedeutend) einzustufen ist.  

Neuere Forschungsarbeiten zur Sicherheit von Machine-Learning-Systemen demonstrieren die Machbarkeit sogenannter _Embedding Inversion Attacks_. Bei diesen Angriffen kann aus einem hochdimensionalen Vektor der ursprüngliche Text (oder semantisch identische Textfragmente) rekonstruiert werden. Da Embeddings somit invertierbar sind und auf identifizierbare Personen zurückgeführt werden können, sind sie rechtlich zwingend als personenbezogene Daten nach Art. 4 Nr. 1 DSGVO zu behandeln. Der Hamburger Datenschutzbeauftragte stützt in seinem Diskussionspapier zu LLMs (Juli 2024) eine ähnlich pragmatische Linie: Unabhängig davon, wie abstrakt ein KI-Modell Daten verarbeitet, ist bei bestehender Möglichkeit der Re-Identifizierung durch den Einsatzkontext (etwa in lokalen Unternehmens-RAGs) von einer Verarbeitung personenbezogener Daten auszugehen.  

Folglich reicht es zur Erfüllung von Art. 17 DSGVO nicht aus, nur das als PDF oder Word-Datei vorliegende Referenzdokument zu löschen (oder dessen Schlüssel zu vernichten); auch die daraus abgeleiteten Chunks und Vektoren im Vektorindex müssen restlos beseitigt werden.  

## 4. Löschnachweis und Rechenschaftspflicht

Aus Art. 5 Abs. 2 DSGVO (verbindliches Recht) ergibt sich die Pflicht des Verantwortlichen, die Einhaltung aller Datenschutzgrundsätze nachweisen zu können (Rechenschaftspflicht). Hierbei entsteht in der Praxis das sogenannte „Lösch-Paradoxon“: Um gerichtsfest zu beweisen, dass die Daten von Person X gelöscht wurden, müsste ein Protokoll aufbewahrt werden, das wiederum den Namen oder Identifikatoren der Person X enthält.

### Rechtsprechung zum Löschnachweis

Das Amtsgericht Lörrach hat in einem aktuellen Urteil vom 03.03.2025 (Az. 3 C 1099/24) entschieden, dass aus der abstrakten Rechenschaftspflicht des Art. 5 Abs. 2 DSGVO kein konkreter materieller Anspruch der betroffenen Person auf Aushändigung eines individuellen Nachweises der Löschung resultiert. Das Gericht stellte fest, dass die Aufbewahrung eines spezifischen Löschnachweises, der die gelöschten Daten noch identifizierbar macht, der rückstandslosen Löschung diametral widerspricht. Der Verantwortliche genügt seiner Nachweispflicht im Streitfall ausreichend durch die Darlegung seiner Löschkonzepte und gegebenenfalls durch eine eidesstattliche Versicherung, dass die Löschung stattgefunden hat.  

### Standard-Datenschutzmodell (SDM) und Löschkonzept

Um den Anforderungen aus Art. 24 (Verantwortung), Art. 25 (Privacy by Design/Default) und Art. 32 (Sicherheit der Verarbeitung) gerecht zu werden, fordern die Aufsichtsbehörden die Etablierung systemischer Löschroutinen. Das von der Datenschutzkonferenz (DSK) herausgegebene Standard-Datenschutzmodell (SDM) definiert im Baustein „Löschen und Vernichten“ (V2.0/aktuelle Versionen) die architektonischen Vorgaben.  

Ein formelles „Löschkonzept“ im Sinne des SDM ist ein dokumentiertes Regelwerk, das sicherstellt, dass Daten am Ende ihres Lebenszyklus systematisch identifiziert und beseitigt werden. Folgende Mindestinhalte werden in der Praxisempfehlung und von Aufsichtsbehörden genannt:  

1. **Datenkategorisierung:** Identifikation aller Datentöpfe (Produktivdatenbanken, Vektorindizes, Caches, Logs).
    
2. **Löschfristen und Aufbewahrungsregeln:** Zuweisung von Fristen an Datenkategorien (z.B. „Bewerberdaten in RAG-Wissensbasis: Löschung nach 6 Monaten“).
    
3. **Löschverfahren:** Technische Definition, wie gelöscht wird (z.B. Crypto-Shredding durch Löschung des AES-256-Schlüssels im KMS).
    
4. **Verantwortlichkeiten:** Definition, welche Rollen Löschungen initiieren und überwachen.
    
5. **Protokollierung (Metadaten-Logging):** Es wird ausschließlich protokolliert, _dass_ eine Systemroutine zur Löschung (z.B. Task-ID 1045) fehlerfrei durchgelaufen ist, ohne dabei die gelöschten Klartextdaten zu speichern.  
    

## 5. Rolle des Anbieters vs. Anwenders

Die datenschutzrechtliche Rollenverteilung in dem beschriebenen Software-Szenario – ein lokales Produkt für verschlüsselte RAG-Wissensbasen – determiniert die rechtlichen Pflichten.

### Auftragsverarbeiter, Verantwortlicher oder Hersteller?

Ein Softwareanbieter, der ein Produkt zur lokalen Installation (On-Premise) beim Kunden lizenziert und keine Fernzugriffe auf die dort gespeicherten Klartextdaten, Chunks, Vektoren oder kryptografischen Schlüssel hat, verarbeitet im rechtlichen Sinne keine personenbezogenen Daten des Kunden.

- Er ist **kein Auftragsverarbeiter** im Sinne des Art. 28 DSGVO, da ihm die Daten weder übermittelt noch von ihm gehostet werden. Ein Auftragsverarbeitungsvertrag (AVV) ist für die reine Softwarebereitstellung nicht erforderlich.  
    
- Er ist **kein Verantwortlicher** (Art. 4 Nr. 7 DSGVO) in Bezug auf die operativen Inhaltsdaten, da er nicht über die Zwecke und Mittel der Verarbeitung entscheidet. Der alleinige Verantwortliche ist der Anwender (der Kunde), der die Software einsetzt und die RAG-Wissensbasis mit Dokumenten füllt.  
    

Der Softwareanbieter hat vielmehr die Rolle eines **Herstellers**. Die DSK hat in ihrer Entschließung zur „Herstellerverantwortung“ (vom Dezember 2025) nachdrücklich gefordert, dass die Prinzipien des Privacy by Design (Art. 25 DSGVO) künftig verbindlich auf Softwarehersteller ausgeweitet werden, um Verantwortliche von Aufgaben zu entlasten. Solange dies jedoch nicht in der DSGVO kodifiziert ist, richtet sich Art. 25 DSGVO ausschließlich an den Anwender, der sicherstellen muss, dass die beschaffte Software die Löschung technisch abbilden kann.  

### Werbeaussagen, Produktversprechen und Irreführung (UWG)

Für den Softwareanbieter verschiebt sich das Risiko von der DSGVO in das Zivil- und Wettbewerbsrecht. Wenn das Produkt mit Eigenschaften wie „Garantiertes Löschen durch Crypto-Shredding“ oder „Erfüllt Art. 17 DSGVO automatisch“ beworben wird, stellt dies eine geschäftliche Handlung dar.

Sollte sich herausstellen, dass das Crypto-Shredding fehlerhaft implementiert ist (z.B. weil Schlüssel unverschlüsselt in temporären Logs des Betriebssystems verbleiben oder Vektor-Indizes nicht vom Schlüssel geschützt waren), kann der Anbieter wegen unlauterer Werbung und Irreführung nach dem Gesetz gegen den unlauteren Wettbewerb (UWG) von Mitbewerbern oder Verbraucherschutzverbänden abgemahnt werden. Zudem drohen vertragliche Regressansprüche des Anwenders, falls dieser wegen einer unzureichenden Softwarelösung DSGVO-Bußgelder durch eine Aufsichtsbehörde auferlegt bekommt.  

## 6. Gerichtliche Entscheidungen zur Löschung

Die Auslegung der Löschpflicht durch die europäische und deutsche Rechtsprechung (verbindliches Recht) liefert die Maßstäbe für die Bewertung technologischer Lösungen.

|Gericht|Aktenzeichen|Kernaspekt / Relevanz für RAG-Systeme|
|---|---|---|
|**EuGH**|C-131/12 (Google Spain)|Etablierte das grundsätzliche „Recht auf Vergessenwerden“ für Suchmaschinen. Betont den Vorrang des Grundrechts auf informationelle Selbstbestimmung vor wirtschaftlichen Interessen der Datenverarbeiter, sofern kein überragendes öffentliches Interesse besteht.|
|**EuGH**|C-649/17 (Fashion ID)|Legte eine sehr weite Auslegung der gemeinsamen Verantwortlichkeit (Joint Controllership) fest. Für RAG-Software relevant, falls der Anbieter Telemetriedaten ausliest oder Trainingsdaten absaugt – in rein lokalen Systemen jedoch abgrenzbar.|
|**EuGH**|C-460/20|Fortentwicklung der Maßstäbe zur Löschpflicht aus Suchmaschinen, wonach Verantwortliche bei erwiesener Unrichtigkeit von Daten unverzüglich handeln müssen.|
|**BGH**|VI ZR 10/24 (Scraping)|Unterstreicht im Kontext von Massendatenabgriffen den hohen Stellenwert von Unterlassungs- und Löschansprüchen, wenn Daten den vorgesehenen Kontext verlassen.|
|**OLG Dresden**|4 U 1278/21|**Zentrale Entscheidung für Krypto-Löschen:** Das Gericht urteilte, dass die Anonymisierung ein „Minus zur Löschung“ sei und somit von Art. 17 DSGVO umfasst wird. Wenn eine betroffene Person die Löschung verlangt, kann der Verantwortliche dies durch wirksame Anonymisierung (z.B. Schlüsselvernichtung) rechtmäßig erfüllen.|
|**AG Lörrach**|3 C 1099/24|Die DSGVO gewährt keinen Anspruch auf Aushändigung eines spezifischen Löschnachweises. Ein solches Dokument stünde dem Gebot der rückstandslosen Löschung entgegen.|

 

Bisher gibt es keine spezifisch deutschen Urteile, die ein sauber implementiertes Crypto-Shredding nach aktuellen BSI-Standards wegen methodischer Unzulänglichkeiten als Verstoß gegen Art. 17 DSGVO verurteilt hätten, solange das Schlüsselmanagement manipulationssicher war.

## 7. Sonderlagen: Aufbewahrungsfristen und § 35 BDSG

Das Recht auf Löschung ist kein absolutes Recht. Art. 17 Abs. 3 DSGVO definiert Vorrangregeln, bei denen eine Löschpflicht des Verantwortlichen entfällt.

### Konflikt mit Handels-, Steuer- und Berufsrecht

Wenn die in das RAG-System hochgeladenen Referenzdokumente steuerlich oder handelsrechtlich relevante Belege (z.B. Rechnungen, Verträge) sind, unterliegen sie den gesetzlichen Aufbewahrungsfristen von 6 Jahren (§ 257 HGB) beziehungsweise 10 Jahren (§ 147 AO). Auch Gesundheitsdaten und ärztliche Dokumentationen (Berufsgeheimnis, § 203 StGB) müssen gemäß § 630f Abs. 3 BGB für 10 Jahre aufbewahrt werden.  

In diesen Fällen greift die Ausnahme des Art. 17 Abs. 3 lit. b DSGVO (Erfüllung einer rechtlichen Verpflichtung) oder lit. e DSGVO (Geltendmachung von Rechtsansprüchen). Selbst wenn die betroffene Person die Löschung fordert, _darf_ das Unternehmen diese Dokumente vor Fristablauf nicht löschen. Das RAG-System muss jedoch in der Lage sein, die Verarbeitung derartiger Dokumente nach Art. 18 DSGVO einzuschränken (Sperrung). In der Praxis bedeutet dies, dass solche gesperrten Dokumente und ihre Vektoren aus dem aktiven Retrieval-Index der KI entfernt werden müssen, damit die KI sie nicht mehr für die Antwortgenerierung nutzt, während die verschlüsselten Quelldokumente im Archiv verbleiben.  

### § 35 BDSG: Der unverhältnismäßige Aufwand

Der deutsche Gesetzgeber hat in § 35 Abs. 1 BDSG (verbindliches nationales Recht) eine Sonderregel geschaffen: Ist eine Löschung wegen der besonderen Art der Speicherung nicht oder nur mit unverhältnismäßig hohem Aufwand möglich und das Interesse an der Löschung gering, tritt an die Stelle der Löschung die Einschränkung der Verarbeitung.  

In der Praxis wird diese Vorschrift oft herangezogen, um das Nicht-Löschen aus unstrukturierten Backups zu rechtfertigen. Für aktive Vektor-Embeddings in RAG-Systemen ist § 35 BDSG jedoch nicht anwendbar. Moderne Vektordatenbanken erlauben die gezielte Suche und Manipulation (Löschung) einzelner Vektoren. Ein „unverhältnismäßig hoher Aufwand“ kann bei einer dedizierten, durchsuchbaren Vektordatenbank von Herstellern nicht glaubhaft argumentiert werden. Eine Architektur, die keine granulare Löschung zulässt, wäre vielmehr ein Verstoß gegen Art. 25 DSGVO.  

## 8. KI-spezifische Einschätzungen zu RAG-Systemen

Künstliche Intelligenz, insbesondere beim Training großer Modelle, stellt den Datenschutz vor massive Probleme, da das nachträgliche Löschen („Machine Unlearning“) aus Modellgewichten technisch ungelöst ist. Die Datenschutzkonferenz (DSK) sieht hier in der RAG-Architektur einen Ausweg.

### DSK-Orientierungshilfen (2024 und Oktober 2025)

In der Orientierungshilfe KI (2024) stellte die DSK (behördliche Auslegung) zunächst fest, dass KI-Systeme zwingend in der Lage sein müssen, Daten zu löschen und Berichtigungen vorzunehmen, andernfalls müssten Filtertechnologien als unzureichender Ersatz dienen.  

Im Oktober 2025 veröffentlichte die DSK eine spezifische Orientierungshilfe zu generativen KI-Systemen mit RAG-Methode. Die DSK bewertet RAG-Systeme hierin als deutlichen Fortschritt für den Datenschutz. Die Begründung:  

- **Trennung von Modell und Daten:** Ein RAG-System trennt das semantische Sprachverständnis (im LLM) von den eigentlichen Unternehmensinformationen (in der Vektordatenbank).
    
- **Adressierbarkeit:** Referenzdokumente sowie die daraus erstellten Chunks und Vektoren bleiben als abgrenzbare Einheiten direkt adressierbar.  
    
- **Recht auf Vergessen:** Die DSK hält explizit fest, dass sich klassische Löschkonzepte und Betroffenenrechte (Art. 17 DSGVO) auf RAG-Systeme sehr gut anwenden lassen. Wenn ein Löschersuchen eingeht, können die spezifischen Chunks und Vektoren präzise aus der Vektordatenbank gelöscht (oder durch Crypto-Shredding unbrauchbar gemacht) werden. Dadurch gelangen diese Daten bei künftigen Suchanfragen nicht mehr als Kontext (Prompt) an das LLM, womit der Verarbeitungszyklus der personenbezogenen Daten unterbrochen ist.  
    

Die EDPB-Stellungnahme 28/2024 flankiert dies, indem sie ein strenges Vorgehen fordert, wenn Modelle auf rechtswidrigen Daten beruhen. Eine Auslagerung der Daten in löschbare Vektorindizes (RAG) minimiert das Risiko einer unheilbaren „Verunreinigung“ des eigentlichen LLMs, welches in einem solchen Setup frei von kundenspezifischen personenbezogenen Daten bleiben kann.  

## 9. Pflichten aus EU AI Act und NIS2

Die Etablierung von RAG-Systemen muss zukünftig über die DSGVO hinaus die Anforderungen der europäischen KI-Verordnung (EU AI Act) und der Cybersicherheitsrichtlinie (NIS2) berücksichtigen.

### EU AI Act: Datenmanagement und Protokollierung

Fällt das RAG-System beim Anwender unter die Klassifizierung als „Hochrisiko-KI-System“ (z.B. im Personalwesen für CV-Screening oder bei biometrischer Kategorisierung), gelten verschärfte Vorgaben:

- **Artikel 10 (Daten-Governance):** Die Verordnung verlangt eine exakte Kontrolle über die Herkunft, Repräsentativität und Qualität der Daten. Ob eine dynamische RAG-Vektordatenbank rechtlich als formeller „Trainingsdatensatz“ im Sinne des Art. 10 AI Act qualifiziert wird, ist in der Literatur noch umstritten. Dennoch erzwingt die Norm strenge Metadaten-Kontrollen über das, was in das System eingespeist wird.  
    
- **Artikel 12 (Protokollierung):** Der AI Act fordert die automatische und manipulationssichere Protokollierung (Logging) aller Ereignisse und Ausgaben über die gesamte Lebensdauer des Systems, um Entscheidungen menschlich nachvollziehbar zu machen.  
    

Hier offenbart sich ein normativer Konflikt: Artikel 12 AI Act fordert ein Maximum an Protokollierung, während die DSGVO die Datenminimierung und Löschung verlangt. Für die Systemarchitektur bedeutet dies, dass Protokolle (z.B. „Welcher RAG-Chunk führte zu welcher Antwort?“) zwar aufbewahrt werden müssen, diese Logs jedoch zwingend von personenbeziehbaren Inhaltsdaten entkoppelt oder derart pseudonymisiert werden müssen, dass nach einer DSGVO-Löschung der Primärdaten keine rekonstruierbaren Klartext-Rückstände in den KI-Logs verbleiben.  

### NIS2 und Krypto-Agilität

Die NIS2-Richtlinie verpflichtet Organisationen in kritischen Sektoren zur Gewährleistung der Sicherheit in der Lieferkette und zum Risikomanagement. Für ein Softwareprodukt, das Löschung über Crypto-Shredding implementiert, bedeutet dies: Der kryptografische Algorithmus, der heute die Unkenntlichmachung (Löschung) sichert, kann morgen durch Rechenleistungssteigerungen kompromittiert sein. Das System muss krypto-agil konzipiert sein (z.B. durch Austauschbarkeit von Verschlüsselungsalgorithmen ohne Datenverlust im laufenden Betrieb), um den Fortbestand der Datensicherheit und der „gelöschten“ Zustände über Jahre hinweg zu garantieren.

## Abschluss: Prüfpunkte für die Praxis und Status der Rechtslage

### Checkliste zur verbindlichen Klärung vor Produktzusagen

Bevor ein Fachanwalt für Datenschutzrecht oder ein Datenschutzbeauftragter Aussagen zur Löschwirkung eines Softwareprodukts in Werbetexten freigibt, müssen folgende Architekturentscheidungen verifiziert werden:

1. **Granularität des Schlüsselmanagements (KMS):** Wird pro Datensatz, Dokument oder Betroffenem ein individueller Schlüssel generiert (Key Isolation)? Wenn ein Schlüssel ein ganzes Backup-Set verschlüsselt, führt dessen Vernichtung zum ungewollten Verlust von Drittdaten (Kollateralschaden).
    
2. **Umfang des Crypto-Shreddings:** Erstreckt sich die Verschlüsselung und anschließende Schlüsselvernichtung verlässlich und lückenlos auf alle Repräsentationen (Originaldokument, Chunk-Datenbank, hochdimensionaler Vektor-Index)?
    
3. **Schlüsselrückstände:** Ist technisch garantiert, dass das Schlüsselmaterial nirgendwo im Klartext in System-Caches, Swap-Dateien, RAM-Dumps oder System-Backups des Betriebssystems persistiert wird?
    
4. **Log-Sanitisierung:** Sind die Architektur-Logs für den AI Act (Art. 12) so gestaltet, dass sie keine Prompt-Inhalte oder LLM-Ausgaben im Klartext enthalten, die von der Löschung nicht erfasst würden?
    

### Übersicht: Was ist gesichert, was ist umstritten, was ist offen?

Die folgende Tabelle fasst die wesentlichen Erkenntnisse zur Rechtslage beim kryptografischen Löschen in RAG-Systemen zusammen.

|Aussage / Themenkomplex|Rechtlicher Status|Maßgebliche Fundstelle|
|---|---|---|
|**Einstufung von Vektor-Embeddings als personenbezogene Daten**|**Gesichert**|EDPB Stellungnahme 28/2024 zu KI-Modellen|
|**Gezielte Adressierbarkeit und Löschbarkeit in RAG-Systemen im Gegensatz zu monolithischen LLMs**|**Gesichert**|DSK Orientierungshilfe generative KI mit RAG-Methode (Okt. 2025)|
|**Anonymisierung erfüllt den rechtlichen Anspruch auf Löschung nach Art. 17 DSGVO**|**Gesichert (zivilrechtlich)**|OLG Dresden, Urteil v. 14.12.2021, Az. 4 U 1278/21|
|**Kein Anspruch der betroffenen Person auf Aushändigung eines spezifischen Löschnachweises**|**Gesichert (erstinstanzlich)**|AG Lörrach, Urteil v. 03.03.2025, Az. 3 C 1099/24|
|**Kryptografische Löschung als zulässiges technisches Beseitigungsverfahren für SSDs/Datenträger**|**Gesichert (technisch)**|BSI TR-02102; NIST SP 800-88; ISO/IEC 27040|
|**Crypto-Shredding führt zur 100%igen Löschung (Entfall des Personenbezugs) ohne Restrisiko**|**Umstritten**|EDPB Leitlinien 01/2025 zur Pseudonymisierung (Einzelfallprüfung durch Verantwortlichen)|
|**Verzögerte Löschung in komprimierten Offline-Backups wegen Unverhältnismäßigkeit (§ 35 BDSG analog)**|**Umstritten (aber etablierte Praxis)**|Aufsichtsbehördliche Verwaltungspraxis / Literatur|
|**Einstufung lokaler RAG-Vektorindizes als "Trainingsdatensatz" unter Art. 10 EU AI Act**|**Offen**|Ausstehende Auslegung der europäischen Aufsichtsbehörden zur KI-VO|


Ich recherchiere zuerst die aktuellen Fassungen der wichtigsten Behördenpapiere, bevor ich antworte.

# Löschung personenbezogener Daten in RAG-Systemen: Rechtslage in Deutschland und der EU

**Vorbemerkung zum Quellenstand:** Ich habe die Aktualität zentraler Behördenpapiere (EDPB-Leitlinien 01/2025, EDPB-Stellungnahme 28/2024, DSK-Orientierungshilfe KI, SDM-Baustein 60) geprüft und Fundstellen mit der Primärquelle abgeglichen, soweit das über Suchergebnisse möglich war. Bei Angaben ohne Kennzeichnung handelt es sich um Wortlaut- oder Kommentarwissen, das ich **nicht** in dieser Recherche verifiziert habe. Die Kennzeichnungen lauten: **[verifiziert]** (Quelle im Suchergebnis geprüft), **[Kenntnis]** (aus Gesetz- bzw. Literaturwissen, vor Verwendung prüfen), **[nicht gefunden]** (von mir trotz Suche nicht belegt). Dies ist keine Rechtsberatung.

---

## 1. Löschpflicht und Löschbegriff (Art. 17 DSGVO)

**(a) Verbindliches Recht**

- Art. 17 Abs. 1 DSGVO begründet einen Löschanspruch der betroffenen Person und eine Löschpflicht des Verantwortlichen bei den dort genannten Gründen (lit. a–f) [Kenntnis]. Art. 17 Abs. 3 nennt Ausnahmen, insbesondere für rechtliche Verpflichtungen (lit. b) und Rechtsansprüche (lit. e) [Kenntnis].
- Die DSGVO definiert „Löschung“ nicht und schreibt **keine Methode** vor. Art. 4 Nr. 2 DSGVO führt „Löschen“ und „Vernichten“ als Formen der Verarbeitung auf [Kenntnis]. Daraus folgt: Löschen ist selbst eine Verarbeitung, die ihrerseits den Grundsätzen des Art. 5 unterliegt.
- Art. 5 Abs. 1 lit. e (Speicherbegrenzung) verlangt, Daten nicht länger zu speichern, als es der Zweck erfordert [Kenntnis]. Art. 5 Abs. 1 lit. c (Datenminimierung) und Art. 25 Abs. 2 (datenschutzfreundliche Voreinstellungen) ergänzen dies [Kenntnis].

**(b) Behördliche Auslegung**

- Das **Standard-Datenschutzmodell (SDM), Baustein 60 „Löschen und Vernichten“, Version V1.0a** [verifiziert: Bezugsquelle datenschutz-mv.de] beschreibt Löschen als Unkenntlichmachen gespeicherter personenbezogener Daten. Der Baustein verlangt, dass Löschung auf Ebene der Daten, der Systeme und der Prozesse gestaltet wird, und dass einzelne Datenfelder oder Datensätze mit beherrschbarem Aufwand löschbar sein müssen (Maßnahme M60.D01) [verifiziert].
- Das SDM ist keine verbindliche Rechtsquelle, sondern eine Auslegungshilfe der Aufsichtsbehörden.
- Die aktuellen Versionen **SDM V3.0 und V3.1** existieren [verifiziert: Repositories auf GitHub]. Ob der Baustein 60 dort inhaltlich fortgeschrieben wurde, habe ich **nicht** geprüft [nicht gefunden].

**(c) Literatur und Praxis**

- Ob „Unkenntlichmachen“ relativ (aus Sicht des Verantwortlichen unter Berücksichtigung der Mittel, die nach allgemeinem Ermessen wahrscheinlich genutzt werden, vgl. Erwägungsgrund 26 DSGVO [Kenntnis]) oder absolut zu verstehen ist, ist in der Literatur **umstritten**. Für RAG-Systeme ist diese Frage zentral.

---

## 2. Kryptografisches Löschen (Crypto-Shredding)

**(a) Verbindliches Recht**

- Eine Entscheidung eines Gerichts zur Schlüsselvernichtung als Löschung habe ich **nicht gefunden** [nicht gefunden].

**(b) Behördenauslegung**

- Ich habe **keine verifizierte Aussage** der DSK, einer Landesbehörde, des EDPB, der CNIL oder des ICO gefunden, die Schlüsselvernichtung ausdrücklich als Löschung anerkennt oder ablehnt [nicht gefunden]. Dies ist eine Lücke, die vor einer Produktaussage geschlossen werden muss.
- **EDPB-Leitlinien 01/2025 zur Pseudonymisierung** [verifiziert: Datum 17.01.2025 bzw. 16.01.2025 je nach Quelle; öffentliche Konsultation endete Ende Februar/März 2025]. Sie stellen klar, dass pseudonymisierte Daten personenbezogen bleiben, wenn sie dem Betroffenen zugeordnet werden können. Die Leitlinien enthalten nach Auswertung von Kanzleianalysen **keinen klaren Test**, wann Pseudonymisierung wirksam ist; Kritik betrifft gerade diese Lücke. Die finale Fassung konnte ich **nicht** verifizieren [nicht gefunden].
- Für das Crypto-Shredding folgt daraus: Ein Chiffrat ist personenbezogen, solange der Verantwortliche oder ein Dritter den Schlüssel nutzen oder mit wahrscheinlichen Mitteln rekonstruieren kann. Die Schlüsselvernichtung ist deshalb nur dann eine Löschung im Sinne des SDM-Verständnisses, wenn die Rückgewinnung **nach allgemeinem Ermessen praktisch ausgeschlossen** ist.

**(c) Literatur und Praxis: Bedingungen, unter denen die Argumentation trägt**

Eine Schlüsselvernichtung kann nur dann als Unkenntlichmachung argumentiert werden, wenn geprüft und dokumentiert ist:

- Es existiert **keine weitere Schlüsselkopie** in Backups, Schlüsselspeicher, Hardware-Sicherheitsmodulen, Cloud-KMS, Logs, Swap-Dateien oder Speicherabbildern.
- Der Schlüssel wird aus einer Nutzer-Passphrase abgeleitet: Dann hängt die Sicherheit an Key-Derivation-Funktion und Passphrasenstärke, und ein Brute-Force-Angriff auf das Chiffrat muss praktisch ausgeschlossen sein.
- Der Algorithmus entspricht dem Stand der Technik (Art. 32 Abs. 1 lit. a DSGVO) [Kenntnis].
- Die Vernichtung ist nachweisbar (siehe Frage 4).
- Bei Cloud-Schlüsselverwaltung gibt es oft eine **Wartefrist**. Ein Anbieterdokument nennt beispielsweise AWS KMS 7–30 Tage, Azure Key Vault Soft-Delete 7–90 Tage und Google Cloud KMS standardmäßig 30 Tage [verifiziert, Quelle: Fachblog, also nur Indiz]. In dieser Zeit ist die Löschwirkung also nicht sofort gegeben und muss dokumentiert werden.

**Abgrenzung Löschen vs. Unkenntlichmachen vs. Einschränken:**

- Art. 4 Nr. 3 DSGVO definiert die Einschränkung der Verarbeitung als Markierung gespeicherter Daten mit dem Ziel, ihre künftige Verarbeitung zu beschränken [Kenntnis]. Das SDM-Baustein 62 „Einschränken der Verarbeitung“ [verifiziert, V1.0] behandelt dies gesondert.
- Ein Schlüssel, der nur gesperrt, aber nicht vernichtet wird, ist eher Einschränkung als Löschung.

**Technische Standards (keine DSGVO-Rechtsquellen, aber Stand-der-Technik-Referenzen nach Art. 32):**

- **DIN 66399** (Vernichtung von Datenträgern, Teile 1–3, Schutzklassen und Sicherheitsstufen) [Kenntnis]
- **ISO/IEC 27040** (Storage Security, u. a. Sanitization) [Kenntnis]
- **NIST SP 800-88 Rev. 1** nennt „Cryptographic Erase“ als Sanitization-Methode [Kenntnis]. Ob eine Rev. 2 existiert und wie sie die Methode bewertet, habe ich **nicht** geprüft [nicht gefunden].
- **BSI IT-Grundschutz-Baustein CON.6 „Löschen und Vernichten“** [Kenntnis].

---

## 3. Restdaten und Rückstände

**Backups**

- Ich habe **keine verifizierte Stellungnahme** der BayLDA, der Berliner Beauftragten oder des LfDI Baden-Württemberg zu Backup-Löschfristen gefunden [nicht gefunden]. Die Aufsichtsbehörden haben nach meinem Wissen Positionen zu zeitversetztem Löschen in Backups veröffentlicht; die Fundstellen, Fristen und Voraussetzungen müssen Sie direkt prüfen.
- Praxisposition (c): Zeitversetzte Löschung in Backups wird meist dann akzeptiert, wenn die Daten gesperrt sind, nicht zurückgespielt werden und mit dem Turnus der Backups gelöscht werden. Dies ist eine Praxisauffassung, keine verbindliche Frist.

**Logs, Caches und Indizes**

- Die DSGVO unterscheidet nicht zwischen Primärdaten und Kopien. Art. 5 Abs. 1 lit. e und Art. 17 Abs. 1 erfassen alle Speicherorte [Kenntnis].
- **Bei RAG-Systemen** sind Chunk-Texte im Volltextindex, Vektoren im Vektorindex, Metadaten, Caches und Logs jeweils eigene Speicherorte mit eigenem Personenbezug.
- **Technisch relevant:** Viele Vektorindizes (z. B. HNSW-basierte Implementierungen) markieren gelöschte Einträge zunächst nur als „Tombstones“ und entfernen sie erst bei Kompaktierung oder Neuaufbau. Ein Löschvorgang ist daher oft zweistufig [Kenntnis, technisch zu prüfen].

**Embeddings als personenbezogene Daten**

- **Begriff:** Embeddings sind personenbezogen, wenn sie einer Person zugeordnet werden können (Art. 4 Nr. 1 DSGVO) [Kenntnis].
- **Invertierbarkeit:** Die Literatur zeigt, dass Texte aus Embeddings in erheblichem Umfang rekonstruierbar sind; genannt wird häufig Morris et al., „Text Embeddings Reveal (Almost) As Much As Text“ (EMNLP 2023) [Kenntnis, Fundstelle prüfen].
- **Einordnung (c):** Embeddings sind in der Praxis als personenbezogen zu behandeln, sobald sie mit Quelltext oder Metadaten verknüpft sind oder die Rekonstruktion realistisch ist. Bei isolierten Vektoren ohne Metadaten und mit geringer Invertierbarkeit ist die Frage **umstritten**.
- **EDPB-Stellungnahme 28/2024 (17.12.2024)** [verifiziert]: KI-Modelle, die mit personenbezogenen Daten trainiert wurden, sind nicht in allen Fällen anonym. Erforderlich ist eine Einzelfallprüfung mit hoher Schwelle, die die Wahrscheinlichkeit der direkten oder indirekten Identifizierung und der Extraktion über Abfragen berücksichtigt. **Wichtig:** Die Stellungnahme betrifft trainierte Modelle, nicht den Retrieval-Index. Eine Übertragung auf RAG-Indizes ist eine **Analogie** und keine ausdrückliche Aussage.
- **HmbBfDI-Thesen (Juli 2024)** [verifiziert über Sekundärquelle]: Betroffenenrechte wie Auskunft, Löschung und Berichtigung können sich auf In- und Output eines KI-Systems beziehen, nicht auf das Modell selbst. Für RAG bedeutet das: Der Index ist nicht das Modell, sondern eine eigene Datenhaltung, und dort setzt Löschung an. Die Hamburger Position ist eine Behördenauffassung, keine verbindliche Entscheidung, und der EDPB differenziert anders (siehe oben).

---

## 4. Löschnachweis und Rechenschaftspflicht

**(a) Verbindliches Recht**

- **Art. 5 Abs. 2 DSGVO:** Der Verantwortliche muss die Einhaltung der Grundsätze des Art. 5 Abs. 1 nachweisen können [Kenntnis]. Dies umfasst Löschvorgänge.
- **Art. 24 DSGVO:** Verantwortung des Verantwortlichen, geeignete technische und organisatorische Maßnahmen umzusetzen und nachzuweisen [Kenntnis].
- **Art. 25 DSGVO:** Datenschutz durch Technikgestaltung und datenschutzfreundliche Voreinstellungen [Kenntnis].
- **Art. 30 DSGVO:** Verzeichnis der Verarbeitungstätigkeiten mit vorgesehenen Löschfristen (Abs. 1 lit. f) [Kenntnis].
- **Art. 32 DSGVO:** Sicherheit der Verarbeitung, u. a. Verschlüsselung und Pseudonymisierung (Abs. 1 lit. a) sowie die Fähigkeit zur Wiederherstellung (lit. b–c) [Kenntnis].

**(b) Behördenauslegung**

- **SDM Baustein 60** [verifiziert]: Löschen muss planbar, dokumentiert und überprüfbar sein. Die genaue Liste von Mindestanforderungen an ein Löschkonzept konnte ich **nicht** verifizieren [nicht gefunden].
- **DSK-Kurzpapier Nr. 11 „Recht auf Löschung“** [Kenntnis]: Fundstelle und Inhalt nicht in dieser Recherche geprüft.

**(c) Praxis: Was ein Löschkonzept typischerweise enthält**

Dies ist eine Praxisempfehlung, keine Vorgabe aus dem SDM:

- Datenkategorien und Verarbeitungstätigkeiten (aus dem Verzeichnis nach Art. 30)
- Löschfristen und Auslöser (Zweckerreichung, Widerruf, Widerspruch, Ablauf der Aufbewahrungsfrist)
- Verantwortliche und Eskalationswege
- technische Verfahren je Speicherort (Index, Cache, Logs, Backup, Schlüssel)
- Behandlung von Backups mit Sperrung und Rücksicherungsschutz
- Nachweis: Löschprotokoll mit Zeitpunkt, Umfang, Verfahren, Prüfung; bei Krypto-Shredding zusätzlich Schlüssellebenszyklus und Vernichtungsnachweis
- Kontrolle: stichprobenartige Verifikation, dass gelöschte Inhalte nicht mehr abrufbar sind (z. B. Testabfragen gegen Index und Cache)

**Spannung zu Protokollen:** Die Löschpflicht kollidiert mit Protokollierungspflichten (Art. 32; für Hochrisiko-KI nach AI Act Art. 12). Protokolle enthalten selbst oft personenbezogene Daten (Abfragen, Nutzer-IDs). Lösung ist Datenminimierung in den Logs, nicht ein Verzicht auf Logs [Kenntnis, Literatur].

---

## 5. Rolle des Anbieters vs. Anwenders

**Anbieter der Software**

- Ist die Wissensbasis lokal verschlüsselt und hat der Anbieter weder Zugriff auf die Daten noch auf den Schlüssel, ist er **regelmäßig weder Verantwortlicher noch Auftragsverarbeiter** für die Wissensbasisdaten. Die Definitionen in Art. 4 Nr. 7 (Verantwortlicher) und Nr. 8 (Auftragsverarbeiter) sowie Art. 28 setzen eine Verarbeitung im Auftrag bzw. über Zwecke und Mittel voraus [Kenntnis].
- **Ausnahmen:** Support mit Datenzugriff, Telemetrie, Fehlerberichte, Cloud-Schlüsselverwaltung, Update- oder Lizenzprüfung mit personenbezogenen Daten. Dann kann der Anbieter Auftragsverarbeiter (Art. 28) oder eigenständig Verantwortlicher sein. Dies ist eine **Einzelfallfrage** (c).
- Erwägungsgrund 78 DSGVO empfiehlt Herstellern, Datenschutz bei der Produktgestaltung zu berücksichtigen, verpflichtet sie aber nicht unmittelbar [Kenntnis].

**Anwender**

- Der Kunde, der die Software einsetzt, ist in der Regel **Verantwortlicher** (Art. 4 Nr. 7). Er trägt die Pflichten aus Art. 5, 17, 24, 25, 30 und 32, ggf. Informationspflichten (Art. 13, 14) und die Datenschutz-Folgenabschätzung (Art. 35) [Kenntnis].
- Die DSK geht laut Auswertungen davon aus, dass beim Einsatz von KI-Anwendungen häufig eine DSFA nötig sein wird [verifiziert über Fachquelle zur DSK-OH].

**Produkt- und Werbeaussagen**

- Aussagen wie „vergisst Daten“, „unwiederbringlich gelöscht“ oder „DSGVO-konforme Löschung“ sind **Tatsachenbehauptungen**, die belegt sein müssen. Bei Irreführung droht eine Abmahnung nach **§ 5 UWG** (irreführende geschäftliche Handlungen) [Kenntnis]. Ein Risiko besteht, wenn Restdaten in Backups, Caches oder Logs existieren und das Produkt dies verschweigt.
- Dazu kommt Art. 5 Abs. 2 DSGVO: Die Löschwirkung muss nachweisbar sein.
- Ob „DSGVO-konform“ als Eigenschaft beworben werden darf, ist eine Frage der Rechtsprechung zu Gütesiegeln und Werbeaussagen; ich habe keine einschlägige Entscheidung geprüft [nicht gefunden].

---

## 6. Gerichtliche Entscheidungen

- **EuGH C-131/12 (Google Spain, 13.05.2014)** [Kenntnis]: Grundsatzentscheidung zur Auslistung und zur Abwägung zwischen Rechten der betroffenen Person und Informationsinteressen. Sie betrifft **nicht** technische Löschverfahren.
- **EuGH C-649/17** [nicht gefunden]: Das von Ihnen genannte Aktenzeichen konnte ich **nicht verifizieren**. Ich habe es daher nicht zitiert. Bitte auf curia.europa.eu prüfen, bevor es in ein Gutachten geht.
- **Art. 17 Abs. 2 DSGVO** [Kenntnis]: Pflicht, andere Verantwortliche über das Löschersuchen zu informieren, wenn die Daten öffentlich gemacht wurden. Relevanz für RAG nur, wenn die Quelldokumente öffentlich zugänglich waren.
- **EuGH C-300/21 (Österreichische Post, 04.05.2023)** [Kenntnis]: Immaterieller Schadensersatz nach Art. 82 DSGVO setzt keine Erheblichkeitsschwelle voraus. Dies erhöht das Risiko bei Löschversäumnissen.
- **Deutsche Urteile zur Unzulänglichkeit technischer Löschversuche:** Ich habe **keine verifizierte Entscheidung** gefunden [nicht gefunden]. Ich nenne deshalb keine. Dies ist ein **offener Punkt**.
- **Krypto-Shredding in der Rechtsprechung:** keine Entscheidung gefunden [nicht gefunden].

---

## 7. Sonderlagen und Aufbewahrungspflichten

- **Art. 17 Abs. 3 lit. b DSGVO** [Kenntnis]: Ausnahme, wenn gesetzliche Aufbewahrungspflichten bestehen. Diese Vorrangregel gilt nur für die betroffenen Daten, nicht für das ganze System.
- **§ 147 AO** [Kenntnis]: Aufbewahrung von Buchungsbelegen und steuerlich relevanten Unterlagen (in der Regel 10 Jahre, je nach Kategorie).
- **§ 257 HGB** [Kenntnis]: Aufbewahrung von Handelsbriefen und Buchungsbelegen (6 bzw. 10 Jahre).
- **§ 630f BGB** [Kenntnis]: Dokumentation in der Behandlung, grundsätzlich 10 Jahre.
- **Berufsgeheimnis (§ 203 StGB)** [Kenntnis]: begrenzt die Weitergabe, nicht direkt die Löschung, kann aber bei Entsorgung und Systemzugriff relevant sein.
- **§ 35 BDSG** [Kenntnis, Wortlaut prüfen]: Abs. 1 Satz 1 erlaubt eine Ersetzung der Löschung nur bei **nicht automatisierter** Verarbeitung mit unverhältnismäßigem Aufwand. Für automatisierte RAG-Indizes ist diese Ausnahme **eher nicht einschlägig**. Abs. 3 regelt die Einschränkung bei Aufbewahrungspflichten.

---

## 8. KI-spezifisch

- **DSK-Orientierungshilfe „Künstliche Intelligenz und Datenschutz“, Stand 06.05.2024** [verifiziert]: Die Orientierungshilfe enthält ein Kapitel 1.11 „Berichtigung, Löschung und weitere Betroffenenrechte“ [verifiziert im Inhaltsverzeichnis]. Ein Kommentar beschreibt, dass Berichtigung über Anpassung von Trainingsdaten und Filter erfolgen kann [verifiziert über Sekundärquelle]. Eine **konkrete Aussage zu RAG-Indizes** konnte ich nicht finden [nicht gefunden]. Die DSK hat die Orientierungshilfe als fortlaufend zu aktualisierendes Dokument angekündigt [verifiziert]. Ein Hinweis aus einer Kanzleiveröffentlichung deutet auf eine Überarbeitung im Mai 2026 hin, die Art. 50 AI Act betrifft; dies habe ich **nicht bestätigt** [nicht gefunden].
- **EDPB-Stellungnahme 28/2024** [verifiziert]: Die Stellungnahme nennt technische Maßnahmen wie Output-Filter und Techniken zum nachträglichen Entfernen oder Unterdrücken personenbezogener Daten als Möglichkeiten, Betroffenenrechte zu unterstützen. Sie erwägt außerdem, das Löschrecht auch dann zu gewähren, wenn die Voraussetzungen des Art. 17 Abs. 1 nicht vorliegen [verifiziert über Fachquelle]. Das ist eine **Empfehlung**, kein verbindliches Recht.
- **Recht auf Vergessen bei RAG konkret:** Ich habe **keine verbindliche Entscheidung und keine verifizierte Behördenaussage** gefunden, die RAG-Systeme ausdrücklich behandelt [nicht gefunden]. Dies ist **offen**.

**Praktische Unterscheidung (c):** Bei RAG ist Löschen technisch deutlich einfacher als bei trainierten Modellen, weil Dokumente und Chunks über IDs adressierbar sind. Das Problem liegt eher in der vollständigen Entfernung aus allen Nebenspeichern (Index-Kompaktierung, Caches, Logs, Backups).

---

## 9. EU AI Act und NIS2

**AI Act** [Kenntnis, Status prüfen]

- **Art. 4** (KI-Kompetenz): seit 02.02.2025 anwendbar [Kenntnis].
- **Art. 50** (Transparenzpflichten): nach der Verordnung ab 02.08.2026 anwendbar. Ein Vorschlag zum „Digital Omnibus“ (Ende 2025) sah Verschiebungen bei Hochrisiko-Pflichten vor. **Den aktuellen Stand habe ich nicht geprüft.** [nicht gefunden]
- **Art. 10** (Daten-Governance) und **Art. 12** (Protokollierung) gelten für Hochrisiko-KI. Ob ein RAG-System hochriskant ist, hängt vom Einsatzzweck ab (Anhang III); für interne Wissenssysteme ist das in der Regel nicht der Fall (c).
- **Art. 26** (Betreiberpflichten) betrifft Hochrisiko-Systeme.
- Der AI Act ersetzt die DSGVO nicht; die Datenschutzpflichten gelten parallel [Kenntnis, Art. 2 Abs. 7 AI Act].

**NIS2** [Kenntnis, Status prüfen]

- **Art. 21 Abs. 2 lit. h** NIS2: Maßnahmen zur Kryptografie und Verschlüsselung, einschließlich Schlüsselmanagement, sind Teil des Risikomanagements [Kenntnis].
- Die Umsetzung in Deutschland (NIS2UmsuCG) war zum Stand meiner Recherche **nicht verifiziert** [nicht gefunden]. Bitte den aktuellen Gesetzesstand prüfen.
- Anwendbarkeit hängt davon ab, ob der Anwender zu den betroffenen Sektoren gehört.

---

## Aktualitätsübersicht

|Dokument|Stand|Status|
|---|---|---|
|EDPB-Leitlinien 01/2025 Pseudonymisierung|16./17.01.2025, Konsultation bis Ende Feb. 2025|Finale Fassung nicht verifiziert|
|EDPB-Stellungnahme 28/2024 KI-Modelle|17.12.2024|verifiziert, gültig|
|DSK-Orientierungshilfe KI und Datenschutz|06.05.2024|verifiziert; Überarbeitung angekündigt, Stand unklar|
|HmbBfDI-Thesen LLM|Juli 2024|verifiziert über Sekundärquelle|
|SDM V2.0, Baustein 60 V1.0a|Version V1.0a|verifiziert; SDM V3.0/V3.1 existieren, Bausteininhalt ungeprüft|
|BayLDA-Checkliste KI|Januar 2024|nicht verifiziert|
|AI Act Art. 50 / Digital Omnibus|Anwendung ab 02.08.2026|Status ungeprüft|
|NIS2-Umsetzung Deutschland|—|Status ungeprüft|

---

## Was ist gesichert, was umstritten, was offen

|Aussage|Einordnung|Fundstelle|
|---|---|---|
|Löschen ist eine Form der Verarbeitung|gesichert|Art. 4 Nr. 2 DSGVO [Kenntnis]|
|Die DSGVO schreibt keine Löschmethode vor|gesichert|DSGVO-Wortlaut, Art. 17 [Kenntnis]|
|Pseudonymisierte Daten bleiben personenbezogen, solange Zuordnung möglich ist|gesichert (Behördenauslegung)|EDPB-Leitlinien 01/2025 [verifiziert]|
|KI-Modelle sind nicht per se anonym, Einzelfallprüfung|gesichert (Behördenauslegung)|EDPB-Stellungnahme 28/2024 [verifiziert]|
|Chunk-Texte im RAG-Index sind personenbezogen, wenn Personenbezug besteht|gesichert|Art. 4 Nr. 1 DSGVO [Kenntnis]|
|Rechenschaftspflicht erstreckt sich auf Löschvorgänge|gesichert|Art. 5 Abs. 2 DSGVO [Kenntnis]|
|Löschen muss dokumentiert, planbar und überprüfbar sein|gesichert (Behördenauslegung)|SDM Baustein 60 [verifiziert]|
|Crypto-Shredding als Löschung|**umstritten**|keine verifizierte Aufsichtsaussage [nicht gefunden]|
|Embeddings sind personenbezogen|**umstritten** (Einzelfall)|Art. 4 Nr. 1 [Kenntnis]; Morris et al. 2023 [Kenntnis]|
|Zeitversetzte Löschung in Backups|**umstritten** (Praxis)|keine verifizierte Behördenposition [nicht gefunden]|
|Anbieter ist Auftragsverarbeiter bei lokaler Verschlüsselung|**umstritten**, meist nein|Art. 4 Nr. 8, Art. 28 [Kenntnis]|
|§ 35 BDSG greift für automatisierte RAG-Speicherung|**eher nein**|§ 35 Abs. 1 BDSG [Kenntnis, prüfen]|
|Konkrete Löschfristen für Backups|**offen**|keine verifizierte Quelle|
|Deutsche Urteile zu Unzulänglichkeit technischer Löschversuche|**offen**|nicht gefunden|
|Krypto-Shredding in der Rechtsprechung|**offen**|nicht gefunden|
|Recht auf Vergessen bei RAG konkret|**offen**|DSK-OH KI nur allgemein; keine Entscheidung|
|Inhalt von EuGH C-649/17|**offen** (Aktenzeichen prüfen)|nicht verifiziert|
|Finale Fassung EDPB-Pseudonymisierungsleitlinien|**offen**|nicht verifiziert|
|Aktueller Stand DSK-OH KI (Überarbeitung 2026?)|**offen**|Hinweis, nicht bestätigt|
|BayLDA/Berlin/BW-Positionen zu Backups|**offen**|nicht verifiziert|
|AI Act Digital Omnibus und Art. 50|**offen**|Status ungeprüft|
|NIS2-Umsetzung in Deutschland|**offen**|Status ungeprüft|

---

## Punkte, die ein Fachanwalt für Datenschutzrecht oder der Datenschutzbeauftragte vor Produkttexten klären muss

1. **Kryptografische Argumentation:** Ist die Schlüsselvernichtung unter den konkreten Schlüsselverwaltungsbedingungen (Passphrase, KDF, Backup-Schlüssel, Hardware-Module, Cloud-KMS-Wartefristen, Speicherreste) so belastbar, dass sie als Unkenntlichmachung gilt? Dies ist eine Einzelfallprüfung.
2. **Vollständige Speicherorte:** Welche Kopien existieren (Index, Volltext, Caches, Logs, Backups, Snapshots, Debug-Dumps, Exporte), und für jede: Wie wird sie gelöscht oder unzugänglich gemacht?
3. **Embeddings:** Ist im konkreten Fall eine Rekonstruktion realistisch? Wenn ja, sind Embeddings personenbezogen zu behandeln.
4. **Backup-Frist:** Welche Frist gilt, und welche Aufsichtsbehörde hat sich dazu geäußert? Diese Frage muss mit den einschlägigen Landesbehörden geklärt werden.
5. **Rolle des Anbieters:** Gibt es Zugriff, Telemetrie, Support oder Schlüsselhinterlegung, die zur Auftragsverarbeitung (Art. 28) oder zur eigenen Verantwortlichkeit führt? Ist ein AV-Vertrag nötig?
6. **Werbeaussagen:** Welche Formulierungen sind belegbar? Insbesondere „vergisst“, „unwiederbringlich“ und „DSGVO-konform“ bedürfen einer Prüfung nach § 5 UWG.
7. **Aufbewahrungspflichten:** Welche Datenkategorien unterliegen Aufbewahrungspflichten, und wie wird damit im Löschkonzept umgegangen?
8. **Löschkonzept und Nachweis:** Liegt ein Löschkonzept vor, das die Anforderungen aus SDM und Art. 5 Abs. 2 erfüllt, und wie wird der Vollzug dokumentiert?
9. **Protokollierung:** Welche Logs sind erforderlich, und wie wird ihr Personenbezug begrenzt?
10. **DSFA:** Ist für den konkreten Einsatz eine Datenschutz-Folgenabschätzung erforderlich (Art. 35)?
11. **AI Act und NIS2:** Ist das System hochriskant oder betroffen, und welche Pflichten folgen daraus?
12. **Aktualität:** Stand der EDPB-Leitlinien, der DSK-Orientierungshilfe und der AI-Act-Anpassungen zum Zeitpunkt der Veröffentlichung.

Eine Formulierung wie „löscht Daten unwiederbringlich“ oder „DSGVO-konform“ sollte erst verwendet werden, wenn diese Punkte geklärt und dokumentiert sind. Stattdessen ist eine präzise, belegbare Beschreibung des Verfahrens und seiner Grenzen vorzuziehen, etwa: „Der Schlüssel wird vernichtet; die Wiederherstellung der Daten ist nach dem dokumentierten Verfahren nicht vorgesehen; Kopien in Backups werden innerhalb von [Frist] überschrieben.“