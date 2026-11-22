# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO

> **Hinweis:** Entwurf — ersetzt keine Rechtsberatung.

**Verantwortlicher (Auftraggeber):** Kanzlei Dr. Mustermann
**Auftragsverarbeiter (Auftragnehmer):** Contextra Cloud GmbH
**Mandanten-ID (TenantId):** TenantId\(42\)
**Vertragsdatum:** 2026\-10\-29

---

## 1. Gegenstand und Dauer der Verarbeitung
Gegenstand der Verarbeitung ist die Bereitstellung und Nutzung des Contextra RAG- & Vektordatenbank-Systems zur hochsicheren Speicherung, Indizierung und semantischen Abfrage von Dokumenten und Kontexten.
Die Dauer der Verarbeitung entspricht der Laufzeit des Hauptvertrages zwischen dem Verantwortlichen und dem Auftragsverarbeiter.

## 2. Art und Zweck der Verarbeitung
Die Verarbeitung umfasst die automatisierte Vektorisierung, Speicherung, semantische Indexierung und den Abruf von Text- und Wissensdaten zum Zweck des wissensbasierten Kontext-Retrievals (RAG) für den Verantwortlichen.

## 3. Art der personenbezogenen Daten und Kategorien betroffener Personen
**Art der personenbezogenen Daten:** In Dokumenten und Texten enthaltene Personenstammdaten, Kommunikationsdaten, Metadaten sowie geschäftsspezifische Inhalte, die im Vektorspeicher abgelegt werden.
**Kategorien betroffener Personen:** Kunden, Beschäftigte, Mandanten, Lieferanten und Vertragspartner des Verantwortlichen.

## 4. Pflichten und Rechte des Verantwortlichen & Weisungsbindung
1. Der Verantwortliche ist für die Beurteilung der Zulässigkeit der Datenverarbeitung sowie für die Wahrung der Rechte der betroffenen Personen allein verantwortlich.
2. Der Auftragsverarbeiter verarbeitet personenbezogene Daten ausschließlich auf dokumentierte Weisung des Verantwortlichen — einschließlich hinsichtlich der Übermittlung personenbezogener Daten an ein Drittland oder eine internationale Organisation —, sofern er nicht durch das Recht der Union oder der Mitgliedstaaten, dem der Auftragsverarbeiter unterliegt, hierzu verpflichtet ist.
3. Der Auftragsverarbeiter informiert den Verantwortlichen unverzüglich, wenn er der Auffassung ist, dass eine Weisung gegen die DSGVO oder andere Datenschutzbestimmungen der Union oder der Mitgliedstaaten verstößt.

## 5. Vertraulichkeitsverpflichtung
1. Der Auftragsverarbeiter gewährleistet, dass sich die zur Verarbeitung der personenbezogenen Daten befugten Personen zur Vertraulichkeit verpflichtet haben oder einer angemessenen gesetzlichen Verschwiegenheitspflicht unterliegen.
2. Die Vertraulichkeitsverpflichtung besteht auch nach Beendigung des Auftragsverarbeitungsvertrages fort.

## 6. Technische und organisatorische Maßnahmen (TOM)
Der Auftragsverarbeiter garantiert die Umsetzung folgender produktimmanenter technischer Sicherheitsgarantien gemäß Art. 32 DSGVO:

### Kryptografischer Löschbeweis \(Ed25519\) (Art. 32 Abs. 1 lit. b & Art. 17 DSGVO)
Nach Ausführung einer Löschanforderung wird ein fälschungssicherer, Ed25519\-signierter Löschbeleg generiert, der die unwiderrufliche Löschung aller Datenpunkte mathematisch und auditierbar nachweist.

### Zero\-Egress\-Default / Egress\-Gateway (Art. 32 Abs. 1 lit. b DSGVO)
Sämtlicher Datenverkehr nach außen ist standardmäßig blockiert. Ausgehende Anfragen verlaufen erzwingend über ein abgesichertes Egress\-Gateway mit strikter Whitelist und DLP\-Inhaltsprüfung.

### Kryptografische Mandantentrennung (Art. 32 Abs. 1 lit. b DSGVO)
Vektoren, Indizes und Metadaten sind pro Tenant\-ID kryptografisch isoliert. Zugriffe über Mandantengrenzen hinweg werden auf Speicherebene unterbindungssicher verhindert.

