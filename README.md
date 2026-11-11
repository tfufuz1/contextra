<img width="3168" height="1344" alt="banner" src="https://github.com/user-attachments/assets/443fa9d1-7b8f-4ffe-aa61-46e85f5a414a" />

<p align="center"><b>CONTEXTRA</b></p>
<p align="center"><i>Das lokale Wissens- und Gedächtnissystem für KI-Assistenten und Agenten</i></p>

<p align="center">
  <a href="#überblick">Überblick</a> ·
  <a href="#welche-herausforderungen-contextra-löst">Herausforderungen</a> ·
  <a href="#so-funktioniert-contextra">Funktionsweise</a> ·
  <a href="#funktionen">Funktionen</a> ·
  <a href="#ein-beispiel-aus-der-praxis">Beispiel</a> ·
  <a href="#einsatzbereiche">Einsatzbereiche</a>
</p>

---

## Überblick

KI-Assistenten sind nur so hilfreich wie das Wissen, auf das sie zugreifen können. Contextra stellt dieses Wissen bereit, ohne dass Daten das Unternehmen verlassen.

Contextra ist eine Softwarebibliothek, die direkt in eine bestehende Anwendung eingebunden wird. Es gibt keinen externen Dienst, keinen zusätzlichen Server-Prozess und keine Cloud-Verbindung. Dokumente, Erinnerungen und Zusammenhänge zwischen Begriffen werden auf dem eigenen Rechner, im eigenen Netzwerk oder auf einem Gerät ohne Internetzugang gespeichert und durchsucht.

Dabei vereint Contextra Fähigkeiten, die sonst auf mehrere Werkzeuge verteilt sind:

- **Suche** nach Bedeutung, nach exakten Begriffen und nach Zusammenhängen
- **Bewertung der Ergebnisqualität**, damit erkennbar ist, wie verlässlich ein Treffer ist
- **Gedächtnis für KI-Agenten**, das gepflegt, verdichtet und wieder abgerufen wird
- **Datenschutz und Verschlüsselung**, einschließlich nachweisbarer Löschung
- **Zuverlässige Speicherung** mit gesicherten Arbeitsschritten und Rückkehr zu früheren Zuständen

Das Ziel: Ein KI-System soll nicht nur Treffer finden, sondern auch einschätzen können, wie belastbar sie sind.

---

## Welche Herausforderungen Contextra löst

| Herausforderung | Lösung in Contextra |
|---|---|
| **Sensible Daten dürfen das Unternehmen nicht verlassen** | Speicherung, Suche und Auswertung erfolgen vollständig lokal. Ausgehende Anfragen werden auf personenbezogene Daten geprüft, sensible Werte durch Platzhalter ersetzt. |
| **Löschungen müssen nachweisbar sein** | Jede Löschung wird in einem digital signierten Nachweis dokumentiert. Verschlüsselte Daten werden durch Vernichtung ihres Schlüssels dauerhaft unlesbar. |
| **Exakte Begriffe gehen bei reiner Bedeutungssuche verloren** | Drei Suchverfahren laufen parallel und werden zu einer gemeinsamen Ergebnisliste zusammengeführt. |
| **Ähnlich klingende Treffer sind nicht automatisch verlässlich** | Jedes Ergebnis erhält einen statistisch geprüften Vertrauenswert, sodass Schwellenwerte eine klare Bedeutung haben. |
| **Lange Arbeitsabläufe scheitern an einem einzelnen Fehler** | Jeder Arbeitsschritt wird gesichert. Nach einem Fehler wird der letzte gültige Zustand wiederhergestellt. |
| **Kosten und Textmengen wachsen unkontrolliert** | Verläufe werden verdichtet, und ein Budget begrenzt Arbeitsschritte und Verbrauch. |
| **Nicht jede Anfrage braucht das leistungsstärkste Modell** | Das System wählt je Anfrage das Modell mit dem besten Verhältnis aus Qualität, Geschwindigkeit und Kosten. |

---

## So funktioniert Contextra

