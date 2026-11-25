# AGENTS.md — contextra-testkit
> Ring Tooling · stable · Quelle: capabilities.toml · Spec: K.28, §0, §3, §4, §20

## 1. Zweck

`contextra-testkit` stellt Determinismus- und Testinfrastruktur bereit (`ManualClock` für Zeit-Injektion gemäß P28, `FaultVfs` für Crash- und I/O-Fehlerinjektion, `InMemoryStorageEngine` für speicherbasierte Tests und `ReferenceModel`).
Die Crate dient ausschließlich als Testinfrastruktur.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
| :--- | :--- |
| `src/lib.rs` | Re-Exporte der Test-Utilities (`ManualClock`, `FaultVfs`, `InMemoryStorageEngine`, `ReferenceModel`). |
| `src/manual_clock.rs` | `ManualClock` zur exakten manuellen Zeitsteuerung in Tests. |
| `src/fault_vfs.rs` | `FaultVfs` für Fehlersimulation (Bitrot, Partial Writes, Sync Failures, Crash Simulation). |
| `src/in_memory_store.rs` | `InMemoryStorageEngine` für Null-I/O In-Memory Transaktions- und Storage-Testing. |
| `src/reference_model.rs` | Referenzmodell zur Validierung von Invarianten und Resultaten gegen eine vereinfachte Vergleichsimplementierung. |

## 3. Invarianten

- **INV-TOOLING-ONLY:** `contextra-testkit` darf NIEMALS als reguläre Abhängigkeit (`[dependencies]`) einer Produktions-Crate eingebunden werden, sondern AUSSCHLIESSLICH als Test-Abhängigkeit (`[dev-dependencies]`).
- **INV-DETERMINISTIC-TIME:** `ManualClock` gewährt nanosekundengenaue deterministische Zeitfortschaltung über `.advance_nanos()` / `.advance_secs()`.

## 4. Verboten / Anti-Patterns

- **VERBOTEN:** Einbindung von `contextra-testkit` in Produktions-Binaries oder Produktions-Bibliotheken (`Cargo.toml` `[dependencies]`).
- **VERBOTEN:** Verwendung von `InMemoryStorageEngine` für Tests, die explizit SSI-Tracking erwarten (`supports_ssi_tracking()` gibt `false` zurück).

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Thread-safe für parallele Testausführung durch `parking_lot::{Mutex, RwLock}`.
- Frei von Async-Runtimes im Crate-Kern; nutzt Tokio nur in Dev-Dependencies / Tests.

## 6. Verifikation

```bash
cargo test -p contextra-testkit
cargo xtask check-agents-integrity
```

## 7. Bekannte Lücken / SOLL

- `InMemoryStorageEngine` unterstützt beabsichtigt kein SSI-Tracking, um einfachen Unit-Tests ohne Lock-Overhead zu dienen.

