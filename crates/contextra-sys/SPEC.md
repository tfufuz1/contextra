# contextra-sys — Spezifikation (Baukasten v3)

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v3, 10.11.2026).
> **Faktenbasis:** Statische Analyse und Isolationsnachweis am aktuellen Repository-Stand.
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem `tools/`-Bericht, *[S]* aus `docs/SPEC.md` übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-sys/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Plattformnahe Primitive (mmap, mlock, Durability-Hilfen, ACL-Prüfungen, POSIX-Syscalls) und gesperrter Speicher (`sys::vault`). *[S]*
Dieses Crate dient als **autarker, modularer Baustein (Klasse A, Ebene E0)** in der Contextra-Baukastenarchitektur und kann ohne interne Abhängigkeiten in beliebigen anderen Rust-Lösungen eingesetzt werden.

## 2. Zielzustand (docs/SPEC.md v3 §3.1) [S]

| Merkmal | Ziel | Ist-Zustand | Status |
|---|---|---|---|
| Gruppe | foundation | foundation | Erfüllt *[V]* |
| Ebene (Kern-Closure) | 0 (E0) | 0 (keine internen Crates) | Erfüllt *[V]* |
| Kern-Closure | — | leer | Erfüllt *[V]* |
| Default-Features | — | leer | Erfüllt *[V]* |
| Fehlertyp (R3) | `SysError` (Alias für `Error`) | `SysError` (thiserror, `#[non_exhaustive]`) | Erfüllt *[V]* |
| Brücke (hinter Feature `workspace`) | keine | keine (vollständig autark) | Erfüllt *[V]* |
| Modularitäts-Score | 100/100 | 100/100 (Exzellent / Baustein) | Erfüllt *[T]* |
| Zero-Panic in Produktion | 0 | 0 (geprüft mit `pattern_diagnostic.py`) | Erfüllt *[T]* |

## 3. Modulorganisation und Baukasten-Exporte

Das Crate strukturiert systemnahe Funktionalität in thematische Submodule und bietet sowohl direkte Re-Exporte als auch logische Baukasten-Zielmodule:

1. **`mem` (Speicherschutz & Locking):**
   - [`mlock`]: `mem_lock`, `mem_lock_typed`, `mem_lock_with_policy`, `mem_unlock`, `apply_dont_dump`, `MemLockPolicy`.
   - [`vault`]: RAII-Puffer ([`LockedRegions`], [`VaultBuffer`]) mit volatile zeroize und Memory-Pinning.
2. **`fsmeta` (Dateisystemsicherheit & ACLs):**
   - [`acl_win32`]: Restriktive Zugriffsrechte (`set_restrictive_file_acl`, `verify_file_acl_owner_only`) für POSIX (`0600`) und Windows DACL.
3. **`map` (Speicherabbildung):**
   - [`mmap`]: Sicheres Nur-Lese-Mmap ([`mmap_readonly`], [`SealedFile`], [`SealedMap`]) mit Boundaries-Prüfung.
4. **`durability` (Dauerhaftigkeit & Synchronisation):**
   - [`durable_flush_file`], [`durable_flush_dir`], [`write_all_durable`], [`DurableFile`].
   - Erzwingt Handle-Poisoning bei E/A-Flush-Fehlern (INV-SYS-FSYNC-1).
5. **`posix` (Ressourcenverwaltung):**
   - Deskriptor-Limits (`ensure_fd_limit`, `set_fd_limit`) und atomare Deskriptor-Ersetzung (`reopen_and_dup2`).
6. **`error` (Fehlermodell):**
   - [`Error`] / [`SysError`] mit typisierten Varianten und OS-Errno.

## 4. Invarianten und Sicherheitsanforderungen

- **INV-IO-1 / INV-ENCRYPT-REST-2:** Ausschließlich unveränderliche Nur-Lese-Speicherabbildungen (`mmap_readonly`); keine schreibenden Mmaps im Rust-Adressraum zur Vermeidung von Data Races.
- **INV-SYS-MEMLOCK-1:** Sensible Puffer werden vor der Speicherfreigabe mit Nullen überschrieben (`write_volatile` + `compiler_fence`), per `madvise(MADV_DONTDUMP)` aus Core-Dumps ausgeschlossen und erst danach entsperrt (`munlock`).
- **INV-SYS-FSYNC-1:** Fehlgeschlagene Synchronisationen vergiften das betroffene `DurableFile`-Handle permanent; keine stillen Retries bei defekten E/A-Zuständen.
- **SPEC §6.4:** Keine unzulässigen Garantieversprechen in Doku/Kommentaren (`FORBIDDEN_WORDING` = 0).

## 5. Abnahme und Isolation

Das Crate kompiliert und testet autark:
```bash
python3 /home/jules/self_created_tools/test_isolated_crate.py
python3 tools/pattern_diagnostic.py -c contextra-sys --prod-only
python3 tools/decouple_inspector.py -c contextra-sys
```
