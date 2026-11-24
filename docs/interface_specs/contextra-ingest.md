# Schnittstellenspezifikation: `contextra-ingest`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-ingest` |
| Version | workspace |
| Pfad | `crates/contextra-ingest` |
| Rolle | Dokumentenerfassung |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Modul
>
> # Zweck & Architektur
> Komponente des  Subsystems (Ring 4 Adapter-Schicht).
> Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
>
> # Invarianten
> - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
> - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
> - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `blake3` | normal | workspace | nein |  |
| `calamine` | normal | workspace | ja |  |
| `cfb` | dev | workspace | nein |  |
| `contextra-types` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `csv` | normal | workspace | ja |  |
| `encoding_rs` | normal | workspace | ja |  |
| `html5ever` | normal | workspace | ja |  |
| `lopdf` | dev | workspace | nein |  |
| `mail-parser` | normal | workspace | ja |  |
| `markup5ever_rcdom` | normal | workspace | ja |  |
| `proptest` | dev | registry | nein |  |
| `quick-xml` | normal | workspace | ja |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `toml` | normal | workspace | ja |  |
| `unicode-segmentation` | normal | workspace | nein |  |
| `yaml-rust2` | normal | workspace | ja |  |
| `zip` | dev | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-db`, `contextra-engine`, `contextra-ingest-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `all-formats` | `fmt-text`, `fmt-web`, `fmt-docx`, `fmt-pptx`, `fmt-odf`, `fmt-sheets`, `fmt-pdf`, `fmt-ole`, `fmt-email`, `fmt-structured` |
| `default` | `fmt-text` |
| `fmt-docx` | `zip-container`, `xml` |
| `fmt-email` | `dep:mail-parser`, `dep:encoding_rs` |
| `fmt-odf` | `zip-container`, `xml` |
| `fmt-ole` | `dep:cfb` |
| `fmt-pdf` | `dep:lopdf` |
| `fmt-pptx` | `zip-container`, `xml` |
| `fmt-sheets` | `dep:calamine`, `dep:csv`, `dep:encoding_rs`, `zip-container`, `xml` |
| `fmt-structured` | `dep:yaml-rust2`, `dep:toml`, `xml` |
| `fmt-text` | `dep:encoding_rs` |
| `fmt-web` | `dep:html5ever`, `dep:markup5ever_rcdom`, `dep:encoding_rs`, `zip-container`, `xml` |
| `fuzz` | – |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types` |
| `xml` | `dep:quick-xml`, `dep:encoding_rs` |
| `zip-container` | `dep:zip` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 80 |
| Quellzeilen | 21140 |
| Öffentliche Items (Modul-Ebene) | 242 |
| davon `fn` | 76 |
| davon `struct` | 69 |
| davon `enum` | 14 |
| davon `trait` | 2 |
| davon `const` | 1 |
| davon `mod` | 80 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 12 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 9 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 21 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 24 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 317 / 317 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `canonical` | `crates/contextra-ingest/src/canonical.rs` | 323 |
| `chunking::common` | `crates/contextra-ingest/src/chunking/common.rs` | 240 |
| `chunking::config` | `crates/contextra-ingest/src/chunking/config.rs` | 191 |
| `chunking::email` | `crates/contextra-ingest/src/chunking/email/mod.rs` | 473 |
| `chunking::fallback` | `crates/contextra-ingest/src/chunking/fallback.rs` | 78 |
| `chunking::meta` | `crates/contextra-ingest/src/chunking/meta.rs` | 166 |
| `chunking` | `crates/contextra-ingest/src/chunking/mod.rs` | 128 |
| `chunking::paged` | `crates/contextra-ingest/src/chunking/paged/mod.rs` | 52 |
| `chunking::paged::page` | `crates/contextra-ingest/src/chunking/paged/page.rs` | 810 |
| `chunking::prose::markdown_parse` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs` | 856 |
| `chunking::prose::merge` | `crates/contextra-ingest/src/chunking/prose/merge.rs` | 163 |
| `chunking::prose` | `crates/contextra-ingest/src/chunking/prose/mod.rs` | 516 |
| `chunking::prose::sentence` | `crates/contextra-ingest/src/chunking/prose/sentence.rs` | 155 |
| `chunking::prose::split` | `crates/contextra-ingest/src/chunking/prose/split.rs` | 419 |
| `chunking::structured` | `crates/contextra-ingest/src/chunking/structured/mod.rs` | 540 |
| `chunking::stubs` | `crates/contextra-ingest/src/chunking/stubs.rs` | 80 |
| `chunking::tabular::helpers` | `crates/contextra-ingest/src/chunking/tabular/helpers.rs` | 121 |
| `chunking::tabular` | `crates/contextra-ingest/src/chunking/tabular/mod.rs` | 780 |
| `chunking::types` | `crates/contextra-ingest/src/chunking/types.rs` | 71 |
| `compat` | `crates/contextra-ingest/src/compat/mod.rs` | 5 |
| `compat::types` | `crates/contextra-ingest/src/compat/types.rs` | 91 |
| `detect` | `crates/contextra-ingest/src/detect/mod.rs` | 768 |
| `error` | `crates/contextra-ingest/src/error.rs` | 106 |
| `extractor` | `crates/contextra-ingest/src/extractor.rs` | 223 |
| `format` | `crates/contextra-ingest/src/format.rs` | 250 |
| `formats::docx::document` | `crates/contextra-ingest/src/formats/docx/document.rs` | 609 |
| `formats::docx` | `crates/contextra-ingest/src/formats/docx/mod.rs` | 317 |
| `formats::docx::notes` | `crates/contextra-ingest/src/formats/docx/notes.rs` | 212 |
| `formats::docx::numbering` | `crates/contextra-ingest/src/formats/docx/numbering.rs` | 147 |
| `formats::docx::styles` | `crates/contextra-ingest/src/formats/docx/styles.rs` | 217 |
| `formats::email::eml` | `crates/contextra-ingest/src/formats/email/eml.rs` | 391 |
| `formats::email::html` | `crates/contextra-ingest/src/formats/email/html.rs` | 226 |
| `formats::email::mbox` | `crates/contextra-ingest/src/formats/email/mbox.rs` | 204 |
| `formats::email` | `crates/contextra-ingest/src/formats/email/mod.rs` | 57 |
| `formats` | `crates/contextra-ingest/src/formats/mod.rs` | 45 |
| `formats::odf::content` | `crates/contextra-ingest/src/formats/odf/content.rs` | 473 |
| `formats::odf::meta` | `crates/contextra-ingest/src/formats/odf/meta.rs` | 93 |
| `formats::odf` | `crates/contextra-ingest/src/formats/odf/mod.rs` | 198 |
| `formats::odf::presentation` | `crates/contextra-ingest/src/formats/odf/presentation.rs` | 231 |
| `formats::ole::cfb_reader` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs` | 151 |
| `formats::ole::doc` | `crates/contextra-ingest/src/formats/ole/doc.rs` | 426 |
| `formats::ole` | `crates/contextra-ingest/src/formats/ole/mod.rs` | 64 |
| `formats::ole::msg` | `crates/contextra-ingest/src/formats/ole/msg.rs` | 574 |
| `formats::ole::ppt` | `crates/contextra-ingest/src/formats/ole/ppt.rs` | 291 |
| `formats::pdf` | `crates/contextra-ingest/src/formats/pdf/mod.rs` | 330 |
| `formats::pdf::text_processing` | `crates/contextra-ingest/src/formats/pdf/text_processing.rs` | 259 |
| `formats::pptx` | `crates/contextra-ingest/src/formats/pptx/mod.rs` | 127 |
| `formats::pptx::presentation` | `crates/contextra-ingest/src/formats/pptx/presentation.rs` | 64 |
| `formats::pptx::slide` | `crates/contextra-ingest/src/formats/pptx/slide.rs` | 503 |
| `formats::sheets::csv_fmt` | `crates/contextra-ingest/src/formats/sheets/csv_fmt.rs` | 332 |
| `formats::sheets::excel` | `crates/contextra-ingest/src/formats/sheets/excel.rs` | 380 |
| `formats::sheets` | `crates/contextra-ingest/src/formats/sheets/mod.rs` | 68 |
| `formats::structured::json` | `crates/contextra-ingest/src/formats/structured/json.rs` | 425 |
| `formats::structured` | `crates/contextra-ingest/src/formats/structured/mod.rs` | 92 |
| `formats::structured::path` | `crates/contextra-ingest/src/formats/structured/path.rs` | 115 |
| `formats::structured::toml` | `crates/contextra-ingest/src/formats/structured/toml.rs` | 189 |
| `formats::structured::xml` | `crates/contextra-ingest/src/formats/structured/xml.rs` | 384 |
| `formats::structured::yaml` | `crates/contextra-ingest/src/formats/structured/yaml.rs` | 260 |
| `formats::text::asciidoc` | `crates/contextra-ingest/src/formats/text/asciidoc.rs` | 212 |
| `formats::text::encoding` | `crates/contextra-ingest/src/formats/text/encoding.rs` | 211 |
| `formats::text::markdown` | `crates/contextra-ingest/src/formats/text/markdown.rs` | 111 |
| `formats::text` | `crates/contextra-ingest/src/formats/text/mod.rs` | 93 |
| `formats::text::rst` | `crates/contextra-ingest/src/formats/text/rst.rs` | 297 |
| `formats::text::rtf` | `crates/contextra-ingest/src/formats/text/rtf.rs` | 398 |
| `formats::text::tex` | `crates/contextra-ingest/src/formats/text/tex.rs` | 302 |
| `formats::text::txt` | `crates/contextra-ingest/src/formats/text/txt.rs` | 79 |
| `formats::web::encoding` | `crates/contextra-ingest/src/formats/web/encoding.rs` | 129 |
| `formats::web::epub` | `crates/contextra-ingest/src/formats/web/epub.rs` | 412 |
| `formats::web::html` | `crates/contextra-ingest/src/formats/web/html.rs` | 983 |
| `formats::web` | `crates/contextra-ingest/src/formats/web/mod.rs` | 60 |
| `fuzz_support` | `crates/contextra-ingest/src/fuzz_support.rs` | 37 |
| `ir` | `crates/contextra-ingest/src/ir.rs` | 253 |
| `crate` | `crates/contextra-ingest/src/lib.rs` | 39 |
| `limits` | `crates/contextra-ingest/src/limits.rs` | 94 |
| `pipeline` | `crates/contextra-ingest/src/pipeline.rs` | 177 |
| `registry` | `crates/contextra-ingest/src/registry.rs` | 164 |
| `safe::archive` | `crates/contextra-ingest/src/safe/archive.rs` | 381 |
| `safe` | `crates/contextra-ingest/src/safe/mod.rs` | 24 |
| `safe::opc` | `crates/contextra-ingest/src/safe/opc.rs` | 259 |
| `safe::xml` | `crates/contextra-ingest/src/safe/xml.rs` | 366 |

## 6. Re-Exports (`pub use`)

- `pub use types::` — `crates/contextra-ingest/src/chunking/mod.rs:20` (Modul `chunking`)
- `pub use crate::chunking::stubs::EmailChunker` — `crates/contextra-ingest/src/chunking/mod.rs:23` (Modul `chunking::email`)
- `pub use crate::chunking::stubs::PagedChunker` — `crates/contextra-ingest/src/chunking/mod.rs:26` (Modul `chunking::paged`)
- `pub use crate::chunking::stubs::ProseChunker` — `crates/contextra-ingest/src/chunking/mod.rs:29` (Modul `chunking::prose`)
- `pub use crate::chunking::stubs::StructuredChunker` — `crates/contextra-ingest/src/chunking/mod.rs:32` (Modul `chunking::structured`)
- `pub use crate::chunking::stubs::TabularChunker` — `crates/contextra-ingest/src/chunking/mod.rs:35` (Modul `chunking::tabular`)
- `pub use prose_impl::*` — `crates/contextra-ingest/src/chunking/stubs.rs:23` (Modul `chunking::stubs`)
- `pub use email_impl::EmailChunker` — `crates/contextra-ingest/src/chunking/stubs.rs:71` (Modul `chunking::stubs`)
- `pub use paged_impl::PagedChunker` — `crates/contextra-ingest/src/chunking/stubs.rs:79` (Modul `chunking::stubs`)
- `pub use structured_impl::StructuredChunker` — `crates/contextra-ingest/src/chunking/stubs.rs:80` (Modul `chunking::stubs`)
- `pub use chunking::types::` — `crates/contextra-ingest/src/lib.rs:34` (Modul `crate`)
- `pub use error::` — `crates/contextra-ingest/src/lib.rs:35` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `canonical`

#### `struct CanonicalText`
- Ort: `crates/contextra-ingest/src/canonical.rs:25`
```rust
pub struct CanonicalText
```
Dokumentation: Erzeugter kanonischer Text mit exakten Byte-Spans je Block in der `DocumentIr`.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text` | `String` | `crates/contextra-ingest/src/canonical.rs:26` |
| `block_spans` | `Vec<Range<usize>>` | `crates/contextra-ingest/src/canonical.rs:27` |

#### `fn sanitize_text`
- Ort: `crates/contextra-ingest/src/canonical.rs:34`
```rust
pub fn sanitize_text(input: &str) -> (String, u32)
```
Dokumentation: Entfernt NUL und C0-Steuerzeichen (außer \t und \n), U+FEFF (BOM), Unicode-Tag-Zeichen U+E0000..U+E007F und Bidi-Overrides U+202A..U+202E / U+2066..U+2069. Normalisiert \r\n und \r zu \n. Behält ZWJ/ZWNJ. Liefert gereinigten String und die Anzahl entfernter/normalisierter Zeichen.

