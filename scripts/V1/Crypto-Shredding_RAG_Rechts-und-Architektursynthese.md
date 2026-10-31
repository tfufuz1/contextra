# Löschung personenbezogener Daten in RAG-Systemen durch Schlüsselvernichtung (Crypto-Shredding)

**Rechts- und Architektursynthese: DSGVO, BDSG, EU-KI-Regulierung, technische Standards**
Stand der zugrunde liegenden Prüfung: 27.10.2026. Keine erneute Verifikation bei Erstellung dieser Synthese. Keine Rechtsberatung, kein Gutachten.

---

## 0. Gegenstand, Ergebnis, Legende

### 0.1 Gegenstand

Geprüft wird, ob und unter welchen Bedingungen ein lokal (On-Premise) betriebenes, verschlüsseltes Retrieval-Augmented-Generation-System (RAG) das Vergessen personenbezogener Daten durch Vernichtung kryptografischer Schlüssel rechtlich belastbar umsetzen kann. Rechtsbewertung (Art. 17 DSGVO, Anonymisierung, Nachweis, Rollen, Aufbewahrung, KI-Regulierung) und Technikanalyse (Speicherorte, Vektorindizes, Schlüsselarchitektur, Löschnachweis) sind zusammengeführt. Verbindliches Recht, Behördenauslegung, Literatur/Praxis und ungeklärte Fragen werden durchgehend getrennt.

### 0.2 Ergebnis

1. **Kein Methodenzwang, klarer Maßstab.** Die DSGVO definiert „Löschung“ nicht und schreibt keine Methode vor. Löschen ist selbst Verarbeitung. Behördlicher Maßstab (SDM): Unkenntlichmachen, sodass keine Person mehr identifizierbar und der Informationsgehalt nicht oder nach allgemeinem Ermessen nur unwahrscheinlich reproduzierbar ist.
2. **Schlüsselvernichtung als Löschung: rechtlich offen.** Weder Aufsichtsbehörde noch Gericht hat Crypto-Shredding als Löschung nach Art. 17 DSGVO anerkannt oder abgelehnt. Die Argumentation ist möglich, aber an strenge, dokumentierbare Bedingungen geknüpft (3.5).
3. **Verschlüsselte Daten bleiben personenbezogen, solange irgendein Schlüssel existiert.** Nach Vernichtung *aller* Schlüsselkopien ist der Chiffretext nur unlesbar, bis der Algorithmus gebrochen wird, Entschlüsselungstechniken fortschreiten oder der Schlüssel zuvor kompromittiert wurde (EDPB 02/2025). Restrisiko dokumentieren, periodisch neu bewerten.
4. **Wirkungsbereich entscheidet.** Ein RAG-System hält Daten in mehreren Repräsentationen, darunter unverschlüsselte Such- und Vektorstrukturen im Arbeitsspeicher. Schlüsselvernichtung erfasst diese nicht. Belastbar ist nur die Kombination: physische Bereinigung der Klartext-Strukturen (Ebene A) plus kryptografische Entwertung persistenter Reste (Ebene B).
5. **Embeddings vorsorglich wie personenbezogene Daten behandeln.** Verbindliche Klärung fehlt. Dafür sprechen gemeinsame Speicherung mit Chunk-Text, Verknüpfung über IDs und belegte Rekonstruierbarkeit.
6. **Backups nicht überall sofort, aber nach Restore sofort erneut löschen.** Feste Höchstfrist nennt keine Behörde. Rotationsfristen (30/60/90 Tage) sind Praxis, keine Vorgabe.
7. **Nachweis über Metadaten, nicht Inhalte.** Art. 5 Abs. 2 DSGVO verlangt Nachweisbarkeit. Der Nachweis darf keine Inhalte oder Klartext-Kennungen dauerhaft vorhalten.
8. **Rollen.** Anbieter lokal installierter Software ohne Zugriff auf Klartext, Chunks, Vektoren und Schlüssel ist in der Regel weder Verantwortlicher noch Auftragsverarbeiter. Der Anwender trägt die datenschutzrechtliche Verantwortung. Das Risiko des Anbieters liegt im Wettbewerbs- und Vertragsrecht (Werbeversprechen).
9. **Aufbewahrungspflichten gehen vor, je Datum geprüft.** Dann Einschränkung statt Löschung. § 35 BDSG trägt für automatisierte Vektordatenbanken und Backups nicht.
10. **KI-Regulierung ersetzt die DSGVO nicht.** AI Act verlangt Ereignisprotokolle (nur Hochrisiko), keine dauerhafte Klartextspeicherung von Inhalten. Hochrisiko-Fristen (Anhang III) nach den vorliegenden Angaben auf 02.12.2027 verschoben.
11. **Keine absoluten Löschversprechen.** Ausgewiesen werden darf nur der messtechnisch belegte Zustand mit deklarierten Grenzen (Klassen L/P/C/E, Abschnitt 5.4).

### 0.3 Legende

| Kürzel | Bedeutung |
|---|---|
| **[R]** | Verbindliches Recht: Gesetz, Verordnung, EuGH-/BGH-Urteil |
| **[R-Instanz]** | Instanzgericht: bindet über den Einzelfall hinaus nicht |
| **[B]** | Behördenauslegung (EDPB, DSK, Landesbehörden, SDM): nicht bindend, Maßstab der Aufsichtspraxis |
| **[P]** | Literatur, Praxisempfehlung, technische Norm, Architekturentscheidung |
| **✔** | Am Primärtext/Primärquelle geprüft |
| **◐** | Nur über Sekundärquelle, Leitsatz oder Ausschnitt belegt: Volltext vor Verwendung prüfen |
| **○** | Allgemeiner Rechtskenntnisstand, nicht verifiziert |
| **⚠** | Umstritten oder offen |
| **Ergänzung** | Schlussfolgerung oder Maßnahme dieser Synthese, in keiner Quelle ausdrücklich enthalten |

### 0.4 Vorbehalt

Gesetzestexte, Behördenpapiere und Entscheidungen sind teils nur über Sekundärberichte oder Leitsätze belegt. Vor jeder Außenverwendung (Produkttexte, Verträge, Gutachten) alle Fundstellen am Primärtext verifizieren; Liste in Abschnitt 14.

---

## 1. Technische Ausgangslage

### 1.1 Aufbau

Quelldokumente werden in Chunks zerlegt. Ein Embedding-Modell wandelt Chunks in Vektoren um, die in einer Vektordatenbank mit Näherungssuche indiziert werden (häufig HNSW, Hierarchical Navigable Small World). Ergänzend bestehen oft Volltextindizes (BM25, Posting-Listen, Term-Wörterbücher). Bei Anfrage werden die relevantesten Chunks gesucht und dem LLM als Kontext übergeben. In RAG-Systemen speichert die Vektordatenbank typischerweise Chunk, Tokensequenz und Vektor gemeinsam.

### 1.2 Speicherorte mit eigenem Personenbezug

Jeder Ort braucht eine eigene Löschbegründung.

| Speicherort | Hinweis |
|---|---|
| Quelldokument (Original, Archiv) | ggf. aufbewahrungspflichtig |
| Chunk-Text, Chunk-Metadaten | Dokument-ID, Seite, Zeitstempel, Autor |
| Vektor-Embeddings | im RAM oft fp32 und quantisiert (z. B. SQ8) |
| Vektorindex (Graph) | Kanten, Nachbarschaften, Tombstones |
| Volltextindex | Posting-Listen, Term-Wörterbücher, Segmente |
| Caches | Embedding-, Abfrage-, LLM-Kontext-Caches |
| Protokolle | Abfrage-/Prompt-Historien, Fehlerlogs, Telemetrie |
| Persistente Strukturen | WAL, SSTables, Blob-Segmente, Checkpoints |
| Temporäres | Swap, Crash-/Speicherabbilder, Debug-Dumps, Exporte |
| Backups, Snapshots | Wissensbasis **und** Schlüsselspeicher |
| Schlüsselmaterial | Schlüsselspeicher, HSM, Cloud-KMS, Recovery-/Escrow-Kopien |
| Externe LLM-Dienste | sofern Chunks übermittelt werden |

### 1.3 Warum Löschen technisch schwierig ist

- **Immutable/append-only Strukturen** (WAL, SSTables, Event-Logs): Löschung erst bei späterer Kompaktierung.
- **Tombstones/Soft Delete** in Vektorindizes: Vektor und Kanten bleiben bis Kompaktierung oder Neuaufbau im Index und sind darüber rekonstruierbar („Ghost Vectors“).
- **SSD-Controller:** Wear-Leveling und Over-Provisioning verhindern verlässliches Überschreiben einzelner Zellen.
- **Segment-Merges** im Volltextindex: alte Segmente bestehen zeitweise neben neuen.
- **Replikation/Backups** vervielfachen Kopien außerhalb der Kontrolle des Löschbefehls.

### 1.4 Folgerung

Zwei getrennt zu beantwortende Fragen: (1) Gilt das Verfahren rechtlich als Löschung oder gleichwertige Unkenntlichmachung? (2) Wirkt es technisch tatsächlich und ist es nachweisbar? Die Rechtsfrage hängt an der Technikfrage, weil Löschung nach SDM erst vollzogen ist, wenn keine lesbare Kopie beim Verantwortlichen oder seinen Auftragsverarbeitern mehr besteht.

---

## 2. Löschpflicht und Löschbegriff

### 2.1 Pflicht und Auslöser **[R]**

- Art. 17 Abs. 1 DSGVO: unverzügliche Löschung bei (a) Zweckwegfall, (b) Einwilligungswiderruf ohne andere Grundlage, (c) Widerspruch, (d) Unrechtmäßigkeit, (e) rechtlicher Verpflichtung, (f) Diensten nach Art. 8 gegenüber Kindern. Pflicht besteht auch ohne Antrag (SDM Baustein 60 ✔ **[B]**). Antwortfrist auf Anträge grundsätzlich ein Monat (Art. 12 Abs. 3 ○).
- Hat der Verantwortliche Daten öffentlich gemacht oder weitergegeben: Art. 17 Abs. 2, Art. 19. EuGH C-129/21 (Proximus, 27.10.2022): angemessene Maßnahmen, andere Verantwortliche über das Löschverlangen zu informieren ○. Für RAG relevant bei öffentlich zugänglichen Quelldokumenten oder Chunk-Übermittlung an externe Dienste.
- Ausnahmen: Art. 17 Abs. 3 (Abschnitt 7).

### 2.2 Kein gesetzlicher Löschbegriff, keine vorgeschriebene Methode

- **[R]** Art. 4 Nr. 2 DSGVO führt „Löschen“ und „Vernichtung“ nebeneinander als Verarbeitung. Folgen: (a) Die Löschung unterliegt selbst Art. 5 und Art. 32. (b) Löschen bedeutet nicht zwingend physische Zerstörung des Datenträgers (so österr. DSB, Bescheid 05.12.2018, DSB-D123.270/0009-DSB/2018 ◐ **[B]**).
- **[P]** Herrschende Auffassung: technikneutral.
- **[R-Instanz]** OLG München, Urt. v. 22.03.2022 – 18 U 1697/21 Pre ◐: Löschbegriff autonom auszulegen; Löschanspruch schließt Nicht-Wiederverarbeitung gelöschter Daten ein.

