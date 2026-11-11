<img width="3168" height="1344" alt="banner" src="https://github.com/user-attachments/assets/443fa9d1-7b8f-4ffe-aa61-46e85f5a414a" />

<p align="center"><b>CONTEXTRA</b></p>
<p align="center"><i>Ein lokales Wissens- und Gedächtnissystem für KI-Agenten, geschrieben in Rust</i></p>

<p align="center">
  <a href="#was-contextra-ist">Überblick</a> ·
  <a href="#die-herausforderungen-die-contextra-löst">Herausforderungen</a> ·
  <a href="#so-funktioniert-es">Funktionsweise</a> ·
  <a href="#funktionen">Funktionen</a> ·
  <a href="#ein-beispiel">Beispiel</a> ·
  <a href="#anwendungsfälle">Anwendungsfälle</a>
</p>

---

## Was Contextra ist

KI-Agenten sind nur so gut wie das Wissen, auf das sie zugreifen können. Contextra stellt dieses Wissen als Bibliothek bereit, die direkt im eigenen Programm läuft. Dokumente, Erinnerungen und Beziehungen zwischen Begriffen werden lokal gespeichert, durchsucht und zu wenigen, belastbaren Treffern verdichtet.

Es gibt keinen externen Dienst, keinen zweiten Prozess und keine Cloud-Verbindung. Daten bleiben dort, wo sie entstehen: auf dem eigenen Rechner, im eigenen Netz oder auf einem Gerät ohne Internetzugang.

Contextra vereint Verfahren, die sonst in getrennten Werkzeugen stehen: Bedeutungssuche, Volltextsuche, Graphen, Rangfusion, statistische Kalibrierung, transaktionaler Speicher, Verschlüsselung und Löschnachweise. Diese Teile arbeiten zusammen, damit ein Agent nicht nur Treffer findet, sondern auch einschätzen kann, wie belastbar sie sind.

---

## Die Herausforderungen, die Contextra löst

| Herausforderung | Wie Contextra damit umgeht |
|---|---|
| **Sensible Daten sollen das Haus nicht verlassen** | Speicherung, Suche und Auswertung laufen vollständig lokal. Ausgehende Anfragen werden auf personenbezogene Daten geprüft, und sensible Werte werden durch Platzhalter ersetzt. |
| **Löschen muss nachvollziehbar sein** | Jede Löschung erzeugt einen digital signierten Nachweis. Verschlüsselte Daten werden durch Vernichtung ihres Schlüssels unlesbar, ohne dass der gesamte Bestand neu aufgebaut werden muss. |
| **Exakte Begriffe gehen in reiner Bedeutungssuche verloren** | Vektorsuche, Volltextsuche und Graphsuche laufen parallel. Ihre Ergebnisse werden zu einer gemeinsamen Rangliste verschmolzen. |
| **Ähnlich klingende Treffer sind nicht gleich verlässlich** | Konfidenzwerte werden statistisch kalibriert, so dass Schwellenwerte eine klare Bedeutung haben. |
| **Lange Agentenläufe scheitern an einem einzigen Fehler** | Jeder Schritt wird gesichert. Bei einem Fehler wird der letzte gültige Zustand wiederhergestellt. |
| **Kontext und Kosten wachsen unkontrolliert** | Verläufe werden verdichtet, und ein Token-Budget begrenzt Schritte und Verbrauch. |
| **Nicht jede Anfrage braucht das größte Modell** | Ein kontextueller Bandit wählt je Anfrage das Modell mit dem besten Verhältnis aus Qualität, Latenz und Kosten. |

---

## So funktioniert es

Contextra arbeitet in fünf Stufen. Jede Stufe ist für sich verständlich, und zusammen ergeben sie einen durchgehenden Weg von der Quelle bis zur Antwort.

**1. Aufnehmen.** Dokumente in Formaten wie PDF, DOCX, Markdown oder E-Mail werden gelesen. Der Text wird entlang seiner Struktur in Abschnitte zerlegt, sodass Überschriften, Listen und Absätze erhalten bleiben.