Contextra arbeitet in fünf Stufen, vom Dokument bis zur Antwort.

**1. Aufnehmen.** Dokumente wie PDF, Word (DOCX), Markdown oder E-Mails werden eingelesen und entlang ihrer Struktur in Abschnitte gegliedert. Überschriften, Listen und Absätze bleiben erhalten. Dieselbe Eingabe ergibt immer dieselbe Gliederung.

**2. Speichern.** Die Abschnitte werden verschlüsselt abgelegt. Jeder Schreibvorgang wird vollständig übernommen oder vollständig verworfen und zusätzlich in einem Protokoll gesichert. Ein Absturz hinterlässt daher keine beschädigten Daten.

**3. Auf mehreren Wegen suchen.** Eine Anfrage wird gleichzeitig über drei Wege beantwortet: nach inhaltlicher Bedeutung, nach exakten Begriffen und Zahlen sowie nach Beziehungen zwischen Personen, Begriffen und Ereignissen.

**4. Zusammenführen und prüfen.** Die Ergebnisse der drei Wege werden gewichtet zu einer Liste verschmolzen. Treffer, die über mehrere Wege bestätigt werden, rücken nach oben. Anschließend wird für jedes Ergebnis ein Vertrauenswert berechnet und die Verteilung dieser Werte laufend überwacht.

**5. Liefern.** Der Anwender oder das angebundene KI-System erhält wenige, begründete Treffer. Ist die Beleglage dünn, wird das ausdrücklich ausgewiesen, statt eine scheinbar sichere Antwort vorzutäuschen.

```
   Dokument oder Erinnerung
            │
            ▼
   ┌───────────────────────┐     ┌──────────────────────────────┐
   │ Aufnehmen             │ ──▶ │ Speichern                    │
   │ PDF · Word · Mail     │     │ verschlüsselt · abgesichert  │
   └───────────────────────┘     └──────────────┬───────────────┘
                                                │
        ┌──────────────────┬────────────────────┼─────────────────┐
        ▼                  ▼                    ▼                 ▼
   Bedeutungssuche   Begriffssuche        Beziehungssuche    Zeitbezug
   (Inhalt)          (exakte Wörter)      (Zusammenhänge)    (wann gültig)
        └──────────────────┴─────────┬──────────┴─────────────────┘
                                     ▼
                  Zusammenführen · Vertrauenswerte · Überwachung
                                     │
                                     ▼
                     Wenige, begründete Treffer
```

---

## Funktionen

### 1. Intelligente Suche

<img width="2752" height="1536" alt="hybrid-retrieval" src="https://github.com/user-attachments/assets/0c24b7b2-7037-42f2-98a0-b439c33c83c7" />

Contextra kombiniert drei Suchverfahren. Jedes deckt Fälle ab, die die anderen übersehen würden.

**Bedeutungssuche.** Findet Passagen mit gleichem Inhalt, auch wenn andere Wörter verwendet werden. Wer nach „Kündigungsfrist“ sucht, findet auch Abschnitte zur „Beendigung des Vertrags“. Die Suche bleibt auch in sehr großen Beständen schnell. Platzsparende Speicherverfahren reduzieren den Speicherbedarf deutlich, und für extrem große Bestände steht eine Variante bereit, die nicht vollständig im Arbeitsspeicher liegen muss.

**Begriffssuche.** Findet exakte Begriffe, Namen, Produktnummern und Zahlen und bewertet sie nach Häufigkeit und Seltenheit. Die deutsche Sprache wird berücksichtigt: Zusammengesetzte Wörter wie „Datenschutzfolgenabschätzung“ werden zerlegt, Umlaute vereinheitlicht und Füllwörter ignoriert. Lesende Zugriffe müssen nicht auf laufende Schreibvorgänge warten.

**Beziehungssuche.** Personen, Begriffe und Ereignisse bilden ein Netz aus Verbindungen. Contextra erkennt, welche Elemente rund um eine Frage relevant sind, und belegt Zusammenhänge mit nachvollziehbaren Pfaden. Auch Beziehungen, an denen mehr als zwei Parteien beteiligt sind, werden berücksichtigt.

