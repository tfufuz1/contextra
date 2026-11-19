# contextra-text — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§5–7 und `docs/crates/contextra-text.md`. **Ausgangslage:** BM25 und Morphologie bestehen, aber 38 Dateien mit lokalen und kanonischen Typduplikaten, E2-Kompatibilität und 48 Nicht-Test-Panikmustern.

1. Konsolidiere auf einen invertierten Index mit kanonischem `DocId`; behalte die deterministische deutsche Normalisierung, Stemming und Komposita-Zerlegung als reine E1-Logik.
2. Entferne `types`/`ports`-Kompatibilitäts-Features aus dem Defaultpfad und die lokalen Alias-Dubletten; `default = []`, Closure nur `ids, env`.
3. Ergänze Lineage je Posting/abgeleitetem Artefakt und Gate-Prüfung vor Aufnahme in den Top-k-Heap; abgewiesene Kandidaten müssen durch weitere Postings ersetzt werden.
4. Implementiere Layer-Hooks: Gate-Filter, Segmentrotation/Rewriting, Korrektur von df/dl und atomischer Generationwechsel; Witness zählt Posting-Residuen.
5. Ersetze Produktions-`unwrap`/`expect` in Tokenizer, Posting, BM25 und MemoryIndex durch `TextError`; überprüfe Eingabe- und Größenlimits.

**Abnahme:** deterministische Token-/BM25-Golden-Tests, Gate-/Auffülltest, Segment-Purge mit Statistikvergleich, Lineage-Kaskade und Zero-Panic-Gate.