**2. Speichern.** Die Abschnitte landen in einem transaktionalen Speicher. Schreibvorgänge sind atomar, und ein Protokoll sichert sie gegen Abstürze. Die Daten liegen verschlüsselt auf dem Datenträger.

**3. Auf mehreren Wegen suchen.** Eine Frage wird gleichzeitig an drei Suchwege gestellt. Die Bedeutungssuche findet inhaltlich verwandte Passagen, die Volltextsuche findet exakte Begriffe und Zahlen, und die Graphsuche folgt Beziehungen zwischen Personen, Begriffen und Ereignissen.

**4. Zusammenführen und prüfen.** Die drei Trefferlisten werden mit gewichteter Rangfusion verschmolzen. Treffer, die über mehrere Wege bestätigt werden, erhalten einen Bonus. Statistische Kalibrierung macht die Konfidenz der Ergebnisse vergleichbar.

**5. Liefern.** Der Agent erhält wenige, begründete Treffer. Ist die Beleglage dünn, wird das ausgewiesen, statt eine sichere Antwort vorzutäuschen.

```
   Dokument oder Erinnerung
            │
            ▼
   ┌──────────────────────┐     ┌──────────────────────────────┐
   │ Aufnehmen            │ ──▶ │ Speichern                    │
   │ PDF · DOCX · MD · Mail│    │ verschlüsselt · transaktional│
   └──────────────────────┘     └──────────────┬───────────────┘
                                               │
        ┌──────────────────┬──────────────────┼──────────────────┐
        ▼                  ▼                  ▼                  ▼
  Bedeutungssuche    Volltextsuche      Graphsuche         Zeitbezug
  (Vektoren)         (BM25, Deutsch)    (Beziehungen)      (wann gültig)
        └──────────────────┴─────────┬────────┴──────────────────┘
                                     ▼
                     Rangfusion · Kalibrierung · Drift-Erkennung
                                     │
                                     ▼
                     Wenige, begründete Treffer für den Agenten
```

---

## Funktionen

### 1. Hybride Suche und Retrieval

<img width="2752" height="1536" alt="hybrid-retrieval" src="https://github.com/user-attachments/assets/0c24b7b2-7037-42f2-98a0-b439c33c83c7" />

Contextra kombiniert Bedeutung, exakte Begriffe und Beziehungen. Jede Suchart deckt Fälle ab, die die anderen übersehen würden.

**Vektorsuche.** Passagen werden über Embeddings verglichen. Ein HNSW-Graph findet die nächsten Nachbarn auch in großen Beständen in Millisekunden. Die skalare Quantisierung (SQ8) speichert jede Dimension mit 8 statt 32 Bit, der Speicherbedarf der Vektoren sinkt dadurch auf ein Viertel. Für sehr große Bestände steht ein Out-of-Core-Index (DiskANN) bereit, der nicht vollständig im Arbeitsspeicher liegen muss. Binäre Quantisierung (RaBitQ) und filtergestützte Suche (Acorn) erweitern das Spektrum für spezielle Lasten.

**Distanzberechnung in Hardware.** Kosinus-, Euklid- und Skalarproduktdistanz sind mit SIMD-Kernen für AVX2, AVX-512 und ARM NEON umgesetzt. Auf Systemen ohne diese Befehlssätze greift eine saubere Referenzimplementierung.

**Volltextsuche.** BM25 bewertet Dokumente nach Termhäufigkeit und Seltenheit. Die Suche nutzt Block-Max-WAND, um unwichtige Trefferblöcke früh zu überspringen. Die deutsche Sprachbehandlung zerlegt Komposita wie *Datenschutzfolgenabschätzung*, normalisiert Umlaute und filtert Stoppwörter. Der invertierte Index arbeitet mit Snapshots, sodass Lesezugriffe nicht auf Schreibvorgänge warten.

**Graphsuche.** Beziehungen zwischen Entitäten bilden einen Graphen, der kompakt in einer CSR-Struktur liegt. Personalisierter PageRank mit Forward-Push (nach Andersen, Chung und Lang) zeigt, welche Knoten rund um eine Frage relevant sind. Die Pfadsuche PathRAG sammelt mehrere begründete Pfade unter einem festen Knotenbudget. Diffusionsverfahren über Hyperkanten (APPRH, TL-HFD) erweitern die Suche auf Beziehungen, die mehr als zwei Entitäten verbinden.