**Themengruppen.** Zusammengehörige Begriffe werden automatisch zu Wissensbereichen gruppiert. Häufig gemeinsam genutzte Verbindungen werden gestärkt, und das System erkennt, wenn ein Themenbereich zerfällt.

**Zusammenführung der Ergebnisse.** Die Trefferlisten der drei Verfahren werden gewichtet zu einer gemeinsamen Rangliste verschmolzen. Passagen, die über mehrere Wege gleichzeitig auftauchen, erhalten einen Bonus. Die Länge der Ergebnisliste wird als echtes Budget behandelt und nicht nur als feste Grenze.

**Verlässliche Vertrauenswerte.** Rohwerte der Suche werden in belastbare Wahrscheinlichkeiten umgerechnet. Statistische Verfahren liefern zudem Vertrauensbereiche mit mathematisch abgesicherter Garantie. Ein Überwachungsmodul meldet, wenn sich die Qualität der Ergebnisse im laufenden Betrieb verschiebt.

---

### 2. Gedächtnis für KI-Agenten

<img width="2752" height="1536" alt="agentic" src="https://github.com/user-attachments/assets/87780e2a-54bb-492f-817b-df9f0d67c1c3" />

Ein KI-Agent braucht ein Gedächtnis, das er pflegt, verdichtet und gezielt abruft. Contextra bietet dafür eine einfache Oberfläche und im Hintergrund die Technik, die lange Arbeitsläufe zuverlässig macht.

**Vier einfache Operationen.**

| Operation | Wirkung |
|---|---|
| `remember` | Speichert eine Erinnerung, optional mit Zusatzinformationen |
| `recall` | Ruft relevante Erinnerungen zu einer Frage ab |
| `relate` | Verknüpft zwei Erinnerungen miteinander |
| `forget` | Entfernt Erinnerungen kontrolliert |

Suche, Zusammenführung und Vertrauensbewertung laufen dabei automatisch im Hintergrund.

**Ausführung in gesicherten Schritten.** Ein Agentenlauf folgt dem Zyklus Sicherungspunkt, Ausführung, Übernahme und Protokollierung. Vor jedem Schritt entsteht automatisch ein Abbild des Zustands. Scheitert ein Schritt, kehrt der Agent zuverlässig zum letzten gültigen Zustand zurück.

**Begrenzung von Kosten und Endlosschleifen.** Ein Budget begrenzt Verbrauch und Anzahl der Schritte. Ein Agent endet dadurch auch dann, wenn er sich verrennt.

**Verdichtung statt Überlauf.** Lange Dialoge werden gekürzt, zusammengefasst oder selektiv behalten. Ein Notizbereich erlaubt dem Agenten, Zwischenstände zu bearbeiten und bei Teilzielen zu leeren. Jede Änderung wird protokolliert.

**Aufräumen und Zusammenführen.** Doppelte oder nahezu gleiche Erinnerungen werden erkannt und zusammengeführt. Eine Schutzregel verhindert, dass unterschiedliche Inhalte allein wegen einer Kette ähnlicher Einträge verschmolzen werden. Übergeordnete Wissensknoten halten auch große Wissensbestände überschaubar.

**Nachvollziehbarkeit.** Jeder Agentenschritt und jede Änderung am Gedächtnis hinterlässt eine Spur, die sich später prüfen lässt.

---

### 3. Adaptive Modellauswahl

<img width="2752" height="1536" alt="routing" src="https://github.com/user-attachments/assets/d792ecc5-2744-4010-b258-f9011030ad81" />

Nicht jede Anfrage verdient dasselbe KI-Modell. Einfache Fragen brauchen kein teures Spitzenmodell, anspruchsvolle schon. Contextra lernt aus den Ergebnissen, welches Modell für welche Art von Anfrage am besten passt.

**Lernende Auswahl.** Das System wählt je Anfrage das Modell mit dem besten erwarteten Ergebnis. Der Rechenaufwand bleibt auch bei vielen unterschiedlichen Anfragearten gering.

