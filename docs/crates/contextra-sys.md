# contextra-sys — Crate-Spezifikation

**Ebene:** E0 (Kern-Closure leer)  
**Bindung:** `docs/SPEC.md` §2

---

## 1 Zweck

Unsafe-Insel für Betriebssystem-nahe Operationen: `mlock`, `madvise`, Dateisystem-Synchronisation.

## 2 Abhängigkeiten

Keine internen Abhängigkeiten. `libc` als externe Abhängigkeit.

## 3 Regeln

- Einziges Crate (neben `simd`) mit erlaubtem `unsafe`.
- Jeder `unsafe`-Block MUSS einen `// SAFETY:` Kommentar haben.
- Alle Funktionen MÜSSEN durch Wrapper in sicherer API exponiert werden.