**Gemeinschaften im Wissen.** Community-Erkennung gruppiert zusammengehörige Begriffe. Perkolationsschwellen zeigen, wann ein Teilgraph zerfällt, und dynamisches Kanten-Reinforcement stärkt Beziehungen, die oft gemeinsam genutzt werden.

**Rangfusion.** Die Ergebnisse der Suchwege werden mit gewichteter Reciprocal Rank Fusion (RRF) verschmolzen. Varianten wie mRRF passen die Gewichte an den Abstand zwischen den Treffern an. Ein Resonanz-Bonus belohnt Passagen, die über mehrere Kanäle gleichzeitig auftauchen. Der Ansatz DiBud behandelt die Länge der Ergebnisliste als echtes Budget, statt sie nur als feste Schwelle zu nutzen.

**Kalibrierung.** Rohwerte werden in verlässliche Wahrscheinlichkeiten überführt, mit isotonischer Regression (PAVA), Platt-Skalierung oder konformer Vorhersage. Konforme Verfahren liefern Konfidenzbereiche mit einer statistischen Garantie für endliche Stichproben. Ein Drift-Monitor beobachtet die Verteilung der Scores zur Laufzeit und meldet, wenn sie sich verschiebt.

---

### 2. Autonome Agenten und Gedächtnis

<img width="2752" height="1536" alt="agentic" src="https://github.com/user-attachments/assets/87780e2a-54bb-492f-817b-df9f0d67c1c3" />

Ein Agent braucht ein Gedächtnis, das er pflegt, verdichtet und wieder hervorholt. Contextra stellt dafür eine einfache Oberfläche bereit und darunter die Mechanik, die lange Läufe zuverlässig macht.

**Gedächtnis in vier Operationen.** `remember` speichert eine Erinnerung mit optionalen Metadaten. `recall` findet relevante Erinnerungen zu einer Frage. `relate` verbindet zwei Erinnerungen, und `forget` entfernt sie kontrolliert. Die Suchwege, die Rangfusion und die Kalibrierung bleiben dabei im Hintergrund.

**Ausführung in Schritten.** Ein Agentenlauf folgt dem Zyklus Checkpoint, Ausführung, Commit und Audit. Vor jedem Schritt entsteht automatisch ein Zustandsabbild. Scheitert ein Schritt, kehrt der Agent deterministisch zum letzten gültigen Zustand zurück.

**Begrenzung von Kosten und Schleifen.** Ein Token-Budget begrenzt Verbrauch und Anzahl der Schritte. So endet ein Agent auch dann, wenn er sich verrennt.

**Verdichtung statt Überlauf.** Lange Dialoge und Verläufe werden über Strategien wie Kürzen, Zusammenfassen oder selektives Behalten verdichtet. Ein Kontext-Scratchpad erlaubt dem Agenten, Zwischenstände zu bearbeiten und bei Teilzielen zu leeren. Jede Änderung wird protokolliert.

**Konsolidierung.** Beinahe-Duplikate werden erkannt und zusammengeführt. Schiebefenster-Clustering findet thematische Gruppen. Ein Transitivitäts-Veto verhindert, dass zwei Erinnerungen allein wegen einer Kette von Ähnlichkeiten verschmolzen werden.

**Aggregation nach LeanRAG.** Die dritte Stufe von LeanRAG gruppiert Wissen mit einem Gauß-Mischmodell, bereinigt Typen und erzeugt übergeordnete Knoten. Große Wissensnetze bleiben dadurch überschaubar.

**Nachvollziehbarkeit.** Jeder Agentenschritt und jede Änderung am Gedächtnis hinterlässt eine Spur, die sich später nachvollziehen lässt.

---

### 3. Adaptive Steuerung und Modellauswahl

<img width="2752" height="1536" alt="routing" src="https://github.com/user-attachments/assets/d792ecc5-2744-4010-b258-f9011030ad81" />

