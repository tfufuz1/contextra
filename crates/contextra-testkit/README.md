# contextra-testkit

`contextra-testkit` stellt Test-Infrastruktur und Determinismus-Werkzeuge bereit (Tooling).

## Zweck

Liefert deterministische Test-Doubles zur Validierung von Fault-Injection, Crash-Simulation und Simulationstests:
- `FaultVfs`: Deterministische I/O-Fehlerinjektion.
- `ManualClock`: Manuell steuerbarer Test-Zeitgeber (P28).
- `InMemoryStorageEngine`: In-Memory-Backend für schnelle Integrationstests.

## Ring-Zugehörigkeit & Status

- **Ring:** Tooling / Testkit
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Fault VFS:** `FaultVfs`, `FaultConfig`
- **In-Memory Storage:** `InMemoryStorageEngine`
- **Manual Clock:** `ManualClock`

## Architektur & Verweise

Details zur Test-Infrastruktur und Determinismus-Grundsätzen (P28) finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §15.