**SDM Baustein 60 „Löschen und Vernichten“ (V1.0a, gültig seit 02.09.2020) ✔ [B]:**

- Löschen = Unkenntlichmachen gespeicherter personenbezogener Daten: keine Person mehr identifizierbar, Informationsgehalt nicht oder nach allgemeinem Ermessen nur unwahrscheinlich reproduzierbar (ErwGr 26).
- Vollzogen erst, wenn auch keine Kopie oder Replikation mehr beim Verantwortlichen oder Auftragsverarbeiter besteht.
- „Vernichten“ = Zerstörung des Datenträgers, unwiderrufliche Form des Löschens.
- Abgestufte Methoden: Datenbank-Löschbefehle mit Reorganisation und Überschreiben, Wipe-Tools, Überschreiben ganzer Datenträger unter Beachtung von Wear-Leveling, physische Zerstörung.
- Nicht ausreichend: einfache Betriebssystem-Löschbefehle, Schnellformatierung, bloße Nutzungsverbote.
- **Kryptografisches Löschen wird nicht als Methode genannt.** Verlangt werden aber Regeln zum Umgang mit Schlüsseln verschlüsselter Daten und zu deren Löschung (M60.P09, P10).
- Auslegungshilfe, keine Rechtsquelle. Aktuelle SDM-Versionen 3.0/3.1 existieren: prüfen, ob Baustein 60 fortgeschrieben wurde ⚠.

### 2.3 ⚠ Umstritten: Kann Anonymisierung die Löschpflicht erfüllen?

| Position | Fundstelle | Status |
|---|---|---|
| Ja: Anonymisierung ist „Minus zur Löschung“ und erfüllt die Pflicht, *wenn die betroffene Person die Entfernung ihres Personenbezugs verlangt* | OLG Dresden, Urt. v. 03.02.2026 – 4 U 292/25 (Verfahren gegen Meta; Leitsatz) | [R-Instanz] ◐ |
| Ja: Anonymisierung zulässiger Unterfall der Löschung | LG München II, Urt. v. 14.03.2025 – 14 O 1461/24, Rn. 27–34 (Leitsatz) | [R-Instanz] ◐ |
| Ja | BfDI-Positionspapier Anonymisierung (2020); österr. DSB 2018 | [B] ◐ |
| Vorbehalt: Praxis gibt häufig bloße Pseudonymisierung, Verfremdung, Maskierung als Löschung aus; genügt nicht. Nachweis nötig, dass Re-Identifizierung nach allgemeinem Ermessen unwahrscheinlich ist | EDPB, CEF-Bericht 2025 zum Recht auf Löschung (angenommen 10.02.2026), Issue 7 / Abschn. 4.2.7 | [B] ✔ |

Kein BGH-/EuGH-Urteil zur Frage bekannt. EDPB-Leitlinien zur Anonymisierung in Vorbereitung (auch im Anschluss an EuGH C-413/23 P) ◐.

Abgrenzung: OLG Dresden, 14.12.2021 – 4 U 1278/21 betrifft Inkasso, namensgleiche Person und das Zusammenwirken von Aufbewahrungspflichten und Löschanspruch (Abschnitt 7.3). Es enthält die „Minus“-Formel nicht und nichts zu Crypto-Shredding.

### 2.4 Speicherbegrenzung, Datenminimierung, Privacy by Design

- **[R]** Art. 5 Abs. 1 lit. e: identifizierbare Speicherung nur solange zweckerforderlich. Lit. c: Beschränkung auf das notwendige Maß. Art. 25 Abs. 2 S. 2 bezieht die Speicherfrist in die datenschutzfreundlichen Voreinstellungen ein.
- **[B] DSK-Orientierungshilfe RAG (10/2025), S. 10, 12 f. ✔:** Bei der Datenaufbereitung nicht erforderliche personenbezogene Daten entfernen oder anonymisieren. Auswahl der in die Vektordatenbank übernommenen Dokumente ist der zentrale Hebel; er reduziert die spätere Löschlast. RAG verringert den Umfang personenbezogener Daten im LLM selbst nicht.
- **[B] EDPB-Leitlinien 02/2025 (v2), Rn. 50, Fn. 23 ✔:** Technische Unmöglichkeit rechtfertigt keine Verstöße (Verweis auf Art. 25 Abs. 1 und ChatGPT-Taskforce-Bericht 05/2024). Aussage steht im Blockchain-Kontext, ist aber allgemein formuliert. Eine Architektur, die granulare Löschung konstruktiv verhindert, ist kaum verteidigbar.

### 2.5 Einschränkung statt Löschung

Art. 4 Nr. 3 DSGVO: Einschränkung = Markierung gespeicherter Daten mit dem Ziel beschränkter künftiger Verarbeitung (Art. 18). Ein nur **gesperrter**, nicht vernichteter Schlüssel ist Einschränkung, keine Löschung. Gleiches gilt für aus dem Suchindex genommene, aber archivierte Dokumente. Bei Aufbewahrungspflichten das richtige Mittel, darf aber nicht „Löschung“ heißen.

---

## 3. Crypto-Shredding: Rechtliche Würdigung

### 3.1 Wirkprinzip

Daten werden von Beginn an verschlüsselt abgelegt. Beseitigung erfolgt nicht durch Überschreiben des Chiffretexts, sondern durch unwiederbringliche Vernichtung des zugehörigen Data Encryption Key (DEK). Der Chiffretext bleibt physisch vorhanden, ist aber ohne Schlüssel unlesbar. Vorteil bei immutablen und flashbasierten Speichern: Es genügt, ein kleines kontrollierbares Objekt zu vernichten statt jede verstreute Kopie zu finden. Voraussetzung: Jede Datenkopie steht unter genau diesem Schlüssel.

### 3.2 Rechtlicher Ausgangspunkt

- **[R]** Verschlüsselung ist Sicherheitsmaßnahme (Art. 32 Abs. 1 lit. a) und kann Pseudonymisierung sein (Art. 4 Nr. 5). Pseudonymisierte Daten bleiben personenbezogen, solange Zusatzinformationen (hier: Schlüssel) existieren (ErwGr 26).
- **[R]** ErwGr 26: maßgeblich sind Mittel, die „nach allgemeinem Ermessen wahrscheinlich“ zur Identifizierung genutzt werden, unter Berücksichtigung von Kosten, Zeitaufwand, verfügbarer Technologie und technologischer Entwicklung.
- **[R]** Relativer Ansatz des EuGH: Personenbezug beurteilt sich aus Sicht der Stelle, die über vernünftigerweise einsetzbare Mittel zur Re-Identifizierung verfügt (C-582/14 Breyer, 19.10.2016; C-413/23 P EDPS/SRB, 04.09.2025; ○ Volltext prüfen). Für den Verantwortlichen, der den Schlüssel hält, bleiben die Daten personenbezogen.
- **Argumentationslinie [P], keine gesicherte Rechtslage:** Sind *alle* Schlüsselkopien unwiederbringlich vernichtet, kann der Chiffretext für jede Stelle ohne vernünftigerweise einsetzbare Mittel nicht mehr identifizierbar sein. Ob als „Löschung“ oder „Anonymisierung“ zu führen: ⚠ offen. Beide Wege führen zur selben Wirksamkeitsprüfung.

### 3.3 Aussagen der Aufsichtsbehörden

| Quelle | Aussage | Reichweite |
|---|---|---|
| **EDPB, Leitlinien 02/2025 Blockchain** (v1 04/2025; v2 Endfassung 07.07.2026), Rn. 51 ✔ | Nach Löschung des Schlüssels sind verschlüsselte Daten unverständlich, *bis* der Algorithmus gebrochen wird, Entschlüsselungstechniken fortschreiten oder der Schlüssel zuvor kompromittiert/kopiert wurde. Verschlüsselte personenbezogene Daten bleiben personenbezogen. Selbst perfekt implementierte Verschlüsselung wird bei unbegrenzter Speicherung von der Zeit überholt | Einzige gefundene EDPB-Aussage zur Schlüsselvernichtung. Für Blockchains formuliert, Übertragung auf RAG = Analogie |
| dto. Rn. 52 ✔ | Gleiche Logik für Hash mit geheimem Schlüssel/Salt | |
| dto. Rn. 63 ✔ | Löschung muss *wirksam* sein; kann je nach Methode und Fakten künftige Identifizierung verhindern | |
| dto. Rn. 85, 96 ✔ | Notfallplan für Algorithmenwechsel; Risikobewertung mit Obsoleszenz und Quantencomputern; periodische Neubewertung | |
| dto. Rn. 87 | Alle Schlüsselkopien erfassen | |
| dto. Rn. 103–104 ✔ | Erasure by Design; „effektiv anonymisieren“ setzt Löschung zusätzlicher (off-chain) Daten voraus, die indirekt identifizieren könnten; verschlüsselte/gehashte Daten nicht auf die Blockchain | |
| EDPB, Leitlinien 01/2025 Pseudonymisierung (angenommen 01/2025; Endfassung nicht verifiziert) | Pseudonymisierte Daten bleiben personenbezogen, solange Zuordnung möglich. Klarer Wirksamkeitstest fehlt (Kritik). **Keine** verifizierte Aussage „Schlüsselvernichtung = Löschung“ | ⚠ Status prüfen |
| EDPB, CEF-Bericht 2025 (10.02.2026) ✔ | Pseudonymisierung/Maskierung genügt nicht als Löschung; vollständige Anonymisierung kann genügen; Löschung verifizieren und nachweisen | allgemein für Anonymisierungsbehauptungen |
| DSK, Landesbehörden | Keine ausdrückliche Aussage zu Crypto-Shredding. SDM verlangt Schlüssel-Handhabungs- und -Löschregeln. DSK-OH RAG erwähnt Crypto-Shredding nicht | ⚠ Lücke |
| ICO (UK) | Backups: „beyond use“ genügt bis Überschreiben möglich. Leitfaden seit Data (Use and Access) Act (19.06.2025) in Überprüfung ◐. Keine verifizierte Aussage zu Crypto-Shredding | UK, nur vergleichend |
| CNIL | Keine belegte Aussage | ○ offen |

### 3.4 Normen und technische Standards **[P]**

Stand-der-Technik-Maßstäbe im Rahmen Art. 32 DSGVO, keine DSGVO-Rechtsquellen. Technische Anerkennung von Cryptographic Erase begründet nicht automatisch die rechtliche Wirkung als Löschung nach Art. 17.

