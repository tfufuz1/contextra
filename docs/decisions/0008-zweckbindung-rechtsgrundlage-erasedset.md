# ADR 0008: Zweckbindung und Rechtsgrundlage für ErasedSet, Referenz-Hashes und Berichte

- **Status**: Proposed — benötigt Maintainer-Freigabe
- **Datum**: 2026-10-29
- **Kontext**: Datenhaltung für Nachweis- und Sperrzwecke ist selbst personenbezogen (EDPB Rn. 52, SPEC C-12.7.10, SPEC C-12.10)

## Kontext und Problemstellung

Zur Verhinderung der unbeabsichtigten Wiederauferstehung gelöschter Daten beim Einspielen von Backups (*Restore-Guard*) sowie zur Erfüllung von Rechenschaftspflichten (Art. 5 Abs. 2 DSGVO) führt Contextra Nachweisdatenstrukturen wie das `ErasedSet`, das `RevocationLog` und signierte Löschberichte (`DeletionProof`).

Da die darin enthaltenen `subject_ref_hash`-Einträge mittels `HMAC-SHA256(K_ref, kanonische_ref)` erzeugt werden, stellen sie nach Leitlinien des Europäischen Datenschutzausschusses (EDPB Rn. 52) pseudonisierte und damit weiterhin **personenbezogene Daten** dar, solange der Referenzschlüssel `K_ref` existiert (SPEC C-12.10). Erst nach Vernichtung von `K_ref` nach Ablauf der Aufbewahrungsfrist (`ref_epoch_len` + `erased_set_retention`) verliert der Hash seine Zuordenbarkeit.

Es muss geklärt werden, wie Contextra Betreibern dabei hilft, die Zweckbindung und Rechtsgrundlage nach Art. 30 DSGVO (Verzeichnis von Verarbeitungstätigkeiten) für diese interne Nachweisdatenhaltung sauber zu dokumentieren, ohne unzulässige Rechtsberatung zu leisten.

## Entscheidungsentwurf (Vorschlag)

1. **Dokumentationsvorlage für Betreiber**:
   Contextra liefert eine standardisierte Dokumentationsvorlage (als Bestandteil der Dokumentation und exportierbar über `contextra doctor export-concept`), die den technischen Zweck, die Speicherdauer und die empfohlenen Erwägungen für die Rechtsgrundlage (z. B. Art. 6 Abs. 1 lit. c DSGVO i. V. m. Art. 17/5 Abs. 2 DSGVO für Nachweiszwecke bzw. Art. 6 Abs. 1 lit. f DSGVO zur Systemintegrität via Restore-Guard) beschreibt. Diese Vorlage ist explizit als technischer Entwurf gekennzeichnet und stellt kein Rechtsgutachten dar.

2. **Konfigurierbare Aufbewahrungsfristen**:
   Die Aufbewahrungsfristen (`erased_set_retention`, `ref_epoch_len`, `key_backup_max_age`) bleiben vollständig konfigurierbare Parameter (O-02, O-43), damit Betreiber diese an ihre spezifischen betrieblichen und rechtlichen Aufbewahrungskonzepte anpassen können.

3. **Anonymisierung durch Schlüsselvernichtung**:
   Nach Ablauf von `ref_epoch_len` plus `erased_set_retention` wird der Referenzschlüssel `K_ref` gemäß Destroy-Protokoll unwiederbringlich vernichtet. Ab diesem Zeitpunkt sind verbleibende Einträge im `ErasedSet` und im `RevocationLog` anonymisiert und nicht mehr auf natürliche Personen rückführbar.

## Alternativen

1. **Alternative 1: Hart codierte, starre Löschfristen im Core**:
   Feste Verdrahtung unveränderlicher Fristen (z. B. starr 90 Tage) direkt im Quellcode von `contextra-engine`, ohne Konfigurationsmöglichkeit.

2. **Alternative 2: Unmittelbare Vernichtung aller Berichte und Referenz-Hashes**:
   Verzicht auf `ErasedSet` und Aufbewahrung von Löschberichten. Sobald die Schlüsselvernichtung vollzogen ist, werden auch alle Prüf-Hashes und Berichte gelöscht.

## Konsequenzen und Rechtsfragen

- **Vorteil der Nachweisbarkeit**: Betreiber können gegenüber Aufsichtsbehörden lückenlos belegen, dass Löschpflichten nachgekommen wurde, ohne dafür die ursprünglichen Klartextdaten aufbewahren zu müssen.
- **Transparenzpflicht**: Betreiber müssen die Verarbeitung im `ErasedSet` in ihr Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO) aufnehmen.
- **Konfiguration**: In regulierten Profilen (z. B. `EnterpriseRegulated`) muss die Frist explizit gesetzt werden und darf nicht unbegrenzt sein.
- **Offene Rechtsfragen**:
  - Welche konkrete Aufbewahrungsfrist für den Nachweis einer DSGVO-Löschung von den jeweiligen nationalen Aufsichtsbehörden akzeptiert wird, bleibt eine Einzelfallentscheidung des Betreibers.
  - Das Verhältnis zwischen handels-/steuerrechtlichen Aufbewahrungspflichten (z. B. GoBD) und der Vernichtung des `K_ref` muss vom Betreiber im Löschkonzept gewürdigt werden.

## Betroffene Komponenten

- `contextra-crypto` (`K_ref`-Lebenszyklus, `RevocationLog`)
- `contextra-engine` (`ErasedSetMirror`, `RestoreGuard`)
- `contextra-avv-generator` / CLI (`doctor export-concept`)