Nicht jede Anfrage verdient dasselbe Modell. Contextra lernt aus den Ergebnissen, welches Modell für welche Art von Frage am besten passt, und hält die Auslastung der Modellpools im Gleichgewicht.

**Kontextuelle Bandits.** LinUCB wählt je Anfrage das Modell mit dem besten Erwartungswert. Die Matrixaktualisierung nach Sherman-Morrison hält die Berechnung schnell, und ein sketchbasierter Ansatz reduziert den Speicherbedarf bei vielen Kontexten.

**Umgang mit Veränderung.** Flow-Corrected Thompson Sampling gleicht Leistungsdrift aus, wenn sich die Güte eines Modells über die Zeit ändert. Regularisierungsinduzierte Exploration (RIE-Greedy) sorgt dafür, dass neue oder selten genutzte Modelle eine faire Chance bekommen.

**Latenzregelung.** Ein PID-Regler passt die Größe der Modellpools an eine Zielantwortzeit an. Eine Anti-Windup-Logik verhindert, dass der Regler bei Lastspitzen überreagiert.

**Gleichgewicht und Abklingen.** Ein Homöostat hält die Auslastung der Pools in einem Korridor. Ein Decay-Controller lässt Erinnerungen mit einer Halbwertszeit verblassen, die sich an der Auslastung des Systems orientiert. Ein Lyapunov-basiertes Kriterium überwacht die Stabilität der Regelung.

---

### 4. Datenschutz, Sicherheit und Compliance

<img width="2752" height="1536" alt="security" src="https://github.com/user-attachments/assets/2c02271c-171c-4256-92fc-d43a4b98623d" />

Wissen ist wertvoll, und sein Abfluss hat Folgen. Contextra schützt an den Stellen, an denen Daten die Maschine verlassen könnten, und dokumentiert, was dabei geschieht.

**Erkennung personenbezogener Daten.** Regelbasierte und wortlistengestützte Verfahren erkennen IBAN, E-Mail-Adressen und weitere Kennungen in Texten.

**Ersetzen statt Weitergeben.** Sensible Werte werden vor einer Modellanfrage durch deterministische Platzhalter ersetzt. Die Antwort des Modells wird danach zurückübersetzt, sodass der Nutzer das vollständige Ergebnis erhält, das Modell aber nie den Originalwert sieht.

**Ausgangskontrolle.** Ein k-NN-Wächter vergleicht ausgehende Inhalte mit geschützten Referenzen. Ein Detektor für Massenabfluss erkennt ungewöhnlich große Datenmengen in kurzer Zeit, auch wenn jede einzelne Anfrage unauffällig wirkt.

**Abwehr von Prompt-Injection.** Versteckte Anweisungen in Dokumenten werden über Muster und Phrasen erkannt und in Quarantäne gesetzt, bevor sie einen Agenten beeinflussen.

**Nachvollziehbare Änderungen.** Jede Änderung am Kontext wird in einem Audit-Protokoll festgehalten. Die Einträge sind mit BLAKE3 kryptografisch verkettet, sodass nachträgliche Manipulation auffällt.

**Verschlüsselung.** Daten liegen auf dem Datenträger in AES-256-GCM-SIV. Schlüssel werden über eine hierarchische Ableitung mit Argon2id und HKDF erzeugt. Schlüsselmaterial wird nach Gebrauch aus dem Speicher gelöscht.

**Speicherschutz.** Sensible Puffer werden mit `mlock` vor dem Auslagern auf die Festplatte geschützt. Flüchtige Werte im Arbeitsspeicher werden beim Verlassen ihres Gültigkeitsbereichs sicher überschrieben.

**Löschen durch Schlüsselvernichtung.** Wird der Schlüssel eines Dokuments vernichtet, sind seine verschlüsselten Daten dauerhaft unlesbar. Ein Neuaufbau des Index entfällt. Die Löschung läuft als nachvollziehbare Saga mit definierten Zuständen, einem Journal und einer Sperre für Lesezugriffe.

**Löschnachweis.** Über jede Löschung entsteht ein mit Ed25519 signierter Bericht. Er enthält Zeugen aus der Messung und lässt sich unabhängig prüfen. Beim Einspielen eines Backups bleiben bereits gelöschte Daten gelöscht. Die rechtliche Bewertung eines Löschvorgangs obliegt dem Betreiber.

