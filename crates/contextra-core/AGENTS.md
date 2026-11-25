# AGENTS.md — contextra-core
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.7

## 1. Zweck
Rückwärtskompatible Strangler-Fassade (Deprecated Strangler Facade) in Ring 0. Sie re-exportiert als Migrationsbrücke Typen, Traits, MVCC-Komponenten und IPC-Schnittstellen aus den modularisierten Ring-0-Crates (`contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire`).

## 2. Modul-Karte
| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Einzige Quelldatei — Re-Exports aller Ring-0 Bausteine mit Deprecation-Markern (`#![forbid(unsafe_code)]`) |

## 3. Invarianten

- `INV-CORE-STRANGLER-FACADE`: `contextra-core` enthält selbst keine neuen Geschäftslogik-Implementierungen, sondern delegiert ausschließlich an modulare Ring-0 Crates.
- `INV-CORE-NO-IO`: Crate-Ebene ist rein deklarativ und frei von I/O-, Async- oder Dateisystem-Operationen (Rule P26).

## 4. Verboten / Anti-Patterns

- Erfinden neuer lokaler `ContextraError`-Varianten in `contextra-core` (Zentrale Error-Enum liegt in `contextra-types`).
- Einfügen von `unsafe`-Code in `lib.rs` (Crate erzwingt `#![forbid(unsafe_code)]`).

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Die Fassade selbst verwaltet keine eigenen Locks oder Async-Laufzeiten.
- Re-exportierte Typen folgen den Concurrency-Garantien der jeweiligen Ziel-Crates.

## 6. Verifikation

```bash
cargo test -p contextra-core
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-core
cargo xtask check-ring0-async-purity
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- Sämtliche Exporte in `lib.rs` tragen `#[deprecated]`, da der Direktzugriff auf `contextra-types`, `contextra-ports`, `contextra-mvcc` und `contextra-wire` bevorzugt wird.
