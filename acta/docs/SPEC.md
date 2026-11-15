# Acta – Spezifikation

_Wissens- und Entscheidungsplattform mit versionierbarem Unternehmenswissen und entkoppeltem, nachweisbar löschbarem Personendatenschutz_
_Fassung 2.0 (final) · Stand 2026-11-15 · Status: normativ · eigenständig und vollständig: kein weiteres Dokument ist zur Umsetzung erforderlich_

---

## 0. Lesehinweise

### 0.1 Normative Schlüsselwörter

**MUSS**, **DARF NICHT**, **SOLL**, **KANN** haben die Bedeutung nach RFC 2119.

### 0.2 Kennzeichnung der Aussagen

|Marke|Bedeutung|
|---|---|
|**[E]**|Verbindliche Architekturentscheidung dieser Spezifikation|
|**[R]**|Rechtliche Anforderung oder Einordnung, aus der eine technische Eigenschaft folgt|
|**[Q]**|Messbares Qualitätsziel. Die Zahl ist ein Startwert; freigegeben wird nur nach gemessenem Bericht (16.5)|
|**[A]**|Annahme über die Umgebung; im Betrieb zu bestätigen und im Bericht auszuweisen (20.3)|
|**[⚠]**|Rechtlich offen oder umstritten. Die Spezifikation trifft eine vorsichtige technische Wahl und behauptet keine Rechtsfolge|
|**[V]**|Fundstelle oder Rechtstatsache ist vor jeder Außenverwendung am Primärtext zu verifizieren (20.7)|

### 0.3 Konventionen

- Alle Plattenformate und Protokollrahmen: Big-Endian, Längenfelder geprüft, Zeiten in UTC. `tx_time` in Millisekunden seit Unix-Epoche (u64), Kalendertage als `i32` Tage seit 1970-01-01. Größenangaben in KiB/MiB/GiB (Zweierpotenzen).
- Rust-Signaturen in dieser Spezifikation sind **normativ in Form und Semantik**, nicht in Nebensächlichkeiten wie Lebensdauer-Annotationen.
- `<…>` in Dateinamen und Domänenpräfixen sind Platzhalter. Alle Domänenpräfixe haben die Form `acta/<zweck>/v<N>` (Tabelle in 4.1).
- „Mandant“ ist die Isolationseinheit (Amt, Gesellschaft). „Installation“ ist ein Knoten mit einem oder mehreren Mandanten.

### 0.4 Vollständigkeits- und Vorrangregel [E]

1. **Default-deny:** Was diese Spezifikation nicht erlaubt, ist verboten. Es gibt keine stillen Standardwerte für Pflichtparameter (Anhang A) und keine Fallbacks im Sicherheitspfad (Z10).
2. **Vorrang bei scheinbarem Widerspruch:** Zusagen (1.5) vor Invarianten (15.4) vor Verfahren (Kap. 4 bis 14) vor Parametern (Anhang A). Wo ein Verfahren eine Zusage nicht erfüllen kann, ist das Verfahren falsch und die Zusage maßgeblich; der Fehler ist als Defekt zu behandeln, nicht zu „umgehen“.
3. **Jeder Lebenszyklus ist vollständig:** Kapitel 19 listet für jede Funktion Auslöser, Vorbedingungen, Schritte, Nachbedingungen, Fehlerpfade, Absturzverhalten und Nachweis. Eine Funktion ohne Eintrag dort gilt als nicht spezifiziert und DARF NICHT implementiert werden.

### 0.5 Aufbau

1 Produkt · 2 Grundmodell · 3 Architektur · 4 Kryptografie · 5 Speicher · 6 Wissensebene · 7 Entscheidungsprotokoll · 8 Löschkern · 9 Ingest, Abruf, Antwort · 10 Fachfunktionen · 11 Inferenz · 12 Mandanten, Rollen, Aufbewahrung · 13 Sicherheit · 14 Betrieb · 15 Qualität und Konfiguration · 16 Test und Freigabe · 17 Unabhängiger Prüfer · 18 Berichte und Schnittstellen · 19 Lebenszyklen · 20 Rechtliche Einordnung · Anhänge A bis F.

---

## 1. Produkt

### 1.1 Zweck

Acta ist eine eigenständige Software für **Behörden und mittelständische Unternehmen**. Sie bringt lokale Sprachmodelle an den Arbeitsplatz, speist sie mit belegtem Wissen und liefert Sachbearbeitung, Fachabteilungen, Verwaltung und Organisation Entscheidungshilfen und Werkzeuge. Fünf Eigenschaften machen Acta aus:

1. **Belegtes Antworten (RAG).** Jede inhaltliche Aussage trägt einen Quellenverweis auf einen tatsächlich abgerufenen Beleg. Fehlt der Beleg, enthält sich das System („nicht belegt“).
2. **Versionskontrolle für Wissen („Git für RAG“).** Der gesamte nicht personenbezogene Wissensbestand samt Richtlinien, Prompts, Chunking-, Abruf- und Modellkonfiguration, Schwellenwerten und Zugriffsrichtlinie ist unveränderlich versioniert, verzweigbar und zu jedem früheren Stand abfragbar. Alte Entscheidungen sind nachvollziehbar, „Wenn-Dann“-Szenarien laufen auf Verzweigungen des Wissens.
3. **Entkoppeltes, atomares und feingranulares Löschen personenbezogener Daten.** Personenbezogene Daten liegen getrennt vom versionierten Wissen, verschlüsselt, einzeln pro Dokument-Version und Betroffenem adressierbar. Löschung geschieht durch sofortige Lesesperre, physische Bereinigung aller Klartext-Strukturen und Ableitungen, Schlüsselvernichtung mit Rotation und Sicherungsfenster, jeweils gemessen. Die Versionsgeschichte des Wissens bleibt dabei unverändert. Acta stützt sich **nie allein** auf Schlüsselvernichtung (20.2).
4. **Rechtskonforme Datenversorgung des Modells.** Das Modell sieht ausschließlich Belege, auf die der Nutzer im konkreten Vorgang zugreifen darf. Personenbezogene Inhalte sind nur im ausdrücklichen Vorgangskontext abrufbar (Zweckbindung, Datenminimierung), nie im Freitext-Korpus.
5. **Entscheidungsnachvollziehbarkeit.** Zu jeder Antwort existiert ein Entscheidungsprotokoll (Wissensstand, Prompt, Modell, Parameter, Eingabe, Belege). Die Begründungskette überdauert die Löschung von Personen.

### 1.2 Einsatzrahmen [E]

|Merkmal|Festlegung|
|---|---|
|Betreiber|Behörden, Kommunen, Mittelstand; On-Premise oder vom Kunden kontrollierte EU-Umgebung|
|Mandanten|Mehrere Organisationseinheiten je Installation, strikt getrennt bis hinunter zu Schlüsseln, Ordinalräumen, Ledgern, Tresoren und Indizes (12.1)|
|Nutzer|Sachbearbeitung, Fachabteilungen, Datenschutzfunktion, Recht, Administration, Revision|
|Inferenz|Lokal oder in vom Kunden kontrollierter Infrastruktur als **separater Prozess**; keine Inferenz im Kernprozess (11); externe Dienste nur durch ausdrückliche Freigabe (13.3)|
|Plattform|Linux x86_64 und aarch64; **ein Knoten je Installation**; Wiederherstellung kalt (14.6)|
|Netz|Betrieb ohne ausgehende Verbindungen ist Standard und voll funktionsfähig|
|Rechtsrahmen|DSGVO; für öffentliche Stellen des Bundes das BDSG, der Länder zusätzlich Landesdatenschutz- und Archivrecht. Dessen Abweichungen (Löschung, Anbietungspflicht an Archive) sind im Löschkonzept des Mandanten abzubilden (12.4) [R]|
|Sprachen|Deutsch und Englisch voll unterstützt (Normalisierung, Stemming, Goldsets); weitere Sprachen über `retrieval_cfg` ohne Zusage zur Güte|

### 1.3 Mission und Umsetzung

|Säule|Umsetzung|Kapitel|
|---|---|---|
|Souveränes Unternehmenswissen|On-Premise, Schlüssel beim Kunden, kein Egress im Standard, lokale Inferenz, kein Herstellerzugriff|4, 11, 13, 14|
|Transparenz und Nachvollziehbarkeit|Versionierung, Entscheidungsprotokoll, Hash-Ketten je Mandant, unabhängiger Prüfer|6, 7, 17|
|Präzision und Kontrolle|Belegpflicht, Hybridsuche mit Reranking, Zugriffskontrolle vor der Suche, Messbarkeit|9, 16|
|Effizienz und Ressourcenschonung|Segmentierte Indizes, Datei-Drop statt Umschreiben, Batch-Vernichtung, kleine Binärdatei, Festkomma-Ranking|3, 5, 9|
|Schutz personenbezogener Daten|Ebene P, Schlüsselhierarchie, Lesesperre, versiegelte Ableitungen, Mehrbetroffenen-Prüfung|2, 4, 8|
|Verwaltung und Organisation|Mandanten, Sammlungen, Metadatenschema, Aufbewahrungsklassen, Löschkonzept, Vorgänge|12|
|Entscheidungs- und Verwaltungshilfe|Vorgangsassistent, Prüflisten, Entwürfe, Fristen, Änderungsradar, Szenarien, menschliche Freigabe|7, 10|
|Europäischer Datenschutz|Abläufe zu Art. 5, 12 bis 19, 25, 30, 32 DSGVO; Unterstützung für Art. 28, 33, 35; Berichte mit Annahmen|8, 18, 20|

### 1.4 Nicht-Ziele (Version 1) [E]

- Kein Dokumentenmanagementsystem für Originale und keine Langzeitarchivierung. Acta hält Kopien und Repräsentationen; das Original bleibt im Quellsystem (`source_of_record`, 9.1).
- Keine Mehrknoten-Replikation und kein Warm-Standby des Tresors (14.6).
- Keine autonomen Agenten und keine Werkzeugausführung durch das Sprachmodell. Deterministische Werkzeuge (Kapitel 10) werden von der Anwendung aufgerufen, nie vom Modell.
- **Keine automatisierte Einzelentscheidung** im Sinne des Art. 22 DSGVO. Antworten sind Entscheidungshilfe; die Entscheidung trifft ein Mensch (Entscheidungsart `Review`, 7.1).
- Keine grafische Oberfläche als Teil dieser Spezifikation; Referenzoberflächen sind eigene Projekte über die API.
- Keine Zertifizierungsaussage (1.6) und keine Rechtsfeststellung (20).
- Kein Zusammenführen zweier Betroffener nachträglich (8.2): jede Kennung wird einzeln gelöscht.

### 1.5 Normative Zusagen

Jede Zusage MUSS durch automatisierte Tests belegt sein; eine Zusage ohne Test gilt als nicht erfüllt (Gate G-ZUSAGE). **Deterministisch (D)**: der Test entscheidet vollständig. **Statistisch (S)**: ein Gütewert wird gemessen (nur Z17 in Teilen).

|ID|Zusage|Nachweis|
|---|---|---|
|**Z1**|**Sofortige Lesesperre (D).** Nachdem eine Löschung im Zustand `Gated` an den Aufrufer zurückgekehrt ist, liefert kein Standard-Lesepfad Klartext des Ziels: nicht Suche, Antwort, Zeitreise, Szenario, Cache, Export, Vorschau oder Sitzung, auch nicht ältere Versionen (`Superseded`). Schreibzugriffe auf das Ziel werden verweigert. Für `Restricted` gilt dasselbe mit der einzigen Ausnahme `restricted_read` (8.1). Für Takedown in Ebene K gilt die Sperre analog (`RedactedK`, 5.3).|`T-GATE-*`|
|**Z2**|**Kryptografische Isolation (D).** Jeder persistierte Klartext personenbezogener Daten und jede daraus abgeleitete Struktur liegt vor dem Schreiben in einem AEAD-Umschlag unter Schlüsseln, die nach der Vernichtung nicht mehr existieren. Persistierte Metadaten tragen **keine dauerhaften deterministischen Pseudonyme** einer Person (2.4) und keine exakten Längen (4.5).|`T-CRY-*`, `T-DERIVED-SEALED`, `T-META-*`|
|**Z3**|**Mehrbetroffenen-Behandlung (D).** Betrifft ein Dokument mehrere Betroffene, wird es im Standardmodus `DeleteAll` für alle unlesbar, sobald einer von ihnen gelöscht wird. Ein automatischer **Trockenlauf** innerhalb `erase_return_timeout` prüft die Mitbetroffenen auf Aufbewahrungspflicht, `legal_hold`, Einwand und offenen Vorgang. Konfliktdokumente sind **sofort** `Restricted(MultiSubjectReview)`; es gibt nie ein automatisches `DeleteAll` nach Fristablauf. Ein Review entscheidet zwischen `DeleteAll`, `RedactSplit` und `RestrictShared` (8.9). Anzahl und Modus der Mitbetroffenen stehen im Bericht, nie ihre Kennungen.|`T-MULTI-*`|
|**Z4**|**Crash-Invarianz (D).** Ein Absturz an jedem Durabilitätspunkt (Anhang C) führt nach dem Neustart entweder zur Vollendung der Löschung oder zu `Pending` bei dauerhaft aktiver Sperre. Eine durable Sperre geht nie verloren; ein Schlüssel geht nie ohne Absicht verloren. Physische Tatsachen (Schlüssel vernichtet, Segment ersetzt) haben Vorrang vor dem Journalstand.|`T-CRASH-*`|
|**Z5**|**Trennung von Wissen und Personendaten (D).** Die versionierte Wissensebene K enthält keine personenbezogenen Daten. Verweise auf Personendaten sind zufällige Handles ohne Inhaltsbezug. Ein Schreibversuch mit erkanntem Personenbezug scheitert (Fail-Closed); die Prüfung ist typerzwungen (`Validated<T>`, 7.5). Inhalte im Modus `Split` sind nur mit bestandenem Anonymitätstest, Validator-Prüfung und Vier-Augen-Freigabe committbar; ohne bestandenen Test bleibt das Dokument in `Whole` (2.3).|`T-SEP-*`, `T-ANON-*`|
|**Z6**|**Löschung in P berührt die Versionskontrolle nicht (D).** Eine Löschung in Ebene P ändert weder Objekt-IDs noch Commit-Hashes noch bestehende Einträge des K-Ledgers. Löschnachweise liegen im E-Ledger (6.2). (Takedown und Fristlöschung **in K** sind eigene Operationen mit Vermerk im K-Ledger, 6.7.)|`T-VCS-ERASE-*`|
|**Z7**|**Zeitreise-Determinismus (D).** Abfragen der Wissensebene zu einem Stand (Inhalte, Kandidatenmenge, Rangfolge der Textsuche und exakten Vektorsuche) liefern bei gleichen Parametern, gleichem Abfragevektor, gleicher Sichtklasse und gleicher Index-Generation byte-identische Ergebnisse, auch zwischen x86_64 und aarch64. Statistiken sind commit- und sichtklassenrelativ (9.2). Objekte mit Vermerk `Redacted` oder `Expired` liefern einen Platzhalter. Modellausgaben sind ausgenommen (A-INFER).|`T-TT-*`|
|**Z8**|**Entscheidungsnachvollziehbarkeit (D).** Zu jeder Antwort existiert ein Entscheidungsprotokoll mit Wissensstand, Prompt-Version, Modellstand, Parametern, Eingabe und Belegen. Nach Löschung einer Person bleibt die Begründungskette erhalten; personenbezogene Eingaben und Antworten liefern `Erased`.|`T-DEC-*`|
|**Z9**|**Szenario-Isolation (D).** Ein Szenario verändert weder die Hauptlinie des Wissens noch Ebene P noch Zugriffsrechte.|`T-SCEN-*`|
|**Z10**|**Fail-Closed-Kryptografie (D).** Kein Fehler im Schlüssel-, Zufalls-, Ableitungs- oder Siegelpfad führt zu Ersatzwert, Fallback-Schlüssel oder Klartext. Es gibt keine Standardschlüssel. Ein **fehlender** Schlüssel ohne Gate-Eintrag oder Registerbeleg ist ein Integritätsfehler (Vergiftung), nicht `Erased`.|`T-NOFALLBACK-*`, `T-RNG-*`|
|**Z11**|**Nachweis nur mit Positivkontrolle (D).** Ein Löschnachweis entsteht ausschließlich aus einer Messung, deren Messverfahren seine Funktionsfähigkeit an einem synthetischen Treffer bewiesen hat. Nicht implementierte Prüfungen liefern `NotImplemented`, nie `reached`. Stichproben werden als solche ausgewiesen.|`T-EVID-*`|
|**Z12**|**Restore-Schutz (D, bedingt).** Das Einspielen einer alten Sicherung kann gelöschte Daten nicht wiederbeleben, **sofern** das Löschregister unabhängig von der Sicherung verfügbar ist (Registersenke oder Ankerstand, 8.8) und `commit_key_retention` den Restore-Horizont abdeckt. Andernfalls startet der Knoten in `Quarantine` und liefert keine Daten.|`T-REST-*`|
|**Z13**|**Herkunftszwang (D).** Jede abgeleitete Struktur trägt ihre Herkunft (Lineage); Ableitungen ohne Herkunft sind nicht konstruierbar.|Compile-Fail-Tests|
|**Z14**|**Schlüsselisolation je Dokument-Version (D).** Zwei Dokument-Versionen der Ebene P teilen niemals denselben Datenschlüssel.|Property-Test|
|**Z15**|**Kennungsarme Protokolle (D).** Protokolle, Ledger, Metriken und Fehlertexte enthalten weder Schlüsselmaterial noch Roh-Kennungen von Personen noch dauerhafte deterministische Pseudonyme. Zulässig sind ausschließlich epochengeschlüsselte Commitments (6.2), die nach Ablauf von `commit_key_retention` anonymisiert werden, sowie zufällige Handles.|`T-LOG-*`|
|**Z16**|**Zugriffskontrolle vor der Suche (D).** Ein Nutzer erhält nie Treffer, Belege, Texte, Zählwerte, Quellennamen, Rangverschiebungen oder unterscheidbare Fehlerbilder zu Dokumenten, auf die er keinen Zugriff hat; die Prüfung wirkt auf die Kandidatenmenge und die Statistik, nicht erst auf die Ausgabe.|`T-ACL-*`|
|**Z17**|**Belegpflicht (D/S).** Jede inhaltliche Aussage einer Antwort trägt einen Verweis auf einen in dieser Anfrage abgerufenen Beleg, oder die Antwort besteht aus der ausdrücklichen Enthaltung „nicht belegt“ (strukturell, D). Die Güte der Stützungsprüfung wird gemessen (S, 9.6).|`T-GROUND-*`|
|**Z18**|**Kein Egress im Standard (D).** Ohne ausdrückliche, protokollierte, vier-Augen-freigegebene Freigabe verlässt kein Inhalt die Installation.|`T-EGRESS-*`|
|**Z19**|**Anbieter ohne Zugriff (D).** Die Software enthält keine Telemetrie, keinen Fernzugriff, keine Lizenzprüfung mit Nutzerdaten, keine Schlüsselhinterlegung (Escrow) und keinen Wiederherstellungsweg über den Hersteller. Supportpakete werden lokal erzeugt, enthalten nachweislich keine Inhalte und werden vom Betreiber freigegeben (14.7).|`T-VENDOR-*`|

### 1.6 Sprachregeln und Berichtsgrenzen [R]

**Verbotene Formulierungen** in Berichten, Oberflächen, Dokumentation, Werbematerial, Quelltext und dieser Spezifikation (Liste A, exakte Wortformen, Groß-/Kleinschreibung ignoriert): „DSGVO-konform“, „DSGVO-zertifiziert“, „erfüllt Art. 17 automatisch“, „100 % gelöscht“, „garantiert gelöscht“, „garantiert vernichtet“, „unwiederbringlich“, „vergisst Daten“, „revisionssicher“, „fehlerfrei“, „löscht aus allen Backups“, „perfekte Antworten“, sowie jede Behauptung einer Zertifizierung oder behördlichen Billigung.
**Regel B** (nur für Berichtsvorlagen, Oberflächentexte, Exportvorlagen): Das Wort „gelöscht“ steht nur zusammen mit der erreichten Erfüllungsklasse („Klasse K erreicht“).

Konformität ist ein Betriebszustand der Organisation, keine Eigenschaft einer Software. Acta berichtet ausschließlich **messbare Erfüllungsklassen** (8.7) mit `reached` oder `not_reached` und weist die **Annahmen** A-FLASH, A-RAM, A-ANCHOR, A-CRYPTO, A-EXT, A-INFER, A-CLOCK, A-DETECT, A-ARCH, A-FS (20.3) aus. Das Gate G-WORD prüft Liste A über Quelltext, Texte, Vorlagen und diese Spezifikation; Regel B über Vorlagen und Oberflächentexte. Der Abschnitt 1.6 selbst ist durch `g-word:ignore` ausgenommen. Rechtsgrundlage: § 5 UWG (Irreführung), Sachmangelrecht (§ 434 BGB) [R][V].

### 1.7 Begriffe

|Begriff|Bedeutung|
|---|---|
|Ebene K / P / D / E|Wissen (versioniert) / Personendaten (löschbar) / Ableitungen (Indizes, Caches, Embeddings) / Nachweise (Ledger, Gate-Log, Berichte, Tresore)|
|Betroffener (`Subject`)|Natürliche Person, deren Daten verarbeitet werden, einschließlich Mitarbeitenden (Nutzer sind Betroffene)|
|Version (`Ver`)|Unveränderlicher Stand eines P-Dokuments; kleinste Einheit für Schlüssel, Gate und Ordinal (`VerRef`, `VerOrd`)|
|Vorgang (`Case`)|Fachliche Klammer (Akte) in P, die Dokumente und Zugriffsrechte bündelt; Voraussetzung für Abruf von P-Inhalten (9.4)|
|Gate|O(1)-Sperrstruktur, die jeder Lesepfad vor jeder Entschlüsselung befragt (5.3)|
|Saga|Dauerhafter, idempotenter Löschablauf S0 bis S9 (8.3)|
|Klassen G, B, K, M|Erfüllungsklassen: gesperrt, bereinigt, Schlüssel vernichtet, vollständig gemessen (8.7)|
|Handle|Zufälliger 128-Bit-Verweis ohne Inhaltsbezug (`PersonalRef`, `ActorRef`)|
|Ordinal|Dichte interne Nummer (`SubjOrd`, `VerOrd`, `KOrd`), nie wiederverwendet; nur intern sichtbar|
|Lineage / `source_set`|Menge der Ordinale, aus denen eine Ableitung entstanden ist (5.2)|
|Sichtklasse|Menge von Nutzern mit identischer effektiver Zugriffsmenge; teilt Statistiken und Caches (9.2)|
|Anker (`KeyAnchor`)|HSM, KMS oder Datei-Anker, der Generationsgeheimnisse hält und vernichtet (4.4)|
|Senke (`RegistrySink`)|Unabhängiger Ausfallbereich, in den Gate-Log und E-Ledger gespiegelt werden (6.2)|
|Canary|Synthetischer Betroffener zur Positivkontrolle von Messungen (8.5)|
|Index-Generation (`IndexGen`)|Tripel aus Chunking-Konfiguration, Embedder-Stand und Normalisierungsvariante; Grundlage eines Indexsatzes (9.2, 11.4)|
---

## 2. Grundmodell

### 2.1 Vier Ebenen

```
┌───────────────────────────────────────────────────────────────────────────┐
│ Ebene K – Wissen (versioniert, unveränderlich, ohne Personenbezug)        │
│   Blobs, Trees, Commits, Branches, Tags; Entscheidungsprotokolle (Beleg)  │
│   Verweise auf Personendaten nur als zufällige PersonalRef-Handles        │
├───────────────────────────────────────────────────────────────────────────┤
│ Ebene P – Personendaten (verschlüsselt, löschbar, nicht Teil der Hashes)  │
│   Dokument-Versionen unter Versions-, Dokument- und Betroffenenschlüsseln,│
│   Betroffenenverzeichnis, Handle-Register, Vorgänge                       │
├───────────────────────────────────────────────────────────────────────────┤
│ Ebene D – Ableitungen (Indizes, Caches, Embeddings, Sitzungen)            │
│   Tragen Herkunft (Lineage). DP: aus P, versiegelt, löschbar, je Cluster. │
│   DK: aus K, versiegelt, global je Mandant, ohne Löschlogik               │
├───────────────────────────────────────────────────────────────────────────┤
│ Ebene E – Nachweise (E-Ledger, Gate-Log, Löschregister, Berichte,         │
│   Tresore, Anker-Anbindung, Registersenke)                                │
└───────────────────────────────────────────────────────────────────────────┘
```

**Kerngedanke [E]:** Versionskontrolle und Löschung sind **konstruktiv entkoppelt**. Die Versionsgeschichte (K) enthält nie Inhalte, die auf Personenanfrage gelöscht werden müssen. Sie enthält an deren Stelle ein **PersonalRef**, ein zufälliges 128-Bit-Handle ohne Ableitung aus dem Inhalt. Der Hash eines Trees oder Commits wird über das Handle gebildet, nie über Personendaten. Die Löschung entfernt die Verschlüsselung der Daten in Ebene P und die Ableitungen; das Handle bleibt gültig und löst nach der Löschung zu `Erased` auf.

**Restrisiko des Handles [A] (R-HANDLE):** Ein Handle verbindet K-Stellen und Entscheidungen zu einer Akte oder Person, auch nach der Löschung (Muster aus Zeitpunkt und Häufigkeit). Es ist ohne das entfernte Handle-Register nicht auf eine Person abbildbar. Die Restwirkung wird im Bericht ausgewiesen.

### 2.2 Datenklassen (Regel R-CLS) [E]

Jeder Schreibvorgang MUSS eine Datenklasse deklarieren. Fehlt sie, bricht die Operation mit `ClassViolation` ab.

|Klasse|Ebene|Beschreibung|Aufbewahrung|Löschmechanismus|
|---|---|---|---|---|
|`Wissen`|K|Sachinhalte ohne Personenbezug: Vorschriften, Richtlinien, Handbücher, Vorlagen, Konfigurationen, Prompts|unbegrenzt (versioniert)|Takedown (6.7)|
|`Beleg`|K|Entscheidungsprotokolle ohne Personenbezug|`decision_retention` je Entscheidungsart, **Pflichtparameter ohne Standardwert**|Frist- und Takedown-Löschung (6.7)|
|`Personal`|P|Personenbezogene Daten|nach Aufbewahrungsklasse (12.2)|Saga (8.3)|
|`Besonders`|P|Daten nach Art. 9 und 10 DSGVO|Karenz nach Fristablauf: 0 Tage|Saga mit vorgezogener Schlüsselvernichtung und Rotation auf Anforderung|
|`Abgeleitet`|D|Indizes, Caches, Embeddings, Sitzungen|keine eigene Frist|herkunftsbasierte Bereinigung, Rebuild|
|`Unklassifiziert`|P|Eingang ohne Entscheidung oder ohne Betroffenenzuordnung|Zustand `Quarantine` höchstens `quarantine_max` (24 h), danach `Restricted(Precautionary)`, nach `quarantine_purge_after` (7 Tage) automatisch Saga `Expired`|wie `Personal`|

### 2.3 Dokumente mit gemischtem Inhalt [E]

Der Ingest wählt je Dokument genau einen Modus:

|Modus|Wirkung|Wann|
|---|---|---|
|`Whole` (Standard bei erkanntem oder möglichem Personenbezug)|Das gesamte Dokument liegt in Ebene P. In K entsteht höchstens ein PersonalRef|Akten, Fallbearbeitung, Schreiben an Personen|
|`Split`|Personenbezogene Spannen werden entfernt und in P abgelegt; K enthält den bereinigten Text mit Platzhaltern `⟦P:<handle>⟧`. **Pflicht vor dem Commit:** bestandener N-Validator (7.5), **bestandener Anonymitätstest** (Kontext erlaubt keine Identifizierung; Verfahren und Ergebnis dokumentiert), **Vier-Augen-Freigabe**|nur bei ausdrücklicher Richtlinie `split_allowed` des Mandanten|
|`Open`|Das Dokument ist vollständig `Wissen`|Dokument besitzt bestätigt keinen Personenbezug (Vorschrift, Handbuch); Vier-Augen-Freigabe|

**Entscheidung:** Ein `Split`-Dokument ohne bestandenen Anonymitätstest ist **nicht committbar**; es bleibt in `Whole`. Es gibt keinen Zwischenzustand „pseudonymisiert in K“. `Split`-Blobs tragen `origin=Split`, werden alle `split_retest_interval` (12 Monate) neu getestet und im Takedown vorrangig behandelt. Der Bericht spricht von „unter dokumentierter Methode als anonym bewertet, Restrisiko [⚠]“. Besteht ein Wiederholungstest nicht, löst das Takedown aus (6.7). Maßstab ist der relative Personenbezug (20.1).

**Betroffenenzuordnung [E].** Für jede Klasse `Personal` und `Besonders` MUSS der Ingest die **Betroffenen des Dokuments** kennen: Liste externer Kennungen aus dem Fachverfahren (z. B. Aktenzeichen → Person), geliefert von der integrierenden Schnittstelle oder durch Handentscheidung. `acta-pii` liefert nur Hinweise, nie die Zuordnung. Fehlt die Zuordnung, ist das Dokument `Unklassifiziert`. Eine nachträglich erkannte weitere Person löst `AddSubject` aus (neue Version, neuer Schlüssel, 4.2); bis dahin gilt Z3 für sie nicht und der Bericht weist das aus. Die Datenschutzfunktion kann über `subjects.discover` (18.5) Kandidaten für `AddSubject` finden.

### 2.4 Identifikatoren [E]

|Typ|Größe|Bildung|Sichtbarkeit|
|---|---|---|---|
|`TenantId`|16 B|zufällig bei Anlage|Mandant; nicht geheim; Teil der AAD und Dateinamen|
|`TenantOrd`|u32|dicht, nie wiederverwendet; 0 ist reserviert für Systeminterna|intern|
|`SubjectTag`|16 B|`HMAC-SHA256(K_tag(t), "acta/tag/v1" ‖ kind u8 ‖ NFC(ext_id))[..16]`, `K_tag(t) = HKDF(k_idx, salt = TenantId, info = "acta/tag-key/v1")`; `ext_id` trägt einen Namensraum (z. B. `az:`, `pnr:`, `oidc:<iss>\|<sub>`)|**nur** als versiegelte Tresor-ID (4.3) und im Betroffenenverzeichnis. Nie im Klartext in Segmenten, Journalen, Ledgern, Berichten, Dokumentköpfen|
|`SubjOrd`|u32|dicht je Mandant, nie wiederverwendet|Gate, Sperrmengen, intern|
|`ClusterId`|16 B|`HKDF(SubjectKey, "acta/cluster/v1")[..16]`; **nicht gespeichert**, nur mit `SubjectKey` berechenbar. Bei Dokumenten ohne Betroffenen: `DocRef`|Lokalität in Segmenten (5.1)|
|`DocRef`|16 B|zufällig|stabile Identität eines P-Dokuments über Versionen|
|`VerRef`|16 B|zufällig je Version|Tresor-ID des `DocumentKey` einer Version; Salz der Ableitungen|
|`VerOrd`|u32|dicht je Mandant, nie wiederverwendet; Überlauf → `Poisoned(Node)`|Lineage, Gate, intern|
|`PersonalRef`|16 B|zufällig, getrennt von `DocRef`|Verweis aus K; Auflösung nur über das Handle-Register|
|`CaseId`|16 B|zufällig|Vorgang (12.3)|
|`ActorRef`|16 B|zufällig je Person, Mandant und Epoche (`actor_handle_epoch`)|Handle auf Identität in P|
|`KOrd`|u64|dicht, monoton **je Mandant**, nie wiederverwendet|Sichtbarkeitsmengen, DK-Lineage, intern|
|`ObjectId`|32 B|SHA-256 über kanonische Bytes mit Domänenpräfix (6.1)|Objekte der Ebene K|
|`ErasureId`|16 B|UUIDv7|Löschvorgang|
|`IndexGen`|16 B|`SHA-256(chunk_cfg ‖ embedder_fingerprint ‖ norm_variant)[..16]`|Index-Generation (9.2)|

`k_idx` ist ein 256-Bit-Geheimnis **ohne Standardwert**, geliefert vom Schlüsselanker (4.4). Ein Start ohne `k_idx` wird verweigert. **Verlust von `k_idx` macht Betroffene nicht mehr auffindbar** (Auskunft, Löschung); `k_idx` ist daher Teil der Ankersicherung (14.5). `k_idx` wird in Version 1 nicht rotiert; wer `k_idx` und eine externe Kennung besitzt, kann prüfen, ob eine Person erfasst war (Restrisiko ADR-15, ausgewiesen).

**Ordinal-Hochwassermarken.** Nach einer Wiederherstellung werden alle Ordinalzähler (`SubjOrd`, `VerOrd`, `KOrd`, `TenantOrd`) auf `max(Stand der Sicherung, Hochwassermarke der Registersenke) + 1` gesetzt (8.8), damit nie ein bereits gesperrtes Ordinal erneut vergeben wird.

### 2.5 Bedrohungsmodell [E]

|ID|Angreifer oder Ereignis|Zusicherung von Acta|Außerhalb der Zusage|
|---|---|---|---|
|T1|Netzangreifer ohne Konto|Authentifizierung, Ratenbegrenzung, kein Egress|Schwachstellen der TLS-Umgebung des Betreibers|
|T2|Nutzer mit Konto|Zugriffskontrolle vor der Suche (Z16), Mandantentrennung, Vorgangsbindung von P|Weitergabe ausgegebener Texte durch den Nutzer (A-EXT)|
|T3|Administrator **ohne** Ankerzugriff|kein Klartext in P über die API; Dateien nur als Chiffrat|Metadaten (Größenklassen, Zeitpunkte, Ordinale), `ClusterId`-Gruppierung solange `SubjectKey` lebt (R-CLUSTER)|
|T4|Besitzer eines Datenträgerabbilds oder einer Sicherung|alles Persistierte ist AEAD-versiegelt; Tresorabbilder werden mit der Rotation unlesbar; Exporte mit Ablauf der Sicherungsepoche|Abbild **zusammen mit** Anker (T5)|
|T5|Besitzer eines alten Abbilds **und** späterer Ankerzugriff|vernichtete Schlüssel sind nach Rotation der Generation und Ablauf der betroffenen Sicherungsepochen unlesbar (4.3)|Schlüssel, die vor Vernichtung und Rotation kopiert wurden|
|T6|Bösartiges Dokument (Parser-Angriff)|Parser laufen in einem Prozess ohne Schlüssel, Netz und Dateisystem (13.2)|Fehler im Betriebssystemkern|
|T7|Bösartiger Inhalt (Prompt-Injection)|Belege sind Daten; Ausgabeprüfung; keine Werkzeuge; keine aktiven Inhalte (9.4)|Fehlverhalten des Modells (A-INFER)|
|T8|Administrator **mit** Root auf dem Host|Erschwerung (`mlock`, kein Core-Dump, kein `ptrace`), **keine Zusicherung**|Root-Zugriff auf Prozessspeicher (A-RAM); Gegenmaßnahme organisatorisch oder durch Vertrauensumgebungen|
|T9|Langfristige Kryptoschwäche|Algorithmus-Byte, Neubewertung, Migration (4.7)|Bruch vor Migration (A-CRYPTO)|
|T10|Inferenzdienst|Gehört zum Vertrauensbereich (A-INFER); Pflichtkonfiguration „keine Protokollierung von Inhalten, kein persistenter Prompt-Cache“; `ContextPurge` (9.5); sonst als extern behandelt (13.3)|Verhalten des Dienstes|

---

## 3. Systemarchitektur

### 3.1 Prinzipien [E]

1. **Anforderung vor Mechanismus.** Kein Bauteil ohne Zusage oder Produktanforderung.
2. **Eigenbau nur dort, wo physische Kontrolle erforderlich ist** (Segmentspeicher, Schlüsseltresor, Löschkern, Versionskontrolle). Sonst bewährte Bibliotheken aus der Whitelist (3.8).
3. **Fehler unmöglich machen statt verbieten:** Typzwang (`Sealed<P>`, `Derived<T>`, `Validated<T>`, Schlüsseltypen ohne `Default`, `Clone`, `Debug`, `Serialize`).
4. **Eine Lösung je Problem:** ein Hash-Ketten-Typ (mehrere Instanzen), ein Zeit- und Zufallsport, ein Fehlerklassifikator.
5. **Kleine Crates, harte Größenbudgets, keine Sammel-Crates.**
6. **`unsafe` nur in `acta-sys`.** Abhängigkeiten mit `unsafe` sind erlaubt und werden mit `cargo geiger` im Bericht geführt.
7. **Kern synchron.** `tokio` nur an den Rändern (Level 8 und höher), nie in Level 0 bis 7.
8. **Nicht vertrauenswürdige Eingaben laufen nie im Prozess der Schlüssel** (13.2).
9. **Ein Schreiber je Mandant, Leser lock-frei gegen unveränderliche Schnappschüsse** (3.6).

