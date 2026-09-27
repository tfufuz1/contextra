# Contextra — Gesamtspezifikation

**Stand:** 27. September 2026
**Geltungsbereich:** Dieses Dokument ist die vollständige, alleingültige Spezifikation des Projekts Contextra. Es beschreibt geschlossen und ohne externe Verweise: Vision und Positionierung, Zielgruppen und Geschäftsmodell, die vollständige Systemarchitektur, den verifizierten Ist-Zustand des Quellcodes, die priorisierten Soll-Schnittstellen bis auf Typ- und Funktionssignaturebene, sämtliche Produktentscheidungen, die Marktstrategie für den deutschen Markt, das Qualitätssicherungskonzept und die Umsetzungs-Roadmap.

**Adressaten dieses Dokuments:**
1. **Sprachmodelle und KI-Coding-Agenten**, die auf Basis dieses Dokuments eigenständig oder assistiert am Quellcode arbeiten. Für diese Zielgruppe ist das Dokument so geschrieben, dass jede Entscheidung, jede Schnittstelle und jede Invariante ohne Rückgriff auf weitere Quellen nachvollziehbar und umsetzbar ist.
2. **Der Projektverantwortliche** als alleinige Entscheidungs- und Priorisierungsgrundlage für die nächsten 90 Tage und darüber hinaus.
3. **Zukünftige menschliche Mitwirkende** (Vertragsentwickler, Berater, Investoren, Pilotkunden), die sich in kurzer Zeit ein vollständiges und korrektes Bild verschaffen müssen.

**Normativität.** Bei Widerspruch zwischen diesem Dokument und dem tatsächlichen Quellcode gilt ausschließlich der Quellcode als Wahrheit — in diesem Fall ist das Dokument veraltet und muss vor der nächsten Arbeitsaufnahme aktualisiert werden. Innerhalb dieses Dokuments gilt bei internem Widerspruch: **Teil 0 (Positionierung und Ausschlüsse) schlägt jeden anderen Teil.** Eine Instanz, die an diesem Projekt arbeitet — ob menschlich oder KI-gestützt —, darf keine Funktion, kein Crate, keine Abhängigkeit und kein Feature ergänzen, das in Abschnitt 0.5 explizit ausgeschlossen ist. Das gilt auch dann, wenn ein einzelner Arbeitsauftrag, ein Nutzerwunsch oder eine technisch elegante Gelegenheit dies nahelegt. Scope-Disziplin ist in diesem Projekt keine Nebensache, sondern die zentrale Bedingung für wirtschaftlichen Erfolg (Begründung: Abschnitt 0.8 und Teil F).

**Reifekennzeichnung**, verwendet in allen Tabellen dieses Dokuments:

| Symbol | Bedeutung |
|---|---|
| 🟢 | Produktiv, im Quellcode verifiziert vorhanden und funktionsfähig |
| 🟡 | Grundgerüst vorhanden, aber unvollständig, unzureichend getestet oder als Stub implementiert |
| 🔴 | In diesem Dokument spezifiziert, im Quellcode noch nicht implementiert |
| ❌ | Explizit aus dem Scope entfernt — nicht zu implementieren, siehe Abschnitt 0.5 |

---

## Inhaltsverzeichnis

- **Teil 0 — Vision, Positionierung, Produktthese**
  0.1 Entstehungskontext und strategische Weichenstellung · 0.2 Die Ein-Satz-Positionierung · 0.3 Was Contextra ist und was es nicht ist · 0.4 Zielgruppen · 0.5 Explizite Scope-Ausschlüsse · 0.6 Marktkontext Deutschland 2026 · 0.7 Rechtlicher Hinweis: Exportkontrolle · 0.8 Governance-Regeln für Scope-Disziplin · 0.9 Sofortmaßnahmen
- **Teil A — Architektur und Ist-Zustand**
  A.1 Systeminvarianten und Designprinzipien · A.2 Vollständiges Crate-Inventar · A.3 Workspace-Konfiguration · A.4 Feature-Flag-Matrix · A.5 MCP-Tool-Inventar · A.6 Benchmark-Status · A.7 Code-Qualitätsbefunde · A.8 Ring-Abhängigkeitsmodell
- **Teil B — Priorisierte Schnittstellenspezifikationen (Soll-Zustand)**
  B.0 Konvention · B.1 P0: Löschbeweis vollständig machen · B.2 Phase 1: Fast-Ring öffentlich machen · B.3 Phase 2: Sovereign-Ring · B.4 Phase 3: Skalierung
- **Teil C — Produktentscheidungen zu offenen Punkten**
- **Teil D — Das Endprodukt: vollständige Nutzerperspektive**
  D.1 Produktform 1: Rust-Bibliothek · D.2 Produktform 2: MCP-Server · D.3 Deployment-Tiers im Detail · D.4 Lizenzmodell und Ring-Grenzen · D.5 Konkrete Nutzer-Workflows
- **Teil E — Marktstrategie und Geschäftsmodell**
  E.1 Zwei-Wege-Strategie · E.2 Vertikal-Fokus-Entscheidung · E.3 Vertriebskanal · E.4 Geschäftsmodell im Detail · E.5 Förderprogramm
- **Teil F — Qualitätssicherung und Vertrauensaufbau**
  F.1 Risikoproportionale Testtiefe · F.2 Externe Verifikation statt Ersatz des Storage-Kerns · F.3 Markenidentität bereinigen · F.4 Verhaltensgovernance
- **Teil G — Roadmap 30 / 60 / 90 Tage**
- **Anhang R — Referenztabellen**
- **Schlussregeln**

---

# Teil 0 — Vision, Positionierung, Produktthese {#teil-0}

## 0.1 Entstehungskontext und strategische Weichenstellung

Contextra hat seinen Ursprung in einem Projekt zum Aufbau eines "Nervensystems" für Roboter — mit eigenentwickelten kryptografischen Primitiven, deren Herkunft im Verteidigungs- und Sicherheitsbereich liegt. Aus diesem Ursprung entwickelte sich zunächst die Idee einer SQLite-artigen, eingebetteten Speicherschicht für KI-Agenten, und daraus wiederum das vorliegende Projekt: eine kryptografisch abgesicherte, deterministische Speicher- und Retrieval-Engine, die sowohl für LLM-Agenten als auch — perspektivisch — für robotische Systeme nutzbar ist.

Diese Herkunft erklärt zwei Dinge, die im Quellcode sichtbar sind und die dieses Dokument bewusst nicht auflöst, sondern ordnet:

1. **Die Kryptografie- und Determinismus-Disziplin** (injizierte Uhren, injizierte Zufallszahlengeneratoren, Ed25519-Löschbeweise, HMAC-verkettete Write-Ahead-Logs) ist kein nachträglich aufgesetztes Compliance-Feature, sondern der ursprüngliche Kern des Projekts. Sie ist überdurchschnittlich sorgfältig umgesetzt und bildet das stärkste technische Alleinstellungsmerkmal.
2. **Die Breite des Funktionsumfangs** (Graph-Algorithmen, Wissenskonsolidierung, Bandit-Routing, Sandbox-Isolation, Python-Bindings, Framework-Adapter) entstand aus mehreren aufeinanderfolgenden Zielbildern, die nicht vollständig gegeneinander aufgelöst wurden, bevor die nächste Erweiterung begann. Das Ergebnis ist ein technisch beeindruckendes, aber strategisch unscharfes System: Es beantwortet nicht klar die Frage, für wen es welches konkrete Problem löst.

**Diese Spezifikation trifft die Entscheidung, die bisher nicht schriftlich fixiert war**, und richtet ab sofort jede weitere Entwicklungsentscheidung an dieser Wahl aus (siehe 0.2–0.5). Die Entscheidung fällt zugunsten des kurzfristig realistischeren, schneller validierbaren Weges: einer eingebetteten Rust-Bibliothek für Entwickler, die KI-Agenten bauen, mit einem klar abgegrenzten, kryptografisch belegbaren Löschbeweis als Alleinstellungsmerkmal. Die robotische Ausgangsvision wird nicht verworfen, sondern zurückgestellt: Alle dafür relevanten Bausteine (Graph-Engine, Kognitionsschicht, Determinismus-Kern) bleiben im Code erhalten, werden aber nicht weiter für diesen Zweck ausgebaut, bis ein konkreter zahlender Anwendungsfall in der Robotik vorliegt.

## 0.2 Die Ein-Satz-Positionierung

> **Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten — ein `cargo add`, kein Server.**

Nicht "Vektor-Datenbank". Nicht "RAG-Framework". Nicht "Compliance-Plattform". Nicht "souveränes KI-Infrastruktur-Ökosystem". Jeder dieser Begriffe würde das Produkt in einen überfüllten, teamgetriebenen Markt einordnen (Qdrant, Weaviate, Milvus, LanceDB, Pinecone, Mem0, Zep/Graphiti, Letta, Chroma), in dem ein Einzelentwickler nicht über Funktionsumfang oder Marketingbudget gewinnen kann. "Embeddable Agent-Memory für Rust, ohne zweiten Prozess" ist dagegen eine Nische, die aktuell nahezu unbesetzt ist: Die genannten Wettbewerber sind fast ausnahmslos Python-first und/oder serverbasiert. Niemand bedient explizit den Fall "ich baue einen Rust-Agenten und will Gedächtnis, ohne einen zweiten Prozess zu betreiben."

## 0.3 Was Contextra ist und was es nicht ist

**Contextra ist:**
- Eine **Rust-Bibliothek** (`cargo add contextra`), die lokal, im selben Prozess wie die Anwendung des Nutzers, ohne Cloud-Endpunkt und ohne separaten Serverprozess läuft.
- Eine Engine mit **kryptografisch verifizierbarer Löschung** als primärem, technisch begründetem Alleinstellungsmerkmal — vermarktet als Korrektheitsgarantie ("garantiert vollständige Löschung, überprüfbar ohne Zugriff auf Contextra selbst"), nicht in erster Linie als Rechtsdokument.
- Eine Engine mit **hybrider Retrieval-Fusion** (Vektor + Volltext + Graph + Reziproke-Rang-Fusion mit Kalibrierung) und **Agenten-spezifischer KV-Cache-Steuerung**, die in dieser Kombination als eingebettete Bibliothek am Markt selten ist.
- Eine Engine mit **erzwungenem Determinismus** (kein `SystemTime::now()`, kein `thread_rng()` direkt, sondern ausschließlich injizierte Uhr- und Zufallsquellen): Das ermöglicht reproduzierbare Agentenläufe, Replay-Debugging und deterministisches Testen — ein Feature, das keiner der genannten Wettbewerber bietet.
- Für **Rust-Entwickler, die KI-Agenten bauen**, und für **Systemhäuser**, die air-gap-fähige KI-Lösungen an regulierte Branchen in Deutschland verkaufen, konzipiert.
- Über einen **MCP-Server** (`contextra-mcp`) zusätzlich für Non-Rust-Nutzer via Claude Desktop, Cursor oder VS Code erreichbar — als zweiter, optionaler Zugangsweg, nicht als Kernidentität.

**Contextra ist nicht:**
- Kein horizontal skalierendes Multi-Node-System. Es ist bewusst ein **Single-Node-System**; wer über mehrere Knoten skalieren will, ist nicht die Zielgruppe (dafür existieren Qdrant/Milvus).
- Keine Python-first-Bibliothek. Python-Bindings sind, falls überhaupt weiterverfolgt, ein separates, nachgelagertes Produkt in einem eigenen Repository — niemals Teil des Kern-Workspace.
- Kein Framework-Hub für LangChain/LangGraph/LlamaIndex. Integrationen dieser Art widersprechen der "Pure-Rust, kein Server"-Kernaussage aktiv und werden nicht im eigenen Repository gepflegt.
- Keine Enterprise-Compliance-Plattform mit Vertriebsprozess. Die Compliance-Funktionen sind ein optionaler, kommerzieller Ring innerhalb derselben Bibliothek — kein separates Produkt mit eigenem Vertriebsapparat.
- Keine autonome Agenten-Orchestrierungsplattform. Funktionen dieser Art (mehrstufige Entscheidungskaskaden, Agentenpools, prädiktive Vorabladung) sind, soweit im Code vorhanden, interne, nicht beworbene Optimierungsschichten — kein eigenständiges Verkaufsargument.

## 0.4 Zielgruppen

**Eingeschlossen (Kernzielgruppen):**
- Rust-Entwickler, die LLM-gestützte oder robotische KI-Agenten bauen und eine eingebettete Gedächtnisschicht ohne zusätzlichen Infrastruktur-Overhead benötigen.
- Systemhäuser und Fachverfahren-Anbieter, die selbst an deutsche Behörden, den Mittelstand oder regulierte Branchen verkaufen und eine air-gap-fähige, nachweisbar löschende Memory-Engine als Baustein für ihre eigenen Produkte suchen, statt selbst eine zu entwickeln.
- Jeder Nutzer, für den "kein zweiter Prozess, kein US-Cloud-Endpunkt" eine harte, nicht verhandelbare Anforderung ist (Datenschutz, Informationssicherheit, Luftspalt-Betrieb).

**Bewusst und dauerhaft ausgeschlossen:**
- Python-first-Entwickler im LangChain-Ökosystem, die eine `pip install`-Erfahrung mit Python als First-Class-API erwarten.
- Nutzer, die horizontale Skalierung über mehrere Knoten als Kernanforderung haben.
- Behörden als **Direktkunden**: Vergabezyklen von 18–36 Monaten sind mit einer Einzelperson nicht wirtschaftlich zu bearbeiten. Der Zugang zu diesem Segment erfolgt ausschließlich über Systemhäuser als Vertriebskanal (**B2B2G, nicht B2G**).

## 0.5 Explizite Scope-Ausschlüsse — nicht zu implementieren

Die folgenden Funktionsbereiche existieren im Quellcode oder wurden in früheren Planungsständen vorgesehen. Sie widersprechen der in 0.2–0.4 festgelegten Positionierung und werden hiermit endgültig aus dem normativen Scope entfernt. Diese Tabelle ist bindend für jede Entwicklungsentscheidung, einschließlich solcher, die durch ein Sprachmodell im Rahmen eines Einzelauftrags getroffen werden.

