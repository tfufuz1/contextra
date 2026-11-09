# contextra-db

Interne Fassade und Integrationsbaustein. Geht im Zuge der Bereinigung in `contextra` auf.

## Zweck
`contextra-db` stellt temporär eine Collection- und Hybridsuch-API für Contextra bereit. Im Zuge der Architektur-Harmonisierung geht die Funktionalität dieses Crates vollständig in der Produktfassade `contextra` auf (siehe Falt-Plan).

## Status & Entnahme
- **Rolle:** Interne Fassade (geplant für Einfaltung in `contextra`)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Keine Rechts- oder Löschgarantieversprechen:** Mechanisches Entfernen von Dokumenten und Indizes; keine Compliance-Beweisketten im Kern.