### 3.2 Level-Modell [E]

Eine Abhängigkeit `A → B` ist erlaubt genau dann, wenn `level(A) > level(B)`. Die Level stehen in `rings.toml`; Gate G1 prüft sie gegen `cargo metadata`. Querkanten auf gleichem Level sind verboten.

### 3.3 Crate-Verteilung [E]

Größenbudgets sind harte Obergrenzen für Produktionscode (ohne Tests). Überschreitung erfordert einen ADR-Eintrag.

|Level|Crate|Aufgabe|Budget (LOC)|Interne Abhängigkeiten|
|---|---|---|---|---|
|L0|`acta-base`|Newtypes (Ids, Ordinale), `DataClass`, `ErrorClass`, `ClassifiedError`, Ports `Clock`, `CryptoRng` (fallibel), kanonische Kodierung (geprüfte Big-Endian-Felder), Festkomma-Arithmetik `fx` (Q32.32, `fx_ln`, `isqrt`), Grenzwerte|2 500|–|
|L1|`acta-crypto`|AEAD (XChaCha20-Poly1305), HKDF, HMAC, Ed25519, `Sealed<P>`, Schlüsseltypen (`Zeroizing`), `HashChain`, Auffüllung (Padding)|4 200|base|
|L1|`acta-fs`|`Vfs`-Port, `DurableVfs` (fsync, atomares Ersetzen, Scrub, `O_DIRECT`-Rücklesen), `FaultVfs` (Fehlerinjektion)|2 500|base|
|L1|`acta-sys`|Einziges eigenes `unsafe`: `mlock`, `MADV_DONTDUMP`, `RLIMIT_CORE`, `PR_SET_DUMPABLE`, `SecureBuf`; Sandbox (Landlock, seccomp); Hostprüfungen; `SystemClock`, `OsCryptoRng`|1 600|base|
|L1|`acta-gate`|`ErasedView`, `GateWriter`, Roaring-Mengen, Epoche, Traits `ErasureLayer`, `LayerAck`|1 200|base|
|L1|`acta-access`|Mandanten, Rollen, Attribute (ABAC), Richtlinienauswertung, Sichtklassen, Zugriffsmengen|3 000|base|
|L1|`acta-infer`|Verträge `Embedder`, `Reranker`, `Generator`, `Verifier`, `ModelFingerprint`, `ContextPurge`, Kostenbeschreibung|700|base|
|L1|`acta-pii`|Erkennung personenbezogener Muster (Regeln, Wörterbücher, Prüfsummen), N-Validator-Bausteine, `Validated<T>`|4 000|base|
|L1|`acta-ingest`|Chunking, Normalisierung, Ressourcenlimits, Worker-Protokoll (Rahmen, Client und Server-Seite), **ohne** Parser-Abhängigkeiten|3 000|base|
|L1|`acta-parse`|Parser für PDF, Office, HTML, Text, Archive (nicht vertrauenswürdige Eingaben); **einziger** Crate mit Parser-Abhängigkeiten|5 000|base|
|L2|`acta-ledger`|Append-only-Protokolle mit Hash-Kette (K-, E-, A-Ledger), signierte Checkpoints, keyed Commitments, Trait `RegistrySink`, `DirSink`|2 800|base, crypto, fs|
|L2|`acta-vault`|Mandantentresore, `KeyAnchor`-Trait, `FileAnchor`, Wurzelrotation, Sicherungsexport, Vernichtung, Schlüsselableitungen|5 500|base, crypto, fs|
|L2|`acta-seg`|Versiegelter, **schlüsselfreier** Segmentspeicher: Journal, Segmente, Manifest, Aufräumen, Kopienregister. Schlüssel werden als Parameter übergeben|6 500|base, crypto, fs, gate|
|L3|`acta-pstore`|P-Speicher: Dokumentkopf, Versionen, Betroffenenverzeichnis, Handle-Register, Vorgänge, Fallindex (DP), Auflösung von Schlüsseln und DEK|5 000|base, crypto, fs, gate, ledger, vault, seg|
|L3|`acta-history`|Versionskontrolle K: Objekte, Refs, Sichtbarkeitsmengen, Diff, Merge, Takedown|6 500|base, crypto, pii, ledger, vault, seg, gate|
|L3|`acta-text`|Segmentierter Volltextindex (BM25 Festkomma, sichtklassenrelative Statistik) mit Lineage, DE/EN-Normalisierung|5 000|base, crypto, gate, vault, seg|
|L3|`acta-vec`|Segmentierter Vektorindex (exakt, `i8`), Lineage|4 000|base, crypto, gate, vault, seg|
|L4|`acta-erasure`|Löschkern: Gate-Journal, Zustandsmaschine, Messung, Bericht-Rohdaten, Restore-Schutz, Mehrbetroffenen-Plan|9 000|base, crypto, fs, gate, ledger, vault, seg, pstore|
|L4|`acta-decision`|Entscheidungsprotokoll, Szenarien, Vergleich, Akteure|4 500|base, crypto, ledger, vault, seg, history, pstore, gate, access, infer|
|L4|`acta-rag`|Abrufpipeline: Hybridsuche, Fusion, Reranking, Kontextaufbau, Belegprüfung, Bewertungswerkzeug; implementiert `ErasureLayer` für Antwort- und Sitzungs-Caches|7 000|base, crypto, ledger, text, vec, history, pstore, gate, access, infer|
|L5|`acta-reports`|Löschbericht (JCS), Verzeichnis nach Art. 30, DSFA-Entwurf, Fristen-/Aufbewahrungs-/Krypto-/Betriebsberichte, Wortfilter|3 500|base, crypto, erasure, ledger, history|
|L6|`acta-engine`|Anwendungsfälle: Ingest, Antwort, Fachfunktionen, Löschung, Szenario, Aufbewahrung, Wartung|12 000|alle darunter|
|L7|`acta-node`|Composition Root der Kernports: Konfiguration, Verdrahtung, Selbsttest, Hintergrundaufgaben, Thread-Pools|3 000|engine und alle Kernadapter|
|L8|`acta-infer-http`|Client für lokale und freigegebene Inferenzdienste (OpenAI-kompatible Schnittstelle); `tokio` erlaubt|3 000|base, infer|
|L8|`acta-sink-net`|Netz-Implementierung von `RegistrySink`; `tokio` erlaubt|600|base, ledger|
|L8|`acta-api`|HTTP/JSON-Schnittstelle, OIDC, mTLS, Streaming; `tokio` erlaubt|6 000|node|
|L9|`acta-server` (Binary)|Prozessstart, Signalbehandlung; injiziert Kantenports (Inferenz, Netzsenke, Notifier, ArchiveHook)|700|api, node, infer-http, sink-net|
|L9|`acta-cli` (Binary)|Administration: Anfragen, Szenarien, Berichte, Sicherung, Reparatur, Prüfung, Support-Paket|3 000|node, infer-http, sink-net|
|L9|`acta-parse-worker` (Binary)|Sandbox-Prozess für Parser; keine Schlüssel, kein Netz|600|ingest, parse, sys|
|–|`acta-verify` (Binary)|Unabhängige Prüfung von Berichten, Ledgern und (V2) Tresoreinträgen; **keine** interne Abhängigkeit|1 200|–|
|dev|`acta-testkit`|`ManualClock`, `SeededRng`, Fehlerinjektion, Konformitätstests, Testdaten|5 000|base, crypto, fs|
|dev|`xtask`|`cargo xtask gates`: Architektur- und Hygieneprüfungen|3 000|–|

Zusammen sind es **30 Produktions- und 2 Entwicklungs-Crates** (ca. 117 100 Zeilen Produktionscode); Binaries: `acta-server`, `acta-cli`, `acta-parse-worker`, `acta-verify`. Der Löschkern erreicht Segmente, Indizes und Caches **ausschließlich** über `ErasureLayer` aus `acta-gate`.

### 3.4 Hub-Regeln [E]

Es gibt **keinen** Sammel-Crate für „Typen“ oder „Ports“. Gemeinsame Verträge sind kleine, einem Zweck gewidmete Crates (`acta-base`, `acta-gate`, `acta-infer`).

|Regel|Inhalt|
|---|---|
|HR-1|Ein Vertrags-Crate hat genau einen Zweck und bleibt unter seinem Budget|
|HR-2|Ein Trait gehört dem Crate, das ihn **braucht**, wenn der Erbringer in einem höheren Level liegt; liegt der Erbringer tiefer, besitzt der Erbringer ihn|
|HR-3|Nutzen Erbringer und Nutzer einen Vertrag aus verschiedenen Richtungen, entsteht ein eigenes kleines Vertrags-Crate auf dem niedrigsten tragfähigen Level (`acta-gate` für `ErasureLayer`)|
|HR-4|Ein Fehlertyp je Crate (`thiserror`, `#[non_exhaustive]`, kein `Other(String)`); Aggregation erst ab L6|
|HR-5|`default = []`; Features sind additiv und haben keine Rückkante nach oben|
|HR-6|Keine Alias-Re-Exports, kein `include!`, kein `#[deprecated]`, keine Brücken-Module|
|HR-7|`acta-node` ist die einzige Stelle, die konkrete **Kern**implementierungen verdrahtet; Kantenports (Inferenz, Netzsenke, Notifier, ArchiveHook) injiziert das Binary über den Builder. Jeder Port hat im Betrieb genau eine Implementierung|
|HR-8|Fan-in-Limit: Von `acta-engine` hängt höchstens `acta-node` ab|
|HR-9|`acta-seg` kennt keine Schlüsselverwaltung: Schlüssel gelangen als Parameter (`&SegmentKey`) hinein. Schlüsselauflösung liegt in `acta-vault`/`acta-pstore`|

### 3.5 Fehlerarchitektur [E]

```rust
#[non_exhaustive]
pub enum ErrorClass { Transient, Integrity, Crypto, Erased, Policy, Invalid }

pub trait ClassifiedError: std::error::Error {
    fn class(&self) -> ErrorClass;
    fn code(&self) -> &'static str; // stabil, ohne Nutzdaten und Kennungen
}
```

`Integrity` und `Crypto` lösen Vergiftung aus (15.2). `Display` und `code()` DÜRFEN keine Personendaten oder Roh-Kennungen enthalten (Z15). `Erased` entsteht **nur** aus dem Gate oder einem Registerbeleg, nie aus der bloßen Abwesenheit eines Schlüssels (Z10).

### 3.6 Laufzeit- und Nebenläufigkeitsmodell [E]

**Synchroner Kern.** Level 0 bis 7 sind synchron und nutzen Threads. **Begründung:** Durabilitätspunkte lassen sich in synchronem Code deterministisch durch Fehlerinjektion prüfen; es entfällt der Nachweis der Abbruchsicherheit an jedem `fsync`; Modelltests (`loom`) sind direkt anwendbar. Netzwerk und Inferenzclients laufen asynchron (L8) und rufen den Kern über begrenzte Thread-Pools. **Revisionstrigger:** Messung zeigt, dass `erase_return_timeout` unter Last verletzt wird.

**Threads.**

|Pool|Größe|Aufgabe|
|---|---|---|
|`writer(t)`|1 je Mandant|alle schreibenden Anwendungsfälle und Saga-Schritte des Mandanten, seriell|
|`read`|Kernzahl|Anfragen, Suche, Entschlüsselung; nie blockierend auf `writer`|
|`rewrite`|`max(2, Kernzahl/4)`|Segmentersatz, Rewrap, Reindex, GC (niedrige Priorität)|
|`maint`|2|Rotation, Export, Sicherung, Epochenverwaltung, Fristenjob|
|`bridge`|Konfiguration (`bridge_threads`, Start 2 × Kernzahl)|Aufrufe aus L8; begrenzte Warteschlange `bridge_queue_max`; bei Überlauf `Transient(Overloaded)`|

**Schnappschüsse.** Das Manifest, die Gate-Mengen und die Sichtbarkeitsmengen sind unveränderliche Strukturen hinter `Arc`; Schreiber tauschen den Zeiger atomar, Leser laden einmal je Anfrage und arbeiten auf diesem Stand (Epoche-basierte Freigabe). Eine Datei wird physisch erst entfernt, wenn kein Leser-Schnappschuss mehr auf sie zeigt, spätestens nach `reader_drain_max` (30 s); verbliebene Leser werden dann mit `Erased` bzw. `Transient(Retry)` abgebrochen. Die Gate-Prüfung bleibt davon unberührt und wirkt sofort (Z1).

**Sperrreihenfolge (Lock-Order), strikt von oben nach unten:**
1. `rotation_lock` (global, schließt Mandantenanlage und Rotation gegeneinander aus)
2. `writer(t)` (Mandant)
3. `vault_freeze(t)` (`RwLock`: Schlüsseloperationen halten Lesesperre, Freeze-Phase der Rotation die Schreibsperre)
4. Manifest-Zeigertausch (kurz, ohne E/A unter der Sperre)
5. Gate-Zeigertausch (kurz)
6. Ledger-Anhängen (eigener Mutex je Kette)

Keine Sperre wird über einen `fsync` gehalten, außer `writer(t)`. Rückwärtsnahme ist verboten; `loom`-Tests prüfen die Ordnung.

**Gruppen-Commit.** Journal-, Gate-Log- und Tresor-Anfügungen werden innerhalb eines Fensters von `group_commit_window` (2 ms) gebündelt und mit **einem** `fsync` bestätigt; jeder Aufrufer erhält seine Bestätigung erst nach dem `fsync`.

**Speicherbudget.** Alle Puffer in Anfragepfaden sind begrenzt (Anfragegröße, Kandidatenmenge, Kontextgröße); es gibt keine unbeschränkte Allokation aus Eingaben. Schlüsselmaterial liegt nur in `SecureBuf`; der DEK-Cache (`dek_cache_max` = 4 096 Einträge, LRU) ist selbst Teil des `ErasureLayer`-Satzes und wird bei Sperre und Vernichtung der betroffenen Einträge geleert. Entschlüsselte Chunk-Texte im Chunk-Cache tragen Lineage, werden beim Verdrängen genullt (`Zeroizing`) und bei Gate-Treffer verworfen.

### 3.7 Normative Port-Signaturen (Kernverträge) [E]

```rust
// acta-base
pub trait Clock: Send + Sync { fn now_ms(&self) -> u64; }
pub trait CryptoRng: Send + Sync { fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError>; }

// acta-gate
pub enum GateState { Live, Restricted(RestrictReason), Superseded, Erased }
pub trait ErasedView: Send + Sync {            // nur lesend; von allen Lesepfaden genutzt
    fn check_ver(&self, v: VerOrd) -> GateState;           // O(1), keine E/A
    fn is_redacted_k(&self, k: KOrd) -> bool;              // O(1)
    fn epoch(&self) -> u64;
    fn lineage_hit(&self, l: &Lineage) -> LineageHit;      // Live | Hit{restricted|erased|superseded}
}
pub struct TargetSet { pub subjects: RoaringBitmap, pub vers: RoaringBitmap, pub kords: RoaringTreemap }
pub trait ErasureLayer: Send + Sync {          // implementiert von seg, pstore, text, vec, rag, Schlüssel-Cache
    fn name(&self) -> &'static str;
    fn plan(&self, t: TenantOrd, target: &TargetSet) -> Result<LayerPlan, LayerError>;
    fn purge(&self, t: TenantOrd, plan: &LayerPlan) -> Result<LayerAck, LayerError>;   // idempotent
    fn residue(&self, t: TenantOrd, target: &TargetSet) -> Result<Vec<Residue>, LayerError>; // Messung
}

// acta-fs
pub trait Vfs: Send + Sync {
    fn create_new(&self, p: &Path) -> io::Result<Box<dyn VFile>>;
    fn open_read(&self, p: &Path) -> io::Result<Box<dyn VFile>>;
    fn open_direct_read(&self, p: &Path) -> io::Result<Box<dyn VFile>>;  // O_DIRECT, ausgerichtet auf 4096
    fn rename_atomic(&self, from: &Path, to: &Path) -> io::Result<()>;   // rename + fsync(Verzeichnis)
    fn remove_scrub(&self, p: &Path) -> io::Result<()>;                  // Nullen überschreiben, fsync, unlink
    fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>>;
}
pub trait VFile: Send + Sync {
    fn write_all_at(&self, off: u64, buf: &[u8]) -> io::Result<()>;
    fn read_exact_at(&self, off: u64, buf: &mut [u8]) -> io::Result<()>;
    fn sync_data(&self) -> Result<(), FsyncFailed>;   // FsyncFailed ist endgültig für diese Datei
    fn len(&self) -> io::Result<u64>;
}

// acta-crypto
pub fn seal<P>(key: &DataKey, ctx: &SealCtx, plaintext: &[u8], rng: &dyn CryptoRng)
    -> Result<Sealed<P>, CryptoError>;                       // Nonce intern, nie Parameter
pub fn open<P>(key: &DataKey, ctx: &SealCtx, s: &Sealed<P>) -> Result<Zeroizing<Vec<u8>>, CryptoError>;
```

**Fsync-Regel [E]:** Nach `FsyncFailed` DARF dieselbe Datei NICHT erneut synchronisiert und als stabil angenommen werden (das Betriebssystem kann fehlgeschlagene Seiten verwerfen). Der Inhalt wird aus dem Journal oder dem Speicher in eine **neue** Datei geschrieben; scheitert auch das, wird der betroffene Bereich vergiftet (15.2). Kantenports (`Embedder`, `Reranker`, `Generator`, `Verifier`, `Notifier`, `ArchiveHook`, `RegistrySink`, `KeyAnchor`) stehen in 11.1, 8.4, 8.6, 6.2 und 4.4.

### 3.8 Abhängigkeits-Whitelist [E]

Externe Crates sind nur aus der in `deny.toml` geführten Whitelist zulässig: `chacha20poly1305`, `hkdf`, `hmac`, `sha2`, `ed25519-dalek`, `argon2`, `zeroize`, `subtle`, `roaring`, `crc32c`, `thiserror`, `unicode-normalization`, `unicode-segmentation`, `regex` (linear, ohne Rückverfolgung), `rustix` (Features `mm`, `process`, `rand`), `getrandom`, `serde` und `serde_json` (nur API und Berichte, **nie** Plattenformate), eine RFC-8785-Bibliothek (JCS; Auswahl per ADR). Tests: `proptest`, `loom`, `cargo-fuzz`, `trybuild`, `cargo-mutants`. Rand (L8 und höher): `rustls` (mTLS), `hyper`/`axum`, `tokio`. Sandbox: `landlock`, `seccompiler`. Parser-Abhängigkeiten (PDF, Office, Archive) sind **nur** im Abhängigkeitsbaum von `acta-parse` und `acta-parse-worker` zulässig und werden getrennt geführt. Jede Aufnahme verlangt Lizenz-, Wartungs- und Schwachstellenprüfung (G-SUPPLY) und einen ADR-Eintrag.
---

## 4. Kryptografie und Schlüsselverwahrung

### 4.1 Primitive und Domänenpräfixe [E]

|Zweck|Verfahren|Bibliothek|
|---|---|---|
|Datensatz-Verschlüsselung|XChaCha20-Poly1305, 256-Bit-Schlüssel, 192-Bit-Zufallsnonce aus `CryptoRng` (kein API-Parameter)|`chacha20poly1305`|
|Schlüsselableitung|HKDF-SHA256 (RFC 5869); Fehler werden propagiert|`hkdf`, `sha2`|
|Passwortbasierte Ableitung (nur `FileAnchor`)|Argon2id nach RFC 9106, **m ≥ 65 536 KiB, t ≥ 3, p ≥ 4**; Parameter und Salt je Generation im Kopf der Wurzeldatei|`argon2`|
|Integrität, Commitments, Tags|HMAC-SHA256|`hmac`|
|Objekt- und Kettenhashes|SHA-256|`sha2`|
|Signatur (Berichte, Checkpoints)|Ed25519 (RFC 8032) mit `sig_alg`; Domänenpräfix `acta/report/v1` bzw. `acta/checkpoint/v1`|`ed25519-dalek`|
|Speicherhygiene|`Zeroizing`, `SecureBuf`, konstante Zeit beim Vergleich|`zeroize`, `subtle`|

**Begründung:** Ein einziger Primitivsatz (SHA-2-Familie, ChaCha, Ed25519) hält den unabhängigen Prüfer klein und das Krypto-Audit überschaubar. Die 192-Bit-Zufallsnonce macht zufällige Wahl unproblematisch. Die Argon2id-Parameter folgen der zweiten Empfehlung des RFC 9106, weil ein Angreifer mit gestohlenem Tresorabbild offline rechnen kann [V]. **Preis:** XChaCha20-Poly1305 ist kein FIPS-140-Algorithmus; das Umschlagformat trägt ein Algorithmus-Byte (`alg`), `0x02` ist für AES-256-GCM reserviert (4.7). Eigene Primitive sind verboten.

**Domänenpräfixe (vollständig):** `acta/tag/v1`, `acta/tag-key/v1`, `acta/vault/v1`, `acta/vaultbk/v1`, `acta/hdr/v1`, `acta/dek/v1`, `acta/cluster/v1`, `acta/dir/v1`, `acta/wal/v1`, `acta/seg/v1`, `acta/gate/v1`, `acta/dedup/v1`, `acta/blob/v1`, `acta/tree/v1`, `acta/commit/v1`, `acta/cmt/v1` (Commitments), `acta/ledger/v1`, `acta/checkpoint/v1`, `acta/report/v1`, `acta/rec/v1`, `acta/log/v1`, `acta/scope/v1`, `acta/gen/v1`. Neue Zwecke verlangen ein neues Präfix; Wiederverwendung ist verboten.

### 4.2 Schlüsselhierarchie [E]

```
          KeyAnchor (HSM, KMS oder FileAnchor)
   liefert: Generationsgeheimnis M_g · Sicherungs-Epochenschlüssel M_bk(e)
            Notar-Signaturschlüssel · k_idx · Anker-Köpfe der Ketten
                        │
          je Mandant t: Tresor  VK(t,g) = HKDF(M_g, salt = TenantId, info = "acta/vault/v1" ‖ g u32)
                        │   Einträge einzeln versiegelt unter VK(t,g)
   ┌─────────┬──────────┼───────────┬───────────┬────────────┬────────────┐
 SubjectKey DocumentKey SegmentKey ScopeKey   CommitKey(e)  LogKey(m)
 je Betrof- je Dokument- je versie- je Lösch-  je Epoche    je Monat des
 fenem      Version     geltem      vorgang    für Commit-  Zugriffs-
 (+Alias)   (id=VerRef) Segment     (kurz-     ments        protokolls
                        (K, DK, DP) lebig)

 Dokumentkopf-Schlüssel  HK(v)  = HKDF(DocumentKey(v), salt = VerRef(v), info = "acta/hdr/v1")
 Datenschlüssel          DEK(v) = HKDF( IKM  = DocumentKey(v) ‖ SubjectKey(s₁) ‖ … ‖ SubjectKey(sₙ),
                                        salt = VerRef(v),
                                        info = "acta/dek/v1" ‖ n u8 )     s₁ … sₙ aufsteigend nach SubjectTag
 Verzeichnisschlüssel    DirKey(s) = HKDF(SubjectKey(s), "acta/dir/v1")
 Clusterkennung          ClusterId(s) = HKDF(SubjectKey(s), "acta/cluster/v1")[..16]
```

**Dokumentkopf [E].** Die Liste der Betroffenen einer Version (`SubjectTag`s, `n`, Modus, `DocRef`, `VerOrd`, `case_ids`, Klasse, `retain_until`, `supersede_reason`) steht im **Dokumentkopf**, einem kleinen Umschlag unter `HK(v)`. Er ist allein mit dem `DocumentKey` lesbar; daraus werden `SubjectKey`s nachgeschlagen und der `DEK` abgeleitet. Kein Klartext-Tag steht in Segmenten oder Journalen.

**Wirkung:**

- Wird `SubjectKey(sᵢ)` vernichtet, sind **alle** noch an `sᵢ` gebundenen Dokument-Versionen unlesbar (Modus `DeleteAll`, 8.9; Z3).
- Wird `DocumentKey(v)` vernichtet, ist genau diese Version unlesbar (Einzellöschung, Versionsende).
- Zwei Versionen haben nie denselben DEK (Z14); jede Berichtigung, `AddSubject`, `RedactSplit` oder `RestrictShared` erzeugt eine **neue Version** mit neuer `VerRef`, neuem `DocumentKey` und neuem DEK. Die alte Version unterliegt `Superseded` (8.1).
- **Bindungsregel (Beschluss):** `SubjectKey(s)` wird **erst** vernichtet, wenn keine Version mehr an `s` gebunden und lesbar bleiben muss. Dokumente, die wegen Aufbewahrungspflicht oder Hold eines Mitbetroffenen erhalten bleiben (`RestrictShared`), werden zuvor als neue Version **ohne Bindung an `s`** abgelegt (die Daten bleiben, der Schlüssel von `s` ist nicht mehr beteiligt). So blockiert eine zurückgestellte Version nie die Löschung der übrigen Daten von `s` (8.9).
- Höchstens `max_subjects_per_doc` = 64 gebundene Betroffene je Version. Darüber ist das Dokument eine **Sammelakte**: DEK mit `n = 0` (nur `DocumentKey`), die Mitgliederliste steht im Dokumentkopf, jedes Mitglied führt die Version in seinem Betroffenenverzeichnis. Die Löschung eines Mitglieds einer Sammelakte löst **immer** den Review aus und endet in `RedactSplit` (8.9); nie in `DeleteAll`.
- Dokumente ohne Betroffenen (`Unklassifiziert`, Entscheidungsnutzlasten ohne Personenbezug) nutzen ebenfalls `n = 0`.
- **Aliase.** Eine Person kann mehrere externe Kennungen haben. Jede weitere Kennung erhält einen Tresoreintrag `SubjectAlias` (Tag der Kennung → Primär-Tag). Beim Löschen werden alle Aliase aus dem Betroffenenverzeichnis vernichtet.

Fehlt ein erforderlicher Schlüssel und Gate oder Löschregister belegt die Löschung, ist das Ergebnis `Err(Erased)`. Fehlt er ohne Beleg oder ist der Zustand nicht eindeutig (Tresor nicht lesbar), ist das Ergebnis `Err(Integrity)` bzw. `Err(Crypto)` mit Vergiftung. Ein Fallback ist ausgeschlossen (Z10). Lesepfade prüfen das Gate **vor** der Schlüsselsuche (3.6, 8.1); die Reihenfolge verhindert, dass ein bereits entfernter Tresoreintrag eine gesperrte Version als Integritätsfehler erscheinen lässt.

### 4.3 Tresor, Rotation, Sicherungsexport [E]

**Eintrag (Klartext):** `[kind u8][status u8][id 16][key 32]` = 50 Byte; versiegelt zu `[len u16][nonce 24][ciphertext 50][tag 16]` = 92 Byte. AAD = `acta/vault/v1 ‖ TenantId ‖ generation u32 ‖ slot u64` (**Position im Tresor**; die `id` steht nur im Chiffrat). `kind`: `0x01` SubjectKey, `0x02` DocumentKey (id = VerRef), `0x03` SegmentKey (id = `file_id`), `0x04` ScopeKey (id = ErasureId), `0x05` CommitKey (id = Epoche), `0x06` SubjectAlias (id = Alias-Tag, key = Primär-Tag ‖ 16 Nullbyte), `0x07` LogKey (id = Monatsindex). `status`: `0x01` aktiv, `0x02` vernichtet.

**Ein Tresor je Mandant** (`vault-<TenantOrd>.<generation>`). Der **Speicherindex** (`KeyId → slot`) existiert **nur im Arbeitsspeicher** und wird beim Öffnen durch Scan aufgebaut; es gibt keinen persistenten Index mit Klartext-IDs. **[Q]** Öffnen eines Tresors mit 10 Mio. Einträgen unter 30 s.

**R-KEY-FIRST [E] (Schlüssel zuerst).** Jeder Schlüssel wird im aktuellen Tresor (Generation g) angefügt und mit `fsync` bestätigt (Gruppen-Commit, 3.6), **bevor** ein Umschlag unter ihm geschrieben oder referenziert wird. Es gibt kein Schlüsseljournal außerhalb des Tresors und keine Schlüsselhaltung nur im Arbeitsspeicher.

**Vernichtung eines Schlüssels:** Der Eintrag wird durch einen Grabstein ersetzt (`status = 0x02`, `id` und `key` genullt, gleiche Länge, gleiche Versiegelung unter gleichem Slot), die Datei wird synchronisiert, und der Zustand wird mit `O_DIRECT` zurückgelesen (Direct-Read-Verifikation). Ein Grabstein trägt keine Kennung. **Batch-Vernichtung:** `destroy_many(&[KeyId])` bündelt Schreibvorgänge und `fsync`.

**Rest-Fenster auf Flash [E].** Das Überschreiben an Ort und Stelle erreicht auf Flash und in Dateisystem-Journalen nicht zuverlässig die physische Zelle (A-FLASH). Der alte Chiffreeintrag unter `VK(t,g)` bleibt bis zur Rotation lesbar für jeden, der Abbild **und** `M_g` besitzt (T5). Ein Grabstein allein erreicht daher nur den Zwischenstand `key_tombstoned` (8.7).

**Wurzelrotation (orchestriert über alle Mandanten).** Auslöser: alle `T_ROT` = 12 h, nach `N_SHRED` = 10 000 Vernichtungen, oder **auf Anforderung** (Klasse `Besonders`, Löschvorgänge mit 70 % verbrauchtem `erase_sla_B`; frühestens `rotation_min_gap` = 15 min nach der letzten Rotation, Anforderungen werden gebündelt). Ablauf:

1. `rotation_lock` nehmen (Mandantenanlage wartet). `anchor.rotate()` liefert `g+1`; die Mandantenliste wird festgehalten.
2. Je Mandant (bis `rotation_parallel` = 4 gleichzeitig): Schnappschuss `N0` aller **lebenden** Einträge; Neuschreiben unter `VK(t,g+1)` in `vault-<ord>.<g+1>.tmp` **ohne Sperre** für Schlüsselerzeugung und -vernichtung. Tresor `g` arbeitet unverändert weiter (Anfügen, Grabsteine). Grabsteine werden **nicht** übernommen (Verdichtung).
3. `vault_freeze(t)` (kurz): der **Schwanz** (alle Anfügungen und Grabsteine nach `N0`) wird in die neue Datei übernommen; `fsync` der Datei, atomares `rename`, `fsync` des Verzeichnisses; Speicherindex wird getauscht; `vault_freeze` endet.
4. Wenn **alle** Mandanten umgestellt sind: `anchor.destroy(g)` liefert `DestroyProof`; Journal-Eintrag `RootRotated { generation }` im E-Ledger jedes Mandanten (Reihenfolge: Tatsache vor Journal, C16).

Fällt ein Mandant aus, bleibt `M_g` erhalten, die Rotation wird wiederholt (`RotationOverdue`, Klasse K verzögert sich). Eine Tresordatei der Generation ≤ g ist nach Schritt 4 nicht mehr lesbar. Absturz zwischen Schritt 3 und 4: Der Start setzt Schritt 4 fort, sobald `anchor.current_generation() > vault.generation` für alle Mandanten gilt, und beendet unvollständige Mandanten zuerst.

**Sicherungsexport [E].** Eine Sicherung des Tresors ist **kein** Dateikopie-Abbild. `acta-cli backup vault` und die Hintergrundaufgabe schreiben einen **Export**, in dem alle lebenden Einträge unter `VKb(t,e) = HKDF(M_bk(e), salt = TenantId, info = "acta/vaultbk/v1")` neu versiegelt sind (keine Grabsteine, einschließlich `CommitKey`s und `LogKey`s). Der Anker erzeugt `M_bk(e)` je Sicherungsepoche `backup_epoch` = 24 h und vernichtet ihn zum Zeitpunkt `end(e) + T_BACKUP` (`destroy_backup_epoch`). Ein Export ist damit höchstens `T_BACKUP + backup_epoch` gültig; danach ist er unlesbar, unabhängig davon, wohin er kopiert wurde. Exportdateien werden nach Epochenende + `T_BACKUP` mit Scrub entfernt. Export und Schlüsseländerungen sind über `vault_freeze` serialisiert (konsistenter Stand).

**Wiederherstellungspunkt für Schlüssel (RPO) [E].** Schlüssel, die nach dem letzten gültigen Export entstanden, gehen bei Tresorverlust verloren; die zugehörigen Dokumente sind dann unlesbar. Daher gilt `vault_export_interval` = 1 h; der Betriebsbericht weist das tatsächliche Intervall aus; fehlt ein gültiger Export innerhalb `T_BACKUP`, löst `BackupOverdue` aus (14.3). Ein vom Tresor getrenntes Schlüsseljournal ist ausgeschlossen (es wäre eine nicht rotierte Schlüsselkopie).

**Tresorgröße.** 92 Byte je Eintrag; bei 50 Mio. Dokument-/Personenschlüsseln ca. 4,6 GB. **[Q]** Rotation von 10 Mio. Einträgen unter 120 s.

**Off-Boarding eines Mandanten:** Tresordatei und Exportdateien des Mandanten werden entfernt (Scrub). Weil `M_bk(e)` global ist, endet die Lesbarkeit kopierter Exporte erst mit Ablauf der Sicherungsepochen; der Bericht weist `offboarding_exposure_until` aus (12.6).

### 4.4 KeyAnchor [E]

```rust
pub trait KeyAnchor: Send + Sync {
    fn current_generation(&self) -> Result<u32, AnchorError>;
    fn min_generation(&self) -> Result<u32, AnchorError>;                       // niedrigste noch gültige Generation
    fn master(&self, generation: u32) -> Result<MasterKey, AnchorError>;       // Zeroizing, kein Clone/Default
    fn rotate(&self) -> Result<u32, AnchorError>;
    fn destroy(&self, generation: u32) -> Result<DestroyProof, AnchorError>;
    fn backup_master(&self, epoch: u32) -> Result<MasterKey, AnchorError>;
    fn destroy_backup_epoch(&self, epoch: u32) -> Result<DestroyProof, AnchorError>;
    fn index_secret(&self) -> Result<IndexSecret, AnchorError>;                 // k_idx
    fn notary_sign(&self, domain: &str, msg: &[u8]) -> Result<Signature, AnchorError>;
    fn notary_public(&self) -> Result<PublicKeyInfo, AnchorError>;              // inkl. sig_alg, key_id
    fn record_head(&self, chain: ChainId, seq: u64, hash: [u8; 32]) -> Result<(), AnchorError>;
    fn last_head(&self, chain: ChainId) -> Result<Option<(u64, [u8; 32])>, AnchorError>;
    fn assurance(&self) -> AnchorAssurance;           // Hardware | Kms { destroy_window } | FilePassphrase
}
```

Implementierungen: `FileAnchor` (Entwicklung, kleine Installationen ohne `Besonders`), `Pkcs11Anchor` und `KmsAnchor` (Produktion; getrennte Features, additiv).

- **[A] A-ANCHOR:** Der Anker kann ein Generationsgeheimnis innerhalb einer deklarierten Frist `anchor_destroy_max` so vernichten, dass es nicht rekonstruierbar ist. Besitzt ein KMS eine Pflicht-Löschfrist (z. B. Wartefenster bei Cloud-Diensten, Herstellerangaben [V]), MUSS `anchor_destroy_max` sie enthalten; Klasse K ist dann frühestens nach dieser Frist erreicht, und der Bericht weist die tatsächliche Frist aus.
- **`FileAnchor`:** Die Generationsdatei liegt auf demselben Medium wie die Daten; `destroy` kann Flash-Reste nicht ausschließen. Klasse K wird mit `assurance = FilePassphrase` ausgewiesen; die Unlesbarkeit beruht dann allein auf Passphrase und Argon2id. Die Passphrase MUSS mindestens 80 Bit geschätzte Entropie haben (Stärkeprüfung beim Anlegen), DARF nirgends gespeichert werden, ist auf mindestens zwei Personen aufzuteilen und bei Verdacht zu wechseln. `FileAnchor` ist für Produktion mit `Besonders` **nicht zulässig** (Invariante R-ANCHOR).
- **Anker-Köpfe.** Ledger und Gate-Log melden ihren Kopf (`seq`, `hash`) beim Checkpoint an den Anker (`record_head`). Nach einer Wiederherstellung erkennt der Knoten damit ein zurückgerolltes Ledger (8.8).
- Schlüsselhinterlegung beim Hersteller und Wiederherstellungsdienste des Herstellers sind ausgeschlossen (Z19).

