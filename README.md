<img width="3168" height="1344" alt="banner" src="https://github.com/user-attachments/assets/443fa9d1-7b8f-4ffe-aa61-46e85f5a414a" />

<p align="center"><b>CONTEXTRA</b></p>
<p align="center"><i>Das lokale Wissens- und Gedächtnissystem für KI-Assistenten und Agenten</i></p>

---

## Was ist CONTEXTRA?

KI-Assistenten sind nur so gut wie das Wissen, auf das sie zugreifen können. In der Praxis scheitern Projekte selten am Sprachmodell, sondern an fünf Fragen:

* Findet die KI **zuverlässig** die richtige Information in großen, gewachsenen Wissensbeständen?
* Kann sie sich **über Wochen und Monate erinnern**, ohne mit veraltetem oder widersprüchlichem Wissen zu arbeiten?
* Lässt sich ihr Verhalten **steuern, begrenzen und nachvollziehen**?
* Bleiben **vertrauliche Daten** unter Ihrer Kontrolle, auch wenn externe Modelle im Spiel sind?
* Läuft das Ganze **stabil, nachweisbar korrekt und wirtschaftlich** im Dauerbetrieb?

CONTEXTRA beantwortet diese Fragen in einem einzigen, lokal betriebenen System. Es gibt Ihren KI-Anwendungen ein präzises Wissensfundament, ein belastbares Langzeitgedächtnis, intelligente Steuerung, eingebauten Datenschutz und ein Speicherfundament, das auch bei Störungen konsistent bleibt.

### Auf einen Blick

| Baustein | Das leistet er für Sie |
|---|---|
| **1. Retrieval & Suche** | Die KI findet die richtige Information, auch bei Fachbegriffen, deutschen Wortzusammensetzungen und verknüpftem Wissen. |
| **2. Agenten & Gedächtnis** | Mehrstufige Aufgaben laufen kontrolliert und nachvollziehbar; Wissen bleibt über lange Zeit aktuell und verlässlich. |
| **3. Adaptives Routing** | Jede Anfrage geht an das passende Modell, mit Qualität, Geschwindigkeit und Kosten im Gleichgewicht. |
| **4. Sicherheit & Compliance** | Sensible Daten verlassen Ihr Haus nicht ungeschützt; Löschungen sind belegbar; Nachweise entstehen automatisch. |
| **5. Speicher & Durability** | Keine verlorenen Daten, keine inkonsistenten Zustände, Rückkehr zu jedem früheren Stand möglich. |
| **6. Ingest, Anbindung & Qualitätssicherung** | Ihre Dokumente kommen sauber ins System, Modelle und Tools lassen sich einfach anbinden, Qualität wird laufend gemessen. |

---

## 1. Finden statt Suchen: Retrieval & Wissenszugriff

<img width="2752" height="1536" alt="hybrid-retrieval" src="https://github.com/user-attachments/assets/0c24b7b2-7037-42f2-98a0-b439c33c83c7" />

**Das Ziel:** Ihre KI soll auf Anhieb die Informationen finden, die für eine Antwort wirklich zählen, auch wenn Ihr Wissensbestand groß ist, aus vielen Quellen stammt und sich ständig ändert.

CONTEXTRA kombiniert dafür mehrere Suchverfahren, die sich gegenseitig ergänzen: Bedeutungssuche, exakte Stichwortsuche und die Suche entlang von Zusammenhängen im Wissensgraphen. Ein Dokument, das in mehreren dieser Perspektiven gleichzeitig überzeugt, wird bevorzugt.

### Bedeutung verstehen: Semantische Suche