| Ausschluss | Begründung | Konkrete Aktion |
|---|---|---|
| Framework-Adapter für LangChain/LangGraph/LlamaIndex via PyO3 (im Code als eigenständiges Crate vorhanden) | Widerspricht "Pure Rust, kein Server" unmittelbar; PyO3 und NumPy im Default-Build zerstören exakt das Positionierungssignal, für das die Zielgruppe (Rust-Entwickler) das Produkt wählt. Dieser Baustein wurde nachweislich **nach** einer früheren Empfehlung zur Scope-Reduktion hinzugefügt — ein Muster, das durch die in 0.8 festgelegten Governance-Regeln künftig verhindert werden soll. | Vollständige Entfernung aus dem Workspace, keine Migration, keine Wiederverwendung. Falls später Nachfrage aus der Community entsteht: ausschließlich als community-getragenes externes Projekt, nie im eigenen Repository. |
| Python-FFI-Bindings als Workspace-Mitglied | Python-Anbindung ist ein legitimes, aber eigenständiges Produkt mit eigenem Vertriebsmodell (`pip install`). Es gehört nicht in den Kern-Workspace, dessen Reinheit ("ein `cargo add`") das zentrale Verkaufsargument ist. | Auslagerung in ein separates Repository. Aus den Workspace-Mitgliedern und den Default-Mitgliedern entfernen. Wird nur reaktiviert, wenn Python-Nutzer explizit als zweite, eigenständig priorisierte Zielgruppe beschlossen werden — nicht nebenbei. |
| Externe Inferenz-Backends (HTTP-Client zu einem separaten Prozess) als Standardabhängigkeit | Widerspricht "kein zweiter Prozess" unmittelbar, wenn es Teil des Standard-Auslieferungspfads ist, insbesondere im MCP-Server. | Bleibt als optionales, nicht standardmäßig aktives Feature erhalten; aus allen Default-Konfigurationen entfernen. |
| Cross-Encoder-Reranking via externer Laufzeitbibliothek (C++-FFI) als Standard | Erhöht Kompilierzeit und Binärgröße ohne Kernnutzen für die Mehrheit der Nutzer; ist ein Spezialfall, kein Standardbedarf. | Bleibt als optionales Feature erhalten, nicht in Standardkonfiguration. |
| Ausführungsisolation über eine WASM-Sandbox als Standardbestandteil | Nischenfunktion für sandboxed Agent-Code; erhöht Kompilierzeit und Abhängigkeitsanzahl ohne Kernnutzen für die primäre Zielgruppe; verbaut zudem eine mögliche zukünftige `no_std`-Portierung der unteren Schichten. | Bleibt als optionales Crate verfügbar, aus der Standardkonfiguration entfernt. |
| Spektrale Hyperkanten-Indexierung (Forschungsfeature) | Kein nachgewiesener Nutzerbedarf; hoher Umsetzungsaufwand ohne validierten Anwendungsfall. | Zurückgestellt auf unbestimmte Zeit; nicht Teil der aktiven Roadmap. |
| Vertikal-spezifische Ontologie-Schnittstelle | Verfrüht ohne validierten Pilotkunden in einem konkreten regulierten Vertical. | Zurückgestellt bis zum ersten validierten Vertical-Pilotprojekt. |
| Horizontales Sharding über mehrere Knoten | Widerspricht der Positionierung "Single-Node ist das Produkt" aktiv; kein aktueller Kundenbedarf nachweisbar. | Zurückgestellt; die RAM-Skalierungsgrenze wird stattdessen ehrlich in der Dokumentation kommuniziert (siehe A.6). |
| Peer-to-Peer-Synchronisationsprotokoll zwischen Instanzen | Sehr hoher Umsetzungsaufwand; kein validierter Nutzerbedarf; setzt zudem eine noch nicht produktionsreife Vorbedingung (Haltbarkeits-/Durability-Konfiguration, siehe B.1.2) voraus. | Zurückgestellt bis nach Validierung eines Mehrknoten-Anwendungsfalls. |
| EU-AI-Act-Risikoeinstufungs-Mapper | Der maßgebliche Durchsetzungszeitplan des EU AI Act liegt bei 2027; eine produktive Implementierung ist zum jetzigen Zeitpunkt verfrüht. | Zurückgestellt, Wiedervorlage wenn das Durchsetzungsdatum näher rückt. |
| Vollständige Implementierung einer Verarbeitungsverzeichnis-Quelle (DSGVO Art. 30) | Hängt von einem konkreten behördlichen Pilotprojekt ab und ist ohne dieses nicht eigenständig nutzbar. | Zurückgestellt bis zu einem konkreten behördlichen oder Legal-Tech-Pilotprojekt. |
| Mehrstufige, klassische-KI-vor-LLM-Entscheidungskaskade als beworbenes Standardfeature | Akademisch fundiert, aber kognitiv zu komplex für eine öffentliche API: Kein Nutzer versteht oder benötigt eine sechsstufige Routing-Kaskade als Konfigurationsoberfläche. | Bleibt als internes, hinter einem Feature-Flag verstecktes Subsystem erhalten; nie Standardverhalten, nie in der öffentlichen Dokumentation beworben. |
| Interne Bandit-/Regelungstechnik-Optimierungsschicht (lineare Banditen mit oberer Konfidenzschranke, Lyapunov-Drift-Regelung, PID-Regelung) als öffentlich konfigurierbares Feature | Akademisch beeindruckend, aber die Zielgruppe (Entwickler, die `cargo add contextra` ausführen) will keine Regelungstechnik konfigurieren müssen; die kognitive Last einer solchen Oberfläche schadet der Adoption mehr, als die Flexibilität nützt. | Bleibt als **internes**, nicht öffentlich dokumentiertes Subsystem erhalten. Nutzer konfigurieren ausschließlich über die grob gerasterten Bereitstellungsprofile (siehe D.3); intern entscheidet das System selbst, welche Optimierungsstrategie angewendet wird. Die öffentliche Erzählung lautet: "Das System optimiert sich selbst, du musst nichts konfigurieren." |

**Wichtiger Grundsatz zu dieser Tabelle:** Ausschluss bedeutet in den meisten Fällen nicht Löschung des vorhandenen Codes, sondern Entfernung aus dem Standard-Kompilierpfad (`default-members`, Standard-Feature-Flags) und aus der öffentlichen Kommunikation. Bereits geleistete Entwicklungsarbeit wird dadurch nicht vernichtet, sondern hinter einer expliziten Opt-in-Schwelle aufbewahrt, bis ein validierter Bedarf eine Reaktivierung rechtfertigt (vollständige Liste mit Reaktivierungsbedingungen: Anhang R.4).

## 0.6 Marktkontext Deutschland 2026

Drei regulatorische Entwicklungen haben sich im Jahr 2026 zu einem für dieses Produkt günstigen Zeitfenster verdichtet:

1. **Vergabebeschleunigungsgesetz** (in Kraft seit 1. Juli 2026): Verankert digitale Souveränität erstmals als rechtlich definierbares Zuschlagskriterium im deutschen Vergaberecht. Öffentliche Auftraggeber dürfen europäische Herkunft und Kontrollierbarkeit einer Lösung als Bewertungsmaßstab heranziehen — direkt relevant für ein air-gap-fähiges, ohne US-Cloud-Abhängigkeit betreibbares Produkt.
2. **BSI-C3A-Kriterienkatalog** (veröffentlicht April 2026): Definiert Souveränitätskriterien für Cloud- und KI-Dienste. Für Contextra relevant, wenn das Crate innerhalb einer C3A-konformen Lösung eines Systemhaus-Kunden eingesetzt wird: Eine lokale, auditierbare, quelloffene Speicher-Engine mit nachweisbarer Löschung erfüllt genau die Anforderung nach kontrollierbarer Infrastruktur.
3. **Marktstudie zur Präferenz deutscher Unternehmen** (2026): Eine deutliche Mehrheit deutscher Unternehmen gibt an, KI-Lösungen aus Deutschland gegenüber Angeboten aus den USA zu bevorzugen. Das ist ein belastbares Beschaffungssignal, kein Nischeneffekt.

**Realistische Marktebene: B2B2G, nicht B2G.** Der direkte Verkauf an Bundesbehörden ist für einen Einzelentwickler nicht realistisch — Großaufträge dieser Art gehen an etablierte Konzerne mit entsprechenden Kapazitäten. Der realistische Zielmarkt liegt eine Ebene darunter: Systemhäuser und Fachverfahren-Anbieter, die selbst an Behörden, Mittelstand und regulierte Branchen verkaufen und eine embeddbare, air-gap-fähige, nachweisbar löschende Memory-Engine als Baustein suchen. Konkrete Anbietertypen: Dokumenten-Management-System-Anbieter mit KI-Assistenzfunktion, Legal-Tech-Unternehmen mit DSGVO-Nachweispflicht, Medizintechnik-Software-Anbieter mit Rückverfolgbarkeitsanforderungen (vergleichbar ISO 13485).

**Fördermöglichkeit mit enger Frist.** Ein Bundesförderprogramm für die Überführung digitaler Technologien in marktfähige Produkte ("Digital-Tech-to-Product") adressiert explizit KI, Cybersicherheit und Datenökosysteme mit Fokus auf digitale Souveränität und fördert Projekte auf Technologiereifegrad 8 (funktionsfähiges System, noch nicht kommerziell) mit bis zu fünf Millionen Euro pro Projekt. **Der erste Förderaufruf endet am 11. Oktober 2026.** Ein Pure-Rust, air-gap-fähiges, DSGVO-konform löschendes Agenten-Gedächtnis aus Deutschland trifft die Förderkriterien direkt; die Bewerbung erfordert einen Businessplan (drei bis fünf Seiten) und einen technischen Demonstrator — beide Voraussetzungen sind mit dem bestehenden Code und einem bereinigten, reproduzierbaren Benchmark de facto bereits erfüllbar (siehe Teil G, Roadmap 60-Tage-Meilenstein).

## 0.7 Rechtlicher Hinweis: Exportkontrolle

Kryptografische Software (Ed25519-Signaturen, HMAC-SHA-256, authentifizierte Verschlüsselung, Blake3-Hashing) unterliegt in Deutschland und der EU grundsätzlich dem Außenwirtschaftsgesetz und der EU-Dual-Use-Verordnung (EU 2021/821) — unabhängig davon, ob der Quellcode offen zugänglich ist. Dies gilt uneingeschränkt, sobald Gespräche mit verteidigungsnahen oder sicherheitsbehördlichen Kunden geführt werden sollen, angesichts des in 0.1 beschriebenen Projektursprungs.

**Verbindliche Vorgabe:** Vor dem ersten Vertriebsgespräch mit einem verteidigungsnahen oder behördlichen Kunden ist eine schriftliche Ausfuhrkontroll-Einschätzung durch einen auf Exportkontrolle spezialisierten Rechtsanwalt oder direkt beim Bundesamt für Wirtschaft und Ausfuhrkontrolle (BAFA) einzuholen. Dies ist keine optionale Nebenmaßnahme, sondern eine Voraussetzung, die vor dem ersten Kontakt erfüllt sein muss — nicht danach. Sollte sich herausstellen, dass eine der eingesetzten kryptografischen Implementierungen unter eine einschlägige Kontrolllistennummer der Dual-Use-Verordnung fällt, ist für bestimmte Exportziele eine Genehmigung erforderlich; dies ist im Vorfeld lösbar, aber nur, wenn es rechtzeitig bekannt ist.

## 0.8 Governance-Regeln für Scope-Disziplin

Der Quellcode dieses Projekts zeigt ein wiederkehrendes Muster: Technisch interessante, aber strategisch nicht priorisierte Erweiterungen wurden auch dann umgesetzt, wenn eine vorangegangene Analyse explizit von genau dieser Erweiterung abgeraten hatte. Ein Beispiel aus der jüngeren Historie: Wenige Tage vor der Erstellung dieser Spezifikation wurde ein Framework-Adapter-Crate für Python-Ökosysteme hinzugefügt — exakt der Funktionsumfang, dessen Vermeidung zuvor empfohlen worden war. Dieses Verhaltensmuster ist die eigentliche Ursache dafür, dass ein technisch überdurchschnittlich diszipliniertes Projekt bislang keine kommerzielle Wirkung entfaltet hat, nicht ein Mangel an Code-Qualität.

Um dieses Muster zu unterbrechen, gelten ab sofort folgende, für jede beteiligte Instanz — menschlich oder KI-gestützt — bindende Regeln:

1. **Kein neues Crate, keine neue Abhängigkeit** wird ohne schriftliche Begründung in diesem Dokument (als Ergänzung von Teil B oder C) und eine Wartezeit von 48 Stunden zwischen Entscheidung und Umsetzung hinzugefügt.
2. **Externe Validierung vor interner Implementierung.** Vor dem Bau eines neuen, in diesem Dokument nicht bereits spezifizierten Features ist die Frage "Würde ein konkreter potenzieller Nutzer dafür zahlen?" an mindestens eine reale, projektexterne Person zu stellen. Bleibt die Frage unbeantwortet oder unklar beantwortet, wird das Feature nicht gebaut.
3. **Ein Benchmark, eine Wahrheit.** Es existiert zu jedem Zeitpunkt nie mehr als ein öffentlich kommunizierter Satz von Benchmark-Zahlen. Bei Widerspruch zwischen historischen und aktuellen Messungen gilt ausschließlich der zuletzt reproduzierte Wert; ältere Werte werden explizit als veraltet gekennzeichnet, und die Ursache der Abweichung wird dokumentiert (siehe A.6, C.5).
4. **Menschliche externe Kontrollinstanz.** Da die oben beschriebene Musterentstehung typischerweise dadurch begünstigt wird, dass ein KI-gestützter Entwicklungsassistent lokalen Kontext übernimmt und Vorschlägen tendenziell zustimmt, wird eine projektexterne, menschliche Person (ein Rust-Entwickler oder ein potenzieller Kunde) etabliert, die in regelmäßigem Abstand (empfohlen: alle zwei Wochen) eine unabhängige, kritische Einschätzung abgibt. Diese Person ersetzt keine KI-gestützte Analyse, ergänzt sie aber um die Fähigkeit, unabhängig von lokalem Gesprächskontext "nein" zu sagen.

## 0.9 Sofortmaßnahmen

Folgende Korrektur ist vor jeder anderen in diesem Dokument beschriebenen Maßnahme durchzuführen, da sie das Kernversprechen "kein zweiter Prozess" in der aktuell ausgelieferten Konfiguration des MCP-Servers unmittelbar verletzt:

```toml
# crates/contextra-mcp/Cargo.toml — Korrektur mit sofortiger Wirkung
# VORHER (verletzt das Kernversprechen):
contextra = { workspace = true, features = ["ollama", "router"] }
# NACHHER:
contextra = { workspace = true, features = ["router"] }
```

Ein extern angebundenes Inferenz-Backend, das einen separaten Prozess voraussetzt, ist ein legitimes Opt-in-Feature für Nutzer, die es explizit wünschen. Es darf niemals Standardbestandteil eines MCP-Servers sein, dessen zentrales Verkaufsargument "kein zweiter Prozess" lautet.

---

# Teil A — Architektur und Ist-Zustand {#teil-a}