### 4.5 Umschlag v1 [E]

```
┌─────────┬─────────┬──────────────┬────────────┬──────────────────────────────┬──────────┐
│ ver u8  │ alg u8  │ key_class u8 │ nonce (24) │ ciphertext (gepolstert)      │ tag (16) │
│  0x01   │  0x01   │ 0x01 P·0x02 K│            │ [len u32][Daten][Nullfüllung]│ Poly1305 │
│         │         │ ·0x03 D      │            │                              │          │
└─────────┴─────────┴──────────────┴────────────┴──────────────────────────────┴──────────┘
AAD = "acta/rec/v1" ‖ tenant_id ‖ class_u8 ‖ keyspace_u16 ‖ record_ref(16) ‖ version_u64be
```

**Keyspaces (`keyspace_u16`):** `0x0001` P-Datensatz, `0x0002` P-Dokumentkopf, `0x0003` Betroffenenverzeichnis, `0x0004` Handle-Register, `0x0005` Vorgang, `0x0006` Entscheidungsnutzlast, `0x0010` K-Pack-Block, `0x0011` Entscheidungsprotokoll-Block, `0x0020` DK-Block, `0x0021` DP-Block, `0x0030` Journalnutzlast, `0x0040` Arbeitsdaten (ScopeKey), `0x0050` Zugriffsprotokoll-Eintrag, `0x0060` Ausgangskorb, `0x0070` Idempotenztabelle.

**Auffüllung (Padding).** Der Klartext wird vor dem Verschlüsseln auf Größenklassen aufgefüllt: bis 4 096 Byte auf ein Vielfaches von 256, darüber auf das nächste Vielfache von `2^(⌊log2 n⌋−3)` (höchstens 12,5 % Mehrbedarf). Damit verraten exakte Längen weder Dokument noch Betroffene; die Größenklasse bleibt als Restmerkmal und wird ausgewiesen.

`open` liefert `Zeroizing<Vec<u8>>` mit vorab reservierter Kapazität (kein Wachsen, damit keine Restkopien entstehen). Jeder Fehler (Länge, MAC, Schlüssel) führt ohne unterscheidbares Orakel zu `DecryptionFailed`; ist der Schlüssel nachweislich vernichtet (Gate oder Register), zu `Erased`. Die Nonce ist **kein** Parameter der öffentlichen Schnittstelle.

### 4.6 Zufall, Zeit, Geheimnisse [E]

`CryptoRng::try_fill` ist fehlbar; ein Entropiefehler erzeugt nie einen Umschlag oder Schlüssel (`T-RNG-*`). `Clock` ist ein Port; `SystemTime::now` und `thread_rng` sind im Kern verboten. **Zeit ist nicht vertrauenswürdig (A-CLOCK):** `tx_time` fällt nie (6.3); ein Rücksprung der Uhr über `clock_skew_max` = 5 min setzt den Knoten in `Poisoned(Node, ClockRegression)`. Geheimnisse (`Dek`, `MasterKey`, `IndexSecret`, `SubjectKey`, `DocumentKey`, `SegmentKey`, `ScopeKey`, `LogKey`) besitzen weder `Default`, `Clone`, `Debug` noch `Serialize`. Schlüsselmaterial liegt ausschließlich in `SecureBuf` (`mlock`, `MADV_DONTDUMP`, feste Kapazität).

### 4.7 Kryptoagilität und Neubewertung [E]

Die Unlesbarkeit vernichteter Daten gilt nur, solange die Primitive halten (EDPB-Leitlinien 02/2025 zur Blockchain, Rn. 51, 85, 96: Notfallplan für den Algorithmenwechsel, Risikobewertung unter Einbeziehung von Obsoleszenz und Quantencomputern, periodische Neubewertung [V]). Acta setzt um:

1. **Neubewertung:** Mindestens alle `crypto_review_interval` = 12 Monate erzeugt `acta-reports` einen Kryptobericht (verwendete `alg`/`sig_alg`, Schlüssellängen, Abgleich gegen die jeweils aktuelle BSI TR-02102 [V] und Herstellerhinweise). Er ist Teil der Freigabepflichten (20.4).
2. **Migration ohne Datenverlust (`Rewrap`):** liest Umschläge mit altem `alg`, schreibt sie mit neuem `alg` unter demselben Schlüssel neu (Hintergrundaufgabe, Segment für Segment, atomarer Austausch) und weist den Anteil migrierter Segmente aus. Zuerst werden noch **nicht vernichtete** Daten umgeschrieben; Chiffrate vernichteter Daten sind ohne Schlüssel und werden mit der physischen Bereinigung entfernt.
3. **Signaturen:** Berichte und Checkpoints tragen `sig_alg`. Ed25519 ist nicht quantenresistent; Berichte DÜRFEN zusätzlich mit einem zweiten Verfahren signiert werden (hybrid), sobald ein standardisiertes Verfahren verfügbar ist. Alte Checkpoints werden vor Ablauf der Vertrauensdauer des alten Verfahrens neu bezeugt (Re-Witness, Eintrag im E-Ledger).
4. **Symmetrische Schlüssel** mit 256 Bit gelten nach heutigem Kenntnisstand als ausreichend; das ist eine Annahme (A-CRYPTO), keine Zusage.
5. **Notfallplan Algorithmenwechsel:** Auslöser (Herstellerhinweis, BSI-Empfehlung, Kryptoanalyse), Verantwortliche, Reihenfolge (zuerst `Rewrap` lebender Daten, dann Signaturen, dann Tresor und Anker), Zielzeiten, Entscheidung über die vorzeitige Beendigung des alten Verfahrens. Der Plan ist Teil des Löschkonzepts (12.4).
6. **Algorithmus-Byte `0x02` (AES-256-GCM)** bleibt reserviert; ein Wechsel erfordert `Rewrap` und einen ADR-Eintrag.
---

## 5. Speicher

### 5.1 Segmentspeicher (`acta-seg`) [E]

Der Speicher ist ein **append-only Segmentspeicher**, kein allgemeiner LSM-Baum. **Begründung:** Die Last ist „einmal schreiben, häufig lesen, gezielt nach Betroffenem entfernen“; Aktualisierungen in der Fläche kommen nicht vor. Segmente lassen sich als Ganzes versiegeln, gezielt ersetzen oder entfernen. `acta-seg` ist **schlüsselfrei** (HR-9): Schlüssel werden als Parameter übergeben.

**Siegelmodi:**

|Modus|Schlüssel|Einsatz|Löschwirkung|
|---|---|---|---|
|`RecordSealed`|DEK je Dokument-Version (4.2)|Ebene P: Datensätze, Köpfe|Schlüsselvernichtung macht Datensätze unlesbar|
|`SegmentSealed`|`SegmentKey` je Segment|Ebene K (Packs, Entscheidungsprotokolle), D (Indexsegmente), Journal|Ersatzsegment schreiben, alten `SegmentKey` vernichten, Altdatei mit Scrub entfernen|

**Partitionen (Invariante I-PART):** `K`, `P`, `DK` (Ableitungen aus K) und `DP` (Ableitungen aus P) werden **niemals** in derselben Datei gemischt. Ein Indexsegment hat genau **eine** Quellart; `source_set` ist typisiert (`KOrd` oder `VerOrd`).

**Dateien je Mandant:** Journal (`wal-<journal_id>`), Segmente (`seg-<file_id>.acta`), Manifest (`MANIFEST`, `manifest-<n>`), Tresor, Gate-Log, Ledger, Betriebsdaten. Alle Dateizugriffe laufen ausschließlich über `acta-fs`.

**Journal-Rahmen:** `[magic u8][flags u8][len u32][hdr_crc32c u32][payload][chain_tag 32]`. `chain_tag` ist eine HMAC-SHA256-Kette über alle Rahmen mit Schlüssel `HKDF(k_idx, "acta/wal/v1" ‖ TenantId ‖ journal_id)`. Nutzlasten der Klasse P liegen ausnahmslos als Umschlag unter dem Versions-DEK vor; Nutzlasten der Klassen K und D sind unter dem `SegmentKey` des Journals (keyspace `0x0030`) versiegelt. Das Journal enthält nie Klartext. **Flush-Politik:** Das Journal enthält nur noch nicht in Segmente überführte Daten; es wird bei `wal_max_bytes` (64 MiB), `wal_max_age` (30 s) und bei jedem Saga-Schritt S3 in Segmente überführt, rotiert, und die Altdatei wird mit Scrub entfernt (Journal-`SegmentKey` vernichtet).

**Segmentformat (`format_version` 1):**

|Abschnitt|Inhalt|
|---|---|
|Kopf (256 B)|Magic `ACTASEG\0`, `format_version u16`, `partition u8`, `seal_mode u8`, `tenant_ord u32`, `file_id 16`, `created_seq u64`, `block_size u32` (64 KiB Klartext), `block_count u32`, Kopf-MAC (HMAC mit `HKDF(k_idx, "acta/seg/v1" ‖ file_id)`)|
|Datenblöcke|`SegmentSealed`: je Block ein Umschlag (AAD: `file_id`, Blockindex). `RecordSealed`: Rahmen `[rec_key][len u32][Umschlag]`|
|Blockindex|Folge `(erster_Schlüssel, Offset, Länge, crc32c)`; bei `SegmentSealed` selbst versiegelt|
|Fuß|`source_set` (Roaring, serialisiert; nur bei `P`, `DK`, `DP`), Statistik (Anzahl, Schlüsselbereich), SHA-256 über alle vorangehenden Bytes, Fuß-MAC, `footer_len u32`, End-Magic|

**Manifest:** atomare Batches `AddSegment`, `RemoveSegment`, `SeqHwm`, `Partition`, HMAC-gekettet. Ein fehlgeschlagener Manifest-Schreibvorgang vergiftet den Speicher sofort (Fail-Closed, `Poisoned(Tenant)`). Sichtbar ist ein Segment erst durch das Manifest (C03, C04).

**Größen:** `segment_max_bytes` = 256 MiB begrenzt die Kosten eines gefilterten Rewrite.

**Schlüsselordnung der Datensätze (P):** `[kind u8][tenant_ord u32][partition u8][cluster_id 16][ver_ref 16][!seq u64][flags u8]`. `cluster_id` ist die `ClusterId` des Hauptbetroffenen (bei Dokumenten ohne Betroffenen die `DocRef`). **Wirkung:** Alle Daten einer Person liegen zusammen; bei Löschung genügt häufig das Entfernen ganzer Segmente (Datei-Drop). Die Ingest-Planung bündelt nach `ClusterId`: Cluster mit mehr als `cluster_seg_min` (4 MiB) erhalten eigene Segmente, kleinere werden nach Schlüsselordnung in gemeinsame Segmente gepackt (dann Rewrite ohne Zielrecords, begrenzt durch `segment_max_bytes`).

**`ClusterId`:** nicht gespeichert, nur mit `SubjectKey` berechenbar (4.2). Nach dessen Vernichtung ist jede in Sicherungen verbliebene `cluster_id` ein bedeutungsloser Zufallswert; ein dauerhaftes, mit `k_idx` prüfbares Pseudonym entsteht nicht (Z2). **`SubjectTag` steht nie im Klartext in Segmenten, Journalen oder Köpfen.** Die Gruppierung gleicher `cluster_id` im Abbild bleibt, solange der Schlüssel lebt, als Restrisiko R-CLUSTER ausgewiesen.

**Handkodierte Formate:** Alle Plattenformate werden von Hand mit geprüften Feldzugriffen kodiert (`from_be_bytes`, `get(..)`); `serde` und `bincode` sind für Plattenformate verboten. **Begründung:** Ein Plattenformat ist eine über Jahre stabile Schnittstelle und darf nicht aus Strukturlayouts folgen. Jedes Format hat Golden-Tests und ein Fuzz-Target.

**Entscheidungsregel Eigenbau [E]:** Eine bestehende eingebettete Speicher-Engine wird nur verwendet, wenn sie (1) gezieltes Entfernen und Überschreiben der Dateien eines Betroffenen ohne globale Verdichtung erlaubt, (2) deterministisch unter `FaultVfs` prüfbar ist, (3) Werte vor dem Schreiben im Journal als Umschlag entgegennimmt und (4) keine FFI-Fläche mitbringt. Das Ergebnis wird vor Beginn der Speicherimplementierung als Messbericht dokumentiert; fällt die Prüfung negativ aus, gilt diese Spezifikation unverändert (Eigenbau, Budget 6 500 LOC).

### 5.2 Ordinale, Herkunft (Lineage) und Sperrmengen [E]

1. Jede **Version** eines P-Dokuments erhält beim Ingest eine `VerOrd`, jeder Betroffene eine `SubjOrd`, jedes K-Objekt eine `KOrd` (alle monoton je Mandant, nie wiederverwendet). Sie sind rein intern, erscheinen nie in Ebene K, Berichten, Ledgern oder Metriken und stehen nur im versiegelten Dokumentkopf und Betroffenenverzeichnis, in `DP`-/`DK`-Fußabschnitten und im **Gate-Log**.
2. Jede abgeleitete Struktur (Indexsegment, Cache-Eintrag, Embedding-Block, Sitzungsschritt, Fallindex) trägt `source_set` der Ordinale, aus denen sie entstanden ist.
3. Ein Segment oder Cache-Eintrag ist **betroffen** genau dann, wenn `source_set ∩ (ErasedVers ∪ RestrictedVers ∪ SupersededVers) ≠ ∅` (DP) bzw. `source_set ∩ RedactedK ≠ ∅` (DK).
4. **Typzwang:** `Derived<T>` kapselt Nutzdaten untrennbar mit ihrer `Lineage` (nicht leer). Indizes nehmen Daten nur über `put_derived(Derived<T>)` an. Eine Ableitung ohne Herkunft ist nicht konstruierbar (Z13).
5. Ordinale ohne Zuordnung sind keine Pseudonyme, sobald Verzeichnis und Handle-Register entfernt sind; `ErasedVers` darf daher dauerhaft bestehen (Verdichtung des Logs, 5.3). Restrisiko R-ORD (Zeitmuster dichter Ordinale) wird ausgewiesen.
6. **Versionswechsel.** Ersetzt eine neue Version eine alte, werden alle Ableitungen der alten Version (Indexeinträge, Embeddings, Caches) bereinigt; nur die neue Version wird indiziert.

### 5.3 Gate (`acta-gate`) [E]

`ReadGate`-Zustand (Roaring-Mengen, Epoche, alles im Arbeitsspeicher, Änderungen durch Zeigertausch):

|Menge|Elemente|Bedeutung|
|---|---|---|
|`ErasedSubjects`|`SubjOrd`|Betroffene mit laufender oder abgeschlossener Löschung; verhindert neue Zuordnung|
|`ErasedVers`|`VerOrd`|Versionen im Zustand `Gated`, `Expired` oder erledigt: `check_ver → Erased`|
|`RestrictedVers`|`VerOrd` + Grundcode|`Restricted(grund)`: nur `restricted_read`|
|`SupersededVers`|`VerOrd`|ältere Version: nur `restricted_read`|
|`RedactedK`|`KOrd`|Takedown in K: Objekt gesperrt, Platzhalter `Redacted`|

`ErasedVers` hat Vorrang vor `RestrictedVers` und `SupersededVers`. `check_ver` und `is_redacted_k` sind O(1) ohne E/A. **Sondierung:** Jeder Lesepfad (Suche, Antwort, Zeitreise, Szenario, Cache-Treffer, Export, Vorschau, Sitzung, Streaming) ruft das Gate **vor jeder Entschlüsselung und vor jeder Ausgabe** auf; Cache-Einträge sind betroffen, wenn ihre Lineage das Gate trifft (Miss). Eine Anfrage merkt die Epoche und prüft sie vor jedem Ausgaberahmen (9.5).

**Gate-Log.** Rahmen `[seq u64][op u8][len u16][payload][chain_tag 32]`, HMAC-Kette mit `HKDF(k_idx, "acta/gate/v1" ‖ TenantId)`. Operationen: `0x01 EraseSubject(SubjOrd)`, `0x02 EraseVer(VerOrd, reason)`, `0x03 Restrict(VerOrd, grund)`, `0x04 Release(VerOrd)` (nur aus `Restricted`, nie aus `Erased`), `0x05 Supersede(VerOrd)`, `0x06 RedactK(KOrd)`, `0x07 HighWater(kind, value)`, `0x08 Snapshot(seq)`. Die Epoche ist die `seq` des letzten Rahmens. **Gruppen-`fsync`** (3.6). **Verdichtung** wöchentlich: neuer Snapshot-Stand (Roaring-Mengen, MAC) in neuer Datei, atomarer Austausch, alter Log wird archiviert (C23); `ErasedVers`, `ErasedSubjects`, `RedactedK` werden nie verkleinert.

**Reihenfolge einer Sperre (Z4).** Zuerst im RAM (Epoche erhöht), Erfolg wird erst nach `fsync` gemeldet. Schlägt der `fsync` fehl, bleibt die RAM-Sperre bestehen, der Aufruf scheitert mit `Integrity`, der Mandant wird vergiftet (`Poisoned(Tenant)`, 15.2); nach dem Neustart gilt der durable Stand, fehlt der Eintrag, ist der Antrag erneut zu stellen (der Aufruf hat nie Erfolg gemeldet).

### 5.4 P-Speicher (`acta-pstore`) [E]

**Dokument-Version.** Eine Version besteht aus (a) Dokumentkopf unter `HK(v)` (4.2), (b) Inhaltsrecords (Blöcke des Quelltexts, Chunk-Tabelle, Metadaten) unter `DEK(v)`, (c) Ableitungen in DP. `DocRef` ist die stabile Dokumentidentität; die aktuelle Version steht im Handle-Register.

**Betroffenenverzeichnis.** Je Betroffenem ein Datensatz unter `cluster_id`, versiegelt unter `DirKey(s)`. Inhalt: `SubjOrd`, `subject_kind` (`External`, `Staff`, `Synthetic`), Aliasliste, Liste `(DocRef, VerRef, VerOrd, Rolle main|co, Zustand)`, `ActorRef`-Handles (bei `Staff`, je Epoche), Empfängerliste für Art. 19 (`DisclosureLog`), Anfragen (`request_id`, `due_at`, `identity_verified`), Fristen. Auffinden einer Person: externe Kennung → `SubjectTag` → Tresoreintrag → `SubjectKey` (bei Alias: Primär-Tag) → `ClusterId` → Verzeichnis.

**Handle-Register.** Abbildung `PersonalRef → (DocRef, VerRef, VerOrd, ClusterId)` im P-Teil, versiegelt unter dem Tresorschlüssel `RegistryKey(t)` (abgeleitet `HKDF(VK(t,g), "acta/gen/v1")` und mit jeder Rotation neu versiegelt). Wird bei der physischen Bereinigung (S3) entfernt. Verbleibende Kopien in Sicherungen enthalten nur Zufallswerte ohne Zuordnung (R-HANDLE).

**Vorgang (`Case`).** P-Objekt der Klasse `Personal`: `CaseId`, `external_ref` (Aktenzeichen des Fachverfahrens), Mitglieds-`VerRef`s, zugewiesene Nutzer (`ActorRef`), Status (`open`, `closed`), `closed_at`. DEK mit `n = 0`. Der **Fallindex** (Roaring-Bitmap `VerOrd` je `CaseId`) ist ein DP-Segment mit Lineage; er speist die Zugriffsmenge `A_P` (9.4). Ein Vorgang endet, wenn die Fachanwendung ihn schließt oder alle Mitglieder erledigt sind; bei Löschung des letzten Mitglieds wird der Vorgang vernichtet (S3). Restrisiko R-CASE: Das Aktenzeichen im Vorgang ist pseudonym, bis der Vorgang vernichtet ist.

**Idempotenztabelle.** `ingest_id → (VerRef, Ergebnis)` je Mandant, versiegelt (keyspace `0x0070`), Lebensdauer `ingest_idem_ttl` = 7 Tage.

### 5.5 Ableitungen D [E]

- **DK-Segmente** sind global je Mandant und `IndexGen`, `SegmentSealed`, ohne Löschlogik; sie tragen `KOrd`-Lineage; Takedown erzeugt Ersatzsegmente (6.7).
- **DP-Segmente** sind **je `ClusterId`** des Hauptbetroffenen organisiert: Cluster mit mehr als `dp_cluster_min_bytes` (1 MiB Index) erhalten eigene Segmente (Verschmelzung nur innerhalb des Clusters), kleinere werden in gemeinsamen Bucket-Segmenten nach Schlüsselbereich gepackt. Erledigt eine Löschung den Hauptbetroffenen, genügt Datei-Drop; bei Nebenbetroffenen oder Bucket-Segmenten Ersatzsegment mit neuem `SegmentKey` ohne die Zielordinale.
- **Datensicherung der Ableitungen:** DK-Segmente sind Teil der Datensicherung (Z7 nach Wiederherstellung); DP-Segmente **nicht** (`backup_derived_dp = false`, Standard und Pflichtwert bei `Besonders`); sie werden nach Wiederherstellung neu aufgebaut (14.4). Ein Betreiber DARF `backup_derived_dp = true` setzen; dann gilt für wiederhergestellte DP-Segmente der Wiederholungsschritt S3 (8.8).
- **Caches** (Antwort-, Sitzungs-, Chunk-, DEK-Cache) sind Ebene D im Arbeitsspeicher, tragen Lineage und sind `ErasureLayer`.

### 5.6 Schreibpfad-Regeln [E]

|ID|Regel|
|---|---|
|WP-1|**Schlüssel zuerst:** Schlüssel durabel im Tresor, dann Umschlag (R-KEY-FIRST)|
|WP-2|**Ledger zuerst:** Ledger-Eintrag durabel, dann Ref-Bewegung bzw. Sichtbarkeit|
|WP-3|**Sichtbarkeit zuletzt:** Manifest-Batch bzw. K-Commit macht Daten sichtbar; Absturz davor hinterlässt nur unsichtbare, verwaiste Dateien (C03 bis C05)|
|WP-4|**Physische Tatsachen vor Journal:** beim Neustart gelten Tresorzustand, Manifest und vorhandene Dateien vor dem Journalstand|
|WP-5|**Atomar ersetzen:** neue Datei schreiben, `fsync`, `rename`, `fsync` des Verzeichnisses (`rename_atomic`)|
|WP-6|**Scrub:** entfernte Dateien werden überschrieben (Nullen), synchronisiert und gelöscht (`remove_scrub`); Flash-Reste bleiben Annahme (A-FLASH)|
|WP-7|**Kein Retry nach `FsyncFailed`** (3.7)|
---

## 6. Versionskontrolle der Wissensebene („Git für RAG“)

### 6.1 Objektmodell [E]

Alle Objekte sind unveränderlich und inhaltsadressiert: `ObjectId = SHA-256(Domänenpräfix ‖ kanonische Bytes)`.

|Objekt|Inhalt|
|---|---|
|**Blob**|Bytes eines Wissensdokuments oder einer Konfiguration. Kanonische Bytes: `acta/blob/v1 ‖ salt(16) ‖ Inhalt`, `salt = HMAC-SHA256(K_dedup(t), Inhalt)[..16]`, `K_dedup(t) = HKDF(k_idx, "acta/dedup/v1" ‖ TenantId)`. Keine Personendaten (Z5)|
|**Tree**|Sortierte Einträge `{name, kind, id, meta}`; `kind ∈ {Blob 0x01, Tree 0x02, PersonalRef 0x03}`. Bei `PersonalRef` ist `id` das 16-Byte-Handle, **nie** ein Inhaltshash. Präfix `acta/tree/v1`|
|**Commit**|`{tree, parents[], tx_seq, tx_time, valid_from?, valid_to?, author: ActorRef, kind, note_tag, flags}`; `kind ∈ {Ingest, Edit, Merge, Config, Takedown, Release}`; `flags`: `clamped`. Präfix `acta/commit/v1`|
|**Ref**|Benannter Zeiger auf einen Commit: Branch (`main`, `szenario/<id>`, `entwurf/<id>`, `policy`) oder Tag (unveränderlich)|

**Schlüssel-Salz statt reinem Inhaltshash.** Ein reiner Inhaltshash ist für Inhalte mit geringer Entropie wörterbuchprüfbar und bliebe nach einem Takedown als prüfbares Pseudonym zurück. Das Salz ist inhaltsabhängig, aber **geheim abgeleitet**: Gleicher Inhalt ergibt im Mandanten dieselbe `ObjectId` (Idempotenz), ohne `K_dedup` ist der Hash nicht prüfbar. Ein Takedown entfernt Inhalt **und** Salz. Mandantenübergreifende Gleichheit gibt es bewusst nicht.

`ActorRef` ist ein zufälliges Handle auf eine Identität in P. `note_tag` ist eine strukturierte Kennung (`u16`) aus einer Whitelist; freier Text wird nie in Commits gespeichert, sondern als validierter Blob abgelegt. Baumnamen sind NFC-normalisiert, enthalten weder NUL noch `/` und sind höchstens 255 Byte lang.

**Wissensobjekte im Tree (Pfadkonvention):**

|Pfad|Inhalt|
|---|---|
|`collections/<name>/docs/…`|Wissensdokumente (Blobs) mit Metadaten im `meta`-Feld (12.2)|
|`collections/<name>/collection.cfg`|Sammlungskonfiguration (12.2)|
|`config/chunk_cfg`, `config/retrieval_cfg`, `config/models`|Chunking-, Abruf- und Modellbeschreibung (`ModelFingerprint`)|
|`prompts/…`|Prompt-Vorlagen, Systemprompt, Verifier-Prompt|
|`templates/…`|Schreiben-Vorlagen und Textbausteine (10)|
|`checklists/…`, `calendar/…`|Prüflisten, Feiertags- und Fristenkalender (10)|
|`goldsets/…`|Fragensets (9.7)|
|`rules/…`|Schwellenwerte (`support_min`, `K_MIN`) und Regeln|

Die Zugriffsrichtlinie (`policy`) liegt auf eigenem Branch `policy` und wirkt **immer in ihrem aktuellen Stand**, nie zeitreisend: ein früherer Wissensstand erweitert nie die Zugriffsrechte (9.4). **Alles, was eine Entscheidung beeinflusst, ist versioniert.**

### 6.2 Protokolle und Integrität [E]

Je Mandant gibt es **drei** getrennte Ledger desselben Typs `HashChain`: **K** (Wissen), **E** (Nachweise) und **A** (Zugriff), `chain_id = SHA-256(TenantId ‖ "K" | "E" | "A")`.

|Ledger|Inhalt|Änderung durch Löschung in P|
|---|---|---|
|**K-Ledger**|Commit, Ref-Bewegung, Tag, `Redacted`, `Expired`, `Waiver`, `DecisionAppended { id, record_hash }`, `PolicyChanged { hash }`|**nie** (Z6)|
|**E-Ledger**|Löschvorgänge (Commitments, Zustandswechsel, Berichts-Hashes), Rotationen (`RootRotated`), Restore-Ereignisse (`Restored`), Re-Witness, `restricted_read`-Zugriffe (Zweckcode, ohne Inhalte), Konfigurationsänderungen von Aufbewahrung und Holds|ja, aber getrennt von K|
|**A-Ledger**|Zugriffsprotokoll: Zeit, `ActorRef`, Aktion (`read`, `query`, `export`, `review`, `admin`), Handle (`PersonalRef`, `CaseId` oder `ObjectId`), Zweckcode, Ergebnis|Einträge sind **versiegelt** unter `LogKey(m)` (Monat); Ablauf von `access_log_retention` vernichtet den Monatsschlüssel, die Kette bleibt über die Chiffrate prüfbar|

`entry_hash = SHA-256(prev_hash ‖ kanonischer Eintrag)`, Domäne `acta/ledger/v1` und `chain_id`; kanonischer Eintrag `[seq u64][kind u8][len u32][payload]`. Beim A-Ledger bildet der Hash die **versiegelte** Nutzlast. Alle `checkpoint_every` = 1 024 Einträge entsteht ein vom Anker signierter Checkpoint (`notary_sign`, `acta/checkpoint/v1`), sein Kopf wird gemeldet (`record_head`). Damit lässt sich die Geschichte ohne Inhaltszugriff prüfen und nicht unbemerkt umschreiben oder zurückrollen. Ledger enthalten keine Personendaten (Z15). Mandanten sehen nur ihre eigenen Ledger; Lücken in globalen Folgenummern gibt es nicht. Abgelaufene A-Ledger-Monate erhalten den Vermerk `Pruned { block, last_hash }`, die Chiffrate dürfen entfernt werden.

**Commitments (Lösch-Paradoxon).** Für jeden vernichteten Schlüssel enthält das E-Ledger `commit(e, KeyId) = HMAC-SHA256(CommitKey(e), "acta/cmt/v1" ‖ KeyId)[..16]`. `KeyId` ist `SubjectTag`, `VerRef` oder Segment-`file_id`, **niemals** eine externe Kennung. `CommitKey(e)` gilt je Epoche `commit_epoch` = 30 Tage, liegt im Mandantentresor und wird `commit_key_retention` nach Epochenende vernichtet (Standard 90 Tage); danach sind die Commitments anonym, die Kette bleibt verifizierbar. **[⚠]** Solange `CommitKey(e)` existiert, sind Commitments pseudonym und personenbezogen; sie brauchen Grundlage, Zweck und Frist (Art. 5 Abs. 2, Art. 17 Abs. 3 lit. e DSGVO; Verweis auf Art. 11 DSGVO in der Literatur). Das AG Lörrach (3 C 1099/24, erstinstanzlich) verneint einen Anspruch der betroffenen Person auf einen dauerhaften individuellen Löschnachweis; ein Gericht hat die Dauer pseudonymer Lösch-IDs nicht entschieden [V]. Die Frist ist Konfiguration des Verantwortlichen mit Untergrenze (R-HORIZON, 15.4).

**Registersenke [E].** Gate-Log und E-Ledger (einschließlich Checkpoints und Hochwassermarken) werden über `erase_registry_sink` (Pflichtparameter, **getrennter Ausfallbereich**: anderes Medium, anderes System) gespiegelt. Das Spiegeln ist asynchron und blockiert die Sperre nie. Der Bericht weist `registry_mirrored` aus.

```rust
pub trait RegistrySink: Send + Sync {
    fn append(&self, tenant: TenantId, rec: &SinkRecord) -> Result<(), SinkError>;
    fn last_acked(&self, tenant: TenantId) -> Result<SinkPosition, SinkError>;
    fn fetch_since(&self, tenant: TenantId, pos: SinkPosition) -> Result<Vec<SinkRecord>, SinkError>;
}
// SinkRecord = GateFrame | LedgerEntry | Checkpoint | HighWater   (keine Inhalte, keine Kennungen)
```

`DirSink` (im Kern): anderes Medium oder Mount, append-only, `fsync`; der Start prüft `st_dev` verschieden vom Datenmedium (R-SINK). Netzsenken liegen in `acta-sink-net`.

### 6.3 Zeitachsen [E]

|Achse|Bedeutung|Abfrageparameter|
|---|---|---|
|**Gültigkeitszeit** `valid`|Fachliche Geltung einer Vorschrift: `[valid_from, valid_to)`, Kalendertage; „kein Wert“ über Präsenzbit, nicht über `0`|`valid_at: Date`|
|**Transaktionszeit** `tx`|Wann Acta den Stand erfasst hat; Commit-Folge `tx_seq` (je Mandant, dicht) mit `tx_time` (nicht fallend)|`as_of: CommitId \| tx_seq \| Zeitpunkt`|
|**Löschzeit** `erase`|Wann P ein Ziel gesperrt hat; nur im E-Ledger und Gate-Log|–|

`tx_time = max(Clock::now(), tx_time(Vorgänger))`; ein geklemmter Wert setzt das Flag `clamped`. Ein Rücksprung über `clock_skew_max` ist `Poisoned(Node, ClockRegression)`. Ein Zeitpunkt für `as_of` wird auf den letzten Commit mit `tx_time ≤ Zeitpunkt` abgebildet (Gleichstand: höchste `tx_seq`). Beispiel: „Welche Fassung der Satzung galt am 1.3. (`valid_at`) und was wusste das System am 15.3. (`as_of`)?“ Die Antwort ist deterministisch (Z7).

### 6.4 Operationen [E]

|Operation|Bedeutung und Pflichten|
|---|---|
|`commit`|Neuer Stand aus Änderungen. Pflicht: `Validated<Tree>` aus dem N-Validator (7.5), Klassendeklaration. Schreibreihenfolge: Objekte → K-Ledger → Ref (WP-2)|
|`branch`, `tag`|Verzweigung für Szenarien, Entwürfe; unveränderliche Markierung|
|`checkout(as_of, valid_at)`|Lesender Stand (Sicht), keine Kopie|
|`diff(a, b)`|Unterschied zweier Stände auf Dokument-, Chunk- und Metadatenebene|
|`merge(src, dst)`|Drei-Wege-Zusammenführung; Konflikte pro Pfad; `PersonalRef`-Einträge werden nach Handle verglichen. Nach `main`: Vier-Augen-Freigabe (konfigurierbar `merge_main_four_eyes`, Standard an) und, wenn der Mandant es aktiviert, **Regressionsprüfung** (Goldset, 9.7)|
|`retire(branch)`|Branch freigeben; nicht mehr erreichbare Objekte werden nach `branch_retention` (Standard 90 Tage, anpinnbar) bereinigt|

Jeder Branch ist an eine **Index-Generation** gebunden (`retrieval_cfg`, `chunk_cfg`, Embedder); ändert ein Commit diese Konfiguration, entsteht beim `merge` nach `main` ein Indexaufbau im Hintergrund, und `main` zeigt erst nach dessen Abschluss auf die neue Generation (11.4).

### 6.5 Sichtbarkeitsmengen und versionsbewusste Suche [E]

Jeder Wissens-Blob erhält eine `KOrd`. Je Commit gibt es eine **Sichtbarkeitsmenge** `V(commit)` (Roaring-Treemap der `KOrd` im Tree), gespeichert als Basismenge plus Deltas zum Elterncommit (Schnappschuss alle 256 Commits). Die Suche arbeitet auf **globalen, unveränderlichen DK-Indexsegmenten** des Mandanten und filtert die Kandidaten mit `V ∩ A` (9.2). So kostet ein Branch oder Zeitpunkt keinen eigenen Index. Dokumente der Ebene P sind **nicht** Teil von Sichtbarkeitsmengen; für sie gelten Zugriff und Gate (9.4).

### 6.6 Zeitreise über Personendaten (Ebene P) [E]

Ebene P hat **keine** allgemeine Zeitreise. Sie führt je Dokument höchstens kurze `Superseded`-Versionen für den Nachweis der Berichtigung (Art. 16) und für Aufbewahrungsfristen, begrenzt durch `history_horizon` (`Personal`: 30 Tage; `Besonders`: 0). `Superseded`-Versionen sind für **alle** Lesepfade gesperrt (Z1) und nur über `restricted_read` lesbar; danach werden sie bereinigt. Superseded-Versionen aus **Löschvorgängen** (`RedactSplit`, `RestrictShared`) werden sofort bereinigt, nicht erst nach `history_horizon` (8.9). Abfragen mit `as_of` auf P prüfen **zuerst** das Gate; ein gelöschtes Ziel liefert `Erased`, nie Klartext. **Begründung:** Eine unbegrenzte Historie personenbezogener Inhalte widerspricht dem Zweck der Löschung (Art. 5 Abs. 1 lit. d, e); Nachvollziehbarkeit gelingt über die Begründungskette in K (7).

### 6.7 Garbage Collection, Fristlöschung und Takedown in K [E]

**Erreichbarkeit.** Objekte, die von keinem Ref und keinem aufbewahrten Entscheidungsprotokoll erreichbar sind, werden nach `branch_retention` entfernt. Ein Entscheidungsprotokoll hält die Objekte nur bis zum Ende seiner `decision_retention` (10.3 → 12.5); danach ist es selbst Löschkandidat. Pack-Segmente werden ohne unerreichbare Blobs neu geschrieben (neuer `SegmentKey`, alter wird vernichtet). Die Hash-Kette bleibt unberührt, weil sie nur Commits, Refs und Vermerke enthält.

**Fristlöschung (`Expired`).** Nach `decision_retention` entsteht ein Vermerk `Expired { id }` im K-Ledger; der Eintrag wird nach der `ArchiveHook`-Prüfung (8.4) aus dem Entscheidungsspeicher entfernt (Ersatzsegment, `SegmentKey` vernichtet). Kein Vier-Augen-Schritt.

