# ADR-N24: F9 AVV-Generator und Audit-Export als Entwurfs- und Berichtswerkzeuge

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Die Generierung von Auftragsverarbeitungsverträgen (AVV / GDPR Art. 28) über `contextra-avv-generator` sowie der Export von Audit-Spuren über `contextra-audit-export` unterstützen Unternehmen bei der Compliance-Dokumentation.
  Werden automatisch generierte AVV-Dokumente oder Audit-Exporte jedoch fälschlicherweise als rechtsverbindliche, schlüsselfertige Verträge oder vollumfängliche behördliche Zertifikate verstanden, drohen rechtliche Haftungsrisiken und Fehlannahmen bezüglich der tatsächlichen Rechtswirksamkeit (Gesamtspezifikation §F9).

## Entscheidung

1. **Einstufung als reine Entwurfs- und Berichtswerkzeuge:**
   Sämtliche von `contextra-avv-generator` und `contextra-audit-export` erzeugten Artefakte werden rechtlich und technisch strikt als **vorläufige Entwurfs- und Berichtswerkzeuge ("Draft / Report Tools")** klassifiziert [V].

2. **Verpflichtender Prüfungshinweis ("Legal Review Notice"):**
   Jedes generierte Dokument muss an prominenter Stelle (Kopfzeile/Header) einen unübersehbaren Hinweis enthalten: Das Dokument stellt keinen rechtsverbindlichen Vertrag dar und erfordert vor dem rechtlichen Vollzug oder der Einreichung bei Aufsichtsbehörden eine manuelle juristische Prüfung durch einen qualifizierten Rechtsbeistand bzw. Datenschutzbeauftragten [V].

3. **Kryptografische Hash-Ketten ohne Geheimnis-Leaks (`INV-EGRESS-AUDIT-1`):**
   Audit-Log-Exporte sichern die Unverfälschbarkeit über fälschungssichere Hash-Ketten ab, ohne unverschlüsselte vertrauliche Inhalte oder Plaintext-Geheimnisse zu leaken [V].

## Alternativen

### Option A: Bereitstellung als schlüsselfertige Verträge ohne Disclaimer
* **Vorteile:** Anscheinend höhere Bequemlichkeit für Endanwender.
* **Nachteile:** Unverantwortliche rechtliche Haftungsrisiken für das Projekt; Irreführung über die juristische Gültigkeit automatischer Entwürfe.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Haftungsschutz:** Eindeutige juristische Abgrenzung schützt Betreiber und Entwickler vor Haftungsansprüchen [V].
* **Integrität der Audit-Spur:** Audit-Log-Exporte garantieren die Hash-Ketten-Integrität nach dem WORM-Prinzip (`INV-EGRESS-AUDIT-1`) [V].

## Reversibilitäts-Bedingung

Eine Änderung erfordert eine formelle Akkreditierung des Systems durch eine zuständige Datenschutz-Aufsichtsbehörde, die automatisierte Verträge als rechtsgültig ohne Prüfvorbehalt anerkennt [S].