* **Treffer nach Sinn, nicht nach Wortlaut.** Eine Frage wie „Kündigungsfrist bei Lieferverzug“ findet auch Passagen, die andere Formulierungen verwenden.
* **Skaliert mit Ihrem Bestand.** Der Zugriff bleibt auch bei sehr großen Datenmengen schnell. Neue und gelöschte Inhalte werden laufend im Index nachgeführt, ohne dass dieser neu aufgebaut werden muss.
* **Deutlich geringerer Hardwarebedarf.** Platzsparende Speicherverfahren reduzieren den Arbeitsspeicherbedarf auf ein Viertel, in der kompaktesten Variante auf ein Zweiunddreißigstel, bei nur minimalem Qualitätsverlust.
* **Auch jenseits des Arbeitsspeichers.** Bestände, die nicht mehr in den RAM passen, werden direkt von der SSD bedient. Kleinere Bestände nutzen automatisch den schnelleren In-Memory-Weg.
* **Filter, die die Trefferqualität nicht kosten.** Einschränkungen wie Mandant, Dokumenttyp oder Zeitraum werden schon während der Suche angewandt und nicht erst danach. So bleiben auch bei sehr engen Filtern passende Treffer erhalten.
* **Hardware optimal ausgenutzt.** Das System erkennt automatisch, welche Prozessorfunktionen verfügbar sind (Intel/AMD und ARM), und nutzt die schnellste Variante. Durch automatische Gegenprüfung liefern alle Pfade identische Ergebnisse.

### Exakt treffen: Volltextsuche mit deutscher Sprachkompetenz

* **Bewährte Relevanzbewertung.** Titel, Überschriften, Fließtext und Schlagworte lassen sich unterschiedlich gewichten.
* **Schnell bei großen Korpora.** Intelligentes Überspringen irrelevanter Bereiche beschleunigt die Suche, ohne Treffer zu verlieren.
* **Für deutsche Sprache gebaut.** „Straße“ und „Strasse“, „Müller“ und „Mueller“ führen zum selben Ergebnis. Zusammengesetzte Wörter wie „Haftpflichtversicherungsvertrag“ werden zerlegt und gefunden, ob Sie nach dem ganzen Wort oder nach einem Teil suchen. Stoppwörter und Wortendungen werden berücksichtigt.
* **Suchen ohne Wartezeit bei Updates.** Leser arbeiten stets auf einem konsistenten Stand, während im Hintergrund neue Daten eingepflegt werden.

### Zusammenhänge nutzen: Wissensgraph und GraphRAG

* **Verknüpftes Wissen wird auffindbar.** Wenn die Antwort auf mehrere Dokumente verteilt ist, folgt CONTEXTRA den Beziehungen zwischen Personen, Organisationen, Verträgen oder Sachverhalten und holt das relevante Umfeld hinzu.
* **Mehrstellige Beziehungen.** Nicht nur „A hängt mit B zusammen“, sondern auch Sachverhalte mit mehreren Beteiligten und Rollen, inklusive zeitlicher Gültigkeit.
* **Wissen mit Beziehungstypen.** Aussagen können sich stützen, widersprechen, ergänzen, voneinander abstammen oder einander ablösen. Wird ein Dokument durch eine neuere Fassung ersetzt, gelten davon abhängige Aussagen automatisch als überholt, die Historie bleibt nachvollziehbar.
* **Themenlandkarte statt Dokumenthaufen.** Der Wissensbestand wird automatisch in Themengruppen auf mehreren Ebenen gegliedert. Das System erkennt, wenn das Netz unübersichtlich wird, und stärkt häufig genutzte, bestätigte Pfade, während Ungenutztes allmählich zurücktritt.

### Wissen automatisch strukturieren: Informationsextraktion

* **Aus Fließtext wird strukturiertes Wissen.** Beim Einlesen werden Aussagen der Form „wer – tut was – mit wem/was“ erkannt, ebenso Organisationen, Personen, Normen, Zeitangaben und Kennungen.
* **Jede Aussage ist belegt.** Zu jedem extrahierten Fakt ist Quelldokument, Textstelle und Verlässlichkeit gespeichert.
* **Sie steuern den Aufwand.** Die Extraktion lässt sich abschalten, auf Regeln beschränken oder voll nutzen.

### Komplexe Fragen beantworten: Mehrstufige Suche und Steuerung

