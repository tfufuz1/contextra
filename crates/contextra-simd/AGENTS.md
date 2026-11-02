# AGENTS – contextra-simd

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`.

**Verboten:** u8-VNNI-Kernel (D-28), `unsafe` außerhalb der Insel, Kernel ohne Cross-Check gegen die Skalar-Referenz

**Entscheidungen:** D-28, D-29

**Tests:** T-SIMD-1, T-UNSAFE-1

**Befunde zu prüfen:** F-18
