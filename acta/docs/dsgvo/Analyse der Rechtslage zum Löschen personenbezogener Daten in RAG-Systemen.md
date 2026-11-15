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
