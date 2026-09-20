# memfuse

`memfuse` ist die primäre Fassade und Composition Root des MemFuse Cognitive OS (Ring 4).

## Zweck

Stellt den zentralen Einstiegspunkt und Builder-Muster (`MemFuseBuilder`) für Rust-Anwendungen bereit. Dient als einzige Composition Root des Systems, um Backends, Memory Engines, Privatsphäre-Filter und Inferenz-Anbindungen sauber zu verbinden.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 4 (Ränder / Composition Root)
- **Status:** 🟡 In Migration / Shell (Fassade)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

In Phase 1b des Migrationsplans wird die Fassade kontinuierlich erweitert. Momentan stellt `memfuse` grundlegende Module und Builder-Typen bereit:

- **Composition Root / Builder:** `MemFuseBuilder`
- **Konfiguration:** `MemFuseConfig`

## Architektur & Verweise

Für Details zur Zielarchitektur, Ring-Struktur und den einzelnen Migrationsphasen siehe [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §A2 / §4.2 im Repository-Root.