* **Fragen, die mehr als einen Schritt brauchen.** CONTEXTRA verfeinert die Anfrage selbstständig anhand der bisherigen Funde und erschließt Folgeinformationen.
* **Stoppt, wenn genug gefunden ist.** Ein Regelmechanismus misst den Informationsgewinn pro Schritt und beendet die Suche, sobald nichts Neues mehr hinzukommt. So werden Zeit und Kosten gespart.
* **Eine Abfrage, voll steuerbar.** Sie legen fest: in welchem Bereich gesucht wird (Mandant, Sammlung, Dokumentgruppe), nach welcher Strategie, wie stark die einzelnen Suchverfahren zählen und ob nur der Stand zu einem bestimmten Zeitpunkt maßgeblich ist.
* **Schlanke Antworten.** Schwere Inhalte werden erst für die endgültigen Treffer geladen. Kontexte werden auf das Token-Budget gekürzt, ohne die entscheidenden Textstellen zu verlieren.
* **Planbares Verhalten bei Störungen.** Fällt ein Suchkanal aus, bestimmen Sie vorab, ob abgebrochen, mit den übrigen Kanälen weitergearbeitet oder ein Pflichtkanal verlangt wird.
* **Zeitlich korrekt.** Wissen hat ein Gültigkeitsdatum. Gefragt werden kann nach dem, was heute gilt, aber auch nach dem, was zu einem früheren Zeitpunkt galt oder bekannt war.

### Ein Ergebnis, dem man vertrauen kann: Fusion und Kalibrierung

* **Fairer Zusammenschluss der Suchverfahren.** Die Ergebnisse unterschiedlicher Verfahren werden zu einer gemeinsamen Rangliste zusammengeführt. Auffällig klare Spitzentreffer heben sich von diffusen Feldern ab, Mehrfachbestätigungen erhalten einen Bonus.
* **Aussagekräftige Wahrscheinlichkeiten.** Rohwerte werden in kalibrierte Konfidenzen umgerechnet, sodass „80 % relevant“ auch tatsächlich ungefähr 80 % bedeutet, mit statistisch abgesicherten Unsicherheitsbereichen.
* **Frühwarnsystem gegen schleichende Qualitätsverluste.** Das System erkennt automatisch, wenn sich das Verhalten der Suche gegenüber der Referenz verschiebt, etwa durch geänderte Daten oder Nutzungsmuster.

---

## 2. Aufgaben selbstständig erledigen: Agenten und Langzeitgedächtnis

<img width="2752" height="1536" alt="agentic" src="https://github.com/user-attachments/assets/87780e2a-54bb-492f-817b-df9f0d67c1c3" />

**Das Ziel:** KI-Agenten sollen mehrstufige Aufgaben zuverlässig abarbeiten, im Rahmen klarer Regeln und Budgets, mit lückenloser Dokumentation, und über die Zeit dazulernen, ohne zu „zumüllen“ oder zu halluzinieren.

### Kontrollierte Agenten-Ausführung

* **Jeder Schritt ist abgesichert.** Agenten arbeiten Prozesse als Folge klar definierter Schritte ab: Zustand sichern, ausführen, bestätigen, protokollieren. Misslingt etwas, ist der letzte gute Stand wiederherstellbar.
* **Reproduzierbar und testbar.** Abläufe lassen sich exakt nachspielen, Fehler gezielt nachstellen und Tests verlässlich automatisieren.
* **Wege und Ziele sind definiert.** Verzweigungen, Fallbacks und parallele Pfade folgen festen Bedingungen. Ein Workflow endet erst, wenn sein Ziel nachweislich erreicht ist.
* **Kein Endlosdrehen.** Schrittgrenzen und die Erkennung wiederkehrender Zustände stoppen Agenten, die sich im Kreis bewegen oder keinen Fortschritt mehr machen.
* **Fehler gehen nicht verloren.** Fehlgeschlagene Schritte landen in einer Warteschlange mit Fehlerklassifizierung und automatischen Wiederholversuchen. Sie lassen sich später gezielt erneut anstoßen.
* **Reagiert auf Ereignisse.** Agenten können auf eingehende Ereignisse reagieren, ohne vom Ansturm überrollt zu werden.
* **Ein aufgeräumter Arbeitsbereich.** Zwischenergebnisse und Hypothesen liegen getrennt von den eigentlichen Anweisungen. Grundregeln und Rollenprofile bleiben geschützt im Kontext und werden bei Verdichtung nicht verdrängt.
* **Kosten und Ressourcen im Griff.** Token, Schritte, Werkzeugaufrufe und Kosten werden pro Aufgabe und über parallele Teilagenten hinweg exakt begrenzt.
* **Werkzeuge mit klaren Spielregeln.** Jedes Tool hat eine Beschreibung mit erlaubten Eingaben, Berechtigungen, Zeitlimit und Auswirkungen. Aufrufe werden vorab geprüft, die Wissenssuche ist direkt eingebaut.
* **Vollständige Nachvollziehbarkeit.** Zustandswechsel, Entscheidungen, Werkzeugaufrufe, Verbräuche und Fehler werden manipulationssicher protokolliert.

