# AGENTS.md — `contextra-durable-fs`

> **Ring 0** · Klasse `[K]` · Reifegrad `(in `capabilities.toml` **nicht geführt**, D6)` · Unsafe: nein · Rolle: Dateisystem-Härtung: Durability-Flush, atomares Rename, Health-State-Machine, Fehlerinjektion.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-durable-fs --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `sys` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `sys`, `ports` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]`, `deny(clippy::unwrap_used/expect_used/panic)`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Schulden

- **Eintrag in `capabilities.toml` fehlt** (Ring 0, `may_depend_on = [sys, ports]`): WP 1.1.
- **Eigene `Vfs`-Typfamilie** (`vfs.rs`: `Vfs`, `VfsFile`, `VfsOpenOptions`, `RealFs`) neben `contextra_ports::Vfs`. Das ist die „zwei historischen Vfs-Konzepte“-Schuld (D7). Kein Dep auf `ports`.
- `store` hängt **nicht** von diesem Crate ab (nutzt direkt `std::fs`/`tokio::fs`, ~77 Treffer).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `vfs.rs` | `RealFs`, `RealFsFile`, eigener `Vfs`-Trait |
| `lib.rs` | `FlushKind`, `FsClass`, `flush_file`, `flush_dir`, `rename_durable`, `DurableHandle<F: FaultInjector>`, `PoisonedHandle` |
| `health.rs` | `HealthState`, `HealthEvent`, `AtomicHealthState` (Healthy→Degraded→ReadOnly→Poisoned) |
| `fault_injector.rs` | `FaultInjector`, `NoFault` |
| `journal.rs` | `PendingDeleteJournal`: aufgeschobene Datei-Löschungen (mechanisch) |
| `scrub.rs` | `scrub_if_safe`: Überschreiben, nur wenn das Dateisystem es zulässt |

## 4. Verträge & Invarianten

1. **Fehler propagieren (INV-FSYNC-1):** nach `fsync`-Fehler ist der Handle `Poisoned`; weder Ack noch Retry auf demselben Handle.
2. **Gesundheitsübergänge** laufen ausschließlich über `HealthState::transition`; `allows_writes()` ist die einzige Schreibfreigabe.
3. `rename_durable` = tmp-Datei schreiben, `fsync`, `rename`, Parent-Verzeichnis `fsync`.
4. `scrub`/`journal` sind **mechanisch**: keine Rechts- oder Nachweislogik (M7). Nachweise erzeugt `contextra-erasure`.
5. `FaultInjector` ist generisch (Monomorphisierung): `NoFault` kostet im Produktionspfad nichts.

## 5. Ports

**Ist:** definiert einen eigenen `Vfs`. **Soll (WP 2.1):** implementiert `contextra_ports::Vfs` (`RealFs`) und löscht die eigene Trait-Familie.

## 6. Nebenläufigkeit & Performance

- `AtomicHealthState` ist lock-frei (Atomics). Kein Shared Mutable State außer diesem Zustand.
- Blockierend: in `async`-Kontext nur über `spawn_blocking`.

## 7. Anti-Patterns

- Zweiten `Vfs`-Trait weiter ausbauen.
- `let _ = flush_dir(..)`.
- Plattformabhängiges Verhalten ohne `FsClass`-Prüfung annehmen (z. B. Scrub auf CoW-/Flash-Medien als Beweis werten).

## 8. Tests & Messung

```bash
cargo test -p contextra-durable-fs --locked
cargo clippy -p contextra-durable-fs --all-targets -- -D warnings
```

- Vorhanden: 8 Testdateien in `tests/`. Ergänze Konformitätslauf gegen `ports::contract_tests` (feature `contract-tests`) sobald WP 2.1 steht.
- Chaos: `FaultInjector` für Torn Writes, `ENOSPC`, `fsync`-Fehler.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 1.1 | in `capabilities.toml` aufnehmen |
| 2.1 | an `ports::Vfs` binden, eigene Vfs-Typen entfernen |
| 4.4 | `store`/`vector` auf `Vfs` umstellen |
