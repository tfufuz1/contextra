# AGENTS.md — `contextra-durable-fs`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.5](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-durable-fs` stellt `RealVfs` bereit — die einzige produktive Implementierung des `Vfs`-Ports aus `contextra-ports`. Es garantiert crash-sichere Dateisystemoperationen, atomare Datei-Ersetzungen und best-effort Datenbereinigung unter Verwendung von `contextra-sys`.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 1 (Dateisystem-Durability)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-sys`
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]` erzwungen; Low-Level-Systemaufrufe delegieren ausschließlich an `contextra-sys`).
  - Kein Verwerfen von I/O- oder Sync-Fehlern (`let _ =` auf `sync_*` ist strikt verboten).
  - Kein direktes Überschreiben persistenter Datendateien ohne Staging und atomares Rename.

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.5)

| Modul | Inhalt & Verantwortung |
|---|---|
| `real_vfs` | Vollständige Implementierung des konsolidierten `Vfs`-Traits für reale POSIX-/macOS-Dateisysteme. |
| `file` | `RealVfsFile` implementiert `VfsFile` mit synchronen und gepufferten I/O-Methoden. |
| `atomic` | Crash-sicheres Schreibmuster: Temp-Datei schreiben → `sync_all` Datei → atomares `rename` → `sync_dir` Verzeichnis. |
| `scrub` | Best-effort physisches Überschreiben mit Nullen (`ZeroFill`) vor dem Unlinken, gefolgt von `sync_data`. Dokumentiert explizit die technischen Grenzen auf Flash-/CoW-Medien. |
| `probe` | Validierung beim Start (`HEADER`, Verzeichnisprüfung, Test-Rename, Verzeichnis-`fsync`, Lock-Prüfung). |

## 4. Kritische Invarianten
1. **Zweistufige Haltbarkeit:** Ein atomares Rename (`rename_durable`) gilt erst dann als erfolgreich abgeschlossen, wenn sowohl die Zieldatei als auch das übergeordnete Verzeichnis synchronisiert (`sync_dir`) wurden.
2. **Propagierung aller Sync-Fehler:** Sämtliche Rückgabewerte von `sync_data()`, `sync_all()` und `sync_dir()` werden verbindlich geprüft und propagiert.
3. **Dateisystem-Rechte (0600 / 0700):** Neu angelegte Verzeichnisse erzwingen Modus `0700`, Datendateien erzwingen Modus `0600`. Abweichende Rechte beim Start führen zu `Io`-Fehlern.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-1.3: Veraltete Löschfragmente entfernen**
  - Vollständiges Entfernen des alten `PendingDeleteJournal` (das Löschjournal wird ausschließlich in `contextra-erasure` geführt).
- **[ ] WP-2.6: Vfs konsolidieren (D-04)**
  - Zusammenführen der doppelten `Vfs`-Definitionen auf den einheitlichen Port aus `contextra-ports`.
  - Bereitstellung einer vollständigen, sauberen `RealVfs`-Instanz für `contextra-store`.

## 6. Verifikation & Tests
- **Vfs-Conformance-Suite:** Ausführung gegen `ports::contract_tests`.
- **Fehlerinjektions-Tests:** Verhalten bei vollen Festplatten (`ENOSPC`) und synchronen I/O-Fehlern (`EIO`).
- **T-PERM-1:** Test auf korrekte Durchsetzung der Dateirechte (0600/0700).