### Ein Gedächtnis, das mitwächst und aufräumt

* **Doppeltes wird zusammengeführt.** Im Hintergrund erkennt das System Duplikate und Beinahe-Duplikate und bündelt sie, effizient auch bei großen Beständen. Sperren und Schutzmechanismen verhindern, dass parallele Läufe sich in die Quere kommen.
* **Lange Gespräche bleiben handhabbar.** Wächst ein Verlauf an die Kontextgrenze, wird er automatisch verdichtet. Sie wählen die Strategie: Ältestes entfernen, zusammenfassen oder nur das Wesentliche (Fakten, Entscheidungen, Randbedingungen) behalten. Gepinnte Vorgaben bleiben stets erhalten, die Rohdaten archiviert.
* **Aus Einzelheiten werden Konzepte.** Zusammengehörige Erinnerungen werden zu übergeordneten Wissensknoten verdichtet, die mit den Details verknüpft bleiben, so dass Antworten auch über mehrere Stationen hinweg gelingen.
* **Schutz vor Fehlschlüssen.** Das System verhindert, dass Dinge fälschlich verschmelzen, nur weil sie über Umwege ähnlich wirken.
* **Keine erfundenen Erinnerungen.** Jede verdichtete Zusammenfassung wird gegen die Originalquellen geprüft. Unbelegte Aussagen werden gar nicht erst gespeichert.
* **Vergessen, wie ein gutes Gedächtnis.** Ungenutztes tritt mit der Zeit zurück, häufig Abgerufenes wird gestärkt. Ein Hintergrunddienst übernimmt Konsolidierung, Auffrischung, Archivierung und Ablauf nach Ihrer Konfiguration.

---

## 3. Das richtige Modell zur richtigen Zeit: Adaptives Routing und Regelung

<img width="2752" height="1536" alt="routing" src="https://github.com/user-attachments/assets/d792ecc5-2744-4010-b258-f9011030ad81" />

**Das Ziel:** Nicht jede Frage braucht das größte und teuerste Modell. CONTEXTRA wählt pro Anfrage automatisch das Modell aus, das Qualität, Antwortzeit und Kosten am besten ausbalanciert, und lernt dabei aus der eigenen Praxis.

### Lernende Modellauswahl

* **Entscheidung nach Kontext.** Art und Anforderungen einer Anfrage bestimmen, ob ein lokales Kleinmodell, ein leistungsstarkes Remote-Modell oder ein anderer Kandidat zum Zug kommt.
* **Probieren und Bewähren.** Das System testet gezielt Alternativen, um bessere Optionen zu entdecken, und nutzt Bewährtes, wo es sich lohnt. Die Mechanik ist kontrollierbar und nachvollziehbar.
* **Robust gegen Schwankungen.** Tageszeitabhängige Latenzen oder Modell-Updates verfälschen die Auswahl nicht. Veraltete Beobachtungen werden schrittweise aus dem Lernen verdrängt, einzelne Ausreißer (etwa ein Netzwerk-Timeout) dämpfen die Bewertung nicht.
* **Neue Strategien gefahrlos prüfen.** Mit historischen Daten lässt sich vorab abschätzen, wie eine neue Routing-Strategie abgeschnitten hätte. Im Schattenbetrieb läuft sie parallel mit und protokolliert, was sie getan hätte, ohne dass Nutzer davon etwas merken.

### Stabiler Betrieb durch Regelung