**Umgang mit Veränderung.** Verändert sich die Leistung eines Modells, passt sich die Auswahl an. Neue oder selten genutzte Modelle erhalten bewusst eine faire Chance, sich zu bewähren.

**Steuerung der Antwortzeit.** Die Kapazität der Modelle wird so geregelt, dass eine vorgegebene Antwortzeit eingehalten wird. Bei Lastspitzen reagiert die Regelung besonnen und schwingt nicht über.

**Gleichgewicht und Vergessen.** Die Auslastung der Modelle wird in einem stabilen Bereich gehalten. Erinnerungen verblassen über die Zeit, und das Tempo orientiert sich an der Auslastung des Systems. Die Stabilität der gesamten Regelung wird mathematisch überwacht.

---

### 4. Datenschutz, Sicherheit und Compliance

<img width="2752" height="1536" alt="security" src="https://github.com/user-attachments/assets/2c02271c-171c-4256-92fc-d43a4b98623d" />

Wissen ist wertvoll, und sein Abfluss hat Folgen. Contextra schützt an den Stellen, an denen Daten das System verlassen könnten, und dokumentiert, was dabei geschieht.

**Erkennung personenbezogener Daten.** Kennungen wie IBAN und E-Mail-Adressen werden in Texten automatisch erkannt.

**Platzhalter statt Originalwerte.** Vor einer Modellanfrage werden sensible Werte durch Platzhalter ersetzt. Nach der Antwort werden die Originalwerte wieder eingesetzt. Der Anwender erhält das vollständige Ergebnis, das Modell sieht die Originalwerte nie.

**Ausgangskontrolle.** Ausgehende Inhalte werden gegen geschützte Referenzen geprüft. Ein Frühwarnsystem erkennt ungewöhnlich große Datenmengen in kurzer Zeit, auch wenn jede einzelne Anfrage unauffällig wirkt.

**Schutz vor manipulierten Dokumenten.** Versteckte Anweisungen in Dokumenten, die einen Agenten fehlleiten sollen (sogenannte Prompt-Injection), werden erkannt und isoliert, bevor sie wirken.

**Lückenloses Protokoll.** Jede Änderung am Wissensbestand wird festgehalten. Die Einträge sind kryptografisch verkettet, sodass nachträgliche Manipulationen auffallen.

**Verschlüsselung.** Alle gespeicherten Daten sind mit AES-256 verschlüsselt, einem anerkannten Industriestandard. Die Schlüssel werden über ein mehrstufiges Verfahren abgeleitet und nach Gebrauch aus dem Arbeitsspeicher gelöscht.

**Speicherschutz.** Sensible Daten im Arbeitsspeicher werden vor dem Auslagern auf die Festplatte geschützt und beim Verlassen ihres Gültigkeitsbereichs sicher überschrieben.

**Löschen durch Schlüsselvernichtung.** Wird der Schlüssel eines Dokuments vernichtet, sind dessen Daten dauerhaft unlesbar. Der Bestand muss dafür nicht neu aufgebaut werden. Die Löschung läuft als nachvollziehbarer, mehrstufiger Vorgang mit Journal, und während der Löschung ist der Lesezugriff gesperrt.

**Löschnachweis.** Über jede Löschung entsteht ein digital signierter Bericht, der sich unabhängig prüfen lässt. Beim Einspielen eines Backups bleiben bereits gelöschte Daten gelöscht. Die rechtliche Bewertung eines Löschvorgangs obliegt dem Betreiber.

**Unterstützung bei der DSGVO-Dokumentation.** Contextra kann ein Verzeichnis der Verarbeitungstätigkeiten nach Art. 30 DSGVO exportieren und liefert Vorlagen für Auftragsverarbeitungsverträge, die auf die technischen Schutzmaßnahmen verweisen. Ein Lizenzmodul steuert, welche Funktionsbereiche freigeschaltet sind.

---

### 5. Zuverlässige Speicherung