Dieser Teil beschreibt den verifizierten, bereinigten Ist-Zustand des Systems: 34 Crates, ca. 218.650 Zeilen produktiven Rust-Codes, entstanden über einen Entwicklungszeitraum von rund fünf Monaten mit sehr hoher Commit-Dichte (Spitzenwert: über 160 Commits an einem einzigen Tag). Diese Entstehungsgeschwindigkeit ist mit rein menschlicher Entwicklung nicht erklärbar und weist auf einen KI-gestützten, agentischen Entwicklungsprozess hin. Das ist keine Qualitätsaussage per se — die im Folgenden dokumentierte Zero-Panic-Disziplin hält bemerkenswert gut —, aber es bedeutet, dass das Verständnis einzelner Subsysteme in der Tiefe geringer sein kann, als der Codeumfang suggeriert. Konsequenz für die Arbeit mit diesem Dokument: Jede Schnittstellenänderung muss gegen die hier gelisteten Invarianten geprüft werden, nicht gegen ein angenommenes Verständnis des Subsystems.

## A.1 Systeminvarianten und Designprinzipien

**Tragende Designprinzipien (unveränderlich, für jede Codeänderung bindend):**

| Kürzel | Prinzip |
|---|---|
| P7 | **Zero-Panic-Doktrin:** Kein `unwrap()`, kein `expect()`, kein `panic!()` in Produktionspfaden. Jeder Fehlerfall wird über den `Result`-Typ und die zentrale Fehlertaxonomie propagiert. |
| P24 | **Lokalität:** Algorithmen skalieren mit dem lokalen Kontext einer Anfrage, nicht mit der Gesamtdatenmenge des Systems. Eine Operation auf einem einzelnen Dokument darf keine Komplexität aufweisen, die vom Gesamtindex abhängt. |
| P26 | **Ring-0-Synchronität:** Code in Ring 0 (siehe A.8) ist frei von `async`-Notation und referenziert `tokio` nicht direkt. Asynchronität ist ausschließlich an den I/O-Grenzen der höheren Ringe zulässig. |
| P27 | **dyn-kompatible Ports:** Jeder Trait in der zentralen Port-Schicht ist objektsicher (`dyn`-kompatibel), um Testdoubles und alternative Implementierungen ohne generische Aufblähung zu ermöglichen. |
| P28 | **Injizierter Determinismus:** Kein direkter Aufruf von `SystemTime::now()` oder `thread_rng()` an irgendeiner Stelle im Produktionscode. Zeit- und Zufallsquellen werden ausschließlich über injizierte `Clock`- und `Rng`-Ports bezogen. Dies ist die technische Grundlage für reproduzierbare Agentenläufe, deterministisches Testen und Replay-Debugging. **Explizite Ausnahme (ergänzt durch Architektur-Audit, siehe X.11 und ADR-098):** Kryptografisches Schlüssel- und Saltmaterial (WAL-Integritätsschlüssel, Verschlüsselungs-Salt, temporäre Dateinamen zur Kollisionsvermeidung) ist von P28 ausgenommen und MUSS `rand::thread_rng()` bzw. einen CSPRNG verwenden — NIEMALS den injizierten, deterministischen `Rng`-Port, da dieser in Tests durch einen festen Seed ersetzbar ist und ein darüber erzeugter Schlüssel damit vorhersagbar und kryptografisch wertlos wäre. Reine Dateinamens-Eindeutigkeit ohne Sicherheits- oder Sichtbarkeitsbezug (z. B. SSTable-/WAL-Segment-Namen über `SystemTime::now()`) ist ebenfalls ausgenommen, solange der Wert nicht in eine Sichtbarkeits-, Sequenz- oder Prüfsummenentscheidung einfließt. Jede Verwendung von Wanduhrzeit, die in einem für den Nutzer sichtbaren, deterministisch erwarteten Artefakt landet (z. B. ein `StateCheckpoint`-Zeitstempel), fällt NICHT unter diese Ausnahme und MUSS über `Clock` laufen. |
| P29 | **Kein globaler veränderlicher Zustand.** |

**Systeminvarianten (unveränderlich, durch Tests und/oder Typsystem durchgesetzt):**

| Invariante | Beschreibung |
|---|---|
| `INV-DELETION-1` | Ein kryptografischer Löschbeweis darf ausschließlich nach abgeschlossener physischer Bereinigung aller betroffenen Speicherschichten ausgestellt werden — niemals vorher oder parallel dazu. |
| `INV-DURABILITY-RING` | Ein rein speicherresidenter Haltbarkeitsmodus (kein Write-Ahead-Log) ist inkompatibel mit dem Löschbeweis-Feature und mit dem souveränen Feature-Ring — beides gemeinsam zu aktivieren ist ein Konfigurationsfehler, der zur Kompilierzeit oder spätestens zur Laufzeit abgelehnt wird. |
| `INV-TENANT-1` | Die Mandanten-Kennung Null ist system-reserviert; der Versuch, sie als reguläre Mandanten-ID zu konstruieren, schlägt fehl. |
| `INV-HASHER-SEEDS` | Der Hash-Algorithmus für Schlüsselsperren verwendet ausschließlich die vier verifizierten, festen Konstanten — keine zur Laufzeit generierten Seeds. |

## A.2 Vollständiges Crate-Inventar

Die folgende Tabelle beschreibt den bereinigten Soll-Zustand nach Durchführung aller in Abschnitt 0.5 und 0.9 beschriebenen Ausschlüsse und Sofortmaßnahmen.

| # | Crate | Ring | Reife | Zweck |
|---|---|---|---|---|
| 1 | `contextra-types` | 0 | 🟢 | Kanonische ID-, Fehler- und Enum-Typbasis; jedes andere Crate hängt hiervon ab. |
| 2 | `contextra-ports` | 0 | 🟢 | Hexagonale Trait-Grenzen (rund 30 Traits), einschließlich der Feature-Ring- und Lizenz-Gate-Abstraktion. |
| 3 | `contextra-mvcc` | 0 | 🟢 | MVCC-Primitive: Sequenzlog, Snapshot-Registry, Transaktionspuffer. |
| 4 | `contextra-wire` | 0 (Insel) | 🟢 | FlatBuffers-basiertes IPC-Protokoll, Zero-Copy-Adapter. |
| 5 | `contextra-sys` | 0 (Insel) | 🟢 | Betriebssystem-Abstraktionen: Memory-Mapping, Speicherverriegelung, Zugriffskontrolllisten. |
| 6 | `contextra-simd` | 0 (Insel) | 🟢 | SIMD-Kernels für Distanzberechnungen (AVX2/AVX-512/NEON/Skalar-Fallback). |
| 7 | `contextra-crypto` | 0 (Insel) | 🟢 | AES-256-GCM-SIV, Argon2id, Ed25519-basierter Löschbeweis. Rund 1.600 Zeilen allein im Löschbeweis-Modul — das dichteste, sicherheitskritischste Modul des Systems. |
| 8 | `contextra-vector` | 0 | 🟢 | HNSW- und DiskANN-Vektorindizes, ACORN-Filterung, RaBitQ-Quantisierung. Ghost-Vektor-Schutz nach Löschung noch offen (siehe B.1.1, P0-1). |
| 9 | `contextra-text` | 0 | 🟢 | BM25F/WAND-Volltextsuche mit deutscher Morphologie-Unterstützung. |
| 10 | `contextra-graph` | 0 | 🟢 | Compressed-Sparse-Row-Graphrepräsentation, Personalized-PageRank/Forward-Push, Leiden-Community-Detection, Hyperkanten-Unterstützung. |
| 11 | `contextra-rank` | 0 | 🟢 | Reziproke-Rang-Fusion über Vektor-, Text-, Graph- und KV-Cache-Signale gleichzeitig, isotonische/Platt-Kalibrierung, Erklärbarkeits-Backend. |
| 12 | `contextra-adapt` | 0 | 🟡 | Interne Bandit-Mathematik (lineare Banditen mit oberer Konfidenzschranke, faktorisierte Thompson-Sampling-Variante), Lyapunov-Drift-Regelung, PID-Regelung — **ausschließlich intern, nicht Teil der öffentlichen API** (siehe 0.5). |
| 13 | `contextra-store` | 1 | 🟢 | LSM-Baum, Write-Ahead-Log mit Gruppencommit, Manifest-Verwaltung, Schlüsselsperren. Haltbarkeitsmodus-Konfiguration noch offen (siehe B.1.2, P0-2). |
| 14 | `contextra-kvcache` | 1 | 🟢 | Verschlüsselter Key-Value-Cache mit Radix-Präfixbaum und authentifiziert verschlüsselten Segmentdateien — Agenten-spezifisches Feature, das in dieser Form embeddbar selten angeboten wird. |
| 15 | `contextra-checkpoint` | 1 | 🟢 | Time-Travel-Registry mit Blake3-basiertem Manifest. |
| 16 | `contextra-infer-candle` | 2 | 🟢 | GGUF-Modellinferenz über die Candle-Bibliothek — Standard-Inferenz-Backend (reines Rust, kein externer Prozess, kein C++-FFI). |
| 17 | `contextra-infer-ollama` | 2 | 🟢 | HTTP-Client für einen externen Ollama-Prozess — **ausschließlich Opt-in**, niemals Standardbestandteil (siehe 0.5, 0.9). |
| 18 | `contextra-infer-onnx` | 2 | 🟡 | ONNX-basiertes Cross-Encoder-Reranking über C++-FFI — **ausschließlich Opt-in**. |
| 19 | `contextra-sandbox` | 2 | 🟢 | WASM-Ausführungsisolation — **ausschließlich Opt-in**, nicht Teil der Standardkonfiguration. |
| 20 | `contextra-engine` | 3 | 🟢 | Collection-CRUD-Operationen, Zwei-Phasen-Commit-Transaktionen. Mit rund 17.800 Zeilen das größte Crate nach `contextra-store`, gleichzeitig mit der schwächsten Test-zu-Code-Ratio — größtes technisches Einzelrisiko des Systems (siehe A.7, B.2.1). |
| 21 | `contextra-cognition` | 3 | 🟢 | Gedächtniskonsolidierung, LeanRAG-Aggregation, Sitzungs-Kompaktierung. Konsolidierungslogik bislang unzureichend gegen reale LLM-Agenten-Workflows validiert (siehe C.4). |
| 22 | `contextra-privacy` | 3 | 🟢 | Fünfschichtiges Egress-Gateway, PII-Tresor, mandantenfähige Typumhüllung. |
| 23 | `contextra-router` | 3 | 🟢 | Arm-Registry und Thompson-Sampling-Dispatch für Modellauswahl. |
| 24 | `contextra-agent` | 3 | 🟢 | Workflow-Engine mit dem Muster Checkpoint → Ausführung → Commit → Audit, inklusive Dead-Letter-Queue für fehlgeschlagene Schritte. |
| 25 | `contextra-audit-export` | 4 | 🟡 | Mapping auf Anforderungen des BSI-Grundschutzes, DSGVO Art. 30. |
| 26 | `contextra-avv-generator` | 4 | 🟡 | Generator für Auftragsverarbeitungsvertrags-Vorlagen (DSGVO Art. 28). |
| 27 | `contextra-license` | 4 | 🟡 | Bislang nur mit einer offenen Stub-Implementierung des Lizenz-Gates versehen; re-exportiert die Feature-Ring-Abstraktion aus `contextra-ports`. |
| 28 | `contextra` | 4 | 🟢 | Öffentliche Composition-Root der gesamten Bibliothek, mit Builder-Muster als primärer Konstruktionsschnittstelle. Dies ist der Einstiegspunkt, den externe Nutzer via `cargo add contextra` verwenden. |
| 29 | `contextra-mcp` | 4 | 🟢 | Model-Context-Protocol-Server über stdio mit JSON-RPC 2.0, aktuell zehn produktiv nutzbare Werkzeuge. |
| 30 | `contextra-db` | 3 | 🟢 | Historisch gewachsene, monolithische Fassade — Abbaupfad, bleibt ausschließlich zur Abwärtskompatibilität erhalten (siehe C.1). |
| T1 | `contextra-testkit` | Tooling | 🟢 | Fehlerinjizierendes virtuelles Dateisystem, speicherresidenter Store für Tests, manuell steuerbare Uhr. |
| T2 | `contextra-bench` | Tooling | 🟢 | Benchmark-Harness, von der regulären Feature-/Capability-Prüfung ausgenommen. |
| T3 | `xtask` | Tooling | 🟢 | Interne Build- und Verifikationswerkzeuge mit über 60 Modulen und mindestens 50 CI-Prüfgates. |

**Aus dem Workspace vollständig entfernt beziehungsweise ausgelagert:** ein Framework-Adapter-Crate für Python-Ökosysteme (❌, ersatzlos entfernt, siehe 0.5) sowie die Python-FFI-Bindings (❌ als Workspace-Mitglied, Auslagerung in ein separates Repository, siehe 0.5).

## A.3 Workspace-Konfiguration

Die folgende Konfiguration definiert den Soll-Zustand der Standard-Kompiliermitgliedschaft (`default-members`) im Wurzel-`Cargo.toml` des Workspace. Nur die hier gelisteten Crates werden bei einem Standard-Build ohne explizite Feature-Auswahl kompiliert:

```toml
# Workspace Cargo.toml — default-members[] im Soll-Zustand
default-members = [
    "crates/contextra-wire",
    "crates/contextra-sys",
    "crates/contextra-simd",
    "crates/contextra-core",
    "crates/contextra-store",
    "crates/contextra-vector",
    "crates/contextra-db",
    "crates/contextra-text",
    "crates/contextra-checkpoint",
    "crates/contextra-crypto",
    "crates/contextra-privacy",
    "crates/contextra-graph",
    "crates/contextra-mcp",
    "crates/contextra-agent",
    "crates/contextra-router",
    "crates/contextra-infer-candle",  # Candle ist das Standard-Inferenz-Backend
    "crates/contextra-kvcache",
    "crates/contextra-engine",
    "crates/contextra-cognition",
    "crates/contextra-rank",
    "crates/contextra-types",
    "crates/contextra-ports",
    "crates/contextra-mvcc",
    "crates/contextra-adapt",
    "crates/contextra-audit-export",
    "crates/contextra-avv-generator",
    "crates/contextra-license",
    "crates/contextra",
    # AUSDRÜCKLICH NICHT in default-members:
    # ein Framework-Adapter-Crate für Python-Ökosysteme  → vollständig entfernt
    # Python-FFI-Bindings                                 → eigenes, separates Repository
    # contextra-infer-ollama  → Opt-in (externer Prozess)
    # contextra-infer-onnx    → Opt-in (C++-FFI)
    # contextra-sandbox       → Opt-in (WASM-Isolation)
]
```

## A.4 Feature-Flag-Matrix

```toml
# crates/contextra/Cargo.toml — [features] im Soll-Zustand
[features]
default = ["fast", "candle"]
fast    = []
candle  = ["contextra-infer-candle"]
sovereign  = ["contextra-crypto", "contextra-privacy"]
compliance = ["sovereign", "audit-export", "avv-generator"]
audit-export   = ["dep:contextra-audit-export"]
avv-generator  = ["dep:contextra-avv-generator"]
# Ausschließlich Opt-in, niemals Bestandteil von "default":
ollama = ["contextra-infer-ollama"]
onnx   = ["contextra-infer-onnx/onnx", "contextra-db/onnx"]
router = ["contextra-router", "contextra-rank"]
sandbox = ["dep:contextra-sandbox"]
deterministic-cascade = ["contextra-adapt/bandit-routing"]
```