| Standard | Befund |
|---|---|
| NIST SP 800-88 Rev. 2 (26.09.2025, ersetzt Rev. 1) | Cryptographic Erase als Purge-Methode unter strengen Voraussetzungen (Abschn. 3.2, FAQ Q9, Stand 16.07.2026). Fokus verschiebt sich auf Sanitisierungsprogramm mit Verifikation und Validierung. Gerätespezifisch: IEEE 2883:2022. ◐ Voraussetzungen im Einzelnen prüfen. CE ersetzt nicht die physische Unzugänglichkeit des Mediums, sondern ist logische Bereinigung unter Audit/Verifikation |
| ISO/IEC 27040 | Speichersicherheit inkl. Sanitisierung. Ausdrückliche „Akzeptanz“ von Schlüsselvernichtung nicht belegt ○ |
| ISO/IEC 27001:2022, A.8.10, A.8.24 | Allgemeine Kontrollen (Löschung von Informationen, Kryptografie), keine Aussage zu Crypto-Shredding |
| BSI IT-Grundschutz CON.6 | Auswahl geeigneter Lösch- und Vernichtungsverfahren (A4). Fassung prüfen (SDM referenziert Stand 02/2020) |
| BSI TR-02102 | Regelt Verfahren und Schlüssellängen, **nicht** Löschen. Maßgeblich für Algorithmenwahl, keine Löschempfehlung |
| DIN 66399 | Vernichtung von Datenträgern; SDM: Sicherheitsstufe ≥ 4 für personenbezogene Daten. Keine Aussage zu Crypto-Shredding |
| DIN 66398, ISO/IEC 27555 | Löschkonzept: Löschregeln, Löschfristen, Startzeitpunkte |

### 3.5 Bedingungen, unter denen die Argumentation trägt **[P]**

Schlüsselvernichtung ist nur als Unkenntlichmachung vertretbar, wenn geprüft und dokumentiert ist:

1. **Schlüsselgranularität.** Schlüssel deckt genau die zu löschende Einheit ab (Betroffener/Dokument). Schlüssel je Mandant/Collection führt zu Kollateralverlust fremder Daten oder zu unterbliebener Löschung.
2. **Vernichtung aller Schlüsselkopien:** Schlüsselspeicher, KMS, HSM, Sicherungen des Schlüsselspeichers, Escrow/Recovery, Swap, Crash-/Speicherabbilder, Protokolle, Exporte. Ein append-only Schlüsselprotokoll wäre selbst Schlüsselkopie und ist unzulässig.
3. **Kein unverschlüsselter Parallelbestand:** Embeddings, Volltextindex, Caches, Logs, Temp-Dateien, Kontexte externer LLM-Anbieter, es sei denn gesondert und nachweisbar entfernt.
4. **Metadaten ohne Re-Identifizierung.** Verbleibende Dokument-IDs, Größen, Zeitstempel, Schlüssel-IDs, Suchstatistiken dürfen den Personenbezug nicht wiederherstellen. Chiffretexte fester Länge und unveränderte IDs ermöglichen Längenanalyse und Membership-Inference.
5. **Stand der Technik und Algorithmenwechsel** (Art. 32 Abs. 1 lit. a; Algorithmenwahl nach BSI-Empfehlungen): Obsoleszenz-Risikobewertung, periodische Neubewertung, Wechselverfahren.
6. **Passphrase-basierte Schlüsselableitung:** Sicherheit hängt an KDF und Passphrasenstärke; Brute-Force auf das Chiffrat muss praktisch ausgeschlossen sein.
7. **Echte Vernichtung:** Schlüsselmaterial überschreiben, Persistieren auf Datenträgern ausschließen. Entfernen eines Verzeichniseintrags genügt nicht.
8. **Wartefristen bei Cloud-KMS:** Endgültige Vernichtung oft verzögert. Fachangaben (nur Indiz, Herstellerdokumentation prüfen): AWS KMS 7–30 Tage, Azure Key Vault Soft Delete 7–90 Tage, Google Cloud KMS standardmäßig 30 Tage. In dieser Zeit ist die Löschwirkung nicht gegeben (dokumentieren).
9. **Keine Kompromittierung vor Vernichtung.** Wurde der Schlüssel exportiert, kopiert oder in unkontrolliertem System genutzt, trägt die Argumentation nicht.
10. **Nachweisbarkeit** von Schlüssellebenszyklus und Vernichtung (Abschnitt 6).

### 3.6 Würdigung

- Kein Gerichtsurteil und keine Behördenaussage erkennt Crypto-Shredding als ausreichende Löschung an. Gegenteilige Behauptungen (Gerichte sähen Chiffretext als hinreichend anonymisiert) sind nicht belegt und beruhen auf Verwechslung der in 2.3 genannten Entscheidungen. Ebenso wenig existiert eine Entscheidung, die ein Löschverfahren wegen technischer Unzulänglichkeit beanstandet. Beides sind Nicht-Funde, keine Belege der Nichtexistenz.
- Der EDPB-Vorbehalt („bis“ der Algorithmus gebrochen wird) zeigt das Restrisiko: Unlesbarkeit gilt nur nach dem Stand von Kryptografie und Rechenleistung. Folge: Restrisiko dokumentieren, periodisch neu bewerten. Kein Verbot des Verfahrens.
- Für persistente, unveränderliche Speicher (WAL, SSTables, Flash, Backups) ist Crypto-Shredding das sachgerechte Mittel, weil physisches Überschreiben dort unzuverlässig ist. Für Klartext-Strukturen im Arbeitsspeicher und in aktiven Indizes wirkt es nicht: physisch bereinigen.
- Gesperrter, nicht vernichteter Schlüssel = Einschränkung.
- Produktkommunikation muss Verfahren, Voraussetzungen und Grenzen nennen, keine absolute Löschwirkung versprechen (Abschnitt 12).

---

## 4. Embeddings und Restdaten

### 4.1 Sind Vektor-Embeddings personenbezogen? ⚠ Offen

| Aussage | Fundstelle | Wertung |
|---|---|---|
| KI-**Modell** ist nur anonym, wenn direkte (auch probabilistische) Extraktion *und* Gewinnung durch Abfragen jeweils „insignifikant“ wahrscheinlich sind; Einzelfallprüfung | EDPB-Stellungnahme 28/2024 (17.12.2024) ✔ [B] | Betrifft Modelle, nicht die Vektordatenbank eines RAG-Systems. Übertragung = Analogie |
| Vektordatenbank speichert Chunk, Tokens, Vektor gemeinsam; Rechtsgrundlage nötig, weil Embedding-Modell Referenzdokumente verarbeitet und Datenbank sie speichert | DSK-OH RAG S. 5 f., 13 f., 17 f. ✔ [B] | Behandelt Datenbankinhalt als personenbezogenen Verarbeitungsgegenstand, ohne Invertierbarkeitsprüfung |
| Embeddings nicht nachvollziehbar; Membership Inference/Model Inversion berücksichtigen | DSK-OH RAG S. 4, 10 f. ✔ | |
| Rekonstruktion aus Vektordatenbank als Risiko | BayLfD OH KI (03/2026), Rn. 215 ◐ | |
| Tokens/Embeddings *innerhalb eines LLM* sind keine personenbezogenen Daten; bloße Modellspeicherung keine Verarbeitung | HmbBfDI-Diskussionspapier 15.07.2024 ✔ [B] | Gegenposition, für Modellparameter formuliert; nicht auf RAG-Datenbank übertragbar. Laut Literaturfußnote (08/2025) ggf. aufgegeben ◐ |
| Texte aus Embeddings erheblich rekonstruierbar (Inversionsangriffe; z. B. Morris et al., „Text Embeddings Reveal (Almost) As Much As Text“, EMNLP 2023; „Vec2Text“) | Technische Forschung ○ | Belegt technische Möglichkeit, nicht die Rechtslage. Preprint arXiv 2606.18497 (06/2026) zu rekonstruierbaren soft-gelöschten HNSW-Embeddings ◐, ungeprüft |

**Bewertung:** Keine Entscheidung und keine Leitlinie stellt verbindlich fest, dass jedes Text-Embedding personenbezogen ist, und keine verneint es für RAG-Datenbanken. Vorsichtige Praxis: Chunks, Vektoren und Indizes wie personenbezogene Daten behandeln. Entscheidung dokumentieren. Für den Löscherfolg müssen alle Repräsentationen im Wirkungsbereich der Schlüsselvernichtung liegen oder nachweisbar anderweitig entfernt werden.

### 4.2 Backups

- **[B] SDM Baustein 60, S. 2, 5 ✔:** Löschpflicht betrifft grundsätzlich auch Sicherungskopien, aber nicht zwingend zeitgleich in jedem Backup. Fällt der Aufwand integritätswahrender Einzellöschung mit dem planmäßigen Überschreiben zusammen, kann dann gelöscht werden. **In jedem Fall** unmittelbare erneute Löschung nach Rücksicherung (M60.S10) und Überwachungsprozess (M60.P08). Bei hohem Schutzbedarf außerplanmäßige Korrekturläufe (S. 7 f.). Keine Höchstfrist.
- **[B] EDPB-CEF-Bericht (10.02.2026), Abschn. 4.2.6, S. 20 ✔ (Kanzleibericht):** Backups schützen Integrität; Veränderung je nach Setup nicht immer ratsam. Kritik an uneinheitlichen Praktiken, die sich nur auf automatisches Überschreiben oder feste Intervalle stützen. Empfehlung: anerkannte Standards sicherer Löschung, Verifikation und Nachweisbarkeit.
- **[B] LfDI Baden-Württemberg, Best Practices (aus CEF) ◐:** Backups strukturell schützen, z. B. zu löschende Daten durch Zufallszeichen ersetzen; Löschung überprüfen und nachweisen.
- **[B] ICO ◐:** „beyond use“, bis Überschreiben nach etabliertem Plan.
- **⚠ Nicht belegt:** Behördenaussage, Landesbeauftragte (BY, BW, BE) akzeptierten Rotationszyklen von 30/60/90 Tagen. Keine Primärquelle. Frühere strengere Haltung des BayLfD (OH Löschung, 2022) erwähnt ◐. Konkrete Fristen sind **[P]**. Ebenso unbelegt: TTL von 7–14 Tagen für Logs/Caches als Behördenvorgabe (Praxiswert).
- Keine Gerichtsentscheidung zu Fristen/Verhältnismäßigkeit der Backup-Löschung gefunden.
- **[R] § 35 Abs. 1 BDSG trägt nicht** (4.4).
- **Ergänzung (Spannung zu 3.5 Nr. 2):** Enthalten Backups des Schlüsselspeichers noch den (gewrappten) DocDek, ist Klasse C für Backup-Inhalte nicht erreicht. Rechtlich bleibt dort nur „beyond use“ plus Restore-Guard (5.8) bis zum Überschreiben. Gegenmaßnahmen: Schlüsselspeicher aus regulären Backups ausnehmen oder mit eigenem, kurz rotierendem Backup-Schlüssel sichern. Rechtlich nicht abschließend bewertet.

### 4.3 Logs, Caches, temporäre Daten, Indizes