**Takedown.** Wird in K irrtümlich Personenbezug festgestellt, besteht der Wiederholungstest von `Split` nicht, oder ordnet ein Gericht die Entfernung an:

1. Vier-Augen-Freigabe, kryptografisch signiert (Rollen `DataProtection` und `Legal` oder `TenantAdmin`).
2. **Sofort** `RedactK(KOrd)` im Gate-Log (Sperre analog Z1) und Aufnahme in `RedactedK`; alle Lesepfade liefern den Platzhalter `Redacted(ObjectId)`.
3. DK-Segmente mit `KOrd` im `source_set` werden ersetzt (neuer `SegmentKey`, alter vernichtet); Pack-Segment ohne den Blob **inklusive Salz** neu schreiben, `SegmentKey` vernichten, Rotation anfordern (4.3).
4. Im K-Ledger Vermerk `Redacted { object_id, grund_code }`; dessen Commitment im E-Ledger und in der Registersenke.

Die Objekt-ID und die Kette bleiben erhalten, der Inhalt nicht. Prüfer behandeln eine ID mit `Redacted`-Vermerk als undurchsichtig und prüfen nur die Kette. Nach Wiederherstellung wird jeder `Redacted`-Vermerk erneut angewendet (8.8). **Restrisiko [A] R-OBJID:** Wer `K_dedup` und den vermuteten Inhalt besitzt, kann die Objekt-ID dennoch testen.

---

## 7. Entscheidungsprotokoll, Szenarien, Akteure, Validator

### 7.1 Entscheidungsprotokoll [E]

Zu jeder Antwort, jedem Szenariolauf, jeder Prüfung und jedem Werkzeuglauf entsteht ein Eintrag der Klasse `Beleg`. Er liegt im **Entscheidungsspeicher** (K-Partition, `SegmentSealed`), **nicht** im Commit-DAG; das K-Ledger erhält nur `DecisionAppended { id, record_hash }`.

```rust
pub struct DecisionRecord {
    pub id: [u8; 16],                // zufällig
    pub kind: DecisionKind,          // Query | ScenarioRun | Review | Release | Tool
    pub commit: ObjectId,            // Wissensstand
    pub valid_at: Option<Date>,
    pub index_gen: IndexGen,
    pub prompt: ObjectId,            // Prompt-Version (Blob in K)
    pub retrieval_cfg: ObjectId,
    pub chunk_cfg: ObjectId,         // macht Chunk-Nummern eindeutig
    pub models: ModelSet,            // ModelFingerprint je Rolle: Embedder, Reranker, Generator, Verifier (inkl. Parameter, Seed)
    pub view_class: [u8; 16],        // Hash der Sichtklasse (Replay-Determinismus)
    pub input: PayloadRef,           // die Frage
    pub query_rewrite: Option<PayloadRef>,
    pub evidence: Vec<EvidenceRef>,
    pub answer: PayloadRef,
    pub abstained: bool,
    pub reviewer: Option<ActorRef>,  // Pflicht bei kind = Review
    pub actor: ActorRef,             // Handle, löschbar
    pub scenario: Option<[u8; 16]>,
    pub linked: Option<[u8; 16]>,    // Review → zugehörige Query
    pub retention: RetentionRef,     // decision_retention, payload_retention
    pub prev_hash: [u8; 32],         // K-Ledger-Verkettung
}
pub enum EvidenceRef { Knowledge { blob: ObjectId, chunk: u32 }, Personal(PersonalRef) }
pub enum PayloadRef  { Open(ObjectId), Sealed(PersonalRef) }
```

- **Standard ist `Sealed`.** Fragen enthalten oft Personenbezug (Akten, Namen), den ein Mustervalidator übersehen kann. `Open` ist nur zulässig, wenn die Richtlinie `open_decisions` des Mandanten aktiv ist, **alle** Belege `Knowledge` sind und Eingabe wie Antwort den N-Validator bestehen.
- **Betroffene einer `Sealed`-Nutzlast** (bestimmen ihren DEK, 4.2) sind: Betroffene der Belege ∪ Betroffene des Vorgangs (API-Parameter `subject_refs`/`case_id`) ∪ in der Eingabe erkannte und über das Verzeichnis auflösbare Personen. Wird Personenbezug erkannt, der sich nicht auflösen lässt, gilt `payload_retention` als harte Obergrenze; `payload_retention` ist je Entscheidungsart **Pflichtparameter ohne Standardwert**, unabhängig von `decision_retention`. Nach Ablauf wird der Schlüssel der Nutzlast vernichtet; die Begründungskette (Stand, Prompt, Modell, Belege) bleibt.
- **Der Akteur ist nicht Teil der Schlüsselableitung:** Das Ausscheiden einer Mitarbeiterin löscht ihre Identität (`Actor`, 8.1), nicht die Begründungskette fremder Vorgänge (Art. 17 Abs. 3 lit. b, e DSGVO).
- Nach einer Löschung bleibt die Begründungskette **vollständig sichtbar**: Wissensstand, Prompt, Modell, Wissensbelege. Eingaben und Antworten mit Personenbezug lösen zu `Erased` auf.
- Bei `kind = Review` ist `reviewer` Pflicht: Ohne menschliche Freigabe gilt eine Antwort nicht als Entscheidungsgrundlage (Art. 22 DSGVO; Art. 14 KI-VO bei Hochrisiko-Einsatz, 20.6) [R][V]. `Release` protokolliert die Freigabe eines Entwurfs oder Merge nach `main`. `Tool` protokolliert deterministische Werkzeuge (10): Werkzeugkennung, Version, Eingabe (`Sealed` bei Personenbezug), Ergebnis.
- **Ereignisprotokoll, kein Klartextprotokoll aller Ein- und Ausgaben:** Der Datensatz ist auf die genannten Felder begrenzt.

### 7.2 Nachvollziehen einer alten Entscheidung [E]

`explain(decision_id)` liefert: Stand der Wissensebene (Commit, `valid_at`), Prompt, Modell und Parameter, jeden Beleg (soweit nicht gelöscht und nicht durch Zugriffsrechte verborgen), die gespeicherte Antwort. Optional `replay(decision_id)`: erneuter Lauf mit denselben Parametern, derselben `IndexGen` und derselben `view_class`. Ist die `IndexGen` außer Dienst (11.4), antwortet `replay` mit `GenerationRetired`, `explain` bleibt vollständig. **Grenze (A-INFER):** Byte-identische Wiederholung ist nur möglich, wenn das Inferenzsystem deterministische Dekodierung unterstützt; die **gespeicherte Antwort** ist der Beleg, der Wiederholungslauf ein Vergleich. Der Bericht weist aus, ob der Wiederholungslauf identisch war. Der Zugriff auf `explain` unterliegt den Rechten des fragenden Nutzers zum Zeitpunkt der Abfrage (nicht denen des damaligen Nutzers).

### 7.3 Szenarien („Wenn-Dann“) [E]

Ein **Szenario** besteht aus: Basis-Commit, Szenario-Branch mit Änderungen (geänderte Vorschrift, anderer Schwellenwert, anderer Prompt, anderes Modell, andere Abrufkonfiguration), einem **Fragenset** (Blob in K, N-Validator-geprüft, ohne Personenbezug) und den Ergebnissen. Ablauf: (1) Branch aus Basis-Commit, (2) Änderungen als Commits, (3) Fragenset gegen Basis und Szenario laufen lassen (`ScenarioRun`), (4) Vergleich: Unterschiede in Belegen, Antworten, Enthaltungen, Metriken, (5) bei Bedarf `merge` oder `retire`. Z9 gilt: Ein Szenario schreibt nie in `main` und nie in Ebene P, verändert keine Zugriffsrechte. Personendaten dürfen in einem Szenario nur gelesen werden, wenn Zugriffsrecht, Vorgangsbindung und Gate es erlauben; Ergebnisse mit Personenbezug werden `Sealed`. `ScenarioRun`-Protokolle folgen der Aufbewahrung des Szenario-Branches. Das Szenario nutzt Index-Generationen des Basis-Standes; ändert es Chunking oder Embedder, baut es einen **temporären** Index (`szenario/<id>`), der mit `retire` entfällt.

### 7.4 Akteure und Mitarbeitende [E]

Nutzer sind selbst Betroffene (`subject_kind = Staff`). Anmeldung: `oidc:<iss>|<sub>` → `SubjectTag` → `SubjectKey` → Verzeichnis → aktuelles `ActorRef`-Handle der Epoche (`actor_handle_epoch`, Standard 12 Monate; neue Commits und Protokolle erhalten das neue Handle, die Zuordnung liegt in P). Eine Löschung oder Auskunft über Mitarbeitende folgt denselben Mechanismen wie jede Betroffenenanfrage; die Begründungskette fremder Vorgänge bleibt. **[A]** Personalvertretung und interne Regeln zur Auswertung von Nutzungsdaten (A-Ledger) liegen beim Betreiber.

### 7.5 N-Validator (Schutz der Ebene K) [E]

Vor jedem Schreiben in K prüft der Validator: (1) **Schema-Whitelist** für strukturierte Felder, (2) **Mustererkennung** (Namen, Anschriften, Kennnummern, E-Mail, Telefon, Kontonummern, Aktenzeichen mit Personenbezug) über `acta-pii`, (3) **Entropie- und Handle-Prüfung** für Kennungen, (4) bei Aggregaten **k-Anonymität** mit `K_MIN` ≥ 5 (vom Mandanten erhöhbar). Ein Treffer über der Schwelle blockiert das Schreiben (Fail-Closed); der Fehler nennt nur Befundcodes und Zeichenbereiche, nie Inhalte. **Typzwang:** `acta-history` nimmt nur `Validated<Tree>` entgegen; `Validated` ist ausschließlich von `acta-pii` konstruierbar. **Falschpositive:** Ein Befund DARF nur durch einen dokumentierten `Waiver` übergangen werden (zwei verschiedene Personen, davon `DataProtection`; Erklärung „kein Personenbezug“; Eintrag `Waiver` im K-Ledger); echte Treffer können nicht freigegeben werden und gehören in `Whole`. **Grenze [A] A-DETECT:** Automatische Erkennung ist nicht vollständig; deshalb Standardmodus `Whole`, Vier-Augen-Freigabe für `Open` und `Split` und Takedown (6.7).
---

## 8. Löschkern

### 8.1 Zustände und Lesesperre [E]

|Zustand|Bedeutung|Lesen|Schreiben|Schlüsselvernichtung|
|---|---|---|---|---|
|`Live`|regulär|ja (Z16)|ja|nein|
|`Quarantine`|Klasse `Unklassifiziert`, höchstens `quarantine_max`|nein|nein|nur durch Ablauf|
|`Restricted(grund)`|eingeschränkt; Gründe: `Retention`, `LegalHold`, `Objection`, `Contested`, `MultiSubjectReview`, `Precautionary`|nur `restricted_read`|nein|**nein**|
|`Gated`|Löschvorgang läuft, Sperre durabel|**nein**|nein|ja (8.3)|
|`Superseded`|ältere Version eines Dokuments in P|nur `restricted_read`|nein|nach `history_horizon` bzw. sofort bei Löschgrund|
|`Pending`|Sperre aktiv, Fortsetzung wartet (Anker, Senke, Review, Archiv, Sicherungsfenster)|wie Ausgangszustand|nein|wartet|
|`Expired`|Frist abgelaufen, Löschvorgang ausgelöst|wie `Gated`|nein|ja|
|`Erased`|Endzustand nach Schlüsselvernichtung; Gate bleibt dauerhaft|**nein**|nein|erledigt|

**Lesepfadregel.** Jeder Lesepfad (Suche, Antwort, Zeitreise, Szenario, Cache-Treffer, Export, Vorschau, Sitzung, Streaming) ruft `check_ver(ord)` **vor** jeder Entschlüsselung und Ausgabe auf (O(1), keine E/A). Ein Nutzer **mit** Zugriffsrecht erhält `Erased`; ein Nutzer **ohne** Zugriffsrecht erhält dasselbe Fehlerbild wie bei einem nicht vorhandenen Ziel (Z16).

**`restricted_read`.** Die Rolle `DataProtection` darf `Restricted`- und `Superseded`-Inhalte für festgelegte Zwecke lesen: `Auskunft`, `Rechtsanspruch`, `Berichtigung`, `Review`, `Aufbewahrung`, `Aufsicht`. Jeder Zugriff ist zweckgebunden, wird mit `ActorRef` und Zweckcode im E-Ledger protokolliert (ohne Inhalte, Z15) und erreicht nie die RAG-Pipeline, nie Caches und nie Sitzungen. Erledigte Ziele (`Gated`, `Erased`) sind auch für `restricted_read` gesperrt.

**Mitarbeitende (`Actor`).** Das Ausscheiden löscht die Identität in Ebene P (Zuordnung `ActorRef` → Person durch Saga auf dem `Staff`-Betroffenen), nicht die Begründungskette fremder Vorgänge (7.1).

**Mandanten-Off-Boarding.** Tresor und Exportdateien des Mandanten werden entfernt (Scrub), seine Segmente bereinigt, seine Ledger nach Aufbewahrungsvorgabe archiviert oder vernichtet. Kopien Dritter werden mit ihrer Sicherungsepoche unlesbar; der Bericht weist `offboarding_exposure_until` aus (12.6).

### 8.2 Auslöser und Betroffenenanfragen [E][R]

**Auslöser:** Antrag (Art. 17 Abs. 1), Widerruf oder Widerspruch (Art. 21), Fristablauf der Aufbewahrungsklasse, Zweckfortfall, Takedown (6.7), Off-Boarding, Ausscheiden von Mitarbeitenden, Ablauf von `quarantine_purge_after`, `payload_retention`, `history_horizon`, gerichtliche Anordnung.

**Ablauf einer Anfrage.**

1. Eingang über API oder CLI durch eine berechtigte Rolle des Betreibers (`request.create`) mit einer oder mehreren externen Kennungen derselben Person.
2. **Identitätsprüfung** (Art. 12 Abs. 6) liegt beim Betreiber; Acta speichert nur das Ergebnis (`identity_verified`), keine Ausweisdaten. Ohne `identity_verified` laufen weder Auskunft noch Löschung; Einschränkung (`Restricted(Contested)`) ist vorab zulässig.
3. Auffinden: externe Kennung → `SubjectTag` → Tresor → `SubjectKey` → `ClusterId` → Verzeichnis (5.4). Mehrere Kennungen werden über Aliase vereinigt; ist eine Person nachträglich als zwei Betroffene geführt, wird die Anfrage mit allen Kennungen gestellt (Zusammenführung bestehender Betroffener ist nicht vorgesehen).
4. Plan und Trockenlauf (8.9).
5. Entscheidung durch Rolle gemäß 12.5.
6. Ausführung.
7. Antwort an die betroffene Person durch den Betreiber.

Acta führt je Anfrage `due_at` = Eingang + 1 Monat (Art. 12 Abs. 3; Verlängerung um bis zu 2 Monate mit Begründung ist ein Betreiberakt) und warnt ab 70 % der Frist (`RequestDue`).

**Auffinden unerkannter Datensätze.** `subjects.discover` (nur `DataProtection`, `restricted_read`) sucht in P und in K nach Name- und Kennungsmustern der Person und liefert Kandidaten (Handle, Fundstelle ohne Volltext). Die Entscheidung über `AddSubject` (P) bzw. Takedown (K) trifft der Review; es gibt keine automatische Ausweitung.

|Recht|Acta-Funktion|
|---|---|
|Auskunft, Art. 15|`restricted_read` auf Verzeichnis und Dokumente; Ausgabe: Datenkopie, Zwecke, Empfänger (`DisclosureLog`), Aufbewahrungsklasse, Herkunft; Ableitungen (Embeddings) werden nach Art und Existenz genannt, nicht als Vektoren; **Daten Mitbetroffener werden vor dem Schreiben des Exports serverseitig geschwärzt (Art. 15 Abs. 4)**; ist das nicht sicher möglich, Review|
|Berichtigung, Art. 16|neue Version (neue `VerRef`), alte `Superseded` (Berichtigungsnachweis bis `history_horizon`); Mitteilung nach Art. 19|
|Löschung, Art. 17|Saga 8.3|
|Einschränkung, Art. 18|`Restricted(Contested)` oder `Restricted(Objection)`; Ableitungen werden bereinigt|
|Mitteilung, Art. 19|Ausgangskorb 8.6|
|Datenübertragbarkeit, Art. 20|signierter JSON-Export der von der Person bereitgestellten Daten (nur Ebene P), Eintrag im Kopienregister|
|Widerspruch, Art. 21|`Restricted(Objection)` bis zur Prüfung; danach Saga oder Aufhebung (`Release`)|

Die Zuordnung zu den Pflichten des Betreibers steht in 20.5.

### 8.3 Löschsaga [E]

Die Saga ist ein dauerhafter, idempotenter Zustandsautomat je **Löschvorgang** (`ErasureId`) mit dem Umfang `Subject` (Betroffener), `Document` (Dokument mit allen Versionen), `Version`, `Expired` (Stapel fällig gewordener Ziele) oder `Takedown` (K, 6.7). Ein Löschvorgang enthält **Dokument-Aufgaben** (`DocTask`) je Version mit eigenem Modus (8.9). Jeder Schritt ist wiederholbar; **physische Tatsachen** (Schlüssel vernichtet, Segment ersetzt) haben beim Neustart Vorrang vor dem Journalstand (Z4).

|Schritt|Zustand nach Abschluss|Inhalt|Durabilitätspunkt|
|---|---|---|---|
|S0|`Requested`|Anfrage im E-Ledger (Commitment auf `SubjectTag` bzw. `VerRef`), `ErasureId` (UUIDv7)|Ledger-`fsync`|
|S1|`Gated`|**Trockenlauf** (8.9) über alle Versionen der Zielmenge einschließlich `Superseded`; Konfliktfreie Versionen: `EraseVer`; Konfliktversionen: `Restrict(MultiSubjectReview)`; wird der Trockenlauf nicht innerhalb `erase_return_timeout` fertig, werden **alle** Versionen der Zielmenge vorläufig `Restrict(Precautionary)` und der Lauf wiederholt; Sperre im RAM, Gate-Log mit Gruppen-`fsync`, Epoche erhöhen, Registersenke **asynchron**; danach Rückkehr an den Aufrufer|Gate-Log-`fsync`|
|S2|`Planned`|Messmuster und Canary-Bindung erzeugen (8.5), Ausgangskorb füllen (8.6), Kopienregister erfassen, Segment- und Record-Listen der Zielmenge ermitteln. Alle Arbeitsdaten unter `ScopeKey`|Arbeitsdatei-`fsync`|
|S3|`Purged`|Caches, Sitzungen, Antwortcache, DEK-Cache, Chunk-Cache, `ContextPurge`; DP-Ersatzsegmente schreiben (neuer `SegmentKey`, Manifest-Batch `AddSegment`/`RemoveSegment` atomar) oder Datei-Drop; P-Segmente ohne Zielumschläge neu schreiben oder ganze Segmente entfernen; Handle-Register-Einträge, Fallindex-Mitgliedschaften und Vorgänge entfernen; Journal flushen, rotieren, Altdatei mit Scrub entfernen; Tempdateien; Arbeitslisten stammen aus S2 (kein `SubjectKey` erforderlich)|Manifest-`fsync` je Batch|
|S4|`PlaintextMeasured`|Messung Klasse B (8.5)|Messprotokoll|
|S5|`KeyTombstoned`|**S5a:** `destroy_many` für `DocumentKey(v)` aller Zielversionen sowie `SubjectKey(s)` und `SubjectAlias`-Einträge, **sobald keine Version mehr an `s` gebunden und lesbar bleiben muss** (4.2). **S5b:** nach S3 `destroy_many` der ersetzten `SegmentKey`s. Grabstein, Direct-Read-Verifikation, **Canary-Probe**|Tresor-`fsync`|
|S6|`Rotated`|Rotation auf Anforderung (4.3), `DestroyProof`, `RootRotated`|Ledger-`fsync`|
|S7|`BackupsExpired`|wartet bis `backup_exposure_until`, dann `destroy_backup_epoch` je Epoche (zeitgesteuert)|Anker-Nachweis|
|S8|`Measured`|Messung Klasse K und M|Messprotokoll|
|S9|`Reported`|Bericht (18.1), E-Ledger, Senke bestätigt; `ScopeKey` vernichtet (bei offenem Ausgangskorb erst nach dessen Abschluss)|Ledger-`fsync`|

**Reihenfolge der Schlüsselvernichtung [E].**
- *Standardpfad:* S1, S2, S3, S4, S5, … Solange der Zielschlüssel lebt, lässt sich eine unvollständige Bereinigung nachholen (Wiederableitung von Messmustern, Reparatur von Ersatzsegmenten), und eine Fehlbedienung ist bis S5 durch den Review rücknehmbar (nur vor S1 durch Abbruch; nach S1 ist die Sperre dauerhaft, außer `Release` aus `Restricted`).
- *Vorgezogener Pfad* (Klasse `Besonders` oder wenn `erase_sla_B` zu 70 % verbraucht ist): S1, S2, **S5a**, S3, S4, S5b. S3 arbeitet mit den Listen aus S2 und braucht keine Zielschlüssel. Die `SegmentKey`s gemischter Segmente bleiben bis zu ihrem Ersatz am Leben (S5b), sonst gingen Daten Dritter verloren. Die Messmuster (S2) wurden vor der Vernichtung abgeleitet; Messung B (S4) läuft nach S5a mit diesen Mustern.

**Teilläufe (Tranchen).** Ein Löschvorgang mit Konfliktversionen läuft in Tranchen: Tranche 1 = konfliktfreie Versionen (S1 bis S5 sofort), Tranche 2 = Versionen nach Review-Entscheidung (S1 bis S5 je Entscheidung). `SubjectKey(s)` wird mit der letzten Tranche vernichtet. Bis dahin steht der Löschvorgang als `Pending(Review)` mit aktiver Sperre.

**Nebenläufigkeit.** Ein Ausführer je Mandant (Einzelschreiber, 3.6); Leser laufen parallel gegen die Gate-Epoche. Sagas mit überlappenden Zielen werden nach `ErasureId` serialisiert; der zweite Plan wird nach dem ersten neu berechnet. S6 fängt Anforderungen mehrerer Sagas in einer Rotation (Mindestabstand `rotation_min_gap`). Rotation (4.3) und S5 stimmen sich über `vault_freeze` ab.

**Fehlerverhalten.** Transiente Fehler: Wiederholung mit Backoff (1 s bis 1 h, `saga_retry_max` = 8), Zustand bleibt. Nicht behebbar: `Pending` mit Alarm (`SagaStuck`). `Integrity`/`Crypto`: Vergiftung des kleinsten nachweislich isolierenden Bereichs (15.2). Ein Fehler in S5 nach teilweiser Vernichtung wird beim Start anhand des Tresorzustands zu Ende geführt (Vernichtung ist idempotent).

### 8.4 Aufbewahrung, Einschränkung, Legal Hold, Archiv-Hook [E][R]

- **Aufbewahrungsklasse** (12.2) liefert `retain_until` je Datum (nicht je Person; OLG Dresden, 14.12.2021, 4 U 1278/21 [V]). Ein Löschantrag während der Frist führt zu `Restricted(Retention)`: Ableitungen (Embeddings, Indexeinträge) werden bereinigt (Datenminimierung; bei Aufhebung Neuaufbau), der Datensatz bleibt unter seinem Schlüssel. Zum Fristende läuft die Saga automatisch (`Besonders`: Karenz 0 Tage).
- **`legal_hold`** ist ein Kennzeichen je Dokument, Betroffenem oder Mandant (Art. 17 Abs. 3 lit. b, e), gesetzt durch die Rolle `Legal` mit Vier-Augen-Freigabe, Grundcode und Wiedervorlagefrist `hold_review_interval`. Es blockiert S5.
- **Original und Repräsentation.** Das Original liegt im Quellsystem (`source_of_record`, 9.1). Die Aufbewahrungspflicht für das Original rechtfertigt nicht automatisch die Weiternutzung im aktiven Retrieval **[⚠]**: Acta isoliert die Repräsentation (Gate, Ableitungen bereinigt) und lässt das Original unberührt.
- **`ArchiveHook`** (Kantenport): Vor Frist- und Takedown-Löschung eines Objekts, für das Acta selbst Quelle ist (Entscheidungsprotokolle, `source_of_record = acta` nur mit schriftlicher Betreiberentscheidung und DSFA), wird `offer` aufgerufen. Fehlt eine Entscheidung nach `archive_wait_max` (30 Tage) oder schlägt der Hook fehl, bleibt das Objekt `Restricted(Retention)`. Anbietungspflichten gegenüber Archiven folgen aus Bundes- oder Landesarchivrecht [R][V] und sind im Löschkonzept (12.4) abzubilden.

```rust
pub trait ArchiveHook: Send + Sync {
    fn offer(&self, tenant: TenantId, class: ObjectClass, due: Date) -> Result<ArchiveOutcome, HookError>;
}
pub enum ArchiveOutcome { Accepted, Declined, NotRequired }
```

### 8.5 Messung und Positivkontrolle [E] (Z11)

Jede Messung läuft zunächst gegen einen **Canary**: einen synthetischen Betroffenen (`subject_kind = Synthetic`) mit künstlichem Dokument im selben Mandanten, der durch dieselben Pfade (Ingest, Index, Cache, Cluster-Segment) erzeugt wurde. Canaries sind für alle Lesepfade und Berichte unsichtbar (nur Zähler `synthetic_subjects`). Trifft das Messverfahren den Canary nicht, ist der Lauf `Invalid`; nach `measure_retries` = 3 Fehlschlägen entsteht `Pending` mit Alarm `MeasureInvalid`. Messmuster (bis zu 256 Fenster-Prüfwerte `HMAC(ScopeKey, 32-Byte-Fenster)` aus dem Ziel) entstehen in S2 und leben nur unter `ScopeKey`.

|Schicht|Verfahren|Art|
|---|---|---|
|Gate (alle Lesepfade)|Sondierungslesung je Pfad liefert `Erased`; Positivkontrolle: gleiche Sondierung am Canary vor dessen Sperre trifft|vollständig|
|DP-Segmente, Indizes|Manifest-Aufzählung: kein Segment mit `source_set` ∩ Ziel ≠ ∅|vollständig|
|Caches, Sitzungen, Antwort-, DEK-, Chunk-Cache|Aufzählung und Messmuster-Scan|vollständig|
|P-Segmente, Journal|Suche nach `cluster_id` und `ver_ref` der Zielmenge in allen Dateien (Aufzählung der Schlüsselräume)|vollständig (Klasse M)|
|Tempdateien, Arbeitsverzeichnisse|Messmuster-Scan|Stichprobe, ausgewiesen|
|Tresor|Grabstein, Direct-Read; **Canary-Probe:** vor S5 wird ein Umschlag unter dem echten Zielschlüssel erzeugt, nach S5 muss `open` mit `Erased` oder `DecryptionFailed` scheitern|vollständig|
|Anker, Sicherungsepochen|`DestroyProof`, `destroy_backup_epoch`|Beleg|
|Inferenzdienst|`ContextPurge`-Bestätigung des Dienstes|`not_measured` (A-INFER)|
|RAM, Swap|nicht messbar|deklariert (A-RAM)|
|Flash-Zellen|nicht messbar|deklariert (A-FLASH)|
|Externe Kopien|nicht messbar|deklariert (A-EXT)|

Nicht implementierte Messungen liefern `NotImplemented`, nie `reached`.

### 8.6 Nachlauf: Art. 19, Kopienregister [E][R]

- **Ausgangskorb (`Outbox`).** In S2 liest Acta das `DisclosureLog` und legt je Empfänger eine Mitteilung ab (Empfängerhandle, Art der Mitteilung `Erasure|Rectification|Restriction`, externe Kennung der Person für das Fachverfahren des Betreibers; versiegelt unter `ScopeKey`, keyspace `0x0060`). Der Betreiber stellt über den Kantenport `Notifier` zu und bestätigt. Abgeschlossen ist ein Eintrag mit `Delivered`, `Impossible` (Begründung) oder `Expired` nach `outbox_ttl` = 90 Tage; erst dann wird `ScopeKey` vernichtet. Acta veröffentlicht keine Daten und löst Art. 17 Abs. 2 nur als Hinweis an das Fachverfahren aus. Restrisiko R-OUTBOX: Die externe Kennung bleibt bis zum Abschluss des Korbs unter `ScopeKey` erhalten.
- **Kopienregister.** Je Mandant führt Acta ein Register aller Kopien außerhalb der Primärspeicher: Tresorexporte, Datensicherungen, Art.-15/20-Exporte, Berichte, Supportpakete, externe Inferenzfreigaben. Eintrag: Art, Anzahl, Ort (Bezeichnung des Betreibers), Ablauf. Das Register enthält keine personenbezogenen Inhalte.

```rust
pub trait Notifier: Send + Sync {
    fn deliver(&self, tenant: TenantId, m: &OutboxMessage) -> Result<DeliveryAck, NotifyError>;
}
```

### 8.7 Erfüllungsklassen [E]

Klassen bauen aufeinander auf und werden über die **Zielmenge** des Löschvorgangs (alle Versionen mit Modus `DeleteAll` oder Löschgrund) bewertet. Der Bericht nennt je Klasse `reached` oder `not_reached` mit Zeitpunkt und Annahmen (18.1). Das Wort „gelöscht“ steht nur mit der erreichten Klasse (1.6).

|Klasse|Name|`reached`, wenn|
|---|---|---|
|**G**|gesperrt|Gate durabel, alle Lesepfad-Sondierungen liefern `Erased`, Positivkontrolle bestanden|
|**B**|bereinigt|S3 vollzogen; Messung Klasse B (Caches, Sitzungen, DP-Segmente, Tempdaten) mit Positivkontrolle|
|**K**|Schlüssel vernichtet|S5 und S6 vollzogen, Canary-Probe wie erwartet gescheitert, `DestroyProof` des Ankers **und alle betroffenen Sicherungsepochen vernichtet** (`backup_exposure_until` erreicht)|
|**M**|vollständig gemessen|Klasse K, P-Segment- und Journalmessung ohne Zielschlüsselräume, Registersenke bestätigt (`registry_mirrored`), alle Schichten gemessen oder deklariert|

Zwischenstände: `key_tombstoned` (Grabstein, Rotation offen) und `K_pending_backups` (Rotation erfolgt, Sicherungsfenster offen). `Restricted` wird als „eingeschränkt, nicht gelöscht“ ausgewiesen; der Bericht trennt Zielmenge und zurückgestellte Versionen (`scope_result ∈ {complete, partial_restricted, pending}`). Klasse M ist eine technische Aussage innerhalb der Annahmen, **keine** rechtliche Feststellung der Löschung nach Art. 17 (20.2).

**Erwartungswerte.** Klasse G sofort, Klasse B innerhalb `erase_sla_B` (24 h), Klasse K nach `max(T_ROT, anchor_destroy_max)` und dem Sicherungsfenster (`backup_exposure_until` ≤ Tombstone + `T_BACKUP` + `backup_epoch`, höchstens 36 Tage). Dass Sicherungen nicht bei jedem Antrag punktuell verändert werden müssen, solange nach einer Wiederherstellung erneut gelöscht wird, entspricht der Auffassung der Aufsicht (SDM-Baustein 60, EDPB-CEF 2026; Fristen ungeregelt) [V].

### 8.8 Restore-Schutz [E] (Z12)

Eine Wiederherstellung durchläuft den Modus `Restoring` (alle Lesepfade geschlossen):

1. **Voraussetzung:** Registersenke erreichbar **oder** Ankerstand (`last_head`) für Gate-Log und beide Ledger (K, E) vorhanden. Fehlt beides: `Quarantine`.
2. **Tresor** nur aus einem gültigen Export innerhalb des Restore-Horizonts (`T_BACKUP + backup_epoch`); ältere Sicherungen werden mit `RestoreBeyondHorizon` verweigert. Datensicherungen (Segmente, Journal) ohne gültigen Export sind unlesbar. Dokumente ohne Schlüssel im Export (nach dem letzten Export entstanden) werden gezählt (`unrecoverable_docs`) und im Wiederherstellungsbericht ausgewiesen.
3. **Ordinale:** alle Zähler auf `max(Stand der Sicherung, Hochwassermarke der Senke) + 1` (2.4).
4. **Senke lesen:** Gate-Log und E-Ledger holen; Kette, signierte Checkpoints, Kopf ≥ Ankerkopf prüfen, sonst `RestoreBehindAnchor`. **Gate-Log und E-Ledger werden aus der Senke vorwärts ergänzt.** Ein zurückgerolltes **K-Ledger** bedeutet verlorene Commits: der Knoten bleibt für diesen Mandanten in `Quarantine`, bis `DataProtection` und `TenantAdmin` den Verlust mit Vier-Augen bestätigen (`accept_rollback`, Eintrag im E-Ledger).
5. **Gate-Mengen** = Vereinigung aus Sicherungsstand und Senke.
6. **Wiedervernichtung:** Für jeden wiederhergestellten `SubjectKey` und `DocumentKey` wird geprüft, ob (a) die `SubjOrd`/`VerOrd` im Gate steht (Verzeichnis oder Dokumentkopf lesbar, solange der Export den Schlüssel noch lebend enthält) oder (b) `commit(e, KeyId)` für eine Epoche im Horizont im E-Ledger vorkommt (alle `CommitKey(e)` im Horizont sind im Export enthalten). Treffer: Grabstein, Saga ab S3 wiederholen. **Ferner:** jedes wiederhergestellte Segment mit `source_set ∩ Gate ≠ ∅` wird ersetzt oder entfernt (betrifft DP-Segmente bei `backup_derived_dp = true` und P-Segmente).
7. **Takedown:** jeder `Redacted`-Vermerk wird erneut angewendet (6.7).
8. **Offene Sagas** der Senke (Status ≠ `Reported`) laufen weiter.
9. **Ableitungen:** DP-Indizes werden nach Wiederherstellung neu aufgebaut (nur für Versionen `Live` und nicht im Gate); DK folgt der Sicherung.
10. Erst danach `Live`; Ereignis `Restored` im E-Ledger (Re-Witness).

Die Zusage ist **bedingt** (Z12): Sie hängt an der Verfügbarkeit der Senke oder des Ankerstands und an R-HORIZON.

### 8.9 Mehrbetroffene Datensätze [E] (Z3)

**Trockenlauf.** Für jede Version `d` der Zielperson `s` berechnet Acta die Mitbetroffenen `C = subjects(d) \ {s}` (Dokumentkopf) und prüft je `c ∈ C`: (1) Aufbewahrungsklasse mit `retain_until` in der Zukunft, (2) `legal_hold`, (3) registrierter Widerspruch oder offener Streit, (4) offener Vorgang (Status `open` mit Zuweisung im Fachverfahren). Kein Befund: Modus `DeleteAll`. Mindestens ein Befund oder Sammelakte: Review.

|Modus|Wirkung|Wann|
|---|---|---|
|`DeleteAll` (Standard)|`SubjectKey(s)` und `DocumentKey(v)` werden vernichtet; die Version ist für **alle** unlesbar|kein Konflikt, Mitbetroffene vertretbar|
|`RedactSplit`|Neue Version (neue `VerRef`, neue Betroffenenliste ohne `s`, Inhalt mit geschwärzten Spannen von `s`); die alte Version wird sofort als Löschgrund-`Superseded` behandelt und durchläuft S1 bis S5|Dokument lässt sich sinnvoll schwärzen; **Pflicht für Sammelakten**|
|`RestrictShared`|Neue Version **ohne Bindung an `s`** (DEK aus `DocumentKey` und den Schlüsseln der Mitbetroffenen), Zustand `Restricted(grund)`, Ableitungen nicht aufgebaut; die alte Version durchläuft S1 bis S5. Die Daten von `s` bleiben nur im Rahmen der Aufbewahrungspflicht oder des Holds eines Mitbetroffenen erhalten; Bewertung wiederholt sich bei Wegfall des Konflikts (dann `DeleteAll` oder `RedactSplit`)|Aufbewahrungspflicht oder Hold eines Mitbetroffenen|

