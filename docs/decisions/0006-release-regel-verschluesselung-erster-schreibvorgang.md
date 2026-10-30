# ADR 0006: Release-Regel: Verschlüsselung ab dem ersten Schreibvorgang

> **Status-Update 30.10.2026:** Ausgesetzt (D-R5). Bezieht sich auf die Löschkette.

- **Status**: Proposed — benötigt Maintainer-Freigabe
- **Datum**: 2026-10-29
- **Kontext**: NIST-Vorbedingung für Cryptographic Erase (SPEC LA-15), LegacyUnkeyed-Deckel (SPEC C-12.2.5 Nr. 7)

## Kontext und Problemstellung

Gemäß den Vorgaben des NIST Special Publication 800-88 Rev. 1 (sowie SPEC LA-15) ist eine der zwingenden Vorbedingungen für wirksames kryptografisches Löschen (*Cryptographic Erase*), dass die betreffenden Daten von ihrem allerersten Schreibvorgang an auf dem nicht-flüchtigen Speichermedium verschlüsselt abgelegt wurden. Wurden Daten ursprünglich unverschlüsselt (Klartext) auf Flash- oder Magnetmedien geschrieben und erst nachträglich verschlüsselt, verbleiben durch Wear-Leveling und Garbage-Collection physische Klartext-Reste auf dem Medium, die durch die spätere Schlüsselvernichtung nicht erfasst werden.

Zudem definiert SPEC C-12.2.5 Nr. 7 für Datenbestände ohne indviduellen Dokumentenschlüssel (`DocDek`, z. B. Altformate oder rein auf Ebene des Storage-Scopes verschlüsselte Daten) den Status `LegacyUnkeyed`. Für solche Bestände ist höchstens die Löschwirkungsklasse P (Physische Überschreibung) oder L (Lesesperre) erreichbar, nicht jedoch die vollwertigen kryptografischen Klassen C1, C2 oder C3. Der Diagnosedienst `doctor` weist den Anteil von `LegacyUnkeyed` im Bestand aus.

Es stellt sich die Frage, wie mit Releases verfahren werden soll, solange das Arbeitspaket PLAN WP 5.3 (durchgängige DocDek-Verschlüsselung je Betroffenen-Einheit ab dem ersten Schreibvorgang) noch nicht vollständig umgesetzt ist.

## Entscheidungsentwurf (Vorschlag)

1. **Release-Sperre für Echtdaten vor WP 5.3**:
   Es wird kein Release für den produktiven Einsatz freigegeben, das echte Nutzerdaten dauerhaft auf Medien speichert, bevor PLAN WP 5.3 vollständig abgeschlossen und verifiziert ist.

2. **Garantie der Erstverschlüsselung**:
   Ab WP 5.3 garantiert Contextra, dass jeder Schreibvorgang in WAL, SSTable und Blob-Segmente unmittelbar unter dem DocDek der jeweiligen Betroffenen-Einheit erfolgt. Ein Schreiben unverschlüsselter Zwischenstände auf das Speichermedium wird architektonisch ausgeschlossen.

## Alternativen

1. **Alternative 1: Release nur mit Custody-Stufe K0 und sichtbarem Hinweis**:
   Releases vor Abschluss von WP 5.3 werden zugelassen, aber streng auf die Custody-Stufe K0 (keine Verschlüsselung) beschränkt. In der Systemdiagnose (`doctor`) und in allen erzeugten Berichten wird unmissverständlich ausgewiesen, dass nur die Löschwirkungsklassen L (Lesesperre) und P (Physisches Löschen) erreichbar sind und kein Cryptographic Erase stattfindet.

2. **Alternative 2: Release zulassen mit verpflichtender Migrationsanforderung**:
   Releases werden mit Übergangsformaten zugelassen. Datenbestände, die vor WP 5.3 geschrieben wurden, werden als `LegacyUnkeyed` markiert. Vor der Durchführung kryptografischer Löschungen muss das Migrationstool `contextra migrate --rekey-units` ausgeführt werden, welches die Daten liest, in Betroffenen-Einheiten segmentiert und unter neuen DocDeks verschlüsselt auf dem Medium ablegt.

## Konsequenzen

- **Migration**: Werden Datenbestände aus früheren Versionen oder Testphasen übernommen, müssen diese mittels `contextra migrate --rekey-units` neu verschlüsselt werden, um für spätere Löschungen die Klassen C1–C3 erreichen zu können.
- **Transparenz**: `doctor` und der Löschbericht weisen den exakten `LegacyUnkeyed`-Anteil aus. Solange unverschlüsselte Altbestände existieren, lehnt die Engine die Ausstellung von C1–C3-Berichten ab (Fail-Closed).
- **Positiv**: Die strikte Erfüllung der NIST-Vorbedingung schützt Betreiber vor Scheinsicherheit bei Löschberichten.
- **Negativ**: Verzögerung von Produktiv-Releases bis zum vollständigen Abschluss von WP 5.3.

## Offene Fragen

- Soll das Migrationstool `contextra migrate --rekey-units` beim Systemstart automatisch ausgeführt werden oder ausschließlich als manueller CLI-Befehl zur Verfügung stehen?
- Wie behandelt das Rekeying Teilunterbrechungen (z. B. Stromausfall während der Migration mehrerer Terabyte an Altbeständen)?

## Betroffene Komponenten

- `contextra-crypto`
- `contextra-store`
- `contextra-engine`
- `contextra-db` / CLI (`contextra migrate`)
