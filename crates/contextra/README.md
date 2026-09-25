# contextra

`contextra` ist die primäre Fassade und Composition Root des Contextra Cognitive OS (Ring 4).

## Zweck

Stellt den zentralen Einstiegspunkt und Builder-Muster (`ContextraBuilder`) für Rust-Anwendungen bereit. Dient als einzige Composition Root des Systems, um Backends, Memory Engines, Privatsphäre-Filter und Inferenz-Anbindungen sauber zu verbinden.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 4 (Ränder / Composition Root)
- **Status:** 🟡 In Migration / Shell (Fassade)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

In Phase 1b des Migrationsplans wird die Fassade kontinuierlich erweitert. Momentan stellt `contextra` grundlegende Module und Builder-Typen bereit:

- **Composition Root / Builder:** `ContextraBuilder`
- **Konfiguration:** `ContextraConfig`

## Architektur & Verweise

Für Details zur Zielarchitektur, Ring-Struktur und den einzelnen Migrationsphasen siehe [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §A2 / §4.2 im Repository-Root.

## Feature-Ringe (Ring-Split)

| Ring-Name | Enthaltene Features | Zielgruppe | Lizenzmodell |
| --- | --- | --- | --- |
| `fast` | `fast` | OSS-Adoption | Open Source (MIT/Apache-2.0) |
| `sovereign` | `sovereign`, `contextra-crypto`, `contextra-privacy` | Kanzlei-Appliance | Open Source (MIT/Apache-2.0) |
| `compliance` | `compliance`, `sovereign`, `audit-export`, `avv-generator` | Kommerziell | Trade-Secret / Kommerziell (`compliance`) |
