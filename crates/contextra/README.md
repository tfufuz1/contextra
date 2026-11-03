Öffentliche API und Verdrahtung. Enthält keine Logik.

## Status

Ring 4. Einzige öffentliche Programmierschnittstelle gemäß [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitte 5.1, 7 und 8.14 (Entscheidungen D-66, D-89, D-95).

## Zulässige Außenaussage (SPEC §6.4)

> Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der zugehörige Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.

## Architektur & Modulschnitt

Gemäß SPEC D-89 enthält die Fassade keine eigene Geschäftslogik und beschränkt sich auf:
- `api`: Handles (`Contextra`, `Tenant`, `Collection`), Dokumenttypen, `EngineBuilder` mit statischer Pflichtparameterprüfung.
- `wiring`: Standard-Systemverdrahtung (`system_defaults()`) mit `RealVfs`, Systemuhr und OS-CSPRNG.
- `error`: Typisierte Fassaden-Fehler mit Handlungsempfehlungen ohne Informationsleckage.

## Abhängigkeiten

Erlaubt laut SPEC 5.3 und D-88: `types`, `durable-fs`, `engine`.