**Review** (`DataProtection` plus Fachbereich, Vier-Augen) entscheidet innerhalb `multi_review_max` = 10 Werktage; bis dahin `Restricted(MultiSubjectReview)`. Es gibt kein automatisches `DeleteAll` nach Fristablauf; bei Überschreitung eskaliert `ReviewOverdue` an die Datenschutzfunktion, das Dokument bleibt eingeschränkt. Die Schwärzung für `RedactSplit` erfolgt im Review-Werkzeug (`review.redact`): `DataProtection` markiert die Spannen von `s`, `acta-pii` schlägt Spannen vor, die Prüfung auf Restvorkommen läuft vor dem Schreiben der neuen Version; scheitert sie, bleibt das Dokument `Restricted`. Der Bericht nennt die **Anzahl** der Mitbetroffenen je Dokument, nie deren Kennungen. Gleichzeitige Anträge mehrerer Betroffener desselben Dokuments werden nach `ErasureId` serialisiert; der Plan wird je Lauf neu berechnet. Ein nachträglich erkannter Mitbetroffener löst `AddSubject` aus (2.3); bis dahin gilt Z3 für ihn nicht und der Bericht weist das aus.
---

## 9. Ingest, Abruf und Antwort

### 9.1 Ingest [E]

|Stufe|Inhalt|
|---|---|
|1 Annahme|API prüft Pflichtfelder: Mandant, Sammlung, **`source_of_record`** (Quellsystem plus Referenz; ohne diese Angabe verweigert die API den Ingest), Datenklassendeklaration (R-CLS), bei `Personal`/`Besonders` die Betroffenenzuordnung (`subject_refs`, 2.3), Aufbewahrungsklasse, Sichtbarkeitsmetadaten, `ingest_id` (Zufallswert des Aufrufers für Idempotenz). Fehlt etwas: `ClassViolation` oder `Invalid`, nichts wird geschrieben|
|2 Parsing|Nur im Prozess `acta-parse-worker`: keine Schlüssel, kein Netz, kein Dateisystem (13.2). Protokoll: längenpräfixierte Rahmen (u32 big-endian, höchstens 64 MiB). Ein Worker verarbeitet genau ein Dokument und endet|
|3 Normalisierung|Unicode NFC, Entfernen von Steuerzeichen, Spracherkennung (DE, EN, sonst `und`)|
|4 Erkennung|`acta-pii` liefert Hinweise und Klassenvorschlag; Standardmodus `Whole` (2.3); bei Personenbezug ohne Zuordnung: `Unklassifiziert`|
|5 Zuordnung|`subject_refs` → `SubjectTag` → Tresor; unbekannte Betroffene werden angelegt (Schlüssel zuerst, WP-1)|
|6 Chunking|deterministisch aus Inhalt und versionierter `chunk_cfg` (9.1.1); Chunk-Nummern sind stabil|
|7 Embedding|Port `Embedder` (Art `Passage`); `ModelFingerprint` wird mit der Ableitung gespeichert; Vektoren `i8`-quantisiert (9.3)|
|8 Schreiben|Reihenfolge: Schlüssel durabel (WP-1) → P-Umschläge → DP-/DK-Segmente mit `Derived<T>` → **zuletzt** der K-Commit bzw. das Manifest, das die Sichtbarkeit herstellt (WP-3)|

**Ressourcenlimits (Startwerte, Anhang A):** Eingabe 512 MiB, CPU-Zeit 60 s, Speicher 2 GiB, Expansionsfaktor 100:1, Verschachtelungstiefe 8, 10 000 Seiten. Verletzung ergibt `Invalid(ResourceLimit)`; der Worker-Absturz bleibt auf den Aufruf beschränkt (T6).

**Atomarität und Idempotenz.** Der Ingest ist ein Journal-Batch. Ein Absturz vor dem Sichtbarkeitsschritt hinterlässt nur unsichtbare Segmente, die der Start verwirft (verwaiste Dateien werden bereinigt, die zugehörigen Schlüssel vernichtet). Wiederholung mit gleicher `ingest_id` liefert dasselbe Ergebnis (Idempotenztabelle 5.4); in K ist der Ingest zusätzlich durch `ObjectId` idempotent (6.1).

**Aktualisierung.** Wissensdokumente (K): neuer Commit mit neuem Blob, alter Blob bleibt versioniert. P-Dokumente: neue Version (Berichtigung, 8.2), Ableitungen der alten Version werden bereinigt (5.2 Nr. 6). **Entfernung eines Quelldokuments** durch das Quellsystem: `document.retire` setzt K-Blobs im neuen Commit außer Kraft (bleibt abfragbar per Zeitreise) bzw. startet die Saga für P-Versionen mit Grund `SourceRemoved`.

#### 9.1.1 Chunking [E]

Ein Chunk ist eine zusammenhängende, strukturbewusste Textspanne. Einheit sind Wörter nach UAX #29. `chunk_cfg` (versionierter Blob) enthält und legt für Version 1 fest:

|Parameter|Wert|
|---|---|
|Zielgröße / Maximum / Minimum|300 / 450 / 60 Wörter|
|Überlappung (nur bei unstrukturiertem Text)|40 Wörter|
|Grenzen (Priorität)|Überschrift > Paragraph/§/Absatz einer Norm > Listenelement-Gruppe > Satzgrenze > Wortgrenze|
|Tabellen|zeilenweise Gruppen bis Maximum, Kopfzeile wird in jede Gruppe wiederholt|
|`heading_path`|Pfad der Überschriften (höchstens 6 Ebenen) als eigenes Feld; wird dem Chunk für Embedding und BM25 **vorangestellt**, aber nicht im Belegtext doppelt angezeigt|
|`norm_ref`|erkannte Normenkennung (z. B. „§ 35 Abs. 2“), als Feld für Zitate|
|`section_id`|Kennung des Abschnitts für Nachbar-Erweiterung (9.4)|

Chunking ist eine reine Funktion `(Bytes, chunk_cfg) → Vec<Chunk>`; gleiche Eingabe liefert gleiche Chunks (Golden-Tests). Die Chunk-Kennung ist `(ObjectId bzw. VerRef, chunk u32)`. Eine Änderung von `chunk_cfg` erzeugt eine neue `IndexGen` (11.4).

### 9.2 Textsuche [E] (Z7, Z16)

- **Normalisierung:** Unicode-Casefolding, Wortgrenzen nach UAX #29, Stoppwörter und leichter Stemmer für DE und EN; die Variante samt **Hash der Unicode-Tabellen und Stemmer-Version** ist Teil der `retrieval_cfg` (versioniert); ein Bibliotheks-Update, das die Tabellen ändert, erzeugt eine neue `IndexGen`.
- **Ranking:** BM25 mit `k1 = 1,2`, `b = 0,75` in **Festkomma Q32.32** (Rundung half-even). `idf(t) = ln(1 + (N − df + 0,5)/(df + 0,5))` über `fx_ln`: Zerlegung `x = m·2^e` mit `m ∈ [1,2)`, `ln x = e·LN2 + 2·atanh((m−1)/(m+1))` als Ganzzahlreihe mit 12 Termen; Golden-Vektoren sind Teil der Tests (`T-TT-FX`). Kein `f32`/`f64` in Ranking-Pfaden.
- **Statistik:** `N = |V ∩ A|`, `df(t) = |Postings(t) ∩ V ∩ A|`, `avgdl = Σ dl / N` mit `V` = Sichtbarkeitsmenge des Standes (6.5) und `A` = **effektive Zugriffsmenge der Sichtklasse** des Nutzers (9.4). Die Längensumme je (Commit, Sichtklasse) liegt in einem DK-Cache und wird bei Änderung der Zellen-Bitmaps der Sichtklasse neu berechnet. Damit hängt keine Rangfolge von Dokumenten ab, auf die der Nutzer keinen Zugriff hat (Z16).
- **Gleichstand:** (Score absteigend, `KOrd` aufsteigend, Chunk aufsteigend).
- **Ebene P:** eigene DP-Indizes; Statistik: `idf` aus der K-Statistik der Sichtklasse (Terme ohne K-Eintrag: `df = 0`), `tf` und Länge aus dem Chunk. Die Fusion mit K erfolgt über Ränge (9.4), nie über Scores.

### 9.3 Vektorsuche [E]

- **Quantisierung (Ingest und Anfrage):** Der Embedder liefert `f32`. Je Vektor wird mit `s = 127 / max|xᵢ|` skaliert und `qᵢ = round_half_even(xᵢ·s)` als `i8` gespeichert; zusätzlich `norm2 = Σ qᵢ²` als `u32`. Gespeichert werden die Bytes; Z7 gilt ab diesen Bytes.
- **Ähnlichkeit:** `dot = Σ qᵢ·dᵢ` (`i32`), `sim = (dot << 32) / isqrt(norm2_q · norm2_d)` als Q32.32 (`isqrt` ganzzahlig). Exakte Flat-Suche je Segment, Segmente parallel, Zusammenführung in fester Reihenfolge; Top-k mit festem Gleichstand wie 9.2.
- **Kandidatenmenge** vor dem Scan: K: `V ∩ A`; P: `A_P ∩ Live` (nicht im Gate, 9.4). **[Q]** 1 Mio. Chunks × 768 Dimensionen, p95 unter 150 ms je Anfrage auf 8 Kernen; Recall@10 der Quantisierung ≥ 0,99 gegenüber `f32`.
- Ein ANN-Backend ist erst nach Messbericht zulässig, **nur** als Kandidatenerzeuger mit exaktem Nachrang, und wird als nicht deterministisch ausgewiesen (Z7 gilt dann nicht).

### 9.4 Zugriffskontrolle und Abrufpipeline [E] (Z16)

**Zugriffsmodell (ABAC).** Die Richtlinie (`policy`-Branch, 6.1) besteht aus geordneten Regeln `allow`/`deny` (deny überschreibt) über Nutzerattribute (Rolle, Organisationseinheit, Freigabestufe, Gruppen) und Ressourcenattribute (Sammlung, `confidentiality`, `owner_unit`, `acl_labels`). **Zugriffszellen** sind `(collection, confidentiality, owner_unit)` und `label:<name>`; je Zelle hält `acta-access` eine Roaring-Bitmap der `KOrd`, gepflegt bei Commit und Ingest. Die **Sichtklasse** eines Nutzers ist die sortierte Menge seiner erlaubten Zellen (Hash als `view_class`); `A_K` ist die Vereinigung der Zell-Bitmaps. Nutzer mit gleicher Zellenmenge teilen Statistiken und Caches. **Für P** gilt `A_P` = Vereinigung der Fallindex-Bitmaps (5.4) der dem Nutzer zugewiesenen, offenen Vorgänge, eingeschränkt auf die in der Anfrage genannten `case_id`s (höchstens `max_cases_per_query` = 5). Ohne `case_id` werden **keine** P-Inhalte abgerufen (Zweckbindung, Datenminimierung); ein freier Korpus über P existiert nicht. Die Richtlinie wirkt immer im aktuellen Stand.

**Reihenfolge:** Authentifizierung → Mandant → Sichtklasse → Stand (`as_of`, `valid_at`) → Kandidatenmengen (K: `V ∩ A_K`; P: `A_P`, `Live`, nicht im Gate) → **Anfrageaufbereitung** → BM25 und Vektorsuche parallel → **Reciprocal Rank Fusion** → optional Reranker → Kontextaufbau → Prompt → Generierung (9.5) → Belegprüfung (9.6).

**Parameter (in `retrieval_cfg`, Startwerte):**

|Parameter|Wert|
|---|---|
|`k_text`, `k_vec` (K)|100, 100|
|`k_text_p`, `k_vec_p` (P)|50, 50|
|`rrf_k`|60 (Ganzzahl: `floor(2³² / (rrf_k + rang))`, Summe über Listen)|
|`fuse_keep`|40|
|`rerank_keep`|12|
|`max_chunks_per_doc`|3|
|`neighbor_expand`|1 (benachbarte Chunks desselben `section_id`)|
|`overlap_min` (lexikalischer Stützungstest)|0,30|

**Anfrageaufbereitung.** (1) Spracherkennung; (2) bei Folgefragen in einer Sitzung **Umformulierung** zu einer eigenständigen Frage durch den `Generator` (Ergebnis wird als `query_rewrite` im Protokoll abgelegt, `Sealed`); (3) Anfragevektor über den `Embedder` (Art `Query`) und Quantisierung. Die Umformulierung ist modellabhängig (A-INFER); Z7 gilt ab dem Anfragevektor.

**Fusion und Reranking.** Vier Listen (BM25-K, Vektor-K, BM25-P, Vektor-P) werden über RRF zu einer Liste vereinigt (`fuse_keep`). Der Reranker (Port, A-INFER) bewertet die Kandidaten; seine Gleitkommawerte werden an der Portgrenze mit half-even auf Q32.32 gerundet; Gleichstand: (Reranker-Score absteigend, RRF-Rang aufsteigend, `KOrd`/`VerOrd` aufsteigend, Chunk aufsteigend). Ohne Reranker gilt der RRF-Rang. Danach: höchstens `max_chunks_per_doc` je Dokument, Top `rerank_keep`.

**Kontextaufbau.** Das Kontextbudget ist `ctx_len(Generator) − Systemprompt − Frage − Verlauf − answer_reserve` (Tokenzahlen über `Generator::count_tokens`). Belege werden in Rangfolge aufgenommen, jeweils mit Nachbar-Chunks, bis das Budget erschöpft ist; überlappende Spannen werden zusammengeführt; jeder Beleg erhält eine anfragelokale Kennung `E1…En` und einen Kopf (Titel, `heading_path`, `norm_ref`, Gültigkeitszeitraum `valid_from/valid_to`, Herkunft `K|P`, Stand: Commit-Kurzform bzw. Dokumentdatum). Blöcke sind durch ein **zufälliges Anfrage-Token** begrenzt (`<<E3:7f3a…>>`), das nicht im Beleginhalt vorkommen darf (bei Treffer neu würfeln); Markup und Steuerzeichen werden aus Belegen entfernt.

**Prompt-Injection (T7).** Belege sind **Daten**: Der Systemprompt weist das Modell an, Anweisungen in Belegen zu ignorieren; das Modell hat keine Werkzeuge, kein Netz und kein Gedächtnis außerhalb der Anfrage. Die Antwort ist ein strukturiertes Objekt (9.6), das als Text mit Belegverweisen ausgegeben wird, **ohne aktive Inhalte** (keine Links, Bilder, Markup, keine Beleg-URLs aus dem Modell), damit kein Datenabfluss über Darstellung möglich ist.

**Last und Seitenkanäle.** Die Arbeit ist proportional zur **zugänglichen** Menge; Trefferzahlen, Fehlerbilder, Antwortzeiten-Klassen und Quellennamen unterscheiden zugängliche und unzugängliche Dokumente nicht. Ratenbegrenzung je Nutzer (`rate_limit_per_min`) verhindert Abtasten.

### 9.5 Generierung, Gate-Prüfung, `ContextPurge` [E] (Z1)

- **Epochenprüfung.** Eine Anfrage merkt die Gate-Epoche. Vor jedem Ausgaberahmen und vor Abschluss wird die Epoche verglichen; ändert sie sich, werden die Beleg-Ordinale erneut geprüft. Ist ein Beleg gesperrt, bricht die Anfrage ab, der Puffer wird verworfen, der Aufrufer erhält `Erased` ohne Inhalt. Bis zur Sperre bereits ausgelieferte Rahmen liegen außerhalb der Zusage (A-EXT); Grenze `gate_propagation_max` = 1 Rahmen.
- **`ContextPurge`.** Der Port `Generator` bietet `purge(request_id)`. Persistente Prompt- oder Präfix-Caches des Inferenzdienstes sind für Anfragen mit Personenbezug **abgeschaltet** (Pflichtkonfiguration, T10); KV-Caches sind anfragegebunden und werden mit der Anfrage freigegeben.
- **Antwort-Cache.** Antworten mit P-Belegen werden nicht gecacht (`cache_personal = false`). Antworten aus K sind je (Stand, `view_class`, `retrieval_cfg`, normalisierte Frage) cachebar (`answer_cache_ttl` = 24 h), **nur** wenn die Frage den N-Validator besteht; der Cache ist DK-Lineage-gebunden und verwirft bei Takedown.
- **Sitzungen.** Gesprächskontext liegt nur im RAM (`session_ttl` = 8 h). Jeder Gesprächsschritt trägt eine Lineage (`source_set` der Belege); ein Gate-Eintrag verwirft betroffene Schritte (und alle Folgeschritte, die darauf aufbauen). Sitzungen überleben keinen Neustart.
- **Export und Vorschau** nutzen dieselbe Lesepfadregel (8.1).

### 9.6 Belegpflicht und Stützungsprüfung [E] (Z17)

**Antwortobjekt:** `{ abstain: bool, statements: [{ text, evidence: [E-Kennungen] }], notes: [...] }`. Aussagen mit leerer `evidence` oder mit Kennungen außerhalb der Abrufmenge werden entfernt; Kennungen außerhalb der Abrufmenge sind zusätzlich ein harter Fehler (`GroundingViolation`, Alarm). Bleibt keine Aussage, entsteht die Enthaltung „nicht belegt“ (`abstain = true`). Wird die Generierung nicht strukturkonform geliefert, folgt höchstens ein Wiederholungsversuch mit strengerem Schema, danach Enthaltung.

**Stützung (S).** Der Port `Verifier` bewertet je Aussage, ob die zitierten Belege sie stützen: lexikalischer Überlapptest (Anteil der Inhaltswörter der Aussage in den Belegen ≥ `overlap_min`) **und** Modellurteil; Aussagen unter `support_min` werden entfernt (`support_min` ist Pflichtparameter aus dem Messbericht). Gemessen werden auf einem Goldset (mindestens 300 Fragen, DE und EN, davon 20 % unbeantwortbar): **Zitiergenauigkeit**, **Zitiervollständigkeit**, **Enthaltungsgenauigkeit**, **Halluzinationsrate**. **[Q]** Zitiergenauigkeit ≥ 0,95, Enthaltungsgenauigkeit ≥ 0,90. Die Stützungsprüfung ist ein statistisches Verfahren und wird so ausgewiesen.

**Ausgabe:** Text mit Belegmarken `[1]`, `[2]` …, dazu die Belegliste (Titel, Fundstelle `norm_ref`/Abschnitt, Gültigkeit, Stand, Herkunft K/P). Antworten tragen den Hinweis „Entscheidungshilfe, keine Entscheidung“ (Art. 22).

### 9.7 Bewertungswerkzeug [E]

`acta-cli eval` führt Goldsets (Fragen, erwartete Belege, erwartete Enthaltung; Blob in K, N-Validator-geprüft, ohne Personenbezug) gegen einen Wissensstand und vergleicht zwei Stände. Metriken: Recall@k, MRR, nDCG, Zitiergenauigkeit, Zitiervollständigkeit, Enthaltungsgenauigkeit, Latenzverteilung (p50, p95, p99). Ergebnisse sind `Beleg`-Einträge. **Regressionsgrenzen** (Anhang A: Recall@10 −0,02, Zitiergenauigkeit −0,02, Enthaltungsgenauigkeit −0,03, Latenz p95 +20 %) sind Freigabekriterium für `merge` nach `main` und für Modell-/Konfigurationswechsel, wenn der Mandant es aktiviert (Standard: aktiviert, `merge_main_eval_gate`).

---

## 10. Fachfunktionen für Sachbearbeitung und Verwaltung

Alle Fachfunktionen sind Anwendungsfälle in `acta-engine` über die API (18.5). Das Modell ruft **keine** Werkzeuge auf; deterministische Werkzeuge werden von der Anwendung aufgerufen und als `Tool`-Entscheidung protokolliert (7.1). Jede Funktion unterliegt Zugriffskontrolle (9.4), Gate (8.1) und Belegpflicht, soweit sie Modellausgaben erzeugt. **Entscheidungshilfe, keine Entscheidung (Art. 22):** Ergebnisse tragen den Hinweis und sind durch einen Menschen freizugeben (`Review`).

|ID|Funktion|Eingaben|Verarbeitung|Ausgabe und Persistenz|Löschverhalten|
|---|---|---|---|---|---|
|FF1|**Belegte Antwort**|Frage, Sammlungen, optional `as_of`/`valid_at`|Pipeline 9.4 bis 9.6 nur über K|Antwort mit Belegen; `Query`-Entscheidung (Eingabe `Sealed`, falls Personenbezug)|Nutzlast nach `payload_retention`|
|FF2|**Vorgangsassistent**|`case_id`, Frage oder Auftrag (Zusammenfassung, Chronologie, offene Punkte)|Pipeline mit K **und** P im Vorgangskontext; Belege `P` werden nie gecacht (9.5)|Antwort/Zusammenfassung mit Belegen aus Akte und Vorschriften; `Query`, `Sealed` an die Betroffenen der Belege und des Vorgangs|Löschung einer Person macht Anteile unlesbar (Z3); Begründungskette bleibt|
|FF3|**Prüfliste**|Verfahrensart oder Rechtsgrundlage, Stand|Zerlegung der einschlägigen Vorschriften in Prüfschritte mit Beleg je Schritt; Vorlagen aus `checklists/`|Liste mit Fundstellen; `Query`; speicherbar als Entwurf (`entwurf/<id>`)|wie FF1|
|FF4|**Entwurfsassistent**|Vorlage aus `templates/`, `case_id`, Variablen|Der Entwurf entsteht **in P** (Klasse `Personal`): Platzhalter für personenbezogene Felder werden aus P gefüllt, nie in K geschrieben; Modell schlägt Textbausteine und Begründung mit Belegen vor|Entwurf als P-Dokument-Version (`Draft`), Export als Datei; `Release` bei Freigabe durch Menschen (`Review`)|wie FF2; Entwurfsversionen sind Versionen im Sinne 4.2|
|FF5|**Fristen- und Datumsrechner**|Ereignisdatum, Fristart (Tage, Wochen, Monate), Bundesland|Deterministisch (kein Modell): Fristbeginn, Fristende, Verlängerung bei Sonn-/Feiertag/Samstag nach den Regeln des BGB (§§ 187 bis 193) [V], Feiertagskalender je Bundesland aus `calendar/` (versioniert, Gültigkeitszeit); Ergebnis mit Herleitung|`Tool`-Entscheidung; Ergebnis als Beleg (Regeltext, Kalenderstand)|Eingabedaten `Sealed`|
|FF6|**Änderungsradar**|Sammlung, Zeitraum oder zwei Stände|`diff` der Normen-Blobs; Zuordnung betroffener Chunks; frühere `Query`/`Release`-Entscheidungen mit `EvidenceRef::Knowledge` auf geänderte Blobs; Neulauf des Goldsets; **kein Zugriff auf P**|Bericht: geänderte Normen, betroffene frühere Entscheidungen (nur Kennung, Datum, Art), Antwortunterschiede; `Beleg`|keine Personendaten, daher keine Löschwirkung|
|FF7|**Vergleich und Szenario**|zwei Stände oder Szenario-Definition|Kapitel 7.3|Vergleichsbericht; `ScenarioRun`|Ergebnisse mit Personenbezug `Sealed`|
|FF8|**Begründungsbericht**|`decision_id`|`explain` (7.2) als lesbarer, signierter Bericht (Wissensstand, Belege, Modell, Prüfer)|Export (Kopienregister); keine neuen Daten|Erased-Anteile als „nicht mehr verfügbar“|
|FF9|**Wissenspflege-Workflow**|Änderungen an Wissensdokumenten|Entwurf (`entwurf/<id>`), Validator, Regression (9.7), Vier-Augen-`merge` nach `main`; Veröffentlichung löst ggf. Indexaufbau aus|`Release`; K-Ledger|Takedown nach 6.7|

**Qualitätsregel:** Jede Fachfunktion mit Modellausgabe hat ein Goldset im Mandanten (9.7); ohne Goldset ist die Funktion für den Mandanten nicht freigeschaltet (`feature_enabled`).

---

## 11. Inferenz und Modell-Lebenszyklus

### 11.1 Ports [E]

```rust
pub struct ModelFingerprint {
    pub role: ModelRole,              // Embedder | Reranker | Generator | Verifier
    pub provider: String, pub name: String,
    pub digest: [u8; 32],             // SHA-256 der Gewichtsdatei bzw. vom Dienst gemeldeter Digest
    pub tokenizer_digest: [u8; 32], pub params_hash: [u8; 32],
    pub ctx_len: u32, pub dim: Option<u16>,
}
pub trait Embedder: Send + Sync {
    fn fingerprint(&self) -> &ModelFingerprint;
    fn embed(&self, texts: &[&str], kind: EmbedKind) -> Result<Vec<Vec<f32>>, InferError>; // Query | Passage
}
pub trait Reranker: Send + Sync {
    fn fingerprint(&self) -> &ModelFingerprint;
    fn rerank(&self, query: &str, cands: &[&str]) -> Result<Vec<f32>, InferError>;
}
pub trait Generator: Send + Sync {
    fn fingerprint(&self) -> &ModelFingerprint;
    fn count_tokens(&self, text: &str) -> Result<u32, InferError>;
    fn generate(&self, req: &GenRequest, sink: &mut dyn FnMut(Frame) -> Control) -> Result<GenResult, InferError>;
    fn purge(&self, request: &RequestId) -> Result<PurgeAck, InferError>;   // ContextPurge
}
pub trait Verifier: Send + Sync {
    fn fingerprint(&self) -> &ModelFingerprint;
    fn support(&self, statement: &str, evidence: &[&str]) -> Result<f32, InferError>; // 0..1, an der Grenze nach Q32.32 gerundet
}
pub struct GenRequest { pub id: RequestId, pub system: String, pub messages: Vec<Message>,
    pub params: GenParams /* temperature, top_p, seed, max_tokens */, pub schema: Option<JsonSchema>, pub stop: Vec<String> }
```

`acta-infer-http` implementiert die Ports gegen eine OpenAI-kompatible Schnittstelle (Chat-Completions mit `response_format`/Schema, Embeddings, optional Rerank). Zeitüberschreitungen (`infer_timeout`: Embedding 10 s, Rerank 10 s, Generierung 120 s) und Wiederholung (höchstens 1) sind Konfiguration; Ausfall führt zu `Degraded(Inference)` (14.6).

### 11.2 Anforderungen an den Inferenzdienst [E][A]

Separater Prozess auf Loopback oder Unix-Socket; Pflichtkonfiguration: **keine Protokollierung von Inhalten**, **kein persistenter Prompt-/Präfix-Cache** für Anfragen mit Personenbezug, KV-Cache anfragegebunden, Abbruch laufender Anfragen auf `purge`, kein Netzzugang des Dienstes, Gewichte schreibgeschützt. Der Dienst gehört zum Vertrauensbereich (A-INFER). Ein Dienst, der das nicht zusichert, gilt als externer Dienst und braucht einen `EgressGrant` (13.3). Der Betreiber bestätigt die Zusicherungen im Löschkonzept (12.4).

### 11.3 Referenzprofile (Orientierung, keine Zusage) [Q]

|Profil|Hardware|Generator|Embedder und Reranker|Last|
|---|---|---|---|---|
|S|CPU, 64 GB RAM|7 bis 8 Mrd. Parameter, 4 Bit|mehrsprachiges Embedding (384 bis 768 Dimensionen), kleiner Reranker|bis ca. 10 Nutzer|
|M|1 GPU mit 24 GB|14 bis 32 Mrd. Parameter, 4 Bit|mehrsprachiges Embedding (768 bis 1024), Cross-Encoder|bis ca. 50 Nutzer|
|L|2 GPU mit 80 GB|70 Mrd. Parameter|wie M, höherer Durchsatz|ab ca. 200 Nutzer|

Die Modellauswahl ist Betreiberentscheidung; jedes Modell durchläuft vor Aktivierung den Lebenszyklus 11.4 (Goldset-Messung mit Freigabegrenzen). Ausgeliefert werden keine Modelle.

### 11.4 Modell- und Index-Lebenszyklus [E]

|Phase|Ablauf|
|---|---|
|1 Registrieren|Modell im Inferenzdienst bereitstellen; `ModelFingerprint` erfassen; Eintrag in `config/models` auf Branch `entwurf/<id>`|
|2 Bewerten|Goldset-Lauf (9.7) gegen den Entwurfsstand; bei Embedder-, Chunking- oder Normalisierungswechsel Aufbau einer neuen **`IndexGen`** im Hintergrund (DK; für DP als Reindex-Aufgabe unter dem Systemzweck `Reindex`, aggregiert protokolliert)|
|3 Aktivieren|`merge` nach `main` (Vier-Augen, Regressionsprüfung); `main` zeigt auf die neue `IndexGen` erst nach vollständigem Aufbau; bis dahin antworten Abfragen mit der alten Generation|
|4 Betreiben|Metriken je Modell (Latenz, Ausfallrate, Enthaltungsquote), Drift-Überwachung durch periodischen Goldset-Lauf (`goldset_interval` = 7 Tage); Überschreiten einer Regressionsgrenze löst `QualityDrift` aus|
|5 Auslaufen|Vorgängergeneration bleibt für `index_gen_retention` (= `branch_retention`) erhalten, damit Zeitreisen und `replay` aktueller Entscheidungen möglich sind|
|6 Außerdienststellen|Danach wird die `IndexGen` entfernt (Segmente, `SegmentKey`s vernichtet). Abfragen mit `as_of` auf Stände, deren `IndexGen` außer Dienst ist, antworten mit der aktuellen Generation und der Kennzeichnung `regenerated` oder, auf Wunsch, mit `GenerationRetired`; `rebuild_generation` baut sie aus den Blobs mit dem damaligen Modell neu auf, sofern verfügbar (nicht deterministisch, A-INFER)|

DP-Indizes werden bei Modellwechsel vollständig neu aufgebaut; alte DP-Generationen werden sofort nach Umschaltung entfernt (Datenminimierung, keine Aufbewahrung alter personenbezogener Ableitungen).

### 11.5 Quoten, Budget, Fehlerbilder [E]

Je Mandant: `inference_budget` (Tokens je Stunde), `max_parallel_generations`, Warteschlange `infer_queue_max`; Überschreitung: `Transient(Overloaded)` mit `Retry-After`. Verhalten bei Ausfall: Suche läuft weiter (Belegliste ohne Antwort, `Unavailable`), Ingest staut Embedding-Aufträge in einer begrenzten Warteschlange (`embed_queue_max`) und liefert danach `Transient`; kein Teilzustand wird sichtbar (Embedding vor Manifest, WP-3).
---

## 12. Mandanten, Rollen, Aufbewahrung

### 12.1 Mandantentrennung [E]

Je Mandant getrennt: `TenantId`/`TenantOrd`, Tresor (4.3), alle Ordinalräume, drei Ledger (K, E, A), Indexsegmente (K, P, DK, DP, nie gemischt), Gate-Log, Konfiguration, Quoten, Betroffenenverzeichnisse, Schlüsselableitungen (Salz `TenantId`, `K_dedup(t)`, `K_tag(t)`, Journal-Schlüssel). Mandantenübergreifende Abfragen und Deduplikation gibt es nicht. Quoten: Speicher, Anfragen je Minute, Inferenzbudget, Parallelität. Test: `T-TENANT-*` (Eigenschaftstest mit zufälligen Mehrmandantenlasten: kein Lesen, Zählen oder Zeitkanal über Grenzen; getrennte Prozess-Hintergrundlast beeinflusst Latenzklassen nicht über die Quote hinaus).

### 12.2 Sammlungen, Metadaten, Aufbewahrungsklassen [E][R]

**Sammlung:** Name, Standard-Datenklasse, Zugriffszellen-Regel (9.4), Standard-Aufbewahrungsklasse, Sprache, `chunk_cfg`, Quellsystem (`source_system`), `feature_enabled` je Fachfunktion.

**Metadatenschema** (versioniert in K, Whitelist, N-Validator): `doc_id`, `title`, `source_of_record`, `valid_from`/`valid_to`, `owner_unit`, `confidentiality` (`offen`, `intern`, `vertraulich`, `streng`), `acl_labels`, `retention_class`, `language`, `tags`, `norm_refs`. Freitext in Metadaten ist verboten.

**Aufbewahrungsklasse** = Löschregel nach DIN 66398 [V]: Datenart, Frist, Startzeitpunkt, Löschart. `retain_until = Start + Frist` in Kalendertagen (UTC), je Datum berechnet. Beispiele (der Mandant wählt; Fristen vor Verwendung am Primärtext zu prüfen [V]):

|Klasse|Datenart|Frist|Rechtsgrund|
|---|---|---|---|
|`ak-wissen`|Wissen ohne Personenbezug|unbegrenzt|Betriebszweck|
|`ak-buch-10`|Bücher, Jahresabschlüsse|10 Jahre|§ 147 Abs. 1 Nr. 1 AO, § 257 HGB|
|`ak-beleg-8`|Buchungsbelege|8 Jahre seit 01.01.2025 (Banken, Versicherungen weiter 10)|§ 147 Abs. 3 AO, § 257 Abs. 4 HGB|
|`ak-gesch-6`|Handels- und Geschäftsbriefe, auch E-Mails|6 Jahre|§ 147 Abs. 1 Nr. 2, 3 AO, § 257 HGB|
|`ak-patient-10`|ärztliche Dokumentation|10 Jahre nach Behandlungsabschluss|§ 630f Abs. 3 BGB|
|`ak-behoerde`|Verwaltungsakten|nach Aktenplan, Bundes- oder Landesrecht|Landesrecht [V]|
|`ak-sitzung`|Sitzungsdaten|`session_ttl`|Betriebszweck|

Änderungen: Verlängerung frei, Verkürzung nur mit Review und Vermerk im E-Ledger. Ein täglicher Fristenjob löst Sagas fällig gewordener Ziele gebündelt aus (Batch-Vernichtung, 4.3). Jede Sammlung MUSS eine Aufbewahrungsklasse tragen.

### 12.3 Vorgänge (Cases) [E]

Ein **Vorgang** ist die Zweckbindung für P-Inhalte (9.4). Lebenszyklus:

1. **Registrieren** (`case.register`): `external_ref`, Betroffene (`subject_refs`), `owner_unit`, Aufbewahrungsklasse; erzeugt `CaseId`, Vorgangsobjekt in P (5.4).
2. **Zuweisen** (`case.grant`): Nutzer (`ActorRef`) erhalten Zugriff; Quelle ist das Fachverfahren oder `TenantAdmin`; jede Zuweisung steht im A-Ledger.
3. **Befüllen:** Ingest mit `case_id`; neue Versionen erweitern den Fallindex.
4. **Betreiben:** FF2 bis FF5 im Vorgangskontext; Zugriff nur für zugewiesene Nutzer.
5. **Schließen** (`case.close`): Zuweisungen enden; Dokumente bleiben nach Aufbewahrungsklasse (`Restricted(Retention)` nach Antrag, sonst `Live` bis `retain_until`); der Fallindex wird bereinigt (kein aktives Retrieval nach Schließung außer Wiedereröffnung).
6. **Beenden:** Fristablauf löst die Saga für alle Dokumente aus; der Vorgang wird mit dem letzten Dokument vernichtet.

### 12.4 Mandanten-Löschkonzept [E]

Pflichtartefakt je Mandant; `acta-reports` erzeugt den Entwurf. Inhalt (SDM-Baustein 60, DIN 66398 [V]): Datenkategorien und Verarbeitungstätigkeiten; Fristen und Auslöser; Rollen (anordnen, prüfen, ausführen); technisches Verfahren je Speicherort (Acta liefert die Tabelle Ebene × Speicher × Verfahren × Erfüllungsklasse); Backup-Sperrung und Rücksicherungsschutz (`T_BACKUP`, 8.8); Löschprotokoll; Stichprobenkontrolle; Kopienregister; Auftragsverarbeiter; Archivierung; Zusicherungen des Inferenzdienstes (11.2); Notfallplan Algorithmenwechsel (4.7). Der Start verweigert den Betrieb eines Mandanten, dessen Sammlungen keine Aufbewahrungsklasse oder dessen Pflichtparameter keinen Wert haben.

### 12.5 Rollen und Vier-Augen-Aktionen [E]

|Rolle|Rechte|
|---|---|
|`SystemAdmin`|Betrieb, Konfiguration; **kein** Datenzugriff (T3)|
|`TenantAdmin`|Benutzer, Sammlungen, Sichtklassen, Fallzuweisung des Mandanten|
|`Sachbearbeitung`, `Fachabteilung`|Abfragen, Vorgangsarbeit und Wissenspflege im Rahmen der Sichtklasse|
|`DataProtection`|`restricted_read`, Reviews, Betroffenenanfragen, `subjects.discover`|
|`Legal`|`legal_hold`, Aufbewahrungsentscheidungen|
|`Revision`|nur lesend auf Ledger und Berichte|

Das Prinzip „anordnen ≠ ausführen ≠ prüfen“ wird technisch erzwungen: Anordnende und Freigebende sind verschiedene Personen. **Vier-Augen** (zwei verschiedene Personen, signiert, Zeitfenster 72 h) für: `Open`- und `Split`-Freigabe, Takedown, `legal_hold`, Verkürzung der Aufbewahrung, `merge` nach `main`, Review-Entscheidungen nach 8.9, `Waiver` (7.5), `EgressGrant` (13.3), `accept_rollback` (8.8), Aufhebung einer Vergiftung bei `Tenant`/`Node` (15.2).