Diese Struktur bildet direkt das in Teil D beschriebene Drei-Ring-Lizenzmodell ab: `fast` ist der offene, großzügig funktionsfähige Standardring; `sovereign` und `compliance` sind additive, opt-in aktivierbare Ringe mit steigender Funktionalität und — im Fall von `compliance` — kommerzieller Lizenzbedingung (siehe D.4).

## A.5 MCP-Tool-Inventar

Der MCP-Server stellt aktuell zehn produktiv funktionsfähige Werkzeuge bereit. Vier weitere Werkzeugnamen sind bereits in der Sicherheits-Klassifikationslogik und in Tests referenziert, besitzen aber keinen tatsächlichen Dispatch-Zweig — ein Client, der eines dieser vier Werkzeuge aufruft, erhält einen "Methode nicht gefunden"-Fehler. Ein fünftes, an anderer Stelle bereits spezifiziertes Werkzeug fehlt vollständig.

| Werkzeug | Status | Bemerkung |
|---|---|---|
| `contextra_search` | 🟢 | Hybride Suche über Vektor/Text/Graph. |
| `contextra_insert` | 🟢 | Dokument einfügen. |
| `contextra_get` | 🟢 | Dokument abrufen. |
| `contextra_forget` | 🟢 | Bestehende Vergessens-Operation (Vorläufer des vollständigen Löschbeweis-Pfads). |
| `contextra_collections` | 🟢 | Collections auflisten. |
| `contextra_consolidate` | 🟢 | Gedächtniskonsolidierung auslösen. |
| `contextra_cloud_query` | 🟢 | Abfrage gegen externe Erweiterung. |
| `contextra_relate` | 🟢 | Binäre Relation zwischen zwei Entitäten anlegen. |
| `contextra_relate_n_ary` | 🟢 | n-äre Relation (Hyperkante) anlegen. |
| `contextra_explain` | 🟢 | Erklärbarkeits-Ausgabe zu einer Suchentscheidung, über einen separaten Behandlungspfad realisiert. |
| `contextra_upsert` | 🔴 Phantom | Name in Sandbox-Klassifikation und Tests vorhanden, kein Dispatch-Zweig implementiert. |
| `contextra_delete` | 🔴 Phantom | Dito. |
| `contextra_create_collection` | 🔴 Phantom | Dito. |
| `contextra_drop_collection` | 🔴 Phantom | Dito. |
| `contextra_plugin_status` | 🔴 Fehlend | An keiner Stelle im Workspace implementiert. |

**Produktentscheidung (Details: C.2):** Die vier Phantom-Werkzeuge werden implementiert, nicht ersatzlos entfernt — sie sind für ein aus Agentensicht vollständiges Speicher-Interface notwendig (ein MCP-Interface ohne vollständige CRUD-Operationen genügt für Agenten-Nutzer nicht).

## A.6 Benchmark-Status

| Korpusgröße | Einfügerate (Dok./s) | Suchlatenz p50 | Spitzen-Speicherverbrauch | Status |
|---|---|---|---|---|
| 1.000 Dokumente | 409,7 | 2,61 ms | 148,85 MB | ✅ Reproduziert |
| 5.000 Dokumente | 126,9 | 5,11 ms | 332,36 MB | ✅ Reproduziert |
| 10.000 Dokumente | 72,0 | 5,13 ms | 597,16 MB | ✅ Reproduziert |
| 100.000 Dokumente | ca. 25 | ca. 3.500 ms | ca. 1.950 MB | ⚠️ Nur extrapoliert, nicht gemessen |
| 1.000.000 Dokumente | ca. 5 | über 30 s | über 18,5 GB | ❌ Überschreitet den typisch verfügbaren Arbeitsspeicher einer Entwicklungsumgebung |

**Historische Diskrepanz.** Frühere dokumentierte Messungen wiesen für 1.000 Dokumente eine Suchlatenz auf, die um den Faktor 34 über der zuletzt reproduzierten Messung lag (rund 88 Millisekunden gegenüber 2,61 Millisekunden). Die wahrscheinlichste Ursache ist, dass die historische Messung die Latenz eines externen Embedding-Aufrufs (ONNX- oder Ollama-Inferenz) mit einschloss, während die aktuelle Messung ausschließlich Speicher- und Indexlatenz erfasst — dies muss vor jeder öffentlichen Kommunikation der Zahlen explizit dokumentiert werden (siehe C.5). Historische Werte gelten ab sofort als veraltet und sind entsprechend zu kennzeichnen. Es existiert ab sofort nur noch eine kanonische Quelle der Wahrheit für Benchmark-Zahlen (Governance-Regel 3, Abschnitt 0.8).

**Skalierungsgrenze — ehrlich zu kommunizieren.** Contextra ist ein Single-Node-System. Der HNSW-Vektorindex liegt in seiner Standardkonfiguration vollständig im Arbeitsspeicher. Ab etwa 100.000 Dokumenten ist die DiskANN-Variante (teilweise plattenresident, aktuell experimentell) zu verwenden. Diese Grenze ist keine verschwiegene Schwäche, sondern ein bewusster Bestandteil der Positionierung: Contextra tritt nicht gegen Systeme an, die für horizontale Skalierung über hunderte Millionen Vektoren gebaut sind (siehe 0.4, 0.5).

## A.7 Code-Qualitätsbefunde

**Zero-Panic-Disziplin — Befund: hält, mit wenigen benannten Ausnahmen.** Im gesamten Produktionscode (nach Ausschluss aller Testverzeichnisse, bedingt kompilierter Testblöcke und automatisch generierten Codes) finden sich Fälle von `unwrap()`/`expect()`, die eine gezielte Nachbesserung rechtfertigen — konzentriert im internen Optimierungssubsystem (lineare Banditen-Implementierung, Lyapunov-Drift-Regelung) sowie vereinzelt in der Checkpoint-Verwaltung. Der speicherkritischste Kern (`contextra-store`) weist **keine** derartigen Stellen im Produktionscode auf — ein für ein Projekt dieser Entstehungsgeschwindigkeit ungewöhnlicher Befund und ein echtes Qualitätsmerkmal. Automatisch generierter Code (FlatBuffers-Ausgabe) enthält naturgemäß Stellen dieser Art; diese sind strukturell unkritisch, sollten aber explizit als Ausnahme in der Prüflogik dokumentiert werden, damit künftige automatische Prüfungen nicht fälschlich Alarm schlagen.

**Unsafe-Inseln — Befund: weitgehend integer.** Die drei deklarierten Unsafe-Inseln (Betriebssystem-Abstraktion, SIMD-Kernels, Wire-Protokoll) enthalten die überwiegende Mehrheit aller `unsafe`-Blöcke im System, wie es die Inseln-Architektur vorsieht. Vereinzelte `unsafe`-Vorkommen außerhalb dieser Inseln liegen ausschließlich in Testcode (etwa ein benutzerdefinierter Allokator zur Heap-Nachverfolgung in Graph-Tests) oder sind mit expliziten Sicherheitskommentaren und Ausnahme-Markierungen versehen. Die deklarierte `unsafe`-Verbotsregel hält im Produktionspfad.

**Testabdeckung — Befund: strukturell gut, aber ungleich verteilt.** Der Speicherkern (`contextra-store`) und die Graph-Engine sind mit dedizierten Testverzeichnissen für Wiederherstellung, HMAC-Kettenintegrität, Replay und Personalized-PageRank-Berechnung gut abgedeckt. **Das größte einzelne technische Risiko ist `contextra-engine`:** Mit der zweithöchsten Zeilenzahl im gesamten System und der gesamten Verantwortung für Collection-CRUD sowie Transaktionsausführung weist dieses Crate die sichtbar schwächste Testtiefe aller zentralen Komponenten auf. Dies ist der wichtigste einzelne Ort für gezielte Investition vor jedem öffentlichen Launch (siehe B.2.1). Die Gedächtniskonsolidierungslogik (`contextra-cognition`) ist ebenfalls unzureichend getestet und wurde nach vorliegendem Kenntnisstand noch nicht gegen einen realen mehrstufigen LLM-Agenten-Workflow validiert, sondern bislang überwiegend gegen synthetische Vektordaten (siehe C.4).

**Markenidentität — Befund: nicht bereinigt.** Das Projekt wurde erst kurz vor Erstellung dieser Spezifikation von seinem vorherigen Arbeitsnamen auf "Contextra" umbenannt. Ein erheblicher Teil der Commit-Historie und vereinzelte Code-Kommentare referenzieren noch den alten Namen. Dies ist eine rein hygienische Aufgabe, die vor jeder öffentlichen Sichtbarkeit (Veröffentlichung auf einem Paket-Register, öffentliche Repository-Bewerbung) abzuschließen ist (siehe F.3).

## A.8 Ring-Abhängigkeitsmodell

```
Ring 0  [synchron, ohne tokio-Abhängigkeit]
        types, ports, mvcc, wire, sys, simd, crypto, vector, text, graph, rank, adapt, core

Ring 1  [asynchrone I/O ausschließlich an den Grenzen]
        store, kvcache, checkpoint

Ring 2  [Blätter, ausnahmslos Opt-in]
        infer-candle (Standard), infer-ollama (Opt-in), infer-onnx (Opt-in), sandbox (Opt-in)

Ring 3  [Orchestrierung]
        engine, db (Legacy/Abbaupfad), cognition, privacy, router, agent

Ring 4  [Grenzschicht, Composition Root]
        contextra (Fassade), mcp, audit-export, avv-generator, license

Separat, kein Workspace-Mitglied mehr:
        Python-FFI-Bindings → eigenes, separates Repository

Vollständig entfernt:
        Framework-Adapter-Crate für Python-Ökosysteme ❌
```

Diese Ring-Struktur ist die architektonische Grundlage für zwei strategisch wichtige Eigenschaften: Erstens erlaubt sie eine spätere, im Moment nicht priorisierte `no_std`-Portierung der unteren Ringe für eine mögliche robotische Zielanwendung (siehe 0.1), ohne dass die oberen Ringe angetastet werden müssten. Zweitens stellt sie sicher, dass Opt-in-Komponenten (Ring 2, Sandbox) niemals eine harte Abhängigkeit für den Kern erzeugen können.

---

# Teil B — Priorisierte Schnittstellenspezifikationen (Soll-Zustand) {#teil-b}

## B.0 Konvention

Jede Einzelspezifikation in diesem Teil folgt demselben siebenteiligen Schema, um sie für ein Sprachmodell ohne zusätzlichen Kontext unmittelbar umsetzbar zu machen:

**1. Problem** · **2. Soll-Schnittstelle** (konkreter Rust-Code) · **3. Algorithmus** · **4. Invarianten** · **5. Fehlerfälle** · **6. Tests** · **7. Aufwand und Priorität**

**Prioritätsstufen:** **P0** (blockiert das primäre Produktversprechen und muss vor jeder Außendarstellung gelöst sein) · **P1** (nächste Umsetzungsphase, vor öffentlichem Launch) · **P2** (Roadmap-Horizont 60 Tage) · **P3** (Backlog, nur bei validiertem Bedarf).

---

## B.1 P0-Kern: Löschbeweis vollständig machen

Der kryptografische Löschbeweis ist das zentrale Alleinstellungsmerkmal des Produkts (siehe 0.2–0.3). Aktuell behauptet der Beweis vollständige Löschung, obwohl mindestens ein Teilsystem (der Vektorindex) nach Löschung noch Restinformation in Nachbarschaftsstrukturen enthalten kann. Dieser Widerspruch ist der höchste Einzelrisikofaktor des gesamten Produkts und muss vor jeder Vermarktung als "kryptografisch beweisbar löschend" geschlossen sein.

### B.1.1 Geisterknoten-Schutz im Vektorindex (P0-1)

**1. Problem.** Der kryptografische Löschbeweis behauptet vollständige Datenlöschung. Der HNSW-Vektorindex lässt jedoch nach einer Lösch-Operation sogenannte Geisterzeiger (Ghost-Pointer) auf den gelöschten Knoten in den Nachbarschaftslisten anderer Knoten bestehen — eine bereits gelöschte Instanz ist damit über eine k-nächste-Nachbarn-Suche noch indirekt rekonstruierbar oder zumindest in ihrer ungefähren Position im Vektorraum ableitbar. Ein Angreifer, der Zugriff auf den Index nach einer behaupteten Löschung erhält, kann über diese Restinformation Rückschlüsse ziehen, die dem Löschversprechen widersprechen.

**2. Soll-Schnittstelle.**

```rust
// crates/contextra-vector/src/hnsw/deletion.rs — neue Datei

pub struct DeletionStats {
    pub doc_id: DocId,
    pub repaired_edges: usize,
    pub orphaned_replacements: usize,
    pub verified_no_ghost_pointers: bool,
}

pub trait GhostFreeVectorIndex {
    /// Löscht doc_id UND repariert alle Nachbarschaftszeiger synchron, bevor
    /// die Funktion zurückkehrt.
    /// Invariante INV-DELETION-2: Nach Ok(..) zeigt kein Nachbarschaftszeiger
    /// im gesamten Index mehr auf doc_id.
    fn remove_with_graph_repair(&mut self, doc_id: DocId) -> Result<DeletionStats>;
}

// crates/contextra-crypto/src/deletion_proof.rs — Signaturerweiterung
pub struct GraphRepairAttestation {
    pub doc_id: DocId,
    pub verified_no_ghost_pointers: bool,
    pub attested_at: i64, // ausschließlich über den injizierten Clock-Port (P28)
}

impl DeletionProof {
    // Signatur-Breaking-Change für contextra-crypto (SemVer MAJOR):
    pub fn create_with_wal_receipt_v3(
        /* bestehende Parameter unverändert */
        graph_repair: &[GraphRepairAttestation],
    ) -> Result<Self, CryptoError>;
}
```

**3. Algorithmus.** Für jeden Nachbarn des gelöschten Knotens auf jeder Ebene des HNSW-Graphen: Kante entfernen, besten Ersatzkandidaten aus der ursprünglichen Kandidatenliste dieser Ebene auswählen (bestehende heuristische Nachbarschaftsauswahl wiederverwenden, keine Neuimplementierung). Abschließend ein Verifikationsdurchlauf über die betroffene Nachbarschaft zweiter Ordnung mit einem festen Budget (Grad multipliziert mit maximaler Verbindungsanzahl multipliziert mit vier). Die resultierende Komplexität ist proportional zur lokalen Nachbarschaftsgröße, nicht zur Gesamtgröße des Index — konform mit Designprinzip P24.

