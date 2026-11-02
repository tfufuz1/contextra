# AGENTS – contextra-rank

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Reciprocal Rank Fusion für die Hybridsuche.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`.

**Verboten:** andere Fusionsverfahren, Kalibrierung, Drift-Erkennung, Score-Normalisierung als Pflicht (D-26)

**Entscheidungen:** D-26; `k_rrf = 60`; deterministische Tie-Breaks nach `doc_tag` (8.11)

**Tests:** T-RANK-1

**Befunde zu prüfen:** F-31