### 12.6 Mandanten-Lebenszyklus [E]

- **Anlage** (`rotation_lock`): `TenantId`/`TenantOrd`, Tresor, Ledger (`chain_id`, Anker-Köpfe), Standard-Sichtklassen, Löschkonzept-Gerüst, Canary-Betroffener, Pflichtparameter-Abfrage; bis dahin Zustand `Quarantine`.
- **Betrieb:** Kapitel 8 bis 11.
- **Off-Boarding:** (1) Sperre aller Lesepfade (alle Versionen `Restrict(Offboarding)`), (2) Übergabe oder Export nach Betreiberentscheidung (signiert, Kopienregister), (3) Saga `Expired` für alle P-Ziele, (4) Entfernen von K-, DK-, DP-, P-Segmenten (Scrub), Tresor und Exportdateien, (5) Ledger nach Aufbewahrungsvorgabe archivieren oder vernichten, (6) Bericht mit `offboarding_exposure_until` = Ende der letzten Sicherungsepoche, die Schlüssel des Mandanten lebend enthielt, + `T_BACKUP`; danach sind alle Kopien Dritter unlesbar (C27).

### 12.7 Aufbewahrung von Entscheidungen [E]

`decision_retention` und `payload_retention` sind je `DecisionKind` Pflichtparameter **ohne Standardwert**. Empfehlungen, keine Vorgaben: `Query` 90 Tage (Nutzlast 30 Tage), `ScenarioRun` gemäß `branch_retention`, `Review` nach Fachrecht des Vorgangs, `Release` bis zur Änderung der Richtlinie (Nutzlast nach Fachrecht), `Tool` 90 Tage. Nach `decision_retention` entsteht ein `Expired`-Vermerk im K-Ledger, der Eintrag wird nach 8.4 entfernt; nach `payload_retention` wird der Nutzlastschlüssel vernichtet (Art. 5 Abs. 1 lit. e).

---

## 13. Sicherheit, Souveränität und Lieferkette

### 13.1 Prozess- und Speicherhärtung [E]

`mlock`, `MADV_DONTDUMP`, `RLIMIT_CORE = 0`, `PR_SET_DUMPABLE = 0`; `ptrace` nur bei `kernel.yama.ptrace_scope ≥ 1`; Schlüsselmaterial nur in `SecureBuf`. Der **Selbsttest beim Start** (14.1) prüft: Swap (deaktiviert oder mit Zufallsschlüssel verschlüsselt), Core-Pattern, `RLIMIT_MEMLOCK` ausreichend für Schlüsselspeicher, `O_DIRECT`-Unterstützung, Dateisystemtyp. **Zulässig** sind `ext4` und `xfs` (überschreibende Semantik, `remove_scrub` wirksam); **Copy-on-Write-Dateisysteme** (`btrfs`, `zfs`) und Dateisysteme mit Snapshots verweigern den Start, es sei denn, der Betreiber hat sie ausdrücklich deklariert (A-FS); sie erscheinen dann im Bericht, Scrub gilt als nicht wirksam. Jede andere Abweichung verweigert den Start, es sei denn, deklariert. Root-Zugriff auf den Prozessspeicher bleibt ausgenommen (T8).

### 13.2 Worker-Sandbox [E]

Landlock ohne Dateisystemzugriff, seccomp-Positivliste, kein Netz, rlimits (CPU, Speicher, Dateien), eigener Benutzer, keine geerbten Dateideskriptoren außer Ein- und Ausgabe, `PR_SET_NO_NEW_PRIVS`. Der Worker lebt für genau ein Dokument; Absturz, Zeitüberschreitung oder Limitverletzung bleiben auf den Aufruf beschränkt. Parser-Abhängigkeiten sind nur in `acta-parse` erlaubt (3.8). Der Worker erhält weder Tresor-, Anker- noch Mandantenwissen; die Ergebnisse (Text, Struktur) werden vom Kern erneut validiert (Längen, UTF-8, Struktur).

### 13.3 Netz und Egress [E] (Z18)

Standard: kein ausgehender Verkehr. Lokale Inferenz über Loopback oder Unix-Socket. Ein externer Dienst braucht einen **`EgressGrant`** (Vier-Augen): Dienst, Zweck, zulässige Datenklassen, Ablauf, DSFA-Referenz, Auftragsverarbeitungsvertrag, Drittlandprüfung [R]. Inhalte der Ebene P gehen **nie** an externe Dienste, es sei denn, der Grant nennt `Personal` ausdrücklich und die Prüfungen sind dokumentiert. Jeder Aufruf wird protokolliert (Dienst, Größe, Klasse, Grant-ID; keine Inhalte) und im Kopienregister geführt. Ein Inferenzdienst ohne die Zusicherungen aus 11.2 gilt als extern. Die Löschwirkung bei externen Diensten ist nicht messbar und wird als A-EXT ausgewiesen. Ein Grant läuft ab; nach Ablauf blockiert die Egress-Schicht (Positivliste von Zielen).

### 13.4 Authentifizierung und Autorisierung [E]

OIDC (Token-Prüfung mit Signaturprüfung, Schlüsselrotation des Anbieters, `exp`/`nbf`, maximale Tokenlebensdauer `token_max_age` = 1 h, Zielgruppenprüfung) für Nutzer, mTLS für Dienstkonten; ABAC-Auswertung in `acta-access`; Ratenbegrenzung und Eingabegrenzen; Administratoren ohne Ankerzugriff erhalten keinen Klartext (T3). Es gibt keine statischen API-Schlüssel und keine Standardkonten. Sitzungs-Widerruf: Entzug einer Rolle oder Zuweisung wirkt ab der nächsten Anfrage (Sichtklasse wird je Anfrage ermittelt, Cache `view_class_ttl` = 60 s mit Invalidierung bei Richtlinienänderung).

### 13.5 Protokollierung [E] (Z15)

Protokollfelder stehen auf einer Positivliste (Handles, Codes, Größenklassen, Zeitpunkte, Zahlen). Fehlertexte und `code()` enthalten keine Kennungen. Metriken verwenden keine Kennungen als Label. Betriebsprotokolle haben `ops_log_retention` = 90 Tage und sind je Monat unter einem Betriebs-`LogKey` versiegelt (Ablauf vernichtet den Schlüssel). Aufzeichnungen nach KI-VO sind Ereignisprotokolle, keine Klartextprotokolle (20.6). Zugriffsprotokolle: A-Ledger (6.2), Aufbewahrung `access_log_retention` (Pflicht).

### 13.6 Lieferkette [E]

G-SUPPLY: `cargo-deny` (Lizenz, Advisories, Quellen), Audit der kryptografischen Abhängigkeiten, SBOM (CycloneDX), reproduzierbare Builds, signierte Releases, `cargo geiger`-Bericht. Hersteller-Pflichten nach dem Cyber Resilience Act (Meldepflichten ab 11.09.2026, Hauptpflichten ab 11.12.2027) [V]: Prozess für Schwachstellenmeldung und Sicherheitsupdates, ohne Telemetrie (Z19, 14.7).

---

## 14. Betrieb

### 14.1 Start und Selbsttest [E]

Der Knoten öffnet die Schnittstelle erst nach dieser Reihenfolge; ein Fehler führt zu `Quarantine` oder `Poisoned` (15.2), nie zu einem Teilbetrieb ohne Hinweis:

1. Konfiguration validieren (Pflichtparameter, Invarianten 15.4).
2. Hostprüfungen (13.1).
3. Anker öffnen: `assurance`, `current_generation`, `min_generation`; Vergleich mit dem Tresor (Rotation fortsetzen, falls `anchor.current_generation() > vault.generation`).
4. Tresore öffnen (Scan, RAM-Index); Schwanz der Rotation anwenden (R-KEY-FIRST).
5. Ledger prüfen (Kette, Checkpoints) und Kopf gegen `last_head` des Ankers (`RestoreBehindAnchor`).
6. Gate-Log laden; **Abgleich physischer Tatsachen** (Tresorzustände, Manifest) mit dem Journal; offene Sagas wieder aufnehmen (Z4).
7. Manifest prüfen; verwaiste Dateien bereinigen (9.1); verwaiste Schlüssel (C01) nach Karenz vernichten.
8. Registersenke abgleichen (`fetch_since`); fehlt sie, `Degraded(Sink)` (nur zulässig, wenn der Ankerstand vorhanden ist).
9. **Positivkontroll-Selbsttest:** Ein synthetischer Betroffener wird angelegt, gesperrt, sein Schlüssel vernichtet und die Messung gegen den Canary durchlaufen. Scheitert das, ist der Löschpfad defekt und der Knoten geht in `Quarantine`.

### 14.2 Hintergrundaufgaben

|Aufgabe|Takt|Bemerkung|
|---|---|---|
|Wurzelrotation|`T_ROT` 12 h, `N_SHRED`, auf Anforderung|4.3|
|Tresorexport|`vault_export_interval` 1 h|4.3, 14.4|
|Datensicherung|`data_backup_interval` 24 h|14.4|
|Fristenjob|täglich|12.2|
|Quarantäne-Sweeper|stündlich|2.2|
|Epochenverwaltung|laufend|`CommitKey`, `M_bk(e)`, `LogKey`|
|Sicherungsepochen vernichten|zeitgesteuert|S7 in 8.3|
|Senkenspiegelung|laufend, asynchron|6.2|
|Checkpoints|alle 1 024 Einträge|6.2|
|`Rewrap`|nach Bedarf|4.7|
|Ausgangskorb|laufend|8.6|
|Garbage Collection|täglich|6.7|
|Gate-Log-Verdichtung|wöchentlich|5.3|
|A-Ledger-Pruning|monatlich|6.2|
|Wiederholungstest `Split`|`split_retest_interval`|2.3|
|Kryptobericht|`crypto_review_interval`|4.7|
|Canary-Selbsttest|täglich|14.1 Nr. 9|
|Goldset-Drift|`goldset_interval` 7 Tage|11.4|
|Indexaufbau, Reindex|bei Bedarf|11.4|

### 14.3 Metriken und Alarme [E]

Metriken ohne Kennungen (13.5): Gate-Latenz, Alter der `Pending`-Sagas, Verbrauch von `erase_sla_B`, Rotationsverzug, Exportalter, Senken-Rückstand, Vergiftungszustand, Messfehler (`Invalid`), Ausgangskorb-Alter, Speicher, Quoten, Suchlatenz, Enthaltungsquote, Modelllatenz. Alarme (Mindestumfang): `SlaBreach`, `RotationOverdue`, `ExportLate` (Export älter als `2 × vault_export_interval`), `BackupOverdue` (**kein** gültiger Export innerhalb `T_BACKUP`), `SinkLag`, `MeasureInvalid`, `SagaStuck`, `ReviewOverdue`, `Poisoned`, `RequestDue` (70 % der Frist), `QualityDrift`, `GroundingViolation`.

### 14.4 Sicherung und Wiederherstellung [E]

Drei Teile, die zusammen die Wiederherstellung tragen: (1) **Datensicherung** der unveränderlichen Segmente (K, DK, P), des Manifests, Gate-Log-Snapshots, Ledger und Verzeichnisse (nur Chiffrate; Konsistenzpunkt durch Manifest-Schnappschuss und Journalrotation; **ohne** DP-Segmente, ohne Caches, ohne Tempdateien, ohne Tresordateien); (2) **Tresorexport** nach 4.3 (Epochenschlüssel, begrenzte Gültigkeit); (3) **Ankersicherung** (14.5). Die Registersenke (6.2) ist **nicht** Teil der Sicherung, sondern ein eigener Ausfallbereich. Ohne gültigen Export ist eine Datensicherung unlesbar; die Aufbewahrung von Datensicherungen darf `T_BACKUP` nicht überschreiten (R-BK). **Restore-Tests** sind quartalsweise verpflichtend (`T-REST` auf Produktionsumgebung mit Testmandant). Die Wiederherstellung folgt 8.8; Ableitungen DP werden danach neu aufgebaut (Dauer proportional zum P-Bestand, `RTO`-relevant).

### 14.5 Ankersicherung [E]

`k_idx` und die Anker-Köpfe sind Teil der Ankersicherung; sie werden nicht rotiert und getrennt von den Datenschlüsseln gesichert. Verlust von `k_idx`: Betroffene sind nicht mehr auffindbar (Auskunft und Löschung über `SubjectTag` unmöglich). Verlust von `M_g`: die Daten in Ebene P sind unlesbar (so gewollt, daher Hochverfügbarkeit des Ankers). Beim `FileAnchor` MUSS die Passphrase auf mindestens zwei Personen verteilt sein (Wissensaufteilung) und DARF nirgends gespeichert werden; bei HSM und KMS gelten die Sicherungsverfahren des Herstellers des Geräts bzw. Dienstes. **Kein Wiederherstellungsweg über den Acta-Hersteller** (Z19).

### 14.6 Verfügbarkeit [E]

Ein Knoten je Installation; Wiederherstellung ist **kalt**. **[Q]** Startwerte: RTO 4 Stunden (ohne DP-Neuaufbau), RPO für Schlüssel höchstens `vault_export_interval` (1 Stunde), RPO für Daten höchstens `data_backup_interval` (24 Stunden; Segmente und Journal enthalten nur Chiffrate und dürfen laufend auf ein zweites Medium gespiegelt werden, was den RPO senkt). Ein Dokument, dessen Schlüssel nach dem letzten Export entstand, ist nach einem Tresorverlust unlesbar.

|Betriebszustand|Auslöser|Wirkung|
|---|---|---|
|`Normal`|–|alles aktiv|
|`Degraded(Anchor)`|Anker nicht erreichbar|Lesen und Schreiben laufen (Schlüssel im RAM); keine Rotation, kein Export; Klasse K bleibt `not_reached`, Sagas `Pending`|
|`Degraded(Sink)`|Senke nicht erreichbar|Sperren wirken unverändert (Spiegelung blockiert nie); `registry_mirrored = false`, Klasse M `not_reached`, Alarm `SinkLag`|
|`Degraded(Inference)`|Inferenzdienst nicht erreichbar|Suche läuft, Antworten enthalten sich (`Unavailable`)|
|`Restoring`|Wiederherstellung läuft|alle Lesepfade geschlossen (8.8)|
|`Quarantine`|Registerbeleg fehlt, Selbsttest fehlgeschlagen, Ordinalkonflikt, `RestoreBehindAnchor`|keine Daten|
|`Poisoned(Scope)`|Integritäts- oder Kryptofehler|Bereich geschlossen (15.2)|

### 14.7 Anbieterneutralität [E] (Z19)

Keine Telemetrie, kein Fernzugriff, keine Lizenzprüfung mit Nutzerdaten, kein Escrow. **Lizenz:** offline prüfbare signierte Datei ohne Nutzerdaten. **Updates:** signierte Pakete, offline einspielbar. **Support:** `acta-cli support-bundle` erzeugt lokal ein Paket aus einer Positivliste (Version, Konfiguration ohne Geheimnisse, Aggregatmetriken, Fehlercodes, Hostprüfungen, Größenklassen). Test `T-VENDOR-BUNDLE`: ein Canary mit synthetischen Inhalten und Kennungen darf im Paket nicht auftauchen. Der Betreiber gibt das Paket frei und trägt es selbst weiter (Kopienregister). **Einordnung [⚠]:** Bei diesem Zuschnitt ist der Hersteller für Daten der Betreiber in der Regel weder Auftragsverarbeiter noch Verantwortlicher; das ist eine Einschätzung, keine Feststellung (20.1). Sie entfällt bei Fernwartung, Telemetrie, Hosting, Escrow, Lizenzdiensten mit Nutzerdaten oder Weiterleitung an externe Dienste.

### 14.8 Upgrade und Formatmigration [E]

Plattenformate tragen `format_version`; Migrationen laufen nur vorwärts, nach Sicherung und mit Golden-Tests je Version; ein Downgrade wird nicht unterstützt. Algorithmenwechsel nutzen `Rewrap` (4.7). Ein Upgrade ist ein kurzes Wartungsfenster (ein Knoten); der Server fährt geordnet herunter (Anfragen abschließen, Journal flushen, Gate-Log synchronisieren, Schlüsselspeicher freigeben). Nach dem Upgrade läuft der Selbsttest (14.1); ein Fehler stellt den Knoten auf `Quarantine` und verlangt die Wiederherstellung der Vorversion aus der Sicherung.

---

## 15. Qualität, Fehler und Konfiguration

### 15.1 Fehlerklassen und Behandlung [E]

|Klasse|Behandlung|
|---|---|
|`Transient`|Wiederholung mit Backoff, Zustand bleibt, Alarm nach Schwelle|
|`Integrity`, `Crypto`|Vergiftung (15.2)|
|`Erased`|normales Ergebnis, nur aus Gate oder Registerbeleg (3.5)|
|`Policy`|Ablehnung (Rolle, Zugriff, Freigabe)|
|`Invalid`|Ablehnung der Eingabe, nichts wird geschrieben|

### 15.2 Vergiftung [E]

Vergiftung schließt den **kleinsten nachweislich isolierenden Bereich**: `Document` (ein Umschlag mit Authentisierungsfehler bei vorhandenem Schlüssel), `Segment` (Kopf-MAC oder Prüfsumme), `Tenant` (Tresor, Manifest, Gate-Log, Ledger-Kette, Senkenwiderspruch, `FsyncFailed` ohne Ersatz), `Node` (Anker, Uhr, Zufall, Ordinalüberlauf). In einem vergifteten Bereich scheitern alle Lese- und Schreibzugriffe mit `Integrity` oder `Crypto`; **Gate-Sperren bleiben wirksam**. Aufhebung nur durch `acta-cli repair` mit Prüfbericht und, bei `Tenant` und `Node`, Vier-Augen-Freigabe; ein Neustart allein hebt eine Integritätsvergiftung nicht auf.

### 15.3 Leistungsziele [Q]

Referenz: 8 Kerne, 64 GiB RAM, NVMe, ein Knoten. Alle Werte sind Startwerte; Freigabe nur nach gemessenem Bericht.

|Messgröße|Zielwert|
|---|---|
|Rückkehr einer Löschung im Zustand `Gated` (Person mit 1 000 Versionen)|p99 unter 50 ms (harte Grenze `erase_return_timeout` 2 s)|
|Gate-Prüfung|unter 1 µs je Prüfung|
|Hybridsuche (1 Mio. Chunks, ein Mandant)|p95 unter 1,5 s bis zu den Top-n Belegen|
|Antwortbeginn (lokale Inferenz)|abhängig vom Modell; Pipeline ohne Modell unter 2 s p95|
|Tresor öffnen (10 Mio. Einträge)|unter 30 s|
|Rotation (10 Mio. Einträge)|unter 120 s|
|Ingest (Text, ohne Embedding)|mindestens 50 MiB/s gesamt|
|Segmentersatz (256 MiB gefiltert)|unter 20 s|
|Klasse B einer Einzelperson mit 1 000 Dokumenten|unter 24 h (`erase_sla_B`), erwartbar Minuten|

### 15.4 Konfiguration und Invarianten [E]

Der Start prüft folgende Invarianten und verweigert bei Verletzung den Betrieb mit `Invalid(Config)`:

|ID|Invariante|
|---|---|
|R-HORIZON|`commit_key_retention ≥ T_BACKUP + backup_epoch`|
|R-BK|`backup_epoch ≤ T_BACKUP`; `data_backup_retention ≤ T_BACKUP`|
|R-ROT|`T_ROT ≤ 24 h` und `rotation_min_gap ≤ T_ROT`|
|R-EXP|`vault_export_interval ≤ 4 h`|
|R-SINK|Registersenke liegt auf einem anderen Gerät als die Daten (`DirSink`: `st_dev` verschieden)|
|R-ANCHOR|`anchor_destroy_max` gesetzt; bei `Besonders` kein `FilePassphrase`|
|R-HIST|`history_horizon(Besonders) = 0`|
|R-RET|`decision_retention` und `payload_retention` je Art gesetzt; `access_log_retention` gesetzt|
|R-RETMAX|`commit_key_retention ≤ 3 Jahre`, darüber nur mit dokumentierter Begründung im Löschkonzept [⚠]|
|R-KMIN|`K_MIN ≥ 5`|
|R-DPBK|bei `Besonders`: `backup_derived_dp = false`|
|R-FS|Dateisystem zulässig oder deklariert (13.1)|
---

## 16. Test und Freigabe

### 16.1 Prinzipien [E]

1. Eine Zusage (1.5) ohne automatisierten Test gilt als nicht erfüllt (Gate G-ZUSAGE). Jeder Test hat ein **unabhängiges Orakel**: Testvektoren aus RFCs, eine unabhängige Referenzimplementierung im Test, Eigenschaften (Hin- und Rückumwandlung, Invarianten) oder Golden-Dateien. Ein Test, dessen Orakel aus dem getesteten Code abgeleitet ist, zählt nicht.
2. **Positivkontrolle vor Negativaussage:** Jeder Test, der die Abwesenheit von etwas behauptet (kein Klartext, kein Schlüssel, kein Treffer), enthält einen Teil, der die Anwesenheit am Canary nachweist.
3. Testdaten sind synthetisch. Echte personenbezogene Daten sind in Tests verboten.
4. Tests laufen deterministisch (`ManualClock`, `SeededRng`, `FaultVfs`); Nichtdeterminismus (Inferenz) ist hinter Ports durch Doubles ersetzt, die Fingerprints und Ausgaben festlegen.

### 16.2 Testebenen [E]

|Ebene|Werkzeug|Gegenstand|
|---|---|---|
|Einheit|`cargo test`|reine Funktionen, Parser, Kodierer, Ableitungen|
|Eigenschaft|`proptest`|Kodierung Hin und Zurück, `no_shared_dek`, Roaring-Schnitte, Festkomma-Arithmetik, Chunking-Determinismus, Mandantentrennung|
|Nebenläufigkeit|`loom`|Sperrreihenfolge (3.6), Gate-Zeigertausch, Schnappschüsse, Rotation gegen Schlüsseloperationen|
|Fehlerinjektion|`FaultVfs`|Absturz an jedem Durabilitätspunkt (Anhang C), zerrissene Schreibvorgänge, `FsyncFailed`, `ENOSPC`, Lesefehler|
|Fuzzing|`cargo-fuzz`|ein Ziel je Plattenformat (Journal, Segment, Manifest, Tresor, Gate-Log, Ledger), Worker-Protokoll, jeder Parser|
|Compile-Fail|`trybuild`|Z13 (`Derived<T>` ohne Lineage), `Validated<T>`, `Sealed<P>`, Schlüsseltypen ohne `Default`/`Clone`/`Debug`/`Serialize`|
|Golden|Dateien im Repository|jedes Plattenformat je `format_version`, kanonische Objektformate, Berichtsformat, Festkomma-Vektoren `T-TT-FX`|
|Integration|Anwendungsfälle|Lebenszyklen aus Kapitel 19, Ende-zu-Ende mit Doubles für Inferenz|
|Mutation|`cargo-mutants`|Löschkern, Gate, Tresor, Restore-Schutz: überlebende Mutanten sind Defekte|
|Plattform|CI auf x86_64 und aarch64|Z7 byte-identisch über beide Architekturen|
|Last und Messung|eigene Benchmarks|[Q]-Werte aus 15.3, 9.3, 9.6|
|Sicherheit|extern|Penetrationstest (API, Parser, Worker-Sandbox), Krypto-Audit|

### 16.3 Zusagenmatrix [E]

Jede Zusage hat mindestens die folgenden Tests; die vollständige Familienliste steht in Anhang D.

|Zusage|Primäre Tests|
|---|---|
|Z1|`T-GATE-<Lesepfad>` (Suche, Antwort, `as_of`, Szenario, Cache, Export, Vorschau, Sitzung, Streaming, `Superseded`); `T-CONC-GATE` (`loom`: Linearisierung)|
|Z2|`T-CRY-1` (Bytes-Grep über alle Dateien mit bekannten Klartexten und Kennungen), `T-DERIVED-SEALED`, `T-META-*` (keine Klartext-Tags, keine exakten Längen)|
|Z3|`T-MULTI-*`: Trockenlauf, jeder Konfliktmodus, Zwischenzustand `Restricted(MultiSubjectReview)`, Sammelakte, `AddSubject`|
|Z4|`T-CRASH-*` für jeden Punkt C01 bis C27|
|Z5|`T-SEP-1` (K enthält keine Muster aus dem Personenkorpus), `T-ANON-*`, Compile-Fail `Validated`|
|Z6|`T-VCS-ERASE-1`: Objekt-IDs, Commit-Hashes und K-Ledger vor und nach einer P-Löschung identisch|
|Z7|`T-TT-*` (x86_64 und aarch64), `T-TT-FX` (Festkomma-Goldvektoren), `T-TT-REDACTED`|
|Z8|`T-DEC-*`: Begründungskette nach Löschung vollständig, personenbezogene Nutzlast `Erased`|
|Z9|`T-SCEN-*`: `main`, P und Zugriffsrechte vor und nach dem Szenario identisch|
|Z10|`T-NOFALLBACK-*`, `T-RNG-*` (Entropiefehler erzeugt weder Umschlag noch Schlüssel)|
|Z11|`T-EVID-*`: jedes Messverfahren mit absichtlich verstümmeltem Scanner muss scheitern|
|Z12|`T-REST-*`: mit Senke, nur mit Ankerstand, ohne beides (`Quarantine`), außerhalb Horizont, zurückgerolltes K-Ledger|
|Z13|Compile-Fail `derived_requires_lineage`|
|Z14|Eigenschaft `no_shared_dek` über `VerRef`|
|Z15|`T-LOG-*`: Canary-Kennungen in Journalen, Ledgern, Fehlertexten, Metriken, Supportpaket|
|Z16|`T-ACL-*`, `T-ACL-RANK` (Hinzufügen, Entfernen, Ändern eines unzugänglichen Dokuments ändert Ergebnisse, Zählwerte, Fehlerbilder und Zeitklassen byte-genau nicht)|
|Z17|`T-GROUND-*` (Struktur, D); Goldset-Messung (S, 9.6)|
|Z18|`T-EGRESS-*`: Netzblockade im Standard, Grant-Prüfung, Ablauf|
|Z19|`T-VENDOR-*`, `T-VENDOR-BUNDLE`|

### 16.4 Crash-Matrix und Positivkontroll-Tests [E]

Für jeden Durabilitätspunkt `C-nn` läuft ein Szenario: Zustand aufbauen, am Punkt abbrechen (`FaultVfs`), neu starten, Invarianten prüfen:

- **I1** Eine durable Sperre geht nie verloren.
- **I2** Kein Lesepfad liefert Klartext eines gesperrten Ziels.
- **I3** Ein Schlüssel geht nie ohne Absicht verloren (R-KEY-FIRST).
- **I4** Jede Saga ist vollendet oder `Pending` bei aktiver Sperre.
- **I5** Ledger-Ketten, Manifest und Gate-Log sind konsistent.
- **I6** Ordinale werden nie wiederverwendet.
- **I7** Nach `FsyncFailed` wird dieselbe Datei nicht als stabil angenommen (WP-7).

Zusätzlich je Messverfahren (8.5) ein Positivkontroll-Test: Das Verfahren wird deaktiviert oder verstümmelt, der Test MUSS dann scheitern. Ein Messverfahren ohne diesen Test liefert `NotImplemented`.

### 16.5 Messberichte und Freigabekriterien [E]

Alle [Q]-Werte (15.3, 9.3, 9.6, 4.3) sind Startwerte. Freigegeben wird eine Version, wenn:

1. alle Zusagen-Tests grün sind und G-ZUSAGE besteht;
2. die vollständige Crash-Matrix besteht und kein Mutant im Löschkern überlebt;
3. ein **Messbericht** (Referenzhardware, Korpus, Messdatum, Konfigurationsstand als Commit) jeden [Q]-Wert erfüllt oder die Abweichung mit Begründung und ADR ausweist;
4. `support_min` und die Regressionsgrenzen (9.7) aus dem Messbericht festgelegt sind (Pflichtparameter);
5. der Kryptobericht (4.7) und ein Penetrationstest (API, Parser, Sandbox) vorliegen;
6. die Freigabepflichten aus 20.4 erledigt sind;
7. ein Restore-Test (14.4) auf Produktionsumgebung mit Testmandant im laufenden Quartal bestanden ist.

Der Messbericht der Speicher-Engine-Prüfung (5.1, Entscheidungsregel Eigenbau) liegt vor Beginn der Speicherimplementierung vor.

### 16.6 Architektur-Gates (`cargo xtask gates`) [E]

|Gate|Prüfung|
|---|---|
|G1|Level-Graph gegen `rings.toml` und `cargo metadata` (negative Fixtures)|
|G2|Jeder Port hat im Betrieb genau eine Verdrahtung (HR-7)|
|G3|keine nicht injizierte Zeit- oder Zufallsquelle (`SystemTime::now`, `thread_rng`)|
|G-CRY|keine `Default`/`Clone`/`Debug`/`Serialize` auf Geheimnissen; kein `unwrap_or_default()` und kein `let _ =` auf `Result` in `acta-crypto`, `acta-vault`, `acta-seg`, `acta-erasure`, `acta-history`, `acta-pstore`|
|G-FS|Dateizugriffe nur in `acta-fs`|
|G-ERA|Löschungen nur über den Löschkern; Segmente, Indizes, Caches nur über `ErasureLayer`|
|G-READ|öffentliche Lesepfade nur über den gegateten Leser (`ErasedView`)|
|G-UNSAFE|`unsafe` nur in `acta-sys`, jeder Block mit `// SAFETY:`|
|G-NOTOKIO|kein `tokio` in Level 0 bis 7|
|G-NOSERDE-DISK|kein `serde`/`bincode` in Plattenformaten|
|G-HUB|Größenbudgets (3.3), HR-1 bis HR-9|
|G-WORD|Wortfilter (1.6) über Quelltext, Texte, Vorlagen und diese Spezifikation; Regel B über Vorlagen und Oberflächentexte|
|G-SUPPLY|`cargo-deny` (Lizenz, Advisories, Quellen), Audit kryptografischer Abhängigkeiten, SBOM, `cargo geiger`-Bericht|
|G-DEF|alle Crates in `default-members`|
|G-ZUSAGE|jede Zusage Z1 bis Z19 hat mindestens einen grünen Test|
|G-LINT|Produktionscode: `deny(clippy::unwrap_used, expect_used, panic, indexing_slicing, let_underscore_must_use)`|

---

## 17. Unabhängiger Prüfer (`acta-verify`)

### 17.1 Zweck und Abgrenzung [E]

`acta-verify` ist ein eigenständiges Programm **ohne interne Abhängigkeit** (3.3). Es nutzt nur `sha2`, `ed25519-dalek`, `serde_json` und eine RFC-8785-Bibliothek und kann deshalb von Revision, Datenschutzfunktion oder Dritten gebaut und geprüft werden. Es prüft **Struktur und Echtheit** von Nachweisen, nicht die Wahrheit einer Messung (die ist Gegenstand von 16.4).

### 17.2 Prüfstufen [E]

|Stufe|Eingaben|Prüfung|
|---|---|---|
|**V1**|Löschbericht, Ledger-Ausschnitt bis zum nächsten Checkpoint, öffentlicher Notar-Schlüssel (`key_id`, `sig_alg`)|(1) JCS-Kanonisierung und Hash des Berichts; (2) Signatur mit Domänenpräfix `acta/report/v1`; (3) Berichtshash steht als Eintrag im E-Ledger; (4) Kette ab Vorgängerhash nachrechnen; (5) Checkpoint-Signatur (`acta/checkpoint/v1`); (6) Konsistenz: jede Klasse mit `reached` trägt Belegfelder, `NotImplemented` und `not_measured` sind nie `reached`, Positivkontrolle `passed` bei jeder vollständigen Messung, Klasse K nur mit `backup_exposure_until` in der Vergangenheit, Klasse M nur mit `registry_mirrored = true`; (7) Wortfilter (1.6) über Textfelder; (8) Annahmenliste vollständig|
|**V2**|zusätzlich Tresorabbild und Generationsschlüssel unter Aufsicht des Betreibers|Grabstein-Struktur am im Bericht genannten Slot, Direct-Read-Nachweis, Generation gegen `DestroyProof`|
|**V3**|zusätzlich `CommitKey(e)` (solange vorhanden)|Nachrechnen eines Commitments für eine dem Betreiber bekannte Kennung|

Ausgabe: `verified` oder `failed` mit stabilen Codes; nie Inhalte oder Kennungen. Ein Objekt mit `Redacted`-Vermerk ist undurchsichtig; geprüft wird nur die Kette.

### 17.3 Vertrauensgrenzen [E]

Der Notar-Schlüssel liegt im Anker; ein kompromittierter Anker kann Nachweise fälschen (A-ANCHOR). Gegenmittel: Anker-Köpfe der Ketten, Registersenke in getrenntem Ausfallbereich, Re-Witness (4.7), mehrere unabhängige Prüfer. Der Prüfer ersetzt keine Prüfung der Messverfahren.

---

## 18. Berichte und Schnittstellen

### 18.1 Löschbericht [E]

Kanonisches JSON (RFC 8785), signiert mit Ed25519 (`acta/report/v1`, `sig_alg`, `key_id`). **Keine** Kennungen von Personen, keine Inhalte. Alle Felder Pflicht, sofern nicht vermerkt.

|Feld|Inhalt|
|---|---|
|`report_version`, `tenant`, `erasure_id`|Kennzeichnung (`TenantId`, `ErasureId`)|
|`trigger`, `scope_kind`|Auslöser (8.2) und Ziel (`Subject`, `Document`, `Version`, `Expired`, `Takedown`)|
|`ledger_ref`|Eintragshash im E-Ledger|
|`times`|`requested`, `planned`, `gated`, `purged`, `tombstoned`, `rotated`, `backups_expired`, `measured`|
|`classes`|je G, B, K, M: `reached` / `not_reached` / `not_applicable` mit Zeitpunkt und Belegverweis; Zwischenstände `key_tombstoned`, `K_pending_backups`|
|`scope_result`|`complete` / `partial_restricted` / `pending`; getrennte Zählung von Zielmenge und zurückgestellten Versionen|
|`restricted`|Grundcode (`Retention`, `LegalHold`, `Objection`, `Contested`, `MultiSubjectReview`, `Precautionary`); Aussage „eingeschränkt, nicht gelöscht“|
|`measurements`|je Schicht: Verfahren, Art (`complete`, `sample`, `declared`, `not_measured`, `not_implemented`), Positivkontrolle (`passed`/`failed`), Ergebnis, Protokollhash|
|`co_subjects`|Anzahl und Modus je Dokument (8.9), nie Kennungen|
|`registry_mirrored`, `backup_exposure_until`|Spiegelstand der Senke; Ende des Sicherungsfensters|
|`anchor`|`assurance`, Vernichtungsfenster, Hash des `DestroyProof`|
|`synthetic_subjects`|Zähler der Canaries|
|`assumptions`, `residual_risks`|A-* und R-* aus 20.3|
|`crypto`|`alg`, `sig_alg`, `key_id`|
|`generated_at`, `signature`|Zeit (Port `Clock`), Signatur über `acta/report/v1 ‖ kanonische Bytes ohne signature`|

Textformulierungen folgen 1.6, zum Beispiel „Klasse K erreicht am …; Klasse M nicht erreicht: Senke nicht bestätigt“. Ein Bericht entsteht erst mit Eintrag im E-Ledger (C20).

### 18.2 Verzeichnis von Verarbeitungstätigkeiten, Art. 30 (Entwurf) [E]

Je Mandant erzeugt `acta-reports` einen Entwurf: Zwecke (aus Sammlungen), Kategorien betroffener Personen und Daten, Empfänger (`DisclosureLog`, Egress-Freigaben), Fristen (Aufbewahrungsklassen), technische und organisatorische Maßnahmen (Verschlüsselung, Zugriffskontrolle, Löschkonzept, Protokollierung). Der Betreiber ergänzt und verantwortet das Verzeichnis.

### 18.3 DSFA-Entwurf, Art. 35 [E]

Gliederung: Beschreibung der Verarbeitung; Notwendigkeit und Verhältnismäßigkeit; Risiken (aus 2.5); Abhilfemaßnahmen; Restrisiken (R-*, A-*); KI-Besonderheiten (Halluzination, Prompt-Injection, Embeddings, Membership Inference); Mehrbetroffene; Betroffenenrechte. Der Entwurf ersetzt die Bewertung durch den Verantwortlichen nicht.

### 18.4 Weitere Berichte [E]