**4. Invarianten.** Neue Invariante `INV-DELETION-2`: Nach Aufruf von `remove_with_graph_repair(doc_id)` zeigt kein Nachbarschaftszeiger im gesamten Index mehr auf `doc_id`. Bestehende Invariante `INV-DELETION-1` wird verschärft: Die neue Löschbeweis-Erstellungsfunktion verlangt eine nicht-leere Liste von Graph-Reparatur-Attestierungen als Pflichtparameter — dies ist zur Kompilierzeit erzwingbar über die Typsignatur.

**5. Fehlerfälle.** Neue Fehlervariante `GraphRepairFailed(HnswDeletionError)` mit den Untervarianten: Knoten nicht gefunden, Reparatur führte zu getrennter Komponente, Verifikation fehlgeschlagen mit Angabe der Anzahl verbleibender Restzeiger.

**6. Tests.**
- Eigenschaftsbasierter Test (Property-Test) über zufällig generierte Graphtopologien mit erschöpfendem Scan nach jeder Löschung, um sicherzustellen, dass keine Restzeiger verbleiben.
- Fehlerinjektionstest, der eine Rekonstruktionsattacke simuliert: Eine k-nächste-Nachbarn-Suche nach Löschung darf keine bessere Trefferqualität liefern als eine Suche gegen einen Vektor, der nie eingefügt wurde.
- Test, der sicherstellt, dass die neue Löschbeweis-Erstellungsfunktion mit einer leeren Attestierungsliste für eine vektorindex-tragende Collection fehlschlägt.

**7. Aufwand und Priorität.** Mittel bis hoch (geschätzt 400 Zeilen Kernlogik zuzüglich 200 Zeilen Tests sowie Signaturpropagation durch drei Crates). **P0-1 — blockiert das primäre Produktversprechen.**

---

### B.1.2 Explizite Haltbarkeitsmodi (P0-2)

**1. Problem.** Die Konfiguration des Speicherkerns besitzt aktuell kein Feld zur Steuerung der Haltbarkeitsgarantie. Das Write-Ahead-Log ist implizit immer aktiv. Ein "Performance-Modus ohne Absturzkonsistenz" für ephemere Collections (etwa flüchtiges Arbeitsgedächtnis eines Agenten) fehlt vollständig — und dürfte, naiv nachgerüstet, versehentlich mit dem souveränen Feature-Ring oder dem Löschbeweis kombinierbar sein, was eine Sicherheitslücke darstellen würde.

**2. Soll-Schnittstelle.**

```rust
// crates/contextra-store/src/lsm/config.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum DurabilityMode {
    #[default]
    Full,       // WAL + HMAC-Integritätskette + fsync — einziger löschbeweis-fähiger Modus
    WalNoHmac,  // WAL aktiv, ohne HMAC-Kette; kein Löschbeweis möglich
    MemoryOnly, // Kein WAL; ausschließlich für ephemere Collections (Agent-Arbeitsgedächtnis, Caches)
}

impl DurabilityMode {
    pub fn validate_against_features(
        self,
        deletion_proof_active: bool,
        feature_ring: FeatureRing,
    ) -> Result<(), DurabilityConfigError>;
}
```

**Kompatibilitätsmatrix:**

| Modus | Löschbeweis | Souveräner Ring | Zielnutzung |
|---|---|---|---|
| `Full` | ✅ | ✅ | Alle produktiven Deployments (Standard) |
| `WalNoHmac` | ❌ | ❌ | Analyse-Collections ohne DSGVO-Pflicht |
| `MemoryOnly` | ❌ | ❌ | Ephemere Agent-Arbeitsgedächtnisse, Caches |

**4. Invarianten.** `INV-DURABILITY-RING` (siehe A.1): `MemoryOnly` ist mit dem Löschbeweis-Feature und dem souveränen Ring inkompatibel; die Validierungsfunktion lehnt diese Kombination ab.

**7. Aufwand und Priorität.** Niedrig bis mittel (geschätzt 150 Zeilen für Enum, Validierung und Propagation). **P0-2.**

---

### B.1.3 Automatische Extraktion standardmäßig aktivieren (P0-3)

**1. Problem.** Die automatische Tripel-Extraktion (Umwandlung von Freitext über ein Sprachmodell in Wissensgraph-Tripel) ist aktuell standardmäßig deaktiviert. Dies ist exakt das Feature, das Contextra von einer reinen Vektordatenbank unterscheidet — als deaktivierter Standard ist es für neue Nutzer unsichtbar und wird daher faktisch nicht genutzt.

**2. Soll.** Die Standardkonfiguration wird auf aktiviert umgestellt. Neue Nutzer erhalten automatisch generierte Graph-Tripel; Deaktivierung erfolgt nur noch über eine explizite Konfigurationsangabe.

**Hinweis.** Dies ist ein Breaking Change auf Verhaltensebene (nicht auf Typsignaturebene) und muss im Änderungsprotokoll explizit vermerkt werden. Für kostensensitive Nutzer (jede Extraktion verursacht einen zusätzlichen Sprachmodell-Aufruf) wird ein separates Cargo-Feature als expliziter Fluchtweg bereitgestellt.

**7. Aufwand und Priorität.** Niedrig (Standardwertänderung zuzüglich rund 80 Zeilen Tests). **P0-3.**

---

### B.1.4 Vier MCP-Phantom-Werkzeuge implementieren (P0-4)

**1. Problem.** Vier Werkzeugnamen (`contextra_upsert`, `contextra_delete`, `contextra_create_collection`, `contextra_drop_collection`) existieren bereits als Namen in der Sicherheitsklassifikationslogik und in Tests, besitzen aber keinen tatsächlichen Dispatch-Zweig. Ein Client, der eines dieser Werkzeuge aufruft, erhält einen Standard-JSON-RPC-Fehler "Methode nicht gefunden".

**2. Soll.** Alle vier Werkzeuge werden implementiert und in die zentrale Dispatch-Logik eingetragen:

| Werkzeug | Semantik |
|---|---|
| `contextra_upsert` | Einfügen oder Aktualisieren eines Dokuments, idempotent, schlüsselbasiert. |
| `contextra_delete` | Löscht ein Dokument — ruft intern `remove_with_graph_repair` (B.1.1) auf und stellt einen vollständigen Löschbeweis aus. |
| `contextra_create_collection` | Erstellt eine neue Collection mit wählbarem Bereitstellungsprofil (siehe B.1.5 und D.3). |
| `contextra_drop_collection` | Löscht eine gesamte Collection und stellt einen collection-weiten Löschbeweis aus. |

**Reihenfolgeabhängigkeit.** `contextra_delete` kann erst nach produktiver Fertigstellung von B.1.1 sicher implementiert werden, da sonst ein Löschbeweis ausgestellt würde, ohne dass die zugrundeliegende Löschung tatsächlich vollständig ist. `contextra_create_collection` hängt von den in B.1.5 spezifizierten Bereitstellungsprofilen ab. Verbindliche Umsetzungsreihenfolge: **B.1.1 → B.1.5 → B.1.4.**

**7. Aufwand und Priorität.** Mittel (vier Werkzeug-Handler zuzüglich Dispatch-Einträge und Tests). **P0-4, nach Abschluss von B.1.1 und B.1.5.**

---

### B.1.5 Bereitstellungsprofile: `PerformanceProfile` und `DeploymentTier` (P0-5)

**1. Problem.** Der Haltbarkeitsmodus (B.1.2) und der Vektor-Löschmodus (siehe unten) sind zueinander orthogonale Konfigurationsbausteine. Ein Entwickler müsste beide manuell konsistent konfigurieren — ohne eine übergreifende Validierungsschicht ist eine fehlerhafte, aber compilierbare Kombination leicht möglich.

**2. Soll.** Zwei neue Typen in der öffentlichen Builder-Schicht:

```rust
// Wahl des Vektor-Löschpfads (Gegenstück zum Haltbarkeitsmodus)
pub enum VectorDeleteMode { SynchronousRepair, BackgroundRepair }

// Gebündeltes technisches Preset
pub enum PerformanceProfile {
    #[default]
    Compliance, // DurabilityMode::Full + SynchronousRepair + Löschbeweis aktiv
    Balanced,   // WalNoHmac + BackgroundRepair, kein Löschbeweis
    BareMetal,  // MemoryOnly + BackgroundRepair, kein Löschbeweis
}

// Nutzerfreundliche, grob gerasterte Bereitstellungsprofile (öffentliche API)
pub enum DeploymentTier {
    EdgeMinimal,         // BareMetal, ca. 64 MB RAM-Zielbudget, reines Arbeitsgedächtnis
    PowerUserLocal,      // Balanced, ca. 512 MB RAM-Zielbudget, persistente Wissensbasis
    EnterpriseShared,    // Balanced + Crypto-Shredding für personenbezogene Daten
    EnterpriseRegulated, // Compliance, vollständiger Löschbeweis, kompatibel mit Audit-Export
}
```

Dies ist zugleich die konkrete Konfigurationsoberfläche, über die ein Nutzer die in Teil D beschriebenen Deployment-Szenarien wählt, ohne die darunterliegende Regelungstechnik (siehe 0.5, `contextra-adapt`) verstehen zu müssen.

**4. Invarianten.** `INV-PERF-PROFILE-1`: `BareMetal` und `Balanced` sind mit dem Löschbeweis-Feature inkompatibel. `INV-COLLECTION-PROFILE-1`: Ein aktiver Löschbeweis erfordert zwingend Crypto-Shredding auf der Key-Value-Seite (siehe B.1.6). `INV-COLLECTION-PROFILE-2`: Jedes `DeploymentTier`-Preset ist per Konstruktion mit der Validierungsfunktion konsistent — es kann keine invalide Voreinstellung geben.

**7. Aufwand und Priorität.** Niedrig bis mittel (Bündelungsschicht, geschätzt 250 Zeilen). **P0-5.**

---

### B.1.6 Crypto-Shredding für Key-Value-Segmente (P0-6)

**1. Problem.** Löschung auf WAL- und HNSW-Ebene ist mit B.1.1 und B.1.2 spezifiziert. Der Key-Value-Pfad (Cache- und Speicher-Crate) besitzt kein analoges Crypto-Shredding: Gelöschte Key-Value-Segmente werden aktuell nur kompaktierungsgebunden entfernt, nicht sofort und nicht unabhängig von einem Kompaktierungslauf.

**2. Soll.**

```rust
// crates/contextra-store/src/kv/delete_mode.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KvDeleteMode {
    #[default]
    CryptoShred,   // Segment-individueller, per HKDF abgeleiteter Unterschlüssel;
                   // Löschung = sofortige Schlüsselvernichtung, O(1)
    TombstoneOnly, // LSM-Tombstone, kompaktierungsgebunden
}
```

Bei `CryptoShred` liegt jedes Key-Value-Segment unter einem eigenen, per HKDF abgeleiteten Unterschlüssel. Löschung bedeutet Vernichtung dieses Unterschlüssels: Der Chiffretext bleibt physisch auf dem Datenträger bestehen, ist aber informationstheoretisch wertlos. Dies ist die Vorbedingung für einen Löschbeweis auf der Key-Value-Seite (Invariante `INV-KV-DELETE-1`).

**7. Aufwand und Priorität.** Mittel (geschätzt 250 Zeilen für die HKDF-Unterschlüsselverwaltung). **P0-6.**

---

### B.1.7 Eigenschaftsbasierter Test gegen HMAC-Kollisionen (P0-7)

Die Funktion zur längenpräfixierten Hashbildung über gelöschte Schlüssel benötigt einen eigenschaftsbasierten Test, der Segmentierungs-Mehrdeutigkeit ausschließt (Beispiel: die Zeichenfolgenfolge "ab", "c" darf nicht denselben Hash ergeben wie die Folge "a", "bc"). Es entsteht kein neuer Produktionscode. **P0-7 (niedriger Aufwand, hohe Sicherheitsrelevanz).**

---

### B.1.8 Laufzeit-Stresstest für die WAL-Wiederherstellung (P0-8)

Ein synthetisches Write-Ahead-Log mit einer Million Einträgen wird über das fehlerinjizierende virtuelle Testdateisystem erzeugt; der Wiedergabedurchsatz muss über den gesamten Testlauf hinweg über 50 Prozent des Anfangsdurchsatzes bleiben. Dieser Test deckt eingeschlichene quadratische Laufzeitkomplexität auf, die bei linearem Wachstum der Log-Größe unbemerkt bliebe. Neue Schnittstelle: ein Fortschritts-Sink-Trait für die Wiedergabe. **P0-8 (additiv, kein Breaking Change).**

---

## B.2 Phase 1: Fast-Ring öffentlich machen

Diese Phase bereitet den offenen, kostenlosen Standardring für eine öffentliche Bewerbung (Paket-Register-Veröffentlichung, öffentliche Ankündigung) vor.

### B.2.1 Invarianten-Tests für die Ausführungsschicht (P1-1)

Mit rund 17.800 Zeilen Code und der schwächsten Testdichte aller zentralen Crates (siehe A.7) benötigt die Ausführungsschicht vor jedem öffentlichen Launch drei deterministische Integrationstests für die drei wichtigsten Systeminvarianten:

```
crates/contextra-engine/tests/invariant_durability.rs
  Haltbarkeit: Nach WAL-Commit und simuliertem Absturz müssen die Daten nach
  Wiederherstellung vollständig vorhanden sein.
crates/contextra-engine/tests/invariant_isolation.rs
  Isolation: Parallele Transaktionen dürfen keine unbestätigten Schreibvorgänge
  anderer Transaktionen sehen.
crates/contextra-engine/tests/invariant_determinism.rs
  Determinismus: Dieselbe Eingabesequenz mit identischen injizierten Uhr-/
  Zufallszahlen-Seeds muss ein identisches Ergebnis liefern (P28).
```

**P1-1 — verbindliche Voraussetzung vor jeder Paket-Register-Veröffentlichung.**

### B.2.2 Bereinigung von Panic-Stellen im internen Optimierungssubsystem (P1-2)

Folgende Stellen im internen Bandit-/Regelungstechnik-Subsystem sind auf ordnungsgemäße Fehlerbehandlung umzustellen: eine `unwrap()`-Stelle in der linearen Banditen-Implementierung wird durch eine explizite Fehlerpropagation mit einer neuen Fehlervariante für ungültige Konfiguration ersetzt; drei `expect()`-Stellen in der Lyapunov-Drift-Regelung sind zunächst auf tatsächliche Erreichbarkeit zu prüfen — sind sie erreichbar, ist eine analoge `Result`-Propagation vorzunehmen. **P1-2.**

### B.2.3 Bereinigung toter Referenzen auf Phantom-Werkzeuge (P1-3, Vorstufe zu B.1.4)

