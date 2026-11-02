# AGENTS – contextra-sys

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Einzige `unsafe`-Stelle für Betriebssystemaufrufe: `fsync`, `mlock`, Prozesshärtung.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`.

**Verboten:** Windows-Code (D-84), `mmap` und `mmap_readonly` (D-94), `unsafe fn` ohne dokumentierte Vorbedingung, Anbieten von Funktionen mit offener Vorbedingung in Produktcrates

**Entscheidungen:** D-29, D-55, D-63, D-84, D-94

**Tests:** T-UNSAFE-1 (Miri, ASan), T-FSYNC-1, T-DUMP-1, T-SEC-2

**Befunde zu prüfen:** F-18, F-22, F-32