### Verschlüsselte Segmente im KV\-Cache (Art. 32 Abs. 1 lit. a DSGVO)
Flüchtige Zwischenspeicher und KV\-Cache\-Segmente werden im Arbeitsspeicher mit rotierenden Schlüsseln verschlüsselt, um Unbefugten den Zugriff auf Klartextkontexte im Memory\-Dump zu verwehren.

## 7. Unterauftragsverarbeiter
1. Der Auftragsverarbeiter darf Unterauftragnehmer nur nach vorheriger schriftlicher oder dokumentierter Genehmigung des Verantwortlichen hinzuziehen.
2. Der Auftragsverarbeiter informiert den Verantwortlichen immer über jede beabsichtigte Änderung bezüglich der Hinzuziehung oder Ersetzung anderer Unterauftragsverarbeiter. Der Verantwortliche kann gegen derartige Änderungen Einspruch erheben.
3. Der Verantwortliche stimmt dem Einsatz folgender Unterauftragsverarbeiter zu:
- Hetzner Online GmbH \(Hosting Germany\)

## 8. Löschung von Daten und SLA
1. Der Auftragsverarbeiter löscht oder gibt nach Abschluss der Erbringung der Verarbeitungsleistungen alle personenbezogenen Daten nach Wahl des Verantwortlichen zurück und löscht bestehende Kopien, sofern nicht nach dem Recht der Union oder der Mitgliedstaaten eine Verpflichtung zur Speicherung der personenbezogenen Daten besteht.
2. Bei Beendigung der Leistungserbringung oder konkreter Löschanforderung löscht der Auftragsverarbeiter alle verarbeiteten Daten des Mandanten (`TenantId\(42\)`) innerhalb des vereinbarten Deletion-SLA von **14 Tagen**.
3. Auf Wunsch stellt der Auftragsverarbeiter dem Verantwortlichen einen fälschungssicheren, kryptografischen Löschbeweis (Ed25519) zum Nachweis der unwiderruflichen Löschung bereit.

## 9. Unterstützung bei Betroffenenrechten
1. Der Auftragsverarbeiter unterstützt den Verantwortlichen unter Berücksichtigung der Art der Verarbeitung nach Möglichkeit mit geeigneten technischen und organisatorischen Maßnahmen bei der Erfüllung von dessen Pflicht zur Beantwortung von Anträgen auf Wahrnehmung der in Kapitel III der DSGVO genannten Rechte der betroffenen Person (z. B. Auskunft, Berichtigung, Löschung, Einschränkung der Verarbeitung, Datenübertragbarkeit).
2. Wendet sich eine betroffene Person direkt an den Auftragsverarbeiter, wird dieser den Antrag unverzüglich an den Verantwortlichen weiterleiten.

## 10. Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO
1. Der Auftragsverarbeiter unterstützt den Verantwortlichen unter Berücksichtigung der Art der Verarbeitung und der ihm zur Verfügung stehenden Informationen bei der Einhaltung der in den Artikeln 32 bis 36 DSGVO genannten Pflichten:
   - **Art. 32 DSGVO:** Gewährleistung der Sicherheit der Verarbeitung durch geeignete technische und organisatorische Maßnahmen.
   - **Art. 33 & 34 DSGVO:** Unverzügliche Meldung von Verletzungen des Schutzes personenbezogener Daten an den Verantwortlichen sowie Unterstützung bei der Benachrichtigung betroffener Personen.
   - **Art. 35 & 36 DSGVO:** Unterstützung bei der Durchführung von Datenschutz-Folgenabschätzungen und eventuellen vorherigen Konsultationen der Aufsichtsbehörde.

## 11. Nachweis- und Kontrollrechte
1. Der Auftragsverarbeiter stellt dem Verantwortlichen alle erforderlichen Informationen zum Nachweis der Einhaltung der in Art. 28 DSGVO niedergelegten Pflichten zur Verfügung.
2. Der Auftragsverarbeiter ermöglicht Überprüfungen — einschließlich Inspektionen —, die vom Verantwortlichen oder einem anderen von diesem beauftragten Prüfer durchgeführt werden, und trägt zu diesen Überprüfungen bei.