|Bericht|Inhalt|
|---|---|
|Fristen und Aufbewahrung|fällige Löschungen, `Restricted`, `legal_hold`, offene Anfragen, `RequestDue`|
|Betrieb|Zustände (14.6), Metriken (14.3), Exportintervall, `offboarding_exposure_until`, Hostprüfungen, Deklarationen (A-FS, A-RAM)|
|Messbericht|[Q]-Werte, Goldset-Messungen, Zusagen-Tests (16.5)|
|Kryptobericht|4.7|
|Kopienregister|8.6|
|Wiederherstellung|`unrecoverable_docs`, Horizont, Wiedervernichtungen (8.8)|
|Löschkonzept-Entwurf|12.4|

Alle Berichte nennen ihre Annahmen, tragen keine verbotenen Formulierungen und enthalten keine Kennungen.

### 18.5 Schnittstelle [E]

HTTP/JSON über TLS, Authentifizierung OIDC oder mTLS (13.4), Streaming über Server-Sent Events, Versionskennung in der URL (`/v1/…`). Jede Operation prüft Rolle, Sichtklasse und Mandant vor jeder Verarbeitung.

|Bereich|Operationen|
|---|---|
|Wissen|`ingest` (K und P), `commit`, `branch`, `tag`, `diff`, `merge`, `retire`, `checkout`, `document.retire`|
|Abfrage|`query` (mit `as_of`, `valid_at`, `case_id`), `explain`, `replay`, `session.*`|
|Fachfunktionen|`ff.answer`, `ff.case_assist`, `ff.checklist`, `ff.draft`, `ff.deadline`, `ff.radar`, `ff.compare`, `ff.rationale`, `ff.knowledge_flow` (FF1 bis FF9, 10)|
|Szenarien|`scenario.create`, `scenario.run`, `scenario.compare`|
|Vorgänge|`case.register`, `case.grant`, `case.close`|
|Betroffene|`request.create`, `request.status`, `subjects.discover`, `export` (Art. 15, 20), `erasure.create`, `erasure.status`, `review.redact`, `review.decide`|
|Evaluation|`eval.run`, `eval.compare`|
|Administration|`tenant.*`, `collection.*`, `retention.set`, `hold.set`, `egress.grant`, `backup.*`, `restore`, `repair`, `report.*`, `support-bundle`, `accept_rollback`|

**Fehlerabbildung:** `Policy` → 403; `Invalid` → 400 oder 422; `Erased` → 410 (nur mit Zugriffsrecht; ohne Recht 404, Z16); `Integrity`/`Crypto` → 503 (Vergiftung); `Transient` → 503 mit `Retry-After`. Fehlerkörper enthalten nur `code()` und eine Korrelations-ID ohne Personenbezug. Ressourcennamen und Antwortzeitklassen unterscheiden unzugängliche und nicht vorhandene Ziele nicht.

---

## 19. Lebenszyklen

Jede Funktion ist vollständig beschrieben: Auslöser, Vorbedingungen, Schritte, Nachbedingungen, Fehlerpfade, Absturzverhalten, Nachweis. `C-nn` verweist auf Anhang C. Eine Funktion ohne Eintrag hier ist nicht spezifiziert und DARF NICHT implementiert werden (0.4).

### L01 Ingest in Ebene P [E]
- **Auslöser:** `ingest` mit Klasse `Personal` oder `Besonders`.
- **Vorbedingungen:** Rolle erlaubt; Mandant `Live`; Pflichtfelder (9.1 Stufe 1); `subject_refs` vorhanden, sonst `Unklassifiziert` (L01-Q).
- **Schritte:** 9.1 Stufen 1 bis 8. Parsing im Worker; Normalisierung; Erkennung; Zuordnung (Schlüssel zuerst, WP-1); Chunking; Embedding; `DocumentKey(v)` und DEK; P-Umschläge; Dokumentkopf; DP-Segmente mit `Derived<T>`; Betroffenenverzeichnis, Handle-Register, Fallindex-Mitgliedschaft; Idempotenzeintrag; **zuletzt** Manifest-Batch (Sichtbarkeit).
- **Nachbedingungen:** Version `Live`; Ableitungen in DP mit Lineage; `VerOrd` vergeben; Ergebnis unter `ingest_id` abgelegt.
- **Fehler:** `Invalid(ResourceLimit)`, `ClassViolation`, Worker-Absturz (Aufruf scheitert, nichts geschrieben), Entropiefehler (kein Umschlag), `Transient(Overloaded)`.
- **Absturz:** C01 bis C04; Wiederholung mit gleicher `ingest_id` liefert dasselbe Ergebnis.
- **Nachweis:** `T-CRY-1`, `T-DERIVED-SEALED`, `T-CRASH-*`.
- **L01-Q Quarantäne:** Ohne Zuordnung: `Quarantine`, nicht lesbar; Sweeper stündlich; nach `quarantine_max` `Restricted(Precautionary)`; nach `quarantine_purge_after` Saga `Expired` (nur die Acta-Kopie, 8.4); Zuordnung innerhalb der Frist macht die Version `Live`.

### L02 Wissenspflege in Ebene K [E]
- **Auslöser:** `ingest` Klasse `Wissen`, `commit`, `merge`, `document.retire`, FF9.
- **Vorbedingungen:** `Validated<Tree>` (N-Validator bestanden); bei `Open` und `Split` Vier-Augen-Freigabe, bei `Split` Anonymitätstest; `source_of_record`; Branch-Regeln.
- **Schritte:** Blobs mit Schlüssel-Salz, Trees, Commit; K-Ledger-Eintrag; Ref-Bewegung (WP-2); Sichtbarkeitsmenge `V(commit)` (Delta); DK-Indexsegmente für neue Blobs; bei Konfigurationsänderung neue `IndexGen` im Hintergrund (L13). Nach `main`: Vier-Augen und, wenn aktiv, Regressionsprüfung (9.7).
- **Nachbedingungen:** Commit unveränderlich; `main` zeigt erst nach vollständigem Indexaufbau auf neue `IndexGen`.
- **Fehler:** Validator-Treffer (`Policy`, Befundcodes ohne Inhalte), Merge-Konflikt, Regression über Grenze, `Waiver` fehlt.
- **Absturz:** C05, C06; unerreichbare Objekte werden durch GC entfernt.
- **Nachweis:** `T-SEP-*`, `T-TT-*`, `T-VCS-ERASE-1`.

### L03 Abfrage und Antwort [E]
- **Auslöser:** `query`, FF1 bis FF4.
- **Vorbedingungen:** Authentifizierung; Mandant; Ratenbegrenzung; bei P-Zugriff `case_id` (höchstens `max_cases_per_query`) und Zuweisung; Generator verfügbar sonst `Degraded(Inference)`.
- **Schritte:** Sichtklasse → Stand → Kandidatenmengen (K: `V ∩ A_K`; P: `A_P`, `Live`, nicht im Gate) → Anfrageaufbereitung → BM25 und Vektorsuche → RRF → Reranker → Kontextaufbau mit Beleg-Kennungen und Anfrage-Token → Generierung mit Epochenprüfung (9.5) → Belegprüfung (9.6) → Antwort oder Enthaltung → `Query`-Entscheidung (Eingabe `Sealed` bei Personenbezug).
- **Nachbedingungen:** Entscheidungsprotokoll im K-Ledger (`DecisionAppended`); keine P-Antwort im Cache; Sitzungsschritt mit Lineage.
- **Fehler:** `Erased` bei Gate-Treffer während der Anfrage, `Transient(Overloaded)`, `Unavailable`, `GroundingViolation`, Strukturfehler (ein Wiederholungsversuch, dann Enthaltung).
- **Absturz:** keine Persistenz außer Protokoll; Sitzungen überleben keinen Neustart; Protokolleintrag nach Ledger-Regel (WP-2).
- **Nachweis:** `T-ACL-*`, `T-ACL-RANK`, `T-GROUND-*`, `T-GATE-*`, `T-DEC-*`.

### L04 Vorgang (Case) [E]
- **Auslöser:** `case.register`, `case.grant`, `case.close`, Fristablauf, Fachverfahren.
- **Vorbedingungen:** `external_ref`, Betroffene, `owner_unit`, Aufbewahrungsklasse; Zuweisung nur durch Fachverfahren oder `TenantAdmin`.
- **Schritte:** Registrieren (Vorgangsobjekt, DEK `n = 0`, Fallindex); Zuweisen (A-Ledger); Befüllen (neue Versionen erweitern den Fallindex); Betreiben (FF2 bis FF5); Schließen (Zuweisungen enden, Fallindex bereinigt, Dokumente nach Aufbewahrungsklasse); Beenden (Saga für alle Dokumente, Vorgang mit dem letzten Dokument vernichtet).
- **Fehler:** unbekannte Betroffene (werden angelegt, WP-1), Zuweisung ohne Recht (`Policy`).
- **Absturz:** Fallindex ist Ableitung und wird aus P neu gebaut.
- **Nachweis:** `T-ACL-CASE`, `T-CASE-*`.

### L05 Betroffenenanfrage und Löschsaga [E]
- **Auslöser:** `request.create`, `erasure.create`, Fristjob, Takedown, Off-Boarding, Ausscheiden Mitarbeitender.
- **Vorbedingungen:** `identity_verified` (außer vorsorgliche Einschränkung); Rolle; bei `Besonders` Standardpfad vorgezogen.
- **Schritte:** 8.2 (Anfrage), 8.3 (S0 bis S9), 8.9 (Mehrbetroffene), 8.5 (Messung), 8.6 (Art. 19), 8.7 (Klassen), 18.1 (Bericht).
- **Nachbedingungen:** Zielmenge je Klasse G, B, K, M `reached` oder `not_reached`; zurückgestellte Versionen `Restricted`; Bericht und E-Ledger-Eintrag; `ScopeKey` vernichtet; Canary unberührt.
- **Fehler:** `Pending(Anchor|Sink|Review|Archive|Backup)`, `SagaStuck`, `MeasureInvalid`, `Poisoned`; Fehlschlag des Gate-`fsync`: Aufruf scheitert, RAM-Sperre bleibt, Mandant vergiftet.
- **Absturz:** C07 bis C13, C18, C20, C21; physische Tatsachen haben Vorrang.
- **Nachweis:** `T-CRASH-*`, `T-GATE-*`, `T-EVID-*`, `T-MULTI-*`, `T-REST-*`, `acta-verify` V1.

### L06 Berichtigung (Art. 16) [E]
- **Auslöser:** `request.create` Art Berichtigung, `AddSubject`, `RedactSplit`.
- **Schritte:** neue Version (neue `VerRef`, neuer `DocumentKey`, neuer DEK), alte Version `Superseded` (nur `restricted_read`), Ableitungen der alten Version bereinigt (5.2 Nr. 6), neue Ableitungen; Mitteilung nach Art. 19 über Ausgangskorb; nach `history_horizon` (bei Löschgrund sofort) Saga auf die alte Version.
- **Fehler:** `Besonders`: `history_horizon = 0`, alte Version sofort Saga.
- **Absturz:** C03, C04, C10, C11.
- **Nachweis:** `T-DEC-*`, `T-GATE-SUPERSEDED`.

### L07 Einschränkung, Widerspruch, Legal Hold [E]
- **Auslöser:** Art. 18, Art. 21, Aufbewahrungspflicht, `legal_hold`, vorsorglich (Trockenlauf-Timeout, Quarantäne).
- **Schritte:** `Restrict(grund)` im Gate-Log (RAM zuerst, `fsync`, Epoche); Ableitungen bereinigt; Daten bleiben unter ihrem Schlüssel; `Release` nur aus `Restricted`, nie aus `Erased`; `legal_hold` nur Rolle `Legal` mit Vier-Augen, Wiedervorlage `hold_review_interval`; Hold blockiert S5.
- **Nachbedingungen:** `restricted_read` zweckgebunden im E-Ledger; bei `Release` Neuaufbau der Ableitungen.
- **Absturz:** C07 bis C09.
- **Nachweis:** `T-GATE-RESTRICTED`, `T-HOLD-*`.

### L08 Takedown und Fristlöschung in K [E]
- **Auslöser:** irrtümlicher Personenbezug, fehlgeschlagener `Split`-Wiederholungstest, gerichtliche Anordnung, `decision_retention` abgelaufen.
- **Vorbedingungen:** Takedown: Vier-Augen (`DataProtection` und `Legal` oder `TenantAdmin`), signiert; Fristlöschung: keine Freigabe, `ArchiveHook.offer`.
- **Schritte:** 6.7: `RedactK` im Gate-Log sofort; DK-Segmente ersetzen; Pack ohne Blob inklusive Salz neu schreiben, `SegmentKey` vernichten, Rotation anfordern; K-Ledger `Redacted` bzw. `Expired`; Commitment im E-Ledger und in der Senke.
- **Absturz:** C22; `Redacted`-Vermerk erst nach Vernichtung des alten `SegmentKey`.
- **Nachweis:** `T-TT-REDACTED`, `T-VCS-ERASE-1`, `T-CRASH-TAKEDOWN`.

### L09 Fristenjob [E]
- **Auslöser:** täglich; `retain_until` erreicht; `payload_retention` und `decision_retention` abgelaufen; `quarantine_purge_after`; `history_horizon`; `access_log_retention`.
- **Schritte:** fällige Ziele bündeln; `Expired`-Saga je Stapel (Batch-Vernichtung); Nutzlastschlüssel vernichten; A-Ledger-Monat mit `Pruned`-Vermerk; Entscheidungen nach L08.
- **Nachweis:** `T-RET-*`, `T-CRASH-*`.

### L10 Wurzelrotation [E]
- **Auslöser:** `T_ROT`, `N_SHRED`, Anforderung (Besonders, 70 % `erase_sla_B`).
- **Schritte:** 4.3 Schritte 1 bis 4 mit `rotation_lock`, Schnappschuss, Neuschreiben ohne Sperre, `vault_freeze` mit Schwanz, atomarer Austausch, `anchor.destroy(g)`, `RootRotated`.
- **Fehler:** Mandant fällt aus: `M_g` bleibt, `RotationOverdue`; Anker nicht erreichbar: `Degraded(Anchor)`.
- **Absturz:** C14, C15, C16.
- **Nachweis:** `T-ROT-*`, `T-CRASH-VAULT-*`, Messung 120 s.

### L11 Tresorexport und Datensicherung [E]
- **Auslöser:** `vault_export_interval`, `data_backup_interval`, `acta-cli backup`.
- **Schritte:** Export unter `VKb(t,e)` ohne Grabsteine, serialisiert über `vault_freeze`; Epochenverwaltung (`M_bk(e)` erzeugen, nach `end(e) + T_BACKUP` vernichten); Datensicherung der Segmente, des Manifests, Gate-Log-Snapshots, Ledger, Verzeichnisse (ohne DP, Caches, Temp, Tresordateien); Exportdateien nach Ablauf mit Scrub entfernen; Kopienregister aktualisieren.
- **Fehler:** `ExportLate`, `BackupOverdue`.
- **Absturz:** C17, C18.
- **Nachweis:** `T-BACKUP-*`, `T-REST-*`.

### L12 Wiederherstellung [E]
- **Auslöser:** `restore` nach Verlust oder Wiederherstellungsübung.
- **Schritte:** 8.8 (Modus `Restoring`, Schritte 1 bis 10); anschließender Neuaufbau der DP-Indizes.
- **Fehler:** `RestoreBeyondHorizon`, `RestoreBehindAnchor`, `Quarantine` (kein Registerbeleg), zurückgerolltes K-Ledger (`accept_rollback`).
- **Absturz:** C21.
- **Nachweis:** `T-REST-*`, Quartals-Restore-Test.

### L13 Modell-, Chunking- und Indexwechsel [E]
- **Auslöser:** neues Modell, neue `chunk_cfg`, neue Normalisierungsvariante.
- **Schritte:** 11.4 Phasen 1 bis 6; DK-Neuaufbau als Hintergrund; DP-Reindex unter Systemzweck `Reindex` (aggregiert protokolliert); alte DP-Generationen sofort entfernt; alte DK-Generationen nach `index_gen_retention`.
- **Fehler:** `QualityDrift`, Regression über Grenze verhindert Aktivierung.
- **Absturz:** Neuaufbau ist verwerfbar; `main` zeigt auf die alte Generation bis zum Abschluss.
- **Nachweis:** `T-TT-*`, `T-GEN-*`.

### L14 Szenario [E]
- **Auslöser:** `scenario.create`, `scenario.run`.
- **Schritte:** 7.3; Szenario-Branch, Änderungen, Fragenset (N-Validator-geprüft), Lauf gegen Basis und Szenario, Vergleich, `merge` oder `retire`; temporärer Index bei geändertem Chunking oder Embedder.
- **Nachbedingungen:** `main` und P unverändert (Z9); Ergebnisse mit Personenbezug `Sealed`.
- **Nachweis:** `T-SCEN-*`.

### L15 Mandant: Anlage und Off-Boarding [E]
- **Anlage:** 12.6 (unter `rotation_lock`); bis Abschluss `Quarantine`; Pflichtparameter erfragt; Canary angelegt.
- **Off-Boarding:** 12.6 Schritte 1 bis 6; Sperre aller Versionen, Export nach Betreiberentscheid, Saga `Expired`, Entfernen aller Segmente und des Tresors mit Scrub, Ledger archivieren oder vernichten, Bericht mit `offboarding_exposure_until`.
- **Absturz:** C27; Fortsetzung anhand der verbleibenden Dateien.
- **Nachweis:** `T-TENANT-*`, `T-OFFBOARD-*`.

### L16 Start, Betrieb, Herunterfahren [E]
- **Start:** 14.1 Nr. 1 bis 9; Fehler führt zu `Quarantine` oder `Poisoned`.
- **Betrieb:** Hintergrundaufgaben 14.2; Zustandsmaschine 14.6.
- **Herunterfahren:** Anfragen abschließen, Journal flushen, Gate-Log synchronisieren, Schlüsselspeicher freigeben (14.8).
- **Absturz:** C24, C25.
- **Nachweis:** Selbsttest, `T-HOST-*`.

### L17 Fachfunktionen FF1 bis FF9 [E]
Eingaben, Verarbeitung, Persistenz und Löschverhalten stehen in Tabelle 10. Gemeinsame Vor- und Nachbedingungen: Zugriffskontrolle (9.4), Gate (8.1), Belegpflicht bei Modellausgaben, Entscheidungsart `Query`, `Tool`, `Review` oder `Release`, menschliche Freigabe für Entscheidungsgrundlagen; `feature_enabled` nur mit Goldset. FF4: Entwurf entsteht als P-Dokument-Version, `Release` nach Freigabe. FF5: deterministisch, `Tool`-Entscheidung. FF6: kein Zugriff auf P. FF9: Kette L02.
- **Fehler:** `Unavailable`, Enthaltung, `GroundingViolation`, Kalender fehlt (FF5: `Invalid`).
- **Nachweis:** `T-FF-*`, Goldsets.

### L18 Mitarbeitende: Ausscheiden und Handles [E]
- **Auslöser:** Ausscheiden, Anfrage eines Mitarbeitenden, `actor_handle_epoch` Ablauf.
- **Schritte:** Saga auf dem `Staff`-Betroffenen; Zuordnung `ActorRef` → Person in P entfernt; Handles in K lösen zu `Erased` auf; Begründungskette fremder Vorgänge bleibt; neue Epoche vergibt neues Handle für neue Commits.
- **Nachweis:** `T-ACTOR-*`.

### L19 Egress-Freigabe und externe Inferenz [E]
- **Auslöser:** `egress.grant`.
- **Vorbedingungen:** Vier-Augen; Dienst, Zweck, Datenklassen, Ablauf, DSFA-Referenz, Auftragsverarbeitungsvertrag, Drittlandprüfung.
- **Schritte:** Eintrag im E-Ledger; Positivliste der Ziele; jeder Aufruf protokolliert (Dienst, Größe, Klasse, Grant-ID); Kopienregister; Ablauf sperrt.
- **Fehler:** P-Inhalt ohne ausdrückliche Freigabe: `Policy`.
- **Nachweis:** `T-EGRESS-*`.

### L20 Supportpaket und Upgrade [E]
- **Support:** `acta-cli support-bundle` erzeugt lokal aus Positivliste; Betreiber gibt frei; Kopienregister; `T-VENDOR-BUNDLE`.
- **Upgrade:** 14.8; Sicherung vorab, Wartungsfenster, Migration vorwärts, Selbsttest; Fehler: `Quarantine` und Wiederherstellung der Vorversion.
- **Absturz:** Migration atomar je Datei (WP-5).

### L21 Vergiftung und Reparatur [E]
- **Auslöser:** `Integrity`/`Crypto` (15.2).
- **Schritte:** kleinster isolierender Bereich geschlossen; Gate-Sperren bleiben; `acta-cli repair` mit Prüfbericht; bei `Tenant`/`Node` Vier-Augen; Neustart allein hebt nicht auf.
- **Nachweis:** `T-POISON-*`.

### L22 Betroffenenverzeichnis und Alias [E]
- **Auslöser:** neuer Betroffener, weitere externe Kennung, `subjects.discover`.
- **Schritte:** Tresoreintrag `SubjectKey`; `SubjectAlias` (Tag → Primär-Tag); Verzeichnissatz unter `cluster_id`; `discover` nur Rolle `DataProtection`, Ergebnis Kandidaten ohne Volltext; `AddSubject` nur nach Review.
- **Nachweis:** `T-ALIAS-*`, `T-MULTI-ADD`.

---

## 20. Rechtliche Einordnung

Dieses Kapitel ordnet ein und **stellt nichts rechtlich fest**. Konformität ist ein Betriebszustand der Organisation (1.6). Fundstellen tragen den Prüfstatus aus Anhang E.

### 20.1 Personenbezug und Rollen [R]

- Verschlüsselte Daten bleiben personenbezogen, solange ein Schlüssel existiert (Art. 4 Nr. 5, Erwägungsgrund 26; EDPB-Leitlinien 02/2025 Rn. 51). Nach Vernichtung ist der **relative** Maßstab anzulegen (EuGH C-582/14 Breyer; C-413/23 P EDPS/SRB, 04.09.2025); ob der Personenbezug für jede Stelle entfällt, ist vor Außenverwendung zu prüfen [V][⚠].
- Vektor-Embeddings aus personenbezogenen Quellen behandelt Acta vorsorglich als personenbezogen (DP, versiegelt, löschbar). Die Rechtslage ist offen [⚠]; die DSK-Orientierungshilfe RAG behandelt die Datenbank als personenbezogenen Verarbeitungsgegenstand ohne Invertierbarkeitsprüfung. Die EDPB-Stellungnahme 28/2024 betrifft Modelle, nicht RAG-Datenbanken (Analogie).
- `PersonalRef`, Ordinale und Commitments sind pseudonym, solange Zuordnungen oder `CommitKey(e)` existieren; danach nach dem relativen Maßstab nicht mehr zuordenbar, mit Restrisiko R-HANDLE, R-ORD, R-HORIZON.
- Löschen ist selbst Verarbeitung (Art. 4 Nr. 2) und braucht eine Grundlage: die Erfüllung der Löschpflicht.
- Verantwortlicher ist der Betreiber (Art. 4 Nr. 7). Zum Hersteller siehe 14.7: Bei dem Zuschnitt ohne Zugriff auf Klartext, Chunks, Vektoren und Schlüssel ist der Hersteller in der Regel weder Auftragsverarbeiter noch Verantwortlicher [⚠]. Art. 25 bindet den Verantwortlichen; eine unmittelbare Herstellerhaftung nach DSGVO besteht nicht (die DSK-Entschließung vom 12.12.2025 fordert sie *de lege ferenda*).
- Produktaussagen unterliegen § 5 UWG und dem Sachmangelrecht (§ 434 BGB) [V]; daher 1.6.

### 20.2 Löschbegriff und Methode [R][⚠]

Die DSGVO enthält keinen Legalbegriff der Löschung und keine Methode. Nach dem Stand der Rechercheprüfung (27.10.2026) hat **weder eine Aufsichtsbehörde noch ein Gericht** Schlüsselvernichtung als Löschung nach Art. 17 anerkannt oder abgelehnt; der EDPB nennt im Blockchain-Kontext Bedingungen (Rn. 51 bis 53, 63, 85, 87, 96, 103, 104). NIST SP 800-88 Rev. 2 erkennt Cryptographic Erase technisch unter Voraussetzungen als Purge an [V]. Das SDM (Baustein 60) nennt kryptografisches Löschen nicht als Methode, verlangt aber Regeln zu Schlüsseln und ihrer Löschung.

**Acta stützt sich nie allein auf Schlüsselvernichtung.** Die Löschung besteht aus drei Säulen: (1) sofortige Lesesperre (Klasse G), (2) physische Bereinigung der Klartext-Strukturen und Ableitungen (Klasse B), (3) Schlüsselvernichtung nach Rotation und Sicherungsfenster (Klasse K), jeweils gemessen (Klasse M). Die Bedingungen für die Tragfähigkeit der Schlüsselvernichtung und ihre Umsetzung:

|Bedingung|Umsetzung|
|---|---|
|Schlüsselgranularität je Betroffenem und je Version|4.2 (`SubjectKey`, `DocumentKey`, DEK je `VerRef`)|
|Vernichtung aller Schlüsselkopien|Tresor ohne Schlüsseljournal, Rotation, Exportepochen, Anker; Core-Dumps und Swap unterbunden (13.1)|
|Kein unverschlüsselter Parallelbestand|Ableitungen DP versiegelt, Caches mit Lineage, `ContextPurge`, Tempdateien gemessen|
|Metadaten ohne Re-Identifizierung|Auffüllung (4.5), keine Klartext-Tags, `ClusterId` abgeleitet, Ordinale ohne Zuordnung|
|Stand der Technik, Algorithmenwechsel|4.7|
|Passphrase-Ableitung|Argon2id nach RFC 9106, Entropieprüfung 80 Bit (4.1, 4.4)|
|Echte Vernichtung|Grabstein, Direct-Read, Scrub, Rotation|
|Wartefristen Cloud-KMS|`anchor_destroy_max`, Bericht weist Frist aus|
|Keine Kompromittierung vor Vernichtung|`mlock`, Sandbox, Bedrohungsmodell 2.5|
|Nachweisbarkeit|Löschbericht, Ledger, Positivkontrolle, `acta-verify`|

Anonymisierung als Erfüllung der Löschpflicht ist umstritten (Instanzgerichte bejahen, EDPB-CEF 2026 skeptisch gegenüber Praxis) und wird von Acta **nicht** als Löschmethode verwendet; `Split` setzt sie nur als Aufnahmebedingung für K voraus (2.3). Bei Aufbewahrungspflicht gilt Einschränkung statt Löschung (Art. 17 Abs. 3, Art. 18). Ein nur gesperrter Schlüssel ist Einschränkung. § 35 BDSG trägt Vektordatenbanken und automatisierte Backups nicht (nur nicht automatisierte Verarbeitung; unionsrechtliche Zweifel).

**Backups.** Nach SDM Baustein 60 und EDPB-CEF 2026 müssen Sicherungen nicht bei jedem Antrag punktuell verändert werden, solange nach einer Wiederherstellung unmittelbar erneut gelöscht wird. Eine feste Behördenfrist gibt es nicht; die Fristen in Anhang A sind Auslegungsentscheidungen dieser Spezifikation. Acta ergänzt die Rücksicherungssperre durch ablaufende Epochenschlüssel, damit Kopien nach `T_BACKUP + backup_epoch` unlesbar sind.

### 20.3 Annahmen und Restrisiken [A]

|ID|Annahme oder Restrisiko|Wirkung|
|---|---|---|
|A-FLASH|Überschreiben erreicht nicht verlässlich die physische Zelle (Wear-Leveling, Dateisystem-Journale)|Klasse K hängt an Rotation, Ankervernichtung und Sicherungsfenster|
|A-RAM|Schlüssel- und Klartextreste in RAM, Swap, Dumps, Root-Zugriff sind nicht auszuschließen|ausgewiesen; T8 ohne Zusage|
|A-ANCHOR|Der Anker vernichtet Generationsgeheimnisse innerhalb `anchor_destroy_max`|Frist geht in Klasse K ein|
|A-CRYPTO|XChaCha20-Poly1305, HKDF, HMAC, SHA-2, Ed25519, Argon2id halten|Neubewertung 4.7|
|A-EXT|Kopien außerhalb von Acta (Nutzer, Exporte, externe Dienste, Bildschirmfotos, ausgelieferte Streamrahmen) sind nicht erfasst|ausgewiesen|
|A-INFER|Der Inferenzdienst hält seine Zusicherungen (11.2); Modellausgaben sind nicht deterministisch|`not_measured` im Bericht; Z7 gilt ab Abfragevektor|
|A-CLOCK|Zeit ist nicht vertrauenswürdig|Klemmung, `Poisoned(Node, ClockRegression)`|
|A-DETECT|Automatische Erkennung von Personenbezug ist unvollständig|Standardmodus `Whole`, Vier-Augen, Takedown|
|A-ARCH|Ganzzahlarithmetik verhält sich auf x86_64 und aarch64 gleich|Z7|
|A-FS|Das Dateisystem hält `fsync`-Zusagen; `O_DIRECT`-Rücklesen ist möglich; kein Copy-on-Write ohne Deklaration|Selbsttest 13.1|
|R-HANDLE|Handles verknüpfen K-Stellen und Entscheidungen zu einer Person oder Akte|ausgewiesen; `actor_handle_epoch`|
|R-CLUSTER|`ClusterId` gruppiert Datensätze im Abbild, solange `SubjectKey` lebt|nach Vernichtung bedeutungslos|
|R-ORD|Zeitmuster dichter Ordinale|ausgewiesen|
|R-CASE|Aktenzeichen im Vorgang pseudonym bis zur Vernichtung|ausgewiesen|
|R-OUTBOX|Externe Kennung bleibt bis Korb-Abschluss unter `ScopeKey`|ausgewiesen|
|R-OBJID|Wer `K_dedup` und den vermuteten Inhalt besitzt, kann eine Objekt-ID prüfen|ausgewiesen|
|R-HORIZON|Commitments pseudonym, solange `CommitKey(e)` lebt|Frist mit Untergrenze (15.4)|
|ADR-15|`k_idx` wird nicht rotiert; Besitz von `k_idx` und einer Kennung erlaubt die Prüfung, ob eine Person erfasst war|ausgewiesen|

### 20.4 Freigabepflichten vor dem Produktivbetrieb und vor Produktzusagen [R]

Vor Außenverwendung einer Aussage und vor jeder Freigabe sind mit Fachanwalt oder Datenschutzfunktion zu klären: (1) Qualifikation der Schlüsselvernichtung (Löschung, Anonymisierung, Unzugänglichmachung, Einschränkung) und Haltung der zuständigen Aufsicht am Sitz des Betreibers; (2) relativer Personenbezug nach Vernichtung für jede Stelle; (3) Wirksamkeitsbeweis (Penetrationstest, Audit der Schlüsselvernichtung, KMS, HSM, RAM, Swap, Dumps, Backups); (4) Umfang (Quelle, Chunks, Vektoren, Indizes, Caches, Protokolle, Prompt-Historie, Kontexte externer Dienste); (5) Embeddings; (6) Backups (Frist, Wiedereinspielsperre, Backup-Schlüssel, Haltung der Aufsicht); (7) Nachweis-Metadaten und Dauer (Art. 5 Abs. 2 gegen Rückstandslosigkeit; AG Lörrach; OLG Dresden 4 U 940/24); (8) Rolle des Herstellers (Support, Telemetrie, Updates, Schlüsselwiederherstellung, Inferenzanbindung); (9) Aufbewahrungspflichten und Archivrecht; (10) Wortwahl nach UWG und BGB; (11) KI-Verordnung; (12) NIS2 und CRA; (13) DSFA; (14) Aktualität (EDPB-Anonymisierungsleitlinien, Neufassung SDM-Baustein 60, Digital Omnibus, Rechtsprechung zu Embeddings und Schlüsselvernichtung). Zusätzlich: Kryptobericht (4.7), Messbericht (16.5), unabhängige kryptografische Prüfung, Penetrationstest, Wiederherstellungsübung.

### 20.5 Zuordnung zu Betreiberpflichten [R]

|Norm|Pflicht des Verantwortlichen|Unterstützung durch Acta|
|---|---|---|
|Art. 5 Abs. 1 lit. c, e|Datenminimierung, Speicherbegrenzung|Aufbewahrungsklassen, `history_horizon`, `payload_retention`, Fristenjob, Auswahl der Ingest-Dokumente|
|Art. 5 Abs. 2, 24|Rechenschaft|Löschbericht, Ledger, Entscheidungsprotokoll|
|Art. 12|Fristen, Identitätsprüfung|`due_at`, `RequestDue`; Prüfung beim Betreiber|
|Art. 13, 14|Informationspflichten|Beschreibung der Verarbeitung (18.2); Betreiberakt|
|Art. 15|Auskunft|8.2, Schwärzung Mitbetroffener|
|Art. 16|Berichtigung|Versionen, L06|
|Art. 17|Löschung|Saga 8.3, Klassen 8.7|
|Art. 18|Einschränkung|`Restricted`, L07|
|Art. 19|Mitteilung|Ausgangskorb 8.6|
|Art. 20|Übertragbarkeit|signierter Export|
|Art. 21|Widerspruch|`Restricted(Objection)`|
|Art. 22|automatisierte Einzelentscheidung|Nicht-Ziel; `Review` mit Pflichtreviewer|
|Art. 25|Technikgestaltung, Voreinstellung|Entkopplung K/P, Fail-Closed, Typzwang, `Whole` als Standard|
|Art. 28|Auftragsverarbeitung|kein Herstellerzugriff (14.7), Egress-Prüfung (13.3)|
|Art. 30|Verzeichnis|18.2|
|Art. 32|Sicherheit, Wirksamkeitsprüfung|Kapitel 4, 13, 14, 16|
|Art. 33, 34|Meldung von Verletzungen|Alarme `Poisoned`, Integritätsberichte; Meldung ist Betreiberakt|
|Art. 35|DSFA|18.3|
|Art. 44 ff.|Drittlandtransfer|kein Egress im Standard (13.3)|

### 20.6 KI-Verordnung, NIS2, Cyber Resilience Act [R][V]

- **KI-VO (VO (EU) 2024/1689).** Ein RAG-System ist ein KI-System (Art. 3 Nr. 1). Die Risikoklasse hängt vom Einsatz ab (Anhang III, zum Beispiel Personalwesen, Zugang zu wesentlichen öffentlichen Leistungen); sie ist vom Betreiber vor Einsatz zu prüfen. Bei Hochrisiko: Ereignisprotokolle (Art. 12), menschliche Aufsicht (Art. 14; technisch `Review`), Betreiberpflichten (Art. 26; Aufbewahrung der Protokolle mindestens sechs Monate, soweit Unionsrecht, insbesondere Datenschutzrecht, nichts anderes vorsieht), für bestimmte öffentliche Stellen Grundrechte-Folgenabschätzung (Art. 27). Transparenz (Art. 50) und KI-Kompetenz (Art. 4) liegen beim Betreiber. Ob Art. 10 auf eine RAG-Wissensbasis anwendbar ist, ist offen [⚠]. Nach den Rechercheangaben verschiebt der Digital Omnibus (VO (EU) 2026/1744) Pflichten für eigenständige Hochrisiko-Systeme (Anhang III) auf 02.12.2027 [V]. Das Entscheidungsprotokoll (7.1) ist ein **Ereignisprotokoll**, kein Klartextprotokoll aller Ein- und Ausgaben, und lässt sich mit der Löschpflicht vereinbaren.
- **NIS2 (RL (EU) 2022/2555).** Art. 21 Abs. 2 lit. d, h (Lieferkette, Kryptografie) gilt für wesentliche und wichtige Einrichtungen und gelangt über die Lieferkette zum Hersteller. Krypto-Agilität folgt aus Art. 32 DSGVO, EDPB 02/2025 Rn. 85, 96 und BSI-Empfehlungen, nicht aus dem NIS2-Text. Die nationale Umsetzung ist zu prüfen.
- **CRA (VO (EU) 2024/2847).** Meldepflichten ab 11.09.2026, Hauptpflichten ab 11.12.2027; Anwendbarkeit auf den Hersteller prüfen (13.6).

### 20.7 Verifikation der Fundstellen [V]

Jede mit [V] oder ◐/○ markierte Fundstelle ist vor Außenverwendung am Primärtext zu verifizieren (Volltext, Datum, Aktenzeichen, amtliche Fassung). Anhang E führt Prüfstatus. Der Betreiber ergänzt Prüfdatum und Prüfer. Quartalsweise Aktualisierung der Beobachtungsliste: EDPB-Anonymisierungsleitlinien, Endfassung der EDPB-Leitlinien 01/2025, Neufassung SDM-Baustein 60, Digital Omnibus, Rechtsprechung zu Embeddings und Schlüsselvernichtung, Rechtsmittel gegen AG Lörrach. Rechtsfolgen ohne Fundstelle werden nicht in Berichte übernommen.
---

