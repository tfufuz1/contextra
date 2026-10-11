# AGENTS.md — contextra-durable-fs
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.8

## 1. Zweck
Dauerhafte Dateisystem-Operationen und synchrone Disk-Flushes (`fsync`/`fdatasync`, atomare Ersetzung, sichere Löschung) für Contextra-Komponenten.

## 2. Modul-Karte
| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Dateisystem-Hilfsfunktionen für atomares Ersetzen (`atomic_replace`) und dauerhaftes Löschen (`durable_remove`) |

## 3. Invarianten

- `INV-DURABLE-FS-SAFE`: Rein sichere Dateisystem-Verwaltung; Fehler werden deterministisch als `std::io::Result` propagiert.
- `INV-DURABLE-FS-NO-PANIC`: Keine `panic!`, `unwrap()`, oder `expect()` in Produktions-Dateisystem-Pfaden.

## 4. Verboten / Anti-Patterns

- Nicht-atomares Überschreiben von Daten-Dateien ohne `.tmp` Staging.
- Ignorieren von Dateisystem-Flush-Fehlern.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Die Dateisystem-Operationen laufen synchron ab.
- Lock-Sperren werden auf Aufrufer-Ebene in den jeweiligen Storage-/Index-Crates gehalten.

## 6. Verifikation

```bash
cargo test -p contextra-durable-fs
cargo xtask check-agents-integrity
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- Keine.
