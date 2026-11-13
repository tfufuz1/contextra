# AGENTS.md — `contextra-ingest`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-ingest` · **Ring:** 4 · **Version:** 0.1.0
- **Beschreibung:** Document format extractors and structure-aware chunking adapter crate
- **src:** 80 Dateien, 17015 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 1):** `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 17):** `blake3`, `calamine`, `cfb`, `csv`, `encoding_rs`, `html5ever`, `lopdf`, `mail-parser`, `markup5ever_rcdom`, `quick-xml`, `serde`, `serde_json`, `thiserror`, `toml`, `unicode-segmentation`, `yaml-rust2`, `zip`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-types`; eingehend –
- **Features (17):** `all-formats`, `default`, `fmt-docx`, `fmt-email`, `fmt-odf`, `fmt-ole`, `fmt-pdf`, `fmt-pptx`, `fmt-sheets`, `fmt-structured`, `fmt-text`, `fmt-web`, `fuzz`, `types-compat`, `workspace`, `xml`, `zip-container`

### Öffentliche API (grob, aus src/)
- struct (68): `Block`, `CanonicalText`, `ChunkDraft`, `ChunkingConfig`, `ContextChunk`, `ContextChunkLink`, `DecodedHtml`, `DetectHints`, `Detection`, `DocId`, `DocMeta`, `DocumentIr`, `DocumentParseResult`, `DocxExtractor`, `DomArena` … (+53)
- enum (14): `BlockKind`, `Boundary`, `ChunkStrategy`, `Confidence`, `FormatId`, `IngestError`, `JsonValue`, `LimitKind`, `NodeKind`, `NoteKind`, `ParsedBlockKind`, `TabularMode`, `WarningCode`, `XmlNode`
- trait (2): `DocumentExtractor`, `IrChunker`
- fn (131): `analyze_headers_footers`, `append_index`, `append_key`, `append_nested`, `as_u64`, `attr`, `build_chunk_metadata`, `build_tables_from_rows`, `canonical`, `canonical_hash`, `charge_cells`, `charge_decompressed`, `charge_pages`, `charge_text`, `charge_xml_events` … (+116)
- const (1): `MAX_DETECT_BYTES`
- mod (72): `archive`, `asciidoc`, `canonical`, `cfb_reader`, `chunking`, `common`, `compat`, `config`, `content`, `csv_fmt`, `detect`, `doc`, `document`, `docx`, `email` … (+57)
- pub use: `chunking::types::{ContextChunk, ContextChunkLink, DocId}`, `crate::chunking::stubs::EmailChunker`, `crate::chunking::stubs::PagedChunker`, `crate::chunking::stubs::ProseChunker`, `crate::chunking::stubs::StructuredChunker`, `crate::chunking::stubs::TabularChunker`, `email_impl::EmailChunker`, `error::{IngestError, LimitKind}`, `paged_impl::PagedChunker`, `prose_impl::*` … (+2)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