## Anhang A – Parameterverzeichnis

**Pflicht** = kein Standardwert; der Start verweigert den Betrieb ohne Wert (15.4). Alle Startwerte sind [Q] und werden nach Messbericht (16.5) bestätigt oder mit ADR geändert. Zeitangaben in UTC; Größen in Zweierpotenzen.

### A.1 Schlüssel, Tresor, Sicherung

|Parameter|Startwert|Bereich|Abschnitt|
|---|---|---|---|
|`T_ROT`|12 h|1 bis 24 h|4.3|
|`N_SHRED`|10 000|100 bis 1 000 000|4.3|
|`rotation_min_gap`|15 min|1 min bis `T_ROT`|4.3|
|`rotation_parallel`|4|1 bis 16|4.3|
|`backup_epoch`|24 h|≤ `T_BACKUP`|4.3|
|`T_BACKUP`|**Pflicht**, Start 14 Tage|7 bis 35 Tage|4.3, 8.7|
|`vault_export_interval`|1 h|bis 4 h|4.3|
|`data_backup_interval`|24 h|bis 7 Tage|14.4|
|`data_backup_retention`|= `T_BACKUP`|≤ `T_BACKUP`|14.4|
|`anchor_destroy_max`|**Pflicht**|je Anker|4.4|
|`crypto_review_interval`|12 Monate|3 bis 12 Monate|4.7|
|`clock_skew_max`|5 min|1 bis 30 min|4.6|
|`max_subjects_per_doc`|64|2 bis 256|4.2|
|`dek_cache_max`|4 096 Einträge|256 bis 65 536|3.6|
|Argon2id (`FileAnchor`)|m ≥ 65 536 KiB, t ≥ 3, p ≥ 4|nur heraufsetzbar|4.1|
|Passphrase-Entropie|≥ 80 Bit|nur heraufsetzbar|4.4|

### A.2 Ledger, Gate, Senke, Commitments

|Parameter|Startwert|Bereich|Abschnitt|
|---|---|---|---|
|`checkpoint_every`|1 024|64 bis 65 536|6.2|
|`commit_epoch`|30 Tage|7 bis 90 Tage|6.2|
|`commit_key_retention`|90 Tage|≥ `T_BACKUP + backup_epoch`, ≤ 3 Jahre|6.2, 15.4|
|`erase_registry_sink`|**Pflicht**|`DirSink` oder Netzsenke|6.2|
|`access_log_retention`|**Pflicht**|–|6.2, 13.5|
|`ops_log_retention`|90 Tage|7 bis 365 Tage|13.5|
|Gate-Log-Verdichtung|wöchentlich|täglich bis monatlich|5.3|
|`gate_propagation_max`|1 Rahmen|fest|9.5|

### A.3 Speicher

|Parameter|Startwert|Bereich|Abschnitt|
|---|---|---|---|
|`wal_max_bytes`|64 MiB|4 MiB bis 1 GiB|5.1|
|`wal_max_age`|30 s|1 s bis 10 min|5.1|
|`segment_max_bytes`|256 MiB|16 MiB bis 1 GiB|5.1|
|`cluster_seg_min`|4 MiB|256 KiB bis 64 MiB|5.1|
|`dp_cluster_min_bytes`|1 MiB|64 KiB bis 16 MiB|5.5|
|Segmentblock (Klartext)|64 KiB|fest je `format_version`|5.1|
|`group_commit_window`|2 ms|0 bis 20 ms|3.6|
|`reader_drain_max`|30 s|5 s bis 5 min|3.6|
|`ingest_idem_ttl`|7 Tage|1 bis 30 Tage|5.4|
|`bridge_threads`|2 × Kernzahl|1 bis 8 × Kernzahl|3.6|
|`bridge_queue_max`|1 024|64 bis 65 536|3.6|
|`min_free_bytes`|1 GiB|64 MiB bis 100 GiB|8.3|

### A.4 Löschkern

|Parameter|Startwert|Bereich|Abschnitt|
|---|---|---|---|
|`erase_return_timeout`|2 s|100 ms bis 10 s|8.3|
|`erase_sla_B`|24 h|1 h bis 72 h|8.7|
|`multi_review_max`|10 Werktage|1 bis 20|8.9|
|`measure_retries`|3|1 bis 10|8.5|
|`saga_retry_max`|8 (Backoff 1 s bis 1 h)|1 bis 100|8.3|
|`quarantine_max`|24 h|1 bis 72 h|2.2|
|`quarantine_purge_after`|7 Tage|2 bis 30 Tage|2.2|
|`history_horizon`|`Personal` 30 Tage; `Besonders` 0|0 bis 90 Tage; `Besonders` fest 0|6.6|
|`hold_review_interval`|90 Tage|30 bis 365 Tage|8.4|
|`archive_wait_max`|30 Tage|7 bis 180 Tage|8.4|
|`outbox_ttl`|90 Tage|30 bis 365 Tage|8.6|
|`backup_derived_dp`|false; bei `Besonders` fest false|false/true|5.5|

### A.5 Wissensebene und Entscheidungen

|Parameter|Startwert|Bereich|Abschnitt|
|---|---|---|---|
|`branch_retention`|90 Tage|7 Tage bis unbegrenzt|6.4, 6.7|
|`K_MIN`|5|≥ 5|7.5|
|`split_retest_interval`|12 Monate|3 bis 36 Monate|2.3|
|`decision_retention`, `payload_retention`|**Pflicht** je `DecisionKind`|–|7.1, 12.7|
|`actor_handle_epoch`|12 Monate|3 Monate bis aus|7.4|
|`index_gen_retention`|= `branch_retention`|–|11.4|
|`merge_main_four_eyes`|an|an/aus|6.4|
|`merge_main_eval_gate`|an|an/aus|9.7|

### A.6 Abruf und Antwort

|Parameter|Startwert|Bereich|Abschnitt|
|---|---|---|---|
|`k_text`, `k_vec` (K)|100, 100|10 bis 1 000|9.4|
|`k_text_p`, `k_vec_p` (P)|50, 50|10 bis 500|9.4|
|`rrf_k`|60|1 bis 1 000|9.4|
|`fuse_keep`, `rerank_keep`|40, 12|je 5 bis 200|9.4|
|`max_chunks_per_doc`|3|1 bis 10|9.4|
|`neighbor_expand`|1|0 bis 3|9.4|
|`overlap_min`|0,30|0,1 bis 0,8|9.4, 9.6|
|`support_min`|**Pflicht** (Messbericht)|–|9.6|
|`max_cases_per_query`|5|1 bis 20|9.4|
|`answer_cache_ttl`|24 h|0 bis 7 Tage|9.5|
|`cache_personal`|false|fest false|9.5|
|`session_ttl`|8 h|15 min bis 24 h|9.5|
|`rate_limit_per_min`|**Pflicht** je Rolle|–|9.4|
|`view_class_ttl`|60 s|1 bis 300 s|13.4|
|`token_max_age`|1 h|5 min bis 1 h|13.4|
|`chunk_cfg` v1|300 / 450 / 60 Wörter, Überlappung 40|siehe 9.1.1|9.1.1|
|BM25|`k1` 1,2; `b` 0,75|fest je `retrieval_cfg`|9.2|
|Regressionsgrenzen|Recall@10 −0,02; Zitiergenauigkeit −0,02; Enthaltungsgenauigkeit −0,03; Latenz p95 +20 %|enger setzbar|9.7|
|`goldset_interval`|7 Tage|1 bis 30 Tage|11.4|

### A.7 Ingest und Worker

|Parameter|Startwert|Abschnitt|
|---|---|---|
|Eingabegröße|512 MiB|9.1|
|CPU-Zeit je Dokument|60 s|9.1|
|Speicher des Workers|2 GiB|9.1|
|Expansionsfaktor|100:1|9.1|
|Verschachtelungstiefe|8|9.1|
|Seiten|10 000|9.1|
|Rahmen im Worker-Protokoll|höchstens 64 MiB|9.1|

Worker-Limits sind herabsetzbar, nicht heraufsetzbar ohne ADR.

### A.8 Inferenz

|Parameter|Startwert|Abschnitt|
|---|---|---|
|`infer_timeout`|Embedding 10 s, Rerank 10 s, Generierung 120 s|11.1|
|Wiederholungen|höchstens 1|11.1|
|`inference_budget`|**Pflicht** (Tokens je Stunde je Mandant)|11.5|
|`max_parallel_generations`|je Profil|11.5|
|`infer_queue_max`, `embed_queue_max`|256, 1 024|11.5|

### A.9 Betrieb

|Parameter|Startwert|Abschnitt|
|---|---|---|
|RTO|4 h (ohne DP-Neuaufbau)|14.6|
|RPO Schlüssel|≤ `vault_export_interval`|14.6|
|RPO Daten|≤ `data_backup_interval`|14.6|
|Restore-Test|quartalsweise|14.4|
|`ExportLate`|Export älter als 2 × `vault_export_interval`|14.3|
|`RequestDue`|70 % der Frist|8.2|
|`SlaBreach`|`erase_sla_B` überschritten|14.3|

---

## Anhang B – Zustandsautomaten

### B.1 Löschsaga

|Von|Ereignis|Nach|Bedingung|
|---|---|---|---|
|(neu)|Anfrage|`Requested`|Ledger-`fsync`|
|`Requested`|Trockenlauf und Plan fertig|`Planned`|8.3 S1|
|`Requested`|Trockenlauf nicht rechtzeitig|`Planned` mit allen Versionen `Restricted(Precautionary)`|`erase_return_timeout`|
|`Planned`|Konflikt, Aufbewahrungspflicht, Hold|`Restricted(grund)` für betroffene Versionen; übrige weiter|8.4, 8.9|
|`Planned`|Sperre durabel|`Gated`|Gate-Log-`fsync`|
|`Gated`|Bereinigung vollzogen|`Purged`|S3|
|`Purged`|Messung B gültig|`PlaintextMeasured`|Positivkontrolle bestanden|
|`PlaintextMeasured`|Schlüssel vernichtet|`KeyTombstoned`|S5|
|`Gated`|`Besonders` oder SLA ≥ 70 %|S5a vor S3 (vorgezogener Pfad)|8.3|
|`KeyTombstoned`|Rotation und `DestroyProof`|`Rotated`|S6|
|`Rotated`|Sicherungsfenster abgelaufen|`BackupsExpired`|S7|
|`BackupsExpired`|Messung K und M|`Measured`|S8|
|`Measured`|Bericht und Ledger|`Reported`|S9|
|jeder|Voraussetzung fehlt|`Pending(grund)`|Anker, Senke, Review, Archiv, Sicherungsfenster|
|`Pending`|Voraussetzung erfüllt|vorheriger Zustand|–|
|`Restricted`|Frist, Hold weg, Review `DeleteAll`/`RedactSplit`|`Planned`|8.4, 8.9|
|jeder|Integritäts- oder Kryptofehler|`Poisoned(Scope)`|15.2|

Es gibt keinen Zustand `Failed`. Eine gesetzte Sperre wird nur durch `Release` aus `Restricted` aufgehoben, nie aus `Gated`, `Expired` oder `Erased`.

### B.2 Version (P)

|Von|Ereignis|Nach|
|---|---|---|
|(neu)|Ingest ohne Betroffenenzuordnung|`Quarantine`|
|`Quarantine`|Zuordnung und Klassifikation|`Live`|
|`Quarantine`|`quarantine_max`|`Restricted(Precautionary)`|
|`Quarantine`/`Restricted(Precautionary)`|`quarantine_purge_after`|`Expired` (Saga)|
|(neu)|Ingest mit Zuordnung|`Live`|
|`Live`|Berichtigung, `AddSubject`, `RedactSplit`, `RestrictShared`|neue Version `Live`/`Restricted`; alte `Superseded`|
|`Superseded`|`history_horizon` oder Löschgrund|Saga auf die Version|
|`Live`|Aufbewahrungspflicht beim Antrag|`Restricted(Retention)`|
|`Live`|Widerspruch, bestrittene Richtigkeit|`Restricted(Objection|Contested)`|
|`Live`|Mehrbetroffenen-Konflikt|`Restricted(MultiSubjectReview)`|
|`Restricted`|Löschentscheidung, Fristende|`Gated` (über `Planned`)|
|`Live`|Antrag, Fristablauf|`Gated` (über `Planned`) oder `Expired`|
|`Gated`/`Expired`|Schlüsselvernichtung abgeschlossen|`Erased`|

### B.3 Objekt in K

`Live` → `Redacted` (Takedown, Gate `RedactedK`, nie zurück) · `Live` → `Expired` (Fristlöschung eines Entscheidungsprotokolls nach `ArchiveHook`) · unerreichbar → entfernt (GC nach `branch_retention`).

### B.4 Betriebszustand

`Normal` ↔ `Degraded(Anchor|Sink|Inference)` · jeder → `Restoring` (Wiederherstellung) → `Normal` oder `Quarantine` · jeder → `Quarantine` (Registerbeleg fehlt, Selbsttest fehlgeschlagen, Ordinalkonflikt, `RestoreBehindAnchor`, `RestoreBeyondHorizon`) · jeder → `Poisoned(Scope)`. Aus `Quarantine` und `Poisoned` führt nur ein dokumentierter Vorgang mit Prüfbericht (`acta-cli repair`, `accept_rollback`) heraus.

### B.5 Tresor-Eintrag

`aktiv (0x01)` → `Grabstein (0x02)` → bei Rotation nicht übernommen → mit `M_g` unlesbar.

### B.6 Mandant

`Quarantine` (Anlage) → `Normal` → `OffBoarding` (alle Versionen `Restrict(Offboarding)`) → entfernt.

---

## Anhang C – Durabilitäts- und Absturzpunkte

Grundregeln: **Schlüssel zuerst** (WP-1); **Ledger zuerst**, dann Ref-Bewegung (WP-2); **Sichtbarkeit zuletzt** (WP-3); **physische Tatsachen vor Journal** (WP-4); Segmente, Manifeste und Dateien atomar über `rename` und Verzeichnis-`fsync` (WP-5). Jeder Punkt hat einen `T-CRASH-*`-Test.

|ID|Absturzpunkt|Zustand nach Neustart|
|---|---|---|
|C01|Schlüssel durabel, noch kein Umschlag|verwaister Schlüssel; Sweeper vernichtet ihn nach Karenz, wenn kein Bezug entsteht|
|C02|Umschlag im Journal, Segment fehlt|Journalwiederholung, idempotent|
|C03|Segment geschrieben, Manifest fehlt|Segment verwaist, wird bereinigt, nichts sichtbar|
|C04|Manifest durabel, K-Commit fehlt|unsichtbar bis Commit; Ingest mit gleicher `ingest_id` wiederholbar|
|C05|K-Objekte geschrieben, Ref nicht bewegt|unerreichbar; Garbage Collection|
|C06|K-Ledger-Eintrag durabel, Ref nicht bewegt|Start vollzieht die Ref-Bewegung|
|C07|RAM-Sperre gesetzt, Gate-Log-`fsync` fehlt|kein Erfolg gemeldet, Sperre entfällt, Antrag erneut stellen|
|C08|Gate-Log durabel, Rückkehr fehlt|Sperre aktiv, Saga `Gated`; Aufruf mit gleicher `ErasureId` wiederholbar|
|C09|Gate durabel, Senke ohne Eintrag|Senke wird asynchron nachgeholt|
|C10|Ersatzsegment geschrieben, Manifest fehlt|Ersatz verwaist, S3 wiederholt|
|C11|Manifest-Batch durabel, Altdatei nicht entfernt|Altdatei als verwaist mit Scrub entfernt|
|C12|`destroy_many` mitten im Batch|Tresorzustand maßgeblich; Rest wird vernichtet (idempotent)|
|C13|Grabstein geschrieben, Direct-Read fehlt|Verifikation wiederholt|
|C14|Rotation Schritt 2 mitten (Neuschreiben)|neue Datei verworfen, Rotation neu; Tresor g maßgeblich|
|C15|Rotation nach `rename`, Anker nicht vernichtet|Start setzt Schritt 4 fort|
|C16|Anker vernichtet, `RootRotated` fehlt|Start schreibt `RootRotated` nach (Tatsache vor Journal)|
|C17|Tresorexport halb geschrieben|verworfen; letzter vollständiger Export gilt|
|C18|`destroy_backup_epoch` vollzogen, E-Ledger fehlt|Eintrag nachgetragen|
|C19|Checkpoint signiert, `record_head` fehlt|Kopf nachgemeldet|
|C20|Bericht signiert, E-Ledger-Eintrag fehlt|Eintrag nachgeholt; Bericht gilt erst mit Eintrag|
|C21|Restore mitten in der Wiedervernichtung|bleibt `Restoring`, Verfahren neu|
|C22|Takedown: Pack ersetzt, alter `SegmentKey` lebt|fortsetzen; `Redacted`-Vermerk erst nach Vernichtung|
|C23|Gate-Log-Verdichtung mitten|atomar über neue Datei und `rename`; alter Stand gültig, Archivierung wiederholt|
|C24|Journalrotation mitten|alter Journalstand gültig, Rotation wiederholt|
|C25|Journal-Flush: Segmente geschrieben, Altjournal nicht entfernt|Altjournal wird anhand des Manifests als überführt erkannt und mit Scrub entfernt, Journal-`SegmentKey` vernichtet|
|C26|Registersenke: Eintrag geschrieben, `last_acked` fehlt|Eintrag idempotent erneut gesendet|
|C27|Off-Boarding mitten|Zustand `OffBoarding`; Fortsetzung anhand der verbleibenden Dateien; Sperre bleibt|

Zusätzliche Fehlerklassen mit Test: `FsyncFailed` (Datei nicht erneut synchronisieren, neue Datei, sonst Vergiftung), `ENOSPC` (Reserve `min_free_bytes`, Abbruch vor Beginn), zerrissene Schreibvorgänge (Prüfsumme und Kettenmarke).

---

## Anhang D – Testkatalog

|Familie|Gegenstand|Zusage|
|---|---|---|
|`T-GATE-*`|alle Lesepfade nach Sperre; `Superseded`; `Restricted`; Cache; Export; Sitzung; Streaming|Z1|
|`T-CONC-*`|`loom`-Modelle: Sperrreihenfolge, Gate, Rotation gegen Schlüsseloperationen|Z1, Z4|
|`T-CRY-*`, `T-SEAL-*`|Umschläge, AAD-Bindung, Auffüllung, Bytes-Grep|Z2|
|`T-DERIVED-SEALED`|Terme und Embeddings nicht im Klartext auf Platte|Z2|
|`T-META-*`|keine Klartext-Tags, keine exakten Längen|Z2|
|`T-MULTI-*`|Trockenlauf, Modi, Sammelakte, `AddSubject`|Z3|
|`T-CRASH-*`|jeder Punkt C01 bis C27, `T-CRASH-VAULT-*`, `T-CRASH-TAKEDOWN`|Z4|
|`T-SEP-*`, `T-ANON-*`|K ohne Personenbezug, Anonymitätstest, `Validated`|Z5|
|`T-VCS-ERASE-*`|Hashes und K-Ledger unverändert|Z6|
|`T-TT-*`, `T-TT-FX`, `T-TT-REDACTED`|Determinismus, Festkomma, Platzhalter|Z7|
|`T-DEC-*`|Begründungskette, `payload_retention`|Z8|
|`T-SCEN-*`|Szenario-Isolation|Z9|
|`T-NOFALLBACK-*`, `T-RNG-*`|Fail-Closed|Z10|
|`T-EVID-*`|Positivkontrolle, `NotImplemented`|Z11|
|`T-REST-*`|Restore-Schutz, Horizont, Senke, Ankerstand|Z12|
|Compile-Fail|`derived_requires_lineage`, `validated_only_from_pii`, `secret_no_clone`|Z13, Z5, Z10|
|`no_shared_dek`|Eigenschaftstest|Z14|
|`T-LOG-*`|Kennungen in Logs, Ledgern, Fehlern, Metriken|Z15|
|`T-ACL-*`, `T-ACL-RANK`, `T-ACL-CASE`|Zugriff vor Suche, Rangverschiebung, Vorgangsbindung|Z16|
|`T-GROUND-*`|Belegstruktur; Goldset-Messung|Z17|
|`T-EGRESS-*`|kein Egress im Standard, Grant|Z18|
|`T-VENDOR-*`, `T-VENDOR-BUNDLE`|kein Herstellerzugriff, Supportpaket ohne Inhalte|Z19|
|`T-TENANT-*`, `T-OFFBOARD-*`|Mandantentrennung, Off-Boarding|–|
|`T-VAULT-*`, `T-ROT-*`, `T-BACKUP-*`|Tresor, Rotation, Export|–|
|`T-SINK-*`|Senke, `st_dev`, Rückstand|–|
|`T-HOST-*`|Hostprüfungen, Dateisystem|–|
|`T-FF-*`, `T-GEN-*`, `T-RET-*`, `T-HOLD-*`, `T-CASE-*`, `T-ACTOR-*`, `T-ALIAS-*`, `T-POISON-*`|Fachfunktionen, Indexgenerationen, Aufbewahrung, Holds, Vorgänge, Akteure, Aliase, Vergiftung|–|

---

## Anhang E – Fundstellenregister

Prüfstatus aus den Recherchen mit Stand 27.10.2026: ✔ am Primärtext oder Volltext geprüft, ◐ nur über Sekundärquelle oder Leitsatz belegt, ○ allgemeiner Rechtskenntnisstand, nicht verifiziert. **Jede Verwendung außerhalb dieser Spezifikation erfordert Verifikation am Primärtext (20.7).** Prüfdatum und Prüfer trägt der Betreiber ein. Dieser Anhang ist durch `g-word:ignore` vom Wortfilter ausgenommen, soweit er verbotene Formulierungen als Fehlzuordnungen zitiert.

### E.1 Normen und Behördenpapiere

|Fundstelle|Verwendung|Status|
|---|---|---|
|DSGVO Art. 4 Nr. 2, 3, 5, 7, 8; Art. 5, 11, 12, 15 bis 22, 24, 25, 28, 30, 32 bis 35, 44 ff.; Erwägungsgrund 26|Normtext|✔/○|
|EDPB-Leitlinien 02/2025 (Blockchain; v1 04/2025, v2 07.07.2026), Rn. 50, 51 bis 53, 63, 85, 87, 96, 103, 104|Bedingungen bei Schlüsselvernichtung, Notfallplan, Neubewertung (Blockchain-Kontext)|✔|
|EDPB-Leitlinien 01/2025 (Pseudonymisierung)|Pseudonymisierte Daten bleiben personenbezogen; Endfassung nicht verifiziert|◐|
|EDPB-Stellungnahme 28/2024 (17.12.2024)|KI-Modelle, Anonymität|✔|
|EDPB-CEF-Bericht Recht auf Löschung (10.02.2026), Abschn. 4.2.6, 4.2.7|Pseudonymisierung genügt nicht als Löschung; Backups; Verifikation|✔/◐|
|DSK-Orientierungshilfe RAG (V1.0, 10/2025)|On-Premise-Annahme, Embeddings, Adressierbarkeit|✔|
|DSK-Orientierungshilfe KI und Datenschutz (V1.0, 05/2024)|Betroffenenrechte bei KI|◐|
|DSK-Entschließung 12.12.2025|Herstellerverantwortung *de lege ferenda*|✔|
|HmbBfDI-Diskussionspapier LLM (15.07.2024)|Gegenposition für modellinterne Embeddings|✔|
|SDM Baustein 60 (V1.0a, seit 02.09.2020; M60.P01 bis P14, M60.S04 bis S10, M60.D01, D02)|Löschkonzept, Protokolle, Rücksicherung, Schlüssel, Kopien|✔; Neufassung in SDM V3.x prüfen|
|DIN 66398; DIN 66399; ISO/IEC 27555; ISO/IEC 27040; ISO/IEC 27001:2022 A.8.10, A.8.24; BSI IT-Grundschutz CON.6; BSI TR-02102|Löschkonzept, Vernichtung, Kryptoempfehlungen (TR-02102 regelt Verfahren, nicht Löschen)|○|
|NIST SP 800-88 Rev. 2 (26.09.2025)|Cryptographic Erase als Purge unter Voraussetzungen|◐|
|RFC 2119, 5869, 8032, 8785, 9106|Schlüsselwörter, HKDF, Ed25519, JCS, Argon2|Primärtext prüfen|
|§ 35 BDSG|nur nicht automatisierte Verarbeitung|✔|
|§ 147 AO, § 257 HGB, § 630f BGB, BEG IV (BGBl. 2024 I Nr. 323)|Aufbewahrungsfristen|◐|
|§ 5 UWG, § 434 BGB, § 203 StGB|Irreführung, Sachmangel; Schweigepflicht (keine Aufbewahrungsnorm)|○|
|KI-VO (VO (EU) 2024/1689) Art. 3, 4, 10, 12, 14, 19, 26, 27, 50; Digital Omnibus VO (EU) 2026/1744|KI-Einordnung, Protokolle, Aufsicht, Fristen|○/◐|
|NIS2 (RL (EU) 2022/2555) Art. 21; CRA (VO (EU) 2024/2847)|Lieferkette, Kryptografie, Herstellerpflichten|○|
|Bundes- und Landesarchivrecht|Anbietungspflicht (8.4)|nicht recherchiert [V]|
|Herstellerangaben Cloud-KMS (Wartefenster)|`anchor_destroy_max`|nur Indiz, Herstellerdokumentation prüfen|

### E.2 Rechtsprechung

|Entscheidung|Aussage und Relevanz|Status|
|---|---|---|
|EuGH C-582/14 Breyer (19.10.2016); C-413/23 P EDPS/SRB (04.09.2025)|relativer Personenbezug|○|
|EuGH C-131/12 Google Spain (13.05.2014)|Auslistung; nicht zu technischen Löschverfahren|○|
|EuGH C-129/21 Proximus (27.10.2022)|Art. 17 Abs. 2: andere Verantwortliche informieren|○|
|EuGH C-340/21 (14.12.2023)|Nachweis der Angemessenheit von Sicherheitsmaßnahmen|○|
|EuGH C-300/21 (04.05.2023)|Art. 82: keine Erheblichkeitsschwelle|○|
|EuGH C-40/17 Fashion ID (29.07.2019)|gemeinsame Verantwortlichkeit|○|
|EuGH C-460/20 (08.12.2022)|Auslistung bei offensichtlicher Unrichtigkeit|○|
|BGH VI ZR 10/24 (18.11.2024)|Art. 82, Kontrollverlust (Scraping), nicht Löschanspruch|○|
|BGH I ZR 98/23 (27.06.2024)|Aufklärung in der Werbung (Analogie)|○|
|OLG Dresden, 03.02.2026, 4 U 292/25|Anonymisierung als „Minus zur Löschung“ (Leitsatz)|◐|
|LG München II, 14.03.2025, 14 O 1461/24|Anonymisierung als Unterfall der Löschung (Leitsatz)|◐|
|OLG Dresden, 14.12.2021, 4 U 1278/21|Inkasso, Aufbewahrungspflichten je Datum, Zugriffsunterbindung, Schwärzung|◐|
|OLG München, 22.03.2022, 18 U 1697/21 Pre|Löschbegriff autonom; Nicht-Wiederverarbeitung|◐|
|AG Lörrach, 03.03.2025, 3 C 1099/24|kein Anspruch auf individuellen Löschnachweis (erstinstanzlich)|◐|
|OLG Dresden, 4 U 940/24 (Datum offen)|Nachweis der Löschung durch Auftragsverarbeiter|◐|
|OLG Nürnberg, 11.06.2025, 3 U 383/25|Darlegungs- und Beweislast (nur Titel)|◐|

Es ist keine Entscheidung bekannt, die Crypto-Shredding billigt oder ein Löschverfahren wegen technischer Unzulänglichkeit beanstandet (Nicht-Fund, kein Beleg der Nichtexistenz).

### E.3 Häufige Fehlzuordnungen (nicht übernehmen)

|Verbreitete Aussage|Richtig|
|---|---|
|OLG Dresden 4 U 1278/21 begründe „Anonymisierung ist Minus zur Löschung“|Die Formel stammt aus 4 U 292/25; 4 U 1278/21 betrifft Inkasso und Aufbewahrungspflichten. Keine der Entscheidungen äußert sich zu Crypto-Shredding|
|Gerichte sähen Chiffretext als hinreichend anonymisiert|nicht belegt|
|EDPB 01/2025 (Pseudonymisierung) enthalte Aussagen zu Quantencomputing bei Schlüsselvernichtung|diese stehen in 02/2025 (Blockchain)|
|ICO und CNIL erkennen Crypto-Shredding als legitim an|nicht belegt; ICO: „beyond use“ für Backups|
|BSI TR-02102 empfehle kryptografisches Löschen; ISO/IEC 27040 akzeptiere es ausdrücklich|TR-02102 regelt Algorithmen und Längen; Akzeptanz nicht belegt|
|NIST SP 800-88 ohne Revisionsangabe|maßgeblich Rev. 2|
|Landesbehörden tolerierten 30/60/90 Tage Backup-Rotation; „§ 35 BDSG analog“|keine Primärquelle; § 35 BDSG nur nicht automatisiert|
|EDPB 28/2024 stelle Embeddings gesichert als personenbezogen dar|betrifft Modelle; für RAG-Datenbanken offen; HmbBfDI vertritt für modellinterne Embeddings die Gegenposition|
|AG Lörrach: Löschkonzept plus eidesstattliche Versicherung genüge|nicht in der geprüften Berichterstattung enthalten|
|SDM-Baustein 60 „V2.0“ mit fünf Mindestinhalten|Baustein 60 ist V1.0a; die Fünferliste ist Praxis|
|EuGH „C-649/17 (Fashion ID)“|Fashion ID ist C-40/17|
|HGB 6 Jahre, AO 10 Jahre; § 203 StGB als Aufbewahrungsgrundlage|seit 2025 Buchungsbelege 8 Jahre; § 203 StGB ist Schweigepflicht|
|AI Act Art. 12 verlange Klartextprotokollierung aller Ein- und Ausgaben|nur Hochrisiko; Ereignisprotokolle; Art. 26 Abs. 6 verweist auf Datenschutzrecht|
|NIS2 verlange Krypto-Agilität|Begriff nicht im Text|
|DSK-OH RAG nenne Crypto-Shredding oder verlange physische Entfernung|Crypto-Shredding nicht erwähnt|

---

## Anhang F – Entscheidungsverzeichnis (ADR)

|ID|Entscheidung|Begründung|Preis|Revisionstrigger|
|---|---|---|---|---|
|ADR-01|Vier Ebenen K, P, D, E; `PersonalRef` als Zufallshandle|Versionierung und Löschung entkoppeln (Z5, Z6)|P-Dokumente nur über Vorgang abrufbar|–|
|ADR-02|Standardmodus `Whole`; `Split` nur nach bestandenem Anonymitätstest|Erkennung unvollständig; K ohne Personenbezug|weniger Wissen in K|Rechtsauffassung der Aufsicht|
|ADR-03|Zeitreise nur über K; kurze `Superseded`-Kette in P|Nachvollziehbarkeit ohne Personenhistorie|keine historische Rekonstruktion von P|–|
|ADR-04|Sichtbarkeitsmengen je Commit statt Index je Stand|Branches ohne Indexkopien|Filterkosten|Messung|
|ADR-05|XChaCha20-Poly1305, SHA-2, Ed25519; Algorithmus-Byte|kleiner Primitivsatz, kleiner Prüfer|kein FIPS-Algorithmus|Anforderung|
|ADR-06|DEK = HKDF über Dokument- und Betroffenenschlüssel|Mehrbetroffenen-Löschung ohne Wrapping|92 Byte je Schlüssel|–|
|ADR-07|Wurzelrotation mit Generationen|ein Mechanismus, alte Tresorabbilder unlesbar|Ankerfrist|Anker-Messung|
|ADR-08|Append-only Segmentspeicher|Lastprofil, gezielte Löschung|Eigenbau 6 500 LOC|Messbericht Fremdengine|
|ADR-09|DP versiegelt, kein Klartext-`mmap`|Z2|Speicherbedarf|Ladezeit|
|ADR-10|Kern synchron|deterministische Fehlerinjektion|Thread-Pool-Brücke|`erase_return_timeout` unter Last|
|ADR-11|`unsafe` nur in `acta-sys`|kleine Prüffläche|Spitzenleistung|Messung|
|ADR-12|Exakte Flat-Vektorsuche, ANN nur nach Messung|Z7, Löschung per Segmentersatz|Rechenzeit|Latenzziel verfehlt|
|ADR-13|Level-Modell mit `rings.toml`|prüfbare Schichtung|Konfigurationsdatei|–|
|ADR-14|Keine Sammel-Crates|keine Hubs|`From`-Code|–|
|ADR-15|Deterministische Tags aus `k_idx`; Zufall für Handles|Auffinden ohne Zuordnungstabelle|`k_idx`-Besitz erlaubt Existenztest|Rotationsverfahren|
|ADR-16|Handkodierte Plattenformate|Stabilität|mehr Code|–|
|ADR-17|Egress gesperrt, Freigabe vier-Augen|Souveränität|Einrichtungsaufwand|–|
|ADR-18|Nutzer sind Betroffene; `ActorRef`|Rechte für Mitarbeitende|Auflösung über P|–|
|ADR-19|Belegpflicht mit Enthaltung|Präzision|mehr Enthaltungen|Messbericht|
|ADR-20|Schlüssel zuerst im Tresor g, Rotation übernimmt den Schwanz (R-KEY-FIRST)|kein Schlüsselverlust bei Absturz|Rotationsdauer|Messung|
|ADR-21|Klasse K schließt das Sicherungsfenster ein|Exporte enthalten lebende Schlüssel bis Epochenende|spätere `reached`|Aufsichtsaussage zu Fristen|
|ADR-22|Suchstatistik über `V ∩ A` je Sichtklasse|Z16 ohne Seitenkanal|Cache je Sichtklasse|Kosten über Budget|
|ADR-23|Festkomma-BM25, `i8`-Vektoren|Z7 über Plattformen|Qualitätsverlust gemessen|Recall@10 < 0,99|
|ADR-24|`VerRef` und DEK je Version|Einzelvernichtung alter Versionen|Register|–|
|ADR-25|Zwischensperre `Restricted(MultiSubjectReview)` statt Sperre nach Review|Z1 und Z3 vereinbar|vorsorgliche Sperre [⚠]|Rechtsauffassung|
|ADR-26|Tresor-AAD mit `slot`; Index nur im RAM|keine Klartext-Kennung auf Platte|Öffnungszeit|10 Mio. Einträge über 30 s|
|ADR-27|Zielschlüssel nach Bereinigung (Standard), vorab bei `Besonders` oder SLA ≥ 70 %|Nachholbarkeit gegen Dringlichkeit|Fenster bis S5|Fehlbedienungsrate|
|ADR-28|`tx_time` klemmen statt abweisen|keine Lähmung bei kleiner Uhrkorrektur|Flag `clamped`|–|
|ADR-29|`RegistrySink` als Port; `DirSink` im Kern, Netz am Rand|Kern ohne Netz|zweiter Crate|–|
|ADR-30|Betroffene der Nutzlast aus Belegen, Vorgang, Eingabe; harte `payload_retention`|Löschung erreicht Eingaben|Auflösung nötig|Quote nicht auflösbarer Eingaben|
|ADR-31|Journal-Schlüssel für Journal-Nutzlasten K und D als `SegmentKey` des Journals; Chain-Schlüssel aus `k_idx`|stabil über Tresorrotation|zweiter Schlüsselpfad|–|
|ADR-32|`ClusterId` aus `SubjectKey` abgeleitet, nicht gespeichert|kein dauerhaftes Pseudonym im Abbild|Verzeichnis nötig|–|
|ADR-33|Segmentierung nach Partition (K, P, DK, DP) und Lineage je Quellart|Löschlogik nur für DP|vier Segmentarten|–|

**Entscheidungen, die nicht in dieser Spezifikation liegen:** Archiv- und Aufbewahrungsrecht des Mandanten, Wahl der Aufbewahrungsklassen und Fristen, Einstufung nach KI-VO, Zulässigkeit und Dauer pseudonymer Commitments im konkreten Fall, Haltung der zuständigen Aufsicht zur Schlüsselvernichtung, Freigabe externer Dienste. Sie sind Pflichtparameter oder Freigabepflichten (20.4), nicht offene Fragen der Architektur.