### Modul `chunking`

#### `mod common`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:13`
```rust
pub mod common
```

#### `mod config`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:14`
```rust
pub mod config
```

#### `mod email`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:22`
```rust
pub mod email
```

#### `mod fallback`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:15`
```rust
pub mod fallback
```

#### `mod meta`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:16`
```rust
pub mod meta
```

#### `mod paged`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:25`
```rust
pub mod paged
```

#### `mod prose`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:28`
```rust
pub mod prose
```

#### `mod structured`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:31`
```rust
pub mod structured
```

#### `mod stubs`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:17`
```rust
pub mod stubs
```

#### `mod tabular`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:34`
```rust
pub mod tabular
```

#### `mod types`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:18`
```rust
pub mod types
```

#### `struct ChunkDraft`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:50`
```rust
pub struct ChunkDraft
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text` | `String` | `crates/contextra-ingest/src/chunking/mod.rs:51` |
| `span` | `Range<usize>` | `crates/contextra-ingest/src/chunking/mod.rs:52` |
| `locator` | `Locator` | `crates/contextra-ingest/src/chunking/mod.rs:53` |
| `breadcrumb` | `Option<String>` | `crates/contextra-ingest/src/chunking/mod.rs:54` |
| `heading_level` | `Option<u8>` | `crates/contextra-ingest/src/chunking/mod.rs:55` |

#### `trait IrChunker`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:62`
```rust
pub trait IrChunker: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn strategy(&self) -> ChunkStrategy` | `crates/contextra-ingest/src/chunking/mod.rs:63` |  |
| `fn draft( &self, ir: &DocumentIr, canon: &CanonicalText, cfg: &ChunkingConfig, ) -> Result<Vec<ChunkDraft>, IngestError>` | `crates/contextra-ingest/src/chunking/mod.rs:64` |  |

#### `fn draft_block_range`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:96`
```rust
pub fn draft_block_range( ir: &DocumentIr, canon: &CanonicalText, blocks_range: Range<usize>, strategy: ChunkStrategy, cfg: &ChunkingConfig, ) -> Result<Vec<ChunkDraft>, IngestError>
```

#### `fn draft_document`
- Ort: `crates/contextra-ingest/src/chunking/mod.rs:75`
```rust
pub fn draft_document( ir: &DocumentIr, canon: &CanonicalText, cfg: &ChunkingConfig, ) -> Result<Vec<ChunkDraft>, IngestError>
```

### Modul `chunking::common`

#### `fn chunk_text_with_overlap`
- Ort: `crates/contextra-ingest/src/chunking/common.rs:120`
```rust
pub fn chunk_text_with_overlap(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<&str>
```
Dokumentation: Erzeugt überlappende Text-Chunks mit definierter Fenstergröße und Überlappung (autark ohne Engine).

#### `fn estimate_tokens`
- Ort: `crates/contextra-ingest/src/chunking/common.rs:32`
```rust
pub fn estimate_tokens(text: &str) -> usize
```
Dokumentation: Autarke heuristische Token-Schätzung für Fließtext, Codeblöcke und Tabellen.

#### `fn finalize`
- Ort: `crates/contextra-ingest/src/chunking/common.rs:165`
```rust
pub fn finalize( drafts: Vec<ChunkDraft>, ir: &DocumentIr, _canon: &CanonicalText, cfg: &ChunkingConfig, doc_id: DocId, limits: &IngestLimits, ) -> Result<Vec<ContextChunk>, IngestError>
```

#### `fn token_count`
- Ort: `crates/contextra-ingest/src/chunking/common.rs:26`
```rust
pub fn token_count(text: &str) -> usize
```

#### `fn window_split`
- Ort: `crates/contextra-ingest/src/chunking/common.rs:100`
```rust
pub fn window_split(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<Range<usize>>
```

### Modul `chunking::config`

#### `struct ChunkingConfig`
- Ort: `crates/contextra-ingest/src/chunking/config.rs:22`
```rust
pub struct ChunkingConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/chunking/config.rs:146`
- Impl-Block `crates/contextra-ingest/src/chunking/config.rs:165`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `strategy` | `Option<ChunkStrategy>` | `crates/contextra-ingest/src/chunking/config.rs:23` |
| `max_tokens` | `usize` | `crates/contextra-ingest/src/chunking/config.rs:24` |
| `min_tokens` | `usize` | `crates/contextra-ingest/src/chunking/config.rs:25` |
| `hard_limit_pct` | `u32` | `crates/contextra-ingest/src/chunking/config.rs:26` |
| `overlap_pct` | `u32` | `crates/contextra-ingest/src/chunking/config.rs:27` |
| `include_breadcrumbs` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:28` |
| `split_levels` | `Vec<u8>` | `crates/contextra-ingest/src/chunking/config.rs:29` |
| `prose` | `ProseOptions` | `crates/contextra-ingest/src/chunking/config.rs:31` |
| `tabular` | `TabularOptions` | `crates/contextra-ingest/src/chunking/config.rs:32` |
| `paged` | `PagedOptions` | `crates/contextra-ingest/src/chunking/config.rs:33` |
| `structured` | `StructuredOptions` | `crates/contextra-ingest/src/chunking/config.rs:34` |
| `email` | `EmailOptions` | `crates/contextra-ingest/src/chunking/config.rs:35` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn fingerprint(&self) -> String` | `crates/contextra-ingest/src/chunking/config.rs:167` | Erzeugt einen deterministischen BLAKE3-Fingerprint der Konfiguration (Hex-String). |

#### `struct EmailOptions`
- Ort: `crates/contextra-ingest/src/chunking/config.rs:132`
```rust
pub struct EmailOptions
```
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/chunking/config.rs:137`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `include_quoted` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:133` |
| `header_context` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:134` |

#### `struct PagedOptions`
- Ort: `crates/contextra-ingest/src/chunking/config.rs:94`
```rust
pub struct PagedOptions
```
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/chunking/config.rs:100`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `merge_small_pages` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:95` |
| `join_cross_page_paragraphs` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:96` |
| `notes_with_slide` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:97` |

#### `struct ProseOptions`
- Ort: `crates/contextra-ingest/src/chunking/config.rs:42`
```rust
pub struct ProseOptions
```
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/chunking/config.rs:48`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sentence_aware_windows` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:43` |
| `footnotes_with_section` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:44` |
| `keep_lists_together` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:45` |

#### `struct StructuredOptions`
- Ort: `crates/contextra-ingest/src/chunking/config.rs:114`
```rust
pub struct StructuredOptions
```
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/chunking/config.rs:119`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `group_depth` | `u32` | `crates/contextra-ingest/src/chunking/config.rs:115` |
| `batch_small_records` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:116` |

#### `struct TabularOptions`
- Ort: `crates/contextra-ingest/src/chunking/config.rs:72`
```rust
pub struct TabularOptions
```
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/chunking/config.rs:79`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `mode` | `TabularMode` | `crates/contextra-ingest/src/chunking/config.rs:73` |
| `wide_table_cols` | `u32` | `crates/contextra-ingest/src/chunking/config.rs:74` |
| `skip_empty_rows` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:75` |
| `drop_empty_columns` | `bool` | `crates/contextra-ingest/src/chunking/config.rs:76` |

#### `enum TabularMode`
- Ort: `crates/contextra-ingest/src/chunking/config.rs:62`
```rust
pub enum TabularMode
```

Varianten:

- `Auto` — `crates/contextra-ingest/src/chunking/config.rs:63`
- `RowGroups` — `crates/contextra-ingest/src/chunking/config.rs:64`
- `RecordRows` — `crates/contextra-ingest/src/chunking/config.rs:65`

### Modul `chunking::email`

#### `struct EmailChunker`
- Ort: `crates/contextra-ingest/src/chunking/email/mod.rs:28`
```rust
pub struct EmailChunker
```
- Impl-Block (Trait `IrChunker`) `crates/contextra-ingest/src/chunking/email/mod.rs:30`

### Modul `chunking::fallback`

#### `struct FallbackChunker`
- Ort: `crates/contextra-ingest/src/chunking/fallback.rs:25`
```rust
pub struct FallbackChunker
```
- Impl-Block (Trait `IrChunker`) `crates/contextra-ingest/src/chunking/fallback.rs:27`

### Modul `chunking::meta`

#### `fn build_chunk_metadata`
- Ort: `crates/contextra-ingest/src/chunking/meta.rs:22`
```rust
pub fn build_chunk_metadata( canonical_hash: &str, cfg_fingerprint: &str, strategy: ChunkStrategy, draft: &ChunkDraft, chunk_index: u32, chunk_total: u32, ir: &DocumentIr, ) -> Value
```

### Modul `chunking::meta::hex`

#### `fn encode`
- Ort: `crates/contextra-ingest/src/chunking/meta.rs:117`
```rust
pub fn encode(bytes: &[u8]) -> String
```

### Modul `chunking::paged`

#### `mod page`
- Ort: `crates/contextra-ingest/src/chunking/paged/mod.rs:13`
```rust
pub mod page
```

#### `struct PagedChunker`
- Ort: `crates/contextra-ingest/src/chunking/paged/mod.rs:26`
```rust
pub struct PagedChunker
```
- Impl-Block (Trait `IrChunker`) `crates/contextra-ingest/src/chunking/paged/mod.rs:28`

#### `fn draft_paged`
- Ort: `crates/contextra-ingest/src/chunking/paged/mod.rs:46`
```rust
pub fn draft_paged( ir: &DocumentIr, canon: &CanonicalText, cfg: &ChunkingConfig, ) -> Result<Vec<ChunkDraft>, IngestError>
```

### Modul `chunking::paged::page`

#### `fn process_paged_document`
- Ort: `crates/contextra-ingest/src/chunking/paged/page.rs:39`
```rust
pub fn process_paged_document( ir: &DocumentIr, canon: &CanonicalText, cfg: &ChunkingConfig, ) -> Result<Vec<ChunkDraft>, IngestError>
```

### Modul `chunking::prose`

#### `mod markdown_parse`
- Ort: `crates/contextra-ingest/src/chunking/prose/mod.rs:13`
```rust
pub mod markdown_parse
```

#### `mod merge`
- Ort: `crates/contextra-ingest/src/chunking/prose/mod.rs:14`
```rust
pub mod merge
```

#### `mod sentence`
- Ort: `crates/contextra-ingest/src/chunking/prose/mod.rs:15`
```rust
pub mod sentence
```

#### `mod split`
- Ort: `crates/contextra-ingest/src/chunking/prose/mod.rs:16`
```rust
pub mod split
```

#### `struct ProseChunker`
- Ort: `crates/contextra-ingest/src/chunking/prose/mod.rs:37`
```rust
pub struct ProseChunker
```
- Impl-Block (Trait `IrChunker`) `crates/contextra-ingest/src/chunking/prose/mod.rs:39`
- Impl-Block (Trait `IrChunker`) `crates/contextra-ingest/src/chunking/stubs.rs:30`

#### `fn draft_prose`
- Ort: `crates/contextra-ingest/src/chunking/prose/mod.rs:91`
```rust
pub fn draft_prose( ir: &DocumentIr, canon: &CanonicalText, cfg: &ChunkingConfig, ) -> Result<Vec<ChunkDraft>, IngestError>
```

### Modul `chunking::prose::markdown_parse`

#### `struct ParsedListItem`
- Ort: `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:61`
```rust
pub struct ParsedListItem
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `depth` | `u8` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:62` |
| `ordered` | `bool` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:63` |
| `text` | `String` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:64` |
| `span` | `Range<usize>` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:65` |

#### `struct ParsedMarkdownBlock`
- Ort: `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:71`
```rust
pub struct ParsedMarkdownBlock
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kind` | `ParsedBlockKind` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:72` |
| `text` | `String` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:73` |
| `span` | `Range<usize>` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:74` |
| `locator` | `Locator` | `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:75` |

#### `enum ParsedBlockKind`
- Ort: `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:20`
```rust
pub enum ParsedBlockKind
```

Varianten:

- `YamlFrontMatter` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:21`
- `Heading{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:22`
- `FencedCode{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:26`
- `IndentedCode{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:32`
- `Table{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:35`
- `List{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:39`
- `Blockquote{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:42`
- `HtmlBlock{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:45`
- `Note{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:48`
- `ThematicBreak` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:52`
- `Paragraph{` — `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:53`

#### `fn parse_markdown_blocks`
- Ort: `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs:88`
```rust
pub fn parse_markdown_blocks(full_text: &str, base_offset: usize) -> Vec<ParsedMarkdownBlock>
```

### Modul `chunking::prose::merge`

#### `fn merge_small_chunks`
- Ort: `crates/contextra-ingest/src/chunking/prose/merge.rs:21`
```rust
pub fn merge_small_chunks( drafts: Vec<ChunkDraft>, hard_limit: usize, cfg: &ChunkingConfig, ) -> Vec<ChunkDraft>
```

### Modul `chunking::prose::sentence`

#### `struct SentenceSpan`
- Ort: `crates/contextra-ingest/src/chunking/prose/sentence.rs:27`
```rust
pub struct SentenceSpan
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text` | `String` | `crates/contextra-ingest/src/chunking/prose/sentence.rs:28` |
| `span` | `Range<usize>` | `crates/contextra-ingest/src/chunking/prose/sentence.rs:29` |

#### `fn split_sentences`
- Ort: `crates/contextra-ingest/src/chunking/prose/sentence.rs:35`
```rust
pub fn split_sentences(text: &str, base_offset: usize) -> Vec<SentenceSpan>
```

### Modul `chunking::prose::split`

#### `struct SplitContext`
- Ort: `crates/contextra-ingest/src/chunking/prose/split.rs:26`
```rust
pub struct SplitContext<'a>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `span` | `Range<usize>` | `crates/contextra-ingest/src/chunking/prose/split.rs:27` |
| `locator` | `Locator` | `crates/contextra-ingest/src/chunking/prose/split.rs:28` |
| `breadcrumb` | `Option<String>` | `crates/contextra-ingest/src/chunking/prose/split.rs:29` |
| `heading_level` | `Option<u8>` | `crates/contextra-ingest/src/chunking/prose/split.rs:30` |
| `hard_limit` | `usize` | `crates/contextra-ingest/src/chunking/prose/split.rs:31` |
| `cfg` | `&'a ChunkingConfig` | `crates/contextra-ingest/src/chunking/prose/split.rs:32` |

#### `fn split_oversized_code`
- Ort: `crates/contextra-ingest/src/chunking/prose/split.rs:38`
```rust
pub fn split_oversized_code( fence_head: &str, fence_tail: &str, body_lines: &[&str], sctx: &SplitContext<'_>, ) -> Vec<ChunkDraft>
```

#### `fn split_oversized_list`
- Ort: `crates/contextra-ingest/src/chunking/prose/split.rs:221`
```rust
pub fn split_oversized_list(items: &[ParsedListItem], sctx: &SplitContext<'_>) -> Vec<ChunkDraft>
```

#### `fn split_oversized_paragraph`
- Ort: `crates/contextra-ingest/src/chunking/prose/split.rs:292`
```rust
pub fn split_oversized_paragraph(para_text: &str, sctx: &SplitContext<'_>) -> Vec<ChunkDraft>
```

#### `fn split_oversized_table`
- Ort: `crates/contextra-ingest/src/chunking/prose/split.rs:125`
```rust
pub fn split_oversized_table( header_lines: &[String], body_lines: &[String], sctx: &SplitContext<'_>, ) -> Vec<ChunkDraft>
```

### Modul `chunking::structured`

#### `struct StructuredChunker`
- Ort: `crates/contextra-ingest/src/chunking/structured/mod.rs:27`
```rust
pub struct StructuredChunker
```
- Impl-Block (Trait `IrChunker`) `crates/contextra-ingest/src/chunking/structured/mod.rs:29`

### Modul `chunking::stubs`

#### `mod prose_impl`
- Ort: `crates/contextra-ingest/src/chunking/stubs.rs:21`
```rust
pub mod prose_impl
```

#### `struct ProseChunker`
- Ort: `crates/contextra-ingest/src/chunking/stubs.rs:28`
```rust
pub struct ProseChunker
```

#### `struct TabularChunker`
- Ort: `crates/contextra-ingest/src/chunking/stubs.rs:51`
```rust
pub struct TabularChunker
```
- Impl-Block (Trait `IrChunker`) `crates/contextra-ingest/src/chunking/stubs.rs:53`

### Modul `chunking::tabular`

#### `mod helpers`
- Ort: `crates/contextra-ingest/src/chunking/tabular/mod.rs:13`
```rust
pub mod helpers
```

#### `fn draft_tabular`
- Ort: `crates/contextra-ingest/src/chunking/tabular/mod.rs:43`
```rust
pub fn draft_tabular( ir: &DocumentIr, canon: &CanonicalText, cfg: &ChunkingConfig, ) -> Result<Vec<ChunkDraft>, IngestError>
```

### Modul `chunking::tabular::helpers`

#### `fn col_to_letter`
- Ort: `crates/contextra-ingest/src/chunking/tabular/helpers.rs:17`
```rust
pub fn col_to_letter(mut col: u32) -> String
```

#### `fn format_a1_range`
- Ort: `crates/contextra-ingest/src/chunking/tabular/helpers.rs:37`
```rust
pub fn format_a1_range( origin: (u32, u32), header_rows: u8, active_cols: &[usize], first_data_row_1based: usize, last_data_row_1based: usize, ) -> String
```

### Modul `chunking::types`

#### `struct ContextChunk`
- Ort: `crates/contextra-ingest/src/chunking/types.rs:56`
```rust
pub struct ContextChunk
```
Dokumentation: Finalisierter Text-Chunk eines Dokuments für Indexierung und Retrieval.
- Impl-Block (Trait `From<LocalContextChunk>`) `crates/contextra-ingest/src/compat/types.rs:26`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-ingest/src/chunking/types.rs:58` |
| `content` | `String` | `crates/contextra-ingest/src/chunking/types.rs:60` |
| `relevance` | `f32` | `crates/contextra-ingest/src/chunking/types.rs:62` |
| `token_count` | `usize` | `crates/contextra-ingest/src/chunking/types.rs:64` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-ingest/src/chunking/types.rs:66` |
| `contextual_prefix` | `Option<String>` | `crates/contextra-ingest/src/chunking/types.rs:68` |
| `links` | `Vec<ContextChunkLink>` | `crates/contextra-ingest/src/chunking/types.rs:70` |

#### `struct ContextChunkLink`
- Ort: `crates/contextra-ingest/src/chunking/types.rs:47`
```rust
pub struct ContextChunkLink
```
Dokumentation: Verweis auf einen anderen Chunk oder ein referenziertes Dokument.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kind` | `String` | `crates/contextra-ingest/src/chunking/types.rs:49` |
| `target` | `String` | `crates/contextra-ingest/src/chunking/types.rs:51` |

#### `struct DocId`
- Ort: `crates/contextra-ingest/src/chunking/types.rs:17`
```rust
pub struct DocId(pub u64)
```
Dokumentation: Eindeutige Kennung eines Dokuments.
- Impl-Block `crates/contextra-ingest/src/chunking/types.rs:19`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-ingest/src/chunking/types.rs:33`
- Impl-Block (Trait `From<u64>`) `crates/contextra-ingest/src/chunking/types.rs:39`
- Impl-Block (Trait `From<LocalDocId>`) `crates/contextra-ingest/src/compat/types.rs:9`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-ingest/src/chunking/types.rs:22` | Erstellt eine neue `DocId`. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-ingest/src/chunking/types.rs:28` | Gibt den numerischen Wert zurück. |

### Modul `compat`

#### `mod types`
- Ort: `crates/contextra-ingest/src/compat/mod.rs:5`
```rust
pub mod types
```

### Modul `crate`

#### `mod canonical`
- Ort: `crates/contextra-ingest/src/lib.rs:21`
```rust
pub mod canonical
```

#### `mod chunking`
- Ort: `crates/contextra-ingest/src/lib.rs:22`
```rust
pub mod chunking
```

#### `mod compat`
- Ort: `crates/contextra-ingest/src/lib.rs:19`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod detect`
- Ort: `crates/contextra-ingest/src/lib.rs:23`
```rust
pub mod detect
```

#### `mod error`
- Ort: `crates/contextra-ingest/src/lib.rs:24`
```rust
pub mod error
```

#### `mod extractor`
- Ort: `crates/contextra-ingest/src/lib.rs:25`
```rust
pub mod extractor
```

#### `mod format`
- Ort: `crates/contextra-ingest/src/lib.rs:26`
```rust
pub mod format
```

#### `mod formats`
- Ort: `crates/contextra-ingest/src/lib.rs:27`
```rust
pub mod formats
```

#### `mod fuzz_support`
- Ort: `crates/contextra-ingest/src/lib.rs:39`
```rust
pub mod fuzz_support
```
- Gate: `#[cfg(feature = "fuzz")]`

#### `mod ir`
- Ort: `crates/contextra-ingest/src/lib.rs:28`
```rust
pub mod ir
```

#### `mod limits`
- Ort: `crates/contextra-ingest/src/lib.rs:29`
```rust
pub mod limits
```

#### `mod pipeline`
- Ort: `crates/contextra-ingest/src/lib.rs:30`
```rust
pub mod pipeline
```

#### `mod registry`
- Ort: `crates/contextra-ingest/src/lib.rs:31`
```rust
pub mod registry
```

#### `mod safe`
- Ort: `crates/contextra-ingest/src/lib.rs:32`
```rust
pub mod safe
```

### Modul `detect`

#### `struct DetectHints`
- Ort: `crates/contextra-ingest/src/detect/mod.rs:34`
```rust
pub struct DetectHints<'a>
```
Dokumentation: Hinweise zur Formaterkennung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `filename` | `Option<&'a str>` | `crates/contextra-ingest/src/detect/mod.rs:35` |
| `mime` | `Option<&'a str>` | `crates/contextra-ingest/src/detect/mod.rs:36` |

#### `struct Detection`
- Ort: `crates/contextra-ingest/src/detect/mod.rs:41`
```rust
pub struct Detection
```
Dokumentation: Ergebnis der Formaterkennung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `format` | `FormatId` | `crates/contextra-ingest/src/detect/mod.rs:42` |
| `confidence` | `Confidence` | `crates/contextra-ingest/src/detect/mod.rs:43` |
| `reason` | `&'static str` | `crates/contextra-ingest/src/detect/mod.rs:44` |

#### `enum Confidence`
- Ort: `crates/contextra-ingest/src/detect/mod.rs:26`
```rust
pub enum Confidence
```
Dokumentation: Konfidenz der Formaterkennung.

Varianten:

- `Low` — `crates/contextra-ingest/src/detect/mod.rs:27`
- `Medium` — `crates/contextra-ingest/src/detect/mod.rs:28`
- `High` — `crates/contextra-ingest/src/detect/mod.rs:29`

#### `fn detect`
- Ort: `crates/contextra-ingest/src/detect/mod.rs:76`
```rust
pub fn detect(bytes: &[u8], hints: &DetectHints<'_>) -> Result<Detection, IngestError>
```
Dokumentation: Formaterkennung anhand von Inhalts-Bytes (max 64 KiB) und Hinweisen. **Invariante `INV-INGEST-BOUNDED-1`**: Analysiert strikt niemals mehr als `MAX_DETECT_BYTES` (64 KiB).

#### `fn detect_reader`
- Ort: `crates/contextra-ingest/src/detect/mod.rs:57`
```rust
pub fn detect_reader<R: std::io::Read>( reader: R, hints: &DetectHints<'_>, ) -> Result<Detection, IngestError>
```
Dokumentation: Formaterkennung anhand eines Streams (z. B. Reader ohne bekannte Gesamtlänge). **Invariante `INV-INGEST-BOUNDED-1`**: Liest strikt niemals mehr als `MAX_DETECT_BYTES` (64 KiB).

#### `const MAX_DETECT_BYTES`
- Ort: `crates/contextra-ingest/src/detect/mod.rs:21`
```rust
pub const MAX_DETECT_BYTES: usize = 64 * 1024
```
Dokumentation: Maximale Anzahl an Bytes, die zur Formaterkennung analysiert werden (64 KiB).

### Modul `error`

#### `enum IngestError`
- Ort: `crates/contextra-ingest/src/error.rs:53`
```rust
pub enum IngestError
```
Dokumentation: Einheitlicher Fehler-Enum für alle Ingestion- und Chunking-Vorgänge.

Varianten:

- `UnsupportedFormat{` — `crates/contextra-ingest/src/error.rs:55`
- `Encrypted{ format: Option<FormatId> }` — `crates/contextra-ingest/src/error.rs:61`
- `Corrupt{` — `crates/contextra-ingest/src/error.rs:64`
- `LimitExceeded{` — `crates/contextra-ingest/src/error.rs:70`
- `NoExtractableText{ format: FormatId }` — `crates/contextra-ingest/src/error.rs:77`
- `ParserPanic{ format: FormatId }` — `crates/contextra-ingest/src/error.rs:80`
- `Chunking{ detail: String }` — `crates/contextra-ingest/src/error.rs:83`

#### `enum LimitKind`
- Ort: `crates/contextra-ingest/src/error.rs:23`
```rust
pub enum LimitKind
```
Dokumentation: Kategorien von Ressourceneinschränkungen.

Varianten:

- `DocumentBytes` — `crates/contextra-ingest/src/error.rs:25`
- `DecompressedBytes` — `crates/contextra-ingest/src/error.rs:27`
- `CompressionRatio` — `crates/contextra-ingest/src/error.rs:29`
- `ArchiveEntries` — `crates/contextra-ingest/src/error.rs:31`
- `NestingDepth` — `crates/contextra-ingest/src/error.rs:33`
- `XmlDepth` — `crates/contextra-ingest/src/error.rs:35`
- `XmlEvents` — `crates/contextra-ingest/src/error.rs:37`
- `TextBytes` — `crates/contextra-ingest/src/error.rs:39`
- `Pages` — `crates/contextra-ingest/src/error.rs:41`
- `TableCells` — `crates/contextra-ingest/src/error.rs:43`
- `Chunks` — `crates/contextra-ingest/src/error.rs:45`
- `PathLength` — `crates/contextra-ingest/src/error.rs:47`

### Modul `extractor`

#### `struct ExtractContext`
- Ort: `crates/contextra-ingest/src/extractor.rs:167`
```rust
pub struct ExtractContext<'a>
```
Dokumentation: Extraktions-Kontext, der während des gesamten Parsing-Vorgangs mitgeführt wird.
- Impl-Block `crates/contextra-ingest/src/extractor.rs:175`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `limits` | `&'a IngestLimits` | `crates/contextra-ingest/src/extractor.rs:168` |
| `policy` | `&'a ExtractPolicy` | `crates/contextra-ingest/src/extractor.rs:169` |
| `budget` | `&'a ResourceBudget` | `crates/contextra-ingest/src/extractor.rs:170` |
| `registry` | `&'a crate::registry::ExtractorRegistry` | `crates/contextra-ingest/src/extractor.rs:171` |
| `filename` | `Option<&'a str>` | `crates/contextra-ingest/src/extractor.rs:172` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn extract_nested( &self, name: &str, mime: Option<&str>, bytes: &[u8], ) -> Result<DocumentIr, IngestError>` | `crates/contextra-ingest/src/extractor.rs:177` | Hilfsmethode zum rekursiven Extrahesieren eines verschachtelten Dokuments (Anhangs). |

#### `struct NestGuard`
- Ort: `crates/contextra-ingest/src/extractor.rs:156`
```rust
pub struct NestGuard<'a>
```
Dokumentation: RAII-Guard für die Verschachtelungstiefe.
- Impl-Block (Trait `Drop`) `crates/contextra-ingest/src/extractor.rs:160`

#### `struct ResourceBudget`
- Ort: `crates/contextra-ingest/src/extractor.rs:39`
```rust
pub struct ResourceBudget
```
Dokumentation: Atomares Ressourcen-Budget zur Absicherung gegen DoS/Zip-Bomben (arbeitsbasiert, ohne Zeit).
- Impl-Block `crates/contextra-ingest/src/extractor.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `limits` | `IngestLimits` | `crates/contextra-ingest/src/extractor.rs:40` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(limits: IngestLimits) -> Self` | `crates/contextra-ingest/src/extractor.rs:51` | Erstellt ein neues Budget basierend auf den konfigurierten `IngestLimits`. |
| `pub fn charge_decompressed(&self, n: u64) -> Result<(), IngestError>` | `crates/contextra-ingest/src/extractor.rs:64` | Verbucht entkomprimierte Bytes. |
| `pub fn charge_xml_events(&self, n: u64) -> Result<(), IngestError>` | `crates/contextra-ingest/src/extractor.rs:79` | Verbucht XML-Events. |
| `pub fn charge_text(&self, n: u64) -> Result<(), IngestError>` | `crates/contextra-ingest/src/extractor.rs:94` | Verbucht Text-Bytes. |
| `pub fn charge_cells(&self, n: u64) -> Result<(), IngestError>` | `crates/contextra-ingest/src/extractor.rs:109` | Verbucht Tabellenzellen. |
| `pub fn charge_pages(&self, n: u64) -> Result<(), IngestError>` | `crates/contextra-ingest/src/extractor.rs:124` | Verbucht Seiten. |
| `pub fn enter_nested(&self) -> Result<NestGuard<'_>, IngestError>` | `crates/contextra-ingest/src/extractor.rs:139` | Tritt in eine tiefere Dokument-Verschachtelung ein (RAII NestGuard). |

#### `trait DocumentExtractor`
- Ort: `crates/contextra-ingest/src/extractor.rs:24`
```rust
pub trait DocumentExtractor: Send + Sync
```
Dokumentation: Abstrakter Extractor-Trait für ein oder mehrere Dokumentformate.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn formats(&self) -> &'static [FormatId]` | `crates/contextra-ingest/src/extractor.rs:26` | Liste der von diesem Extractor unterstützten Formate. |
| `fn extract( &self, format: FormatId, input: &[u8], ctx: &ExtractContext<'_>, ) -> Result<DocumentIr, IngestError>` | `crates/contextra-ingest/src/extractor.rs:29` | Extrahiert ein Dokument in die `DocumentIr`. |

### Modul `format`

#### `enum ChunkStrategy`
- Ort: `crates/contextra-ingest/src/format.rs:58`
```rust
pub enum ChunkStrategy
```
Dokumentation: Strategie für die strukturierte Zerlegung (Chunking) eines Dokuments.

Varianten:

- `Prose` — `crates/contextra-ingest/src/format.rs:59`
- `Tabular` — `crates/contextra-ingest/src/format.rs:60`
- `Paged` — `crates/contextra-ingest/src/format.rs:61`
- `Structured` — `crates/contextra-ingest/src/format.rs:62`
- `Email` — `crates/contextra-ingest/src/format.rs:63`

#### `enum FormatId`
- Ort: `crates/contextra-ingest/src/format.rs:22`
```rust
pub enum FormatId
```
Dokumentation: Aufzählung aller unterstützen Dokumentformate in contextra-ingest.
- Impl-Block `crates/contextra-ingest/src/format.rs:66`

Varianten:

- `Txt` — `crates/contextra-ingest/src/format.rs:23`
- `Markdown` — `crates/contextra-ingest/src/format.rs:24`
- `Rst` — `crates/contextra-ingest/src/format.rs:25`
- `AsciiDoc` — `crates/contextra-ingest/src/format.rs:26`
- `Tex` — `crates/contextra-ingest/src/format.rs:27`
- `Rtf` — `crates/contextra-ingest/src/format.rs:28`
- `Html` — `crates/contextra-ingest/src/format.rs:29`
- `Xhtml` — `crates/contextra-ingest/src/format.rs:30`
- `Epub` — `crates/contextra-ingest/src/format.rs:31`
- `Docx` — `crates/contextra-ingest/src/format.rs:32`
- `Pptx` — `crates/contextra-ingest/src/format.rs:33`
- `Xlsx` — `crates/contextra-ingest/src/format.rs:34`
- `Xls` — `crates/contextra-ingest/src/format.rs:35`
- `Xlsb` — `crates/contextra-ingest/src/format.rs:36`
- `Ods` — `crates/contextra-ingest/src/format.rs:37`
- `Csv` — `crates/contextra-ingest/src/format.rs:38`
- `Tsv` — `crates/contextra-ingest/src/format.rs:39`
- `Odt` — `crates/contextra-ingest/src/format.rs:40`
- `Odp` — `crates/contextra-ingest/src/format.rs:41`
- `Pdf` — `crates/contextra-ingest/src/format.rs:42`
- `Doc` — `crates/contextra-ingest/src/format.rs:43`
- `Ppt` — `crates/contextra-ingest/src/format.rs:44`
- `Msg` — `crates/contextra-ingest/src/format.rs:45`
- `Eml` — `crates/contextra-ingest/src/format.rs:46`
- `Mbox` — `crates/contextra-ingest/src/format.rs:47`
- `Json` — `crates/contextra-ingest/src/format.rs:48`
- `Jsonl` — `crates/contextra-ingest/src/format.rs:49`
- `Xml` — `crates/contextra-ingest/src/format.rs:50`
- `Yaml` — `crates/contextra-ingest/src/format.rs:51`
- `Toml` — `crates/contextra-ingest/src/format.rs:52`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn name(self) -> &'static str` | `crates/contextra-ingest/src/format.rs:68` | Stabile, kleingeschriebene Textidentifikation des Formats. |
| `pub fn extensions(self) -> &'static [&'static str]` | `crates/contextra-ingest/src/format.rs:104` | Liste bekannter Dateiendungen (ohne führenden Punkt). |
| `pub fn mime(self) -> &'static str` | `crates/contextra-ingest/src/format.rs:140` | Standard-MIME-Type des Formats. |
| `pub fn from_extension(ext: &str) -> Option<Self>` | `crates/contextra-ingest/src/format.rs:180` | Erstellt `FormatId` anhand einer Dateiendung (ohne Punkt, Case-Insensitive). |
| `pub fn default_strategy(self) -> ChunkStrategy` | `crates/contextra-ingest/src/format.rs:218` | Standard-Chunking-Strategie für ein Format. |

### Modul `formats`

#### `mod docx`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:24`
```rust
pub mod docx
```
- Gate: `#[cfg(feature = "fmt-docx")]`

#### `mod email`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:42`
```rust
pub mod email
```
- Gate: `#[cfg(feature = "fmt-email")]`

#### `mod odf`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:33`
```rust
pub mod odf
```
- Gate: `#[cfg(feature = "fmt-odf")]`

#### `mod ole`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:39`
```rust
pub mod ole
```
- Gate: `#[cfg(feature = "fmt-ole")]`

#### `mod pdf`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:36`
```rust
pub mod pdf
```
- Gate: `#[cfg(feature = "fmt-pdf")]`

#### `mod pptx`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:27`
```rust
pub mod pptx
```
- Gate: `#[cfg(feature = "fmt-pptx")]`

#### `mod sheets`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:30`
```rust
pub mod sheets
```
- Gate: `#[cfg(feature = "fmt-sheets")]`

#### `mod structured`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:45`
```rust
pub mod structured
```
- Gate: `#[cfg(feature = "fmt-structured")]`

#### `mod text`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:18`
```rust
pub mod text
```
- Gate: `#[cfg(feature = "fmt-text")]`

#### `mod web`
- Ort: `crates/contextra-ingest/src/formats/mod.rs:21`
```rust
pub mod web
```
- Gate: `#[cfg(feature = "fmt-web")]`

### Modul `formats::docx`

#### `mod document`
- Ort: `crates/contextra-ingest/src/formats/docx/mod.rs:17`
```rust
pub mod document
```

#### `mod notes`
- Ort: `crates/contextra-ingest/src/formats/docx/mod.rs:18`
```rust
pub mod notes
```

#### `mod numbering`
- Ort: `crates/contextra-ingest/src/formats/docx/mod.rs:19`
```rust
pub mod numbering
```

#### `mod styles`
- Ort: `crates/contextra-ingest/src/formats/docx/mod.rs:20`
```rust
pub mod styles
```

#### `struct DocxExtractor`
- Ort: `crates/contextra-ingest/src/formats/docx/mod.rs:31`
```rust
pub struct DocxExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/docx/mod.rs:33`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/docx/mod.rs:41`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/docx/mod.rs:34` |  |

### Modul `formats::docx::document`

#### `struct DocumentParseResult`
- Ort: `crates/contextra-ingest/src/formats/docx/document.rs:27`
```rust
pub struct DocumentParseResult
```
Dokumentation: Resultat des Parsevorgangs von document.xml.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `hidden_content_skipped` | `bool` | `crates/contextra-ingest/src/formats/docx/document.rs:28` |
| `embedded_object_count` | `u32` | `crates/contextra-ingest/src/formats/docx/document.rs:29` |

#### `fn parse_document_xml`
- Ort: `crates/contextra-ingest/src/formats/docx/document.rs:52`
```rust
pub fn parse_document_xml( bytes: &[u8], styles: &StyleMap, numbering: &NumMap, ir: &mut DocumentIr, ctx: &ExtractContext<'_>, ) -> Result<DocumentParseResult, IngestError>
```
Dokumentation: Parst die Hauptdokument-XML (word/document.xml).

### Modul `formats::docx::notes`

#### `struct ExtractedNote`
- Ort: `crates/contextra-ingest/src/formats/docx/notes.rs:25`
```rust
pub struct ExtractedNote
```
Dokumentation: Extrahierte Notiz aus Fußnoten, Endnoten, Kommentaren, Kopf-/Fußzeilen.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kind` | `NoteKind` | `crates/contextra-ingest/src/formats/docx/notes.rs:26` |
| `text` | `String` | `crates/contextra-ingest/src/formats/docx/notes.rs:27` |

#### `fn parse_comments`
- Ort: `crates/contextra-ingest/src/formats/docx/notes.rs:112`
```rust
pub fn parse_comments( bytes: &[u8], ctx: &ExtractContext<'_>, ) -> Result<Vec<ExtractedNote>, IngestError>
```
Dokumentation: Parst word/comments.xml wenn `policy.include_comments` true ist.

#### `fn parse_footnotes_or_endnotes`
- Ort: `crates/contextra-ingest/src/formats/docx/notes.rs:31`
```rust
pub fn parse_footnotes_or_endnotes( bytes: &[u8], is_endnote: bool, max_xml_depth: u32, budget: &ResourceBudget, ) -> Result<Vec<ExtractedNote>, IngestError>
```
Dokumentation: Parst eine Footnotes- oder Endnotes-XML-Datei.

#### `fn parse_header_footer`
- Ort: `crates/contextra-ingest/src/formats/docx/notes.rs:166`
```rust
pub fn parse_header_footer( bytes: &[u8], is_footer: bool, ctx: &ExtractContext<'_>, ) -> Result<Option<ExtractedNote>, IngestError>
```
Dokumentation: Parst eine Header/Footer-XML-Datei wenn `policy.include_headers_footers` true ist.

### Modul `formats::docx::numbering`

#### `struct NumMap`
- Ort: `crates/contextra-ingest/src/formats/docx/numbering.rs:26`
```rust
pub struct NumMap
```
Dokumentation: Zuordnungstabelle für Listenformatierungen.
- Impl-Block `crates/contextra-ingest/src/formats/docx/numbering.rs:33`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse( bytes: &[u8], max_xml_depth: u32, budget: &ResourceBudget, ) -> Result<Self, IngestError>` | `crates/contextra-ingest/src/formats/docx/numbering.rs:35` | Parsen der numbering.xml-Datei. |
| `pub fn is_ordered(&self, num_id: &str, ilvl: u8) -> bool` | `crates/contextra-ingest/src/formats/docx/numbering.rs:137` | Bestimmt, ob eine Liste an `numId` und `ilvl` ordered (nummeriert) ist. Liefert `false` bei "bullet" oder "none", sonst `true`. |

### Modul `formats::docx::styles`

#### `struct StyleInfo`
- Ort: `crates/contextra-ingest/src/formats/docx/styles.rs:26`
```rust
pub struct StyleInfo
```
Dokumentation: Metadaten eines Absatzstils in styles.xml.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `style_id` | `String` | `crates/contextra-ingest/src/formats/docx/styles.rs:27` |
| `name` | `String` | `crates/contextra-ingest/src/formats/docx/styles.rs:28` |
| `based_on` | `Option<String>` | `crates/contextra-ingest/src/formats/docx/styles.rs:29` |
| `outline_lvl` | `Option<u8>` | `crates/contextra-ingest/src/formats/docx/styles.rs:30` |

#### `struct StyleMap`
- Ort: `crates/contextra-ingest/src/formats/docx/styles.rs:36`
```rust
pub struct StyleMap
```
Dokumentation: Sammlung aller Stile in styles.xml mit Vererbungsauswertung.
- Impl-Block `crates/contextra-ingest/src/formats/docx/styles.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse( bytes: &[u8], max_xml_depth: u32, budget: &ResourceBudget, ) -> Result<Self, IngestError>` | `crates/contextra-ingest/src/formats/docx/styles.rs:42` | Parsen der styles.xml-Datei. |
| `pub fn heading_level(&self, style_id: &str) -> Option<u8>` | `crates/contextra-ingest/src/formats/docx/styles.rs:151` | Ermittelt die Überschriftenebene (1..=9) für eine Stil-ID oder liefert `None`. |

#### `fn match_heading_string`
- Ort: `crates/contextra-ingest/src/formats/docx/styles.rs:192`
```rust
pub fn match_heading_string(s: &str) -> Option<u8>
```
Dokumentation: Prüft, ob ein String ein Heading-Muster wie "heading 1", "Überschrift 1", "Title" etc. ist.

### Modul `formats::email`

#### `mod eml`
- Ort: `crates/contextra-ingest/src/formats/email/mod.rs:17`
```rust
pub mod eml
```

#### `mod html`
- Ort: `crates/contextra-ingest/src/formats/email/mod.rs:18`
```rust
pub mod html
```

#### `mod mbox`
- Ort: `crates/contextra-ingest/src/formats/email/mod.rs:19`
```rust
pub mod mbox
```

#### `struct EmailExtractor`
- Ort: `crates/contextra-ingest/src/formats/email/mod.rs:27`
```rust
pub struct EmailExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/email/mod.rs:29`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/email/mod.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/email/mod.rs:30` |  |

### Modul `formats::email::eml`

#### `fn extract_eml`
- Ort: `crates/contextra-ingest/src/formats/email/eml.rs:30`
```rust
pub fn extract_eml( input: &[u8], ctx: &ExtractContext<'_>, message_number: Option<u32>, ) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert eine einzelne EML-Nachricht.

### Modul `formats::email::html`

#### `fn strip_html`
- Ort: `crates/contextra-ingest/src/formats/email/html.rs:23`
```rust
pub fn strip_html(html: &str) -> String
```
Dokumentation: Minimaler HTML-Entferner für E-Mail-Bodies, wenn nur HTML verfügbar ist. DECISION & LIMITATION: Skript- und Style-Blöcke (<script...>, <style...>) werden inklusive Inhalt vollständig verworfen, um Prompt-Injection und unerwünschte CSS-Artefakte zu verhindern. `html5ever` wird hier bewusst nicht verwendet, um das E-Mail-Modul unabhängig vom `fmt-web`-Feature zu halten.

### Modul `formats::email::mbox`

#### `fn extract_mbox`
- Ort: `crates/contextra-ingest/src/formats/email/mbox.rs:24`
```rust
pub fn extract_mbox(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert eine MBOX-Datei mit mehreren Nachrichten.

### Modul `formats::odf`

#### `mod content`
- Ort: `crates/contextra-ingest/src/formats/odf/mod.rs:17`
```rust
pub mod content
```

#### `mod meta`
- Ort: `crates/contextra-ingest/src/formats/odf/mod.rs:18`
```rust
pub mod meta
```

#### `mod presentation`
- Ort: `crates/contextra-ingest/src/formats/odf/mod.rs:19`
```rust
pub mod presentation
```

#### `struct OdfExtractor`
- Ort: `crates/contextra-ingest/src/formats/odf/mod.rs:28`
```rust
pub struct OdfExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/odf/mod.rs:30`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/odf/mod.rs:38`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/odf/mod.rs:31` |  |

### Modul `formats::odf::content`

#### `fn parse_content`
- Ort: `crates/contextra-ingest/src/formats/odf/content.rs:27`
```rust
pub fn parse_content( format: crate::format::FormatId, xml_bytes: &[u8], ctx: &ExtractContext<'_>, ir: &mut DocumentIr, ) -> Result<(), IngestError>
```
Dokumentation: Parsing-Kontext für ODT Content-Streams.

### Modul `formats::odf::meta`

#### `fn parse_meta`
- Ort: `crates/contextra-ingest/src/formats/odf/meta.rs:24`
```rust
pub fn parse_meta(xml_bytes: &[u8], ctx: &ExtractContext<'_>) -> Result<DocMeta, IngestError>
```
Dokumentation: Extrahiert Metadaten aus `meta.xml` oder einem `<office:meta>`-Element.

### Modul `formats::odf::presentation`

#### `fn parse_presentation`
- Ort: `crates/contextra-ingest/src/formats/odf/presentation.rs:24`
```rust
pub fn parse_presentation( xml_bytes: &[u8], ctx: &ExtractContext<'_>, ir: &mut DocumentIr, ) -> Result<(), IngestError>
```
Dokumentation: Extrahiert Folien, Titel, Inhalte und Sprechernotizen aus ODP-Content XML.

### Modul `formats::ole`

#### `mod cfb_reader`
- Ort: `crates/contextra-ingest/src/formats/ole/mod.rs:17`
```rust
pub mod cfb_reader
```

#### `mod doc`
- Ort: `crates/contextra-ingest/src/formats/ole/mod.rs:18`
```rust
pub mod doc
```

#### `mod msg`
- Ort: `crates/contextra-ingest/src/formats/ole/mod.rs:19`
```rust
pub mod msg
```

#### `mod ppt`
- Ort: `crates/contextra-ingest/src/formats/ole/mod.rs:20`
```rust
pub mod ppt
```

#### `struct OleExtractor`
- Ort: `crates/contextra-ingest/src/formats/ole/mod.rs:29`
```rust
pub struct OleExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/ole/mod.rs:31`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/ole/mod.rs:39`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/ole/mod.rs:32` |  |

### Modul `formats::ole::cfb_reader`

#### `struct SafeCfbReader`
- Ort: `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:27`
```rust
pub struct SafeCfbReader<'a>
```
Dokumentation: Sumpf-sicherer Wrapper für Compound File Binary (CFB) Subsystem.
- Impl-Block `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:32`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(input: &'a [u8], format: FormatId) -> Result<Self, IngestError>` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:34` | Öffnet eine CFB-Datei aus Byte-Slice. Fängt E/A-Fehler ab und wandelt sie in `IngestError::Corrupt` um. |
| `pub fn exists(&self, path: &str) -> bool` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:45` | Prüft, ob ein Pfad (Stream oder Storage) in der CFB existiert. |
| `pub fn is_stream(&self, path: &str) -> bool` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:50` | Prüft, ob ein Pfad ein Stream ist. |
| `pub fn is_storage(&self, path: &str) -> bool` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:55` | Prüft, ob ein Pfad ein Storage ist. |
| `pub fn read_stream( &mut self, path: &str, ctx: &ExtractContext<'_>, ) -> Result<Vec<u8>, IngestError>` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:60` | Liest einen Stream vollständig unter Berücksichtigung des Decompress-Budgets. |
| `pub fn list_children(&self, storage_path: &str) -> Result<Vec<Entry>, IngestError>` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:105` | Listet alle direkten Kinder eines Storages auf (mit Schleifenbegrenzung). |
| `pub fn walk_all(&self) -> Result<Vec<Entry>, IngestError>` | `crates/contextra-ingest/src/formats/ole/cfb_reader.rs:133` | Rekursiver Walk über alle Einträge in der CFB mit Schleifenbegrenzer gegen Zyklen. |

### Modul `formats::ole::doc`

#### `fn extract_doc`
- Ort: `crates/contextra-ingest/src/formats/ole/doc.rs:69`
```rust
pub fn extract_doc( reader: &mut SafeCfbReader<'_>, ctx: &ExtractContext<'_>, ) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert MS-DOC aus einer OLE/CFB Struktur.

### Modul `formats::ole::msg`

#### `fn extract_msg`
- Ort: `crates/contextra-ingest/src/formats/ole/msg.rs:47`
```rust
pub fn extract_msg( reader: &mut SafeCfbReader<'_>, ctx: &ExtractContext<'_>, ) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert MS-OXMSG aus einer OLE/CFB Struktur.

### Modul `formats::ole::ppt`

#### `fn extract_ppt`
- Ort: `crates/contextra-ingest/src/formats/ole/ppt.rs:90`
```rust
pub fn extract_ppt( reader: &mut SafeCfbReader<'_>, ctx: &ExtractContext<'_>, ) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert MS-PPT aus einer OLE/CFB Struktur.

### Modul `formats::pdf`

#### `mod text_processing`
- Ort: `crates/contextra-ingest/src/formats/pdf/mod.rs:19`
```rust
pub mod text_processing
```

#### `struct PdfExtractor`
- Ort: `crates/contextra-ingest/src/formats/pdf/mod.rs:39`
```rust
pub struct PdfExtractor
```
Dokumentation: PDF Extractor basierend auf `lopdf`.
- Impl-Block `crates/contextra-ingest/src/formats/pdf/mod.rs:41`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/pdf/mod.rs:50`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/pdf/mod.rs:43` | Erzeugt eine neue `PdfExtractor`-Instanz. |

### Modul `formats::pdf::text_processing`

#### `struct HeaderFooterAnalysis`
- Ort: `crates/contextra-ingest/src/formats/pdf/text_processing.rs:152`
```rust
pub struct HeaderFooterAnalysis
```
Dokumentation: Ergebnis der Header/Footer-Analyse pro Seite.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `detected_headers` | `Vec<String>` | `crates/contextra-ingest/src/formats/pdf/text_processing.rs:153` |
| `detected_footers` | `Vec<String>` | `crates/contextra-ingest/src/formats/pdf/text_processing.rs:154` |

#### `fn analyze_headers_footers`
- Ort: `crates/contextra-ingest/src/formats/pdf/text_processing.rs:159`
```rust
pub fn analyze_headers_footers(page_lines: &[Vec<String>]) -> HeaderFooterAnalysis
```
Dokumentation: Analysiert Seitenzeilen auf wiederkehrende Kopf-/Fußzeilen. Regel: Gleiche Zeile mit Ziffern-Normalisierung auf >= 3 Seiten und >= 50% aller Seiten.

#### `fn clean_page_text`
- Ort: `crates/contextra-ingest/src/formats/pdf/text_processing.rs:20`
```rust
pub fn clean_page_text(text: &str) -> String
```
Dokumentation: Dehnt Unicode-Ligaturen (U+FB00..=U+FB06) aus und entfernt Soft-Hyphens (U+00AD).

#### `fn join_end_of_line_hyphens`
- Ort: `crates/contextra-ingest/src/formats/pdf/text_processing.rs:41`
```rust
pub fn join_end_of_line_hyphens(text: &str) -> String
```
Dokumentation: Führt Silbentrennung am Zeilenende zusammen: NUR wenn Buchstabe + "-" + Zeilenumbruch ('\n' oder '\r\n') + Kleinbuchstabe folgt. Behält Bindestriche wie in "E-Mail" oder "Rhein-Main" bei.

#### `fn normalize_line_digits`
- Ort: `crates/contextra-ingest/src/formats/pdf/text_processing.rs:138`
```rust
pub fn normalize_line_digits(line: &str) -> String
```
Dokumentation: Normalisiert Ziffern in einer Zeile zu '#' für die Wiedererkennung wiederkehrender Kopf-/Fußzeilen.

#### `fn reconstruct_paragraphs`
- Ort: `crates/contextra-ingest/src/formats/pdf/text_processing.rs:99`
```rust
pub fn reconstruct_paragraphs(text: &str) -> Vec<String>
```
Dokumentation: Baut Fließtextabsätze auf: Leerzeilen und Einrückung dienen als Absatzgrenzen. Harte Zeilenumbrüche innerhalb eines Absatzes werden zu Leerzeichen.

### Modul `formats::pptx`

#### `mod presentation`
- Ort: `crates/contextra-ingest/src/formats/pptx/mod.rs:17`
```rust
pub mod presentation
```

#### `mod slide`
- Ort: `crates/contextra-ingest/src/formats/pptx/mod.rs:18`
```rust
pub mod slide
```

#### `struct PptxExtractor`
- Ort: `crates/contextra-ingest/src/formats/pptx/mod.rs:30`
```rust
pub struct PptxExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/pptx/mod.rs:32`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/pptx/mod.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/pptx/mod.rs:33` |  |

### Modul `formats::pptx::presentation`

#### `struct SlideRef`
- Ort: `crates/contextra-ingest/src/formats/pptx/presentation.rs:24`
```rust
pub struct SlideRef
```
Dokumentation: Metadaten einer Folie aus presentation.xml.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `r_id` | `String` | `crates/contextra-ingest/src/formats/pptx/presentation.rs:25` |
| `slide_path` | `String` | `crates/contextra-ingest/src/formats/pptx/presentation.rs:26` |
| `is_hidden` | `bool` | `crates/contextra-ingest/src/formats/pptx/presentation.rs:27` |

#### `fn parse_presentation_slides`
- Ort: `crates/contextra-ingest/src/formats/pptx/presentation.rs:31`
```rust
pub fn parse_presentation_slides( pres_xml_bytes: &[u8], pres_rels: &OpcRels, budget: &ResourceBudget, ) -> Result<Vec<SlideRef>, IngestError>
```
Dokumentation: Parst ppt/presentation.xml und liest die exakte Folienreihenfolge aus p:sldIdLst.

### Modul `formats::pptx::slide`

#### `struct SlideExtractionResult`
- Ort: `crates/contextra-ingest/src/formats/pptx/slide.rs:28`
```rust
pub struct SlideExtractionResult
```
Dokumentation: Ergebnis der Verarbeitung einer einzelnen Folie.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `warnings` | `Vec<ExtractWarning>` | `crates/contextra-ingest/src/formats/pptx/slide.rs:30` |

#### `fn parse_slide_content`
- Ort: `crates/contextra-ingest/src/formats/pptx/slide.rs:34`
```rust
pub fn parse_slide_content( zip: &mut SafeZipReader<'_>, slide_path: &str, slide_num: u32, is_hidden_in_pres: bool, policy: &ExtractPolicy, budget: &ResourceBudget, ) -> Result<SlideExtractionResult, IngestError>
```
Dokumentation: Parst eine Folie, verarbeitet deren Formen, Tabellen, Notizen und Kommentare.

### Modul `formats::sheets`

#### `mod csv_fmt`
- Ort: `crates/contextra-ingest/src/formats/sheets/mod.rs:17`
```rust
pub mod csv_fmt
```

#### `mod excel`
- Ort: `crates/contextra-ingest/src/formats/sheets/mod.rs:18`
```rust
pub mod excel
```

#### `struct SheetsExtractor`
- Ort: `crates/contextra-ingest/src/formats/sheets/mod.rs:26`
```rust
pub struct SheetsExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/sheets/mod.rs:28`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/sheets/mod.rs:43`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/sheets/mod.rs:29` |  |

### Modul `formats::sheets::csv_fmt`

#### `fn build_tables_from_rows`
- Ort: `crates/contextra-ingest/src/formats/sheets/csv_fmt.rs:186`
```rust
pub fn build_tables_from_rows( ir: &mut DocumentIr, rows: Vec<Vec<String>>, sheet_name: &str, ctx: &ExtractContext<'_>, ) -> Result<(), IngestError>
```
Dokumentation: Teilt Zeilen in Blöcke durch Leerzeilen, schneidet leere Ränder ab und erzeugt `Table`-Blöcke.

#### `fn determine_header_rows`
- Ort: `crates/contextra-ingest/src/formats/sheets/csv_fmt.rs:295`
```rust
pub fn determine_header_rows(rows: &[Vec<String>]) -> u8
```
Dokumentation: Bestimmt, ob die erste Zeile eines Tabellenblocks eine Kopfzeile ist.

#### `fn extract_csv_tsv`
- Ort: `crates/contextra-ingest/src/formats/sheets/csv_fmt.rs:25`
```rust
pub fn extract_csv_tsv( format: FormatId, input: &[u8], ctx: &ExtractContext<'_>, ) -> Result<DocumentIr, IngestError>
```
Dokumentation: Parst CSV oder TSV Dokumente in eine `DocumentIr`.

### Modul `formats::sheets::excel`

#### `fn extract_excel`
- Ort: `crates/contextra-ingest/src/formats/sheets/excel.rs:27`
```rust
pub fn extract_excel( format: FormatId, input: &[u8], ctx: &ExtractContext<'_>, ) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert Tabellen aus Xlsx, Xls, Xlsb und Ods Dokumenten.

### Modul `formats::structured`

#### `mod json`
- Ort: `crates/contextra-ingest/src/formats/structured/mod.rs:17`
```rust
pub mod json
```

#### `mod path`
- Ort: `crates/contextra-ingest/src/formats/structured/mod.rs:18`
```rust
pub mod path
```

#### `mod toml`
- Ort: `crates/contextra-ingest/src/formats/structured/mod.rs:19`
```rust
pub mod toml
```

#### `mod xml`
- Ort: `crates/contextra-ingest/src/formats/structured/mod.rs:20`
```rust
pub mod xml
```

#### `mod yaml`
- Ort: `crates/contextra-ingest/src/formats/structured/mod.rs:21`
```rust
pub mod yaml
```

#### `struct StructuredExtractor`
- Ort: `crates/contextra-ingest/src/formats/structured/mod.rs:30`
```rust
pub struct StructuredExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/structured/mod.rs:32`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/structured/mod.rs:46`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/structured/mod.rs:33` |  |

### Modul `formats::structured::json`

#### `enum JsonValue`
- Ort: `crates/contextra-ingest/src/formats/structured/json.rs:28`
```rust
pub enum JsonValue
```
Dokumentation: Eigenes JSON-AST-Element zur Erhaltung doppelter Schlüssel und exakter Dokumentreihenfolge.
- Impl-Block `crates/contextra-ingest/src/formats/structured/json.rs:151`

Varianten:

- `Null` — `crates/contextra-ingest/src/formats/structured/json.rs:29`
- `Bool(bool)` — `crates/contextra-ingest/src/formats/structured/json.rs:30`
- `Number(serde_json::Number)` — `crates/contextra-ingest/src/formats/structured/json.rs:31`
- `String(String)` — `crates/contextra-ingest/src/formats/structured/json.rs:32`
- `Array(Vec<JsonValue>)` — `crates/contextra-ingest/src/formats/structured/json.rs:33`
- `Object(Vec<(String, JsonValue)>)` — `crates/contextra-ingest/src/formats/structured/json.rs:34`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse(bytes: &[u8], max_depth: usize) -> Result<JsonValue, IngestError>` | `crates/contextra-ingest/src/formats/structured/json.rs:152` |  |

#### `fn extract_json`
- Ort: `crates/contextra-ingest/src/formats/structured/json.rs:178`
```rust
pub fn extract_json(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert ein JSON-Dokument in die `DocumentIr`.

#### `fn extract_jsonl`
- Ort: `crates/contextra-ingest/src/formats/structured/json.rs:203`
```rust
pub fn extract_jsonl(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert ein JSONL-Dokument zeilenweise in die `DocumentIr`.

### Modul `formats::structured::path`

#### `fn append_index`
- Ort: `crates/contextra-ingest/src/formats/structured/path.rs:77`
```rust
pub fn append_index(parent_path: &str, index: usize) -> String
```
Dokumentation: Hängt einen Array-Index `[index]` an einen bestehenden Pfad an.

#### `fn append_key`
- Ort: `crates/contextra-ingest/src/formats/structured/path.rs:61`
```rust
pub fn append_key(parent_path: &str, key: &str) -> String
```
Dokumentation: Hängt einen Objektschlüssel an einen bestehenden Pfad an.

#### `fn escape_json_string`
- Ort: `crates/contextra-ingest/src/formats/structured/path.rs:37`
```rust
pub fn escape_json_string(input: &str) -> String
```
Dokumentation: Maskiert Sonderzeichen in JSON-Strings für Pfade in `["..."]`.

#### `fn needs_quoting`
- Ort: `crates/contextra-ingest/src/formats/structured/path.rs:20`
```rust
pub fn needs_quoting(key: &str) -> bool
```
Dokumentation: Prüft, ob ein Objektschlüssel in `["..."]` gefasst werden muss. Ein Schlüssel benötigt `["..."]`, wenn er leer ist, mit einer Ziffer beginnt, oder Sonderzeichen (außer ASCII-Alphanumerik, `_`, `-`) enthält.

### Modul `formats::structured::toml`

#### `fn extract_toml`
- Ort: `crates/contextra-ingest/src/formats/structured/toml.rs:25`
```rust
pub fn extract_toml(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert ein TOML-Dokument in die `DocumentIr`.

### Modul `formats::structured::xml`

#### `fn extract_xml`
- Ort: `crates/contextra-ingest/src/formats/structured/xml.rs:40`
```rust
pub fn extract_xml(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert ein generisches XML-Dokument in die `DocumentIr`.

### Modul `formats::structured::yaml`

#### `fn extract_yaml`
- Ort: `crates/contextra-ingest/src/formats/structured/yaml.rs:26`
```rust
pub fn extract_yaml(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert ein YAML-Dokument in die `DocumentIr`.

### Modul `formats::text`

#### `mod asciidoc`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:17`
```rust
pub mod asciidoc
```

#### `mod encoding`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:18`
```rust
pub mod encoding
```

#### `mod markdown`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:19`
```rust
pub mod markdown
```

#### `mod rst`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:20`
```rust
pub mod rst
```

#### `mod rtf`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:21`
```rust
pub mod rtf
```

#### `mod tex`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:22`
```rust
pub mod tex
```

#### `mod txt`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:23`
```rust
pub mod txt
```

#### `struct TextExtractor`
- Ort: `crates/contextra-ingest/src/formats/text/mod.rs:31`
```rust
pub struct TextExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/text/mod.rs:33`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/text/mod.rs:48`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/text/mod.rs:34` |  |

### Modul `formats::text::asciidoc`

#### `fn parse_asciidoc`
- Ort: `crates/contextra-ingest/src/formats/text/asciidoc.rs:22`
```rust
pub fn parse_asciidoc(text: &str) -> DocumentIr
```
Dokumentation: Extrahiert AsciiDoc Dokumente.

### Modul `formats::text::encoding`

#### `fn decode_text`
- Ort: `crates/contextra-ingest/src/formats/text/encoding.rs:22`
```rust
pub fn decode_text(input: &[u8]) -> (String, Vec<ExtractWarning>)
```
Dokumentation: Dekodiert Rohbytes unter Berücksichtigung von BOMs, UTF-8 Validierung, UTF-16 NUL-Muster und Windows-1252 Fallback. Wendet anschließend `sanitize_text` an.

### Modul `formats::text::markdown`

#### `fn parse_markdown`
- Ort: `crates/contextra-ingest/src/formats/text/markdown.rs:21`
```rust
pub fn parse_markdown(text: &str) -> DocumentIr
```
Dokumentation: Extrahiert Markdown als einzelnen `BlockKind::Raw` Block.

### Modul `formats::text::rst`

#### `fn parse_rst`
- Ort: `crates/contextra-ingest/src/formats/text/rst.rs:22`
```rust
pub fn parse_rst(text: &str) -> DocumentIr
```
Dokumentation: Extrahiert reStructuredText (Rst) Dokumente.

### Modul `formats::text::rtf`

#### `fn parse_rtf`
- Ort: `crates/contextra-ingest/src/formats/text/rtf.rs:50`
```rust
pub fn parse_rtf(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```

### Modul `formats::text::tex`

#### `fn parse_tex`
- Ort: `crates/contextra-ingest/src/formats/text/tex.rs:30`
```rust
pub fn parse_tex(text: &str) -> DocumentIr
```
Dokumentation: Extrahiert TeX/LaTeX-Dokumente.

### Modul `formats::text::txt`

#### `fn parse_txt`
- Ort: `crates/contextra-ingest/src/formats/text/txt.rs:21`
```rust
pub fn parse_txt(text: &str) -> DocumentIr
```
Dokumentation: Extrahiert Plaintext/Log-Dateien in Absätze an Leerzeilen.

### Modul `formats::web`

#### `mod encoding`
- Ort: `crates/contextra-ingest/src/formats/web/mod.rs:17`
```rust
pub mod encoding
```

#### `mod epub`
- Ort: `crates/contextra-ingest/src/formats/web/mod.rs:18`
```rust
pub mod epub
```

#### `mod html`
- Ort: `crates/contextra-ingest/src/formats/web/mod.rs:19`
```rust
pub mod html
```

#### `struct WebExtractor`
- Ort: `crates/contextra-ingest/src/formats/web/mod.rs:27`
```rust
pub struct WebExtractor
```
- Impl-Block `crates/contextra-ingest/src/formats/web/mod.rs:29`
- Impl-Block (Trait `DocumentExtractor`) `crates/contextra-ingest/src/formats/web/mod.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-ingest/src/formats/web/mod.rs:30` |  |

### Modul `formats::web::encoding`

#### `struct DecodedHtml`
- Ort: `crates/contextra-ingest/src/formats/web/encoding.rs:22`
```rust
pub struct DecodedHtml
```
Dokumentation: Ergebnis der Dekodierung eines HTML/XHTML-Byte-Streams.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text` | `String` | `crates/contextra-ingest/src/formats/web/encoding.rs:24` |
| `is_guessed` | `bool` | `crates/contextra-ingest/src/formats/web/encoding.rs:26` |

#### `fn decode_html_bytes`
- Ort: `crates/contextra-ingest/src/formats/web/encoding.rs:30`
```rust
pub fn decode_html_bytes(input: &[u8]) -> DecodedHtml
```
Dokumentation: Dekodiert Eingabe-Bytes unter Berücksichtigung von BOM, `<meta charset>` und UTF-8-Fallback.

### Modul `formats::web::epub`

#### `fn extract_epub`
- Ort: `crates/contextra-ingest/src/formats/web/epub.rs:29`
```rust
pub fn extract_epub(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert ein EPUB-Dokument aus ZIP-Bytes in ein `DocumentIr`.

### Modul `formats::web::html`

#### `struct DomArena`
- Ort: `crates/contextra-ingest/src/formats/web/html.rs:58`
```rust
pub struct DomArena
```
Dokumentation: Speicher-Arena für den HTML-DOM-Baum.
- Impl-Block `crates/contextra-ingest/src/formats/web/html.rs:66`
- Impl-Block (Trait `TreeSink`) `crates/contextra-ingest/src/formats/web/html.rs:102`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `document` | `Rc<Node>` | `crates/contextra-ingest/src/formats/web/html.rs:59` |
| `quirks_mode` | `RefCell<QuirksMode>` | `crates/contextra-ingest/src/formats/web/html.rs:60` |
| `max_depth_exceeded` | `RefCell<bool>` | `crates/contextra-ingest/src/formats/web/html.rs:61` |
| `max_depth_limit` | `u32` | `crates/contextra-ingest/src/formats/web/html.rs:62` |
| `observed_max_depth` | `RefCell<u32>` | `crates/contextra-ingest/src/formats/web/html.rs:63` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(max_depth_limit: u32) -> Self` | `crates/contextra-ingest/src/formats/web/html.rs:67` |  |

#### `struct Node`
- Ort: `crates/contextra-ingest/src/formats/web/html.rs:50`
```rust
pub struct Node
```
Dokumentation: Knoten im DOM-Baum.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `parent` | `RefCell<Option<Weak<Node>>>` | `crates/contextra-ingest/src/formats/web/html.rs:51` |
| `children` | `RefCell<Vec<Rc<Node>>>` | `crates/contextra-ingest/src/formats/web/html.rs:52` |
| `kind` | `NodeKind` | `crates/contextra-ingest/src/formats/web/html.rs:53` |

#### `enum NodeKind`
- Ort: `crates/contextra-ingest/src/formats/web/html.rs:37`
```rust
pub enum NodeKind
```
Dokumentation: Arten von Knoten im DOM-Baum.

Varianten:

- `Document` — `crates/contextra-ingest/src/formats/web/html.rs:38`
- `Element{` — `crates/contextra-ingest/src/formats/web/html.rs:39`
- `Text(RefCell<String>)` — `crates/contextra-ingest/src/formats/web/html.rs:43`
- `Comment(String)` — `crates/contextra-ingest/src/formats/web/html.rs:44`
- `Doctype` — `crates/contextra-ingest/src/formats/web/html.rs:45`

#### `fn destroy_tree_iteratively`
- Ort: `crates/contextra-ingest/src/formats/web/html.rs:270`
```rust
pub fn destroy_tree_iteratively(root: Rc<Node>)
```
Dokumentation: Baut den Baum iterativ ab, um rekursiven Stack-Overflow beim Drop zu verhindern.

#### `fn extract_html`
- Ort: `crates/contextra-ingest/src/formats/web/html.rs:282`
```rust
pub fn extract_html( html_text: &str, format: FormatId, ctx: &ExtractContext<'_>, is_guessed_encoding: bool, ) -> Result<DocumentIr, IngestError>
```
Dokumentation: Extrahiert HTML/XHTML-Inhalt defensiv in ein `DocumentIr`.

### Modul `fuzz_support`

#### `fn run`
- Ort: `crates/contextra-ingest/src/fuzz_support.rs:24`
```rust
pub fn run(format: FormatId, data: &[u8])
```
Dokumentation: Fuzzing-Einstiegspunkt für alle Ingestion-Extraktoren. Muss für jede beliebige Eingabe panikfrei durchlaufen.

### Modul `ir`

#### `struct Block`
- Ort: `crates/contextra-ingest/src/ir.rs:47`
```rust
pub struct Block
```
Dokumentation: Ein struktureller Inhaltsblock.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kind` | `BlockKind` | `crates/contextra-ingest/src/ir.rs:48` |
| `locator` | `Locator` | `crates/contextra-ingest/src/ir.rs:49` |

#### `struct DocMeta`
- Ort: `crates/contextra-ingest/src/ir.rs:34`
```rust
pub struct DocMeta
```
Dokumentation: Metadaten des Dokuments aus der Quelle (ISO-8601 Strings, niemals Systemzeit).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `title` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:35` |
| `language` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:36` |
| `authors` | `Vec<String>` | `crates/contextra-ingest/src/ir.rs:37` |
| `created` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:38` |
| `modified` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:39` |
| `page_count` | `Option<u32>` | `crates/contextra-ingest/src/ir.rs:40` |

#### `struct DocumentIr`
- Ort: `crates/contextra-ingest/src/ir.rs:24`
```rust
pub struct DocumentIr
```
Dokumentation: Zwischenrepräsentation eines extrahierten Dokuments.
- Impl-Block `crates/contextra-ingest/src/canonical.rs:79`
- Impl-Block `crates/contextra-ingest/src/ir.rs:175`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `format` | `FormatId` | `crates/contextra-ingest/src/ir.rs:25` |
| `meta` | `DocMeta` | `crates/contextra-ingest/src/ir.rs:26` |
| `blocks` | `Vec<Block>` | `crates/contextra-ingest/src/ir.rs:27` |
| `warnings` | `Vec<ExtractWarning>` | `crates/contextra-ingest/src/ir.rs:28` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn canonical(&self) -> CanonicalText` | `crates/contextra-ingest/src/canonical.rs:82` | Rendering der `DocumentIr` in ein `CanonicalText` Objekt. Blöcke werden durch "\n\n" getrennt (Trenner gehören zu keinem Block-Span). |
| `pub fn canonical_hash(&self) -> String` | `crates/contextra-ingest/src/canonical.rs:102` | Erzeugt den BLAKE3-Hex-Hash des kanonischen Textes. |
| `pub fn new(format: FormatId) -> Self` | `crates/contextra-ingest/src/ir.rs:177` | Erstellt eine neue leere `DocumentIr` für ein gegebenes Format. |
| `pub fn push(&mut self, kind: BlockKind, locator: Locator)` | `crates/contextra-ingest/src/ir.rs:187` | Fügt einen neuen Block mit gegebenem Locator hinzu. |
| `pub fn append_nested(&mut self, name: &str, mime: Option<&str>, mut nested: DocumentIr)` | `crates/contextra-ingest/src/ir.rs:192` | Bettet ein verschachteltes Dokument (z. B. Anhang) flach ein. |

#### `struct ExtractWarning`
- Ort: `crates/contextra-ingest/src/ir.rs:158`
```rust
pub struct ExtractWarning
```
Dokumentation: Warnungen, die während der Extraktion auftreten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `code` | `WarningCode` | `crates/contextra-ingest/src/ir.rs:159` |
| `detail` | `String` | `crates/contextra-ingest/src/ir.rs:160` |

#### `struct Locator`
- Ort: `crates/contextra-ingest/src/ir.rs:143`
```rust
pub struct Locator
```
Dokumentation: Lokationsangabe eines Blocks im Quelldokument.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `page` | `Option<u32>` | `crates/contextra-ingest/src/ir.rs:144` |
| `page_end` | `Option<u32>` | `crates/contextra-ingest/src/ir.rs:145` |
| `slide` | `Option<u32>` | `crates/contextra-ingest/src/ir.rs:146` |
| `slide_end` | `Option<u32>` | `crates/contextra-ingest/src/ir.rs:147` |
| `sheet` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:148` |
| `range` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:149` |
| `message` | `Option<u32>` | `crates/contextra-ingest/src/ir.rs:150` |
| `path` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:151` |
| `attachment` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:152` |

#### `struct MessageHeader`
- Ort: `crates/contextra-ingest/src/ir.rs:130`
```rust
pub struct MessageHeader
```
Dokumentation: Header-Daten von E-Mail-Nachrichten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `subject` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:131` |
| `from` | `Vec<String>` | `crates/contextra-ingest/src/ir.rs:132` |
| `to` | `Vec<String>` | `crates/contextra-ingest/src/ir.rs:133` |
| `cc` | `Vec<String>` | `crates/contextra-ingest/src/ir.rs:134` |
| `date` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:135` |
| `message_id` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:136` |
| `in_reply_to` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:137` |

#### `struct Table`
- Ort: `crates/contextra-ingest/src/ir.rs:97`
```rust
pub struct Table
```
Dokumentation: Tabellenstruktur in der Zwischenrepräsentation.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `caption` | `Option<String>` | `crates/contextra-ingest/src/ir.rs:98` |
| `header_rows` | `u8` | `crates/contextra-ingest/src/ir.rs:99` |
| `rows` | `Vec<Vec<String>>` | `crates/contextra-ingest/src/ir.rs:100` |

#### `enum BlockKind`
- Ort: `crates/contextra-ingest/src/ir.rs:55`
```rust
pub enum BlockKind
```
Dokumentation: Arten von Inhaltsblöcken in der Zwischenrepräsentation.

Varianten:

- `Heading{` — `crates/contextra-ingest/src/ir.rs:56`
- `Paragraph{` — `crates/contextra-ingest/src/ir.rs:60`
- `ListItem{` — `crates/contextra-ingest/src/ir.rs:63`
- `Table(Table)` — `crates/contextra-ingest/src/ir.rs:68`
- `Code{` — `crates/contextra-ingest/src/ir.rs:69`
- `Quote{` — `crates/contextra-ingest/src/ir.rs:73`
- `Caption{` — `crates/contextra-ingest/src/ir.rs:76`
- `Note{` — `crates/contextra-ingest/src/ir.rs:79`
- `Boundary(Boundary)` — `crates/contextra-ingest/src/ir.rs:83`
- `Record{` — `crates/contextra-ingest/src/ir.rs:84`
- `Message(MessageHeader)` — `crates/contextra-ingest/src/ir.rs:88`
- `Raw{` — `crates/contextra-ingest/src/ir.rs:89`

#### `enum Boundary`
- Ort: `crates/contextra-ingest/src/ir.rs:119`
```rust
pub enum Boundary
```
Dokumentation: Strukturelle Grenzen innerhalb von Dokumenten.

Varianten:

- `Page{ number: u32 }` — `crates/contextra-ingest/src/ir.rs:120`
- `Slide{ number: u32, title: Option<String> }` — `crates/contextra-ingest/src/ir.rs:121`
- `Sheet{ name: String, index: u32 }` — `crates/contextra-ingest/src/ir.rs:122`
- `Section{ title: Option<String> }` — `crates/contextra-ingest/src/ir.rs:123`
- `Attachment{ name: String, mime: Option<String> }` — `crates/contextra-ingest/src/ir.rs:124`

#### `enum NoteKind`
- Ort: `crates/contextra-ingest/src/ir.rs:107`
```rust
pub enum NoteKind
```
Dokumentation: Arten von Anmerkungen/Notizen.

Varianten:

- `Footnote` — `crates/contextra-ingest/src/ir.rs:108`
- `Endnote` — `crates/contextra-ingest/src/ir.rs:109`
- `Comment` — `crates/contextra-ingest/src/ir.rs:110`
- `SpeakerNotes` — `crates/contextra-ingest/src/ir.rs:111`
- `Header` — `crates/contextra-ingest/src/ir.rs:112`
- `Footer` — `crates/contextra-ingest/src/ir.rs:113`

#### `enum WarningCode`
- Ort: `crates/contextra-ingest/src/ir.rs:166`
```rust
pub enum WarningCode
```
Dokumentation: Klassifikation von Extraktionswarnungen.

Varianten:

- `HiddenContentSkipped` — `crates/contextra-ingest/src/ir.rs:167`
- `TextSanitized` — `crates/contextra-ingest/src/ir.rs:168`
- `EncodingGuessed` — `crates/contextra-ingest/src/ir.rs:169`
- `AttachmentSkipped` — `crates/contextra-ingest/src/ir.rs:170`
- `EmbeddedObjectSkipped` — `crates/contextra-ingest/src/ir.rs:171`
- `FeatureLimited` — `crates/contextra-ingest/src/ir.rs:172`

### Modul `limits`

#### `struct ExtractPolicy`
- Ort: `crates/contextra-ingest/src/limits.rs:71`
```rust
pub struct ExtractPolicy
```
Dokumentation: Extraktions-Richtlinien für sensible/versteckte Inhalte und Metadaten.
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/limits.rs:84`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `include_hidden` | `bool` | `crates/contextra-ingest/src/limits.rs:73` |
| `include_comments` | `bool` | `crates/contextra-ingest/src/limits.rs:75` |
| `include_tracked_deletions` | `bool` | `crates/contextra-ingest/src/limits.rs:77` |
| `include_headers_footers` | `bool` | `crates/contextra-ingest/src/limits.rs:79` |
| `keep_personal_metadata` | `bool` | `crates/contextra-ingest/src/limits.rs:81` |

#### `struct IngestLimits`
- Ort: `crates/contextra-ingest/src/limits.rs:22`
```rust
pub struct IngestLimits
```
Dokumentation: Konfigurierbare Obergrenzen für Ingestion-Vorgänge (Zip-Bomb-Schutz, Speicher- und Komplexitätsgrenzen).
- Impl-Block (Trait `Default`) `crates/contextra-ingest/src/limits.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_document_bytes` | `u64` | `crates/contextra-ingest/src/limits.rs:24` |
| `max_chunks_per_document` | `u32` | `crates/contextra-ingest/src/limits.rs:26` |
| `max_decompressed_bytes` | `u64` | `crates/contextra-ingest/src/limits.rs:28` |
| `max_compression_ratio` | `u32` | `crates/contextra-ingest/src/limits.rs:30` |
| `max_archive_entries` | `u32` | `crates/contextra-ingest/src/limits.rs:32` |
| `max_nesting_depth` | `u8` | `crates/contextra-ingest/src/limits.rs:34` |
| `max_xml_depth` | `u32` | `crates/contextra-ingest/src/limits.rs:36` |
| `max_xml_events` | `u64` | `crates/contextra-ingest/src/limits.rs:38` |
| `max_text_bytes` | `u64` | `crates/contextra-ingest/src/limits.rs:40` |
| `max_pages` | `u32` | `crates/contextra-ingest/src/limits.rs:42` |
| `max_table_cells` | `u64` | `crates/contextra-ingest/src/limits.rs:44` |
| `max_path_len` | `usize` | `crates/contextra-ingest/src/limits.rs:46` |

### Modul `pipeline`

#### `struct IngestConfig`
- Ort: `crates/contextra-ingest/src/pipeline.rs:39`
```rust
pub struct IngestConfig
```
Dokumentation: Ingestion-Konfiguration.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `limits` | `IngestLimits` | `crates/contextra-ingest/src/pipeline.rs:40` |
| `policy` | `ExtractPolicy` | `crates/contextra-ingest/src/pipeline.rs:41` |
| `chunking` | `ChunkingConfig` | `crates/contextra-ingest/src/pipeline.rs:42` |

#### `struct IngestOutput`
- Ort: `crates/contextra-ingest/src/pipeline.rs:47`
```rust
pub struct IngestOutput
```
Dokumentation: Ergebnis der Ingestion-Pipeline.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `format` | `FormatId` | `crates/contextra-ingest/src/pipeline.rs:48` |
| `meta` | `DocMeta` | `crates/contextra-ingest/src/pipeline.rs:49` |
| `chunks` | `Vec<ContextChunk>` | `crates/contextra-ingest/src/pipeline.rs:50` |
| `warnings` | `Vec<ExtractWarning>` | `crates/contextra-ingest/src/pipeline.rs:51` |
| `canonical_hash` | `String` | `crates/contextra-ingest/src/pipeline.rs:52` |

#### `struct IngestRequest`
- Ort: `crates/contextra-ingest/src/pipeline.rs:29`
```rust
pub struct IngestRequest<'a>
```
Dokumentation: Ingestion-Anforderung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `bytes` | `&'a [u8]` | `crates/contextra-ingest/src/pipeline.rs:30` |
| `filename` | `Option<&'a str>` | `crates/contextra-ingest/src/pipeline.rs:31` |
| `mime` | `Option<&'a str>` | `crates/contextra-ingest/src/pipeline.rs:32` |
| `format_override` | `Option<FormatId>` | `crates/contextra-ingest/src/pipeline.rs:33` |
| `doc_id` | `DocId` | `crates/contextra-ingest/src/pipeline.rs:34` |

#### `fn ingest`
- Ort: `crates/contextra-ingest/src/pipeline.rs:56`
```rust
pub fn ingest( req: &IngestRequest<'_>, cfg: &IngestConfig, registry: &ExtractorRegistry, ) -> Result<IngestOutput, IngestError>
```
Dokumentation: Führt die vollständige Ingestion-Pipeline für ein Dokument durch.

### Modul `registry`

#### `struct ExtractorRegistry`
- Ort: `crates/contextra-ingest/src/registry.rs:27`
```rust
pub struct ExtractorRegistry
```
Dokumentation: Registrierungstabelle für Format-Extraktoren.
- Impl-Block `crates/contextra-ingest/src/registry.rs:31`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_empty() -> Self` | `crates/contextra-ingest/src/registry.rs:33` | Erzeugt eine leere ExtractorRegistry. |
| `pub fn register(&mut self, extractor: Box<dyn DocumentExtractor>)` | `crates/contextra-ingest/src/registry.rs:40` | Registriert einen Extraktor für alle von ihm unterstützten Formate. |
| `pub fn with_defaults() -> Self` | `crates/contextra-ingest/src/registry.rs:48` | Registriert Standard-Extraktoren gemäß den aktivierten Feature-Flags (`fmt-*`). |
| `pub fn extract( &self, format: FormatId, bytes: &[u8], ctx: &ExtractContext<'_>, ) -> Result<DocumentIr, IngestError>` | `crates/contextra-ingest/src/registry.rs:88` | Fuehrt die Extraktion fuer ein Format durch und fängt Panics des Parsers fail-closed ab. |

### Modul `safe`

#### `mod archive`
- Ort: `crates/contextra-ingest/src/safe/mod.rs:18`
```rust
pub mod archive
```
- Gate: `#[cfg(feature = "zip-container")]`

#### `mod opc`
- Ort: `crates/contextra-ingest/src/safe/mod.rs:24`
```rust
pub mod opc
```

#### `mod xml`
- Ort: `crates/contextra-ingest/src/safe/mod.rs:21`
```rust
pub mod xml
```
- Gate: `#[cfg(feature = "xml")]`

### Modul `safe::archive`

#### `struct SafeArchive`
- Ort: `crates/contextra-ingest/src/safe/archive.rs:107`
```rust
pub struct SafeArchive<'a>
```
Dokumentation: Safe Zip archive reader enforcing resource limits (z. B. für OPC / DOCX / PPTX).
- Impl-Block `crates/contextra-ingest/src/safe/archive.rs:111`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bytes: &'a [u8], _budget: &ResourceBudget) -> Result<Self, IngestError>` | `crates/contextra-ingest/src/safe/archive.rs:113` | Opens a ZIP archive from raw input bytes and checks entry count limits. |
| `pub fn read_entry( &mut self, name: &str, budget: &ResourceBudget, ) -> Result<Vec<u8>, IngestError>` | `crates/contextra-ingest/src/safe/archive.rs:133` | Reads raw bytes for a file entry matching `name` (case-insensitive / path normalized). |
| `pub fn has_entry(&mut self, name: &str) -> bool` | `crates/contextra-ingest/src/safe/archive.rs:203` | Checks if an entry exists in the archive. |
| `pub fn list_entries(&mut self) -> Vec<String>` | `crates/contextra-ingest/src/safe/archive.rs:221` | Lists all file paths present in the archive. |

#### `struct SafeZipReader`
- Ort: `crates/contextra-ingest/src/safe/archive.rs:24`
```rust
pub struct SafeZipReader<'a>
```
Dokumentation: Sichere ZIP-Archiv-Hülle zur lesenden Extraktion von ZIP-Einträgen.
- Impl-Block `crates/contextra-ingest/src/safe/archive.rs:28`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bytes: &'a [u8]) -> Result<Self, IngestError>` | `crates/contextra-ingest/src/safe/archive.rs:30` | Erstellt einen neuen `SafeZipReader` aus Eingabe-Bytes. |
| `pub fn read_file( &mut self, name: &str, budget: &ResourceBudget, ) -> Result<Vec<u8>, IngestError>` | `crates/contextra-ingest/src/safe/archive.rs:41` | Liest den gesamten Inhalt der angegebenen Datei im Archiv und verbucht entkomprimierte Bytes. |
| `pub fn contains_file(&mut self, name: &str) -> bool` | `crates/contextra-ingest/src/safe/archive.rs:89` | Prüft, ob eine Datei unter dem Namen im Archiv existiert. |
| `pub fn file_names(&mut self) -> Vec<String>` | `crates/contextra-ingest/src/safe/archive.rs:94` | Gibt eine Liste aller Dateipfade im Archiv zurück. |

#### `fn validate_archive_entry_path`
- Ort: `crates/contextra-ingest/src/safe/archive.rs:233`
```rust
pub fn validate_archive_entry_path(path: &str, max_len: usize) -> Result<(), IngestError>
```
Dokumentation: Validiert einen Pfadnamen im Archiv gegen Zip-Slip und Pfadlängenbeschränkungen.

#### `fn validate_zip_archive`
- Ort: `crates/contextra-ingest/src/safe/archive.rs:272`
```rust
pub fn validate_zip_archive<R: Read + Seek>( reader: R, limits: &IngestLimits, ) -> Result<(), IngestError>
```
Dokumentation: Validiert ein Zip-Archiv vor der Verarbeitung gegen `IngestLimits`. Prüft Eintragsanzahl, kumulierte unkomprimierte Bytes, Kompressionsverhältnis, Zip-Slip-Pfade und Pfadlängen, um Zip-Bomben und DoS-Angriffe frühzeitig abzulehnen.

### Modul `safe::opc`

#### `struct OpcPackage`
- Ort: `crates/contextra-ingest/src/safe/opc.rs:98`
```rust
pub struct OpcPackage<'a>
```
Dokumentation: Helper for OPC package navigation.
- Impl-Block `crates/contextra-ingest/src/safe/opc.rs:102`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(bytes: &'a [u8], budget: &ResourceBudget) -> Result<Self, IngestError>` | `crates/contextra-ingest/src/safe/opc.rs:104` | Opens an OPC package from bytes. |
| `pub fn has_part(&mut self, path: &str) -> bool` | `crates/contextra-ingest/src/safe/opc.rs:110` | Checks if a part exists in the package. |
| `pub fn read_part( &mut self, path: &str, budget: &ResourceBudget, ) -> Result<Vec<u8>, IngestError>` | `crates/contextra-ingest/src/safe/opc.rs:115` | Reads raw bytes for a part path. |
| `pub fn parse_relationships( &mut self, rels_path: &str, max_xml_depth: u32, budget: &ResourceBudget, ) -> Result<Vec<OpcRelationship>, IngestError>` | `crates/contextra-ingest/src/safe/opc.rs:124` | Parses relationships XML at `rels_path` (e.g., `_rels/.rels` or `word/_rels/document.xml.rels`). |
| `pub fn main_document_part( &mut self, max_xml_depth: u32, budget: &ResourceBudget, ) -> Result<String, IngestError>` | `crates/contextra-ingest/src/safe/opc.rs:180` | Finds main document part path from `_rels/.rels`. Defaults to `word/document.xml`. |
| `pub fn resolve_target(base_part_path: &str, target: &str) -> String` | `crates/contextra-ingest/src/safe/opc.rs:204` | Resolves target path for a relationship relative to a base part directory. |

#### `struct OpcRelationship`
- Ort: `crates/contextra-ingest/src/safe/opc.rs:26`
```rust
pub struct OpcRelationship
```
Dokumentation: Eine einzelne OPC-Beziehung (<Relationship Id="..." Type="..." Target="..." />).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-ingest/src/safe/opc.rs:27` |
| `rel_type` | `String` | `crates/contextra-ingest/src/safe/opc.rs:28` |
| `target` | `String` | `crates/contextra-ingest/src/safe/opc.rs:29` |
| `target_mode` | `Option<String>` | `crates/contextra-ingest/src/safe/opc.rs:30` |

#### `struct OpcRels`
- Ort: `crates/contextra-ingest/src/safe/opc.rs:35`
```rust
pub struct OpcRels
```
Dokumentation: Sammlung aller Beziehungen einer .rels-Datei.
- Impl-Block `crates/contextra-ingest/src/safe/opc.rs:39`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse(xml_bytes: &[u8], budget: &ResourceBudget) -> Result<Self, IngestError>` | `crates/contextra-ingest/src/safe/opc.rs:41` | Parst .rels-XML-Bytes in eine `OpcRels`-Struktur. |
| `pub fn target_for_id(&self, id: &str) -> Option<&str>` | `crates/contextra-ingest/src/safe/opc.rs:65` | Sucht das Ziel einer Beziehung über deren ID. |
| `pub fn get_target(&self, id: &str) -> Option<&str>` | `crates/contextra-ingest/src/safe/opc.rs:70` | Alias für `target_for_id` zur Rückwärtskompatibilität. |
| `pub fn target_for_type(&self, rel_type: &str) -> Option<&str>` | `crates/contextra-ingest/src/safe/opc.rs:75` | Sucht das erste Ziel eines bestimmten Beziehungstyps. |
| `pub fn find_targets_by_type_suffix(&self, suffix: &str) -> Vec<&str>` | `crates/contextra-ingest/src/safe/opc.rs:85` | Findet alle Targets, deren Typ auf ein bestimmtes Suffix endet. |

#### `fn resolve_opc_target`
- Ort: `crates/contextra-ingest/src/safe/opc.rs:210`
```rust
pub fn resolve_opc_target(base_part_path: &str, target: &str) -> String
```
Dokumentation: Löst ein relatives OPC-Target bezüglich eines Basispfads deterministisch auf.

### Modul `safe::xml`

#### `struct SafeXmlReader`
- Ort: `crates/contextra-ingest/src/safe/xml.rs:25`
```rust
pub struct SafeXmlReader<'a>
```
Dokumentation: Safe XML reader enforcing XML depth limits, event budget, and XXE protection.
- Impl-Block `crates/contextra-ingest/src/safe/xml.rs:31`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bytes: &'a [u8], max_depth: u32) -> Self` | `crates/contextra-ingest/src/safe/xml.rs:33` | Creates a new `SafeXmlReader` from XML byte slice. |
| `pub fn next_event(&mut self, budget: &ResourceBudget) -> Result<Event<'static>, IngestError>` | `crates/contextra-ingest/src/safe/xml.rs:47` | Advances to the next XML event while charging the resource budget and enforcing depth limits. |

#### `struct XmlElement`
- Ort: `crates/contextra-ingest/src/safe/xml.rs:109`
```rust
pub struct XmlElement
```
Dokumentation: Ein XML-Element mit Name, Attributen und Kind-Knoten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-ingest/src/safe/xml.rs:110` |
| `children` | `Vec<XmlNode>` | `crates/contextra-ingest/src/safe/xml.rs:112` |

#### `enum XmlNode`
- Ort: `crates/contextra-ingest/src/safe/xml.rs:102`
```rust
pub enum XmlNode
```
Dokumentation: Ein Knoten in einem geparsten XML-Baum.

Varianten:

- `Element(XmlElement)` — `crates/contextra-ingest/src/safe/xml.rs:103`
- `Text(String)` — `crates/contextra-ingest/src/safe/xml.rs:104`

#### `fn parse_xml`
- Ort: `crates/contextra-ingest/src/safe/xml.rs:195`
```rust
pub fn parse_xml(bytes: &[u8], budget: &ResourceBudget) -> Result<XmlElement, IngestError>
```
Dokumentation: Liest ein XML-Dokument aus Bytes sicher in ein Wurzel-`XmlElement` ein.

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl TryFrom<contextra_types::DocId> for LocalDocId` — `crates/contextra-ingest/src/compat/types.rs:16` (0 pub Methoden)
- `impl From<contextra_types::ContextChunk> for LocalContextChunk` — `crates/contextra-ingest/src/compat/types.rs:62` (0 pub Methoden)