- **[B] SDM S. 2 f. ✔:** Protokolldaten mit Personenbezug unterliegen der Löschpflicht (meist eigene Rechtsgrundlage und Zweckbindung). Temporäre Daten sind zu löschen. „Automatisches Löschen“ von Temp-Dateien ist oft nur Austragen aus dem Dateisystem (M60.P04). Löschen muss ohne Integritätsverlust im Restbestand möglich sein (M60.D01/D02/S04); Kopien sind zu erfassen (M60.P14).
- **[P] RAG-typische Rückstandsquellen:** Tombstones, Segment-Merges, Query-/Prompt-Historien, Embedding-Caches, LLM-Kontext-Caches, Swap/Crash-Dumps, Telemetrie.
- Die DSGVO unterscheidet nicht zwischen Primärdaten und Kopien (Art. 5 Abs. 1 lit. e, Art. 17 Abs. 1) ○.

### 4.4 § 35 BDSG **[R] ✔**

- Abs. 1 S. 1: Einschränkung statt Löschung nur bei **nicht automatisierter** Verarbeitung, unverhältnismäßigem Aufwand und geringem Löschinteresse. S. 3: nicht bei unrechtmäßiger Verarbeitung. Abs. 3: ergänzend bei satzungsmäßigen/vertraglichen Aufbewahrungsfristen.
- ⚠ Unionsrechtliche Zweifel (Art. 23 DSGVO; DSK-Kurzpapier Nr. 11 ◐).
- **Folgerung:** Für RAG-Vektordatenbanken und automatisierte Backups nicht anwendbar. Tragender Grund ist das Tatbestandsmerkmal „nicht automatisiert“, nicht (nur) die Verfügbarkeit granularer Löschung. „§ 35 BDSG analog“ für Backups trägt nicht.

---

## 5. Referenzarchitektur: Zwei-Ebenen-Löschung

Alle Elemente dieses Abschnitts sind **[P]** (Architekturentscheidungen, keine Rechtsvorgaben). Parameterwerte sind Auslegungsentscheidungen und gegen aktuelle Empfehlungen zu prüfen (u. a. BSI TR-02102, RFC 9106 für Argon2).

### 5.1 Ebenen

Reine Schlüsselvernichtung greift zu kurz, weil sie unverschlüsselte Hauptspeicher-Indizes ignoriert. Reines physisches Überschreiben scheitert an SSD-Wear-Leveling, immutablen WAL-Strukturen und Backup-Snapshots.

| | **Ebene A: Klartext-Strukturen (Purge)** | **Ebene B: Persistente Reste (Crypto-Shredding)** |
|---|---|---|
| Verortung | RAM, flüchtige Caches, aktive HNSW-Indizes (fp32/SQ8), BM25-Postings, Term-Wörterbücher | Immutable SSTables, WAL, Blob-Segmente, Checkpoint-Speicher, SSD-Flash |
| Methode | Physische Bereinigung: Zeroizing von Arena-Slots, Segment-Rewrite der Posting-Listen, Cache-Invalidierung | Vernichtung des dokumentspezifischen Schlüssels (DocDek) |
| Begründung | Suchindizes benötigen Klartext zur Traversierung; Schlüsselvernichtung wirkt hier nicht | Überschreiben einzelner Blöcke auf SSD unzuverlässig; Crypto-Shredding wirkt sofort |
| Messung | Struktureller Scan der Speicher-Slots und Dateisegmente; Positivkontrolle per Canary-Record | Schlüsselabwesenheit in KeyTable; fehlgeschlagene Canary-Entschlüsselung |

### 5.2 Schlüsselhierarchie auf Dokumentebene

`KEK_root → KEK_coll → DocDek_doc`

- `KEK_root`: aus Passphrase per **Argon2id** (m ≥ 64 MiB, t ≥ 3).
- `KEK_coll`: je Collection abgeleitet.
- `DocDek_doc`: je Dokument bzw. Betroffenen-Einheit ein eigener 256-Bit-AEAD-Schlüssel. **Alle** zugehörigen Chunks, Vektoren und Blob-Einträge werden unter genau diesem DocDek verschlüsselt (Voraussetzung aus 3.1/3.5 Nr. 1, 3).
- Gewrappter DocDek liegt in veränderbarer `KeyTable` mit **Ersetzen-Semantik**. Löschbefehl: Eintrag mit Nullen überschreiben, `fsync`, Segmentdatei der KeyTable atomar ohne den Schlüssel neu schreiben. Append-only Schlüsselprotokoll unzulässig (wäre Schlüsselkopie).
- Granularität Mandant/Collection scheidet aus: Kollateralschaden oder unterbliebene Löschung.
- Epochen-Rotation (periodische Re-Verschlüsselung unter neuen Schlüsselgenerationen) genügt nicht, wenn alte Schlüsselkopien in Backups oder Hauptspeicher-Dumps überdauern.
- **Ergänzung:** Schlüsselmaterial im RAM nur in gegen Auslagerung gesperrtem Speicher halten, Core-/Crash-Dumps und Swap für den Prozess unterbinden. Sonst entstehen unkontrollierte Schlüsselkopien (3.5 Nr. 2, 7). Bleibende Remanenz im nicht adressierbaren RAM/Swap ist als Restrisiko zu deklarieren.

### 5.3 HNSW-Slot-Zeroing ohne Graph-Rewiring

Vermeidet aufwendiges, nicht deterministisches Neuverknüpfen von Nachbarn:

1. Betroffener Knoten bleibt als reiner Routing-Knoten im Graphen (Topologie erhalten).
2. Nutzdaten des Slots (fp32 **und** SQ8-Codes) sofort im Hauptspeicher mit Nullen überschreiben.
3. Der Knoten enthält danach keine Inhaltsdaten und keine Vektorrepräsentation.
4. Überschreitet der Anteil inaktiver Tombstone-Knoten eine Schwelle (z. B. `tombstone_rebuild_ratio` = 0,2), Index im Hintergrund vollständig neu aufbauen.
5. Arena-Slots erst wiederverwenden, wenn keine veralteten Kantenreferenzen mehr auf sie zeigen.

Tombstone-Löschung ohne Nullung (nur Flag/Bitset) lässt den Vektor rekonstruierbar (Ghost Vector) und genügt dem SDM-Maßstab nicht.

### 5.4 Stufenmodell der Löschwirkung

Statische Garantien („100 % gelöscht“) sind unzulässig. Das System gibt nur die Klasse aus, die messtechnisch belegt ist.

| Klasse | Name | Aussage | Zertifizierungsvoraussetzung |
|---|---|---|---|
| **L** | Logisch gesperrt | Scope für alle Lese- und Suchpfade gesperrt | `ErasedSet`-Eintrag durabel (`fsync`); Lesesperre aktiv |
| **P** | Klartext bereinigt | In aktiven Indizes/Caches (Ebene A) keine Klartextkopie | Ebene-A-Purge vollzogen; Canary-Scan erfolgreich |
| **C** | Persistente Reste unlesbar | Alle Ebene-B-Kopien nur als Chiffrat unter vernichtetem DocDek | Key Destruction vollzogen; Schlüsselabwesenheit gemessen; Canary-Entschlüsselung fehlgeschlagen |
| **E** | Technisch vollständig | P und C lückenlos nachgewiesen, alle Schichten gemessen | `LayerWitness` je Schicht; vollständige Positivkontrolle; eIDAS-Zeitstempel |

Klasse E ist eine technische Aussage innerhalb deklarierter Schranken (5.11), keine rechtliche Feststellung der Löschung nach Art. 17.

### 5.5 Löschsaga

Dauerhafte, zustandsbasierte Saga, die nach Crash/Power-Loss deterministisch fortsetzt (acht Zustände, sieben Übergänge):

`Requested → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured → KeyDestroyed → Measured → Reported/Completed`

- `Frozen`: `ErasedSet`-Eintrag durabel per `fsync`; ab hier Lesesperre für alle Anfragen.
- `PlaintextPurged`: Ebene-A-Indizes bereinigt, Dateiblöcke neu geschrieben.
- `PlaintextMeasured`: messtechnische Bestätigung Ebene A. **Erst danach** darf `KeyDestroyed` laufen. Begründung: Würde der Schlüssel vor der Ebene-A-Bereinigung vernichtet, wären Blöcke mit Daten mehrerer Dokumente nicht mehr geordnet rekonstruierbar.
- **Ergänzung:** Jeder Schritt idempotent, damit Wiederaufsetzen nach Abbruch keine Zwischenzustände erzeugt.

### 5.6 Positivkontrollen

Ziel: Fehlpositive durch defekte Scanner ausschließen.

1. **Canary-Klartext-Scan (Ebene A):** Kontrolldatensatz wird vor dem Scan eingeschleust. Findet der Scanner ihn nicht, ist der Lauf ungültig.
2. **Canary-Entschlüsselungsprobe (Ebene B):** Vor Vernichtung wird ein Test-Chiffrat unter dem DocDek erzeugt. Nach Vernichtung muss die Entschlüsselung nachweislich fehlschlagen. Erst dann gilt Ebene B als bewiesen.

### 5.7 Datensparsamer Nachweis (Lösch-Paradoxon)

Löschnachweis und `ErasedSet` speichern ausschließlich HMAC-Commitments:

`SubjectCommitment = HMAC-SHA256(K_commit[t,e], kanonische Betroffenen-ID)`

- `K_commit` wird je Mandant `t` und Epoche `e` geführt und nach Ablauf der Aufbewahrungsfrist der Löschnachweise vernichtet. Danach sind die Nachweise anonymisiert; die kryptografische Verkettung der Audit-Protokolle bleibt erhalten.
- Ergebnisdokument: signierter, datensparsamer `DeletionProof` über die gemessenen Zustände, mit Positivkontrolle und eIDAS-Zeitstempel.
- **Ergänzung / ⚠:** Solange `K_commit` existiert und die Betroffenen-ID dem Verantwortlichen bekannt ist, ist das Commitment pseudonym und damit personenbezogen. Es braucht Grundlage, Zweck und Frist (Abschnitt 6.4). Zeitliche Constraint: Lebensdauer von `K_commit` ≥ `erased_set_retention`, sonst kann der Restore-Guard Einträge nicht mehr zuordnen; bei Epochenwechsel alle Epochen im Restore-Horizont prüfen.

### 5.8 Backup-Handling, Restore-Guard, Restore-Horizont

Rechtliche Basis: 4.2 (Backups müssen nicht bei jedem Antrag punktuell verändert werden, nach Restore ist sofort erneut zu löschen).

- **Restore-Guard:** Vor dem ersten Lesezugriff nach Backup-Einspielung liest das System zwingend das `ErasedSet` des Zielsystems. Gelöschte Betroffene werden sofort nachgesperrt, wiederhergestellte DocDeks erneut vernichtet, die Saga wiederholt.
- **Restore-Horizont:** Backups/Checkpoints älter als `erased_set_retention` werden verweigert (`RestoreBeyondHorizon`), weil der Guard ihre Bereinigung nicht mehr garantiert. Regelschranke: `backup_max_age ≤ erased_set_retention`.
- Spannung zu Klasse C bei Schlüsselkopien in Backups: siehe 4.2 (Ergänzung).