<img width="2752" height="1536" alt="storage" src="https://github.com/user-attachments/assets/4ac5cdb6-b731-4653-af1c-4435c670a7fc" />

Ein Gedächtnis ist nur so verlässlich wie sein Speicher. Contextra setzt auf eine bewährte Speicherarchitektur mit Transaktionen, Zeitbezug und sauberer Trennung gleichzeitiger Zugriffe.

**Schneller und abgesicherter Speicherkern.** Schreibvorgänge werden zunächst im Arbeitsspeicher gesammelt und später dauerhaft abgelegt. Im Hintergrund wird der Bestand laufend aufgeräumt, damit Lesezugriffe schnell bleiben. Vorgeschaltete Filter klären meist ohne Festplattenzugriff, ob ein Eintrag existiert.

**Absturzsicheres Protokoll.** Jeder Schreibvorgang wird zuerst in ein Protokoll geschrieben, das gegen Beschädigung gesichert ist. Fehler werden dadurch erkannt.

**Alles oder nichts.** Beliebig viele Einträge werden in einer gemeinsamen Transaktion übernommen. Wie streng die Dauerhaftigkeit gesichert wird, lässt sich je Vorgang einstellen.

**Konsistenz bei gleichzeitigem Zugriff.** Jede Transaktion arbeitet auf einer konsistenten Momentaufnahme. Das System erkennt und verhindert Konflikte, bei denen sich zwei gleichzeitige Änderungen widersprechen würden.

**Zeitreisen.** Contextra unterscheidet, wann etwas erfasst wurde und wann es fachlich galt. So lässt sich abfragen, welcher Wissensstand an einem bestimmten Datum vorlag oder was in einem bestimmten Zeitraum gültig war.

**Sicherung und Wiederherstellung.** Sicherungspunkte und Snapshots sichern den Bestand. Daten werden crash-sicher auf den Datenträger geschrieben.

**Abgeschottete Werkzeuge.** Zusätzliche Werkzeuge laufen in einer isolierten Umgebung mit festen Grenzen für Rechenaufwand und Laufzeit. Freigaben erfolgen über Anfragen mit Risikobewertung.

---

### 6. Aufnahme, Modelle und Schnittstellen

<img width="2752" height="1536" alt="ingest" src="https://github.com/user-attachments/assets/18e529ac-5458-4d62-8dcb-48c962df873c" />

Wissen kommt aus Dokumenten, und Antworten entstehen mit KI-Modellen. Contextra verbindet beides und lässt sich in bestehende Werkzeuge einfügen.

**Dokumentaufnahme.** PDF, Word, Markdown, E-Mails und weitere Formate werden formatbewusst gelesen. Die Gliederung folgt der Struktur des Dokuments und ist reproduzierbar.

**Lokale KI-Modelle.** Modelle laufen direkt im eigenen Programm. Alternativ kann ein eigener Modellserver (z. B. Ollama) angebunden werden. Fällt dieser aus, fängt eine Schutzschaltung den Fehler ab.

**Effiziente Wiederverwendung.** Wiederkehrende Textanfänge werden nur einmal berechnet und von mehreren Anfragen gemeinsam genutzt. Mandanten bleiben dabei strikt getrennt. Ein Kompressionsverfahren reduziert den Speicherbedarf zusätzlich.

**Verständnis komplexer Fragen.** Ein KI-gestützter Umformulierer zerlegt komplexe Fragen in Teilfragen, die schrittweise beantwortet werden.

**Anbindung an KI-Werkzeuge.** Über das Model Context Protocol (MCP) ist Contextra mit Anwendungen wie Claude Desktop, Cursor oder VS Code nutzbar.

**Python-Schnittstelle.** Bestehende Python-Anwendungen können Contextra direkt verwenden.

**Keine verlorenen Ereignisse.** Ereignisse, die nicht verarbeitet werden konnten, werden gesondert festgehalten und können später nachbearbeitet werden.

---

## Ein Beispiel aus der Praxis

