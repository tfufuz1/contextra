# AGENTS.md — `contextra-sys`

> **Ring 0 · Unsafe-Insel** · Klasse `[K]` · Reifegrad `stable` · Rolle: autarke Betriebssystem-Primitive (Durability-Flush, `mmap`, Memory-Lock, FD-Limits, Datei-ACL) hinter einer **sicheren** API.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`. Stand 2026-11-07, statisch gelesen, nicht kompiliert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-sys --locked` (keine Contextra-Abhängigkeit, keine Features) |
| Contextra-Abhängigkeiten (Ist = Soll) | keine |
| Lints | `#![allow(unsafe_code)]` (Insel) mit `#![deny(unsafe_op_in_unsafe_fn)]`, `#![warn(missing_docs)]` |
| Unsafe-Umfang | ~20 `unsafe`-Treffer, plattformabhängig (`posix.rs`, `mmap.rs`, `mlock.rs`, `acl_win32.rs`) |
| Konsumenten (Ist, laut Cargo) | `store`, `vector`, `durable-fs`, `engine`, `db` (die Kanten `engine`/`db` entfallen laut D9) |

## 2. Abhängigkeiten

Keine. Das Crate steht am Boden des DAG und darf nie ein `contextra-*`-Crate importieren. (SPEC §4.4 erlaubt `types` nur falls nötig; heute nicht gebraucht.)

## 3. Modul-Karte (am Code verifiziert)

| Datei | Öffentliche API | Verantwortung |
|---|---|---|
| `durability.rs` | `durable_flush_file`, `durable_flush_dir`, `write_all_durable` | plattformgerechte Durability-Primitive (`fdatasync`, `F_FULLFSYNC`, Verzeichnis-Sync) |
| `mmap.rs` | `mmap_readonly` | schreibgeschützte Abbildung |
| `mlock.rs` | `mem_lock`, `mem_lock_with_policy`, `mem_unlock`, `MemLockPolicy` | Speicher sperren (Swap-Schutz) |
| `vault.rs` | `LockedRegions` | verwaltete gesperrte Regionen für Schlüsselmaterial |
| `posix.rs` | `ensure_fd_limit`, `set_fd_limit`, `reopen_and_dup2` | FD-Limits, Umleitung |
| `acl_win32.rs` | `set_restrictive_file_acl`, `verify_file_acl_owner_only` | Windows-ACL (nur Best Effort, INV-PLATFORM-1) |

## 4. Verträge & Invarianten

1. **Sichere API nach außen (M9).** Jede `pub fn` ist sicher aufrufbar. Alle Vorbedingungen (Längen, Ausrichtung, Dateityp) werden **vor** dem `unsafe`-Block geprüft und als `Err` gemeldet.
2. **Jeder `unsafe`-Block trägt `// SAFETY:`** mit den eingehaltenen Invarianten (Pointer-Gültigkeit, Alignment, Lebensdauer, Aliasing). `clippy::undocumented_unsafe_blocks = deny` gilt workspace-weit. Auch Plattformcode (Win32) ist eingeschlossen.
3. **`mmap` nur für vollständig prüfsummenverifizierte, unveränderliche, volumenlokale Dateien** (INV-IO-1). Andernfalls `pread` über `Vfs`. Zero-Copy-`mmap` auf unverschlüsselten Volumes ist nur zulässig, wenn `doctor` ein verschlüsseltes Volume nachweist (INV-ENCRYPT-REST-2).
4. **Durability-Fehler propagieren (INV-FSYNC-1).** `durable_flush_*` gibt jeden Fehler zurück. Der Aufrufer vergiftet den Handle und wiederholt **nie** auf demselben Handle.
5. **Plattformäquivalente deklarieren.** Funktionen ohne Äquivalent (`mlock` unter Windows) liefern einen benannten Fehler oder eine `MemLockPolicy`-Entscheidung, nie ein stilles `Ok`.
6. **Kein SIGBUS durch `mmap`.** Truncate-Races sind zu bedenken: Datei nach Verifikation nicht mehr verkürzbar halten (Lock) oder `pread` verwenden.
7. **Keine Heap-Allokation mit Geheimnissen ohne Zeroization** in `LockedRegions` (INV-HEAP-ZERO-1).

## 5. Ports

Konsumiert und implementiert keine Ports (das Crate ist port-frei, damit es überall einsetzbar ist). Ein `Vfs`, das diese Primitive nutzt, lebt in `contextra-durable-fs`.

## 6. Nebenläufigkeit & Performance

Keine globalen Zustände, kein Runtime-Bezug. `ensure_fd_limit`/`set_fd_limit` verändern **prozessweiten** Zustand: nur aus der Composition Root aufrufen, nie aus Bibliothekscode. `durable_flush_*` blockiert: in `async`-Kontext nur über `spawn_blocking`.

## 7. Anti-Patterns

- `unsafe` ohne `// SAFETY:` oder mit nur beschreibendem Kommentar („reads pointer“).
- `let _ = durable_flush_dir(..)`: verschluckt eine Durability-Verletzung.
- `mmap` auf Dateien, die konkurrierend wachsen oder schrumpfen können.
- Plattform-`cfg`-Zweige, die das Verhalten **semantisch** ändern, ohne dass `doctor` es ausweist.
- Neue `unsafe`-Nutzung außerhalb dieses Crates, `simd` und `wire` (M9): stattdessen hier eine sichere Funktion ergänzen.

## 8. Tests & Messung

```bash
cargo test -p contextra-sys --locked
cargo +nightly miri test -p contextra-sys            # soweit Miri die Syscalls unterstützt; sonst Referenztest
cargo clippy -p contextra-sys --all-targets -- -D warnings
```

- Gegen eine sichere Referenz testen (Property-Test: `mmap_readonly` liefert dieselben Bytes wie `std::fs::read`).
- Fehlerinjektion über `contextra-durable-fs::FaultInjector` für den Flush-Pfad.
- Es gibt **kein** `tests/`-Verzeichnis im Crate. Ergänze eines für Plattform- und Referenztests.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.2 | `mmap`-Abstraktion mit versiegelten Speicherbereichen (`SealedMap`), robuste Signal-/Fehlerbehandlung ohne Panics |
| 5.6 | Unsafe-Audit: jeder Block mit `// SAFETY:`, Miri-Abdeckung belegen |
| D9 | Tote Kante `engine → sys` und `db → sys` entfernen (laut PLAN D9) |