### 5.9 Aufbewahrungspflicht: Zustand `Restricted`

Bei entgegenstehender gesetzlicher Aufbewahrungspflicht (Art. 17 Abs. 3 lit. b) wechselt das System in `Restricted` (Einschränkung, Art. 18): **keine** Schlüsselvernichtung, **kein** physischer Purge. Das Dokument wird aus dem Retrieval-Index isoliert und im `ErasedSet` als gesperrt markiert; Suchen liefern es nicht mehr. Das Archivdokument verbleibt unverändert im gesperrten Speicher bis Fristablauf. Anschließend reguläre Löschsaga. Bezeichnung „gelöscht“ ist in diesem Zustand unzulässig (2.5).

### 5.10 Protokolle

Ereignis- und Metadatenprotokolle strikt von Inhaltsdaten trennen (keine Prompt-/Ausgabe-Klartexte, keine Klartext-Betroffenen-IDs). So bleiben AI-Act-Aufzeichnungspflichten (Abschnitt 9) und Löschpflicht vereinbar.

### 5.11 Deklarierte Restrisiken

Remanenz im nicht adressierbaren RAM/Swap; Krypto-Obsoleszenz (Algorithmenbruch, Quantencomputer); Kompromittierung des Schlüssels vor Vernichtung; Metadaten-Leakage (Längen, IDs, Statistiken); Schlüsselkopien in Backups bis Überschreiben; Cloud-KMS-Wartefristen; externe Kopien außerhalb der Anwendung (z. B. LLM-Anbieter, Exporte). Notfallplan für Algorithmenwechsel und periodische Neubewertung sind vorzuhalten (EDPB 02/2025 Rn. 85, 96; Krypto-Agilität, Abschnitt 9.3).

### 5.12 Vergleich der Ansätze

| Dimension | Soft Delete (Tombstone) | Pauschales Crypto-Shredding (Scope-Key) | Physisches Überschreiben | Zwei-Ebenen-Architektur |
|---|---|---|---|---|
| Durchführung | Inaktiv-Flag/Tombstone | Vernichtung eines Mandanten-/Collection-Keys | Sektorweises Überschreiben aller Blöcke | Purge Ebene A + DocDek-Vernichtung Ebene B |
| Erreichte Klasse | L | C (blockweise), aber Kollateralschaden; Ebene A unberührt | P; scheitert auf Flash | E (bei lückenlosem Nachweis) |
| Rechtliche Bewertung | Genügt SDM-Maßstab nicht: Daten bleiben rekonstruierbar | ⚠ umstritten; EDPB 02/2025 verweist auf Restrisiken | SDM nennt Überschreiben als Methode, unter Beachtung Wear-Leveling | Kein Urteil/keine Behördenaussage; Argumentation am besten belegbar, Anerkennung nicht belegt |
| SSD/RAG-Eignung | Problemlos, rechtlich wirkungslos | Zerstört bei grober Granularität fremde Daten | Unzuverlässig (Wear-Leveling, HNSW-Graphbrüche), hohe I/O-Latenz | Für HNSW, LSM-Trees, SSD ausgelegt |
| Restrisiken | Inversionsangriffe auf Ghost Vectors | Obsoleszenz, Kompromittierung vor Vernichtung | Zerstörung von Nachbarknoten | Remanenz RAM/Swap (deklariert), Backups |
| Nachweis | Internes DB-Log | KMS-Löscheintrag | Wipe-Tool-Protokoll | Signierter `DeletionProof` mit Positivkontrolle, Zeitstempel |

---

## 6. Löschnachweis und Rechenschaftspflicht

### 6.1 Gesetzliche Anforderungen **[R]**

Art. 5 Abs. 2 (Nachweisbarkeit der Einhaltung der Grundsätze); Art. 24 Abs. 1 (TOM und Nachweis, regelmäßige Überprüfung); Art. 25 Abs. 1, 2 (Technikgestaltung, Voreinstellung einschl. Speicherfrist); Art. 32 Abs. 1 (lit. a Verschlüsselung, lit. d regelmäßige Wirksamkeitsprüfung); Art. 30 Abs. 1 lit. f ○ (Löschfristen im Verzeichnis); Art. 17 Abs. 2, 19 ○. EuGH C-340/21 (14.12.2023) ○: Im Streit muss der Verantwortliche die Angemessenheit seiner Sicherheitsmaßnahmen nachweisen.

### 6.2 Rechtsprechung

- **AG Lörrach, Urt. v. 03.03.2025 – 3 C 1099/24 [R-Instanz] ◐:** Nachbarstreit um Videoaufnahmen. Löschung angeordnet, Anspruch auf *Löschnachweis* abgewiesen: Art. 5 Abs. 2 enthalte nur abstrakte Nachweispflicht, keinen Anspruch der betroffenen Person ohne Streitfall; dauerhaft aufbewahrter Löschnachweis widerspreche zudem der rückstandslosen Löschung (Bericht ppc.land, 14.04.2025). Einschränkungen: erstinstanzlich, kein RAG-Bezug, Volltext (landesrecht-bw.de) prüfen. **Nicht** belegt: Aussage, Löschkonzept plus eidesstattliche Versicherung genüge der Nachweispflicht.
- **Gegenläufig:** OLG Dresden, 4 U 940/24 (Datum offen, ◐): Verantwortlicher muss sich Löschung durch den Auftragsverarbeiter nach Vertragsende nachweisen lassen; Ankündigung genügt nicht. Spricht gegen generelle Entbehrlichkeit von Nachweisen.
- OLG Nürnberg, Hinweisbeschl. v. 11.06.2025 – 3 U 383/25: Darlegungs- und Beweislast beim Löschanspruch (nur Titel bekannt, ◐ ungelesen).
- **[B]** EDPB-CEF 2026: Löschung verifizieren und nachweisen können ✔.

### 6.3 SDM Baustein 60: Anforderungen an Löschkonzept und Nachweis **[B] ✔**

| Maßnahme | Ref. |
|---|---|
| Löschfristen festlegen, im Verzeichnis von Verarbeitungstätigkeiten dokumentieren | M60.P01, P02 |
| Löschkonzept (hilfreich: DIN 66398, Löschregeln mit Löschfrist und Startzeitpunkt) | M60.P03 |
| Berechtigungs-/Rollenkonzept (prüft, ordnet an, führt aus) | S. 6 |
| Löschbarkeit auf Ebene Datenfeld/Datensatz/Gruppe, beherrschbarer Aufwand, ohne Integritätsverlust | M60.D01, D02, S04 |
| Protokollierung von Löschungen **und** datensparsame Ausgestaltung: Protokolle enthalten keine Daten, für die Löschpflicht besteht | M60.S05, S06; S. 7 |
| Fristüberwachung, automatisierte Löschung, Aussetzen automatischer Läufe | M60.S07–S09, P06 |
| Rücksicherung: erneute Löschung, Überwachungsprozess | M60.S10, P08 |
| Regeln zu Verschlüsselungsschlüsseln zu löschender Daten und zu deren Löschung | M60.P09, P10 |
| Dokumentation von Anzahl und Speicherort der Kopien | M60.P14 |
| Löschgrund bei Methodenwahl berücksichtigen | M60.P07 |
| Löschpflichten bei Auftragsverarbeitern vertraglich absichern | M60.P05 |

**[P] Praxis-Löschkonzept** (nicht aus dem SDM): Datenkategorien und Verarbeitungstätigkeiten; Fristen und Auslöser; Verantwortliche/Eskalation; technisches Verfahren je Speicherort; Backup-Sperrung und Rücksicherungsschutz; Löschprotokoll (Zeitpunkt, Umfang, Verfahren, Prüfung; bei Crypto-Shredding zusätzlich Schlüssellebenszyklus und Vernichtungsnachweis); Stichprobenkontrolle (Testabfragen gegen Index und Cache). Spannung Löschpflicht/Protokollpflicht (Art. 32; AI Act Art. 12) löst Datenminimierung in den Logs, nicht Verzicht auf Logs.

### 6.4 Lösch-Paradoxon

Nachweis der Löschung für Person X verlangt scheinbar ein Protokoll mit Identifikator von X. Auflösung nach SDM und AG Lörrach: Nachweis über Metadaten der Löschung (Zeitpunkt, System, Verfahren, Ergebnis, Prüfwert), nicht über Inhalt oder Klartext-Kennungen. ⚠ **Umstritten**, ob und wie lange ein Löschnachweis mit verknüpfbarer pseudonymer Lösch-ID zulässig ist (AG Lörrach dagegen; Verweis auf Art. 11 DSGVO in der Literatur ◐). Technische Antwort: 5.7.

---

## 7. Aufbewahrungspflichten, Berufsgeheimnis

### 7.1 Art. 17 Abs. 3 DSGVO **[R]**

Keine Löschpflicht, soweit Verarbeitung erforderlich zur Erfüllung rechtlicher Verpflichtung oder öffentlicher Aufgabe (lit. b), für Archiv-/Forschungs-/Statistikzwecke (lit. d) oder zur Geltendmachung, Ausübung, Verteidigung von Rechtsansprüchen (lit. e). Statt Löschung kommt Einschränkung (Art. 18) in Betracht ○. Die Vorrangregel gilt nur für die betroffenen Daten, nicht für das ganze System.

### 7.2 Fristen

| Unterlagen | Frist | Norm |
|---|---|---|
| Bücher, Aufzeichnungen, Inventare, Jahresabschlüsse, Lageberichte, Eröffnungsbilanzen | 10 Jahre | § 147 Abs. 1 Nr. 1, Abs. 3 AO; § 257 Abs. 1 Nr. 1, Abs. 4 HGB |
| **Buchungsbelege** (z. B. Rechnungen) | **8 Jahre** seit 01.01.2025 (BEG IV, BGBl. 2024 I Nr. 323); Banken, Versicherungen, Wertpapierinstitute weiter 10 Jahre | § 147 Abs. 3 AO; § 257 Abs. 4 HGB (◐ Sekundärquellen) |
| Handels-/Geschäftsbriefe (auch E-Mails) | 6 Jahre | § 147 Abs. 1 Nr. 2, 3 AO; § 257 Abs. 1 Nr. 2, 3, Abs. 2 HGB |
| Patientenakte/ärztliche Dokumentation | 10 Jahre nach Behandlungsabschluss (soweit nicht länger) | § 630f Abs. 3 BGB; § 10 Abs. 3 MBO-Ä (Stand 2015, nach SDM S. 8) |

§ 203 StGB ist Schweigepflicht, keine Aufbewahrungsnorm; er begrenzt Weitergabe/Zugriff und wird bei Entsorgung und Anbieterzugriff relevant (mitwirkende Personen, Abs. 3, 4 ○). Vereinfachung „6 Jahre HGB / 10 Jahre AO“ ist veraltet.

### 7.3 Wirkung auf RAG-Systeme

