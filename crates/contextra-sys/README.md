# contextra-sys

Plattformnahe Systemprimitive, Dateisystem-Synchronisation und Speicher-Härtung.

- **Ring:** Ring 1 (Unsafe-Insel 1 von 2, SPEC §8.3, D-29)
- **Normativ:** [docs/SPEC.md §8.3](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-sys` kapselt plattformspezifische I/O- und Speicherprimitive hinter sicheren Rust-Funktionen:
- `fsync`: `fsync_file`, `fdatasync`, `F_FULLFSYNC` unter macOS. Fehler werden ausnahmslos propagiert.
- `dir`: `flush_dir`, `rename_durable` (atomares Rename inklusive Verzeichnis-`fsync`).
- `mem`: `mlock_best_effort`, `munlock` zum Schutz von Schlüsseln im RAM vor Paging.
- `perm`: `set_mode`, `check_mode` (0600 für Dateien, 0700 für Verzeichnisse).

## Invarianten & Insel-Doktrin
- Jeder einzelne `unsafe`-Block erfordert ein `// SAFETY:`-Kommentar mit Bounds, Alignment und Gültigkeitsnachweis.
- Safe Funktionen ohne `unsafe` dürfen keine Vorbedingungen vor dem Aufrufer verbergen.
- Fail-Closed: Nicht unterstützte Plattformen verweigern kontrolliert den Dienst.
