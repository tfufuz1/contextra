# contextra-durable-fs — Crate-Spezifikation

**Ebene:** E0 (Kern-Closure leer)  
**Bindung:** `docs/SPEC.md` §2

---

## 1 Zweck

Durabilitätsgarantien für Dateisystem-Operationen: atomare Schreibvorgänge, fsync-Wrapper, Rename-basierte Commits.

## 2 Abhängigkeiten

Keine internen Abhängigkeiten.

## 3 Rolle in der Löschkette

- Liefert die fsync-Primitive, die das Erasure-Journal für die 4 fsync-Ankerpunkte nutzt.
- `DurableFile::write_and_sync` garantiert, dass ein Journal-Eintrag durabel ist, bevor `Ok(())` zurückgegeben wird.
- `AtomicRename` für SSTable-Rewrites im Purge-Pfad.
