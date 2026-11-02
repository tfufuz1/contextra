# Routine: Unsafe-Prüfung

- **Takt:** wöchentlich
- **Ziel:** Gültigkeit der `// SAFETY:`-Kommentare und Speichersicherheit der Unsafe-Inseln `contextra-sys` und `contextra-simd` (D-29, T-UNSAFE-1)
- **Eingaben:** Miri auf der Skalar-Referenz von `contextra-simd`, Miri auf `contextra-types` und `contextra-mvcc`, AddressSanitizer auf `contextra-sys` (siehe [`docs/COMPILATION.md`](../../../docs/COMPILATION.md)). Miri auf dem gesamten Workspace ist nicht vorgesehen.
- **Harte Grenzen:** höchstens 15 Dateien in einem Crate je PR. Scope: `crates/contextra-sys/**`, `crates/contextra-simd/**`.
- **Stopp:** Undefined Behavior in Fremdcode ohne direkte Korrektur wird gemeldet, nicht überdeckt.
- **Ergebnis:** Bericht mit Protokoll der Miri- und ASan-Läufe im PR.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).
