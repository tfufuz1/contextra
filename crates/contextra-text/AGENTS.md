# AGENTS – contextra-text

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.9, 9.7, D-20, D-49, D-69, D-71, D-81, E-L5, E-L15). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** BM25-Volltextindex als verschlüsselte Segmente mit deutschem Analyzer.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`, `crypto`. Implementiert `TextIndex` und `ErasureLayer`.

## Fachprofil (Arbeitsrolle)

Information-Retrieval-Ingenieur mit Kenntnis von Searchable-Encryption-Leakage-Profilen. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- BM25 (Parameter, Längennormalisierung), WAND/Block-Max-Pruning, Postinglisten-Kompression (Delta, Bitpacking), Segmentierung und Merge, Korpusstatistiken unter Löschung.
- Deutsche Morphologie (Dekomposition von Komposita, Stoppwörter, Umlaute, `ß`) und deren Determinismus.
- Identitätsentkopplung: Postings über segmentlokale `doc_ord` statt roher Dokumentkennung; Wörterbuch- und Häufigkeitsangriffe auf gelöschte Bestände; Löschung per Segment-Rewrite und Dateischlüssel-Vernichtung statt Posting für Posting.

## Normative Anker

- **Segmentindex statt LSM-Postings (D-20).** Segmente `text/<tenant_tag>/<coll_tag>/<file_id>.seg` (D-75), unveränderlich, Block-AEAD unter zufälligem Dateischlüssel (D-85). Inhalt nach F.7: Header (`bucket`, `bucket_count`, `analyzer_id`, `analyzer_version`, `lang`, `positions`), Termwörterbuch **nur als `term_tag`**, Postings als Delta-codierte `doc_ord`-Listen in 128er-Blöcken mit Block-Max-Metadaten, optional Positionen, Dokumenttabelle, Segmentstatistik (Lösch-Bitmap entfällt nach SPEC 5.1.0 F.7 / D-121).
- **Entkopplung (D-69):** Postings tragen nur `doc_ord`. Die Bindung `doc_ord → (cluster_tag, doc_tag)` liegt als Envelope unter dem DEK des Dokuments (AAD: Segment-`file_id` ‖ `doc_ord`). Nach Schlüsselvernichtung: `key_coverage = unlinked`, Restrisiko `TextIndexUntilPurge` bis zum Segment-Rewrite (T-TEXT-4, T-KEYCOV-1).
- **Ein Dokument liegt zu jedem Zeitpunkt in genau einem Segment oder im Textpuffer** (D-49). Der Rewrite-Aufwand eines Subject-Erase ist Σ Größe der berührten Segmente (D-71c, `segments_rewritten`, BM-ERASE-WA-TEXT). Segmentziel adaptiv (8 MiB; ab 1 Mio. Chunks 32 MiB); `bucket_count = 1` in v1 (D-71).
- **Erase:** Rewrite im **ersten Purge-Fenster**; alle Statistiken (df, Längensummen, Blockmaxima) werden exakt neu berechnet (E-L5, T-TEXT-3); altes Segment geht durch Dateischlüssel-Vernichtung und `unlink`.
- **Analyzer (D-81):** `de/1` (UAX-#29, NFKC, Case-Folding, `ß→ss`, Snowball-German, Stoppwortliste, wörterbuchbasierter Dekompounder) und `und/1`; je Collection unveränderlich; `term_tag = tag("ctx/term/v1", analyzer_id ‖ lang ‖ normalisierter_term)`.
- **Suche:** `k1 = 1,2`, `b = 0,75`, Block-Max-WAND, globale Statistiken aus Segmentsummen; Bindung nur für Top-k-Kandidaten entschlüsseln; ReadGate-Filter mit Nachladen bis `k` oder Erschöpfung (D-62). Fehlt der DEK einer Bindung, ist der Treffer gelöscht und wird übersprungen.
- **Grenzen (ausgewiesen):** keine Präfix-, Wildcard- oder Regex-Suche (1.4); Wörterbuchangriff gegen `term_tag` setzt `K_idx` voraus (9.7). Termhäufigkeiten und Dokumentlängen sind ein Leakage-Profil des Index und gehören ins Restrisiko, nicht in Schweigen.
- **Determinismus:** gleiche Eingabe ergibt gleiche Ausgabe; Tie-Breaks nach `doc_tag` (8.11).

## Verboten

Postings im LSM, Klartextterme im Index, rohe Dokumentkennung (`DocId`) in Postings, variable Präfixe, Analyzerwechsel ohne Neuaufbau, Gleitkomma-Reihenfolgen, die von Threads abhängen, Feature-Schalter.

## Abnahme

`T-TEXT-1` bis `T-TEXT-4`, `T-TEXT-ANALYZER-1` (Goldenset Komposita, Umlaute, `ß`), `T-TEXT-PHRASE-1`, `T-KEYCOV-1`, `T-RAW-4`; `BM-TEXT` (p99 ≤ 30 ms bei 1 Mio. Chunks), `BM-ERASE-WA-TEXT` (Entscheidung `bucket_count > 1` nach D-71d); Relevanz-Regression (nDCG auf Referenzkorpus) und Purge-Proptests als Arbeitshinweis.

## Arbeitspakete

WP-7.1 (Segmentindex, Analyzer, BMW, Positionen, Entkopplung, Rewrite). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-text --locked
cargo test --locked --test tk_conformance_text_index
cargo test -p contextra-arch-tests --locked
```
