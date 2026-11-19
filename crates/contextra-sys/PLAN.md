# contextra-sys — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/crates/contextra-sys.md`; einzige zugelassene Unsafe-Insel für OS-Primitiven. **Ausgangslage:** Low-level-Code ist vorhanden, aber zwanzig Nicht-Test-Panikmuster und eine zu breite historische API verhindern die Freigabe.

1. Begrenze die Exportfläche auf `LockedMemory<T>`, `MmapHandle` und geprüfte POSIX/Windows-Dateioperationen; verschiebe fachliche Vault-/Speicherlogik in ihre Besitzer-Crates.
2. Audit aller `unsafe`-Blöcke: minimale Kapselung, unmittelbarer `// SAFETY:`-Nachweis, Pointer-/Längen-/Ausrichtungsvalidierung und kein Safe-API-Weg zu UB.
3. Implementiere fehlbares `mlock`/`munlock`, `madvise` und Mapping; `LockedMemory` zeroized vor Freigabe und ist nicht clonbar, wenn es Geheimnisse trägt.
4. Ersetze alle Nicht-Test-Panikpfade durch `SysError`, auch in Plattform-Fallbacks und Bench-Shared-Code.
5. Teste Unix und Windows per CI-Matrix, inklusive Mmap-Grenzen, Lock-Failure und Zeroize-Beobachtung.

**Abnahme:** Unsafe-Audit je Block, Miri/Plattformtests, Zero-Panic-Gate und ausschließlich dokumentierte Safe-Exports.