**Lizenzschicht und Vorlagen.** Ein Lizenz-Gate steuert freigeschaltete Funktionsringe. Ein Export erstellt ein Verzeichnis der Verarbeitungstätigkeiten nach Art. 30 DSGVO, und ein Generator liefert Vorlagen für Auftragsverarbeitungsverträge, die auf die technischen Garantien verweisen.

---

### 5. Speicher, Transaktionen und Isolation

<img width="2752" height="1536" alt="storage" src="https://github.com/user-attachments/assets/4ac5cdb6-b731-4653-af1c-4435c670a7fc" />

Ein Gedächtnis ist nur so verlässlich wie sein Speicher. Contextra baut auf einem Speicherkern im LSM-Stil auf, mit Transaktionen, Zeitbezug und sauberer Isolation.

**LSM-Speicherkern.** Schreibvorgänge landen zunächst in einer MemTable und werden später in unveränderlichen SSTables abgelegt. Hintergrund-Kompaktierung hält die Lesewege kurz. Zweistufige Bloom-Filter entscheiden meist ohne Festplattenzugriff, ob ein Schlüssel existiert.

**Write-Ahead-Log.** Jeder Schreibvorgang wird zuerst in ein crash-konsistentes Protokoll geschrieben. Die Einträge sind mit HMAC-Ketten gesichert, sodass Beschädigungen erkannt werden.

**Atomare Gruppenschreibungen.** Beliebig viele Einträge werden unter einer gemeinsamen Transaktionsnummer in einem Block festgeschrieben. Die Haltbarkeit lässt sich pro Batch fein einstellen.

**Serializable Snapshot Isolation.** Transaktionen laufen auf konsistenten Momentaufnahmen. Die Validierung erkennt Write-Skew und verhindert, dass zwei gleichzeitige Änderungen einander widersprechen. Ein SequenceLog und Read-Sets erfassen, was jede Transaktion gelesen und geschrieben hat.

**Zeitreisen.** Contextra führt Transaktionszeit und fachliche Gültigkeitszeit getrennt. Abfragen lassen sich so stellen, wie das Wissen zu einem bestimmten Zeitpunkt aussah oder wie es für einen bestimmten Zeitraum galt.

**Sicherung und Wiederherstellung.** Checkpoints und Snapshots sichern den Bestand. Ein dauerhaftes Dateisystem-Layer schreibt Daten crash-sicher und synchronisiert sie auf den Datenträger.

**Isolierte Werkzeuge.** Werkzeuge laufen in einer WebAssembly-Sandbox. Ein Fuel-Budget begrenzt Rechenschritte und Laufzeit, und Freigaben werden über Anfragen mit Risikobewertung gesteuert.

---

### 6. Aufnahme, Modelle und Schnittstellen

<img width="2752" height="1536" alt="ingest" src="https://github.com/user-attachments/assets/18e529ac-5458-4d62-8dcb-48c962df873c" />

Wissen kommt aus Dokumenten, und Antworten entstehen mit Modellen. Contextra verbindet beides und lässt sich in bestehende Werkzeuge einfügen.

**Dokumentaufnahme.** PDF, DOCX, Markdown, E-Mail und weitere Formate werden formatbewusst gelesen. Das Chunking folgt der Struktur des Dokuments und ist deterministisch: derselbe Eingang ergibt immer dieselben Abschnitte.

**Lokale Modelle.** Inferenz läuft direkt im Prozess über Candle mit GGUF-Modellen. Alternativ übernimmt ONNX Runtime Embeddings und Cross-Encoder-Reranking. Für Ollama-Server gibt es eine Anbindung mit Circuit-Breaker, der Ausfälle abfängt.

**Wiederverwendung im KV-Cache.** Wiederkehrende Textanfänge teilen sich ihren Rechenstand über einen Präfixbaum. Mandanten bleiben dabei streng getrennt. Die Quantisierung nach KIVI reduziert den Speicherbedarf der Zwischenergebnisse.