* **Konstante Antwortzeiten.** Ein Regler passt Suchtiefe, Kandidatenzahl und Zeitbudgets laufend an das Latenzziel an und bleibt auch bei Lastspitzen stabil.
* **Keine Modell-Monokultur.** Ein Ausgleichsmechanismus hält Auslastung, Warteschlangen und Modellvielfalt in einem gesunden Korridor, damit nicht dauerhaft alles auf ein einziges Modell läuft.
* **Aktuelle Zahlen zählen mehr.** Ältere Leistungsdaten verlieren nach einer einstellbaren Halbwertszeit an Gewicht.
* **Mathematisch überwachte Stabilität.** Zur Laufzeit wird geprüft, ob sich das Zusammenspiel der Regelkreise beruhigt oder aufschaukelt. Droht Instabilität, schaltet das System selbstständig auf vorsichtige Sicherheitswerte um.

---

## 4. Vertraulichkeit und Nachweisbarkeit: Sicherheit, Datenschutz und Compliance

<img width="2752" height="1536" alt="security" src="https://github.com/user-attachments/assets/2c02271c-171c-4256-92fc-d43a4b98623d" />

**Das Ziel:** Sie können KI intensiv nutzen, auch mit externen Modellen, ohne die Kontrolle über sensible Daten zu verlieren, und können Ihre Schutzmaßnahmen gegenüber Dritten belegen.

### Schutz vor Datenabfluss und Manipulation

* **Sensible Daten werden erkannt, bevor sie das Haus verlassen.** Personenbezogene und vertrauliche Angaben wie IBAN, Kreditkartennummern, Steuer- und Ausweisnummern, E-Mail-Adressen und Telefonnummern werden mit gezielten Regeln und rechnerischen Prüfungen erkannt, nicht nur geraten.
* **Pseudonymisierung mit Rückweg.** Vor dem Versand an ein externes Modell werden sensible Angaben durch neutrale Platzhalter ersetzt (z. B. `[IBAN_01]`). In der Antwort werden sie verlustfrei wieder eingesetzt. Das externe Modell sieht die echten Werte nie.
* **Schutz auch vor Umschreibungen.** Ein semantischer Abgleich blockiert Übertragungen, die vertrauliche Inhalte nur in anderen Worten wiedergeben würden.
* **Erkennt unnormale Datenabflüsse.** Sowohl plötzliche Massenabflüsse als auch langsames, unauffälliges Abgreifen werden pro Mandant und Ziel überwacht und gestoppt.
* **Abwehr von Prompt-Injection.** Manipulationsversuche, sei es direkt oder versteckt in abgerufenen Dokumenten oder Werkzeugantworten, werden erkannt. Anweisungen und Nutzdaten bleiben strikt getrennt.
* **Lückenloses Eingriffsprotokoll.** Jede Maskierung, Ersetzung und Blockade wird in einer verketteten, nachträglich nicht veränderbaren Protokollfolge festgehalten.

### Verschlüsselung und Löschung, die man beweisen kann

* **Moderne Verschlüsselung mit doppeltem Boden.** Daten sind auch bei Fehlern in der Anwendung robust geschützt. Mandanten und Dokumente haben getrennte Schlüssel in einer klaren Schlüsselhierarchie, Passwörter werden mit speicherintensiven Verfahren abgesichert.
* **Löschen heißt wirklich löschen.** Wird der Schlüssel eines Dokuments oder Mandanten vernichtet, sind alle zugehörigen Daten sofort unwiederbringlich unlesbar, in Datenbank, Indizes und Backups, ohne langwieriges Neuindizieren.
* **Löschnachweise mit digitaler Signatur.** Jede Löschung kann extern überprüfbar belegt werden, mit unveränderlichem Widerrufsprotokoll.
* **Manipulationssichere Protokolle.** Jeder Eintrag im Transaktionslog ist kryptografisch an den vorherigen gebunden. Auch gelöschte oder eingeschleuste Einträge werden bei Wiederherstellung und im Betrieb entdeckt.
* **Echte Modelle, nachweislich.** Vor jeder Nutzung wird per Fingerabdruck geprüft, dass Modellgewichte und Konfiguration unverändert sind.
* **Geheimnisse bleiben im Speicher.** Schlüssel und Passwörter werden vor dem Auslagern auf die Festplatte geschützt und nach Gebrauch zuverlässig überschrieben.

### Compliance auf Knopfdruck

