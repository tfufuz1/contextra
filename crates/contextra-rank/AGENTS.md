# AGENTS – contextra-rank

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.11, D-26, D-62, T-RANK-1). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Reciprocal Rank Fusion für die Hybridsuche.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`.

## Fachprofil (Arbeitsrolle)

Ranking- und Evaluationsingenieur (Reciprocal Rank Fusion, Retrieval-Metriken). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- RRF mit festem `k`, stabile deterministische Tie-Breaks, Fusion bei fehlenden Kandidaten, Robustheit bei leeren oder teilweisen Ergebnislisten.
- Metriken (nDCG, Recall@k, MRR) als Prüfwerkzeug, keine Score-Skalenannahmen zwischen BM25 und Vektor (RRF arbeitet auf Rängen).

## Normative Anker

- **Nur RRF (D-26)** mit `k_rrf = 60`; deterministische Tie-Breaks nach `doc_tag` (8.11, T-RANK-1).
- Fusion ist eine reine Funktion ohne I/O, ohne Zeit und ohne Zufall (P7). **Folgerung:** gleiche Eingabe liefert gleiche Ausgabe.
- Die Hybridsuche liefert `SearchPage { hits, exhausted }`; `rank` verkürzt nie still (D-62, T-HYB-1).

## Verboten

Andere Fusionsverfahren, Kalibrierung, Drift-Erkennung, Erklär-Module, Score-Normalisierung als Pflicht (D-26); Gleitkomma-Ergebnisse, die von der Reihenfolge der Eingaben abhängen; Features.

## Abnahme

`T-RANK-1`, `T-HYB-1`; Proptests auf Permutations- und Duplikat-Invarianz und Golden-Rankings als Arbeitshinweis. Der Umfang außerhalb des Soll (`dibud`, `explain`, `drift`, Teile von `fusion`; F-31, F-36) wird zurückgebaut.

## Arbeitspakete

WP-8.2 (RRF, Hybridsuche). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-rank --locked
cargo test -p contextra-arch-tests --locked
```
