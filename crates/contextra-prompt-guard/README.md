# contextra-prompt-guard

Signatur- und phrasenbasierte Prompt-Injection-Erkennung und Quarantäne-Policy.

## Übersicht

`contextra-prompt-guard` bietet ein leichtgewichtiges, autarkes System zur Erkennung bekannter Prompt-Injection-Muster in RAG-Pipelines und Tool-Outputs.

- **Vornormalisierte Muster:** Whitespace-Kollabierung, Stripping, Skeletonisierung (Confusables/Homoglyphen für Kyrillisch/Griechisch).
- **Dekodierung:** Rekursive Base64-Erkennung bis Rekursionstiefe 2.
- **Quarantäne-Modi:** `Strict` (Redaktion), `FlagOnly` (Markierung), `Escalate` (Redaktion + Sicherheits-Audit-Protokollierung).
- **Keine internen Abhängigkeiten:** E0-Baustein ohne Abhängigkeit von anderen `contextra-*`-Crates.