- **[R-Instanz] OLG Dresden, 14.12.2021 – 4 U 1278/21 ◐:** Aufbewahrungspflichten sind je *einzelnem Datum* zu prüfen und rechtfertigen keine dauerhafte Speicherung rechtswidrig erlangter Daten. Verantwortlicher muss Datenbestand so organisieren, dass Zugriff auf rechtswidrig gespeicherte Daten unterbleibt. Schwärzung kann Löschhandlung bei Geschäftskorrespondenz sein.
- **[P]** Aufbewahrungspflichtiges *Original* (Archiv) und *RAG-Repräsentationen* (Chunk, Vektor, Index) sind getrennte Verarbeitungen mit eigener Grundlage. Aufbewahrungspflicht rechtfertigt nicht automatisch Weiternutzung im aktiven Retrieval. Praxis: aus Index entfernen/sperren, Original unter Aufbewahrungsregeln archivieren (Zustand `Restricted`, 5.9). ⚠ Wie Behörden die Einschränkung im Index konkret verlangen, ist nicht geregelt.
- **[B]** Bei hohem Schutzbedarf/besonderen Kategorien/Berufsgeheimnissen strengere Lösch- und Backup-Maßnahmen (SDM S. 7 f.).

---

## 8. Rollen, Herstellerverantwortung, Haftung, Werbung

### 8.1 Einordnung des Anbieters

- **[R]** Maßgeblich sind die tatsächlichen Verhältnisse, nicht Vertragsbezeichnungen (Art. 4 Nr. 7, 8; Art. 28 Abs. 10). **[B]** EDPB-Leitlinien 07/2020 zu Verantwortlichem/Auftragsverarbeiter (V2.0, 07.07.2021).
- **[B]** DSK-OH RAG, S. 3 ✔: unterstellt On-Premise-Szenario beim Verantwortlichen; bei Verteilung auf mehrere Betreiber ergeben sich weitere Aspekte.
- **[P] Ergebnis:** Anbieter lizenzierter Software ohne Zugriff auf Klartext, Chunks, Vektoren, Schlüssel und ohne Empfang personenbezogener Daten ist für diese Daten in der Regel **weder** Auftragsverarbeiter (keine Übermittlung, kein Hosting; AVV für reine Softwarebereitstellung nicht erforderlich) **noch** Verantwortlicher (keine Entscheidung über Zwecke und Mittel). Alleiniger Verantwortlicher ist der Anwender.
- **Das ändert sich** bei: Fernwartung/Support mit Datenzugriff, Telemetrie oder Fehlerberichten mit Personenbezug, Hosting, Cloud-Schlüsselverwaltung, Schlüssel-Escrow/-Recovery, Lizenz-/Update-Server mit Nutzerdaten, Weiterleitung an externe LLM-Dienste ⚠ (Einzelfall; Auftragsverarbeitung oder eigene Verantwortlichkeit). Gemeinsame Verantwortlichkeit: EuGH C-40/17 (Fashion ID, 29.07.2019) ○. Key-Escrow/Recovery widerspricht zudem der Schlüsselvernichtung (3.5 Nr. 2).

### 8.2 Herstellerverantwortung

- **[R]** Art. 25 DSGVO richtet sich an den **Verantwortlichen**. ErwGr 78 S. 4 ermutigt Hersteller nur (○ Wortlaut prüfen). EDPB-Leitlinien 4/2019 (V2.0, 20.10.2020) rufen Hersteller zur Unterstützung auf ◐.
- **[B]** DSK-Entschließung 12.12.2025 „IT-Hersteller in die Verantwortung nehmen“ ✔: fordert *de lege ferenda* Herstellerverantwortung auch für Art. 25. Geltendes Recht ist das nicht; eine direkte Herstellerhaftung nach DSGVO besteht nicht.
- Weitere Produktregulierung ○: RAG ist KI-System (Art. 3 Nr. 1 AI Act; DSK-Glossar ✔), Anbieter ggf. „Anbieter“ (Art. 3 Nr. 3). Cyber Resilience Act (VO (EU) 2024/2847): Meldepflichten ab 11.09.2026, Hauptpflichten ab 11.12.2027; Anwendbarkeit prüfen.

### 8.3 Pflichten des Anwenders (Verantwortlicher) **[R]**

Rechtsgrundlage für Referenzdokumente, Chunks, Vektordatenbank (Art. 6, ggf. 9; DSK-OH RAG S. 13 f. ✔); Löschkonzept und -fristen (Art. 5, 17, 30); TOM (Art. 24, 25, 32); Betroffenenrechte (Art. 12 ff.); ggf. Datenschutz-Folgenabschätzung (Art. 35; bei KI-Einsatz laut DSK-Auswertung häufig nötig); Verträge mit Dienstleistern (Art. 28); Drittlandtransfer. Der Verantwortliche muss Eignung des Systems prüfen und dokumentieren (DSK-OH RAG S. 12, 14). Wahl der Löschparameter und rechtliche Bewertung der Löschwirkung liegen beim Anwender, Art. 25 bindet ihn dazu, nur Software zu beschaffen, die Löschung technisch abbilden kann.

### 8.4 Produktversprechen und Werbung (UWG, BGB)

- **[R] ○** Irreführende geschäftliche Handlungen: § 5 Abs. 1, § 5a UWG. Anspruchsberechtigt u. a. Mitbewerber und Verbände (§ 8 Abs. 3 UWG). Unzutreffende Behauptungen von Zertifizierung oder behördlicher Billigung besonders riskant (Anhang zu § 3 Abs. 3 UWG, Nr. 2, 4).
- Aussagen wie „garantiert gelöscht“, „erfüllt Art. 17 automatisch“, „DSGVO-konform“ sind belegpflichtige Tatsachenbehauptungen. Risiko bei Restdaten in Backups, Caches, Logs, Swap oder Schlüsselresten in OS-Strukturen, Vektorindizes ohne Schlüsselschutz.
- **Vertragsrecht ○:** Werbeaussagen können Beschaffenheit vereinbaren (Sachmangel § 434 BGB; Schadensersatz §§ 280 ff. BGB; bei Verbrauchern ggf. §§ 327 ff. BGB). Regress des Anwenders möglich, wenn er wegen unzureichender Software Bußgelder erhält. Immaterieller Schadensersatz nach Art. 82 DSGVO ohne Erheblichkeitsschwelle (EuGH C-300/21, 04.05.2023 ○) erhöht das Risiko bei Löschversäumnissen.
- **Analogie [P]:** BGH, 27.06.2024 – I ZR 98/23 („klimaneutral“) ○: strenge Aufklärungsanforderungen in der Werbung selbst bei mehrdeutigen Aussagen. Übertragbarkeit auf Datenschutzversprechen offen. Rechtsprechung zu Löschversprechen von Software nicht gefunden.

---

## 9. KI-Regulierung und Cybersicherheit

### 9.1 DSK-Orientierungshilfen

- **DSK-OH generative KI-Systeme mit RAG-Methode (V1.0, 10/2025; Pressemitteilung 17.10.2025; 18 S.) [B] ✔:**
  - RAG ändert nichts am LLM-Training; ein rechtswidrig trainiertes Modell bleibt auch im RAG-System rechtswidrig trainiert (S. 3).
  - Informationen in der Vektordatenbank lassen sich im Gegensatz zum im LLM Gelernten „einfach aktualisieren, löschen und beauskunften“ (S. 8).
  - Löschung erfolgt technisch durch Löschung von Datenbankeinträgen und Entfernen aus den Referenzdokumenten; Einträge sind direkt adressierbar (S. 13).
  - **Aber:** Betroffenenrechte in KI-Modellen „weitgehend ungelöst“ und wirken auf RAG zurück; Löschung im Sprachmodell bleibt Problem; Transparenz, Zweckbindung, Betroffenenrechte im Gesamtsystem einzelfallabhängig (S. 13 f.).
  - **Crypto-Shredding wird nicht erwähnt.** Was als ausreichende Löschmethode gilt, lässt die OH offen. Dass Schlüsselvernichtung anerkannte Umsetzung ist, ist ⚠ nicht belegt.
- **DSK-OH „KI und Datenschutz“ (V1.0, 05/2024) ◐:** Betroffenenrechte Rn. 26 ff. Die verbreitete Aussage, KI-Systeme müssten löschen können und Filter seien unzureichender Ersatz, ist nicht verifiziert. Fortlaufende Aktualisierung angekündigt; Überarbeitung 05/2026 (Art. 50 AI Act) nicht bestätigt.
- DSK-OH empfohlene TOM bei KI (06/2025) ◐ nicht ausgewertet. BayLfD OH KI (03/2026; Rn. 178 ff. RAG, Rn. 215 Rekonstruktion) ◐ nicht im Volltext geprüft.

### 9.2 EDPB-Stellungnahme 28/2024 (17.12.2024) ✔ [B]

Anonymität von KI-Modellen im Einzelfall; berechtigtes Interesse als Rechtsgrundlage; Folgen unrechtmäßiger Entwicklung (Aufsichtsbehörden können u. a. Löschung von Teilen des Datensatzes oder erneutes Training anordnen ◐). Nennt Output-Filter und Techniken zum nachträglichen Entfernen/Unterdrücken personenbezogener Daten als Unterstützung der Betroffenenrechte; erwägt Löschrecht auch ohne Voraussetzungen des Art. 17 Abs. 1 (Empfehlung, nicht verbindlich). Die DSK verweist für RAG-Nachnutzung auf Szenario 2 (Rn. 124 ff.) und risikomindernde Maßnahmen (Rn. 96 ff.). Eine EDPB-Aussage speziell zum Recht auf Vergessenwerden bei RAG wurde nicht gefunden.

### 9.3 AI Act (VO (EU) 2024/1689) und Digital Omnibus

- **Einordnung:** RAG ist KI-System (Art. 3 Nr. 1). Hochrisiko nur nach Einsatz (Anhang III, z. B. Personalwesen/CV-Screening); interne Wissensbasen nicht automatisch. AI Act ersetzt die DSGVO nicht (Art. 2 Abs. 7 ○). Art. 4 (KI-Kompetenz, seit 02.02.2025) und Verbote gelten bereits; Art. 50 (Transparenz) gesondert prüfen.
- **Fristen ◐:** Digital Omnibus on AI, VO (EU) 2026/1744, erlassen 08.07.2026, ABl. 24.07.2026 (Parlament 16.06., Rat 29.06.2026): Hochrisiko-Pflichten für eigenständige Systeme (Anhang III) ab **02.12.2027**, Anhang I ab 02.08.2028. Amtsblatt vor Verwendung prüfen.
- **Art. 10 (Daten-Governance) ○ ⚠:** Gilt für Trainings-, Validierungs-, Testdaten von Hochrisiko-Systemen. Ob eine RAG-Wissensbasis darunter fällt: offen (Literatur uneinig, keine behördliche Klärung). Unabhängig davon sinnvolle Metadaten-Kontrolle über eingespeiste Daten.
- **Art. 12 (Aufzeichnung) ○:** Nur Hochrisiko-Systeme; automatische Aufzeichnung von *Ereignissen* zur Nachvollziehbarkeit. Verlangt **nicht** Klartextprotokollierung aller Ein- und Ausgaben. Betreiber bewahren Protokolle mindestens sechs Monate auf, **sofern Unionsrecht, insbesondere Datenschutzrecht, nichts anderes vorsieht** (Art. 26 Abs. 6, Art. 19). Konflikt mit DSGVO damit gesetzlich entschärft und über Protokolldesign zu lösen (5.10).

