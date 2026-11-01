# AGENTS.md — `contextra-sys`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.3](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-sys` ist eine der genau zwei zugelassenen `unsafe`-Inseln (D-29) im Gesamtsystem. Es kapselt plattformspezifische Systemprimitive (I/O-Synchronisation, Dateisystem-Dauerhaftigkeit, Speicher-Locking und Berechtigungen) hinter sicheren Rust-Schnittstellen.

## 2. Ring, Abhängigkeiten & Insel-Regeln
- **Ring:** Ring 1 (Plattform-Primitive, `unsafe`-Insel 1 von 2)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `libc`
- **Insel-Doktrin (P2, SPEC §8.3):**
  - Jeder einzelne `unsafe`-Block erfordert zwingend eine präzise `// SAFETY:`-Dokumentation, die Alignment, Pointer-Gültigkeit, Bounds und Plattformannahmen belegt.
  - Öffentliche Funktionen ohne `unsafe` dürfen **keinen** unsicheren Inhalt mit offenen Vorbedingungen haben. Bekannte historische Schwachstellen (`mmap_readonly`, `reopen_and_dup2`) müssen als `unsafe fn` deklariert oder restlos beseitigt werden.

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.3)

| Modul | Inhalt & Verantwortung |
|---|---|
| `fsync` | Robuste Datei-Synchronisation: `fsync_file`, `fdatasync`, `F_FULLFSYNC` unter macOS; Fehler werden ausnahmslos propagiert. |
| `dir` | Verzeichnis-Dauerhaftigkeit: `flush_dir`, `rename_durable` (atomares Rename inklusive Verzeichnis-`fsync`). |
| `mem` | Speicherhygiene: `mlock_best_effort`, `munlock` zum Schutz sensibler Schlüssel im RAM gegen Auslagerung (Swap). |
| `perm` | Dateisystem-Rechte: `set_mode`, `check_mode` (Erzwingung von 0600 für Dateien, 0700 für Verzeichnisse). |

## 4. Kritische Invarianten
1. **Kein verschlucktes `fsync`:** Rückgabewerte von OS-Sync-Befehlen werden niemals mit `let _ =` verworfen, sondern als `io::Result<()>` propagiert.
2. **Fail-Closed bei Nicht-POSIX:** Plattformen ohne Unterstützung verweigern die Arbeit kontrolliert mit typisiertem Fehler.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-1.2: Wire- und IPC-Reste entfernen**
  - Vollständiges Entfernen aller historischen `contextra-wire`-Primitiven aus `contextra-sys`.
- **[ ] WP-2.6: Primitive für konsolidiertes Vfs bereitstellen**
  - Saubere Bereitstellung von `flush_dir`, `rename_durable` und `fsync_file` für `contextra-durable-fs`.
- **[ ] WP-6.4: Unsafe-Audit & Miri-Validierung**
  - Lückenlose Dokumentation aller `// SAFETY:`-Blöcke; Miri-Testläufe für Pointer- und Memory-Mapping-Operationen.

## 6. Verifikation & Tests
- **T-UNSAFE-1:** CI-Prüfung aller `unsafe`-Blöcke auf `// SAFETY:`-Begründung.
- **Miri-Suite:** Prüfung der Pointer- und Speicherausrichtung.
- **T-PERM-1:** Validierung der strikten Dateirechte-Prüfung.
