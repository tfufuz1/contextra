# ADR 0007: Betroffenen-Erkennung: Contextra erkennt Personen in Freitext nicht

> **Status-Update 30.10.2026:** Ausgesetzt (D-R5). Bezieht sich auf die entfernte Extraktion.

- **Status**: Proposed — benötigt Maintainer-Freigabe
- **Datum**: 2026-10-29
- **Kontext**: Löschung über `subject_ref` des Aufrufers (SPEC C-12.15), optionale Erkennung in SPEC [R] vorgesehen

## Kontext und Problemstellung

Betroffene Personen werden im Contextra-Ökosystem primär über das Feld `subject_ref` identifiziert, das beim Speichern von Dokumenten oder Records vom Aufrufer/Anwendungssystem mitgegeben wird (SPEC C-12.15). Ist kein `subject_ref` explizit angegeben, gilt das gesamte Dokument als eigene Betroffenen-Einheit des Aufrufers.

In der Praxis enthalten Freitexte (z. B. Fließtexte, E-Mails, Chatverläufe) häufig ungeplante personenbezogene Daten oder Namen weiterer Betroffener. SPEC Fassung 3.0 sieht zwar in der Baukasten-Klasse [R] (Rand/Option) theoretische Erweiterungen für eine automatische Personenerkennung (z. B. Named Entity Recognition / PII-Scanning) vor, eine solche Analyse ist jedoch fehleranfällig und birgt das Risiko von False Positives und False Negatives.

Es muss entschieden werden, ob und wie Contextra in Version v1.0 Personen in unstrukturierten Freitexten automatisch erkennt und wie dies in Löschberichten transparent gemacht wird.

## Entscheidungsentwurf (Vorschlag)

1. **Keine automatische Personenerkennung in v1.0**:
   Contextra v1.0 führt im Kern [K] und in den ausgelieferten Modulen [M] keine automatische Identifikation von Personen oder PII in Freitexten durch. Die Zuordnung von Dokumenten zu Betroffenen stützt sich exklusiv auf die vom Aufrufer übergebenen `subject_ref`-Attribute.

2. **Pflicht-Ausschluss `SubjectDiscoveryNotPerformed` im Löschbericht**:
   Jeder erzeugte Löschbericht (V5) enthält verpflichtend unter den Ausnahmen/Ausschlüssen (`ExcludedScope`) den Eintrag `SubjectDiscoveryNotPerformed`. Damit wird gegenüber Auditoren und Aufsichtsbehörden explizit offengelegt, dass keine eigenständige Erkennung ungenannter Personen im Text stattgefunden hat.

3. **Transparenz in Systemdiagnose**:
   Der Befehl `contextra doctor export-concept` weist darauf hin, dass das Aufrufersystem für die vollständige Erfassung aller `subject_ref` verantwortlich ist.

## Alternativen

1. **Alternative 1: Optionale Erkennung als Baukasten-Modul [R]**:
   Entwicklung eines optionalen Rand-Moduls (`contextra-privacy` / PII-Scanner) auf Basis von Regelwerken oder leichten ML-Modellen. Dieses Modul könnte beim Ingest Freitexte scannen und automatisch Vorschläge für `subject_ref` generieren oder Warnungen ausgeben.

2. **Alternative 2: Pflicht-Rückfrage vor Abschluss der Löschung**:
   Vor Abschluss einer Lösch-Saga scannt ein heuristisches Modul den Text auf typische Namensmuster. Werden potenzielle Personenbezüge gefunden, verbleibt das Dokument im Zustand `Partial{ReviewRequired}` und fordert eine manuelle Entscheidung des Betreibers an, bevor die Schlüsselvernichtung freigegeben wird.

## Konsequenzen

- **Löschanträge**: Ein Löschantrag für `subject_ref = A` löscht verlässlich genau die Dokumente und Segmente, die explizit mit `A` getaggt wurden. Werden Personen im Freitext anderer Dokumente erwähnt, ohne als `subject_ref` registriert zu sein, werden diese Dokumente durch den Löschantrag von A nicht erfasst.
- **Verantwortungstrennung**: Die Pflicht zur korrekten Betroffenen-Zuordnung verbleibt klar beim integrierenden Anwendungssystem.
- **Klarheit im Audit**: Der Pflicht-Ausschluss `SubjectDiscoveryNotPerformed` im Bericht verhindert Rechtsirrtümer über den Prüfungsumfang der Löschkette.

## Offene Fragen

- Reicht der Ausschluss `SubjectDiscoveryNotPerformed` in regulatorisch stark überwachten Umgebungen (z. B. `EnterpriseRegulated`) aus, oder wird dort ein vorgeschaltetes PII-Tagging im Ingest-Pfad zur Zwingenden Anforderung?
- Wie soll das System reagieren, wenn nachträglich ein Dokument neu mit einem weiteren `subject_ref` nachgetaggt werden muss?

## Betroffene Komponenten

- `contextra-engine`
- `contextra-crypto` (Löschbericht V5 / ExcludedScope)
- `contextra-privacy`
- `contextra-mcp` / CLI (`doctor export-concept`)