* **Nachweise für Audits.** Verarbeitungsverzeichnisse und Audit-Exporte (JSON, CSV, Markdown) werden automatisch erzeugt, vertrauliche Nutzdaten dabei bereinigt, und lassen sich den Bausteinen des BSI IT-Grundschutz-Kompendiums zuordnen.
* **Auftragsverarbeitungsvertrag (Art. 28 DSGVO) aus der tatsächlichen Konfiguration.** Der Vertrag entsteht aus den Funktionen Ihres Systems, mit Klauseln zu Weisungsbindung, Vertraulichkeit, Betroffenenrechten, Unterstützungspflichten, Kontroll- und Auditrechten sowie Beendigung und Löschung. Vertrag und Realität stimmen so überein. Eine rechtliche Prüfung im Einzelfall ersetzt dies nicht.

---

## 5. Verlässlich im Dauerbetrieb: Speicher, Transaktionen und Wiederherstellung

<img width="2752" height="1536" alt="storage" src="https://github.com/user-attachments/assets/4ac5cdb6-b731-4653-af1c-4435c670a7fc" />

**Das Ziel:** Ihre Daten bleiben vollständig und konsistent, auch bei Stromausfall, Absturz oder gleichzeitigen Zugriffen, und Sie können jederzeit zu einem früheren Stand zurückkehren.

### Absturzsichere Datenhaltung

* **Nach einem Absturz sofort wieder konsistent.** Jede Änderung wird vorab geprüft protokolliert. Beim Neustart stellt das System den korrekten Zustand automatisch und gefahrlos wieder her.
* **Schutz vor schleichender Datenkorruption.** Im Hintergrund werden gespeicherte Daten gegen Prüfsummen abgeglichen, damit auch stille Fehler auf dem Datenträger auffallen. Verwaiste Dateien werden aufgeräumt, der Gesundheitszustand ist jederzeit einsehbar.
* **Gründlich getestet.** Gezielte Fehlersimulation prüft systematisch, was bei Abstürzen an jeder denkbaren Stelle passiert.
* **Schnelle Schreibleistung ohne Kompromiss.** Der Speicherkern ist auf hohen Schreibdurchsatz ausgelegt. Mehrstufige Filter vermeiden unnötige Lesezugriffe, ein geschütztes Protokoll sichert jede Änderung und eine Hintergrundpflege hält den Bestand kompakt.

### Gleichzeitiger Zugriff ohne Überraschungen

* **Vollständige Konsistenz bei parallelen Zugriffen.** Mehrere Nutzer und Agenten arbeiten gleichzeitig, ohne sich zu blockieren. Typische Konflikte wie Lost Updates oder widersprüchliche Lesestände werden zuverlässig verhindert.
* **Alles oder nichts, auch über mehrere Indizes.** Eine Änderung wird in Vektor-, Volltext-, Graph- und Schlüsselspeicher entweder vollständig übernommen oder vollständig zurückgenommen. Mandanten bleiben strikt getrennt, bei Überlast schützt das System sich selbst.

### Zeitreise und Rückkehr zu guten Zuständen

* **Automatischer Rücksetzpunkt.** Vor kritischen Operationen wird ein Zustand gesichert und bei Fehlern selbsttätig wiederhergestellt.
* **Sofortige Sicherungen ohne Kopieraufwand.** Zustandskopien entstehen in kürzester Zeit und fast ohne zusätzlichen Speicherbedarf.
* **Zurück zu jedem früheren Stand.** Die gesamte Datenbank lässt sich auf einen belegten, früheren Zeitpunkt zurücksetzen. Die Prüfung der Sicherungen verhindert, dass beschädigte oder unpassende Stände eingespielt werden.

### Gefahrlose Erweiterbarkeit

* **Fremder Code in der Sandbox.** Externe Tools und Erweiterungen laufen in einer abgeschotteten Umgebung ohne Zugriff auf das Hostsystem. Rechenzeit und Arbeitsspeicher sind exakt begrenzt, so dass kein Zusatzbaustein das System lahmlegen kann.

---

## 6. Vom Dokument zur Qualitätskontrolle: Ingest, Anbindung und Evaluation

<img width="2752" height="1536" alt="ingest" src="https://github.com/user-attachments/assets/18e529ac-5458-4d62-8dcb-48c962df873c" />

