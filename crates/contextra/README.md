# contextra

> **Haupt-Bibliotheksfassade und einzige Composition Root für Endanwender**
>
> Normativitätsklasse: **Kern `[K]`** (Ring 4, `stable`; SPEC §4, `capabilities.toml`).

---

## Zweck & Rolle im System

`contextra` ist die primäre Fassade und die einzige **Composition Root** des Gesamtsystems (Ring 4). Sie stellt den zentralen Einstiegspunkt und das Builder-Muster (`ContextraBuilder`) für Anwendungen bereit, um Speicher-Engines, Vektor- und Textindizes, Krypto-Verwaltung und Inferenzanbindungen sauber und sicher zu instanziieren.

## Ring-Zugehörigkeit & Sicherheitsgarantien

- **Ring:** Ring 4 (Fassade / Composition Root)
- **Status:** `stable` / Primäre Schnittstelle
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero Panic:** Alle öffentlichen APIs geben `Result<T, ContextraError>` zurück.
- **Fail-Closed:** Fehlerhafte Konfigurationen oder fehlende Schlüssel verhindern den Start.

## Öffentliche API-Übersicht

- **Composition Root / Builder:** `ContextraBuilder` zur sicheren Assemblierung aller Subsysteme.
- **Konfiguration:** `ContextraConfig`, `StorageConfig`, `SecurityConfig`.
- **Datenbank-Zugriff:** Re-Exports der Sammlungs- und Transaktionsverwaltung aus `contextra-db`.
- **Löschpfad:** Initiierung von physischem Purge und Crypto-Shredding über die `ErasureSaga`.

## Weiterführende Dokumentation

- 📖 [Gesamtspezifikation (SPEC.md)](../../docs/SPEC.md)
- 📋 [Implementierungsplan (PLAN.md)](../../PLAN.md)
- 🏗️ [Architektur & Schichtenmodell](../../docs/ARCHITECTURE.md)
- 🛠️ [Kompilieranleitung für Implementierer](../../docs/COMPILATION.md)