### 9.4 NIS2 und Krypto-Agilität

- NIS2 (RL (EU) 2022/2555) ○: Art. 21 Abs. 2 verlangt u. a. Risikomanagement, Lieferkettensicherheit (lit. d), Konzepte für Kryptografie und Verschlüsselung (lit. h). Gilt für wesentliche/wichtige Einrichtungen, nicht automatisch für jeden Softwareanbieter; Kunden können Anforderungen über die Lieferkette weitergeben. Deutsche Umsetzung (NIS2UmsuCG/BSIG): Inkrafttreten, Anwendungsbereich prüfen.
- **Krypto-Agilität** steht nicht im NIS2-Text. Die Erwartung folgt aus Art. 32 DSGVO, EDPB 02/2025 Rn. 85, 96 (Notfallplan Algorithmenwechsel, periodische Neubewertung) und BSI-Empfehlungen. Technische Konsequenz: Algorithmen ohne Datenverlust austauschbar, damit der gelöschte Zustand über Jahre belastbar bleibt.

---

## 10. Entscheidungen im Überblick

| Gericht, Az., Datum | Gegenstand / Relevanz | Status |
|---|---|---|
| EuGH C-131/12 (Google Spain, 13.05.2014) | Auslistung, Grundrechtsabwägung; vor Geltung der DSGVO; nicht zu technischen Löschverfahren | ○ |
| EuGH C-582/14 (Breyer, 19.10.2016); C-413/23 P (EDPS/SRB, 04.09.2025) | Relativer Personenbezug, Pseudonymisierung | ○ Volltext prüfen |
| EuGH C-40/17 (Fashion ID, 29.07.2019) | Gemeinsame Verantwortlichkeit | ○ |
| EuGH C-460/20 (08.12.2022) | Auslistung bei offensichtlicher Unrichtigkeit ohne vorherige gerichtliche Entscheidung | ○ |
| EuGH C-129/21 (Proximus, 27.10.2022) | Art. 17 Abs. 2: Information anderer Verantwortlicher | ○ |
| EuGH C-340/21 (14.12.2023) | Nachweis der Angemessenheit von Sicherheitsmaßnahmen beim Verantwortlichen | ○ |
| EuGH C-300/21 (04.05.2023) | Art. 82: keine Erheblichkeitsschwelle | ○ |
| BGH VI ZR 10/24 (18.11.2024, Scraping) | Nach Kenntnisstand Schadensersatz bei Kontrollverlust (Art. 82), nicht Löschanspruch | ○ prüfen |
| OLG München, 22.03.2022 – 18 U 1697/21 Pre | Löschbegriff autonom; Nicht-Wiederverarbeitung | ◐ |
| OLG Dresden, 14.12.2021 – 4 U 1278/21 | Inkasso, Aufbewahrungspflichten je Datum, Zugriffsunterbindung, Schwärzung | ◐ |
| OLG Dresden, 03.02.2026 – 4 U 292/25 | Anonymisierung als „Minus zur Löschung“ | ◐ (Leitsatz; juris D_NJRE001633156) |
| LG München II, 14.03.2025 – 14 O 1461/24 | Anonymisierung als Unterfall der Löschung | ◐ |
| AG Lörrach, 03.03.2025 – 3 C 1099/24 | Kein Anspruch auf Löschnachweis | ◐ |
| OLG Dresden, 4 U 940/24 | Löschnachweis des Auftragsverarbeiters | ◐ Datum offen |
| OLG Nürnberg, 11.06.2025 – 3 U 383/25 | Darlegungs-/Beweislast Löschanspruch | ◐ nur Titel |
| BGH, 27.06.2024 – I ZR 98/23 | „klimaneutral“, Aufklärung in der Werbung (Analogie) | ○ |

Keine Entscheidung gefunden, die Crypto-Shredding billigt oder ein Löschverfahren wegen technischer Unzulänglichkeit beanstandet (Nicht-Fund, kein Beleg der Nichtexistenz).

---

## 11. Gesichert, umstritten, offen

| # | Aussage | Status | Fundstelle | Prüfstatus |
|---|---|---|---|---|
| 1 | Löschen/Vernichten sind selbst Verarbeitung | **Gesichert** [R] | Art. 4 Nr. 2 DSGVO | ✔ |
| 2 | DSGVO schreibt keine Löschmethode vor; Behörden erwarten risikoangemessene Verfahren und Kopienerfassung | **Gesichert** (Text) / [B] | Art. 17; SDM B60 S. 1–3 | ✔ |
| 3 | Verschlüsselte Daten bleiben personenbezogen, solange Schlüssel existiert | **Gesichert** | Art. 4 Nr. 5, ErwGr 26; EDPB 02/2025 Rn. 51 | ✔/○ |
| 4 | Anonymisierung kann Löschpflicht erfüllen | **Umstritten** (Instanzgerichte ja; EDPB skeptisch bei Praxis; Leitlinien ausstehend) | OLG Dresden 4 U 292/25; LG München II 14 O 1461/24; EDPB CEF 2026 | ◐/✔ |
| 5 | Schlüsselvernichtung ist Löschung nach Art. 17 | **Offen** (keine Behörden-/Gerichtsaussage; EDPB nennt Bedingungen im Blockchain-Kontext) | EDPB 02/2025 Rn. 51–53, 63, 96, 103–104 | ✔ |
| 6 | Crypto Erase technisch anerkannte Sanitisierung (Purge) unter Voraussetzungen | **Gesichert (technisch)**; rechtliche Wirkung offen | NIST SP 800-88 Rev. 2, Abschn. 3.2, FAQ Q9 | ◐ |
| 7 | Backup: nicht sofort überall löschen, aber Wiedereinspiel-Sperre und erneute Löschung nach Restore | **Überwiegend akzeptiert** [B]; Fristen ungeregelt | SDM S. 5, M60.S10/P08; EDPB CEF 2026 S. 20 | ✔/◐ |
| 8 | § 35 BDSG deckt Backups/RAG-Indizes | **Nein** (nur nicht automatisiert; Unionsrechtszweifel) | § 35 Abs. 1 S. 1 BDSG | ✔ |
| 9 | Text-Embeddings in RAG-Datenbanken sind personenbezogen | **Offen**; vorsorglich so behandeln | DSK-OH RAG S. 5 f., 13; EDPB 28/2024 (nur Modelle); HmbBfDI (modellintern, gegenläufig) | ✔/◐ |
| 10 | RAG-Referenzdokumente und Vektoreinträge gezielt adressier-/löschbar; Löschung im LLM bleibt ungelöst | **[B]** | DSK-OH RAG S. 8, 13 f. | ✔ |
| 11 | Kein Anspruch der betroffenen Person auf individuellen Löschnachweis (außerhalb Streitfall) | **Erstinstanzlich** | AG Lörrach 3 C 1099/24 | ◐ |
| 12 | Nachweispflicht gegenüber Aufsicht/im Streit; Löschung verifizieren und nachweisen | **Gesichert** | Art. 5 Abs. 2, 24, 32; EDPB CEF 2026; EuGH C-340/21 | ✔/○ |
| 13 | Löschprotokoll nur mit Metadaten, ohne Inhaltsdaten | **[B]**; Reichweite bei Personenbezug umstritten | SDM M60.S05/S06 | ✔ |
| 14 | On-Premise-Anbieter ohne Datenzugriff weder AV noch Verantwortlicher | **Überwiegend vertreten**, einzelfallabhängig; offen bei Fernzugriff/Telemetrie/Key-Recovery | EDPB 07/2020; DSK-OH RAG S. 3 | ✔/◐ |
| 15 | Art. 25 DSGVO bindet Hersteller direkt | **Nein**; DSK fordert Gesetzesänderung | DSK-Entschließung 12.12.2025 | ✔ |
| 16 | Aufbewahrungspflichten gehen vor, je Datum zu prüfen; Fristen 6/8/10 Jahre | **Gesichert** | Art. 17 Abs. 3 lit. b; § 147 AO, § 257 HGB; OLG Dresden 4 U 1278/21 | ✔/◐ |
| 17 | AI Act Art. 12 gilt für RAG-Protokolle | **Nur bei Hochrisiko-Einsatz**; Fristen Anhang III auf 02.12.2027 verschoben | VO 2024/1689; Omnibus 2026/1744 | ○/◐ |
| 18 | Art. 10 AI Act erfasst RAG-Wissensbasen | **Offen** | Literatur, keine Behördenklärung | ○ |
| 19 | Pauschale Werbeaussagen bergen UWG-/Vertragsrisiko | **Risiko**; keine einschlägige Rechtsprechung gefunden | § 5 UWG; § 434 BGB | ○ |
| 20 | Zulässigkeit/Dauer personenbezogener (pseudonymer) Lösch-IDs im Nachweis | **Umstritten** | AG Lörrach; Art. 11 DSGVO (Literatur) | ◐ |
| 21 | Konkrete Backup-Löschfristen (30/60/90 Tage) als Behördenvorgabe | **Nicht belegt**; Praxis | – | – |

---

## 12. Außenkommunikation

Vermeiden: „garantiert“, „DSGVO-konform“, „erfüllt Art. 17 automatisch“, „unwiederbringlich“, „vergisst Daten“, Zertifizierungs- oder Behördenbilligungsbehauptungen. Stattdessen beschreiben: **was technisch geschieht**, **welche Voraussetzungen** gelten (Konfiguration, Schlüsselmanagement), **was nicht erfasst ist**, dass der **Anwender** die rechtliche Gesamtverantwortung trägt.

```
UNZULÄSSIG (irreführend, § 5 UWG):
- „Unsere Vektordatenbank löscht personenbezogene Daten garantiert DSGVO-konform per Crypto-Shredding.“
- „Mit unserer Software erfüllen Sie Art. 17 DSGVO vollautomatisch auf Knopfdruck.“

BELASTBAR (technisch präzise):
- „Das System führt bei Löschaufrufen einen physischen Purge der Arbeitsspeicher-Indizes
  (HNSW/BM25) durch. Unveränderliche Speichersegmente werden durch Vernichtung des
  dokumentspezifischen Schlüssels (DocDek) unlesbar gemacht. Das System stellt einen
  signierten, datensparsamen Löschbericht (DeletionProof) über die gemessenen
  Systemzustände aus. Nicht erfasst sind externe Backups und Kopien außerhalb der
  Anwendung. Die rechtliche Bewertung der Löschwirkung sowie die Einhaltung von
  Aufbewahrungsfristen verbleiben in der Verantwortung des Betreibers.“
```

---

