# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)

**Exportdatum:** 2026-10-28 | **Rechtsgrundlage:** Art. 30 Abs. 1 DSGVO

| Mandant | Verantwortlicher / Kontakt | Zweck der Verarbeitung | Betroffene / Datenkategorien | Empfänger | Drittland & Garantien | Löschfristen | TOM (Art. 32) | Rechtsgrundlage | Löschnachweise | Egress-Events |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 42 | Musterfirma GmbH (Datenschutzbeauftragter, Hauptstraße 1, 10115 Berlin, dsb@musterfirma.de)<br/>Vertreter: Max Mustermann, Geschäftsführung<br/>DSB: Dr. Anna Schmidt, dsb@musterfirma.de | Dokumenten-Graph-Relationierung | Betroffene: Kunden<br/>Daten: Entitäten, Metadaten | Interne Fachabteilung | Keine | Entitäten &amp; Metadaten: Löschung bei Mandantenkündigung | Integritätssicherung via SHA-256 / HMAC<br/>Rollenbasierte Zugriffskontrolle (RBAC) | Art. 6 Abs. 1 lit. f DSGVO | 1 | 0 |
| 42 | Musterfirma GmbH (Datenschutzbeauftragter, Hauptstraße 1, 10115 Berlin, dsb@musterfirma.de)<br/>Vertreter: Max Mustermann, Geschäftsführung<br/>DSB: Dr. Anna Schmidt, dsb@musterfirma.de | Kontextsensitive Vektorsuche | Betroffene: Kunden, Mitarbeiter<br/>Daten: Protokolldaten, Search Queries | Auftragsverarbeiter, Interne IT | Keine (Verarbeitung im EWR): Nicht zutreffend (EWR-Hosting) | Search Queries &amp; Protokolle: 30 Tage nach Abwicklung, danach automatische Löschung | Kryptographischer Löschnachweis (Ed25519 Signatur)<br/>Mandantentrennung &amp; Zugriffskontrolle<br/>Verschlüsselung at rest (AES-256-GCM-SIV) | Art. 6 Abs. 1 lit. b DSGVO | 1 | 1 |
