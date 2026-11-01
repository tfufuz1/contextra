# contextra-text

BM25-Segmentindex, Block-Max WAND, blinde Term-Tags und Segment-Purge.

- **Ring:** Ring 2 (Volltextindexierung)
- **Normativ:** [docs/SPEC.md §8.11](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-text` stellt eine Inverted-Index Volltextsuchmaschine mit BM25-Scoring und Block-Max WAND auf unveränderlichen Segmenten (SPEC §9.7) bereit:
- `tokenizer`: NFKC-Normalisierung, Kleinschreibung, Unicode-Wortgrenzen nach UAX #29.
- `buffer`: In-Memory Textpuffer für neue Dokumente.
- `segment`: AEAD-verschlüsselte Segmente (`SegmentWriter`, `SegmentReader`, Zielgröße 32 MiB).
- `bm25` / `wand`: Deterministisches Scoring ohne Term-Klartext auf Platte.
- `layer`: Implementiert den `ErasureLayer`-Port für Textindizes.
- `docseg`: Abbildung `doc_tag ↔ segment_id` für gezielte Segment-Rewrites.

## Invarianten
- `#![forbid(unsafe_code)]`
- **Kein Term-Klartext auf Datenträgern:** Postings enthalten ausschließlich blinde `term_tag`-Hashes.
- Jedes Segment ist AEAD-verschlüsselt via `HKDF(K_enc, "ctx/text/seg/v1" ‖ segment_id)`.
- Globale Statistiken werden dynamisch aus Segmentheadern berechnet; keine unbereinigbaren globalen Zähler.
- Segment-Purge: Puffer-Bereinigung, Segment-Rewrite ohne Ziel-Tags, Alt-Segment löschen mit `Vfs::exists`-Prüfung.
- Veraltete Module (`morphology`, `data`, `domain`) und alte LSM-Postinglisten sind entfernt.