Bis B.1.1 produktiv abgeschlossen ist: Die toten Referenzen auf die vier Phantom-Werkzeugnamen in der Sicherheits-Klassifikationslogik und in Tests werden bereinigt. Sicherheitshinweis: Die Klassifikationslogik ordnet diesen Namen aktuell die Kategorie "Datenbank-Schreibzugriff" zu, obwohl sie nie tatsächlich dispatcht werden — dies ist keine aktive Schwachstelle, aber irreführend für jede spätere Sicherheitsprüfung. **P1-3.**

### B.2.4 `contextra_plugin_status` implementieren (P1-4)

Ein MCP-Werkzeug zur Statusabfrage der Plugin-Registrierung ist bereits an anderer Stelle spezifiziert, aber an keiner Stelle im Workspace implementiert. **P1-4.**

### B.2.5 Benchmark-Dokumentation bereinigen (P1-5)

- Alle historischen Benchmark-Einträge werden explizit als veraltet gekennzeichnet, mit Verweis auf den Widerspruch zur zuletzt reproduzierten Messung.
- Ein einziger, kanonischer Benchmark-Commit wird erstellt, mit exakter Angabe der Build-Flags, der Hardware-Spezifikation und der expliziten Feststellung, dass Embedding-Latenz nicht Teil der gemessenen Zahl ist.
- Die DiskANN-Skalierungsgrenze bei rund 100.000 Dokumenten wird explizit als bekannte, bewusste Einschränkung kommuniziert (siehe A.6).

**P1-5 — verbindliche Voraussetzung vor jeder öffentlichen Ankündigung und vor der Paket-Register-Veröffentlichung.**

---

## B.3 Phase 2: Sovereign-Ring

### B.3.1 Signiertes Lizenz-Gate mit Ed25519 (P2-1)

**1. Problem.** Das Lizenz-Gate besitzt aktuell nur eine offene Stub-Implementierung als produktive Umsetzung. Der souveräne und der kommerzielle Compliance-Ring sind damit technisch nicht durchsetzbar — jeder Nutzer könnte diese Ringe ohne gültige Lizenz aktivieren.

**2. Soll.**

```rust
// crates/contextra-license/src/signed_gate.rs
pub struct SignedLicenseGate {
    verifying_key: ed25519_dalek::VerifyingKey,
    license_payload: LicensePayload,
}

pub struct LicensePayload {
    pub tenant_id: TenantId,
    pub allowed_rings: Vec<FeatureRing>,
    pub expires_at: Option<i64>,
    pub feature_flags: BTreeMap<String, bool>,
}

impl LicenseGate for SignedLicenseGate {
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {
        if !self.allowed_rings.contains(&ring) {
            return Err(LicenseError::NotActivated(ring));
        }
        if let Some(exp) = self.license_payload.expires_at {
            // Prüfung ausschließlich über den injizierten Clock-Port (P28)
        }
        Ok(())
    }
}
```

**4. Invariante.** `INV-LICENSE-2`: Der Standardring (`FeatureRing::Fast`) muss für jeden internen Systemzustand ohne gültige Lizenz erfolgreich prüfbar bleiben — der offene Ring darf durch das Lizenzsystem niemals blockiert werden.

**7. Aufwand und Priorität.** Mittel. **P2-1 — Voraussetzung für den kommerziell nutzbaren Compliance-Ring.**

### B.3.2 Vollständige Routing-Provenienz in der Erklärbarkeits-Ausgabe (P2-2)

Erweiterung des bestehenden Erklärbarkeits-Werkzeugs um eine vollständige Routing-Erklärung: Warum wurde ein bestimmtes Sprachmodell-Profil ausgewählt? Aufschlüsselung des oberen-Konfidenzschranke-Scores (Skalarprodukt-Term, Unsicherheitsbonus, Kostenabschlag), die knapp unterlegenen Alternativprofile, sowie der Drift-Status zum Entscheidungszeitpunkt. Rein additiv, keine Breaking Changes. **P2-2.**

---

## B.4 Phase 3: Skalierung (ausschließlich nach validierten Nutzern)

### B.4.1 DiskANN produktiv machen (P3-1)

Die plattenresidente DiskANN-Indexvariante ist aktuell hinter einem experimentellen Feature-Flag verborgen. Dies ist die einzige Komponente, die die RAM-Skalierungsgrenze von rund zwei Gigabyte bei 100.000 Dokumenten durchbricht. Sobald reale Nutzer mehr als 100.000 Dokumente benötigen, wird diese Aufgabe sofort zu P0. Bis dahin: Feature-Flag beibehalten, aber als dokumentierte erste Option für großvolumige Deployments kommunizieren. **P3-1.**

### B.4.2 Adaptive Suchparameter und inkrementelle Reparatur (P3-2)

Dynamische Anpassung des Suchparameters basierend auf einer laufenden Recall-Schätzung. Nur relevant oberhalb von etwa 500.000 Vektoren und bei spezifischen Recall-Anforderungen konkreter Kunden. **P3-2 (Backlog, bis validierter Bedarf vorliegt).**

---

# Teil C — Produktentscheidungen zu offenen Punkten {#teil-c}

## C.1 Der historische monolithische Zugang — Abbaupfad

Die historisch gewachsene, monolithische Fassade (`contextra-db`) bleibt ausschließlich zur Abwärtskompatibilität bestehender Integrationen erhalten, wird aber nicht mehr weiterentwickelt. Neue Funktionalität entsteht ausschließlich in der Ausführungsschicht (`contextra-engine`) und wird über die neue, öffentliche Composition-Root (`contextra`, Ring 4, Builder-Muster) zugänglich gemacht. Diese Composition-Root ist der alleinige empfohlene Einstiegspunkt für neue Integrationen.

## C.2 Phantom-MCP-Werkzeuge: implementieren, nicht entfernen

Entscheidung: Die vier Phantom-Werkzeugnamen (siehe A.5, B.1.4) werden implementiert, nicht ersatzlos entfernt. Begründung: Ein MCP-Interface ohne vollständige CRUD-Operationen ist für Agenten-Nutzer nicht ausreichend — ein reines Einfüge-Werkzeug allein deckt keinen vollständigen Speicher-Workflow ab, den ein Agent im Alltag benötigt (Aktualisieren, gezieltes Löschen mit Beweis, Collection-Verwaltung).

## C.3 Das interne Optimierungssubsystem bleibt intern

Das interne Bandit-/Regelungstechnik-Subsystem erscheint an keiner Stelle in öffentlich gerichteter Dokumentation, nicht in der öffentlichen API und nicht in Benchmark-Kommunikation. Nutzer konfigurieren ausschließlich über die in B.1.5/D.3 beschriebenen Bereitstellungsprofile; intern entscheidet das System selbst über die anzuwendende Optimierungsstrategie. Die öffentliche Erzählung lautet konsequent: "Das System optimiert sich selbst, du musst nichts konfigurieren." Dies ist keine vorübergehende Vereinfachung, sondern eine dauerhafte Designentscheidung (siehe 0.5).

## C.4 Gedächtniskonsolidierung — Validierung vor Bewerbung erforderlich

Die Gedächtniskonsolidierungslogik (rund 6.900 Zeilen) ist bislang unzureichend gegen reale Nutzungsszenarien getestet — die vorhandenen Tests liegen überwiegend außerhalb dedizierter Testverzeichnisse und decken primär synthetische Vektordaten ab, nicht einen realen mehrstufigen Sprachmodell-Agenten-Workflow. Vor jeder öffentlichen Bewerbung dieses Features ist eine konkrete Demonstration erforderlich: Ein Beispielprojekt oder Notizbuch, das einen realen Dialog mit einem Sprachmodell über mindestens zehn Gesprächsrunden führt, eine Konsolidierung auslöst und das Ergebnis nachvollziehbar darstellt.

## C.5 Benchmark-Glaubwürdigkeit

Die 34-fache Diskrepanz zwischen historischen und aktuell reproduzierten Messwerten (siehe A.6) muss vor jeder öffentlichen Ankündigung erklärt sein. Die wahrscheinlichste Ursache — historische Messung schloss die Latenz eines externen Embedding-Aufrufs ein, aktuelle Messung misst ausschließlich Speicher- und Indexlatenz — ist explizit in der Benchmark-Dokumentation zu vermerken, nicht nur intern bekannt zu sein. Ein Nutzer, der die alte Zahl sieht und selbst eine abweichende Messung erhält, verliert andernfalls berechtigterweise Vertrauen in alle veröffentlichten Zahlen.

## C.6 Feature-Ring-Standort

Die Feature-Ring- und Lizenzfehler-Abstraktion liegt bereits korrekt in der zentralen Port-Schicht (Ring 0, objektsicher) und wird von der Lizenz-Schicht ausschließlich zur Abwärtskompatibilität re-exportiert. Dieser strukturelle Punkt ist bereits erfüllt und erfordert keine weitere Maßnahme.

## C.7 Der Speicherkern bleibt eigenständig — keine Migration auf eine generische Storage-Engine

Eine naheliegende Überlegung wäre, den eigenentwickelten LSM-Baum-Speicherkern durch eine etablierte, generische Speicher-Engine (etwa eine reine Rust-B-Baum- oder LSM-Bibliothek eines Drittanbieters) zu ersetzen, um von deren Betriebsstunden und Härtungsgrad zu profitieren. **Diese Spezifikation trifft die gegenteilige Entscheidung: Der eigenentwickelte Speicherkern bleibt bestehen und wird nicht ersetzt.**

**Begründung.** Der bestehende Speicherkern verfügt über eine HMAC-verkettete Integritätssicherung im Write-Ahead-Log (jeder Log-Eintrag enthält den HMAC seines Vorgängers — eine nachweisbare Manipulationserkennungskette, die keine generische Storage-Engine von Haus aus bietet), über einen expliziten Tombstone-Bereinigungswächter, der verhindert, dass als gelöscht markierte Daten entfernt werden, solange ein aktiver Snapshot noch auf sie verweist (die Storage-Ebene des Löschbeweises), sowie über eine belastbare Anzahl dedizierter Wiederherstellungs- und Kompaktierungstests und die in A.7 dokumentierte Zero-Panic-Sauberkeit. Eine Migration auf eine generische Engine würde zwar Betriebsstunden "erben", aber genau den Baustein verwerfen, der die technische Grundlage des zentralen Alleinstellungsmerkmals (kryptografisch beweisbare Löschung) bildet — dieser HMAC-verkettete, löschbeweisfähige WAL ist domänenspezifisch und existiert in keiner generischen Speicher-Engine.

**Konsequenz statt Ersatz: externe Verifikation.** Das eigentliche Problem ist nicht die technische Qualität des Speicherkerns, sondern das fehlende externe Vertrauen in ihn (siehe Teil F.2). Dieses Problem wird durch Ersatz nicht gelöst, sondern durch gezielte externe Prüfung: öffentlich dokumentiertes Chaos-Testing (das vorhandene fehlerinjizierende virtuelle Testdateisystem wird aktiv für dokumentierte Absturzszenarien genutzt: WAL-Kürzung, Teilschreibungen, HMAC-Kettenbruch nach Absturz) sowie ein bezahltes externes Review durch eine in Rust-Speichersystemen erfahrene Person oder Organisation, dessen Ergebnis öffentlich referenziert werden kann.

## C.8 Größtes technisches Einzelrisiko: die Ausführungsschicht

Die Ausführungsschicht (`contextra-engine`, siehe A.2, A.7) bildet den gesamten Compute-Pool des Systems und weist gleichzeitig die schwächste Test-zu-Code-Ratio aller zentralen Komponenten auf. Ein Fehler an dieser Stelle wirkt sich auf das gesamte darüberliegende System aus. Die in B.2.1 spezifizierten drei Invarianten-Tests sind die Mindestmaßnahme vor jedem öffentlichen Launch und wichtiger als jede zusätzliche Feature-Implementierung in diesem Zeitraum.

---

# Teil D — Das Endprodukt: vollständige Nutzerperspektive {#teil-d}

Dieser Teil beschreibt, was ein Nutzer konkret bekommt, wenn er Contextra einsetzt — unabhängig von internen Implementierungsdetails. Er ist die verbindliche Referenz dafür, wie das Produkt in Dokumentation, Marketing und Verkaufsgesprächen dargestellt wird.

## D.1 Produktform 1: die Rust-Bibliothek

**Installation:**

```toml
# Cargo.toml des Nutzerprojekts
[dependencies]
contextra = "0.1"
```

Das ist die vollständige Installationsprozedur. Es gibt keinen zweiten Prozess zu starten, keinen Datenbankserver zu betreiben, keine Netzwerkverbindung zu konfigurieren, keine Cloud-Anmeldedaten zu hinterlegen. Die Bibliothek läuft im selben Betriebssystemprozess wie die Anwendung des Nutzers.

**Minimalbeispiel:**

```rust
use contextra::{Contextra, DeploymentTier};

let memory = Contextra::builder()
    .tier(DeploymentTier::PowerUserLocal)
    .path("./agent-memory")
    .build()?;

memory.insert("Der Nutzer bevorzugt kurze, direkte Antworten.")?;

let results = memory.search("Wie soll ich antworten?", 5)?;
```

**Was der Entwickler ohne weitere Konfiguration erhält (Standardring, kostenlos, MIT/Apache-2.0):**
- Hybride Suche über Vektor-Ähnlichkeit, Volltext (BM25F, deutsche Morphologie) und Graph-Nachbarschaft gleichzeitig, fusioniert über reziproke Rang-Fusion mit Kalibrierung.
- Automatische Extraktion von Wissensgraph-Tripeln aus eingefügtem Freitext (siehe B.1.3).
- Agenten-spezifische KV-Cache-Steuerung mit expliziten Direktiven (dauerhaft vorhalten, nie zwischenspeichern, nach einem definierten Schritt freigeben).
- Vollständiger Determinismus: Bei Verwendung einer injizierten, festen Uhr- und Zufallsquelle liefert eine identische Eingabesequenz ein bit-identisches Ergebnis — Grundlage für reproduzierbares Testen, Replay-Debugging und Audit von Agentenverhalten.
- Ghost-Vektor-freie Löschung (nach Fertigstellung von B.1.1): Löschen bedeutet tatsächlich vollständiges Löschen, nicht nur Entfernen aus dem primären Index.

**Was zusätzlich mit dem souveränen Ring (`features = ["sovereign"]`, weiterhin quelloffen) verfügbar ist:**
- Vollständiger kryptografischer Löschbeweis: eine überprüfbare, Ed25519-signierte Bescheinigung, dass ein Dokument aus allen Speicherschichten (WAL, Vektorindex, Key-Value-Segmente) physisch entfernt oder kryptografisch unwiederherstellbar gemacht wurde.
- Crypto-Shredding für personenbezogene Daten auf Key-Value-Ebene.
- Fünfschichtiges Egress-Gateway und PII-Tresor zur Vermeidung von unbeabsichtigtem Abfluss sensibler Daten an externe Sprachmodell-Anbieter.