Eine Anwaltskanzlei möchte, dass ein KI-Assistent Fragen zu internen Akten beantwortet, ohne dass Mandantendaten die Kanzlei verlassen.

1. Akten (PDF, Word, E-Mails) werden in Contextra eingelesen und verschlüsselt gespeichert.
2. Ein Mitarbeiter fragt: „Welche Fristen gelten im Fall Müller?“
3. Contextra sucht gleichzeitig nach Inhalt, exakten Begriffen („Müller“, Aktenzeichen) und Zusammenhängen zwischen Personen und Ereignissen.
4. Der Assistent erhält wenige, begründete Treffer samt Vertrauenswert. Personenbezogene Daten werden bei Bedarf durch Platzhalter ersetzt.
5. Ist die Beleglage dünn, weist das System darauf hin.
6. Wird ein Mandat beendet, löscht Contextra die zugehörigen Daten durch Schlüsselvernichtung und stellt einen signierten Löschnachweis aus.

Für Entwickler genügen wenige Zeilen, um das Gedächtnis einzubinden:

```rust
use contextra::{AgentMemory, ContextraBuilder};
use contextra_types::DistanceMetric;

let db = ContextraBuilder::new(4)
    .with_storage_path("./wissen")
    .with_distance_metric(DistanceMetric::Cosine)
    .build()
    .await?;

let memory = AgentMemory::new(db);

memory.remember("Rust ist eine typsichere Systemprogrammiersprache.", None).await?;

let treffer = memory.recall("Systemprogrammiersprache", 5).await?;
```

---

## Einsatzbereiche

**Wissensassistent für Kanzleien, Verwaltungen und Beratungen.** Akten werden lokal durchsucht, Fristen und Zusammenhänge erkannt, und kein Dokument verlässt das Haus.

**Entwicklungsassistenten mit Projektgedächtnis.** Architekturentscheidungen und frühere Fehler bleiben erhalten, und die Arbeit wird nach einem Abbruch an der richtigen Stelle fortgesetzt.

**Industrie, Robotik und Geräte ohne Netzverbindung.** Wissen über Orte, Abläufe und Ereignisse bleibt lokal verfügbar, Entscheidungen fallen direkt auf dem Gerät.

**Gesundheitswesen und industrielle Systeme.** Datenschutz, Nachvollziehbarkeit und nachweisbares Löschen sind Teil der Architektur.

**Kundenservice auf Basis geprüfter Quellen.** Antworten stützen sich auf belegte Treffer. Ist die Beleglage dünn, erkennt das System dies und gibt es zurück.

**Forschung zur Suche.** Bedeutungs-, Begriffs- und Beziehungssuche lassen sich einzeln und kombiniert vergleichen, mit denselben Daten und denselben Messwerten.

---

## Technische Eckdaten

| Bereich | Merkmale |
|---|---|
| Betrieb | Vollständig lokal, ohne Cloud und ohne externen Dienst |
| Entwicklung | Programmiersprache Rust, nutzbar aus Rust und Python |
| Suche | Bedeutungs-, Begriffs- und Beziehungssuche mit gewichteter Zusammenführung und statistisch geprüften Vertrauenswerten |
| Sprache | Deutsche Sprachverarbeitung inklusive Zerlegung zusammengesetzter Wörter |
| Sicherheit | AES-256-Verschlüsselung, Erkennung personenbezogener Daten, manipulationssicheres Protokoll |
| Löschung | Schlüsselvernichtung mit digital signiertem Nachweis |
| Speicher | Transaktional, absturzsicher, mit Zeitreisen und Wiederherstellung |
| Hardware | Beschleunigung auf x86 (AVX2, AVX-512) und ARM (NEON), Standardbetrieb auf allen anderen Systemen |
| Integration | MCP-Server, Python-Schnittstelle, lokale Modelle, Ollama-Anbindung |
| Compliance | Export des Verarbeitungsverzeichnisses (Art. 30 DSGVO), Vorlagen für Auftragsverarbeitungsverträge |

---

<p align="center"><i>Lizenz: MIT oder Apache-2.0</i></p>
