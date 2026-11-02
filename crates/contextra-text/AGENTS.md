# AGENTS – contextra-text

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** BM25-Volltextindex als verschlüsselte Segmente mit deutschem Analyzer.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`, `crypto`.

**Verboten:** Postings im LSM (D-20), Klartextterme im Index (D-21), Präfix-, Wildcard- oder Regex-Suche (SPEC 1.4), variable Präfixe, ein Analyzerwechsel ohne Neuaufbau (D-81)

**Entscheidungen:** D-20, D-49, D-69, D-71, D-81

**Tests:** T-TEXT-1 bis T-TEXT-4, T-TEXT-ANALYZER-1, T-TEXT-PHRASE-1, BM-TEXT, BM-ERASE-WA-TEXT

**Befunde zu prüfen:** F-05, F-36
