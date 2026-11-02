# AGENTS – contextra

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Öffentliche API und Verdrahtung. Enthält keine Logik.

**Ring und Abhängigkeiten:** Ring 4. Erlaubt: `types`, `durable-fs`, `engine` (D-88).

**Verboten:** Logik, Profile und Rewriter (D-89), `EncryptionMode` (D-05), `LicenseGate` (D-23), Re-Exporte von `DocId`, `chunker`, `rank` (D-89), Erzeugung des Tresors (D-95)

**Entscheidungen:** D-66, D-89, D-95; Module `api`, `wiring`, `error`

**Tests:** T-API-1 (trybuild), T-API-2, T-API-3, T-ARCH-7

**Befunde zu prüfen:** F-26, F-34