**Was zusätzlich mit dem kommerziellen Compliance-Ring (`features = ["compliance"]`, kostenpflichtige Lizenz) verfügbar ist:**
- Mandantenfähige Isolation für Mehrmandantenbetrieb.
- Automatisierter Export für Nachweispflichten (Mapping auf Anforderungen des BSI-Grundschutzes, DSGVO-Verarbeitungsverzeichnis-Unterstützung).
- Generierung von Auftragsverarbeitungsvertrags-Vorlagen.
- Priorisierter Support und erweiterte Persistenz-Tuning-Voreinstellungen.

## D.2 Produktform 2: der MCP-Server

Für Nutzer, die keinen eigenen Rust-Code schreiben, sondern Contextra als Gedächtnisschicht für einen bestehenden KI-Assistenten (Claude Desktop, Cursor, VS Code mit Copilot-artiger Integration) nutzen wollen, steht ein eigenständiger Server zur Verfügung, der über das Model Context Protocol per Standard-Ein-/Ausgabe kommuniziert.

**Installation und Einbindung:** Der Server wird als eigenständige Binärdatei bereitgestellt und in der Konfiguration des jeweiligen MCP-Clients als lokaler Prozess eingetragen. Es findet keine Netzwerkkommunikation nach außen statt; die gesamte Kommunikation läuft über Standard-Ein-/Ausgabe zwischen dem MCP-Client und dem lokal laufenden Server-Prozess.

**Verfügbare Werkzeuge nach Fertigstellung von B.1.4** (vollständiges CRUD-Set für Agenten-Workflows):

| Werkzeug | Zweck aus Nutzersicht |
|---|---|
| `contextra_search` | "Was weiß ich schon über X?" |
| `contextra_insert` | "Merke dir diese neue Information." |
| `contextra_upsert` | "Aktualisiere diesen bekannten Fakt." |
| `contextra_get` | "Zeig mir genau diesen gespeicherten Eintrag." |
| `contextra_delete` | "Vergiss das — und beweise es mir kryptografisch." |
| `contextra_collections` | "Welche Wissensbereiche verwaltest du gerade?" |
| `contextra_create_collection` | "Leg einen neuen, isolierten Wissensbereich an." |
| `contextra_drop_collection` | "Lösche diesen gesamten Wissensbereich vollständig." |
| `contextra_consolidate` | "Fasse das bisherige Gespräch zu dauerhaftem Wissen zusammen." |
| `contextra_relate` / `contextra_relate_n_ary` | "Verknüpfe diese Konzepte miteinander." |
| `contextra_explain` | "Warum hast du gerade diese Information als relevant ausgewählt?" |
| `contextra_plugin_status` | "Welche optionalen Erweiterungen sind aktiv?" |

Diese Werkzeugliste ist bewusst vollständig CRUD-fähig gehalten (siehe C.2): Ein Agent, der Gedächtnis verwaltet, benötigt nicht nur die Möglichkeit, etwas zu speichern, sondern ebenso, es gezielt zu korrigieren, zu organisieren und nachweisbar zu löschen.

## D.3 Deployment-Tiers im Detail

Die in B.1.5 spezifizierten vier Bereitstellungsprofile sind die zentrale, für Endnutzer sichtbare Konfigurationsoberfläche. Sie abstrahieren die darunterliegende technische Komplexität (Haltbarkeitsmodus, Vektor-Löschpfad, Regelungstechnik) vollständig weg.

| Profil | Technische Grundlage | Typisches RAM-Zielbudget | Löschbeweis | Typischer Anwendungsfall |
|---|---|---|---|---|
| `EdgeMinimal` | Rein speicherresident, Hintergrund-Reparatur | ca. 64 MB | Nein | Kurzlebiges Arbeitsgedächtnis eines Agenten während einer einzelnen Sitzung, Edge-Geräte mit knappem Speicherbudget |
| `PowerUserLocal` | WAL ohne HMAC-Kette, Hintergrund-Reparatur | ca. 512 MB | Nein | Persönliche, dauerhafte Wissensbasis eines einzelnen Entwicklers oder einer einzelnen Anwendung |
| `EnterpriseShared` | WAL ohne HMAC-Kette + Crypto-Shredding für personenbezogene Daten, Hintergrund-Reparatur | konfigurierbar | Teilweise (nur PII-Segmente) | Mehrnutzer-Anwendung ohne vollständige, aber mit selektiver Nachweispflicht |
| `EnterpriseRegulated` | Vollständiges WAL mit HMAC-Kette, synchrone Reparatur | konfigurierbar | Vollständig | Regulierte Branchen mit vollständiger DSGVO-Art.-17-Nachweispflicht (Gesundheitswesen, Rechtsberatung, öffentliche Verwaltung über Systemhaus-Partner) |

Ein Entwickler wählt genau eines dieser vier Worte als Konfigurationswert — nicht mehr. Die interne Regelungstechnik (siehe C.3) entscheidet innerhalb der durch das gewählte Profil gesetzten Grenzen selbstständig über feingranulare Optimierung.

## D.4 Lizenzmodell und Ring-Grenzen

Das Geschäftsmodell folgt dem bewährten Muster quelloffener, kommerziell dual-lizenzierter Infrastrukturprojekte (vergleichbare Strukturen bestehen etwa im Umfeld eingebetteter Datenbanken und Suchbibliotheken):

| Ring | Lizenz | Inhalt | Aktivierung |
|---|---|---|---|
| **Fast** | MIT/Apache-2.0, vollständig quelloffen | Hybrides Retrieval, KV-Cache-Direktiven, Determinismus-Garantien, Ghost-Vektor-freie Löschung | Standard, keine Aktion erforderlich |
| **Sovereign** | MIT/Apache-2.0, quelloffen, funktional gated | Vollständiger kryptografischer Löschbeweis, Crypto-Shredding, Egress-Gateway | `features = ["sovereign"]` |
| **Compliance** | Kommerzielle Lizenz | Mandantenfähigkeit, Audit-Export, AVV-Generator, priorisierter Support | `features = ["compliance"]` + gültige signierte Lizenz (B.3.1) |

**Grundsatz zur Ringgrenze.** Der Standardring muss großzügig und uneingeschränkt funktionsfähig sein — eine künstlich beschnittene kostenlose Stufe zerstört das für die Adoption notwendige Vertrauen. Die Grenze zwischen dem offenen souveränen Ring und dem kommerziellen Compliance-Ring verläuft entlang echter, durch reale Kundengespräche belegter Zahlungsbereitschaft, nicht entlang des internen Implementierungsaufwands. Diese Grenze wird vor endgültiger Festschreibung im Code durch die in Teil E beschriebenen Kundengespräche empirisch validiert.

## D.5 Konkrete Nutzer-Workflows

**Workflow 1 — Rust-Entwickler baut einen Agenten mit Gedächtnis.** Der Entwickler fügt die Abhängigkeit hinzu, konfiguriert `DeploymentTier::PowerUserLocal`, und erhält innerhalb weniger Codezeilen eine funktionsfähige, lokale Gedächtnisschicht mit hybrider Suche — ohne einen zusätzlichen Prozess zu betreiben, ohne eine Cloud-Anmeldedaten-Konfiguration, ohne eine Datenbank separat zu administrieren.

**Workflow 2 — Systemhaus integriert Contextra in eine Fachverfahren-Lösung für einen Behördenkunden.** Das Systemhaus wählt `DeploymentTier::EnterpriseRegulated`, aktiviert den souveränen und ggf. den kommerziellen Compliance-Ring, und kann gegenüber der Datenschutzaufsichtsbehörde des Endkunden einen kryptografisch überprüfbaren Löschnachweis vorlegen — ohne dass die Behörde selbst Zugriff auf den internen Speicher benötigt, um die Löschung zu verifizieren.

**Workflow 3 — Nicht-Rust-Entwickler nutzt Contextra über Claude Desktop.** Der Nutzer trägt den `contextra-mcp`-Server in seine Client-Konfiguration ein und erhält ab dem nächsten Sitzungsstart ein persistentes, lokales Gedächtnis für seinen KI-Assistenten, das über Sitzungsgrenzen hinweg erhalten bleibt, vollständig lokal gespeichert wird und über die in D.2 gelisteten Werkzeuge gesteuert werden kann.

**Workflow 4 — Entwickler debuggt nicht-deterministisches Agentenverhalten.** Durch Verwendung einer injizierten, festen Zufallsquelle anstelle der Systemzeit kann derselbe Agentenlauf beliebig oft mit identischem Ergebnis wiederholt werden — ein Debugging-Workflow, der bei den meisten Wettbewerbsprodukten mangels erzwungenem Determinismus nicht in dieser Form möglich ist.

---

# Teil E — Marktstrategie und Geschäftsmodell {#teil-e}

## E.1 Zwei-Wege-Strategie

Es existieren zwei kohärente, aber unterschiedliche Vermarktungswege. Beide sind legitim, aber sie können von einer einzelnen Person nicht gleichzeitig mit voller Kraft verfolgt werden:

**Weg A — Entwickler-Bibliothek (quelloffen, Community-getrieben).** Ziel: Veröffentlichung auf einem öffentlichen Paket-Register, erste organische Nutzer unter Rust-Entwicklern, die Agenten bauen. Erforderlicher Aufwand: Scope einfrieren (Teil 0.5/0.8), Benchmark bereinigen (B.2.5), Dokumentation vereinfachen. Realistischer Markteintritt: vier bis sechs Wochen.

**Weg B — Compliance-Produkt (kommerziell, vertriebsgetrieben).** Ziel: ein erster zahlender Pilotkunde in einer regulierten Branche. Erforderlicher Aufwand: mindestens zehn direkte Gespräche mit potenziellen Kunden innerhalb der nächsten 30 Tage. Realistischer Markteintritt: drei bis sechs Monate.

**Verbindliche Reihenfolge: Weg A zuerst, dann Weg B mit der so aufgebauten Sichtbarkeit als Vertrauensbeweis.** Weg A baut in kurzer Zeit Sichtbarkeit und technisches Vertrauen auf, die für Weg B notwendig sind — niemand kauft ein selbstgebautes, kryptografisch abgesichertes Speichersystem mit Löschbeweis-Versprechen von einem Einzelentwickler ohne jede sichtbare Nutzerbasis oder externe Verifikation, unabhängig von der tatsächlichen Codequalität. Das ist keine technische Frage, sondern eine Vertrauensfrage, und diese wird nicht durch weitere Features gelöst, sondern durch Sichtbarkeit, externe Prüfung (siehe F.2) und Zeit.

## E.2 Vertikal-Fokus-Entscheidung

"Systemhäuser" (siehe 0.4) ist eine Vertriebskanal-Entscheidung, kein Marktsegment. Für Weg B muss zusätzlich ein enger vertikaler Fokus schriftlich festgelegt werden — **eine Branche, nicht alle gleichzeitig:**

| Option | Kernargument | Konkreter Beweispunkt für den Löschbeweis |
|---|---|---|
| Medizinische KI | DSGVO-Artikel-17-Nachweispflicht ist im Gesundheitswesen existenziell; Krankenhausinformationssysteme und diagnostische KI verarbeiten hochsensible Patientendaten. | Der Löschbeweis ist exakt die Technologie, die ein Anbieter medizinischer KI benötigt, um eine Datenschutzaufsichtsbehörde zu überzeugen. |
| Rechtliche KI (Legal Tech) | Anwaltskanzleien und Legal-Tech-Anbieter verarbeiten hochsensible Mandantendaten und benötigen nachweisbare Löschung; der deutsche Markt umfasst über 160.000 zugelassene Anwälte mit strenger Berufsordnung. | Nachweisbare Löschung als Baustein für Mandantendatenschutz-Zusicherungen. |
| Industrielle Robotik / Edge-KI | Fertigungsanlagen können keine Cloud-Abhängigkeit tolerieren; "air-gap" ist hier keine Compliance-Zierde, sondern technische Notwendigkeit. Passt zudem an den in 0.1 beschriebenen ursprünglichen Projektkontext an. | Air-gap-Betrieb als überprüfbarer Systemzustand für ein Gerät ohne Cloud-Anbindung. |

**Verbindliche Vorgabe:** Die Entscheidung für genau eine dieser drei Optionen erfolgt nicht am Schreibtisch, sondern über zehn direkte Gespräche mit tatsächlichen Entwicklern oder Entscheidungsträgern in einem dieser Bereiche (siehe Teil G, Roadmap). Diese Entscheidung wird schriftlich in einer Ergänzung zu diesem Dokument festgehalten, sobald sie getroffen ist.

## E.3 Vertriebskanal

**Für Weg A:** `cargo add` als Kaufentscheidung, keine Vertragsverhandlung. Sichtbarkeit über Benchmarks, das öffentliche Repository, eine Ankündigung auf einer einschlägigen Entwickler-Plattform sowie direkte Ansprache von Betreibern anderer Rust-Agenten-Frameworks, die aktuell kein natives Gedächtnis-Backend anbieten.

**Für Weg B:** Kein Selbstbedienungsmodell mit Kreditkarte. Direkter Kontakt zu Entscheidungsträgern (typischerweise CTOs) von Systemhäusern, Angebot eines Pilotprojekts, das sich durch Beratungsleistung selbst finanziert; die Lizenzierung des Compliance-Rings folgt danach, sobald Vertrauen aufgebaut ist. Direktverkauf an Bundes- oder Landesbehörden wird bewusst nicht verfolgt (siehe 0.4, 0.6).

## E.4 Geschäftsmodell im Detail

Die Struktur folgt dem in D.4 beschriebenen Drei-Ring-Modell. Wesentlich ist, dass der offene Standardring nicht als Lockmittel für einen kommerziellen Ring künstlich beschnitten wird, sondern aus sich heraus vollständig überzeugend sein muss — Entwickler bauen Vertrauen zu einem Werkzeug auf, das sie im Alltag ohne Einschränkung nutzen können, nicht zu einem, das ihnen ständig eine Bezahlschranke zeigt. Der kommerzielle Ring monetarisiert ausschließlich Funktionen, die eine reale, durch Kundengespräche belegte Zahlungsbereitschaft aufweisen: Mandantenfähigkeit, automatisierter Nachweis-Export und priorisierter Support für Organisationen, die selbst keine Compliance-Tooling-Kompetenz aufbauen wollen.

## E.5 Förderprogramm

Siehe 0.6 für die vollständige Beschreibung des Bundesförderprogramms mit Bewerbungsfrist 11. Oktober 2026. Die konkrete Umsetzung dieses Punkts ist Teil der 60-Tage-Roadmap (Teil G).

---

# Teil F — Qualitätssicherung und Vertrauensaufbau {#teil-f}

## F.1 Risikoproportionale Testtiefe

Vollständige Testabdeckung ("100 Prozent") existiert bei keinem Speichersystem, auch nicht bei etablierten Referenzimplementierungen. Der richtige Maßstab ist risikoproportionale Testtiefe, in vier Eskalationsstufen:

1. **Identifikation der Invarianten, die niemals brechen dürfen.** Für Contextra sind dies konkret: kein Datenverlust nach einem bestätigten fsync, kein Sichtbarkeits-Leck zwischen Mandanten, und der Löschbeweis lügt niemals (das heißt: er wird ausschließlich nach tatsächlich vollständiger physischer Bereinigung ausgestellt, siehe `INV-DELETION-1`/`INV-DELETION-2`). Alles außerhalb dieser Kernmenge darf iterativ verbessert werden und muss nicht von Beginn an fehlerfrei sein.
2. **Für genau diese Invarianten: eigenschaftsbasiertes Testen, Fuzzing und deterministische Simulationstests.** Der im System bereits erzwungene Determinismus (P28: injizierte Uhr, injizierter Zufallszahlengenerator) ist die ideale technische Grundlage für deterministisches Simulationstesten nach dem Vorbild verteilter Speichersysteme mit hohem Korrektheitsanspruch: Abstürze, Uhrsprünge und Netzwerkausfälle werden mit einem festen Seed deterministisch simuliert, sodass tausende Szenarien in Sekunden statt Stunden durchlaufen werden können.
3. **Fehlerinjektion auf Dateisystemebene.** Das bereits vorhandene fehlerinjizierende virtuelle Testdateisystem im Test-Werkzeugkasten wird aktiv und systematisch gegen den Speicherkern eingesetzt (siehe C.7, F.2).
4. **Für alles Übrige: normale Unit- und Integrationstests, kontinuierliche Integration, und reale Nutzung als zusätzliches Prüfverfahren.** Frühes Ausliefern an echte Nutzer ist selbst ein Testverfahren — der Anspruch ist nicht "vollständiges Vertrauen in jede Codezeile", sondern chirurgische Präzision genau dort, wo ein Fehler dem Kunden am meisten schaden würde.

## F.2 Externe Verifikation statt Ersatz des Storage-Kerns

Wie in C.7 begründet, wird der Speicherkern nicht durch eine generische Alternative ersetzt. Stattdessen wird Vertrauen über zwei konkrete, öffentlich sichtbare Maßnahmen aufgebaut:

1. **Öffentlich dokumentiertes Chaos-Testing.** Ein Bericht (Blogpost oder gleichwertiges Format), der konkret zeigt, welche Absturzszenarien systematisch getestet werden (WAL-Kürzung durch abgebrochenes Schreiben, Teilschreibungen, HMAC-Kettenbruch nach einem simulierten Absturz) und mit welchem Ergebnis. Dies ist glaubwürdiger als jede zusätzliche Spezifikationsseite.
2. **Ein bezahltes externes Review.** Eine in Rust-basierten Speichersystemen erfahrene externe Person oder Organisation wird um ein bezahltes Review des Write-Ahead-Log- und Kompaktierungskerns gebeten. Ein öffentlich sichtbarer Vermerk ("geprüft von [Name]") auf dem entsprechenden Commit ist mehr wert als zehn zusätzliche Features und ist eine vergleichsweise kostengünstige Vertrauensinvestition.

## F.3 Markenidentität bereinigen

Vor jeder öffentlichen Sichtbarkeitsmaßnahme (Paket-Register-Veröffentlichung, öffentliche Ankündigung) sind sämtliche Referenzen auf den vorherigen Arbeitsnamen des Projekts aus Code-Kommentaren, internen Ankertexten und öffentlich sichtbaren Artefakten zu entfernen. Dies ist eine rein hygienische, aber notwendige Aufgabe vor jeder externen Kommunikation (siehe A.7).

## F.4 Verhaltensgovernance

Siehe 0.8 für die vollständigen, verbindlichen Governance-Regeln. Ergänzend hier festgehalten: Die in 0.1 und 0.8 beschriebene Tendenz, technisch interessante, aber strategisch nicht priorisierte Erweiterungen trotz gegenteiliger Empfehlung umzusetzen, ist die eigentliche Ursache dafür, dass ein technisch überdurchschnittlich diszipliniertes Projekt bislang wirtschaftlich wirkungslos geblieben ist. Diese Erkenntnis ist wichtiger als jede technische Einzelentscheidung in diesem Dokument und ist bei jeder künftigen Priorisierungsentscheidung explizit zu berücksichtigen — auch und gerade dann, wenn ein Sprachmodell im Rahmen eines Einzelauftrags eine technisch elegante, aber nicht spezifizierte Erweiterung vorschlägt.

---

# Teil G — Roadmap 30 / 60 / 90 Tage {#teil-g}

## G.1 Sofort — heute, Risiko null

```bash
# 1. MCP-Server-Default korrigieren (siehe 0.9)
# crates/contextra-mcp/Cargo.toml:
# contextra = { workspace = true, features = ["router"] }

# 2. Standard-Kompiliermitgliedschaft bereinigen (siehe A.3)
# Framework-Adapter-Crate für Python-Ökosysteme vollständig entfernen (git rm -r)
# Sandbox-, Ollama- und ONNX-Crates aus default-members entfernen (Code bleibt erhalten)

# 3. Inferenz-Standard setzen (siehe A.4)
# crates/contextra/Cargo.toml [features]: default = ["fast", "candle"]

# 4. Verifikation
./verify_workspace.sh --dag-check
./verify_workspace.sh --panic-inventory
./verify_workspace.sh --feature-matrix --only contextra,contextra-mcp
./verify_workspace.sh --fast

git commit -m "chore(scope): Ausschlüsse gemäß Gesamtspezifikation umsetzen"
```

## G.2 30 Tage — technische Schulden tilgen

| Priorität | Aufgabe | Spezifikationsabschnitt | Aufwand | Abhängigkeit |
|---|---|---|---|---|
| P0-1 | Geisterknoten-Schutz im Vektorindex | B.1.1 | Mittel–hoch | — |
| P0-2 | Haltbarkeitsmodus-Enum und Validierung | B.1.2 | Mittel | — |
| P0-3 | Automatische Extraktion standardmäßig aktivieren | B.1.3 | Niedrig | — |
| P0-7 | Eigenschaftsbasierter HMAC-Kollisionstest | B.1.7 | Niedrig | — |
| P0-8 | WAL-Wiederherstellungs-Stresstest | B.1.8 | Niedrig | — |
| P1-1 | Drei Invarianten-Tests für die Ausführungsschicht | B.2.1 | Niedrig | — |
| P1-2 | Panic-Bereinigung im internen Optimierungssubsystem | B.2.2 | Niedrig | — |
| P1-5 | Benchmark-Dokumentation bereinigen | B.2.5 | Niedrig | — |
| Zusatz | Markenidentität bereinigen | F.3 | Niedrig | — |
| Zusatz | Beratungsgespräch Exportkontrolle (BAFA oder spezialisierte Kanzlei) | 0.7 | Gering, Kosten 200–400 € | Vor erstem Kontakt mit verteidigungsnahen Kunden |

**Meilenstein nach 30 Tagen:** Die vollständige Testsuite läuft grün, und es existiert genau eine kanonische, konsistente Quelle der Wahrheit für Benchmark-Zahlen.

## G.3 60 Tage — Sichtbarkeit und erste Validierung

| Aufgabe | Erfolgskriterium | Abhängigkeit |
|---|---|---|
| Bereitstellungsprofile produktiv | B.1.5 produktiv, kontinuierliche Integration grün | B.1.2 |
| Crypto-Shredding für Key-Value-Segmente | B.1.6 produktiv | B.1.2 |
| Invarianten-Tests für die Ausführungsschicht | Drei Tests grün, Ausführungsschicht stabil | — |
| **Veröffentlichung auf dem öffentlichen Paket-Register** | Erste Downloads ohne aktive Eigenwerbung | vorangehende P0-/P1-Aufgaben |
| **Öffentliche Ankündigung mit Titel entlang "Rust Memory Engine mit kryptografischem Löschbeweis"** | Mehr als fünf ernsthafte Rückfragen oder Kommentare | Bereinigte Benchmarks (B.2.5) |
| Zehn direkte Gespräche mit potenziellen Nutzern | Mindestens drei Antworten auf die Frage "Wofür würdest du zahlen?" | — |
| **Bewerbung beim Bundesförderprogramm "Digital-Tech-to-Product"** (Bewerbungsfrist 11. Oktober 2026) | Bewerbung fristgerecht eingereicht | Businessplan (drei bis fünf Seiten) und technischer Demonstrator |

**Hinweis zur Förderfrist:** Der erste Förderaufruf endet am 11. Oktober 2026. Der technische Demonstrator existiert bereits in Form der Bibliothek zuzüglich des bereinigten, reproduzierten Benchmarks; der Businessplan ist gesondert zu erstellen. Diese Aufgabe ist zeitkritisch und darf nicht auf den letzten Tag der Frist verschoben werden.

## G.4 90 Tage — erste Zahlungsbereitschaft validieren

| Aufgabe | Erfolgskriterium |
|---|---|
| Vier Phantom-MCP-Werkzeuge produktiv | B.1.4 vollständig, einschließlich `contextra_delete` mit vollständigem Löschbeweis |
| Signiertes Lizenz-Gate | B.3.1 produktiv; erster kommerzieller Ring aktivierbar |
| Ein Pilotkunde (Systemhaus oder Direktentwickler) | Zahlt für Implementierungsunterstützung oder eine Lizenz |
| Demonstrationsprojekt für die Gedächtniskonsolidierung | Gegen einen realen mehrstufigen Sprachmodell-Workflow validiert (siehe C.4) |
| Schriftliche Vertikal-Marktentscheidung | Medizinische KI **oder** Legal-Tech **oder** Industrial-KI — eine Branche, nicht alle drei (siehe E.2) |

---

# Anhang R — Referenztabellen {#anhang-r}

## R.1 Neue Fehlervarianten

| Variante | Herkunft | Kategorie |
|---|---|---|
| `GraphRepairFailed(HnswDeletionError)` | B.1.1 | Vektor/Embedding |
| `DurabilityModeConfig(&'static str)` | B.1.2 | Konfiguration/Grenzwerte |
| `KvDeleteModeConfig(&'static str)` | B.1.6 | Konfiguration/Grenzwerte |
| `PerformanceProfileConfig(PerformanceProfileError)` | B.1.5 | Konfiguration/Grenzwerte |
| `CollectionProfileConfig(CollectionProfileError)` | B.1.5 | Konfiguration/Grenzwerte |
| `LicenseExpired { tenant_id, expired_at }` | B.3.1 | Konfiguration/Grenzwerte |
| `SyncConflictUnresolved` | Backlog (zurückgestellt, siehe 0.5) | Nebenläufigkeit |

## R.2 Neue und präzisierte Invarianten

| Invariante | Herkunft | Kurzbeschreibung |
|---|---|---|
| `INV-DELETION-2` | B.1.1 | Nach Graph-Reparatur-Löschung zeigt kein Nachbarschaftszeiger mehr auf den gelöschten Knoten. |
| `INV-DURABILITY-RING` | B.1.2 | Rein speicherresidenter Modus ist inkompatibel mit Löschbeweis und souveränem Ring. |
| `INV-PERF-PROFILE-1` | B.1.5 | `BareMetal`/`Balanced` sind inkompatibel mit dem Löschbeweis-Feature. |
| `INV-KV-DELETE-1` | B.1.6 | Löschbeweis auf Key-Value-Seite ausschließlich für Crypto-Shred-Segmente möglich. |
| `INV-COLLECTION-PROFILE-1` | B.1.5 | Aktiver Löschbeweis erfordert Crypto-Shredding auf Key-Value-Seite. |
| `INV-COLLECTION-PROFILE-2` | B.1.5 | Jedes Bereitstellungsprofil-Preset ist per Konstruktion konsistent. |
| `INV-LICENSE-2` | B.3.1 | Der Standardring muss für jeden internen Zustand ohne Lizenz erfolgreich prüfbar bleiben. |

## R.3 Vollständige Liste zurückgestellter Funktionsbereiche mit Reaktivierungsbedingung

| Funktionsbereich | Grund der Zurückstellung | Reaktivierungsbedingung |
|---|---|---|
| Framework-Adapter für Python-Ökosysteme | Widerspricht der Positionierung fundamental | Nie im eigenen Repository; allenfalls community-getragen bei nachgewiesener Nachfrage |
| Spektrale Hyperkanten-Indexierung | Kein validierter Nutzerbedarf, hoher Aufwand | Nach validiertem Bedarf in einem regulierten Vertical |
| EU-AI-Act-Risikoeinstufungs-Mapper | Durchsetzungszeitplan liegt bei 2027 | Wenn das Durchsetzungsdatum näher rückt |
| Horizontales Sharding über mehrere Knoten | Single-Node ist das Produkt; kein Kundenbedarf | Wenn mehrere Kunden mit sehr großen Vektormengen dies konkret benötigen |
| Peer-to-Peer-Synchronisation | Sehr hoher Aufwand, kein validierter Bedarf | Nach Validierung eines Mehrknoten-Anwendungsfalls |
| Mehrstufige Entscheidungskaskade als öffentliches Feature | Kognitiv zu komplex für die öffentliche API | Bleibt dauerhaft als opt-in Feature-Flag verfügbar, nie Standard |
| Vertikal-spezifische Ontologie-Schnittstelle | Abhängig von einem konkreten Vertical-Piloten | Nach dem ersten regulierten Vertical-Kunden |
| Vollständige Verarbeitungsverzeichnis-Quelle (DSGVO Art. 30) | Abhängig von einem behördlichen Piloten | Nach einem konkreten behördlichen Projekt |

---

# Schlussregeln {#schlussregeln}

Die folgenden drei Regeln gelten für jede Weiterentwicklung dieses Projekts, unabhängig vom Zeithorizont dieses Dokuments, und sind identisch mit den in Abschnitt 0.8 begründeten Governance-Regeln:

**Regel 1 — Kein neues Crate, keine neue Abhängigkeit** ohne schriftliche Begründung in diesem Dokument und eine Wartezeit von 48 Stunden zwischen Entscheidung und Umsetzung.

**Regel 2 — Externe Validierung vor interner Implementierung.** Vor dem Bau eines neuen, hier nicht spezifizierten Features: eine Frage an eine reale, projektexterne Person ("Würdest du dafür zahlen?"). Bleibt sie unbeantwortet: nicht bauen.

**Regel 3 — Ein Benchmark, eine Wahrheit.** Nie mehr als ein Satz öffentlich kommunizierter Benchmark-Zahlen gleichzeitig. Bei Widerspruch gilt ausschließlich der zuletzt reproduzierte Wert; ältere Werte werden explizit als veraltet gekennzeichnet, und die Ursache der Abweichung wird dokumentiert.

---

*Diese Spezifikation ist vollständig und in sich geschlossen. Sie bildet die alleinige, verbindliche Grundlage für jede weitere Entwicklungs-, Produkt- und Vertriebsentscheidung des Projekts Contextra.*

