<!-- doc-stamp: contextra-ingest v0.1.0 (SPEC C-10.3) -->
# MODULE.md — `contextra-ingest`

## Zweck
`contextra-ingest` ist das Ring-4 Adapter-Crate für Dokumentformat-Extraktion und strukturbewusstes Chunking. Es entkoppelt das Parsing von ~30 Dateiformaten vom Datenbank-Kern.

## Vertrag (Typtabelle)

| Typ / Schnittstelle | Beschreibung |
|---|---|
| `DocumentExtractor` | Trait für Format-Extraktoren (`formats()`, `extract()`) |
| `ExtractorRegistry` | Registrierungstabelle mit `catch_unwind` Panik-Isolierung |
| `DocumentIr` | Flache Zwischenrepräsentation (Blöcke, Metadaten, Warnungen) |
| `CanonicalText` | Deterministischer Fließtext mit exakten Block-Spans |
| `IngestLimits` & `ResourceBudget` | Zip-Bomb- und Ressourcenschutz |
| `IngestError` | Typisierte Fehler (Mapping zu `ContextraError`) |
| `ingest(&IngestRequest, &IngestConfig, &ExtractorRegistry)` | Haupt-Pipeline |

## Metadatenschema `contextra.chunk.v1`
Jedes erzeugte `ContextChunk` speichert im JSON-Feld `metadata`:
- `schema`: `"contextra.chunk.v1"`
- `chunker_version`: `"ingest-chunk/1"`
- `chunk_id`: BLAKE3-Hex (16 Byte / 32 Zeichen)
- `chunk_index`, `chunk_total`, `format`, `strategy`
- `source_span`: `{ "start": usize, "end": usize }`
- `locator`: `{ page?, slide?, sheet?, range?, message?, attachment? }`
- `breadcrumb?`, `heading_level?`, `doc_title?`

## Revier-Tabelle
- `crates/contextra-ingest/` (Exklusives Ingestion-Revier)
- `fuzz/` (Fuzzing-Ziele für alle Extraktoren)

## SPEC-Abweichungen
- **Obergrenze Hard Limit**: 614 Tokens statt 1024 (orientiert an Dense Embedding Modellen 512 + 20%).
- **Overlap**: 20 % der Window-Größe statt 10–15 %.
- **`source_span`-Semantik**: Bezieht sich strikt auf den `CanonicalText`.

## Nicht-Ziele
- Keine I/O- oder Netzwerk-Zugriffe.
- Keine Änderung an `contextra-types` (`ContextChunk`).