**Das Ziel:** Ihr vorhandenes Wissen wird ohne Aufwand erschlossen, passende KI-Modelle lassen sich einfach anbinden, und die Qualität des Gesamtsystems wird fortlaufend belegt statt behauptet.

### Dokumente sauber einlesen

* **Viele Formate, erhaltene Struktur.** PDF, Word, Markdown und Klartext werden gelesen. Überschriften, Absätze, Tabellen und Codeblöcke bleiben als Struktur erkennbar.
* **Sinnvoll zerteilt.** Texte werden an natürlichen Satz- und Abschnittsgrenzen in passend große, leicht überlappende Abschnitte geteilt, damit nichts an Schnittstellen verloren geht und das Token-Budget eingehalten wird.
* **Vollständig portabel.** Komplette Bestände mit Vektoren, Volltext, Beziehungen und Metadaten lassen sich verlustfrei exportieren und wieder importieren, inklusive Integritätsprüfung.

### Modelle nach Wahl anbinden

* **Direkt im Prozess.** Lokale Sprach- und Embedding-Modelle laufen ressourcenschonend ohne zusätzlichen Server. Modellinformationen werden automatisch ausgelesen.
* **Industrieller Standard.** ONNX-Modelle für Embeddings, Nachsortierung der Treffer (Reranking) und Klassifikation werden mit Pooling und Bündelung effizient betrieben.
* **Ollama-Anbindung mit Selbstschutz.** Lokale und entfernte Ollama-Server werden stabil angebunden. Bei Ausfall werden sie automatisch umgangen und nach Erholung wieder zugeschaltet.

### Einfach integrierbar

* **Für Entwicklungsumgebungen und Agenten-Hosts.** Der MCP-Server stellt Suche, Wissensgraph, Gedächtnis und Agentenfunktionen als eigenständiges Programm ohne Abhängigkeiten bereit.
* **Für Python-Teams.** Als Python-Paket installierbar und ohne Leistungseinbußen nutzbar, auch bei parallelen Aufgaben und großen Vektordaten.
* **Ein zentraler Einstiegspunkt.** Die Integrations-Engine verwaltet Start und Betrieb, liefert Messwerte und Gesundheitsdiagnosen und stellt sicher, dass die übergreifenden Qualitätsregeln des Systems eingehalten werden.

### Qualität messbar und abgesichert

* **Suchqualität in Zahlen.** Standardisierte Benchmarks (NDCG, MRR, Recall) auf bekannten Datensätzen sowie auf einem eigenen deutschen Kompositakorpus.
* **Geschwindigkeit und Trefferquote im Zusammenspiel.** Durchsatz und Antwortzeiten (Median bis p99) werden für jede Suchvariante vermessen.
* **Gedächtnisqualität.** Benchmarks für langfristiges, sitzungsübergreifendes Erinnern, zeitliche Konsistenz und die Güte der Verdichtung.
* **Keine unbemerkten Rückschritte.** Automatisierte Tests, auch bei Mehrschritt-Wissenssuche und mit 100.000-Dokument-Skalierung, blockieren Änderungen, die Qualität oder Geschwindigkeit verschlechtern würden, bevor sie ausgeliefert werden.

---

## Zusammenfassung

CONTEXTRA ist kein einzelnes Werkzeug, sondern ein zusammenhängendes Fundament für produktive KI-Anwendungen:

* **Treffsicher:** die richtige Information zur richtigen Frage, auch bei deutschem Fachvokabular und verteiltem Wissen.
* **Verlässlich:** ein Gedächtnis, das mitwächst, Widersprüche auflöst und nichts erfindet.
* **Wirtschaftlich:** das passende Modell pro Anfrage, mit planbaren Kosten und Antwortzeiten.
* **Vertrauenswürdig:** Datenschutz, Löschbeweise und Audit-Unterlagen als fester Bestandteil.
* **Robust:** konsistente Daten bei jedem Ausfall, Rückkehr zu jedem früheren Stand.
* **Offen:** einfache Anbindung von Dokumenten, Modellen, Entwicklungsumgebungen und Python.
* **Belegt:** Qualität, die laufend gemessen und abgesichert wird.
