# contextra

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Build-Blocker: `contextra_core` und Feature `candle` (SPEC §16, B3). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

Haupt-Library Facade für Endanwender.

Normativitätsklasse: **Kern `[K]`** (Ring 4, `stable`; §6.0, `capabilities.toml`).


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

| Feature-Name | Enthaltene Features / Abhängigkeiten | Zielgruppe | Lizenzmodell |
| --- | --- | --- | --- |
| `embedded` | Default (embedded local search) | OSS-Adoption | Open Source (MIT/Apache-2.0) |
| `candle` | `contextra-infer-candle` | Local Candle Inference | Open Source (MIT/Apache-2.0) |
| `compliance` | `audit-export`, `avv-generator` | Kommerziell | Trade-Secret / Kommerziell (`compliance`) |