## 13. Prüfpunkte für Fachanwalt Datenschutzrecht / Datenschutzbeauftragte vor Produktzusagen

1. **Qualifikation der Schlüsselvernichtung:** Löschung (Art. 17), Anonymisierung (Minus-Theorie, OLG Dresden 4 U 292/25), Unzugänglichmachung oder Einschränkung? Position der zuständigen Aufsichtsbehörde (Sitz des Kunden).
2. **Relativer Personenbezug (EuGH C-413/23 P):** Entfällt der Personenbezug nach Vernichtung für *jede* Stelle, auch Anwender und Anbieter?
3. **Wirksamkeitsbeweis:** Welche Prüfungen (Pentest, Schlüsselvernichtungs-Audit, Zertifizierung) belegen, dass kein Schlüsselmaterial verbleibt (KMS/HSM, RAM, Swap, Dumps, Backups)?
4. **Umfang:** Liegen Quelldokument, Chunks, Vektoren, Volltextindex, Caches, Logs, Prompt-Historie, LLM-Kontexte im Wirkungsbereich der Vernichtung?
5. **Embeddings:** Behandlungsstrategie bei offener Rechtslage, dokumentierte Entscheidung; Rekonstruktionsrisiko im Einsatzfall.
6. **Backups:** Zulässige Frist, Wiedereinspiel-Sperre (M60.S10/P08), separater Backup-Schlüssel, Behandlung von Schlüsselkopien in Backups; Haltung der zuständigen Aufsicht (keine belegte Frist; § 35 BDSG nicht anwendbar).
7. **Nachweis:** Welche Metadaten wie lange (Art. 5 Abs. 2 gegen Rückstandslosigkeit; AG Lörrach; OLG Dresden 4 U 940/24)? Dürfen Lösch-IDs/HMAC-Commitments personenbezogen bleiben?
8. **Rolle des Anbieters:** Vertragliche und technische Festlegung „kein Zugriff“; Support, Telemetrie, Updates, Key-Recovery, LLM-Anbindung (AV, gemeinsame Verantwortlichkeit).
9. **Aufbewahrungspflichten:** § 147 AO, § 257 HGB (8/10/6 Jahre), § 630f BGB; Trennung Archiv und RAG-Index; Einschränkung statt Löschung.
10. **Wording:** Jede Werbeaussage auf UWG, Beschaffenheitsvereinbarung (BGB), Zertifizierungsbehauptungen prüfen; Haftungsbegrenzung, Kundenhinweise zur Anwenderverantwortung.
11. **KI-Verordnung:** Rolle als Anbieter; Risikoklasse typischer Szenarien; Fristverschiebung; Protokollkonzept (Art. 12, 26 Abs. 6) vs. Löschung.
12. **NIS2/CRA:** Betroffenheit von Anbieter und Zielkunden; Kryptografie-Richtlinie, Algorithmenwechsel.
13. **DSFA (Art. 35)** für den konkreten Einsatz.
14. **Aktualität:** EDPB-Anonymisierungsleitlinien, Endfassung Leitlinien 01/2025, mögliche Neufassung SDM Baustein 60 (V3.x), Digital Omnibus (DSGVO-Änderungen), Entscheidungen zu Embeddings/KI-Modellen.

---

## 14. Verifikation und Quellen

### 14.1 Vor Verwendung im Volltext prüfen (◐/○)

OLG Dresden 4 U 292/25; LG München II 14 O 1461/24; AG Lörrach 3 C 1099/24; OLG Dresden 4 U 940/24 (Datum); OLG Dresden 4 U 1278/21; OLG München 18 U 1697/21 Pre; OLG Nürnberg 3 U 383/25; EuGH C-413/23 P und C-582/14; EDPB-CEF-Bericht S. 20 und Issue 7; NIST SP 800-88 Rev. 2 (Abschn. 3.2, FAQ Q9); SDM Baustein 60 in aktueller SDM-Version; Digital Omnibus (Amtsblatt); BEG IV-Fristen im Gesetzestext; Endfassung EDPB 01/2025; HmbBfDI-Status; BayLfD OH KI; DSK-OH TOM; NIS2-Umsetzung Deutschland; CRA-Termine; Cloud-KMS-Wartefristen in Herstellerdokumentation; arXiv 2606.18497.

### 14.2 Aktualität der Behördenpapiere

| Dokument | Stand |
|---|---|
| DSK-OH RAG | V1.0, 10/2025 ✔ |
| DSK-OH KI und Datenschutz | V1.0, 05/2024; Überarbeitung angekündigt |
| DSK-OH TOM KI | V1.0, 06/2025 ◐ |
| DSK-Entschließung Herstellerverantwortung | 12.12.2025 ✔ |
| BayLfD OH KI | 03/2026 ◐ |
| EDPB-Stellungnahme 28/2024 | 17.12.2024 ✔ |
| EDPB-Leitlinien 01/2025 Pseudonymisierung | angenommen 01/2025; Endfassung nicht verifiziert |
| EDPB-Leitlinien 02/2025 Blockchain | v1 04/2025; v2 07.07.2026 ✔ |
| EDPB-CEF-Bericht Recht auf Löschung | 10.02.2026 ✔ |
| EDPB-Leitlinien Anonymisierung | in Arbeit ◐ |
| HmbBfDI-Diskussionspapier LLM | 15.07.2024 ✔ |
| SDM Baustein 60 | V1.0a (SDM V2.0), seit 02.09.2020 ✔; SDM V3.0/3.1 prüfen |
| NIST SP 800-88 | Rev. 2, 26.09.2025 |

### 14.3 Primärquellen

- EDPB Leitlinien 02/2025 Blockchain: edpb.europa.eu/documents/guideline/guidelines-022025-on-processing-of-personal-data-through-blockchain_en
- EDPB Stellungnahme 28/2024: edpb.europa.eu/system/files/2024-12/edpb_opinion_202428_ai-models_en.pdf
- EDPB CEF 2025 Recht auf Löschung: edpb.europa.eu/system/files/2026-02/edpb_cef-report_2025_right-to-erasure_en.pdf
- EDPB Leitlinien 01/2025 Pseudonymisierung: edpb.europa.eu/documents/guideline/guidelines-012025-on-pseudonymisation_en
- DSK-OH RAG: datenschutzkonferenz-online.de/media/oh/DSK_OH_RAG.pdf
- SDM Baustein 60: datenschutz-mv.de (Datenschutzmodell, Bausteine)
- NIST SP 800-88 Rev. 2: csrc.nist.gov/pubs/sp/800/88/r2/final
- § 35 BDSG: gesetze-im-internet.de/bdsg_2018/__35.html
- Sekundärquellen für ◐-Angaben: dsn-group (OLG Dresden 4 U 292/25, CEF-Bericht), ppc.land (AG Lörrach), gesetze-bayern.de (LG München II, OLG München), dr-bahr.com/delegedata (OLG Dresden 4 U 1278/21), DLA Piper (Digital Omnibus)

---

## 15. Häufige Fehlzuordnungen in Sekundärliteratur und Produktunterlagen (nicht übernehmen)

| Verbreitete Aussage | Richtig |
|---|---|
| OLG Dresden 14.12.2021 – 4 U 1278/21 begründe „Anonymisierung ist Minus zur Löschung“ und sei zentrale Entscheidung für Crypto-Löschen | Das Urteil betrifft Inkasso/Aufbewahrungspflichten. Die „Minus“-Formel stammt aus OLG Dresden 03.02.2026 – 4 U 292/25. Keine der Entscheidungen äußert sich zu Crypto-Shredding |
| Gerichte sähen Chiffretext als hinreichend anonymisiert | Nicht belegt; kein Urteil erkennt Crypto-Shredding an |
| EDPB-Leitlinien 01/2025 (Pseudonymisierung) enthalten Aussagen zu Quantencomputing/Algorithmenbruch bei Schlüsselvernichtung | Diese stehen in Leitlinien 02/2025 (Blockchain), Rn. 51, 96 |
| ICO und CNIL betrachten Crypto-Shredding als legitim | Nicht belegt. ICO: „beyond use“ für Backups |
| BSI TR-02102 empfiehlt kryptografisches Löschen für SSDs; ISO/IEC 27040 „akzeptiere“ Schlüsselvernichtung | TR-02102 regelt Algorithmen/Schlüssellängen, nicht Löschen. ISO-Akzeptanz nicht belegt |
| NIST SP 800-88 ohne Revisionsangabe | Maßgeblich Rev. 2 (26.09.2025); Rev. 1 zurückgezogen |
| BayLDA/LfDI BW/Berlin tolerierten 30/60/90-Tage-Backup-Rotation; „§ 35 BDSG analog“ | Keine Primärquelle; § 35 BDSG gilt nur für nicht automatisierte Verarbeitung |
| Embeddings „gesichert“ personenbezogen nach EDPB 28/2024; Hamburg stütze ähnliche Linie | Stellungnahme betrifft KI-**Modelle**; HmbBfDI vertritt für LLM-interne Embeddings die **Gegenposition**. Für RAG-Datenbanken: offen |
| AG Lörrach: Nachweis genüge durch Löschkonzept und eidesstattliche Versicherung | In der geprüften Berichterstattung nicht enthalten; Fall betrifft Nachbarn/Videoaufnahmen |
| SDM-Baustein „V2.0“ mit fünf „Mindestinhalten“ (u. a. „Task-ID 1045“) | Baustein 60 ist V1.0a (in SDM V2.0). Tatsächlicher Inhalt: 6.3. Die Fünferliste ist Praxis |
| EuGH „C-649/17 (Fashion ID)“ | Fashion ID ist C-40/17; C-649/17 ist ein Verbraucherrechte-Fall |
| BGH VI ZR 10/24 als Löschentscheidung; C-460/20 als „Löschpflicht aus Suchmaschinen“ | VI ZR 10/24: Art. 82 (Kontrollverlust). C-460/20: Auslistung |
| DSK-OH RAG: Daten könnten „durch Crypto-Shredding unbrauchbar gemacht“ werden; Löschkonzepte „sehr gut anwendbar“; Löschung erfordere „physische Entfernung“ | Crypto-Shredding nicht erwähnt. DSK: Löschung durch Datenbankeinträge, zugleich ungelöste Betroffenenrechte im LLM |
| HGB 6 Jahre / AO 10 Jahre; § 203 StGB als Aufbewahrungsgrundlage | Seit 2025 Buchungsbelege 8 Jahre; § 203 StGB ist Schweigepflicht |
| AI Act Art. 12 verlange Protokollierung aller Ereignisse und Ausgaben für jedes RAG | Nur Hochrisiko; Ereignisprotokolle; Art. 26 Abs. 6 verweist auf Datenschutzrecht |
| NIS2 verlange Krypto-Agilität | Begriff nicht im Text; Ableitung aus Art. 32 DSGVO, EDPB 02/2025, BSI |
| Verbreitete Werbung: „garantiert DSGVO-konform gelöscht“ | UWG-/Mangelrisiko (8.4, 12) |
