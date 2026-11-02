# AGENTS – contextra-types

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O.

**Ring und Abhängigkeiten:** Ring 0. Erlaubt: keine (Ring 0).

**Verboten:** I/O, `async`, Glob-Re-Exporte, Feature-Schalter, `DocId(u64)` (D-08), `saos` (außerhalb Soll)

**Entscheidungen:** D-08, D-09, D-21, D-27, D-70, D-87, D-90, D-97

**Tests:** T-KEY-1, T-DOCREF-1, T-ISO-2, T-ISO-3, T-ARCH-1, T-ARCH-5, T-LIMIT-1

**Befunde zu prüfen:** F-12, F-30, F-31 (`saos` 940 Zeilen), F-36