**Anfragen verstehen.** Ein LLM-gestützter Query-Rewriter formuliert Anfragen so um, dass sie in mehreren Schritten beantwortet werden können. Komplexe Fragen werden in Teilfragen zerlegt.

**Model Context Protocol.** Der MCP-Server macht Contextra für IDEs und Agentenumgebungen nutzbar, etwa für Claude Desktop, Cursor oder VS Code. Die Kommunikation läuft über JSON-RPC 2.0 über Standardeingabe und -ausgabe.

**Python.** Über PyO3 lässt sich Contextra aus Python heraus nutzen, ohne den Kern neu zu schreiben.

**Ereignisgesteuerte Agenten.** Eine Dead-Letter-Queue fängt Ereignisse auf, die nicht verarbeitet werden konnten, sodass nichts verloren geht.

---

## Ein Beispiel

Ein Gedächtnis in wenigen Zeilen. Die Suchwege, die Rangfusion und die Kalibrierung laufen im Hintergrund.

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

## Anwendungsfälle

**Wissensassistent für Kanzleien und Verwaltungen.** Akten werden lokal durchsucht, Fristen und Zusammenhänge erkannt, und kein Dokument verlässt das Haus.

**Coding-Agenten mit Projektgedächtnis.** Der Agent behält Architekturentscheidungen, merkt sich frühere Fehler und setzt nach einem Abbruch an der richtigen Stelle fort.

**Robotik und Edge-Geräte.** Ein Gerät ohne Netzverbindung behält Wissen über Orte, Abläufe und Ereignisse und entscheidet lokal.

**Medizinische und industrielle Systeme.** Datenschutz, Nachvollziehbarkeit und nachweisbares Löschen sind Teil der Architektur.

**Forschung zur Suche.** Graph-, Vektor- und Volltextsuche lassen sich einzeln und zusammen vergleichen, mit denselben Daten und denselben Messwerten.

**Wissensbasierter Kundenservice.** Antworten stützen sich auf geprüfte Treffer. Ist die Beleglage dünn, erkennt das System das und gibt es zurück.

---

## Bausteine im Überblick

| Bereich | Bausteine |
|---|---|
| Suche | HNSW, SQ8, DiskANN, RaBitQ, Acorn, SIMD-Distanzen, BM25, Block-Max-WAND, deutsche Morphologie, Graph-PPR, PathRAG, APPRH, TL-HFD, Community-Erkennung |
| Fusion und Kalibrierung | RRF, gewichtete RRF, mRRF, Resonanz-Bonus, DiBud, isotonische Regression, Platt-Skalierung, konforme Vorhersage, Drift-Monitor |
| Gedächtnis und Agenten | AgentMemory, Checkpoints, Rollback, Token-Budget, Konsolidierung, Verdichtung, Scratchpad, LeanRAG, Transitivitäts-Veto |
| Steuerung | LinUCB, FC-TS, RIE-Greedy, PID-Regler, Homöostat, Decay-Controller, Lyapunov-Kriterium |
| Schutz | Erkennung personenbezogener Daten, Platzhalter-Vault, Ausgangsprüfung, Massenabfluss-Detektor, Prompt-Injection-Schutz, BLAKE3-Audit |
| Verschlüsselung und Löschen | AES-256-GCM-SIV, Argon2id, HKDF, Schlüsselvernichtung, Löschsaga, Ed25519-Löschnachweis, Restore-Schutz, mlock, Zeroize |
| Speicher | LSM-Baum, SSTables, Bloom-Filter, WAL mit HMAC, MVCC mit SSI, Zeitreisen, Checkpoints, dauerhaftes Dateisystem |
| Isolation | WASM-Sandbox mit Fuel-Budget, Freigaben mit Risikobewertung |
| Aufnahme und Modelle | PDF, DOCX, Markdown, E-Mail, Chunking, Candle, ONNX, Ollama, KV-Cache mit Präfixbaum, KIVI |
| Integration | MCP-Server, Python-Bindings, Dead-Letter-Queue, Lizenz-Gate, Verarbeitungsverzeichnis, AVV-Vorlagen |

---

<p align="center"><i>Lizenz: MIT oder Apache-2.0</i></p>
