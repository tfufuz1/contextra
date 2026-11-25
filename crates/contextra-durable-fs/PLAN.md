# Implementierungsplan: contextra-durable-fs

**Ebene:** L1 · **Ziel-Zustand:** sichere Dateischicht mit Poisoning, atomarem Ersetzen, positionsbasierten Handles und Fault-Injection (DEC-39).
**Basis:** SPEC R14 §2.1, §2.5 (F-3), §2.7, AN-02…AN-04, AN-10, INV-FSYNC-1, INV-VFS-ATOMIC-1; R15 §2.3.

## 1. Ist-Befund (gemessen)

- 8 Dateien, 1 910 Zeilen. Module: `error`, `fault_injector`, `health`, `journal`, `platform`, `scrub`, `vfs`.
- **Abweichung 1:** `Cargo.toml` hat **keine** Abhängigkeit auf `contextra-sys`. SPEC §2.1 verlangt `durable-fs → sys`. Stattdessen nutzt `lib.rs`/`platform` direkt `std::fs`.
- **Abweichung 2:** `crc32fast` optional. SPEC verlangt `crc32c` (FIX-21).
- **Abweichung 3:** `vfs.rs` ist dateiweise (`Vfs`). SPEC verlangt positionsbasierte Handles; `Vfs` soll weg.
- 16 Treffer verbotener APIs, z. B. `std::fs` in `journal.rs`, `lib.rs`, `scrub.rs`.
- `serde`, `tracing` optional. Prüfen, ob für Journal nötig.

## 2. Entfernen

| Ziel | Aktion |
|---|---|
| `crc32fast` | durch `crc32c` ersetzen (FIX-21) |
| dateiweises `Vfs` in `vfs.rs` | durch Handle-API ersetzen; Modul bleibt als Name, Inhalt neu |
| direktes `std::fs` in `journal.rs`, `scrub.rs`, `lib.rs` | durch `contextra_sys`-Funktionen ersetzen |
| `serde` in `journal`, falls nur für Debug | entfernen |

## 3. Ändern

- **`lib.rs`:** Öffentliche Oberfläche auf `Fs`-Konformanz ausrichten. Jede Methode gibt `Result` mit typisierter Fehlerklasse F-1…F-4 zurück.
- **`error.rs`:** Klassen F-1 transient (`EINTR`, Retry mit Backoff), F-2 Ressource (`ENOSPC`), F-3 hart (`EIO`, Fsync), F-4 Integrität. Jede Klasse hat einen Konstruktor; F-3 vergiftet das Handle.
- **`health.rs`:** Zustandsautomat `Normal → DiskFull → Normal` mit Hysterese, `Normal → Poisoned` bei F-3 (Anhang A, „Instanz“).
- **`platform.rs`:** Plattformspezifika nur über `contextra-sys`; hier nur Policy.
- **`scrub.rs`:** Prüfsummenläufe über Dateien mit CRC32C; Fehler werden als F-4 gemeldet, nicht still korrigiert.

## 4. Implementieren

1. **Handle-API** (INV-FSYNC-1, INV-VFS-ATOMIC-1):
   - `open_ro`, `create_new`, `open_rw`.
   - `read_at(offset, buf)`, `write_at(offset, buf)`, `read_ranges(&[(off, len)])`.
   - `append_prealloc(bytes)`: nur in vorallokierte Bereiche (AN-10).
   - `preallocate(len)`, `sync_data()`, `sync_all()`.
   - `rename(from, to)` im selben Verzeichnis, danach `sync_dir`.
   - `lock()` als Advisory-Lock mit Rückgabe eines `FsLock`-Guards.
2. **Poisoning:** Nach Fsync-Fehler setzt das Handle `poisoned = true`. Alle weiteren Operationen geben `Err(Poisoned)` zurück. Kein Retry auf demselben Handle (AN-03).
3. **Atomares Ersetzen:** Schreiben in `.tmp`, `sync_data`, `rename`, `sync_dir`. Ein Test verifiziert die Reihenfolge über `FaultInjector`.
4. **Journal:** Redo-unabhängiges Journal für Metadaten des Dateisystems (Anlage, Rename, Unlink). Format wird vor G-F eingefroren.
5. **`FaultInjector`:** jede Operation hat eine Nummer; der Injector kann Fehler, Crash (Abbruch nach N Operationen) oder verkürzte Schreibvorgänge (Torn Write, AN-02) auslösen.
6. **Konformanztest** für `ports::kernel::Fs` (siehe Abschnitt 5).

## 5. Tests

- **Konformanz-Suite** für `Fs`: jede Operation, jede Fehlerklasse. Dieselbe Suite läuft gegen `MemFs` (Sim) und gegen das echte Dateisystem.
- **Crash-Sweep:** für jede Operation ein Abbruchpunkt; nach Neustart ist der Zustand entweder alt oder neu, nie gemischt.
- **Torn-Write-Test:** Schreiben in der Mitte eines Blocks, Prüfsumme muss den Riss erkennen.
- **ENOSPC-Test:** `preallocate` schlägt fehl, Handle bleibt nutzbar für Lesen.
- Pflicht laut SPEC: **Crash-Sweep wird mit jeder Phase um neue Operationen erweitert, nie nachgezogen** (§13.1). Die Testliste ist daher append-only.

## 6. Abhängigkeiten nach Umbau

Erlaubt: `thiserror`, `contextra-sys`, `contextra-ports` (nur Trait-Definition), optional `tracing`. Verboten: `std::fs` außerhalb von `contextra-sys`, `memmap2`, `libc`, `crc32fast`.

## 7. Gate und Reihenfolge

1. `Cargo.toml`: Abhängigkeit auf `contextra-sys` eintragen. `crc32c` statt `crc32fast`.
2. Handle-API und Poisoning, mit Konformanz-Suite gegen `MemFs`.
3. Atomares Ersetzen und Journal.
4. `FaultInjector` und Crash-Sweep.
5. **G-0:** Konformanz `Fs` grün (Linux). **Vorbedingung für P1.**

**Risiko:** Ohne Journal-Format-Festlegung vor G-F sind spätere Änderungen Migrationen. Journal bleibt bis dahin als „nicht stabil“ markiert.
